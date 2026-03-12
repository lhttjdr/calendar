//! Vondrák et al. (2011) 长期岁差，A&A 534, A22。Vondrák 长期岁差。矩阵与向量统一 Real。

use crate::math::real::{real_const, real, zero, Real, RealOps, ToReal};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::angular_rate::AngularRate;
use crate::quantity::unit::AngularRateUnit;
use crate::math::series::{
    arcsec_to_rad, evaluate_poisson_periodic, evaluate_poisson_periodic_derivative,
    evaluate_poisson_periodic_two, evaluate_poisson_periodic_two_derivative, power_series_at,
    power_series_derivative_at,
};

/// J2000.0 平黄赤交角 84381.406″（弧度）
const EPS0_RAD: f64 = 84381.406 * (core::f64::consts::PI / 180.0 / 3600.0);

// --------------- 黄道岁差 PA, QA (Eq.8, Table 1) ---------------
const PA_QA_POL_P: [f64; 4] = [5851.607687, -0.1189000, -0.00028913, 0.000000101];
const PA_QA_POL_Q: [f64; 4] = [-1600.886300, 1.1689818, -0.00000020, -0.000000437];

const PA_QA_PER: [(f64, f64, f64, f64, f64); 8] = [
    (708.15, -5486.751211, -684.661560, 667.666730, -5523.863691),
    (2309.00, -17.127623, 2446.283880, -2354.886252, -549.747450),
    (1620.00, -617.517403, 399.671049, -428.152441, -310.998056),
    (492.20, 413.442940, -356.652376, 376.202861, 421.535876),
    (1183.00, 78.614193, -186.387003, 184.778874, -36.776172),
    (622.00, -180.732815, -316.800070, 335.321713, -145.278396),
    (882.00, -87.676083, 198.296071, -185.138669, -34.744450),
    (547.00, 46.140315, 101.135679, -120.972830, 22.885731),
];

// --------------- 赤道岁差 XA, YA (Eq.9, Table 2) ---------------
const XA_YA_POL_X: [f64; 4] = [5453.282155, 0.4252841, -0.00037173, -0.000000152];
const XA_YA_POL_Y: [f64; 4] = [-73750.930350, -0.7675452, -0.00018725, 0.000000231];

const XA_YA_PER: [(f64, f64, f64, f64, f64); 14] = [
    (256.75, -819.940624, 75004.344875, 81491.287984, 1558.515853),
    (708.15, -8444.676815, 624.033993, 787.163481, 7774.939698),
    (274.20, 2600.009459, 1251.136893, 1251.296102, -2219.534038),
    (241.45, 2755.175630, -1102.212834, -1257.950837, -2523.969396),
    (2309.00, -167.659835, -2660.664980, -2966.799730, 247.850422),
    (492.20, 871.855056, 699.291817, 639.744522, -846.485643),
    (396.10, 44.769698, 153.167220, 131.600209, -1393.124055),
    (288.90, -512.313065, -950.865637, -445.040117, 368.526116),
    (231.10, -819.415595, 499.754645, 584.522874, 749.045012),
    (1610.00, -538.071099, -145.188210, -89.756563, 444.704518),
    (620.00, -189.793622, 558.116553, 524.429630, 235.934465),
    (157.87, -402.922932, -23.923029, -13.549067, 374.049623),
    (220.30, 179.516345, -165.405086, -210.157124, -171.330180),
    (1200.00, -9.814756, 9.344131, -44.919798, -22.899655),
];

// --------------- 平均黄赤交角 εA (Eq.10, Table 3) ---------------
const EPS_A_POL: [f64; 4] = [84028.206305, 0.3624445, -0.00004039, -0.000000110];
const EPS_A_PER: [(f64, f64, f64); 10] = [
    (409.90, 753.872780, -1704.720302),
    (396.15, -247.805823, -862.308358),
    (537.22, 379.471484, 447.832178),
    (402.90, -53.880558, -889.571909),
    (417.15, -90.109153, 190.402846),
    (288.92, -353.600190, -56.564991),
    (4043.00, -63.115353, -296.222622),
    (306.00, -28.248187, -75.859952),
    (277.00, 17.703387, 67.473503),
    (203.00, 38.911307, 3.014055),
];

