//! 视赤道位置、站心视位置与地平坐标的测试。

use super::*;
use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::frame::nutation::{nutation_for_apparent, try_init_full_nutation_from_repo};
use crate::astronomy::frame::precession::mean_obliquity;
use crate::astronomy::correction::{refraction_from_geometric, Atmosphere, RefractionModel};
use crate::astronomy::geodesy::Observer;
use crate::astronomy::pipeline::{node_origin, OriginRole, State6, TransformGraph, TransitionForm, TransitionKind};
use crate::astronomy::time::delta_t::{DeltaTModel, MorrisonStephenson2021};
use crate::astronomy::time::earth_rotation::gast;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::epoch::Epoch;
use crate::quantity::length::Length;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;
use crate::quantity::unit::LengthUnit;

fn deg(d: f64) -> PlaneAngle {
    PlaneAngle::from_rad(real(d.to_radians()))
}

fn to_deg(a: PlaneAngle) -> f64 {
    a.rad().as_f64().to_degrees()
}

/// Meeus《天文算法》例 13.b：1987-04-10 19:21 UT 华盛顿（西经 77°03′56″，北纬 38°55′17″）观测金星，
/// α = 23h09m16.641s，δ = −6°43′11.61″，该时刻视恒星时 θ₀ = 8h34m56.853s；结果 H = 64.352133°，
/// A = 68.0337°（Meeus 自南向西量），h = 15.1249°。本模块方位角自北向东量，故期望 248.0337°。
/// θ₀ 只给到 0.001 s，取整误差约 0.5″，容差取 2e-4°。
#[test]
fn horizontal_matches_meeus_example_13b() {
    let ra = (23.0 + 9.0 / 60.0 + 16.641 / 3600.0) * 15.0;
    let dec = -(6.0 + 43.0 / 60.0 + 11.61 / 3600.0);
    let theta0 = (8.0 + 34.0 / 60.0 + 56.853 / 3600.0) * 15.0;
    let observer = Observer::new(deg(-(77.0 + 3.0 / 60.0 + 56.0 / 3600.0)), deg(38.0 + 55.0 / 60.0 + 17.0 / 3600.0), Length::from_value(real(0.0), LengthUnit::Meter));
    let p = Position::from_spherical_in_frame(ReferenceFrame::TrueEquator(Epoch::new(real(2446896.30625))), deg(ra), deg(dec), Length::from_value(real(1.0e11), LengthUnit::Meter));
    let h = hour_angle(&p, &observer, deg(theta0));
    assert!((to_deg(h) - 64.352133).abs() < 2e-4, "H = {}", to_deg(h));
    let hz = horizontal(&p, &observer, deg(theta0));
    assert!((to_deg(hz.azimuth) - 248.0337).abs() < 2e-4, "A = {}", to_deg(hz.azimuth));
    assert!((to_deg(hz.altitude) - 15.1249).abs() < 2e-4, "h = {}", to_deg(hz.altitude));
}

fn load() -> Option<(crate::astronomy::ephemeris::Vsop87, crate::astronomy::ephemeris::Elpmpp02Data)> {
    let vsop = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).ok()?;
    let loader = crate::repo::default_loader();
    let elp = load_all(&loader, crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).ok()?;
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    Some((vsop, elp))
}

/// 视赤道位置用真黄赤交角转回黄经，应与现有视黄经接口一致（二者架历元相差光行时，岁差差异 < 0.001″）。
#[test]
fn equatorial_consistent_with_ecliptic_longitude() {
    let Some((vsop, elp)) = load() else {
        println!("skipped: 历表数据缺失");
        return;
    };
    let opts = ApparentPipelineOptions::pipeline_default();
    for jd in [2451545.0, 2460000.25, 1660000.5] {
        let t = TimePoint::new(TimeScale::TT, real(jd));
        let tc = (real(jd) - real(2451545.0)) / 36525.0;
        let eps = mean_obliquity(tc.clone()).rad() + nutation_for_apparent(tc).1.rad();
        let ecl_lon = |p: Position| {
            let [x, y, z] = p.to_meters();
            let ye = y * eps.clone().cos() + z * eps.clone().sin();
            PlaneAngle::from_rad(ye.atan2(x)).wrap_to_2pi()
        };
        for (name, from_eq, from_lon) in [
            ("太阳", ecl_lon(sun_apparent_equatorial(&vsop, t, &opts)), sun_apparent_ecliptic_longitude_with_options(&vsop, t, &opts)),
            ("月亮", ecl_lon(moon_apparent_equatorial(&elp, t, &opts)), moon_apparent_ecliptic_longitude_with_options(&elp, t, &opts)),
        ] {
            let d = PlaneAngle::from_rad(from_eq.rad() - from_lon.rad()).wrap_to_signed_pi().arcsec().as_f64();
            assert!(d.abs() < 0.01, "{name} JD {jd}：视赤道→黄经与视黄经接口差 {d}″");
        }
    }
}

