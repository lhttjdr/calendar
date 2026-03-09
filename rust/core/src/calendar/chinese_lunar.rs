//! 农历：岁数据、农历日期与 JD 换算。
//! 制造日历核心：`compute_year_data` 用定气定朔算一岁 14 朔 + 12 中气；`from_julian_day_in_year` / `to_julian_day` 做 JD↔农历。

use crate::astronomy::aspects::{new_moon_jds_in_range, solar_term_jds_for_year_cached};
use crate::astronomy::ephemeris::{Elpmpp02Data, Vsop87};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::calendar::gregorian::Gregorian;
use crate::math::real::{real, Real, RealOps};
use crate::quantity::angle::PlaneAngle;

/// 农历日期：年（春节所在公历年）、月(1–12)、日(1–30)、是否闰月、该农历月天数(29/30)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChineseLunarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub is_leap_month: bool,
    /// 该农历月天数，29=小月 30=大月
    pub days_in_month: u8,
}

/// 一岁内的气朔数据（冬至到下一冬至）：14 朔、12 中气。JD 用 Real。
#[derive(Clone, Debug)]
pub struct ChineseLunarYearData {
    pub lunar_year: i32,
    pub new_moon_jds: Vec<Real>,
    pub zhong_qi_jds: Vec<Real>,
}

impl ChineseLunarYearData {
    pub fn new(lunar_year: i32, new_moon_jds: Vec<Real>, zhong_qi_jds: Vec<Real>) -> Self {
        Self {
            lunar_year,
            new_moon_jds,
            zhong_qi_jds,
        }
    }
}

/// 公历与农历均按 UTC+8 日界，时区一致；公历日 (y,m,d) 用 JDN，朔日等用 JD→日号。
///
/// 为什么需要 ε：不是时区没抵消，而是浮点误差。数学上 0h 北京 (y,m,d) 对应
///   JD_UTC + 0.5 + 8/24 = JDN（整数），但 2461088.166… + 0.8333… 在 f64 里常变成 2461088.999…
///   floor 就得到 2461088，与 JDN=2461089 不一致。加 ε 补偿该边界上的浮点误差。常量直接 Real。
const UTC_PLUS_8_OFFSET_DAYS: Real = crate::math::real::real_const(8.0 / 24.0);
const TZ_DAY_EPSILON: Real = crate::math::real::real_const(1e-4);

/// 仅用于「时刻」JD（朔日等）→ 日号。公历日用 Gregorian::day_number_utc8。
fn jd_ut_to_day_number_utc8(jd_ut: Real) -> i64 {
    (jd_ut + crate::math::real::real_const(0.5) + UTC_PLUS_8_OFFSET_DAYS + TZ_DAY_EPSILON)
        .floor()
        .to_i64_floor()
}

/// 朔日 JD(TT) 转为 0h 北京时间下的日号（先 TT→UTC 再按 UTC+8 取日）
fn new_moon_tt_to_day_number_utc8(jd_tt: Real) -> i64 {
    let jd_ut = TimePoint::new(TimeScale::TT, jd_tt).to_scale(TimeScale::UTC).jd;
    jd_ut_to_day_number_utc8(jd_ut)
}

/// 朔日 JD(TT) 转为日号，标量 Real。
fn new_moon_tt_to_day_number_utc8_r(jd_tt: Real) -> i64 {
    new_moon_tt_to_day_number_utc8(jd_tt)
}

/// 一岁内所有朔日在 UTC+8 下的日号（用于整月循环时预计算，保证边界一致）
pub fn new_moon_day_numbers_utc8(year_data: &ChineseLunarYearData) -> Vec<i64> {
    year_data
        .new_moon_jds
        .iter()
        .map(|jd_tt| new_moon_tt_to_day_number_utc8_r(*jd_tt))
        .collect()
}

/// 中气下标 0..11 -> 农历月 11,12,1..10
fn zhong_qi_to_month(k: usize) -> u8 {
    if k <= 1 {
        (k + 11) as u8
    } else {
        (k - 1) as u8
    }
}

fn build_month_has_zhong_qi(year_data: &ChineseLunarYearData) -> Vec<i32> {
    let zq = &year_data.zhong_qi_jds;
    let nm = &year_data.new_moon_jds;
    let mut arr = vec![-1_i32; 13];
    for mi in 0..13 {
        if mi + 1 >= nm.len() {
            break;
        }
        let month_start = &nm[mi];
        let month_end = &nm[mi + 1];
        for k in 0..12 {
            let jd_z = &zq[k];
            if jd_z >= month_start && jd_z < month_end {
                arr[mi] = k as i32;
                break;
            }
        }
    }
    arr
}