fn cross3(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm3(v: [Real; 3]) -> Real {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [Real; 3]) -> [Real; 3] {
    let n = norm3(v);
    if n > zero() {
        [v[0] / n, v[1] / n, v[2] / n]
    } else {
        v
    }
}

/// 黄道极参数 PA, QA（弧秒），返回 Real。t 支持 Real，内部仅一处 as_f64 调表。
fn pa_qa_arcsec(t: impl crate::math::real::ToReal) -> (Real, Real) {
    let t = crate::math::real::real(t).as_f64();
    let (p_per, q_per) = evaluate_poisson_periodic_two(&PA_QA_PER, t);
    (
        real(p_per) + power_series_at(&PA_QA_POL_P, t),
        real(q_per) + power_series_at(&PA_QA_POL_Q, t),
    )
}

/// 赤道极参数 XA, YA（弧秒），返回 Real。
fn xa_ya_arcsec(t: impl crate::math::real::ToReal) -> (Real, Real) {
    let t = crate::math::real::real(t).as_f64();
    let (x_per, y_per) = evaluate_poisson_periodic_two(&XA_YA_PER, t);
    (
        real(x_per) + power_series_at(&XA_YA_POL_X, t),
        real(y_per) + power_series_at(&XA_YA_POL_Y, t),
    )
}

/// 黄道极单位向量（J2000.0 平赤道平春分系），ltp_PECL。t 支持 Real。
pub fn ecliptic_pole(t: impl ToReal) -> [Real; 3] {
    let t_r = real(t);
    let (p_sec, q_sec) = pa_qa_arcsec(t_r);
    let p = arcsec_to_rad(p_sec);
    let q = arcsec_to_rad(q_sec);
    let p2 = p * p;
    let q2 = q * q;
    let z = (real_const(1.0) - p2 - q2).max(zero()).sqrt();
    let eps0 = real(EPS0_RAD);
    let (s, c) = (eps0.sin(), eps0.cos());
    [p, -q * c - z * s, -q * s + z * c]
}

/// 赤道极单位向量（J2000.0 平赤道平春分系），ltp_PEQU。t 支持 Real。
pub fn equator_pole(t: impl ToReal) -> [Real; 3] {
    let t_r = real(t);
    let (x_sec, y_sec) = xa_ya_arcsec(t_r);
    let x = arcsec_to_rad(x_sec);
    let y = arcsec_to_rad(y_sec);
    let w = x * x + y * y;
    let z = if w < real_const(1.0) { (real_const(1.0) - w).sqrt() } else { zero() };
    [x, y, z]
}

/// PA, QA 对 t 的导数（弧秒/世纪），返回 Real。
fn pa_qa_arcsec_derivative(t: impl crate::math::real::ToReal) -> (Real, Real) {
    let t = crate::math::real::real(t).as_f64();
    let (dp_per, dq_per) = evaluate_poisson_periodic_two_derivative(&PA_QA_PER, t);
    (
        real(dp_per) + power_series_derivative_at(&PA_QA_POL_P, t),
        real(dq_per) + power_series_derivative_at(&PA_QA_POL_Q, t),
    )
}

/// XA, YA 对 t 的导数（弧秒/世纪），返回 Real。
fn xa_ya_arcsec_derivative(t: impl crate::math::real::ToReal) -> (Real, Real) {
    let t = crate::math::real::real(t).as_f64();
    let (dx_per, dy_per) = evaluate_poisson_periodic_two_derivative(&XA_YA_PER, t);
    (
        real(dx_per) + power_series_derivative_at(&XA_YA_POL_X, t),
        real(dy_per) + power_series_derivative_at(&XA_YA_POL_Y, t),
    )
}

/// 黄道极对 t 的导数（弧度/世纪）
fn ecliptic_pole_derivative(t: impl ToReal) -> [Real; 3] {
    let t_r = real(t);
    let (dp_sec, dq_sec) = pa_qa_arcsec_derivative(t_r);
    let dp = arcsec_to_rad(dp_sec);
    let dq = arcsec_to_rad(dq_sec);
    let (p_sec, q_sec) = pa_qa_arcsec(t_r);
    let p = arcsec_to_rad(p_sec);
    let q = arcsec_to_rad(q_sec);
    let p2 = p * p;
    let q2 = q * q;
    let z = (real_const(1.0) - p2 - q2).max(zero()).sqrt();
    let dz = if z > zero() { -(p * dp + q * dq) / z } else { zero() };
    let eps0 = real(EPS0_RAD);
    let (s, c) = (eps0.sin(), eps0.cos());
    [dp, -c * dq - s * dz, -s * dq + c * dz]
}

/// 赤道极对 t 的导数（弧度/世纪）
fn equator_pole_derivative(t: impl ToReal) -> [Real; 3] {
    let t_r = real(t);
    let (dx_sec, dy_sec) = xa_ya_arcsec_derivative(t_r);
    let dx = arcsec_to_rad(dx_sec);
    let dy = arcsec_to_rad(dy_sec);
    let (x_sec, y_sec) = xa_ya_arcsec(t_r);
    let x = arcsec_to_rad(x_sec);
    let y = arcsec_to_rad(y_sec);
    let w = x * x + y * y;
    let z = if w < real_const(1.0) { (real_const(1.0) - w).sqrt() } else { zero() };
    let dz = if z > zero() { -(x * dx + y * dy) / z } else { zero() };
    [dx, dy, dz]
}

/// 岁差矩阵 P：J2000.0 平赤道 → 该历元平赤道，ltp_PMAT。t 支持 Real。
pub fn precession_matrix(t: impl ToReal) -> [[Real; 3]; 3] {
    let t_r = real(t);
    let pequ = equator_pole(t_r);
    let pecl = ecliptic_pole(t_r);
    let v = cross3(pequ, pecl);
    let eqx = normalize3(v);
    let mid = cross3(pequ, eqx);
    [
        [eqx[0], eqx[1], eqx[2]],
        [mid[0], mid[1], mid[2]],
        [pequ[0], pequ[1], pequ[2]],
    ]
}

/// 平均黄赤交角 εA。t 支持 Real。
pub fn epsilon(t: impl ToReal) -> PlaneAngle {
    let t_r = real(t);
    let t_f64 = t_r.as_f64();
    PlaneAngle::from_rad(arcsec_to_rad(
        real(evaluate_poisson_periodic(&EPS_A_PER, t_f64)) + power_series_at(&EPS_A_POL, t_r),
    ))
}

/// εA 对 t 的导数。t 支持 Real。
pub fn epsilon_derivative(t: impl ToReal) -> AngularRate {
    let t_r = real(t);
    let t_f64 = t_r.as_f64();
    AngularRate::from_value(
        arcsec_to_rad(
            real(evaluate_poisson_periodic_derivative(&EPS_A_PER, t_f64))
                + power_series_derivative_at(&EPS_A_POL, t_r),
        ),
        AngularRateUnit::RadPerJulianCentury,
    )
}

/// (dP/dt)·r，Vondrak 岁差矩阵导数，t 儒略世纪，r 直角坐标（米）。返回 m/世纪。
pub fn precession_matrix_derivative_times_vector(r: [Real; 3], t: impl ToReal) -> [Real; 3] {
    let t_r = real(t);
    let pequ = equator_pole(t_r);
    let pecl = ecliptic_pole(t_r);
    let dpequ = equator_pole_derivative(t_r);
    let dpecl = ecliptic_pole_derivative(t_r);
    let v = cross3(pequ, pecl);
    let nv = norm3(v);
    if nv <= zero() {
        return [zero(), zero(), zero()];
    }
    let dv = [
        dpequ[1] * pecl[2] - dpequ[2] * pecl[1] + pequ[1] * dpecl[2] - pequ[2] * dpecl[1],
        dpequ[2] * pecl[0] - dpequ[0] * pecl[2] + pequ[2] * dpecl[0] - pequ[0] * dpecl[2],
        dpequ[0] * pecl[1] - dpequ[1] * pecl[0] + pequ[0] * dpecl[1] - pequ[1] * dpecl[0],
    ];
    let eqx = [v[0] / nv, v[1] / nv, v[2] / nv];
    let nv2 = nv * nv;
    let dnv = (v[0] * dv[0] + v[1] * dv[1] + v[2] * dv[2]) / nv;
    let deqx = [
        (dv[0] * nv - v[0] * dnv) / nv2,
        (dv[1] * nv - v[1] * dnv) / nv2,
        (dv[2] * nv - v[2] * dnv) / nv2,
    ];
    let _mid = cross3(pequ, eqx);
    let dmid = [
        dpequ[1] * eqx[2] - dpequ[2] * eqx[1] + pequ[1] * deqx[2] - pequ[2] * deqx[1],
        dpequ[2] * eqx[0] - dpequ[0] * eqx[2] + pequ[2] * deqx[0] - pequ[0] * deqx[2],
        dpequ[0] * eqx[1] - dpequ[1] * eqx[0] + pequ[0] * deqx[1] - pequ[1] * deqx[0],
    ];
    [
        deqx[0] * r[0] + deqx[1] * r[1] + deqx[2] * r[2],
        dmid[0] * r[0] + dmid[1] * r[1] + dmid[2] * r[2],
        dpequ[0] * r[0] + dpequ[1] * r[1] + dpequ[2] * r[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::constant::J2000;
    use crate::math::real::{real, RealOps};

    /// 儒略世纪 T = (JD - J2000) / 36525
    fn jd_to_centuries(jd: f64) -> f64 {
        (jd - J2000.as_f64()) / 36525.0
    }

    /// 附录算例：–1374 (1375 BCE) May 3, 13:52:19.2 TT；JD(TT)=1219339.078，T≈-33.736 cy
    const TEST_JD: f64 = 1219339.078;

    #[test]
    fn vondrak_ecliptic_pole_vs_paper() {
        let t = jd_to_centuries(TEST_JD);
        let pecl = ecliptic_pole(t);
        let paper = (0.00041724785764001342, -0.40495491104576162693, 0.91433656053126552350);
        let tol = 2e-14;
        assert!(pecl[0].is_near(real(paper.0), tol), "pecl(0)");
        assert!(pecl[1].is_near(real(paper.1), tol), "pecl(1)");
        assert!(pecl[2].is_near(real(paper.2), tol), "pecl(2)");
    }

    #[test]
    fn vondrak_equator_pole_vs_paper() {
        let t = jd_to_centuries(TEST_JD);
        let pequ = equator_pole(t);
        let paper = (-0.29437643797369031532, -0.11719098023370257855, 0.94847708824082091796);
        let tol = 2e-14;
        assert!(pequ[0].is_near(real(paper.0), tol), "pequ(0)");
        assert!(pequ[1].is_near(real(paper.1), tol), "pequ(1)");
        assert!(pequ[2].is_near(real(paper.2), tol), "pequ(2)");
    }

    #[test]
    fn vondrak_precession_matrix_vs_paper() {
        let t = jd_to_centuries(TEST_JD);
        let rp = precession_matrix(t);
        let tol = 2e-14;
        assert!(rp[0][0].is_near(real(0.68473390570729557360), tol), "Rp(0,0)");
        assert!(rp[0][1].is_near(real(0.66647794042757610444), tol), "Rp(0,1)");
        assert!(rp[0][2].is_near(real(0.29486714516583357655), tol), "Rp(0,2)");
        assert!(rp[1][0].is_near(real(-0.66669482609418419936), tol), "Rp(1,0)");
        assert!(rp[1][1].is_near(real(0.73625636097440967969), tol), "Rp(1,1)");
        assert!(rp[1][2].is_near(real(-0.11595076448202158534), tol), "Rp(1,2)");
        assert!(rp[2][0].is_near(real(-0.29437643797369031532), tol), "Rp(2,0)");
        assert!(rp[2][1].is_near(real(-0.11719098023370257855), tol), "Rp(2,1)");
        assert!(rp[2][2].is_near(real(0.94847708824082091796), tol), "Rp(2,2)");
    }

    #[test]
    fn vondrak_at_j2000_equator_pole_near_z() {
        use crate::math::real::{real, RealOps};
        let t = 0.0;
        let pequ = equator_pole(t);
        assert!(pequ[0].abs().is_near(real(0), 1e-10));
        assert!(pequ[1].abs().is_near(real(0), 1e-10));
        assert!(pequ[2] > real(0.99999));
    }

    #[test]
    fn vondrak_at_j2000_ecliptic_pole_reasonable() {
        use crate::math::real::{real, RealOps};
        let t = 0.0;
        let pecl = ecliptic_pole(t);
        assert!(pecl[0].abs().is_near(real(0), 1e-10));
        assert!(pecl[2] > real(0.9));
    }

    #[test]
    fn vondrak_epsilon_at_j2000_in_range() {
        use crate::math::real::real;
        let eps = epsilon(0.0);
        let deg = eps.rad() * real(180.0 / core::f64::consts::PI);
        assert!(deg > real(23.0) && deg < real(24.0));
    }
}
