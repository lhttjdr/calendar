//! XJW 大气折射公式：仅高度角，无气压/气温。

use crate::math::real::{real, zero};
use crate::quantity::angle::PlaneAngle;

/// R(altitude)：XJW 公式，折射量（弧度）。无 P/T 参数。
pub fn xjw_refraction(altitude: PlaneAngle) -> PlaneAngle {
    let h = altitude.rad();
    if h <= zero() {
        return PlaneAngle::from_rad(zero());
    }
    let denom = h + real(0.003138) / (h + real(0.08919));
    let r_rad = real(0.0002967) / denom.tan();
    PlaneAngle::from_rad(r_rad)
}
