//! 数值抽象：标量类型 [Real]，运算经 [RealOps]。
//!
//! **分层约定：只有本模块使用 f64、TwoFloat 等底层类型；其它模块一律只依赖 Real/RealOps。**
//! Real 为泛型 newtype `Real<R: Backend>`，由 feature 选择后端（twofloat 或 f64）。

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// ============== Backend trait ==============

/// 标量后端：仅本模块使用。Real 的运算委托给 Backend；新增后端只需 impl 本 trait。
pub trait Backend:
    Copy
    + Clone
    + Default
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    fn from_f64(x: f64) -> Option<Self>
    where
        Self: Sized;
    fn as_f64(self) -> f64;
    fn zero() -> Self
    where
        Self: Sized;
    fn one() -> Self
    where
        Self: Sized;
    fn pi() -> Self
    where
        Self: Sized;
    fn two_pi() -> Self
    where
        Self: Sized;
    fn sin(self) -> Self
    where
        Self: Sized;
    fn cos(self) -> Self
    where
        Self: Sized;
    fn tan(self) -> Self
    where
        Self: Sized;
    fn sqrt(self) -> Self
    where
        Self: Sized;
    fn asin(self) -> Self
    where
        Self: Sized;
    fn atan2(self, other: Self) -> Self
    where
        Self: Sized;
}

// ============== TwoFloat backend (default) ==============

#[cfg(not(feature = "real-f64"))]
use num_traits::ToPrimitive;
#[cfg(not(feature = "real-f64"))]
use twofloat::TwoFloat;

#[cfg(not(feature = "real-f64"))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct TwoFloatBackend(pub(crate) TwoFloat);

#[cfg(not(feature = "real-f64"))]
impl Default for TwoFloatBackend {
    #[inline]
    fn default() -> Self {
        TwoFloatBackend(TwoFloat::from_f64(0.0))
    }
}

#[cfg(not(feature = "real-f64"))]
impl Backend for TwoFloatBackend {
    fn from_f64(x: f64) -> Option<Self> {
        if x.is_finite() {
            Some(TwoFloatBackend(TwoFloat::from_f64(x)))
        } else {
            None
        }
    }
    fn as_f64(self) -> f64 {
        self.0.to_f64().unwrap_or(0.0)
    }
    fn zero() -> Self {
        TwoFloatBackend(TwoFloat::from_f64(0.0))
    }
    fn one() -> Self {
        TwoFloatBackend(TwoFloat::from_f64(1.0))
    }
    fn pi() -> Self {
        TwoFloatBackend(twofloat::consts::PI)
    }
    fn two_pi() -> Self {
        TwoFloatBackend(twofloat::consts::TAU)
    }
    fn sin(self) -> Self {
        TwoFloatBackend(self.0.sin())
    }
    fn cos(self) -> Self {
        TwoFloatBackend(self.0.cos())
    }
    fn tan(self) -> Self {
        TwoFloatBackend(self.0.tan())
    }
    fn sqrt(self) -> Self {
        TwoFloatBackend(self.0.sqrt())
    }
    fn asin(self) -> Self {
        TwoFloatBackend(self.0.asin())
    }
    fn atan2(self, other: Self) -> Self {
        TwoFloatBackend(self.0.atan2(other.0))
    }
}

#[cfg(not(feature = "real-f64"))]
impl std::ops::Add for TwoFloatBackend {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        TwoFloatBackend(self.0 + rhs.0)
    }
}
#[cfg(not(feature = "real-f64"))]
impl std::ops::Sub for TwoFloatBackend {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        TwoFloatBackend(self.0 - rhs.0)
    }
}
#[cfg(not(feature = "real-f64"))]
impl std::ops::Mul for TwoFloatBackend {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        TwoFloatBackend(self.0 * rhs.0)
    }
}
#[cfg(not(feature = "real-f64"))]
impl std::ops::Div for TwoFloatBackend {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        TwoFloatBackend(self.0 / rhs.0)
    }
}
#[cfg(not(feature = "real-f64"))]
impl std::ops::Neg for TwoFloatBackend {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        TwoFloatBackend(-self.0)
    }
}

// ============== f64 backend (feature real-f64) ==============

#[cfg(feature = "real-f64")]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct F64Backend(pub(crate) f64);

