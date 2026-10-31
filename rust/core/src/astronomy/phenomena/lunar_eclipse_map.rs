//! 月食图：各食相时刻的月出月没界线、月下点，某地食况，月亮穿过地影的路径。
//!
//! 月食的时刻全球同一，地图只回答「何处月亮在地平上」。某时刻的界线是站心几何高度为 0 的点：地面法向 û（大地经纬度即其方向）的
//! 地面点 r(û) 在椭球上，sin h = (m − r)·û / |m − r|。沿月下点各方位对 û 与月下方向的夹角求根，故已含月亮视差（约 1°）与地球扁率，
//! 界线不是 90° 的圆。不计大气折射（与日食图的带食出没界线同为几何地平）。
//!
//! 某地「可见」按标准地平高度 −0°50′（月亮上缘切地平：折射 34′ + 视半径约 16′）。
//! ΔT 只经 GAST 进入，换 ΔT 时界线严格沿经度平移 [`longitude_shift_for_delta_t`]，与日食图同。

use crate::astronomy::geodesy::Observer;
use crate::astronomy::time::earth_rotation::gast;
use crate::math::real::{real, real_const, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;

use super::lunar_eclipse::{lunar_eclipse_geometry, lunar_eclipse_in, moon_altitude_in, LunarEclipse};
use super::search::{bisect_root, seconds};
use super::solar_eclipse::{dot, norm, scale, sub, sun_standard_altitude, SunMoonSource, EARTH_E2, EARTH_EQUATORIAL_RADIUS_M, V3};
use super::solar_eclipse_map::{MapPoint, Polyline};

/// 食相。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LunarPhase {
    /// 半影食始
    P1,
    /// 初亏
    U1,
    /// 食既
    U2,
    /// 食甚
    Greatest,
    /// 生光
    U3,
    /// 复圆
    U4,
    /// 半影食终
    P4,
}

impl LunarPhase {
    pub fn code(self) -> &'static str {
        match self {
            LunarPhase::P1 => "P1",
            LunarPhase::U1 => "U1",
            LunarPhase::U2 => "U2",
            LunarPhase::Greatest => "G",
            LunarPhase::U3 => "U3",
            LunarPhase::U4 => "U4",
            LunarPhase::P4 => "P4",
        }
    }
}

/// 该次月食实有的食相及时刻（TT），按时序。
pub fn lunar_phases(e: &LunarEclipse) -> Vec<(LunarPhase, Real)> {
    let mut v = vec![(LunarPhase::P1, e.penumbral.0)];
    if let Some((a, _)) = e.partial {
        v.push((LunarPhase::U1, a));
    }
    if let Some((a, _)) = e.total {
        v.push((LunarPhase::U2, a));
    }
    v.push((LunarPhase::Greatest, e.greatest_tt));
    if let Some((_, b)) = e.total {
        v.push((LunarPhase::U3, b));
    }
    if let Some((_, b)) = e.partial {
        v.push((LunarPhase::U4, b));
    }
    v.push((LunarPhase::P4, e.penumbral.1));
    v
}

/// 某食相的月下点与月出月没界线（闭合折线）。
#[derive(Clone, Debug)]
pub struct LunarPhaseCurve {
    pub phase: LunarPhase,
    pub jd_tt: Real,
    /// 月下点：地面法向指向月心的点（站心高度 90°）
    pub sublunar: MapPoint,
    pub horizon: Polyline,
    /// 此刻月亮相对影心的位置（穿影图用）
    pub offset: ShadowPathPoint,
}

/// 月亮穿过地影图上一点：以影心为原点，向东、向北的角距（弧度）。
#[derive(Clone, Copy, Debug)]
pub struct ShadowPathPoint {
    pub jd_tt: Real,
    pub east: Real,
    pub north: Real,
}

/// 月食图。
#[derive(Clone, Debug)]
pub struct LunarEclipseMap {
    pub eclipse: LunarEclipse,
    pub delta_t: Duration,
    pub curves: Vec<LunarPhaseCurve>,
    /// 食甚时的本影、半影半径与月亮视半径（弧度）
    pub umbra_radius: Real,
    pub penumbra_radius: Real,
    pub moon_semidiameter: Real,
    pub shadow_path: Vec<ShadowPathPoint>,
}

fn itrs(v: V3, g: Real) -> V3 {
    let (s, c) = (g.sin(), g.cos());
    [v[0] * c + v[1] * s, -v[0] * s + v[1] * c, v[2]]
}

