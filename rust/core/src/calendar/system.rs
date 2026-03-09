//! 历法系统抽象：基于儒略日的双向换算基于儒略日的双向换算。

use super::chinese_lunar::{from_julian_day_in_year, to_julian_day, ChineseLunarDate, ChineseLunarYearData};
use super::gregorian::Gregorian;
use crate::math::real::Real;

/// 历法系统：日期类型与上下文，JD 统一为 Real，不在 core 内使用 f64。
pub trait CalendarSystem {
    type Date;
    type Context;

    fn to_julian_day(&self, date: &Self::Date, ctx: &Self::Context) -> Option<Real>;
    fn from_julian_day(&self, jd: Real, ctx: &Self::Context) -> Option<Self::Date>;
}

/// 公历系统：Date = (年, 月, 日)，Context = ()
impl CalendarSystem for Gregorian {
    type Date = (i32, i32, i32);
    type Context = ();

    fn to_julian_day(&self, date: &Self::Date, _ctx: &Self::Context) -> Option<Real> {
        Some(Gregorian::to_julian_day(date.0, date.1, date.2))
    }

    fn from_julian_day(&self, jd: Real, _ctx: &Self::Context) -> Option<Self::Date> {
        Some(Gregorian::from_julian_day(jd))
    }
}

/// 农历系统：需岁数据作为 Context
pub struct ChineseLunar;

impl CalendarSystem for ChineseLunar {
    type Date = ChineseLunarDate;
    type Context = ChineseLunarYearData;

    fn to_julian_day(&self, date: &Self::Date, ctx: &Self::Context) -> Option<Real> {
        to_julian_day(*date, ctx)
    }

    fn from_julian_day(&self, jd: Real, ctx: &Self::Context) -> Option<Self::Date> {
        from_julian_day_in_year(jd, ctx, None)
    }
}

/// 通过儒略日将一种历法日期转换为另一种
pub fn convert<Cal1, Cal2>(
    from_cal: &Cal1,
    to_cal: &Cal2,
    date: &Cal1::Date,
    from_ctx: &Cal1::Context,
    to_ctx: &Cal2::Context,
) -> Option<Cal2::Date>
where
    Cal1: CalendarSystem,
    Cal2: CalendarSystem,
{
    let jd = from_cal.to_julian_day(date, from_ctx)?;
    to_cal.from_julian_day(jd, to_ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gregorian_calendar_system_roundtrip() {
        let cal = Gregorian;
        let date = (2000, 1, 1);
        let jd = cal.to_julian_day(&date, &()).unwrap();
        let back = cal.from_julian_day(jd, &()).unwrap();
        assert_eq!(date, back);
    }

    #[test]
    fn convert_gregorian_to_gregorian() {
        let cal = Gregorian;
        let date = (2025, 6, 15);
        let out: Option<(i32, i32, i32)> = convert(&cal, &cal, &date, &(), &());
        assert_eq!(out, Some(date));
    }
}
