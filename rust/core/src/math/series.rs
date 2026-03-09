//! 幂级数、泊松级数。标量用 Real；入参通过 [crate::math::real::ToReal] 多态构造。

use crate::math::real::{real, Real, RealOps, ToReal};

/// 弧秒 → 弧度。入参可为 f64、i32、Real 等，统一得到 Real。
#[inline]
pub fn arcsec_to_rad(arcsec: impl ToReal) -> Real {
    real(arcsec) * (Real::pi() / (real(180) * real(3600)))
}

/// 幂级数 P(x) = Σ coeffs[k]*x^k，Horner 求值。x 可为 f64/i32/Real，结果 Real，外界无感 f64。
#[inline]
pub fn power_series_at(coeffs: &[f64], x: impl ToReal) -> Real {
    let x = real(x);
    coeffs
        .iter()
        .rev()
        .fold(Real::zero(), |sum, a| sum * x + real(*a))
}

/// 幂级数 P(x) 的导数系数：P'(x) = Σ (k+1)*coeffs[k+1]*x^k，返回 [1*coeffs[1], 2*coeffs[2], ...]
#[inline]
pub fn power_series_derivative_coeffs(coeffs: &[f64]) -> Vec<f64> {
    (1..coeffs.len()).map(|k| k as f64 * coeffs[k]).collect()
}

/// 幂级数导数 P'(x) 在 x 处的值。x 可为 f64/i32/Real，结果 Real。
#[inline]
pub fn power_series_derivative_at(coeffs: &[f64], x: impl ToReal) -> Real {
    if coeffs.len() <= 1 {
        return Real::zero();
    }
    let x = real(x);
    (1..coeffs.len())
        .rev()
        .fold(Real::zero(), |sum, k| sum * x + real((k as f64) * coeffs[k]))
}

/// 幂级数 P(x) = Σ coeffs[k]*x^k，系数为 Real，Horner 求值。
#[inline]
pub fn power_series_at_real(coeffs: &[Real], x: impl ToReal) -> Real {
    let x = real(x);
    coeffs
        .iter()
        .rev()
        .fold(Real::zero(), |sum, a| sum * x + *a)
}

/// 幂级数导数 P'(x) 在 x 处的值，系数为 Real。
#[inline]
pub fn power_series_derivative_at_real(coeffs: &[Real], x: impl ToReal) -> Real {
    if coeffs.len() <= 1 {
        return Real::zero();
    }
    let x = real(x);
    (1..coeffs.len())
        .rev()
        .fold(Real::zero(), |sum, k| sum * x + real(k as f64) * coeffs[k])
}

/// 单变量泊松周期项：θ = 2π·t/period，每项贡献 coef_c*cos(θ) + coef_s*sin(θ)。
/// 项格式 (period_cy, coef_c, coef_s)；t 为儒略世纪。
pub fn evaluate_poisson_periodic(terms: &[(f64, f64, f64)], t: f64) -> f64 {
    let two_pi = core::f64::consts::TAU;
    terms
        .iter()
        .map(|&(period, coef_c, coef_s)| {
            let theta = two_pi * t / period;
            coef_c * theta.cos() + coef_s * theta.sin()
        })
        .sum()
}

/// 单变量泊松周期项对 t 的导数（同单位/世纪）
pub fn evaluate_poisson_periodic_derivative(terms: &[(f64, f64, f64)], t: f64) -> f64 {
    let two_pi = core::f64::consts::TAU;
    terms
        .iter()
        .map(|&(period, coef_c, coef_s)| {
            let theta = two_pi * t / period;
            let omega = two_pi / period;
            (coef_s * theta.cos() - coef_c * theta.sin()) * omega
        })
        .sum()
}

/// 两路周期项：每项 (period, p_c, q_c, p_s, q_s)，返回 (p, q)
pub fn evaluate_poisson_periodic_two(terms: &[(f64, f64, f64, f64, f64)], t: f64) -> (f64, f64) {
    let two_pi = core::f64::consts::TAU;
    let mut p = 0.0_f64;
    let mut q = 0.0_f64;
    for &(period, p_c, q_c, p_s, q_s) in terms {
        let theta = two_pi * t / period;
        let c = theta.cos();
        let s = theta.sin();
        p += c * p_c + s * p_s;
        q += c * q_c + s * q_s;
    }
    (p, q)
}

