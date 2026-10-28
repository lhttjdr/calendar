//! 历法域第 2 层（聚合/实算层）：查表得到实历朔日（主表朔日），与 [crate::calendar::chinese_lunar] 现算是这一层的两种实现。
//!
//! 本模块解析 `vendor/calendar/chinese_calendar/实历朔日表.csv`，为**主表**（历法 id 见 [lifa::LIFA_ID_ZHONGYUAN_SHILI]）提供与 [ChineseLunarYearData] 兼容的岁数据。
//! 主表 = 《三千五百年历日天象》按**各朝代颁行历法据历还原、推步**得出的连续历日，书中并无名为「中原正朔」的单一历法。
//! 在实历覆盖范围内应优先使用实历（该判断属第 4 层仲裁职责，见 [crate::calendar::convert]，不在本模块内）。
//! 第 3 层政权·帝王·年号由 [crate::calendar::era_names] 按农历年提供，与本朔日表同线。

use super::chinese_lunar::ChineseLunarYearData;
use super::gregorian::civil_to_julian_day;
use crate::math::real::{real, Real};
use std::collections::BTreeMap;

/// 实历朔日一条：儒略历/格里高利历 (年,月,日) 为该农历月初一，lunar_month 1..12，is_leap 表示闰月。
#[derive(Clone, Copy, Debug)]
struct ShuoriRow {
    gregorian_year: i32,
    gregorian_month: i32,
    gregorian_day: i32,
    lunar_year: i32,
    lunar_month: u8,
    is_leap: u8,
}

/// 实历数据：按农历年索引的朔日序列（每岁内已按公历日期排序）。
#[derive(Clone, Debug, Default)]
pub struct HistoricalCalendar {
    /// lunar_year -> 该岁所有初一 [(儒略历/格里高利历 y,m,d, 农历月 1..12, 是否闰月 0/1)]，按日期升序。
    by_year: BTreeMap<i32, Vec<(i32, i32, i32, u8, u8)>>,
}

/// 实历覆盖的农历年范围（含端点）。数据来源见 `vendor/calendar/chinese_calendar/实历朔日表.csv`。
pub const HISTORICAL_LUNAR_YEAR_MIN: i32 = -105;
pub const HISTORICAL_LUNAR_YEAR_MAX: i32 = 2200;

impl HistoricalCalendar {
    /// 从 CSV 文本解析实历朔日表。
    ///
    /// CSV 格式：`gregorian_year,gregorian_month,gregorian_day,lunar_year,lunar_month,is_leap`（不含年号，年号见 era_names 表）。
    /// 每行表示该儒略历/格里高利历日为对应农历年的某月初一。闰月行为 `is_leap=1`。1582 年前为儒略历，之后为格里高利历。
    pub fn from_csv(csv: &str) -> Result<Self, String> {
        let mut by_year: BTreeMap<i32, Vec<(i32, i32, i32, u8, u8)>> = BTreeMap::new();
        for (line_no, line) in csv.lines().enumerate() {
            let line_no = line_no + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with("gregorian_year") || line.starts_with("western_year") {
                continue;
            }
            let row = parse_csv_row(line).ok_or_else(|| format!("line {}: invalid row", line_no))?;
            by_year
                .entry(row.lunar_year)
                .or_default()
                .push((
                    row.gregorian_year,
                    row.gregorian_month,
                    row.gregorian_day,
                    row.lunar_month,
                    row.is_leap,
                ));
        }
        for (ly, vec) in by_year.iter_mut() {
            vec.sort_by_key(|&(y, m, d, _, _)| (y, m, d));
            // 完整岁 12 或 13 朔；仅含去年十一、十二的边界年可为 2 朔
            if vec.len() < 2 || vec.len() > 13 {
                return Err(format!(
                    "lunar year {}: expected 2..=13 new moons, got {}",
                    ly,
                    vec.len()
                ));
            }
        }
        Ok(HistoricalCalendar { by_year })
    }

    /// 判断指定农历年是否在实历表中有数据。
    pub fn has_year(&self, lunar_year: i32) -> bool {
        self.by_year.contains_key(&lunar_year)
    }

