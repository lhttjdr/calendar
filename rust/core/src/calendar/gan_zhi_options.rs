//! 干支历选项与多套标准：换年/换月/闰月处理/换日，及预设（八字、紫微、黄历、协纪辨方）。

use crate::calendar::gan_zhi::{jd_to_gan_zhi_index, GAN_ZHI_60};
use crate::calendar::gregorian::Gregorian;
use crate::math::real::{real, Real, ToReal};

/// 换年干支
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum YearBoundary {
    /// 立春交节时刻（八字、风水等）
    LiChun,
    /// 正月初一零点（民俗、紫微等）
    #[default]
    LunarNewYear,
    /// 冬至交节时刻（奇门遁甲等，预留）
    WinterSolstice,
}

/// 换月干支
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum MonthBoundary {
    /// 十二节交节时刻（立春、惊蛰、清明…）
    SolarTerm,
    /// 农历每月初一零点
    #[default]
    LunarFirstDay,
}

/// 闰月干支处理（仅当换月=初一时有效）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum LeapMonthHandling {
    /// 按节气时此项不适用
    Ignore,
    /// 随前月（闰几月用几月干支，黄历常用）
    #[default]
    InheritPrevious,
    /// 月中切分：前 15 日本月，后 15 日下月（紫微古派）
    SplitMidway,
    /// 整个闰月作下月算（紫微部分现代派）
    ShiftToNext,
}

/// 换日干支
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DayBoundary {
    /// 子初换日 23:00（古典派）
    Hour23,
    /// 子正换日 00:00（现代派）
    #[default]
    Hour0,
}

/// 干支历完整配置
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GanzhiOptions {
    pub year_boundary: YearBoundary,
    pub month_boundary: MonthBoundary,
    pub leap_month_handling: LeapMonthHandling,
    pub day_boundary: DayBoundary,
}

impl Default for GanzhiOptions {
    fn default() -> Self {
        Self {
            year_boundary: YearBoundary::default(),
            month_boundary: MonthBoundary::default(),
            leap_month_handling: LeapMonthHandling::default(),
            day_boundary: DayBoundary::default(),
        }
    }
}

/// 子平八字/专业风水：立春换年、节气换月、子初换日
pub fn preset_zi_ping_ba_zi() -> GanzhiOptions {
    GanzhiOptions {
        year_boundary: YearBoundary::LiChun,
        month_boundary: MonthBoundary::SolarTerm,
        leap_month_handling: LeapMonthHandling::Ignore,
        day_boundary: DayBoundary::Hour23,
    }
}

/// 紫微斗数（中州派）：正月初一换年、初一换月、月中切分闰月、子正换日
pub fn preset_purple_star() -> GanzhiOptions {
    GanzhiOptions {
        year_boundary: YearBoundary::LunarNewYear,
        month_boundary: MonthBoundary::LunarFirstDay,
        leap_month_handling: LeapMonthHandling::SplitMidway,
        day_boundary: DayBoundary::Hour0,
    }
}

/// 民俗老黄历：正月初一换年、初一换月、闰月随前月、子初换日
pub fn preset_folk_almanac() -> GanzhiOptions {
    GanzhiOptions {
        year_boundary: YearBoundary::LunarNewYear,
        month_boundary: MonthBoundary::LunarFirstDay,
        leap_month_handling: LeapMonthHandling::InheritPrevious,
        day_boundary: DayBoundary::Hour23,
    }
}

/// 协纪辨方书（择吉）：立春换年、节气换月、子初换日
pub fn preset_xie_ji_bian_fang() -> GanzhiOptions {
    GanzhiOptions {
        year_boundary: YearBoundary::LiChun,
        month_boundary: MonthBoundary::SolarTerm,
        leap_month_handling: LeapMonthHandling::InheritPrevious,
        day_boundary: DayBoundary::Hour23,
    }
}

// ---------- 换日（仅依赖 JD） ----------

/// 按选项的日界计算干支日序。Hour0 = floor(JD+0.5)；Hour23 = 日提前 1 小时起算。JD 支持 Real。
#[inline]
pub fn jd_to_gan_zhi_index_with_options(jd: impl ToReal, day_boundary: DayBoundary) -> u8 {
    let jd_r = real(jd);
    let jd_for_day = match day_boundary {
        DayBoundary::Hour0 => jd_r,
        DayBoundary::Hour23 => jd_r - real(1.0 / 24.0),
    };
    jd_to_gan_zhi_index(jd_for_day)
}

/// 按选项的日界得到干支日名称。JD 支持 Real。
#[inline]
pub fn jd_to_gan_zhi_day_with_options(jd: impl ToReal, day_boundary: DayBoundary) -> &'static str {
    GAN_ZHI_60[jd_to_gan_zhi_index_with_options(jd, day_boundary) as usize]
}

