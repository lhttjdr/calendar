//! 月掩星（月掩恒星、月掩行星）：某地的站心几何与肉眼可见性。
//!
//! 几何：
//! - 站心视位置（当日真赤道）：月、天体由历表管线得地心视位置，再减观测者位置（[`topocentric`]）；
//!   恒星方向乘以 10²⁰ m 量级的距离，使站心修正只剩可忽略的视差。
//! - 月面视半径 s = asin(k·a_e / d)，k = 0.2725076（IAU 平均月球半径与地球赤道半径之比，与日食半影 k 同值），
//!   a_e = 6378137 m；不计月面地形（实际边缘起伏可达 ±2″，掩星时刻相应差约 ±4 s）。
//! - 入掩、出掩：天体中心与月心站心角距 = s 的时刻（行星取中心）。
//! - 位置角自北经东量；亮边判定：接触点相对月心的方向与日向（亮边中点方向）夹角 < 90° 为亮边。尖角 C 为接触点到较近
//!   尖点的角距（0–90°）。
//!
//! 搜索：月与天体地心视位置按 1 日取样、四点插值，逐 10 分钟算站心角距粗筛；候选附近改用逐点精算，黄金分割求最近，
//! 二分求入掩、出掩（0.1 s）。
//!
//! 可见性（[`occultation_visibility`]）：Schaefer, Bulder & Bourgeois 1992（*Icarus* 100, 60）的模型取肉眼情形
//! （放大率 1、无望远镜衍射与镜面散射，视场内月面比例 f_FOV = 1）：
//! - 亮边：背景 = 月面亮度 5.67×10¹⁰ I_月/S（式 7，S 为亮面比例）+ 天空（式 43）；
//! - 暗边：背景 = 地照 + 天空 + 月光眩光（式 44）。地照取 Schaefer 1993 式 13：B = 2.59·10^{−0.4 m_月(180° − α)}，即 1992
//!   式 8、9（原文式 9 系数 78 按月面反照率 0.067 才成立，文中印作 0.67，属排印错误；1993 文明言 0.067）；
//!   眩光 = 大气 6.25×10⁷ I* θ⁻²(10^{−0.4kX} − 10^{−0.8kX})（式 10）+ 眼内 4.63×10⁷ 10^{−0.4kX} I* θ⁻²（式 13），
//!   有效角距 θ = (1 − 0.4e^{−C/30})θ_月 [cos²C + (1 − S + sin C)²]^{1/2}（式 11）。
//! - 天空不含月光（月光由上项计入），用 [`sky_brightness_nl`] 去掉月亮；阈值用 [`limiting_magnitude`]。
//! - 式 46 的 ΔM（计时观测所需的高置信度，原文拟合 1.2 等）与观测者经验项作为参数，缺省 0（50% 探测概率）。

use crate::astronomy::apparent::{body_apparent_equatorial, horizontal, star_apparent_equatorial, topocentric, ApparentPipelineOptions};
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

use super::search::{bisect_root, minimize_golden, seconds};
use super::solar_eclipse::{EARTH_EQUATORIAL_RADIUS_M, K_PENUMBRA};
use super::track::Track;
use super::visibility::{air_mass, limiting_magnitude, moon_magnitude, sky_brightness_nl, Eye, SkyConditions, SkyGeometry};

/// 被掩天体。
#[derive(Clone, Copy, Debug)]
pub enum OccultedBody {
    Planet(Planet),
    Star(CatalogStar),
}

/// 恒星方向的放大倍数（米）：使站心修正等效于无穷远。
const STAR_DISTANCE_M: f64 = 1.0e20;

const R2D: f64 = 180.0 / core::f64::consts::PI;

/// 一次接触（入掩或出掩）的情况。角度均为度。
#[derive(Clone, Copy, Debug)]
pub struct OccultationContact {
    pub jd_tt: Real,
    /// 接触点位置角（自北经东，相对月心）
    pub position_angle_deg: Real,
    /// 是否亮边
    pub bright_limb: bool,
    /// 尖角：接触点到较近尖点的角距（0–90°）
    pub cusp_angle_deg: Real,
    /// 月心站心几何高度、天体视高度（含折射前的几何高度）、太阳几何高度
    pub moon_altitude_deg: Real,
    pub body_altitude_deg: Real,
    pub sun_altitude_deg: Real,
    /// 月相角（日—月—观测者）与亮面比例 S = (1 + cos α)/2
    pub moon_phase_angle_deg: Real,
    pub sunlit_fraction: Real,
    /// 月面站心视半径
    pub moon_semidiameter_deg: Real,
    /// 天体与太阳的角距、方位差（天空亮度用）
    pub sun_separation_deg: Real,
    pub sun_azimuth_difference_deg: Real,
}

