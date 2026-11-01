use crate::math::angle::deg2rad;
use crate::math::real::{real, zero, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

/// 角分 → 弧度（Real）
fn arcmin_to_rad() -> Real {
    Real::pi() / real(10800.0)
}

/// 无截断、无气压气温因子的 Bennett 式：视高度 [°] → 折射 [′]。
pub(super) fn bennett_arcmin(ha_deg: Real) -> Real {
    let arg = ha_deg + real(7.31) / (ha_deg + real(4.4));
    Real::one() / deg2rad(arg).tan()
}

/// 视高度 → 折射量（Bennett 1982）。**入参为视高度**；视高度 < 0 时返回 0，地平附近连续的结果用 [`super::refraction_from_geometric`]。
pub fn bennett_refraction(
    altitude: PlaneAngle,
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PlaneAngle {
    let r = altitude.rad();
    let ha = (r * real(180.0) / Real::pi()).max(real(0.05));
    let r_arcmin = bennett_arcmin(ha);
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
