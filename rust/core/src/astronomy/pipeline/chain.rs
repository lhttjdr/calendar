//! Fluent 管线 API（文档 3.2）。泛型于 R: Real，不在本层指定 f64；精度由顶层 calendar 选项决定。

use crate::astronomy::time::TimePoint;
use crate::math::real::Real;
use super::ephemeris_provider::{Body, EphemerisProvider};
use super::frame_mapper::FrameMapper;
use super::light_time::LightTimeCorrector;
use super::optical::OpticalCorrector;
use super::state::{CoordinateRepresentation, SphericalCoords, State6};
use super::transform_graph::TransformGraph;

/// Fluent 管线：按文档 3.2 拼接 1→2→3→4→5→6。
pub struct Pipeline<'a, P, M> {
    pub ephemeris: &'a P,
    pub mapper: Option<&'a M>,
    pub graph: &'a TransformGraph,
    pub light_time_iter: usize,
}

impl<'a, P, M> Pipeline<'a, P, M> {
    /// 1. 基准获取 (Frame: 历表输出架，如 MeanEcliptic(J2000))。
    pub fn compute_state(&self, body: Body, epoch: TimePoint) -> State6
    where
        P: EphemerisProvider,
        M: FrameMapper,
    {
        self.ephemeris.compute_state(body, epoch)
    }

    /// 2. 跨架跃迁 (如 FK5 -> ICRS + DE406 fit)
    pub fn apply_mapping(&self, state: State6, epoch: TimePoint) -> State6
    where
        P: EphemerisProvider,
        M: FrameMapper,
    {
        match self.mapper {
            Some(m) => m.apply(state, epoch),
            None => state,
        }
    }

    /// 3. 光行时回溯：返回 (推迟时 tr, 在 tr 时刻的状态)
    pub fn apply_light_time(&self, t: TimePoint, body: Body) -> (TimePoint, State6)
    where
        P: EphemerisProvider,
        M: FrameMapper,
    {
        let corrector = LightTimeCorrector {
            ephemeris: self.ephemeris,
            mapper: self.mapper,
            max_iter: self.light_time_iter,
        };
        corrector.retarded_state(t, body)
    }

    /// 4. 物理空间旋转到目标架（图语义：起止点最短路执行，见 TransformGraph）
    pub fn transform_to(&self, state: State6, target: crate::quantity::reference_frame::ReferenceFrame, jd_tt: Real) -> State6 {
        self.graph.transform_to(state, target, jd_tt)
    }

    /// 5. 光行差/折射（占位：同一架返回）
    pub fn apply_optical_effect<O: OpticalCorrector>(&self, state: State6, _optical: &O) -> State6 {
        _optical.apply(state)
    }

    /// 6. 数学降维：输出球面供展示
    pub fn into_representation(&self, state: State6, _repr: CoordinateRepresentation) -> SphericalCoords {
        state.to_spherical()
    }
}
