use crate::math::angle::deg2rad;
use crate::math::real::{real, zero, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

/// 角分 → 弧度（Real）
fn arcmin_to_rad() -> Real {
    Real::pi() / real(10800.0)
}

pub fn bennett_refraction(
    altitude: PlaneAngle,
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PlaneAngle {
    let r = altitude.rad();
    let ha = (r * real(180.0) / Real::pi()).max(real(0.05));
    let arg = ha + real(7.31) / (ha + real(4.4));
    let r_arcmin = Real::one() / deg2rad(arg).tan();
    let f = pressure_temperature_factor(pressure, temperature);
    if r < zero() {
        PlaneAngle::from_rad(zero())
    } else {
        PlaneAngle::from_rad(r_arcmin * f * arcmin_to_rad())
    }
}

pub fn bennett_refraction_default(altitude: PlaneAngle) -> PlaneAngle {
    bennett_refraction(
        altitude,
        Pressure::from_kpa(real(101.0)),
        ThermodynamicTemperature::from_celsius(real(10.0)),
    )
}
