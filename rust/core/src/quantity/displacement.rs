//! 位移矢量（直角坐标 + 参考架）。位移用 Vector3<Length> 表示，标量 Real。

use super::duration::Duration;
use super::length::Length;
use super::reference_frame::ReferenceFrame;
use super::unit::LengthUnit;
use super::vector3::Vector3;
use super::velocity::Velocity;
use crate::math::real::Real;

/// 某参考架下的位移（架 + 位移矢量）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Displacement {
    pub frame: ReferenceFrame,
    pub vec: Vector3<Length>,
}

impl Displacement {
    pub fn from_si_meters_in_frame(frame: ReferenceFrame, x: Real, y: Real, z: Real) -> Self {
        Self {
            frame,
            vec: Vector3::from_lengths([
                Length::from_value(x, LengthUnit::Meter),
                Length::from_value(y, LengthUnit::Meter),
                Length::from_value(z, LengthUnit::Meter),
            ]),
        }
    }

    /// 从指定架下的位移矢量构造。
    pub fn from_vec_in_frame(frame: ReferenceFrame, v: Vector3<Length>) -> Self {
        Self { frame, vec: v }
    }

    #[inline]
    pub fn x(self) -> Length {
        self.vec.x()
    }
    #[inline]
    pub fn y(self) -> Length {
        self.vec.y()
    }
    #[inline]
    pub fn z(self) -> Length {
        self.vec.z()
    }

    pub fn to_meters(self) -> [Real; 3] {
        [
            self.vec.x().meters(),
            self.vec.y().meters(),
            self.vec.z().meters(),
        ]
    }

    pub fn same_frame_as(self, other: Displacement) -> bool {
        self.frame == other.frame
    }

    /// 位移的模。
    pub fn magnitude(self) -> Length {
        self.vec.magnitude()
    }

    /// 同架下两位移相加（矢量加法）。
    pub fn add_displacement(self, other: Displacement) -> Self {
        assert!(self.same_frame_as(other), "坐标系不一致");
        Self {
            frame: self.frame,
            vec: self.vec + other.vec,
        }
    }

    /// 位移 ÷ 时间 → 速度。
    pub fn div_duration(self, d: Duration) -> Velocity {
        Velocity {
            frame: self.frame,
            vx: self.vec.x().div_duration(d),
            vy: self.vec.y().div_duration(d),
            vz: self.vec.z().div_duration(d),
        }
    }

    /// 用给定变换将当前系下的 [x,y,z] 映到目标系，返回目标系下的 Displacement。
    pub fn apply_transform<F>(self, target: ReferenceFrame, f: F) -> Self
    where
        F: FnOnce([Real; 3]) -> [Real; 3],
    {
        let out = f(self.to_meters());
        Self::from_si_meters_in_frame(target, out[0], out[1], out[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};
    use crate::quantity::duration::Duration;

    #[test]
    fn displacement_from_si_and_components() {
        let d = Displacement::from_si_meters_in_frame(ReferenceFrame::FK5, real(3.0), real(4.0), real(0.0));
        assert!(d.x().meters().is_near(real(3.0), 1e-10));
        assert!(d.y().meters().is_near(real(4.0), 1e-10));
        assert!(d.z().meters().is_near(real(0.0), 1e-10));
        let m = d.to_meters();
        assert!(m[0].is_near(real(3.0), 1e-10) && m[1].is_near(real(4.0), 1e-10));
    }

    #[test]
    fn displacement_magnitude_and_same_frame() {
        let d = Displacement::from_si_meters_in_frame(ReferenceFrame::FK5, real(3.0), real(4.0), real(0.0));
        assert!(d.magnitude().meters().is_near(real(5.0), 1e-10));
        let e = Displacement::from_si_meters_in_frame(ReferenceFrame::FK5, real(0.0), real(0.0), real(0.0));
        assert!(d.same_frame_as(e));
        let other = Displacement::from_si_meters_in_frame(ReferenceFrame::ICRS, real(0.0), real(0.0), real(0.0));
        assert!(!d.same_frame_as(other));
    }

    #[test]
    fn displacement_add_div_duration_apply_transform() {
        let a = Displacement::from_si_meters_in_frame(ReferenceFrame::FK5, real(1.0), real(0.0), real(0.0));
        let b = Displacement::from_si_meters_in_frame(ReferenceFrame::FK5, real(0.0), real(1.0), real(0.0));
        let c = a.add_displacement(b);
        assert!(c.x().meters().is_near(real(1.0), 1e-10));
        assert!(c.y().meters().is_near(real(1.0), 1e-10));
        let dt = Duration::in_seconds(real(2.0));
        let v = a.div_duration(dt);
        assert!(v.vx.m_per_s().is_near(real(0.5), 1e-10));
        let flipped = a.apply_transform(ReferenceFrame::ICRS, |[x, y, z]| [x, z, -y]);
        assert!(flipped.z().meters().is_near(real(0.0), 1e-10));
    }
}
