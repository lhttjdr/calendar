//! 近现代（1914—2016）主序列历日：按当时编算规则现算的逐月朔日（第 2 层，聚合/实算）。
//!
//! 民国三年（1914）起中央观象台历书「用东西各国通行之法数推算，且以太阳平时为标准」：定气、定朔、无中气置闰，
//! 与今日规则相同，只是定日基准随年段不同——1914—1927 北京地方平时（东经 116°25′），1928 年起东经 120°
//! （1928 年两种基准逐月相同；1930 年天文研究所《天文年历》用首都南京地方平时，与东经 120° 无判别之月）。
//! 逐月朔日由 `rust/core/examples/jindai_months.rs` 用与网页同一组历表（VSOP87B + DE431 改正、ELPMPP02、IAU2000 章动）
//! 现算，写入 `data/calendar/jindai_months.csv` 编入，这里不再依赖历表数据，仲裁层（[super::lifa]）同步取用。
//! 规则依据、官历载不载旧历、各年段存疑之月见 `doc/calendar/jindai-calendar.md`，逐月检验见 `doc/calendar/jindai-checks.md`。
//!
//! 张培瑜历表此段 1274 月与本表逐月全同（检验表「合张表」列），张表只作对照。

use super::historical::MonthStart;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// 主序列行用历名（`data/calendar/汇总/main_series_calendar_periods.csv`）中按本模块现算者。
pub const JINDAI_LIFA: &[&str] = &["中华民国历书（新法）", "国民历", "夏历（现代天文编算）"];

/// 本表覆盖的农历年。
pub const YEARS: std::ops::RangeInclusive<i32> = 1914..=2016;

fn table() -> &'static BTreeMap<i32, Vec<MonthStart>> {
    static T: OnceLock<BTreeMap<i32, Vec<MonthStart>>> = OnceLock::new();
    T.get_or_init(|| {
        let csv = include_str!("../../../../data/calendar/jindai_months.csv");
        let mut by: BTreeMap<i32, Vec<MonthStart>> = BTreeMap::new();
        for l in csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("农历年") && !l.trim().is_empty()) {
            let c: Vec<&str> = l.split(',').collect();
            let y: i32 = c[0].parse().expect("农历年");
            let m: u8 = c[1].parse().expect("月");
            let leap = c[2] == "1";
            let jdn: i64 = c[5].parse().expect("朔日JDN");
            assert_eq!(super::gan_zhi::GAN_ZHI_60[(jdn + 49).rem_euclid(60) as usize], c[6], "近现代月表 {l}");
            by.entry(y).or_default().push((jdn, m, leap));
        }
        by
    })
}

/// 某农历年各月 (朔日 JDN, 月, 闰否)；不在 [YEARS] 内返回 None。
pub fn year_months(lunar_year: i32) -> Option<Vec<MonthStart>> {
    table().get(&lunar_year).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_1914_2016_contiguously() {
        let t = table();
        assert_eq!(t.keys().next(), Some(&1914));
        assert_eq!(t.keys().last(), Some(&2016));
        let all: Vec<i64> = t.values().flatten().map(|m| m.0).collect();
        for w in all.windows(2) {
            assert!(matches!(w[1] - w[0], 29 | 30), "月长 {}", w[1] - w[0]);
        }
        for (y, ms) in t {
            assert!(ms.len() == 12 || ms.len() == 13, "{y}");
            assert_eq!(ms[0].1, 1, "{y} 正月");
        }
    }

    /// 《三年历书·凡例》：新法九小十大（1914 年十月朔 11 月 17 日，北京平时 23:47 合朔）
    #[test]
    fn minguo_3_new_method() {
        let y = year_months(1914).unwrap();
        let ten = y.iter().find(|m| m.1 == 10 && !m.2).unwrap().0;
        let nine = y.iter().find(|m| m.1 == 9 && !m.2).unwrap().0;
        assert_eq!(ten - nine, 29, "九月小");
        assert_eq!(super::super::gregorian::civil_from_jdn(ten), (1914, 11, 17));
    }

    /// 与张表逐月全同（张表只作对照）
    #[test]
    fn same_as_zhang_table() {
        let Ok(csv) = std::fs::read_to_string(crate::repo::repo_root().join("vendor/calendar/chinese_calendar/实历朔日表.csv")) else { return };
        let z = crate::calendar::historical::ShuoriTable::from_csv(&csv).unwrap();
        for y in YEARS {
            assert_eq!(year_months(y).unwrap(), z.year_months(y).unwrap().to_vec(), "{y}");
        }
    }
}
