//! 日月食表与日月食图的批量检验（对 NASA 五千年日月食典，vendor/nasa_eclipse/）：
//! - 按年列表与目录逐条对应，无漏无多；
//! - 对目录中日月食批量生成地图，查崩溃、NaN 与结构异常。
//! 耗时较长，默认忽略；地图抽样间隔 `ECLIPSE_MAP_STRIDE`（缺省 1）。

use super::tests_nasa::{catalog_rows, jd, load, month};
use super::*;
use crate::math::real::{real, Real, RealOps};
use crate::quantity::duration::Duration;

/// 目录对列表：(目录行数, 漏, 多)。匹配容差 1 日（古代两套月历表差数十秒，同一次食不会差到 1 日）。
fn compare(name: &str, cat: &[(f64, String)], ours: &[f64]) {
    let missing: Vec<&String> = cat.iter().filter(|(t, _)| !ours.iter().any(|o| (o - t).abs() < 1.0)).map(|(_, s)| s).collect();
    let extra: Vec<f64> = ours.iter().copied().filter(|o| !cat.iter().any(|(t, _)| (o - t).abs() < 1.0)).collect();
    println!("{name}：目录 {} 次，本实现 {} 次，漏 {} 多 {}", cat.len(), ours.len(), missing.len(), extra.len());
    for m in &missing {
        println!("  漏：{m}");
    }
    for e in &extra {
        println!("  多：JD {e:.3}（{}）", crate::test_util::jd_to_calendar(*e));
    }
    assert!(missing.is_empty() && extra.is_empty(), "{name} 年表与目录不符");
}

fn catalog(file: &str) -> (Vec<(f64, String)>, f64, f64) {
    let rows = catalog_rows(file);
    let v: Vec<(f64, String)> = rows.iter().map(|r| (jd(r[1].parse().unwrap(), month(&r[2]), r[3].parse().unwrap(), &r[4]), r[1..=4].join(" ") + " " + &r[8] + " γ=" + &r[10])).collect();
    let y0: i32 = rows.first().unwrap()[1].parse().unwrap();
    let y1: i32 = rows.last().unwrap()[1].parse().unwrap();
    (v, jd(y0 as i64, 1, 1, "0:0:0"), jd(y1 as i64 + 1, 1, 1, "0:0:0"))
}

/// 把时段切成约 5 年一段，并行列出（模拟网页的逐段查询，也检验段界不重不漏）。
fn split(a: f64, b: f64) -> Vec<(f64, f64)> {
    let w = 5.0 * 365.25;
    let n = ((b - a) / w).ceil() as usize;
    (0..n).map(|k| (a + w * k as f64, (a + w * (k + 1) as f64).min(b))).collect()
}

#[test]
#[ignore]
fn eclipse_lists_match_nasa_catalog() {
    let (vsop, elp) = load().expect("历表");
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for file in ["SE-0299--0200.html", "SE-0199--0100.html", "SE-0099-0000.html", "SE0001-0100.html", "SE0101-0200.html", "SE0201-0300.html", "SE2001-2100.html"] {
        let (cat, a, b) = catalog(file);
        let ours: Vec<f64> = crate::test_util::par_map(&split(a, b), |_, s| solar_eclipses_between(&eph, real(s.0), real(s.1)).iter().map(|e| e.greatest_tt.as_f64()).collect::<Vec<_>>()).concat();
        compare(file, &cat, &ours);
    }
    for file in ["LE-0199--0100.html", "LE0001-0100.html", "LE0101-0200.html", "LE2001-2100.html"] {
        let (cat, a, b) = catalog(file);
        let ours: Vec<f64> = crate::test_util::par_map(&split(a, b), |_, s| lunar_eclipses_between(&eph, real(s.0), real(s.1)).iter().map(|e| e.greatest_tt.as_f64()).collect::<Vec<_>>()).concat();
        compare(file, &cat, &ours);
    }
}

fn stride() -> usize {
    std::env::var("ECLIPSE_MAP_STRIDE").ok().and_then(|s| s.parse().ok()).unwrap_or(1)
}

fn bad(p: &MapPoint) -> bool {
    !(p.longitude.rad().as_f64().is_finite() && p.latitude.rad().as_f64().is_finite() && p.jd_tt.as_f64().is_finite()) || p.latitude.rad().as_f64().abs() > 1.5708
}

fn chord_km(a: &MapPoint, b: &MapPoint) -> f64 {
    let v = |p: &MapPoint| {
        let (l, f) = (p.longitude.rad().as_f64(), p.latitude.rad().as_f64());
        [f.cos() * l.cos(), f.cos() * l.sin(), f.sin()]
    };
    let (x, y) = (v(a), v(b));
    6378.0 * (0..3).map(|i| (x[i] - y[i]).powi(2)).sum::<f64>().sqrt()
}

