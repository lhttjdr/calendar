//! 《明实录》月朔检验：`data/calendar/mingshilu_shuo.csv`（实录「某月某某朔」，`scripts/calendar_data/mingshilu_shuo.py`
//! 由识典古籍录文抽出）与大统推步（`push_calendar::reckon_lifa_year("大统历")`）、张表（vendor/calendar/chinese_calendar/实历朔日表.csv）
//! 逐月比较，限 1368—1644 年。
//!
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example mingshilu_check
//! 输出 data/calendar/mingshilu_check.csv（逐条），终端打印汇总与「只合一家」之条。
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::gregorian::civil_from_jdn;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::push_calendar::reckon_lifa_year;
use std::collections::BTreeMap;

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let src = read("data/calendar/mingshilu_shuo.csv");
    let mut reck: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
    let mut out = String::from("年,月,闰,实录朔,推步朔,推步日期,张表朔,张表日期,合推步,合张表,卷,原句,备注\n");
    let (mut n, mut hr, mut ht, mut only_r, mut only_t, mut neither, mut nomonth) = (0, 0, 0, 0, 0, 0, 0);
    let mut lines_only = Vec::new();
    for l in src.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let c: Vec<&str> = l.splitn(7, ',').collect();
        let (y, m, leap): (i32, u8, bool) = (c[0].parse().unwrap(), c[1].parse().unwrap(), c[2] == "1");
        if !(1368..=1644).contains(&y) {
            continue;
        }
        let r = reck.entry(y).or_insert_with(|| reckon_lifa_year("大统历", y).unwrap());
        let rj = r.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0);
        let tj = zhang.year_months(y).and_then(|t| t.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0));
        let fmt = |j: Option<i64>| j.map(|j| { let (a, b, d) = civil_from_jdn(j); (gz(j), format!("{a}-{b:02}-{d:02}")) }).unwrap_or(("", String::new()));
        let (rg, rd) = fmt(rj);
        let (tg, td) = fmt(tj);
        let (ok_r, ok_t) = ((rg == c[3]) as u8, (tg == c[3]) as u8);
        if rj.is_none() || tj.is_none() {
            nomonth += 1;
        }
        n += 1;
        hr += ok_r as usize;
        ht += ok_t as usize;
        match (ok_r, ok_t) {
            (1, 0) => only_r += 1,
            (0, 1) => only_t += 1,
            (0, 0) => neither += 1,
            _ => {}
        }
        if ok_r != ok_t || ok_r == 0 {
            lines_only.push(format!("{y}-{m}{} 实录 {} | 推步 {rg} {rd} | 张表 {tg} {td} | {}", if leap { "闰" } else { "" }, c[3], c[5]));
        }
        out += &format!("{y},{m},{},{},{rg},{rd},{tg},{td},{ok_r},{ok_t},{},{},{}\n", leap as u8, c[3], c[4], c[5], c.get(6).unwrap_or(&""));
    }
    std::fs::write(root.join("data/calendar/mingshilu_check.csv"), out).unwrap();
    println!("实录月朔 {n} 条（1368—1644）：合推步 {hr}，合张表 {ht}；只合推步 {only_r}，只合张表 {only_t}，两皆不合 {neither}（其中月序在推步或张表中不存在 {nomonth}）");
    for l in lines_only {
        println!("  {l}");
    }
}