/// 一次月掩星（某地）。
#[derive(Clone, Copy, Debug)]
pub struct LunarOccultation {
    pub immersion: OccultationContact,
    pub emersion: OccultationContact,
    /// 站心角距最小的时刻与此时角距 / 月面视半径（< 1；越小越接近中心掩）
    pub closest_jd_tt: Real,
    pub closest_ratio: Real,
}

struct Ctx<'a, P> {
    eph: &'a P,
    body: OccultedBody,
    observer: &'a Observer,
    dt_days: Real,
    opts: &'a ApparentPipelineOptions,
}

fn ra_dec(v: [Real; 3]) -> (Real, Real) {
    let rho = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (v[1].atan2(v[0]), v[2].atan2(rho))
}

fn angle_between(a: [Real; 3], b: [Real; 3]) -> Real {
    let cr = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let n = (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt();
    n.atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
}

/// 自 `from` 方向看 `to` 方向的位置角（自北经东，rad）。
fn position_angle(from: [Real; 3], to: [Real; 3]) -> Real {
    let (a1, d1) = ra_dec(from);
    let (a2, d2) = ra_dec(to);
    let da = a2 - a1;
    (da.sin() * d2.cos()).atan2(d1.cos() * d2.sin() - d1.sin() * d2.cos() * da.cos()).wrap_to_2pi()
}

impl<P: EphemerisProvider> Ctx<'_, P> {
    fn tp(jd: Real) -> TimePoint {
        TimePoint::new(TimeScale::TT, jd)
    }

    fn moon_geo(&self, jd: Real) -> [Real; 3] {
        body_apparent_equatorial(self.eph, Body::Moon, Self::tp(jd), self.opts).to_meters()
    }

    fn body_geo(&self, jd: Real) -> [Real; 3] {
        match self.body {
            OccultedBody::Planet(p) => body_apparent_equatorial(self.eph, Body::Planet(p), Self::tp(jd), self.opts).to_meters(),
            OccultedBody::Star(s) => {
                let u = star_apparent_equatorial(self.eph, &s, Self::tp(jd), self.opts).to_meters();
                let n = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
                let d = real(STAR_DISTANCE_M);
                [u[0] / n * d, u[1] / n * d, u[2] / n * d]
            }
        }
    }

    fn gast(&self, jd: Real) -> PlaneAngle {
        gast(jd - self.dt_days, jd)
    }

    fn topo(&self, geo: [Real; 3], jd: Real, g: PlaneAngle) -> [Real; 3] {
        let p = Position::from_si_meters_in_frame(ReferenceFrame::TrueEquator(Epoch::new(jd)), geo[0], geo[1], geo[2]);
        topocentric(p, self.observer, g).to_meters()
    }

    /// 站心角距 − 月面视半径（rad），由给定的地心月、天体位置。
    fn gap(&self, moon: [Real; 3], body: [Real; 3], jd: Real, g: PlaneAngle) -> Real {
        let (m, b) = (self.topo(moon, jd, g), self.topo(body, jd, g));
        let d = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt();
        angle_between(m, b) - (K_PENUMBRA * EARTH_EQUATORIAL_RADIUS_M / d).asin()
    }

    fn gap_exact(&self, jd: Real) -> Real {
        self.gap(self.moon_geo(jd), self.body_geo(jd), jd, self.gast(jd))
    }

    fn contact(&self, jd: Real) -> OccultationContact {
        let g = self.gast(jd);
        let (mg, bg) = (self.moon_geo(jd), self.body_geo(jd));
        let sg = body_apparent_equatorial(self.eph, Body::Sun, Self::tp(jd), self.opts).to_meters();
        let (m, b, s) = (self.topo(mg, jd, g), self.topo(bg, jd, g), self.topo(sg, jd, g));
        let d = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt();
        let semi = (K_PENUMBRA * EARTH_EQUATORIAL_RADIUS_M / d).asin();
        let pa = position_angle(m, b);
        let pa_sun = position_angle(m, s);
        let diff = (pa - pa_sun).wrap_to_signed_pi().abs() * real(R2D);
        let to_sun = [s[0] - m[0], s[1] - m[1], s[2] - m[2]];
        let alpha = angle_between(to_sun, [-m[0], -m[1], -m[2]]);
        let hz = |v: [Real; 3]| {
            let p = Position::from_si_meters_in_frame(ReferenceFrame::TrueEquator(Epoch::new(jd)), v[0], v[1], v[2]);
            let h = horizontal(&p, self.observer, g);
            (h.altitude.rad() * real(R2D), h.azimuth.rad())
        };
        let alt = |v: [Real; 3]| hz(v).0;
        let daz = (hz(b).1 - hz(s).1).wrap_to_signed_pi().abs() * real(R2D);
        OccultationContact {
            jd_tt: jd,
            position_angle_deg: pa * real(R2D),
            bright_limb: diff < real(90.0),
            cusp_angle_deg: (real(90.0) - diff).abs(),
            moon_altitude_deg: alt(m),
            body_altitude_deg: alt(b),
            sun_altitude_deg: alt(s),
            moon_phase_angle_deg: alpha * real(R2D),
            sunlit_fraction: (real(1.0) + alpha.cos()) / real(2.0),
            moon_semidiameter_deg: semi * real(R2D),
            sun_separation_deg: angle_between(b, s) * real(R2D),
            sun_azimuth_difference_deg: daz,
        }
    }
}

