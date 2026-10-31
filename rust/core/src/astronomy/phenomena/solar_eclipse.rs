//! 日食：全球情况（贝塞尔基本面）与局地情况（站心视位置）。
//!
//! - 全球 [`solar_eclipse_global`]：基本面过地心、垂直于影轴，z 轴指向从月到日方向；x、y 为月心在基本面上的投影（地球赤道半径
//!   为单位，y 向北），γ = ±√(x²+y²)（取 y 符号），l₁、l₂ 为半影/本影锥在基本面上的半径（l₂ < 0 为本影）。判定影轴是否
//!   扫过地球时 y 按扁率缩放 y₁ = y/ρ₁，ρ₁ = √(1 − e²cos²d)。与 ΔT 无关，可直接对照 NASA 日食典。
//! - 局地 [`solar_eclipse_local`]：站心日月角距与视半径，求初亏、食既、食甚、生光、复圆与食分；给出太阳几何高度，并按标准地平
//!   高度 −0°50′（日面上缘切地平：折射 34′ + 视半径 16′）求可见部分（日出带食、日没带食）。ΔT 由调用方给出。
//!
//! 常数取 NASA 日食典：月球半径比 k = 0.2722810（全/环判定、食分）与 0.2725076（半影、初亏复圆），太阳视半径 1 AU 处 959.63″。
//! 依据：*Explanatory Supplement to the Astronomical Almanac* (2013) 第 11 章；Espenak & Meeus (2006)。

use crate::astronomy::apparent::{
    horizontal, moon_apparent_equatorial, sun_apparent_equatorial, topocentric, ApparentPipelineOptions,
};
use crate::astronomy::constant::AU_METERS;
use crate::astronomy::ephemeris::{Elpmpp02Data, Vsop87};
use crate::astronomy::geodesy::Observer;
use crate::astronomy::time::earth_rotation::gast;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real_const, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;
use crate::quantity::epoch::Epoch;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;

use super::search::{bisect_root, minimize_golden, seconds};

/// 地球赤道半径（GRS80）[m]。
pub const EARTH_EQUATORIAL_RADIUS_M: Real = real_const(6378137.0);
/// GRS80 第一偏心率平方 e² = f(2 − f)。
pub const EARTH_E2: Real = real_const(0.00669438002290);
/// 月球半径/地球赤道半径：本影（全/环判定、食分）。
pub const K_UMBRA: Real = real_const(0.2722810);
/// 月球半径/地球赤道半径：半影（初亏、复圆）。
pub const K_PENUMBRA: Real = real_const(0.2725076);
/// 太阳视半径（1 AU 处，角秒）。
pub const SUN_SEMIDIAMETER_1AU_ARCSEC: Real = real_const(959.63);

/// 太阳半径 [m]（由 1 AU 处视半径换算）。
pub fn sun_radius_m() -> Real {
    AU_METERS * PlaneAngle::from_arcsec(SUN_SEMIDIAMETER_1AU_ARCSEC).rad().sin()
}

/// 日月历表与视位置管线选项。日月食请用 [`ApparentPipelineOptions::pipeline_default`]（月球光行时）。
pub struct EclipseEphemeris<'a> {
    pub vsop: &'a Vsop87,
    pub elp: &'a Elpmpp02Data,
    pub options: ApparentPipelineOptions,
}

impl<'a> EclipseEphemeris<'a> {
    pub fn new(vsop: &'a Vsop87, elp: &'a Elpmpp02Data) -> Self {
        Self { vsop, elp, options: ApparentPipelineOptions::pipeline_default() }
    }
    /// 太阳、月亮地心视位置（当日真赤道架）。
    pub fn sun_moon(&self, jd_tt: Real) -> (Position, Position) {
        let t = TimePoint::new(TimeScale::TT, jd_tt);
        (sun_apparent_equatorial(self.vsop, t, &self.options), moon_apparent_equatorial(self.elp, t, &self.options))
    }
}

pub(crate) type V3 = [Real; 3];

/// 日月地心视位置（当日真赤道架，米）的来源：直接计算 [`EclipseEphemeris`] 或其插值 [`InterpolatedSunMoon`]。
pub trait SunMoonSource {
    fn sun_moon_m(&self, jd_tt: Real) -> (V3, V3);
}

impl SunMoonSource for EclipseEphemeris<'_> {
    fn sun_moon_m(&self, jd_tt: Real) -> (V3, V3) {
        let (s, m) = self.sun_moon(jd_tt);
        (s.to_meters(), m.to_meters())
    }
}

