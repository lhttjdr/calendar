//! ELP-MPP02 月球历表。
//!
//! **当前实现**：仅平均根数（无周期/泊松项）求位置与速度，用于定朔等；常数为 DE405。
//!
//! **完整 ELPMPP02**：解析 ELP_MAIN.S1/S2/S3、ELP_PERT.S1/S2/S3，求值 6 组周期/泊松项；
//! `load()` 加载后可用 `position_velocity()` 得完整位置与速度。

pub mod parse;
pub mod parse_constants;
pub mod table7;

#[cfg(all(test, not(target_arch = "wasm32"), feature = "python-test"))]
mod tests_jpl_python;

use crate::astronomy::constant::J2000;
use crate::astronomy::time::TimePoint;
use crate::math::algebra::mat::Mat;
use crate::math::angle::dms2rad;
use crate::quantity::rotation_matrix::RotationMatrix;
use crate::quantity::{angular_rate::AngularRate, angle::PlaneAngle, epoch::Epoch, julian_centuries::JulianCenturies, length::Length, position::Position, reference_frame::ReferenceFrame, speed::Speed, unit::{AngularRateUnit, LengthUnit, SpeedUnit}, vector3::Vector3, velocity::Velocity};
use crate::math::real::{real_const, real, zero, Real, ToReal};
use crate::math::series::{arcsec_to_rad, power_series_at_real, power_series_derivative_at_real};

pub use parse::{load_all, load_all_from_binary, split_fortran_main, terms_from_binary, terms_to_binary};

/// 从数据目录加载 ELPMPP02；base_path 下需有 ELP_MAIN.S1/S2/S3、ELP_PERT.S1/S2/S3。
pub fn load(
    loader: &dyn crate::platform::DataLoader,
    base_path: &str,
    correction: Elpmpp02Correction,
) -> Result<Elpmpp02Data, crate::platform::LoadError> {
    load_all(loader, base_path, correction)
}
pub use parse_constants::{de405, ParseConstants};

/// 儒略世纪天数
const DAYS_PER_JULIAN_CENTURY: f64 = 36525.0;

/// ELPMPP02 常数：平均经度系数（弧度）、Laskar P/Q、地月参考距离、比例。标量统一 Real，距离用 Length。
/// 仅平均根数时：lon = power_series_at_real(longitude_lunar1, T)，r = a0（pv3p=1）。
#[derive(Clone, Debug)]
pub struct Elpmpp02Constants {
    /// 平均经度 W1 的幂级数系数（rad, rad/cy, ...），1,T,T²,T³,T⁴
    pub longitude_lunar1: [Real; 5],
    /// Laskar P 多项式系数（无量纲），0..6 对应 1,T,...,T⁵
    pub laskar_p: [Real; 6],
    /// Laskar Q 多项式系数（无量纲）
    pub laskar_q: [Real; 6],
    /// 地月参考距离（ELP 平均距离→km 的换算），r_km = pv3p_elp * a0.km()
    pub a0: Length,
    /// ra0 = a0_de405/a0_elp，完整求值时 r_km = pv3p * ra0
    pub ra0: Real,
}

impl Elpmpp02Constants {
    /// DE405 拟合常数（论文 Table 3 Col.2 + Table 6 长期项），DE405 拟合常数。全程 Real。
    pub fn de405() -> Self {
        let lon1_0 = dms2rad(218.0, 18.0, 59.95571) + arcsec_to_rad(-0.07008);
        let lon1_1 = arcsec_to_rad(1732559343.73604 - 0.35106);
        let lon1_2 = arcsec_to_rad(-6.8084 - 0.03743);
        let lon1_3 = arcsec_to_rad(0.66040e-2) + arcsec_to_rad(-0.00018865);
        let lon1_4 = arcsec_to_rad(-0.31690e-4) + arcsec_to_rad(-0.00001024);
        Elpmpp02Constants {
            longitude_lunar1: [lon1_0, lon1_1, lon1_2, lon1_3, lon1_4],
            laskar_p: [
                real_const(0.0),
                real_const(0.10180391e-04),
                real_const(0.47020439e-06),
                real_const(-0.5417367e-09),
                real_const(-0.2507948e-11),
                real_const(0.463486e-14),
            ],
            laskar_q: [
                real_const(0.0),
                real_const(-0.113469002e-03),
                real_const(0.12372674e-06),
                real_const(0.1265417e-08),
                real_const(-0.1371808e-11),
                real_const(-0.320334e-14),
            ],
            a0: Length::from_value(real_const(384747.9613701725), LengthUnit::Kilometer),
            ra0: real_const(384747.9613701725 / 384747.980674318),
        }
    }
}

