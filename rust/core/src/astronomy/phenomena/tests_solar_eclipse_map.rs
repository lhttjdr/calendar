//! 日食地图检验：
//! 1. 对照 NASA 公布的极大食点、中心食带宽（2017-08-21、2024-04-08 全食）；
//! 2. 与站心局地算法 [`solar_eclipse_local_in`] 互验——两者独立（前者在基本面上求包络，后者逐地求站心日月角距）：
//!    本影界内侧有食既、外侧无；半影界内侧有食、外侧无；带食出没界线上初亏或复圆时太阳几何高度为 0。
//!
//! TwoFloat debug 构建较慢，建议 `cargo test --release`。

use super::*;
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::geodesy::Observer;
use crate::math::real::{real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;
use crate::quantity::length::Length;
use crate::quantity::unit::LengthUnit;

fn load() -> (crate::astronomy::ephemeris::Vsop87, crate::astronomy::ephemeris::Elpmpp02Data) {
    let vsop = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).expect("VSOP87");
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).expect("ELPMPP02");
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    (vsop, elp)
}

fn deg(p: &MapPoint) -> (f64, f64) {
    (p.longitude.rad().as_f64().to_degrees(), p.latitude.rad().as_f64().to_degrees())
}

fn observer(lon: f64, lat: f64) -> Observer {
    Observer::new(
        PlaneAngle::from_deg(real(lon)),
        PlaneAngle::from_deg(real(lat)),
        Length::from_value(real(0.0), LengthUnit::Meter),
    )
}

fn xyz_km(lon: f64, lat: f64) -> [f64; 3] {
    observer(lon, lat).geocentric_itrs_meters().map(|v| v.as_f64() / 1000.0)
}

/// 点到折线的最短弦长（km）。
fn distance_to_polyline_km(p: [f64; 3], line: &[MapPoint]) -> f64 {
    let mut best = f64::INFINITY;
    for w in line.windows(2) {
        let (a, b) = (deg(&w[0]), deg(&w[1]));
        let (a, b) = (xyz_km(a.0, a.1), xyz_km(b.0, b.1));
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
        let len2 = ab.iter().map(|v| v * v).sum::<f64>();
        let s = if len2 > 0.0 { (ap.iter().zip(&ab).map(|(x, y)| x * y).sum::<f64>() / len2).clamp(0.0, 1.0) } else { 0.0 };
        let d = (0..3).map(|n| (ap[n] - s * ab[n]).powi(2)).sum::<f64>().sqrt();
        best = best.min(d);
    }
    best
}

/// NASA（Espenak）公布值：极大食点经纬度（度）、该处中心食带宽（km）。ΔT 取 NASA 当期所用值。
struct NasaCase {
    name: &'static str,
    jd_tt_guess: f64,
    delta_t_s: f64,
    greatest_lon: f64,
    greatest_lat: f64,
    width_km: f64,
}

const CASES: [NasaCase; 2] = [
    // https://eclipse.gsfc.nasa.gov/SEplot/SEplot2001/SE2017Aug21T.GIF ：36°58′N 87°40′W，宽 115 km，ΔT 70.3 s
    NasaCase { name: "2017-08-21", jd_tt_guess: 2457987.27, delta_t_s: 70.3, greatest_lon: -87.67, greatest_lat: 36.97, width_km: 115.0 },
    // https://eclipse.gsfc.nasa.gov/SEplot/SEplot2001/SE2024Apr08T.GIF ：25°17′N 104°08′W，宽 198 km，ΔT 70.6 s
    NasaCase { name: "2024-04-08", jd_tt_guess: 2460409.26, delta_t_s: 70.6, greatest_lon: -104.14, greatest_lat: 25.29, width_km: 198.0 },
];

