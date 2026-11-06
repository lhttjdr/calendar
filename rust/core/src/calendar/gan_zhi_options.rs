//! 干支历选项与多套标准：换年/换月/闰月处理/换日，及预设。
//!
//! 各选项与预设的原典出处见 `doc/folk/ganzhi-calendar.md`；没有原典的选项在注释里注明「原典未见」。

use crate::calendar::gan_zhi::{jd_to_gan_zhi_index, GAN_ZHI_60};
use crate::calendar::gregorian::Gregorian;
use crate::math::real::{real, real_const, Real, RealOps, ToReal};

/// 换年干支
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum YearBoundary {
    /// 立春交节时刻：子平（《命理探源》卷二推年法「以立春節為綱」）、择日（《協紀》卷三十四「歳官交承」）
    LiChun,
    /// 正月初一零点：农历年；紫微斗数按农历年月起盘（《紫微斗數全書》未明言年界）
    #[default]
    LunarNewYear,
    /// 冬至交节时刻：太乙积年以上元甲子冬至为元，太乙、奇门以冬至分阴阳遁；以冬至换年柱者原典未见，作选项保留
    WinterSolstice,
}

/// 换月干支
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum MonthBoundary {
    /// 十二节交节时刻（立春、惊蛰、清明…）：子平（《命理探源》推月法「以節令為綱」）、
    /// 时宪书建除（《協紀》卷四「每月交節則疊兩值日」）、太乙月计（「若有閏月只以逐月節氣時刻為正」）
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
    /// 随前月（闰几月用几月干支）：今人万年历给农历月配干支的做法，原典未见
    #[default]
    InheritPrevious,
    /// 月中切分：前 15 日本月，后 15 日下月。近人之说，原典未见
    SplitMidway,
    /// 整个闰月作下月：《紫微斗數全書》卷三《安身命例》「若閏正月生者，要在二月內起安身命，凡有閏月俱要依此為例」
    ShiftToNext,
}

/// 换日干支
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DayBoundary {
    /// 子初换日 23:00：今人多从，原典未见（唐傅仁均以子初为朔，李淳风驳之，见《新唐書·曆志一》）
    Hour23,
    /// 子正换日 00:00：历法「古曆分日，起於子半」；《三命通會》「上半時在夜半前屬昨日」；
    /// 《星平大成》「子正者今日之早，夜子者今日之夜」（夜子时时干按次日起，见前端时柱）
    #[default]
    Hour0,
}

/// 节气派用定气还是平气（只影响立春换年、按节换月；农历派的月份来自实历表与定朔）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum SolarTermMode {
    /// 定气：太阳视黄经每 15° 一气。清时宪历顺治二年（1645）起行用
    #[default]
    Dingqi,
    /// 平气：按当时行用历法的岁实与历元推算（[`crate::calendar::pingqi`]，参数表 data/calendar/pingqi_lifa.csv）。
    /// 表外年代（尚未据律历志核定的历、清时宪历以后）仍用定气。
    Pingqi,
}

/// 干支历完整配置
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GanzhiOptions {
    pub year_boundary: YearBoundary,
    pub month_boundary: MonthBoundary,
    pub leap_month_handling: LeapMonthHandling,
    pub day_boundary: DayBoundary,
    pub solar_term_mode: SolarTermMode,
}

impl Default for GanzhiOptions {
    fn default() -> Self {
        Self {
            year_boundary: YearBoundary::default(),
            month_boundary: MonthBoundary::default(),
            leap_month_handling: LeapMonthHandling::default(),
            day_boundary: DayBoundary::default(),
            solar_term_mode: SolarTermMode::default(),
        }
    }
}

/// 子平八字（子正换日）：立春换年、十二节换月、子正换日。
/// 依据《三命通會》卷二、《星平大成》（经《命理探源》卷五转引）、《命理探源》卷二。
/// 年、月两项与《協紀辨方書》择日相同。
pub fn preset_zi_ping_ba_zi() -> GanzhiOptions {
    GanzhiOptions {
        year_boundary: YearBoundary::LiChun,
        month_boundary: MonthBoundary::SolarTerm,
        leap_month_handling: LeapMonthHandling::Ignore,
        day_boundary: DayBoundary::Hour0,
        solar_term_mode: SolarTermMode::Dingqi,
    }
}