/// 历表拟合类型：与哪套行星历表/观测一致（用于缓存键；DE406 与 DE405 使用相同常数，见论文 §4.3.3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elpmpp02Correction {
    LLR,
    DE405,
    DE406,
}

/// 仅常数、无周期/泊松项的数据，用于“仅平均根数”求位置
#[derive(Clone, Debug)]
pub struct Elpmpp02Data {
    pub period_v: Vec<Elpmpp02Term>,
    pub period_u: Vec<Elpmpp02Term>,
    pub period_r: Vec<Elpmpp02Term>,
    pub poisson_v: Vec<Elpmpp02Term>,
    pub poisson_u: Vec<Elpmpp02Term>,
    pub poisson_r: Vec<Elpmpp02Term>,
    pub constants: Elpmpp02Constants,
    pub correction: Elpmpp02Correction,
}

impl Elpmpp02Data {
    pub fn de405_mean_only() -> Self {
        Self {
            period_v: Vec::new(),
            period_u: Vec::new(),
            period_r: Vec::new(),
            poisson_v: Vec::new(),
            poisson_u: Vec::new(),
            poisson_r: Vec::new(),
            constants: Elpmpp02Constants::de405(),
            correction: Elpmpp02Correction::DE405,
        }
    }

    /// 论文 §4.3.3：DE406 与 DE405 使用相同常数。
    pub fn de406_mean_only() -> Self {
        Self {
            period_v: Vec::new(),
            period_u: Vec::new(),
            period_r: Vec::new(),
            poisson_v: Vec::new(),
            poisson_u: Vec::new(),
            poisson_r: Vec::new(),
            constants: Elpmpp02Constants::de405(),
            correction: Elpmpp02Correction::DE406,
        }
    }

    /// 是否仅平均根数（无周期/泊松项）
    pub fn is_mean_only(&self) -> bool {
        self.period_v.is_empty()
    }
}

/// 单条历表项（完整解析用）：Ci 振幅（角秒）、Fi 相位多项式系数（弧度）、alpha 为 T 的幂次、ilu 为 Delaunay 系数。
#[derive(Clone, Debug)]
pub struct Elpmpp02Term {
    /// 振幅（角秒），物理量
    pub ci: PlaneAngle,
    /// 相位多项式系数（弧度）
    pub fi: Vec<Real>,
    pub alpha: i32,
    pub ilu: [i32; 4],
}

/// Laskar 矩阵 P（3×3），中间系→J2000 平黄道。返回无量纲旋转矩阵。
///
/// * `pw`：Laskar P 多项式在时刻 T 的值，即 P(T)；`c.laskar_p` 的幂级数在 t_cy 处求值。
/// * `qw`：Laskar Q 多项式在时刻 T 的值，即 Q(T)；`c.laskar_q` 的幂级数在 t_cy 处求值。
/// P、Q 为黄道长期进动（行星岁差/拉普拉斯平面）的小参数，由 (pw, qw) 唯一确定中间黄道相对 J2000 平黄道的旋转。
fn laskar_p_matrix(pw: impl ToReal, qw: impl ToReal) -> RotationMatrix {
    let (pw, qw) = (real(pw), real(qw));
    let one = real_const(1.0);
    let two = real_const(2.0);
    let inner = one - pw * pw - qw * qw;
    let inner_clamped = inner.max(real_const(1e-20));
    let ra = two * inner_clamped.sqrt();
    RotationMatrix::from_array([
        [one - two * pw * pw, two * pw * qw, pw * ra],
        [two * pw * qw, one - two * qw * qw, -qw * ra],
        [-pw * ra, qw * ra, (one - two * pw * pw) + (one - two * qw * qw) - one],
    ])
}

/// 仅用平均根数求月球在 J2000 平黄道直角系下的位置与速度。标量 Real。
pub fn position_velocity(data: &Elpmpp02Data, t: TimePoint) -> (Position, Velocity) {
    position_velocity_with_max_terms_impl(data, t, None)
}

