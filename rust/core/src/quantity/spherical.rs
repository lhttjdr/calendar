//! 球面坐标与直角坐标的换算（物理量）。经度、纬度、距离 → 直角矢量；球面速度 → 直角速度矢量。

use super::angle::PlaneAngle;
use super::angular_rate::AngularRate;
use super::length::Length;
use super::speed::Speed;
use super::vector3::Vector3;

/// 球面 → 直角：经度 lon、纬度 lat、距离 r（物理量），返回直角坐标矢量（与 r 同单位）。
/// 约定：x = r cos(lat) cos(lon)，y = r cos(lat) sin(lon)，z = r sin(lat)。
#[inline]
pub fn spherical_to_cartesian(lon: PlaneAngle, lat: PlaneAngle, r: Length) -> Vector3<Length> {
    let (cl, sl) = (lon.rad().cos(), lon.rad().sin());
    let (cb, sb) = (lat.rad().cos(), lat.rad().sin());
    let w = r.scale(cb);
    Vector3::from_array([
        w.scale(cl),
        w.scale(sl),
        r.scale(sb),
    ])
}

/// 球面坐标及球面速度 → 直角速度矢量。公式：dx/dt, dy/dt, dz/dt 用 (lon, lat, r) 与 (lon_dot, lat_dot, r_dot)。
pub fn spherical_to_cartesian_velocity(
    lon: PlaneAngle,
    lat: PlaneAngle,
    r: Length,
    lon_dot: AngularRate,
    lat_dot: AngularRate,
    r_dot: Speed,
) -> Vector3<Speed> {
    let (cl, sl) = (lon.rad().cos(), lon.rad().sin());
    let (cb, sb) = (lat.rad().cos(), lat.rad().sin());
    let lon_r: Speed = lon_dot * r;
    let lat_r: Speed = lat_dot * r;
    let vx = r_dot.scale(cb * cl) + (-lat_r).scale(sb * cl) + (-lon_r).scale(cb * sl);
    let vy = r_dot.scale(cb * sl) + (-lat_r).scale(sb * sl) + lon_r.scale(cb * cl);
    let vz = r_dot.scale(sb) + lat_r.scale(cb);
    Vector3::from_speeds([vx, vy, vz])
}
