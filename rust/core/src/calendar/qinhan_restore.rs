//! 秦至汉初历日的今人复原（第 5 级，与张表同级）：李忠林（2012，《中国史研究》2012 年第 2 期）据出土历简排定，
//! 本项目照 ytliu0/ChineseCalendar 说明页（QinHanCalendars）所列公式与参数独立实现（不执行上游代码），与
//! `scripts/calendar_data/qinhan_eval/hzx.py` 同。复原结果不是史实，见 doc/calendar/qin-han-calendar-evaluation.md。
//!
//! - 三个合朔历元 A、B、C（四分术朔策 27759/940），十九年七闰按 3-3-3-2-3-3-2 年序，文帝后元元年（前 163）改章首；
//! - 行用：A 至汉高祖五年十二月，B 至文帝前元十六年后九月，C 至太初元年四月；
//! - 张表前 106 年正月至太初元年四月与此逐月全同（测试 `lifa::tests::qinhan_restore_matches_zhang_table`）。
//!
//! 张表起于前 106 年，前 221—前 107 年中原一线的今人历表即取本复原（[super::lifa::zhongyuan_year_months]）。
//! 2026-11-22 起秦汉初缺省取出土历谱实物（[super::artifacts]）与实用历推步（[super::qinhan_shiyong]），本复原只在
//! 「今人历表优先」选项下取用、并作对照：其参数据历简拟合，但各月不注所据，与实物逐月比较见 doc/calendar/qinhan-shiwu-checks.md。

/// 今人历表名（[super::lifa::RiliSource::ModernTable]）。
pub const QINHAN_TABLE_NAME: &str = "李忠林秦汉初复原历";

/// 覆盖的夏正年（秦统一至太初元年改历前一年；太初元年正月至四月另由张表或推步给出）。
pub const QINHAN_YEARS: (i32, i32) = (-220, -104);

const J: [i64; 19] = [0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1];

/// 建亥年 N 各月：(朔日 JDN, 月, 是否后九月)，自十月起。
pub fn jianhai_year_months(n: i32) -> Vec<(i64, u8, bool)> {
    // (十月朔历元之日 JDN，即历元时刻 x.5 + 0.5；章首年；历元起积月)
    let (j0, yz, da): (i64, i32, i64) = if n >= -162 {
        (1_646_164, -179, 321) // 历法 C
    } else if n > -201 {
        (1_633_702, -225, 174) // 历法 B
    } else {
        (1_589_524, -225, 1670) // 历法 A
    };
    let d = (n - yz) as i64;
    let (c, k) = (d.div_euclid(19), d.rem_euclid(19) as usize);
    let r: i64 = (0..k).map(|i| 12 + J[i]).sum();
    let a = da + 235 * c + r;
    let len = 12 + J[k] as usize;
    let day = |base: i64, m: i64| base + (m * 27759).div_euclid(940);
    const LABELS: [u8; 12] = [10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    (0..len)
        .map(|j| {
            // 汉高祖五年正月起改用历法 B
            let jdn = if n == -201 && j >= 3 { day(1_633_702, 470 + j as i64) } else { day(j0, a + j as i64) };
            if j < 12 { (jdn, LABELS[j], false) } else { (jdn, 9, true) }
        })
        .collect()
}

/// 夏正年 y 各月 (朔日 JDN, 月, 闰否)，后九月记作闰九月；y 在 [QINHAN_YEARS] 外返回 None。
pub fn year_months(lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    (QINHAN_YEARS.0..=QINHAN_YEARS.1)
        .contains(&lunar_year)
        .then(|| super::zhuanxu::jianhai_months_to_xiazheng(lunar_year, jianhai_year_months))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 说明页算例：元光四年十月朔壬申（前 132-11-25）。
    #[test]
    fn upstream_example() {
        let m = jianhai_year_months(-130);
        assert_eq!(m[0].1, 10);
        assert_eq!((m[0].0 + 49).rem_euclid(60), 8, "壬申");
        assert_eq!(m[0].0, crate::calendar::gregorian::civil_to_jdn(-131, 11, 25) as i64);
    }

    /// 颛顼历推步（冬至在十一月 → 后元起雨水在正月）与复原历置闰逐年全同，朔日只差 0 或复原晚 1 日。
    #[test]
    fn zhuanxu_reckoning_vs_restore() {
        let (mut same, mut later) = (0, 0);
        for n in -220..=-104 {
            let (a, b) = (jianhai_year_months(n), super::super::zhuanxu::jianhai_year_months(n));
            assert_eq!(a.len(), b.len(), "{n} 年置闰");
            for (x, y) in a.iter().zip(&b) {
                assert_eq!((x.1, x.2), (y.1, y.2));
                match x.0 - y.0 {
                    0 => same += 1,
                    1 => later += 1,
                    d => panic!("{n} 年 {} 月差 {d}", x.1),
                }
            }
        }
        assert_eq!((same, later), (660, 788));
    }
}