#[cfg(feature = "real-f64")]
impl Backend for F64Backend {
    fn from_f64(x: f64) -> Option<Self> {
        if x.is_finite() {
            Some(F64Backend(x))
        } else {
            None
        }
    }
    fn as_f64(self) -> f64 {
        self.0
    }
    fn zero() -> Self {
        F64Backend(0.0)
    }
    fn one() -> Self {
        F64Backend(1.0)
    }
    fn pi() -> Self {
        F64Backend(core::f64::consts::PI)
    }
    fn two_pi() -> Self {
        F64Backend(core::f64::consts::TAU)
    }
    fn sin(self) -> Self {
        F64Backend(self.0.sin())
    }
    fn cos(self) -> Self {
        F64Backend(self.0.cos())
    }
    fn tan(self) -> Self {
        F64Backend(self.0.tan())
    }
    fn sqrt(self) -> Self {
        F64Backend(self.0.sqrt())
    }
    fn asin(self) -> Self {
        F64Backend(self.0.asin())
    }
    fn atan2(self, other: Self) -> Self {
        F64Backend(self.0.atan2(other.0))
    }
}

#[cfg(feature = "real-f64")]
impl std::ops::Add for F64Backend {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        F64Backend(self.0 + rhs.0)
    }
}
#[cfg(feature = "real-f64")]
impl std::ops::Sub for F64Backend {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        F64Backend(self.0 - rhs.0)
    }
}
#[cfg(feature = "real-f64")]
impl std::ops::Mul for F64Backend {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        F64Backend(self.0 * rhs.0)
    }
}
#[cfg(feature = "real-f64")]
impl std::ops::Div for F64Backend {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        F64Backend(self.0 / rhs.0)
    }
}
#[cfg(feature = "real-f64")]
impl std::ops::Neg for F64Backend {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        F64Backend(-self.0)
    }
}

// ============== Real<R> ==============

/// 标量类型：泛型于后端 R。对外通过 type alias [Real] 使用，不暴露 R。
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct RealInner<R>(pub(crate) R);

impl<R: Backend> Default for RealInner<R> {
    #[inline]
    fn default() -> Self {
        RealInner(R::zero())
    }
}

impl<R: Backend> Add for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn add(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(self.0 + rhs.0)
    }
}
impl<R: Backend> Sub for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn sub(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(self.0 - rhs.0)
    }
}
impl<R: Backend> Mul for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn mul(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(self.0 * rhs.0)
    }
}
impl<R: Backend> Div for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn div(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(self.0 / rhs.0)
    }
}
impl<R: Backend> Neg for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn neg(self) -> RealInner<R> {
        RealInner(-self.0)
    }
}

impl<R: Backend> AddAssign for RealInner<R> {
    #[inline]
    fn add_assign(&mut self, rhs: RealInner<R>) {
        self.0 = self.0 + rhs.0;
    }
}
impl<R: Backend> SubAssign for RealInner<R> {
    #[inline]
    fn sub_assign(&mut self, rhs: RealInner<R>) {
        self.0 = self.0 - rhs.0;
    }
}
impl<R: Backend> MulAssign for RealInner<R> {
    #[inline]
    fn mul_assign(&mut self, rhs: RealInner<R>) {
        self.0 = self.0 * rhs.0;
    }
}
impl<R: Backend> DivAssign for RealInner<R> {
    #[inline]
    fn div_assign(&mut self, rhs: RealInner<R>) {
        self.0 = self.0 / rhs.0;
    }
}

/// 固有方法：不依赖 trait 作用域即可调用。
impl<R: Backend> RealInner<R> {
    #[inline]
    pub fn sqrt(self) -> RealInner<R> {
        RealInner(self.0.sqrt())
    }
    #[inline]
    pub fn sin(self) -> RealInner<R> {
        RealInner(self.0.sin())
    }
    #[inline]
    pub fn cos(self) -> RealInner<R> {
        RealInner(self.0.cos())
    }
    #[inline]
    pub fn tan(self) -> RealInner<R> {
        RealInner(self.0.tan())
    }
    #[inline]
    pub fn max(self, other: RealInner<R>) -> RealInner<R> {
        if self >= other {
            self
        } else {
            other
        }
    }
    #[inline]
    pub fn min(self, other: RealInner<R>) -> RealInner<R> {
        if self <= other {
            self
        } else {
            other
        }
    }
}

/// 后端与 f64 的混合运算（Backend 层实现，避免 Real 层依赖具体类型）。
pub(crate) trait FromF64Like: Backend {
    fn mul_f64(r: Self, f: f64) -> Self;
    fn div_f64(r: Self, f: f64) -> Self;
    fn div_f64_right(f: f64, r: Self) -> Self;
    fn add_f64(r: Self, f: f64) -> Self;
    fn sub_f64(r: Self, f: f64) -> Self;
    fn sub_f64_right(f: f64, r: Self) -> Self;
}

