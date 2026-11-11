//! 历法域第 4 层（仲裁/路由层）：按时代/政权路由该用哪部历法。
//!
//! 历法指一套推步或颁行的历制，与政权、年号无关；时间锚定在儒略日或儒略历/格里高利历。
//! 数据来源：`data/calendar/汇总/lifa_summary.csv`（历法总表）。第 3 层政权·帝王·年号见 [crate::calendar::era_names]，
//! 某历的具体历日（第 2 层）：实物历日 [crate::calendar::artifacts]、按术推步 [crate::calendar::push_calendar]、
//! 今人历表 [crate::calendar::historical]、现算 [crate::calendar::chinese_lunar]；中原一线取哪一家见 [resolve_year]。

use std::collections::BTreeMap;

use super::chinese_lunar::{compute_year_data, ChineseLunarYearData};
use super::dunhuang::{dunhuang_calendar_name, DunhuangCalendar, ShuoBasis};
use super::historical::{year_data_from_months, MonthStart, ShuoriTable, ZHANG_TABLE_NAME};
use crate::astronomy::ephemeris::{Elpmpp02Data, Vsop87};
use crate::quantity::angle::PlaneAngle;

/// 单条历法记录：历名、起止年（CE）、制作者、锚定说明、来源。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LifaEntry {
    /// 唯一标识，如 gregorian, shoushi
    pub id: String,
    /// 历名（简体），如 格里高利历、授时历
    pub name_simp: String,
    /// 起年（公历），负为公元前；空表示不详
    pub start_ce: Option<i32>,
    /// 止年（公历）；空表示沿用或至今
    pub end_ce: Option<i32>,
    /// 制历/颁行政权
    pub maker_regime: String,
    /// 制历者或说明（如 郭守敬、GB/T 33661）
    pub maker_person: String,
    /// 锚定说明（儒略日/儒略历+格里高利历等）
    pub anchor_note: String,
    /// 来源（如 lifa_tongzhi、公历、国标）
    pub source: String,
}

impl LifaEntry {
    /// 该历法在给定公历年份是否在施行区间内（含端点；无起止视为覆盖）。
    pub fn active_in_ce_year(&self, ce_year: i32) -> bool {
        if let Some(s) = self.start_ce {
            if ce_year < s {
                return false;
            }
        }
        if let Some(e) = self.end_ce {
            if ce_year > e {
                return false;
            }
        }
        true
    }
}

/// 历法注册表：按 id 索引，支持按公历年份查「在该年施行的历法」。
#[derive(Clone, Debug, Default)]
pub struct LifaRegistry {
    by_id: BTreeMap<String, LifaEntry>,
    #[allow(dead_code)] // 保留供日后按起年排序查询
    ids_by_start: Vec<(i32, String)>,
}

impl LifaRegistry {
    /// 从 CSV 文本解析历法总表。
    ///
    /// 格式：`id,name_simp,start_ce,end_ce,maker_regime,maker_person,anchor_note,source`。
    /// start_ce/end_ce 为空时解析为 None。
    pub fn from_csv(csv: &str) -> Result<Self, String> {
        let mut by_id = BTreeMap::new();
        let mut ids_by_start: Vec<(i32, String)> = Vec::new();
        for (line_no, line) in csv.lines().enumerate() {
            let line_no = line_no + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with("id,") {
                continue;
            }
            let entry = parse_lifa_row(line)
                .ok_or_else(|| format!("lifa line {}: invalid row", line_no))?;
            let start = entry.start_ce.unwrap_or(i32::MIN);
            ids_by_start.push((start, entry.id.clone()));
            by_id.insert(entry.id.clone(), entry);
        }
        ids_by_start.sort_by_key(|&(s, _)| s);
        Ok(LifaRegistry {
            by_id,
            ids_by_start,
        })
    }

    /// 按 id 获取历法。
    pub fn get(&self, id: &str) -> Option<&LifaEntry> {
        self.by_id.get(id)
    }

    /// 在给定公历年份处于施行区间的历法（含无起止的条目）。
    pub fn active_at_ce_year(&self, ce_year: i32) -> Vec<&LifaEntry> {
        self.by_id
            .values()
            .filter(|e| e.active_in_ce_year(ce_year))
            .collect()
    }

    /// 所有历法 id 的迭代器。
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.by_id.keys()
    }
}

