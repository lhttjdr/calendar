//! 公历（proleptic Gregorian），(年, 月, 日)，月与日从 1 起。
//! Meeus 算法。

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

    #[test]
    fn roundtrip_j2000() {
        let jd = real(2451545.0); // 2000-01-01 12h UT 附近
        let (y, m, d) = from_julian_day(jd);
        let jd2 = to_julian_day(y, m, d);
        assert!(jd.is_near(jd2, 1.0), "{} {} {} -> {}", y, m, d, jd2.as_f64());
    }
}