/// 子平八字（子初换日）：同上，唯 23 时即换日。今人多从，原典未见。
pub fn preset_zi_ping_zi_chu() -> GanzhiOptions {
    GanzhiOptions {
        day_boundary: DayBoundary::Hour23,
        ..preset_zi_ping_ba_zi()
    }
}

/// 紫微斗数：农历年月起盘、闰月整月作下月（《紫微斗數全書》卷三《安身命例》）。
/// 书中未言日界，取子正。
pub fn preset_purple_star() -> GanzhiOptions {
    GanzhiOptions {
        year_boundary: YearBoundary::LunarNewYear,
        month_boundary: MonthBoundary::LunarFirstDay,
        leap_month_handling: LeapMonthHandling::ShiftToNext,
        day_boundary: DayBoundary::Hour0,
        solar_term_mode: SolarTermMode::Dingqi,
    }
}

/// 农历月干支：正月初一换年、初一换月、闰月随前月、子正换日。今人万年历的标注法，原典未见。
pub fn preset_lunar_month_ganzhi() -> GanzhiOptions {
    GanzhiOptions {
        year_boundary: YearBoundary::LunarNewYear,
        month_boundary: MonthBoundary::LunarFirstDay,
        leap_month_handling: LeapMonthHandling::InheritPrevious,
        day_boundary: DayBoundary::Hour0,
        solar_term_mode: SolarTermMode::Dingqi,
    }
}

// ---------- 换日（仅依赖 JD） ----------