/// 同上，可限制 ELP 级数项数。None = 全部项。
pub fn position_velocity_with_max_terms(
    data: &Elpmpp02Data,
    t: TimePoint,
    max_terms: Option<u32>,
) -> (Position, Velocity) {
    position_velocity_with_max_terms_impl(data, t, max_terms)
}

fn position_velocity_with_max_terms_impl(
    data: &Elpmpp02Data,
    t: TimePoint,
    max_terms: Option<u32>,
) -> (Position, Velocity) {
    if data.is_mean_only() {
        position_velocity_mean_only_impl(data, t)
    } else {
        position_velocity_full(data, t, max_terms)
    }
}

/// 仅平均根数。
pub fn position_velocity_mean_only(data: &Elpmpp02Data, t: TimePoint) -> (Position, Velocity) {
    position_velocity_mean_only_impl(data, t)
}

fn position_velocity_mean_only_impl(data: &Elpmpp02Data, t: TimePoint) -> (Position, Velocity) {
    let epoch_tt = Epoch::new(t.to_scale(crate::astronomy::time::TimeScale::TT).jd);
    let t_cy: JulianCenturies = epoch_tt.offset_in_julian_centuries(J2000, real(DAYS_PER_JULIAN_CENTURY));
    let t_cy_r = t_cy.value();
    let c = &data.constants;

    let lon = PlaneAngle::from_rad(power_series_at_real(&c.longitude_lunar1, t_cy_r));
    let lat = PlaneAngle::from_rad(zero());
    let pos_i = Position::from_spherical_in_frame(
        ReferenceFrame::Elpmpp02MeanLunar,
        lon,
        lat,
        c.a0,
    );

    let pw = power_series_at_real(&c.laskar_p, t_cy_r);
    let qw = power_series_at_real(&c.laskar_q, t_cy_r);
    let p = laskar_p_matrix(pw, qw);
    let frame_j2000 = ReferenceFrame::MeanEcliptic(Epoch::j2000());
    let pos_j2000 = pos_i.apply_transform(frame_j2000, |v| p.mul_vec(v));

    let lon_dot = AngularRate::from_value(
        power_series_derivative_at_real(&c.longitude_lunar1, t_cy_r),
        AngularRateUnit::RadPerJulianCentury,
    );
    // 中间架下 v = ω × r（仅经向角速，纬向为 0）：v_x = -ω·y, v_y = ω·x, v_z = 0
    let vel_i = Vector3::from_speeds([
        (-lon_dot) * pos_i.y,
        lon_dot * pos_i.x,
        Speed::from_value(zero(), SpeedUnit::MPerS),
    ]);
    let vel_j2000 = p.mul_vec_vec3(&vel_i);
    (
        pos_j2000,
        Velocity::from_speeds_in_frame(frame_j2000, vel_j2000),
    )
}

/// Table 6 相位修正系数（角秒），与论文下标一致：d_ω₂,₂、d_ω₂,₃、d_ω₃,₂、d_ω₃,₃。DE405/406 用。
struct Table6Dw {
    d_w2_2: PlaneAngle,
    d_w2_3: PlaneAngle,
    d_w3_2: PlaneAngle,
    d_w3_3: PlaneAngle,
}

fn table6_d_w() -> Table6Dw {
    Table6Dw {
        d_w2_2: PlaneAngle::from_arcsec(real(0.00470602)),
        d_w2_3: PlaneAngle::from_arcsec(real(-0.00025213)),
        d_w3_2: PlaneAngle::from_arcsec(real(-0.00261070)),
        d_w3_3: PlaneAngle::from_arcsec(real(-0.00010712)),
    }
}

/// Table 6 相位修正，DE405/DE406 时应用。系数与 T²、T³ 的乘法用 [PlaneAngle](PlaneAngle) 带量纲计算。
///
/// **参数物理意义**：
/// - `ilu`：该历表项的 [Delaunay 系数](Elpmpp02Term) 四元组 [i_D, i_l, i_l′, i_F]，即相位中 Delaunay 角组合（D·τ + i_l·l + i_l′·l′ + i_F·F）的整数系数；本公式只用 `ilu[1]`(i_F)、`ilu[2]`(i_l) 参与 Table 6 的系数。
/// - `t_cy`：[儒略世纪数](JulianCenturies)，相对 J2000 的无量纲时间 T，历表幂级数自变量。
fn table6_phase_correction_rad(
    correction: Elpmpp02Correction,
    ilu: &[i32; 4],
    t_cy: JulianCenturies,
) -> PlaneAngle {
    if correction != Elpmpp02Correction::DE405 && correction != Elpmpp02Correction::DE406 {
        return PlaneAngle::from_rad(zero());
    }
    let d = table6_d_w();
    let i_f = real(ilu[1]);
    let i_l = real(ilu[2]);
    let coeff1 = i_f * (-d.d_w3_2) + i_l * (-d.d_w2_2);
    let coeff2 = i_f * (-d.d_w3_3) + i_l * (-d.d_w2_3);
    let t_cy_r = t_cy.value();
    let t2 = t_cy_r * t_cy_r;
    coeff1 * t2 + coeff2 * (t2 * t_cy_r)
}

