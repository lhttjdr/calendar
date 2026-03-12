use super::iau2000a::Iau2000a;
use crate::astronomy::frame::precession::{mean_obliquity_rad_and_derivative_for, PrecessionModel};
use crate::math::algebra::mat::Mat;
use crate::math::real::{real, zero, one, Real, RealOps, ToReal};
use crate::math::series::{arcsec_to_rad, power_series_at, power_series_derivative_at};
use crate::quantity::angle::PlaneAngle;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::sync::RwLock;

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

/// 章动模型：重边时用 edge_key 区分（IAU2000A 完整表 / IAU2000B 77 项）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NutationModel {
    /// IAU 2000A：IERS 5.3a+5.3b 完整表（需 try_init_full_nutation）。
    IAU2000A,
    /// IAU 2000B：77 项（MHB_2000_SHORT），与 SOFA iauNut00b 对应。
    IAU2000B,
}

/// 按模型返回章动 (Δψ, Δε)。IAU2000B 用 77 项；IAU2000A 用完整表覆盖（若已 init）否则 77 项。
pub fn nutation_for_model(t: impl ToReal, model: NutationModel) -> (PlaneAngle, PlaneAngle) {
    let t_r = real(t);
    match model {
        NutationModel::IAU2000B => nutation_77(t_r),
        NutationModel::IAU2000A => NUTATION_OVERRIDE.with(|cell| {
            if let Some(ref f) = *cell.borrow() {
                f(t_r)
            } else {
                nutation_77(t_r)
            }
        }),
    }
}

/// 77 项章动缓存（IAU 2000B = MHB_2000_SHORT）：从 IERS 5.3a+5.3b 合并表前 77 行加载，与完整表同源。未初始化时 nutation_77 / nutation_derivative 返回零。
static NUTATION_77_CACHE: Lazy<RwLock<Option<Iau2000a>>> = Lazy::new(|| RwLock::new(None));

fn set_nutation_77_cache(iau: Option<Iau2000a>) {
    *NUTATION_77_CACHE.write().unwrap() = iau;
}

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

/// 章动对 t 的导数 (dψ/dt, dε/dt)，弧度/世纪。使用 77 项缓存（来自 tab5.3a 前 77 行）；未初始化时返回零。
pub fn nutation_derivative(t: impl ToReal) -> (Real, Real) {
    if let Some(ref m) = *NUTATION_77_CACHE.read().unwrap() {
        m.nutation_derivative(t)
    } else {
        (zero(), zero())
    }
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

/// 77 项章动 (Δψ, Δε)；t 为儒略世纪。数据来自 IERS 5.3a+5.3b 合并表前 77 行（需先调用 try_init_full_nutation 或 try_init_nutation）；未初始化时返回零。
pub fn nutation_77(t: impl ToReal) -> (PlaneAngle, PlaneAngle) {
    if let Some(ref m) = *NUTATION_77_CACHE.read().unwrap() {
        m.nutation(t)
    } else {
        (
            PlaneAngle::from_rad(zero()),
            PlaneAngle::from_rad(zero()),
        )
    }
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

/// 若 loader 能加载 IERS 5.3a+5.3b 双文件，则启用完整 IAU2000A 章动，并缓存前 77 项；否则 77 项缓存为空。
#[cfg(not(target_arch = "wasm32"))]
pub fn try_init_full_nutation(
    loader: &dyn crate::platform::DataLoader,
    path_53a: &str,
    path_53b: &str,
) -> bool {
    match super::load::load_iau2000a(loader, path_53a, path_53b) {
        Ok(iau) => {
            let iau77 = iau.first_n(77);
            set_nutation_77_cache(Some(iau77));
            set_nutation_override(Some(Box::new(move |t| iau.nutation(t))));
            true
        }
        Err(_) => false,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn try_init_full_nutation(
    loader: &dyn crate::platform::DataLoader,
    path_53a: &str,
    path_53b: &str,
) -> bool {
    match super::load::load_iau2000a(loader, path_53a, path_53b) {
        Ok(iau) => {
            let iau77 = iau.first_n(77);
            set_nutation_77_cache(Some(iau77));
            set_nutation_override(Some(Box::new(move |t| iau.nutation(t))));
            true
        }
        Err(_) => false,
    }
}

/// 仅加载 IERS 5.3a+5.3b 前 77 项并设为默认章动（不启用完整表）。
pub fn try_init_nutation(
    loader: &dyn crate::platform::DataLoader,
    path_53a: &str,
    path_53b: &str,
) -> bool {
    match super::load::load_iau2000a(loader, path_53a, path_53b) {
        Ok(iau) => {
            let iau77 = iau.first_n(77);
            set_nutation_77_cache(Some(iau77));
            set_nutation_override(None);
            true
        }
        Err(_) => false,
    }
}

/// 从「repo」加载完整章动并启用（Native=本地文件，Wasm=宿主 set_loader 注入）。
pub fn try_init_full_nutation_from_repo() -> bool {
    match super::load::load_iau2000a_from_repo() {
        Ok(iau) => {
            let iau77 = iau.first_n(77);
            set_nutation_77_cache(Some(iau77));
            set_nutation_override(Some(Box::new(move |t| iau.nutation(t))));
            true
        }
        Err(e) => {
            eprintln!("  [章动] 加载 IERS 5.3a/5.3b 失败: {}（仓库根: {}）", e, crate::repo::repo_root().display());
            false
        }
    }
}

/// 从「repo」仅加载 77 项并设为默认章动。
pub fn try_init_nutation_from_repo() -> bool {
    match super::load::load_iau2000a_from_repo() {
        Ok(iau) => {
            let iau77 = iau.first_n(77);
            set_nutation_77_cache(Some(iau77));
            set_nutation_override(None);
            true
        }
        Err(e) => {
            eprintln!("  [章动] 加载 IERS 5.3a/5.3b 失败: {}（仓库根: {}）", e, crate::repo::repo_root().display());
            false
        }
    }
}

/// 从二进制 buffer 启用完整章动（.bin 或解压后的 .br），并缓存前 77 项。成功返回 true。
pub fn try_init_full_nutation_from_binary(bytes: &[u8]) -> bool {
    match super::load::load_iau2000a_from_binary(bytes) {
        Ok(iau) => {
            let iau77 = iau.first_n(77);
            set_nutation_77_cache(Some(iau77));
            set_nutation_override(Some(Box::new(move |t| iau.nutation(t))));
            true
        }
        Err(_) => false,
    }
}

/// 在闭包内临时使用 77 项章动（用于迭代粗算阶段），执行完后恢复原覆盖。
pub fn with_nutation_77<R, F: FnOnce() -> R>(f: F) -> R {
    NUTATION_OVERRIDE.with(|cell| {
        let prev = RefCell::replace(cell, None);
        let out = f();
        let _ = RefCell::replace(cell, prev);
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn nutation_77_j2000() {
        if !try_init_nutation_from_repo() {
            eprintln!("nutation_77_j2000: skipped (tab5.3a/tab5.3b not found)");
            return;
        }
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
