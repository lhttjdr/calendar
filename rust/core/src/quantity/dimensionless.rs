//! 无量纲量（量纲为 1 的物理量）。用于比例、因子、折射修正系数等。

use super::dimension::{Dimension, Quantity};
use crate::math::real::Real;
use std::ops::Mul;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimensionless(Quantity);

impl Dimensionless {
    pub fn from_value(value: Real) -> Self {
        Self(Quantity::dimensionless(value))
    }

    pub fn from_quantity(q: Quantity) -> Result<Self, &'static str> {
        if q.dimension != Dimension::DIMENSIONLESS {
            return Err("量纲须为无量纲");
        }
        Ok(Self(q))
    }

    pub fn value(self) -> Real {
        self.0.value
    }

    pub fn to_quantity(self) -> Quantity {
        self.0
    }
}

/// 无量纲 × 无量纲 → 无量纲
impl Mul<Dimensionless> for Dimensionless {
    type Output = Dimensionless;
    fn mul(self, other: Dimensionless) -> Dimensionless {
        Dimensionless(self.0 * other.0)
    }
}

/// Real × 无量纲 → Real（公式中常用：系数 × 因子）
impl Mul<Dimensionless> for Real {
    type Output = Real;
    fn mul(self, d: Dimensionless) -> Real {
        self * d.value()
    }
}

/// 无量纲 × Real → Real
impl Mul<Real> for Dimensionless {
    type Output = Real;
    fn mul(self, r: Real) -> Real {
        self.value() * r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};
    use crate::quantity::dimension::Quantity;

    #[test]
    fn dimensionless_from_value_and_value() {
        let d = Dimensionless::from_value(real(0.5));
        assert!(d.value().is_near(real(0.5), 1e-10));
    }

    #[test]
    fn dimensionless_from_quantity_ok_err() {
        let q = Quantity::dimensionless(real(1.0));
        let d = Dimensionless::from_quantity(q).unwrap();
        assert!(d.value().is_near(real(1.0), 1e-10));
        let q_time = Quantity::new(real(1.0), crate::quantity::dimension::Dimension::D_TIME);
        assert!(Dimensionless::from_quantity(q_time).is_err());
    }

    #[test]
    fn dimensionless_to_quantity_and_mul() {
        let a = Dimensionless::from_value(real(2.0));
        let b = Dimensionless::from_value(real(3.0));
        let c = a * b;
        assert!(c.value().is_near(real(6.0), 1e-10));
        let r = real(4.0);
        assert!((r * a).is_near(real(8.0), 1e-10));
        assert!((a * r).is_near(real(8.0), 1e-10));
    }
}
