//! 天文时间尺度与时间点。标量用 math::Real。

use crate::astronomy::constant::{J2000, JULIAN_MILLENNIUM};
use crate::math::real::{real_const, from_i32, Real, RealOps};
use crate::quantity::duration::Duration;

/// 时标：TT、TDB 等；换算至 UTC/UT1 需 TimeScaleContext（暂未实现）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeScale {
    TT,
    TAI,
    TDB,
    UTC,
    UT1,
    GPST,
}

/// 秒/日
pub const SECONDS_PER_DAY: Real = real_const(86400.0);
/// TT = TAI + 32.184 s（非 const：底层除法非 const）
#[inline]
pub fn tt_tai_offset_days() -> Real {
    real_const(32.184) / SECONDS_PER_DAY
}

/// JD → MJD（MJD = JD − 2400000.5）
#[inline]
pub fn jd_to_mjd(jd: Real) -> Real {
    jd - real_const(2400000.5)
}

/// 闰秒表：(MJD_UTC, TAI−UTC 秒)；与 IERS Bulletin 一致，更晚日期沿用 37 秒
const LEAP_SECONDS_TABLE: &[(Real, i32)] = &[
    (real_const(41317.0), 10),   // 1972-01-01
    (real_const(41499.0), 11),   // 1972-07-01
    (real_const(41683.0), 12),   // 1973-01-01
    (real_const(42048.0), 13),   // 1974-01-01
    (real_const(42413.0), 14),   // 1975-01-01
    (real_const(42778.0), 15),   // 1976-01-01
    (real_const(43144.0), 16),   // 1977-01-01
    (real_const(43509.0), 17),   // 1978-01-01
    (real_const(43874.0), 18),   // 1979-01-01
    (real_const(44239.0), 19),   // 1980-01-01
    (real_const(44786.0), 20),   // 1981-07-01
    (real_const(45151.0), 21),   // 1982-07-01
    (real_const(45516.0), 22),   // 1983-07-01
    (real_const(46247.0), 23),   // 1985-07-01
    (real_const(47161.0), 24),   // 1988-01-01
    (real_const(47892.0), 25),   // 1990-01-01
    (real_const(48257.0), 26),   // 1991-01-01
    (real_const(48804.0), 27),   // 1992-07-01
    (real_const(49169.0), 28),   // 1993-07-01
    (real_const(49534.0), 29),   // 1994-07-01
    (real_const(50083.0), 30),   // 1996-01-01
    (real_const(50630.0), 31),   // 1997-07-01
    (real_const(51179.0), 32),   // 1999-01-01
    (real_const(53736.0), 33),   // 2006-01-01
    (real_const(54832.0), 34),   // 2009-01-01
    (real_const(56109.0), 35),   // 2012-07-01
    (real_const(57204.0), 36),   // 2015-07-01
    (real_const(57754.0), 37),   // 2017-01-01
];

/// 给定 UTC 儒略日，查该时刻的 TAI − UTC（秒）
pub fn tai_minus_utc_at_utc_jd(jd_utc: Real) -> i32 {
    let mjd = jd_to_mjd(jd_utc);
    if mjd < LEAP_SECONDS_TABLE[0].0 {
        return 0;
    }
    let idx = LEAP_SECONDS_TABLE
        .iter()
        .rposition(|(m, _)| *m <= mjd)
        .unwrap_or(0);
    LEAP_SECONDS_TABLE[idx].1
}

/// 给定 TAI 儒略日，返回该时刻的 TAI − UTC（秒）；迭代查表
pub fn leap_seconds_at_tai(jd_tai: Real) -> i32 {
    let mjd_tai = jd_to_mjd(jd_tai);
    let mut leap = 0i32;
    for _ in 0..20 {
        let mjd_utc = mjd_tai - from_i32(leap) / SECONDS_PER_DAY;
        let jd_utc = mjd_utc + real_const(2400000.5);
        let next = tai_minus_utc_at_utc_jd(jd_utc);
        if next == leap {
            return leap;
        }
        leap = next;
    }
    leap
}

/// UTC 儒略日 → TAI 儒略日
#[inline]
pub fn utc_to_tai_jd(jd_utc: Real) -> Real {
    jd_utc + from_i32(tai_minus_utc_at_utc_jd(jd_utc)) / SECONDS_PER_DAY
}

