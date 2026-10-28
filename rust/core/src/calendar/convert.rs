use super::chinese_lunar::{
    from_gregorian_day_in_year, from_julian_day_in_year, new_moon_day_numbers_utc8, to_julian_day, ChineseLunarDate,
    ChineseLunarYearData,
};
use super::gregorian::Gregorian;
use super::historical::HistoricalCalendar;
use super::lifa;
pub fn gregorian_to_chinese_lunar(
    year: i32,
    month: i32,
    day: i32,
    year_data: &ChineseLunarYearData,
) -> Option<ChineseLunarDate> {
    from_gregorian_day_in_year(year, month, day, year_data, None)
}

pub fn gregorian_month_to_lunar(
    year: i32,
    month: i32,
    year_data: &ChineseLunarYearData,
) -> Vec<Option<ChineseLunarDate>> {
    let days = Gregorian::days_in_month(year, month) as i32;
    let precomputed = new_moon_day_numbers_utc8(year_data);
    (1..=days)
        .map(|day| from_gregorian_day_in_year(year, month, day, year_data, Some(&precomputed)))
        .collect()
}

pub fn chinese_lunar_to_gregorian(
    date: ChineseLunarDate,
    year_data: &ChineseLunarYearData,
) -> Option<(i32, i32, i32)> {
    let jd = to_julian_day(date, year_data)?;
    Some(Gregorian::from_julian_day(jd))
}

/// 公历→农历（现场算岁数据）。默认精度由本层传入：`tolerance: PlaneAngle`，内部调用 `compute_year_data`。
pub fn gregorian_to_chinese_lunar_calc(
    year: i32,
    month: i32,
    day: i32,
    vsop: &crate::astronomy::ephemeris::Vsop87,
    elp: &crate::astronomy::ephemeris::Elpmpp02Data,
    tolerance: crate::quantity::angle::PlaneAngle,
    max_iterations: usize,
) -> Option<ChineseLunarDate> {
    gregorian_to_chinese_lunar_prefer_historical(None, year, month, day, vsop, elp, tolerance, max_iterations)
}

/// 公历→农历：**优先实历**。目标日所在农历年该用实历查表还是定气定朔现算，由第 4 层仲裁 [lifa::resolve_year_data]
/// 决定，本函数只负责"该查哪一两个候选农历年、算出来的岁数据能不能落到这一天"，不重复仲裁判断。
/// 历史区间当时行用的是各朝颁行的历法（平朔、经朔、定朔各有其法），不是现代定气定朔，所以优先用按各朝历法复原的实历表；该表是复原结果（《三千五百年历日天象》据历推步），不是史料原记录，与史书日干支不合之处须另行核验。主表 = 各朝颁历据历还原推步得出的连续表，年号表按年体现中原政权更替（汉/魏/晋/…/清）。
pub fn gregorian_to_chinese_lunar_prefer_historical(
    historical: Option<&HistoricalCalendar>,
    year: i32,
    month: i32,
    day: i32,
    vsop: &crate::astronomy::ephemeris::Vsop87,
    elp: &crate::astronomy::ephemeris::Elpmpp02Data,
    tolerance: crate::quantity::angle::PlaneAngle,
    max_iterations: usize,
) -> Option<ChineseLunarDate> {
    let jd = Gregorian::to_julian_day(year, month, day);
    for &ly in &[year, year - 1] {
        let year_data = lifa::resolve_year_data(historical, ly, vsop, elp, tolerance, max_iterations)?;
        if let Some(date) = from_julian_day_in_year(jd, &year_data, None) {
            return Some(date);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gregorian::Gregorian;

    #[test]
    fn chinese_lunar_to_gregorian_roundtrip() {
        use crate::calendar::chinese_lunar::ChineseLunarYearData;
        use crate::math::real::Real;
        let jd_17 = Gregorian::to_julian_day_0h_utc8(2026, 2, 17);
        let nm2_tt = jd_17 + crate::math::real::real_const(0.5);
        let new_moon_jds: Vec<Real> = (0..14)
            .map(|i| nm2_tt + crate::math::real::real((i as f64 - 2.0) * 29.5))
            .collect();
        let zhong_qi_jds: Vec<Real> = (0..12)
            .map(|k| nm2_tt - crate::math::real::real_const(30.0) + crate::math::real::real(k as f64 * 30.0))
            .collect();
        let year_data = ChineseLunarYearData::new(2026, new_moon_jds, zhong_qi_jds);
        let date = gregorian_to_chinese_lunar(2026, 2, 20, &year_data).unwrap();
        let (y, m, d) = chinese_lunar_to_gregorian(date, &year_data).unwrap();
        assert_eq!((y, m, d), (2026, 2, 20));
    }
}
