//! 古历复原模块：用分数推步完美复现中国古代历法逻辑。
//!
//! 设计为离线计算引擎：**常量配置 → 平朔/平气（线性时钟）→ [招差术修正，可选] → 历谱生成 → 干支输出**。
//! 全部使用有理数（`BigRational`），杜绝浮点精度漂移导致的千年误差。
//! 《三千五百年历日天象》等实历表**仅作验证用**，不作为计算依据。
//!
//! 第一版仅实现：配置 + 平朔平气 + 历谱生成（大小月、无中气置闰、建子），不包含招差术。

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::Signed;

/// 大整数有理数，用于历算（日法、岁实、朔策等均为分数）。
pub type DayFraction = BigRational;

/// 历法配置：历元、岁实、朔策、气策（= 岁实/24）。
/// 历元由合朔历元 JD 与冬至历元 JD 给出；距算 = 农历年 - 历元年，由调用方约定历元年。
#[derive(Clone, Debug)]
pub struct AncientConfig {
    /// 合朔历元（某次合朔的 JD，0h 为日界则通常为 xx.5）
    pub jd_epoch: DayFraction,
    /// 冬至历元（某年冬至的 JD）
    pub jdw: DayFraction,
    /// 朔策（朔望月长度，日）
    pub lunar: DayFraction,
    /// 岁实（回归年长度，日）
    pub solar: DayFraction,
    /// 气策 = solar / 24（两节气间隔）
    pub qi_interval: DayFraction,
}

impl AncientConfig {
    /// 从 岁实、朔策 与历元 JD 构造；气策自动为 solar/24。
    pub fn new(
        jd_epoch: DayFraction,
        jdw: DayFraction,
        lunar: DayFraction,
        solar: DayFraction,
    ) -> Self {
        let qi_interval = &solar / BigRational::from_integer(BigInt::from(24));
        AncientConfig {
            jd_epoch,
            jdw,
            lunar: lunar.clone(),
            solar,
            qi_interval,
        }
    }

    /// 东汉四分历（朔策 29+499/940，岁实 365.25）。历元取 doc 中参数。
    pub fn sifen() -> Self {
        let lunar = BigRational::new(BigInt::from(499), BigInt::from(940))
            + BigRational::from_integer(BigInt::from(29));
        let solar = BigRational::new(BigInt::from(97), BigInt::from(400))
            + BigRational::from_integer(BigInt::from(365));
        let jd_epoch = BigRational::from_integer(BigInt::from(1662610))
            + BigRational::new(BigInt::from(1), BigInt::from(2));
        let jdw = BigRational::from_integer(BigInt::from(1721050))
            + BigRational::new(BigInt::from(1), BigInt::from(2));
        Self::new(jd_epoch, jdw, lunar, solar)
    }

    /// 古六历通用朔策、岁实（与四分同）：lunar = 29+499/940，solar = 365.25。
    /// 历元见 [guliuli_calendar_parameters](doc/calendar/push-algorithms.md)，来源 ytliu0/ChineseCalendar。
    pub fn guliuli_lunar_solar() -> (DayFraction, DayFraction) {
        let lunar = BigRational::new(BigInt::from(499), BigInt::from(940))
            + BigRational::from_integer(BigInt::from(29));
        let solar = BigRational::new(BigInt::from(97), BigInt::from(400))
            + BigRational::from_integer(BigInt::from(365));
        (lunar, solar)
    }

    /// 古六历·周历（yEpoch=-104，建子）。历元见 doc 五、古六历。
    pub fn guliuli_zhou() -> Self {
        let (lunar, solar) = Self::guliuli_lunar_solar();
        let y_epoch = -104i32;
        let jd_epoch = BigRational::from_integer(BigInt::from(1683430))
            + BigRational::new(BigInt::from(5001), BigInt::from(10000));
        let jdw = &jd_epoch - BigRational::from_integer(BigInt::from(y_epoch)) * &solar;
        Self::new(jd_epoch.clone(), jdw, lunar, solar)
    }

    /// 古六历·黄帝历（yEpoch=170，建子）。
    pub fn guliuli_huangdi() -> Self {
        let (lunar, solar) = Self::guliuli_lunar_solar();
        let y_epoch = 170i32;
        let jd_epoch = BigRational::from_integer(BigInt::from(1783510))
            + BigRational::new(BigInt::from(5001), BigInt::from(10000));
        let jdw = &jd_epoch - BigRational::from_integer(BigInt::from(y_epoch)) * &solar;
        Self::new(jd_epoch.clone(), jdw, lunar, solar)
    }