// ---------- 五虎遁：年干支 + 月序(0..12) → 月干支序 ----------
// 寅月=0, 卯=1, …, 丑=11。甲己年丙作首，乙庚戊为头，丙辛寻庚上，丁壬壬寅流，戊癸甲寅求。

fn month_gan_zhi_from_year_and_ord(year_gan_zhi: u8, month_ord: u8) -> u8 {
    let stem = year_gan_zhi % 10;
    let yin_stem = (stem * 2 + 2) % 10;
    let month_stem = (yin_stem + month_ord) % 10;
    let month_branch = (month_ord + 2) % 12;
    (month_stem + 10 * ((month_branch + 12 - month_stem) % 12)) % 60
}

// ---------- 十二节在 24 节气中的序号（k=0 春分…）：立春21, 惊蛰23, 清明1, 立夏3, 芒种5, 小暑7, 立秋9, 白露11, 寒露13, 立冬15, 大雪17, 小寒19 ----------
pub const JIE_TERM_INDICES: [usize; 12] = [21, 23, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];

/// 立春在 24 节气中的序号
pub const LI_CHUN_TERM_INDEX: usize = 21;

/// 冬至在 24 节气中的序号（换年预留）
#[allow(dead_code)]
pub const WINTER_SOLSTICE_TERM_INDEX: usize = 18;

// ---------- 干支纪年/纪月（节气派：立春换年 + 十二节换月） ----------

/// 用节气派（立春换年、十二节换月）计算 JD 对应的年/月/日干支序。需 VSOP87。JD 用 Real。
pub fn ganzhi_from_jd_solar(
    vsop: &crate::astronomy::ephemeris::Vsop87,
    jd: impl ToReal,
    options: &GanzhiOptions,
) -> (u8, u8, u8) {
    use crate::astronomy::aspects::solar_term_jds_for_year_cached;

    let jd_r = real(jd);
    let (y, _, _) = Gregorian::from_julian_day(jd_r);
    let terms_this = solar_term_jds_for_year_cached(vsop, y);
    let terms_next = solar_term_jds_for_year_cached(vsop, y + 1);
    let lichun_this = terms_this[LI_CHUN_TERM_INDEX];

    let (ganzhi_year, jie_jds): (i32, [Real; 12]) = if jd_r < lichun_this {
        let terms_prev = solar_term_jds_for_year_cached(vsop, y - 1);
        let jie = [
            terms_prev[JIE_TERM_INDICES[0]],
            terms_prev[JIE_TERM_INDICES[1]],
            terms_prev[JIE_TERM_INDICES[2]],
            terms_prev[JIE_TERM_INDICES[3]],
            terms_prev[JIE_TERM_INDICES[4]],
            terms_prev[JIE_TERM_INDICES[5]],
            terms_prev[JIE_TERM_INDICES[6]],
            terms_prev[JIE_TERM_INDICES[7]],
            terms_prev[JIE_TERM_INDICES[8]],
            terms_prev[JIE_TERM_INDICES[9]],
            terms_prev[JIE_TERM_INDICES[10]],
            terms_this[JIE_TERM_INDICES[11]],
        ];
        (y - 1, jie)
    } else {
        let jie = [
            terms_this[JIE_TERM_INDICES[0]],
            terms_this[JIE_TERM_INDICES[1]],
            terms_this[JIE_TERM_INDICES[2]],
            terms_this[JIE_TERM_INDICES[3]],
            terms_this[JIE_TERM_INDICES[4]],
            terms_this[JIE_TERM_INDICES[5]],
            terms_this[JIE_TERM_INDICES[6]],
            terms_this[JIE_TERM_INDICES[7]],
            terms_this[JIE_TERM_INDICES[8]],
            terms_this[JIE_TERM_INDICES[9]],
            terms_this[JIE_TERM_INDICES[10]],
            terms_next[JIE_TERM_INDICES[11]],
        ];
        (y, jie)
    };

    let year_idx = (ganzhi_year + 56).rem_euclid(60) as u8;
    let month_ord = jie_jds
        .iter()
        .rposition(|&j| j <= jd_r)
        .unwrap_or(0);
    let month_idx = month_gan_zhi_from_year_and_ord(year_idx, month_ord as u8);
    let day_idx = jd_to_gan_zhi_index_with_options(jd_r, options.day_boundary);
    (year_idx, month_idx, day_idx)
}