fn month_number_and_leap(month_index: usize, month_has_zhong_qi: &[i32]) -> (u8, bool) {
    let zq_idx = month_has_zhong_qi.get(month_index).copied().unwrap_or(-1);
    if zq_idx < 0 {
        if month_index == 0 {
            (11, true)
        } else {
            let prev_zq = month_has_zhong_qi.get(month_index - 1).copied().unwrap_or(-1);
            let prev_num = if prev_zq >= 0 {
                zhong_qi_to_month(prev_zq as usize)
            } else {
                11
            };
            (prev_num, true)
        }
    } else {
        (zhong_qi_to_month(zq_idx as usize), false)
    }
}

/// 调试：公历 (y,m,d) 转农历时的中间量，用于排查「两个初一」等日界问题。
pub fn gregorian_to_chinese_lunar_debug(
    year: i32,
    month: i32,
    day: i32,
    year_data: &ChineseLunarYearData,
) -> String {
    let dn = Gregorian::day_number_utc8(year, month, day);
    let nm = &year_data.new_moon_jds;
    let mut out = format!("gy={} gm={} gd={} dn={}", year, month, day, dn);
    for i in 0..nm.len().min(4) {
        let start_dn = new_moon_tt_to_day_number_utc8_r(nm[i]);
        out.push_str(&format!(" nm[{}]_dn={}", i, start_dn));
    }
    let mut month_index: i32 = -1;
    for i in 0..nm.len().saturating_sub(1) {
        let start_dn = new_moon_tt_to_day_number_utc8_r(nm[i]);
        let end_dn = new_moon_tt_to_day_number_utc8_r(nm[i + 1]);
        if dn >= start_dn && dn < end_dn {
            month_index = i as i32;
            break;
        }
    }
    if month_index >= 0 {
        let mi = month_index as usize;
        let start_dn_utc8 = new_moon_tt_to_day_number_utc8_r(nm[mi]);
        let day_of_month = (dn - start_dn_utc8) as i32 + 1;
        let month_has = build_month_has_zhong_qi(year_data);
        let (month_number, is_leap) = month_number_and_leap(mi, &month_has);
        out.push_str(&format!(
            " mi={} start_dn={} day_of_month={} lunar={}{}月{}日",
            mi,
            start_dn_utc8,
            day_of_month,
            if is_leap { "闰" } else { "" },
            month_number,
            day_of_month
        ));
    } else {
        out.push_str(" (无匹配月)");
    }
    out
}

/// 用整数日号在该岁内查农历。公历日请走 from_gregorian_day_in_year，避免 JD 边界舍入。
pub fn from_day_number_in_year(
    dn: i64,
    year_data: &ChineseLunarYearData,
    precomputed_new_moon_dn: Option<&[i64]>,
) -> Option<ChineseLunarDate> {
    let nm = &year_data.new_moon_jds;
    let mut month_index: i32 = -1;
    let mut end_dn_opt: Option<i64> = None;
    for i in 0..nm.len().saturating_sub(1) {
        let start_dn = match precomputed_new_moon_dn {
            Some(s) if i < s.len() => s[i],
            _ => new_moon_tt_to_day_number_utc8_r(nm[i]),
        };
        let end_dn = match precomputed_new_moon_dn {
            Some(s) if i + 1 < s.len() => s[i + 1],
            _ => new_moon_tt_to_day_number_utc8_r(nm[i + 1]),
        };
        if dn >= start_dn && dn < end_dn {
            month_index = i as i32;
            end_dn_opt = Some(end_dn);
            break;
        }
    }
    let mi = month_index as usize;
    let end_dn = end_dn_opt?;
    let start_dn_utc8 = match precomputed_new_moon_dn {
        Some(s) if mi < s.len() => s[mi],
        _ => new_moon_tt_to_day_number_utc8_r(nm[mi]),
    };
    let day_of_month = (dn - start_dn_utc8) as i32 + 1;
    if day_of_month < 1 || day_of_month > 30 {
        return None;
    }
    let days_in_month = (end_dn - start_dn_utc8) as i32;
    let days_in_month = if days_in_month >= 30 { 30 } else { 29 };
    let month_has = build_month_has_zhong_qi(year_data);
    let (month_number, is_leap) = month_number_and_leap(mi, &month_has);
    Some(ChineseLunarDate {
        year: year_data.lunar_year,
        month: month_number,
        day: day_of_month as u8,
        is_leap_month: is_leap,
        days_in_month: days_in_month as u8,
    })
}

/// 时刻 JD(UTC) → 该岁内农历（先转日号再查）。公历 (y,m,d) 请用 from_gregorian_day_in_year。
pub fn from_julian_day_in_year(
    jd: Real,
    year_data: &ChineseLunarYearData,
    precomputed_new_moon_dn: Option<&[i64]>,
) -> Option<ChineseLunarDate> {
    let dn = jd_ut_to_day_number_utc8(jd);
    from_day_number_in_year(dn, year_data, precomputed_new_moon_dn)
}

