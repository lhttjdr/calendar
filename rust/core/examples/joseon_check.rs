//! 《朝鲜王朝实录》逐月朔日（`data/calendar/joseon_shuo.csv`，`scripts/calendar_data/joseon_shuo.py` 由维基文库录文抽出）
//! 与大统推步（`push_calendar::reckon_lifa_year("大统历")`；1645 年以后 `dingshuo::reckon_year_any`）、张表
//! （vendor/calendar/chinese_calendar/实历朔日表.csv）、《明实录》月朔（data/calendar/mingshilu_shuo.csv）逐月比较。
//! 推步定朔时刻照《大统暦日通轨》立成（`dingshuo::li_by_name("大统历")`，北京时）；汉阳时＝北京时＋42 分
//! （汉阳东经约 126°59′，北京约 116°25′，差 10.6°）。
//!
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example joseon_check
//! 输出 data/calendar/joseon_vs_ming_check.csv（逐月），终端打印同日率、不同之月定朔时刻分布、闰月异同。
use lunar_core::calendar::dingshuo::{li_by_name, reckon_year_any};
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::gregorian::civil_from_jdn;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::push_calendar::reckon_lifa_year;
use num_traits::ToPrimitive;
use std::collections::{BTreeMap, BTreeSet};

/// 汉阳较北京晚（日）：42 分
const HANYANG: f64 = 42.0 / 1440.0;

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

fn date(j: i64) -> String {
    let (a, b, d) = civil_from_jdn(j);
    format!("{a}-{b:02}-{d:02}")
}

fn hm(j: i64, frac: f64) -> String {
    let (j, frac) = if frac >= 1.0 { (j + 1, frac - 1.0) } else { (j, frac) };
    let mins = (frac * 1440.0).floor() as i64;
    format!("{} {:02}:{:02}", date(j), mins / 60, mins % 60)
}

