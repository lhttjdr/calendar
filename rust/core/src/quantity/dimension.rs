//! 量纲与带量纲标量。
//! 量纲为 7 个 SI 基本单位的幂次 [M, L, T, I, Θ, N, J]。数值全面用 Real 抽象，不写死 f64。

use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::math::real::{real, from_i32, Real, RealOps};

/// 量纲：7 元组 (M, L, T, I, Θ, N, J)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dimension(pub [i8; 7]);

impl Dimension {
    pub const DIMENSIONLESS: Self = Self([0, 0, 0, 0, 0, 0, 0]);
    pub const D_MASS: Self = Self([1, 0, 0, 0, 0, 0, 0]);
    pub const D_LENGTH: Self = Self([0, 1, 0, 0, 0, 0, 0]);
    pub const D_TIME: Self = Self([0, 0, 1, 0, 0, 0, 0]);
    pub const D_VELOCITY: Self = Self([0, 1, -1, 0, 0, 0, 0]);
    pub const D_ANGULAR_VELOCITY: Self = Self([0, 0, -1, 0, 0, 0, 0]);
    pub const D_ANGULAR_ACCELERATION: Self = Self([0, 0, -2, 0, 0, 0, 0]);
    pub const D_ACCELERATION: Self = Self([0, 1, -2, 0, 0, 0, 0]);
    /// 压强（Pa = kg/(m·s²)），量纲 M L⁻¹ T⁻²
    pub const D_PRESSURE: Self = Self([1, -1, -2, 0, 0, 0, 0]);
    /// 热力学温度（K），量纲 Θ
    pub const D_THERMODYNAMIC_TEMPERATURE: Self = Self([0, 0, 0, 0, 1, 0, 0]);

    #[inline]
    pub fn add(self, other: Self) -> Self {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
            self.0[4] + other.0[4],
            self.0[5] + other.0[5],
            self.0[6] + other.0[6],
        ])
    }

    #[inline]
    pub fn sub(self, other: Self) -> Self {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            self.0[3] - other.0[3],
            self.0[4] - other.0[4],
            self.0[5] - other.0[5],
            self.0[6] - other.0[6],
        ])
    }

    /// 量纲数乘：仅支持整数或 0.5（开方）。0.5 时各分量须为偶数。
    pub fn scale(self, n: Real) -> Result<Self, &'static str> {
        if let Some(k) = n.to_i8_trunc_if_in_range() {
            return Ok(Self([
                self.0[0].saturating_mul(k),
                self.0[1].saturating_mul(k),
                self.0[2].saturating_mul(k),
                self.0[3].saturating_mul(k),
                self.0[4].saturating_mul(k),
                self.0[5].saturating_mul(k),
                self.0[6].saturating_mul(k),
            ]));
        }
        if (n - real(0.5)).is_near(real(0.0), 1e-10) {
            for &e in &self.0 {
                if e % 2 != 0 {
                    return Err("sqrt 要求量纲各分量为偶数");
                }
            }
            return Ok(Self([
                self.0[0] / 2,
                self.0[1] / 2,
                self.0[2] / 2,
                self.0[3] / 2,
                self.0[4] / 2,
                self.0[5] / 2,
                self.0[6] / 2,
            ]));
        }
        Err("量纲数乘仅支持整数或 0.5")
    }

    #[inline]
    pub fn eq_dim(self, other: Self) -> bool {
        self == other
    }

    #[inline]
    pub fn is_dimensionless(self) -> bool {
        self == Self::DIMENSIONLESS
    }
}

/// 带量纲标量：数值 Real（SI）+ 量纲。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quantity {
    pub value: Real,
    pub dimension: Dimension,
}

impl Quantity {
    pub fn new(value: Real, dimension: Dimension) -> Self {
        Self { value, dimension }
    }

    pub fn dimensionless(value: Real) -> Self {
        Self::new(value, Dimension::DIMENSIONLESS)
    }

    pub fn add(self, other: Self) -> Result<Self, &'static str> {
        if self.dimension != other.dimension {
            return Err("量纲不匹配");
        }
        Ok(Self::new(self.value + other.value, self.dimension))
    }

    pub fn sub(self, other: Self) -> Result<Self, &'static str> {
        if self.dimension != other.dimension {
            return Err("量纲不匹配");
        }
        Ok(Self::new(self.value - other.value, self.dimension))
    }

    pub fn mul(self, other: Self) -> Self {
        Self::new(
            self.value * other.value,
            self.dimension.add(other.dimension),
        )
    }

    pub fn div(self, other: Self) -> Result<Self, &'static str> {
        if other.value == Real::zero() {
            return Err("除零");
        }
        Ok(Self::new(
            self.value / other.value,
            self.dimension.sub(other.dimension),
        ))
    }

    pub fn scale(self, scalar: Real) -> Self {
        Self::new(self.value * scalar, self.dimension)
    }

    pub fn neg(self) -> Self {
        Self::new(-self.value, self.dimension)
    }

    pub fn pow(self, exponent: i32) -> Result<Self, &'static str> {
        let dim = self.dimension.scale(from_i32(exponent))?;
        let v = self.value.powi(exponent);
        Ok(Self::new(v, dim))
    }

    pub fn sqrt(self) -> Result<Self, &'static str> {
        let dim = self.dimension.scale(real(0.5))?;
        Ok(Self::new(self.value.sqrt(), dim))
    }

    /// 换算到以 unit_factor 为 1 的单位：value / unit_factor
    pub fn in_unit(self, unit_factor: Real) -> Real {
        self.value / unit_factor
    }

    /// 从某单位数值构造：value_in_unit * unit_factor 得到 SI
    pub fn from_unit(value_in_unit: Real, unit_factor: Real, dimension: Dimension) -> Self {
        Self::new(value_in_unit * unit_factor, dimension)
    }
}