/// 两路泊松周期项对 t 的导数
pub fn evaluate_poisson_periodic_two_derivative(
    terms: &[(f64, f64, f64, f64, f64)],
    t: f64,
) -> (f64, f64) {
    let two_pi = core::f64::consts::TAU;
    let mut dp = 0.0_f64;
    let mut dq = 0.0_f64;
    for &(period, p_c, q_c, p_s, q_s) in terms {
        let theta = two_pi * t / period;
        let omega = two_pi / period;
        let c = theta.cos();
        let s = theta.sin();
        dp += (-p_c * s + p_s * c) * omega;
        dq += (-q_c * s + q_s * c) * omega;
    }
    (dp, dq)
}

/// 泊松单项：θ = argument_coeffs · args，贡献 coef_sin*sin(θ) + coef_cos*cos(θ)
pub fn evaluate_poisson_term(
    argument_coeffs: &[i32],
    args: &[f64],
    coef_sin: f64,
    coef_cos: f64,
    reduce_angle: bool,
) -> f64 {
    let theta: f64 = argument_coeffs
        .iter()
        .zip(args.iter())
        .map(|(c, a)| (*c as f64) * a)
        .sum();
    let th = if reduce_angle {
        let two_pi = core::f64::consts::TAU;
        let r = theta % two_pi;
        if r >= 0.0 {
            r
        } else {
            r + two_pi
        }
    } else {
        theta
    };
    coef_sin * th.sin() + coef_cos * th.cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arcsec_to_rad_3600_equals_deg() {
        let rad = arcsec_to_rad(3600.0);
        assert!(real(1.0_f64.to_radians()).is_near(rad, 1e-10));
    }

    #[test]
    fn power_series_at_constant_and_linear() {
        let c = [2.0];
        assert!(power_series_at(&c, 10.0).is_near(real(2.0), 1e-10));
        let lin = [0.0, 3.0];
        assert!(power_series_at(&lin, 4.0).is_near(real(12.0), 1e-10));
        let quad = [1.0, 0.0, 1.0];
        assert!(power_series_at(&quad, 2.0).is_near(real(5.0), 1e-10));
    }

    #[test]
    fn power_series_derivative_coeffs_and_at() {
        let coeffs = [1.0, 2.0, 3.0];
        let deriv = power_series_derivative_coeffs(&coeffs);
        assert_eq!(deriv.len(), 2);
        assert!((deriv[0] - 2.0).abs() < 1e-10);
        assert!((deriv[1] - 6.0).abs() < 1e-10);
        assert!(power_series_derivative_at(&coeffs, 1.0).is_near(real(8.0), 1e-10));
        assert!(power_series_derivative_at(&[1.0], 1.0).is_near(real(0.0), 1e-10));
    }

    #[test]
    fn power_series_at_real_and_derivative() {
        let r = [real(1.0), real(0.0), real(1.0)];
        assert!(power_series_at_real(&r, real(2.0)).is_near(real(5.0), 1e-10));
        assert!(power_series_derivative_at_real(&r, real(2.0)).is_near(real(4.0), 1e-10));
        let single = [real(3.0)];
        assert!(power_series_derivative_at_real(&single, real(1.0)).is_near(real(0.0), 1e-10));
    }

    #[test]
    fn evaluate_poisson_periodic_and_derivative() {
        let terms = [(1.0, 1.0, 0.0)];
        let v = evaluate_poisson_periodic(&terms, 0.0);
        assert!((v - 1.0).abs() < 1e-10);
        let d = evaluate_poisson_periodic_derivative(&terms, 0.0);
        assert!(d.abs() < 1e-8);
    }

    #[test]
    fn evaluate_poisson_periodic_two_and_derivative() {
        let terms = [(1.0, 1.0, 0.0, 0.0, 0.0)];
        let (p, q) = evaluate_poisson_periodic_two(&terms, 0.0);
        assert!((p - 1.0).abs() < 1e-10 && (q - 0.0).abs() < 1e-10);
        let (dp, dq) = evaluate_poisson_periodic_two_derivative(&terms, 0.0);
        assert!(dp.abs() < 1e-8 && dq.abs() < 1e-8);
    }

    #[test]
    fn evaluate_poisson_term_reduce_angle() {
        let coeffs = [1];
        let args = [core::f64::consts::PI];
        let v = evaluate_poisson_term(&coeffs, &args, 1.0, 0.0, false);
        assert!((v - 0.0).abs() < 1e-10);
        let v2 = evaluate_poisson_term(&coeffs, &args, 1.0, 0.0, true);
        assert!(v2.abs() < 1e-10);
    }
}