/// 一次日月食时间窗内的日月位置切比雪夫插值。
///
/// 日月食搜索要在几天的窗口里做上百次求值；逐次走完整视位置管线（ELPMPP02 全级数 + 光行时迭代）很慢。按日食典的通行做法，
/// 在窗口内取 n 个切比雪夫节点精确计算，之后的搜索在插值上进行。地心日月运动平滑，窗口 ±2 日、32 节点时插值误差远小于 1 m
/// （见测试 `interpolation_matches_direct`）。
pub struct InterpolatedSunMoon {
    center: Real,
    half_width: Real,
    /// [太阳 x,y,z, 月亮 x,y,z] 各分量的切比雪夫系数。
    coeffs: [Vec<Real>; 6],
}

impl InterpolatedSunMoon {
    pub fn new(src: &dyn SunMoonSource, center: Real, half_width_days: Real, nodes: usize) -> Self {
        let n = nodes;
        let pi = Real::pi();
        let samples: Vec<[Real; 6]> = (0..n)
            .map(|k| {
                let x = (pi * (real_const(k as f64) + real_const(0.5)) / real_const(n as f64)).cos();
                let (sv, mv) = src.sun_moon_m(center + half_width_days * x);
                [sv[0], sv[1], sv[2], mv[0], mv[1], mv[2]]
            })
            .collect();
        let coeffs = std::array::from_fn(|c| {
            (0..n)
                .map(|j| {
                    let mut sum = real_const(0.0);
                    for (k, smp) in samples.iter().enumerate() {
                        let ang = pi * real_const(j as f64) * (real_const(k as f64) + real_const(0.5)) / real_const(n as f64);
                        sum = sum + smp[c] * ang.cos();
                    }
                    sum * real_const(2.0) / real_const(n as f64)
                })
                .collect()
        });
        Self { center, half_width: half_width_days, coeffs }
    }

    fn eval(&self, c: usize, x: Real) -> Real {
        // Clenshaw；首项系数折半
        let a = &self.coeffs[c];
        let (mut b1, mut b2) = (real_const(0.0), real_const(0.0));
        for j in (1..a.len()).rev() {
            let b0 = real_const(2.0) * x * b1 - b2 + a[j];
            b2 = b1;
            b1 = b0;
        }
        x * b1 - b2 + a[0] / real_const(2.0)
    }
}

impl SunMoonSource for InterpolatedSunMoon {
    fn sun_moon_m(&self, jd_tt: Real) -> (V3, V3) {
        let x = (jd_tt - self.center) / self.half_width;
        let v: [Real; 6] = std::array::from_fn(|c| self.eval(c, x));
        ([v[0], v[1], v[2]], [v[3], v[4], v[5]])
    }
}

/// 日月食搜索窗：猜测时刻 ±2 日，32 个切比雪夫节点。
pub fn eclipse_window(eph: &EclipseEphemeris, jd_tt_guess: Real) -> InterpolatedSunMoon {
    InterpolatedSunMoon::new(eph, jd_tt_guess, real_const(2.0), 32)
}

