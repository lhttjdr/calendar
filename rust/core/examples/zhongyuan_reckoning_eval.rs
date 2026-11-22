//! 中原一线按术推步逐历检验：与张培瑜历表逐月比较（朔日、闰位），并以正史本纪明书朔日（`data/calendar/benji_shuo_check.csv`）
//! 逐条检验推步与张表；秦汉初（前 221—太初元年四月）张表以前的今人历表为李忠林复原历，本纪取《汉书》本纪书朔、晦
//! （`data/calendar/hanshu_qinhan_benji.csv`，逐条结果写 data/calendar/qinhan_benji_check.csv）。结果定第 4 层逐历缺省（`lifa::reckoning_preferred_by_evidence`，见 doc/calendar/rili-sources.md）。
//!
//! 输出 data/calendar/zhongyuan_reckoning_eval.csv（逐历汇总）、data/calendar/zhongyuan_reckoning_benji.csv（推步与张表不同、
//! 或本纪只合其一之条）。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example zhongyuan_reckoning_eval
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{zhongyuan_year_months, RiliSource, ZhongyuanCalendarPeriods, ZhongyuanPreference};
use lunar_core::calendar::month_labels::{standard_month_name, MonthLabel, MonthLabelReforms};
use lunar_core::calendar::qinhan_restore;
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
    /// 已考为讹误（data/calendar/benji_shuo_errata.csv）而不计的本纪条数
    errata: usize,
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
    // 今人历表：张表；张表以前（秦汉初）为李忠林复原历
    let table = |y: i32| -> Vec<(i64, u8, bool)> {
        zhang.year_months(y).map(|t| t.to_vec()).or_else(|| qinhan_restore::year_months(y)).unwrap()
    };
    for y in -220..=1913 {
        let Some((r, RiliSource::Reckoned(name))) = zhongyuan_year_months(None, y, ZhongyuanPreference::Reckoning) else { continue };
        let t = table(y);
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
    // 已考讹误的本纪朔日：不作证据，另计
    let errata: std::collections::BTreeSet<(String, String, String)> = parse_csv(&read("data/calendar/benji_shuo_errata.csv"))
        .into_iter()
        .filter(|r| r.len() >= 3 && !r[0].starts_with('#') && r[0] != "书")
        .map(|r| (r[0].clone(), r[1].clone(), r[2].clone()))
        .collect();
    // 讹误表每条须在本纪检验总表中（防书名、年月写错）；落在取张表之段（如观象）者不进本表计分
    let all_keys: std::collections::BTreeSet<(String, String, String)> =
        rows.iter().filter(|r| r.len() == head.len()).map(|r| (r[cb].clone(), r[cy].clone(), r[cm].clone())).collect();
    let missing: Vec<_> = errata.iter().filter(|k| !all_keys.contains(*k)).collect();
    assert!(missing.is_empty(), "benji_shuo_errata.csv 有条目不在 benji_shuo_check.csv 中：{missing:?}");
    let reforms = MonthLabelReforms::from_csv(&read("data/calendar/month_label_reforms.csv")).unwrap();
    let skip_books = ["liaoshi_benji", "jinshi_benji", "weishu_benji", "beiqishu_benji", "zhoushu_benji"];
    let mut detail = String::from("历,书,年,月,本纪朔,推步朔,张表朔,本纪合推步,本纪合张表,类,原文\n");
    for r in rows.iter().filter(|r| r.len() == head.len() && r[0] != "书") {
        if skip_books.contains(&r[cb].as_str()) || ["吴", "蜀"].contains(&r[cs].as_str()) || r[ck] == "其他" {
            continue;
        }
        let (Ok(y), Ok(rec)) = (r[cy].parse::<i32>(), r[cr].parse::<i64>()) else { continue };
        // 元 1280 年以前行金重修大明历，不是中原一线（南宋）之历，其本纪纪日不检中原诸历
        if r[cs] == "元" && y < 1280 {
            continue;
        }
        let (leap, m) = match r[cm].strip_prefix('闰').or_else(|| r[cm].strip_prefix('閏')) {
            Some(x) => (true, x),
            None => (false, r[cm].as_str()),
        };
        let Ok(m) = m.parse::<u8>() else { continue };
        // 本纪记当时月名、纪年年；经月序改制表换为建寅表月（如秦汉初十月岁首：太初元年十一月 = 夏正前一年十一月）
        let regime = match r[cs].as_str() {
            "西汉" | "东汉" => "汉",
            x => x,
        };
        let label = MonthLabel { year: y, name: standard_month_name(m).to_string(), is_leap: leap };
        let (y, m, leap) = reforms.table_of(regime, &label).map_or((y, m, leap), |t| (t.lunar_year, t.month, t.is_leap));
        // 太初改历以前的《汉书》本纪另由下面秦汉初一节逐条计（hanshu_qinhan_benji.csv），此处不重计
        if r[cb] == "hanshu_benji" && y <= -104 {
            continue;
        }
        let Some(months) = reck.get(&y) else { continue };
        let name = periods.name_for_lunar_year(y);
        let fy = *first_year.iter().find(|(n, _)| **n == name).map(|(_, f)| f).unwrap();
        let rj = months.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0);
        let tj = zhang.year_months(y).and_then(|t| t.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0));
        let (hr, ht) = (rj == Some(rec), tj == Some(rec));
        let s = stats.get_mut(&(fy, name)).unwrap();
        let is_errata = errata.contains(&(r[cb].clone(), r[cy].clone(), r[cm].clone()));
        if is_errata {
            // 已考讹误：不计分，明细照列并注明
            s.errata += 1;
        } else {
            s.n += 1;
            s.hit_r += hr as usize;
            s.hit_t += ht as usize;
            s.only_r += (hr && !ht) as usize;
            s.only_t += (ht && !hr) as usize;
        }
        if hr != ht || rj != tj {
            let g = |j: Option<i64>| j.map_or("—", gz);
            let kind = if is_errata { "已考讹不计" } else { r[ck].as_str() };
            detail += &format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                name, r[cb], y, r[cm], gz(rec), g(rj), g(tj), hr as u8, ht as u8, kind, r[cx].replace([',', '\n'], "，")
            );
        }
    }
    // 秦汉初：《汉书》本纪纪日。书朔、晦者计分（晦为该月末日）；余者只判在月内否，逐条列出
    {
        let recs = parse_csv(&read("data/calendar/hanshu_qinhan_benji.csv"));
        let h = recs.iter().find(|r| r.first().is_some_and(|c| c == "卷")).unwrap().clone();
        let c = |n: &str| h.iter().position(|x| x == n).unwrap();
        let (cv, cn, cy, cm, cl, cg, cs, cx) = (c("卷"), c("纪年年"), c("夏正年"), c("月"), c("闰"), c("干支"), c("朔晦"), c("原文"));
        let mut per = String::from("卷,纪年年,夏正年,月,干支,朔晦,推步该月朔,推步合,复原该月朔,复原合,原文\n");
        // 月 (朔, 下月朔)
        let span = |ms: &dyn Fn(i32) -> Vec<(i64, u8, bool)>, y: i32, m: u8, leap: bool| -> Option<(i64, i64)> {
            let mut all: Vec<(i64, u8, bool)> = ms(y);
            all.extend(ms(y + 1));
            let i = all.iter().position(|e| e.1 == m && e.2 == leap)?;
            Some((all[i].0, all.get(i + 1)?.0))
        };
        let reck_y = |y: i32| zhongyuan_year_months(None, y, ZhongyuanPreference::Reckoning).unwrap().0;
        for r in recs.iter().filter(|r| r.len() == h.len() && r[0] != "卷" && !r[0].starts_with('#')) {
            let (y, m, leap): (i32, u8, bool) = (r[cy].parse().unwrap(), r[cm].parse().unwrap(), r[cl] == "1");
            let gi = GAN_ZHI_60.iter().position(|g| *g == r[cg]).unwrap() as i64;
            let judge = |sp: Option<(i64, i64)>| -> (String, bool) {
                let Some((a, b)) = sp else { return ("—".into(), false) };
                let d = a + (gi - (a + 49)).rem_euclid(60);
                let ok = d < b && match r[cs].as_str() { "朔" => d == a, "晦" => d == b - 1, _ => true };
                (gz(a).to_string(), ok)
            };
            let (gr, okr) = judge(span(&reck_y, y, m, leap));
            let (gt, okt) = judge(span(&table, y, m, leap));
            per += &format!("{},{},{},{}{},{},{},{},{},{},{},{}\n", r[cv], r[cn], y, if leap { "闰" } else { "" }, m, r[cg], r[cs], gr, okr as u8, gt, okt as u8, r[cx]);
            if r[cs].is_empty() {
                continue;
            }
            let name = periods.name_for_lunar_year(y);
            let fy = *first_year.iter().find(|(n, _)| **n == name).map(|(_, f)| f).unwrap();
            let s = stats.get_mut(&(fy, name)).unwrap();
            s.n += 1;
            s.hit_r += okr as usize;
            s.hit_t += okt as usize;
            s.only_r += (okr && !okt) as usize;
            s.only_t += (okt && !okr) as usize;
            if okr != okt {
                detail += &format!("{},hanshu_benji,{},{}{},{}{},{},{},{},{},书{},{}\n", name, y, if leap { "闰" } else { "" }, m, r[cg], r[cs], gr, gt, okr as u8, okt as u8, r[cs], r[cx]);
            }
        }
        std::fs::write(root.join("data/calendar/qinhan_benji_check.csv"), per).unwrap();
    }
    let mut out = String::from("历,起,止,月数,与张表同,闰位不同之岁,本纪条数,本纪合推步,本纪合张表,只合推步,只合张表,已考讹不计\n");
    for ((_, name), s) in &stats {
        let ld: Vec<String> = s.leap_diff.iter().map(|y| y.to_string()).collect();
        out += &format!("{name},{},{},{},{},{},{},{},{},{},{},{}\n", s.years.0, s.years.1, s.months, s.same, ld.join(" "), s.n, s.hit_r, s.hit_t, s.only_r, s.only_t, s.errata);
        println!("{name:<20} {}—{} 月 {} 同 {} 闰异 {:?} | 本纪 {} 推步 {} 张表 {} （只推步 {} 只张表 {}）", s.years.0, s.years.1, s.months, s.same, s.leap_diff, s.n, s.hit_r, s.hit_t, s.only_r, s.only_t);
    }
    std::fs::write(root.join("data/calendar/zhongyuan_reckoning_eval.csv"), out).unwrap();
    std::fs::write(root.join("data/calendar/zhongyuan_reckoning_benji.csv"), detail).unwrap();
}
