//! 与 NASA 五千年日月食典（vendor/nasa_eclipse/）对照：极大食力学时、类型、γ、食分。
//! 默认每 `NASA_ECLIPSE_STRIDE`（缺省 20）条抽一条以控制耗时；设为 1 可全量比对。
//! TwoFloat 下每次日月食约 30 次完整视位置求值，debug 构建很慢，建议 `cargo test --release`。

use super::*;
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::math::real::{real, RealOps};

/// 儒略日（正午为整数）：1582-10-15 前为儒略历（NASA 日食典约定），年份为天文纪年。
pub(super) fn jd(y: i64, m: i64, d: i64, hms: &str) -> f64 {
    let a = (14 - m) / 12;
    let yy = y + 4800 - a;
    let mm = m + 12 * a - 3;
    let jdn = if (y, m, d) >= (1582, 10, 15) {
        d + (153 * mm + 2) / 5 + 365 * yy + yy.div_euclid(4) - yy.div_euclid(100) + yy.div_euclid(400) - 32045
    } else {
        d + (153 * mm + 2) / 5 + 365 * yy + yy.div_euclid(4) - 32083
    };
    let p: Vec<f64> = hms.split(':').map(|x| x.parse().unwrap()).collect();
    jdn as f64 - 0.5 + (p[0] + p[1] / 60.0 + p[2] / 3600.0) / 24.0
}

pub(super) fn month(s: &str) -> i64 {
    ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].iter().position(|m| *m == s).unwrap() as i64 + 1
}

/// 目录行：去标签后以 5 位编号开头的行，按空白分词。
pub(super) fn catalog_rows(file: &str) -> Vec<Vec<String>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/nasa_eclipse").join(file);
    let html = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    let mut text = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => text.push(c),
            _ => {}
        }
    }
    text.lines()
        .map(|l| l.split_whitespace().map(String::from).collect::<Vec<_>>())
        .filter(|t| t.len() >= 12 && t[0].len() == 5 && t[0].chars().all(|c| c.is_ascii_digit()) && t[4].contains(':'))
        .collect()
}

fn stride() -> usize {
    std::env::var("NASA_ECLIPSE_STRIDE").ok().and_then(|s| s.parse().ok()).unwrap_or(20)
}

pub(super) fn load() -> Option<(crate::astronomy::ephemeris::Vsop87, crate::astronomy::ephemeris::Elpmpp02Data)> {
    let vsop = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).ok()?;
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).ok()?;
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    Some((vsop, elp))
}

/// 打印并检查差值：|平均| ≤ `mean_tol`，最大|差| ≤ `max_tol`。
fn check(name: &str, v: &[f64], mean_tol: f64, max_tol: f64) {
    let max = v.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    println!("  {name}: n={} 平均 {mean:+.4} 最大|差| {max:.4}", v.len());
    assert!(mean.abs() <= mean_tol && max <= max_tol, "{name}：平均 {mean} / 最大 {max} 超出容差 {mean_tol} / {max_tol}");
}

/// 对照容差（时刻单位秒）。
/// - 2001–2100：NASA 表时刻取整到 1 s，月食极大附近角距变化平缓，时刻差放宽到 3 s；γ、食分取到 1e-4，容差 5e-4 / 3e-3。
/// - 前 199–前 100：NASA 用 ELP-2000/82（Espenak & Meeus 2006），本实现用拟合 DE406 的 ELPMPP02。极大食 TT 系统偏差实测约 −18 s
///   （日、月食一致，按月日相对运动约 0.5″/s 折合月亮黄经约 9″，属两套月历表在两千年前的差异，γ、食分无对应偏差），故时刻容差取 |平均| ≤ 30 s、最大 ≤ 40 s；γ、食分实测差 ≤ 3e-3，容差 1e-3 / 5e-3。
struct Tolerance {
    time_mean: f64,
    time_max: f64,
    gamma: f64,
    magnitude: f64,
}

fn tolerance(file: &str) -> Tolerance {
    if file.contains("2001") {
        Tolerance { time_mean: 1.0, time_max: 3.0, gamma: 5e-4, magnitude: 3e-3 }
    } else {
        Tolerance { time_mean: 30.0, time_max: 40.0, gamma: 1e-3, magnitude: 5e-3 }
    }
}