pub(crate) fn dot(a: V3, b: V3) -> Real {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub(crate) fn norm(a: V3) -> Real {
    dot(a, a).sqrt()
}
pub(crate) fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub(crate) fn scale(a: V3, s: Real) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
/// 两向量夹角（atan2(|a×b|, a·b)，小角度下数值稳定）。
pub(crate) fn angle_between(a: V3, b: V3) -> Real {
    let c = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    norm(c).atan2(dot(a, b))
}

/// 某时刻的贝塞尔量（长度以地球赤道半径为单位）。
#[derive(Clone, Copy, Debug)]
pub struct Besselian {
    pub x: Real,
    pub y: Real,
    pub z: Real,
    /// 影轴赤纬 d（弧度）。
    pub d: Real,
    pub l1: Real,
    pub l2: Real,
    pub tan_f1: Real,
    pub tan_f2: Real,
}

impl Besselian {
    /// ρ₁ = √(1 − e²cos²d)，用于按扁率缩放 y。
    pub fn rho1(&self) -> Real {
        let c = self.d.cos();
        (real_const(1.0) - EARTH_E2 * c * c).sqrt()
    }
    /// 影轴到地心距离（按扁率缩放，< 1 则影轴扫过地球）。
    pub fn m1(&self) -> Real {
        let y1 = self.y / self.rho1();
        (self.x * self.x + y1 * y1).sqrt()
    }
}

/// 计算贝塞尔量。
pub fn besselian(src: &dyn SunMoonSource, jd_tt: Real) -> Besselian {
    besselian_with_axes(src, jd_tt).0
}

/// 贝塞尔量及基本面三轴 i、j、k（当日真赤道架单位向量；k 由月指向日，i 在赤道面内，j 向北）。
pub fn besselian_with_axes(src: &dyn SunMoonSource, jd_tt: Real) -> (Besselian, [V3; 3]) {
    let (s, m) = src.sun_moon_m(jd_tt);
    let g = sub(s, m);
    let dist_sm = norm(g);
    let k = scale(g, real_const(1.0) / dist_sm);
    let a = k[1].atan2(k[0]);
    let d = k[2].asin();
    let (sa, ca, sd, cd) = (a.sin(), a.cos(), d.sin(), d.cos());
    let i = [-sa, ca, real_const(0.0)];
    let j = [-sd * ca, -sd * sa, cd];
    let ae = EARTH_EQUATORIAL_RADIUS_M;
    let (x, y, z) = (dot(m, i) / ae, dot(m, j) / ae, dot(m, k) / ae);
    let rs = sun_radius_m();
    let (rp, ru) = (K_PENUMBRA * ae, K_UMBRA * ae);
    let sin_f1 = (rs + rp) / dist_sm;
    let sin_f2 = (rs - ru) / dist_sm;
    let cos_f1 = (real_const(1.0) - sin_f1 * sin_f1).sqrt();
    let cos_f2 = (real_const(1.0) - sin_f2 * sin_f2).sqrt();
    let (tan_f1, tan_f2) = (sin_f1 / cos_f1, sin_f2 / cos_f2);
    let l1 = (z * ae * tan_f1 + rp / cos_f1) / ae;
    let l2 = (z * ae * tan_f2 - ru / cos_f2) / ae;
    (Besselian { x, y, z, d, l1, l2, tan_f1, tan_f2 }, [i, j, k])
}

/// 日食类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolarEclipseKind {
    Partial,
    Annular,
    Total,
    /// 全环食：中心线上部分为全食、部分为环食。
    Hybrid,
}

/// 日食全球情况（极大食时刻为 TT，与 ΔT 无关）。
#[derive(Clone, Copy, Debug)]
pub struct SolarEclipseGlobal {
    pub greatest_tt: Real,
    /// γ：极大食时影轴到地心距离（地球赤道半径），影轴在地心以北为正。
    pub gamma: Real,
    /// 食分（NASA 定义：中心食为极大食点日月视直径比；非中心食为最接近影轴的地面点的食分）。
    pub magnitude: Real,
    pub kind: SolarEclipseKind,
    /// 影轴是否扫过地球（中心食）。
    pub central: bool,
}

/// 在 `jd_tt_guess`（近朔时刻，误差 1 日内）附近求日食全球情况；无日食返回 None。
pub fn solar_eclipse_global(eph: &EclipseEphemeris, jd_tt_guess: Real) -> Option<SolarEclipseGlobal> {
    solar_eclipse_global_in(&eclipse_window(eph, jd_tt_guess), jd_tt_guess)
}

