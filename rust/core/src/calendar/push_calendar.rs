//! 推步历：平朔 + 无中气置闰，用于各政权历法（蜀四分、吴乾象等）；定朔的金大明历在 [super::daming_li]，经本模块政权表统一出口。
//! 算法与参数见 doc/calendar/push-algorithms.md。

use super::gregorian;
use crate::math::real::RealOps;

/// 历术参数：朔策、岁实、历元（合朔、冬至），舍入修正。
#[derive(Clone, Debug)]
pub struct LiParams {
    pub lunar: f64,
    pub solar: f64,
    pub jd_epoch: f64,
    pub jdw: f64,
    pub dtc_round: f64,
    pub dts_round: f64,
}

impl LiParams {
    fn jd_epoch_adj(&self) -> f64 {
        self.jd_epoch + self.dtc_round
    }
    fn jdw_adj(&self) -> f64 {
        self.jdw + self.dts_round
    }
}

/// 平朔 + 无中气置闰：对农历年 N，算正月朔 m1、闰月号、各月朔日（JDN，共 12 或 13，有闰时闰月插在其所闰之月后）。
///
/// 规则（平朔平气历通例，见 doc/calendar/push-algorithms.md）：
/// - 平气：冬至历元 jdw 起每 solar/24 一气；N 岁雨水（正月中气）Z1 = jdw + (N + 1/6)·solar，其后中气相隔 solar/12；
/// - 平朔：历元 jd_epoch 起每 lunar 一朔；正月为含雨水之月，即雨水当日或之前最近的朔；
/// - 置闰：自正月朔起 13 个朔望月仍在下岁雨水之前，则本岁有闰；闰月为本岁第一个不含中气之月（该月朔日至下月朔日前一日
///   都没有中气所在之日），闰在其前一月之后，称「闰某月」。
/// 气、朔的「日」均取 floor(JD + 0.5)（JD 自正午起算，日界在半夜）。
fn year_new_moons(p: &LiParams, lunar_year: i32) -> (f64, u8, Vec<f64>) {
    let jde = p.jd_epoch_adj();
    let jdw = p.jdw_adj();
    let (lunar, solar) = (p.lunar, p.solar);
    let n = lunar_year as f64;
    let day = |jd: f64| (jd + 0.5).floor();
    let z1 = jdw + (n + 1.0 / 6.0) * solar;
    let i1 = ((day(z1) + 0.5 - jde) / lunar).floor();
    let m1 = jde + i1 * lunar;
    // 下岁雨水所在日的次日零时（半夜）
    let z1_next = day(z1 + solar) + 0.5;
    let has_leap = m1 + 13.0 * lunar <= z1_next;
    let n_months = if has_leap { 13 } else { 12 };
    let starts: Vec<f64> = (0..=n_months).map(|j| day(m1 + j as f64 * lunar)).collect();
    let mut leap_month = 0u8;
    if has_leap {
        // 第 k 个中气（k = 0 为雨水）所在日
        let zq: Vec<f64> = (0..=12).map(|k| day(z1 + k as f64 * solar / 12.0)).collect();
        for j in 0..n_months {
            let (a, b) = (starts[j], starts[j + 1]);
            if !zq.iter().any(|&q| q >= a && q < b) {
                leap_month = j as u8; // 前有 j 个正常月，闰在第 j 月之后
                break;
            }
        }
        assert!(leap_month > 0, "推步：闰岁 {lunar_year} 找不到无中气之月或正月无中气");
    }
    (m1, leap_month, starts[..n_months].to_vec())
}

/// 某历术下农历年 N 的各月：(朔日 JDN, 月序 1..12, 是否闰月)，按时间先后。JDN 与日期约定无关，史料（民用历）与前端
/// （前推格里高利历）各自换算。
pub fn year_months(params: &LiParams, lunar_year: i32) -> Vec<(i64, u8, bool)> {
    let (_m1, leap_month, starts) = year_new_moons(params, lunar_year);
    starts
        .iter()
        .enumerate()
        .map(|(k, &jd)| {
            let (month, leap) = if leap_month > 0 && k == leap_month as usize {
                (leap_month, true)
            } else if leap_month > 0 && k > leap_month as usize {
                (k as u8, false)
            } else {
                (k as u8 + 1, false)
            };
            (jd as i64, month, leap)
        })
        .collect()
}

