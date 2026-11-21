//! 术不传之段的代推检验：观象历（807—821）以正元、宣明代推，会天历（1253—1270）以开禧、成天代推（候选见
//! `dingshuo::REGIME_REGISTRY`），与张培瑜历表一起，以正史本纪明书朔日（`data/calendar/benji_shuo_check.csv`，剔除
//! `benji_shuo_errata.csv` 已考讹误者与「其他」类）、本纪閏月（`data/calendar/本纪纪日/*_閏月.csv`）与实物历日（1256 年会天历）检验。
//!
//! 输出 data/calendar/lost_proxy_eval.csv（逐候选汇总）、data/calendar/lost_proxy_detail.csv（候选间或与张表不同、或本纪只合其一之条）。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example lost_proxy_eval
use lunar_core::calendar::artifacts::zhongyuan_artifact_year;
use lunar_core::calendar::dingshuo::reckon_year_any;
use lunar_core::calendar::song_early;

/// 候选推步：宋初诸历（应天等）之代推走 song_early，余走 dingshuo
fn reckon(name: &str, y: i32) -> Option<Vec<(i64, u8, bool)>> {
    if name.starts_with("应天历") { song_early::reckon_year(name, y) } else { reckon_year_any(name, y) }
}
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use std::collections::{BTreeMap, BTreeSet};

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

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

struct Seg {
    name: &'static str,
    years: (i32, i32),
    regime: &'static str,
    books: &'static [&'static str],
    candidates: &'static [&'static str],
}