/// 历日出自哪一级来源（资料可靠性次序见 `doc/calendar/calendar-compendium.md`「资料可靠性」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiliSource {
    /// 实物历日：当时颁行的历书原件（第 1 级），如《宝祐四年会天历》。
    Artifact(&'static str),
    /// 按术推步：本项目按该历术文推算（第 4 级），值为历名。
    Reckoned(&'static str),
    /// 今人历表（第 5 级），值为表名（[super::historical::ZHANG_TABLE_NAME]）。
    ModernTable(&'static str),
    /// 现代定气定朔现算：历表与推步都不覆盖的年份（太初以前、2200 年以后），或 2017 年起行用的现行农历（GB/T 33661）本身。
    Computed,
}

impl RiliSource {
    /// 供显示的来源说明，如「按术推步（景初历）」。
    pub fn label(&self) -> String {
        match self {
            RiliSource::Artifact(n) => format!("实物历日（{n}）"),
            RiliSource::Reckoned(n) => format!("按术推步（{n}）"),
            RiliSource::ModernTable(n) => format!("今人历表（{n}）"),
            RiliSource::Computed => "现代定气定朔".to_string(),
        }
    }
}

/// 中原一线推步与今人历表都覆盖的年份取哪一家。实物历日不受此选项影响，总是优先。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ZhongyuanPreference {
    /// 逐历（缺省）：以正史本纪明书朔日检验，推步相合不低于张表之历取推步，低于张表之历（[RECKONING_BELOW_TABLE]）
    /// 取张表、推步作对照。依据 `data/calendar/zhongyuan_reckoning_eval.csv`（`examples/zhongyuan_reckoning_eval.rs`）。
    #[default]
    Evidence,
    /// 一律推步优先：按资料可靠性次序，推步（第 4 级）先于今人历表（第 5 级），不论本纪检验。
    Reckoning,
    /// 今人历表优先：与张培瑜历表逐月一致，推步只作对照。
    ModernTable,
}

/// 本纪明书朔日检验中推步相合少于张表之历（2026-11-10 检验：没有一条只合推步而不合张表；这些历各有若干条只合张表）。
/// 逐历取法下这些历缺省取张表。测试 `reckoning_below_table_matches_eval` 与检验结果核对。
pub const RECKONING_BELOW_TABLE: &[&str] =
    &["戊寅元历", "麟德历", "大衍历", "五纪历", "正元历", "宣明历", "崇玄历", "应天历", "乾元历", "仪天历", "崇天历", "观天历", "成天历", "授时历", "大统历"];

/// 现行农历的历名前缀（`zhongyuan_calendar_periods.csv`）：其规则即定气定朔，现算就是按术推步。
const MODERN_RULE_PREFIX: &str = "农历（GB/T 33661";

/// 第 4 层仲裁的结果：岁数据与其来源。
#[derive(Clone, Debug)]
pub struct ResolvedYear {
    pub data: ChineseLunarYearData,
    pub source: RiliSource,
}

/// 中原一线某农历年的各月及来源：实物历日 → 按术推步（按 [ZhongyuanCalendarPeriods] 所定行用之历，限已实现者）
/// → 今人历表；都没有则 None（由调用方现算）。`pref` 只决定推步与今人历表的先后。
pub fn zhongyuan_year_months(
    table: Option<&ShuoriTable>,
    lunar_year: i32,
    pref: ZhongyuanPreference,
) -> Option<(Vec<MonthStart>, RiliSource)> {
    if let Some(a) = super::artifacts::zhongyuan_artifact_year(lunar_year) {
        return Some((a.months.clone(), RiliSource::Artifact(a.name)));
    }
    let name: &'static str = ZhongyuanCalendarPeriods::embedded().name_for_lunar_year(lunar_year);
    // 现行农历（GB/T 33661—2017）的规则就是定气定朔，现算即按术推步：推步优先时不取今人历表，由调用方现算
    if pref != ZhongyuanPreference::ModernTable && name.starts_with(MODERN_RULE_PREFIX) {
        return None;
    }
    let reckon = || Some((super::push_calendar::reckon_lifa_year(name, lunar_year)?, RiliSource::Reckoned(name)));
    let from_table = || {
        table?.year_months(lunar_year).filter(|m| m.len() >= 12).map(|m| (m.to_vec(), RiliSource::ModernTable(ZHANG_TABLE_NAME)))
    };
    let reckon_first = match pref {
        ZhongyuanPreference::Evidence => !RECKONING_BELOW_TABLE.contains(&name),
        ZhongyuanPreference::Reckoning => true,
        ZhongyuanPreference::ModernTable => false,
    };
    if reckon_first {
        reckon().or_else(from_table)
    } else {
        from_table().or_else(reckon)
    }
}

/// 第 4 层仲裁：中原一线某农历年的岁数据及来源。
///
/// 这是"中原一线历日取哪一家"的唯一归属：依次取实物历日、按术推步、今人历表（次序见 [zhongyuan_year_months]），
/// 都不覆盖时现算定气定朔（第 2 层 [compute_year_data]）。本岁之前的十一、十二月取自上一岁的最优来源，
/// 两岁可以出自不同来源（如 236 年四分、237 年景初）。来源标在结果上，显示时须如实说明。
/// 调用方（如 [crate::calendar::convert]）只应调用本函数，不应各自重复这条判断。
pub fn resolve_year(
    table: Option<&ShuoriTable>,
    lunar_year: i32,
    pref: ZhongyuanPreference,
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Option<ResolvedYear> {
    if let Some((curr, source)) = zhongyuan_year_months(table, lunar_year, pref) {
        let prev = zhongyuan_year_months(table, lunar_year - 1, pref).map(|p| p.0).unwrap_or_default();
        return Some(ResolvedYear { data: year_data_from_months(lunar_year, &prev, &curr), source });
    }
    compute_year_data(vsop, elp, lunar_year, tolerance, max_iterations)
        .ok()
        .map(|data| ResolvedYear { data, source: RiliSource::Computed })
}

/// [resolve_year] 取缺省选项、只要岁数据。
pub fn resolve_year_data(
    table: Option<&ShuoriTable>,
    lunar_year: i32,
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Option<ChineseLunarYearData> {
    resolve_year(table, lunar_year, ZhongyuanPreference::default(), vsop, elp, tolerance, max_iterations).map(|r| r.data)
}

/// 中原一线 `[from_jdn, to_jdn]` 内各月：(朔日 JDN, 农历年, 月, 闰否)，按 [zhongyuan_year_months] 逐岁取最优来源。
pub fn zhongyuan_months_in_jdn_range(
    table: Option<&ShuoriTable>,
    from_jdn: i64,
    to_jdn: i64,
    pref: ZhongyuanPreference,
) -> Vec<(i64, i32, u8, bool)> {
    let (y0, _, _) = super::gregorian::civil_from_jdn(from_jdn);
    let (y1, _, _) = super::gregorian::civil_from_jdn(to_jdn);
    let mut out: Vec<(i64, i32, u8, bool)> = (y0 - 1..=y1 + 1)
        .filter_map(|ly| zhongyuan_year_months(table, ly, pref).map(|(m, _)| (ly, m)))
        .flat_map(|(ly, v)| v.into_iter().map(move |(j, m, leap)| (j, ly, m, leap)))
        .filter(|&(j, ..)| (from_jdn..=to_jdn).contains(&j))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// 敦煌历日（吐蕃、归义军时期）的历法 id，见 `lifa_summary.csv`。
pub const LIFA_ID_DUNHUANG_TUBO: &str = "dunhuang_tubo";
pub const LIFA_ID_DUNHUANG_GUIYIJUN: &str = "dunhuang_guiyijun";

/// 敦煌无据之月（出土历日未存之月）怎样补。
///
/// 缺省 [DunhuangFallback::Zhongyuan]：归义军原件 131 月与中原历同朔 56%、差一日以内 93%，与实际合朔（敦煌地方时）
/// 同日 49%、一日以内 91% 且偏晚；闰位 35 年中只 5 年与中原历不同，而现代定气置闰的闰位与两者都无关。故退回中原历，
/// 不退回定朔推算（依据见 `doc/calendar/dunhuang-calendar.md`「无据之月」）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DunhuangFallback {
    /// 无据之月用中原历（中原历）之月；与有据之月相接处，月首取有据之月的已知止日。结果可差一二日。
    #[default]
    Zhongyuan,
    /// 只认有据之月：无据之月、及止日不知之月的第 30 日，都不给结果。
    EvidenceOnly,
}

/// 敦煌某月的月首依据。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DunhuangMonthBasis {
    /// 出土历日原件。
    Original,
    /// 原书据残历条件推算。
    Inferred,
    /// 无据，退回中原历之月（月首或据前一有据之月的大小定）。
    Zhongyuan,
}

/// 敦煌历下的一月（拼合后）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DunhuangMonthSpan {
    pub start_jdn: i64,
    /// 止日（不含）；最后一月且无从定止日时为 None。
    pub end_jdn: Option<i64>,
    pub lunar_year: i32,
    pub month: u8,
    pub is_leap: bool,
    pub calendar: &'static str,
    pub basis: DunhuangMonthBasis,
}

/// 敦煌历下某日。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DunhuangDay {
    pub lunar_year: i32,
    pub month: u8,
    pub is_leap: bool,
    pub day: u8,
    pub calendar: &'static str,
    pub basis: DunhuangMonthBasis,
}