/// 公历 (年,月,日) 直接按整数日号查农历，不经 JD，避免 0h 边界舍入。
pub fn from_gregorian_day_in_year(
    year: i32,
    month: i32,
    day: i32,
    year_data: &ChineseLunarYearData,
    precomputed_new_moon_dn: Option<&[i64]>,
) -> Option<ChineseLunarDate> {
    let dn = Gregorian::day_number_utc8(year, month, day);
    from_day_number_in_year(dn, year_data, precomputed_new_moon_dn)
}

/// 农历日期格式化为显示字符串（正月、初二等）
pub fn format_lunar_date_to_string(date: ChineseLunarDate) -> String {
    const MONTH_NAMES: &[&str] = &["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "冬", "腊"];
    let month_str = if date.is_leap_month {
        format!("闰{}月", MONTH_NAMES[date.month as usize - 1])
    } else {
        format!("{}月", MONTH_NAMES[date.month as usize - 1])
    };
    let day_str = match date.day {
        1 => "初一".to_string(),
        d if d <= 9 => format!("初{}", ["一", "二", "三", "四", "五", "六", "七", "八", "九"][d as usize - 2]),
        10 => "初十".to_string(),
        d if d <= 19 => format!("十{}", ["一", "二", "三", "四", "五", "六", "七", "八", "九"][d as usize - 11]),
        20 => "二十".to_string(),
        d if d <= 29 => format!("廿{}", ["一", "二", "三", "四", "五", "六", "七", "八", "九"][d as usize - 21]),
        _ => "三十".to_string(),
    };
    format!("{}{}", month_str, day_str)
}

/// 将农历日期转换为儒略日
pub fn to_julian_day(
    date: ChineseLunarDate,
    year_data: &ChineseLunarYearData,
) -> Option<Real> {
    if year_data.lunar_year != date.year {
        return None;
    }
    let nm = &year_data.new_moon_jds;
    let month_has = build_month_has_zhong_qi(year_data);
    let mut target_month_index: i32 = -1;
    for mi in 0..13 {
        let (num, is_leap) = month_number_and_leap(mi, &month_has);
        if num == date.month && is_leap == date.is_leap_month {
            target_month_index = mi as i32;
            break;
        }
    }
    if target_month_index < 0 || target_month_index as usize + 1 >= nm.len() {
        return None;
    }
    let month_start_jd = nm[target_month_index as usize];
    let day_offset = (date.day as i32) - 1;
    let jd_r = month_start_jd + crate::math::real::from_i32(day_offset);
    Some(jd_r)
}

/// 中气节气序号：冬至18、大寒20、雨水22、春分0、谷雨2、小满4、夏至6、大暑8、处暑10、秋分12、霜降14、小雪16
const ZHONG_QI_TERM_INDICES: [usize; 12] =
    [18, 20, 22, 0, 2, 4, 6, 8, 10, 12, 14, 16];

/// 平均朔望月（日），与定朔模块一致，仅用于岁范围估算。
const MEAN_SYNODIC_MONTH_DAYS: f64 = 29.530_588_861;
/// 回归年约 365.2425/29.53 ≈ 12.37 朔；前后各 2.5 朔望月（约 74 天）以覆盖 14 朔并抵消 f64 定朔误差导致的“少算”。
const MARGIN_SYNODIC_COUNT: f64 = 2.5;
const MARGIN_END_BUFFER_DAYS: f64 = 2.0;

/// 用定气与定朔计算指定农历年的岁数据。中气从按年缓存的 24 节气中取，与节气派干支历共用缓存，避免重复算定气。
/// 精度由调用处通过 `tolerance: PlaneAngle` 与 `compute_year_data` 传入。
/// 岁范围：去年冬至前 1.5 朔望月 ～ 今年冬至后 1.5 朔望月，保证 14 朔在范围内。
pub fn compute_year_data(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    lunar_year: i32,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Result<ChineseLunarYearData, String> {
    let terms_prev = solar_term_jds_for_year_cached(vsop, lunar_year - 1);
    let terms_curr = solar_term_jds_for_year_cached(vsop, lunar_year);
    let jd_winter1 = terms_prev[18];
    let jd_winter2 = terms_curr[18];

    let margin_days = MARGIN_SYNODIC_COUNT * MEAN_SYNODIC_MONTH_DAYS;
    let jd_start = jd_winter1 - real(margin_days);
    let jd_end = jd_winter2 + real(margin_days + MARGIN_END_BUFFER_DAYS);
    let all_new_moons =
        new_moon_jds_in_range(vsop, elp, jd_start, jd_end, tolerance, max_iterations);
    let start_idx = all_new_moons
        .iter()
        .rposition(|jd| *jd <= jd_winter1)
        .unwrap_or(0);
    let new_moon_jds: Vec<Real> = all_new_moons
        .into_iter()
        .skip(start_idx)
        .take(14)
        .collect();
    if new_moon_jds.len() != 14 {
        return Err(format!(
            "need 14 new moons for lunar year {}, got {} in range (margin {:.0} d)",
            lunar_year,
            new_moon_jds.len(),
            margin_days
        ));
    }

    let zhong_qi_jds: Vec<Real> = [
        terms_prev[ZHONG_QI_TERM_INDICES[0]],
        terms_curr[ZHONG_QI_TERM_INDICES[1]],
        terms_curr[ZHONG_QI_TERM_INDICES[2]],
        terms_curr[ZHONG_QI_TERM_INDICES[3]],
        terms_curr[ZHONG_QI_TERM_INDICES[4]],
        terms_curr[ZHONG_QI_TERM_INDICES[5]],
        terms_curr[ZHONG_QI_TERM_INDICES[6]],
        terms_curr[ZHONG_QI_TERM_INDICES[7]],
        terms_curr[ZHONG_QI_TERM_INDICES[8]],
        terms_curr[ZHONG_QI_TERM_INDICES[9]],
        terms_curr[ZHONG_QI_TERM_INDICES[10]],
        terms_curr[ZHONG_QI_TERM_INDICES[11]],
    ]
    .to_vec();
    debug_assert_eq!(zhong_qi_jds.len(), 12, "zhong_qi always 12 terms");
    Ok(ChineseLunarYearData::new(
        lunar_year,
        new_moon_jds,
        zhong_qi_jds,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zhong_qi_to_month_smoke() {
        assert_eq!(zhong_qi_to_month(0), 11);
        assert_eq!(zhong_qi_to_month(1), 12);
        assert_eq!(zhong_qi_to_month(2), 1);
    }

    /// 2026 年 2 月 17、18 日按 0h 北京时间不应同为正月初一（日界 UTC+8 下初一只占一日）
    #[test]
    fn feb_2026_only_one_chu_yi() {
        use crate::calendar::convert::gregorian_to_chinese_lunar;
        use crate::calendar::gregorian::Gregorian;
        // 构造 2026 岁：正月朔在 2 月 17 日北京时间附近，14 朔 + 12 中气
        let jd_17 = Gregorian::to_julian_day_0h_utc8(2026, 2, 17);
        let nm2_tt = jd_17 + crate::math::real::real_const(0.5);
        let new_moon_jds: Vec<Real> = (0..14)
            .map(|i| nm2_tt + crate::math::real::real((i as f64 - 2.0) * 29.5))
            .collect();
        let zhong_qi_jds: Vec<Real> = (0..12)
            .map(|k| nm2_tt - crate::math::real::real_const(30.0) + crate::math::real::real(k as f64 * 30.0))
            .collect();
        let year_data = ChineseLunarYearData::new(2026, new_moon_jds, zhong_qi_jds);

        let r17 = gregorian_to_chinese_lunar(2026, 2, 17, &year_data).unwrap();
        let r18 = gregorian_to_chinese_lunar(2026, 2, 18, &year_data).unwrap();
        assert!(
            !(r17.month == 1 && r17.day == 1 && r18.month == 1 && r18.day == 1),
            "17 与 18 不应同时为正月初一: 17={}月{}日 18={}月{}日",
            r17.month,
            r17.day,
            r18.month,
            r18.day
        );
    }

    /// 整月接口：17 与 18 应为同一农历月且日期连续（预计算朔日日号保证不出现两个初一）
    #[test]
    fn feb_2026_month_to_lunar_17_18() {
        use crate::calendar::convert::gregorian_month_to_lunar;
        use crate::calendar::gregorian::Gregorian;
        let jd_17 = Gregorian::to_julian_day_0h_utc8(2026, 2, 17);
        let nm2_tt = jd_17 + crate::math::real::real_const(0.5);
        let new_moon_jds: Vec<Real> = (0..14)
            .map(|i| nm2_tt + crate::math::real::real((i as f64 - 2.0) * 29.5))
            .collect();
        let zhong_qi_jds: Vec<Real> = (0..12)
            .map(|k| nm2_tt - crate::math::real::real_const(30.0) + crate::math::real::real(k as f64 * 30.0))
            .collect();
        let year_data = ChineseLunarYearData::new(2026, new_moon_jds, zhong_qi_jds);
        let arr = gregorian_month_to_lunar(2026, 2, &year_data);
        let d17 = arr[16].as_ref().expect("day 17");
        let d18 = arr[17].as_ref().expect("day 18");
        assert_eq!(d17.month, d18.month, "17与18应同农历月");
        assert_eq!(d17.is_leap_month, d18.is_leap_month);
        assert_eq!(d18.day, d17.day + 1, "18应为17的次日，不能两个初一: 17={}月{}日 18={}月{}日", d17.month, d17.day, d18.month, d18.day);
    }
}
