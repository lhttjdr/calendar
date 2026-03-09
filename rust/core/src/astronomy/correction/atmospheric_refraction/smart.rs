use crate::math::real::{real, zero, Real, RealOps};
use crate::math::series::arcsec_to_rad;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

#[inline]
pub fn smart_refraction(
    ha: PlaneAngle,
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PlaneAngle {
    let ha_rad = ha.rad();
    let z = Real::pi() / real(2.0) - ha_rad;
    let tan_z = z.tan();
    let r_arcsec = (real(58.294) * tan_z - real(0.0668) * tan_z.powi(3))
        * pressure_temperature_factor(pressure, temperature);
    if ha_rad < zero() {
        PlaneAngle::from_rad(zero())
    } else {
        PlaneAngle::from_rad(arcsec_to_rad(r_arcsec))
    }
}