/// 精确整数的平朔平气推步：上元为甲子日夜半，合朔与气首（冬至，或雨水）同时。
///
/// - 朔：上元起第 k 朔在第 ⌊k·月法/日法⌋ 日（月法即通数，朔策 = 月法/日法）；
/// - 气：岁实 = `sui_num/sui_den` 日；`year0` 岁天正冬至距上元 `ji0` 岁（术文「积年」，算外），即第 ji0·岁实 日；
///   气首为雨水者（元嘉历），上元是雨水，冬至在其前 1/6 岁；中气每 1/12 岁，取所在之日；
/// - 置闰：十一月为含冬至之月；两冬至间 13 月者，以其间第一个无中气之月为闰，闰月名从前月；
/// - 历元的儒略日数只由术文定：上元须是甲子日，且 `year0` 岁天正冬至须落在实际冬至前后 30 日内（60 日一周，唯一）。
///   不借今人历表对齐。
///
/// 与 [LiParams]（浮点、历元取自廖育栋程序）不同，本式全部是整数运算，参数逐项出自术文，见
/// `doc/calendar/rili-sources.md`「第二期」。
#[derive(Clone, Copy, Debug)]
pub struct ExactPingLi {
    /// 月法（通数）
    pub yue: i64,
    /// 日法
    pub ri: i64,
    /// 岁实分子、分母（日）
    pub sui_num: i64,
    pub sui_den: i64,
    /// 积年所指之岁（天文纪年，该岁天正冬至在上一公历年十二月）
    pub year0: i32,
    /// 上元至 year0 岁天正冬至的积年（算外）
    pub ji0: i64,
    /// 上元为雨水（元嘉历），否则为冬至
    pub qishou_yushui: bool,
}

impl ExactPingLi {
    /// y 岁天正冬至距上元的日数 × 12·岁实分母（整数）。
    fn dongzhi_12(&self, y: i32) -> i128 {
        let ji = self.ji0 as i128 + (y - self.year0) as i128;
        let q = if self.qishou_yushui { -2 * self.sui_num as i128 } else { 0 };
        12 * ji * self.sui_num as i128 + q
    }

    fn day_of_12(&self, v: i128) -> i64 {
        v.div_euclid(12 * self.sui_den as i128) as i64
    }

    fn shuo_day(&self, k: i64) -> i64 {
        ((k as i128 * self.yue as i128).div_euclid(self.ri as i128)) as i64
    }

    /// 上元（第 0 日）的儒略日数：甲子日，且 year0 岁天正冬至距实际冬至（平均回归年自 2000 年冬至外推）不过 30 日。
    pub fn epoch_jdn(&self) -> i64 {
        let d = self.day_of_12(self.dongzhi_12(self.year0));
        // 2000-12-21 冬至 JDN 2451900；回归年 365.2422 日
        let approx = 2451900 + (36_524_220i64 * ((self.year0 - 1 - 2000) as i64)).div_euclid(100_000);
        let lo = approx - d - 30;
        (lo..=lo + 60).find(|&e| (e + 49).rem_euclid(60) == 0 && (e + d - approx).abs() <= 30).expect("历元")
    }

    /// 冬至岁 y（y−1 年十一月至 y 年十月）各月：(上元起日数, 岁, 月, 闰)。
    fn span(&self, y: i32) -> Vec<(i64, i32, u8, bool)> {
        let d0 = self.dongzhi_12(y);
        let d1 = self.dongzhi_12(y + 1);
        let (dz0, dz1) = (self.day_of_12(d0), self.day_of_12(d1));
        // 含冬至之月的朔
        let mut k = ((dz0 as i128 * self.ri as i128) / self.yue as i128) as i64 - 2;
        while self.shuo_day(k + 1) <= dz0 {
            k += 1;
        }
        let mut starts = Vec::with_capacity(15);
        while self.shuo_day(k) <= dz1 {
            starts.push(self.shuo_day(k));
            k += 1;
        }
        // 每月 [starts[i], starts[i+1])；最后一个含下一冬至，属下一冬至岁
        let n = starts.len() - 1;
        let zq: Vec<i64> = (0..13).map(|i| self.day_of_12(d0 + i as i128 * self.sui_num as i128)).collect();
        let leap = if n == 13 {
            (1..13).find(|&i| !zq.iter().any(|&z| starts[i] <= z && z < starts[i + 1]))
        } else {
            None
        };
        let mut out = Vec::with_capacity(n);
        let (mut lab, mut yy) = (11u8, y - 1);
        let (mut plab, mut pyy) = (11u8, y - 1);
        for (i, &s) in starts[..n].iter().enumerate() {
            if Some(i) == leap {
                out.push((s, pyy, plab, true));
                continue;
            }
            out.push((s, yy, lab, false));
            (plab, pyy) = (lab, yy);
            lab += 1;
            if lab == 13 {
                (lab, yy) = (1, y);
            }
        }
        out
    }