#[test]
#[ignore]
fn solar_maps_sweep() {
    let (vsop, elp) = load().expect("历表");
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for (file, dt) in [("SE2001-2100.html", 70.0), ("SE-0199--0100.html", 12500.0)] {
        let rows = catalog_rows(file);
        let picked: Vec<&Vec<String>> = rows.iter().step_by(stride()).collect();
        let results = crate::test_util::par_map(&picked, |_, r| {
            let t = jd(r[1].parse().unwrap(), month(&r[2]), r[3].parse().unwrap(), &r[4]);
            let label = r[1..=4].join(" ") + " " + &r[8] + " γ=" + &r[10];
            let win = eclipse_window(&eph, real(t));
            let opts = SolarEclipseMapOptions::default();
            let m = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| solar_eclipse_map(&win, real(t), Duration::in_seconds(real(dt)), &opts)));
            let mut issues: Vec<String> = vec![];
            let mut gap = 0.0_f64;
            match m {
                Err(_) => issues.push("崩溃".into()),
                Ok(None) => issues.push("地图为空".into()),
                Ok(Some(m)) => {
                    let all = m.central_line.iter().chain(&m.umbra_north).chain(&m.umbra_south).chain(&m.penumbra_limits).chain(&m.rise_set);
                    if all.flatten().any(bad) {
                        issues.push("NaN 或越界".into());
                    }
                    if m.penumbra_limits.is_empty() {
                        issues.push("无半影界".into());
                    }
                    if m.rise_set.is_empty() {
                        issues.push("无带食出没界线".into());
                    }
                    if m.global.central && m.central_line.is_empty() {
                        issues.push("中心食无中心线".into());
                    }
                    if !m.global.central && !m.central_line.is_empty() {
                        issues.push("非中心食有中心线".into());
                    }
                    if m.global.central && (m.umbra_north.is_empty() && m.umbra_south.is_empty()) {
                        issues.push("中心食无本影界".into());
                    }
                    for l in &m.penumbra_limits {
                        for end in [&l[0], l.last().unwrap()] {
                            let d = m.rise_set.iter().flatten().map(|q| chord_km(end, q)).fold(f64::INFINITY, f64::min);
                            gap = gap.max(d);
                        }
                    }
                }
            }
            (label, issues, gap)
        });
        let mut gaps: Vec<f64> = results.iter().map(|r| r.2).collect();
        gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n_bad = results.iter().filter(|r| !r.1.is_empty()).count();
        println!("{file}：{} 次，有问题 {n_bad}；半影界端点到带食出没界线（取顶点）中位 {:.0} km，最大 {:.0} km", results.len(), gaps[gaps.len() / 2], gaps[gaps.len() - 1]);
        for (l, i, _) in results.iter().filter(|r| !r.1.is_empty()) {
            println!("  {l}：{}", i.join("、"));
        }
        for (l, _, g) in results.iter().filter(|r| r.2 > 500.0).take(15) {
            println!("  端点偏离 {g:.0} km：{l}");
        }
    }
}

#[test]
#[ignore]
fn lunar_maps_sweep() {
    let (vsop, elp) = load().expect("历表");
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for (file, dt) in [("LE2001-2100.html", 70.0), ("LE-0199--0100.html", 12500.0)] {
        let rows = catalog_rows(file);
        let results = crate::test_util::par_map(&rows.iter().collect::<Vec<_>>(), |_, r| {
            let t = jd(r[1].parse().unwrap(), month(&r[2]), r[3].parse().unwrap(), &r[4]);
            let label = r[1..=4].join(" ") + " " + &r[8];
            let win = eclipse_window(&eph, real(t));
            let m = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lunar_eclipse_map(&win, real(t), Duration::in_seconds(real(dt)), 90)));
            let mut issues: Vec<String> = vec![];
            match m {
                Err(_) => issues.push("崩溃".into()),
                Ok(None) => issues.push("地图为空".into()),
                Ok(Some(m)) => {
                    let want = match r[8].chars().next().unwrap() {
                        'T' => 7,
                        'P' => 5,
                        _ => 3,
                    };
                    if m.curves.len() != want {
                        issues.push(format!("食相 {} 个，应 {want}", m.curves.len()));
                    }
                    if m.curves.iter().flat_map(|c| c.horizon.iter().chain(std::iter::once(&c.sublunar))).any(bad) {
                        issues.push("NaN 或越界".into());
                    }
                    if m.curves.windows(2).any(|w| w[1].jd_tt <= w[0].jd_tt) {
                        issues.push("食相时序错".into());
                    }
                }
            }
            (label, issues)
        });
        let n_bad = results.iter().filter(|r| !r.1.is_empty()).count();
        println!("{file}：{} 次，有问题 {n_bad}", results.len());
        for (l, i) in results.iter().filter(|r| !r.1.is_empty()).take(20) {
            println!("  {l}：{}", i.join("、"));
        }
        assert_eq!(n_bad, 0);
    }
}

