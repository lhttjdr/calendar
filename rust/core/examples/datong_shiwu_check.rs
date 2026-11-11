//! 明钦天监刊本大统历实物核验：`data/calendar/datong_shiwu_months.csv`（实物逐月朔干支、大小）与大统推步
//! （`push_calendar::reckon_lifa_year("大统历")`）、张表（vendor/calendar/chinese_calendar/实历朔日表.csv）逐月比较。
//! 备注沿用旧表同月之条（关于推步的旧注随推步重算，不沿用）。
//!
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example datong_shiwu_check
//! 输出 data/calendar/datong_shiwu_check.csv
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::push_calendar::reckon_lifa_year;
use std::collections::BTreeMap;

type Months = [(i64, u8, bool)];

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

/// (朔干支, 大小)：本月朔距次月朔 30 日为大
fn find(this: &Months, next: &Months, m: u8, leap: bool) -> Option<(&'static str, &'static str)> {
    let i = this.iter().position(|e| e.1 == m && e.2 == leap)?;
    let j0 = this[i].0;
    let j1 = this.get(i + 1).or(next.first())?.0;
    Some((gz(j0), if j1 - j0 == 30 { "大" } else { "小" }))
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let out_path = root.join("data/calendar/datong_shiwu_check.csv");
    // 旧表备注
    let mut notes: BTreeMap<(String, String, String), String> = BTreeMap::new();
    if let Ok(old) = std::fs::read_to_string(&out_path) {
        for l in old.lines().skip(1) {
            let c: Vec<&str> = l.splitn(12, ',').collect();
            if c.len() == 12 && !c[11].is_empty() && !c[11].starts_with("推步") {
                notes.insert((c[0].into(), c[1].into(), c[2].into()), c[11].into());
            }
        }
    }
    let src = read("data/calendar/datong_shiwu_months.csv");
    let mut out = String::from("年,月,闰,实物朔,实物大小,推步朔,推步大小,张表朔,张表大小,合推步,合张表,备注\n");
    let (mut n, mut hit_r, mut hit_t) = (0, 0, 0);
    for l in src.lines().skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        let (y, m, leap): (i32, u8, bool) = (c[0].parse().unwrap(), c[1].parse().unwrap(), c[2] == "1");
        let (dx, sg) = (c[3], c[4]);
        let r = find(&reckon_lifa_year("大统历", y).unwrap(), &reckon_lifa_year("大统历", y + 1).unwrap(), m, leap).unwrap();
        let t = find(zhang.year_months(y).unwrap(), zhang.year_months(y + 1).unwrap(), m, leap).unwrap();
        let ok_r = (r.0 == sg && (dx.is_empty() || r.1 == dx)) as u8;
        let ok_t = (t.0 == sg && (dx.is_empty() || t.1 == dx)) as u8;
        n += 1;
        hit_r += ok_r as usize;
        hit_t += ok_t as usize;
        let note = notes.get(&(c[0].into(), c[1].into(), c[2].into())).cloned().unwrap_or_default();
        out += &format!("{y},{m},{},{sg},{dx},{},{},{},{},{ok_r},{ok_t},{note}\n", leap as u8, r.0, r.1, t.0, t.1);
    }
    std::fs::write(&out_path, out).unwrap();
    println!("实物 {n} 月：合大统推步 {hit_r}，合张表 {hit_t}");
}