fn point_of_normal(t: Real, u: V3) -> MapPoint {
    MapPoint {
        jd_tt: t,
        longitude: PlaneAngle::from_rad(u[1].atan2(u[0])),
        latitude: PlaneAngle::from_rad(u[2].atan2((u[0] * u[0] + u[1] * u[1]).sqrt())),
    }
}

/// 地面法向 û 处椭球面点（米，地固系）。
fn ground_of_normal(u: V3) -> V3 {
    let sp = u[2];
    let n = EARTH_EQUATORIAL_RADIUS_M / (real_const(1.0) - EARTH_E2 * sp * sp).sqrt();
    [n * u[0], n * u[1], n * (real_const(1.0) - EARTH_E2) * u[2]]
}

/// 法向 û 处的站心几何高度的正弦。
fn sin_alt(m: V3, u: V3) -> Real {
    let d = sub(m, ground_of_normal(u));
    dot(d, u) / norm(d)
}

/// 由站心高度函数求月下点：迭代令 û ∥ (m − r(û))。
fn sublunar_normal(m: V3) -> V3 {
    let mut u = scale(m, real_const(1.0) / norm(m));
    for _ in 0..6 {
        let d = sub(m, ground_of_normal(u));
        u = scale(d, real_const(1.0) / norm(d));
    }
    u
}

fn phase_curve(src: &dyn SunMoonSource, phase: LunarPhase, t: Real, delta_t: Duration, samples: usize) -> LunarPhaseCurve {
    let dt_days = delta_t.seconds() / real_const(86400.0);
    let g = gast(t - dt_days, t).rad();
    let (_, m_eq) = src.sun_moon_m(t);
    let m = itrs(m_eq, g);
    let u0 = sublunar_normal(m);
    // 与 û0 正交的两单位向量
    let refv = if u0[2].abs() < real_const(0.9) { [real_const(0.0), real_const(0.0), real_const(1.0)] } else { [real_const(1.0), real_const(0.0), real_const(0.0)] };
    let a = [u0[1] * refv[2] - u0[2] * refv[1], u0[2] * refv[0] - u0[0] * refv[2], u0[0] * refv[1] - u0[1] * refv[0]];
    let a = scale(a, real_const(1.0) / norm(a));
    let b = [u0[1] * a[2] - u0[2] * a[1], u0[2] * a[0] - u0[0] * a[2], u0[0] * a[1] - u0[1] * a[0]];
    let two_pi = real_const(2.0) * Real::pi();
    let mut horizon: Polyline = (0..samples)
        .map(|k| {
            let th = two_pi * real(k as f64) / real(samples as f64);
            let dir: V3 = std::array::from_fn(|i| th.cos() * a[i] + th.sin() * b[i]);
            let at = |c: Real| -> V3 { std::array::from_fn(|i| c.cos() * u0[i] + c.sin() * dir[i]) };
            // 高度 0 处 c 约 89°；夹角 80°—100° 内单调
            let c = bisect_root(|c| sin_alt(m, at(c)), PlaneAngle::from_deg(real(80.0)).rad(), PlaneAngle::from_deg(real(100.0)).rad(), real_const(1e-10))
                .expect("月亮高度在 80°—100° 夹角内应过零");
            point_of_normal(t, at(c))
        })
        .collect();
    let first = horizon[0];
    horizon.push(first);
    LunarPhaseCurve { phase, jd_tt: t, sublunar: point_of_normal(t, u0), horizon, offset: shadow_offset(src, t) }
}

/// 月亮相对影心的位置（向东、向北角距）。
fn shadow_offset(src: &dyn SunMoonSource, t: Real) -> ShadowPathPoint {
    let (s, m) = src.sun_moon_m(t);
    let shadow = scale(s, real_const(-1.0) / norm(s));
    let z = [real_const(0.0), real_const(0.0), real_const(1.0)];
    let e = [z[1] * shadow[2] - z[2] * shadow[1], z[2] * shadow[0] - z[0] * shadow[2], z[0] * shadow[1] - z[1] * shadow[0]];
    let e = scale(e, real_const(1.0) / norm(e));
    let n = [shadow[1] * e[2] - shadow[2] * e[1], shadow[2] * e[0] - shadow[0] * e[2], shadow[0] * e[1] - shadow[1] * e[0]];
    let w = dot(m, shadow);
    ShadowPathPoint { jd_tt: t, east: dot(m, e).atan2(w), north: dot(m, n).atan2(w) }
}