const SEGS: &[Seg] = &[
    Seg {
        name: "观象历",
        years: (807, 821),
        regime: "唐",
        books: &["jiutangshu_benji", "xintangshu_benji"],
        candidates: &["正元历（代推观象）", "宣明历（代推观象）", "正元历（宣明进朔，代推观象）", "正元历（四分之三进朔，代推观象）"],
    },
    Seg {
        name: "会天历",
        years: (1253, 1270),
        regime: "宋",
        books: &["songshi_benji"],
        candidates: &["开禧历（代推会天）", "成天历（代推会天）"],
    },
    Seg { name: "钦天历", years: (957, 963), regime: "", books: &["jiuwudaishi_benji", "xinwudaishi_benji", "songshi_benji"], candidates: &["宣明历（代推钦天）", "应天历（代推钦天）"] },
    Seg { name: "奉元历", years: (1075, 1093), regime: "宋", books: &["songshi_benji"], candidates: &["崇天历（代推奉元）", "明天历（代推奉元）", "观天历（代推奉元）"] },
    Seg { name: "占天历", years: (1103, 1105), regime: "宋", books: &["songshi_benji"], candidates: &["观天历（代推占天）", "纪元历（代推占天）"] },
    Seg { name: "淳祐历", years: (1252, 1252), regime: "宋", books: &["songshi_benji"], candidates: &["开禧历（代推淳祐）"] },
    Seg { name: "本天历", years: (1277, 1279), regime: "宋", books: &["songshi_benji"], candidates: &["成天历（代推本天）"] },
];

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let errata: BTreeSet<(String, String, String)> = parse_csv(&read("data/calendar/benji_shuo_errata.csv"))
        .into_iter()
        .filter(|r| r.len() >= 3 && !r[0].starts_with('#') && r[0] != "书")
        .map(|r| (r[0].clone(), r[1].clone(), r[2].clone()))
        .collect();
    let rows = parse_csv(&read("data/calendar/benji_shuo_check.csv"));
    let head = rows.iter().find(|r| r.first().is_some_and(|c| c == "书")).unwrap().clone();
    let col = |n: &str| head.iter().position(|h| h == n).unwrap();
    let (cb, cy, cm, cr, ck, cx) = (col("书"), col("年"), col("月"), col("本纪朔 JDN"), col("类"), col("原文"));

    let mut out = String::from("段,年段,候选,月数,与张表同,闰位不同之岁,本纪朔条数,合候选,合张表,只合候选,只合张表,本纪閏月条数,閏合候选,閏合张表,实物月数,实物合候选,实物合张表,本纪纪日条数,在月合候选,在月合张表\n");
    let mut detail = String::from("段,候选,类,年,月,证据,候选,张表,合候选,合张表,原文\n");
    for seg in SEGS {
        // 张表与本纪证据
        let tab = |y: i32| zhang.year_months(y).map(|v| v.to_vec()).unwrap_or_default();
        let benji: Vec<(i32, u8, bool, i64, String, String)> = rows
            .iter()
            .filter(|r| r.len() == head.len() && seg.books.contains(&r[cb].as_str()) && r[ck] != "其他")
            .filter_map(|r| {
                let y: i32 = r[cy].parse().ok()?;
                if !(seg.years.0..=seg.years.1).contains(&y) || errata.contains(&(r[cb].clone(), r[cy].clone(), r[cm].clone())) {
                    return None;
                }
                let (leap, m) = match r[cm].strip_prefix('闰').or_else(|| r[cm].strip_prefix('閏')) {
                    Some(x) => (true, x),
                    None => (false, r[cm].as_str()),
                };
                Some((y, m.parse().ok()?, leap, r[cr].parse().ok()?, r[cb].clone(), r[cx].replace([',', '\n'], "，")))
            })
            .collect();
        // 本纪閏月：(年, 閏在某月之后)，去重
        let mut leaps: BTreeMap<i32, u8> = BTreeMap::new();
        for b in seg.books {
            let p = format!("data/calendar/本纪纪日/{b}_閏月.csv");
            let Ok(t) = std::fs::read_to_string(root.join(&p)) else { continue };
            for r in parse_csv(&t).into_iter().skip(1) {
                if r.len() < 5 {
                    continue;
                }
                let (Ok(y), Ok(m)) = (r[3].parse::<i32>(), r[4].parse::<u8>()) else { continue };
                if (seg.years.0..=seg.years.1).contains(&y) && (seg.regime.is_empty() || r[0] == seg.regime) {
                    leaps.entry(y).or_insert(m);
                }
            }
        }
        // 本纪纪日（年, 月, 闰, 日干支）：检各历所排之月是否容此干支
        let mut days: Vec<(i32, u8, bool, String, String)> = Vec::new();
        for b in seg.books {
            let Ok(t) = std::fs::read_to_string(root.join(format!("data/calendar/本纪纪日/{b}.csv"))) else { continue };
            let rr = parse_csv(&t);
            let Some(h) = rr.iter().find(|r| r.first().is_some_and(|c| c == "regime")).cloned() else { continue };
            let ci = |n: &str| h.iter().position(|x| x == n).unwrap();
            let (iy, im, il, ig, ix) = (ci("year"), ci("month"), ci("leap"), ci("ganzhi"), ci("text"));
            for r in rr.iter().filter(|r| r.len() == h.len() && (seg.regime.is_empty() || r[0] == seg.regime)) {
                let (Ok(y), Ok(m)) = (r[iy].parse::<i32>(), r[im].parse::<u8>()) else { continue };
                if (seg.years.0..=seg.years.1).contains(&y) {
                    days.push((y, m, r[il] == "1", r[ig].clone(), r[ix].chars().take(24).collect::<String>().replace(',', "，")));
                }
            }
        }
        let in_month = |all: &BTreeMap<i32, Vec<(i64, u8, bool)>>, y: i32, m: u8, l: bool, g: &str| -> bool {
            let Some(cur) = all.get(&y) else { return false };
            let Some(&(a, _, _)) = cur.iter().find(|e| e.1 == m && e.2 == l) else { return false };
            let b = (y..=y + 1).flat_map(|yy| all.get(&yy).cloned().unwrap_or_default()).map(|e| e.0).filter(|&j| j > a).min().unwrap_or(a + 30);
            (a..b).any(|j| gz(j) == g)
        };
        let ztab: BTreeMap<i32, Vec<(i64, u8, bool)>> = (seg.years.0 - 1..=seg.years.1 + 1).map(|y| (y, tab(y))).collect();
        for cand in seg.candidates {
            let (mut months, mut same, mut leap_diff) = (0usize, 0usize, Vec::new());
            let mut rk: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
            for y in seg.years.0..=seg.years.1 {
                let r = reckon(cand, y).unwrap_or_default();
                let t = tab(y);
                months += r.len();
                same += r.iter().filter(|m| t.contains(m)).count();
                let lr: Vec<_> = r.iter().filter(|m| m.2).map(|m| m.1).collect();
                let lt: Vec<_> = t.iter().filter(|m| m.2).map(|m| m.1).collect();
                if lr != lt {
                    leap_diff.push(y.to_string());
                }
                rk.insert(y, r);
            }
            let find = |v: &[(i64, u8, bool)], m: u8, l: bool| v.iter().find(|e| e.1 == m && e.2 == l).map(|e| e.0);
            let (mut n, mut hr, mut ht, mut or_, mut ot) = (0, 0, 0, 0, 0);
            for (y, m, l, rec, book, text) in &benji {
                let rj = find(&rk[y], *m, *l);
                let tj = find(&tab(*y), *m, *l);
                let (a, b) = (rj == Some(*rec), tj == Some(*rec));
                n += 1;
                hr += a as usize;
                ht += b as usize;
                or_ += (a && !b) as usize;
                ot += (b && !a) as usize;
                if a != b || rj != tj {
                    let g = |j: Option<i64>| j.map_or("—", gz);
                    detail += &format!(
                        "{},{cand},本纪朔,{y},{}{m},{} {},{},{},{},{},{text}\n",
                        seg.name,
                        if *l { "閏" } else { "" },
                        gz(*rec),
                        book.replace("_benji", ""),
                        g(rj),
                        g(tj),
                        a as u8,
                        b as u8
                    );
                }
            }
            let (mut ln, mut lr_, mut lt_) = (0, 0, 0);
            for (y, after) in &leaps {
                let rl = rk[y].iter().find(|e| e.2).map(|e| e.1);
                let tl = tab(*y).iter().find(|e| e.2).map(|e| e.1);
                ln += 1;
                lr_ += (rl == Some(*after)) as usize;
                lt_ += (tl == Some(*after)) as usize;
                if rl != tl || rl != Some(*after) {
                    let f = |x: Option<u8>| x.map_or("无".to_string(), |v| format!("閏{v}"));
                    detail += &format!(
                        "{},{cand},本纪閏月,{y},閏{after},本纪閏月在{after}月后,{},{},{},{},\n",
                        seg.name,
                        f(rl),
                        f(tl),
                        (rl == Some(*after)) as u8,
                        (tl == Some(*after)) as u8
                    );
                }
            }
            let rkx: BTreeMap<i32, Vec<(i64, u8, bool)>> = (seg.years.0 - 1..=seg.years.1 + 1)
                .map(|y| (y, if (seg.years.0..=seg.years.1).contains(&y) { rk[&y].clone() } else { tab(y) }))
                .collect();
            let (mut dn, mut dr_, mut dt_) = (0, 0, 0);
            for (y, m, l, g, text) in &days {
                let (a, b) = (in_month(&rkx, *y, *m, *l, g), in_month(&ztab, *y, *m, *l, g));
                dn += 1;
                dr_ += a as usize;
                dt_ += b as usize;
                if a != b {
                    detail += &format!("{},{cand},本纪纪日,{y},{}{m},{g},{},{},{},{},{text}\n", seg.name, if *l { "閏" } else { "" }, if a { "在月" } else { "不在月" }, if b { "在月" } else { "不在月" }, a as u8, b as u8);
                }
            }
            let (mut an, mut ar, mut at) = (0, 0, 0);
            for y in seg.years.0..=seg.years.1 {
                let Some(art) = zhongyuan_artifact_year(y) else { continue };
                for &(j, m, l) in &art.months {
                    let rj = find(&rk[&y], m, l);
                    let tj = find(&tab(y), m, l);
                    an += 1;
                    ar += (rj == Some(j)) as usize;
                    at += (tj == Some(j)) as usize;
                    if rj != Some(j) || tj != Some(j) {
                        let g = |j: Option<i64>| j.map_or("—", gz);
                        detail += &format!(
                            "{},{cand},实物,{y},{}{m},{} {},{},{},{},{},\n",
                            seg.name,
                            if l { "閏" } else { "" },
                            gz(j),
                            art.name,
                            g(rj),
                            g(tj),
                            (rj == Some(j)) as u8,
                            (tj == Some(j)) as u8
                        );
                    }
                }
            }
            println!(
                "{} {cand}：月 {months} 同张表 {same} 闰异 {:?} | 本纪朔 {n} 合候选 {hr} 张表 {ht}（只候选 {or_} 只张表 {ot}）| 閏 {ln}：候选 {lr_} 张表 {lt_} | 实物 {an}：候选 {ar} 张表 {at} | 纪日 {dn}：在月候选 {dr_} 张表 {dt_}",
                seg.name, leap_diff
            );
            out += &format!(
                "{},{}—{},{cand},{months},{same},{},{n},{hr},{ht},{or_},{ot},{ln},{lr_},{lt_},{an},{ar},{at},{dn},{dr_},{dt_}\n",
                seg.name,
                seg.years.0,
                seg.years.1,
                leap_diff.join(" ")
            );
        }
    }
    std::fs::write(root.join("data/calendar/lost_proxy_eval.csv"), out).unwrap();
    std::fs::write(root.join("data/calendar/lost_proxy_detail.csv"), detail).unwrap();
}
