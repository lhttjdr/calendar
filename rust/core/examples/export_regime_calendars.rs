//! 导出推步政权（蜀、吴、后秦、北凉、北魏—西魏—北周—隋、东魏—北齐）的朔闰表到 data/calendar/推步政权朔日表.csv，
//! 供史书检验脚本（scripts/calendar_data/gaiyuan_date.py 的 `regime_month_span`）换算非主表政权的日期。
//! 日期为民用历（1582 年前儒略历），与实历朔日表一致。
//! 用法（rust/ 目录）：cargo run --release -p lunar-core --example export_regime_calendars
use lunar_core::calendar::gregorian::civil_from_jdn;
use lunar_core::calendar::push_calendar::{regime_calendars, regime_year_months};

fn main() {
    let mut out = String::from("regime,calendar,gregorian_year,gregorian_month,gregorian_day,lunar_year,lunar_month,is_leap\n");
    let mut n = 0;
    for (regime, y0, y1, _) in regime_calendars() {
        for y in y0..=y1 {
            let (cal, months) = regime_year_months(regime, y).unwrap();
            for (jdn, m, leap) in months {
                let (gy, gm, gd) = civil_from_jdn(jdn);
                out += &format!("{regime},{cal},{gy},{gm},{gd},{y},{m},{}\n", leap as u8);
                n += 1;
            }
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/calendar/推步政权朔日表.csv");
    std::fs::write(&path, out).unwrap();
    println!("写出 {} 行到 {}", n, path.display());
}