#[test]
fn nasa_greatest_point_and_path_width() {
    let (vsop, elp) = load();
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for c in &CASES {
        let win = eclipse_window(&eph, real(c.jd_tt_guess));
        let opts = SolarEclipseMapOptions { outline_interval: None, ..Default::default() };
        let map = solar_eclipse_map(&win, real(c.jd_tt_guess), Duration::in_seconds(real(c.delta_t_s)), &opts).expect("应有日食");
        let (lon, lat) = deg(&map.greatest_point);
        let p = xyz_km(lon, lat);
        let w = distance_to_polyline_km(p, &map.umbra_north.concat()) + distance_to_polyline_km(p, &map.umbra_south.concat());
        println!(
            "{}: 极大食点 {lon:.3}, {lat:.3}（NASA {:.3}, {:.3}），带宽 {w:.1} km（NASA {}）；中心线 {} 段，本影北/南界 {}/{} 段，半影界 {} 段，带食出没 {} 段",
            c.name,
            c.greatest_lon,
            c.greatest_lat,
            c.width_km,
            map.central_line.len(),
            map.umbra_north.len(),
            map.umbra_south.len(),
            map.penumbra_limits.len(),
            map.rise_set.len()
        );
        assert!((lon - c.greatest_lon).abs() < 0.05 && (lat - c.greatest_lat).abs() < 0.05, "{} 极大食点偏差过大", c.name);
        assert!((w - c.width_km).abs() < 3.0, "{} 带宽 {w} km 与 NASA {} km 相差过大", c.name, c.width_km);
    }
}

/// 折线第 i 点沿法向两侧各挪 `km` 公里的两点（局部东-北平面近似，短距离足够）。
fn normal_pair(line: &[MapPoint], i: usize, km: f64) -> ((f64, f64), (f64, f64)) {
    let (a, b) = if i + 1 < line.len() { (deg(&line[i]), deg(&line[i + 1])) } else { (deg(&line[i - 1]), deg(&line[i])) };
    let p = deg(&line[i]);
    let c = p.1.to_radians().cos();
    let mut dlon = b.0 - a.0;
    if dlon > 180.0 { dlon -= 360.0 } else if dlon < -180.0 { dlon += 360.0 }
    let (e, nn) = (dlon * c, b.1 - a.1);
    let len = (e * e + nn * nn).sqrt();
    let (ue, un) = (-nn / len, e / len); // 左法向
    let d = km / 111.2;
    ((p.0 + ue * d / c, p.1 + un * d), (p.0 - ue * d / c, p.1 - un * d))
}

#[test]
fn limits_agree_with_local_circumstances() {
    let (vsop, elp) = load();
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for c in &CASES {
        let dt = Duration::in_seconds(real(c.delta_t_s));
        let win = eclipse_window(&eph, real(c.jd_tt_guess));
        let opts = SolarEclipseMapOptions { outline_interval: None, ..Default::default() };
        let map = solar_eclipse_map(&win, real(c.jd_tt_guess), dt, &opts).unwrap();
        let local = |p: (f64, f64)| solar_eclipse_local_in(&win, &observer(p.0, p.1), real(c.jd_tt_guess), dt);

        // 界线两侧各挪一小段：本影界一侧见全/环食、另一侧不见；半影界一侧有食、另一侧无食
        for (name, lines, km, umbra) in [
            ("本影北界", &map.umbra_north, 3.0, true),
            ("本影南界", &map.umbra_south, 3.0, true),
            ("半影界", &map.penumbra_limits, 10.0, false),
        ] {
            let mut checked = 0;
            for line in lines.iter().filter(|l| l.len() >= 10) {
                for i in (line.len() / 10..line.len() - line.len() / 10).step_by((line.len() / 6).max(1)) {
                    let (l, r) = normal_pair(line, i, km);
                    let has = |p: (f64, f64)| {
                        let e = local(p);
                        if umbra { e.is_some_and(|e| e.second_contact.is_some()) } else { e.is_some() }
                    };
                    assert!(has(l) != has(r), "{} {name} {:?}：两侧同为 {}", c.name, deg(&line[i]), has(l));
                    checked += 1;
                }
            }
            assert!(checked >= 4, "{} {name} 检查点太少", c.name);
        }

        // 食分等值线：太阳不低（> 2°）处，站心算法的食分应等于等值线级别
        for mc in &map.magnitude_contours {
            let pts = mc.lines.concat();
            let mut worst = 0.0_f64;
            let mut n = 0;
            for p in pts.iter().step_by((pts.len() / 15).max(1)) {
                let Some(e) = local(deg(p)) else { panic!("{} {} 级等值线上 {:?} 无食", c.name, mc.level.as_f64(), deg(p)) };
                if e.sun_altitude_greatest.rad().as_f64().to_degrees() < 2.0 {
                    continue;
                }
                worst = worst.max((e.magnitude.as_f64() - mc.level.as_f64()).abs());
                n += 1;
            }
            println!("{}：食分 {} 等值线 {n} 点，站心食分最大偏差 {worst:.4}", c.name, mc.level.as_f64());
            assert!(n >= 3 && worst < 0.003, "{} 食分 {} 等值线偏差 {worst}", c.name, mc.level.as_f64());
        }

        // 带食出没界线：初亏或复圆时太阳（站心几何）高度为 0
        let pts = map.rise_set.concat();
        assert!(pts.len() > 20, "带食出没界线点太少");
        let mut worst = 0.0_f64;
        for p in pts.iter().step_by(pts.len() / 12) {
            let e = local(deg(p)).expect("带食出没界线上应有食");
            let alt = [e.sun_altitude_first, e.sun_altitude_last]
                .iter()
                .map(|a| a.rad().as_f64().to_degrees().abs())
                .fold(f64::INFINITY, f64::min);
            worst = worst.max(alt);
        }
        println!("{}：带食出没界线上初亏/复圆时太阳高度最大偏差 {worst:.4}°", c.name);
        assert!(worst < 0.02, "带食出没界线上太阳高度偏差 {worst}° 过大");
    }
}

