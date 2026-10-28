//! 月历表的有效潮汐加速度 ṅ：ELPMPP02（DE406 改正版）与 JPL DE406、DE441 的月亮视黄经差 Δλ(T) 的长期二次项。
//!
//! 平黄经的潮汐部分为 ṅ/2·T²（T 儒略世纪），故 Δλ ≈ a + bT + (Δṅ/2)T² + 周期项；在前 2900 年至 2900 年取 3000 个随机时刻
//! 最小二乘拟合多项式，Δṅ = 2c（二次项系数 c）。用于检验 ΔT 潮汐修正所用的 ṅ（`delta_t::lunar_tidal_acceleration`）；
//! 有 DE431（MS2021 ΔT 所配历表，`vendor/jpl/de431/`）时另拟合 ELPMPP02 − DE431 的 Δλ(T)，据 δΔT = −Δλ/n_月 给出 ΔT 配套修正。

use super::*;
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::ephemeris::De406Kernel;
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};

/// 最小二乘拟合 y = Σ_{k≤deg} c_k x^k，返回 (系数, 残差 RMS)。
fn fit_poly(xs: &[f64], ys: &[f64], deg: usize) -> (Vec<f64>, f64) {
    let n = deg + 1;
    let mut m = vec![vec![0.0_f64; n]; n];
    let mut v = vec![0.0_f64; n];
    for (&x, &y) in xs.iter().zip(ys) {
        let p: Vec<f64> = (0..n).map(|k| x.powi(k as i32)).collect();
        for i in 0..n {
            v[i] += p[i] * y;
            for j in 0..n {
                m[i][j] += p[i] * p[j];
            }
        }
    }
    for i in 0..n {
        for r in (i + 1)..n {
            let f = m[r][i] / m[i][i];
            for c in i..n {
                m[r][c] -= f * m[i][c];
            }
            v[r] -= f * v[i];
        }
    }
    let mut s = vec![0.0_f64; n];
    for i in (0..n).rev() {
        s[i] = (v[i] - (i + 1..n).map(|j| m[i][j] * s[j]).sum::<f64>()) / m[i][i];
    }
    let eval = |x: f64| s.iter().enumerate().map(|(k, c)| c * x.powi(k as i32)).sum::<f64>();
    let rms = (xs.iter().zip(ys).map(|(&x, &y)| (y - eval(x)).powi(2)).sum::<f64>() / xs.len() as f64).sqrt();
    (s, rms)
}

#[test]
fn elpmpp02_effective_ndot_vs_de406_de441() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let k406 = De406Kernel::open(&bsp).unwrap();
    let k441 = De406Kernel::open(crate::repo::repo_root().join(crate::repo::paths::DE441_BSP)).ok();
    let k431 = De406Kernel::open(crate::repo::repo_root().join(crate::repo::paths::DE431_BSP)).ok();
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
    let opts = ApparentPipelineOptions::pipeline_default();
    let wrap = |d: f64| ((d + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI).to_degrees() * 3600.0;
    // 确定性伪随机（线性同余），避开整日
    let mut seed: u64 = 20261026;
    let mut rnd = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    let (jd0, jd1) = (2451545.0 - 4900.0 * 365.25, 2451545.0 + 900.0 * 365.25);
    let (mut ts, mut d406, mut d441, mut d441_406, mut d431) = (vec![], vec![], vec![], vec![], vec![]);
    for _ in 0..3000 {
        let jd = jd0 + (jd1 - jd0) * rnd();
        let t = TimePoint::new(TimeScale::TT, real(jd));
        let e = moon_apparent_ecliptic_longitude_with_options(&elp, t, &opts).rad().as_f64();
        let a = moon_apparent_ecliptic_longitude_de406_with_options(&k406, t, &opts).rad().as_f64();
        ts.push((jd - 2451545.0) / 36525.0);
        d406.push(wrap(a - e));
        if let Some(k) = &k441 {
            let b = moon_apparent_ecliptic_longitude_de406_with_options(k, t, &opts).rad().as_f64();
            d441.push(wrap(b - e));
            d441_406.push(wrap(b - a));
        }
        if let Some(k) = &k431 {
            // 本仓库月历表相对 MS2021 所配历表：ELPMPP02 − DE431
            let b = moon_apparent_ecliptic_longitude_de406_with_options(k, t, &opts).rad().as_f64();
            d431.push(wrap(e - b));
        }
    }
    let report = |name: &str, ys: &[f64]| {
        let (c2, rms2) = fit_poly(&ts, ys, 2);
        let (c3, rms3) = fit_poly(&ts, ys, 3);
        println!(
            "{name}：二次 [{:+.3}, {:+.4}, {:+.5}]（RMS {rms2:.2}″，Δṅ = {:+.4}″/cy²）；三次 [{:+.3}, {:+.4}, {:+.5}, {:+.6}]（RMS {rms3:.2}″）",
            c2[0], c2[1], c2[2], 2.0 * c2[2], c3[0], c3[1], c3[2], c3[3]
        );
        2.0 * c2[2]
    };
    let n406 = report("DE406 − ELPMPP02", &d406);
    if k441.is_some() {
        report("DE441 − ELPMPP02", &d441);
        report("DE441 − DE406", &d441_406);
    }
    if k431.is_some() {
        report("ELPMPP02 − DE431", &d431);
        let (c, _) = fit_poly(&ts, &d431, 2);
        for year in [-700.0, -200.0, 0.0, 500.0, 1000.0, 1500.0, 1900.0] {
            let tc = (year - 2000.0) / 100.0;
            let dl = c[0] + c[1] * tc + c[2] * tc * tc;
            // δΔT = −Δλ / n_月（n_月 = 1732559343.736″/cy = 0.549016″/s）
            println!("  {year} 年：Δλ = {dl:+.2}″ → δΔT = {:+.1} s（现行 ṅ 二次修正 {:+.1} s）", -dl / 0.549016,
                crate::astronomy::time::delta_t::tidal_correction(real(year), real(-25.857), real(-25.82)).seconds().as_f64());
        }
    }
    // ELPMPP02 的 DE406 改正应使二者 ṅ 基本一致
    assert!(n406.abs() < 0.05, "ELPMPP02 与 DE406 的 ṅ 差 {n406}″/cy²");
}
