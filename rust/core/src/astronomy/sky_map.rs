//! 星图用的天空快照（天文域派生层）：某时刻全部星表恒星、日月五星的地心视位置（当日真赤道与真春分点），黄道极方向，
//! 以及由快照中距星位置构成的二十八宿框架。只给天体位置与星等，投影、连线、星名等绘图事宜在前端。
//!
//! 恒星用 [`stars_apparent_equatorial`] 批量求（与逐颗计算一致，差 < 1e-10″）；日月行星走统一视位置管线
//! [`body_apparent_equatorial`]。星等：行星 Mallama & Hilton 2018（[`planet_magnitude`]），月亮 Krisciunas & Schaefer 1991
//! （[`moon_magnitude`]，按月相角），太阳取 −26.74。
//!
//! 站在地面看的视图另要两项：[`SkySnapshot::topocentric_places`]（日月五星的站心位置，月亮地平视差约 1°）与
//! [`refraction_table`]（真高度 → 折射量）。恒星的站心差可略，不算。天色与星的隐现由 [`SkySnapshot::sky_light`] 给出：
//! 全天网格上的天空亮度与极限星等（Schaefer 模型，见 [`crate::astronomy::phenomena::visibility`]），日食时太阳贡献按未遮比例减小。
//! 模型依据与检验见 doc/astronomy/sky-view-3d.md。

use crate::astronomy::apparent::{
    body_apparent_equatorial, planet_magnitude, right_ascension_declination, stars_apparent_equatorial, topocentric, ApparentPipelineOptions,
};
use crate::astronomy::correction::{refraction_from_geometric, Atmosphere, RefractionModel};
use crate::astronomy::geodesy::Observer;
use crate::astronomy::phenomena::solar_eclipse::{sun_radius_m, K_PENUMBRA};
use crate::astronomy::phenomena::visibility::{air_mass, glare_nl, limiting_magnitude, moon_magnitude, sky_brightness_nl, Eye, SkyConditions, SkyGeometry};
use crate::astronomy::frame::nutation::nutation_for_apparent;
use crate::astronomy::frame::precession::{mean_obliquity_for, precession_matrix};
use crate::astronomy::pipeline::{Body, EphemerisProvider, Planet};
use crate::astronomy::sky_culture::chinese::{LunarMansionSystem, MansionFrame};
use crate::astronomy::stars::StarCatalog;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::position::Position;

/// 太阳 V 星等（Willmer 2018）。
pub const SUN_MAGNITUDE_V: f64 = -26.74;

/// 一颗恒星在快照中的位置。
#[derive(Clone, Copy, Debug)]
pub struct StarPlace {
    pub hip: u32,
    pub ra: PlaneAngle,
    pub dec: PlaneAngle,
    pub vmag: Real,
    pub bv: Option<Real>,
}

/// 日、月或行星在快照中的位置。
#[derive(Clone, Copy, Debug)]
pub struct BodyPlace {
    pub body: Body,
    pub ra: PlaneAngle,
    pub dec: PlaneAngle,
    /// 地心视位置（当日真赤道架）
    pub position: Position,
    /// V 星等；模型不适用（如行星相角超出拟合范围）时为 None
    pub magnitude: Option<Real>,
    /// 月亮、行星的相角（日—天体—地夹角）；太阳为 None
    pub phase_angle: Option<PlaneAngle>,
}

/// 日、月或行星的站心位置。
#[derive(Clone, Copy, Debug)]
pub struct TopocentricPlace {
    pub ra: PlaneAngle,
    pub dec: PlaneAngle,
    /// 站心视半径，常数同日月食模块（太阳 959.63″@1 AU，月亮 k = 0.2725076）。行星不给，肉眼看是点。
    pub semidiameter: Option<PlaneAngle>,
    /// 站心距离 [m]
    pub distance_m: Real,
}

/// 天空亮度网格的方位格数（0°—360°，首尾重合）。
pub const SKY_GRID_AZ: usize = 25;
/// 天空亮度网格的高度格数：高度 = 90°·(i/(n−1))²，地平附近密。
pub const SKY_GRID_ALT: usize = 13;
/// 网格的色指数取值（1990：无月夜空色温对应 B−V = 0.7；极限星等对恒星颜色的依赖很弱）。
const SKY_GRID_SOURCE_BV: f64 = 0.7;

/// 全食时仍照进本影的散射光，相对未食天光的比例。实测天顶约 13 mag/arcsec²（lg B ≈ 5.3 nL，见 doc/astronomy/sky-view-3d.md），
/// 比白昼暗约 4 个量级。只是量级，实际随本影大小、云和高度变化。
const ECLIPSE_RESIDUAL_LIGHT: f64 = 1e-4;