#[cfg(not(feature = "real-f64"))]
impl FromF64Like for TwoFloatBackend {
    #[inline]
    fn mul_f64(r: Self, f: f64) -> Self {
        TwoFloatBackend(r.0 * TwoFloat::from_f64(f))
    }
    #[inline]
    fn div_f64(r: Self, f: f64) -> Self {
        TwoFloatBackend(r.0 / TwoFloat::from_f64(f))
    }
    #[inline]
    fn div_f64_right(f: f64, r: Self) -> Self {
        TwoFloatBackend(TwoFloat::from_f64(f) / r.0)
    }
    #[inline]
    fn add_f64(r: Self, f: f64) -> Self {
        TwoFloatBackend(r.0 + TwoFloat::from_f64(f))
    }
    #[inline]
    fn sub_f64(r: Self, f: f64) -> Self {
        TwoFloatBackend(r.0 - TwoFloat::from_f64(f))
    }
    #[inline]
    fn sub_f64_right(f: f64, r: Self) -> Self {
        TwoFloatBackend(TwoFloat::from_f64(f) - r.0)
    }
}

#[cfg(feature = "real-f64")]
impl FromF64Like for F64Backend {
    #[inline]
    fn mul_f64(r: Self, f: f64) -> Self {
        F64Backend(r.0 * f)
    }
    #[inline]
    fn div_f64(r: Self, f: f64) -> Self {
        F64Backend(r.0 / f)
    }
    #[inline]
    fn div_f64_right(f: f64, r: Self) -> Self {
        F64Backend(f / r.0)
    }
    #[inline]
    fn add_f64(r: Self, f: f64) -> Self {
        F64Backend(r.0 + f)
    }
    #[inline]
    fn sub_f64(r: Self, f: f64) -> Self {
        F64Backend(r.0 - f)
    }
    #[inline]
    fn sub_f64_right(f: f64, r: Self) -> Self {
        F64Backend(f - r.0)
    }
}

// f64 与 Real 混合运算
impl<R: Backend + FromF64Like> Mul<RealInner<R>> for f64 {
    type Output = RealInner<R>;
    #[inline]
    fn mul(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(R::mul_f64(rhs.0, self))
    }
}
impl<R: Backend + FromF64Like> Mul<f64> for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn mul(self, rhs: f64) -> RealInner<R> {
        RealInner(R::mul_f64(self.0, rhs))
    }
}
impl<R: Backend + FromF64Like> Div<RealInner<R>> for f64 {
    type Output = RealInner<R>;
    #[inline]
    fn div(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(R::div_f64_right(self, rhs.0))
    }
}
impl<R: Backend + FromF64Like> Sub<RealInner<R>> for f64 {
    type Output = RealInner<R>;
    #[inline]
    fn sub(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(R::sub_f64_right(self, rhs.0))
    }
}
impl<R: Backend + FromF64Like> Add<RealInner<R>> for f64 {
    type Output = RealInner<R>;
    #[inline]
    fn add(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(R::add_f64(rhs.0, self))
    }
}
impl<R: Backend + FromF64Like> Add<f64> for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn add(self, rhs: f64) -> RealInner<R> {
        RealInner(R::add_f64(self.0, rhs))
    }
}
impl<R: Backend + FromF64Like> Sub<f64> for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn sub(self, rhs: f64) -> RealInner<R> {
        RealInner(R::sub_f64(self.0, rhs))
    }
}
impl<R: Backend + FromF64Like> Div<f64> for RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn div(self, rhs: f64) -> RealInner<R> {
        RealInner(R::div_f64(self.0, rhs))
    }
}
impl<R: Backend> Mul<RealInner<R>> for &RealInner<R> {
    type Output = RealInner<R>;
    #[inline]
    fn mul(self, rhs: RealInner<R>) -> RealInner<R> {
        RealInner(self.0 * rhs.0)
    }
}

impl<R: Backend> fmt::Display for RealInner<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_f64().fmt(f)
    }
}

impl<R: Backend> PartialEq<f64> for RealInner<R> {
    #[inline]
    fn eq(&self, other: &f64) -> bool {
        match R::from_f64(*other) {
            Some(o) => self.0 == o,
            None => false,
        }
    }
}
impl<R: Backend> PartialOrd<f64> for RealInner<R> {
    #[inline]
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        R::from_f64(*other).and_then(|o| self.0.partial_cmp(&o))
    }
}

// ============== Type alias + real_const / zero / one / ToReal / real ==============

#[cfg(not(feature = "real-f64"))]
pub type Real = RealInner<TwoFloatBackend>;

