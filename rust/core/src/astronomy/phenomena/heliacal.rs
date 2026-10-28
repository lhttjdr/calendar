//! 行星与恒星的晨见、晨伏、夕见、夕伏（偕日升/落，heliacal rising/setting）。
//!
//! 定义（Schaefer 1987 §3、1993 §3.10：偕日升为合日后天体首次可在晨光中看见之日）：
//! - 「晨可见」：晨光时段（当地平太阳时子夜至正午之间、太阳几何高度在 `twilight_limit_deg` 与 −50′ 之间，
//!   即日面上缘未出地平）内存在某时刻，天体视高度 > `min_altitude_deg` 且天体大气外 V 星等 ≤ 该方向极限星等
//!   （[`super::visibility`]，50% 探测概率）；
//! - 「昏可见」同理，取正午至次日子夜的昏影时段；
//! - 晨见 = 晨可见由否转是之日；晨伏 = 由是转否前的最后一日；夕见、夕伏同理。
//!
//! 只在晨昏蒙影时段内判定，使四种事件都对应天体相对太阳的位置：内行星「夕伏 → 下合 → 晨见 → 晨伏 → 上合 → 夕见」；
//! 外行星与恒星「夕伏 → 合 → 晨见」即古之伏见，另有冲日前后的「晨伏」（晨光中已没入西方地平，cosmical setting）与
//! 「夕见」（昏影中初见于东方，acronychal rising）。后两者原文（1987 §4、1993 §3.10）指出定义含糊，其日期依赖蒙影下限。
//! 月光会使暗弱天体在相邻几日内反复可见/不可见，亦如实输出（下游可按需合并）。
//!
//! 实现：日、月（[`GeocentricTracks`]，与观测者无关，可供多地共用）与目标天体的地心视位置（当日真赤道）及行星星等
//! 以 1 日为步长取样，四点 Lagrange 插值到扫描时刻；时刻按 `sample_minutes` 步进扫描。GAST 每半日算一次，其间按
//! 自转角速率外推。

use crate::astronomy::apparent::{
    body_apparent_equatorial, horizontal, planet_magnitude, right_ascension_declination, star_apparent_equatorial, topocentric,
    ApparentPipelineOptions,
};
use crate::astronomy::correction::bennett_refraction_default;
use crate::astronomy::geodesy::Observer;
use crate::astronomy::pipeline::{Body, EphemerisProvider, Planet};
use crate::astronomy::stars::CatalogStar;
use crate::astronomy::time::earth_rotation::gast;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;
use crate::quantity::epoch::Epoch;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;

