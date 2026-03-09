use crate::math::angle::deg2rad;
use crate::math::real::{real, zero, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

fn arcmin_to_rad() -> Real {
    Real::pi() / (real(180.0) * real(60.0))
}

#[inline]
pub fn saemundsson_refraction(
    h: PlaneAngle,
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PlaneAngle {
    let h_rad = h.rad();
    let h_deg = h_rad * real(180.0) / Real::pi();
    let h_d = h_deg.max(real(0.05));
    let f = pressure_temperature_factor(pressure, temperature);
    let arg = h_d + real(10.3) / (h_d + real(5.11));
    let r_arcmin = f * real(1.02) / deg2rad(arg).tan();
    if h_rad < zero() {
        PlaneAngle::from_rad(zero())
    } else {
        PlaneAngle::from_rad(r_arcmin * arcmin_to_rad())
    }
}
