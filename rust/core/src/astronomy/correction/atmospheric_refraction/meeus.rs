//! Meeus: altitude -> refraction.

use crate::math::real::{real, zero, Real, RealOps};
use crate::math::series::arcsec_to_rad;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

/// Meeus 公式：地平高度角 → 折射量。
#[inline]
pub fn meeus_refraction(
    altitude: PlaneAngle,
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PlaneAngle {
    let altitude_rad = altitude.rad();
    let z = Real::pi() / real(2.0) - altitude_rad;
    let tan_z = z.tan();
    let r_arcsec = (real(58.276) * tan_z - real(0.0824) * tan_z.powi(3))
        * pressure_temperature_factor(pressure, temperature);
    if altitude_rad < zero() {
        PlaneAngle::from_rad(zero())
    } else {
        PlaneAngle::from_rad(arcsec_to_rad(r_arcsec))
    }
}
