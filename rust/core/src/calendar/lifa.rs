//! 历法域第 4 层（仲裁/路由层）：按时代/政权路由该用哪部历法。
//!
//! 历法指一套推步或颁行的历制，与政权、年号无关；时间锚定在儒略日或儒略历/格里高利历。
//! 数据来源：`data/calendar/汇总/lifa_summary.csv`（历法总表）。第 3 层政权·帝王·年号见 [crate::calendar::era_names]，
//! 实历朔日（第 2 层，某历法的具体岁数据）见 [crate::calendar::historical]。

use std::collections::BTreeMap;

use super::chinese_lunar::{compute_year_data, ChineseLunarYearData};
use super::dunhuang::{dunhuang_calendar_name, DunhuangCalendar, ShuoBasis};
use super::historical::HistoricalCalendar;
use crate::astronomy::ephemeris::{Elpmpp02Data, Vsop87};
use crate::quantity::angle::PlaneAngle;

/// 单条历法记录：历名、起止年（CE）、制作者、锚定说明、来源。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LifaEntry {
    /// 唯一标识，如 gregorian, shoushi, zhongyuan_shili
    pub id: String,
    /// 历名（简体），如 格里高利历、授时历、主表（据历还原）
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

/// 主表所对应的历法 id：按各朝颁历据历还原推步得出的连续表（朔日表 + 年号表主表）；《三千五百年历日天象》无名为「中原正朔」的历法。
pub const LIFA_ID_ZHONGYUAN_SHILI: &str = "zhongyuan_shili";

/// 第 4 层仲裁：给定农历年，返回该年岁数据——**实历覆盖范围内查表**（第 2 层 [HistoricalCalendar]），
/// 否则**现算定气定朔**（第 2 层 [compute_year_data]）。
///
/// 这是"实历优先、缺省退回定气定朔"策略的唯一归属：
/// 历史区间当时行用的是各朝颁行的历法（平朔、经朔、定朔各有其法），不是现代定气定朔，所以优先用按各朝历法复原的实历表；该表是复原结果（《三千五百年历日天象》据历推步），不是史料原记录，与史书日干支不合之处须另行核验。
/// 实历表覆盖范围有限（见 [HistoricalCalendar::year_range]），范围外或未提供实历表时才现算。
/// 调用方（如 [crate::calendar::convert]）只应调用本函数决定"用哪份数据"，不应各自重复这条判断——
/// 该判断一旦分散到多处，后续调整覆盖范围或优先级时就要满仓库找，是本仓库已经踩过的坑（章动表/历表
/// 数据源的仲裁逻辑也曾有过同样的问题，见 `doc/architecture/three-domains.md`）。
pub fn resolve_year_data(
    historical: Option<&HistoricalCalendar>,
    lunar_year: i32,
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Option<ChineseLunarYearData> {
    if let Some(h) = historical {
        if let Some(year_data) = h.year_data(lunar_year) {
            return Some(year_data);
        }
    }
    compute_year_data(vsop, elp, lunar_year, tolerance, max_iterations).ok()
}

/// 敦煌历日（吐蕃、归义军时期）的历法 id，见 `lifa_summary.csv`。
pub const LIFA_ID_DUNHUANG_TUBO: &str = "dunhuang_tubo";
pub const LIFA_ID_DUNHUANG_GUIYIJUN: &str = "dunhuang_guiyijun";

/// 敦煌无据之月（出土历日未存之月）怎样补。
///
/// 缺省 [DunhuangFallback::MainTable]：归义军原件 131 月与主表同朔 56%、差一日以内 93%，与实际合朔（敦煌地方时）
/// 同日 49%、一日以内 91% 且偏晚；闰位 35 年中只 5 年与主表不同，而现代定气置闰的闰位与两者都无关。故退回主表，
/// 不退回定朔推算（依据见 `doc/calendar/dunhuang-calendar.md`「无据之月」）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DunhuangFallback {
    /// 无据之月用主表（中原历）之月；与有据之月相接处，月首取有据之月的已知止日。结果可差一二日。
    #[default]
    MainTable,
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
    /// 无据，退回主表之月（月首或据前一有据之月的大小定）。
    MainTable,
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
/// 无据之月按 `fallback` 取主表（第 2 层 [HistoricalCalendar]）或不取。只做逐月取舍，不推算朔日：
/// - 主表某月首与某有据之月首相距 3 日以内，视为同一月，取有据之月（月序、闰位从敦煌，如 893 年敦煌闰六月当主表六月）；
/// - 无据之月紧接有据之月之后者，月首取前月已知止日（据月日数），前月止日不知则限在前月朔后 29、30 日之间；
/// - 无据之月限敦煌政权年限内（[dunhuang_calendar_name]），月序、闰否照主表；与有据之月相差的一二日在无据之月段内
///   调匀（推到最近的可容之月），不让任何一月成 28 或 31 日。
pub fn dunhuang_months(
    dh: &DunhuangCalendar,
    main: Option<&HistoricalCalendar>,
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
    if let (DunhuangFallback::MainTable, Some(h)) = (fallback, main) {
        let evidenced: Vec<(i64, Option<i64>)> = seq.iter().map(|s| (s.0, s.1)).collect();
        for (s_main, ly, m, leap) in h.months_in_jdn_range(a, b) {
            if dunhuang_calendar_name(ly).is_none() || evidenced.iter().any(|&(s, _)| (s - s_main).abs() <= 3) {
                continue;
            }
            // 紧接其前的有据之月：月首据其止日
            let start = match evidenced.iter().rev().find(|&&(s, _)| s < s_main) {
                Some(&(s, Some(e))) if (e - s_main).abs() <= 3 => e,
                Some(&(s, None)) if s_main - s <= 29 + 3 => s_main.clamp(s + 29, s + 30),
                _ => s_main,
            };
            seq.push((start, None, ly, m, leap, DunhuangMonthBasis::MainTable));
        }
    }
    seq.sort_by_key(|s| s.0);
    // 无据之月一段（主表月）夹在有据之月间：两端月首已由有据之月定，与主表月首差的一二日若都压在相接之月，
    // 会出现 31 或 28 日之月。把差额沿主表月往段内推，推到能容纳（小月变大、大月变小）为止：自前端向后推一遍，
    // 自后端向前推一遍；段首月首（接前一有据之月）与段后有据之月首不动。只挪月首，不改月序。
    let is_main = |x: &(i64, Option<i64>, i32, u8, bool, DunhuangMonthBasis)| x.5 == DunhuangMonthBasis::MainTable;
    let mut p = 0;
    while p < seq.len() {
        if !is_main(&seq[p]) {
            p += 1;
            continue;
        }
        let mut q = p;
        while q + 1 < seq.len() && is_main(&seq[q + 1]) {
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
                (DunhuangMonthBasis::MainTable, _) => next,
                (_, DunhuangFallback::EvidenceOnly) => known_end,
                (_, DunhuangFallback::MainTable) => known_end.or(next.filter(|&j| j - start <= 30)).or(known_end),
            };
            DunhuangMonthSpan { start_jdn: start, end_jdn: end, lunar_year: ly, month: m, is_leap: leap, calendar: calendar_of(ly), basis }
        })
        .filter(|s| s.end_jdn.is_none_or(|e| e > a) && s.start_jdn <= b)
        .collect()
}

