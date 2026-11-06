//! 平气核对：列出各年天文冬至（定气）在指定都城地方平时下的日期与干支，以及参数表平气（若有）。
//!
//! 用法（仓库根）：cargo run -p lunar-core --no-default-features --features real-f64 --example pingqi_check -- 起年 止年 [东经]
//! 年为冬至所在公历年（即其后一岁的天正冬至）。东经缺省 113.0。

use lunar_core::astronomy::aspects::solar_term_jds_for_year_cached;
use lunar_core::astronomy::ephemeris::load_earth_vsop87_from_repo;
use lunar_core::astronomy::time::delta_t;
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::gregorian::Gregorian;
use lunar_core::calendar::pingqi::pingqi_tt;
use lunar_core::math::real::{real, RealOps};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let y0: i32 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(960);
    let y1: i32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(y0 + 20);
    let lon: f64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(113.0);
    let vsop = load_earth_vsop87_from_repo().expect("VSOP87");
    println!("冬至所在年\t天文冬至（地方平时）\t干支\t夜半后（日）\t平气冬至\t干支\t夜半后\t平−定（日）\t历");
    for y in y0..=y1 {
        let tt = solar_term_jds_for_year_cached(&vsop, y)[18];
        let to_local = |tt: lunar_core::math::real::Real| {
            let ut = tt - delta_t(tt).in_days_value();
            (ut + real(lon / 360.0)).as_f64()
        };
        let fmt = |jd_local: f64| {
            let jdn = (jd_local + 0.5).floor();
            let (yy, mm, dd) = Gregorian::from_julian_day(real(jdn));
            let gz = GAN_ZHI_60[((jdn as i64 + 49).rem_euclid(60)) as usize];
            (format!("{yy}-{mm:02}-{dd:02}"), gz, jd_local + 0.5 - jdn)
        };
        let (d, gz, f) = fmt(to_local(tt));
        let (pd, pgz, pf, diff, name) = match pingqi_tt(tt, 0) {
            Some((li, ptt)) => {
                let (pd, pgz, pf) = fmt(to_local(ptt));
                (pd, pgz.to_string(), format!("{pf:.4}"), format!("{:+.3}", (ptt - tt).as_f64()), li.name.clone())
            }
            None => ("—".into(), "—".into(), "—".into(), "—".into(), String::new()),
        };
        println!("{y}\t{d}\t{gz}\t{f:.4}\t{pd}\t{pgz}\t{pf}\t{diff}\t{name}");
    }
}
