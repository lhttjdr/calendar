use crate::math::angle::deg2rad;
use crate::math::real::{real, zero, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::factor::pressure_temperature_factor;

fn arcmin_to_rad() -> Real {
    Real::pi() / (real(180.0) * real(60.0))
}

/// 无截断、无气压气温因子的 Saemundsson 式：真高度 [°] → 折射 [′]。
pub(super) fn saemundsson_arcmin(h_deg: Real) -> Real {
    let arg = h_deg + real(10.3) / (h_deg + real(5.11));
    real(1.02) / deg2rad(arg).tan()
}

/// 真高度 → 折射量（Saemundsson 1986）。**入参为真（几何）高度**；真高度 < 0 时返回 0，地平附近连续的结果用 [`super::refraction_from_geometric`]。
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
    let r_arcmin = f * saemundsson_arcmin(h_d);
    if h_rad < zero() {
        PlaneAngle::from_rad(zero())
    } else {
        PlaneAngle::from_rad(r_arcmin * arcmin_to_rad())
    }
}