/// J2000 赤道 → 银道（Hipparcos 卷 1 §1.5.3 的 A_G′），NASA Deep Star Maps 所用的转换。
const J2000_TO_GALACTIC: [[f64; 3]; 3] = [
    [-0.0548755604, -0.8734370902, -0.4838350155],
    [0.4941094279, -0.4448296300, 0.7469822445],
    [-0.8676661490, -0.1980763734, 0.4559837762],
];

/// 当日赤道 → 银道的矩阵：去岁差后转银道。显示银河用，不计章动（≤20″，小于贴图一个像素）。
pub fn true_equator_to_galactic(t: TimePoint) -> [[Real; 3]; 3] {
    let tc = (t.to_scale(TimeScale::TT).jd - real(2451545.0)) / real(36525.0);
    let p = precession_matrix(tc);
    let mut out = [[real(0.0); 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            // 岁差矩阵 J2000 → 当日；其转置回到 J2000
            for k in 0..3 {
                out[r][c] = out[r][c] + real(J2000_TO_GALACTIC[r][k]) * p[c][k];
            }
        }
    }
    out
}

/// 全天的天空亮度与极限星等。两个数组都按「先高度后方位」排，第 i 行第 j 列在 `i * SKY_GRID_AZ + j`。
#[derive(Clone, Debug)]
pub struct SkyLight {
    pub log10_brightness_nl: Vec<Real>,
    pub limiting_magnitude: Vec<Real>,
    /// 日面未被月亮遮住的面积比例（1 为无食）。
    pub sun_visible_fraction: Real,
    /// 月亮眼内眩光系数（nL·度²）：离月 θ 度处眩光亮度 = 此值 / max(θ, [`MOON_GLARE_MIN_DEG`])²。月在地平下为 0。
    pub moon_glare_nl_deg2: Real,
    /// 极限星等随 lg 亮度的变化，取样 lg B = i·[`LIMIT_CURVE_STEP`]，无消光。前端用它把眩光折算到网格的极限星等上。
    pub limit_curve: Vec<Real>,
    /// 日面中心亮度的 10⁻⁶（nL），日冕公式（Baumbach 1937）的单位。
    pub corona_unit_nl: Real,
}

/// 太阳 V 星等、视半径 → 日面平均面亮度（nL），再按临边昏暗换算到日面中心，取其 10⁻⁶。
/// mag/arcsec² → nL：Garstang 式 B = 34.08·exp(20.7233 − 0.92104 m)。中心/平均 = 1/0.805（550 nm 临边昏暗）。
fn corona_unit_nl(semidiameter: PlaneAngle) -> Real {
    let arcsec = semidiameter.rad() * real(180.0 * 3600.0) / Real::pi();
    let area = Real::pi() * arcsec * arcsec;
    let m = real(SUN_MAGNITUDE_V) + real(2.5) * area.log10();
    let mean = real(34.08) * (real(20.7233) - real(0.92104) * m).exp();
    mean / real(0.805) * real(1e-6)
}

/// 眩光的最小角距 [°]：月亮视半径，同偕日（heliacal）。
pub const MOON_GLARE_MIN_DEG: f64 = 0.26;
/// [`SkyLight::limit_curve`] 的取样间隔与点数。
pub const LIMIT_CURVE_STEP: f64 = 0.25;
pub const LIMIT_CURVE_SAMPLES: usize = 41;

fn dex10(x: Real) -> Real {
    (x * real(10f64.ln())).exp()
}

/// 半径 rs、rm 的两圆盘（小角近似为平面）中心距 d 时，rs 圆未被 rm 圆遮住的面积比例。
pub fn disc_visible_fraction(d: Real, rs: Real, rm: Real) -> Real {
    let one = real(1.0);
    if d >= rs + rm {
        return one;
    }
    if d <= (rs - rm).abs() {
        return if rm >= rs { real(0.0) } else { one - (rm * rm) / (rs * rs) };
    }
    let acos = |c: Real| {
        let c = c.max(-one).min(one);
        (one - c * c).max(real(0.0)).sqrt().atan2(c)
    };
    let a = acos((d * d + rs * rs - rm * rm) / (real(2.0) * d * rs));
    let b = acos((d * d + rm * rm - rs * rs) / (real(2.0) * d * rm));
    let k = (-d + rs + rm) * (d + rs - rm) * (d - rs + rm) * (d + rs + rm);
    let lens = rs * rs * a + rm * rm * b - k.max(real(0.0)).sqrt() / real(2.0);
    one - lens / (Real::pi() * rs * rs)
}

/// 天空快照。
#[derive(Clone, Debug)]
pub struct SkySnapshot {
    /// TT 时刻
    pub t: TimePoint,
    pub stars: Vec<StarPlace>,
    pub bodies: Vec<BodyPlace>,
    /// 当日真黄道之北极在当日真赤道架中的赤经、赤纬（前端据此画黄道）
    pub ecliptic_pole: (PlaneAngle, PlaneAngle),
}

