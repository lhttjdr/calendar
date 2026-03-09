//! 视位置：光行时 → r(tr)=Xproper + 岁差+章动 → 太阳视黄经（与定气参考一致）；另 VSOP87+FK5→ICRS+patch → 太阳 ICRS。
//! 与 LightTime 一致：式(37) Xproper(t)=x(tr)−xE(tr) 已含光行时+光行差，故不再施光行差。
//!
//! **矩阵与向量**：岁差/章动/旋转的矩阵与向量已统一为 `Mat::<Real,3,3>` 与 `[Real; 3]`，整条链标量均为 Real。

use crate::astronomy::constant::J2000;
use crate::astronomy::ephemeris::{Elpmpp02Data, Vsop87};
use crate::astronomy::frame::fk5_icrs;
use crate::astronomy::frame::vsop87_de406_icrs_patch;
use crate::astronomy::frame::nutation::nutation_for_apparent;
use crate::astronomy::frame::nutation::{eps_true_dot, nutation_derivative_times_vector};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::astronomy::frame::precession::{
    mean_obliquity, precession_derivative_times_vector_for, precession_transform_for, PrecessionModel,
};
use crate::astronomy::pipeline::{Body, EphemerisProvider, FrameMapper, LightTimeCorrector, TransformGraph, VsopToDe406IcrsFit};
use crate::math::algebra::mat::Mat;
use crate::math::real::{real_const, real, zero, one, Real, RealOps, ToReal};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{epoch::Epoch, position::Position, reference_frame::ReferenceFrame};

/// 数值导数步长（日），与数值速度步长参考一致。直接 Real。
const NUMERICAL_VELOCITY_DELTA_JD: Real = real_const(0.01);

/// 每日秒数（Real，与标量约定一致）。
const SEC_PER_DAY: Real = real_const(86400.0);
/// 每儒略世纪秒数。
const SEC_PER_CENTURY: Real = real_const(36525.0 * 86400.0);

/// 视位置 pipeline 选项。
#[derive(Clone, Debug)]
pub struct ApparentPipelineOptions {
    /// 定气用 true（P03）；其它场景可用 false（Vondrak2011）。若 `precession_model` 为 Some 则优先使用。
    pub use_p03_precession: bool,
    /// 显式指定岁差模型；None 时由 use_p03_precession 决定（true→P03，false→Vondrak2011）。
    pub precession_model: Option<PrecessionModel>,
    /// 月球视黄经是否施光行时（默认 true）。
    pub use_light_time_moon: bool,
    /// 视黄经速度用解析导数（true）还是数值微分（false）。
    pub use_analytic_velocity: bool,
    /// 质心行星 (Vsop87, M_planet/M_sun)，用于 BCRS 地心速度（光行差等可选）。None = 仅地球。
    pub vsop_barycentric_planets: Option<Vec<(Vsop87, f64)>>,
}

impl Default for ApparentPipelineOptions {
    fn default() -> Self {
        Self {
            use_p03_precession: true,
            precession_model: None,
            use_light_time_moon: false,
            use_analytic_velocity: false,
            vsop_barycentric_planets: None,
        }
    }
}

impl ApparentPipelineOptions {
    /// 定气/视黄经默认：定气用 P03，月球施光行时。
    pub fn pipeline_default() -> Self {
        Self {
            use_p03_precession: true,
            precession_model: None,
            use_light_time_moon: true,
            use_analytic_velocity: false,
            vsop_barycentric_planets: None,
        }
    }

    /// 解析得到的岁差模型（用于 TransformGraph）。
    pub fn effective_precession_model(&self) -> PrecessionModel {
        self.precession_model.unwrap_or(if self.use_p03_precession {
            PrecessionModel::P03
        } else {
            PrecessionModel::Vondrak2011
        })
    }
}

/// 儒略世纪 t = (JD_TT - J2000) / 36525。入参与结果均为 Real。
fn julian_centuries_from_jd(jd_tt: impl ToReal) -> Real {
    (real(jd_tt) - J2000) / real(36525.0)
}

fn rotation_x(angle: PlaneAngle) -> [[Real; 3]; 3] {
    let (c, s) = (angle.rad().cos(), angle.rad().sin());
    [[one(), zero(), zero()], [zero(), c, -s], [zero(), s, c]]
}

