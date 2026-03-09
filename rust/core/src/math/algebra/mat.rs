//! 泛型矩阵：维度为 const 泛型 R×C，标量 T 满足四则运算。
//! 6×6 状态转移 [R R_dot; 0 R] 用 Mat<T, 6, 6> 表达。

use std::ops::{Add, Div, Mul, Sub};

use crate::math::real::{Real, RealOps};

/// 标量约束：四则运算；矩阵元与向量分量用。
pub trait Scalar: Copy + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> {}

impl<T> Scalar for T where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>
{
}

/// 在 Scalar 之上、范数/归一化所需的最小扩展：sqrt、zero、one。
/// 物理量系统与 Vec::norm/normalize 只依赖 Scalar + ScalarNorm，不依赖 Real。
pub trait ScalarNorm: Scalar + Default + PartialEq {
    fn sqrt(self) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
}

impl ScalarNorm for f64 {
    #[inline]
    fn sqrt(self) -> Self {
        self.sqrt()
    }
    #[inline]
    fn zero() -> Self {
        0.0
    }
    #[inline]
    fn one() -> Self {
        1.0
    }
}

impl ScalarNorm for Real {
    #[inline]
    fn sqrt(self) -> Self {
        RealOps::sqrt(self)
    }
    #[inline]
    fn zero() -> Self {
        RealOps::zero()
    }
    #[inline]
    fn one() -> Self {
        RealOps::one()
    }
}

/// 可被标量 **S** 数乘的类型：提供 `zero()` 与 `scaled_by(self, s: S)`。泛型 **S**（物理量用 Real，如 f64；矩阵元 T 亦可）。
/// 用于 Mat×向量 的 `mul_vec_typed`：向量元 V 满足 `ScaledBy<T>` 即可，不要求 V 是「域元素」。
pub trait ScaledBy<S>: Copy + Add<Output = Self> {
    fn zero() -> Self;
    fn scaled_by(self, s: S) -> Self;
}

/// **与 ScaledBy 的区别**：ScaledBy&lt;S&gt; 只描述「可被 S 数乘」；LinearComponent&lt;R&gt; 是**概念名**，
/// 表示「实数域 R 上的矢量分量」= Add + Sub + ScaledBy&lt;R&gt;，标量用 **Real** 抽象，不固定 f64。无额外方法，由 blanket 自动实现。
pub trait LinearComponent<R>: Copy + Add<Output = Self> + Sub<Output = Self> + ScaledBy<R>
where
    R: RealOps,
{
}

impl<T, R> LinearComponent<R> for T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + ScaledBy<R>,
    R: RealOps,
{
}

/// R×C 矩阵，行优先：`rows[i][j]` 为第 i 行第 j 列。
#[derive(Clone, Copy, Debug)]
pub struct Mat<T, const R: usize, const C: usize>
where
    T: Scalar,
{
    pub rows: [[T; C]; R],
}

impl<T, const R: usize, const C: usize> Mat<T, R, C>
where
    T: Scalar + Default,
{
    #[inline]
    pub fn new(rows: [[T; C]; R]) -> Self {
        Mat { rows }
    }

    /// 矩阵乘向量（泛型维度）：`(M·v)[i] = sum_j rows[i][j]*v[j]`。
    #[inline]
    pub fn mul_vec_generic(&self, v: &[T; C]) -> [T; R] {
        let mut out: [T; R] = std::array::from_fn(|_| T::default());
        for i in 0..R {
            let mut s = T::default();
            for j in 0..C {
                s = s + self.rows[i][j] * v[j];
            }
            out[i] = s;
        }
        out
    }

    /// 数乘：每元乘以 s。
    #[inline]
    pub fn scale(&self, s: T) -> Mat<T, R, C> {
        let mut out = self.rows;
        for i in 0..R {
            for j in 0..C {
                out[i][j] = out[i][j] * s;
            }
        }
        Mat::new(out)
    }

    #[inline]
    pub fn to_array(self) -> [[T; C]; R] {
        self.rows
    }
}

impl<T: Scalar> Mat<T, 3, 3> {
    #[inline]
    pub fn new_3x3(rows: [[T; 3]; 3]) -> Self {
        Mat { rows }
    }

    #[inline]
    pub fn mul_vec(&self, v: [T; 3]) -> [T; 3] {
        let m = &self.rows;
        [
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ]
    }