/// 快照中显示的日月五星（肉眼行星）。
pub const SNAPSHOT_BODIES: [Body; 7] = [
    Body::Sun,
    Body::Moon,
    Body::Planet(Planet::Mercury),
    Body::Planet(Planet::Venus),
    Body::Planet(Planet::Mars),
    Body::Planet(Planet::Jupiter),
    Body::Planet(Planet::Saturn),
];

fn unit(p: [Real; 3]) -> [Real; 3] {
    let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    [p[0] / n, p[1] / n, p[2] / n]
}

fn angle_between(a: [Real; 3], b: [Real; 3]) -> PlaneAngle {
    let (a, b) = (unit(a), unit(b));
    let c = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    PlaneAngle::from_rad((c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]))
}

/// 求天空快照。`eph` 须提供日、月与五星（如 [`crate::astronomy::pipeline::Vsop87Elp`]）。
pub fn sky_snapshot<P: EphemerisProvider>(eph: &P, catalog: &StarCatalog, t: TimePoint, options: &ApparentPipelineOptions) -> SkySnapshot {
    let t = t.to_scale(TimeScale::TT);
    let stars = stars_apparent_equatorial(eph, &catalog.stars, t, options)
        .iter()
        .zip(&catalog.stars)
        .map(|(p, s)| {
            let (ra, dec) = right_ascension_declination(p);
            StarPlace { hip: s.hip, ra, dec, vmag: s.vmag, bv: s.bv }
        })
        .collect();

    let sun = body_apparent_equatorial(eph, Body::Sun, t, options).to_meters();
    let bodies = SNAPSHOT_BODIES
        .iter()
        .map(|&body| {
            let p = body_apparent_equatorial(eph, body, t, options);
            let (ra, dec) = right_ascension_declination(&p);
            let v = p.to_meters();
            // 相角：天体处看日与看地之夹角
            let phase = || angle_between([sun[0] - v[0], sun[1] - v[1], sun[2] - v[2]], [-v[0], -v[1], -v[2]]);
            let (magnitude, phase_angle) = match body {
                Body::Sun => (Some(real(SUN_MAGNITUDE_V)), None),
                Body::Moon => {
                    let a = phase();
                    (Some(moon_magnitude(a.rad() * real(180.0) / Real::pi())), Some(a))
                }
                Body::Planet(pl) => (planet_magnitude(eph, pl, t), Some(phase())),
            };
            BodyPlace { body, ra, dec, position: p, magnitude, phase_angle }
        })
        .collect();

    // 黄道极：当日真赤道架以真春分点为 x 轴，黄道面即赤道面绕 x 轴转过真黄赤交角 ε = ε_A + Δε，故黄道极严格为
    // (0, −sin ε, cos ε)，即赤经 270°、赤纬 90° − ε。章动取管线同一来源（nutation_for_apparent）。
    let tc = (t.jd - real(2451545.0)) / real(36525.0);
    let eps = mean_obliquity_for(tc, options.effective_precession_model()).rad() + nutation_for_apparent(tc).1.rad();
    let ecliptic_pole = (PlaneAngle::from_deg(real(270.0)), PlaneAngle::from_rad(Real::pi() / real(2.0) - eps));

    SkySnapshot { t, stars, bodies, ecliptic_pole }
}

/// 各真高度（度）处的折射量（度）。前端显示用的对照表：模型与大气由调用方给，地平以下由
/// [`refraction_from_geometric`] 保证连续。
pub fn refraction_table(altitudes_deg: &[Real], model: RefractionModel, atmosphere: &Atmosphere) -> Vec<Real> {
    altitudes_deg
        .iter()
        .map(|&h| refraction_from_geometric(PlaneAngle::from_deg(h), model, atmosphere).rad() * real(180.0) / Real::pi())
        .collect()
}

impl SkySnapshot {
    /// 日月五星的站心位置，与 `bodies` 同序。`gast` 须与快照同一时刻。
    pub fn topocentric_places(&self, observer: &Observer, gast: PlaneAngle) -> Vec<TopocentricPlace> {
        let (a, _) = observer.ellipsoid.parameters();
        self.bodies
            .iter()
            .map(|b| {
                let p = topocentric(b.position, observer, gast);
                let (ra, dec) = right_ascension_declination(&p);
                let [x, y, z] = p.to_meters();
                let d = (x * x + y * y + z * z).sqrt();
                let semidiameter = match b.body {
                    Body::Sun => Some(PlaneAngle::from_rad((sun_radius_m() / d).asin())),
                    Body::Moon => Some(PlaneAngle::from_rad((K_PENUMBRA * a / d).asin())),
                    Body::Planet(_) => None,
                };
                TopocentricPlace { ra, dec, semidiameter, distance_m: d }
            })
            .collect()
    }

