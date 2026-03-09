//! 角度工具：弧度归一化 [0, 2π)、[-π, π] 等。角度与平面角。
//! 标量统一用 Real；入参通过 [crate::math::real::ToReal] 多态构造，外界无感 f64/TwoFloat。

use crate::math::real::{real, Real, RealOps, ToReal};

/// 度 → 弧度。入参可为 f64、i32、Real 等，统一得到 Real。
#[inline]
pub fn deg2rad(deg: impl ToReal) -> Real {
    real(deg) * (Real::pi() / real(180))
}

/// 度分秒 (d,m,s) → 弧度（Real）
#[inline]
pub fn dms2rad(d: impl ToReal, m: impl ToReal, s: impl ToReal) -> Real {
    deg2rad(real(d) + real(m) / real(60) + real(s) / real(3600))
}

/// 弧度归化到 [0, 2pi)
pub fn wrap_to_2pi(rad: f64) -> f64 {
    let two_pi = core::f64::consts::TAU;
    let r = rad % two_pi;
    if r >= 0.0 {
        r
    } else {
        r + two_pi
    }
}

/// 将弧度归化到 [-π, π]
#[inline]
pub fn wrap_to_signed_pi(rad: f64) -> f64 {
    let two_pi = core::f64::consts::TAU;
    let r = rad % two_pi;
    if r > core::f64::consts::PI {
        r - two_pi
    } else if r < -core::f64::consts::PI {
        r + two_pi
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn test_wrap_to_2pi() {
        assert!(real(wrap_to_2pi(0.0)).is_near(real(0), 1e-15));
        assert!(real(wrap_to_2pi(core::f64::consts::TAU)).is_near(real(0), 1e-15));
        assert!(real(wrap_to_2pi(-0.1)).is_near(real(core::f64::consts::TAU - 0.1), 1e-15));
    }

    #[test]
    fn test_wrap_to_signed_pi() {
        assert!(real(wrap_to_signed_pi(0.0)).is_near(real(0), 1e-15));
        assert!(real(wrap_to_signed_pi(core::f64::consts::PI)).is_near(real(core::f64::consts::PI), 1e-15));
    }

    /// 测试: deg2rad(360) == TAU
    #[test]
    fn deg2rad_360_equals_tau() {
        let r = deg2rad(360.0);
        assert!(r.is_near(real(core::f64::consts::TAU), 1e-12));
    }

    /// 测试: sec2rad(180*60*60) == PI
    #[test]
    fn sec2rad_180_deg_equals_pi() {
        let r = dms2rad(0.0, 0.0, 180.0 * 3600.0);
        assert!(r.is_near(real(core::f64::consts::PI), 1e-10));
    }
}