    /// 农历年 y（正月至十二月，含闰）各月：(朔日 JDN, 月, 闰否)。
    pub fn year_months(&self, lunar_year: i32) -> Vec<(i64, u8, bool)> {
        let e = self.epoch_jdn();
        self.span(lunar_year)
            .into_iter()
            .chain(self.span(lunar_year + 1))
            .filter(|&(_, yy, _, _)| yy == lunar_year)
            .map(|(s, _, m, leap)| (s + e, m, leap))
            .collect()
    }
}

/// 三统历（《汉书·律历志》）：日法 81，月法 2392；统法 1539，岁实 562120/1539；上元至太初元年（前 104，天文纪年 −103）
/// 天正冬至 143127 岁（与 `han_pingshuo_check.py` 同一积年口径）。
pub fn santong_exact() -> ExactPingLi {
    ExactPingLi { yue: 2392, ri: 81, sui_num: 562120, sui_den: 1539, year0: -104, ji0: 143126, qishou_yushui: false }
}

/// 元嘉历（何承天，《宋书·律历志下》）：日法 752，通数 22207；度法 304，岁实 111035/304；上元庚辰甲子纪首（雨水）
/// 至元嘉二十年（443）5703 岁算外。
pub fn yuanjia_exact() -> ExactPingLi {
    ExactPingLi { yue: 22207, ri: 752, sui_num: 111035, sui_den: 304, year0: 443, ji0: 5703, qishou_yushui: true }
}

/// 大明历（祖冲之，《宋书·律历志下》）：日法 3939，月法 116321；纪法 39491，岁实 14423804/39491；上元甲子
/// 至大明七年（463）51939 岁算外。
pub fn daming_zu_exact() -> ExactPingLi {
    ExactPingLi { yue: 116321, ri: 3939, sui_num: 14423804, sui_den: 39491, year0: 463, ji0: 51939, qishou_yushui: false }
}

/// 大业历（张胄玄，《隋书·律历志中》所录大业四年戊辰所定之术）：日法 1144，月法 33783；度法 42640，岁分 15573963；
/// 甲子元至大业四年（608）1427644 岁算外。597—607 年所行开皇十七年术未载，以本术代推。
pub fn daye_exact() -> ExactPingLi {
    ExactPingLi { yue: 33783, ri: 1144, sui_num: 15573963, sui_den: 42640, year0: 608, ji0: 1427644, qishou_yushui: false }
}

/// 按历名（`data/calendar/汇总/zhongyuan_calendar_periods.csv`）推某农历年各月 (朔日 JDN, 月, 闰否)。
/// 平朔诸历在此；定朔诸历见 [super::dingshuo]。只收已实现、参数经术文核验之历（见 doc/calendar/calendar-compendium.md「平朔诸历：以术文核验」与
/// doc/calendar/rili-sources.md）；其余返回 None，由第 4 层退到下一级来源。
pub fn reckon_lifa_year(name: &str, lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    let exact = match name {
        "太初历（三统历）" => Some(santong_exact()),
        "元嘉历" => Some(yuanjia_exact()),
        "大明历（祖冲之）" => Some(daming_zu_exact()),
        "大业历" | "张胄玄历（开皇十七年术不传，以大业历代推）" => Some(daye_exact()),
        _ => None,
    };
    if let Some(e) = exact {
        return Some(e.year_months(lunar_year));
    }
    let p = match name {
        "四分历" => sifen_params(),
        "景初历" => jingchu_params(),
        "开皇历" => kaihuang_params(),
        // 宋初应天、乾元、仪天
        "应天历" | "乾元历" | "仪天历" => return super::song_early::reckon_year(name, lunar_year),
        // 清初汤若望新法（照《新法算书》表复原）
        "时宪历" => return Some(super::shixian_xinfa::year_months(lunar_year)),
        // 康熙五至七年复用大统
        "大统历（康熙复用）" => return super::dingshuo::reckon_year("大统历", lunar_year),
        // 清《历象考成》（康熙甲子元法，雍正四年起）
        "时宪历（历象考成）" => return Some(super::shixian_kaocheng::year_months(lunar_year)),
        // 清《历象考成后编》（民国元年、二年历书沿用）
        "时宪历（历象考成后编）" | "中华民国历书（沿用历象考成后编）" => return Some(super::shixian::year_months(lunar_year)),
        // 元初（1280）仍行金重修大明历
        "重修大明历" if lunar_year == 1280 => return Some(super::daming_li::year_months(lunar_year)),
        // 唐至明定朔诸历（限各历推步适用的年段）
        _ => return super::dingshuo::reckon_year(name, lunar_year),
    };
    Some(year_months(&p, lunar_year))
}

