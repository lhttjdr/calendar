//! Bennett 改进公式：在 Bennett 基础上加 dR 修正项。

use crate::math::angle::deg2rad;
use crate::math::real::{real, zero, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

fn arcmin_to_rad() -> Real {
    Real::pi() / real(10800.0)
}

/// R(altitude, P?, T?)：Bennett 改进公式，dR = -0.06·sin(14.7·rVal+13 度)。
pub fn bennett_improved_refraction(
    altitude: PlaneAngle,
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PlaneAngle {
    let r = altitude.rad();
    let ha = (r * real(180.0) / Real::pi()).max(real(0.05));
    let arg = ha + real(7.31) / (ha + real(4.4));
    let r_val = Real::one() / deg2rad(arg).tan();
    let d_r = real(-0.06) * deg2rad(real(14.7) * r_val + real(13.0)).sin();
    let f = pressure_temperature_factor(pressure, temperature);
    if r < zero() {
        PlaneAngle::from_rad(zero())
    } else {
        PlaneAngle::from_rad((r_val + d_r) * f * arcmin_to_rad())
    }
}

/// 默认气压 101 kPa、气温 10°C。
pub fn bennett_improved_refraction_default(altitude: PlaneAngle) -> PlaneAngle {
    bennett_improved_refraction(
        altitude,
        Pressure::from_kpa(real(101.0)),
        ThermodynamicTemperature::from_celsius(real(10.0)),
    )
}
