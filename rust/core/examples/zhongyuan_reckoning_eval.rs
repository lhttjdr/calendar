//! 中原一线按术推步逐历检验：与张培瑜历表逐月比较（朔日、闰位），并以正史本纪明书朔日（`data/calendar/benji_shuo_check.csv`）
//! 逐条检验推步与张表。结果定第 4 层逐历缺省（`lifa::reckoning_preferred_by_evidence`，见 doc/calendar/rili-sources.md）。
//!
//! 输出 data/calendar/zhongyuan_reckoning_eval.csv（逐历汇总）、data/calendar/zhongyuan_reckoning_benji.csv（推步与张表不同、
//! 或本纪只合其一之条）。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example zhongyuan_reckoning_eval
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{zhongyuan_year_months, RiliSource, ZhongyuanCalendarPeriods, ZhongyuanPreference};
use std::collections::BTreeMap;

/// 带引号与换行的 CSV
fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let (mut rows, mut row, mut cell, mut quoted) = (Vec::new(), Vec::new(), String::new(), false);
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match (c, quoted) {
            ('"', true) if it.peek() == Some(&'"') => {
                cell.push('"');
                it.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => row.push(std::mem::take(&mut cell)),
            ('\n', false) => {
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
            }
            ('\r', false) => {}
            _ => cell.push(c),
        }
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        rows.push(row);
    }
    rows
}

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

#[derive(Default)]
struct Stat {
    years: (i32, i32),
    months: usize,
    same: usize,
    leap_diff: Vec<i32>,
    n: usize,
    hit_r: usize,
    hit_t: usize,
    only_r: usize,
    only_t: usize,
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let periods = ZhongyuanCalendarPeriods::embedded();
    let mut stats: BTreeMap<(i32, &'static str), Stat> = BTreeMap::new();
    let mut first_year: BTreeMap<&'static str, i32> = BTreeMap::new();
    // 逐年：推步与张表
    let mut reck: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
    for y in -102..=1913 {
        let Some((r, RiliSource::Reckoned(name))) = zhongyuan_year_months(None, y, ZhongyuanPreference::Reckoning) else { continue };
        let t = zhang.year_months(y).unwrap();
        let fy = *first_year.entry(name).or_insert(y);
        let s = stats.entry((fy, name)).or_default();
        if s.months == 0 {
            s.years.0 = y;
        }
        s.years.1 = y;
        s.months += r.len();
        s.same += r.iter().filter(|m| t.contains(m)).count();
        let lr: Vec<_> = r.iter().filter(|m| m.2).map(|m| m.1).collect();
        let lt: Vec<_> = t.iter().filter(|m| m.2).map(|m| m.1).collect();
        if lr != lt {
            s.leap_diff.push(y);
        }
        reck.insert(y, r);
    }
    // 本纪明书朔日
    let rows = parse_csv(&read("data/calendar/benji_shuo_check.csv"));
    let head = rows.iter().find(|r| r.first().is_some_and(|c| c == "书")).unwrap().clone();
    let col = |n: &str| head.iter().position(|h| h == n).unwrap();
    let (cb, cy, cm, cr, ct, ck, cx, cs) = (col("书"), col("年"), col("月"), col("本纪朔 JDN"), col("张表朔 JDN"), col("类"), col("原文"), col("政权"));
    let skip_books = ["liaoshi_benji", "jinshi_benji", "weishu_benji", "beiqishu_benji", "zhoushu_benji"];
    let mut detail = String::from("历,书,年,月,本纪朔,推步朔,张表朔,本纪合推步,本纪合张表,类,原文\n");
    for r in rows.iter().filter(|r| r.len() == head.len() && r[0] != "书") {
        if skip_books.contains(&r[cb].as_str()) || ["吴", "蜀"].contains(&r[cs].as_str()) || r[ck] == "其他" {
            continue;
        }
        let (Ok(y), Ok(rec)) = (r[cy].parse::<i32>(), r[cr].parse::<i64>()) else { continue };
        let Some(months) = reck.get(&y) else { continue };
        let (leap, m) = match r[cm].strip_prefix('闰').or_else(|| r[cm].strip_prefix('閏')) {
            Some(x) => (true, x),
            None => (false, r[cm].as_str()),
        };
        let Ok(m) = m.parse::<u8>() else { continue };
        let name = periods.name_for_lunar_year(y);
        let fy = *first_year.iter().find(|(n, _)| **n == name).map(|(_, f)| f).unwrap();
        let rj = months.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0);
        let tj = zhang.year_months(y).and_then(|t| t.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0));
        let (hr, ht) = (rj == Some(rec), tj == Some(rec));
        let s = stats.get_mut(&(fy, name)).unwrap();
        s.n += 1;
        s.hit_r += hr as usize;
        s.hit_t += ht as usize;
        s.only_r += (hr && !ht) as usize;
        s.only_t += (ht && !hr) as usize;
        if hr != ht || rj != tj {
            let g = |j: Option<i64>| j.map_or("—", gz);
            detail += &format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                name, r[cb], y, r[cm], gz(rec), g(rj), g(tj), hr as u8, ht as u8, r[ck], r[cx].replace([',', '\n'], "，")
            );
        }
    }
    let mut out = String::from("历,起,止,月数,与张表同,闰位不同之岁,本纪条数,本纪合推步,本纪合张表,只合推步,只合张表\n");
    for ((_, name), s) in &stats {
        let ld: Vec<String> = s.leap_diff.iter().map(|y| y.to_string()).collect();
        out += &format!("{name},{},{},{},{},{},{},{},{},{},{}\n", s.years.0, s.years.1, s.months, s.same, ld.join(" "), s.n, s.hit_r, s.hit_t, s.only_r, s.only_t);
        println!("{name:<20} {}—{} 月 {} 同 {} 闰异 {:?} | 本纪 {} 推步 {} 张表 {} （只推步 {} 只张表 {}）", s.years.0, s.years.1, s.months, s.same, s.leap_diff, s.n, s.hit_r, s.hit_t, s.only_r, s.only_t);
    }
    std::fs::write(root.join("data/calendar/zhongyuan_reckoning_eval.csv"), out).unwrap();
    std::fs::write(root.join("data/calendar/zhongyuan_reckoning_benji.csv"), detail).unwrap();
}