    /// 古六历·殷历（yEpoch=-47，建丑）。
    pub fn guliuli_yin() -> Self {
        let (lunar, solar) = Self::guliuli_lunar_solar();
        let y_epoch = -47i32;
        let jd_epoch = BigRational::from_integer(BigInt::from(1704250))
            + BigRational::new(BigInt::from(5001), BigInt::from(10000));
        let jdw = &jd_epoch - BigRational::from_integer(BigInt::from(y_epoch)) * &solar;
        Self::new(jd_epoch.clone(), jdw, lunar, solar)
    }

    /// 古六历·鲁历（yEpoch=-481）。冬至历元 jdEpoch=1545730.5，合朔 jdEpoch_lunar = jdEpoch - lunar/19。
    pub fn guliuli_lu() -> Self {
        let (lunar, solar) = Self::guliuli_lunar_solar();
        let y_epoch = -481i32;
        let jd_epoch_dongzhi = BigRational::from_integer(BigInt::from(1545730))
            + BigRational::new(BigInt::from(5001), BigInt::from(10000));
        let jd_epoch = &jd_epoch_dongzhi - &lunar / BigRational::from_integer(BigInt::from(19));
        let jdw = &jd_epoch_dongzhi - BigRational::from_integer(BigInt::from(y_epoch)) * &solar;
        Self::new(jd_epoch, jdw, lunar, solar)
    }

    /// 古六历·颛顼历（yEpoch=14，建亥年终闰）。冬至历元 jdEpoch = jdEpoch_lunar - solar/8，合朔历元 = jdEpoch_lunar。
    pub fn guliuli_zhuanxu() -> Self {
        let (lunar, solar) = Self::guliuli_lunar_solar();
        let y_epoch = 14i32;
        let jd_epoch_lunar = BigRational::from_integer(BigInt::from(1726575))
            + BigRational::new(BigInt::from(5001), BigInt::from(10000));
        let jd_epoch_dongzhi = &jd_epoch_lunar - &solar / BigRational::from_integer(BigInt::from(8));
        let jdw = &jd_epoch_dongzhi - BigRational::from_integer(BigInt::from(y_epoch)) * &solar;
        Self::new(jd_epoch_lunar, jdw, lunar, solar)
    }

    /// 古六历·夏历（春秋用，yEpoch=444，建寅）。
    pub fn guliuli_xia1() -> Self {
        let (lunar, solar) = Self::guliuli_lunar_solar();
        let y_epoch = 444i32;
        let jd_epoch = BigRational::from_integer(BigInt::from(1883590))
            + BigRational::new(BigInt::from(5001), BigInt::from(10000));
        let jdw = &jd_epoch - BigRational::from_integer(BigInt::from(y_epoch)) * &solar;
        Self::new(jd_epoch.clone(), jdw, lunar, solar)
    }

    /// 古六历·夏历（战国用，yEpoch=444）。冬至历元 jdEpoch=1883590.5，合朔 jdEpoch_lunar = jdEpoch - solar/6。
    pub fn guliuli_xia2() -> Self {
        let (lunar, solar) = Self::guliuli_lunar_solar();
        let y_epoch = 444i32;
        let jd_epoch_dongzhi = BigRational::from_integer(BigInt::from(1883590))
            + BigRational::new(BigInt::from(5001), BigInt::from(10000));
        let jd_epoch = &jd_epoch_dongzhi - &solar / BigRational::from_integer(BigInt::from(6));
        let jdw = &jd_epoch_dongzhi - BigRational::from_integer(BigInt::from(y_epoch)) * &solar;
        Self::new(jd_epoch, jdw, lunar, solar)
    }
}

/// 日界：JD 取整为「日」的规则。平朔/平气时间戳 → 当日 0h 的 JD 整数部分。
/// 约定：floor(JD + 0.5) 为「第几天」的序号。
#[inline]
fn jd_to_day_index(jd: &DayFraction) -> i64 {
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));
    let jd_plus = jd + &half;
    floor_ratio_to_i64(&jd_plus)
}

fn floor_ratio_to_i64(r: &DayFraction) -> i64 {
    let (q, _) = r.numer().div_mod_floor(r.denom());
    let neg = q.is_negative();
    use std::convert::TryInto;
    q.try_into().unwrap_or(if neg { i64::MIN } else { i64::MAX })
}

/// 平气：冬至为起点，间隔 气策，共 24 节气。
/// 返回 农历年 n 的 24 个平气时刻（DayFraction，距历元或绝对 JD 均可，此处用 JD）。
/// 约定：n 年冬至 ≈ jdw + n * solar；第 0 气为冬至，第 1 小寒，…，第 23 为下一冬至前。
pub fn mean_solar_terms(config: &AncientConfig, lunar_year: i32) -> Vec<DayFraction> {
    let n = BigRational::from_integer(BigInt::from(lunar_year));
    let winter = &config.jdw + &n * &config.solar;
    (0..24)
        .map(|i| {
            let i = BigRational::from_integer(BigInt::from(i));
            &winter + &i * &config.qi_interval
        })
        .collect()
}