/// 求月食图。`jd_tt_guess` 为近望时刻（误差 1 日内）；无月食返回 None。
pub fn lunar_eclipse_map(src: &dyn SunMoonSource, jd_tt_guess: Real, delta_t: Duration, samples: usize) -> Option<LunarEclipseMap> {
    let eclipse = lunar_eclipse_in(src, jd_tt_guess)?;
    let curves = lunar_phases(&eclipse).into_iter().map(|(p, t)| phase_curve(src, p, t, delta_t, samples)).collect();
    let g = lunar_eclipse_geometry(src, eclipse.greatest_tt);
    let (a, b) = eclipse.penumbral;
    let margin = (b - a) * real_const(0.05);
    let n = 80;
    let shadow_path = (0..=n).map(|k| shadow_offset(src, a - margin + (b - a + margin * real_const(2.0)) * real(k as f64 / n as f64))).collect();
    Some(LunarEclipseMap {
        eclipse,
        delta_t,
        curves,
        umbra_radius: g.umbra_radius,
        penumbra_radius: g.penumbra_radius,
        moon_semidiameter: g.moon_semidiameter,
        shadow_path,
    })
}

/// 某地月食情况。
#[derive(Clone, Debug)]
pub struct LocalLunarEclipse {
    /// 各食相的时刻（TT）与月亮站心几何高度
    pub phases: Vec<(LunarPhase, Real, PlaneAngle)>,
    /// 月亮上缘在地平以上（标准地平高度 −0°50′）的那段食；None 表示全程不可见
    pub visible: Option<(Real, Real)>,
    /// 可见部分的最大本影、半影食分
    pub max_visible_umbral: Option<Real>,
    pub max_visible_penumbral: Option<Real>,
}

