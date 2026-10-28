//! 日、月地心视位置（当日真赤道与真春分点，含距离）：与视黄经同一条管线，终点为 `TrueEquator`，供站心视差、地平坐标、
//! 日月食几何使用。
//!
//! 架的历元取**观测时刻** t（恒星时也按 t 计算，二者须同一真赤道架）；视黄经接口沿用推迟时刻 t_r 的架，二者岁差差异
//! 对太阳约 8 分钟、月亮约 1.3 秒的岁差量，可忽略。

use crate::astronomy::ephemeris::{Elpmpp02Data, Vsop87};
use crate::astronomy::pipeline::{
    Body, EphemerisProvider, Fk5ToIcrsBias, LightTimeCorrector, TransformGraph,
};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::quantity::{epoch::Epoch, position::Position, reference_frame::ReferenceFrame};

use super::longitude::{default_graph_with_icrs_mapper, ApparentPipelineOptions};

/// 太阳地心视位置（当日真赤道架，含距离）。管线同 [`super::sun_apparent_ecliptic_longitude_with_options`]。
pub fn sun_apparent_equatorial(vsop: &Vsop87, t: TimePoint, options: &ApparentPipelineOptions) -> Position {
    let t_tt = t.to_scale(TimeScale::TT);
    let corrector: LightTimeCorrector<'_, Vsop87, Fk5ToIcrsBias> =
        LightTimeCorrector { ephemeris: vsop, mapper: None, max_iter: 2 };
    let (tr, state) = corrector.retarded_state(t_tt, Body::Sun);
    let jd_tr = tr.to_scale(TimeScale::TT).jd;
    let graph = default_graph_with_icrs_mapper().with_precession_model(options.effective_precession_model());
    let state = graph.transform_to(state, ReferenceFrame::ICRS, jd_tr);
    let jd = t_tt.jd;
    graph.transform_to(state, ReferenceFrame::TrueEquator(Epoch::new(jd.clone())), jd).position
}

/// 月球地心视位置（当日真赤道架，含距离）。光行时按 `options.use_light_time_moon`；日月食请用
/// [`ApparentPipelineOptions::pipeline_default`]（开光行时）。
pub fn moon_apparent_equatorial(elp: &Elpmpp02Data, t: TimePoint, options: &ApparentPipelineOptions) -> Position {
    let t_tt = t.to_scale(TimeScale::TT);
    let state = if options.use_light_time_moon {
        let corrector: LightTimeCorrector<'_, Elpmpp02Data, Fk5ToIcrsBias> =
            LightTimeCorrector { ephemeris: elp, mapper: None, max_iter: 2 };
        corrector.retarded_state(t_tt, Body::Moon).1
    } else {
        elp.compute_state(Body::Moon, t_tt)
    };
    let graph = TransformGraph::default_graph().with_precession_model(options.effective_precession_model());
    let jd = t_tt.jd;
    graph.transform_to(state, ReferenceFrame::TrueEquator(Epoch::new(jd.clone())), jd).position
}