use super::track::{MagTrack, Track};
use super::visibility::{glare_nl, limiting_magnitude, moon_magnitude, sky_brightness_nl, Eye, SkyConditions, SkyGeometry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeliacalEventKind {
    /// 晨见（偕日升，morning first）
    MorningFirst,
    /// 晨伏（morning last）
    MorningLast,
    /// 夕见（evening first）
    EveningFirst,
    /// 夕伏（偕日落，evening last）
    EveningLast,
}

/// 一次伏见事件：可见（首日或末日）那一天里可见裕度最大的时刻的情况。
#[derive(Clone, Copy, Debug)]
pub struct HeliacalEvent {
    pub kind: HeliacalEventKind,
    /// 该时刻（TT）
    pub jd_tt: Real,
    /// 天体视高度（度，含折射）
    pub altitude_deg: Real,
    /// 太阳几何高度（度）
    pub sun_altitude_deg: Real,
    /// 天体与太阳方位角之差（度，[0, 180]）
    pub azimuth_difference_deg: Real,
    /// 天体大气外 V 星等
    pub magnitude: Real,
    /// 该方向极限星等
    pub limiting_magnitude: Real,
}

impl HeliacalEvent {
    /// 视弧（arcus visionis）：天体视高度 − 太阳高度。
    pub fn arcus_visionis_deg(&self) -> Real {
        self.altitude_deg - self.sun_altitude_deg
    }
}

/// 搜索参数。
#[derive(Clone, Debug)]
pub struct HeliacalOptions {
    pub sky: SkyConditions,
    pub eye: Eye,
    /// 天体最低视高度（度）。缺省 0：只要求在视地平之上，地平附近由消光自然压低可见性。
    pub min_altitude_deg: Real,
    /// 扫描步长（分钟）
    pub sample_minutes: Real,
    /// 晨昏蒙影时段下限：太阳几何高度（度）。缺省 −18°（天文晨昏蒙影）；高纬夏季太阳不到此高度时整夜都算蒙影时段。
    pub twilight_limit_deg: Real,
    /// 是否计月光（1993 式 15b）。月轨迹须在 [`GeocentricTracks`] 中提供。
    pub include_moon: bool,
    /// 是否计月亮的眼内散射眩光（Schaefer 1991 式 3；大气散射晕已在月光模型中，不重复计）。原文以 161 次月旁观测检验，
    /// 误差约 0.35 等。月亮按位于月心的点源处理，角距下限取月面视半径 0.26°。
    pub include_moon_glare: bool,
}

impl Default for HeliacalOptions {
    fn default() -> Self {
        Self {
            sky: SkyConditions::default(),
            eye: Eye::default(),
            min_altitude_deg: real(0.0),
            sample_minutes: real(2.0),
            twilight_limit_deg: real(-18.0),
            include_moon: true,
            include_moon_glare: true,
        }
    }
}

/// 行星 B−V 色指数：Mallama, Krobusek & Pavlov 2017（Icarus 282, 19）表 3「Reference」行 B − V。
pub fn planet_color_bv(planet: Planet) -> Real {
    real(match planet {
        Planet::Mercury => 0.28 - -0.69,
        Planet::Venus => -3.68 - -4.38,
        Planet::Mars => -0.24 - -1.60,
        Planet::Jupiter => -8.54 - -9.40,
        Planet::Saturn => -7.84 - -8.91,
        Planet::Uranus => -6.61 - -7.11,
        Planet::Neptune => -6.55 - -6.94,
    })
}

/// 伏见的目标天体。
#[derive(Clone, Copy, Debug)]
pub enum HeliacalTarget {
    /// 行星，星等按 Mallama & Hilton 2018 随相角计算，B−V 按 [`planet_color_bv`]。
    Planet(Planet),
    /// 行星，指定固定星等（如复现 Schaefer 1987 表 7 金星取 −3.4）。
    PlanetFixedMagnitude(Planet, Real),
    /// 星表恒星（缺 B−V 时取 0.5）。
    Star(CatalogStar),
}

/// 日、月地心视位置取样（与观测者无关），供多地、多目标共用。
pub struct GeocentricTracks {
    sun: Track,
    moon: Option<Track>,
}

impl GeocentricTracks {
    /// [jd_start, jd_end]（TT）。`with_moon` 为假时不取月（不计月光，或提供者不含月）。
    pub fn new<P: EphemerisProvider>(eph: &P, jd_start: Real, jd_end: Real, with_moon: bool, opts: &ApparentPipelineOptions) -> Self {
        let (t0, t1) = (jd_start - real(2.0), jd_end + real(2.0));
        let body = |b: Body| Track::build(t0, t1, |t| body_apparent_equatorial(eph, b, TimePoint::new(TimeScale::TT, t), opts).to_meters());
        Self { sun: body(Body::Sun), moon: with_moon.then(|| body(Body::Moon)) }
    }
}

fn angle_deg(a: [Real; 3], b: [Real; 3]) -> Real {
    let cr = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let n = (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt();
    n.atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) * real(180.0 / core::f64::consts::PI)
}

/// 方位角、高度角（度）→ 单位向量（地平系），用于角距。
fn horizon_vec(az_deg: Real, alt_deg: Real) -> [Real; 3] {
    let r = real(core::f64::consts::PI / 180.0);
    let (a, h) = (az_deg * r, alt_deg * r);
    [h.cos() * a.cos(), h.cos() * a.sin(), h.sin()]
}

#[derive(Clone, Copy)]
struct Sample {
    jd_tt: Real,
    alt: Real,
    sun_alt: Real,
    daz: Real,
    mag: Real,
    lim: Real,
}

impl Sample {
    fn margin(&self) -> Real {
        self.lim - self.mag
    }
}

/// 目标天体在某时段的取样（与观测者无关，可供多地、多组参数共用）。
pub struct PreparedTarget {
    pos: Track,
    mag: MagTrack,
    bv: Real,
}

impl PreparedTarget {
    /// [jd_start, jd_end]（TT）内取样（两端各留 2 日余量）。
    pub fn new<P: EphemerisProvider>(eph: &P, target: &HeliacalTarget, jd_start: Real, jd_end: Real) -> Self {
        let (t0, t1) = (jd_start - real(2.0), jd_end + real(2.0));
        let po = ApparentPipelineOptions::default();
        let tp = |t: Real| TimePoint::new(TimeScale::TT, t);
        match *target {
            HeliacalTarget::Planet(p) => Self {
                pos: Track::build(t0, t1, |t| body_apparent_equatorial(eph, Body::Planet(p), tp(t), &po).to_meters()),
                mag: MagTrack::build(t0, t1, |t| planet_magnitude(eph, p, tp(t))),
                bv: planet_color_bv(p),
            },
            HeliacalTarget::PlanetFixedMagnitude(p, m) => Self {
                pos: Track::build(t0, t1, |t| body_apparent_equatorial(eph, Body::Planet(p), tp(t), &po).to_meters()),
                mag: MagTrack::build(t0, t1, |_| Some(m)),
                bv: planet_color_bv(p),
            },
            HeliacalTarget::Star(s) => Self {
                pos: Track::build(t0, t1, |t| star_apparent_equatorial(eph, &s, tp(t), &po).to_meters()),
                mag: MagTrack::build(t0, t1, |_| Some(s.vmag)),
                bv: s.bv.unwrap_or(real(0.5)),
            },
        }
    }
}

/// 某地的伏见计算环境。
pub struct HeliacalSky<'a> {
    tracks: &'a GeocentricTracks,
    observer: Observer,
    delta_t: Duration,
    opts: HeliacalOptions,
}