/// 推步政权某岁各月 (朔日 JDN, 月序, 闰否)；返回所用历术名。政权或年份不在表内返回 None。
/// 纯按历术推步，不改个别月份：张培瑜历表与推步不合处（如 430 年二月）是另一份复原的结果，依据未核，不据以改动。
pub fn regime_year_months(regime: &str, lunar_year: i32) -> Option<(&'static str, Vec<(i64, u8, bool)>)> {
    let (_, _, _, pick) = regime_calendars().into_iter().find(|(r, y0, y1, _)| *r == regime && (*y0..=*y1).contains(&lunar_year))?;
    Some(pick(lunar_year)).filter(|(_, m)| !m.is_empty())
}

/// 某岁所用历术名与各月 (朔日 JDN, 月序, 闰否)。
pub type RegimeYearFn = fn(i32) -> (&'static str, Vec<(i64, u8, bool)>);

/// 推步政权表：(政权, 起年, 止年, 按年推步)。平朔历用 [LiParams]，金用定朔的 [super::daming_li]。
/// 与 [gregorian_to_all_regimes_lunar] 所用相同。
pub fn regime_calendars() -> Vec<(&'static str, i32, i32, RegimeYearFn)> {
    fn ping(y: i32, (name, p): (&'static str, LiParams)) -> (&'static str, Vec<(i64, u8, bool)>) {
        (name, year_months(&p, y))
    }
    fn shu(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        ping(y, ("四分历", sifen_params()))
    }
    fn wu(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        ping(y, ("乾象历", qianxiang_params()))
    }
    fn later_qin(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        ping(y, ("三纪历", sanji_params()))
    }
    fn northern_liang(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        ping(y, ("玄始历", xuanshi_params()))
    }
    fn wzs(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        match wei_zhou_sui_params_for_year(y) {
            Some(p) => ping(y, p),
            None => ("术不传", Vec::new()),
        }
    }
    fn wq(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        ping(y, wei_qi_params_for_year(y).unwrap())
    }
    // 天会十五年（1137）颁杨级大明历，术不传，以重修大明历代推；大定二十一年（1181）校验后改用赵知微重修大明历，
    // 行至金亡（1234）。见 doc/calendar/historical-calendar.md「金大明历」
    fn jin(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        (if y < 1182 { "大明历（杨级，以重修大明历代推）" } else { "重修大明历" }, super::daming_li::year_months(y))
    }
    // 民国坊本时宪书、通书：官历自民国三年改用新法、太阳平时，民间历书仍按《历象考成后编》、用时推算，至 1930 年代不改
    // （《民国元年至二十七年时宪书》汇集本：1914 年十月戊申朔即旧法九大十小，1933 年六月朔作 7 月 22 日）。
    // 年限取汇集本所收（民国三年至二十七年）。见 doc/calendar/calendar-compendium.md「日历上的历名」民国条。
    fn minjian(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        ("时宪书（坊本，沿用历象考成后编）", super::shixian::year_months(y))
    }
    // 南明、鲁监国、明郑续用大统历（1645—1683）：照《通轨》立成推步。傅以礼《残明大统历》483 月中 1645—1673 年
    // 除原书两处大小误印外全合推步；1674 年以后（明郑）12 月不合，或合清时宪、或两皆不合，未见明郑历书实物，取推步。
    // 见 doc/calendar/calendar-compendium.md「南明」与 data/calendar/canming_check.csv。
    fn nanming(y: i32) -> (&'static str, Vec<(i64, u8, bool)>) {
        ("大统历", super::dingshuo::reckon_year_any("大统历", y).unwrap_or_default())
    }
    vec![
        ("南明/南明·绍武/鲁监国/明郑", 1645, 1683, nanming),
        ("民间通书", 1914, 1938, minjian),
        ("蜀", 221, 263, shu),
        ("吴", 222, 280, wu),
        ("后秦", 384, 416, later_qin),
        ("北凉", 398, 439, northern_liang),
        ("北魏/西魏/北周/隋", 398, 590, wzs),
        ("东魏/北齐", 534, 577, wq),
        ("金", 1137, 1234, jin),
    ]
}

/// 公历 (年,月,日) 0h UT 的儒略日
fn gregorian_to_jd_f64(y: i32, m: i32, d: i32) -> f64 {
    gregorian::to_julian_day(y, m, d).as_f64()
}

/// 推步历：公历 (y,m,d) → 该历下农历 (lunar_year, month 1..12, is_leap, day 1..30)。
/// 若该日不在任何历术有效年内则返回 None。
pub fn gregorian_to_lunar_push(
    g_year: i32,
    g_month: i32,
    g_day: i32,
    params: &LiParams,
    year_min: i32,
    year_max: i32,
) -> Option<(i32, u8, bool, u8)> {
    let jd = gregorian_to_jd_f64(g_year, g_month, g_day);
    let jd_floor = (jd + 0.5).floor();
    for &n in &[g_year - 1, g_year, g_year + 1] {
        if n < year_min || n > year_max {
            continue;
        }
        let (_m1, leap_month, month_starts_jd) = year_new_moons(params, n);
        for (k, &start_jd) in month_starts_jd.iter().enumerate() {
            let end_jd = if k + 1 < month_starts_jd.len() {
                month_starts_jd[k + 1]
            } else {
                start_jd + 30.0
            };
            if jd_floor >= start_jd && jd_floor < end_jd {
                let day = (jd_floor - start_jd + 1.0) as i32;
                if day < 1 || day > 30 {
                    continue;
                }
                let (month_1based, is_leap) = if leap_month > 0 && k == leap_month as usize {
                    (leap_month, true)
                } else if leap_month > 0 && k > leap_month as usize {
                    (k as u8, false)
                } else {
                    (k as u8 + 1, false)
                };
                return Some((n, month_1based, is_leap, day as u8));
            }
        }
    }
    None
}

/// 历术预设：Sifen 东汉四分（蜀 221–263）
pub fn sifen_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 499.0 / 940.0,
        solar: 365.25,
        jd_epoch: 1662610.5,
        jdw: 1721050.5,
        dtc_round: 1e-3,
        dts_round: 1e-3,
    }
}

