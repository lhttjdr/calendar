//! 非线性映射器：跨架拟合修正（文档 3.1 FrameMapper）。泛型于 R: Real，内部用 f64 计算再转回 R。
//!
//! **时间尺度**：FK5↔ICRS 为架旋转无时间；DE406 patch 公式规定入参 **TT**，调用方应传 TT 的 `epoch`。

use crate::astronomy::frame::fk5_icrs;
use crate::astronomy::time::TimePoint;
use crate::astronomy::frame::vsop87_de406_icrs_patch;
use crate::quantity::{position::Position, reference_frame::ReferenceFrame, velocity::Velocity};
use super::state::State6;

/// 将状态从一个架映射到另一个架；R 由顶层选择，本层不指定 f64。
pub trait FrameMapper {
    fn apply(&self, state: State6, epoch: TimePoint) -> State6;
}

/// VSOP87 赤道 → ICRS + DE406 经验 patch；内部用 f64，边界转换。
pub struct VsopToDe406IcrsFit;

impl FrameMapper for VsopToDe406IcrsFit {
    /// 一步调用内做两子步：① Frame bias B⁻¹（FK5→GCRS/ICRS，标准文献 B 为 ICRS→FK5）② DE406 赤道经验 patch；图中边标签即为此二合一。
    fn apply(&self, state: State6, epoch: TimePoint) -> State6 {
        let (pos_m, vel_m) = state.to_meters_and_m_per_s();
        let (x_icrs, y_icrs, z_icrs) = fk5_icrs::rotate_equatorial(
            pos_m[0],
            pos_m[1],
            pos_m[2],
        );
        let pos_equ = Position::from_si_meters_in_frame(
            ReferenceFrame::ICRS,
            x_icrs, y_icrs, z_icrs,
        );
        let patched = vsop87_de406_icrs_patch::apply_patch_to_equatorial_for_geocentric_sun(
            pos_equ,
            &epoch,
        );
        let (vx, vy, vz) = fk5_icrs::rotate_equatorial(
            vel_m[0],
            vel_m[1],
            vel_m[2],
        );
        let position = Position::from_si_meters_in_frame(
            ReferenceFrame::ICRS,
            patched.x.meters(),
            patched.y.meters(),
            patched.z.meters(),
        );
        let velocity = Velocity::from_si_m_per_s_in_frame(
            ReferenceFrame::ICRS,
            vx, vy, vz,
        );
        State6::new(position, velocity)
    }
}
