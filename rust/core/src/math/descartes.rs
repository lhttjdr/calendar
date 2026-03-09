//! 直角坐标与对偶四元数变换：平移/旋转/变换点与矢量。f64 接口用 Vec3&lt;f64&gt;；rotation_from_matrix 用 Real 计算并返回 DualQuaternion&lt;Real&gt;。

use super::algebra::vec::Vec3;
use super::dual_quaternion::DualQuaternion;
use super::quaternion::Quaternion;
use super::real::{zero, Real};

fn zero_vec3_f64() -> Vec3<f64> {
    Vec3::new_3(0.0, 0.0, 0.0)
}

fn zero_vec3_real() -> Vec3<Real> {
    Vec3::new_3(zero(), zero(), zero())
}

/// 平移对偶四元数：平移向量 p 的负（将点从原点移出）。平移向量 p 的负。
#[inline]
pub fn translation(p: Vec3<f64>) -> DualQuaternion<f64> {
    let v = p.neg();
    let real = Quaternion::new(1.0, zero_vec3_f64());
    let dual = Quaternion::new(0.0, v.scale(0.5));
    DualQuaternion::new(real, dual)
}

/// 绕轴 axis 旋转 theta（弧度）的对偶四元数。
#[inline]
pub fn rotation(axis: Vec3<f64>, theta: f64) -> DualQuaternion<f64> {
    let half_theta = -theta * 0.5;
    let ax = axis.normalize();
    let real = Quaternion::new(
        half_theta.cos(),
        ax.scale(half_theta.sin()),
    );
    let dual = Quaternion::new(0.0, zero_vec3_f64());
    DualQuaternion::new(real, dual)
}

/// 从 3×3 旋转矩阵得到纯旋转的对偶四元数（dual 部分为 0）。全程 Real 运算，返回 DualQuaternion&lt;Real&gt;。
#[inline]
pub fn rotation_from_matrix(r: &[[Real; 3]; 3]) -> DualQuaternion<Real> {
    let q = Quaternion::<Real>::from_rotation_matrix(r);
    DualQuaternion::new(q.normalize(), Quaternion::new(zero(), zero_vec3_real()))
}

/// 用对偶四元数 t 变换点 p，返回直角坐标。result = (t * point) * t.conj3。
#[inline]
pub fn transform(p: Vec3<f64>, t: DualQuaternion<f64>) -> Vec3<f64> {
    let point = DualQuaternion::new(
        Quaternion::new(1.0, zero_vec3_f64()),
        Quaternion::new(0.0, p),
    );
    let result = DualQuaternion::mult(
        DualQuaternion::mult(t, point),
        DualQuaternion::<f64>::conjugate3(t),
    );
    result.dual.vector
}

/// 用对偶四元数 t 的旋转部分变换矢量 v（平移不影响矢量）。
#[inline]
pub fn transform_vector(v: Vec3<f64>, t: DualQuaternion<f64>) -> Vec3<f64> {
    let vec_dq = DualQuaternion::new(
        Quaternion::new(0.0, zero_vec3_f64()),
        Quaternion::new(0.0, v),
    );
    let result = DualQuaternion::mult(
        DualQuaternion::mult(t, vec_dq),
        DualQuaternion::<f64>::conjugate3(t),
    );
    result.dual.vector
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn translation_moves_origin_by_one() {
        let origin = Vec3::new_3(0.0, 0.0, 0.0);
        let t = translation(Vec3::new_3(1.0, 0.0, 0.0));
        let q = transform(origin, t);
        let n2 = q.x() * q.x() + q.y() * q.y() + q.z() * q.z();
        assert!((n2 - 1.0).abs() < 1e-10, "translated point should be at distance 1 from origin");
    }

    #[test]
    fn rotation_z_90_deg_perpendicular_and_unit() {
        let axis = Vec3::new_3(0.0, 0.0, 1.0);
        let t = rotation(axis, std::f64::consts::FRAC_PI_2);
        let v = Vec3::new_3(1.0, 0.0, 0.0);
        let w = transform_vector(v, t);
        assert!(w.z().abs() < 1e-10);
        let n2 = w.x() * w.x() + w.y() * w.y();
        assert!((n2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rotation_from_matrix_identity() {
        let z = real(0.0);
        let o = real(1.0);
        let r = [[o, z, z], [z, o, z], [z, z, o]];
        let dq = rotation_from_matrix(&r);
        assert!(dq.dual.vector.x().is_near(z, 1e-10));
        assert!(dq.dual.vector.y().is_near(z, 1e-10));
        assert!(dq.dual.vector.z().is_near(z, 1e-10));
    }

    #[test]
    fn transform_vector_pure_translation_unchanged() {
        let t = translation(Vec3::new_3(1.0, 2.0, 3.0));
        let v = Vec3::new_3(1.0, 0.0, 0.0);
        let w = transform_vector(v, t);
        assert!((w.x() - 1.0).abs() < 1e-10);
        assert!(w.y().abs() < 1e-10);
        assert!(w.z().abs() < 1e-10);
    }
}
