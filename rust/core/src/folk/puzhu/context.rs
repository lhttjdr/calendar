//! 由历表与历法推一日的铺注条件（[`PuzhuInput`]）：节气、朔弦望、农历日、伏社、土王用事、氣往亡。
//!
//! 口径：
//! - 日以东八区民用日计（[`tt_to_day_number_utc8`]）；月建以交节为界，交节当日属新月（同月表、建除）。
//! - 年（上朔、年神）以立春为界。
//! - 农历日期走「实历优先」（[`lifa::resolve_year_data`]）：有实历朔日表的年份用实历。
//! - 伏：初伏、中伏、末伏三日（[`zajie::san_fu`]）；社：二分前後近戊（《協紀》卷十二引歷例，[`zajie::she_jin_wu`]）。
//! - 土王用事：四立前十八日起至四立前一日（[`zajie::tu_wang`]）。
//! - 氣往亡：「皆自交節日數之」，交节当日为第一日（见 [`QiWangWangCount`]）。

use super::{wulu, FenZhi, PuzhuInput, WuLu, QI_WANG_WANG_DAYS};
use crate::folk::shensha::shen_for_day;
use crate::astronomy::aspects::{lunar_phase_jds_in_range, solar_term_jds_for_year_cached, LunarPhase};
use crate::astronomy::ephemeris::{Elpmpp02Data, Vsop87};
use crate::calendar::chinese_lunar::{from_day_number_in_year, tt_to_day_number_utc8, ChineseLunarDate};
use crate::calendar::gregorian::Gregorian;
use crate::calendar::historical::HistoricalCalendar;
use crate::calendar::lifa;
use crate::folk::zajie;
use crate::math::real::real;
use crate::quantity::angle::PlaneAngle;

/// 氣往亡日数的数法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QiWangWangCount {
    /// 交节当日为第一日（立春後七日 = 立春 + 6）
    Inclusive,
    /// 交节次日为第一日（立春後七日 = 立春 + 7）
    Exclusive,
}

/// 一日的铺注条件及其所依的年月
#[derive(Clone, Debug)]
pub struct PuzhuDay {
    /// 东八区民用日的儒略日数（JD + 0.5 取整）
    pub jdn: i64,
    pub lunar: ChineseLunarDate,
    /// 节气月序（寅月 = 0）、月干支序、年干支序（以立春为界）
    pub month_ord: u8,
    pub month_gz: u8,
    pub year_gz: u8,
    /// 本农历月朔日干支（反支用）
    pub shuo_gz: u8,
    /// 太阳所在宫的支序（以中气交宫：雨水後亥……大寒後子），無祿「太陽填實」用
    pub sun_palace_zhi: u8,
    pub input: PuzhuInput,
}

fn gz_of(jdn: i64) -> u8 {
    (jdn + 49).rem_euclid(60) as u8
}

fn gz_index(gan: u8, zhi: u8) -> u8 {
    (0..60u8).find(|k| k % 10 == gan % 10 && k % 12 == zhi % 12).unwrap()
}

/// 节气索引（0 春分 … 21 立春 … 23 驚蟄）是否为「节」，及其节序（0 立春 … 11 小寒）
fn jie_order(k: usize) -> Option<usize> {
    if k % 2 == 1 { Some(((k + 3) / 2) % 12) } else { None }
}