/// Table 6 相位修正对时间的导数（角速度），单位 rad/世纪。系数与 T 的乘法用 [PlaneAngle](PlaneAngle) 带量纲计算，最后转为 [AngularRate](AngularRate)。
/// 参数物理意义同 [table6_phase_correction_rad]。
fn table6_phase_correction_derivative_rad(
    correction: Elpmpp02Correction,
    ilu: &[i32; 4],
    t_cy: JulianCenturies,
) -> AngularRate {
    if correction != Elpmpp02Correction::DE405 && correction != Elpmpp02Correction::DE406 {
        return AngularRate::from_value(zero(), AngularRateUnit::RadPerJulianCentury);
    }
    let d = table6_d_w();
    let i_f = real(ilu[1]);
    let i_l = real(ilu[2]);
    let coeff1 = i_f * (-d.d_w3_2) + i_l * (-d.d_w2_2);
    let coeff2 = i_f * (-d.d_w3_3) + i_l * (-d.d_w2_3);
    let total_angle = coeff1 * (real(2.0) * t_cy) + coeff2 * (real(3.0) * t_cy * t_cy);
    AngularRate::from_value(total_angle.rad(), AngularRateUnit::RadPerJulianCentury)
}

/// 相位多项式 fi·alpha_t 在 Real 下求值。
fn phase_at_real(fi: &[Real], alpha_t: &[Real]) -> Real {
    let n = fi.len().min(alpha_t.len());
    (0..n).fold(zero(), |s, i| s + fi[i] * alpha_t[i])
}

/// 单组周期+泊松求值：(p, v) 角秒与角秒/世纪。t_cy 为 [JulianCenturies](JulianCenturies)，无量纲直接参与计算（[ToReal] 转 Real）。
fn calculate_component(
    t_cy: JulianCenturies,
    alpha_t: &[Real; 5],
    period: &[Elpmpp02Term],
    poisson: &[Elpmpp02Term],
    correction: Elpmpp02Correction,
    max_terms: Option<usize>,
) -> (Real, Real) {
    let mut sum_p = zero();
    let mut sum_v = zero();
    let take_p = max_terms.unwrap_or(period.len());
    let take_po = max_terms.unwrap_or(poisson.len());
    let alpha_slice: &[Real] = alpha_t.as_slice();
    for t in period.iter().take(take_p) {
        let y = phase_at_real(&t.fi, alpha_slice) + table6_phase_correction_rad(correction, &t.ilu, t_cy).rad();
        sum_p = sum_p + t.ci.arcsec() * y.sin();
        let yp = power_series_derivative_at_real(&t.fi, t_cy)
            + table6_phase_correction_derivative_rad(correction, &t.ilu, t_cy).in_unit(AngularRateUnit::RadPerJulianCentury);
        sum_v = sum_v + t.ci.arcsec() * y.cos() * yp;
    }
    for t in poisson.iter().take(take_po) {
        let y = phase_at_real(&t.fi, alpha_slice) + table6_phase_correction_rad(correction, &t.ilu, t_cy).rad();
        let alpha_val = if t.alpha >= 0 && (t.alpha as usize) < alpha_t.len() {
            alpha_t[t.alpha as usize]
        } else {
            zero()
        };
        let x = t.ci.arcsec() * alpha_val;
        sum_p = sum_p + x * y.sin();
        let yp = power_series_derivative_at_real(&t.fi, t_cy)
            + table6_phase_correction_derivative_rad(correction, &t.ilu, t_cy).in_unit(AngularRateUnit::RadPerJulianCentury);
        let alpha_prev = if t.alpha > 0 && ((t.alpha - 1) as usize) < alpha_t.len() {
            alpha_t[(t.alpha - 1) as usize]
        } else {
            zero()
        };
        let xp = if t.alpha == 0 {
            zero()
        } else {
            real(t.alpha) * t.ci.arcsec() * alpha_prev
        };
        sum_v = sum_v + xp * y.sin() + x * yp * y.cos();
    }
    (sum_p, sum_v)
}