/// 平朔：求农历年 n 的正月朔 JD（分数）。
/// 建正：以「立春附近」为岁首界；正月朔 = 不晚于 Z1 日界的最近合朔。
/// Z1 = jdw + (n + 1/6)*solar（立春附近），Z1_日界 = floor(Z1+0.5)+0.5；
/// i1 = floor((Z1_日界 - jd_epoch) / lunar)，m1 = jd_epoch + i1*lunar。
pub fn first_month_new_moon(config: &AncientConfig, lunar_year: i32) -> DayFraction {
    let n = BigRational::from_integer(BigInt::from(lunar_year));
    let one_sixth = BigRational::new(BigInt::from(1), BigInt::from(6));
    let z1 = &config.jdw + (&n + &one_sixth) * &config.solar;
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));
    let z1_noon = &z1 + &half;
    let z1_day = floor_ratio_to_i64(&z1_noon);
    let z1_bound = BigRational::from_integer(BigInt::from(z1_day)) + half;
    let diff = &z1_bound - &config.jd_epoch;
    let i1_ratio = &diff / &config.lunar;
    let i1 = floor_ratio_to_i64(&i1_ratio);
    let i1 = BigRational::from_integer(BigInt::from(i1));
    &config.jd_epoch + &i1 * &config.lunar
}

/// 农历年 n 是否有闰月（13 个月）：若正月朔 + 13*朔策 仍在下一岁 Z1 日界之前则有闰。
fn has_leap_month(config: &AncientConfig, lunar_year: i32, m1: &DayFraction) -> bool {
    let n = BigRational::from_integer(BigInt::from(lunar_year));
    let one_sixth = BigRational::new(BigInt::from(1), BigInt::from(6));
    let z1 = &config.jdw + (&n + &one_sixth) * &config.solar;
    let z1b = &z1 + &config.solar;
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));
    let z1b_noon = &z1b + &half;
    let thirteen = BigRational::from_integer(BigInt::from(13));
    let m1_13 = m1 + &thirteen * &config.lunar;
    let m1_13_noon = &m1_13 + &half;
    m1_13_noon <= z1b_noon
}

/// 单月是否「无中气」：若该月内没有中气日，则为无中气月（可置闰）。
/// 月朔 JD 为 start_jd，月长 lunar；去年冬至 w0 = jdw + (n-1)*solar；第 k 个中气 (k=1..12) 为 w0 + k*(solar/12)。
fn month_has_no_zhong_qi(
    config: &AncientConfig,
    lunar_year: i32,
    start_jd: &DayFraction,
) -> bool {
    let w0 = &config.jdw
        + BigRational::from_integer(BigInt::from(lunar_year - 1)) * &config.solar;
    let dmqi = &config.solar / BigRational::from_integer(BigInt::from(12));
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));
    let end_jd = start_jd + &config.lunar - BigRational::new(BigInt::from(1), BigInt::from(100));
    let start_day = floor_ratio_to_i64(&(start_jd + &half));
    let end_day = floor_ratio_to_i64(&(end_jd + &half));
    for k in 1..=12 {
        let qi_jd = &w0 + BigRational::from_integer(BigInt::from(k)) * &dmqi + &half;
        let qi_day = floor_ratio_to_i64(&qi_jd);
        if qi_day >= start_day && qi_day <= end_day {
            return false;
        }
    }
    true
}

/// 历谱：农历年 n 的 12 或 13 个月，每月的朔日 JD（分数）、是否闰月、大小月（日数 29 或 30）。
#[derive(Clone, Debug)]
pub struct MonthEntry {
    /// 该月朔的 JD（分数）
    pub new_moon_jd: DayFraction,
    /// 是否闰月
    pub is_leap: bool,
    /// 月份名 1..12（闰月与前一月同号）
    pub month_1based: u8,
    /// 月长（日数，29 或 30）
    pub days: u8,
}

