//! **矢量（Vector）**：某参考架**正交归一化基**下的三分量；三分量**同量纲**，复用 [math::Vec](crate::math::algebra::vec::Vec)&lt;T, 3&gt;。
//! 物理层在本模块为 Vec&lt;T, 3&gt; 实现 Add/Sub/ScaledBy&lt;Real&gt;，数学层不依赖 Real/LinearComponent。
//!
//! 与「三分量元组」区分：坐标基下量纲可不同的分量见 [coord_components](super::coord_components)。

use std::ops::{Add, Sub};

use crate::math::algebra::mat::{LinearComponent, ScaledBy};
use crate::math::algebra::vec::Vec;
use crate::math::real::Real;

/// 矢量：正交归一化基下三分量，复用 math::Vec&lt;T, 3&gt;；T: LinearComponent&lt;Real&gt;。
pub type Vector3<T> = Vec<T, 3>;

impl<T> Add for Vec<T, 3>
where
    T: LinearComponent<Real>,
{
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Vec {
            data: [
                self.data[0] + other.data[0],
                self.data[1] + other.data[1],
                self.data[2] + other.data[2],
            ],
        }
    }
}

impl<T> Sub for Vec<T, 3>
where
    T: LinearComponent<Real>,
{
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Vec {
            data: [
                self.data[0] - other.data[0],
                self.data[1] - other.data[1],
                self.data[2] - other.data[2],
            ],
        }
    }
}

impl<T> ScaledBy<Real> for Vec<T, 3>
where
    T: LinearComponent<Real>,
{
    #[inline]
    fn zero() -> Self {
        let z = T::zero();
        Vec {
            data: [z, z, z],
        }
    }
    #[inline]
    fn scaled_by(self, s: Real) -> Self {
        Vec {
            data: [
                self.data[0].scaled_by(s),
                self.data[1].scaled_by(s),
                self.data[2].scaled_by(s),
            ],
        }
    }
}

impl Vec<super::length::Length, 3> {
    pub fn from_lengths([x, y, z]: [super::length::Length; 3]) -> Self {
        Vec::from_array([x, y, z])
    }
    pub fn to_lengths(self) -> [super::length::Length; 3] {
        self.data
    }
    /// 位移矢量的模长。
    pub fn magnitude(self) -> super::length::Length {
        let d2 = self.data[0].meters() * self.data[0].meters()
            + self.data[1].meters() * self.data[1].meters()
            + self.data[2].meters() * self.data[2].meters();
        super::length::Length::from_value(d2.sqrt(), super::unit::LengthUnit::Meter)
    }
}

impl Vec<super::speed::Speed, 3> {
    pub fn from_speeds([vx, vy, vz]: [super::speed::Speed; 3]) -> Self {
        Vec::from_array([vx, vy, vz])
    }
    pub fn to_speeds(self) -> [super::speed::Speed; 3] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};
    use crate::quantity::length::Length;
    use crate::quantity::unit::{LengthUnit, SpeedUnit};

    #[test]
    fn vector3_length_from_lengths_magnitude() {
        let x = Length::from_value(real(3.0), LengthUnit::Meter);
        let y = Length::from_value(real(4.0), LengthUnit::Meter);
        let z = Length::from_value(real(0.0), LengthUnit::Meter);
        let v = Vector3::from_lengths([x, y, z]);
        let mag = v.magnitude();
        assert!(mag.meters().is_near(real(5.0), 1e-10));
        let back = v.to_lengths();
        assert!(back[0].meters().is_near(real(3.0), 1e-10));
    }

    #[test]
    fn vector3_speed_from_speeds_to_speeds() {
        let vx = crate::quantity::speed::Speed::from_value(real(1.0), SpeedUnit::MPerS);
        let vy = crate::quantity::speed::Speed::from_value(real(0.0), SpeedUnit::MPerS);
        let vz = crate::quantity::speed::Speed::from_value(real(0.0), SpeedUnit::MPerS);
        let v = Vector3::from_speeds([vx, vy, vz]);
        let [a, b, c] = v.to_speeds();
        assert!(a.m_per_s().is_near(real(1.0), 1e-10));
        assert!(b.m_per_s().is_near(real(0.0), 1e-10));
        assert!(c.m_per_s().is_near(real(0.0), 1e-10));
    }

    #[test]
    fn vector3_add_sub_scaled_by_zero() {
        use crate::math::algebra::mat::ScaledBy;
        let l1 = Length::from_value(real(1.0), LengthUnit::Meter);
        let l0 = Length::from_value(real(0.0), LengthUnit::Meter);
        let u = Vector3::from_lengths([l1, l0, l0]);
        let v = Vector3::from_lengths([l0, l1, l0]);
        let w = u + v;
        assert!(w.x().meters().is_near(real(1.0), 1e-10));
        assert!(w.y().meters().is_near(real(1.0), 1e-10));
        let d = u - v;
        assert!(d.x().meters().is_near(real(1.0), 1e-10));
        assert!(d.y().meters().is_near(real(-1.0), 1e-10));
        let z = Vector3::<Length>::zero();
        assert!(z.x().meters().is_near(real(0.0), 1e-10));
        let s = u.scaled_by(real(2.0));
        assert!(s.x().meters().is_near(real(2.0), 1e-10));
    }
}
