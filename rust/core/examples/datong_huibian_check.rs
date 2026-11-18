//! 《国家图书馆藏明代大统历日汇编》（北京图书馆出版社 2007，6 册，大统历日 99 种）逐月核验：
//! `data/calendar/datong_huibian_months_v*.csv`（卷首月表转录：年,月,闰,大小,朔干支,册,书页,备注）与
//! 大统推步（《通轨》立成，`push_calendar::reckon_lifa_year("大统历")`）、张表（vendor/calendar/chinese_calendar/实历朔日表.csv）、
//! 《明实录》月朔（`data/calendar/mingshilu_shuo.csv`，有则比）逐月比较。
//!
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example datong_huibian_check
//! 输出 data/calendar/datong_huibian_check.csv，终端打印汇总与不合之条。
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::gregorian::civil_from_jdn;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::push_calendar::reckon_lifa_year;
use std::collections::BTreeMap;

type Months = [(i64, u8, bool)];

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

/// (朔 JDN, 朔干支, 大小)
fn find(this: &Months, next: &Months, m: u8, leap: bool) -> Option<(i64, &'static str, &'static str)> {
    let i = this.iter().position(|e| e.1 == m && e.2 == leap)?;
    let j0 = this[i].0;
    let j1 = this.get(i + 1).or(next.first())?.0;
    Some((j0, gz(j0), if j1 - j0 == 30 { "大" } else { "小" }))
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    // 实录月朔：(年,月,闰) → 干支（同月异文取第一条）
    let mut shilu: BTreeMap<(i32, u8, bool), String> = BTreeMap::new();
    if let Ok(s) = std::fs::read_to_string(root.join("data/calendar/mingshilu_shuo.csv")) {
        for l in s.lines().filter(|l| !l.starts_with('#')).skip(1) {
            let c: Vec<&str> = l.split(',').collect();
            if let (Ok(y), Ok(m)) = (c[0].parse(), c[1].parse()) {
                shilu.entry((y, m, c[2] == "1")).or_insert_with(|| c[3].to_string());
            }
        }
    }
    let mut rows = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(root.join("data/calendar")).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().into_string().unwrap()).filter(|n| n.starts_with("datong_huibian_months_v") && n.ends_with(".csv")).collect();
    files.sort();
    for f in &files {
        for l in read(&format!("data/calendar/{f}")).lines().filter(|l| !l.starts_with('#')).skip(1) {
            let c: Vec<String> = l.splitn(8, ',').map(String::from).collect();
            if c.len() >= 7 && c[0].parse::<i32>().is_ok() {
                rows.push(c);
            }
        }
    }
    rows.sort_by_key(|c| (c[0].parse::<i32>().unwrap(), c[1].parse::<u8>().unwrap_or(0), c[2].clone()));
    let mut out = String::from("年,月,闰,实物朔,实物大小,立成朔,立成大小,立成朔日期,张表朔,张表大小,实录朔,合立成,合张表,合实录,册,书页,备注\n");
    let (mut n, mut hr, mut ht, mut ns, mut hs) = (0, 0, 0, 0, 0);
    let mut diffs = Vec::new();
    let mut cache: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
    for c in &rows {
        let (y, m, leap): (i32, u8, bool) = (c[0].parse().unwrap(), c[1].parse().unwrap_or(0), c[2] == "1");
        let (dx, sg) = (c[3].as_str(), c[4].as_str());
        if m == 0 || sg.chars().count() != 2 {
            continue;
        }
        for yy in [y, y + 1] {
            cache.entry(yy).or_insert_with(|| reckon_lifa_year("大统历", yy).unwrap_or_default());
        }
        let r = find(&cache[&y], &cache[&(y + 1)], m, leap);
        let t = zhang.year_months(y).zip(zhang.year_months(y + 1)).and_then(|(a, b)| find(a, b, m, leap));
        let sl = shilu.get(&(y, m, leap)).cloned().unwrap_or_default();
        let ok = |x: Option<(i64, &str, &str)>| x.map_or(0, |x| (x.1 == sg && (dx.is_empty() || x.2 == dx)) as u8);
        let (ok_r, ok_t) = (ok(r), ok(t));
        let ok_s = if sl.is_empty() { String::new() } else { ((sl == sg) as u8).to_string() };
        n += 1;
        hr += ok_r as usize;
        ht += ok_t as usize;
        if !sl.is_empty() {
            ns += 1;
            hs += (sl == sg) as usize;
        }
        let date = r.map(|x| { let (a, b, d) = civil_from_jdn(x.0); format!("{a}-{b:02}-{d:02}") }).unwrap_or_default();
        if ok_r == 0 || ok_t == 0 || ok_s == "0" {
            diffs.push(format!("{y}-{}{m} 实物 {sg}{dx} | 立成 {} {} | 张表 {} | 实录 {sl} | {}", if leap { "闰" } else { "" }, r.map_or("", |x| x.1), r.map_or("", |x| x.2), t.map_or("".into(), |x| format!("{}{}", x.1, x.2)), c.get(7).map_or("", |s| s.as_str())));
        }
        out += &format!(
            "{y},{m},{},{sg},{dx},{},{},{date},{},{},{sl},{ok_r},{ok_t},{ok_s},{},{},{}\n",
            leap as u8,
            r.map_or("", |x| x.1),
            r.map_or("", |x| x.2),
            t.map_or("", |x| x.1),
            t.map_or("", |x| x.2),
            c[5],
            c[6],
            c.get(7).map_or("", |s| s.as_str())
        );
    }
    std::fs::write(root.join("data/calendar/datong_huibian_check.csv"), out).unwrap();
    println!("《汇编》实物 {n} 月：合立成推步 {hr}，合张表 {ht}；有实录月朔者 {ns}，合实录 {hs}");
    for d in diffs {
        println!("  {d}");
    }
}
