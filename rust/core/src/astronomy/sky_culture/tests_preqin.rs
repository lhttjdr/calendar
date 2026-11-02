//! 先秦文献日躔与昏旦中星（data/sky_culture/pre_qin_sky_positions.csv：《礼记·月令》36 条、《尚书·尧典》四仲中星、《尚书·胤征》）按岁差逐年代检验。
//!
//! - 太阳：取所注节气的太阳黄经 λ（当日平春分点），赤经 tan α = cos ε tan λ；宿按陈卓体系距星当日平赤经划界。
//! - 昏、旦：日入后、日出前 2.5 刻（36 分钟，9.02° 恒星时），太阳上缘、折射 34′，纬度 34.5°（洛阳、咸阳之间）；子午圈赤经为太阳赤经加减（日入时角 + 9.02°）。
//! - 中星为宿者：子午圈落在该宿（距星至下一宿距星）之内；为星或星官者：其成员之一与子午圈赤经差在 ±7.5° 以内。
//! - 年代：前 3000 至公元元年（天文纪年 −2999 至 1）每 10 年；平位置（空间运动 + Vondrák 2011 长期岁差）。
//! 《月令》另以各月节（中气前 15°）复算，看结论是否依赖取月中还是月首。

use crate::astronomy::apparent::{right_ascension_declination, stars_mean_equatorial, ApparentPipelineOptions};
use crate::astronomy::frame::precession::{mean_obliquity_for, PrecessionModel};
use crate::astronomy::sky_culture::chinese::{load_lunar_mansion_systems_from_repo, MANSIONS};
use crate::astronomy::stars::{load_hipparcos_bright_from_repo, CatalogStar};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};
use std::collections::HashMap;

fn wrap(x: f64) -> f64 {
    x.rem_euclid(360.0)
}
fn wrap180(x: f64) -> f64 {
    (x + 540.0).rem_euclid(360.0) - 180.0
}

struct Rec {
    source: String,
    month: String,
    kind: String,
    target: String,
    hips: Vec<u32>,
    qi: f64,
    text: String,
}

