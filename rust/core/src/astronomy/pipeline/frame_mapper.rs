//! 非线性映射器：跨架拟合修正（文档 3.1 FrameMapper）。泛型于 R: Real，内部用 f64 计算再转回 R。
//!
//! **时间尺度**：FK5↔ICRS 为架旋转无时间。VSOP87 拟合到 DE406/DE441 的改正不在映射器中，而挂在 `Vsop87::patch` 上。

use crate::astronomy::frame::fk5_icrs;
use crate::astronomy::time::TimePoint;
use crate::quantity::{position::Position, reference_frame::ReferenceFrame, velocity::Velocity};
use super::state::State6;

/// 将状态从一个架映射到另一个架；R 由顶层选择，本层不指定 f64。
pub trait FrameMapper {
    fn apply(&self, state: State6, epoch: TimePoint) -> State6;
}

/// FK5 → ICRS：Frame bias（SOFA 偏差矩阵）。
pub struct Fk5ToIcrsBias;

impl FrameMapper for Fk5ToIcrsBias {
    fn apply(&self, state: State6, _epoch: TimePoint) -> State6 {
        let (pos_m, vel_m) = state.to_meters_and_m_per_s();
        let (x_icrs, y_icrs, z_icrs) = fk5_icrs::rotate_equatorial(
            pos_m[0],
            pos_m[1],
            pos_m[2],
        );
        let (vx, vy, vz) = fk5_icrs::rotate_equatorial(
            vel_m[0],
            vel_m[1],
            vel_m[2],
        );
        let position = Position::from_si_meters_in_frame(
            ReferenceFrame::ICRS,
            x_icrs, y_icrs, z_icrs,
        );
        let velocity = Velocity::from_si_m_per_s_in_frame(
            ReferenceFrame::ICRS,
            vx, vy, vz,
        );
        State6::new(position, velocity)
    }
}

/// 顺序组合两个 FrameMapper：先 first 再 second。
pub struct Compose<A, B>(pub A, pub B);

impl<A: FrameMapper, B: FrameMapper> FrameMapper for Compose<A, B> {
    fn apply(&self, state: State6, epoch: TimePoint) -> State6 {
        self.1.apply(self.0.apply(state, epoch), epoch)
    }
}