/// 月亮站心视差：长安（东经 108.94°，北纬 34.26°）。地心与站心高度角之差 p 应满足 sin p = (ρ/Δ)·sin z′
/// （z′ 为站心天顶距；忽略地球扁率导致的天顶方向差，容差 30″）。
#[test]
fn moon_topocentric_parallax_at_changan() {
    let Some((_vsop, elp)) = load() else {
        println!("skipped: 历表数据缺失");
        return;
    };
    let observer = Observer::new(deg(108.94), deg(34.26), Length::from_value(real(400.0), LengthUnit::Meter));
    let opts = ApparentPipelineOptions::pipeline_default();
    let mut checked = 0;
    for k in 0..24 {
        let jd_tt: Real = real(1650000.5) + real(k as f64) * real(0.0417);
        let dt = MorrisonStephenson2021.estimate(jd_tt.clone()).value;
        let jd_ut1 = jd_tt.clone() - dt.seconds() / 86400.0;
        let g = gast(jd_ut1, jd_tt.clone());
        let geo = moon_apparent_equatorial(&elp, TimePoint::new(TimeScale::TT, jd_tt), &opts);
        let topo = topocentric(geo, &observer, g);
        let h_geo = horizontal(&geo, &observer, g).altitude;
        let h_topo = horizontal(&topo, &observer, g).altitude;
        if to_deg(h_topo) < 5.0 {
            continue;
        }
        let rho = observer.geocentric_distance().meters().as_f64();
        let delta = geo.norm().meters().as_f64();
        let z_topo = (90.0 - to_deg(h_topo)).to_radians();
        let expect = ((rho / delta) * z_topo.sin()).asin().to_degrees();
        let got = to_deg(h_geo) - to_deg(h_topo);
        assert!((got - expect).abs() * 3600.0 < 30.0, "k={k}：视差 {got}° vs 期望 {expect}°");
        assert!(got > 0.0, "站心高度应低于地心高度");
        checked += 1;
    }
    assert!(checked >= 5, "月亮在地平以上的样本太少：{checked}");
}

fn changan() -> Observer {
    Observer::new(deg(108.94), deg(34.26), Length::from_value(real(400.0), LengthUnit::Meter))
}

fn true_equator_state(jd: f64) -> State6 {
    State6::from_si_in_frame(
        ReferenceFrame::TrueEquator(Epoch::new(real(jd))),
        real(3.0e8), real(-1.0e8), real(5.0e7),
        real(500.0), real(800.0), real(-200.0),
    )
}

fn ctx() -> ObservationContext {
    ObservationContext::new(changan(), deg(40.0))
}