/// [jd_start, jd_end]（TT）内某地所见的月掩某天体（不论地平上下、昼夜；可见性另判）。`delta_t` 为 TT − UT1。
/// 常用 `ApparentPipelineOptions::pipeline_default()`（月球施光行时）。
pub fn lunar_occultations<P: EphemerisProvider>(
    eph: &P,
    body: OccultedBody,
    observer: &Observer,
    delta_t: Duration,
    jd_start: Real,
    jd_end: Real,
    options: &ApparentPipelineOptions,
) -> Vec<LunarOccultation> {
    let ctx = Ctx { eph, body, observer, dt_days: delta_t.seconds() / real(86400.0), opts: options };
    let (t0, t1) = (jd_start - real(2.0), jd_end + real(2.0));
    let moon_tr = Track::build(t0, t1, |t| ctx.moon_geo(t));
    let body_tr = Track::build(t0, t1, |t| ctx.body_geo(t));
    // 粗筛：逐 10 分钟，插值位置（月插值误差 < 0.01°）
    let step = real(1.0 / 144.0);
    let n = ((jd_end - jd_start) / step).floor().as_f64() as usize;
    let rate = real(2.0 * core::f64::consts::PI * 1.002_737_811_911_354_5);
    let mut day_g: Option<(i64, Real, PlaneAngle)> = None;
    let mut coarse = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = jd_start + step * real(i as f64);
        let day = t.floor().as_f64() as i64;
        if day_g.map(|(d, _, _)| d) != Some(day) {
            let t_d = real(day as f64);
            day_g = Some((day, t_d, ctx.gast(t_d)));
        }
        let (_, t_d, g_d) = day_g.unwrap();
        let g = PlaneAngle::from_rad(g_d.rad() + rate * (t - t_d));
        coarse.push((t, ctx.gap(moon_tr.at(t), body_tr.at(t), t, g)));
    }
    let margin = real(0.15f64.to_radians());
    let mut out: Vec<LunarOccultation> = vec![];
    for i in 1..n {
        let (t, f) = coarse[i];
        if !(f < margin && f <= coarse[i - 1].1 && f < coarse[i + 1].1) {
            continue;
        }
        // 精算：最近时刻
        let (tc, fc) = minimize_golden(|x| ctx.gap_exact(x), t - step * real(2.0), t + step * real(2.0), seconds(1.0));
        if fc >= real(0.0) {
            continue;
        }
        // 入掩、出掩：月相对恒星约 0.5°/时，中心掩历时 ≤ 约 2 时；向两侧找到 gap > 0 的端点再二分
        let edge = |dir: f64| {
            let mut e = tc;
            for _ in 0..30 {
                e = e + real(dir * 0.01);
                if ctx.gap_exact(e) > real(0.0) {
                    return Some(e);
                }
            }
            None
        };
        let (Some(a), Some(b)) = (edge(-1.0), edge(1.0)) else { continue };
        let (Some(ti), Some(te)) = (bisect_root(|x| ctx.gap_exact(x), a, tc, seconds(0.1)), bisect_root(|x| ctx.gap_exact(x), tc, b, seconds(0.1))) else {
            continue;
        };
        if out.last().is_some_and(|o| (o.closest_jd_tt - tc).abs() < real(0.05)) {
            continue;
        }
        let ci = ctx.contact(ti);
        let semi = ci.moon_semidiameter_deg * real(core::f64::consts::PI / 180.0);
        out.push(LunarOccultation { immersion: ci, emersion: ctx.contact(te), closest_jd_tt: tc, closest_ratio: (fc + semi) / semi });
    }
    out
}

/// 掩星时刻天体的肉眼可见性。
#[derive(Clone, Copy, Debug)]
pub struct OccultationVisibility {
    /// 有效背景（nL）
    pub background_nl: Real,
    /// 极限星等（大气外 V，已减 `confidence_offset_mag`）
    pub limiting_magnitude: Real,
}

