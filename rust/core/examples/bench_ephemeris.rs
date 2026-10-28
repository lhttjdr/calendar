//! 历表与视位置的单次耗时基准（见 doc/engineering/project-and-implementation.md §11.2.2）。
//! 用法（rust/ 目录）：cargo run --release -p lunar-core --example bench_ephemeris [-- full]
//! 不带 full 只测三角函数与复数乘法；带 full 另测 ELPMPP02、VSOP87、视位置与 gast。
use lunar_core::astronomy::apparent::{body_apparent_equatorial, horizontal, topocentric, ApparentPipelineOptions};
use lunar_core::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use lunar_core::astronomy::ephemeris::vsop87_patch::PatchTarget;
use lunar_core::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
use lunar_core::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use lunar_core::astronomy::geodesy::Observer;
use lunar_core::astronomy::pipeline::{Body, Planet, Vsop87Elp, Vsop87Planets};
use lunar_core::astronomy::time::earth_rotation::gast;
use lunar_core::astronomy::time::{TimePoint, TimeScale};
use lunar_core::math::real::{real, Real};
use lunar_core::quantity::angle::PlaneAngle;
use lunar_core::quantity::epoch::Epoch;
use lunar_core::quantity::length::Length;
use lunar_core::quantity::position::Position;
use lunar_core::quantity::reference_frame::ReferenceFrame;
use std::time::Instant;
use lunar_core::quantity::unit::LengthUnit;

fn main() {
    micro();
    if std::env::args().nth(1).as_deref() != Some("full") { return; }
    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let loaded: Vec<_> = Planet::ALL.iter().map(|p| (*p, load_planet_vsop87_corrected_from_repo(*p, PatchTarget::De406).unwrap())).collect();
    let elp = load_all(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: loaded.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
    let opts = ApparentPipelineOptions::default();
    let obs = Observer::new(PlaneAngle::from_deg(real(112.45)), PlaneAngle::from_deg(real(34.65)), Length::from_value(real(100.0), LengthUnit::Meter));
    let t0 = 1_850_000.0;
    let n = 200;
    for (name, body) in [("Sun", Body::Sun), ("Moon", Body::Moon), ("Mars", Body::Planet(Planet::Mars))] {
        let s = Instant::now();
        for i in 0..n { let _ = body_apparent_equatorial(&eph, body, TimePoint::new(TimeScale::TT, real(t0 + i as f64)), &opts); }
        println!("body_apparent_equatorial {name}: {:.1} µs", s.elapsed().as_secs_f64() * 1e6 / n as f64);
    }
    let s = Instant::now();
    for i in 0..20 { let _ = lunar_core::astronomy::ephemeris::elpmpp02::position_velocity(&elp, TimePoint::new(TimeScale::TT, real(t0 + i as f64))); }
    println!("elpmpp02::position_velocity: {:.1} µs", s.elapsed().as_secs_f64() * 1e6 / 20.0);
    let s = Instant::now();
    for i in 0..20 { let _ = earth.position_and_velocity_jd(real(t0 + i as f64)); }
    println!("earth vsop87 position_velocity: {:.1} µs", s.elapsed().as_secs_f64() * 1e6 / 20.0);
    let n = 20000;
    let s = Instant::now();
    let mut acc = real(0.0);
    for i in 0..n { let t: Real = real(t0 + i as f64 / 144.0); let g = gast(t - real(0.1), t); acc = acc + g.rad(); }
    println!("gast: {:.2} µs ({:?})", s.elapsed().as_secs_f64() * 1e6 / n as f64, acc > real(0.0));
    let s = Instant::now();
    for i in 0..n {
        let t: Real = real(t0 + i as f64 / 144.0);
        let g = gast(t - real(0.1), t);
        let p = Position::from_si_meters_in_frame(ReferenceFrame::TrueEquator(Epoch::new(t)), real(1.0e11), real(2.0e10), real(3.0e10));
        let q = topocentric(p, &obs, g);
        let _ = horizontal(&q, &obs, g);
    }
    println!("gast + topocentric + horizontal: {:.2} µs", s.elapsed().as_secs_f64() * 1e6 / n as f64);
}

fn micro() {
    let n = 2_000_000;
    let mut x = real(0.1234567);
    let s = Instant::now();
    let mut acc = real(0.0);
    for i in 0..n { let y = x + real(i as f64 * 1e-3); acc = acc + y.sin() + y.cos(); }
    println!("twofloat sin+cos: {:.1} ns", s.elapsed().as_secs_f64() * 1e9 / n as f64);
    let s = Instant::now();
    let (mut c, mut sn) = (real(1.0), real(0.0));
    let (ca, sa) = (real(0.9999), real(0.0141));
    for _ in 0..n { let nc = c * ca - sn * sa; sn = c * sa + sn * ca; c = nc; }
    println!("twofloat complex mul: {:.1} ns ({})", s.elapsed().as_secs_f64() * 1e9 / n as f64, (c + sn + acc) > real(-1e300));
    let s = Instant::now();
    let mut a = 0.0f64;
    for i in 0..n { let y = 0.1234567 + i as f64 * 1e-3; a += y.sin() + y.cos(); }
    println!("f64 sin+cos: {:.1} ns ({})", s.elapsed().as_secs_f64() * 1e9 / n as f64, a > -1e300);
    x = x + real(1.0); let _ = x;
}
