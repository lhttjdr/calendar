//! 位置（直角坐标 + 参考架）。分量用 Length&lt;R&gt;，R: Real，不写死 f64。

use super::length::Length;
use super::reference_frame::ReferenceFrame;
use super::spherical;
use super::unit::LengthUnit;
use super::vector3::Vector3;
use crate::math::real::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub frame: ReferenceFrame,
    pub x: Length,
    pub y: Length,
    pub z: Length,
}

impl Position {
    pub fn from_si_meters_in_frame(frame: ReferenceFrame, x: Real, y: Real, z: Real) -> Self {
        Self {
            frame,
            x: Length::from_value(x, LengthUnit::Meter),
            y: Length::from_value(y, LengthUnit::Meter),
            z: Length::from_value(z, LengthUnit::Meter),
        }
    }

    pub fn from_si_meters(x: Real, y: Real, z: Real) -> Self {
        Self::from_si_meters_in_frame(ReferenceFrame::FK5, x, y, z)
    }

    /// 从给定架下的直角坐标矢量构造位置。
    pub fn from_lengths_in_frame(frame: ReferenceFrame, vec: Vector3<Length>) -> Self {
        Self {
            frame,
            x: vec.x(),
            y: vec.y(),
            z: vec.z(),
        }
    }

    /// 从球面坐标（经度、纬度、距离）在给定架下构造位置。
    pub fn from_spherical_in_frame(
        frame: ReferenceFrame,
        lon: super::angle::PlaneAngle,
        lat: super::angle::PlaneAngle,
        r: Length,
    ) -> Self {
        let vec = spherical::spherical_to_cartesian(lon, lat, r);
        Self {
            frame,
            x: vec.x(),
            y: vec.y(),
            z: vec.z(),
        }
    }

    pub fn to_meters(self) -> [Real; 3] {
        [self.x.meters(), self.y.meters(), self.z.meters()]
    }

    pub fn same_frame_as(self, other: Position) -> bool {
        self.frame == other.frame
    }

    pub fn distance_to(self, other: Position) -> Length {
        assert!(self.same_frame_as(other), "坐标系不一致");
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        let d2 = dx.meters() * dx.meters() + dy.meters() * dy.meters() + dz.meters() * dz.meters();
        Length::from_value(d2.sqrt(), LengthUnit::Meter)
    }

    pub fn norm(self) -> Length {
        let d2 = self.x.meters() * self.x.meters()
            + self.y.meters() * self.y.meters()
            + self.z.meters() * self.z.meters();
        Length::from_value(d2.sqrt(), LengthUnit::Meter)
    }

    /// 用给定变换将当前系下的 [x,y,z] 映到目标系，返回目标系下的 Position。
    pub fn apply_transform<F>(self, target: ReferenceFrame, f: F) -> Self
    where
        F: FnOnce([Real; 3]) -> [Real; 3],
    {
        let out = f(self.to_meters());
        Self::from_si_meters_in_frame(target, out[0], out[1], out[2])
    }
}