/// 图的站心边与 `topocentric` 同一算式；速度减去 ω × r，且 ω × r 与观测者位置对 GAST 的数值微分一致。
#[test]
fn graph_edge_matches_topocentric_and_rotation_velocity() {
    let jd = 2451545.0;
    let (c, g) = (ctx(), deg(40.0));
    let obs = c.observer;
    let geo = true_equator_state(jd);
    let target = ReferenceFrame::TopocentricEquator(Epoch::new(real(jd)));
    let out = TransformGraph::default_graph().transform_to_observed(geo.clone(), target, &c, real(jd));
    assert_eq!(out.frame(), target);
    assert_eq!(out.velocity.frame, out.frame());
    let direct = topocentric(geo.position, &obs, g).to_meters();
    let (pos, vel) = out.to_meters_and_m_per_s();
    for i in 0..3 {
        assert!((pos[i].as_f64() - direct[i].as_f64()).abs() < 1e-6, "位置分量 {i}");
    }
    let epoch = Epoch::new(real(jd));
    let w = crate::astronomy::time::earth_rotation::earth_rotation_rate().as_f64();
    let dt = 1.0;
    let at = |s: f64| obs.position_true_equator(PlaneAngle::from_rad(g.rad() + real(w * s)), epoch).to_meters();
    let (p1, p0) = (at(dt), at(-dt));
    let geo_v = [500.0, 800.0, -200.0];
    for i in 0..3 {
        let numeric = (p1[i].as_f64() - p0[i].as_f64()) / (2.0 * dt);
        let got = geo_v[i] - vel[i].as_f64();
        assert!((got - numeric).abs() < 1e-3, "速度分量 {i}：{got} vs 数值微分 {numeric}");
    }
}

/// 赤道上观测者正东的天体：周日光行差使其向东（运动方向）偏 v/c ≈ 0.32″，距离不变。
#[test]
fn diurnal_aberration_is_toward_motion() {
    let jd = 2451545.0;
    let e = Epoch::new(real(jd));
    let c = ObservationContext::new(Observer::new(deg(0.0), deg(0.0), Length::from_value(real(0.0), LengthUnit::Meter)), deg(0.0));
    // GAST = 0、经度 0：观测者在 +x，自转速度沿 +y。天体在 +x（天顶方向），偏向 +y。
    let d = 1.0e17;
    let s = State6::from_si_in_frame(ReferenceFrame::TopocentricEquator(e), real(d), real(0.0), real(0.0), real(0.0), real(0.0), real(0.0));
    let out = diurnal_aberration_state(&s, &c);
    let [x, y, _] = out.position.to_meters();
    let arcsec = y.as_f64().atan2(x.as_f64()).to_degrees() * 3600.0;
    assert!((arcsec - 0.32).abs() < 0.005, "周日光行差 {arcsec}″");
    assert!((out.position.norm().meters().as_f64() / d - 1.0).abs() < 1e-12);
}

/// 走到几何地平的方位、高度与 `horizontal` 相差不超过周日光行差（0.32″），含 Meeus 例 13.b 的同一式。
#[test]
fn graph_horizontal_matches_function() {
    let jd = 2451545.0;
    let c = ctx();
    let geo = true_equator_state(jd);
    let target = ReferenceFrame::HorizontalGeometric(Epoch::new(real(jd)));
    let out = TransformGraph::default_graph().transform_to_observed(geo.clone(), target, &c, real(jd));
    let hz = horizontal(&topocentric(geo.position, &c.observer, c.gast), &c.observer, c.gast);
    let [e, n, u] = out.position.to_meters();
    let alt = u.atan2((e * e + n * n).sqrt());
    let az = PlaneAngle::from_rad(e.atan2(n)).wrap_to_2pi();
    assert!((alt - hz.altitude.rad()).abs().as_f64().to_degrees() * 3600.0 < 0.33);
    let daz = PlaneAngle::from_rad(az.rad() - hz.azimuth.rad()).wrap_to_signed_pi().rad().as_f64().to_degrees() * 3600.0;
    assert!(daz.abs() * alt.cos().as_f64() < 0.33, "方位差 {daz}″");
}

/// 站心地平直角坐标的速度含 Ṙ·r：与相邻两时刻（GAST 按 ω 变化）的坐标差分一致。
#[test]
fn horizontal_velocity_matches_finite_difference() {
    let jd = 2451545.0;
    let e = Epoch::new(real(jd));
    let w = crate::astronomy::time::earth_rotation::earth_rotation_rate();
    let aberrated = |s: f64| {
        let f = ReferenceFrame::TopocentricAberratedEquator(e);
        State6::from_si_in_frame(f, real(3.0e8 + 500.0 * s), real(-1.0e8 + 800.0 * s), real(5.0e7 - 200.0 * s), real(500.0), real(800.0), real(-200.0))
    };
    let at = |s: f64| {
        let c = ObservationContext::new(changan(), PlaneAngle::from_rad(deg(40.0).rad() + w * real(s)));
        to_horizontal_state(&aberrated(s), &c)
    };
    let (p1, p0) = (at(1.0).position.to_meters(), at(-1.0).position.to_meters());
    let v = at(0.0).velocity.to_m_per_s();
    for i in 0..3 {
        let numeric = (p1[i].as_f64() - p0[i].as_f64()) / 2.0;
        assert!((v[i].as_f64() - numeric).abs() < 1e-2, "分量 {i}：{} vs {numeric}", v[i].as_f64());
    }
}