/// TAI 儒略日 → UTC 儒略日
pub fn tai_to_utc_jd(jd_tai: Real) -> Real {
    let leap = leap_seconds_at_tai(jd_tai);
    jd_tai - from_i32(leap) / SECONDS_PER_DAY
}

/// 时长（日为单位，可正可负）。
#[derive(Clone, Copy, Debug)]
pub struct TimeInterval {
    days: Real,
}

impl TimeInterval {
    pub fn from_days(days: Real) -> Self {
        Self { days }
    }
    pub fn from_seconds(seconds: Real) -> Self {
        Self {
            days: seconds / SECONDS_PER_DAY,
        }
    }
    pub fn in_days(self) -> Real {
        self.days
    }
    pub fn in_seconds(self) -> Real {
        self.days * SECONDS_PER_DAY
    }
}

/// 时间点：某时标下的儒略日。
#[derive(Clone, Copy, Debug)]
pub struct TimePoint {
    pub scale: TimeScale,
    pub jd: Real,
}

impl TimePoint {
    pub fn new(scale: TimeScale, jd: Real) -> Self {
        Self { scale, jd }
    }

    /// TT → TDB（近似，周期项）。内部用 f64 算，结果转 R。
    fn tt_to_tdb(jd_tt: Real) -> Real {
        let t = (jd_tt - J2000) / real_const(36525.0);
        let g = 2.0 * core::f64::consts::PI * (357.528 + 35999.050 * t) / 360.0;
        let delta_sec = 0.001658 * (g + 0.0167 * g.sin()).sin();
        jd_tt + delta_sec / SECONDS_PER_DAY
    }

    /// TDB → TT
    fn tdb_to_tt(jd_tdb: Real) -> Real {
        let t = (jd_tdb - J2000) / real_const(36525.0);
        let g = 2.0 * core::f64::consts::PI * (357.528 + 35999.050 * t) / 360.0;
        let delta_sec = 0.001658 * (g + 0.0167 * g.sin()).sin();
        jd_tdb - delta_sec / SECONDS_PER_DAY
    }

    /// 换算到目标时标。
    pub fn to_scale(self, target: TimeScale) -> TimePoint {
        if self.scale == target {
            return self;
        }
        let jd_f64 = self.jd;
        let result_f64 = match (self.scale, target) {
            (TimeScale::TT, TimeScale::TDB) => Self::tt_to_tdb(jd_f64),
            (TimeScale::TDB, TimeScale::TT) => Self::tdb_to_tt(jd_f64),
            (TimeScale::TT, TimeScale::TAI) => jd_f64 - tt_tai_offset_days(),
            (TimeScale::TAI, TimeScale::TT) => jd_f64 + tt_tai_offset_days(),
            (TimeScale::TAI, TimeScale::UTC) => tai_to_utc_jd(jd_f64),
            (TimeScale::UTC, TimeScale::TAI) => utc_to_tai_jd(jd_f64),
            (TimeScale::TT, TimeScale::UTC) => tai_to_utc_jd(jd_f64 - tt_tai_offset_days()),
            (TimeScale::UTC, TimeScale::TT) => utc_to_tai_jd(jd_f64) + tt_tai_offset_days(),
            (TimeScale::TDB, TimeScale::UTC) => {
                let jd_tt = Self::tdb_to_tt(jd_f64);
                tai_to_utc_jd(jd_tt - tt_tai_offset_days())
            }
            (TimeScale::UTC, TimeScale::TDB) => {
                let jd_tt = utc_to_tai_jd(jd_f64) + tt_tai_offset_days();
                Self::tt_to_tdb(jd_tt)
            }
            (TimeScale::TT, TimeScale::UT1) => jd_f64 - delta_t(jd_f64).seconds() / SECONDS_PER_DAY,
            (TimeScale::UT1, TimeScale::TT) => ut1_to_tt_jd(jd_f64),
            (TimeScale::TAI, TimeScale::GPST) => jd_f64 - real_const(19.0) / SECONDS_PER_DAY,
            (TimeScale::GPST, TimeScale::TAI) => jd_f64 + real_const(19.0) / SECONDS_PER_DAY,
            (TimeScale::TT, TimeScale::GPST) => {
                let tai = jd_f64 - tt_tai_offset_days();
                tai - real_const(19.0) / SECONDS_PER_DAY
            }
            (TimeScale::GPST, TimeScale::TT) => {
                let tai = jd_f64 + real_const(19.0) / SECONDS_PER_DAY;
                tai + tt_tai_offset_days()
            }
            _ => jd_f64,
        };
        TimePoint::new(target, result_f64)
    }