    /// 全天天空亮度与极限星等。`topo` 取自 [`Self::topocentric_places`]；太阳、月亮的天顶距按视高度。
    pub fn sky_light(&self, topo: &[TopocentricPlace], observer: &Observer, gast: PlaneAngle, atmosphere: &Atmosphere, sky: &SkyConditions, eye: &Eye) -> SkyLight {
        let d2r = Real::pi() / real(180.0);
        let lat = observer.latitude.rad();
        // (方位, 真高度)，弧度；方位自北经东
        let horizon = |p: &TopocentricPlace| {
            let h = gast.rad() + observer.longitude.rad() - p.ra.rad();
            let d = p.dec.rad();
            let alt = (lat.sin() * d.sin() + lat.cos() * d.cos() * h.cos()).asin();
            let az = (-(d.cos()) * h.sin()).atan2(d.sin() * lat.cos() - d.cos() * h.cos() * lat.sin());
            (az, alt)
        };
        let seen = |alt: Real| alt + refraction_from_geometric(PlaneAngle::from_rad(alt), RefractionModel::RayTrace, atmosphere).rad();
        let dir = |az: Real, alt: Real| [alt.cos() * az.sin(), alt.cos() * az.cos(), alt.sin()];
        let sep = |a: [Real; 3], b: [Real; 3]| angle_between(a, b).rad();

        let find = |want: fn(&Body) -> bool| self.bodies.iter().position(|b| want(&b.body)).expect("快照含日月");
        let (si, mi) = (find(|b| matches!(b, Body::Sun)), find(|b| matches!(b, Body::Moon)));
        let (s_az, s_true) = horizon(&topo[si]);
        let (m_az, m_true) = horizon(&topo[mi]);
        let (s_alt, m_alt) = (seen(s_true), seen(m_true));
        let (s_dir, m_dir) = (dir(s_az, s_alt), dir(m_az, m_alt));

        // 日食只看几何重叠，用真方向，免得折射把两圆拉开
        let fraction = match (topo[si].semidiameter, topo[mi].semidiameter) {
            (Some(rs), Some(rm)) => disc_visible_fraction(sep(dir(s_az, s_true), dir(m_az, m_true)), rs.rad(), rm.rad()),
            _ => real(1.0),
        };
        let moon_mag = self.bodies[mi].magnitude;
        let height_km0 = observer.height.meters() / real(1000.0);
        let moon_glare_nl_deg2 = match moon_mag {
            Some(m) if m_alt > real(0.0) => {
                let z = real(90.0) * d2r - m_alt;
                let (k, _) = sky.extinction.at(height_km0, lat, topo[si].ra.rad(), z);
                glare_nl(m, real(1.0), k * air_mass(z)).eye
            }
            _ => real(0.0),
        };
        let limit_curve = (0..LIMIT_CURVE_SAMPLES)
            .map(|i| limiting_magnitude(dex10(real(i as f64 * LIMIT_CURVE_STEP)), real(0.0), real(SKY_GRID_SOURCE_BV), eye).limiting_magnitude)
            .collect();
        let height_km = observer.height.meters() / real(1000.0);
        let bv = real(SKY_GRID_SOURCE_BV);
        let n = SKY_GRID_AZ * SKY_GRID_ALT;
        let (mut log_b, mut lim) = (Vec::with_capacity(n), Vec::with_capacity(n));
        for i in 0..SKY_GRID_ALT {
            let t = real(i as f64 / (SKY_GRID_ALT - 1) as f64);
            let alt = t * t * real(90.0) * d2r;
            for j in 0..SKY_GRID_AZ {
                let az = real(j as f64 / (SKY_GRID_AZ - 1) as f64 * 360.0) * d2r;
                let v = dir(az, alt);
                let z_deg = real(90.0) - alt / d2r;
                let (k, dm) = sky.extinction.at(height_km, lat, topo[si].ra.rad(), real(90.0) * d2r - alt);
                let moon = match moon_mag {
                    Some(m) if m_alt > real(0.0) => Some((sep(v, m_dir) / d2r, real(90.0) - m_alt / d2r, m)),
                    _ => None,
                };
                let geom = |z_sun: Real| SkyGeometry {
                    z_deg,
                    rho_sun_deg: sep(v, s_dir) / d2r,
                    azimuth_diff_sun_deg: (PlaneAngle::from_rad(az - s_az).wrap_to_signed_pi().rad() / d2r).abs(),
                    z_sun_deg: z_sun,
                    moon,
                };
                // 太阳的贡献 = 有日 − 无日（太阳深在地平下），按未遮比例计入
                let without = sky_brightness_nl(&geom(real(135.0)), k, sky);
                let with = sky_brightness_nl(&geom(real(90.0) - s_alt / d2r), k, sky);
                let b = without + (with - without).max(real(0.0)) * fraction.max(real(ECLIPSE_RESIDUAL_LIGHT));
                log_b.push(b.log10());
                lim.push(limiting_magnitude(b, dm, bv, eye).limiting_magnitude);
            }
        }
        SkyLight { log10_brightness_nl: log_b, limiting_magnitude: lim, sun_visible_fraction: fraction, moon_glare_nl_deg2, limit_curve, corona_unit_nl: corona_unit_nl(topo[si].semidiameter.expect("太阳有视半径")) }
    }