fn rotation_x_derivative(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[zero(), zero(), zero()], [zero(), -s, -c], [zero(), c, -s]]
}

fn rotation_z(angle: PlaneAngle) -> [[Real; 3]; 3] {
    let (c, s) = (angle.rad().cos(), angle.rad().sin());
    [[c, -s, zero()], [s, c, zero()], [zero(), zero(), one()]]
}

/// 章动矩阵（被动旋转）：平赤道 → 真赤道，v_true = N · v_mean。供 pipeline 使用。
#[inline]
pub fn nutation_matrix_mean_to_true(t_cent: impl ToReal) -> [[Real; 3]; 3] {
    let t_real = real(t_cent);
    let (dpsi, deps) = nutation_for_apparent(t_real);
    nutation_matrix(mean_obliquity(t_real), dpsi, deps)
}

/// 章动矩阵 N^T（旧接口，保留兼容）。请用 nutation_matrix_mean_to_true 以符合被动旋转约定。
#[inline]
pub fn nutation_matrix_transposed(t_cent: impl ToReal) -> [[Real; 3]; 3] {
    let t_real = real(t_cent);
    let (dpsi, deps) = nutation_for_apparent(t_real);
    let n = nutation_matrix(mean_obliquity(t_real), dpsi, deps);
    Mat::<Real, 3, 3>::from(n).transpose().to_array()
}

/// 章动矩阵 N = R1(ε) R3(-Δψ) R1(-(ε+Δε))（被动：平 → 真）。
fn nutation_matrix(eps_mean: PlaneAngle, dpsi: PlaneAngle, deps: PlaneAngle) -> [[Real; 3]; 3] {
    let r1_eps = Mat::<Real, 3, 3>::from(rotation_x(eps_mean));
    let r3_dpsi = Mat::<Real, 3, 3>::from(rotation_z(-dpsi));
    let eps_sum = eps_mean + deps;
    let r1_eps_deps = Mat::<Real, 3, 3>::from(rotation_x(-eps_sum));
    r1_eps.mul_mat(&r3_dpsi).mul_mat(&r1_eps_deps).to_array()
}

/// 太阳地心位置 in ICRS (GCRF)。管线：EphemerisProvider(Sun) → MeanEcliptic→FK5 → VsopToDe406IcrsFit → position。
/// 太阳在 J2000 赤道架下的位置（地心）。
pub fn sun_position_icrs(vsop: &Vsop87, t: TimePoint) -> Position {
    let jd_tt = t.to_scale(TimeScale::TT).jd;
    let state = vsop.compute_state(Body::Sun, t);
    let graph = TransformGraph::default_graph();
    let state = graph.transform_to(state, ReferenceFrame::FK5, jd_tt);
    let state = VsopToDe406IcrsFit.apply(state, t);
    state.position
}

/// 太阳视黄经（弧度 [0, 2π)）：管线为光行时 → EphemerisProvider(Sun) → FK5 → VsopToDe406IcrsFit → TransformGraph → ApparentEcliptic → λ。与定气参考一致。内部 f64，边界转 R。
pub fn sun_apparent_ecliptic_longitude(vsop: &Vsop87, t: TimePoint) -> PlaneAngle {
    let lam = sun_apparent_ecliptic_longitude_impl(vsop, t, &ApparentPipelineOptions::default()).0;
    lam
}