/// 按选项的日界计算干支日序。Hour0 = floor(JD+0.5)；Hour23 = 23:00 起已算次日，故加 1 小时。JD 支持 Real。
#[inline]
pub fn jd_to_gan_zhi_index_with_options(jd: impl ToReal, day_boundary: DayBoundary) -> u8 {
    let jd_r = real(jd);
    let jd_for_day = match day_boundary {
        DayBoundary::Hour0 => jd_r,
        DayBoundary::Hour23 => jd_r + real(1.0 / 24.0),
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
    // 干支序 i 满足 i≡干 (mod 10)、i≡支 (mod 12)，解为 6干−5支 (mod 60)
    ((6 * month_stem as i32 - 5 * month_branch as i32).rem_euclid(60)) as u8
}

// ---------- 十二节在 24 节气中的序号（k=0 春分…）：立春21, 惊蛰23, 清明1, 立夏3, 芒种5, 小暑7, 立秋9, 白露11, 寒露13, 立冬15, 大雪17, 小寒19 ----------
pub const JIE_TERM_INDICES: [usize; 12] = [21, 23, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];

/// 立春在 24 节气中的序号
pub const LI_CHUN_TERM_INDEX: usize = 21;

/// 冬至在 24 节气中的序号（换年预留）
#[allow(dead_code)]
pub const WINTER_SOLSTICE_TERM_INDEX: usize = 18;

// ---------- 平气 ----------

/// 把某公历年的 24 定气（索引同 solar_term_jds_for_year：0=春分 … 18=冬至 … 23=惊蛰）换成按行用历推算的同名平气，
/// JD(TT)。行用历表外的气保留定气。
pub fn pingqi_jds_from_dingqi(dingqi_tt: &[Real]) -> Vec<Real> {
    dingqi_tt
        .iter()
        .enumerate()
        .map(|(k, &dq)| {
            crate::calendar::pingqi::pingqi_tt(dq, k as i32 - 18).map(|(_, jd)| jd).unwrap_or(dq)
        })
        .collect()
}

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
    let terms = |year: i32| {
        let dq = solar_term_jds_for_year_cached(vsop, year);
        match options.solar_term_mode {
            SolarTermMode::Pingqi => pingqi_jds_from_dingqi(&dq),
            _ => dq,
        }
    };
    let terms_this = terms(y);
    let terms_next = terms(y + 1);
    let lichun_this = terms_this[LI_CHUN_TERM_INDEX];

    let (ganzhi_year, jie_jds): (i32, [Real; 12]) = if jd_r < lichun_this {
        let terms_prev = terms(y - 1);
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
        assert_eq!(zp.day_boundary, DayBoundary::Hour0);

        let zc = preset_zi_ping_zi_chu();
        assert_eq!(zc.year_boundary, YearBoundary::LiChun);
        assert_eq!(zc.day_boundary, DayBoundary::Hour23);

        let ps = preset_purple_star();
        assert_eq!(ps.month_boundary, MonthBoundary::LunarFirstDay);
        assert_eq!(ps.leap_month_handling, LeapMonthHandling::ShiftToNext);
        assert_eq!(ps.day_boundary, DayBoundary::Hour0);

        let lm = preset_lunar_month_ganzhi();
        assert_eq!(lm.leap_month_handling, LeapMonthHandling::InheritPrevious);
        assert_eq!(lm.day_boundary, DayBoundary::Hour0);
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

    /// 子初换日：当日 0 时和子正换日同一干支，当日 23 时起算次日。
    #[test]
    fn hour23_boundary_direction() {
        use crate::calendar::gregorian::Gregorian;
        let jd = Gregorian::to_julian_day(2000, 1, 1);
        assert_eq!(jd_to_gan_zhi_day_with_options(jd, DayBoundary::Hour0), "戊午");
        assert_eq!(jd_to_gan_zhi_day_with_options(jd, DayBoundary::Hour23), "戊午");
        let h22 = real(jd) + real(22.5 / 24.0);
        let h23 = real(jd) + real(23.5 / 24.0);
        assert_eq!(jd_to_gan_zhi_day_with_options(h22, DayBoundary::Hour23), "戊午");
        assert_eq!(jd_to_gan_zhi_day_with_options(h23, DayBoundary::Hour23), "己未");
        assert_eq!(jd_to_gan_zhi_day_with_options(h23, DayBoundary::Hour0), "戊午");
    }

    /// 五虎遁全表：甲己丙寅、乙庚戊寅、丙辛庚寅、丁壬壬寅、戊癸甲寅，十二月干支逐月相连。
    #[test]
    fn five_tigers_table() {
        for year in 0..60u8 {
            let first = ["丙寅", "戊寅", "庚寅", "壬寅", "甲寅"][(year % 10 % 5) as usize];
            let start = month_gan_zhi_from_year_and_ord(year, 0);
            assert_eq!(GAN_ZHI_60[start as usize], first, "年序 {year}");
            for ord in 0..12u8 {
                let m = month_gan_zhi_from_year_and_ord(year, ord);
                assert_eq!(m, (start + ord) % 60, "年序 {year} 月序 {ord}");
                assert_eq!(m % 12, (ord + 2) % 12, "月支");
            }
        }
    }

    /// 用真实定气检验：1600 年平气冬至与定气冬至相差在半日内；二分前后平气与定气相差约两日
    /// （王锡阐「西人既用定氣……因譏中歷節氣差至二日」，《清史稿》卷五百六）。
    #[test]
    fn pingqi_vs_dingqi_1600() {
        use crate::astronomy::aspects::solar_term_jds_for_year_cached;
        use crate::astronomy::ephemeris::load_earth_vsop87_from_repo;
        let vsop = load_earth_vsop87_from_repo().unwrap();
        let dq = solar_term_jds_for_year_cached(&vsop, 1600);
        let pq = pingqi_jds_from_dingqi(&dq);
        let d = |k: usize| (pq[k] - dq[k]).as_f64();
        println!("1600 平气−定气（日）：冬至 {:.3} 春分 {:.3} 夏至 {:.3} 秋分 {:.3}", d(18), d(0), d(6), d(12));
        assert!(d(18).abs() < 0.5, "冬至 {}", d(18));
        assert!(d(0) > 1.5 && d(0) < 2.6, "春分 {}", d(0));
        assert!(d(12) < -1.5 && d(12) > -2.6, "秋分 {}", d(12));
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
