use crate::math::real::{real, zero, Real, RealOps};
use crate::math::series::arcsec_to_rad;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

/// 无气压气温因子的 Smart 式：视高度 [rad] → 折射 [″]。
pub(super) fn smart_arcsec(ha_rad: Real) -> Real {
    let tan_z = (Real::pi() / real(2.0) - ha_rad).tan();
    real(58.294) * tan_z - real(0.0668) * tan_z.powi(3)
}

/// 视高度 → 折射量。**入参为视高度**，tan z 多项式只适用于高度 ≥ 15°。
#[inline]
pub fn smart_refraction(
    ha: PlaneAngle,
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PlaneAngle {
    let ha_rad = ha.rad();
    let r_arcsec = smart_arcsec(ha_rad)
        * pressure_temperature_factor(pressure, temperature);
    if ha_rad < zero() {
        PlaneAngle::from_rad(zero())
    } else {
        PlaneAngle::from_rad(arcsec_to_rad(r_arcsec))
    }
}