/// 某地月食情况。可见段按时序扫描（每 2 分钟）并二分界点；一次月食内既没又出的极区情形取首段。
pub fn lunar_eclipse_local_in(src: &dyn SunMoonSource, e: &LunarEclipse, observer: &Observer, delta_t: Duration) -> LocalLunarEclipse {
    let alt = |t: Real| moon_altitude_in(src, observer, t, delta_t);
    let phases: Vec<_> = lunar_phases(e).into_iter().map(|(p, t)| (p, t, alt(t))).collect();
    let h0 = sun_standard_altitude().rad();
    let above = |t: Real| alt(t).rad() - h0;
    let (a, b) = e.penumbral;
    let n = (((b - a) / seconds(120.0)).as_f64().ceil() as usize).max(1);
    let at = |k: usize| a + (b - a) * real(k as f64 / n as f64);
    let tol = seconds(1.0);
    let mut start = None;
    let mut end = None;
    let mut prev = above(at(0));
    if prev > real_const(0.0) {
        start = Some(a);
    }
    for k in 1..=n {
        let cur = above(at(k));
        if start.is_none() && cur > real_const(0.0) {
            start = bisect_root(above, at(k - 1), at(k), tol);
        } else if start.is_some() && prev > real_const(0.0) && cur <= real_const(0.0) {
            end = bisect_root(above, at(k - 1), at(k), tol);
            break;
        }
        prev = cur;
    }
    let visible = start.map(|s| (s, end.unwrap_or(b)));
    let best = visible.map(|(s, x)| if e.greatest_tt < s { s } else if e.greatest_tt > x { x } else { e.greatest_tt });
    let g = best.map(|t| lunar_eclipse_geometry(src, t));
    LocalLunarEclipse {
        phases,
        visible,
        max_visible_umbral: g.map(|g| g.umbral_magnitude().max(real_const(0.0))),
        max_visible_penumbral: g.map(|g| g.penumbral_magnitude().max(real_const(0.0))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
    use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
    use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
    use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
    use crate::astronomy::phenomena::longitude_shift_for_delta_t;
    use crate::astronomy::phenomena::solar_eclipse::{eclipse_window, EclipseEphemeris};
    use crate::quantity::length::Length;
    use crate::quantity::unit::LengthUnit;

    fn obs(p: &MapPoint) -> Observer {
        Observer::new(p.longitude, p.latitude, Length::from_value(real(0.0), LengthUnit::Meter))
    }

    /// 2018 年与前 200 年前后各找一次月食：界线上站心高度为 0、月下点高度 90°（均用独立的管线函数 moon_altitude_in）；
    /// ΔT 变化为纯经度平移；局地可见段自洽；穿影路径的最小角距等于食甚角距。
    #[test]
    fn curves_agree_with_topocentric_altitude() {
        let vsop = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
        assert!(try_init_full_nutation_from_repo());
        let eph = EclipseEphemeris::new(&vsop, &elp);
        for (jd, dt) in [(2458327.35, 69.0), (1648000.5, 12600.0)] {
            // 从该历元起按平望（平朔 + 半个朔望月）逐月找第一次月食
            use crate::astronomy::aspects::approximate_new_moon_jd;
            let n0 = ((jd - approximate_new_moon_jd(0).as_f64()) / 29.530588853).floor() as i32 - 1;
            let mut found = None;
            for n in n0..n0 + 40 {
                let guess = approximate_new_moon_jd(n) + real(14.765);
                let win = eclipse_window(&eph, guess);
                if lunar_eclipse_in(&win, guess).is_some() {
                    found = Some((win, guess));
                    break;
                }
            }
            let (win, guess) = found.expect("应能找到月食");
            let delta = Duration::in_seconds(real(dt));
            let map = lunar_eclipse_map(&win, guess, delta, 72).unwrap();
            let mut worst = 0.0_f64;
            for c in &map.curves {
                for p in c.horizon.iter().step_by(6) {
                    let h = moon_altitude_in(&win, &obs(p), c.jd_tt, delta).rad().as_f64().to_degrees();
                    worst = worst.max(h.abs());
                }
                let h = moon_altitude_in(&win, &obs(&c.sublunar), c.jd_tt, delta).rad().as_f64().to_degrees();
                assert!(h > 89.99, "月下点高度 {h}");
            }
            println!("JD {:.2}：{} 个食相，界线上月亮高度最大偏差 {worst:.6}°", map.eclipse.greatest_tt.as_f64(), map.curves.len());
            assert!(worst < 1e-4, "{worst}");

            let dd = Duration::in_seconds(real(1200.0));
            let b = lunar_eclipse_map(&win, guess, Duration::in_seconds(real(dt + 1200.0)), 72).unwrap();
            let shift = longitude_shift_for_delta_t(dd).rad().as_f64().to_degrees();
            let got = (b.curves[0].sublunar.longitude.rad() - map.curves[0].sublunar.longitude.rad()).as_f64().to_degrees();
            assert!(((got - shift + 540.0).rem_euclid(360.0) - 180.0).abs() < 1e-6, "{got} vs {shift}");

            // 月下点必全程可见（食甚时）；其对跖点食甚时不可见
            let g = map.curves.iter().find(|c| c.phase == LunarPhase::Greatest).unwrap();
            let here = lunar_eclipse_local_in(&win, &map.eclipse, &obs(&g.sublunar), delta);
            let (s, x) = here.visible.expect("月下点应可见");
            assert!(s <= map.eclipse.greatest_tt && x >= map.eclipse.greatest_tt);
            let m = here.max_visible_penumbral.unwrap().as_f64();
            assert!((m - map.eclipse.penumbral_magnitude.as_f64()).abs() < 1e-9, "{m}");
            let anti = Observer::new(
                PlaneAngle::from_rad(g.sublunar.longitude.rad() + Real::pi()),
                PlaneAngle::from_rad(-g.sublunar.latitude.rad()),
                Length::from_value(real(0.0), LengthUnit::Meter),
            );
            let there = lunar_eclipse_local_in(&win, &map.eclipse, &anti, delta);
            assert!(moon_altitude_in(&win, &anti, map.eclipse.greatest_tt, delta).rad().as_f64() < 0.0);
            if let Some((s, x)) = there.visible {
                assert!(map.eclipse.greatest_tt < s || map.eclipse.greatest_tt > x);
            }

            let geo = lunar_eclipse_geometry(&win, map.eclipse.greatest_tt);
            let min = map.shadow_path.iter().map(|p| (p.east * p.east + p.north * p.north).sqrt().as_f64()).fold(f64::INFINITY, f64::min);
            let step_rad = geo.moon_semidiameter.as_f64() * 2.0; // 路径取点间距上限量级
            assert!(min >= geo.separation.as_f64() - 1e-6 && min <= geo.separation.as_f64() + step_rad, "{min} vs {}", geo.separation.as_f64());
        }
    }
}
