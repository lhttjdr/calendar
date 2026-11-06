//! ELP/MPP02 拟合工具共用：取样、JPL 目标位置（Table 7 转 J2000 平黄道）、多线程求值、Householder QR 最小二乘、统计。
//! 由 `elpmpp02_de_fit.rs`（常数两步拟合）与 `elpmpp02_residual_fit.rs`（残差改正）以 `#[path]` 引入。
#![allow(dead_code)]

use lunar_core::astronomy::ephemeris::elpmpp02::table7::icrs_to_ecliptic_j2000;
use lunar_core::astronomy::ephemeris::elpmpp02::{position_velocity, Elpmpp02Data};
use lunar_core::astronomy::ephemeris::De406Kernel;
use lunar_core::astronomy::time::{TimePoint, TimeScale};
use lunar_core::math::real::{real, RealOps};

pub const J2000: f64 = 2451545.0;
pub const ARCSEC: f64 = std::f64::consts::PI / 648000.0;

/// 年份 → JD（儒略年近似，取样用）。
pub fn year_to_jd(y: f64) -> f64 {
    J2000 + (y - 2000.0) * 365.25
}

/// 确定性伪随机取样（线性同余），避开整日以免与历表分段边界重合。
pub fn sample_jds(y0: f64, y1: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut s = seed;
    let (a, b) = (year_to_jd(y0), year_to_jd(y1));
    (0..n)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            a + (b - a) * ((s >> 11) as f64 / (1u64 << 53) as f64)
        })
        .collect()
}

/// 目标历表的地心月球位置，J2000 平黄道（Table 7 ICRS 行），km。
pub fn target_positions(k: &De406Kernel, jds: &[f64]) -> Vec<[f64; 3]> {
    jds.iter()
        .map(|&jd| {
            let (p, _) = k.geocentric_moon(jd).expect("历表时间范围外");
            let (x, y, z) = icrs_to_ecliptic_j2000(p[0] / 1000.0, p[1] / 1000.0, p[2] / 1000.0);
            [x, y, z]
        })
        .collect()
}

/// ELP 位置（km），多线程。时间自变量直接取 JD（TDB），与 JPL 历表同。
pub fn elp_positions(data: &Elpmpp02Data, jds: &[f64]) -> Vec<[f64; 3]> {
    let nt = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let chunk = jds.len().div_ceil(nt).max(1);
    std::thread::scope(|sc| {
        let hs: Vec<_> = jds
            .chunks(chunk)
            .map(|c| {
                sc.spawn(move || {
                    c.iter()
                        .map(|&jd| {
                            let (p, _) = position_velocity(data, TimePoint::new(TimeScale::TT, real(jd)));
                            let m = p.to_meters();
                            [m[0].as_f64() / 1000.0, m[1].as_f64() / 1000.0, m[2].as_f64() / 1000.0]
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}


/// 最小二乘 min |A x − b|（A 为 m×n 列主序），Householder QR。返回 (x, R⁻¹ 的对角线平方和行，用于形式误差)。
pub fn lstsq(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    let n = a.len();
    let m = b.len();
    // 列缩放，改善条件数
    let scale: Vec<f64> = a.iter().map(|c| c.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-300)).collect();
    for (c, s) in a.iter_mut().zip(&scale) {
        c.iter_mut().for_each(|x| *x /= s);
    }
    for j in 0..n {
        let norm = a[j][j..].iter().map(|x| x * x).sum::<f64>().sqrt();
        let alpha = if a[j][j] > 0.0 { -norm } else { norm };
        let mut v: Vec<f64> = a[j][j..].to_vec();
        v[0] -= alpha;
        let vn = v.iter().map(|x| x * x).sum::<f64>();
        if vn == 0.0 {
            continue;
        }
        for c in a.iter_mut().skip(j) {
            let d = 2.0 * v.iter().zip(&c[j..]).map(|(x, y)| x * y).sum::<f64>() / vn;
            c[j..].iter_mut().zip(&v).for_each(|(y, x)| *y -= d * x);
        }
        let d = 2.0 * v.iter().zip(&b[j..]).map(|(x, y)| x * y).sum::<f64>() / vn;
        b[j..].iter_mut().zip(&v).for_each(|(y, x)| *y -= d * x);
    }
    let _ = m;
    // 回代 R x = Qᵀb
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let s: f64 = (i + 1..n).map(|j| a[j][i] * x[j]).sum();
        x[i] = (b[i] - s) / a[i][i];
    }
    // R⁻¹（上三角），协方差 ∝ R⁻¹R⁻ᵀ 的对角线
    let mut rinv = vec![vec![0.0; n]; n];
    for i in 0..n {
        rinv[i][i] = 1.0 / a[i][i];
        for k in (0..i).rev() {
            let s: f64 = (k + 1..=i).map(|j| a[j][k] * rinv[j][i]).sum();
            rinv[k][i] = -s / a[k][k];
        }
    }
    let var: Vec<f64> = (0..n).map(|i| (i..n).map(|j| rinv[i][j] * rinv[i][j]).sum::<f64>() / (scale[i] * scale[i])).collect();
    (x.iter().zip(&scale).map(|(v, s)| v / s).collect(), var)
}

pub fn rms(v: &[f64]) -> f64 {
    (v.iter().map(|x| x * x).sum::<f64>() / v.len() as f64).sqrt()
}


/// 地心黄经、黄纬（角秒）与距离（km）之差的统计：(max|Δλ|, max|Δβ|, max|Δr|, rms Δλ)。
pub fn sph_stats(a: &[[f64; 3]], b: &[[f64; 3]]) -> (f64, f64, f64, f64) {
    let sph = |p: &[f64; 3]| {
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        (p[1].atan2(p[0]), (p[2] / r).asin(), r)
    };
    let (mut ml, mut mb, mut mr, mut sl) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    for (p, q) in a.iter().zip(b) {
        let (l1, b1, r1) = sph(p);
        let (l2, b2, r2) = sph(q);
        let dl = ((l1 - l2 + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI) / ARCSEC;
        ml = ml.max(dl.abs());
        mb = mb.max(((b1 - b2) / ARCSEC).abs());
        mr = mr.max((r1 - r2).abs());
        sl += dl * dl;
    }
    (ml, mb, mr, (sl / a.len() as f64).sqrt())
}

