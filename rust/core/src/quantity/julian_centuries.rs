//! 儒略世纪数：无量纲物理量，单位「世纪」= 36525 日，常用作历表幂级数自变量 T。

use crate::math::real::{Real, ToReal};
use std::ops::Mul;

/// 儒略世纪数（无量纲）：T = (JD − J2000) / 36525，历表幂级数自变量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JulianCenturies(pub Real);

impl JulianCenturies {
    #[inline]
    pub fn from_value(value: Real) -> Self {
        Self(value)
    }

    /// 数值，用于幂级数等公式。
    #[inline]
    pub fn value(self) -> Real {
        self.0
    }
}

/// 无量纲 × 无量纲 → Real，便于与 [PlaneAngle](crate::quantity::angle::PlaneAngle) 等的中缀运算（如 `coeff * (real(2) * t_cy)`）。
impl Mul<Real> for JulianCenturies {
    type Output = Real;
    #[inline]
    fn mul(self, rhs: Real) -> Real {
        self.0 * rhs
    }
}

/// Real × 无量纲 → Real。
impl Mul<JulianCenturies> for Real {
    type Output = Real;
    #[inline]
    fn mul(self, rhs: JulianCenturies) -> Real {
        self * rhs.0
    }
}

/// 无量纲 × 无量纲 → Real，便于直接写 t_cy²、t_cy³ 等参与 [Real] 数组。
impl Mul<JulianCenturies> for JulianCenturies {
    type Output = Real;
    #[inline]
    fn mul(self, rhs: JulianCenturies) -> Real {
        self.0 * rhs.0
    }
}

impl ToReal for JulianCenturies {
    #[inline]
    fn to_real(self) -> Real {
        self.0
    }
}
