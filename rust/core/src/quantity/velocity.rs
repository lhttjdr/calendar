//! 速度向量（直角坐标 + 参考架）。分量用 Speed&lt;R&gt;，R: Real，不写死 f64。

use super::reference_frame::ReferenceFrame;
use super::speed::Speed;
use super::unit::SpeedUnit;
use super::vector3::Vector3;
use crate::math::real::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity {
    pub frame: ReferenceFrame,
    pub vx: Speed,
    pub vy: Speed,
    pub vz: Speed,
}

impl Velocity {
    pub fn from_si_m_per_s_in_frame(frame: ReferenceFrame, vx: Real, vy: Real, vz: Real) -> Self {
        Self {
            frame,
            vx: Speed::from_value(vx, SpeedUnit::MPerS),
            vy: Speed::from_value(vy, SpeedUnit::MPerS),
            vz: Speed::from_value(vz, SpeedUnit::MPerS),
        }
    }

    pub fn from_si_m_per_s(vx: Real, vy: Real, vz: Real) -> Self {
        Self::from_si_m_per_s_in_frame(ReferenceFrame::FK5, vx, vy, vz)
    }

    /// 从给定架下的速度矢量构造。
    pub fn from_speeds_in_frame(frame: ReferenceFrame, vec: Vector3<Speed>) -> Self {
        Self {
            frame,
            vx: vec.x(),
            vy: vec.y(),
            vz: vec.z(),
        }
    }

    pub fn to_m_per_s(self) -> [Real; 3] {
        [self.vx.m_per_s(), self.vy.m_per_s(), self.vz.m_per_s()]
    }

    pub fn same_frame_as(self, other: Velocity) -> bool {
        self.frame == other.frame
    }

    /// 用给定变换将当前系下的速度分量映到目标系。
    pub fn apply_transform<F>(self, target: ReferenceFrame, f: F) -> Self
    where
        F: FnOnce([Real; 3]) -> [Real; 3],
    {
        let out = f(self.to_m_per_s());
        Self::from_si_m_per_s_in_frame(target, out[0], out[1], out[2])
    }
}