    /// 无量纲矩阵 × 物理量向量：M[i][j]: S，v[j]: V，返回 [V; 3]；不落为 f64，满足层级约束。
    #[inline]
    pub fn mul_vec_typed<V>(&self, v: &[V; 3]) -> [V; 3]
    where
        V: ScaledBy<T>,
    {
        let m = &self.rows;
        [
            v[0].scaled_by(m[0][0]) + v[1].scaled_by(m[0][1]) + v[2].scaled_by(m[0][2]),
            v[0].scaled_by(m[1][0]) + v[1].scaled_by(m[1][1]) + v[2].scaled_by(m[1][2]),
            v[0].scaled_by(m[2][0]) + v[1].scaled_by(m[2][1]) + v[2].scaled_by(m[2][2]),
        ]
    }

    #[inline]
    pub fn mul_mat(&self, other: &Mat<T, 3, 3>) -> Mat<T, 3, 3> {
        let a = &self.rows;
        let b = &other.rows;
        let mut out = [[a[0][0] * b[0][0]; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
            }
        }
        Mat { rows: out }
    }

    #[inline]
    pub fn transpose(&self) -> Mat<T, 3, 3> {
        let m = &self.rows;
        Mat {
            rows: [
                [m[0][0], m[1][0], m[2][0]],
                [m[0][1], m[1][1], m[2][1]],
                [m[0][2], m[1][2], m[2][2]],
            ],
        }
    }
}

impl<T: Scalar> From<[[T; 3]; 3]> for Mat<T, 3, 3> {
    fn from(rows: [[T; 3]; 3]) -> Self {
        Mat { rows }
    }
}

impl<T: Scalar> From<Mat<T, 3, 3>> for [[T; 3]; 3] {
    fn from(m: Mat<T, 3, 3>) -> Self {
        m.rows
    }
}

impl<T: Scalar + Default> Mat<T, 6, 6> {
    #[inline]
    pub fn mul_vec6(&self, v: [T; 6]) -> [T; 6] {
        self.mul_vec_generic(&v)
    }

    pub fn from_block_r_rdot(r: &[[T; 3]; 3], r_dot: &[[T; 3]; 3]) -> Self
    where
        T: Copy + Default,
    {
        let [r0, r1, r2] = r;
        let [d0, d1, d2] = r_dot;
        let z = T::default();
        let zero = [z, z, z];
        Mat::new([
            [r0[0], r0[1], r0[2], d0[0], d0[1], d0[2]],
            [r1[0], r1[1], r1[2], d1[0], d1[1], d1[2]],
            [r2[0], r2[1], r2[2], d2[0], d2[1], d2[2]],
            [zero[0], zero[1], zero[2], r0[0], r0[1], r0[2]],
            [zero[0], zero[1], zero[2], r1[0], r1[1], r1[2]],
            [zero[0], zero[1], zero[2], r2[0], r2[1], r2[2]],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn mat3_mul_vec_identity() {
        let i = Mat::new([
            [1.0_f64, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]);
        let v = [1.0_f64, 2.0, 3.0];
        let w = i.mul_vec(v);
        assert!(real(w[0]).is_near(real(1.0), 1e-15) && real(w[1]).is_near(real(2.0), 1e-15) && real(w[2]).is_near(real(3.0), 1e-15));
    }

    #[test]
    fn mat3_transpose() {
        let a = Mat::new([
            [1.0_f64, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ]);
        let at = a.transpose();
        let at_arr = at.to_array();
        assert!(real(at_arr[0][1]).is_near(real(4.0), 1e-15) && real(at_arr[1][0]).is_near(real(2.0), 1e-15));
    }

    #[test]
    fn mat6_from_block_mul_vec() {
        let r = [[1.0_f64, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let r_dot = [[0.0_f64; 3]; 3];
        let m6 = Mat::<f64, 6, 6>::from_block_r_rdot(&r, &r_dot);
        let v = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0];
        let out = m6.mul_vec6(v);
        assert!(real(out[0]).is_near(real(1.0), 1e-15) && real(out[1]).is_near(real(2.0), 1e-15) && real(out[2]).is_near(real(3.0), 1e-15));
        assert!(real(out[3]).is_near(real(4.0), 1e-15) && real(out[4]).is_near(real(5.0), 1e-15) && real(out[5]).is_near(real(6.0), 1e-15));
    }
}