    /// 返回 TDB 儒略日（历表约定）
    pub fn jd_tdb(self) -> Real {
        self.to_scale(TimeScale::TDB).jd
    }

    /// 加上时长，保持时标
    pub fn plus(self, d: TimeInterval) -> TimePoint {
        TimePoint::new(self.scale, self.jd + d.days)
    }

    /// 减去另一时间点，得到时长（仅同时标有效）
    pub fn minus(self, other: TimePoint) -> TimeInterval {
        assert_eq!(self.scale, other.scale, "minus 仅支持相同时标");
        TimeInterval::from_days(self.jd - other.jd)
    }
}

/// 历元与时刻的对应关系：历元约定为 TT 儒略日，故可互转。
impl From<crate::quantity::epoch::Epoch> for TimePoint {
    fn from(epoch: crate::quantity::epoch::Epoch) -> Self {
        TimePoint::new(TimeScale::TT, epoch.jd)
    }
}

impl From<TimePoint> for crate::quantity::epoch::Epoch {
    fn from(t: TimePoint) -> Self {
        crate::quantity::epoch::Epoch::new(t.to_scale(TimeScale::TT).jd)
    }
}

/// J2000.0 在 TT 下为 JD 2451545.0
pub fn j2000_tt() -> TimePoint {
    TimePoint::new(TimeScale::TT, J2000)
}

// -----------------------------------------------------------------------------
// 儒略日与归一化时间（历表通用：VSOP87、岁差等）。不写死 f64。
// -----------------------------------------------------------------------------

/// 儒略日（日），用于历表时刻。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JulianDay(pub Real);

/// 无量纲 T = (JD − J2000) / 365250，儒略千年数，常用作历表级数自变量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JulianMillennia(pub Real);

/// JD → T（儒略千年）。
#[inline]
pub fn jd_to_t(jd: Real) -> JulianMillennia {
    JulianMillennia((jd - J2000) / JULIAN_MILLENNIUM)
}

/// 从 TimePoint 转 T（儒略千年）；历表约定用 TDB。
#[inline]
pub fn time_point_to_t_julian_millennia(t: TimePoint) -> JulianMillennia {
    jd_to_t(t.jd_tdb())
}

// -----------------------------------------------------------------------------
// ΔT = TT − UT1（秒），NASA 多项式（Five Millennium Canon of Solar Eclipses）
// 参考：https://eclipse.gsfc.nasa.gov/SEhelp/deltatpoly2004.html
// -----------------------------------------------------------------------------

/// JD(TT) → 十进制年（用于 ΔT 多项式）
#[inline]
pub fn jd_to_decimal_year(jd: Real) -> Real {
    real_const(2000.0) + (jd - J2000) / real_const(365.25)
}

/// ΔT = TT − UT1。jd 为 TT 儒略日；适用约 -1999 ～ +3000 年。
pub fn delta_t(jd_tt: Real) -> Duration {
    let y = jd_to_decimal_year(jd_tt);
    Duration::in_seconds(delta_t_eval(y))
}