/// 逐日晨、昏两窗的可见性（可见裕度最大时刻；窗口内无满足地平条件的时刻为 None）。
#[derive(Clone, Copy, Debug)]
pub struct DailyVisibility {
    /// 当地平太阳时子夜（TT）
    pub midnight_tt: Real,
    /// [晨, 昏]：(时刻 TT, 极限星等 − 星等)
    pub windows: [Option<(Real, Real)>; 2],
}

impl<'a> HeliacalSky<'a> {
    /// `delta_t` 为 TT − UT1（区间内视为常数）。
    pub fn new(tracks: &'a GeocentricTracks, observer: Observer, delta_t: Duration, options: HeliacalOptions) -> Self {
        Self { tracks, observer, delta_t, opts: options }
    }

    fn horiz(&self, geo: [Real; 3], jd_tt: Real, g: PlaneAngle) -> (Real, Real) {
        let p = Position::from_si_meters_in_frame(ReferenceFrame::TrueEquator(Epoch::new(jd_tt)), geo[0], geo[1], geo[2]);
        let h = horizontal(&topocentric(p, &self.observer, g), &self.observer, g);
        let d = real(180.0 / core::f64::consts::PI);
        (h.azimuth.rad() * d, h.altitude.rad() * d)
    }

    /// 某时刻：太阳在晨昏蒙影时段、天体在最低高度之上且星等有定义时返回该时刻的可见性；否则 None。`g` 为该时刻 GAST。
    fn sample(&self, tg: &PreparedTarget, jd_tt: Real, g: PlaneAngle) -> Option<Sample> {
        let s_geo = self.tracks.sun.at(jd_tt);
        let (s_az, s_alt) = self.horiz(s_geo, jd_tt, g);
        if s_alt >= real(-50.0 / 60.0) || s_alt < self.opts.twilight_limit_deg {
            return None;
        }
        let p_geo = tg.pos.at(jd_tt);
        let (p_az, p_alt_geom) = self.horiz(p_geo, jd_tt, g);
        let refr = bennett_refraction_default(PlaneAngle::from_deg(p_alt_geom)).rad() * real(180.0 / core::f64::consts::PI);
        let p_alt = p_alt_geom + refr;
        if p_alt <= self.opts.min_altitude_deg {
            return None;
        }
        let mag = tg.mag.mag_at(jd_tt)?;
        let p_vec = horizon_vec(p_az, p_alt);
        let moon = if self.opts.include_moon { self.tracks.moon.as_ref() } else { None }.and_then(|m| {
            let m_geo = m.at(jd_tt);
            let (m_az, m_alt) = self.horiz(m_geo, jd_tt, g);
            if m_alt <= real(0.0) {
                return None;
            }
            // 月相角：月→日与月→地夹角
            let to_sun = [s_geo[0] - m_geo[0], s_geo[1] - m_geo[1], s_geo[2] - m_geo[2]];
            let phase = angle_deg(to_sun, [-m_geo[0], -m_geo[1], -m_geo[2]]);
            Some((angle_deg(p_vec, horizon_vec(m_az, m_alt)), real(90.0) - m_alt, moon_magnitude(phase)))
        });
        let daz = (PlaneAngle::from_deg(p_az - s_az).wrap_to_signed_pi().rad() * real(180.0 / core::f64::consts::PI)).abs();
        let z = real(90.0) - p_alt;
        let geom = SkyGeometry {
            z_deg: z,
            rho_sun_deg: angle_deg(p_vec, horizon_vec(s_az, s_alt)),
            azimuth_diff_sun_deg: daz,
            z_sun_deg: real(90.0) - s_alt,
            moon,
        };
        let zr = z * real(core::f64::consts::PI / 180.0);
        let (ra, _) = right_ascension_declination(&Position::from_si_meters_in_frame(
            ReferenceFrame::TrueEquator(Epoch::new(jd_tt)),
            s_geo[0],
            s_geo[1],
            s_geo[2],
        ));
        let (k, dm) = self.opts.sky.extinction.at(self.observer.height.meters() / real(1000.0), self.observer.latitude.rad(), ra.rad(), zr);
        let mut sky = sky_brightness_nl(&geom, k, &self.opts.sky);
        if self.opts.include_moon_glare {
            if let Some((rho, z_moon, m_moon)) = moon {
                let kx = k * super::visibility::air_mass(z_moon * real(core::f64::consts::PI / 180.0));
                sky = sky + glare_nl(m_moon, rho.max(real(0.26)), kx).eye;
            }
        }
        let r = limiting_magnitude(sky, dm, tg.bv, &self.opts.eye);
        Some(Sample { jd_tt, alt: p_alt, sun_alt: s_alt, daz, mag, lim: r.limiting_magnitude })
    }

