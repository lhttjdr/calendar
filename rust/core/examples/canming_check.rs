//! 南明、明郑大统历日核验：傅以礼《残明大统历》（`data/calendar/canming_datong_fuyili.csv`）与大统推步
//! （《通轨》立成，`dingshuo::reckon_year_any("大统历")`）、张表（同年清时宪历）逐月比较。
//!
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example canming_check
//! 输出 data/calendar/canming_check.csv，终端打印汇总与不合之条。
use lunar_core::calendar::dingshuo::reckon_year_any;
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::gregorian::civil_from_jdn;
use lunar_core::calendar::historical::ShuoriTable;

type Months = [(i64, u8, bool)];

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

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
    let src = read("data/calendar/canming_datong_fuyili.csv");
    let mut out = String::from("年,月,闰,傅表朔,傅表大小,推步朔,推步大小,推步朔日期,张表朔,张表大小,合推步,合张表\n");
    let (mut n, mut hr, mut ht) = (0, 0, 0);
    let mut diffs = Vec::new();
    for l in src.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        let (Ok(y), Ok(m)) = (c[0].parse::<i32>(), c[2].parse::<u8>()) else { continue };
        let leap = c[3] == "1";
        let (sg, dx) = (c[4], c[5]);
        if sg.chars().count() != 2 {
            continue;
        }
        let (a, b) = (reckon_year_any("大统历", y).unwrap(), reckon_year_any("大统历", y + 1).unwrap());
        let r = find(&a, &b, m, leap);
        let t = zhang.year_months(y).zip(zhang.year_months(y + 1)).and_then(|(a, b)| find(a, b, m, leap));
        let ok = |x: Option<(i64, &str, &str)>| x.map_or(0, |x| (x.1 == sg && (dx.is_empty() || x.2 == dx)) as u8);
        let (ok_r, ok_t) = (ok(r), ok(t));
        n += 1;
        hr += ok_r as usize;
        ht += ok_t as usize;
        let date = r.map(|x| { let (a, b, d) = civil_from_jdn(x.0); format!("{a}-{b:02}-{d:02}") }).unwrap_or_default();
        if ok_r == 0 {
            diffs.push(format!("{y}-{}{m} 傅表 {sg}{dx} | 推步 {}{} {date} | 张表 {}", if leap { "闰" } else { "" }, r.map_or("", |x| x.1), r.map_or("", |x| x.2), t.map_or("".into(), |x| format!("{}{}", x.1, x.2))));
        }
        out += &format!("{y},{m},{},{sg},{dx},{},{},{date},{},{},{ok_r},{ok_t}\n", leap as u8, r.map_or("", |x| x.1), r.map_or("", |x| x.2), t.map_or("", |x| x.1), t.map_or("", |x| x.2));
    }
    std::fs::write(root.join("data/calendar/canming_check.csv"), out).unwrap();
    println!("《残明大统历》{n} 月：合大统推步 {hr}，合张表（清时宪）{ht}");
    for d in diffs {
        println!("  {d}");
    }
}
