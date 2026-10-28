//! 视赤道位置、站心视位置与地平坐标的测试。

use super::*;
use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::frame::nutation::{nutation_for_apparent, try_init_full_nutation_from_repo};
use crate::astronomy::frame::precession::mean_obliquity;
use crate::astronomy::geodesy::Observer;
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
