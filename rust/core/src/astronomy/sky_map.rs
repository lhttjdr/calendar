//! 星图用的天空快照（天文域派生层）：某时刻全部星表恒星、日月五星的地心视位置（当日真赤道与真春分点），黄道极方向，
//! 以及由快照中距星位置构成的二十八宿框架。只给天体位置与星等，投影、连线、星名等绘图事宜在前端。
//!
//! 恒星用 [`stars_apparent_equatorial`] 批量求（与逐颗计算一致，差 < 1e-10″）；日月行星走统一视位置管线
//! [`body_apparent_equatorial`]。星等：行星 Mallama & Hilton 2018（[`planet_magnitude`]），月亮 Krisciunas & Schaefer 1991
//! （[`moon_magnitude`]，按月相角），太阳取 −26.74。

use crate::astronomy::apparent::{
    body_apparent_equatorial, planet_magnitude, right_ascension_declination, stars_apparent_equatorial, ApparentPipelineOptions,
};
use crate::astronomy::phenomena::visibility::moon_magnitude;
use crate::astronomy::frame::nutation::nutation_for_apparent;
use crate::astronomy::frame::precession::mean_obliquity_for;
use crate::astronomy::pipeline::{Body, EphemerisProvider, Planet};
use crate::astronomy::sky_culture::chinese::{LunarMansionSystem, MansionFrame};
use crate::astronomy::stars::StarCatalog;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};
use crate::quantity::angle::PlaneAngle;

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
    /// V 星等；模型不适用（如行星相角超出拟合范围）时为 None
    pub magnitude: Option<Real>,
    /// 月亮、行星的相角（日—天体—地夹角）；太阳为 None
    pub phase_angle: Option<PlaneAngle>,
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
            BodyPlace { body, ra, dec, magnitude, phase_angle }
        })
        .collect();

    // 黄道极：当日真赤道架以真春分点为 x 轴，黄道面即赤道面绕 x 轴转过真黄赤交角 ε = ε_A + Δε，故黄道极严格为
    // (0, −sin ε, cos ε)，即赤经 270°、赤纬 90° − ε。章动取管线同一来源（nutation_for_apparent）。
    let tc = (t.jd - real(2451545.0)) / real(36525.0);
    let eps = mean_obliquity_for(tc, options.effective_precession_model()).rad() + nutation_for_apparent(tc).1.rad();
    let ecliptic_pole = (PlaneAngle::from_deg(real(270.0)), PlaneAngle::from_rad(Real::pi() / real(2.0) - eps));

    SkySnapshot { t, stars, bodies, ecliptic_pole }
}

impl SkySnapshot {
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
}
