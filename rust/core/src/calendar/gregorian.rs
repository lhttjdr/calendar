//! 公历：(年, 月, 日)，月与日从 1 起，年为天文纪年（公元前 1 年 = 0）。Meeus 算法。
//!
//! 两种约定，不可混用：
//! - **前推格里高利历**（proleptic Gregorian）：[to_julian_day]/[from_julian_day]/[Gregorian]，任何年份都按格里高利历置闰；
//! - **民用历**（儒略历 + 格里高利历）：[civil_to_julian_day]/[civil_from_julian_day]，1582-10-15 以前为儒略历、
//!   以后为格里高利历。史料换算、`实历朔日表`、改元日表、NASA 日月食典等都用这一约定。

use crate::math::real::{real_const, from_i32, real, Real, RealOps};

/// 儒略日（0h UT）转公历 (年, 月, 日)。中间量用 Real，仅取整时 to_i32_floor。
#[inline]
pub fn from_julian_day(jd: Real) -> (i32, i32, i32) {
    let j = (jd + real_const(0.5)).floor().to_i32_floor();
    let a = ((real(j) - real_const(1867216.25)) / real_const(36524.25)).to_i32_floor();
    let b = j + 1 + a - a / 4;
    let c = b + 1524;
    let d = ((real(c) - real_const(122.1)) / real_const(365.25)).to_i32_floor();
    let e = (real_const(365.25) * real(d)).to_i32_floor();
    let f = (real(c - e) / real_const(30.6001)).to_i32_floor();
    let day = c - e - (real_const(30.6001) * real(f)).to_i32_floor();
    let month = if f < 14 { f - 1 } else { f - 13 };
    let year = if month > 2 { d - 4716 } else { d - 4715 };
    (year, month, day)
}

/// 公历 (年, 月, 日) 转儒略日（0h UT）
#[inline]
pub fn to_julian_day(year: i32, month: i32, day: i32) -> Real {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    let jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    from_i32(jdn) - real(0.5)
}

/// 儒略历 (年, 月, 日) → 儒略日数（JDN，该日正午的整数儒略日）。
#[inline]
pub fn julian_calendar_to_jdn(year: i32, month: i32, day: i32) -> i64 {
    let a = (14 - month) / 12;
    let y = (year + 4800 - a) as i64;
    let m = (month + 12 * a - 3) as i64;
    day as i64 + (153 * m + 2) / 5 + 365 * y + y.div_euclid(4) - 32083
}

/// 前推格里高利历 (年, 月, 日) → 儒略日数（JDN）。
#[inline]
pub fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i64 {
    let a = (14 - month) / 12;
    let y = (year + 4800 - a) as i64;
    let m = (month + 12 * a - 3) as i64;
    day as i64 + (153 * m + 2) / 5 + 365 * y + y.div_euclid(4) - y.div_euclid(100) + y.div_euclid(400) - 32045
}

/// 儒略历/格里高利历改历日：格里高利历 1582-10-15 的 JDN（前一日为儒略历 1582-10-04）。
pub const GREGORIAN_REFORM_JDN: i64 = 2299161;

/// 民用历 (年, 月, 日) → JDN：1582-10-15 以前按儒略历，以后按格里高利历。
#[inline]
pub fn civil_to_jdn(year: i32, month: i32, day: i32) -> i64 {
    if (year, month, day) >= (1582, 10, 15) {
        gregorian_to_jdn(year, month, day)
    } else {
        julian_calendar_to_jdn(year, month, day)
    }
}

/// 民用历 (年, 月, 日) → 儒略日（该日 0h UT）。
#[inline]
pub fn civil_to_julian_day(year: i32, month: i32, day: i32) -> Real {
    from_i32(civil_to_jdn(year, month, day) as i32) - real(0.5)
}

/// JDN → 民用历 (年, 月, 日)（Meeus《天文算法》第 7 章：JDN < 2299161 为儒略历）。
pub fn civil_from_jdn(jdn: i64) -> (i32, i32, i32) {
    let a = if jdn < GREGORIAN_REFORM_JDN {
        jdn
    } else {
        let alpha = (4 * jdn - 7468865).div_euclid(146097);
        jdn + 1 + alpha - alpha.div_euclid(4)
    };
    let b = a + 1524;
    let c = (20 * b - 2442).div_euclid(7305); // floor((B − 122.1) / 365.25)
    let d = (1461 * c).div_euclid(4);
    let e = ((b - d) * 10000).div_euclid(306001);
    let day = b - d - (306001 * e).div_euclid(10000);
    let month = if e < 14 { e - 1 } else { e - 13 };
    let year = if month > 2 { c - 4716 } else { c - 4715 };
    (year as i32, month as i32, day as i32)
}

