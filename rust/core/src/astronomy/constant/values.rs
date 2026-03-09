//! 天文常数值。涉及物理概念的尽量用物理量类型（Speed、PlaneAngle 等），标量统一 [crate::math::real::Real]。

use crate::math::real::{real_const, Real};
use crate::quantity::speed::Speed;
use crate::quantity::unit::SpeedUnit;

/// J2000.0 儒略日
pub const J2000: Real = real_const(2451545.0);

/// 天文单位，米（IAU 2012）
pub const AU_METERS: Real = real_const(149_597_870_700.0);

/// 光速（物理量）。用于光行差等；AU/day 数值见 `light_speed_au_per_day()`。
pub fn light_speed() -> Speed {
    Speed::from_value(
        real_const(173.144_632_674_24),
        SpeedUnit::AuPerDay { meters_per_au: AU_METERS },
    )
}

/// 光速，AU/日（由 [light_speed] 导出，便于需裸数值处使用）。
pub fn light_speed_au_per_day() -> Real {
    light_speed().in_unit(SpeedUnit::AuPerDay { meters_per_au: AU_METERS })
}

/// 儒略千年（VSOP87 时间单位）：365250 日
pub const JULIAN_MILLENNIUM: Real = real_const(365250.0);

/// 行星质量/太阳质量（约），用于质心改正
pub const M_MERCURY_OVER_M_SUN: Real = real_const(1.0 / 6023600.0);
pub const M_VENUS_OVER_M_SUN: Real = real_const(1.0 / 408524.0);
pub const M_EARTH_OVER_M_SUN: Real = real_const(1.0 / 328901.0);
pub const M_MARS_OVER_M_SUN: Real = real_const(1.0 / 3098708.0);
pub const M_JUPITER_OVER_M_SUN: Real = real_const(1.0 / 1047.35);
pub const M_SATURN_OVER_M_SUN: Real = real_const(1.0 / 3497.9);
pub const M_URANUS_OVER_M_SUN: Real = real_const(1.0 / 22903.0);
pub const M_NEPTUNE_OVER_M_SUN: Real = real_const(1.0 / 19412.0);

pub struct Constant;
