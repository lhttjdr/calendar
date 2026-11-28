//! 主序列逐年历日来源（逐历取法，缺省）合并为连续年段，供盘点哪些年段仍取今人历表。
//! 用法（仓库根）：cargo run --release -p lunar-core --no-default-features --features real-f64 --example main_series_source_ranges
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{main_series_year_months, MainSeriesPreference};

fn main() {
    let read = |p: &str| std::fs::read_to_string(p).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let mut runs: Vec<(i32, i32, String)> = Vec::new();
    for y in -230..=2210 {
        let s = main_series_year_months(Some(&zhang), y, MainSeriesPreference::Evidence)
            .map(|(_, src)| src.label())
            .unwrap_or_else(|| "现算".to_string());
        match runs.last_mut() {
            Some(r) if r.2 == s => r.1 = y,
            _ => runs.push((y, y, s)),
        }
    }
    for (a, b, s) in runs {
        println!("{a}\t{b}\t{}\t{s}", b - a + 1);
    }
}
