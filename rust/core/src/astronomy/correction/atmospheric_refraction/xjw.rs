//! XJW 大气折射公式：仅高度角，无气压/气温。

use crate::math::real::{real, zero};
use crate::quantity::angle::PlaneAngle;

/// R(altitude)：XJW 公式，折射量（弧度）。无 P/T 参数。**入参为真高度**；系数即 Saemundsson 式换算到弧度
/// （1.02′、10.3°²、5.11°），故不单设 [`super::RefractionModel`]。
pub fn xjw_refraction(altitude: PlaneAngle) -> PlaneAngle {
    let h = altitude.rad();
    if h <= zero() {
        return PlaneAngle::from_rad(zero());
    }
    let denom = h + real(0.003138) / (h + real(0.08919));
    let r_rad = real(0.0002967) / denom.tan();
    PlaneAngle::from_rad(r_rad)
}