    /// [t0, t1]（TT）内可见裕度（极限星等 − 星等）最大的取样；无任何满足条件的取样返回 None。
    fn best(&self, tg: &PreparedTarget, t0: Real, t1: Real) -> Option<Sample> {
        let step = self.opts.sample_minutes / real(1440.0);
        let n = ((t1 - t0) / step).floor().as_f64() as usize;
        // GAST 在半日内按地球自转角速率外推（章动引起的赤经变化半日内 < 1 ms 时），免去逐点章动计算
        let dt_days = self.delta_t.seconds() / real(86400.0);
        let g0 = gast(t0 - dt_days, t0);
        let rate = real(2.0 * core::f64::consts::PI * 1.002_737_811_911_354_5);
        let mut best: Option<Sample> = None;
        for i in 0..=n {
            let dt = step * real(i as f64);
            let g = PlaneAngle::from_rad(g0.rad() + rate * dt);
            if let Some(s) = self.sample(tg, t0 + dt, g) {
                if best.as_ref().is_none_or(|b| s.margin() > b.margin()) {
                    best = Some(s);
                }
            }
        }
        best
    }

    /// 逐日晨昏可见性，从包含 `jd_start` 的当地日起，到 `jd_end` 止（TT，须在 [`GeocentricTracks`] 范围内）。
    fn daily(&self, tg: &PreparedTarget, jd_start: Real, jd_end: Real) -> Vec<(Real, [Option<Sample>; 2])> {
        assert!(self.tracks.sun.covers(jd_start - real(1.0), jd_end + real(1.0)), "伏见区间超出日月取样范围");
        assert!(tg.pos.covers(jd_start - real(1.0), jd_end + real(1.0)), "伏见区间超出目标天体取样范围");
        // 当地平太阳时子夜（UT1）：jd_ut = k − 0.5 − λ/360°
        let dt_days = self.delta_t.seconds() / real(86400.0);
        let lon_days = self.observer.longitude.rad() / real(2.0 * core::f64::consts::PI);
        let first_midnight_ut = (jd_start - dt_days + real(0.5) + lon_days).floor() - real(0.5) - lon_days;
        let mut out = vec![];
        let mut d = 0usize;
        loop {
            let mid_tt = first_midnight_ut + real(d as f64) + dt_days;
            if mid_tt > jd_end {
                break;
            }
            let w = [self.best(tg, mid_tt, mid_tt + real(0.5)), self.best(tg, mid_tt + real(0.5), mid_tt + real(1.0))];
            out.push((mid_tt, w));
            d += 1;
        }
        out
    }

