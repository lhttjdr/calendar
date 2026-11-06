//! 朔与节气时刻：输出供两种数值后端逐条比对（见 doc/engineering/project-and-implementation.md §11.2.2）。
//! 用法（仓库根）：cargo run --release -p lunar-core --no-default-features --features real-f64 --example precision_calendar --manifest-path rust/Cargo.toml > f64.txt
//! 再以 --features twofloat 得 dd.txt，用 scripts/dev/compare_calendar_precision.py 比对。
use lunar_core::astronomy::aspects::{new_moon_jds_in_range, solar_term_jds_in_range};
use lunar_core::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use lunar_core::astronomy::ephemeris::load_earth_vsop87_from_repo;
use lunar_core::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use lunar_core::math::real::{real, RealOps};
use lunar_core::quantity::angle::PlaneAngle;

fn main() {
    let _ = try_init_full_nutation_from_repo();
    let elp = load_all(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
    let earth = load_earth_vsop87_from_repo().unwrap();
    let tolerance = PlaneAngle::from_deg(real(1e-9));
    // 前 3000—公元 3000 每 500 年取一年，每年 120 日（double-double 约 20 倍慢）
    for i in 0..=12 {
        let year = -3000 + 500 * i;
        let start = 2_451_545.0 + 365.2425 * (year as f64 - 2000.0);
        let (a, b) = (real(start), real(start + 120.0));
        for jd in new_moon_jds_in_range(&earth, &elp, a, b, tolerance, 40) {
            println!("{year} 朔 - {:.10}", jd.as_f64());
        }
        let mut terms = solar_term_jds_in_range(&earth, a, b, tolerance, 40);
        terms.sort_by(|x, y| x.0.cmp(&y.0));
        for (k, jd) in terms {
            println!("{year} 气 {k} {:.10}", jd.as_f64());
        }
    }
}
