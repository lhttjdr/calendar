//! 旋转矩阵：坐标系层概念，表示两坐标架之间的旋转变换（无量纲 3×3）。
//! 与 [ReferenceFrame](crate::quantity::reference_frame::ReferenceFrame) 同层，非纯代数；实现依赖 [Mat](crate::math::algebra::mat::Mat)。

use crate::math::algebra::mat::{LinearComponent, Mat, ScaledBy};
use crate::math::real::Real;

use super::vector3::Vector3;

/// 旋转矩阵：3×3 无量纲矩阵，表示坐标架之间的旋转变换。
/// 元素为方向余弦等，左乘 `[Real; 3]`、`[Length; 3]`、`[Speed; 3]` 不改变量纲，仅改变方向。
#[derive(Clone, Copy, Debug)]
pub struct RotationMatrix(pub Mat<Real, 3, 3>);

impl RotationMatrix {
    #[inline]
    pub fn from_array(rows: [[Real; 3]; 3]) -> Self {
        Self(Mat::from(rows))
    }

    #[inline]
    pub fn as_mat(&self) -> &Mat<Real, 3, 3> {
        &self.0
    }

    /// 旋转矩阵 × 数值向量 → 数值向量（如 km、m/s 等，量纲不变）。
    #[inline]
    pub fn mul_vec(&self, v: [Real; 3]) -> [Real; 3] {
        self.0.mul_vec(v)
    }

    /// 旋转矩阵 × 物理量向量 → 物理量向量（如 [Length; 3]、[Speed; 3]，量纲不变）。
    #[inline]
    pub fn mul_vec_typed<V>(&self, v: &[V; 3]) -> [V; 3]
    where
        V: ScaledBy<Real>,
    {
        self.0.mul_vec_typed(v)
    }

    /// 旋转矩阵 × 物理量矢量 → 物理量矢量（[Vector3](Vector3)&lt;Length&gt; / [Vector3](Vector3)&lt;Speed&gt;）。
    #[inline]
    pub fn mul_vec_vec3<V>(&self, v: &Vector3<V>) -> Vector3<V>
    where
        V: LinearComponent<Real>,
    {
        Vector3::from_array(self.0.mul_vec_typed(&v.to_array()))
    }
}

impl From<[[Real; 3]; 3]> for RotationMatrix {
    fn from(rows: [[Real; 3]; 3]) -> Self {
        Self::from_array(rows)
    }
}

impl From<Mat<Real, 3, 3>> for RotationMatrix {
    fn from(m: Mat<Real, 3, 3>) -> Self {
        Self(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};
    use crate::quantity::length::Length;
    use crate::quantity::unit::LengthUnit;

    #[test]
    fn rotation_matrix_from_array_mul_vec() {
        let r = RotationMatrix::from_array([
            [real(1.0), real(0.0), real(0.0)],
            [real(0.0), real(1.0), real(0.0)],
            [real(0.0), real(0.0), real(1.0)],
        ]);
        let v = [real(1.0), real(2.0), real(3.0)];
        let out = r.mul_vec(v);
        assert!(out[0].is_near(real(1.0), 1e-10));
        assert!(out[1].is_near(real(2.0), 1e-10));
        assert!(out[2].is_near(real(3.0), 1e-10));
    }

    #[test]
    fn rotation_matrix_mul_vec_typed_and_vec3() {
        let r = RotationMatrix::from_array([
            [real(0.0), real(-1.0), real(0.0)],
            [real(1.0), real(0.0), real(0.0)],
            [real(0.0), real(0.0), real(1.0)],
        ]);
        let x = Length::from_value(real(1.0), LengthUnit::Meter);
        let z = Length::from_value(real(0.0), LengthUnit::Meter);
        let vec_len: [Length; 3] = [x, z, z];
        let out = r.mul_vec_typed(&vec_len);
        assert!(out[0].meters().is_near(real(0.0), 1e-10));
        assert!(out[1].meters().is_near(real(1.0), 1e-10));
        let v3 = Vector3::from_lengths([x, z, z]);
        let w3 = r.mul_vec_vec3(&v3);
        assert!(w3.x().meters().is_near(real(0.0), 1e-10));
        assert!(w3.y().meters().is_near(real(1.0), 1e-10));
    }

    #[test]
    fn rotation_matrix_from_impls() {
        let arr = [[real(1.0), real(0.0), real(0.0)], [real(0.0), real(1.0), real(0.0)], [real(0.0), real(0.0), real(1.0)]];
        let r1 = RotationMatrix::from(arr);
        let r2 = RotationMatrix::from_array(arr);
        assert!(r1.as_mat().rows[0][0].is_near(r2.as_mat().rows[0][0], 1e-10));
    }
}
