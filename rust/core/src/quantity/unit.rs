//! 单位作为参数管理，避免为每种单位写死 `from_*` 函数名。
//!
//! 约定：内部统一存 SI（m, s, m/s, rad/s 等），单位枚举提供「该单位数值 × factor = SI 数值」的 factor；
//! 新增单位时在对应 `*Unit` 枚举加变体并实现换算，不再增加 `from_km_per_xxx` 之类方法。

use crate::math::real::{real, Real};

const SEC_PER_DAY: f64 = 86400.0;
const JULIAN_CENTURY_DAYS: f64 = 36525.0;
const JULIAN_MILLENNIUM_DAYS: f64 = 365250.0;

/// 速度单位。数值 × 该单位的 SI 因子 = 米/秒。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpeedUnit {
    MPerS,
    KmPerDay,
    MPerJulianCentury,
    KmPerJulianCentury,
    MPerJulianMillennium,
    /// 天文单位/日；参数为 1 AU 的米数。
    AuPerDay { meters_per_au: Real },
}

impl SpeedUnit {
    /// 该单位下的数值 v 换算为 SI（m/s）：`v_si = v * factor`。
    pub fn to_si_factor(self) -> Real {
        match self {
            SpeedUnit::MPerS => real(1.0),
            SpeedUnit::KmPerDay => real(1000.0 / SEC_PER_DAY),
            SpeedUnit::MPerJulianCentury => real(1.0 / (JULIAN_CENTURY_DAYS * SEC_PER_DAY)),
            SpeedUnit::KmPerJulianCentury => real(1000.0 / (JULIAN_CENTURY_DAYS * SEC_PER_DAY)),
            SpeedUnit::MPerJulianMillennium => real(1.0 / (JULIAN_MILLENNIUM_DAYS * SEC_PER_DAY)),
            SpeedUnit::AuPerDay { meters_per_au } => meters_per_au / real(SEC_PER_DAY),
        }
    }
}

/// 长度单位。数值 × 该单位的 SI 因子 = 米。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthUnit {
    Meter,
    Kilometer,
}

impl LengthUnit {
    pub fn to_si_factor(self) -> Real {
        match self {
            LengthUnit::Meter => real(1.0),
            LengthUnit::Kilometer => real(1000.0),
        }
    }
}

/// 角速率单位。数值 × 该单位的 SI 因子 = rad/s。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AngularRateUnit {
    RadPerSecond,
    RadPerDay,
    RadPerJulianCentury,
    RadPerJulianMillennium,
}

impl AngularRateUnit {
    pub fn to_si_factor(self) -> Real {
        match self {
            AngularRateUnit::RadPerSecond => real(1.0),
            AngularRateUnit::RadPerDay => real(1.0 / SEC_PER_DAY),
            AngularRateUnit::RadPerJulianCentury => real(1.0 / (JULIAN_CENTURY_DAYS * SEC_PER_DAY)),
            AngularRateUnit::RadPerJulianMillennium => {
                real(1.0 / (JULIAN_MILLENNIUM_DAYS * SEC_PER_DAY))
            }
        }
    }
}
