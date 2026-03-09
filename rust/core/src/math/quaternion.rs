//! 四元数：标量 + 三维向量 (scalar, vector)。标量可为 f64 或 Real。

use super::algebra::mat::ScalarNorm;
use super::algebra::vec::Vec3;
use super::real::{real, Real};
use std::ops::Neg;

/// 四元数：标量与向量分量类型为 T（f64 或 Real）。
#[derive(Clone, Copy, Debug)]
pub struct Quaternion<T>
where
    T: ScalarNorm + Neg<Output = T>,
{
    pub scalar: T,
    pub vector: Vec3<T>,
}

impl<T: ScalarNorm + Neg<Output = T>> Quaternion<T> {
    #[inline]
    pub fn new(scalar: T, vector: Vec3<T>) -> Self {
        Quaternion { scalar, vector }
    }

    #[inline]
    pub fn from_components(s: T, x: T, y: T, z: T) -> Self {
        Quaternion {
            scalar: s,
            vector: Vec3::new_3(x, y, z),
        }
    }

    /// 共轭：标量不变，向量取反
    #[inline]
    pub fn conjugate(self) -> Self {
        Quaternion {
            scalar: self.scalar,
            vector: self.vector.neg(),
        }
    }

    #[inline]
    pub fn plus(self, q: Quaternion<T>) -> Quaternion<T> {
        Quaternion {
            scalar: self.scalar + q.scalar,
            vector: Vec3::new_3(
                self.vector.x() + q.vector.x(),
                self.vector.y() + q.vector.y(),
                self.vector.z() + q.vector.z(),
            ),
        }
    }

}

impl Quaternion<f64> {
    #[inline]
    pub fn minus(self, q: Quaternion<f64>) -> Quaternion<f64> {
        self.plus(q.scale(-1.0))
    }
}

impl<T: ScalarNorm + Neg<Output = T>> Quaternion<T> {
    #[inline]
    pub fn scale(self, d: T) -> Quaternion<T> {
        Quaternion {
            scalar: self.scalar * d,
            vector: self.vector.scale(d),
        }
    }

    /// Hamilton 积 (Grossman)：p * q
    #[inline]
    pub fn grossman(p: Quaternion<T>, q: Quaternion<T>) -> Quaternion<T> {
        let s = p.scalar * q.scalar - p.vector.dot(q.vector);
        let v_scale_q = q.vector.scale(p.scalar);
        let v_scale_p = p.vector.scale(q.scalar);
        let cross_pq = p.vector.cross(q.vector);
        let v = Vec3::new_3(
            v_scale_q.x() + v_scale_p.x() + cross_pq.x(),
            v_scale_q.y() + v_scale_p.y() + cross_pq.y(),
            v_scale_q.z() + v_scale_p.z() + cross_pq.z(),
        );
        Quaternion { scalar: s, vector: v }
    }

    /// 范数：√(q * q*)
    #[inline]
    pub fn norm(self) -> T {
        Self::grossman(self, self.conjugate()).scalar.sqrt()
    }

    #[inline]
    pub fn normalize(self) -> Quaternion<T> {
        let n = self.norm();
        self.scale(T::one() / n)
    }
}

impl Quaternion<f64> {
    #[inline]
    pub fn div(self, d: f64) -> Quaternion<f64> {
        self.scale(1.0 / d)
    }

    #[inline]
    pub fn dot(p: Quaternion<f64>, q: Quaternion<f64>) -> f64 {
        (Self::grossman(p.conjugate(), q).scalar + Self::grossman(q.conjugate(), p).scalar) * 0.5
    }
}

impl Quaternion<Real> {
    #[inline]
    pub fn minus(self, q: Quaternion<Real>) -> Quaternion<Real> {
        self.plus(q.scale(real(-1.0)))
    }

    #[inline]
    pub fn div(self, d: Real) -> Quaternion<Real> {
        self.scale(real(1.0) / d)
    }

    #[inline]
    pub fn dot(p: Quaternion<Real>, q: Quaternion<Real>) -> Real {
        (Self::grossman(p.conjugate(), q).scalar + Self::grossman(q.conjugate(), p).scalar) * real(0.5)
    }

    /// 从 3×3 旋转矩阵构造单位四元数（满足 v' = q v q*）。数值稳定分支。全程 Real 运算，不转 f64。
    pub fn from_rotation_matrix(r: &[[Real; 3]; 3]) -> Quaternion<Real> {
        let (m00, m01, m02) = (r[0][0], r[0][1], r[0][2]);
        let (m10, m11, m12) = (r[1][0], r[1][1], r[1][2]);
        let (m20, m21, m22) = (r[2][0], r[2][1], r[2][2]);
        let trace = m00 + m11 + m22;
        let neg_999 = real(-0.999);
        let half = real(0.5);
        let quarter = real(0.25);
        if trace > neg_999 {
            let mut w = (real(1.0) + trace).sqrt() * half;
            if w == real(0.0) {
                w = real(1.0);
            }
            let s = real(1.0) / (real(4.0) * w);
            Quaternion::new(
                w,
                Vec3::new_3((m21 - m12) * s, (m02 - m20) * s, (m10 - m01) * s),
            )
        } else if m00 >= m11 && m00 >= m22 {
            let s = real(2.0) * (real(1.0) + m00 - m11 - m22).sqrt();
            Quaternion::new(
                (m21 - m12) / s,
                Vec3::new_3(quarter * s, (m01 + m10) / s, (m02 + m20) / s),
            )
        } else if m11 >= m22 {
            let s = real(2.0) * (real(1.0) + m11 - m00 - m22).sqrt();
            Quaternion::new(
                (m02 - m20) / s,
                Vec3::new_3((m01 + m10) / s, quarter * s, (m12 + m21) / s),
            )
        } else {
            let s = real(2.0) * (real(1.0) + m22 - m00 - m11).sqrt();
            Quaternion::new(
                (m10 - m01) / s,
                Vec3::new_3((m02 + m20) / s, (m12 + m21) / s, quarter * s),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::real::RealOps;
    use super::*;

    #[test]
    fn quaternion_from_rotation_matrix_identity() {
        let r = [
            [real(1.0), real(0.0), real(0.0)],
            [real(0.0), real(1.0), real(0.0)],
            [real(0.0), real(0.0), real(1.0)],
        ];
        let q = Quaternion::<Real>::from_rotation_matrix(&r);
        assert!(q.scalar.is_near(real(1.0), 1e-10));
        assert!(q.vector.x().abs().is_near(real(0), 1e-10) && q.vector.y().abs().is_near(real(0), 1e-10) && q.vector.z().abs().is_near(real(0), 1e-10));
    }

    #[test]
    fn quaternion_grossman_conjugate_norm() {
        let q = Quaternion::from_components(1.0, 0.0, 0.0, 0.0);
        assert!(real(q.norm()).is_near(real(1.0), 1e-15));
    }
}
