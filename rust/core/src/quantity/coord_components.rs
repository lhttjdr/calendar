//! **三分量元组（坐标分量）**：某坐标系**坐标基**下的三分量，量纲可不同（如径向速度 + 两个角速度），
//! 必须与带度规的参考架配合使用。与 [Vector3](crate::quantity::vector3::Vector3)（矢量，正交归一化基、同量纲）区分。
//!
//! 当前提供球坐标速度的坐标分量；与 [frame_metric](super::frame_metric) 的 scale factors 配合转换为矢量。

use super::angle::PlaneAngle;
use super::angular_rate::AngularRate;
use super::length::Length;
use super::speed::Speed;
use super::unit::{AngularRateUnit, SpeedUnit};
use super::vector3::Vector3;

/// 球坐标下速度的**坐标基分量**（三分量元组）：(ṙ, λ̇, φ̇)，量纲分别为 [L/T], [1/T], [1/T]。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphericalVelocityCoordComponents {
    pub r_dot: Speed,
    pub lon_dot: AngularRate,
    pub lat_dot: AngularRate,
}

impl SphericalVelocityCoordComponents {
    /// 在给定位置 (r, lon, lat) 处，用度规的 scale factors 将坐标分量转为正交归一化基下的矢量。
    pub fn to_vector3_at(
        self,
        r: Length,
        lon: PlaneAngle,
        lat: PlaneAngle,
    ) -> Vector3<Speed> {
        super::spherical::spherical_to_cartesian_velocity(
            lon, lat, r,
            self.lon_dot, self.lat_dot, self.r_dot,
        )
    }

    /// 从正交归一化基下的矢量及所在位置，用度规反算坐标基分量。
    pub fn from_vector3_at(
        v: Vector3<Speed>,
        r: Length,
        lon: PlaneAngle,
        lat: PlaneAngle,
    ) -> Self {
        let (cl, sl) = (lon.rad().cos(), lon.rad().sin());
        let (cb, sb) = (lat.rad().cos(), lat.rad().sin());
        let vx = v.x().m_per_s();
        let vy = v.y().m_per_s();
        let vz = v.z().m_per_s();
        let r_si = r.meters();
        let r_dot = vx * cb * cl + vy * cb * sl + vz * sb;
        let lat_dot = (-vx * sb * cl - vy * sb * sl + vz * cb) / r_si;
        let lon_dot = (-vx * sl + vy * cl) / (r_si * cb);
        Self {
            r_dot: Speed::from_value(r_dot, SpeedUnit::MPerS),
            lon_dot: AngularRate::from_value(lon_dot, AngularRateUnit::RadPerSecond),
            lat_dot: AngularRate::from_value(lat_dot, AngularRateUnit::RadPerSecond),
        }
    }
}
