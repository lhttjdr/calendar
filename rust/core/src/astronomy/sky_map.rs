//! 星图用的天空快照（天文域派生层）：某时刻全部星表恒星、日月五星的地心视位置（当日真赤道与真春分点），黄道极方向，
//! 以及由快照中距星位置构成的二十八宿框架。只给天体位置与星等，投影、连线、星名等绘图事宜在前端。
//!
//! 恒星用 [`stars_apparent_equatorial`] 批量求（与逐颗计算一致，差 < 1e-10″）；日月行星走统一视位置管线
//! [`body_apparent_equatorial`]。星等：行星 Mallama & Hilton 2018（[`planet_magnitude`]），月亮 Krisciunas & Schaefer 1991
//! （[`moon_magnitude`]，按月相角），太阳取 −26.74。
//!
//! 站在地面看的视图另要两项：[`SkySnapshot::topocentric_places`]（日月五星的站心位置，月亮地平视差约 1°）与
//! [`refraction_table`]（真高度 → 折射量）。恒星的站心差可略，不算。

use crate::astronomy::apparent::{
    body_apparent_equatorial, planet_magnitude, right_ascension_declination, stars_apparent_equatorial, topocentric, ApparentPipelineOptions,
};
use crate::astronomy::correction::{refraction_from_geometric, Atmosphere, RefractionModel};
use crate::astronomy::geodesy::Observer;
use crate::astronomy::phenomena::solar_eclipse::{sun_radius_m, K_PENUMBRA};
use crate::astronomy::phenomena::visibility::moon_magnitude;
use crate::astronomy::frame::nutation::nutation_for_apparent;
use crate::astronomy::frame::precession::mean_obliquity_for;
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
                TopocentricPlace { ra, dec, semidiameter }
            })
            .collect()
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
}