/// 同 [`solar_eclipse_global`]，日月位置取自已建好的来源（通常是 [`eclipse_window`]，便于与某地情况共用一个插值窗）。
pub fn solar_eclipse_global_in(eph: &dyn SunMoonSource, jd_tt_guess: Real) -> Option<SolarEclipseGlobal> {
    let m_plain = |t: Real| {
        let b = besselian(eph, t);
        (b.x * b.x + b.y * b.y).sqrt()
    };
    let (t0, _) = minimize_golden(m_plain, jd_tt_guess - real_const(1.5), jd_tt_guess + real_const(1.5), seconds(0.05));
    let b = besselian(eph, t0);
    let m1 = b.m1();
    if m1 >= real_const(1.0) + b.l1 {
        return None;
    }
    let gamma = (b.x * b.x + b.y * b.y).sqrt() * if b.y < real_const(0.0) { real_const(-1.0) } else { real_const(1.0) };
    let central = m1 < real_const(1.0);
    let (kind, magnitude) = if central {
        // 极大食点：影轴与地面交点，ζ ≈ ρ₁·√(1 − x² − y₁²)
        let y1 = b.y / b.rho1();
        let zeta = b.rho1() * (real_const(1.0) - b.x * b.x - y1 * y1).sqrt();
        let big_l1 = b.l1 - zeta * b.tan_f1;
        let big_l2 = b.l2 - zeta * b.tan_f2;
        let mag = (big_l1 - big_l2) / (big_l1 + big_l2);
        // 中心线两端（影轴切地球边缘，ζ = 0）的 l₂ 符号与极大食点 L₂ 不同 → 全环食
        let edge = |t: Real| besselian(eph, t).m1() - real_const(1.0);
        let ends = [
            bisect_root(edge, t0 - real_const(0.25), t0, seconds(1.0)),
            bisect_root(edge, t0, t0 + real_const(0.25), seconds(1.0)),
        ];
        let total_at_greatest = big_l2 < real_const(0.0);
        let hybrid = ends.iter().flatten().any(|&te| (besselian(eph, te).l2 < real_const(0.0)) != total_at_greatest);
        let kind = if hybrid {
            SolarEclipseKind::Hybrid
        } else if total_at_greatest {
            SolarEclipseKind::Total
        } else {
            SolarEclipseKind::Annular
        };
        (kind, mag)
    } else {
        let delta = m1 - real_const(1.0);
        let mag = (b.l1 - delta) / (b.l1 + b.l2);
        // 本影/伪本影只擦过地球边缘的非中心全/环食
        let kind = if delta < b.l2.abs() {
            if b.l2 < real_const(0.0) {
                SolarEclipseKind::Total
            } else {
                SolarEclipseKind::Annular
            }
        } else {
            SolarEclipseKind::Partial
        };
        (kind, mag)
    };
    Some(SolarEclipseGlobal { greatest_tt: t0, gamma, magnitude, kind, central })
}

// ---------------------------------------------------------------------------------------------------------------------

/// 标准地平高度：日面上缘切地平（大气折射 34′ + 太阳视半径 16′）。
pub fn sun_standard_altitude() -> PlaneAngle {
    PlaneAngle::from_arcsec(real_const(-50.0 * 60.0))
}

/// 局地某时刻的几何量。
#[derive(Clone, Copy, Debug)]
pub struct LocalSolarGeometry {
    /// 站心日月中心角距。
    pub separation: Real,
    pub sun_semidiameter: Real,
    /// 月亮站心视半径（本影 k）。
    pub moon_semidiameter: Real,
    /// 月亮站心视半径（半影 k）。
    pub moon_semidiameter_penumbral: Real,
    /// 太阳站心几何高度角（未含折射）。
    pub sun_altitude: PlaneAngle,
}

impl LocalSolarGeometry {
    /// 食分 = (日半径 + 月半径 − 角距) / 日直径。
    pub fn magnitude(&self) -> Real {
        (self.sun_semidiameter + self.moon_semidiameter - self.separation) / (real_const(2.0) * self.sun_semidiameter)
    }
}

/// 局地几何量。`delta_t` 为 TT − UT1。
pub fn local_solar_geometry(src: &dyn SunMoonSource, observer: &Observer, jd_tt: Real, delta_t: Duration) -> LocalSolarGeometry {
    let jd_ut1 = jd_tt - delta_t.seconds() / real_const(86400.0);
    let g = gast(jd_ut1, jd_tt);
    let (s, m) = src.sun_moon_m(jd_tt);
    let frame = ReferenceFrame::TrueEquator(Epoch::new(jd_tt));
    let s = Position::from_si_meters_in_frame(frame, s[0], s[1], s[2]);
    let m = Position::from_si_meters_in_frame(frame, m[0], m[1], m[2]);
    let (st, mt) = (topocentric(s, observer, g), topocentric(m, observer, g));
    let (sv, mv) = (st.to_meters(), mt.to_meters());
    let (ds, dm) = (norm(sv), norm(mv));
    let ae = EARTH_EQUATORIAL_RADIUS_M;
    LocalSolarGeometry {
        separation: angle_between(sv, mv),
        sun_semidiameter: (sun_radius_m() / ds).asin(),
        moon_semidiameter: (K_UMBRA * ae / dm).asin(),
        moon_semidiameter_penumbral: (K_PENUMBRA * ae / dm).asin(),
        sun_altitude: horizontal(&st, observer, g).altitude,
    }
}