/// 第 4 层仲裁：JDN 在敦煌历下的月日（见 [dunhuang_months] 的取舍规则）。止日不知之月只认前 29 日。
/// 不在敦煌政权年限内、或 [DunhuangFallback::EvidenceOnly] 下无据，返回 None。`main` 为 None 时等同只认有据之月。
pub fn resolve_dunhuang_day(
    dh: &DunhuangCalendar,
    main: Option<&HistoricalCalendar>,
    jdn: i64,
    fallback: DunhuangFallback,
) -> Option<DunhuangDay> {
    let months = dunhuang_months(dh, main, jdn, jdn, fallback);
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

/// 第 4 层仲裁：敦煌历某月（敦煌自身的岁、月序、闰否）的月首 JDN 与依据；无此月（如主表闰月在敦煌落在别月）为 None。
/// 敦煌文书纪日「某年某月某日」→ JDN = 月首 + 日 − 1（日不得超过该月日数，见 [DunhuangMonthSpan::end_jdn]）。
pub fn resolve_dunhuang_month(
    dh: &DunhuangCalendar,
    main: Option<&HistoricalCalendar>,
    lunar_year: i32,
    month: u8,
    is_leap: bool,
    fallback: DunhuangFallback,
) -> Option<DunhuangMonthSpan> {
    // 先定一个大致的 JDN 窗口：有据之月直接用；否则用主表同名之月
    let around = match dh.find_month(lunar_year, month, is_leap) {
        Some((_, m)) => m.jdn,
        None => {
            let (y, mo, d) = main?.new_moon_date(lunar_year, month, is_leap).or_else(|| {
                // 主表无此闰月（敦煌闰位不同）：取主表同岁同月前后找
                main?.new_moon_date(lunar_year, month, false)
            })?;
            super::gregorian::civil_to_jdn(y, mo, d)
        }
    };
    dunhuang_months(dh, main, around - 60, around + 60, fallback)
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

/// 主线某农历年行用之历的名称表（`data/calendar/汇总/main_calendar_periods.csv`）。
/// 只回答「当时行用何历」，职责是选名，日期仍由主表或现算给出。
#[derive(Debug, Clone, Default)]
pub struct MainCalendarPeriods {
    rows: Vec<(i32, Option<i32>, String)>,
}

/// 主表起点之前没有行用历资料，日期是现代定气定朔算的，显示时须如实说明。
pub const CALENDAR_NAME_COMPUTED: &str = "定气定朔（现代推算）";

impl MainCalendarPeriods {
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
                return Err(format!("main_calendar_periods line {}: invalid row", i + 1));
            };
            let to = if to.is_empty() {
                None
            } else {
                Some(to.parse::<i32>().map_err(|_| format!("main_calendar_periods line {}: bad to_year", i + 1))?)
            };
            rows.push((from, to, name.to_string()));
        }
        Ok(Self { rows })
    }

    /// 该农历年的历名；主表起点之前返回 [CALENDAR_NAME_COMPUTED]。
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
zhongyuan_shili,主表（据历还原）,-105,2200,各朝,据历还原推步,儒略历+格里高利历,《三千五百年历日天象》按各朝颁历推步
"#;
        let reg = LifaRegistry::from_csv(csv).unwrap();
        let g = reg.get("gregorian").unwrap();
        assert_eq!(g.name_simp, "格里高利历");
        assert_eq!(g.start_ce, Some(1582));
        assert_eq!(g.end_ce, None);
        assert!(g.active_in_ce_year(2000));
        assert!(!g.active_in_ce_year(1500));
        let z = reg.get("zhongyuan_shili").unwrap();
        assert!(z.active_in_ce_year(0));
        assert!(z.active_in_ce_year(-105));
        let active = reg.active_at_ce_year(1281);
        assert!(active.iter().any(|e| e.id == "shoushi"));
        assert!(active.iter().any(|e| e.id == "zhongyuan_shili"));
    }

    #[test]
    fn main_calendar_periods_are_contiguous_and_named() {
        let p = MainCalendarPeriods::from_csv(include_str!("../../../../data/calendar/汇总/main_calendar_periods.csv")).unwrap();
        for w in p.rows.windows(2) {
            assert_eq!(w[0].1.map(|t| t + 1), Some(w[1].0), "年段须首尾相接：{:?}", w[0]);
        }
        for (y, n) in [(700, "麟德历"), (1066, "明天历"), (1130, "历术失载"), (1800, "时宪历"), (2026, "农历（GB/T 33661—2017）")] {
            assert_eq!(p.name_for_lunar_year(y), n);
        }
        assert_eq!(p.name_for_lunar_year(-500), CALENDAR_NAME_COMPUTED);
    }

    fn main_table() -> Option<HistoricalCalendar> {
        let text = std::fs::read_to_string(crate::repo::repo_root().join("vendor/calendar/chinese_calendar/实历朔日表.csv")).ok()?;
        HistoricalCalendar::from_csv(&text).ok()
    }

    /// 比较表（邓文宽 1996 附录一）敦煌曆一栏每一格：按敦煌岁月查月首得原书朔干支，月首当日换算回该月初一，
    /// 月大小已知者末日为该月第 29/30 日、次日为次月初一。两种取舍（退回主表、只认有据之月）都须重现。
    #[test]
    fn dunhuang_comparison_table_reproduced() {
        use crate::calendar::dunhuang::{parse_comparison_table, DunhuangCalendar};
        use crate::calendar::gan_zhi::GAN_ZHI_60;
        let dh = DunhuangCalendar::embedded();
        let main = main_table();
        let rows = parse_comparison_table(include_str!("../../../../data/calendar/敦煌历日_比较表.csv")).unwrap();
        assert_eq!(rows.len(), 257, "比较表敦煌曆有朔之格（800 年以后，含两件重出之次年正月三格）");
        let gz = |j: i64| GAN_ZHI_60[(j + 49).rem_euclid(60) as usize];
        for fb in [DunhuangFallback::MainTable, DunhuangFallback::EvidenceOnly] {
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

    /// 退回主表时，786—1036 年敦煌历逐月连续、不重不漏；月长异常（非 29、30 日）者只在有据与无据之月相接处，
    /// 逐条钉住。无据之年与主表全同。
    #[test]
    fn dunhuang_merged_months_are_contiguous() {
        use crate::calendar::dunhuang::DunhuangCalendar;
        use crate::calendar::gregorian::civil_to_jdn;
        let Some(main) = main_table() else { return };
        let dh = DunhuangCalendar::embedded();
        let (a, b) = (civil_to_jdn(786, 3, 1), civil_to_jdn(1036, 12, 1));
        let ms = dunhuang_months(&dh, Some(&main), a, b, DunhuangFallback::MainTable);
        let mut odd = vec![];
        for w in ms.windows(2) {
            assert_eq!(w[0].end_jdn, Some(w[1].start_jdn), "{:?} 与次月不接", w[0]);
            let len = w[1].start_jdn - w[0].start_jdn;
            if !(29..=30).contains(&len) {
                odd.push((w[0].lunar_year, w[0].month, w[0].is_leap, len));
            }
        }
        // 无据之年（如 900 年以前 850 年）与主表同
        let s = resolve_dunhuang_month(&dh, Some(&main), 850, 5, false, DunhuangFallback::MainTable).unwrap();
        let (y, m, d) = main.new_moon_date(850, 5, false).unwrap();
        assert_eq!(s.start_jdn, civil_to_jdn(y, m, d));
        assert_eq!(s.basis, DunhuangMonthBasis::MainTable);
        // 主表閏五月在敦煌为六月（893）：按敦煌月序查不到閏五月
        assert!(resolve_dunhuang_month(&dh, Some(&main), 893, 5, true, DunhuangFallback::MainTable).is_none());
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