    /// 逐日晨昏可见性：[jd_start, jd_end]（TT）各当地日。
    pub fn daily_visibility(&self, target: &PreparedTarget, jd_start: Real, jd_end: Real) -> Vec<DailyVisibility> {
        self.daily(target, jd_start, jd_end)
            .into_iter()
            .map(|(m, w)| DailyVisibility { midnight_tt: m, windows: w.map(|s| s.map(|s| (s.jd_tt, s.margin()))) })
            .collect()
    }

    /// [jd_start, jd_end]（TT）内的晨见、晨伏、夕见、夕伏，按时刻排序。
    pub fn events(&self, target: &PreparedTarget, jd_start: Real, jd_end: Real) -> Vec<HeliacalEvent> {
        let (j0, j1) = (jd_start, jd_end);
        let days = self.daily(target, j0 - real(1.0), j1);
        let mut out = vec![];
        for pair in days.windows(2) {
            let (prev, cur) = (&pair[0].1, &pair[1].1);
            for half in 0..2 {
                let vis = |s: &Option<Sample>| s.as_ref().is_some_and(|s| s.margin() >= real(0.0));
                let (first, last) = if half == 0 {
                    (HeliacalEventKind::MorningFirst, HeliacalEventKind::MorningLast)
                } else {
                    (HeliacalEventKind::EveningFirst, HeliacalEventKind::EveningLast)
                };
                let ev = match (vis(&prev[half]), vis(&cur[half])) {
                    (false, true) => cur[half].map(|s| (first, s)),
                    (true, false) => prev[half].map(|s| (last, s)),
                    _ => None,
                };
                if let Some((kind, s)) = ev {
                    if s.jd_tt >= j0 && s.jd_tt <= j1 {
                        out.push(HeliacalEvent {
                            kind,
                            jd_tt: s.jd_tt,
                            altitude_deg: s.alt,
                            sun_altitude_deg: s.sun_alt,
                            azimuth_difference_deg: s.daz,
                            magnitude: s.mag,
                            limiting_magnitude: s.lim,
                        });
                    }
                }
            }
        }
        out.sort_by(|a, b| a.jd_tt.partial_cmp(&b.jd_tt).unwrap());
        out
    }
}

/// 单个天体的便捷入口：[jd_start, jd_end]（TT）内某地的伏见。多地、多天体请自建 [`GeocentricTracks`] 共用日月取样。
pub fn heliacal_events<P: EphemerisProvider>(
    eph: &P,
    target: &HeliacalTarget,
    observer: &Observer,
    delta_t: Duration,
    jd_start: Real,
    jd_end: Real,
    options: &HeliacalOptions,
) -> Vec<HeliacalEvent> {
    let tracks = GeocentricTracks::new(eph, jd_start, jd_end, options.include_moon, &ApparentPipelineOptions::default());
    let prepared = PreparedTarget::new(eph, target, jd_start, jd_end);
    HeliacalSky::new(&tracks, *observer, delta_t, options.clone()).events(&prepared, jd_start, jd_end)
}