/// 可见性参数。
#[derive(Clone, Debug)]
pub struct OccultationVisibilityOptions {
    pub sky: SkyConditions,
    pub eye: Eye,
    /// Schaefer et al. 1992 式 46 的 ΔM + 0.16(6 − e)：计时观测所需高置信度的偏移（原文对望远镜计时拟合 ΔM = 1.2 等）。
    /// 缺省 0，即 50% 探测概率的「看得见」。
    pub confidence_offset_mag: Real,
}

impl Default for OccultationVisibilityOptions {
    fn default() -> Self {
        Self { sky: SkyConditions::default(), eye: Eye::default(), confidence_offset_mag: real(0.0) }
    }
}

/// 某次接触时天体的肉眼可见性（见模块文档）。`observer` 用于消光的海拔、纬度；`sun_ra_rad` 为太阳赤经（季节）；
/// `body_bv` 为天体色指数。
pub fn occultation_visibility(
    c: &OccultationContact,
    observer: &Observer,
    sun_ra_rad: Real,
    body_bv: Real,
    o: &OccultationVisibilityOptions,
) -> OccultationVisibility {
    let d2r = real(core::f64::consts::PI / 180.0);
    let z_body = (real(90.0) - c.body_altitude_deg) * d2r;
    let z_moon = (real(90.0) - c.moon_altitude_deg) * d2r;
    let (h_km, lat) = (observer.height.meters() / real(1000.0), observer.latitude.rad());
    let (k, dm) = o.sky.extinction.at(h_km, lat, sun_ra_rad, z_body);
    let ext_moon = (real(-0.4) * k * air_mass(z_moon) * real(core::f64::consts::LN_10)).exp();
    let dex = |x: Real| (x * real(core::f64::consts::LN_10)).exp();
    let i_moon_star = dex(real(-0.4) * (moon_magnitude(c.moon_phase_angle_deg) + real(16.57)));
    // 天空（不含月光）
    let g = SkyGeometry {
        z_deg: real(90.0) - c.body_altitude_deg,
        rho_sun_deg: c.sun_separation_deg,
        azimuth_diff_sun_deg: c.sun_azimuth_difference_deg,
        z_sun_deg: real(90.0) - c.sun_altitude_deg,
        moon: None,
    };
    let b_sky = sky_brightness_nl(&g, k, &o.sky);
    let s = c.sunlit_fraction.max(real(1e-4));
    let background = if c.bright_limb {
        // 式 7、43
        real(5.67e10) * i_moon_star * ext_moon / s + b_sky
    } else {
        // 地照（1993 式 13），月面亮度同样受消光
        let b_es = real(2.59) * dex(real(-0.4) * moon_magnitude(real(180.0) - c.moon_phase_angle_deg)) * ext_moon;
        // 式 11 有效角距
        let cusp = c.cusp_angle_deg;
        let cr = cusp * d2r;
        let theta = (real(1.0) - real(0.4) * (-cusp / real(30.0)).exp())
            * c.moon_semidiameter_deg
            * (cr.cos() * cr.cos() + (real(1.0) - s + cr.sin()) * (real(1.0) - s + cr.sin())).sqrt();
        let th2 = theta * theta;
        let b_atm = real(6.25e7) * i_moon_star / th2 * (ext_moon - ext_moon * ext_moon); // 式 10
        let b_eye = real(4.63e7) * ext_moon * i_moon_star / th2; // 式 13，肉眼 f_FOV = 1、M = 1
        b_es + b_sky + b_atm + b_eye
    };
    let lim = limiting_magnitude(background, dm, body_bv, &o.eye).limiting_magnitude - o.confidence_offset_mag;
    OccultationVisibility { background_nl: background, limiting_magnitude: lim }
}

/// 望远镜目视计时的月掩星极限星等：Schaefer, Bulder & Bourgeois 1992（*Icarus* 100, 60）式 1–46 的输入（全为原文 15 项及 ΔM）。
#[derive(Clone, Copy, Debug)]
pub struct TelescopeOccultationInput {
    /// S：月面亮面比例（0—1）
    pub sunlit_fraction: Real,
    /// C：尖角（度）；原文以 C < 0 表示亮边（图 1 的「bright limb」C = −90°）
    pub cusp_angle_deg: Real,
    /// Z：星（月）的天顶距（度）
    pub zenith_distance_deg: Real,
    /// H₀：太阳高度（度）
    pub sun_altitude_deg: Real,
    /// 星的色指数 B − V
    pub star_bv: Real,
    /// k：V 天顶消光（星等/大气质量）
    pub k: Real,
    /// B_city（nL）
    pub city_nl: Real,
    /// θ_s：天顶视宁度圆面（角秒）
    pub seeing_arcsec: Real,
    /// f_FOV：视场内月面比例
    pub f_fov: Real,
    /// D：口径（cm）
    pub aperture_cm: Real,
    /// M：放大率
    pub magnification: Real,
    /// T_tel：透过率
    pub transmission: Real,
    /// A：观测者年龄（岁）
    pub age_years: Real,
    /// R_s：Snellen 视力比
    pub snellen: Real,
    /// e：经验（0—10，常人 5）
    pub experience: Real,
    /// ΔM：计时所需高置信度的偏移（原文拟合 1.2）
    pub delta_m: Real,
    /// 月面视半径（度），缺省 0.259
    pub moon_semidiameter_deg: Real,
}