/// 完整求值：6 组级数 → 球面（物理量）→ Position + 球面速度 → Laskar 旋转 → Position, Velocity。
fn position_velocity_full(
    data: &Elpmpp02Data,
    t: TimePoint,
    max_terms: Option<u32>,
) -> (Position, Velocity) {
    let epoch_tt = Epoch::new(t.to_scale(crate::astronomy::time::TimeScale::TT).jd);
    let t_cy: JulianCenturies = epoch_tt.offset_in_julian_centuries(J2000, real(DAYS_PER_JULIAN_CENTURY));
    let alpha_t: [Real; 5] = [real(1), real(t_cy), t_cy * t_cy, t_cy * t_cy * t_cy, t_cy * t_cy * t_cy * t_cy];
    let mt = max_terms.map(|u| u as usize);
    let (pv1p, pv1v) = calculate_component(
        t_cy,
        &alpha_t,
        &data.period_v,
        &data.poisson_v,
        data.correction,
        mt,
    );
    let (pv2p, pv2v) = calculate_component(
        t_cy,
        &alpha_t,
        &data.period_u,
        &data.poisson_u,
        data.correction,
        mt,
    );
    let (pv3p, pv3v) = calculate_component(
        t_cy,
        &alpha_t,
        &data.period_r,
        &data.poisson_r,
        data.correction,
        mt,
    );
    let c = &data.constants;

    let lon = PlaneAngle::from_rad(arcsec_to_rad(pv1p) + power_series_at_real(&c.longitude_lunar1, t_cy));
    let lat = PlaneAngle::from_rad(arcsec_to_rad(pv2p));
    let r = Length::from_value(pv3p * c.ra0, LengthUnit::Kilometer);
    let lon_dot = AngularRate::from_value(
        arcsec_to_rad(pv1v) + power_series_derivative_at_real(&c.longitude_lunar1, t_cy),
        AngularRateUnit::RadPerJulianCentury,
    );
    let lat_dot = AngularRate::from_value(arcsec_to_rad(pv2v), AngularRateUnit::RadPerJulianCentury);
    let r_dot = Speed::from_value(pv3v, SpeedUnit::KmPerJulianCentury);

    let pos_i = Position::from_spherical_in_frame(
        ReferenceFrame::Elpmpp02MeanLunar,
        lon,
        lat,
        r,
    );
    let vel_i = crate::quantity::spherical::spherical_to_cartesian_velocity(
        lon, lat, r, lon_dot, lat_dot, r_dot,
    );

    let pw = power_series_at_real(&c.laskar_p, t_cy);
    let qw = power_series_at_real(&c.laskar_q, t_cy);
    let p = laskar_p_matrix(pw, qw);
    let frame = ReferenceFrame::MeanEcliptic(Epoch::j2000());
    let pos_j2000 = pos_i.apply_transform(frame, |v| p.mul_vec(v));

    let pos_vec = Vector3::from_lengths([pos_i.x, pos_i.y, pos_i.z]);
    let vel_j2000 = laskar_v_mul(pw, qw, t_cy, c, vel_i, pos_vec);
    (
        pos_j2000,
        Velocity::from_speeds_in_frame(frame, vel_j2000),
    )
}