/// 岁差、章动的 Ṙ：FK5 中静止的点，到真赤道架的速度即 Ṙ·r，须与相邻时刻旋转结果的差分一致。
#[test]
fn precession_nutation_rate_matches_finite_difference() {
    let Some(_) = load() else {
        println!("skipped: 历表数据缺失");
        return;
    };
    let graph = TransformGraph::default_graph();
    let (p, jd) = ([1.5e11, -3.0e10, 2.0e10], 2460000.25);
    let at = |t: f64| {
        let s = State6::from_si_in_frame(ReferenceFrame::FK5, real(p[0]), real(p[1]), real(p[2]), real(0.0), real(0.0), real(0.0));
        graph.transform_to(s, ReferenceFrame::TrueEquator(Epoch::new(real(t))), real(t))
    };
    let h_day = 0.05;
    let (a, b) = (at(jd + h_day).position.to_meters(), at(jd - h_day).position.to_meters());
    let got = at(jd).velocity.to_m_per_s();
    for i in 0..3 {
        let numeric = (a[i].as_f64() - b[i].as_f64()) / (2.0 * h_day * 86400.0);
        assert!((got[i].as_f64() - numeric).abs() < 0.2 * numeric.abs().max(1e-3), "分量 {i}：{} vs {numeric}", got[i].as_f64());
    }
}

/// 视地平：方位不变，高度增加折射量；模型由图的 refraction_model 决定，路径带对应 key。
#[test]
fn graph_apparent_horizontal_applies_selected_model() {
    let jd = 2451545.0;
    let e = Epoch::new(real(jd));
    let c = ctx();
    let geo = true_equator_state(jd);
    let run = |g: TransformGraph, target| g.transform_to_observed(geo.clone(), target, &c, real(jd));
    let geom = run(TransformGraph::default_graph(), ReferenceFrame::HorizontalGeometric(e)).position.to_meters();
    let alt = |p: [Real; 3]| p[2].atan2((p[0] * p[0] + p[1] * p[1]).sqrt());
    for m in [RefractionModel::Bennett, RefractionModel::Saemundsson] {
        let graph = TransformGraph::default_graph().with_refraction_model(m);
        let keys: Vec<_> = graph.shortest_path(ReferenceFrame::HorizontalGeometric(e), ReferenceFrame::HorizontalApparent(e)).iter().map(|p| p.edge_key.clone()).collect();
        assert_eq!(keys, [Some(m.edge_key().to_string())]);
        let out = run(graph, ReferenceFrame::HorizontalApparent(e));
        assert_eq!(out.frame(), ReferenceFrame::HorizontalApparent(e));
        let p = out.position.to_meters();
        let want = refraction_from_geometric(PlaneAngle::from_rad(alt(geom)), m, &Atmosphere::default()).rad();
        assert!((alt(p) - alt(geom) - want).abs().as_f64() < 1e-10, "{m:?} 高度增量");
        assert!((p[0].atan2(p[1]) - geom[0].atan2(geom[1])).abs().as_f64() < 1e-12, "{m:?} 方位应不变");
    }
}

/// 折射后的速度与相邻两时刻折射结果的差分一致。
#[test]
fn refracted_velocity_matches_finite_difference() {
    let e = Epoch::new(real(2451545.0));
    let f = ReferenceFrame::HorizontalGeometric(e);
    let (p, v) = ([3.0e8, 2.0e8, 2.0e7], [300.0, -400.0, 900.0]);
    let at = |s: f64| {
        let st = State6::from_si_in_frame(f, real(p[0] + v[0] * s), real(p[1] + v[1] * s), real(p[2] + v[2] * s), real(v[0]), real(v[1]), real(v[2]));
        refract_state(&st, RefractionModel::Bennett, &Atmosphere::default())
    };
    let (a, b) = (at(10.0).position.to_meters(), at(-10.0).position.to_meters());
    let got = at(0.0).velocity.to_m_per_s();
    for i in 0..3 {
        let numeric = (a[i].as_f64() - b[i].as_f64()) / 20.0;
        assert!((got[i].as_f64() - numeric).abs() < 1e-2, "分量 {i}：{} vs {numeric}", got[i].as_f64());
    }
}

