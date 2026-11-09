//! 民俗各派所取的干支历选项（仲裁层：只选历法域的选项，不另算）。
//!
//! 年界、月界、闰月、日界诸选项本身属历法域（`calendar::gan_zhi_options`）；
//! 「哪一派取哪一套」是民俗域的知识，放在这里。出处见 `doc/folk/ganzhi-calendar.md`。

use crate::calendar::gan_zhi_options::{
    preset_jieqi_ganzhi, DayBoundary, GanzhiOptions, LeapMonthHandling, MonthBoundary,
    SolarTermMode, YearBoundary,
};

/// 子平八字（子正换日）：立春换年、十二节换月、子正换日，即节气干支历。
/// 依据《三命通會》卷二、《星平大成》（经《命理探源》卷五转引）、《命理探源》卷二。
/// 年、月两项与《協紀辨方書》择日相同。
pub fn zi_ping_ba_zi() -> GanzhiOptions {
    preset_jieqi_ganzhi()
}

/// 子平八字（子初换日）：同上，唯 23 时即换日。今人多从，原典未见。
pub fn zi_ping_zi_chu() -> GanzhiOptions {
    GanzhiOptions {
        day_boundary: DayBoundary::Hour23,
        ..zi_ping_ba_zi()
    }
}

/// 紫微斗数：农历年月起盘、闰月整月作下月（《紫微斗數全書》卷三《安身命例》）。
/// 书中未言年界、日界：年取正月初一，日取子正。
pub fn purple_star() -> GanzhiOptions {
    GanzhiOptions {
        year_boundary: YearBoundary::LunarNewYear,
        month_boundary: MonthBoundary::LunarFirstDay,
        leap_month_handling: LeapMonthHandling::ShiftToNext,
        day_boundary: DayBoundary::Hour0,
        solar_term_mode: SolarTermMode::Dingqi,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets() {
        let zp = zi_ping_ba_zi();
        assert_eq!(zp.year_boundary, YearBoundary::LiChun);
        assert_eq!(zp.month_boundary, MonthBoundary::SolarTerm);
        assert_eq!(zp.day_boundary, DayBoundary::Hour0);

        let zc = zi_ping_zi_chu();
        assert_eq!(zc.year_boundary, YearBoundary::LiChun);
        assert_eq!(zc.day_boundary, DayBoundary::Hour23);

        let ps = purple_star();
        assert_eq!(ps.month_boundary, MonthBoundary::LunarFirstDay);
        assert_eq!(ps.leap_month_handling, LeapMonthHandling::ShiftToNext);
        assert_eq!(ps.day_boundary, DayBoundary::Hour0);
    }
}
