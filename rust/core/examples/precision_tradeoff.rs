//! 精度与速度权衡实验（见 doc/engineering/project-and-implementation.md §11.2.2）：
//! 级数逐项三角函数用 double-double（现行）与「double-double 约化相位 + f64 sin/cos、double-double 求和」两种做法，
//! 比较 ELPMPP02（月球 V、U、R 周期与泊松项）与 VSOP87（地球 L、B、R）的结果差与耗时；另打印若干时刻的月球、地球位置，
//! 供与 f64 后端（`--no-default-features --features real-f64`）的同一程序输出对比。
//!
//! 用法（rust/ 目录）：
//!   cargo run --release -p lunar-core --example precision_tradeoff
//!   cargo run --release -p lunar-core --no-default-features --features real-f64 --example precision_tradeoff
use lunar_core::astronomy::ephemeris::elpmpp02::{load_all, position_velocity, Elpmpp02Correction, Elpmpp02Term};
use lunar_core::astronomy::ephemeris::vsop87::Vsop87Amplitude;
use lunar_core::astronomy::ephemeris::load_earth_vsop87_from_repo;
use lunar_core::astronomy::time::{TimePoint, TimeScale};
use lunar_core::math::real::{real, Real, RealOps};
use std::time::Instant;

/// 相位约化到 [−π, π)（double-double），再取 f64 sin/cos。
fn sin_cos_mixed(y: Real) -> (Real, Real) {
    let two_pi = Real::two_pi();
    let k = (y / two_pi + real(0.5)).floor();
    let r = (y - k * two_pi).as_f64();
    let (s, c) = r.sin_cos();
    (real(s), real(c))
}

fn elp_sum(terms: &[Elpmpp02Term], alpha_t: &[Real; 5], mixed: bool) -> Real {
    let mut sum = real(0.0);
    for t in terms {
        let y = (0..t.fi.len().min(5)).fold(real(0.0), |s, i| s + t.fi[i] * alpha_t[i]);
        let sy = if mixed { sin_cos_mixed(y).0 } else { y.sin_cos().0 };
        let x = if t.alpha > 0 { t.ci.arcsec() * alpha_t[t.alpha as usize] } else { t.ci.arcsec() };
        sum = sum + x * sy;
    }
    sum
}

fn main() {
    let elp = load_all(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
    let earth = load_earth_vsop87_from_repo().unwrap();
    let jds: Vec<f64> = (0..=60).map(|i| 625_000.5 + i as f64 * 36_525.0).collect(); // 前 3000—公元 3000，每百年
    let groups: [(&str, &[Elpmpp02Term], &[Elpmpp02Term], f64); 3] = [
        ("ELPMPP02 V（经度，″）", &elp.period_v, &elp.poisson_v, 1.0),
        ("ELPMPP02 U（纬度，″）", &elp.period_u, &elp.poisson_u, 1.0),
        ("ELPMPP02 R（距离，m）", &elp.period_r, &elp.poisson_r, 1000.0 * elp.constants.ra0.as_f64()),
    ];
    for (name, period, poisson, scale) in groups {
        let mut max_d: f64 = 0.0;
        let (mut t_dd, mut t_mx) = (0.0, 0.0);
        for &jd in &jds {
            let t = real((jd - 2_451_545.0) / 36_525.0);
            let a = [real(1.0), t, t * t, t * t * t, t * t * t * t];
            let s = Instant::now();
            let dd = elp_sum(period, &a, false) + elp_sum(poisson, &a, false);
            t_dd += s.elapsed().as_secs_f64();
            let s = Instant::now();
            let mx = elp_sum(period, &a, true) + elp_sum(poisson, &a, true);
            t_mx += s.elapsed().as_secs_f64();
            max_d = max_d.max(((dd - mx).as_f64() * scale).abs());
        }
        println!("{name}：{} 项，最大差 {max_d:.3e}；每次求和 double-double {:.2} ms，混合 {:.2} ms（{:.0} 倍）",
            period.len() + poisson.len(), t_dd * 1e3 / jds.len() as f64, t_mx * 1e3 / jds.len() as f64, t_dd / t_mx);
    }
    // VSOP87 地球：L、B（rad → ″）、R（AU → m）
    let mut max_d = [0.0f64; 3];
    let (mut t_dd, mut t_mx) = (0.0, 0.0);
    let n_terms: usize = earth.blocks.iter().map(|b| b.terms.len()).sum();
    for &jd in &jds {
        let tm = real((jd - 2_451_545.0) / 365_250.0);
        let mut sums = [[real(0.0); 3]; 2];
        for (v, mixed) in [(0usize, false), (1, true)] {
            let s = Instant::now();
            for b in &earth.blocks {
                let c = (b.coords as usize).saturating_sub(1);
                if c >= 3 {
                    continue;
                }
                let tp = (0..b.alpha_t).fold(real(1.0), |p, _| p * tm);
                for term in &b.terms {
                    let ph = term.phase.rad() + term.frequency.rad_per_julian_millennium() * tm;
                    let cb = if mixed { sin_cos_mixed(ph).1 } else { ph.sin_cos().1 };
                    let amp = match &term.amplitude {
                        Vsop87Amplitude::Angle(a) => a.rad(),
                        Vsop87Amplitude::Length(l) => l.meters(),
                    };
                    sums[v][c] = sums[v][c] + amp * tp * cb;
                }
            }
            if mixed { t_mx += s.elapsed().as_secs_f64() } else { t_dd += s.elapsed().as_secs_f64() }
        }
        for c in 0..3 {
            let k = if c < 2 { 206_264.806_247 } else { 1.0 };
            max_d[c] = max_d[c].max(((sums[0][c] - sums[1][c]).as_f64() * k).abs());
        }
    }
    println!("VSOP87 地球：{n_terms} 项，最大差 L {:.3e}″、B {:.3e}″、R {:.3e} m；每次 double-double {:.2} ms，混合 {:.2} ms（{:.0} 倍）",
        max_d[0], max_d[1], max_d[2], t_dd * 1e3 / jds.len() as f64, t_mx * 1e3 / jds.len() as f64, t_dd / t_mx);
    // 供跨后端对比：完整求值的月球位置（m）
    for jd in [625_000.5, 1_200_000.25, 1_850_000.0, 2_451_545.0, 2_900_000.5] {
        let (p, _) = position_velocity(&elp, TimePoint::new(TimeScale::TT, real(jd)));
        let m = p.to_meters();
        println!("MOON {jd} {:.6} {:.6} {:.6}", m[0].as_f64(), m[1].as_f64(), m[2].as_f64());
    }
}
