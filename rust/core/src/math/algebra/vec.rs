//! 泛型向量 Vec&lt;T, N&gt;：N 维，T: Copy；T: Scalar 时有 dot/scale/cross/norm。物理层在 quantity 为 Vec&lt;T, 3&gt; 实现 Add/Sub/ScaledBy&lt;Real&gt;，数学层不依赖 Real/LinearComponent。

use super::mat::{Scalar, ScalarNorm};

/// N 维向量；T: Copy 为最小约束，T: Scalar 时具 dot/cross/norm。
#[derive(Clone, Copy, Debug)]
pub struct Vec<T, const N: usize>
where
    T: Copy,
{
    pub data: [T; N],
}

impl<T: Copy + PartialEq, const N: usize> PartialEq for Vec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T, const N: usize> Vec<T, N>
where
    T: Copy,
{
    #[inline]
    pub fn new(data: [T; N]) -> Self {
        Vec { data }
    }

    #[inline]
    pub fn from_array(a: [T; N]) -> Self {
        Vec { data: a }
    }

    #[inline]
    pub fn to_array(self) -> [T; N] {
        self.data
    }
}

impl<T, const N: usize> Vec<T, N>
where
    T: Scalar,
{
    #[inline]
    pub fn dot(self, other: Vec<T, N>) -> T
    where
        T: Default,
    {
        let mut s = T::default();
        for i in 0..N {
            s = s + self.data[i] * other.data[i];
        }
        s
    }

    #[inline]
    pub fn scale(self, s: T) -> Vec<T, N> {
        let mut out = self.data;
        for i in 0..N {
            out[i] = out[i] * s;
        }
        Vec { data: out }
    }

    #[inline]
    pub fn neg(self) -> Vec<T, N>
    where
        T: std::ops::Neg<Output = T>,
    {
        let mut out = self.data;
        for i in 0..N {
            out[i] = -out[i];
        }
        Vec { data: out }
    }
}

/// 三维向量类型别名。
pub type Vec3<T> = Vec<T, 3>;

impl<T: Copy> Vec<T, 3> {
    #[inline]
    pub fn new_3(x: T, y: T, z: T) -> Self {
        Vec {
            data: [x, y, z],
        }
    }

    #[inline]
    pub fn x(self) -> T {
        self.data[0]
    }
    #[inline]
    pub fn y(self) -> T {
        self.data[1]
    }
    #[inline]
    pub fn z(self) -> T {
        self.data[2]
    }
}

impl<T: Scalar> Vec<T, 3> {
    /// 叉积（仅 3 维，T: Scalar）。
    #[inline]
    pub fn cross(self, other: Vec<T, 3>) -> Vec<T, 3> {
        Vec {
            data: [
                self.data[1] * other.data[2] - self.data[2] * other.data[1],
                self.data[2] * other.data[0] - self.data[0] * other.data[2],
                self.data[0] * other.data[1] - self.data[1] * other.data[0],
            ],
        }
    }
}

impl<T: ScalarNorm> Vec<T, 3> {
    #[inline]
    pub fn norm(self) -> T {
        self.dot(self).sqrt()
    }

    /// 归一化；零向量返回 (1,0,0) 避免除零。
    #[inline]
    pub fn normalize(self) -> Vec<T, 3> {
        let n = self.norm();
        if n == T::zero() {
            Vec {
                data: [T::one(), T::zero(), T::zero()],
            }
        } else {
            self.scale(T::one() / n)
        }
    }
}

impl<T: Scalar + Default> Default for Vec<T, 3> {
    fn default() -> Self {
        Vec {
            data: [T::default(); 3],
        }
    }
}

impl<T: Scalar> From<[T; 3]> for Vec<T, 3> {
    fn from(a: [T; 3]) -> Self {
        Self::from_array(a)
    }
}

impl<T: Scalar> From<Vec<T, 3>> for [T; 3] {
    fn from(v: Vec<T, 3>) -> Self {
        v.to_array()
    }
}

/// 兼容旧接口：对 Vec&lt;T, 3&gt; 的自由函数（委托给方法）。
#[inline]
pub fn dot<T: Scalar + Default>(u: Vec<T, 3>, v: Vec<T, 3>) -> T {
    u.dot(v)
}

#[inline]
pub fn cross<T: Scalar>(u: Vec<T, 3>, v: Vec<T, 3>) -> Vec<T, 3> {
    u.cross(v)
}

#[inline]
pub fn norm<T: ScalarNorm>(v: Vec<T, 3>) -> T {
    v.norm()
}

#[inline]
pub fn normalize<T: ScalarNorm>(v: Vec<T, 3>) -> Vec<T, 3> {
    v.normalize()
}

#[inline]
pub fn scale<T: Scalar>(v: Vec<T, 3>, s: T) -> Vec<T, 3> {
    v.scale(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn vec3_dot_and_norm() {
        let u = Vec::new_3(1.0, 0.0, 0.0);
        let v = Vec::new_3(0.0, 1.0, 0.0);
        assert!(real(u.dot(v)).is_near(real(0), 1e-15));
        assert!(real(u.norm()).is_near(real(1.0), 1e-15));
    }

    #[test]
    fn vec3_cross() {
        let u = Vec::new_3(1.0, 0.0, 0.0);
        let v = Vec::new_3(0.0, 1.0, 0.0);
        let w = u.cross(v);
        assert!(real(w.x()).is_near(real(0), 1e-15) && real(w.y()).is_near(real(0), 1e-15) && real(w.z()).is_near(real(1.0), 1e-15));
    }
}