/// 儒略日（0h UT 起算的该日任意时刻）→ 民用历 (年, 月, 日)。
pub fn civil_from_julian_day(jd: Real) -> (i32, i32, i32) {
    civil_from_jdn((jd + real_const(0.5)).floor().to_i32_floor() as i64)
}

/// 公历 (年, 月, 日) 视为北京时间，转儒略日（该日 0h 北京时间对应的 UTC 时刻）
#[inline]
pub fn to_julian_day_0h_utc8(year: i32, month: i32, day: i32) -> Real {
    let jd = to_julian_day(year, month, day);
    jd - real_const(8.0 / 24.0)
}

/// 公历 (年, 月, 日) 按 UTC+8 日界，与农历一致。日号 = JDN(0h UT 该日)，与 JD→日号 同数轴。
#[inline]
pub fn day_number_utc8(year: i32, month: i32, day: i32) -> i64 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    let jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    jdn as i64
}

/// 公历历法接口（对标 CalendarSystem）
pub struct Gregorian;

impl Gregorian {
    pub fn from_julian_day(jd: Real) -> (i32, i32, i32) {
        from_julian_day(jd)
    }

    pub fn to_julian_day(year: i32, month: i32, day: i32) -> Real {
        to_julian_day(year, month, day)
    }

    pub fn to_julian_day_0h_utc8(year: i32, month: i32, day: i32) -> Real {
        to_julian_day_0h_utc8(year, month, day)
    }

    /// 公历 (年,月,日) 的日号（整数，不经 JD），与 JD→日号 同数轴。
    pub fn day_number_utc8(year: i32, month: i32, day: i32) -> i64 {
        day_number_utc8(year, month, day)
    }

    /// 公历某月天数（年、月从 1 起）
    pub fn days_in_month(year: i32, month: i32) -> u8 {
        match month {
            2 => {
                let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
                if leap { 29 } else { 28 }
            }
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::RealOps;

    /// Gregorian round-trip (2000,1,1)
    #[test]
    fn gregorian_roundtrip_2000_1_1() {
        let (y, m, d) = (2000, 1, 1);
        let jd = to_julian_day(y, m, d);
        let (y2, m2, d2) = from_julian_day(jd);
        assert_eq!((y2, m2, d2), (y, m, d));
    }

    /// 2000-01-01 0h UT = JD 2451544.5
    #[test]
    fn gregorian_2000_01_01_jd_2451544_5() {
        let jd = to_julian_day(2000, 1, 1);
        assert!(jd.is_near(real(2451544.5), 1e-6), "expected 2451544.5, got {}", jd.as_f64());
    }

    /// 民用历：改历前后两日相邻；儒略历日期（史料约定）与前推格里高利历相差 −3（公元前 100 年前后）至 +10（1582 年）日。
    #[test]
    fn civil_calendar_reform_and_offsets() {
        assert_eq!(civil_to_jdn(1582, 10, 4) + 1, civil_to_jdn(1582, 10, 15));
        assert_eq!(civil_from_jdn(2299160), (1582, 10, 4));
        assert_eq!(civil_from_jdn(2299161), (1582, 10, 15));
        // Meeus 例 7.a、7.b：1957-10-04.81 = JD 2436116.31；333-01-27 12h（儒略历）= JD 1842713.0
        assert_eq!(civil_to_jdn(1957, 10, 4), 2436116);
        assert_eq!(civil_to_jdn(333, 1, 27), 1842713);
        // Meeus 第 7 章表例：−584-05-28.63（天文纪年）= JD 1507900.13，即该日 JDN 1507900
        assert_eq!(civil_to_jdn(-584, 5, 28), 1507900);
        assert_eq!(civil_to_jdn(9, 1, 15) - gregorian_to_jdn(9, 1, 15), -2);
        assert_eq!(julian_calendar_to_jdn(1582, 10, 4) - gregorian_to_jdn(1582, 10, 4), 10);
        for jdn in (1_000_000..2_500_000).step_by(997) {
            let (y, m, d) = civil_from_jdn(jdn);
            assert_eq!(civil_to_jdn(y, m, d), jdn, "{y}-{m}-{d}");
        }
    }

    #[test]
    fn roundtrip_j2000() {
        let jd = real(2451545.0); // 2000-01-01 12h UT 附近
        let (y, m, d) = from_julian_day(jd);
        let jd2 = to_julian_day(y, m, d);
        assert!(jd.is_near(jd2, 1.0), "{} {} {} -> {}", y, m, d, jd2.as_f64());
    }
}