/// 生成农历年 n 的历谱（平朔 + 无中气置闰）。
pub fn calendar_year(config: &AncientConfig, lunar_year: i32) -> Vec<MonthEntry> {
    let m1 = first_month_new_moon(config, lunar_year);
    let has_leap = has_leap_month(config, lunar_year, &m1);
    let mut out = Vec::with_capacity(13);
    let mut i: i32 = 0;
    let mut inserted_leap = false;

    if !has_leap {
        for j in 0..12 {
            let start = &m1 + BigRational::from_integer(BigInt::from(j)) * &config.lunar;
            let next = &m1 + BigRational::from_integer(BigInt::from(j + 1)) * &config.lunar;
            let days = (jd_to_day_index(&next) - jd_to_day_index(&start)) as u8;
            out.push(MonthEntry {
                new_moon_jd: start,
                is_leap: false,
                month_1based: (j + 1) as u8,
                days,
            });
        }
        return out;
    }

    for month_1 in 1..=12 {
        let start = &m1 + BigRational::from_integer(BigInt::from(i)) * &config.lunar;
        let no_zhong = month_has_no_zhong_qi(config, lunar_year, &start);
        let next_start = &m1 + BigRational::from_integer(BigInt::from(i + 1)) * &config.lunar;
        let days = (jd_to_day_index(&next_start) - jd_to_day_index(&start)) as u8;

        out.push(MonthEntry {
            new_moon_jd: start.clone(),
            is_leap: false,
            month_1based: month_1 as u8,
            days,
        });
        i += 1;

        if !inserted_leap && no_zhong {
            inserted_leap = true;
            let leap_start = &m1 + BigRational::from_integer(BigInt::from(i)) * &config.lunar;
            let leap_next = &m1 + BigRational::from_integer(BigInt::from(i + 1)) * &config.lunar;
            let leap_days = (jd_to_day_index(&leap_next) - jd_to_day_index(&leap_start)) as u8;
            out.push(MonthEntry {
                new_moon_jd: leap_start.clone(),
                is_leap: true,
                month_1based: month_1 as u8,
                days: leap_days,
            });
            i += 1;
        }
    }
    if !inserted_leap {
        let leap_start = &m1 + BigRational::from_integer(BigInt::from(12)) * &config.lunar;
        let leap_next = &m1 + BigRational::from_integer(BigInt::from(13)) * &config.lunar;
        out.push(MonthEntry {
            new_moon_jd: leap_start.clone(),
            is_leap: true,
            month_1based: 12,
            days: (jd_to_day_index(&leap_next) - jd_to_day_index(&leap_start)) as u8,
        });
    }
    out
}

/// 干支日序（0..60）从 JD（分数）计算：日序 = (floor(JD+0.5) - epoch_day) mod 60。
/// 与 [crate::calendar::gan_zhi] 的现代约定（2000-01-01 庚辰）可换算时，传入对应 epoch_day 与历元干支序。
pub fn jd_to_gan_zhi_index_epoch(jd: &DayFraction, epoch_day: i64, _epoch_gan_zhi: i64) -> u8 {
    let day = jd_to_day_index(jd);
    let idx = (day - epoch_day).rem_euclid(60);
    (idx as i32).rem_euclid(60) as u8
}

/// 将历谱中某月朔的 JD（分数）转为儒略日整数（0h 日界），便于与实历表或《三千五百年历日天象》校验。
#[inline]
pub fn month_new_moon_to_jd_day(entry: &MonthEntry) -> i64 {
    jd_to_day_index(&entry.new_moon_jd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sifen_config() {
        let c = AncientConfig::sifen();
        assert!(c.lunar.numer().is_positive());
        assert!(c.solar.numer().is_positive());
    }

    #[test]
    fn first_month_new_moon_sifen() {
        let c = AncientConfig::sifen();
        let m1 = first_month_new_moon(&c, -105);
        let day = jd_to_day_index(&m1);
        // 平朔与实历（三千五百年）可能差数日；此处仅校验为合理儒略日（约 -105 年前后）
        assert!(day > 1682000 && day < 1684000, "m1 day = {}", day);
    }

    #[test]
    fn calendar_year_12_months() {
        let c = AncientConfig::sifen();
        let months = calendar_year(&c, 0);
        assert!(months.len() >= 12 && months.len() <= 13);
        for m in &months {
            assert!(m.days == 29 || m.days == 30);
        }
    }

    #[test]
    fn guliuli_six_calendars() {
        let configs = [
            AncientConfig::guliuli_zhou(),
            AncientConfig::guliuli_huangdi(),
            AncientConfig::guliuli_yin(),
            AncientConfig::guliuli_lu(),
            AncientConfig::guliuli_zhuanxu(),
            AncientConfig::guliuli_xia1(),
            AncientConfig::guliuli_xia2(),
        ];
        for c in &configs {
            let months = calendar_year(c, 0);
            assert!(months.len() >= 12 && months.len() <= 13, "古六历应得 12 或 13 月");
            for m in &months {
                assert!(m.days == 29 || m.days == 30);
            }
        }
    }
}
