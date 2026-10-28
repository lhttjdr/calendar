//! 历法域第 4 层（仲裁/路由层）：按时代/政权路由该用哪部历法。
//!
//! 历法指一套推步或颁行的历制，与政权、年号无关；时间锚定在儒略日或儒略历/格里高利历。
//! 数据来源：`data/calendar/汇总/lifa_summary.csv`（历法总表）。第 3 层政权·帝王·年号见 [crate::calendar::era_names]，
//! 实历朔日（第 2 层，某历法的具体岁数据）见 [crate::calendar::historical]。

use std::collections::BTreeMap;

use super::chinese_lunar::{compute_year_data, ChineseLunarYearData};
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
}
