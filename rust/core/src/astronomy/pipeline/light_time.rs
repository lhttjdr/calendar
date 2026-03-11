//! 光行时修正器：t → t−τ（文档 3.1 LightTimeCorrector）。泛型于 R: Real。
//!
//! 时间尺度：迭代在 TT 下进行；调用历表时按历表规定转为 TDB 或 TT（[`EphemerisProvider::evaluation_time_scale`]），不沿用上一步尺度。

use crate::astronomy::correction::light_time::retarded_time_point;
use crate::astronomy::time::{TimePoint, TimeScale};
use super::ephemeris_provider::EphemerisProvider;
use super::frame_mapper::FrameMapper;
use super::state::State6;

/// 光行时修正：持有 EphemerisProvider 与可选 FrameMapper，迭代得到推迟时 tr 及该时刻的状态。
pub struct LightTimeCorrector<'a, P, M> {
    pub ephemeris: &'a P,
    pub mapper: Option<&'a M>,
    pub max_iter: usize,
}

impl<'a, P, M> LightTimeCorrector<'a, P, M> {
    /// 观测时刻 t，目标 body；返回 (推迟时 tr 在 **TT**，在 tr 时刻的 6D 状态)。
    /// 历表求值按 [`EphemerisProvider::evaluation_time_scale`] 转为 TDB 或 TT 再调用。
    pub fn retarded_state(&self, t: TimePoint, body: super::ephemeris_provider::Body) -> (TimePoint, State6)
    where
        P: EphemerisProvider,
        M: FrameMapper,
    {
        let t_tt = t.to_scale(TimeScale::TT);
        let scale = self.ephemeris.evaluation_time_scale();
        let tr = retarded_time_point(
            t_tt,
            |tr| {
                let t_eval = tr.to_scale(scale);
                self.ephemeris.compute_state(body, t_eval).position.norm()
            },
            self.max_iter,
        );
        let t_eval = tr.to_scale(scale);
        let state = self.ephemeris.compute_state(body, t_eval);
        let state = if let Some(m) = self.mapper {
            m.apply(state, tr)
        } else {
            state
        };
        (tr, state)
    }
}
