//! 由历表与历法推一日的铺注条件（[`PuzhuInput`]）：节气、朔弦望、农历日、伏社、土王用事、氣往亡。
//!
//! 口径：
//! - 日以东八区民用日计（[`tt_to_day_number_utc8`]）；月建以交节为界，交节当日属新月（同月表、建除）。
//! - 年（上朔、年神）以立春为界。
//! - 农历日期按主序列行用之历（[`lifa::resolve_year_data`]）：实物历日、按术推步、今人历表、现算依次取。
//! - 伏：初伏、中伏、末伏三日（[`zajie::san_fu`]）；社：二分前後近戊（《協紀》卷十二引歷例，[`zajie::she_jin_wu`]）。
//! - 土王用事：四立前十八日起至四立前一日（[`zajie::tu_wang`]）。
//! - 氣往亡：「皆自交節日數之」，交节当日为第一日（见 [`QiWangWangCount`]）。

use super::{wulu, FenZhi, PuzhuInput, WuLu, QI_WANG_WANG_DAYS};
use crate::folk::shensha::shen_for_day;
use crate::astronomy::aspects::{lunar_phase_jds_in_range, solar_term_jds_for_year_cached, LunarPhase};
use crate::astronomy::ephemeris::{Elpmpp02Data, Vsop87};
use crate::calendar::chinese_lunar::{from_day_number_in_year, tt_to_day_number_utc8, ChineseLunarDate};
use crate::calendar::gregorian::Gregorian;
use crate::calendar::historical::ShuoriTable;
use crate::calendar::lifa;
use crate::folk::zajie;
use crate::math::real::{real, RealOps};
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
    /// 当日交节（月建换月之日）
    pub jiao_jie: bool,
    /// 交节时刻（东八区钟点，0—24）；非交节日为 None
    pub jiao_jie_hour: Option<f64>,
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

/// 铺注所用节气（交节之日与时刻）的来源。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TermSource {
    /// 缺省：后编行用期（1742—1913，`main_series_calendar_periods.csv`「时宪历（历象考成后编）」与民国初沿用）取《历象考成后编》
    /// 日躔推得之京师用时（[crate::calendar::shixian::jieqi]），即时宪书定日、印交节时刻所据；其余年份取现代历表（东八区）。
    /// 依据：时宪书书印交节时刻 9 例中 7 例为后编用时截至刻（doc/folk/shensha.md「交节时刻界限」），现代历表与书印之差
    /// 随节气在 −0.5 至 +0.4 时之间，不能以常数补正。
    #[default]
    Auto,
    /// 全程现代历表（VSOP87 视黄经，东八区钟点）
    Modern,
}

/// 后编所据之年（公历年）
pub const HOUBIAN_YEARS: std::ops::RangeInclusive<i32> = 1742..=1913;

/// 一段时间的节气表（东八区日）：铺注逐日推算所需的节气、交节时刻、伏社
#[derive(Clone, Debug)]
pub struct TermTable {
    /// (日, 节气索引 0 春分 … 23 驚蟄)，按日升序
    pub terms: Vec<(i64, usize)>,
    /// 交节之日与交节时刻（钟点：现代历表为东八区，后编为京师用时）
    pub jie_times: Vec<(i64, f64)>,
    /// 伏日（true）、社日（false）
    pub fu_she: Vec<(i64, bool)>,
    /// 各公历年立春之日（定年用）
    li_chun: Vec<(i32, i64)>,
}

impl TermTable {
    /// 覆盖公历 y0—y1 年（各向外多取一年），节气来源取缺省 [TermSource::Auto]
    pub fn new(vsop: &Vsop87, y0: i32, y1: i32) -> Self {
        Self::with_source(vsop, y0, y1, TermSource::Auto)
    }