/// 历术预设：Qianxiang 乾象（吴 222–280）
pub fn qianxiang_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 773.0 / 1457.0,
        solar: 365.0 + 145.0 / 589.0,
        jd_epoch: 1796295.5 + 1067.0 / 1457.0,
        jdw: 1721050.5 + 210.0 / 589.0,
        dtc_round: 1e-4,
        dts_round: 1e-5,
    }
}

/// 历术预设：Jingchu 景初（北魏 398–451）
pub fn jingchu_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 2419.0 / 4559.0,
        solar: 365.0 + 455.0 / 1843.0,
        jd_epoch: 1811120.5,
        jdw: 1721050.5 + 220.0 / 1843.0,
        dtc_round: 1e-4,
        dts_round: 1e-5,
    }
}

/// 历术预设：Sanji 三纪（后秦 384–416）
pub fn sanji_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 3217.0 / 6063.0,
        solar: 365.0 + 605.0 / 2451.0,
        jd_epoch: 1861292.5 + 2826.0 / 6063.0,
        jdw: 1721050.5 + 280.0 / 2451.0,
        dtc_round: 1e-4,
        dts_round: 1e-5,
    }
}

/// 历术预设：Xuanshi 玄始（北凉、北魏 451–522）
pub fn xuanshi_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 47251.0 / 89052.0,
        solar: 365.0 + 1759.0 / 7200.0,
        jd_epoch: 1871510.5 + 4995.0 / 89052.0,
        jdw: 1721048.5 + 1189.0 / 1200.0,
        dtc_round: 1e-6,
        dts_round: 1e-6,
    }
}

/// 历术预设：Zhengguang 正光（北魏/东魏 522–539）
pub fn zhengguang_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 39769.0 / 74952.0,
        solar: 365.0 + 1477.0 / 6060.0,
        jd_epoch: 1911671.5 + 6427.0 / 9369.0,
        jdw: 1721048.5 + 569.0 / 1515.0,
        dtc_round: 1e-6,
        dts_round: 1e-6,
    }
}

/// 历术预设：Xinghe 兴和（东魏 539–550，北齐沿用至 551）
pub fn xinghe_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 110647.0 / 208530.0,
        solar: 365.0 + 4117.0 / 16860.0,
        jd_epoch: 1918286.5 + 111983.0 / 208530.0,
        jdw: 1721048.5 + 1118.0 / 4215.0,
        dtc_round: 1e-6,
        dts_round: 1e-6,
    }
}

/// 历术预设：Tianbao 天保（北齐 552–577）
pub fn tianbao_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 155272.0 / 292635.0,
        solar: 365.0 + 5787.0 / 23660.0,
        jd_epoch: 1921918.5 + 231721.0 / 292635.0,
        jdw: 1721049.5 + 193.0 / 5915.0,
        dtc_round: 1e-6,
        dts_round: 1e-6,
    }
}

