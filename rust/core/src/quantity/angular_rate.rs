//! 角频率/角速率（量纲 1/T），标量用 Real。单位统一用 [super::unit::AngularRateUnit] 参数。
//! 角速率 × 长度 → 线速度（[Mul](std::ops::Mul)<[Length](super::length::Length)> → [Speed](super::speed::Speed)）。

use std::ops::{Add, Mul, Neg};

use super::angle::PlaneAngle;
use super::dimension::{Dimension, Quantity};
use super::length::Length;
use super::speed::Speed;
use super::unit::AngularRateUnit;
use crate::math::real::Real;

/// 角频率（量纲 1/T），内部存 rad/s（SI）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AngularRate(Quantity);

impl AngularRate {
    pub fn from_quantity(q: Quantity) -> Result<Self, &'static str> {
        if q.dimension != Dimension::D_ANGULAR_VELOCITY {
            return Err("量纲须为角速率");
        }
        Ok(Self(q))
    }

    #[inline]
    pub fn from_value(value: Real, unit: AngularRateUnit) -> Self {
        Self(Quantity::new(
            value * unit.to_si_factor(),
            Dimension::D_ANGULAR_VELOCITY,
        ))
    }

    #[inline]
    pub fn in_unit(self, unit: AngularRateUnit) -> Real {
        self.0.value / unit.to_si_factor()
    }

    /// SI 数值（rad/s）。等价于 `self.in_unit(AngularRateUnit::RadPerSecond)`。
    pub fn rad_per_second(self) -> Real {
        self.0.value
    }

    pub fn rad_per_day(self) -> Real {
        self.in_unit(AngularRateUnit::RadPerDay)
    }

    pub fn rad_per_julian_millennium(self) -> Real {
        self.in_unit(AngularRateUnit::RadPerJulianMillennium)
    }

    pub fn rad_per_julian_century(self) -> Real {
        self.in_unit(AngularRateUnit::RadPerJulianCentury)
    }

    /// 频率 × 无量纲 T（儒略千年数）→ 相位角。VSOP87：phase + C×T。
    pub fn angle_for_t_julian_millennia(self, t: Real) -> PlaneAngle {
        PlaneAngle(self.rad_per_julian_millennium() * t)
    }

    pub fn to_quantity(self) -> Quantity {
        self.0
    }
}

impl Add for AngularRate {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Neg for AngularRate {
    type Output = Self;
    fn neg(self) -> Self {
        Self(self.0.neg())
    }
}

/// 角速率 × 长度 → 线速度（如 ω × 半径）。
impl Mul<Length> for AngularRate {
    type Output = Speed;
    fn mul(self, r: Length) -> Speed {
        Speed::from_quantity(self.0 * r.to_quantity()).unwrap()
    }
}
