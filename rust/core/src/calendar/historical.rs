//! 历法域第 2 层（聚合/实算层）：查**今人历表**得到逐月朔日，与 [crate::calendar::chinese_lunar] 现算、
//! [crate::calendar::push_calendar] 按术推步同为这一层的实现。
//!
//! 本模块解析 `vendor/calendar/chinese_calendar/实历朔日表.csv`：张培瑜《三千五百年历日天象》（1990）的历日，经廖育栋
//! ChineseCalendar 转录（个别处据其勘误改），2051 年以后为转录者自行推算。书中政权序列取文物出版社《中国历史年代简表》
//! 的「纪年主序列」，历日自称「按历代实行历法推步得出」，但不载各期所用历法、常数与逐条依据。它是**今人历表**
//! （资料可靠性第 5 级），不是史实记录，也不比按术推步优先；何时用它由第 4 层 [crate::calendar::lifa::resolve_year] 仲裁。
//! 第 3 层政权·帝王·年号由 [crate::calendar::era_names] 按农历年提供。

use super::chinese_lunar::ChineseLunarYearData;
use super::gregorian::{civil_from_jdn, civil_to_jdn};
use crate::math::real::{from_i32, real, Real};
use std::collections::BTreeMap;

/// 一月：(朔日 JDN, 月序 1..=12, 是否闰月)。与 [crate::calendar::push_calendar::year_months] 同形。
pub type MonthStart = (i64, u8, bool);

/// 今人历表（朔日表）：按农历年索引的逐月朔日（每岁内按朔日先后）。
#[derive(Clone, Debug, Default)]
pub struct ShuoriTable {
    by_year: BTreeMap<i32, Vec<MonthStart>>,
}

/// 张培瑜历表（转录本）覆盖的农历年范围（含端点）。
pub const ZHANG_TABLE_YEAR_MIN: i32 = -105;
pub const ZHANG_TABLE_YEAR_MAX: i32 = 2200;

/// 张培瑜历表在 [crate::calendar::lifa::RiliSource::ModernTable] 中的名称。
pub const ZHANG_TABLE_NAME: &str = "张培瑜历表";

impl ShuoriTable {
    /// 从 CSV 文本解析朔日表。
    ///
    /// CSV 格式：`gregorian_year,gregorian_month,gregorian_day,lunar_year,lunar_month,is_leap`（不含年号，年号见 era_names 表）。
    /// 每行表示该民用历日（1582-10-15 以前儒略历）为对应农历年的某月初一；闰月行 `is_leap=1`。
    pub fn from_csv(csv: &str) -> Result<Self, String> {
        let mut by_year: BTreeMap<i32, Vec<MonthStart>> = BTreeMap::new();
        for (line_no, line) in csv.lines().enumerate() {
            let line_no = line_no + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with("gregorian_year") || line.starts_with("western_year") {
                continue;
            }
            let row = parse_csv_row(line).ok_or_else(|| format!("line {}: invalid row", line_no))?;
            by_year.entry(row.lunar_year).or_default().push((
                civil_to_jdn(row.gregorian_year, row.gregorian_month, row.gregorian_day),
                row.lunar_month,
                row.is_leap == 1,
            ));
        }
        for (ly, vec) in by_year.iter_mut() {
            vec.sort_by_key(|&(j, _, _)| j);
            // 完整岁 12 或 13 朔；仅含去年十一、十二的边界年可为 2 朔
            if vec.len() < 2 || vec.len() > 13 {
                return Err(format!("lunar year {}: expected 2..=13 new moons, got {}", ly, vec.len()));
            }
        }
        Ok(ShuoriTable { by_year })
    }

    /// 表中是否有该农历年。
    pub fn has_year(&self, lunar_year: i32) -> bool {
        self.by_year.contains_key(&lunar_year)
    }

    /// 表中实际存在的最小、最大农历年。
    pub fn year_range(&self) -> Option<(i32, i32)> {
        let min = *self.by_year.keys().next()?;
        let max = *self.by_year.keys().next_back()?;
        Some((min, max))
    }

    /// 某岁各月（按朔日先后）。
    pub fn year_months(&self, lunar_year: i32) -> Option<&[MonthStart]> {
        self.by_year.get(&lunar_year).map(Vec::as_slice)
    }