/// 界线内容反验：每次日食在本影界、半影界上取点，沿法向两侧各挪一段，用独立的站心算法看是否一侧有食（全/环）、一侧无；
/// 食分等值线上（太阳高度 > 2°）看站心食分。按目录类型分类统计，并列出不符者。
#[test]
#[ignore]
fn solar_map_content_sweep() {
    use crate::astronomy::geodesy::Observer;
    use crate::quantity::angle::PlaneAngle;
    use crate::quantity::length::Length;
    use crate::quantity::unit::LengthUnit;
    use std::collections::BTreeMap;
    let (vsop, elp) = load().expect("历表");
    let eph = EclipseEphemeris::new(&vsop, &elp);
    let observer = |lon: f64, lat: f64| Observer::new(PlaneAngle::from_deg(real(lon)), PlaneAngle::from_deg(real(lat)), Length::from_value(real(0.0), LengthUnit::Meter));
    let deg = |p: &MapPoint| (p.longitude.rad().as_f64().to_degrees(), p.latitude.rad().as_f64().to_degrees());
    let normal_pair = |line: &[MapPoint], i: usize, km: f64| {
        let (a, b) = if i + 1 < line.len() { (deg(&line[i]), deg(&line[i + 1])) } else { (deg(&line[i - 1]), deg(&line[i])) };
        let p = deg(&line[i]);
        let c = p.1.to_radians().cos().max(0.02);
        let dlon = (b.0 - a.0 + 540.0).rem_euclid(360.0) - 180.0;
        let (e, n) = (dlon * c, b.1 - a.1);
        let len = (e * e + n * n).sqrt().max(1e-12);
        let (ue, un) = (-n / len, e / len);
        let d = km / 111.2;
        ((p.0 + ue * d / c, p.1 + un * d), (p.0 - ue * d / c, p.1 - un * d))
    };
    for (file, dt) in [("SE2001-2100.html", 70.0), ("SE-0199--0100.html", 12500.0)] {
        let rows = catalog_rows(file);
        let picked: Vec<&Vec<String>> = rows.iter().step_by(stride()).collect();
        let results = crate::test_util::par_map(&picked, |_, r| {
            let t = jd(r[1].parse().unwrap(), month(&r[2]), r[3].parse().unwrap(), &r[4]);
            let delta = Duration::in_seconds(real(dt));
            let win = eclipse_window(&eph, real(t));
            let m = solar_eclipse_map(&win, real(t), delta, &SolarEclipseMapOptions::default()).unwrap();
            let local = |p: (f64, f64)| solar_eclipse_local_in(&win, &observer(p.0, p.1), real(t), delta);
            let (mut n_u, mut bad_u, mut n_p, mut bad_p, mut n_m) = (0, vec![], 0, vec![], 0);
            let mut narrow: Vec<f64> = vec![];
            let mut worst_m = 0.0_f64;
            let umbra: Vec<&Polyline> = m.umbra_north.iter().chain(&m.umbra_south).filter(|l| l.len() >= 10).collect();
            for l in umbra {
                for i in [l.len() / 4, l.len() / 2, 3 * l.len() / 4] {
                    let (a, b) = normal_pair(l, i, 3.0);
                    let has = |p| local(p).is_some_and(|e| e.second_contact.is_some());
                    n_u += 1;
                    if has(a) == has(b) {
                        // 全环食带可窄到几公里，3 km 会跨出带外：缩小步长复核，并量当地带宽（到对侧界线的距离）
                        let (a2, b2) = normal_pair(l, i, 0.2);
                        let other: Vec<&MapPoint> = m.umbra_north.iter().chain(&m.umbra_south).filter(|o| !std::ptr::eq(*o, l)).flatten().collect();
                        let width = other.iter().map(|q| chord_km(&l[i], q)).fold(f64::INFINITY, f64::min);
                        if has(a2) == has(b2) {
                            // 全环食由环转全处带宽趋于零（实测此类点带宽 ≤ 0.21 km），比步长还窄者不作核对
                            if width > 1.0 {
                                bad_u.push(deg(&l[i]));
                            } else {
                                narrow.push(width);
                            }
                        } else {
                            narrow.push(width);
                        }
                    }
                }
            }
            for l in m.penumbra_limits.iter().filter(|l| l.len() >= 10) {
                for i in [l.len() / 4, l.len() / 2, 3 * l.len() / 4] {
                    let (a, b) = normal_pair(l, i, 10.0);
                    n_p += 1;
                    if local(a).is_some() == local(b).is_some() {
                        bad_p.push(deg(&l[i]));
                    }
                }
            }
            for c in &m.magnitude_contours {
                let pts = c.lines.concat();
                for p in pts.iter().step_by((pts.len() / 4).max(1)) {
                    if let Some(e) = local(deg(p)) {
                        if e.sun_altitude_greatest.rad().as_f64().to_degrees() > 2.0 {
                            n_m += 1;
                            worst_m = worst_m.max((e.magnitude.as_f64() - c.level.as_f64()).abs());
                        }
                    }
                }
            }
            (r[1..=4].join(" "), r[8].clone(), n_u, bad_u, n_p, bad_p, n_m, worst_m, (m.global.magnitude.as_f64(), r[10].clone()), narrow)
        });
        let mut stat: BTreeMap<String, (usize, usize, usize, usize, usize, f64, usize)> = BTreeMap::new();
        for x in &results {
            let s = stat.entry(x.1.clone()).or_default();
            s.0 += x.2;
            s.1 += x.3.len();
            s.2 += x.4;
            s.3 += x.5.len();
            s.4 += x.6;
            s.5 = s.5.max(x.7);
            s.6 += 1;
        }
        println!("{file}（抽样 1/{}）", stride());
        for (k, s) in &stat {
            println!("  类型 {k}（{} 次）：本影界 {}/{} 点不符；半影界 {}/{} 点不符；等值线 {} 点最大偏差 {:.4}", s.6, s.1, s.0, s.3, s.2, s.4, s.5);
        }
        let nar: Vec<f64> = results.iter().flat_map(|x| x.9.iter().copied()).collect();
        println!("  窄带（带宽不足以两侧各挪 3 km）{} 点，对侧界线最近距离最大 {:.1} km", nar.len(), nar.iter().copied().fold(0.0, f64::max));
        for x in results.iter().filter(|x| !x.3.is_empty() || !x.5.is_empty()).take(25) {
            println!("  不符 {} {} γ={} 食分 {:.4}：本影 {:?} 半影 {:?}", x.0, x.1, x.8 .1, x.8 .0, x.3, x.5);
        }
        let n_bad: usize = results.iter().map(|x| x.3.len() + x.5.len()).sum();
        assert_eq!(n_bad, 0, "{file} 界线与站心算法不符");
        assert!(results.iter().all(|x| x.7 < 0.002), "{file} 等值线偏差过大");
    }
}