/// 第 4 层仲裁：`[from_jdn, to_jdn]` 前后敦煌历的各月——**有据之月用敦煌朔日表**（第 2 层 [DunhuangCalendar]），
/// 无据之月按 `fallback` 取中原历（第 4 层 [zhongyuan_months_in_jdn_range]）或不取。只做逐月取舍，不推算朔日：
/// - 中原历某月首与某有据之月首相距 3 日以内，视为同一月，取有据之月（月序、闰位从敦煌，如 893 年敦煌闰六月当中原历六月）；
/// - 无据之月紧接有据之月之后者，月首取前月已知止日（据月日数），前月止日不知则限在前月朔后 29、30 日之间；
/// - 无据之月限敦煌政权年限内（[dunhuang_calendar_name]），月序、闰否照中原历；与有据之月相差的一二日在无据之月段内
///   调匀（推到最近的可容之月），不让任何一月成 28 或 31 日。
pub fn dunhuang_months(
    dh: &DunhuangCalendar,
    zhongyuan: Option<&ShuoriTable>,
    from_jdn: i64,
    to_jdn: i64,
    fallback: DunhuangFallback,
) -> Vec<DunhuangMonthSpan> {
    // 窗口放宽到前后约一年，使下面的月长调匀不受窗口边缘影响（调匀只波及相接的一二月）
    let (a, b) = (from_jdn - 400, to_jdn + 400);
    let ev = dh.months();
    let lo = ev.partition_point(|m| m.jdn < a);
    let hi = ev.partition_point(|m| m.jdn <= b);
    let calendar_of = |y: i32| dunhuang_calendar_name(y).unwrap_or("敦煌历");
    // (月首, 已知止日, 岁, 月, 闰, 依据)
    let mut seq: Vec<(i64, Option<i64>, i32, u8, bool, DunhuangMonthBasis)> = (lo..hi)
        .map(|i| {
            let m = &ev[i];
            let basis = match m.shuo_basis {
                ShuoBasis::Original => DunhuangMonthBasis::Original,
                ShuoBasis::Inferred => DunhuangMonthBasis::Inferred,
            };
            (m.jdn, dh.known_end(i), m.lunar_year, m.month, m.is_leap, basis)
        })
        .collect();
    if fallback == DunhuangFallback::Zhongyuan && zhongyuan.is_some() {
        let evidenced: Vec<(i64, Option<i64>)> = seq.iter().map(|s| (s.0, s.1)).collect();
        for (s_main, ly, m, leap) in zhongyuan_months_in_jdn_range(zhongyuan, a, b, ZhongyuanPreference::default()) {
            if dunhuang_calendar_name(ly).is_none() || evidenced.iter().any(|&(s, _)| (s - s_main).abs() <= 3) {
                continue;
            }
            // 紧接其前的有据之月：月首据其止日
            let start = match evidenced.iter().rev().find(|&&(s, _)| s < s_main) {
                Some(&(s, Some(e))) if (e - s_main).abs() <= 3 => e,
                Some(&(s, None)) if s_main - s <= 29 + 3 => s_main.clamp(s + 29, s + 30),
                _ => s_main,
            };
            seq.push((start, None, ly, m, leap, DunhuangMonthBasis::Zhongyuan));
        }
    }
    seq.sort_by_key(|s| s.0);
    // 无据之月一段（中原历月）夹在有据之月间：两端月首已由有据之月定，与中原历月首差的一二日若都压在相接之月，
    // 会出现 31 或 28 日之月。把差额沿中原历月往段内推，推到能容纳（小月变大、大月变小）为止：自前端向后推一遍，
    // 自后端向前推一遍；段首月首（接前一有据之月）与段后有据之月首不动。只挪月首，不改月序。
    let is_zy = |x: &(i64, Option<i64>, i32, u8, bool, DunhuangMonthBasis)| x.5 == DunhuangMonthBasis::Zhongyuan;
    let mut p = 0;
    while p < seq.len() {
        if !is_zy(&seq[p]) {
            p += 1;
            continue;
        }
        let mut q = p;
        while q + 1 < seq.len() && is_zy(&seq[q + 1]) {
            q += 1;
        }
        let anchored_left = p > 0;
        for k in p..q {
            let len = seq[k + 1].0 - seq[k].0;
            if anchored_left && len > 30 {
                seq[k + 1].0 -= len - 30;
            } else if anchored_left && len < 29 {
                seq[k + 1].0 += 29 - len;
            }
        }
        if q + 1 < seq.len() {
            for k in (p..=q).rev() {
                let len = seq[k + 1].0 - seq[k].0;
                if k > p && len > 30 {
                    seq[k].0 += len - 30;
                } else if k > p && len < 29 {
                    seq[k].0 -= 29 - len;
                }
            }
        }
        p = q + 1;
    }
    let n = seq.len();
    (0..n)
        .map(|i| {
            let (start, known_end, ly, m, leap, basis) = seq[i];
            let next = seq.get(i + 1).map(|s| s.0);
            let end = match (basis, fallback) {
                (DunhuangMonthBasis::Zhongyuan, _) => next,
                (_, DunhuangFallback::EvidenceOnly) => known_end,
                (_, DunhuangFallback::Zhongyuan) => known_end.or(next.filter(|&j| j - start <= 30)).or(known_end),
            };
            DunhuangMonthSpan { start_jdn: start, end_jdn: end, lunar_year: ly, month: m, is_leap: leap, calendar: calendar_of(ly), basis }
        })
        .filter(|s| s.end_jdn.is_none_or(|e| e > a) && s.start_jdn <= b)
        .collect()
}

