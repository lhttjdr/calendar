//! 月食：地心几何，时刻全球同一（TT）；某地可见与否由月亮地平高度决定。
//!
//! 地影按 Danjon 法（NASA 月食典所用，<https://eclipse.gsfc.nasa.gov/LEcat5/shadow.html>）：
//! 半影半径 Rp = 1.01·Pm + Ss + Ps，本影半径 Ru = 1.01·Pm − Ss + Ps（1.01 ≈ 1 + 1/85 − 1/594，含 45° 纬度扁率修正）；
//! Pm、Ps 为月、日赤道地平视差，Ss 为太阳视半径。食分 = (R + Sm − σ)/(2 Sm)，σ 为月心与影心角距，Sm 为月亮视半径。
//! 影心取太阳地心视位置的反方向。

use crate::astronomy::apparent::{horizontal, topocentric};
use crate::astronomy::geodesy::Observer;
use crate::astronomy::time::earth_rotation::gast;
use crate::math::real::{real_const, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;
use crate::quantity::epoch::Epoch;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;

use super::search::{bisect_root, minimize_golden, seconds};
use super::solar_eclipse::{
    angle_between, dot, eclipse_window, norm, scale, sub, sun_radius_m, EclipseEphemeris, SunMoonSource, EARTH_EQUATORIAL_RADIUS_M,
    K_PENUMBRA,
};

/// 某时刻月食几何量（角度为弧度）。
#[derive(Clone, Copy, Debug)]
pub struct LunarEclipseGeometry {
    /// 月心与影心角距 σ。
    pub separation: Real,
    pub umbra_radius: Real,
    pub penumbra_radius: Real,
    pub moon_semidiameter: Real,
    /// 月心在影心以北为正的 γ（地球赤道半径）。
    pub gamma: Real,
}

impl LunarEclipseGeometry {
    pub fn umbral_magnitude(&self) -> Real {
        (self.umbra_radius + self.moon_semidiameter - self.separation) / (real_const(2.0) * self.moon_semidiameter)
    }
    pub fn penumbral_magnitude(&self) -> Real {
        (self.penumbra_radius + self.moon_semidiameter - self.separation) / (real_const(2.0) * self.moon_semidiameter)
    }
}

pub fn lunar_eclipse_geometry(src: &dyn SunMoonSource, jd_tt: Real) -> LunarEclipseGeometry {
    let (s, m) = src.sun_moon_m(jd_tt);
    let (ds, dm) = (norm(s), norm(m));
    let ae = EARTH_EQUATORIAL_RADIUS_M;
    let pm = (ae / dm).asin();
    let ps = (ae / ds).asin();
    let ss = (sun_radius_m() / ds).asin();
    let sm = (K_PENUMBRA * ae / dm).asin();
    let shadow = scale(s, real_const(-1.0) / ds);
    let moon_dir = scale(m, real_const(1.0) / dm);
    let sep = angle_between(shadow, moon_dir);
    // 影心处的北向单位矢：n = (ẑ − (ẑ·û)û)/|…|
    let zhat = [real_const(0.0), real_const(0.0), real_const(1.0)];
    let n = sub(zhat, scale(shadow, dot(zhat, shadow)));
    let north = dot(moon_dir, scale(n, real_const(1.0) / norm(n)));
    let sign = if north < real_const(0.0) { real_const(-1.0) } else { real_const(1.0) };
    LunarEclipseGeometry {
        separation: sep,
        umbra_radius: real_const(1.01) * pm - ss + ps,
        penumbra_radius: real_const(1.01) * pm + ss + ps,
        moon_semidiameter: sm,
        gamma: sign * sep.sin() * dm / ae,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LunarEclipseKind {
    Penumbral,
    Partial,
    Total,
}

/// 月食（时刻为 TT）。
#[derive(Clone, Copy, Debug)]
pub struct LunarEclipse {
    pub greatest_tt: Real,
    pub gamma: Real,
    pub penumbral_magnitude: Real,
    pub umbral_magnitude: Real,
    pub kind: LunarEclipseKind,
    /// 半影食始、终（P1、P4）。
    pub penumbral: (Real, Real),
    /// 初亏、复圆（U1、U4）。
    pub partial: Option<(Real, Real)>,
    /// 食既、生光（U2、U3）。
    pub total: Option<(Real, Real)>,
}

/// 在 `jd_tt_guess`（近望时刻，误差 1 日内）附近求月食；无月食返回 None。
pub fn lunar_eclipse(eph: &EclipseEphemeris, jd_tt_guess: Real) -> Option<LunarEclipse> {
    lunar_eclipse_in(&eclipse_window(eph, jd_tt_guess), jd_tt_guess)
}

/// 同 [`lunar_eclipse`]，日月位置取自已建好的来源（通常是 [`eclipse_window`]，便于与某地可见性共用一个插值窗）。
pub fn lunar_eclipse_in(src: &dyn SunMoonSource, jd_tt_guess: Real) -> Option<LunarEclipse> {
    let geo = |t: Real| lunar_eclipse_geometry(src, t);
    let (t0, _) = minimize_golden(|t| geo(t).separation, jd_tt_guess - real_const(1.5), jd_tt_guess + real_const(1.5), seconds(0.05));
    let g = geo(t0);
    let pen = g.penumbral_magnitude();
    if pen <= real_const(0.0) {
        return None;
    }
    let umb = g.umbral_magnitude();
    let span = real_const(0.3);
    let tol = seconds(1.0);
    let contacts = |f: &dyn Fn(Real) -> Real| -> Option<(Real, Real)> {
        Some((bisect_root(f, t0 - span, t0, tol)?, bisect_root(f, t0, t0 + span, tol)?))
    };
    let p = contacts(&|t| {
        let x = geo(t);
        x.separation - (x.penumbra_radius + x.moon_semidiameter)
    })?;
    let partial = if umb > real_const(0.0) {
        contacts(&|t| {
            let x = geo(t);
            x.separation - (x.umbra_radius + x.moon_semidiameter)
        })
    } else {
        None
    };
    let total = if umb >= real_const(1.0) {
        contacts(&|t| {
            let x = geo(t);
            x.separation - (x.umbra_radius - x.moon_semidiameter)
        })
    } else {
        None
    };
    let kind = if umb >= real_const(1.0) {
        LunarEclipseKind::Total
    } else if umb > real_const(0.0) {
        LunarEclipseKind::Partial
    } else {
        LunarEclipseKind::Penumbral
    };
    Some(LunarEclipse {
        greatest_tt: t0,
        gamma: g.gamma,
        penumbral_magnitude: pen,
        umbral_magnitude: umb,
        kind,
        penumbral: p,
        partial,
        total,
    })
}

/// 月亮站心几何高度角（某地某 TT 时刻）。
pub fn moon_altitude(eph: &EclipseEphemeris, observer: &Observer, jd_tt: Real, delta_t: Duration) -> PlaneAngle {
    moon_altitude_in(eph, observer, jd_tt, delta_t)
}

/// 同 [`moon_altitude`]，日月位置取自给定来源。
pub fn moon_altitude_in(src: &dyn SunMoonSource, observer: &Observer, jd_tt: Real, delta_t: Duration) -> PlaneAngle {
    let jd_ut1 = jd_tt - delta_t.seconds() / real_const(86400.0);
    let g = gast(jd_ut1, jd_tt);
    let (_, m) = src.sun_moon_m(jd_tt);
    let m = Position::from_si_meters_in_frame(ReferenceFrame::TrueEquator(Epoch::new(jd_tt)), m[0], m[1], m[2]);
    horizontal(&topocentric(m, observer, g), observer, g).altitude
}
