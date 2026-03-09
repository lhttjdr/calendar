//! 长度（量纲 L，SI 米）。内部与 API 均用 Real，不写死 f64。
//! 单位统一用 [super::unit::LengthUnit] 参数，见 [from_value](Self::from_value)、[in_unit](Self::in_unit)。

use std::ops::{Add, Sub};

use super::dimension::{Dimension, Quantity};
use super::duration::Duration;
use super::speed::Speed;
use super::unit::LengthUnit;
use crate::math::algebra::mat::ScaledBy;
use crate::math::real::{Real, RealOps};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Length(Quantity);

impl Length {
    pub fn from_quantity(q: Quantity) -> Result<Self, &'static str> {
        if q.dimension != Dimension::D_LENGTH {
            return Err("量纲须为长度");
        }
        Ok(Self(q))
    }

    /// 按给定单位的数值构造。内部存 SI（m）。
    #[inline]
    pub fn from_value(value: Real, unit: LengthUnit) -> Self {
        Self(Quantity::new(value * unit.to_si_factor(), Dimension::D_LENGTH))
    }

    /// 以给定单位表示的数值。
    #[inline]
    pub fn in_unit(self, unit: LengthUnit) -> Real {
        self.0.value / unit.to_si_factor()
    }

    /// SI 数值（m）。等价于 `self.in_unit(LengthUnit::Meter)`。
    pub fn meters(self) -> Real {
        self.0.value
    }

    pub fn km(self) -> Real {
        self.in_unit(LengthUnit::Kilometer)
    }

    pub fn to_quantity(self) -> Quantity {
        self.0
    }

    pub fn add(self, other: Self) -> Self {
        self + other
    }

    pub fn sub(self, other: Self) -> Self {
        self - other
    }

    pub fn scale(self, s: Real) -> Self {
        Self(self.0.scale(s))
    }

    pub fn neg(self) -> Self {
        Self(self.0.neg())
    }

    /// 长度 ÷ 时间 → 速度
    pub fn div_duration(self, d: Duration) -> Speed {
        Speed::from_quantity(self.0 / d.to_quantity()).unwrap()
    }

    /// 范数（自身作为位移矢量时）；单标量即绝对值
    pub fn norm(self) -> Self {
        Self::from_value(self.meters().abs(), LengthUnit::Meter)
    }
}

impl Add for Length {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for Length {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl ScaledBy<Real> for Length {
    fn zero() -> Self {
        Self::from_value(Real::zero(), LengthUnit::Meter)
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
    fn length_from_quantity_and_value_in_unit() {
        let q = Quantity::new(real(1000.0), Dimension::D_LENGTH);
        let l = Length::from_quantity(q).unwrap();
        assert!(l.meters().is_near(real(1000.0), 1e-10));
        assert!(l.km().is_near(real(1.0), 1e-10));
        let bad = Quantity::new(real(1.0), Dimension::D_TIME);
        assert!(Length::from_quantity(bad).is_err());
        let from_km = Length::from_value(real(2.0), LengthUnit::Kilometer);
        assert!(from_km.meters().is_near(real(2000.0), 1e-10));
        assert!(from_km.in_unit(LengthUnit::Kilometer).is_near(real(2.0), 1e-10));
    }

    #[test]
    fn length_add_sub_scale_neg_norm_div_duration() {
        let a = Length::from_value(real(5.0), LengthUnit::Meter);
        let b = Length::from_value(real(3.0), LengthUnit::Meter);
        assert!((a.add(b)).meters().is_near(real(8.0), 1e-10));
        assert!((a - b).meters().is_near(real(2.0), 1e-10));
        assert!(a.scale(real(2.0)).meters().is_near(real(10.0), 1e-10));
        assert!(a.neg().meters().is_near(real(-5.0), 1e-10));
        assert!(a.norm().meters().is_near(real(5.0), 1e-10));
        let d = Duration::in_seconds(real(2.0));
        let speed = a.div_duration(d);
        assert!(speed.m_per_s().is_near(real(2.5), 1e-10));
    }

    #[test]
    fn length_scaled_by_zero() {
        let z = Length::zero();
        assert!(z.meters().is_near(real(0.0), 1e-10));
        let l = Length::from_value(real(3.0), LengthUnit::Meter);
        let s = l.scaled_by(real(2.0));
        assert!(s.meters().is_near(real(6.0), 1e-10));
    }
}