    pub fn star(&self, hip: u32) -> Option<&StarPlace> {
        self.stars.binary_search_by_key(&hip, |s| s.hip).ok().map(|i| &self.stars[i])
    }

    /// 由快照中的距星位置构成二十八宿框架（宿度、入宿度、去极度用）；缺距星返回 None。
    pub fn mansion_frame(&self, system: &LunarMansionSystem) -> Option<MansionFrame> {
        let mut ra = [real(0.0); 28];
        let mut dec = [real(0.0); 28];
        for (i, hip) in system.determinatives.iter().enumerate() {
            let s = self.star(*hip)?;
            ra[i] = s.ra.rad();
            dec[i] = s.dec.rad();
        }
        Some(MansionFrame { ra, dec })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
    use crate::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
    use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
    use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
    use crate::astronomy::pipeline::{Vsop87Elp, Vsop87Planets};
    use crate::astronomy::sky_culture::chinese::load_lunar_mansion_systems_from_repo;
    use crate::astronomy::stars::load_hipparcos_bright_from_repo;

    /// 快照自洽：黄道极与太阳相距 90°；太阳星等；距星框架与 [`MansionFrame::new`]（逐颗管线）一致；月、行星星等在合理范围。
    #[test]
    fn snapshot_consistency() {
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
            .iter()
            .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::De406).unwrap()))
            .collect();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
        let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let opts = ApparentPipelineOptions::default();
        let systems = load_lunar_mansion_systems_from_repo().unwrap();
        for jd in [2460409.26, 2451545.0, 1655375.9] {
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let snap = sky_snapshot(&eph, &cat, t, &opts);
            assert_eq!(snap.stars.len(), cat.stars.len());
            let dir = |ra: PlaneAngle, dec: PlaneAngle| {
                let (a, d) = (ra.rad(), dec.rad());
                [d.cos() * a.cos(), d.cos() * a.sin(), d.sin()]
            };
            let sun = snap.bodies[0];
            let sep = angle_between(dir(sun.ra, sun.dec), dir(snap.ecliptic_pole.0, snap.ecliptic_pole.1)).rad().as_f64().to_degrees();
            println!("jd {jd} 太阳离黄道极 {sep:.7}°");
            // 太阳黄纬 < 1″
            assert!((sep - 90.0).abs() < 1.0 / 3600.0, "jd {jd} 太阳离黄道极 {sep}°");
            let frame = snap.mansion_frame(&systems[0]).unwrap();
            let direct = MansionFrame::new(&eph.planets, &cat, &systems[0], t, &opts).unwrap();
            for i in 0..28 {
                assert!((frame.ra[i] - direct.ra[i]).abs().as_f64() < 1e-12 && (frame.dec[i] - direct.dec[i]).abs().as_f64() < 1e-12);
            }
            for b in &snap.bodies {
                println!("jd {jd} {:?}: 星等 {:?}", b.body, b.magnitude.map(|m| m.as_f64()));
            }
            let moon = snap.bodies[1].magnitude.unwrap().as_f64();
            assert!((-12.8..-2.0).contains(&moon), "月亮星等 {moon}");
            let venus = snap.bodies[3].magnitude.unwrap().as_f64();
            assert!((-5.0..-3.5).contains(&venus), "金星星等 {venus}");
        }
    }

    #[test]
    fn refraction_table_follows_model_near_horizon() {
        let atm = Atmosphere::default();
        let alts: Vec<Real> = [-5.0, -0.5, 0.0, 1.0, 5.0, 45.0, 90.0].iter().map(|&x| real(x)).collect();
        let t = refraction_table(&alts, RefractionModel::RayTrace, &atm);
        let arcmin: Vec<f64> = t.iter().map(|x| x.as_f64() * 60.0).collect();
        println!("折射（角分）{arcmin:?}");
        assert!(arcmin[0].abs() < 1e-6, "地平下 5° 应已无折射");
        for w in arcmin[1..].windows(2) {
            assert!(w[0] > w[1], "真高度升高折射应减小：{arcmin:?}");
        }
        assert!(arcmin[2] > 25.0 && arcmin[2] < 35.0, "地平折射约半度：{}", arcmin[2]);
        assert!(arcmin[6].abs() < 1e-3, "天顶无折射");
    }

    /// 站心差：月亮在地平附近约 1°，并按 π·cos h 随高度减小；太阳只有约 9″。
    #[test]
    fn topocentric_places_show_lunar_parallax() {
        use crate::astronomy::time::earth_rotation::gast;
        use crate::quantity::length::Length;
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
            .iter()
            .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::De406).unwrap()))
            .collect();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
        let eph = Vsop87Elp {
            planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() },
            moon: &elp,
        };
        let catalog = StarCatalog { stars: vec![] };
        let observer = Observer::new(PlaneAngle::from_deg(real(108.94)), PlaneAngle::from_deg(real(34.27)), Length::from_value(real(0.0), crate::quantity::unit::LengthUnit::Meter));
        let opts = ApparentPipelineOptions::default();
        let alt = |ra: PlaneAngle, dec: PlaneAngle, g: PlaneAngle| {
            let h = g.rad() + observer.longitude.rad() - ra.rad();
            let (d, f) = (dec.rad(), observer.latitude.rad());
            (f.sin() * d.sin() + f.cos() * d.cos() * h.cos()).asin().as_f64().to_degrees()
        };
        let mut largest = 0.0f64;
        for i in 0..96 {
            let jd = 2460000.5 + i as f64 / 4.0;
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let snap = sky_snapshot(&eph, &catalog, t, &opts);
            let g = gast(real(jd - 69.0 / 86400.0), real(jd));
            let topo = snap.topocentric_places(&observer, g);
            for (b, place) in snap.bodies.iter().zip(&topo) {
                let (h0, h1) = (alt(b.ra, b.dec, g), alt(place.ra, place.dec, g));
                // 视半径：太阳 15.7′—16.3′，月亮 14.6′—16.9′（近地点、天顶略增）；行星不给
                let sd = place.semidiameter.map(|a| a.rad().as_f64().to_degrees() * 60.0);
                match b.body {
                    Body::Sun => assert!((15.7..16.3).contains(&sd.unwrap()), "太阳视半径 {sd:?}′"),
                    Body::Moon => assert!((14.6..17.0).contains(&sd.unwrap()), "月亮视半径 {sd:?}′"),
                    _ => assert!(sd.is_none()),
                }
                let drop = h0 - h1;
                match b.body {
                    Body::Moon => {
                        // 地平视差 53′—62′；椭球与弧度近似留 3′ 余量
                        let c = h0.to_radians().cos();
                        assert!(drop > 0.85 * c - 0.05 && drop < 1.05 * c + 0.05, "h={h0} 降低 {drop}");
                        largest = largest.max(drop);
                    }
                    Body::Sun => assert!(drop.abs() < 0.004, "太阳站心差 {drop}"),
                    _ => assert!(drop.abs() < 0.02, "行星站心差 {drop}"),
                }
            }
        }
        assert!(largest > 0.8, "月亮最大站心差 {largest}");
    }

    #[test]
    fn disc_visible_fraction_limits() {
        let f = |d: f64, a: f64, b: f64| disc_visible_fraction(real(d), real(a), real(b)).as_f64();
        assert_eq!(f(1.0, 0.27, 0.27), 1.0);
        assert_eq!(f(0.0, 0.26, 0.27), 0.0);
        assert!((f(0.0, 0.27, 0.135) - 0.75).abs() < 1e-12, "环食：只遮内圆");
        // 等圆心距为半径时，重叠面积 = r²(2π/3 − √3/2)
        let lens = (2.0 * std::f64::consts::PI / 3.0 - 3f64.sqrt() / 2.0) / std::f64::consts::PI;
        assert!((f(1.0, 1.0, 1.0) - (1.0 - lens)).abs() < 1e-12);
    }

    /// 天光：白昼远亮于深夜，白昼极限星等亮到看不见一般恒星，深夜可到 6 等上下；日全食时回到接近夜空的亮度。
    #[test]
    fn sky_light_day_night_and_eclipse() {
        use crate::astronomy::time::earth_rotation::gast;
        use crate::quantity::length::Length;
        use crate::quantity::unit::LengthUnit;
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
            .iter()
            .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::De406).unwrap()))
            .collect();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
        let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
        let cat = StarCatalog { stars: vec![] };
        let opts = ApparentPipelineOptions::default();
        let atm = Atmosphere::default();
        let (sky, eye) = (SkyConditions::default(), Eye::default());
        let run = |lon: f64, lat: f64, jd_ut: f64| {
            let observer = Observer::new(PlaneAngle::from_deg(real(lon)), PlaneAngle::from_deg(real(lat)), Length::from_value(real(0.0), LengthUnit::Meter));
            let jd = jd_ut + 69.0 / 86400.0;
            let snap = sky_snapshot(&eph, &cat, TimePoint::new(TimeScale::TT, real(jd)), &opts);
            let g = gast(real(jd_ut), real(jd));
            let topo = snap.topocentric_places(&observer, g);
            snap.sky_light(&topo, &observer, g, &atm, &sky, &eye)
        };
        // 取天顶那一行
        let i = (SKY_GRID_ALT - 1) * SKY_GRID_AZ;
        // 2009-07-22 长江日全食：武汉约 01:25 UT 全食；前一日同时刻无食
        let (e, d0) = (run(114.3, 30.6, 2455034.5 + 1.42 / 24.0), run(114.3, 30.6, 2455033.5 + 1.42 / 24.0));
        // 当地午夜（同地 16:00 UT，无月影响不大）
        let night = run(114.3, 30.6, 2455033.5 + 16.0 / 24.0);
        let (be, bd, bn) = (e.log10_brightness_nl[i].as_f64(), d0.log10_brightness_nl[i].as_f64(), night.log10_brightness_nl[i].as_f64());
        let (le, ld, ln) = (e.limiting_magnitude[i].as_f64(), d0.limiting_magnitude[i].as_f64(), night.limiting_magnitude[i].as_f64());
        println!("天顶 lg B：白昼 {bd:.2}，全食 {be:.2}，深夜 {bn:.2}；极限星等 {ld:.2}、{le:.2}、{ln:.2}；未遮比例 {}", e.sun_visible_fraction.as_f64());
        assert!(e.log10_brightness_nl.iter().chain(&e.limiting_magnitude).all(|x| x.as_f64().is_finite()), "网格含非数");
        assert!(e.sun_visible_fraction.as_f64() < 0.02, "武汉应近全食：{}", e.sun_visible_fraction.as_f64());
        assert_eq!(d0.sun_visible_fraction.as_f64(), 1.0);
        assert!(bd > 7.5 && bn < 3.5, "白昼 {bd}、深夜 {bn}");
        assert!(ld < 0.0 && ln > 5.0, "极限星等 白昼 {ld}、深夜 {ln}");
        // 极限星等曲线单调：天越亮看到的星越亮；白昼天顶无月光眩光时，曲线与网格同量级
        assert!(e.limit_curve.windows(2).all(|w| w[0] > w[1]), "极限星等应随亮度下降");
        assert!(night.moon_glare_nl_deg2.as_f64() >= 0.0);
        let unit = e.corona_unit_nl.as_f64();
        println!("日冕单位 {unit:.3e} nL（日面中心 {:.3e} cd/m²）", unit * 1e6 * 3.183e-6);
        assert!(unit > 5e8 && unit < 1e9, "日面中心约 2e9 cd/m²：{unit}");
        assert!((be - 5.3).abs() < 0.5, "全食天顶应近实测 13 mag/arcsec²（lg B ≈ 5.3）：{be}");
        assert!(le > 0.0 && le < ln, "全食亮星可见而暗星不可见：{le} vs {ln}");
    }

    /// 银道矩阵：J2000 银心在 (l, b) = (0, 0)，北银极在 b = 90°；岁差去得干净（自行极小的参宿七相隔两千年银道位置不变）。
    #[test]
    fn galactic_matrix_cancels_precession() {
        let dir = |ra: f64, dec: f64| {
            let (a, d) = (ra.to_radians(), dec.to_radians());
            [d.cos() * a.cos(), d.cos() * a.sin(), d.sin()]
        };
        let mul = |m: &[[Real; 3]; 3], v: [f64; 3]| [0, 1, 2].map(|r| (0..3).map(|c| m[r][c].as_f64() * v[c]).sum::<f64>());
        let m0 = true_equator_to_galactic(TimePoint::new(TimeScale::TT, real(2451545.0)));
        let gc = mul(&m0, dir(266.40499, -28.93617));
        assert!((gc[0] - 1.0).abs() < 1e-8 && gc[1].abs() < 1e-4 && gc[2].abs() < 1e-4, "银心 {gc:?}");
        let ngp = mul(&m0, dir(192.85948, 27.12825));
        assert!((ngp[2] - 1.0).abs() < 1e-8, "北银极 {ngp:?}");

        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
            .iter()
            .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::De406).unwrap()))
            .collect();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
        let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let opts = ApparentPipelineOptions::default();
        let gal = |jd: f64| {
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let snap = sky_snapshot(&eph, &cat, t, &opts);
            let rigel = snap.star(24436).unwrap();
            mul(&true_equator_to_galactic(t), dir(rigel.ra.rad().as_f64().to_degrees(), rigel.dec.rad().as_f64().to_degrees()))
        };
        let (a, b) = (gal(2451545.0), gal(2451545.0 - 730500.0));
        let sep = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt().to_degrees();
        println!("参宿七相隔两千年银道位置差 {sep:.4}°");
        assert!(sep < 0.05, "差 {sep}°");
    }

    /// 输出 doc/astronomy/sky-view-3d.md 的数值表；同时检查各设置的方向对不对。
    #[test]
    fn sky_view_doc_tables() {
        use crate::astronomy::correction::{ray_trace_sea_horizon, RayTraceConstants};
        use crate::astronomy::time::earth_rotation::gast;
        use crate::quantity::length::Length;
        use crate::quantity::pressure::Pressure;
        use crate::quantity::thermodynamic_temperature::ThermodynamicTemperature;
        use crate::quantity::unit::LengthUnit;
        let m = |x: f64| Length::from_value(real(x), LengthUnit::Meter);
        let mut last = -1.0;
        for h in [0.0, 10.0, 100.0, 1000.0, 3000.0, 5000.0] {
            let (app, refr) = ray_trace_sea_horizon(&Atmosphere::standard_at_height(m(h)), &RayTraceConstants::default());
            let dip = -app.as_f64().to_degrees();
            println!("俯角 h={h} m：{dip:.3}°，几何 {:.3}°，掠地折射 {:.1}′", (2.0 * h / 6378137.0f64).sqrt().to_degrees(), refr.as_f64().to_degrees() * 60.0);
            assert!(dip > last);
            last = dip;
        }
        let r = |a: &Atmosphere, h: f64| refraction_table(&[real(h)], RefractionModel::RayTrace, a)[0].as_f64() * 60.0;
        let std = Atmosphere::default();
        let mut cold = Atmosphere::default();
        cold.temperature = ThermodynamicTemperature::from_celsius(real(-30.0));
        cold.pressure = Pressure::from_kpa(real(105.0));
        let inv = Atmosphere::default().with_inversion(m(100.0), real(0.05));
        for (name, a) in [("标准", &std), ("−30℃ 105kPa", &cold), ("逆温 100 m 升 5 K", &inv)] {
            println!("折射 {name}：真高度 0° {:.2}′，1° {:.2}′，5° {:.2}′，45° {:.3}′", r(a, 0.0), r(a, 1.0), r(a, 5.0), r(a, 45.0));
        }
        assert!(r(&cold, 5.0) > r(&std, 5.0) && r(&inv, 0.0) > r(&std, 0.0));

        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
            .iter()
            .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::De406).unwrap()))
            .collect();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
        let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
        let cat = StarCatalog { stars: vec![] };
        let opts = ApparentPipelineOptions::default();
        let run = |jd_ut: f64, sky: &SkyConditions| {
            let observer = Observer::new(PlaneAngle::from_deg(real(108.94)), PlaneAngle::from_deg(real(34.27)), m(0.0));
            let jd = jd_ut + 69.0 / 86400.0;
            let snap = sky_snapshot(&eph, &cat, TimePoint::new(TimeScale::TT, real(jd)), &opts);
            let g = gast(real(jd_ut), real(jd));
            let topo = snap.topocentric_places(&observer, g);
            snap.sky_light(&topo, &observer, g, &std, sky, &Eye::default())
        };
        let i = (SKY_GRID_ALT - 1) * SKY_GRID_AZ;
        // 无月深夜：2026-11-10 地方午夜（朔后一日），长安 UT ≈ 16:44
        let dark = 2461354.5 + 16.74 / 24.0;
        let mut rows = vec![];
        for (name, h, city) in [("湿度 20%", 0.2, 0.0), ("湿度 50%", 0.5, 0.0), ("湿度 90%", 0.9, 0.0), ("城市光 1000 nL", 0.5, 1000.0), ("城市光 10000 nL", 0.5, 10000.0)] {
            let mut sky = SkyConditions::schaefer1998(real(h), real(15.0));
            sky.city_nl = real(city);
            let l = run(dark, &sky);
            let (b, lim) = (l.log10_brightness_nl[i].as_f64(), l.limiting_magnitude[i].as_f64());
            // 高度约 5° 一行（i = 3）的南方
            let lo = 3 * SKY_GRID_AZ + 12;
            println!("{name}：天顶 lg B {b:.2}、极限 {lim:.2}；高度 5.6° 极限 {:.2}", l.limiting_magnitude[lo].as_f64());
            rows.push(lim);
        }
        assert!(rows[0] > rows[2] && rows[1] > rows[3] && rows[3] > rows[4]);

        // 满月：2026-11-24 地方午夜（望），月亮高悬
        let full = 2461368.5 + 16.74 / 24.0;
        let l = run(full, &SkyConditions::default());
        let g0 = l.moon_glare_nl_deg2.as_f64();
        let curve = |lg: f64| {
            let x = (lg / LIMIT_CURVE_STEP).clamp(0.0, LIMIT_CURVE_SAMPLES as f64 - 1.001);
            let k = x.floor() as usize;
            l.limit_curve[k].as_f64() + (l.limit_curve[k + 1].as_f64() - l.limit_curve[k].as_f64()) * (x - k as f64)
        };
        let (lg, lim) = (l.log10_brightness_nl[i].as_f64(), l.limiting_magnitude[i].as_f64());
        println!("满月：天顶 lg B {lg:.2}、无眩光极限 {lim:.2}；眩光系数 {g0:.3e} nL·度²");
        let mut last = -99.0;
        for th in [MOON_GLARE_MIN_DEG, 1.0, 2.0, 5.0, 10.0, 20.0, 45.0] {
            let v = lim + curve((10f64.powf(lg) + g0 / (th * th)).log10()) - curve(lg);
            println!("  离月 {th}°：极限 {v:.2}");
            assert!(v > last, "离月越远看得越暗");
            last = v;
        }
    }
}
