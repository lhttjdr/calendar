use crate::astronomy::constant::J2000;
use crate::math::algebra::mat::Mat;
use crate::math::real::{real_const, real, zero, one, Real, ToReal};
use crate::math::series::{arcsec_to_rad, power_series_at, power_series_derivative_at};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::angular_rate::AngularRate;
use crate::quantity::unit::AngularRateUnit;
use crate::quantity::{displacement::Displacement, epoch::Epoch, reference_frame::ReferenceFrame};

use super::vondrak2011;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrecessionModel {
    /// P03：岁差矩阵 J2000 平赤道 → 历元 t 平赤道（doc §3）。
    P03,
    Vondrak2011,
}

#[derive(Clone, Debug)]
pub struct PrecessionTransform {
    pub from_frame: ReferenceFrame,
    pub to_frame: ReferenceFrame,
    pub matrix: [[Real; 3]; 3],
}

impl PrecessionTransform {
    pub fn apply_vec(&self, v: [Real; 3]) -> [Real; 3] {
        Mat::<Real, 3, 3>::from(self.matrix).mul_vec(v)
    }

    pub fn apply_displacement(&self, d: Displacement) -> Displacement {
        assert!(
            d.frame == self.from_frame,
            "precession input frame must be {} (got {:?})",
            self.from_frame.id_str(),
            d.frame
        );
        let out = self.apply_vec(d.to_meters());
        Displacement::from_si_meters_in_frame(self.to_frame, out[0], out[1], out[2])
    }
}

/// 儒略世纪 t → JD(TT)。入参与结果均为 Real，无 f64 边界。
fn jd_from_t_cent(t: impl ToReal) -> Real {
    J2000 + real(t) * real_const(36525.0)
}

#[inline]
pub fn precession_transform_for(t: impl ToReal, model: PrecessionModel) -> PrecessionTransform {
    let t_r = real(t);
    let matrix = match model {
        PrecessionModel::P03 => precession_matrix(t_r),
        PrecessionModel::Vondrak2011 => vondrak2011::precession_matrix(t_r),
    };
    let to_frame = ReferenceFrame::MeanEquator(Epoch::new(jd_from_t_cent(t_r)));
    PrecessionTransform {
        from_frame: ReferenceFrame::FK5,
        to_frame,
        matrix,
    }
}

#[inline]
pub fn precession_transform(t: impl ToReal) -> PrecessionTransform {
    precession_transform_for(t, PrecessionModel::P03)
}

#[inline]
pub fn precession_matrix_for(t: impl ToReal, model: PrecessionModel) -> [[Real; 3]; 3] {
    precession_transform_for(t, model).matrix
}

const ZETA_COEFFS: [f64; 6] = [2.650545, 2306.083227, 0.2988499, 0.01801828, -5.971e-6, -3.173e-7];
const THETA_COEFFS: [f64; 6] = [0.0, 2004.191903, -0.4294934, -0.04182264, -7.089e-6, -1.274e-7];
const Z_COEFFS: [f64; 6] = [-2.650545, 2306.077181, 1.0927348, 0.01826837, -0.000028596, -2.904e-7];
const EPSILON_COEFFS: [f64; 6] = [84381.406000, -46.836769, -0.0001831, 0.00200340, -5.76e-7, -4.34e-8];

fn zeta(t: impl ToReal) -> PlaneAngle {
    PlaneAngle::from_rad(arcsec_to_rad(power_series_at(&ZETA_COEFFS, real(t))))
}
fn theta(t: impl ToReal) -> PlaneAngle {
    PlaneAngle::from_rad(arcsec_to_rad(power_series_at(&THETA_COEFFS, real(t))))
}
fn z(t: impl ToReal) -> PlaneAngle {
    PlaneAngle::from_rad(arcsec_to_rad(power_series_at(&Z_COEFFS, real(t))))
}

#[inline]
pub fn mean_obliquity(t: impl ToReal) -> PlaneAngle {
    PlaneAngle::from_rad(arcsec_to_rad(power_series_at(&EPSILON_COEFFS, real(t))))
}

/// 平黄赤交角对 t 的导数（仅 P03；Vondrak 用 vondrak2011::epsilon_derivative）。
#[inline]
pub fn mean_obliquity_derivative(t: impl ToReal) -> AngularRate {
    AngularRate::from_value(
        arcsec_to_rad(power_series_derivative_at(&EPSILON_COEFFS, real(t))),
        AngularRateUnit::RadPerJulianCentury,
    )
}

#[inline]
fn rotation_z(angle: PlaneAngle) -> [[Real; 3]; 3] {
    let (c, s) = (angle.rad().cos(), angle.rad().sin());
    [[c, -s, zero()], [s, c, zero()], [zero(), zero(), one()]]
}

#[inline]
fn rotation_y(angle: PlaneAngle) -> [[Real; 3]; 3] {
    let (c, s) = (angle.rad().cos(), angle.rad().sin());
    [[c, zero(), s], [zero(), one(), zero()], [-s, zero(), c]]
}

/// dR3(α)/dα：R3(α)=[[c,-s,0],[s,c,0],[0,0,1]] 对 α 求导。角支持 Real。
fn rotation_z_derivative(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[-s, -c, zero()], [c, -s, zero()], [zero(), zero(), zero()]]
}