/// 日食局地情况。时刻均为 TT；UT1 = TT − `delta_t_seconds`。
#[derive(Clone, Debug)]
pub struct LocalSolarEclipse {
    pub delta_t_seconds: Real,
    /// 初亏、复圆（几何，不论太阳是否在地平以上）。
    pub first_contact: Real,
    pub last_contact: Real,
    /// 食既、生光（全食或环食时）。
    pub second_contact: Option<Real>,
    pub third_contact: Option<Real>,
    pub greatest: Real,
    pub magnitude: Real,
    pub sun_altitude_first: PlaneAngle,
    pub sun_altitude_greatest: PlaneAngle,
    pub sun_altitude_last: PlaneAngle,
    /// 太阳上缘在地平以上的那段食（标准地平高度 −0°50′）；None 表示全程不可见。
    pub visible: Option<(Real, Real)>,
    /// 可见部分的最大食分。
    pub max_visible_magnitude: Option<Real>,
}

/// 在 `jd_tt_guess`（近朔，误差 1 日内）附近求某地日食局地情况；该地无食返回 None。
pub fn solar_eclipse_local(
    eph: &EclipseEphemeris,
    observer: &Observer,
    jd_tt_guess: Real,
    delta_t: Duration,
) -> Option<LocalSolarEclipse> {
    solar_eclipse_local_in(&eclipse_window(eph, jd_tt_guess), observer, jd_tt_guess, delta_t)
}

/// 同 [`solar_eclipse_local`]，日月位置取自已建好的来源（同一次日食换不同 ΔT 时可共用一个插值窗）。
pub fn solar_eclipse_local_in(
    src: &dyn SunMoonSource,
    observer: &Observer,
    jd_tt_guess: Real,
    delta_t: Duration,
) -> Option<LocalSolarEclipse> {
    let geo = |t: Real| local_solar_geometry(src, observer, t, delta_t);
    // 站心角距在朔附近 ±1.5 日内单峰
    let (tmax, _) = minimize_golden(|t| geo(t).separation, jd_tt_guess - real_const(1.5), jd_tt_guess + real_const(1.5), seconds(0.5));
    let gmax = geo(tmax);
    if gmax.separation >= gmax.sun_semidiameter + gmax.moon_semidiameter_penumbral {
        return None;
    }
    let outer = |t: Real| {
        let g = geo(t);
        g.separation - (g.sun_semidiameter + g.moon_semidiameter_penumbral)
    };
    let span = real_const(0.2);
    let c1 = bisect_root(outer, tmax - span, tmax, seconds(1.0))?;
    let c4 = bisect_root(outer, tmax, tmax + span, seconds(1.0))?;
    let inner = |t: Real| {
        let g = geo(t);
        g.separation - (g.moon_semidiameter - g.sun_semidiameter).abs()
    };
    let (c2, c3) = if inner(tmax) < real_const(0.0) {
        (bisect_root(inner, c1, tmax, seconds(1.0)), bisect_root(inner, tmax, c4, seconds(1.0)))
    } else {
        (None, None)
    };
    // 可见部分：太阳高度 > 标准地平高度
    let h0 = sun_standard_altitude().rad();
    let above = |t: Real| geo(t).sun_altitude.rad() - h0;
    let (a1, am, a4) = (above(c1), above(tmax), above(c4));
    let up = |v: Real| v > real_const(0.0);
    let tol = seconds(1.0);
    // 初亏、复圆时太阳是否在标准地平高度以上，分四种情况；不处理一次食内既落又升的极地情形
    let visible = match (up(a1), up(a4)) {
        (true, true) => Some((c1, c4)),
        (false, true) => bisect_root(above, c1, c4, tol).map(|r| (r, c4)), // 日出带食
        (true, false) => bisect_root(above, c1, c4, tol).map(|s| (c1, s)), // 日没带食
        (false, false) if up(am) => {
            match (bisect_root(above, c1, tmax, tol), bisect_root(above, tmax, c4, tol)) {
                (Some(r), Some(s)) => Some((r, s)),
                _ => None,
            }
        }
        (false, false) => None,
    };
    let max_visible_magnitude = visible.map(|(a, b)| {
        if tmax >= a && tmax <= b {
            gmax.magnitude()
        } else {
            let edge = if tmax < a { a } else { b };
            geo(edge).magnitude()
        }
    });
    Some(LocalSolarEclipse {
        delta_t_seconds: delta_t.seconds(),
        first_contact: c1,
        last_contact: c4,
        second_contact: c2,
        third_contact: c3,
        greatest: tmax,
        magnitude: gmax.magnitude(),
        sun_altitude_first: geo(c1).sun_altitude,
        sun_altitude_greatest: gmax.sun_altitude,
        sun_altitude_last: geo(c4).sun_altitude,
        visible,
        max_visible_magnitude,
    })
}