/// [j0, j1] 内每日的铺注条件。`historical` 为实历朔日表（无则按定气定朔现算）。
pub fn puzhu_days(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    historical: Option<&HistoricalCalendar>,
    j0: i64,
    j1: i64,
    count: QiWangWangCount,
) -> Vec<PuzhuDay> {
    let (y0, _, _) = Gregorian::from_julian_day(real(j0 as f64));
    let (y1, _, _) = Gregorian::from_julian_day(real(j1 as f64));
    // 节气（含前后一年）：(日, 索引)
    let mut terms: Vec<(i64, usize)> = Vec::new();
    for y in (y0 - 1)..=(y1 + 1) {
        for (k, t) in solar_term_jds_for_year_cached(vsop, y).iter().enumerate() {
            terms.push((tt_to_day_number_utc8(*t), k));
        }
    }
    terms.sort();
    let term_day = |y: i32, k: usize| tt_to_day_number_utc8(solar_term_jds_for_year_cached(vsop, y)[k]);
    // 朔弦望
    let tol = PlaneAngle::from_rad(real(1e-9));
    let phases: Vec<(i64, LunarPhase)> = lunar_phase_jds_in_range(vsop, elp, real((j0 - 2) as f64), real((j1 + 2) as f64), tol, 30)
        .into_iter()
        .map(|(t, p)| (tt_to_day_number_utc8(t), p))
        .collect();
    // 伏、社
    let mut fu_she: Vec<(i64, bool)> = Vec::new();
    for y in (y0 - 1)..=(y1 + 1) {
        let (a, b, c) = zajie::san_fu(term_day(y, 6), term_day(y, 9));
        fu_she.extend([(a, true), (b, true), (c, true)]);
        fu_she.push((zajie::she_jin_wu(term_day(y, 0)), false));
        fu_she.push((zajie::she_jin_wu(term_day(y, 12)), false));
    }
    // 农历岁数据
    let tol_nm = PlaneAngle::from_rad(real(1e-9));
    let years: Vec<_> = ((y0 - 1)..=(y1 + 1)).filter_map(|ly| lifa::resolve_year_data(historical, ly, vsop, elp, tol_nm, 30)).collect();

    let mut out = Vec::new();
    for jdn in j0..=j1 {
        let Some(lunar) = years.iter().find_map(|yd| from_day_number_in_year(jdn, yd, None)) else { continue };
        let gz = gz_of(jdn);
        // 节气月：最近一个已交之节（交节当日属新月）
        let (_, k_jie) = *terms.iter().filter(|(d, k)| *d <= jdn && k % 2 == 1).last().expect("节气");
        let month_ord = jie_order(k_jie).unwrap() as u8;
        // 年：立春为界
        let (gy, _, _) = Gregorian::from_julian_day(real(jdn as f64));
        let year = if jdn >= term_day(gy, 21) { gy } else { gy - 1 };
        let year_gz = (year - 4).rem_euclid(60) as u8;
        let month_gz = gz_index((year_gz % 5) * 2 + 2 + month_ord, 2 + month_ord);
        // 太阳宫：最近一个已交之中气
        let (_, k_zhong) = *terms.iter().filter(|(d, k)| *d <= jdn && k % 2 == 0).last().expect("中气");
        let sun_palace_zhi = (10 - (k_zhong as i32) / 2).rem_euclid(12) as u8;
        let on = |d: i64, ks: &[usize]| terms.iter().any(|(t, k)| *t == d && ks.contains(k));
        let fen_zhi = terms.iter().find(|(t, k)| *t == jdn && [0, 6, 12, 18].contains(k)).map(|(_, k)| match k {
            0 => FenZhi::ChunFen,
            6 => FenZhi::XiaZhi,
            12 => FenZhi::QiuFen,
            _ => FenZhi::DongZhi,
        });
        let tu_wang = terms.iter().any(|(t, k)| [21, 3, 9, 15].contains(k) && jdn >= zajie::tu_wang(*t) && jdn < *t);
        let qi_wang_wang = terms.iter().any(|(t, k)| {
            jie_order(*k).is_some_and(|o| {
                let n = QI_WANG_WANG_DAYS[o] as i64;
                jdn == t + if count == QiWangWangCount::Inclusive { n - 1 } else { n }
            })
        });
        let phase = phases.iter().find(|(d, _)| *d == jdn).map(|(_, p)| p.name());
        let shuo_gz = gz_of(jdn - (lunar.day as i64 - 1));
        let has_de = shen_for_day(month_ord, gz).iter().any(|(n, _)| n == "天德" || n == "月德");
        let wulu = wulu(gz, year_gz, month_gz, sun_palace_zhi, has_de) == Some(WuLu::Yes);
        let input = PuzhuInput {
            day_gz: gz,
            year_gan: year_gz % 10,
            lunar_month: lunar.month,
            lunar_day: lunar.day,
            days_in_month: lunar.days_in_month,
            si_li: on(jdn + 1, &[0, 6, 12, 18]),
            si_jue: on(jdn + 1, &[21, 3, 9, 15]),
            fen_zhi,
            tu_wang,
            fu: fu_she.iter().any(|(d, f)| *d == jdn && *f),
            she: fu_she.iter().any(|(d, f)| *d == jdn && !*f),
            phase,
            qi_wang_wang,
            wulu,
        };
        out.push(PuzhuDay { jdn, lunar, month_ord, month_gz, year_gz, shuo_gz, sun_palace_zhi, input });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jie_order_map() {
        assert_eq!(jie_order(21), Some(0)); // 立春
        assert_eq!(jie_order(23), Some(1)); // 驚蟄
        assert_eq!(jie_order(1), Some(2)); // 清明
        assert_eq!(jie_order(19), Some(11)); // 小寒
        assert_eq!(jie_order(0), None);
    }
}