/// 用农历派（正月初一换年、初一换月）计算 JD 对应的年/月/日干支序。需该日所在农历岁的岁数据；闰月按 options 处理。JD 用 Real。
pub fn ganzhi_from_jd_lunar(
    jd: impl ToReal,
    year_data: &crate::calendar::chinese_lunar::ChineseLunarYearData,
    options: &GanzhiOptions,
) -> Option<(u8, u8, u8)> {
    use crate::calendar::chinese_lunar::from_julian_day_in_year;
    let jd_r = real(jd);

    let date = from_julian_day_in_year(jd_r, year_data, None)?;
    let year_idx = (year_data.lunar_year + 56).rem_euclid(60) as u8;
    let (month_1_12, is_leap) = (date.month as u8, date.is_leap_month);
    let day_in_month = date.day as u8;

    let month_ord = lunar_month_to_ord(month_1_12, is_leap, day_in_month, options.leap_month_handling);
    let month_idx = month_gan_zhi_from_year_and_ord(year_idx, month_ord);
    let day_idx = jd_to_gan_zhi_index_with_options(jd_r, options.day_boundary);
    Some((year_idx, month_idx, day_idx))
}

/// 农历月(1-12)+是否闰月+日 → 月序 0..11（寅=0 … 丑=11）。闰月按选项处理。
fn lunar_month_to_ord(
    month_1_12: u8,
    is_leap: bool,
    day_in_month: u8,
    leap_handling: LeapMonthHandling,
) -> u8 {
    if !is_leap {
        return (month_1_12 + 11) % 12;
    }
    match leap_handling {
        LeapMonthHandling::Ignore | LeapMonthHandling::InheritPrevious => (month_1_12 + 11) % 12,
        LeapMonthHandling::SplitMidway => {
            if day_in_month <= 15 {
                (month_1_12 + 11) % 12
            } else {
                month_1_12 % 12
            }
        }
        LeapMonthHandling::ShiftToNext => month_1_12 % 12,
    }
}

/// 干支序(0..60) → 名称
#[inline]
pub fn gan_zhi_index_to_name(index: u8) -> &'static str {
    GAN_ZHI_60[(index % 60) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::real;

    #[test]
    fn ganzhi_options_default_and_presets() {
        let def = GanzhiOptions::default();
        assert_eq!(def.year_boundary, YearBoundary::LunarNewYear);
        assert_eq!(def.month_boundary, MonthBoundary::LunarFirstDay);
        assert_eq!(def.leap_month_handling, LeapMonthHandling::InheritPrevious);
        assert_eq!(def.day_boundary, DayBoundary::Hour0);

        let zp = preset_zi_ping_ba_zi();
        assert_eq!(zp.year_boundary, YearBoundary::LiChun);
        assert_eq!(zp.month_boundary, MonthBoundary::SolarTerm);
        assert_eq!(zp.day_boundary, DayBoundary::Hour23);

        let ps = preset_purple_star();
        assert_eq!(ps.leap_month_handling, LeapMonthHandling::SplitMidway);
        assert_eq!(ps.day_boundary, DayBoundary::Hour0);

        let fa = preset_folk_almanac();
        assert_eq!(fa.leap_month_handling, LeapMonthHandling::InheritPrevious);
        assert_eq!(fa.day_boundary, DayBoundary::Hour23);

        let xj = preset_xie_ji_bian_fang();
        assert_eq!(xj.year_boundary, YearBoundary::LiChun);
        assert_eq!(xj.month_boundary, MonthBoundary::SolarTerm);
        assert_eq!(xj.leap_month_handling, LeapMonthHandling::InheritPrevious);
    }

    #[test]
    fn jd_to_gan_zhi_index_with_options_day_boundary() {
        // J2000.0 = JD 2451545.0, 子正换日与子初换日差 1h ≈ 1/24 日
        let jd = real(2451545.0);
        let idx0 = jd_to_gan_zhi_index_with_options(jd, DayBoundary::Hour0);
        let idx23 = jd_to_gan_zhi_index_with_options(jd, DayBoundary::Hour23);
        assert!(idx0 < 60);
        assert!(idx23 < 60);
        let name0 = jd_to_gan_zhi_day_with_options(jd, DayBoundary::Hour0);
        let name23 = jd_to_gan_zhi_day_with_options(jd, DayBoundary::Hour23);
        assert!(!name0.is_empty());
        assert!(!name23.is_empty());
    }

    #[test]
    fn gan_zhi_index_to_name_wraps() {
        assert_eq!(gan_zhi_index_to_name(0), "甲子");
        assert_eq!(gan_zhi_index_to_name(59), "癸亥");
        assert_eq!(gan_zhi_index_to_name(60), "甲子");
    }

    #[test]
    fn jie_term_indices_and_li_chun() {
        assert_eq!(JIE_TERM_INDICES.len(), 12);
        assert_eq!(LI_CHUN_TERM_INDEX, 21);
    }
}
