//! 速度标量（量纲 L/T，SI 米/秒）。内部与 API 均用 Real，不写死 f64。
//! 单位统一用 [super::unit::SpeedUnit] 参数，见 [from_value](Self::from_value)、[in_unit](Self::in_unit)。

use std::ops::{Add, Neg, Sub};

use super::dimension::{Dimension, Quantity};
use super::length::Length;
use super::duration::Duration;
use super::unit::SpeedUnit;
use crate::math::algebra::mat::ScaledBy;
use crate::math::real::{real, Real, RealOps};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Speed(Quantity);

impl Speed {
    pub fn from_quantity(q: Quantity) -> Result<Self, &'static str> {
        if q.dimension != Dimension::D_VELOCITY {
            return Err("量纲须为速度");
        }
        Ok(Self(q))
    }

    /// 按给定单位的数值构造。内部存 SI（m/s）。
    #[inline]
    pub fn from_value(value: Real, unit: SpeedUnit) -> Self {
        Self(Quantity::new(value * unit.to_si_factor(), Dimension::D_VELOCITY))
    }

    /// 以给定单位表示的数值。SI 值 ÷ 单位因子。
    #[inline]
    pub fn in_unit(self, unit: SpeedUnit) -> Real {
        self.0.value / unit.to_si_factor()
    }

    /// SI 数值（m/s）。等价于 `self.in_unit(SpeedUnit::MPerS)`。
    pub fn m_per_s(self) -> Real {
        self.0.value
    }

    pub fn au_per_day(self, meters_per_au: impl crate::math::real::ToReal) -> Real {
        self.in_unit(SpeedUnit::AuPerDay {
            meters_per_au: real(meters_per_au),
        })
    }

    pub fn m_per_julian_century(self) -> Real {
        self.in_unit(SpeedUnit::MPerJulianCentury)
    }

    pub fn to_quantity(self) -> Quantity {
        self.0
    }

    pub fn scale(self, s: Real) -> Self {
        Self(self.0.scale(s))
    }

    pub fn mul_duration(self, d: Duration) -> Length {
        Length::from_quantity(self.0 * d.to_quantity()).unwrap()
    }
}

impl Add for Speed {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Neg for Speed {
    type Output = Self;
    fn neg(self) -> Self {
        Self(self.0.neg())
    }
}

impl Sub for Speed {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl ScaledBy<Real> for Speed {
    fn zero() -> Self {
        Self::from_value(Real::zero(), SpeedUnit::MPerS)
    }
    fn scaled_by(self, s: Real) -> Self {
        self.scale(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};
    use crate::quantity::duration::Duration;

    #[test]
    fn speed_from_quantity_and_units() {
        let q = Quantity::new(real(10.0), Dimension::D_VELOCITY);
        let s = Speed::from_quantity(q).unwrap();
        assert!(s.m_per_s().is_near(real(10.0), 1e-10));
        assert!(Speed::from_quantity(Quantity::new(real(1.0), Dimension::D_LENGTH)).is_err());
        let s2 = Speed::from_value(real(1.0), SpeedUnit::MPerS);
        assert!(s2.m_per_s().is_near(real(1.0), 1e-10));
        assert!(s2.in_unit(SpeedUnit::MPerS).is_near(real(1.0), 1e-10));
    }

    #[test]
    fn speed_au_per_day_mul_duration_add_neg_sub() {
        let s = Speed::from_value(real(2.0), SpeedUnit::MPerS);
        let _ = s.au_per_day(1.5e11);
        let _ = s.m_per_julian_century();
        let d = Duration::in_seconds(real(3.0));
        let l = s.mul_duration(d);
        assert!(l.meters().is_near(real(6.0), 1e-10));
        let z = Speed::zero();
        assert!(z.m_per_s().is_near(real(0.0), 1e-10));
        assert!((s + s).m_per_s().is_near(real(4.0), 1e-10));
        assert!((s - s).m_per_s().is_near(real(0.0), 1e-10));
        assert!(s.neg().m_per_s().is_near(real(-2.0), 1e-10));
    }
}

