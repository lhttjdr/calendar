//! 杂节气统计：定气下冬至到清明的日数分布（寒食「冬至後一百五日」与「清明前一二日」两说的关系）。
//! 用法（仓库根）：cargo run -p lunar-core --no-default-features --features real-f64 --example zajie_stats
use lunar_core::astronomy::aspects::solar_term_jds_for_year_cached;
use lunar_core::astronomy::ephemeris::load_earth_vsop87_from_repo;
use lunar_core::calendar::chinese_lunar::tt_to_day_number_utc8;
use std::collections::BTreeMap;
fn main() {
    let vsop = load_earth_vsop87_from_repo().unwrap();
    let mut hist: BTreeMap<i64, u32> = BTreeMap::new();
    for y in 1901..=2100 {
        let dz = tt_to_day_number_utc8(solar_term_jds_for_year_cached(&vsop, y - 1)[18]);
        let qm = tt_to_day_number_utc8(solar_term_jds_for_year_cached(&vsop, y)[1]);
        *hist.entry(qm - dz).or_default() += 1;
    }
    println!("1901—2100 年，清明日 − 上年冬至日（东八区）：");
    for (k, v) in hist {
        println!("  {k} 日：{v} 年（冬至后一百五日 = 清明{}）", if 105 - k >= 0 { format!("后 {} 日", 105 - k) } else { format!("前 {} 日", k - 105) });
    }
    // 社日两家之差
    use lunar_core::folk::zajie::{she_jin_wu, she_wu_wu};
    let mut diff: BTreeMap<(&str, i64), u32> = BTreeMap::new();
    for y in 1901..=2100 {
        let t = solar_term_jds_for_year_cached(&vsop, y);
        let d = |k: usize| tt_to_day_number_utc8(t[k]);
        *diff.entry(("春社", she_jin_wu(d(0)) - she_wu_wu(d(21)))).or_default() += 1;
        *diff.entry(("秋社", she_jin_wu(d(12)) - she_wu_wu(d(9)))).or_default() += 1;
    }
    println!("1901—2100 年，社日（二分近戊 − 立春/立秋后五戊）：");
    for ((n, k), v) in diff {
        println!("  {n} {k:+} 日：{v} 年");
    }
}

