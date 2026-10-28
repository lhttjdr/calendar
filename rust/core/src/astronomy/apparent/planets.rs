//! 行星地心视位置：与太阳同一条管线（光行时 → 架变换 → 岁差章动），历表可为 VSOP87（[`Vsop87Planets`]）或 DE406。
//!
//! VSOP87 的地心行星矢量 = 日心行星 − 日心地球，在 J2000 动力学黄道架；转 ICRS 只施 FK5→ICRS 架偏差
//! （[`Fk5ToIcrsBias`]）。VSOP87 拟合到 DE406/DE441 的改正挂在各 `Vsop87` 对象上（加载时选定）；与 JPL 历表的差异见本模块测试。
//! DE406/DE441 状态本已在 ICRS，图中不经映射器。
//!
//! [`Vsop87Planets`]: crate::astronomy::pipeline::Vsop87Planets

use crate::astronomy::pipeline::{Body, EphemerisProvider, FrameMapper, Fk5ToIcrsBias, LightTimeCorrector, Planet, TransformGraph};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{Real, RealOps};
use crate::quantity::{angle::PlaneAngle, epoch::Epoch, length::Length, position::Position, reference_frame::ReferenceFrame};

use super::longitude::ApparentPipelineOptions;

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
    let jd = t_tt.jd;
    graph.transform_to(state, target(Epoch::new(jd)), jd).position
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