/// Laskar 速度变换：V (3×6) 将月心平架下的 [速度矢量, 位置矢量] 变为平黄道架下的速度矢量。
///
/// **参数物理意义**：
/// - `pw`, `qw`：Laskar P(T)、Q(T)，月球平均赤道/自转的无量纲参数（Cayley-Klein 型），在时刻 T 的取值。
/// - `t_cy`：[儒略世纪](JulianCenturies) T，历表自变量。
/// - `vel_i`：月心平架下速度矢量。
/// - `pos_i`：月心平架下位置矢量（与 `vel_i` 同架）。并成 6 维即**状态向量 (r,v)**（相空间/状态空间中的 (位置, 速度)）。
///
/// **中间量**：`ppw`/`qpw` = dP/dT、dQ/dT；`ra` = 2√(1−P²−Q²)。**V** 为 3×6 **状态→惯性系速度 变换矩阵**（无量纲），即 v_inertial = R·v_rot + (dR/dT)·r_rot 的线性化 [R | dR/dT]；由 R(P,Q) 与 dR/dT 的公式给出。内部用 V 左乘状态向量 (r,v) 得惯性系速度数值，再包成 [Speed](crate::quantity::speed::Speed) 返回。
fn laskar_v_mul(
    pw: impl ToReal,
    qw: impl ToReal,
    t_cy: JulianCenturies,
    c: &Elpmpp02Constants,
    vel_i: Vector3<Speed>,
    pos_i: Vector3<Length>,
) -> Vector3<Speed> {
    let (pw, qw) = (real(pw), real(qw));
    let ppw = power_series_derivative_at_real(&c.laskar_p, t_cy);
    let qpw = power_series_derivative_at_real(&c.laskar_q, t_cy);
    // Laskar 归一化：ra = 2√(1−P²−Q²)，下界 1e-20 避免 sqrt 不稳定；d_pq = −4(P·dP/dT + Q·dQ/dT)
    let ra = real_const(2.0)
        * (real_const(1.0) - pw * pw - qw * qw)
            .max(real_const(1e-20))
            .sqrt();
    let d_pq = -real_const(4.0) * pw * ppw - real_const(4.0) * qw * qpw;
    let ppwra = ppw * ra + pw * d_pq / ra;
    let qpwra = qpw * ra + qw * d_pq / ra;
    // 状态→惯性系速度 变换矩阵 V (3×6)：v_inertial = V · [v_rot; r_rot]，即 [R | dR/dT]
    let state_to_inertial_velocity: Mat<Real, 3, 6> = Mat::new([
        [
            real_const(1.0) - real_const(2.0) * pw * pw,
            real_const(2.0) * pw * qw,
            pw * ra,
            -real_const(4.0) * pw * ppw,
            real_const(2.0) * (ppw * qw + pw * qpw),
            ppwra,
        ],
        [
            real_const(2.0) * pw * qw,
            real_const(1.0) - real_const(2.0) * qw * qw,
            -qw * ra,
            real_const(2.0) * (ppw * qw + pw * qpw),
            -real_const(4.0) * qw * qpw,
            -qpwra,
        ],
        [
            -pw * ra,
            qw * ra,
            (real_const(1.0) - real_const(2.0) * pw * pw) + (real_const(1.0) - real_const(2.0) * qw * qw) - real_const(1.0),
            -ppwra,
            qpwra,
            d_pq,
        ],
    ]);
    // 状态向量 (r,v)：月心平架下 [vx,vy,vz, x,y,z]（物理上即相空间/状态向量），单位一致后乘 V 得惯性系速度
    let state_rv: [Real; 6] = [
        vel_i.x().in_unit(SpeedUnit::KmPerJulianCentury),
        vel_i.y().in_unit(SpeedUnit::KmPerJulianCentury),
        vel_i.z().in_unit(SpeedUnit::KmPerJulianCentury),
        pos_i.x().km(),
        pos_i.y().km(),
        pos_i.z().km(),
    ];
    let out = state_to_inertial_velocity.mul_vec_generic(&state_rv);
    Vector3::from_speeds([
        Speed::from_value(out[0], SpeedUnit::KmPerJulianCentury),
        Speed::from_value(out[1], SpeedUnit::KmPerJulianCentury),
        Speed::from_value(out[2], SpeedUnit::KmPerJulianCentury),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::time::j2000_tt;
    use crate::math::real::RealOps;

    #[test]
    fn de405_mean_only_position_near_j2000() {
        let data = Elpmpp02Data::de405_mean_only();
        let (pos, _vel) = position_velocity_mean_only(&data, j2000_tt());
        let r = pos.norm().meters();
        assert!(r > real(3e8) && r < real(4e8), "地月距离约 3.8e8 m");
    }

    #[test]
    fn de406_mean_only_same_constants_as_de405() {
        let d405 = Elpmpp02Data::de405_mean_only();
        let d406 = Elpmpp02Data::de406_mean_only();
        assert_eq!(d405.constants.a0, d406.constants.a0);
        assert_eq!(d406.correction, Elpmpp02Correction::DE406);
    }

    #[test]
    fn split_fortran_main_smoke() {
        let line = "  1  0  0  0    0.12345D+01  0.1  0.2  0.3  0.4  0.5";
        let v = split_fortran_main(line);
        assert!(v.len() >= 9);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[4], 0.12345e01);
    }

    #[test]
    fn position_velocity_dispatcher_mean_only() {
        let data = Elpmpp02Data::de405_mean_only();
        let (pos1, vel1) = position_velocity(&data, j2000_tt());
        let (pos2, vel2) = position_velocity_mean_only(&data, j2000_tt());
        assert_eq!(pos1, pos2);
        assert_eq!(vel1, vel2);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn load_all_and_position_velocity_full() {
        let loader = crate::repo::default_loader();
        let data = load_all(&loader, crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE405).expect("load elpmpp02");
        assert!(!data.is_mean_only());
        let (pos, vel) = position_velocity(&data, j2000_tt());
        let r = pos.norm().meters();
        let pm = pos.to_meters();
        assert!(
            r > real(3e8) && r < real(4.1e8),
            "full 地月距离约 3.8e8 m（随月相 357–407 Mm）, got r={} pos=({},{},{})",
            r.as_f64(),
            pm[0].as_f64(),
            pm[1].as_f64(),
            pm[2].as_f64()
        );
        let vm = vel.to_m_per_s();
        assert!(vm[0].abs() < real(1e4) && vm[1].abs() < real(1e4) && vm[2].abs() < real(1e4));
    }

    /// ELPMPP02(DE406) vs JPL DE406：从 data/jpl/elp_vs_jpl_de406_samples.csv 读取参考位置（J2000 平黄道 km），与 ELPMPP02 比较。
    /// CSV 可由 Python jplephem 等生成（GCRF→Table 7 转 J2000 黄道）。
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn elpmpp02_vs_jpl_de406_samples() {
        use crate::astronomy::time::{TimePoint, TimeScale};
        use std::io::BufRead;

        let base = crate::repo::repo_root();
        let csv_path = base.join(crate::repo::paths::JPL_ELP_VS_JPL_SAMPLES_CSV);
        let Ok(csv) = std::fs::File::open(&csv_path) else { return };
        let loader = crate::repo::default_loader();
        let data = match load_all(&loader, crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406) {
            Ok(d) => d,
            Err(_) => return,
        };

        const JD1950: f64 = 2433282.5;
        const JD2060: f64 = 2473400.5;
        const JD1500: f64 = 2268922.5;
        const JD2500: f64 = 2637936.5;
        fn tol_km(jd: f64) -> f64 {
            if jd >= JD1950 && jd <= JD2060 {
                0.2
            } else if jd >= JD1500 && jd <= JD2500 {
                1.0
            } else {
                10.0
            }
        }

        let reader = std::io::BufReader::new(csv);
        let mut lines = reader.lines();
        let _ = lines.next();
        for line in lines {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split(',').map(str::trim).collect();
            if parts.len() < 4 {
                continue;
            }
            let jd_tdb: f64 = match parts[0].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let x_jpl: f64 = match parts[1].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let y_jpl: f64 = match parts[2].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let z_jpl: f64 = match parts[3].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let jd_tt = TimePoint::new(TimeScale::TDB, crate::math::real::real(jd_tdb))
                .to_scale(TimeScale::TT)
                .jd;
            let t = TimePoint::new(TimeScale::TT, jd_tt);
            let (pos_m, _) = position_velocity(&data, t);
            let pm = pos_m.to_meters();
            let x_elp_km = pm[0] / 1000.0;
            let y_elp_km = pm[1] / 1000.0;
            let z_elp_km = pm[2] / 1000.0;

            let tol = tol_km(jd_tdb);
            let dx = (x_elp_km - x_jpl).abs();
            let dy = (y_elp_km - y_jpl).abs();
            let dz = (z_elp_km - z_jpl).abs();
            assert!(
                dx <= tol && dy <= tol && dz <= tol,
                "JD(TDB)={} ELP=({:.3},{:.3},{:.3}) JPL=({:.3},{:.3},{:.3}) d=({:.3},{:.3},{:.3}) tol={}",
                jd_tdb, x_elp_km, y_elp_km, z_elp_km, x_jpl, y_jpl, z_jpl, dx, dy, dz, tol
            );
        }
    }
}