/// dR2(β)/dβ：R2(β)=[[c,0,s],[0,1,0],[-s,0,c]] 对 β 求导。角支持 Real。
fn rotation_y_derivative(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[-s, zero(), c], [zero(), zero(), zero()], [-c, zero(), -s]]
}

/// 岁差矩阵：J2000 平赤道 → 历元 t 平赤道（doc §3）。P = R3(ζ) R2(-θ) R3(z)。
pub fn precession_matrix(t: impl ToReal) -> [[Real; 3]; 3] {
    let t = real(t);
    let rz_zeta = Mat::<Real, 3, 3>::from(rotation_z(zeta(t)));
    let ry_neg_theta = Mat::<Real, 3, 3>::from(rotation_y(PlaneAngle::from_rad(-theta(t).rad())));
    let rz_z = Mat::<Real, 3, 3>::from(rotation_z(z(t)));
    rz_zeta.mul_mat(&ry_neg_theta).mul_mat(&rz_z).to_array()
}

pub fn precession_apply(t: impl ToReal, v: [Real; 3]) -> [Real; 3] {
    precession_transform(t).apply_vec(v)
}

pub fn precession_apply_displacement(t: impl ToReal, d: Displacement) -> Displacement {
    precession_transform(t).apply_displacement(d)
}

/// 角速度（弧度/世纪），用于 P03 岁差导数
fn zeta_dot(t: impl ToReal) -> AngularRate {
    AngularRate::from_value(
        arcsec_to_rad(power_series_derivative_at(&ZETA_COEFFS, real(t))),
        AngularRateUnit::RadPerJulianCentury,
    )
}
fn theta_dot(t: impl ToReal) -> AngularRate {
    AngularRate::from_value(
        arcsec_to_rad(power_series_derivative_at(&THETA_COEFFS, real(t))),
        AngularRateUnit::RadPerJulianCentury,
    )
}
fn z_dot(t: impl ToReal) -> AngularRate {
    AngularRate::from_value(
        arcsec_to_rad(power_series_derivative_at(&Z_COEFFS, real(t))),
        AngularRateUnit::RadPerJulianCentury,
    )
}

fn scale_vec(s: impl ToReal, v: [Real; 3]) -> [Real; 3] {
    let s = real(s);
    [v[0] * s, v[1] * s, v[2] * s]
}

fn add_vec(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// (dP/dt)·r，P = R3(ζ) R2(-θ) R3(z)。t 儒略世纪，r 米。返回 m/世纪。
pub fn precession_derivative_times_vector(r: [Real; 3], t: impl ToReal) -> [Real; 3] {
    let t = real(t);
    let zeta_t = zeta(t).rad();
    let theta_t = theta(t).rad();
    let z_t = z(t).rad();
    let r3_zeta = Mat::<Real, 3, 3>::from(rotation_z(zeta(t)));
    let r2_neg_theta = Mat::<Real, 3, 3>::from(rotation_y(PlaneAngle::from_rad(-theta_t)));
    let r3_z = Mat::<Real, 3, 3>::from(rotation_z(z(t)));
    let v_after_z = r3_z.mul_vec(r);
    let v_after_theta = r2_neg_theta.mul_vec(v_after_z);
    let z_dot_r = z_dot(t).rad_per_julian_century();
    let theta_dot_r = theta_dot(t).rad_per_julian_century();
    let zeta_dot_r = zeta_dot(t).rad_per_julian_century();
    let term1 = scale_vec(
        zeta_dot_r,
        Mat::<Real, 3, 3>::from(rotation_z_derivative(zeta_t)).mul_vec(v_after_theta),
    );
    let term2 = r3_zeta.mul_vec(scale_vec(
        -theta_dot_r,
        Mat::<Real, 3, 3>::from(rotation_y_derivative(-theta_t)).mul_vec(v_after_z),
    ));
    let term3 = r3_zeta.mul_vec(r2_neg_theta.mul_vec(scale_vec(
        z_dot_r,
        Mat::<Real, 3, 3>::from(rotation_z_derivative(z_t)).mul_vec(r),
    )));
    add_vec(term1, add_vec(term2, term3))
}

/// 按模型选择岁差导数：(dP/dt)·r，m/世纪。t 支持 Real。
pub fn precession_derivative_times_vector_for(r: [Real; 3], t: impl ToReal, model: PrecessionModel) -> [Real; 3] {
    match model {
        PrecessionModel::P03 => precession_derivative_times_vector(r, t),
        PrecessionModel::Vondrak2011 => vondrak2011::precession_matrix_derivative_times_vector(r, t),
    }
}

/// 平黄赤交角及其对 t 的导数（按岁差模型）。t 支持 Real。
pub fn mean_obliquity_rad_and_derivative_for(
    t: impl ToReal,
    model: PrecessionModel,
) -> (PlaneAngle, AngularRate) {
    let t_real = real(t);
    match model {
        PrecessionModel::P03 => (
            mean_obliquity(t_real),
            mean_obliquity_derivative(t_real),
        ),
        PrecessionModel::Vondrak2011 => (
            vondrak2011::epsilon(t_real),
            vondrak2011::epsilon_derivative(t_real),
        ),
    }
}