    /// 获取实历覆盖的农历年范围（实际存在的最小/最大年）。
    pub fn year_range(&self) -> Option<(i32, i32)> {
        let min = *self.by_year.keys().next()?;
        let max = *self.by_year.keys().next_back()?;
        Some((min, max))
    }

    /// 主表某月（建寅月序）初一的民用历日期（1582-10-15 以前为儒略历，见 [civil_to_julian_day]）；表中无此月返回 None。
    pub fn new_moon_date(&self, lunar_year: i32, month: u8, is_leap: bool) -> Option<(i32, i32, i32)> {
        self.by_year
            .get(&lunar_year)?
            .iter()
            .find(|&&(_, _, _, m, leap)| m == month && (leap == 1) == is_leap)
            .map(|&(y, m, d, _, _)| (y, m, d))
    }

    /// 用实历表为指定农历年生成 [ChineseLunarYearData]。
    ///
    /// 岁 = 去年十一月朔～今年十二月朔（与算历一致：含冬至之月为十一月）。朔日 JD 取该日 0h UT；表中日期为民用历
    /// （1582-10-15 以前儒略历），经 [civil_to_julian_day] 换算，不能当前推格里高利历用。
    /// 朔序固定为：去年十一月、去年十二月、今年正月～十二月（含闰），共 14 或 15 朔。
    /// 中气由相邻朔日中点合成，闰月区间不占中气。
    pub fn year_data(&self, lunar_year: i32) -> Option<ChineseLunarYearData> {
        let curr = self.by_year.get(&lunar_year)?.as_slice();
        let prev_year = lunar_year - 1;
        let mut new_moon_jds: Vec<Real> = Vec::with_capacity(15);
        let mut is_leap: Vec<u8> = Vec::with_capacity(15);
        if let Some(prev) = self.by_year.get(&prev_year) {
            for &(wy, wm, wd, lm, leap) in prev {
                if lm == 11 {
                    new_moon_jds.push(civil_to_julian_day(wy, wm, wd));
                    is_leap.push(leap);
                    break;
                }
            }
            for &(wy, wm, wd, lm, leap) in prev {
                if lm == 12 {
                    new_moon_jds.push(civil_to_julian_day(wy, wm, wd));
                    is_leap.push(leap);
                    break;
                }
            }
        }
        for &(wy, wm, wd, _lm, leap) in curr {
            new_moon_jds.push(civil_to_julian_day(wy, wm, wd));
            is_leap.push(leap);
        }
        // 12 中气：每个非闰月区间取中点
        let mut zhong_qi_jds: Vec<Real> = Vec::with_capacity(12);
        for i in 0..new_moon_jds.len().saturating_sub(1) {
            if zhong_qi_jds.len() >= 12 {
                break;
            }
            if is_leap.get(i).copied().unwrap_or(0) == 0 {
                let jd_a = new_moon_jds[i];
                let jd_b = new_moon_jds[i + 1];
                zhong_qi_jds.push(jd_a + (jd_b - jd_a) * real(0.5));
            }
        }
        debug_assert!(
            zhong_qi_jds.len() == 12,
            "实历岁应有 12 个中气: lunar_year={} len={}",
            lunar_year,
            zhong_qi_jds.len()
        );
        Some(ChineseLunarYearData {
            lunar_year,
            new_moon_jds,
            zhong_qi_jds,
        })
    }
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

    #[test]
    fn historical_year_range() {
        assert!(HISTORICAL_LUNAR_YEAR_MIN <= -105);
        assert!(HISTORICAL_LUNAR_YEAR_MAX >= 2200);
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
        let cal = HistoricalCalendar::from_csv(csv).unwrap();
        assert!(cal.has_year(-105));
        let year_data = cal.year_data(-105).unwrap();
        assert_eq!(year_data.lunar_year, -105);
        // 岁 = 去年十一、十二 + 今年正..十二 → 14 朔
        assert_eq!(year_data.new_moon_jds.len(), 14);
        assert_eq!(year_data.zhong_qi_jds.len(), 12);
        // 实历 -105 年正月初一 = 儒略历 -105/2/15（表中日期为民用历，不是前推格里高利历）
        let date = from_julian_day_in_year(civil_to_julian_day(-105, 2, 15), &year_data, None).unwrap();
        assert_eq!(date.year, -105);
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 1);
        assert!(!date.is_leap_month);
    }
}