fn delta_t_eval(y: Real) -> Real {
    if y < -500.0 {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    } else if y <= 500.0 {
        let u = y / 100.0;
        10583.6 - 1014.41 * u + 33.78311 * u * u - 5.952053 * u * u * u
            - 0.1798452 * u * u * u * u + 0.022174192 * u * u * u * u * u
            + 0.0090316521 * u * u * u * u * u * u
    } else if y <= 1600.0 {
        let u = (y - 1000.0) / 100.0;
        1574.2 - 556.01 * u + 71.23472 * u * u + 0.319781 * u * u * u
            - 0.8503463 * u * u * u * u - 0.005050998 * u * u * u * u * u
            + 0.0083572073 * u * u * u * u * u * u
    } else if y < 1700.0 {
        let t = y - 1600.0;
        120.0 - 0.9808 * t - 0.01532 * t * t + t * t * t / 7129.0
    } else if y < 1800.0 {
        let t = y - 1700.0;
        8.83 + 0.1603 * t - 0.0059285 * t * t + 0.00013336 * t * t * t
            - t * t * t * t / 1174000.0
    } else if y < 1860.0 {
        let t = y - 1800.0;
        13.72 - 0.332447 * t + 0.0068612 * t * t + 0.0041116 * t * t * t
            - 0.00037436 * t * t * t * t + 0.0000121272 * t * t * t * t * t
            - 0.0000001699 * t * t * t * t * t * t
            + 0.000000000875 * t * t * t * t * t * t * t * t
    } else if y < 1900.0 {
        let t = y - 1860.0;
        7.62 + 0.5737 * t - 0.251754 * t * t + 0.01680668 * t * t * t
            - 0.0004473624 * t * t * t * t + t * t * t * t * t / 233174.0
    } else if y < 1920.0 {
        let t = y - 1900.0;
        -2.79 + 1.494119 * t - 0.0598939 * t * t + 0.0061966 * t * t * t
            - 0.000197 * t * t * t * t
    } else if y < 1941.0 {
        let t = y - 1920.0;
        21.20 + 0.84493 * t - 0.076100 * t * t + 0.0020936 * t * t * t
    } else if y < 1961.0 {
        let t = y - 1950.0;
        29.07 + 0.407 * t - t * t / 233.0 + t * t * t / 2547.0
    } else if y < 1986.0 {
        let t = y - 1975.0;
        45.45 + 1.067 * t - t * t / 260.0 - t * t * t / 718.0
    } else if y < 2005.0 {
        let t = y - 2000.0;
        63.86 + 0.3345 * t - 0.060374 * t * t + 0.0017275 * t * t * t
            + 0.000651814 * t * t * t * t + 0.00002373599 * t * t * t * t * t
    } else if y <= 2050.0 {
        let t = y - 2000.0;
        62.92 + 0.32217 * t + 0.005589 * t * t
    } else if y < 2150.0 {
        -20.0 + 32.0 * ((y - 1820.0) / 100.0).powi(2) - 0.5628 * (2150.0 - y)
    } else {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

/// JD(UT1) → JD(TT)：jd_ut1 + ΔT/86400
#[inline]
pub fn ut1_to_tt_jd(jd_ut1: Real) -> Real {
    let jd_tt_approx = jd_ut1 + 69.0 / SECONDS_PER_DAY; // 近似 ΔT 用于求 ΔT
    jd_ut1 + delta_t(jd_tt_approx).seconds() / SECONDS_PER_DAY
}

// -----------------------------------------------------------------------------
// Stephenson–Morrison ΔT 外推（文献：Stephenson et al 2016, Morrison et al 2021）
// 用于未来年代 TT−UT1 近似；可与 delta_t 在 y > 2050 时择一使用。
// -----------------------------------------------------------------------------

/// 由 JD 得十进制年（与 PDF 式一致）：y = (JD − 2451544.5)/365.2425 + 2000
#[inline]
pub fn stephenson_morrison_year_from_jd(jd: Real) -> Real {
    (jd - real_const(2451544.5)) / real_const(365.2425) + real_const(2000.0)
}

/// ΔT，Stephenson–Morrison 外推公式；y 为十进制年。返回 Duration，与 delta_t 一致。
pub fn stephenson_morrison_delta_t_seconds(y: Real) -> Duration {
    let t = (y - real_const(1825.0)) / real_const(100.0);
    let f = real_const(31.4115) * t * t + real_const(284.8436) * (real_const(2.0 * core::f64::consts::PI) * (t + real_const(0.75)) / real_const(14.0)).cos();
    let c2 = real_const(-150.568);
    let term = (y / real_const(100.0) - real_const(19.55)) * (y / real_const(100.0) - real_const(19.55)) - real_const(0.49);
    Duration::in_seconds(c2 + f + real_const(0.1056) * term)
}

/// 由 TT 儒略日得 ΔT，Stephenson–Morrison 外推。返回 Duration。
#[inline]
pub fn stephenson_morrison_delta_t_from_jd(jd_tt: Real) -> Duration {
    stephenson_morrison_delta_t_seconds(stephenson_morrison_year_from_jd(jd_tt))
}

#[cfg(test)]
mod tests_stephenson_morrison {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn stephenson_morrison_year_from_jd_j2000() {
        let y = stephenson_morrison_year_from_jd(J2000);
        assert!(y.is_near(real_const(2000.0), 0.01));
    }

    #[test]
    fn stephenson_morrison_delta_t_future() {
        let dt_2050 = stephenson_morrison_delta_t_seconds(real_const(2050.0));
        let dt_2100 = stephenson_morrison_delta_t_seconds(real_const(2100.0));
        assert!(dt_2050.seconds() > real(0) && dt_2100.seconds() > real(0));
        assert!(dt_2100.seconds() > dt_2050.seconds());
    }
}

#[cfg(test)]
mod tests_delta_t {
    use super::*;
    use crate::math::real::real;

    #[test]
    fn delta_t_j2000() {
        let dt = delta_t(J2000);
        assert!(dt.seconds() > real(60.0) && dt.seconds() < real(70.0), "ΔT near 2000 ~ 64s");
    }
}

#[cfg(test)]
mod tests_leap_seconds {
    use super::*;
    use crate::math::real::RealOps;

    #[test]
    fn leap_2017() {
        let jd_utc = real_const(2457754.5); // 2017-01-01 0h UTC
        assert_eq!(tai_minus_utc_at_utc_jd(jd_utc), 37);
    }

    #[test]
    fn utc_tai_roundtrip() {
        let jd_utc = real_const(2457754.5);
        let jd_tai = utc_to_tai_jd(jd_utc);
        let back = tai_to_utc_jd(jd_tai);
        assert!(jd_utc.is_near(back, 1e-9));
    }

    #[test]
    fn tt_utc_roundtrip() {
        let t_tt = TimePoint::new(TimeScale::TT, J2000);
        let t_utc = t_tt.to_scale(TimeScale::UTC);
        let back = t_utc.to_scale(TimeScale::TT);
        assert!(t_tt.jd.is_near(back.jd, 1e-8));
    }
}

#[cfg(test)]
mod tests_time_point {
    use super::*;
    use crate::math::real::{real, real_const, RealOps};

    #[test]
    fn time_point_tt_and_tai_differ_by_tt_tai_offset_days() {
        let jd = J2000;
        let tt = TimePoint::new(TimeScale::TT, jd);
        let tai = tt.to_scale(TimeScale::TAI);
        assert_eq!(tai.scale, TimeScale::TAI);
        let expected_tai = jd - tt_tai_offset_days();
        assert!(tai.jd.is_near(expected_tai, 1e-12));
    }

    #[test]
    fn time_point_tt_to_tdb_then_back_to_tt() {
        let jd_tt = J2000;
        let tt = TimePoint::new(TimeScale::TT, jd_tt);
        let tdb = tt.to_scale(TimeScale::TDB);
        assert_eq!(tdb.scale, TimeScale::TDB);
        let back = tdb.to_scale(TimeScale::TT);
        assert!(back.jd.is_near(jd_tt, 1e-9));
    }

    #[test]
    fn time_point_plus_time_interval() {
        let tt = TimePoint::new(TimeScale::TT, J2000);
        let one_day = TimeInterval::from_days(real_const(1.0));
        let next = tt.plus(one_day);
        assert_eq!(next.scale, TimeScale::TT);
        assert!(next.jd.is_near(J2000 + real_const(1.0), 1e-12));
    }

    #[test]
    fn time_point_minus_same_scale_gives_time_interval() {
        let tt1 = TimePoint::new(TimeScale::TT, J2000);
        let tt2 = TimePoint::new(TimeScale::TT, J2000 + real_const(2.0));
        let d = tt2.minus(tt1);
        assert!(d.in_days().is_near(real(2.0), 1e-12));
    }

    #[test]
    fn time_point_minus_when_this_before_other_gives_negative_interval() {
        let tt1 = TimePoint::new(TimeScale::TT, J2000 + real_const(2.0));
        let tt2 = TimePoint::new(TimeScale::TT, J2000);
        let d = tt2.minus(tt1);
        assert!(d.in_days() < real(0), "应得到负时长");
        assert!(d.in_days().is_near(real(-2.0), 1e-12));
    }

    #[test]
    fn time_point_j2000_tt() {
        let j2k = j2000_tt();
        assert_eq!(j2k.scale, TimeScale::TT);
        assert!(j2k.jd.is_near(J2000, 1e-12));
    }

    #[test]
    fn time_interval_in_seconds_and_in_days() {
        let d = TimeInterval::from_seconds(real_const(86400.0));
        assert!(d.in_days().is_near(real(1.0), 1e-12));
        assert!(d.in_seconds().is_near(real(86400.0), 1e-9));
    }

    #[test]
    fn time_interval_allows_signed_duration() {
        let neg = TimeInterval::from_days(real_const(-1.0));
        assert!(neg.in_days() < real(0));
        assert!(neg.in_days().is_near(real(-1.0), 1e-12));
    }
}