/// 第 4 层仲裁：JDN 在敦煌历下的月日（见 [dunhuang_months] 的取舍规则）。止日不知之月只认前 29 日。
/// 不在敦煌政权年限内、或 [DunhuangFallback::EvidenceOnly] 下无据，返回 None。`zhongyuan` 为 None 时等同只认有据之月。
pub fn resolve_dunhuang_day(
    dh: &DunhuangCalendar,
    zhongyuan: Option<&ShuoriTable>,
    jdn: i64,
    fallback: DunhuangFallback,
) -> Option<DunhuangDay> {
    let months = dunhuang_months(dh, zhongyuan, jdn, jdn, fallback);
    let s = months.iter().rev().find(|s| s.start_jdn <= jdn)?;
    if jdn >= s.end_jdn.unwrap_or(s.start_jdn + 29) {
        return None;
    }
    Some(DunhuangDay {
        lunar_year: s.lunar_year,
        month: s.month,
        is_leap: s.is_leap,
        day: (jdn - s.start_jdn + 1) as u8,
        calendar: s.calendar,
        basis: s.basis,
    })
}

/// 第 4 层仲裁：敦煌历某月（敦煌自身的岁、月序、闰否）的月首 JDN 与依据；无此月（如中原历闰月在敦煌落在别月）为 None。
/// 敦煌文书纪日「某年某月某日」→ JDN = 月首 + 日 − 1（日不得超过该月日数，见 [DunhuangMonthSpan::end_jdn]）。
pub fn resolve_dunhuang_month(
    dh: &DunhuangCalendar,
    zhongyuan: Option<&ShuoriTable>,
    lunar_year: i32,
    month: u8,
    is_leap: bool,
    fallback: DunhuangFallback,
) -> Option<DunhuangMonthSpan> {
    // 先定一个大致的 JDN 窗口：有据之月直接用；否则用中原历同名之月
    let around = match dh.find_month(lunar_year, month, is_leap) {
        Some((_, m)) => m.jdn,
        None => {
            let (months, _) = zhongyuan_year_months(zhongyuan, lunar_year, ZhongyuanPreference::default())?;
            // 中原历无此闰月（敦煌闰位不同）：取中原历同岁同月前后找
            months.iter().find(|&&(_, m, l)| m == month && l == is_leap).or_else(|| months.iter().find(|&&(_, m, l)| m == month && !l))?.0
        }
    };
    dunhuang_months(dh, zhongyuan, around - 60, around + 60, fallback)
        .into_iter()
        .find(|s| s.lunar_year == lunar_year && s.month == month && s.is_leap == is_leap)
}