/// 历术预设：Tianhe 天和（北周 578–583）
pub fn tianhe_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 153991.0 / 290160.0,
        solar: 365.0 + 5731.0 / 23460.0,
        jd_epoch: 1928976.5 + 246737.0 / 290160.0,
        jdw: 1721047.5 + 1331.0 / 3910.0,
        dtc_round: 1e-6,
        dts_round: 1e-6,
    }
}

/// 历术预设：Daxiang 大象（北周 583–584）
pub fn daxiang_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 28422.0 / 53563.0,
        solar: 365.0 + 3167.0 / 12992.0,
        jd_epoch: 1932520.5 + 36950.0 / 53563.0,
        jdw: 1721048.5 + 281.0 / 6496.0,
        dtc_round: 1e-5,
        dts_round: 1e-6,
    }
}

/// 历术预设：Kaihuang 开皇（隋 584–）
pub fn kaihuang_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 96529.0 / 181920.0,
        solar: 365.0 + 25063.0 / 102960.0,
        jd_epoch: 1934971.5 + 19841.0 / 36384.0,
        jdw: 1721048.5 + 908.0 / 6435.0,
        dtc_round: 1e-6,
        dts_round: 1e-7,
    }
}

/// 某政权下该日的农历 (region, calendar_name, lunar_year, month, is_leap, day)
#[derive(Clone, Debug)]
pub struct RegimeLunarEntry {
    pub region: String,
    pub calendar_name: String,
    pub lunar_year: i32,
    pub month: u8,
    pub is_leap: bool,
    pub day: u8,
}

/// 北朝历术分段（北魏/西魏/北周/隋 398–590）：按年选历术
fn wei_zhou_sui_params_for_year(g_year: i32) -> Option<(&'static str, LiParams)> {
    let (name, params) = if (g_year as f64) < 451.5 {
        ("景初历", jingchu_params())
    } else if (g_year as f64) < 522.5 {
        ("玄始历", xuanshi_params())
    } else if (g_year as f64) < 558.5 {
        ("正光历", zhengguang_params())
    } else if (g_year as f64) < 565.5 {
        // 559—565 年北周所行明克让等历，术不传（《隋书·律历志中》），无从复原，不给推步
        return None;
    } else if (g_year as f64) < 578.5 {
        ("天和历", tianhe_params())
    } else if (g_year as f64) < 583.5 {
        ("大象历", daxiang_params())
    } else {
        ("开皇历", kaihuang_params())
    };
    Some((name, params))
}

/// 东魏/北齐 534–577：按年选历术
fn wei_qi_params_for_year(g_year: i32) -> Option<(&'static str, LiParams)> {
    let (name, params) = if (g_year as f64) < 539.5 {
        ("正光历", zhengguang_params())
    } else if (g_year as f64) < 551.5 {
        // 天保元年（550）受禅后宋景业造天保历，实际行用自 552 年：《北齐书》天保二年（551）「閏月」在四月后、
        // 是年 22 个纪日兴和历合 21、天保历合 17，553 年起则天保历全合（doc/calendar/push-algorithms.md §3.5）
        ("兴和历", xinghe_params())
    } else {
        ("天保历", tianbao_params())
    };
    Some((name, params))
}

