//! 导出推步政权（蜀、吴、后秦、北凉、北魏—西魏—北周—隋、东魏—北齐、金、辽、南明、民间通书）的朔闰表到 data/calendar/推步政权朔日表.csv，
//! 供史书检验脚本（scripts/calendar_data/gaiyuan_date.py 的 `regime_month_span`）换算中原一线以外各政权的日期。
//! 日期为民用历（1582 年前儒略历），与实历朔日表一致。
//!
//! 主表是纯按历术推步（`regime_year_months_raw`），供与史书、张表逐条判别时作「推步」一家；另写
//! `推步政权史载修正表.csv`：只收应用史载例外表（`dingshuo_exceptions.csv` 中「政权：历名」各条）后与纯推步不同之岁，
//! 列整岁各月、格式同主表。`gaiyuan_date.py` 以此覆盖主表，给史书记录定日期时用当时实际行用的历日。
//! 用法（rust/ 目录）：cargo run --release -p lunar-core --example export_regime_calendars
use lunar_core::calendar::gregorian::civil_from_jdn;
use lunar_core::calendar::push_calendar::{regime_calendars, regime_year_months, regime_year_months_raw};

fn main() {
    const HEADER: &str = "regime,calendar,gregorian_year,gregorian_month,gregorian_day,lunar_year,lunar_month,is_leap\n";
    let mut out = String::from(HEADER);
    let mut fixed = String::from(HEADER);
    let mut n = 0;
    let mut n_fixed = 0;
    for (regime, y0, y1, _) in regime_calendars() {
        for y in y0..=y1 {
            // 无推步之年（历术函数给空表者）不导出
            let Some((cal, months)) = regime_year_months_raw(regime, y) else { continue };
            let row = |jdn: i64, m: u8, leap: bool| {
                let (gy, gm, gd) = civil_from_jdn(jdn);
                format!("{regime},{cal},{gy},{gm},{gd},{y},{m},{}\n", leap as u8)
            };
            for &(jdn, m, leap) in &months {
                out += &row(jdn, m, leap);
                n += 1;
            }
            if let Some((_, used)) = regime_year_months(regime, y).filter(|(_, u)| *u != months) {
                for (jdn, m, leap) in used {
                    fixed += &row(jdn, m, leap);
                }
                n_fixed += 1;
            }
        }
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/calendar");
    std::fs::write(dir.join("推步政权朔日表.csv"), out).unwrap();
    std::fs::write(dir.join("推步政权史载修正表.csv"), fixed).unwrap();
    println!("写出 {n} 行到 推步政权朔日表.csv；史载修正 {n_fixed} 岁到 推步政权史载修正表.csv");
}