    /// 某月（建寅月序）初一的民用历日期（1582-10-15 以前为儒略历）；表中无此月返回 None。
    pub fn new_moon_date(&self, lunar_year: i32, month: u8, is_leap: bool) -> Option<(i32, i32, i32)> {
        self.by_year
            .get(&lunar_year)?
            .iter()
            .find(|&&(_, m, leap)| m == month && leap == is_leap)
            .map(|&(j, _, _)| civil_from_jdn(j))
    }

    /// 朔日 JDN 落在 `[from_jdn, to_jdn]` 内的各月：(朔日 JDN, 农历年, 月, 闰否)，按朔日先后。
    pub fn months_in_jdn_range(&self, from_jdn: i64, to_jdn: i64) -> Vec<(i64, i32, u8, bool)> {
        let (y0, _, _) = civil_from_jdn(from_jdn);
        let (y1, _, _) = civil_from_jdn(to_jdn);
        let mut out: Vec<(i64, i32, u8, bool)> = self
            .by_year
            .range(y0 - 1..=y1 + 1)
            .flat_map(|(&ly, v)| v.iter().map(move |&(j, m, leap)| (j, ly, m, leap)))
            .filter(|&(j, ..)| (from_jdn..=to_jdn).contains(&j))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// 用本表为指定农历年生成 [ChineseLunarYearData]（见 [year_data_from_months]）。
    pub fn year_data(&self, lunar_year: i32) -> Option<ChineseLunarYearData> {
        let curr = self.by_year.get(&lunar_year)?;
        let prev = self.by_year.get(&(lunar_year - 1)).map(Vec::as_slice).unwrap_or(&[]);
        Some(year_data_from_months(lunar_year, prev, curr))
    }
}

/// 由逐月朔日构造农历年的 [ChineseLunarYearData]：查表、按术推步、实物历日共用。
///
/// 岁 = 去年十一月朔～今年十二月朔（与现算一致：含冬至之月为十一月）。朔日 JD 取该日 0h UT。
/// 朔序固定为：去年十一月、去年十二月（取自 `prev`，即上一岁各月）、今年各月（`curr`，含闰），共 14 或 15 朔。
/// 中气由相邻朔日中点合成，闰月区间不占中气；这只用于无中气置闰的月序标记，不表示当时历书所注的气。
pub fn year_data_from_months(lunar_year: i32, prev: &[MonthStart], curr: &[MonthStart]) -> ChineseLunarYearData {
    let jd = |j: i64| from_i32(j as i32) - real(0.5);
    let mut new_moon_jds: Vec<Real> = Vec::with_capacity(15);
    let mut is_leap: Vec<bool> = Vec::with_capacity(15);
    for want in [11u8, 12] {
        if let Some(&(j, _, leap)) = prev.iter().find(|&&(_, m, _)| m == want) {
            new_moon_jds.push(jd(j));
            is_leap.push(leap);
        }
    }
    for &(j, _, leap) in curr {
        new_moon_jds.push(jd(j));
        is_leap.push(leap);
    }
    // 13 中气：每个非闰月区间取中点（去年十一月至今年十一月，末一个代下一冬至）
    let mut zhong_qi_jds: Vec<Real> = Vec::with_capacity(13);
    for i in 0..new_moon_jds.len().saturating_sub(1) {
        if zhong_qi_jds.len() >= 13 {
            break;
        }
        if !is_leap[i] {
            let (a, b) = (new_moon_jds[i], new_moon_jds[i + 1]);
            zhong_qi_jds.push(a + (b - a) * real(0.5));
        }
    }
    debug_assert!(zhong_qi_jds.len() == 13 || prev.is_empty(), "岁应有 13 个中气: lunar_year={} len={}", lunar_year, zhong_qi_jds.len());
    ChineseLunarYearData { lunar_year, new_moon_jds, zhong_qi_jds }
}

/// 朔日表一行：民用历 (年,月,日) 为该农历月初一，lunar_month 1..12，is_leap 表示闰月。
#[derive(Clone, Copy, Debug)]
struct ShuoriRow {
    gregorian_year: i32,
    gregorian_month: i32,
    gregorian_day: i32,
    lunar_year: i32,
    lunar_month: u8,
    is_leap: u8,
}

fn parse_csv_row(line: &str) -> Option<ShuoriRow> {
    let mut fields = split_csv_line(line);
    let gregorian_year: i32 = fields.next()?.trim().parse().ok()?;
    let gregorian_month: i32 = fields.next()?.trim().parse().ok()?;
    let gregorian_day: i32 = fields.next()?.trim().parse().ok()?;
    let lunar_year: i32 = fields.next()?.trim().parse().ok()?;
    let lunar_month: u8 = fields.next()?.trim().parse().ok()?;
    let is_leap: u8 = fields.next()?.trim().parse().ok()?;
    if lunar_month < 1 || lunar_month > 12 || is_leap > 1 {
        return None;
    }
    Some(ShuoriRow {
        gregorian_year,
        gregorian_month,
        gregorian_day,
        lunar_year,
        lunar_month,
        is_leap,
    })
}

/// 简单 CSV 行分割：逗号分隔，允许双引号内含逗号。
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
    use crate::calendar::chinese_lunar::from_julian_day_in_year;
    use crate::calendar::gregorian::civil_to_julian_day;

