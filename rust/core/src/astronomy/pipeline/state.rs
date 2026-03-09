//! 6D 状态向量与坐标表示。标量用 Real。

use crate::math::algebra::mat::Mat;
use crate::math::real::{Real, RealOps};
use crate::quantity::{
    length::Length,
    position::Position,
    reference_frame::ReferenceFrame,
    vector3::Vector3,
    velocity::Velocity,
};
use crate::quantity::angle::PlaneAngle;

/// 6D 状态：位置 + 速度，带参考架。
#[derive(Clone, Debug)]
pub struct State6 {
    pub position: Position,
    pub velocity: Velocity,
}

impl State6 {
    pub fn new(position: Position, velocity: Velocity) -> Self {
        assert!(
            position.frame == velocity.frame,
            "State6: position and velocity must be in same frame"
        );
        Self { position, velocity }
    }

    pub fn frame(&self) -> ReferenceFrame {
        self.position.frame
    }

    pub fn to_meters_and_m_per_s(&self) -> ([Real; 3], [Real; 3]) {
        (self.position.to_meters(), self.velocity.to_m_per_s())
    }

    /// 从直角 (米, 米/秒) 构造。
    pub fn from_si_in_frame(
        frame: ReferenceFrame,
        x: Real,
        y: Real,
        z: Real,
        vx: Real,
        vy: Real,
        vz: Real,
    ) -> Self {
        let position = Position::from_si_meters_in_frame(frame, x, y, z);
        let velocity = Velocity::from_si_m_per_s_in_frame(frame, vx, vy, vz);
        Self::new(position, velocity)
    }

    /// 应用 6×6 状态转移： [r_new; v_new] = [R R_dot; 0 R] * [r_old; v_old]，输出架为 `to_frame`。
    pub fn apply_transition(
        &self,
        r: &[[Real; 3]; 3],
        r_dot: &[[Real; 3]; 3],
        to_frame: ReferenceFrame,
    ) -> Self {
        let m6 = Mat::<Real, 6, 6>::from_block_r_rdot(r, r_dot);
        self.apply_transition_mat6(&m6, to_frame)
    }

    /// 用 6×6 矩阵一次乘 6D 向量： [r_new; v_new] = M6 * [r_old; v_old]。
    pub fn apply_transition_mat6(&self, m6: &Mat<Real, 6, 6>, to_frame: ReferenceFrame) -> Self {
        let (pos_m, vel_m) = self.to_meters_and_m_per_s();
        let v6 = [
            pos_m[0], pos_m[1], pos_m[2],
            vel_m[0], vel_m[1], vel_m[2],
        ];
        let out6 = m6.mul_vec6(v6);
        Self::from_si_in_frame(
            to_frame,
            out6[0], out6[1], out6[2],
            out6[3], out6[4], out6[5],
        )
    }

    /// 仅位置旋转（3×3），速度同旋；无 R_dot。
    pub fn apply_rotation(&self, r: &[[Real; 3]; 3], to_frame: ReferenceFrame) -> Self {
        let m = Mat::from(*r);
        let new_pos = m.mul_vec_typed(&[self.position.x, self.position.y, self.position.z]);
        let new_vel = m.mul_vec_typed(&[self.velocity.vx, self.velocity.vy, self.velocity.vz]);
        Self::new(
            Position::from_lengths_in_frame(to_frame, Vector3::from_lengths(new_pos)),
            Velocity::from_speeds_in_frame(to_frame, Vector3::from_speeds(new_vel)),
        )
    }

    /// 将当前状态的位置转为球面表示（同一架下）。
    pub fn to_spherical(&self) -> SphericalCoords {
        let [x, y, z] = self.position.to_meters();
        let r = (x * x + y * y + z * z).sqrt();
        let lon = y.atan2(x);
        let lat = if r > Real::zero() { (z / r).asin() } else { Real::zero() };
        SphericalCoords {
            lon: PlaneAngle::from_rad(lon),
            lat: PlaneAngle::from_rad(lat),
            r: Length::from_value(r, crate::quantity::unit::LengthUnit::Meter),
        }
    }
}

/// 坐标表示法：纯数学几何，与架无关（文档 2.2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateRepresentation {
    Cartesian,
    Spherical,
}

/// 球面坐标（经度、纬度、距离）：物理量类型。
#[derive(Clone, Copy, Debug)]
pub struct SphericalCoords {
    pub lon: PlaneAngle,
    pub lat: PlaneAngle,
    pub r: Length,
}
