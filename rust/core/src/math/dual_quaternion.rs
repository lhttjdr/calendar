//! 对偶四元数：real + ε·dual，用于刚体变换。标量类型 T 为 f64 或 Real。

use super::algebra::mat::ScalarNorm;
use super::quaternion::Quaternion;
use std::ops::Neg;

#[derive(Clone, Copy, Debug)]
pub struct DualQuaternion<T>
where
    T: ScalarNorm + Neg<Output = T>,
{
    pub real: Quaternion<T>,
    pub dual: Quaternion<T>,
}

impl<T: ScalarNorm + Neg<Output = T>> DualQuaternion<T> {
    #[inline]
    pub fn new(real: Quaternion<T>, dual: Quaternion<T>) -> Self {
        DualQuaternion { real, dual }
    }

    #[inline]
    pub fn plus(p: DualQuaternion<T>, q: DualQuaternion<T>) -> DualQuaternion<T> {
        DualQuaternion {
            real: p.real.plus(q.real),
            dual: p.dual.plus(q.dual),
        }
    }

    #[inline]
    pub fn mult(p: DualQuaternion<T>, q: DualQuaternion<T>) -> DualQuaternion<T> {
        let real = Quaternion::grossman(p.real, q.real);
        let dual = Quaternion::grossman(p.real, q.dual).plus(Quaternion::grossman(p.dual, q.real));
        DualQuaternion { real, dual }
    }

    #[inline]
    pub fn scale(q: DualQuaternion<T>, r: T) -> DualQuaternion<T> {
        DualQuaternion {
            real: q.real.scale(r),
            dual: q.dual.scale(r),
        }
    }

    #[inline]
    pub fn conjugate2(q: DualQuaternion<T>) -> DualQuaternion<T> {
        DualQuaternion {
            real: q.real.conjugate(),
            dual: q.dual.conjugate(),
        }
    }
}

impl DualQuaternion<f64> {
    #[inline]
    pub fn minus(p: DualQuaternion<f64>, q: DualQuaternion<f64>) -> DualQuaternion<f64> {
        DualQuaternion {
            real: p.real.minus(q.real),
            dual: p.dual.minus(q.dual),
        }
    }

    #[inline]
    pub fn neg(q: DualQuaternion<f64>) -> DualQuaternion<f64> {
        DualQuaternion {
            real: q.real.scale(-1.0),
            dual: q.dual.scale(-1.0),
        }
    }

    #[inline]
    pub fn conjugate1(q: DualQuaternion<f64>) -> DualQuaternion<f64> {
        DualQuaternion {
            real: q.real,
            dual: q.dual.scale(-1.0),
        }
    }

    #[inline]
    pub fn conjugate3(q: DualQuaternion<f64>) -> DualQuaternion<f64> {
        DualQuaternion {
            real: q.real.conjugate(),
            dual: q.dual.conjugate().scale(-1.0),
        }
    }

    pub fn normalize(q: DualQuaternion<f64>) -> DualQuaternion<f64> {
        let n = Quaternion::norm(q.real);
        let inv = 1.0 / n;
        let dot = Quaternion::<f64>::dot(q.real, q.dual);
        let factor = inv * inv * inv * dot;
        DualQuaternion {
            real: q.real.scale(inv),
            dual: q.dual.scale(inv).minus(q.real.scale(factor)),
        }
    }
}

impl DualQuaternion<super::real::Real> {
    #[inline]
    pub fn minus(
        p: DualQuaternion<super::real::Real>,
        q: DualQuaternion<super::real::Real>,
    ) -> DualQuaternion<super::real::Real> {
        DualQuaternion {
            real: p.real.minus(q.real),
            dual: p.dual.minus(q.dual),
        }
    }

    #[inline]
    pub fn neg(q: DualQuaternion<super::real::Real>) -> DualQuaternion<super::real::Real> {
        use super::real::real;
        DualQuaternion {
            real: q.real.scale(real(-1.0)),
            dual: q.dual.scale(real(-1.0)),
        }
    }

    #[inline]
    pub fn conjugate1(
        q: DualQuaternion<super::real::Real>,
    ) -> DualQuaternion<super::real::Real> {
        use super::real::real;
        DualQuaternion {
            real: q.real,
            dual: q.dual.scale(real(-1.0)),
        }
    }