/// 公历 (y,m,d)（前推格里高利历，前端约定）下所有推步政权（不含中原一线）的农历月日。
pub fn gregorian_to_all_regimes_lunar(g_year: i32, g_month: i32, g_day: i32) -> Vec<RegimeLunarEntry> {
    let jdn = gregorian::gregorian_to_jdn(g_year, g_month, g_day);
    let mut out = Vec::new();
    for (region, y0, y1, _) in regime_calendars() {
        'years: for n in [g_year - 1, g_year, g_year + 1] {
            if n < y0 || n > y1 {
                continue;
            }
            let Some((cal, months)) = regime_year_months(region, n) else { continue };
            let next_start = regime_year_months(region, n + 1).map(|(_, m)| m[0].0);
            for (k, &(start, month, is_leap)) in months.iter().enumerate() {
                let end = months.get(k + 1).map(|e| e.0).or(next_start).unwrap_or(start + 30);
                if jdn >= start && jdn < end {
                    out.push(RegimeLunarEntry {
                        region: region.to_string(),
                        calendar_name: cal.to_string(),
                        lunar_year: n,
                        month,
                        is_leap,
                        day: (jdn - start + 1) as u8,
                    });
                    break 'years;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 民国坊本沿用后编用时：1914 年十月朔 11 月 18 日（旧法九大十小，官历 11 月 17 日），1933 年六月朔 7 月 22 日（官历 23 日）
    /// 南明大统：永历二年闰三月（傅以礼《残明大统历》「閏三月丙寅朔」），清时宪是年闰四月
    #[test]
    fn nanming_follows_datong() {
        let at = |y, m, d| gregorian_to_all_regimes_lunar(y, m, d).into_iter().find(|e| e.region.starts_with("南明")).map(|e| (e.lunar_year, e.month, e.is_leap, e.day));
        assert_eq!(at(1645, 2, 26).map(|e| e.3), Some(1), "弘光元年二月甲寅朔");
        assert_eq!(at(1648, 4, 23), Some((1648, 3, true, 1)), "永历二年闰三月丙寅朔");
        assert_eq!(at(1648, 5, 22), Some((1648, 4, false, 1)), "四月乙未朔（清时宪是日为闰四月朔）");
        assert!(gregorian_to_all_regimes_lunar(1640, 1, 1).iter().all(|e| !e.region.starts_with("南明")));
    }

    #[test]
    fn minjian_tongshu_follows_houbian() {
        let at = |y, m, d| gregorian_to_all_regimes_lunar(y, m, d).into_iter().find(|e| e.region == "民间通书").map(|e| (e.month, e.is_leap, e.day));
        assert_eq!(at(1914, 11, 17), Some((9, false, 30)));
        assert_eq!(at(1914, 11, 18), Some((10, false, 1)));
        assert_eq!(at(1933, 7, 22), Some((6, false, 1)));
        assert_eq!(at(1913, 6, 1), None, "1913 年以前不列");
    }

    /// 精确平朔诸历的上元：只由术文积年与甲子日定出（不借历表），钉住其儒略日数；上元皆甲子日
    #[test]
    fn exact_pingli_epochs() {
        for (li, e) in [(santong_exact(), -50593729), (yuanjia_exact(), -200089), (daming_zu_exact(), -17080189), (daye_exact(), -519493909)] {
            assert_eq!(li.epoch_jdn(), e);
            assert_eq!((e + 49).rem_euclid(60), 0, "上元甲子");
        }
    }

    /// 精确推步与浮点 LiParams 无关的一例：元嘉二十年（443）《宋书》「上元庚辰甲子纪首」，岁首雨水；
    /// 大明七年（463）冬至前后月序完整
    #[test]
    fn exact_pingli_year_shapes() {
        for (li, y) in [(yuanjia_exact(), 443), (daming_zu_exact(), 463), (santong_exact(), 0), (daye_exact(), 610)] {
            let m = li.year_months(y);
            assert!(m.len() == 12 || m.len() == 13, "{y} 年月数 {}", m.len());
            assert_eq!((m[0].1, m[0].2), (1, false));
            assert!(m.windows(2).all(|w| (29..=30).contains(&(w[1].0 - w[0].0))), "{y} 月长");
        }
    }

    /// 398—444 年南北同用景初历：北朝推步与实历朔日表（南朝宋，《三千五百年历日天象》据历还原）逐月比对。
    /// 两者都是景初历的复原，相合只说明本实现与该书的推步一致，不是史实验证。闰月位置全同；朔日只有 430 年二月不同
    /// （推步 3 月 10 日，实历表 3 月 11 日），该书此处何以与平朔推步不同，依据未核，故不据以改推步。
    #[test]
    fn jingchu_push_matches_reconstructed_table_398_444() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let csv = std::fs::read_to_string(root.join("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
        let mut table: std::collections::HashMap<(i32, u8, bool), i64> = Default::default();
        for line in csv.lines().skip(1) {
            let f: Vec<i32> = line.split(',').map(|x| x.parse().unwrap()).collect();
            if (398..=444).contains(&f[3]) {
                table.insert((f[3], f[4] as u8, f[5] == 1), gregorian::civil_to_jdn(f[0], f[1], f[2]));
            }
        }
        let (mut n, mut diffs) = (0, vec![]);
        for y in 398..=444 {
            let (cal, months) = regime_year_months("北魏/西魏/北周/隋", y).unwrap();
            assert_eq!(cal, "景初历");
            for (jdn, m, leap) in months {
                let t = table.get(&(y, m, leap)).unwrap_or_else(|| panic!("实历表无 {y} 年{}{m}月（闰位不同）", if leap { "闰" } else { "" }));
                if *t != jdn {
                    diffs.push((y, m, leap, *t - jdn));
                }
                n += 1;
            }
        }
        assert_eq!(n, table.len(), "月数（含闰）应与实历表相同");
        assert_eq!(diffs, vec![(430, 2, false, 1)], "朔日不同之处");
    }

    /// 与上游 ytliu0/ChineseCalendar 的推步结果（vendor/calendar/chinese_calendar/分政权朔日表.csv，
    /// scripts/calendar_data/extract_region_calendars_from_chinese_calendar.js 导出）逐月比对。两者是同一算法的两份实现、
    /// 参数同源，相合只说明实现一致。不同之处只有上游的人工修正与吴 222 年（上游此年前段仍用四分历，本实现全年用乾象历），
    /// 逐条钉住。三处修正照张培瑜《三千五百年历日天象》附表 2、3，张书照汪曰桢《历代长术辑要》（陈垣、方诗铭同）；司马光
    /// 《资治通鉴目录》在 247、430 两处与本实现纯推步相合，244 年照字面同汪书。各家异说、无一手史料可定，本实现取纯推步
    /// （见 doc/calendar/push-algorithms.md 3.1）。
    #[test]
    fn push_matches_upstream_export_except_known_corrections() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = root.join("vendor/calendar/chinese_calendar/分政权朔日表.csv");
        let Ok(csv) = std::fs::read_to_string(&path) else { return };
        let region = |r: &str| match r {
            "Shu" => "蜀",
            "Wu" => "吴",
            "LaterQin" => "后秦",
            "NorthernLiang" => "北凉",
            "WeiZhouSui" => "北魏/西魏/北周/隋",
            "WeiQi" => "东魏/北齐",
            _ => panic!("未知政权 {r}"),
        };
        let mut up: std::collections::HashMap<(&str, i32), Vec<(i64, u8, bool)>> = Default::default();
        for line in csv.lines().skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            let n = |i: usize| f[i].parse::<i32>().unwrap();
            up.entry((region(f[0]), n(4))).or_default().push((gregorian::civil_to_jdn(n(1), n(2), n(3)), n(5) as u8, f[6] == "1"));
        }
        let (mut leap_diff, mut day_diff, mut n) = (vec![], vec![], 0);
        for ((reg, y), mut theirs) in up {
            let Some((_, ours)) = regime_year_months(reg, y) else { continue };
            theirs.sort();
            let key = |v: &[(i64, u8, bool)]| v.iter().map(|e| (e.1, e.2)).collect::<Vec<_>>();
            if key(&ours) != key(&theirs) {
                leap_diff.push((reg, y));
                continue;
            }
            for (a, b) in ours.iter().zip(&theirs) {
                n += 1;
                if a.0 != b.0 {
                    day_diff.push((reg, y, a.1, b.0 - a.0));
                }
            }
        }
        leap_diff.sort();
        day_diff.sort();
        // 559—565 年北周明克让历术不传，不推步（原为廖育栋拟设），这几年不比
        assert!(n > 4800, "比对月数 {n}");
        // 东魏/北齐 551：上游自 551 年用天保历（闰二），本实现据《北齐书》纪日与閏月位置沿用兴和历至 551 年（闰四）
        assert_eq!(leap_diff, vec![("东魏/北齐", 551), ("吴", 222)], "闰位不同之岁");
        assert_eq!(
            day_diff,
            vec![("北凉", 430, 2, 1), ("北魏/西魏/北周/隋", 430, 2, 1), ("吴", 244, 12, 1), ("吴", 247, 9, -1)],
            "朔日不同之月（上游人工修正）"
        );
    }

    #[test]
    fn test_sifen_epoch() {
        let p = sifen_params();
        let (m1, _, starts) = year_new_moons(&p, 221);
        assert!(starts.len() >= 12);
        // 原断言把 m1（221 年正月朔）跟 jd_epoch（历元本身，儒略历公元前 161 年）比较，容差只给 10000 天
        // （~27 年）——但历元到公元 221 年实际相差约 139208 天（~381 年），断言从写下来那天起就恒假，是笔误
        // （大概是想跟"预期正月朔"比，误写成跟"历元"比）。换成有意义的检查：把算出的具体值钉住（容差 1 天，
        // 只防浮点误差，不是"大概齐"），并转回历法日期核实落在儒略历 221 年 1-2 月（正月朔的合理范围）。
        assert!((m1 - 1_801_818.933).abs() < 1.0, "m1={}", m1);
        let (y, mo, _d) = super::gregorian::Gregorian::from_julian_day(crate::math::real::real(m1));
        assert_eq!(y, 221);
        assert!(mo == 1 || mo == 2, "四分历正月朔应落在儒略历 1-2 月，实际 month={}", mo);
    }
}
