use crate::astronomy::frame::precession::{mean_obliquity_rad_and_derivative_for, PrecessionModel};
use crate::math::algebra::mat::Mat;
use crate::math::real::{real, zero, one, Real, RealOps, ToReal};
use crate::math::series::{arcsec_to_rad, power_series_at, power_series_derivative_at};
use crate::quantity::angle::PlaneAngle;
use std::cell::RefCell;

fn rotation_x(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[one(), zero(), zero()], [zero(), c, -s], [zero(), s, c]]
}

fn rotation_x_derivative(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[zero(), zero(), zero()], [zero(), -s, -c], [zero(), c, -s]]
}

fn rotation_z(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[c, -s, zero()], [s, c, zero()], [zero(), zero(), one()]]
}

fn rotation_z_derivative(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[-s, -c, zero()], [c, -s, zero()], [zero(), zero(), zero()]]
}

fn add_vec(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale_vec(s: impl ToReal, v: [Real; 3]) -> [Real; 3] {
    let s = real(s);
    [v[0] * s, v[1] * s, v[2] * s]
}

/// F1–F5 月日基本角（弧秒）多项式系数 [T^0, T^1, ...]，T 为儒略世纪
const F1_F5_COEFFS: [[f64; 5]; 5] = [
    [485868.249036, 1717915923.2178, 31.8792, 0.051635, -0.00024470],   // l
    [1287104.79305, 129596581.0481, -0.5532, -0.000136, -0.00001149],  // lp
    [335779.526232, 1739527262.8478, -12.7512, -0.001037, 0.00000417], // F
    [1072260.70369, 1602961601.2090, -6.3706, 0.006593, -0.00003169],  // D
    [450160.398036, -6962890.5431, 7.4722, 0.007702, 0.00005939],      // Omega
];

/// 77 项月日章动表：每行 (c0..c4, A, Ap, App, B, Bp, Bpp)；内联前 10 项 + 占位，完整表见 LUNI_SOLAR_77_STR
const LUNI_SOLAR_77_ROWS: [([i32; 5], [i32; 6]); 77] = [
    ([0, 0, 0, 0, 1], [-172064161, -174666, 33386, 92052331, 9086, 15377]),
    ([0, 0, 2, -2, 2], [-13170906, -1675, -13696, 5730336, -3015, -4587]),
    ([0, 0, 2, 0, 2], [-2276413, -234, 2796, 978459, -485, 1374]),
    ([0, 0, 0, 0, 2], [2074554, 207, -698, -897492, 470, -291]),
    ([0, 1, 0, 0, 0], [1475877, -3633, 11817, 73871, -184, -1924]),
    ([0, 1, 2, -2, 2], [-516821, 1226, -524, 224386, -677, -174]),
    ([1, 0, 0, 0, 0], [711159, 73, -872, -6750, 0, 358]),
    ([0, 0, 2, 0, 1], [-387298, -367, 380, 200728, 18, 318]),
    ([1, 0, 2, 0, 2], [-301461, -36, 816, 129025, -63, 367]),
    ([0, -1, 2, -2, 2], [215829, -494, 111, -95929, 299, 132]),
    ([0, 0, 2, -2, 1], [128227, 137, 181, -68982, -9, 39]),
    ([-1, 0, 2, 0, 2], [123457, 11, 19, -53311, 32, -4]),
    ([-1, 0, 0, 2, 0], [156994, 10, -168, -1235, 0, 82]),
    ([1, 0, 0, 0, 1], [63110, 63, 27, -33228, 0, -9]),
    ([-1, 0, 0, 0, 1], [-57976, -63, -189, 31429, 0, -75]),
    ([-1, 0, 2, 2, 2], [-59641, -11, 149, 25543, -11, 66]),
    ([1, 0, 2, 0, 1], [-51613, -42, 129, 26366, 0, 78]),
    ([-2, 0, 2, 0, 1], [45893, 50, 31, -24236, -10, 20]),
    ([0, 0, 0, 2, 0], [63384, 11, -150, -1220, 0, 29]),
    ([0, 0, 2, 2, 2], [-38571, -1, 158, 16452, -11, 68]),
    ([0, -2, 2, -2, 2], [32481, 0, 0, -13870, 0, 0]),
    ([-2, 0, 0, 2, 0], [-47722, 0, -18, 477, 0, -25]),
    ([2, 0, 2, 0, 2], [-31046, -1, 131, 13238, -11, 59]),
    ([1, 0, 2, -2, 2], [28593, 0, -1, -12338, 10, -3]),
    ([-1, 0, 2, 0, 1], [20441, 21, 10, -10758, 0, -3]),
    ([2, 0, 0, 0, 0], [29243, 0, -74, -609, 0, 13]),
    ([0, 0, 2, 0, 0], [25887, 0, -66, -550, 0, 11]),
    ([0, 1, 0, 0, 1], [-14053, -25, 79, 8551, -2, -45]),
    ([-1, 0, 0, 2, 1], [15164, 10, 11, -8001, 0, -1]),
    ([0, 2, 2, -2, 2], [-15794, 72, -16, 6850, -42, -5]),
    ([0, 0, -2, 2, 0], [21783, 0, 13, -167, 0, 13]),
    ([1, 0, 0, -2, 1], [-12873, -10, -37, 6953, 0, -14]),
    ([0, -1, 0, 0, 1], [-12654, 11, 63, 6415, 0, 26]),
    ([-1, 0, 2, 2, 1], [-10204, 0, 25, 5222, 0, 15]),
    ([0, 2, 0, 0, 0], [16707, -85, -10, 168, -1, 10]),
    ([1, 0, 2, 2, 2], [-7691, 0, 44, 3268, 0, 19]),
    ([-2, 0, 2, 0, 0], [-11024, 0, -14, 104, 0, 2]),
    ([0, 1, 2, 0, 2], [7566, -21, -11, -3250, 0, -5]),
    ([0, 0, 2, 2, 1], [-6637, -11, 25, 3353, 0, 14]),
    ([0, -1, 2, 0, 2], [-7141, 21, 8, 3070, 0, 4]),
    ([0, 0, 0, 2, 1], [-6302, -11, 2, 3272, 0, 4]),
    ([1, 0, 2, -2, 1], [5800, 10, 2, -3045, 0, -1]),
    ([2, 0, 2, -2, 2], [6443, 0, -7, -2768, 0, -4]),
    ([-2, 0, 0, 2, 1], [-5774, -11, -15, 3041, 0, -5]),
    ([2, 0, 2, 0, 1], [-5350, 0, 21, 2695, 0, 12]),
    ([0, -1, 2, -2, 1], [-4752, -11, -3, 2719, 0, -3]),
    ([0, 0, 0, -2, 1], [-4940, -11, -21, 2720, 0, -9]),
    ([-1, -1, 0, 2, 0], [7350, 0, -8, -51, 0, 4]),
    ([2, 0, 0, -2, 1], [4065, 0, 6, -2206, 0, 1]),
    ([1, 0, 0, 2, 0], [6579, 0, -24, -199, 0, 2]),
    ([0, 1, 2, -2, 1], [3579, 0, 5, -1900, 0, 1]),
    ([1, -1, 0, 0, 0], [4725, 0, -6, -41, 0, 3]),
    ([-2, 0, 2, 0, 2], [-3075, 0, -2, 1313, 0, -1]),
    ([3, 0, 2, 0, 2], [-2904, 0, 15, 1233, 0, 7]),
    ([0, -1, 0, 2, 0], [4348, 0, -10, -81, 0, 2]),
    ([1, -1, 2, 0, 2], [-2878, 0, 8, 1232, 0, 4]),
    ([0, 0, 0, 1, 0], [-4230, 0, 5, -20, 0, -2]),
    ([-1, -1, 2, 2, 2], [-2819, 0, 7, 1207, 0, 3]),
    ([-1, 0, 2, 0, 0], [-4056, 0, 5, 40, 0, -2]),
    ([0, -1, 2, 2, 2], [-2647, 0, 11, 1129, 0, 5]),
    ([-2, 0, 0, 0, 1], [-2294, 0, -10, 1266, 0, -4]),
    ([1, 1, 2, 0, 2], [2481, 0, -7, -1062, 0, -3]),
    ([2, 0, 0, 0, 1], [2179, 0, -2, -1129, 0, -2]),
    ([-1, 1, 0, 1, 0], [3276, 0, 1, -9, 0, 0]),
    ([1, 1, 0, 0, 0], [-3389, 0, 5, 35, 0, -2]),
    ([1, 0, 2, 0, 0], [3339, 0, -13, -107, 0, 1]),
    ([-1, 0, 2, -2, 1], [-1987, 0, -6, 1073, 0, -2]),
    ([1, 0, 0, 0, 2], [-1981, 0, 0, 854, 0, 0]),
    ([-1, 0, 0, 1, 0], [4026, 0, -353, -553, 0, -139]),
    ([0, 0, 2, 1, 2], [1660, 0, -5, -710, 0, -2]),
    ([-1, 0, 2, 4, 2], [-1521, 0, 9, 647, 0, 4]),
    ([-1, 1, 0, 1, 1], [1314, 0, 0, -700, 0, 0]),
    ([0, -2, 2, -2, 1], [-1283, 0, 0, 672, 0, 0]),
    ([1, 0, 2, 2, 1], [-1331, 0, 8, 663, 0, 4]),
    ([-2, 0, 2, 2, 2], [1383, 0, -2, -594, 0, -2]),
    ([-1, 0, 0, 0, 2], [1405, 0, 4, -610, 0, 2]),
    ([1, 1, 2, -2, 2], [1290, 0, 0, -556, 0, 0]),
];

/// 儒略世纪 t = (JD_TT - 2451545)/36525 下的 5 个基本角（F1–F5：l, l′, F, D, Ω）。t 支持 Real。
pub fn fundamental_arguments(t: impl ToReal) -> [PlaneAngle; 5] {
    let t = real(t);
    let mut out = [PlaneAngle::from_rad(crate::math::real::real(0)); 5];
    for (i, coeffs) in F1_F5_COEFFS.iter().enumerate() {
        let arcsec = power_series_at(coeffs, t);
        out[i] = PlaneAngle::from_rad(arcsec_to_rad(arcsec));
    }
    out
}

/// 基本角对 t 的导数（弧秒/世纪 → 弧度/世纪），用于章动导数。t 支持 Real；返回 f64 仅因下游与表循环兼容。
pub fn fundamental_arguments_derivative(t: impl ToReal) -> [f64; 5] {
    let t = real(t);
    let mut out = [0.0_f64; 5];
    for (i, coeffs) in F1_F5_COEFFS.iter().enumerate() {
        let arcsec_per_century = power_series_derivative_at(coeffs, t);
        out[i] = arcsec_to_rad(arcsec_per_century).as_f64();
    }
    out
}

/// 章动对 t 的导数 (dψ/dt, dε/dt)，弧度/世纪。与 MHB2000Truncated nutationDerivative 一致。t 支持 Real。
pub fn nutation_derivative(t: impl ToReal) -> (Real, Real) {
    let t_r = real(t);
    let f = fundamental_arguments(t_r);
    let f_dot = fundamental_arguments_derivative(t_r);
    let t_f64 = t_r.as_f64();
    let scale = 1e-7_f64 * (core::f64::consts::PI / 180.0 / 3600.0); // 表值×1e-7 弧秒 → 弧度
    let mut dpsi_dt = 0.0_f64;
    let mut deps_dt = 0.0_f64;
    for (c, a) in &LUNI_SOLAR_77_ROWS {
        let fi: f64 = c.iter().zip(f.iter()).map(|(ci, fi)| (*ci as f64) * fi.rad().as_f64()).sum();
        let fi = fi.rem_euclid(core::f64::consts::TAU);
        let dfi_dt: f64 = c.iter().zip(f_dot.iter()).map(|(ci, fdi)| (*ci as f64) * fdi).sum();
        let (sin_fi, cos_fi) = (fi.sin(), fi.cos());
        let a_psi = (a[0] as f64) + (a[1] as f64) * t_f64;
        let a_eps = (a[3] as f64) + (a[4] as f64) * t_f64;
        dpsi_dt += (a[1] as f64) * sin_fi + (a_psi * cos_fi - (a[2] as f64) * sin_fi) * dfi_dt;
        deps_dt += (a[4] as f64) * cos_fi + (-a_eps * sin_fi + (a[5] as f64) * cos_fi) * dfi_dt;
    }
    (real(dpsi_dt * scale), real(deps_dt * scale))
}

/// 真黄赤交角对 t 的导数：ε_true = εA + Δε，故 d(ε_true)/dt = d(εA)/dt + d(Δε)/dt，弧度/世纪。t 支持 Real。
pub fn eps_true_dot(t: impl ToReal, precession_model: PrecessionModel) -> Real {
    let t_real = real(t);
    let (_, eps_a_dot) = mean_obliquity_rad_and_derivative_for(t_real, precession_model);
    let (_, deps_dt) = nutation_derivative(t_real);
    eps_a_dot.rad_per_julian_century() + deps_dt
}

/// (dN^T/dt)·r，N^T 为平赤道→真赤道的章动矩阵的转置，r 为平赤道架下向量（米），返回 m/世纪。t 支持 Real。
pub fn nutation_derivative_times_vector(r: [Real; 3], t: impl ToReal, precession_model: PrecessionModel) -> [Real; 3] {
    let t_real = real(t);
    let (eps_a, eps_a_dot) = mean_obliquity_rad_and_derivative_for(t_real, precession_model);
    let (dpsi, deps) = nutation_for_apparent(t_real);
    let dpsi_rad = dpsi.rad();
    let deps_rad = deps.rad();
    let eps_a_rad = eps_a.rad();
    let eps_true = eps_a_rad + deps_rad;
    let (dpsi_dt, deps_dt) = nutation_derivative(t_real);
    let eps_a_dot_rad_cy = eps_a_dot.rad_per_julian_century();
    let eps_true_dot_val = eps_a_dot_rad_cy + deps_dt;

    let r1_neg_eps_a = rotation_x(-eps_a_rad);
    let r3_dpsi = rotation_z(dpsi_rad);
    let r1_eps_true = rotation_x(eps_true);

    let w1 = Mat::<Real, 3, 3>::from(r1_neg_eps_a).mul_vec(r);
    let w2 = Mat::<Real, 3, 3>::from(r3_dpsi).mul_vec(w1);
    let _w3 = Mat::<Real, 3, 3>::from(r1_eps_true).mul_vec(w2);

    let term1 = scale_vec(eps_true_dot_val, Mat::<Real, 3, 3>::from(rotation_x_derivative(eps_true)).mul_vec(w2));
    let term2 = Mat::<Real, 3, 3>::from(r1_eps_true).mul_vec(scale_vec(dpsi_dt, Mat::<Real, 3, 3>::from(rotation_z_derivative(dpsi_rad)).mul_vec(w1)));
    let term3 = Mat::<Real, 3, 3>::from(r1_eps_true).mul_vec(Mat::<Real, 3, 3>::from(r3_dpsi).mul_vec(scale_vec(-eps_a_dot_rad_cy, Mat::<Real, 3, 3>::from(rotation_x_derivative(-eps_a_rad)).mul_vec(r))));

    add_vec(term1, add_vec(term2, term3))
}

/// 77 项章动 (Δψ, Δε)；t 为儒略世纪。t 支持 Real。
pub fn nutation_77(t: impl ToReal) -> (PlaneAngle, PlaneAngle) {
    let t_r = real(t);
    let t_f64 = t_r.as_f64();
    let f = fundamental_arguments(t_r);
    let scale = 1e-7_f64; // 表值 × 1e-7 = 弧秒
    let mut dpsi_arcsec = 0.0_f64;
    let mut deps_arcsec = 0.0_f64;
    for (c, a) in &LUNI_SOLAR_77_ROWS {
        let fi: f64 = c.iter().zip(f.iter()).map(|(ci, fi)| (*ci as f64) * fi.rad().as_f64()).sum();
        let fi = fi % (2.0 * core::f64::consts::PI);
        let fi = if fi < 0.0 { fi + 2.0 * core::f64::consts::PI } else { fi };
        let (sin_fi, cos_fi) = (fi.sin(), fi.cos());
        let a_psi = (a[0] as f64) + (a[1] as f64) * t_f64;
        let a_eps = (a[3] as f64) + (a[4] as f64) * t_f64;
        dpsi_arcsec += (a_psi * sin_fi + (a[2] as f64) * cos_fi) * scale;
        deps_arcsec += (a_eps * cos_fi + (a[5] as f64) * sin_fi) * scale;
    }
    dpsi_arcsec += -0.135e-6;
    deps_arcsec += 0.388e-6;
    (
        PlaneAngle::from_rad(arcsec_to_rad(crate::math::real::real(dpsi_arcsec))),
        PlaneAngle::from_rad(arcsec_to_rad(crate::math::real::real(deps_arcsec))),
    )
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static NUTATION_OVERRIDE: RefCell<Option<Box<dyn Fn(Real) -> (PlaneAngle, PlaneAngle) + Send>>> = RefCell::new(None);
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static NUTATION_OVERRIDE: RefCell<Option<Box<dyn Fn(Real) -> (PlaneAngle, PlaneAngle)>>> = RefCell::new(None);
}

/// 设置章动覆盖（如加载 data/IAU2000 后的 Iau2000a）。传 None 恢复 77 项。Native 需 Send；WASM 无此要求。回调入参为 Real。
#[cfg(not(target_arch = "wasm32"))]
pub fn set_nutation_override(f: Option<Box<dyn Fn(Real) -> (PlaneAngle, PlaneAngle) + Send>>) {
    NUTATION_OVERRIDE.with(|cell| *cell.borrow_mut() = f);
}

#[cfg(target_arch = "wasm32")]
pub fn set_nutation_override(f: Option<Box<dyn Fn(Real) -> (PlaneAngle, PlaneAngle)>>) {
    NUTATION_OVERRIDE.with(|cell| *cell.borrow_mut() = f);
}

/// 视黄经 pipeline 用：有覆盖则调用覆盖，否则 nutation_77(t)。t 支持 Real。
pub fn nutation_for_apparent(t: impl ToReal) -> (PlaneAngle, PlaneAngle) {
    let t_r = real(t);
    NUTATION_OVERRIDE.with(|cell| {
        if let Some(ref f) = *cell.borrow() {
            return f(t_r);
        }
        nutation_77(t_r)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn nutation_77_j2000() {
        let t = 0.0; // J2000 儒略世纪
        let (dpsi, deps) = nutation_77(t);
        let dpsi_sec = dpsi.rad() * real(180.0 / core::f64::consts::PI) * real(3600.0);
        let deps_sec = deps.rad() * real(180.0 / core::f64::consts::PI) * real(3600.0);
        assert!(dpsi_sec.abs() < real(20.0) && deps_sec.abs() < real(20.0), "dpsi={}″, deps={}″", dpsi_sec.as_f64(), deps_sec.as_f64());
    }

    /// 14 fundamental arguments at J2000 — 本实现校验 F1(l)、F5(Omega) 弧秒值
    #[test]
    fn fundamental_arguments_at_j2000_f1_f5() {
        let t = 0.0;
        let f = fundamental_arguments(t);
        let l_expected = arcsec_to_rad(485868.249036);
        let omega_expected = arcsec_to_rad(450160.398036);
        assert!(f[0].rad().is_near(l_expected, 1e-10), "F1 (l) at t=0");
        assert!(f[4].rad().is_near(omega_expected, 1e-10), "F5 (Omega) at t=0");
    }

    /// Omega-only term (0,0,0,0,1) 的组合角等于 F5
    #[test]
    fn fundamental_arguments_omega_only_equals_f5() {
        let t = 0.0;
        let f = fundamental_arguments(t);
        let c = [0i32, 0, 0, 0, 1];
        let theta: f64 = c.iter().zip(f.iter()).map(|(ci, fi)| (*ci as f64) * fi.rad().as_f64()).sum();
        let two_pi = 2.0 * core::f64::consts::PI;
        let theta = theta % two_pi;
        let theta = if theta < 0.0 { theta + two_pi } else { theta };
        assert!(real(theta).is_near(f[4].rad(), 1e-15));
    }
}