    #[inline]
    pub fn conjugate3(
        q: DualQuaternion<super::real::Real>,
    ) -> DualQuaternion<super::real::Real> {
        use super::real::real;
        DualQuaternion {
            real: q.real.conjugate(),
            dual: q.dual.conjugate().scale(real(-1.0)),
        }
    }

    pub fn normalize(
        q: DualQuaternion<super::real::Real>,
    ) -> DualQuaternion<super::real::Real> {
        use super::real::real;
        let n = Quaternion::norm(q.real);
        let inv = real(1.0) / n;
        let dot = Quaternion::<super::real::Real>::dot(q.real, q.dual);
        let factor = inv * inv * inv * dot;
        DualQuaternion {
            real: q.real.scale(inv),
            dual: q.dual.scale(inv).minus(q.real.scale(factor)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::algebra::vec::Vec3;
    use crate::math::quaternion::Quaternion;
    use crate::math::real::{real, RealOps};

    fn q_id_f64() -> DualQuaternion<f64> {
        DualQuaternion::new(
            Quaternion::new(1.0, Vec3::new_3(0.0, 0.0, 0.0)),
            Quaternion::new(0.0, Vec3::new_3(0.0, 0.0, 0.0)),
        )
    }

    #[test]
    fn dual_quaternion_f64_new_plus_mult_scale() {
        let a = q_id_f64();
        let b = DualQuaternion::new(
            Quaternion::new(0.0, Vec3::new_3(1.0, 0.0, 0.0)),
            Quaternion::new(0.0, Vec3::new_3(0.0, 0.0, 0.0)),
        );
        let sum = DualQuaternion::plus(a, b);
        assert!((sum.real.scalar - 1.0).abs() < 1e-10);
        assert!((sum.real.vector.x() - 1.0).abs() < 1e-10);
        let prod = DualQuaternion::mult(a, b);
        assert!((prod.real.vector.x() - 1.0).abs() < 1e-10);
        let s = DualQuaternion::scale(b, 2.0);
        assert!((s.real.vector.x() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn dual_quaternion_f64_conjugate2_minus_neg_conjugate1_conjugate3() {
        let q: DualQuaternion<f64> = DualQuaternion::new(
            Quaternion::new(1.0, Vec3::new_3(0.1, 0.2, 0.3)),
            Quaternion::new(0.0, Vec3::new_3(0.0, 0.0, 0.0)),
        );
        let c2 = DualQuaternion::conjugate2(q);
        assert!((c2.real.vector.x() + 0.1).abs() < 1e-10);
        let neg = DualQuaternion::<f64>::neg(q);
        assert!((neg.real.scalar + 1.0).abs() < 1e-10);
        let c1 = DualQuaternion::<f64>::conjugate1(q);
        assert!((c1.real.scalar - 1.0).abs() < 1e-10);
        let c3 = DualQuaternion::<f64>::conjugate3(q);
        assert!((c3.real.vector.x() + 0.1).abs() < 1e-10);
        let m = DualQuaternion::<f64>::minus(q, neg);
        assert!((m.real.scalar - 2.0).abs() < 1e-10);
    }

    #[test]
    fn dual_quaternion_f64_normalize() {
        let q: DualQuaternion<f64> = DualQuaternion::new(
            Quaternion::new(1.0, Vec3::new_3(0.0, 0.0, 0.0)),
            Quaternion::new(0.0, Vec3::new_3(0.1, 0.0, 0.0)),
        );
        let n = DualQuaternion::<f64>::normalize(q);
        let nr: f64 = Quaternion::norm(n.real);
        assert!((nr - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dual_quaternion_real_ops() {
        use crate::math::real::Real;
        let z = real(0.0);
        let o = real(1.0);
        let q: DualQuaternion<Real> = DualQuaternion::new(
            Quaternion::new(o, Vec3::new_3(z, z, z)),
            Quaternion::new(z, Vec3::new_3(z, z, z)),
        );
        let neg = DualQuaternion::<Real>::neg(q);
        assert!(neg.real.scalar.is_near(real(-1.0), 1e-10));
        let c1 = DualQuaternion::<Real>::conjugate1(q);
        assert!(c1.real.scalar.is_near(o, 1e-10));
        let c3 = DualQuaternion::<Real>::conjugate3(q);
        assert!(c3.real.scalar.is_near(o, 1e-10));
        let n = DualQuaternion::<Real>::normalize(q);
        assert!(Quaternion::<Real>::norm(n.real).is_near(o, 1e-10));
    }
}