#[cfg(feature = "real-f64")]
pub type Real = RealInner<F64Backend>;

/// 从 f64 构造 Real，**仅用于 const/static**。调用方须保证 `x` 为有限值；运行时一律用 [`real`]。
#[inline]
#[cfg(not(feature = "real-f64"))]
pub const fn real_const(x: f64) -> Real {
    RealInner(TwoFloatBackend(TwoFloat::from_f64(x)))
}

#[inline]
#[cfg(feature = "real-f64")]
pub const fn real_const(x: f64) -> Real {
    RealInner(F64Backend(x))
}

/// 从 i32 构造 Real。
#[inline]
pub fn from_i32(x: i32) -> Real {
    real_const(x as f64)
}

#[inline]
pub fn zero() -> Real {
    <Real as RealOps>::zero()
}

#[inline]
pub fn one() -> Real {
    <Real as RealOps>::one()
}

/// 可转为 Real 的类型（f64、i32、Real 等）。外界无感底层是 f64 还是 TwoFloat，统一用此构造。
pub trait ToReal {
    fn to_real(self) -> Real;
}

impl ToReal for f64 {
    #[inline]
    fn to_real(self) -> Real {
        <Real as RealOps>::from_f64(self).unwrap_or_else(<Real as RealOps>::zero)
    }
}

impl ToReal for i32 {
    #[inline]
    fn to_real(self) -> Real {
        from_i32(self)
    }
}

#[cfg(not(feature = "real-f64"))]
impl ToReal for RealInner<TwoFloatBackend> {
    #[inline]
    fn to_real(self) -> Real {
        self
    }
}
#[cfg(feature = "real-f64")]
impl ToReal for RealInner<F64Backend> {
    #[inline]
    fn to_real(self) -> Real {
        self
    }
}

/// 多态构造：`real(0.0)`、`real(1)`、`real(some_real)` 均得到 Real。**运行时首选**；f64 时非有限会回退为 0。
#[inline]
pub fn real<T: ToReal>(x: T) -> Real {
    x.to_real()
}

/// 弧度 → 角秒的换算系数（180×3600/π）。
#[inline]
pub fn arcsec_per_rad() -> Real {
    real(180.0 * 3600.0 / core::f64::consts::PI)
}

// ============== RealOps trait + impl for Real<R> ==============

/// 标量实数运算：算术、常数、三角函数。
pub trait RealOps:
    Copy
    + Clone
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + PartialOrd
    + Default
{
    fn from_f64(x: f64) -> Option<Self>
    where
        Self: Sized;
    fn as_f64(self) -> f64;
    fn to_i32_floor(self) -> i32 {
        self.as_f64().floor() as i32
    }
    fn to_i64_floor(self) -> i64 {
        self.as_f64().floor() as i64
    }
    fn to_i32_round(self) -> i32 {
        self.as_f64().round() as i32
    }
    fn to_i8_trunc_if_in_range(self) -> Option<i8> {
        let x = self.as_f64();
        if x.fract() == 0.0 && x >= i8::MIN as f64 && x <= i8::MAX as f64 {
            Some(x as i8)
        } else {
            None
        }
    }
    fn is_near(self, other: Self, eps: f64) -> bool {
        (self - other).abs().as_f64() < eps
    }
    fn powi(self, exp: i32) -> Self
    where
        Self: Sized,
    {
        Self::from_f64(self.as_f64().powi(exp)).unwrap_or_else(Self::zero)
    }
    fn zero() -> Self;
    fn one() -> Self;
    fn pi() -> Self;
    fn two_pi() -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self
    where
        Self: Sized,
    {
        Self::from_f64(self.as_f64().tan()).unwrap_or_else(Self::zero)
    }
    fn sqrt(self) -> Self;
    fn asin(self) -> Self
    where
        Self: Sized,
    {
        Self::from_f64(self.as_f64().asin()).unwrap_or_else(Self::zero)
    }
    fn atan2(self, other: Self) -> Self
    where
        Self: Sized,
    {
        Self::from_f64(self.as_f64().atan2(other.as_f64())).unwrap_or_else(Self::zero)
    }
    fn power_series_at(coeffs: &[f64], x: Self) -> Self
    where
        Self: Sized,
    {
        let mut sum = Self::zero();
        for a in coeffs.iter().rev() {
            let a_r = Self::from_f64(*a).unwrap_or_else(Self::zero);
            sum = sum * x + a_r;
        }
        sum
    }
    fn abs(self) -> Self
    where
        Self: Sized,
    {
        Self::from_f64(self.as_f64().abs()).unwrap_or_else(Self::zero)
    }
    fn floor(self) -> Self
    where
        Self: Sized,
    {
        Self::from_f64(self.as_f64().floor()).unwrap_or_else(Self::zero)
    }
    fn wrap_to_2pi(self) -> Self {
        let x = self.as_f64();
        let two_pi = core::f64::consts::TAU;
        let r = x % two_pi;
        let r = if r >= 0.0 { r } else { r + two_pi };
        Self::from_f64(r).unwrap_or_else(Self::zero)
    }
    fn wrap_to_signed_pi(self) -> Self {
        let x = self.as_f64();
        let two_pi = core::f64::consts::TAU;
        let pi = core::f64::consts::PI;
        let r = x % two_pi;
        let r = if r > pi {
            r - two_pi
        } else if r <= -pi {
            r + two_pi
        } else {
            r
        };
        Self::from_f64(r).unwrap_or_else(Self::zero)
    }
}