/// 插值与直接计算的差：窗口 ±2 日内随机取点，日、月位置误差均 < 1 m。
#[test]
fn interpolation_matches_direct() {
    let Some((vsop, elp)) = load() else {
        println!("skipped: 历表数据缺失");
        return;
    };
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for center in [1650000.5_f64, 2460000.5] {
        let w = eclipse_window(&eph, real(center));
        let mut worst = 0.0_f64;
        let mut worst_m = 0.0_f64;
        for k in 0..9 {
            let t = real(center - 1.9 + 0.47 * k as f64);
            let (s1, m1) = eph.sun_moon_m(t);
            let (s2, m2) = w.sun_moon_m(t);
            for i in 0..3 {
                worst = worst.max((s1[i] - s2[i]).as_f64().abs()).max((m1[i] - m2[i]).as_f64().abs());
                worst_m = worst_m.max((m1[i] - m2[i]).as_f64().abs());
            }
        }
        println!("JD {center}：插值最大误差 {worst:.3e} m，其中月亮 {worst_m:.3e} m");
        // f64 构建：太阳约 6 m（1.5e11 m 处约 4e-11 rad，属舍入水平），月亮约 2 cm，容差分开定；double-double 两者都远小于 1 m
        let (tol, tol_m) = if cfg!(feature = "real-f64") { (20.0, 0.1) } else { (1.0, 1.0) };
        assert!(worst < tol && worst_m < tol_m, "插值误差 {worst} m，月亮 {worst_m} m");
    }
}

#[test]
fn solar_eclipses_vs_nasa_canon() {
    let Some((vsop, elp)) = load() else {
        println!("skipped: 历表数据缺失");
        return;
    };
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for file in ["SE-0199--0100.html", "SE2001-2100.html"] {
        let rows = catalog_rows(file);
        assert!(rows.len() > 200, "{file} 解析行数 {}", rows.len());
        let (mut dt, mut dg, mut dm) = (vec![], vec![], vec![]);
        let mut type_mismatch = vec![];
        let mut tab = crate::test_util::CheckTable::new(
            if file.contains("2001") { "nasa_solar_2001_2100" } else { "nasa_solar_-0199_-0100" },
            &["NASA 日期(TD)", "NASA 极大 TD", "NASA 类型", "本实现类型", "NASA γ", "γ 差", "NASA 食分", "食分差", "极大时刻差 s"],
        );
        tab.meta(format!("NASA 五千年日食典 {file} 逐次对照（vendor/nasa_eclipse），本实现 VSOP87（DE406 改正）+ ELPMPP02；抽样 1/{}", stride()));
        // 逐次求食彼此独立，并行求（保序），再按原序统计、写表
        let sampled: Vec<&Vec<String>> = rows.iter().step_by(stride()).collect();
        let found = crate::test_util::par_map(&sampled, |_, r| {
            let t_nasa = jd(r[1].parse().unwrap(), month(&r[2]), r[3].parse().unwrap(), &r[4]);
            solar_eclipse_global(&eph, real(t_nasa)).unwrap_or_else(|| panic!("{file} {:?}：NASA 有日食，本实现未找到", &r[1..5]))
        });
        for (r, e) in sampled.into_iter().zip(found) {
            let t_nasa = jd(r[1].parse().unwrap(), month(&r[2]), r[3].parse().unwrap(), &r[4]);
            let (kind_nasa, gamma_nasa, mag_nasa): (char, f64, f64) = (r[8].chars().next().unwrap(), r[10].parse().unwrap(), r[11].parse().unwrap());
            dt.push((e.greatest_tt.as_f64() - t_nasa) * 86400.0);
            dg.push(e.gamma.as_f64() - gamma_nasa);
            dm.push(e.magnitude.as_f64() - mag_nasa);
            let k = match e.kind {
                SolarEclipseKind::Partial => 'P',
                SolarEclipseKind::Annular => 'A',
                SolarEclipseKind::Total => 'T',
                SolarEclipseKind::Hybrid => 'H',
            };
            if k != kind_nasa {
                type_mismatch.push(format!("{} {} {} NASA {kind_nasa} 本实现 {k} γ={gamma_nasa}", r[1], r[2], r[3]));
            }
            tab.row(vec![
                format!("{} {} {}", r[1], r[2], r[3]),
                r[4].clone(),
                kind_nasa.to_string(),
                k.to_string(),
                format!("{gamma_nasa:.4}"),
                format!("{:+.4}", dg.last().unwrap()),
                format!("{mag_nasa:.4}"),
                format!("{:+.4}", dm.last().unwrap()),
                format!("{:+.1}", dt.last().unwrap()),
            ]);
        }
        tab.write();
        println!("{file}（抽样 1/{}）", stride());
        let tol = tolerance(file);
        check("极大食时刻差 (s)", &dt, tol.time_mean, tol.time_max);
        check("γ 差", &dg, tol.gamma, tol.gamma);
        check("食分差", &dm, tol.magnitude, tol.magnitude);
        assert!(type_mismatch.is_empty(), "类型不一致：{type_mismatch:?}");
    }
}

