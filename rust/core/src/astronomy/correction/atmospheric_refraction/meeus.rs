//! Meeus: altitude -> refraction.

use crate::math::real::{real, zero, Real, RealOps};
use crate::math::series::arcsec_to_rad;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

/// 无气压气温因子的 Meeus 式：真高度 [rad] → 折射 [″]。
pub(super) fn meeus_arcsec(altitude_rad: Real) -> Real {
    let tan_z = (Real::pi() / real(2.0) - altitude_rad).tan();
    real(58.276) * tan_z - real(0.0824) * tan_z.powi(3)
}

/// Meeus 公式：真（几何）高度 → 折射量。**入参为真高度**，tan z 多项式只适用于高度 ≥ 15°。
#[inline]
pub fn meeus_refraction(
    altitude: PlaneAngle,
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PlaneAngle {
    let altitude_rad = altitude.rad();
    let r_arcsec = meeus_arcsec(altitude_rad)
        * pressure_temperature_factor(pressure, temperature);
    if altitude_rad < zero() {
        PlaneAngle::from_rad(zero())
    } else {
        PlaneAngle::from_rad(arcsec_to_rad(r_arcsec))
    }
}