#[test]
fn delta_t_change_is_pure_longitude_shift() {
    let (vsop, elp) = load();
    let eph = EclipseEphemeris::new(&vsop, &elp);
    let c = &CASES[0];
    let win = eclipse_window(&eph, real(c.jd_tt_guess));
    let opts = SolarEclipseMapOptions { outline_interval: None, ..Default::default() };
    let a = solar_eclipse_map(&win, real(c.jd_tt_guess), Duration::in_seconds(real(70.0)), &opts).unwrap();
    let b = solar_eclipse_map(&win, real(c.jd_tt_guess), Duration::in_seconds(real(1270.0)), &opts).unwrap();
    let shift = longitude_shift_for_delta_t(Duration::in_seconds(real(1200.0))).rad().as_f64().to_degrees();
    let (pa, pb) = (deg(&a.greatest_point), deg(&b.greatest_point));
    println!("ΔT +1200 s：极大食点经度 {:.4} → {:.4}，预期平移 {shift:.4}°", pa.0, pb.0);
    assert!((pb.0 - pa.0 - shift).abs() < 1e-6 && (pb.1 - pa.1).abs() < 1e-9);
}

/// 半影界只能终止于地球边缘，即落在带食出没界线上（容差 150 km，约网格 1°）。
/// 含前 181-03-04（高后七年正月己丑晦「日有食之，既」）；2017-08-21 的半影北界横过北极圈（极昼）、南界在南美东岸近边缘处
/// 有尖点，曾用逐格局地计算核对过真实可见食区边界。
#[test]
fn penumbral_limits_end_on_rise_set_curve() {
    let (vsop, elp) = load();
    let eph = EclipseEphemeris::new(&vsop, &elp);
    for (name, jd, dt) in [("前181-03-04", 1655375.9, 12520.0), ("2017-08-21", CASES[0].jd_tt_guess, 70.3), ("2024-04-08", CASES[1].jd_tt_guess, 70.6)] {
        let win = eclipse_window(&eph, real(jd));
        let opts = SolarEclipseMapOptions { outline_interval: None, magnitude_levels: vec![], ..Default::default() };
        let map = solar_eclipse_map(&win, real(jd), Duration::in_seconds(real(dt)), &opts).unwrap();
        let mut worst = 0.0_f64;
        for line in &map.penumbra_limits {
            for end in [line.first().unwrap(), line.last().unwrap()] {
                let (lo, la) = deg(end);
                let d = map.rise_set.iter().map(|l| distance_to_polyline_km(xyz_km(lo, la), l)).fold(f64::INFINITY, f64::min);
                worst = worst.max(d);
            }
        }
        println!("{name}：半影界 {} 段，端点到带食出没界线最远 {worst:.0} km", map.penumbra_limits.len());
        assert!(worst < 150.0, "{name} 半影界端点偏离 {worst} km");
    }
}