#[test]
fn lunar_eclipses_vs_nasa_canon() {
    let Some((vsop, elp)) = load() else {
        println!("skipped: 历表数据缺失");
        return;
    };
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for file in ["LE-0199--0100.html", "LE2001-2100.html"] {
        let rows = catalog_rows(file);
        assert!(rows.len() > 200, "{file} 解析行数 {}", rows.len());
        let (mut dt, mut dg, mut dp, mut du) = (vec![], vec![], vec![], vec![]);
        let mut type_mismatch = vec![];
        let mut tab = crate::test_util::CheckTable::new(
            if file.contains("2001") { "nasa_lunar_2001_2100" } else { "nasa_lunar_-0199_-0100" },
            &["NASA 日期(TD)", "NASA 极大 TD", "NASA 类型", "本实现类型", "NASA γ", "γ 差", "NASA 半影食分", "半影食分差", "NASA 本影食分", "本影食分差", "极大时刻差 s"],
        );
        tab.meta(format!("NASA 五千年月食典 {file} 逐次对照（vendor/nasa_eclipse），地影按 Danjon 规则；抽样 1/{}", stride()));
        let sampled: Vec<&Vec<String>> = rows.iter().step_by(stride()).collect();
        let found = crate::test_util::par_map(&sampled, |_, r| {
            let t_nasa = jd(r[1].parse().unwrap(), month(&r[2]), r[3].parse().unwrap(), &r[4]);
            lunar_eclipse(&eph, real(t_nasa)).unwrap_or_else(|| panic!("{file} {:?}：NASA 有月食，本实现未找到", &r[1..5]))
        });
        for (r, e) in sampled.into_iter().zip(found) {
            let t_nasa = jd(r[1].parse().unwrap(), month(&r[2]), r[3].parse().unwrap(), &r[4]);
            let kind_nasa = r[8].chars().next().unwrap();
            let (gamma_nasa, pen_nasa, umb_nasa): (f64, f64, f64) = (r[10].parse().unwrap(), r[11].parse().unwrap(), r[12].parse().unwrap());
            dt.push((e.greatest_tt.as_f64() - t_nasa) * 86400.0);
            dg.push(e.gamma.as_f64() - gamma_nasa);
            dp.push(e.penumbral_magnitude.as_f64() - pen_nasa);
            du.push(e.umbral_magnitude.as_f64() - umb_nasa);
            let k = match e.kind {
                LunarEclipseKind::Penumbral => 'N',
                LunarEclipseKind::Partial => 'P',
                LunarEclipseKind::Total => 'T',
            };
            if k != kind_nasa {
                type_mismatch.push(format!("{} {} {} NASA {kind_nasa} 本实现 {k} 本影食分 {umb_nasa}", r[1], r[2], r[3]));
            }
            tab.row(vec![
                format!("{} {} {}", r[1], r[2], r[3]),
                r[4].clone(),
                kind_nasa.to_string(),
                k.to_string(),
                format!("{gamma_nasa:.4}"),
                format!("{:+.4}", dg.last().unwrap()),
                format!("{pen_nasa:.4}"),
                format!("{:+.4}", dp.last().unwrap()),
                format!("{umb_nasa:.4}"),
                format!("{:+.4}", du.last().unwrap()),
                format!("{:+.1}", dt.last().unwrap()),
            ]);
        }
        tab.write();
        println!("{file}（抽样 1/{}）", stride());
        let tol = tolerance(file);
        check("极大食时刻差 (s)", &dt, tol.time_mean, tol.time_max);
        check("γ 差", &dg, tol.gamma, tol.gamma);
        check("半影食分差", &dp, tol.magnitude, tol.magnitude);
        check("本影食分差", &du, tol.magnitude, tol.magnitude);
        assert!(type_mismatch.is_empty(), "类型不一致：{type_mismatch:?}");
    }
}
