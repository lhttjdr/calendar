//! 月面实况几何检验：
//! 1. Meeus 例 53.a（1992-04-12 0h TD，给定 λ、β、Δψ、ε、α）：天平动与月轴位置角对照书中结果；
//! 2. 对照 JPL Horizons（DE 历表的月球自转数值解，与 Meeus 解析式独立）：地心与长安站心的月面经纬度（ObsSub-LON/LAT）、
//!    月轴位置角（NP.ang）、站心几何高度与方位，近代三刻、前 180 年两刻。ΔT 取 Horizons 当时所用值，使站心量可比。

use super::moon_disk::{libration_and_axis_for_test, moon_disk_view_in};
use super::solar_eclipse::EclipseEphemeris;
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

fn r(x: f64) -> crate::math::real::Real {
    real(x.to_radians())
}

#[test]
fn meeus_example_53a() {
    let t = real((2448724.5 - 2451545.0) / 36525.0);
    let (l, b, p) = libration_and_axis_for_test(t, r(133.167265), r(-3.229126), r(134.688470), r(0.004610), r(23.440636));
    let (l, b, p) = (l.as_f64().to_degrees(), b.as_f64().to_degrees(), p.as_f64().to_degrees());
    println!("Meeus 53.a：l {l:.3}° b {b:.3}° P {p:.2}°（书中 −1.23°、4.20°、15.08°）");
    assert!((l + 1.23).abs() < 0.01 && (b - 4.20).abs() < 0.01 && (p - 15.08).abs() < 0.02);
}

/// 角度差折到 (−180°, 180°]
fn dang(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

#[test]
fn against_jpl_horizons() {
    let vsop = load_earth_vsop87_corrected_from_repo(PatchTarget::DEFAULT).expect("VSOP87");
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).expect("ELPMPP02");
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let eph = EclipseEphemeris::new(&vsop, &elp);
    let changan = Observer::new(PlaneAngle::from_deg(real(108.88)), PlaneAngle::from_deg(real(34.30)), Length::from_value(real(0.0), LengthUnit::Meter));
    // JD(TT)，Horizons ΔT（s），站心：方位、几何高度、月面经度、纬度、NP.ang（2026-11-05 取自 ssd.jpl.nasa.gov/api/horizons.api，
    // CENTER=coord@399 SITE_COORD=108.88,34.30,0，APPARENT=AIRLESS）
    let rows: [(f64, f64, f64, f64, f64, f64, f64); 5] = [
        (2448724.5, 58.185636, 352.706917, -42.366735, 358.879797, 4.895639, 15.0728),
        (2461102.98, 69.185395, 89.506691, 9.779931, 5.654852, 0.695654, 21.0444),
        (2458327.35, 69.183362, 232.019307, 15.833995, 358.784337, 0.307844, 344.7718),
        (1655390.5, 12567.226987, 257.964973, 25.441505, 355.855761, 0.583805, 24.6121),
        (1655404.3, 12566.755143, 351.134760, -58.735292, 2.542448, 1.622415, 334.8753),
    ];
    let mut worst = [0.0_f64; 5];
    for (jd, dt, az, alt, lon, lat, np) in rows {
        let v = moon_disk_view_in(&eph, &changan, real(jd), Duration::in_seconds(real(dt)));
        let g = |a: PlaneAngle| a.rad().as_f64().to_degrees();
        let d = [
            dang(g(v.azimuth), az),
            g(v.altitude) - alt,
            dang(g(v.libration_longitude), lon),
            g(v.libration_latitude) - lat,
            dang(g(v.axis_position_angle), np),
        ];
        println!(
            "JD {jd}：方位 {:+.4}° 高度 {:+.4}° 月面经度 {:+.4}° 纬度 {:+.4}° 月轴位置角 {:+.4}°（本实现 − Horizons）；天顶位置角 {:.2}°，竖直缩放 {:.4}",
            d[0], d[1], d[2], d[3], d[4], g(v.zenith_position_angle), v.vertical_scale.as_f64()
        );
        for k in 0..5 {
            worst[k] = worst[k].max(d[k].abs());
        }
    }
    println!("最大差：方位 {:.4}° 高度 {:.4}° 月面经度 {:.4}° 纬度 {:.4}° 月轴位置角 {:.4}°", worst[0], worst[1], worst[2], worst[3], worst[4]);
    // 古代两刻月亮位置本身与 DE441 差约 9″（见 celestial-phenomena 的 NASA 对照），地平下高度低处方位对此放大
    assert!(worst[0] < 0.03 && worst[1] < 0.02, "站心方位、高度");
    // 月面纬度近代系统偏 +0.02°～+0.03°、古代至 0.07°：Meeus 解析式（卡西尼定律 + 物理天平动级数）与 DE 月球自转数值解之差
    assert!(worst[2] < 0.05 && worst[3] < 0.1, "天平动");
    assert!(worst[4] < 0.05, "月轴位置角");
}