impl TelescopeOccultationInput {
    /// 原文 §2.F 的「标准」情形：S = 50%，C = 90°，Z = 0°，H₀ = −90°，B − V = 0.7，k = 0.3，B_city = 0，θ_s = 1″，f_FOV = 20%，
    /// D = 15.2 cm，M = 150，T = 80%，A = 30，R_s = 1，e = 5，ΔM = 1.2；原文结果 8.61 等。
    pub fn standard() -> Self {
        Self {
            sunlit_fraction: real(0.5),
            cusp_angle_deg: real(90.0),
            zenith_distance_deg: real(0.0),
            sun_altitude_deg: real(-90.0),
            star_bv: real(0.7),
            k: real(0.3),
            city_nl: real(0.0),
            seeing_arcsec: real(1.0),
            f_fov: real(0.2),
            aperture_cm: real(15.24),
            magnification: real(150.0),
            transmission: real(0.8),
            age_years: real(30.0),
            snellen: real(1.0),
            experience: real(5.0),
            delta_m: real(1.2),
            moon_semidiameter_deg: real(0.259),
        }
    }
}

/// 望远镜月掩星极限星等的结果与中间量（便于逐项核对）。
#[derive(Clone, Copy, Debug)]
pub struct TelescopeOccultationResult {
    pub limiting_magnitude: Real,
    /// 望远镜前的有效背景（nL，式 43/44）
    pub background_nl: Real,
    /// 眼感背景 B / F_B（nL，式 24）
    pub perceived_nl: Real,
    /// 眼瞳直径（cm，式 36）
    pub pupil_cm: Real,
    /// 临界视角（角秒，式 38–39）
    pub critical_visual_angle_arcsec: Real,
    pub f_i: Real,
    pub f_b: Real,
}

fn dexr(x: Real) -> Real {
    (x * real(core::f64::consts::LN_10)).exp()
}

/// 眼瞳直径（cm）：式 36、37。`b` 为眼感背景（nL），`glare_fc` 为进入眼的眩光源照度，`m_theta_deg` 为 Mθ（度）。
fn pupil_cm(age: Real, b: Real, glare_fc: Real, m_theta_deg: Real) -> Real {
    let d0 = real(0.7) * (real(-0.5) * (age / real(100.0)) * (age / real(100.0))).exp();
    let q = real(1e-6) * b + real(0.172) * glare_fc / (m_theta_deg * real(1.122f64.ln())).exp();
    d0 * (real(-0.16) * (q.max(real(1e-30)).ln() * real(0.4)).exp()).exp()
}

