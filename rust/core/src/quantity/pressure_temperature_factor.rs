//! 气压–温度修正因子：物理意义为 (p/p_ref)(T_ref/T)，量纲为 1，用于大气折射等公式中修正气压与温度的影响。

use super::dimensionless::Dimensionless;
use crate::math::real::Real;
use std::ops::Mul;

/// 气压–温度修正因子。与泛用的 [`Dimensionless`] 区分，类型即表达「气压温度因子」这一物理概念。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressureTemperatureFactor(Dimensionless);

impl PressureTemperatureFactor {
    pub fn from_dimensionless(d: Dimensionless) -> Self {
        Self(d)
    }

    pub fn value(self) -> Real {
        self.0.value()
    }

    pub fn to_dimensionless(self) -> Dimensionless {
        self.0
    }
}

/// 数值 × 气压温度因子 → Real，代入折射等公式。
impl Mul<PressureTemperatureFactor> for Real {
    type Output = Real;
    fn mul(self, f: PressureTemperatureFactor) -> Real {
        self * f.0.value()
    }
}

/// 气压温度因子 × 数值 → Real。
impl Mul<Real> for PressureTemperatureFactor {
    type Output = Real;
    fn mul(self, r: Real) -> Real {
        self.0.value() * r
    }
}
