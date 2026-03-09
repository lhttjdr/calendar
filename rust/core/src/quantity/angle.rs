//! 平面角（弧度），标量用 Real。全系统以 Real 为唯一标量根。

use crate::math::series::arcsec_to_rad;
use crate::quantity::angle_parse;

use super::angular_rate::AngularRate;
use super::duration::Duration;
use super::unit::AngularRateUnit;
use crate::math::real::{real, Real, RealOps};

use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaneAngle(pub Real);

impl Add for PlaneAngle {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        PlaneAngle(self.0 + other.0)
    }
}

impl Sub for PlaneAngle {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        PlaneAngle(self.0 - other.0)
    }
}

impl Neg for PlaneAngle {
    type Output = Self;
    fn neg(self) -> Self {
        PlaneAngle(-self.0)
    }
}

/// 角 × 无量纲 → 角（中缀版 [scale](PlaneAngle::scale)）
impl Mul<Real> for PlaneAngle {
    type Output = PlaneAngle;
    fn mul(self, s: Real) -> PlaneAngle {
        self.scale(s)
    }
}

/// 无量纲 × 角 → 角
impl Mul<PlaneAngle> for Real {
    type Output = PlaneAngle;
    fn mul(self, a: PlaneAngle) -> PlaneAngle {
        a.scale(self)
    }
}

impl PlaneAngle {
    pub fn from_rad(rad: Real) -> Self {
        PlaneAngle(rad)
    }

    pub fn from_deg(deg: Real) -> Self {
        let one_eighty = real(180.0);
        PlaneAngle(deg * Real::pi() / one_eighty)
    }

    pub fn from_arcsec(arcsec: Real) -> Self {
        PlaneAngle(arcsec_to_rad(arcsec))
    }

    pub fn rad(self) -> Real {
        self.0
    }

    /// 角秒值（弧度 × 180×3600/π），历表振幅等常用。
    pub fn arcsec(self) -> Real {
        self.0 * crate::math::real::arcsec_per_rad()
    }

    pub fn wrap_to_2pi(self) -> Self {
        PlaneAngle(self.0.wrap_to_2pi())
    }

    pub fn wrap_to_signed_pi(self) -> Self {
        PlaneAngle(self.0.wrap_to_signed_pi())
    }

    /// 标量乘：angle × 无量纲 → angle
    pub fn scale(self, s: Real) -> Self {
        PlaneAngle(self.0 * s)
    }

    /// 平面角 ÷ 时间 → 角速率
    pub fn div_duration(self, d: Duration) -> AngularRate {
        let s = d.seconds();
        let zero = Real::zero();
        let small = real(1e-20);
        AngularRate::from_value(if s.abs() < small { zero } else { self.0 / s }, AngularRateUnit::RadPerSecond)
    }

    /// 解析角度字符串；结果弧度转为 Real。
    pub fn parse(s: &str) -> Result<PlaneAngle, String> {
        angle_parse::plane_angle_parse(s).map(|f| PlaneAngle::from_rad(real(f)))
    }
}

#[allow(dead_code)]
pub fn deg2rad_quantity(deg: Real) -> Real {
    crate::math::angle::deg2rad(deg)
}
#[allow(dead_code)]
pub fn sec2rad_quantity(arcsec: Real) -> Real {
    arcsec_to_rad(arcsec)
}
#[allow(dead_code)]
pub fn rad2sec_quantity(rad: Real) -> Real {
    rad * crate::math::real::arcsec_per_rad()
}