/// 望远镜目视月掩星极限星等（Schaefer, Bulder & Bourgeois 1992）。实现说明：
/// - 式 30 原文印作 F_p = max(1, [D_s/(M D_e)]²)，如此恒为 1；按 Schaefer 1990、1991 的 [D/(M D_e)]²（出瞳/眼瞳）实现；
/// - 式 33 的 0.026、12.44 为毫米单位（同 1990 年原文），瞳孔与口径换算为毫米代入；
/// - 式 42b 原文印作「log(B) > 3.17」，应为「< 3.17」（暗视）；
/// - 式 36 的眩光照度取 f_FOV·I_月（大气内），角距取式 11 的有效 θ；亮边无外部眩光源；
/// - 天空（式 17–22）用 [`sky_brightness_nl`] 的 1993 模型（夜天光 180 nL），太阳角距取 180° − 月相角；
/// - 式 24、23 按 Schaefer 1990 式 14、15 的约定「实际量 = 眼感量 × F」：眼感背景 = B / F_B，大气外所需照度 I* = I·F_I
///   （1992 文式 24 记作 B_eff = B F_B，变量含义易误读；按此约定，放大率使面亮度按 M² 稀释、口径按 (D/D_s)² 聚光）；
/// - B_eff、D_e、θ_CVA 相互依赖，迭代 8 次。
pub fn telescope_occultation_limit(inp: &TelescopeOccultationInput) -> TelescopeOccultationResult {
    use super::visibility::{ExtinctionModel, SkyConditions};
    let d2r = real(core::f64::consts::PI / 180.0);
    let s = inp.sunlit_fraction.max(real(1e-4)).min(real(1.0));
    let alpha = ((real(2.0) * s - real(1.0)).max(real(-1.0)).min(real(1.0))).as_f64().acos().to_degrees(); // 式 1
    let alpha = real(alpha);
    let x = air_mass(inp.zenith_distance_deg * d2r); // 式 5
    let ext = dexr(real(-0.4) * inp.k * x);
    let i_moon_star = dexr(real(-0.4) * (moon_magnitude(alpha) + real(16.57))); // 式 2、3
    let bright = inp.cusp_angle_deg < real(0.0);
    let cusp = inp.cusp_angle_deg.abs();
    // 天空（不含月光）
    let sky = SkyConditions { extinction: ExtinctionModel::Fixed { k: inp.k }, city_nl: inp.city_nl, ..SkyConditions::schaefer1993(real(0.5)) };
    let g = SkyGeometry {
        z_deg: inp.zenith_distance_deg,
        rho_sun_deg: real(180.0) - alpha,
        azimuth_diff_sun_deg: real(180.0) - alpha,
        z_sun_deg: real(90.0) - inp.sun_altitude_deg,
        moon: None,
    };
    let b_sky = sky_brightness_nl(&g, inp.k, &sky);
    // 有效角距（式 11）
    let cr = cusp * d2r;
    let theta = (real(1.0) - real(0.4) * (-cusp / real(30.0)).exp())
        * inp.moon_semidiameter_deg
        * (cr.cos() * cr.cos() + (real(1.0) - s + cr.sin()) * (real(1.0) - s + cr.sin())).sqrt();
    let th2 = theta * theta;
    let d = inp.aperture_cm;
    let (b_total, glare_fc) = if bright {
        (real(5.67e10) * i_moon_star * ext / s + b_sky, real(0.0)) // 式 7、43
    } else {
        let b_es = real(2.59) * dexr(real(-0.4) * moon_magnitude(real(180.0) - alpha)) * ext; // 式 8、9
        let b_atm = real(6.25e7) * i_moon_star * (ext - ext * ext) / th2; // 式 10
        let b_eye = real(4.63e7) * ext * inp.f_fov * i_moon_star / th2; // 式 13
        let b_dif = real(1.13e6) * ext * i_moon_star / (d * th2 * theta); // 式 14
        let b_mir = real(2.60e8) * ext * i_moon_star * (-(theta / real(0.4)) * (theta / real(0.4))).exp(); // 式 15
        (b_es + b_sky + b_atm + b_eye + b_dif + b_mir, inp.f_fov * i_moon_star * ext) // 式 44
    };
    let m = inp.magnification;
    let zeta = (real(2.89) * inp.seeing_arcsec * inp.seeing_arcsec + (real(45.4) / d) * (real(45.4) / d)).sqrt(); // 式 40
    let fc = |bv: Real, phot: bool| if phot { real(1.0) } else { dexr(real(-0.4) * (real(1.0) - bv / real(2.0))) }; // 式 34
    let mut perceived = b_total;
    let mut out = None;
    for _ in 0..8 {
        let phot = perceived.log10() > real(3.17);
        let de = pupil_cm(inp.age_years, perceived, glare_fc, m * theta);
        let ds = pupil_cm(real(25.0), perceived, glare_fc, m * theta);
        let fb_ = real(1.41); // 式 27
        let fe = if phot { dexr(real(0.4) * inp.k * x) } else { dexr(real(0.48) * inp.k * x) }; // 式 28
        let ft = real(1.0) / inp.transmission; // 式 29
        let fp = real(1.0).max((d / (m * de)) * (d / (m * de))); // 式 30（见说明）
        let fa = (ds / d) * (ds / d); // 式 31
        let cva = if phot {
            real(42.0) * dexr(real(8.28) * (real(-0.29) * perceived.ln()).exp())
        } else {
            real(900.0).min(real(380.0) * dexr(real(0.3) * (real(-0.29) * perceived.ln()).exp()))
        } / inp.snellen; // 式 38、39
        let fr = (real(1.0) + real(0.03) * (m * zeta / cva) * (m * zeta / cva)) / (inp.snellen * inp.snellen); // 式 32
        let (ds_mm, d_mm) = (ds * real(10.0), d * real(10.0));
        let fsc = if phot {
            real(1.0).min(
                (d_mm / (ds_mm * m)) * (d_mm / (ds_mm * m)) * (real(1.0) - (real(-0.026) * ds_mm * ds_mm).exp())
                    / (real(1.0) - (real(-0.026) * (d_mm / m) * (d_mm / m)).exp()),
            )
        } else {
            let q = |v: Real| { let r = v / real(12.44); r * r * r * r };
            real(1.0).min((real(1.0) - q(ds_mm)) / (real(1.0) - q(ds_mm / m)))
        }; // 式 33
        let fm = m * m; // 式 35
        let f_b = fb_ * ft * fp * fa * fm * fsc * fc(real(0.7), phot); // 式 26
        let f_i = fb_ * fe * ft * fp * fa * fr * fsc * fc(inp.star_bv, phot); // 式 25
        perceived = b_total / f_b; // 式 24：按 Schaefer 1990 式 15 的约定「实际 = 眼感 × F」，眼感背景 = B / F_B
        let phot2 = perceived.log10() > real(3.17);
        let (c, kk) = if phot2 { (dexr(real(-8.35)), dexr(real(-5.90))) } else { (dexr(real(-9.80)), dexr(real(-1.90))) }; // 式 42
        let one = real(1.0) + (kk * perceived).sqrt();
        let i_th = c * one * one; // 式 41
        let i_star = i_th * f_i; // 式 23
        let mag = real(-16.57) - real(2.5) * i_star.log10(); // 式 45
        let lim = mag + real(0.16) * (inp.experience - real(6.0)) - inp.delta_m; // 式 46
        out = Some(TelescopeOccultationResult {
            limiting_magnitude: lim,
            background_nl: b_total,
            perceived_nl: perceived,
            pupil_cm: de,
            critical_visual_angle_arcsec: cva,
            f_i,
            f_b,
        });
    }
    out.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1992 式 7 与正文：满月（S = 1、无消光）月面亮度 1.65×10⁹ nL；地照 1993 式 13 与 1992 式 8、9（系数 78）一致；
    /// 式 11 有效角距：上弦月赤道处比月心距小约 2–3%，距尖点 5° 处约小 34%（原文「3% … 38%」为对积分的近似说明）。
    #[test]
    fn moon_surface_and_earthshine_constants() {
        let i_full = 10f64.powf(-0.4 * (-12.73 + 16.57));
        assert!((5.67e10 * i_full - 1.65e9).abs() < 0.01e9, "{}", 5.67e10 * i_full);
        // 1992：B_ES = 1.65e9 × 78 × I*_月(α′) / 1.17e4；1993 式 13：2.59 × 10^{−0.4 m(α′)}
        for m in [-12.0, -10.0, -8.0] {
            let i = 10f64.powf(-0.4 * (m + 16.57));
            let b92 = 1.65e9 * 78.0 * i / 1.17e4;
            let b93 = 2.59 * 10f64.powf(-0.4 * m);
            assert!((b92 / b93 - 1.0).abs() < 0.01, "{b92} {b93}");
        }
        // 式 9：(6378/1738)²(0.39/0.067) ≈ 78
        let r: f64 = (6378.0f64 / 1738.0).powi(2) * 0.39 / 0.067;
        assert!((r - 78.0).abs() < 1.0, "{r}");
        let f = |c: f64| 1.0 - 0.4 * (-c / 30.0f64).exp();
        assert!((f(90.0) - 0.98).abs() < 0.01 && (f(5.0) - 0.66).abs() < 0.01);
    }

    fn contact(bright: bool, alpha: f64, cusp: f64) -> OccultationContact {
        OccultationContact {
            jd_tt: real(2451545.0),
            position_angle_deg: real(0.0),
            bright_limb: bright,
            cusp_angle_deg: real(cusp),
            moon_altitude_deg: real(40.0),
            body_altitude_deg: real(40.0),
            sun_altitude_deg: real(-40.0),
            moon_phase_angle_deg: real(alpha),
            sunlit_fraction: real((1.0 + alpha.to_radians().cos()) / 2.0),
            moon_semidiameter_deg: real(0.259),
            sun_separation_deg: real(180.0 - alpha),
            sun_azimuth_difference_deg: real(180.0),
        }
    }

    /// 肉眼可见性的定性检验（原文 §2.F：亮边与白昼掩星比夜间暗边难见得多）：
    /// 夜间暗边极限明显暗于亮边；亮面越小暗边越易见；满月亮边背景 ≈ 满月月面（1.65×10⁹ nL × 消光），肉眼只见 −4 等以上。
    #[test]
    fn naked_eye_occultation_visibility_ordering() {
        use crate::astronomy::geodesy::Observer;
        use crate::quantity::length::Length;
        use crate::quantity::unit::LengthUnit;
        let obs = Observer::new(PlaneAngle::from_deg(real(116.4)), PlaneAngle::from_deg(real(39.9)), Length::from_value(real(50.0), LengthUnit::Meter));
        let o = OccultationVisibilityOptions::default();
        let lim = |c: &OccultationContact| occultation_visibility(c, &obs, real(1.0), real(0.7), &o).limiting_magnitude.as_f64();
        let (dark_crescent, dark_gibbous) = (lim(&contact(false, 120.0, 60.0)), lim(&contact(false, 40.0, 60.0)));
        let bright_crescent = lim(&contact(true, 120.0, 60.0));
        let bright_full = lim(&contact(true, 0.0, 60.0));
        assert!(dark_crescent > bright_crescent + 1.0, "{dark_crescent} {bright_crescent}");
        assert!(dark_crescent > dark_gibbous, "{dark_crescent} {dark_gibbous}");
        assert!(bright_full < -3.5 && bright_full > -5.0, "{bright_full}");
    }
}