/// 初筛（平升交角距）不漏食：前 3000—3000 年逐个朔望，对 |sin F| 在 0.45—0.70 的（阈值两侧，更远的不可能有食）实际搜索，
/// 阈值以外不得有食；同时报告有食者 |sin F| 的最大值，看余量。
#[test]
#[ignore]
fn node_screen_never_drops_an_eclipse() {
    use super::eclipse_list::{abs_sin_mean_f, NODE_LIMIT_SIN_F};
    use crate::astronomy::aspects::approximate_new_moon_jd;
    // 2000 年起算的朔望序号，约 ±3000 年
    let chunks: Vec<i32> = (-62000..12400).step_by(200).collect();
    let (vsop, elp) = load().expect("历表");
    let eph = EclipseEphemeris::new(&vsop, &elp);
    let scan = |offset: f64, hit: &(dyn Fn(Real) -> bool + Sync)| -> (usize, f64) {
        let r = crate::test_util::par_map(&chunks, |_, &c| {
            let (mut tried, mut worst) = (0usize, 0.0_f64);
            for n in c..c + 200 {
                let g = approximate_new_moon_jd(n).as_f64() + offset;
                let s = abs_sin_mean_f(g);
                if s > 0.45 && s <= 0.70 {
                    tried += 1;
                    if hit(real(g)) {
                        worst = worst.max(s);
                    }
                }
            }
            (tried, worst)
        });
        (r.iter().map(|x| x.0).sum(), r.iter().map(|x| x.1).fold(0.0, f64::max))
    };
    let (ns, ws) = scan(0.0, &|g| solar_eclipse_global_in(&eclipse_window(&eph, g), g).is_some());
    let (nl, wl) = scan(29.530588853 / 2.0, &|g| lunar_eclipse_in(&eclipse_window(&eph, g), g).is_some());
    println!("日食：搜索 {ns} 个朔，有食者 |sin F| 最大 {ws:.4}；月食：搜索 {nl} 个望，最大 {wl:.4}；阈值 {NODE_LIMIT_SIN_F}");
    assert!(ws <= NODE_LIMIT_SIN_F && wl <= NODE_LIMIT_SIN_F);
}