fn label(m: u8, leap: bool) -> String {
    format!("{}{m}", if leap { "闰" } else { "" })
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let mut shilu: BTreeMap<(i32, u8, bool), String> = BTreeMap::new();
    for l in read("data/calendar/mingshilu_shuo.csv").lines().filter(|l| !l.starts_with('#')).skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        shilu.entry((c[0].parse().unwrap(), c[1].parse().unwrap(), c[2] == "1")).or_insert_with(|| c[3].to_string());
    }
    let reckon = |y: i32| if y <= 1644 { reckon_lifa_year("大统历", y) } else { reckon_year_any("大统历", y) }.unwrap_or_default();
    // 推步定朔时刻：JDN → 日内分数（北京时）
    let li = li_by_name("大统历").unwrap();
    let f = li.f() as f64;
    let mut moment: BTreeMap<i64, f64> = BTreeMap::new();
    for y in 1388..1655 {
        for m in 0..14 {
            let t = li.month(y, m).1.to_f64().unwrap();
            let day = (t / f).floor();
            moment.insert(day as i64 + li.off(), t / f - day);
        }
    }

    let src = read("data/calendar/joseon_shuo.csv");
    let mut out = String::from("朝鲜年,公历年,月,闰,朝鲜朔,依据,朝鲜日期,推步朔,推步日期,推步定朔（北京时）,推步定朔（汉阳时）,张表朔,实录朔,合推步,合张表,合实录,差日（朝鲜−推步）,页面,备注\n");
    let mut reck: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
    // 统计
    let mut stat: BTreeMap<&str, [usize; 4]> = BTreeMap::new(); // 依据 → [月数, 合推步, 合张表, 有推步之月]（限 1644 年以前）
    let mut stat_p: BTreeMap<&str, [usize; 4]> = BTreeMap::new(); // 时段 → [月数, 合推步, 直书月数, 直书合推步]
    let mut sl = [0usize; 3]; // 有实录之月、合朝鲜、合推步
    let mut diffs: Vec<(String, i64, f64, String)> = Vec::new();
    let mut joseon_leap: BTreeMap<i32, Vec<(u8, i64, String)>> = BTreeMap::new();
    let mut seen_years = BTreeSet::new();
    let mut jdns: BTreeMap<i64, (i32, u8, bool, String)> = BTreeMap::new();
    for l in src.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        let (y, m, leap): (i32, u8, bool) = (c[2].parse().unwrap(), c[3].parse().unwrap(), c[4] == "1");
        let (sg, how, page, note) = (c[5], c[7], c[8], c.get(9).copied().unwrap_or(""));
        seen_years.insert(y);
        let r = reck.entry(y).or_insert_with(|| reckon(y)).clone();
        let rj = r.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0);
        let tj = zhang.year_months(y).and_then(|t| t.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0));
        // 朝鲜朔日 JDN：取与推步（或张表）同月最近的同干支之日；闰月推步无此月者取前月之次月
        let anchor = rj.or(tj).or_else(|| r.iter().find(|e| e.1 == m && !e.2).map(|e| e.0 + 30));
        let Some(a) = anchor else {
            eprintln!("无锚：{l}");
            continue;
        };
        let jj = (a - 30..=a + 30).filter(|&j| gz(j) == sg).min_by_key(|&j| (j - a).abs()).unwrap();
        jdns.insert(jj, (y, m, leap, how.to_string()));
        if leap {
            joseon_leap.entry(y).or_default().push((m, jj, how.to_string()));
        }
        let rg = rj.map(gz).unwrap_or("");
        let tg = tj.map(gz).unwrap_or("");
        let sh = shilu.get(&(y, m, leap)).cloned().unwrap_or_default();
        // 最近的推步定朔
        let (&mk, &mf) = moment.range(jj - 20..=jj + 20).min_by_key(|(k, _)| (*k - jj).abs()).unwrap();
        if (mk - jj).abs() > 3 {
            eprintln!("！距推步朔 {} 日：{l}", jj - mk);
        }
        let period = match y {
            ..=1443 => "1392—1443",
            1444..=1599 => "1444—1599",
            1600..=1644 => "1600—1644",
            _ => "1645—1649（明亡后，大统续推；张表为清时宪）",
        };
        let sp = stat_p.entry(period).or_default();
        sp[0] += 1;
        sp[1] += (rg == sg) as usize;
        sp[2] += (how == "直书") as usize;
        sp[3] += (how == "直书" && rg == sg) as usize;
        if y <= 1644 {
            let s = stat.entry(if how == "直书" { "直书" } else { "反推" }).or_default();
            s[0] += 1;
            s[3] += rj.is_some() as usize;
            s[1] += (rg == sg) as usize;
            s[2] += (tg == sg) as usize;
        }
        if !sh.is_empty() && y <= 1644 {
            sl[0] += 1;
            sl[1] += (sh == sg) as usize;
            sl[2] += (sh == rg) as usize;
        }
        let d = rj.map(|j| jj - j);
        if rg != sg && y <= 1644 {
            diffs.push((format!("{y}-{}", label(m, leap)), d.unwrap_or(99), mf, format!("{how} 朝鲜 {sg} 推步 {rg} 张表 {tg} 实录 {sh}")));
        }
        let ok = |a: &str| if a.is_empty() { String::new() } else { ((a == sg) as u8).to_string() };
        out += &format!(
            "{},{y},{m},{},{sg},{how},{},{rg},{},{},{},{tg},{sh},{},{},{},{},{page},{note}\n",
            c[0],
            leap as u8,
            date(jj),
            rj.map(date).unwrap_or_default(),
            hm(mk, mf),
            hm(mk, mf + HANYANG),
            ok(rg),
            ok(tg),
            ok(&sh),
            d.map(|d| d.to_string()).unwrap_or_default()
        );
    }
    std::fs::write(root.join("data/calendar/joseon_vs_ming_check.csv"), out).unwrap();

    println!("== 同日率（1392—1644）");
    for (k, s) in &stat {
        println!("  {k}：{} 月（推步有同序之月 {}），合推步 {}（{:.2}%），合张表 {}", s[0], s[3], s[1], 100.0 * s[1] as f64 / s[0] as f64, s[2]);
    }
    println!("== 分时段（全部 / 直书）");
    for (k, s) in &stat_p {
        println!("  {k}：{} 月合推步 {}；直书 {} 月合推步 {}", s[0], s[1], s[2], s[3]);
    }
    println!("  有《明实录》月朔之月 {}：实录合朝鲜 {}，实录合推步 {}", sl[0], sl[1], sl[2]);

    println!("== 不合推步之月（1392—1644）：差日、推步定朔北京时");
    let mut hist: BTreeMap<String, usize> = BTreeMap::new();
    for (k, d, fr, s) in &diffs {
        let h = fr * 24.0;
        println!("  {k} 差 {d:+} 定朔 {:02}:{:02}（汉阳 {:.2} 时） {s}", h as i64, ((h - h.floor()) * 60.0) as i64, h + 0.7);
        let bin = if *d == 1 && h >= 24.0 - 0.7 { "差+1，定朔 23:18 后（里差可解）".to_string() } else { format!("差{d:+}，定朔 {:02} 时", h as i64) };
        *hist.entry(bin).or_default() += 1;
    }
    for (k, v) in &hist {
        println!("    {k}：{v}");
    }
    // 里差检验：推步定朔在北京 23:18—24:00 之月（汉阳已过子夜）
    println!("== 推步定朔在北京 23:18 以后之月（若加里差，汉阳应进一日）");
    let (mut n_late, mut n_plus) = (0, 0);
    for (&j, &fr) in &moment {
        if fr * 24.0 < 24.0 - 0.7 {
            continue;
        }
        let hit = jdns.get(&j).map(|e| ("同日", e)).or_else(|| jdns.get(&(j + 1)).map(|e| ("进一日", e)));
        if let Some((tag, (y, m, leap, how))) = hit {
            if *y > 1644 {
                continue;
            }
            n_late += 1;
            n_plus += (tag == "进一日") as usize;
            println!("  {y}-{} {how} 推步 {} {} → 朝鲜 {tag}", label(*m, *leap), gz(j), hm(j, fr));
        }
    }
    println!("  共 {n_late} 月，朝鲜进一日 {n_plus}");

    println!("== 闰月异同（朝鲜有闰月记录、或该年月数与推步相同者）");
    let mut per_year: BTreeMap<i32, usize> = BTreeMap::new();
    for (y, ..) in jdns.values() {
        *per_year.entry(*y).or_default() += 1;
    }
    let (mut cmp_years, mut same_years) = (0, 0);
    for &y in &seen_years {
        if y > 1644 {
            continue;
        }
        let full = reck.get(&y).is_some_and(|r| r.len() == per_year[&y]);
        if !full && !joseon_leap.contains_key(&y) {
            continue;
        }
        cmp_years += 1;
        let rl = reck.get(&y).and_then(|r| r.iter().find(|e| e.2)).map(|e| e.1);
        let tl = zhang.year_months(y).and_then(|t| t.iter().find(|e| e.2)).map(|e| e.1);
        let jl = joseon_leap.get(&y).map(|v| v[0].0);
        same_years += (jl == rl) as usize;
        if jl != rl || jl != tl {
            println!("  {y}：朝鲜 {:?} 推步 {:?} 张表 {:?}", jl, rl, tl);
        }
    }
    println!("  可比之年 {cmp_years}，闰月同推步 {same_years}");
    let nleap = seen_years.iter().filter(|&&y| y <= 1644 && reck.get(&y).is_some_and(|r| r.iter().any(|e| e.2))).count();
    println!("  推步有闰之年 {nleap}；朝鲜有闰月记录之年 {}", joseon_leap.keys().filter(|&&y| y <= 1644).count());
}
