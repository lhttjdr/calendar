//! 6×6 状态转移矩阵。文档 4.1：[r_new; v_new] = [R R_dot; 0 R] * [r_old; v_old]。矩阵与向量统一 Real。

use crate::math::real::{zero, Real};
use crate::quantity::reference_frame::ReferenceFrame;

/// 6×6 状态转移：R 为 3×3 旋转，R_dot 为旋转对时间的导数（科里奥利）。
#[derive(Clone, Debug)]
pub struct StateTransition6 {
    pub from_frame: ReferenceFrame,
    pub to_frame: ReferenceFrame,
    pub r: [[Real; 3]; 3],
    pub r_dot: [[Real; 3]; 3],
}

impl StateTransition6 {
    /// 仅旋转、无导数时 R_dot 为零矩阵。
    pub fn from_rotation(from: ReferenceFrame, to: ReferenceFrame, r: [[Real; 3]; 3]) -> Self {
        Self {
            from_frame: from,
            to_frame: to,
            r,
            r_dot: [
                [zero(), zero(), zero()],
                [zero(), zero(), zero()],
                [zero(), zero(), zero()],
            ],
        }
    }
}

/// 构建从 FK5 到 MeanEquator(epoch) 的岁差转移（仅 R，R_dot 可选后续）。
pub fn precession_transition(
    _t_cent: Real,
    from: ReferenceFrame,
    to: ReferenceFrame,
    matrix_3x3: [[Real; 3]; 3],
) -> StateTransition6 {
    StateTransition6::from_rotation(from, to, matrix_3x3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};
    use crate::quantity::epoch::Epoch;

    #[test]
    fn state_transition6_from_rotation() {
        let z = real(0.0);
        let o = real(1.0);
        let r = [[o, z, z], [z, o, z], [z, z, o]];
        let from = ReferenceFrame::FK5;
        let to = ReferenceFrame::MeanEquator(Epoch::j2000());
        let tr = StateTransition6::from_rotation(from, to, r);
        assert_eq!(tr.from_frame, from);
        assert_eq!(tr.to_frame, to);
        assert!(tr.r_dot[0][0].is_near(z, 1e-20));
    }

    #[test]
    fn precession_transition_smoke() {
        let z = real(0.0);
        let o = real(1.0);
        let r = [[o, z, z], [z, o, z], [z, z, o]];
        let tr = precession_transition(z, ReferenceFrame::FK5, ReferenceFrame::MeanEquator(Epoch::j2000()), r);
        assert_eq!(tr.from_frame, ReferenceFrame::FK5);
    }
}