impl Add for Quantity {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        if self.dimension != other.dimension {
            panic!("Quantity::Add: 量纲不匹配");
        }
        Self::new(self.value + other.value, self.dimension)
    }
}

impl Sub for Quantity {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        if self.dimension != other.dimension {
            panic!("Quantity::Sub: 量纲不匹配");
        }
        Self::new(self.value - other.value, self.dimension)
    }
}

impl Mul for Quantity {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.value * other.value,
            self.dimension.add(other.dimension),
        )
    }
}

impl Div for Quantity {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        if other.value == Real::zero() {
            panic!("Quantity::Div: 除零");
        }
        Self::new(
            self.value / other.value,
            self.dimension.sub(other.dimension),
        )
    }
}

impl Neg for Quantity {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.value, self.dimension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn dimension_add_sub() {
        let l = Dimension::D_LENGTH;
        let t = Dimension::D_TIME;
        let v = Dimension::D_VELOCITY;
        assert_eq!(l.add(l), Dimension([0, 2, 0, 0, 0, 0, 0]));
        assert_eq!(l.sub(t), Dimension([0, 1, -1, 0, 0, 0, 0]));
        assert!(v.eq_dim(l.sub(t)));
    }

    #[test]
    fn dimension_scale_eq_is_dimensionless() {
        let l = Dimension::D_LENGTH;
        let scaled = l.scale(real(2.0)).unwrap();
        assert_eq!(scaled.0[1], 2);
        let l2 = l.add(l);
        let half = l2.scale(real(0.5)).unwrap();
        assert_eq!(half.0[1], 1);
        assert!(Dimension::DIMENSIONLESS.is_dimensionless());
        assert!(!l.is_dimensionless());
        let odd = Dimension([1, 0, 0, 0, 0, 0, 0]);
        assert!(odd.scale(real(0.5)).is_err());
    }

    #[test]
    fn quantity_mul_div() {
        use crate::math::real::RealOps;
        let len = Quantity::new(real(3.0), Dimension::D_LENGTH);
        let time = Quantity::new(real(2.0), Dimension::D_TIME);
        let vel = len / time;
        assert!(vel.dimension.eq_dim(Dimension::D_VELOCITY));
        assert!(vel.value.is_near(real(1.5), 1e-10));
    }

    #[test]
    fn quantity_add_sub_scale_neg_pow_sqrt_in_unit() {
        let len = Quantity::new(real(5.0), Dimension::D_LENGTH);
        let other = Quantity::new(real(3.0), Dimension::D_LENGTH);
        assert!(len.add(other).unwrap().value.is_near(real(8.0), 1e-10));
        assert!(len.sub(other).unwrap().value.is_near(real(2.0), 1e-10));
        assert!(len.add(Quantity::new(real(1.0), Dimension::D_TIME)).is_err());
        let s = len.scale(real(2.0));
        assert!(s.value.is_near(real(10.0), 1e-10));
        assert!(len.neg().value.is_near(real(-5.0), 1e-10));
        let sq = len.pow(2).unwrap();
        assert!(sq.value.is_near(real(25.0), 1e-10));
        let area_dim = Dimension::D_LENGTH.add(Dimension::D_LENGTH);
        let rt = Quantity::new(real(4.0), area_dim).sqrt().unwrap();
        assert!(rt.value.is_near(real(2.0), 1e-10));
        assert!(len.in_unit(real(1000.0)).is_near(real(0.005), 1e-10));
        let from_u = Quantity::from_unit(real(2.0), real(1000.0), Dimension::D_LENGTH);
        assert!(from_u.value.is_near(real(2000.0), 1e-10));
    }

    #[test]
    fn quantity_trait_add_sub_mul_div_neg() {
        let a = Quantity::new(real(3.0), Dimension::D_LENGTH);
        let b = Quantity::new(real(1.0), Dimension::D_LENGTH);
        assert!((a + b).value.is_near(real(4.0), 1e-10));
        assert!((a - b).value.is_near(real(2.0), 1e-10));
        let t = Quantity::new(real(2.0), Dimension::D_TIME);
        let v = a * t;
        assert!(v.dimension.eq_dim(Dimension([0, 1, 1, 0, 0, 0, 0])));
        assert!((a / t).value.is_near(real(1.5), 1e-10));
        assert!((-a).value.is_near(real(-3.0), 1e-10));
    }
}
