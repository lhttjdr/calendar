//! 当地平太阳时与视太阳时（真太阳时），供时柱按当地太阳时划分时辰。
//!
//! - 平太阳时 = UT + 东经/15°（小时）；
//! - 视太阳时 = 12 时 + 太阳时角，时角 H = GAST + 东经 − 太阳视赤经（当日真赤道）。
//!   两者之差即均时差，一年内约 −14 至 +16 分钟。
//!
//! 《協紀辨方書》卷三十四以太阳到方位的真实时刻取时，即视太阳时。

use super::equatorial::sun_apparent_equatorial;
use super::longitude::ApparentPipelineOptions;
use super::topocentric::right_ascension_declination;
use crate::astronomy::ephemeris::Vsop87;
use crate::astronomy::time::earth_rotation::gast;
use crate::astronomy::time::{ut1_to_tt_jd, TimePoint, TimeScale};
use crate::math::real::{real_const, Real, RealOps};
use crate::quantity::angle::PlaneAngle;

/// 当地平太阳时（小时，[0, 24)）。`jd_ut` 为 UT1 儒略日，`longitude` 为东经。
pub fn local_mean_solar_hours(jd_ut: Real, longitude: PlaneAngle) -> Real {
    let day = jd_ut + real_const(0.5) + longitude.rad() / (real_const(2.0) * Real::pi());
    (day - day.floor()) * real_const(24.0)
}

/// 当地视太阳时（真太阳时，小时，[0, 24)）。
pub fn local_apparent_solar_hours(vsop: &Vsop87, jd_ut: Real, longitude: PlaneAngle) -> Real {
    let jd_tt = ut1_to_tt_jd(jd_ut);
    let sun = sun_apparent_equatorial(
        vsop,
        TimePoint::new(TimeScale::TT, jd_tt),
        &ApparentPipelineOptions::pipeline_default(),
    );
    let (ra, _) = right_ascension_declination(&sun);
    let h = PlaneAngle::from_rad(gast(jd_ut, jd_tt).rad() + longitude.rad() - ra.rad()).wrap_to_2pi();
    let hours = h.rad() / (real_const(2.0) * Real::pi()) * real_const(24.0) + real_const(12.0);
    hours - (hours / real_const(24.0)).floor() * real_const(24.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::ephemeris::load_earth_vsop87_from_repo;
    use crate::calendar::gregorian::Gregorian;
    use crate::math::real::real;

    /// 均时差：2 月中旬视太阳时比平太阳时慢约 14 分，11 月初快约 16 分（Meeus《天文算法》第 28 章量级）。
    #[test]
    fn equation_of_time_extremes() {
        let vsop = load_earth_vsop87_from_repo().unwrap();
        let lon = PlaneAngle::from_deg(real(0.0));
        let eot_min = |y, m, d| {
            let jd = Gregorian::to_julian_day(y, m, d) + real(0.5); // 世界时正午
            ((local_apparent_solar_hours(&vsop, jd, lon) - local_mean_solar_hours(jd, lon)) * real(60.0)).as_f64()
        };
        let feb = eot_min(2024, 2, 11);
        let nov = eot_min(2024, 11, 3);
        println!("均时差：2024-02-11 {feb:.2} 分，2024-11-03 {nov:.2} 分");
        assert!((feb + 14.2).abs() < 0.5, "{feb}");
        assert!((nov - 16.4).abs() < 0.5, "{nov}");
    }
}
