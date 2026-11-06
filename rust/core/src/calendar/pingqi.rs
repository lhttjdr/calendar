//! 平气：按当时行用历法推算的二十四气（独立基础层）。
//!
//! 平气历以天正冬至为起点，每气相隔气策（岁实的二十四分之一）。各历岁实、历元不同，
//! 所以平气要按行用历逐朝推算。参数表 `data/calendar/pingqi_lifa.csv` 每部历一行：
//! 日法、岁实（分）、某一岁天正冬至的大余（甲子起算的日序）与小余（夜半后的分）、
//! 都城地方时经度、出处与核对状态。只收已据正史律历志核过的历；表外年代不给平气。
//! 各行顺序不限（多人分头追加），行用年不得重叠，由测试检查。
//!
//! 推算：岁 Y 的天正冬至（在公历 Y−1 年 12 月）= 基准冬至 + (Y − 基准岁) × 岁实分 / 日法，
//! 第 q 气（冬至起算）再加 q × 岁实 / 24。地方平时减经度/360 日为 UT，再加 ΔT 为 TT。
//! 详见 `doc/folk/ganzhi-calendar.md`「平气」一节。

use once_cell::sync::Lazy;

use crate::calendar::gregorian::Gregorian;
use crate::math::real::{real, real_const, Real, RealOps};

const CSV: &str = include_str!("../../../../data/calendar/pingqi_lifa.csv");

/// 一部平气历的参数
#[derive(Clone, Debug)]
pub struct PingqiLi {
    pub id: String,
    pub name: String,
    /// 行用起止（公元年，天文纪年，含两端）
    pub from_year: i32,
    pub to_year: i32,
    pub ri_fa: i64,
    pub sui_shi_fen: i64,
    /// 基准岁：其天正冬至在公历 (基准岁 − 1) 年 12 月
    pub base_sui: i32,
    /// 基准天正冬至的大余（甲子 = 0）与小余（分，日法为一日）
    pub base_da_yu: i64,
    pub base_xiao_yu: i64,
    /// 推步所据地方时的东经（度）
    pub longitude_deg: f64,
    pub source: String,
    pub status: String,
}

static TABLE: Lazy<Vec<PingqiLi>> = Lazy::new(|| {
    CSV.lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            assert!(f.len() >= 12, "pingqi_lifa.csv 列数不足：{l}");
            let i = |k: usize| f[k].trim().parse::<i64>().unwrap_or_else(|_| panic!("第 {k} 列不是整数：{l}"));
            PingqiLi {
                id: f[0].to_string(),
                name: f[1].to_string(),
                from_year: i(2) as i32,
                to_year: i(3) as i32,
                ri_fa: i(4),
                sui_shi_fen: i(5),
                base_sui: i(6) as i32,
                base_da_yu: i(7),
                base_xiao_yu: i(8),
                longitude_deg: f[9].trim().parse().expect("经度"),
                source: f[10].to_string(),
                status: f[11..].join(","),
            }
        })
        .collect()
});

/// 全部平气历参数
pub fn pingqi_table() -> &'static [PingqiLi] {
    &TABLE
}

/// 公元 year 年行用的平气历；表外返回 None。
pub fn pingqi_li_for_year(year: i32) -> Option<&'static PingqiLi> {
    TABLE.iter().find(|li| year >= li.from_year && year <= li.to_year)
}

impl PingqiLi {
    /// 岁实（日）
    pub fn sui_shi_days(&self) -> Real {
        real(self.sui_shi_fen as f64) / real(self.ri_fa as f64)
    }

    /// 基准岁天正冬至的地方时 JD：在 (基准岁 − 1) 年 12 月下旬附近找干支序为大余的那一日，加小余。
    pub fn base_winter_solstice_jd_local(&self) -> Real {
        // 天正冬至约略时刻：2000-12-21 前后（JD 2451900）按回归年外推，误差不过数日
        let approx = real_const(2451900.0) + real((self.base_sui - 1 - 2000) as f64) * real_const(365.2422);
        let approx_jdn = (approx + real_const(0.5)).floor().as_f64() as i64;
        let gz = (approx_jdn + 49).rem_euclid(60);
        let mut d = (self.base_da_yu - gz).rem_euclid(60);
        if d > 30 {
            d -= 60;
        }
        let jdn = approx_jdn + d;
        real(jdn as f64) - real_const(0.5) + real(self.base_xiao_yu as f64) / real(self.ri_fa as f64)
    }
}

/// 与给定定气同名的平气，JD(TT)。`dingqi_tt` 为定气时刻，`qi_from_winter_solstice` 为该气距冬至的气数
/// （冬至 0，小寒 1，…，大雪 23）。行用历按定气所在公元年选取；表外返回 None。
pub fn pingqi_tt(dingqi_tt: Real, qi_from_winter_solstice: i32) -> Option<(&'static PingqiLi, Real)> {
    use crate::astronomy::time::delta_t;
    let (year, _, _) = Gregorian::from_julian_day(dingqi_tt);
    let li = pingqi_li_for_year(year)?;
    let s = li.sui_shi_days();
    let w_ut = li.base_winter_solstice_jd_local() - real(li.longitude_deg) / real_const(360.0);
    let q = real(qi_from_winter_solstice.rem_euclid(24) as f64);
    let base = w_ut + q * s / real_const(24.0);
    let j = ((dingqi_tt - base) / s + real_const(0.5)).floor();
    let ut = base + j * s;
    Some((li, ut + delta_t(ut).in_days_value()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gan_zhi::GAN_ZHI_60;

    /// 授时历元：至元十八年辛巳岁前天正冬至为己未日、夜半后 0.06 日（气应 55.0600），儒略历 1280-12-14。
    #[test]
    fn shoushi_epoch() {
        let li = pingqi_table().iter().find(|l| l.id == "shoushi").unwrap();
        let jd = li.base_winter_solstice_jd_local().as_f64();
        assert!((jd - 2188925.56).abs() < 1e-9, "{jd}");
        let jdn = (jd + 0.5).floor() as i64;
        assert_eq!(GAN_ZHI_60[((jdn + 49).rem_euclid(60)) as usize], "己未");
    }

    #[test]
    fn table_rows_cover_without_overlap() {
        let mut t: Vec<_> = pingqi_table().iter().collect();
        assert!(!t.is_empty());
        t.sort_by_key(|l| l.from_year);
        for (a, b) in t.iter().zip(t.iter().skip(1)) {
            assert!(a.to_year < b.from_year, "{} 与 {} 行用年重叠", a.name, b.name);
        }
        assert!(pingqi_li_for_year(1645).is_none(), "清时宪历用定气");
    }
}