    /// 岁末含下一冬至之月是十一月，不是闰月；真闰十月（1984）仍为闰月
    #[test]
    fn year_end_month_not_leap() {
        let root = crate::repo::repo_root();
        let Ok(text) = std::fs::read_to_string(root.join("vendor/calendar/chinese_calendar/实历朔日表.csv")) else {
            return;
        };
        let cal = ShuoriTable::from_csv(&text).unwrap();
        let on = |ly: i32, y: i32, m: i32, d: i32| {
            from_julian_day_in_year(civil_to_julian_day(y, m, d), &cal.year_data(ly).unwrap(), None).unwrap()
        };
        let a = on(2024, 2024, 12, 1);
        assert_eq!((a.month, a.day, a.is_leap_month), (11, 1, false), "2024-12-01 十一月初一");
        let b = on(1765, 1765, 12, 12);
        assert_eq!((b.month, b.day, b.is_leap_month), (11, 1, false), "乾隆三十年十一月初一（该岁閏二月）");
        let c = on(1984, 1984, 11, 23);
        assert_eq!((c.month, c.day, c.is_leap_month), (10, 1, true), "1984 閏十月初一");
        let e = on(1984, 1984, 12, 22);
        assert_eq!((e.month, e.day, e.is_leap_month), (11, 1, false), "1984 十一月初一");
    }

    #[test]
    fn zhang_table_year_range() {
        assert!(ZHANG_TABLE_YEAR_MIN <= -105);
        assert!(ZHANG_TABLE_YEAR_MAX >= 2200);
    }

    #[test]
    fn parse_csv_line_quoted() {
        let line = r#" -105,2,15,-105,1,0,"(漢武帝元封五年)","(汉武帝元封五年)" "#;
        let row = parse_csv_row(line.trim()).unwrap();
        assert_eq!(row.gregorian_year, -105);
        assert_eq!(row.gregorian_month, 2);
        assert_eq!(row.gregorian_day, 15);
        assert_eq!(row.lunar_year, -105);
        assert_eq!(row.lunar_month, 1);
        assert_eq!(row.is_leap, 0);
    }

    #[test]
    fn year_data_from_csv_snippet() {
        let csv = r#"gregorian_year,gregorian_month,gregorian_day,lunar_year,lunar_month,is_leap
-105,2,15,-105,1,0
-105,3,17,-105,2,0
-105,4,15,-105,3,0
-105,5,15,-105,4,0
-105,6,13,-105,5,0
-105,7,13,-105,6,0
-105,8,11,-105,7,0
-105,9,10,-105,8,0
-105,10,9,-105,9,0
-105,11,8,-105,10,0
-105,12,8,-105,11,0
-104,1,6,-105,12,0
-105,12,8,-106,11,0
-104,1,6,-106,12,0
"#;
        let cal = ShuoriTable::from_csv(csv).unwrap();
        assert!(cal.has_year(-105));
        let year_data = cal.year_data(-105).unwrap();
        assert_eq!(year_data.lunar_year, -105);
        // 岁 = 去年十一、十二 + 今年正..十二 → 14 朔
        assert_eq!(year_data.new_moon_jds.len(), 14);
        assert_eq!(year_data.zhong_qi_jds.len(), 13);
        // 实历 -105 年正月初一 = 儒略历 -105/2/15（表中日期为民用历，不是前推格里高利历）
        let date = from_julian_day_in_year(civil_to_julian_day(-105, 2, 15), &year_data, None).unwrap();
        assert_eq!(date.year, -105);
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 1);
        assert!(!date.is_leap_month);
    }
}
