//! 大统推步存疑诸月的现代天文合朔：与网页同一组历表（VSOP87B 地球 + DE431 改正、ELPMPP02 缺省常数集、IAU2000 章动），
//! 求合朔 UT，按南京（东经 118°47′，UT+7h55m08s）、北京（东经 116°25′，UT+7h45m40s）地方平时定日。
//! 月份取 data/calendar/datong_shilu_check.csv 所列（《明实录》月朔与推步、张表有异之月）。
//!
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example datong_modern_compare
use lunar_core::astronomy::ephemeris::vsop87_patch::Vsop87DePatch;
use lunar_core::astronomy::ephemeris::{load_all_from_binary_with_fit, Elpmpp02Correction, Vsop87};
use lunar_core::astronomy::frame::nutation;
use lunar_core::astronomy::time::{TimePoint, TimeScale};
use lunar_core::calendar::chinese_lunar::compute_year_data;
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::gregorian::{civil_from_jdn, civil_to_jdn};
use lunar_core::math::real::{real, RealOps};
use lunar_core::quantity::angle::PlaneAngle;
use std::fs;

fn read(p: &str) -> Vec<u8> {
    fs::read(p).unwrap_or_else(|e| panic!("读 {p}: {e}"))
}

const NANJING: f64 = (7.0 * 3600.0 + 55.0 * 60.0 + 8.0) / 86400.0;
const BEIJING: f64 = (7.0 * 3600.0 + 45.0 * 60.0 + 40.0) / 86400.0;

fn local(jd_ut: f64, off: f64) -> String {
    let t = jd_ut + 0.5 + off;
    let d = t.floor() as i64;
    let (y, m, dd) = civil_from_jdn(d);
    let h = (t - t.floor()) * 24.0;
    let gz = GAN_ZHI_60[(d + 49).rem_euclid(60) as usize];
    format!("{gz} {y}-{m:02}-{dd:02} {:02}:{:02}", h as i64, ((h - h.floor()) * 60.0).round() as i64)
}

fn main() {
    let patch = Vsop87DePatch::from_binary(&read("data/fit/vsop87-de431-earth.bin")).expect("地球改正");
    let vsop = Vsop87::from_binary(&read("vendor/vsop87/VSOP87B.ear.bin")).expect("VSOP87").with_patch(patch.body("ear").expect("ear").clone());
    let corr = Elpmpp02Correction::DEFAULT;
    let fit = corr.builtin_fit().expect("ELP 常数集");
    let names = ["ELP_MAIN.S1", "ELP_MAIN.S2", "ELP_MAIN.S3", "ELP_PERT.S1", "ELP_PERT.S2", "ELP_PERT.S3"];
    let bufs: Vec<Vec<u8>> = names.iter().map(|n| read(&format!("vendor/elpmpp02/{n}.bin"))).collect();
    let elp = load_all_from_binary_with_fit([&bufs[0], &bufs[1], &bufs[2], &bufs[3], &bufs[4], &bufs[5]], &fit, corr).expect("ELPMPP02");
    assert!(nutation::try_init_full_nutation_from_binary(&read("vendor/IAU2000/tab5.3a.bin")), "章动表");
    let ut = |tt: f64| TimePoint::new(TimeScale::TT, real(tt)).to_scale(TimeScale::UTC).jd.as_f64();
    let tol = PlaneAngle::from_rad(real(1e-9));

    let src = String::from_utf8(read("data/calendar/datong_shilu_check.csv")).unwrap();
    println!("年,月,实录朔,现代合朔（南京地方平时）,现代合朔（北京地方平时）");
    for l in src.lines().skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        let (y, m): (i32, i32) = (c[0].parse().unwrap(), c[1].parse().unwrap());
        // 定朔日期（立成推步列）附近之合朔
        let d = &c[5][..10];
        let near = civil_to_jdn(d[..4].parse().unwrap(), d[5..7].parse().unwrap(), d[8..10].parse().unwrap()) as f64;
        let mut best = f64::NAN;
        for ly in [y - 1, y, y + 1] {
            let yd = compute_year_data(&vsop, &elp, ly, tol, 40).unwrap();
            for t in &yd.new_moon_jds {
                let u = ut(t.as_f64());
                if best.is_nan() || (u - near).abs() < (best - near).abs() {
                    best = u;
                }
            }
        }
        println!("{y},{m},{},{},{}", c[3], local(best, NANJING), local(best, BEIJING));
    }
}