fn parse_lifa_row(line: &str) -> Option<LifaEntry> {
    let mut fields = split_csv_line(line);
    let id = unquote(fields.next()?.trim()).to_string();
    let name_simp = unquote(fields.next()?.trim()).to_string();
    let start_ce = fields.next()?.trim();
    let start_ce = if start_ce.is_empty() {
        None
    } else {
        Some(start_ce.parse().ok()?)
    };
    let end_ce = fields.next()?.trim();
    let end_ce = if end_ce.is_empty() {
        None
    } else {
        Some(end_ce.parse().ok()?)
    };
    let maker_regime = unquote(fields.next()?.trim()).to_string();
    let maker_person = unquote(fields.next()?.trim()).to_string();
    let anchor_note = unquote(fields.next()?.trim()).to_string();
    let source = unquote(fields.next()?.trim()).to_string();
    Some(LifaEntry {
        id,
        name_simp,
        start_ce,
        end_ce,
        maker_regime,
        maker_person,
        anchor_note,
        source,
    })
}

fn unquote(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn split_csv_line(line: &str) -> impl Iterator<Item = &str> {
    let mut in_quote = false;
    let mut start = 0;
    let mut v = Vec::new();
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'"' => in_quote = !in_quote,
            b',' if !in_quote => {
                v.push(&line[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    v.push(&line[start..]);
    v.into_iter()
}

/// 中原一线某农历年行用之历的名称表（`data/calendar/汇总/zhongyuan_calendar_periods.csv`）。
/// 只回答「当时行用何历」；历日由 [resolve_year] 按来源次序取。
#[derive(Debug, Clone, Default)]
pub struct ZhongyuanCalendarPeriods {
    rows: Vec<(i32, Option<i32>, String)>,
}

/// 太初以前（张培瑜历表起点之前）没有中原一线行用历的资料，日期是现代定气定朔算的，显示时须如实说明。
pub const CALENDAR_NAME_COMPUTED: &str = "定气定朔（现代推算）";

impl ZhongyuanCalendarPeriods {
    /// 内嵌的 `data/calendar/汇总/zhongyuan_calendar_periods.csv`。
    pub fn embedded() -> &'static Self {
        use std::sync::OnceLock;
        static P: OnceLock<ZhongyuanCalendarPeriods> = OnceLock::new();
        P.get_or_init(|| {
            Self::from_csv(include_str!("../../../../data/calendar/汇总/zhongyuan_calendar_periods.csv")).expect("zhongyuan_calendar_periods.csv")
        })
    }

    pub fn from_csv(csv: &str) -> Result<Self, String> {
        let mut rows = Vec::new();
        for (i, line) in csv.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("from_year") {
                continue;
            }
            let mut f = line.splitn(4, ',');
            let from = f.next().and_then(|s| s.parse::<i32>().ok());
            let to = f.next().map(str::trim);
            let name = f.next().map(str::trim).filter(|s| !s.is_empty());
            let (Some(from), Some(to), Some(name)) = (from, to, name) else {
                return Err(format!("zhongyuan_calendar_periods line {}: invalid row", i + 1));
            };
            let to = if to.is_empty() {
                None
            } else {
                Some(to.parse::<i32>().map_err(|_| format!("zhongyuan_calendar_periods line {}: bad to_year", i + 1))?)
            };
            rows.push((from, to, name.to_string()));
        }
        Ok(Self { rows })
    }

    /// 该农历年的历名；表起点之前返回 [CALENDAR_NAME_COMPUTED]。
    pub fn name_for_lunar_year(&self, lunar_year: i32) -> &str {
        self.rows
            .iter()
            .find(|(from, to, _)| *from <= lunar_year && to.is_none_or(|t| lunar_year <= t))
            .map_or(CALENDAR_NAME_COMPUTED, |(_, _, n)| n.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lifa_csv() {
        let csv = r#"id,name_simp,start_ce,end_ce,maker_regime,maker_person,anchor_note,source
gregorian,格里高利历,1582,,罗马/教廷,格列高利十三世,儒略历+格里高利历,公历
shoushi,授时历,1281,,元,郭守敬,儒略历+格里高利历,lifa_tongzhi
"#;
        let reg = LifaRegistry::from_csv(csv).unwrap();
        let g = reg.get("gregorian").unwrap();
        assert_eq!(g.name_simp, "格里高利历");
        assert_eq!(g.start_ce, Some(1582));
        assert_eq!(g.end_ce, None);
        assert!(g.active_in_ce_year(2000));
        assert!(!g.active_in_ce_year(1500));
        let active = reg.active_at_ce_year(1281);
        assert!(active.iter().any(|e| e.id == "shoushi"));
        assert!(!active.iter().any(|e| e.id == "gregorian"));
    }

    #[test]
    fn zhongyuan_calendar_periods_are_contiguous_and_named() {
        let p = ZhongyuanCalendarPeriods::embedded();
        for w in p.rows.windows(2) {
            assert_eq!(w[0].1.map(|t| t + 1), Some(w[1].0), "年段须首尾相接：{:?}", w[0]);
        }
        for (y, n) in [(700, "麟德历"), (1066, "明天历"), (1130, "历术失载"), (1700, "时宪历"), (1730, "时宪历（历象考成）"), (1800, "时宪历（历象考成后编）"), (1912, "中华民国历书（沿用历象考成后编）"), (1930, "国民历"), (2026, "农历（GB/T 33661—2017）")] {
            assert_eq!(p.name_for_lunar_year(y), n);
        }
        assert_eq!(p.name_for_lunar_year(-500), CALENDAR_NAME_COMPUTED);
    }

    fn zhang_table() -> Option<ShuoriTable> {
        let text = std::fs::read_to_string(crate::repo::repo_root().join("vendor/calendar/chinese_calendar/实历朔日表.csv")).ok()?;
        ShuoriTable::from_csv(&text).ok()
    }

    /// 中原一线来源次序：1256 实物、景初推步、张表、表外为 None（现算）；选项只调推步与张表的先后
    #[test]
    fn zhongyuan_sources_ranked() {
        let Some(z) = zhang_table() else { return };
        let src = |y: i32, p: ZhongyuanPreference| zhongyuan_year_months(Some(&z), y, p).map(|r| r.1);
        let r = ZhongyuanPreference::Evidence;
        let all = ZhongyuanPreference::Reckoning;
        assert_eq!(src(700, all), Some(RiliSource::Reckoned("麟德历")), "一律推步");
        assert_eq!(src(1110, r), Some(RiliSource::Reckoned("纪元历")), "纪元历本纪检验与张表持平，逐历取推步");
        assert_eq!(src(1300, r), Some(RiliSource::ModernTable(ZHANG_TABLE_NAME)), "授时历推步不及张表，逐历取张表");
        assert_eq!(src(1300, all), Some(RiliSource::Reckoned("授时历")));
        assert!(matches!(src(1256, r), Some(RiliSource::Artifact(_))));
        assert!(matches!(src(1256, ZhongyuanPreference::ModernTable), Some(RiliSource::Artifact(_))));
        assert_eq!(src(300, r), Some(RiliSource::Reckoned("景初历")));
        assert_eq!(src(300, ZhongyuanPreference::ModernTable), Some(RiliSource::ModernTable(ZHANG_TABLE_NAME)));
        assert_eq!(src(700, r), Some(RiliSource::ModernTable(ZHANG_TABLE_NAME)));
        assert_eq!(src(-104, r), Some(RiliSource::ModernTable(ZHANG_TABLE_NAME)), "元封六年，颛顼历未实现");
        assert_eq!(src(-103, r), Some(RiliSource::ModernTable(ZHANG_TABLE_NAME)), "太初元年五月改历，推步不覆盖");
        assert_eq!(src(-102, r), Some(RiliSource::Reckoned("太初历（三统历）")));
        assert_eq!(src(500, r), Some(RiliSource::Reckoned("元嘉历")));
        assert_eq!(src(550, r), Some(RiliSource::Reckoned("大明历（祖冲之）")));
        assert_eq!(src(600, r), Some(RiliSource::Reckoned("张胄玄历（开皇十七年术不传，以大业历代推）")));
        assert_eq!(src(610, r), Some(RiliSource::Reckoned("大业历")));
        assert_eq!(src(-300, r), None);
        assert_eq!(src(1915, r), Some(RiliSource::ModernTable(ZHANG_TABLE_NAME)), "民初历书编算法与今不同，取张表");
        assert_eq!(src(2026, r), None, "现行农历：推步优先即现算");
        assert_eq!(src(2026, ZhongyuanPreference::ModernTable), Some(RiliSource::ModernTable(ZHANG_TABLE_NAME)));
        assert_eq!(src(300, r), zhongyuan_year_months(None, 300, r).map(|r| r.1), "推步不依赖历表");
    }

    /// 逐历取法的名单与本纪检验结果一致：本纪合推步少于合张表者恰是 [RECKONING_BELOW_TABLE]
    #[test]
    fn reckoning_below_table_matches_eval() {
        let Ok(csv) = std::fs::read_to_string(crate::repo::repo_root().join("data/calendar/zhongyuan_reckoning_eval.csv")) else { return };
        let mut below: Vec<String> = Vec::new();
        for line in csv.lines().skip(1) {
            let c: Vec<&str> = line.split(',').collect();
            let (r, t): (usize, usize) = (c[7].parse().unwrap(), c[8].parse().unwrap());
            if r < t {
                below.push(c[0].to_string());
            }
        }
        below.sort();
        let mut want: Vec<String> = RECKONING_BELOW_TABLE.iter().map(|s| s.to_string()).collect();
        want.sort();
        assert_eq!(below, want);
    }

    /// 中原一线推步与张培瑜历表逐月比较，不同之月逐条钉住（三统 −102—84、四分 85—236、元嘉 445—509、大明 510—589、
    /// 开皇 590—596、大业 597—618 全同；景初 237—444 四月不同）。
    /// 这几月本纪无可判定的记载（如 314 年正月推步壬申、张表辛未，《晋书·愍帝纪》「正月己巳朔」两家皆不合），
    /// 依资料可靠性次序缺省取推步；张表的取舍多同汪曰桢《历代长术辑要》，未注依据。
    #[test]
    fn zhongyuan_reckoning_vs_zhang_table() {
        let Some(z) = zhang_table() else { return };
        let mut diff = Vec::new();
        for y in -102..=618 {
            let (r, _) = zhongyuan_year_months(None, y, ZhongyuanPreference::Reckoning).unwrap();
            let t = z.year_months(y).unwrap();
            assert_eq!(r.len(), t.len(), "{y} 年月数");
            for (a, b) in r.iter().zip(t) {
                assert_eq!((a.1, a.2), (b.1, b.2), "{y} 年月序与闰位");
                if a.0 != b.0 {
                    diff.push((y, a.1, a.2, a.0 - b.0));
                }
            }
        }
        assert_eq!(diff, vec![(278, 3, false, -1), (314, 1, false, 1), (314, 3, false, 1), (430, 2, false, -1)]);
    }

    /// 比较表（邓文宽 1996 附录一）敦煌曆一栏每一格：按敦煌岁月查月首得原书朔干支，月首当日换算回该月初一，
    /// 月大小已知者末日为该月第 29/30 日、次日为次月初一。两种取舍（退回中原历、只认有据之月）都须重现。
    #[test]
    fn dunhuang_comparison_table_reproduced() {
        use crate::calendar::dunhuang::{parse_comparison_table, DunhuangCalendar};
        use crate::calendar::gan_zhi::GAN_ZHI_60;
        let dh = DunhuangCalendar::embedded();
        let main = zhang_table();
        let rows = parse_comparison_table(include_str!("../../../../data/calendar/敦煌历日_比较表.csv")).unwrap();
        assert_eq!(rows.len(), 257, "比较表敦煌曆有朔之格（800 年以后，含两件重出之次年正月三格）");
        let gz = |j: i64| GAN_ZHI_60[(j + 49).rem_euclid(60) as usize];
        for fb in [DunhuangFallback::Zhongyuan, DunhuangFallback::EvidenceOnly] {
            for r in &rows {
                let what = format!("{fb:?} {} {}年{}{}月", r.document, r.lunar_year, if r.is_leap { "閏" } else { "" }, r.month);
                let s = resolve_dunhuang_month(&dh, main.as_ref(), r.lunar_year, r.month, r.is_leap, fb).unwrap_or_else(|| panic!("{what} 查无此月"));
                assert_eq!(gz(s.start_jdn), r.shuo_ganzhi, "{what} 朔");
                let d1 = resolve_dunhuang_day(&dh, main.as_ref(), s.start_jdn, fb).unwrap();
                assert_eq!((d1.lunar_year, d1.month, d1.is_leap, d1.day), (r.lunar_year, r.month, r.is_leap, 1), "{what} 初一");
                if let Some(n) = r.days {
                    let last = resolve_dunhuang_day(&dh, main.as_ref(), s.start_jdn + n as i64 - 1, fb).unwrap();
                    assert_eq!((last.month, last.is_leap, last.day), (r.month, r.is_leap, n), "{what} 末日");
                    if let Some(next) = resolve_dunhuang_day(&dh, main.as_ref(), s.start_jdn + n as i64, fb) {
                        assert_eq!(next.day, 1, "{what} 次日应为次月初一");
                    } else {
                        assert!(fb == DunhuangFallback::EvidenceOnly || main.is_none(), "{what} 次月");
                    }
                }
            }
        }
    }

    /// 退回中原历时，786—1036 年敦煌历逐月连续、不重不漏；月长异常（非 29、30 日）者只在有据与无据之月相接处，
    /// 逐条钉住。无据之年与中原历全同。
    #[test]
    fn dunhuang_merged_months_are_contiguous() {
        use crate::calendar::dunhuang::DunhuangCalendar;
        use crate::calendar::gregorian::civil_to_jdn;
        let Some(main) = zhang_table() else { return };
        let dh = DunhuangCalendar::embedded();
        let (a, b) = (civil_to_jdn(786, 3, 1), civil_to_jdn(1036, 12, 1));
        let ms = dunhuang_months(&dh, Some(&main), a, b, DunhuangFallback::Zhongyuan);
        let mut odd = vec![];
        for w in ms.windows(2) {
            assert_eq!(w[0].end_jdn, Some(w[1].start_jdn), "{:?} 与次月不接", w[0]);
            let len = w[1].start_jdn - w[0].start_jdn;
            if !(29..=30).contains(&len) {
                odd.push((w[0].lunar_year, w[0].month, w[0].is_leap, len));
            }
        }
        // 无据之年（如 900 年以前 850 年）与中原历同
        let s = resolve_dunhuang_month(&dh, Some(&main), 850, 5, false, DunhuangFallback::Zhongyuan).unwrap();
        let (y, m, d) = main.new_moon_date(850, 5, false).unwrap();
        assert_eq!(s.start_jdn, civil_to_jdn(y, m, d));
        assert_eq!(s.basis, DunhuangMonthBasis::Zhongyuan);
        // 中原历閏五月在敦煌为六月（893）：按敦煌月序查不到閏五月
        assert!(resolve_dunhuang_month(&dh, Some(&main), 893, 5, true, DunhuangFallback::Zhongyuan).is_none());
        assert_eq!(odd, Vec::<(i32, u8, bool, i64)>::new(), "月长异常之月");
    }

    #[test]
    fn dunhuang_year_names() {
        use crate::calendar::dunhuang::DunhuangCalendar;
        use crate::calendar::era_names::{dunhuang_year_name, EraNames};
        let era = EraNames::from_csv(include_str!("../../../../data/calendar/汇总/era_names_merged.csv")).unwrap();
        let dh = DunhuangCalendar::embedded();
        let t = dunhuang_year_name(808, &era, Some(&dh)).unwrap();
        assert_eq!((t.period, t.ganzhi, t.label.as_str()), ("吐蕃", "戊子", "子年"));
        let g = dunhuang_year_name(922, &era, Some(&dh)).unwrap();
        assert_eq!(g.period, "归义军");
        assert_eq!(g.documents, vec!["後梁貞明八年壬午歲".to_string()]);
        assert!(g.label.contains("龙德"), "中原 922 年为龙德二年：{}", g.label);
        assert!(dunhuang_year_name(1100, &era, Some(&dh)).is_none());
    }
}