#[test]
#[ignore]
fn pre_qin_sky_positions_epoch_scan() {
    let lines = crate::repo::read_lines("data/sky_culture/pre_qin_sky_positions.csv").unwrap();
    let recs: Vec<Rec> = lines
        .iter()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            Rec {
                source: f[0].into(),
                month: f[1].into(),
                kind: f[2].into(),
                target: f[3].into(),
                hips: f[4].split_whitespace().map(|x| x.parse().unwrap()).collect(),
                qi: f[5].parse().unwrap(),
                text: f[6].into(),
            }
        })
        .collect();
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let det = load_lunar_mansion_systems_from_repo().unwrap().into_iter().find(|s| s.id == "chenzhuo").unwrap().determinatives;
    let mut hips: Vec<u32> = det.to_vec();
    hips.extend(recs.iter().flat_map(|r| r.hips.iter().copied()));
    hips.sort();
    hips.dedup();
    let stars: Vec<CatalogStar> = hips.iter().map(|h| *cat.by_hip(*h).unwrap_or_else(|| panic!("星表无 HIP {h}"))).collect();
    let opts = ApparentPipelineOptions::default();
    let phi = 34.5f64.to_radians();
    let h_twilight = 2.5 * 14.4 / 60.0 * 15.041;
    let years: Vec<i64> = (0..=300).map(|k| -2999 + 10 * k).collect();
    // 结果：[记录][年代] = (是否成立, 说明)
    let mut ok: Vec<Vec<bool>> = vec![vec![false; years.len()]; recs.len()];
    let mut ok_jie: Vec<Vec<bool>> = vec![vec![false; years.len()]; recs.len()];
    let mut note_at: HashMap<(usize, i64), String> = HashMap::new();
    for (yi, &y) in years.iter().enumerate() {
        let jd = 2451545.0 + (y as f64 - 2000.0) * 365.25;
        let pos = stars_mean_equatorial(&stars, TimePoint::new(TimeScale::TT, real(jd)), &opts);
        let ra: HashMap<u32, f64> = hips
            .iter()
            .zip(&pos)
            .map(|(h, p)| (*h, right_ascension_declination(p).0.rad().as_f64().to_degrees().rem_euclid(360.0)))
            .collect();
        let eps = mean_obliquity_for(real((jd - 2451545.0) / 36525.0), PrecessionModel::Vondrak2011).rad().as_f64();
        let lodge_of = |x: f64| -> Option<usize> {
            (0..28).find(|&i| {
                let a = ra[&det[i]];
                let b = ra[&det[(i + 1) % 28]];
                let w = wrap(b - a);
                w < 180.0 && wrap(x - a) < w
            })
        };
        for (ri, r) in recs.iter().enumerate() {
            for (variant, shift) in [(0usize, 0.0), (1, -15.0)] {
                if variant == 1 && r.source != "月令" {
                    continue;
                }
                let lam = (r.qi + shift).to_radians();
                let a_s = (lam.sin() * eps.cos()).atan2(lam.cos()).to_degrees().rem_euclid(360.0);
                let d_s = (eps.sin() * lam.sin()).asin();
                let h0 = ((-(0.5667f64 + 0.2667).to_radians().sin() - phi.sin() * d_s.sin()) / (phi.cos() * d_s.cos())).acos().to_degrees();
                let x = match r.kind.as_str() {
                    "sun" => a_s,
                    "dusk" => wrap(a_s + h0 + h_twilight),
                    _ => wrap(a_s - h0 - h_twilight),
                };
                let (good, note) = if r.hips.is_empty() {
                    let ti = MANSIONS.iter().position(|m| *m == r.target).unwrap();
                    // 胤征：季秋全月（寒露 195° 至立冬 225°）中任一日日在房即算成立
                    let good = if r.source == "胤征" {
                        (195..225).any(|l| {
                            let l = (l as f64).to_radians();
                            lodge_of((l.sin() * eps.cos()).atan2(l.cos()).to_degrees().rem_euclid(360.0)) == Some(ti)
                        })
                    } else {
                        lodge_of(x) == Some(ti)
                    };
                    let lo = lodge_of(x).map(|i| MANSIONS[i]).unwrap_or("?");
                    (good, format!("{lo}{:.1}", wrap(x - lodge_of(x).map(|i| ra[&det[i]]).unwrap_or(x))))
                } else {
                    let best = r.hips.iter().map(|h| wrap180(ra[h] - x)).fold(f64::NAN, |m: f64, v| if m.is_nan() || v.abs() < m.abs() { v } else { m });
                    (best.abs() <= 7.5, format!("{:+.1}°", best))
                };
                if variant == 0 {
                    ok[ri][yi] = good;
                    if y % 500 == 1 || y == 1 {
                        note_at.insert((ri, y), note);
                    }
                } else {
                    ok_jie[ri][yi] = good;
                }
            }
        }
    }
    // 逐条：成立的年代区间
    let spans = |v: &Vec<bool>| -> String {
        let mut out = vec![];
        let mut start: Option<i64> = None;
        for (i, &g) in v.iter().enumerate() {
            match (g, start) {
                (true, None) => start = Some(years[i]),
                (false, Some(s)) => {
                    out.push(format!("{s}—{}", years[i - 1]));
                    start = None
                }
                _ => {}
            }
        }
        if let Some(s) = start {
            out.push(format!("{s}—{}", years[years.len() - 1]));
        }
        if out.is_empty() { "无".into() } else { out.join(" ") }
    };
    let mut tab = crate::test_util::CheckTable::new(
        "pre_qin_sky_positions",
        &["来源", "月", "类", "原文", "成立年代（天文纪年，中气）", "成立年代（节，仅月令）", "前1499推算", "前499推算", "公元1推算"],
    );
    tab.meta("先秦文献日躔与昏旦中星按岁差逐年代检验：前 3000 至公元元年每 10 年；年代为天文纪年（−499 即前 500 年）；推算栏为该年代太阳或子午圈所在宿及入宿度（°），或星与子午圈赤经差");
    tab.meta("昏旦为日入后、日出前 2.5 刻，纬度 34.5°；星或星官以与子午圈差 ±7.5° 内为中；胤征按季秋全月任一日日在房");
    for (ri, r) in recs.iter().enumerate() {
        let kind = match r.kind.as_str() { "sun" => "日在", "dusk" => "昏中", _ => "旦中" };
        tab.row(vec![
            r.source.clone(), r.month.clone(), kind.into(), r.text.clone(), spans(&ok[ri]),
            if r.source == "月令" { spans(&ok_jie[ri]) } else { String::new() },
            note_at.get(&(ri, -1499)).cloned().unwrap_or_default(), note_at.get(&(ri, -499)).cloned().unwrap_or_default(), note_at.get(&(ri, 1)).cloned().unwrap_or_default(),
        ]);
    }
    tab.write();
    // 汇总：各组在每个年代成立的条数，取最多者
    let mut sum = crate::test_util::CheckTable::new("pre_qin_sky_positions_summary", &["组", "条数", "最多成立条数", "取得最多的年代", "前1499成立", "前499成立", "前349成立"]);
    sum.meta("各组成立条数随年代的极大（天文纪年）；月令另列按节（月首）的结果");
    let groups: Vec<(&str, Box<dyn Fn(&Rec) -> bool>, bool)> = vec![
        ("月令·日在", Box::new(|r: &Rec| r.source == "月令" && r.kind == "sun"), false),
        ("月令·昏中", Box::new(|r: &Rec| r.source == "月令" && r.kind == "dusk"), false),
        ("月令·旦中", Box::new(|r: &Rec| r.source == "月令" && r.kind == "dawn"), false),
        ("月令·全部", Box::new(|r: &Rec| r.source == "月令"), false),
        ("月令·日在（按节）", Box::new(|r: &Rec| r.source == "月令" && r.kind == "sun"), true),
        ("月令·昏中（按节）", Box::new(|r: &Rec| r.source == "月令" && r.kind == "dusk"), true),
        ("月令·旦中（按节）", Box::new(|r: &Rec| r.source == "月令" && r.kind == "dawn"), true),
        ("月令·全部（按节）", Box::new(|r: &Rec| r.source == "月令"), true),
        ("尧典", Box::new(|r: &Rec| r.source == "尧典"), false),
        ("胤征", Box::new(|r: &Rec| r.source == "胤征"), false),
    ];
    for (name, f, jie) in &groups {
        let idx: Vec<usize> = (0..recs.len()).filter(|&i| f(&recs[i])).collect();
        let table = if *jie { &ok_jie } else { &ok };
        let cnt: Vec<usize> = (0..years.len()).map(|yi| idx.iter().filter(|&&i| table[i][yi]).count()).collect();
        let mx = *cnt.iter().max().unwrap();
        let at: Vec<i64> = (0..years.len()).filter(|&yi| cnt[yi] == mx).map(|yi| years[yi]).collect();
        let at_s = format!("{}—{}", at.first().unwrap(), at.last().unwrap());
        let c = |y: i64| cnt[years.iter().position(|&v| v == y).unwrap()];
        println!("{name}：{} 条，最多 {mx} 条成立于 {at_s}（前1499 {}，前499 {}，前349 {}）", idx.len(), c(-1499), c(-499), c(-349));
        sum.row(vec![name.to_string(), idx.len().to_string(), mx.to_string(), at_s, c(-1499).to_string(), c(-499).to_string(), c(-349).to_string()]);
    }
    sum.write();
}