    /// 同 [Self::new]，指定节气来源
    pub fn with_source(vsop: &Vsop87, y0: i32, y1: i32, source: TermSource) -> Self {
        let mut terms = Vec::new();
        let mut jie_times = Vec::new();
        let mut fu_she = Vec::new();
        let mut li_chun = Vec::new();
        for y in (y0 - 1)..=(y1 + 1) {
            // (日, 钟点)，索引 k 同现代表（0 春分 … 23 驚蟄，皆在公历 y 年内）
            let dh: Vec<(i64, f64)> = if source == TermSource::Auto && HOUBIAN_YEARS.contains(&y) {
                // 后编冬至岁 y：k_h = 0 冬至（y−1 年十二月）…23 大雪、24 次冬至（y 年十二月）；黄经 15k = 270 + 15k_h
                let h = crate::calendar::shixian::jieqi(y);
                (0..24).map(|k| {
                    let kh = (k + 6) % 24;
                    let (d, hour, _) = h[if kh == 0 { 24 } else { kh }];
                    (d, hour)
                }).collect()
            } else {
                solar_term_jds_for_year_cached(vsop, y).iter().map(|jd| {
                    let x = jd.as_f64() + 8.0 / 24.0 + 0.5;
                    (tt_to_day_number_utc8(*jd), (x - x.floor()) * 24.0)
                }).collect()
            };
            for (k, &(d, hour)) in dh.iter().enumerate() {
                terms.push((d, k));
                if k % 2 == 1 {
                    jie_times.push((d, hour));
                }
            }
            let day = |k: usize| dh[k].0;
            let (a, b, c) = zajie::san_fu(day(6), day(9));
            fu_she.extend([(a, true), (b, true), (c, true)]);
            fu_she.push((zajie::she_jin_wu(day(0)), false));
            fu_she.push((zajie::she_jin_wu(day(12)), false));
            li_chun.push((y, day(21)));
        }
        terms.sort();
        Self { terms, jie_times, fu_she, li_chun }
    }

    /// 一日的铺注条件。`lunar` 为该日农历（与所显示者一致）；`phase` 为该日所值朔、上弦、望、下弦（无则 None）
    pub fn day(&self, jdn: i64, lunar: ChineseLunarDate, phase: Option<&'static str>, count: QiWangWangCount) -> PuzhuDay {
        let terms = &self.terms;
        let gz = gz_of(jdn);
        // 节气月：最近一个已交之节（交节当日属新月）
        let (_, k_jie) = *terms.iter().filter(|(d, k)| *d <= jdn && k % 2 == 1).last().expect("节气");
        let month_ord = jie_order(k_jie).unwrap() as u8;
        // 年：立春为界
        let (gy, _, _) = Gregorian::from_julian_day(real(jdn as f64));
        let lc = self.li_chun.iter().find(|(y, _)| *y == gy).map(|(_, d)| *d).expect("立春");
        let year = if jdn >= lc { gy } else { gy - 1 };
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
            fu: self.fu_she.iter().any(|(d, f)| *d == jdn && *f),
            she: self.fu_she.iter().any(|(d, f)| *d == jdn && !*f),
            phase,
            qi_wang_wang,
            wulu,
            month_ord,
            // 本月之节已交、中气未交
            before_zhong_qi: terms.iter().filter(|(d, _)| *d <= jdn).last().is_some_and(|(_, k)| k % 2 == 1),
        };
        let jiao_jie = terms.iter().any(|(t, k)| *t == jdn && k % 2 == 1);
        let jiao_jie_hour = self.jie_times.iter().find(|(d, _)| *d == jdn).map(|(_, h)| *h);
        PuzhuDay { jdn, lunar, month_ord, month_gz, year_gz, shuo_gz, jiao_jie, jiao_jie_hour, sun_palace_zhi, input }
    }
}

/// [j0, j1] 内每日的铺注条件。`historical` 为实历朔日表（无则按定气定朔现算）。
pub fn puzhu_days(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    historical: Option<&ShuoriTable>,
    j0: i64,
    j1: i64,
    count: QiWangWangCount,
) -> Vec<PuzhuDay> {
    let (y0, _, _) = Gregorian::from_julian_day(real(j0 as f64));
    let (y1, _, _) = Gregorian::from_julian_day(real(j1 as f64));
    let table = TermTable::new(vsop, y0, y1);
    let tol = PlaneAngle::from_rad(real(1e-9));
    let phases: Vec<(i64, LunarPhase)> = lunar_phase_jds_in_range(vsop, elp, real((j0 - 2) as f64), real((j1 + 2) as f64), tol, 30)
        .into_iter()
        .map(|(t, p)| (tt_to_day_number_utc8(t), p))
        .collect();
    let years: Vec<_> = ((y0 - 1)..=(y1 + 1)).filter_map(|ly| lifa::resolve_year_data(historical, ly, vsop, elp, tol, 30)).collect();
    (j0..=j1)
        .filter_map(|jdn| {
            let lunar = years.iter().find_map(|yd| from_day_number_in_year(jdn, yd, None))?;
            let phase = phases.iter().find(|(d, _)| *d == jdn).map(|(_, p)| p.name());
            Some(table.day(jdn, lunar, phase, count))
        })
        .collect()
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