#[cfg(test)]
mod telescope_tests {
    use super::*;

    /// Schaefer, Bulder & Bourgeois 1992 的望远镜模型对照：
    /// - 标准情形（§2.F 原文明给）8.61 等；
    /// - 图 1 各曲线在 S = 50% 处的值（读图，约 ±0.2 等）；
    /// - §4.A Povenmire 经验规则四例的模型值（原文：M = 100、Z = 45°、θ_s = 2″，其余同标准；未写明的 S、H₀ 为本项目假设，见表注）。
    #[test]
    fn telescope_model_vs_schaefer1992() {
        let st = TelescopeOccultationInput::standard();
        let pov = TelescopeOccultationInput { magnification: real(100.0), zenith_distance_deg: real(45.0), seeing_arcsec: real(2.0), ..st };
        let cases: Vec<(&str, TelescopeOccultationInput, f64, &str)> = vec![
            ("标准情形", st, 8.61, "§2.F 原文"),
            ("图 1 亮边（C = −90°，S = 50%）", TelescopeOccultationInput { cusp_angle_deg: real(-90.0), ..st }, 5.3, "读图约值"),
            ("图 1 白昼（H₀ = 10°，S = 50%）", TelescopeOccultationInput { sun_altitude_deg: real(10.0), ..st }, 5.1, "读图约值"),
            ("图 1 视宁度差（θ_s = 3″，S = 50%）", TelescopeOccultationInput { seeing_arcsec: real(3.0), ..st }, 8.3, "读图约值"),
            ("图 1 视力差（R_s = 0.5，S = 50%）", TelescopeOccultationInput { snellen: real(0.5), ..st }, 7.9, "读图约值"),
            ("图 1 标准 S = 10%", TelescopeOccultationInput { sunlit_fraction: real(0.1), ..st }, 8.9, "读图约值"),
            ("图 1 标准 S = 90%", TelescopeOccultationInput { sunlit_fraction: real(0.9), ..st }, 7.8, "读图约值"),
            (
                "Povenmire「七之律」：S = 70%、C = 7°、D = 7 in，暗边",
                TelescopeOccultationInput { sunlit_fraction: real(0.7), cusp_angle_deg: real(7.0), aperture_cm: real(17.78), ..pov },
                7.6,
                "§4.A 模型值",
            ),
            ("Povenmire 满月亮边（S = 100%）", TelescopeOccultationInput { sunlit_fraction: real(1.0), cusp_angle_deg: real(-90.0), ..pov }, 3.1, "§4.A 模型值"),
            (
                "Povenmire 蛾眉月亮边（假设 S = 20%）",
                TelescopeOccultationInput { sunlit_fraction: real(0.2), cusp_angle_deg: real(-90.0), ..pov },
                5.5,
                "§4.A 模型值；S 为假设",
            ),
            ("Povenmire 白昼掩星（假设 H₀ = 10°、暗边）", TelescopeOccultationInput { sun_altitude_deg: real(10.0), ..pov }, 4.1, "§4.A 模型值；H₀ 为假设"),
        ];
        let mut tab = crate::test_util::CheckTable::new("schaefer1992_telescope_model", &["情形", "原文", "原文出处", "本实现", "差"]);
        tab.meta("Schaefer, Bulder & Bourgeois 1992 望远镜目视月掩星极限星等（ΔM = 1.2）：标准情形、图 1 读图值、§4.A Povenmire 四例");
        let mut worst_std = 0.0f64;
        for (name, inp, want, src) in cases {
            let got = telescope_occultation_limit(&inp).limiting_magnitude.as_f64();
            println!("{name}: 原文 {want}，本实现 {got:.2}");
            tab.row(vec![name.into(), format!("{want}"), src.into(), format!("{got:.2}"), format!("{:+.2}", got - want)]);
            if name == "标准情形" {
                worst_std = (got - want).abs();
            }
        }
        tab.write();
        // 作者自述模型精度约 1/3 等
        assert!(worst_std < 0.33, "标准情形差 {worst_std}");
    }
}
