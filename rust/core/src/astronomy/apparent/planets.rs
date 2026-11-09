//! 行星地心视位置：与太阳同一条管线（光行时 → 架变换 → 岁差章动），历表可为 VSOP87（[`Vsop87Planets`]）或 DE406。
//!
//! VSOP87 的地心行星矢量 = 日心行星 − 日心地球，在 J2000 动力学黄道架；转 ICRS 只施 FK5→ICRS 架偏差
//! （[`Fk5ToIcrsBias`]）。VSOP87 拟合到 DE406/DE441 的改正挂在各 `Vsop87` 对象上（加载时选定）；与 JPL 历表的差异见本模块测试。
//! DE406/DE441 状态本已在 ICRS，图中不经映射器。
//!
//! 太阳引力光线偏折（[`LightDeflection::Sun`]，缺省开）在 ICRS 中、岁差章动之前施加：p 为推迟时 tr 的地心天体方向，
//! q 为 tr 时的日心天体方向（地心天体 − 地心太阳，同在 tr），e 为观测时刻 t 的日心地球方向。p 取的是折叠法的
//! 「行星与地球同在 tr」矢量，与严格的「行星在 tr、地球在 t」相差周年光行差（约 20″），对偏折的相对影响约 10⁻⁴，可略。
//!
//! [`Vsop87Planets`]: crate::astronomy::pipeline::Vsop87Planets

use crate::astronomy::pipeline::{Body, EphemerisProvider, FrameMapper, Fk5ToIcrsBias, LightTimeCorrector, Planet, TransformGraph};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{Real, RealOps};
use crate::quantity::{angle::PlaneAngle, epoch::Epoch, length::Length, position::Position, reference_frame::ReferenceFrame};

use super::longitude::{ApparentPipelineOptions, LightDeflection};
use crate::astronomy::constant::AU_METERS;
use crate::astronomy::correction::light_deflection::solar_deflection_of_position;
use crate::astronomy::pipeline::State6;

fn planet_graph(options: &ApparentPipelineOptions) -> TransformGraph {
    TransformGraph::default_graph()
        .with_fk5_to_icrs_mapper(|s, jd_tt| Fk5ToIcrsBias.apply(s, TimePoint::new(TimeScale::TT, jd_tt)))
        .with_precession_model(options.effective_precession_model())
}

/// 光行时修正后的地心天体状态，转到 `target` 架（架历元取观测时刻 t）。
fn body_apparent_in<P: EphemerisProvider>(
    eph: &P,
    body: Body,
    t: TimePoint,
    options: &ApparentPipelineOptions,
    target: fn(Epoch) -> ReferenceFrame,
) -> Position {
    let t_tt = t.to_scale(TimeScale::TT);
    let corrector: LightTimeCorrector<'_, P, Fk5ToIcrsBias> = LightTimeCorrector { ephemeris: eph, mapper: None, max_iter: 3 };
    let (tr, state) = corrector.retarded_state(t_tt, body);
    let graph = planet_graph(options);
    let state = graph.transform_to(state, ReferenceFrame::ICRS, tr.to_scale(TimeScale::TT).jd);
    let state = match (options.light_deflection, body) {
        (LightDeflection::Sun, Body::Planet(_)) => deflect_by_sun(eph, &graph, state, t_tt, tr),
        // 太阳本身不偏折；月球近地，太阳偏折 < 10 μas，且月球提供者未必带太阳
        _ => state,
    };
    let jd = t_tt.jd;
    graph.transform_to(state, target(Epoch::new(jd)), jd).position
}

/// 对 ICRS 中的地心天体状态（推迟时 tr）施太阳引力光线偏折；速度不改（偏折的时间变化率可略）。
fn deflect_by_sun<P: EphemerisProvider>(eph: &P, graph: &TransformGraph, state: State6, t_tt: TimePoint, tr: TimePoint) -> State6 {
    let scale = eph.evaluation_time_scale();
    let sun_icrs = |tp: TimePoint| {
        let s = eph.compute_state(Body::Sun, tp.to_scale(scale));
        graph.transform_to(s, ReferenceFrame::ICRS, tp.to_scale(TimeScale::TT).jd).position.to_meters()
    };
    let sun_tr = sun_icrs(tr);
    let sun_t = sun_icrs(t_tt);
    let p = solar_deflection_of_position(state.position.to_meters(), sun_tr, sun_t, AU_METERS);
    State6::new(Position::from_si_meters_in_frame(ReferenceFrame::ICRS, p[0], p[1], p[2]), state.velocity)
}

/// 行星地心视位置（当日真赤道与真春分点，含距离）。
pub fn planet_apparent_equatorial<P: EphemerisProvider>(
    eph: &P,
    planet: Planet,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Position {
    body_apparent_in(eph, Body::Planet(planet), t, options, ReferenceFrame::TrueEquator)
}

/// 行星地心视黄经、视黄纬（当日真黄道与真春分点）与地心距离。
pub fn planet_apparent_ecliptic<P: EphemerisProvider>(
    eph: &P,
    planet: Planet,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> (PlaneAngle, PlaneAngle, Length) {
    body_apparent_ecliptic(eph, Body::Planet(planet), t, options)
}

/// 任意天体（太阳、行星；提供者须支持该天体）的地心视黄经、视黄纬（当日真黄道与真春分点）与地心距离。与行星同一管线，
/// 供合冲、大距等需日与行星同法比较的场合。
pub fn body_apparent_ecliptic<P: EphemerisProvider>(
    eph: &P,
    body: Body,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> (PlaneAngle, PlaneAngle, Length) {
    let p = body_apparent_in(eph, body, t, options, ReferenceFrame::ApparentEcliptic);
    let [x, y, z]: [Real; 3] = p.to_meters();
    let lon = PlaneAngle::from_rad(y.atan2(x)).wrap_to_2pi();
    let lat = PlaneAngle::from_rad(z.atan2((x * x + y * y).sqrt()));
    (lon, lat, p.norm())
}

/// 任意天体的地心视位置（当日真赤道与真春分点，含距离）。
pub fn body_apparent_equatorial<P: EphemerisProvider>(
    eph: &P,
    body: Body,
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Position {
    body_apparent_in(eph, body, t, options, ReferenceFrame::TrueEquator)
}