/// 从 FK5 出发的路径经赤道链后以站心边收尾，且不影响到视黄道的既有路径。
#[test]
fn graph_routes_fk5_to_topocentric_via_true_equator() {
    let graph = TransformGraph::default_graph();
    let e = Epoch::new(real(2451545.0));
    let path = graph.shortest_path(ReferenceFrame::FK5, ReferenceFrame::TopocentricEquator(e));
    let ids: Vec<_> = path.iter().map(|p| p.to.id_str()).collect();
    assert_eq!(ids, ["MeanEquator(epoch)", "TrueEquator(epoch)", "TopocentricEquator(epoch)"]);
    let path = graph.shortest_path(ReferenceFrame::FK5, ReferenceFrame::HorizontalApparent(e));
    let ids: Vec<_> = path.iter().map(|p| p.to.id_str()).collect();
    assert_eq!(ids, [
        "MeanEquator(epoch)", "TrueEquator(epoch)", "TopocentricEquator(epoch)",
        "TopocentricAberratedEquator(epoch)", "HorizontalGeometric(epoch)", "HorizontalApparent(epoch)",
    ]);
    let path = graph.shortest_path(ReferenceFrame::FK5, ReferenceFrame::ApparentEcliptic(e));
    let ids: Vec<_> = path.iter().map(|p| p.to.id_str()).collect();
    assert_eq!(ids, ["MeanEquator(epoch)", "TrueEquator(epoch)", "ApparentEcliptic(epoch)"]);
}

/// 管线图展示同一条边：分类为平移，终点原点角色为站心。
#[test]
fn visualization_includes_topocentric_edge() {
    let viz = TransformGraph::default_graph().visualization_data();
    let id = "TopocentricEquator(epoch)";
    assert!(viz.node_ids.iter().any(|n| n == id));
    assert_eq!(node_origin(id), Some(OriginRole::Topocentric));
    let edge = viz.edges.iter().find(|e| e.to_id == id).expect("缺站心边");
    assert_eq!(edge.from_id, "TrueEquator(epoch)");
    assert_eq!(edge.kind, TransitionKind::FrameTranslation);
    assert_eq!(edge.form, TransitionForm::Translation);
    for (from, to, form) in [
        ("TopocentricEquator(epoch)", "TopocentricAberratedEquator(epoch)", TransitionForm::Aberration),
        ("TopocentricAberratedEquator(epoch)", "HorizontalGeometric(epoch)", TransitionForm::Rotation),
        ("HorizontalGeometric(epoch)", "HorizontalApparent(epoch)", TransitionForm::Mapping),
    ] {
        let found: Vec<_> = viz.edges.iter().filter(|e| e.from_id == from && e.to_id == to).collect();
        assert_eq!(found.len(), 1, "{from} → {to}");
        assert_eq!(found[0].form, form);
        assert_eq!(node_origin(to), Some(OriginRole::Topocentric));
    }
}

/// 不给观测者不能走站心边，不做隐式退回。
#[test]
#[should_panic(expected = "需要观测者")]
fn transform_to_without_observer_panics() {
    let jd = 2451545.0;
    let target = ReferenceFrame::TopocentricEquator(Epoch::new(real(jd)));
    TransformGraph::default_graph().transform_to(true_equator_state(jd), target, real(jd));
}

/// 站心状态有独立标识，重复平移直接失败。
#[test]
#[should_panic(expected = "站心平移要求地心真赤道架")]
fn double_topocentric_panics() {
    let (obs, g) = (changan(), deg(40.0));
    let once = topocentric(true_equator_state(2451545.0).position, &obs, g);
    topocentric(once, &obs, g);
}