impl<R: Backend> RealOps for RealInner<R> {
    fn from_f64(x: f64) -> Option<Self> {
        R::from_f64(x).map(RealInner)
    }
    fn as_f64(self) -> f64 {
        self.0.as_f64()
    }
    fn zero() -> Self {
        RealInner(R::zero())
    }
    fn one() -> Self {
        RealInner(R::one())
    }
    fn pi() -> Self {
        RealInner(R::pi())
    }
    fn two_pi() -> Self {
        RealInner(R::two_pi())
    }
    fn sin(self) -> Self {
        RealInner(self.0.sin())
    }
    fn cos(self) -> Self {
        RealInner(self.0.cos())
    }
    fn sqrt(self) -> Self {
        RealInner(self.0.sqrt())
    }
    fn tan(self) -> Self {
        RealInner(self.0.tan())
    }
    fn asin(self) -> Self {
        RealInner(self.0.asin())
    }
    fn atan2(self, other: Self) -> Self {
        RealInner(self.0.atan2(other.0))
    }
    /// 纯 Real 取模，避免 twofloat 经 as_f64 丢失精度导致牛顿迭代收敛到错误根。
    fn wrap_to_2pi(self) -> Self {
        let two_pi = RealInner(R::two_pi());
        let n = (self / two_pi).floor();
        let r = self - n * two_pi;
        if r < RealInner(R::zero()) {
            r + two_pi
        } else {
            r
        }
    }
    fn wrap_to_signed_pi(self) -> Self {
        let two_pi = RealInner(R::two_pi());
        let pi = RealInner(R::pi());
        let n = (self / two_pi).floor();
        let r = self - n * two_pi;
        let r = if r < RealInner(R::zero()) { r + two_pi } else { r };
        if r > pi {
            r - two_pi
        } else {
            r
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn real_basic() {
        let z = Real::zero();
        let o = Real::one();
        assert_eq!(z.as_f64(), 0.0);
        assert_eq!(o.as_f64(), 1.0);
        assert!(Real::pi().is_near(real(core::f64::consts::PI), 1e-15));
    }

    #[test]
    fn real_abs() {
        let x: Real = <Real as RealOps>::from_f64(-123.34).unwrap();
        assert!(x.abs().is_near(real(123.34), 1e-10));
        assert!(<Real as RealOps>::from_f64(123.34).unwrap().abs().is_near(real(123.34), 1e-10));
    }

    #[test]
    fn real_parse_valid() {
        let f: f64 = "123.34".parse().unwrap();
        let x = <Real as RealOps>::from_f64(f).unwrap();
        assert!(x.is_near(real(123.34), 1e-10));
    }

    #[test]
    fn real_parse_invalid() {
        assert!("abc".parse::<f64>().is_err());
    }

    #[test]
    fn real_wrap() {
        let x = <Real as RealOps>::from_f64(3.0).unwrap();
        let w = x.wrap_to_2pi();
        assert!(w >= real(0) && w < real(core::f64::consts::TAU));
    }

    #[test]
    fn real_tan_zero() {
        let t = Real::zero();
        assert!(t.tan().abs().is_near(real(0), 1e-12));
    }

    #[test]
    fn real_tan_45_deg() {
        let rad: Real = crate::math::angle::deg2rad(45.0);
        let t = rad.tan();
        assert!(t.is_near(real(1.0), 1e-12));
    }

    #[test]
    fn real_tan_60_deg() {
        let rad: Real = crate::math::angle::deg2rad(60.0);
        let t = rad.tan();
        let sqrt3 = <Real as RealOps>::from_f64(3.0_f64.sqrt()).unwrap();
        assert!(t.is_near(sqrt3, 1e-12));
    }
}