/// 同上，可指定岁差等选项（定气宜 `use_p03_precession: true`）。
pub fn sun_apparent_ecliptic_longitude_with_options(
    vsop: &Vsop87,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> PlaneAngle {
    let lam = sun_apparent_ecliptic_longitude_impl(vsop, t, options).0;
    lam
}

/// Sun apparent ecliptic longitude velocity (rad/day) via analytic derivative chain. Returns Real.
pub fn sun_apparent_ecliptic_longitude_velocity_analytic(
    vsop: &Vsop87,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Real {
    let t_tt = t.to_scale(TimeScale::TT);
    let corrector: LightTimeCorrector<'_, Vsop87, VsopToDe406IcrsFit> = LightTimeCorrector {
        ephemeris: vsop,
        mapper: None,
        max_iter: 2,
    };
    let (tr, state) = corrector.retarded_state(t_tt, Body::Sun);
    let jd_tr = tr.to_scale(TimeScale::TT).jd;
    let t_cent = julian_centuries_from_jd(jd_tr);
    let precession_model = options.effective_precession_model();
    let (pos_m, vel_m) = state.to_meters_and_m_per_s();
    let eps0 = mean_obliquity(0.0).rad();
    let (ce, se) = (eps0.cos(), eps0.sin());
    let (xi, yi, zi) = fk5_icrs::rotate_equatorial(
        pos_m[0],
        pos_m[1] * ce - pos_m[2] * se,
        pos_m[1] * se + pos_m[2] * ce,
    );
    let (vxi, vyi, vzi) = fk5_icrs::rotate_equatorial(
        vel_m[0],
        vel_m[1] * ce - vel_m[2] * se,
        vel_m[1] * se + vel_m[2] * ce,
    );
    let pos_icrs = Position::from_si_meters_in_frame(ReferenceFrame::ICRS, xi, yi, zi);
    let (pos_c, vel_c) =
        vsop87_de406_icrs_patch::apply_patch_velocity_to_equatorial_for_geocentric_sun(
            pos_icrs,
            [vxi, vyi, vzi],
            &tr,
        );
    let (r0, r1, r2) = fk5_icrs::rotate_equatorial_icrs_to_fk5(
        pos_c.x.meters(),
        pos_c.y.meters(),
        pos_c.z.meters(),
    );
    let (v0, v1, v2) = fk5_icrs::rotate_equatorial_icrs_to_fk5(vel_c[0], vel_c[1], vel_c[2]);
    let pt = precession_transform_for(t_cent, precession_model);
    let r_fk5 = [r0, r1, r2];
    let pos_me = pt.apply_vec(r_fk5);
    let v_fk5 = [v0, v1, v2];
    let dpr = precession_derivative_times_vector_for(r_fk5, t_cent, precession_model);
    let v_me_rot = pt.apply_vec(v_fk5);
    let vel_me = [
        v_me_rot[0] + dpr[0] / SEC_PER_CENTURY,
        v_me_rot[1] + dpr[1] / SEC_PER_CENTURY,
        v_me_rot[2] + dpr[2] / SEC_PER_CENTURY,
    ];
    let n_t = nutation_matrix_transposed(t_cent);
    let pos_te = Mat::<Real, 3, 3>::from(n_t).mul_vec(pos_me);
    let dnr = nutation_derivative_times_vector(pos_me, t_cent, precession_model);
    let v_te_rot = Mat::<Real, 3, 3>::from(n_t).mul_vec(vel_me);
    let vel_te = [
        v_te_rot[0] + dnr[0] / SEC_PER_CENTURY,
        v_te_rot[1] + dnr[1] / SEC_PER_CENTURY,
        v_te_rot[2] + dnr[2] / SEC_PER_CENTURY,
    ];
    let (_, deps) = nutation_for_apparent(t_cent);
    let eps_true = mean_obliquity(t_cent).rad() + deps.rad();
    let r1_eps = rotation_x(PlaneAngle::from_rad(eps_true));
    let r1p_eps = rotation_x_derivative(eps_true);
    let eps_td = eps_true_dot(t_cent, precession_model);
    let pos_ae = Mat::<Real, 3, 3>::from(r1_eps).mul_vec(pos_te);
    let vel_ae_raw = Mat::<Real, 3, 3>::from(r1_eps).mul_vec(vel_te);
    let r1p_pos = Mat::<Real, 3, 3>::from(r1p_eps).mul_vec(pos_te);
    let vel_ae = [
        vel_ae_raw[0] + r1p_pos[0] * eps_td / SEC_PER_CENTURY,
        vel_ae_raw[1] + r1p_pos[1] * eps_td / SEC_PER_CENTURY,
        vel_ae_raw[2] + r1p_pos[2] * eps_td / SEC_PER_CENTURY,
    ];
    let x = pos_ae[0];
    let y = pos_ae[1];
    let xy2 = x * x + y * y;
    if xy2 <= zero() {
        return real_const(0.0);
    }
    SEC_PER_DAY * (x * vel_ae[1] - y * vel_ae[0]) / xy2
}

/// 太阳视黄经对时间的导数（rad/日）。返回 Real，与标量约定一致；需要 f64 时调用方 `.as_f64()`。
pub fn sun_apparent_ecliptic_longitude_velocity(
    vsop: &Vsop87,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Real {
    if options.use_analytic_velocity {
        return sun_apparent_ecliptic_longitude_velocity_analytic(vsop, t, options);
    }
    sun_apparent_ecliptic_longitude_velocity_numerical(vsop, t, options)
}

fn sun_apparent_ecliptic_longitude_velocity_numerical(
    vsop: &Vsop87,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Real {
    let jd = t.to_scale(TimeScale::TT).jd;
    let t_lo = TimePoint::new(TimeScale::TT, jd - NUMERICAL_VELOCITY_DELTA_JD);
    let t_hi = TimePoint::new(TimeScale::TT, jd + NUMERICAL_VELOCITY_DELTA_JD);
    let lam_lo = sun_apparent_ecliptic_longitude_with_options(vsop, t_lo, options).rad();
    let lam_hi = sun_apparent_ecliptic_longitude_with_options(vsop, t_hi, options).rad();
    let (lo, hi) = (lam_lo.as_f64(), lam_hi.as_f64());
    let (lam_lo_f, mut lam_hi_f) = (lo, hi);
    if lam_hi_f - lam_lo_f > core::f64::consts::PI {
        lam_hi_f -= core::f64::consts::TAU;
    } else if lam_hi_f - lam_lo_f < -core::f64::consts::PI {
        lam_hi_f += core::f64::consts::TAU;
    }
    real_const((lam_hi_f - lam_lo_f) / (2.0 * NUMERICAL_VELOCITY_DELTA_JD.as_f64()))
}

/// 月球视黄经对时间的导数（rad/日）。返回 Real；需要 f64 时调用方 `.as_f64()`。
/// vsop 保留供日后质心/BCRS 选项使用。
pub fn moon_apparent_ecliptic_longitude_velocity(
    elp: &Elpmpp02Data,
    _vsop: &Vsop87,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Real {
    if options.use_analytic_velocity {
        return moon_apparent_ecliptic_longitude_velocity_analytic(elp, t, options);
    }
    moon_apparent_ecliptic_longitude_velocity_numerical(elp, t, options)
}

fn moon_apparent_ecliptic_longitude_velocity_analytic(
    elp: &Elpmpp02Data,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Real {
    let t_tt = t.to_scale(TimeScale::TT);
    let (state, jd_work) = if options.use_light_time_moon {
        let corrector: LightTimeCorrector<'_, Elpmpp02Data, VsopToDe406IcrsFit> = LightTimeCorrector {
            ephemeris: elp,
            mapper: None,
            max_iter: 2,
        };
        let (tr, s) = corrector.retarded_state(t_tt, Body::Moon);
        (s, tr.to_scale(TimeScale::TT).jd)
    } else {
        (elp.compute_state(Body::Moon, t), t_tt.jd)
    };
    let t_cent = julian_centuries_from_jd(jd_work);
    let precession_model = options.effective_precession_model();
    let (pos_m, vel_m) = state.to_meters_and_m_per_s();
    let eps0 = mean_obliquity(0.0).rad();
    let (ce, se) = (eps0.cos(), eps0.sin());
    let r_fk5 = [
        pos_m[0],
        pos_m[1] * ce - pos_m[2] * se,
        pos_m[1] * se + pos_m[2] * ce,
    ];
    let v_fk5 = [
        vel_m[0],
        vel_m[1] * ce - vel_m[2] * se,
        vel_m[1] * se + vel_m[2] * ce,
    ];
    let pt = precession_transform_for(t_cent, precession_model);
    let pos_me = pt.apply_vec(r_fk5);
    let dpr = precession_derivative_times_vector_for(r_fk5, t_cent, precession_model);
    let v_me_rot = pt.apply_vec(v_fk5);
    let vel_me = [
        v_me_rot[0] + dpr[0] / SEC_PER_CENTURY,
        v_me_rot[1] + dpr[1] / SEC_PER_CENTURY,
        v_me_rot[2] + dpr[2] / SEC_PER_CENTURY,
    ];
    let n_t = nutation_matrix_transposed(t_cent);
    let pos_te = Mat::<Real, 3, 3>::from(n_t).mul_vec(pos_me);
    let dnr = nutation_derivative_times_vector(pos_me, t_cent, precession_model);
    let v_te_rot = Mat::<Real, 3, 3>::from(n_t).mul_vec(vel_me);
    let vel_te = [
        v_te_rot[0] + dnr[0] / SEC_PER_CENTURY,
        v_te_rot[1] + dnr[1] / SEC_PER_CENTURY,
        v_te_rot[2] + dnr[2] / SEC_PER_CENTURY,
    ];
    let (_, deps) = nutation_for_apparent(t_cent);
    let eps_true = mean_obliquity(t_cent).rad() + deps.rad();
    let r1_eps = rotation_x(PlaneAngle::from_rad(eps_true));
    let r1p_eps = rotation_x_derivative(eps_true);
    let eps_td = eps_true_dot(t_cent, precession_model);
    let pos_ae = Mat::<Real, 3, 3>::from(r1_eps).mul_vec(pos_te);
    let vel_ae_raw = Mat::<Real, 3, 3>::from(r1_eps).mul_vec(vel_te);
    let r1p_pos = Mat::<Real, 3, 3>::from(r1p_eps).mul_vec(pos_te);
    let vel_ae = [
        vel_ae_raw[0] + r1p_pos[0] * eps_td / SEC_PER_CENTURY,
        vel_ae_raw[1] + r1p_pos[1] * eps_td / SEC_PER_CENTURY,
        vel_ae_raw[2] + r1p_pos[2] * eps_td / SEC_PER_CENTURY,
    ];
    let x = pos_ae[0];
    let y = pos_ae[1];
    let xy2 = x * x + y * y;
    if xy2 <= zero() {
        return real_const(0.0);
    }
    SEC_PER_DAY * (x * vel_ae[1] - y * vel_ae[0]) / xy2
}

fn moon_apparent_ecliptic_longitude_velocity_numerical(
    elp: &Elpmpp02Data,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Real {
    let jd = t.to_scale(TimeScale::TT).jd;
    let t_lo = TimePoint::new(TimeScale::TT, jd - NUMERICAL_VELOCITY_DELTA_JD);
    let t_hi = TimePoint::new(TimeScale::TT, jd + NUMERICAL_VELOCITY_DELTA_JD);
    let lam_lo = moon_apparent_ecliptic_longitude_with_options(elp, t_lo, options).rad();
    let lam_hi = moon_apparent_ecliptic_longitude_with_options(elp, t_hi, options).rad();
    let mut diff = lam_hi - lam_lo;
    if diff > real(core::f64::consts::PI) {
        diff -= real(core::f64::consts::TAU);
    } else if diff < real(-core::f64::consts::PI) {
        diff += real(core::f64::consts::TAU);
    }
    diff / (real_const(2.0) * NUMERICAL_VELOCITY_DELTA_JD)
}

/// 同 pipeline 的中间量，便于诊断 (Δψ, Δε, P 对角, ε_mean, ε_true, λ)。
#[derive(Clone, Debug)]
pub struct ApparentSunDiagnostic {
    /// 儒略世纪 t = (JD_TT_ret - J2000)/36525（光行时后的时刻）
    pub t_cent: f64,
    /// 章动 Δψ
    pub dpsi: PlaneAngle,
    /// 章动 Δε
    pub deps: PlaneAngle,
    /// 岁差矩阵 P 对角元 [P00, P11, P22]
    pub precession_diag: [f64; 3],
    /// 平黄赤交角 ε_mean
    pub eps_mean: PlaneAngle,
    /// 真黄赤交角 ε_true = ε_mean + Δε
    pub eps_true: PlaneAngle,
    /// 平黄经（仅岁差 + 平黄赤交角，无章动）[0, 2π)，用于定位系统差来源
    pub lambda_mean_ecliptic: PlaneAngle,
    /// 视黄经 λ [0, 2π)
    pub lambda: PlaneAngle,
}

/// 月球视黄经（弧度 [0, 2π)）：管线为 EphemerisProvider(Moon) → 可选光行时 → TransformGraph → ApparentEcliptic → λ。内部 f64，边界转 R。
pub fn moon_apparent_ecliptic_longitude(elp: &Elpmpp02Data, t: TimePoint) -> PlaneAngle {
    moon_apparent_ecliptic_longitude_with_options(elp, t, &ApparentPipelineOptions::default())
}

/// 同上，可指定光行时、岁差等选项。管线：EphemerisProvider(Moon) → 可选光行时 → TransformGraph → ApparentEcliptic → λ。
pub fn moon_apparent_ecliptic_longitude_with_options(
    elp: &Elpmpp02Data,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> PlaneAngle {
    let t_tt = t.to_scale(TimeScale::TT);
    let (state, jd_work) = if options.use_light_time_moon {
        let corrector: LightTimeCorrector<'_, Elpmpp02Data, VsopToDe406IcrsFit> = LightTimeCorrector {
            ephemeris: elp,
            mapper: None,
            max_iter: 2,
        };
        let (tr, state) = corrector.retarded_state(t_tt, Body::Moon);
        (state, tr.to_scale(TimeScale::TT).jd)
    } else {
        let state = elp.compute_state(Body::Moon, t_tt);
        (state, t_tt.jd)
    };
    let precession_model = options.effective_precession_model();
    let graph = TransformGraph::default_graph().with_precession_model(precession_model);
    let epoch = Epoch::new(jd_work);
    let state = graph.transform_to(state, ReferenceFrame::ApparentEcliptic(epoch), jd_work);
    let rad = state.to_spherical().lon.rad().wrap_to_2pi();
    PlaneAngle::from_rad(rad)
}

/// 返回 (λ, diagnostic)。诊断用，可对比 dpsi/deps/P/ε/λ。
pub fn sun_apparent_ecliptic_longitude_diagnostic(vsop: &Vsop87, t: TimePoint) -> (PlaneAngle, ApparentSunDiagnostic) {
    sun_apparent_ecliptic_longitude_impl(vsop, t, &ApparentPipelineOptions::default())
}

fn sun_apparent_ecliptic_longitude_impl(
    vsop: &Vsop87,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> (PlaneAngle, ApparentSunDiagnostic) {
    let t_tt = t.to_scale(TimeScale::TT);
    let corrector: LightTimeCorrector<'_, Vsop87, VsopToDe406IcrsFit> = LightTimeCorrector {
        ephemeris: vsop,
        mapper: None,
        max_iter: 2,
    };
    let (tr, state) = corrector.retarded_state(t_tt, Body::Sun);
    let jd_tr = tr.to_scale(TimeScale::TT).jd;
    let t_cent = julian_centuries_from_jd(jd_tr);
    let precession_model = options.effective_precession_model();
    let graph = TransformGraph::default_graph().with_precession_model(precession_model);
    let state = graph.transform_to(state, ReferenceFrame::FK5, jd_tr);
    let state = VsopToDe406IcrsFit.apply(state, tr);
    let epoch = Epoch::new(jd_tr);
    let (dpsi, deps) = nutation_for_apparent(t_cent);
    let eps_mean = mean_obliquity(t_cent).rad();
    let eps_true = eps_mean + deps.rad();

    // 平黄经：仅岁差 → 平赤道，再按平黄赤交角转到平黄道（赤道→黄道 R1(-ε)）
    let state_me = graph.transform_to(state, ReferenceFrame::MeanEquator(epoch), jd_tr);
    let [x, y, z] = state_me.position.to_meters();
    let (c, s) = (eps_mean.cos(), eps_mean.sin());
    let y_ecl = y * c + z * s;
    let x_ecl = x;
    let lambda_mean_ecliptic = PlaneAngle::from_rad(
        real(y_ecl).atan2(real(x_ecl)).wrap_to_2pi(),
    );

    let state = graph.transform_to(state_me, ReferenceFrame::ApparentEcliptic(epoch), jd_tr);
    let lambda = PlaneAngle::from_rad(state.to_spherical().lon.rad().wrap_to_2pi());

    let precession = precession_transform_for(t_cent, precession_model);
    let diag = ApparentSunDiagnostic {
        t_cent: t_cent.as_f64(),
        dpsi,
        deps,
        precession_diag: [precession.matrix[0][0].as_f64(), precession.matrix[1][1].as_f64(), precession.matrix[2][2].as_f64()],
        eps_mean: PlaneAngle::from_rad(eps_mean),
        eps_true: PlaneAngle::from_rad(eps_true),
        lambda_mean_ecliptic,
        lambda,
    };
    (lambda, diag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::time::j2000_tt;
    use crate::math::real::real;

    #[test]
    fn sun_apparent_near_j2000() {
        let vsop = crate::astronomy::ephemeris::vsop87::minimal_earth_vsop();
        let lam = sun_apparent_ecliptic_longitude(&vsop, j2000_tt());
        assert!(lam.rad() >= real(0) && lam.rad() < real(core::f64::consts::TAU));
    }
}
