//! 推步历：平朔 + 无中气置闰，用于各政权历法（蜀四分、吴乾象等）。
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

/// 推步政权某岁各月 (朔日 JDN, 月序, 闰否)；返回所用历术名。政权或年份不在表内返回 None。
/// 纯按历术推步，不改个别月份：实历朔日表与推步不合处（如 430 年二月）是另一份复原的结果，依据未核，不据以改动。
pub fn regime_year_months(regime: &str, lunar_year: i32) -> Option<(&'static str, Vec<(i64, u8, bool)>)> {
    let (_, _, _, pick) = regime_calendars().into_iter().find(|(r, y0, y1, _)| *r == regime && (*y0..=*y1).contains(&lunar_year))?;
    let (cal, params) = pick(lunar_year);
    Some((cal, year_months(&params, lunar_year)))
}

/// 推步政权表：(政权, 起年, 止年, 按年选历术)。与 [gregorian_to_all_regimes_lunar] 所用相同。
pub fn regime_calendars() -> Vec<(&'static str, i32, i32, fn(i32) -> (&'static str, LiParams))> {
    fn shu(_: i32) -> (&'static str, LiParams) {
        ("四分历", sifen_params())
    }
    fn wu(_: i32) -> (&'static str, LiParams) {
        ("乾象历", qianxiang_params())
    }
    fn later_qin(_: i32) -> (&'static str, LiParams) {
        ("三纪历", sanji_params())
    }
    fn northern_liang(_: i32) -> (&'static str, LiParams) {
        ("玄始历", xuanshi_params())
    }
    fn wzs(y: i32) -> (&'static str, LiParams) {
        wei_zhou_sui_params_for_year(y).unwrap()
    }
    fn wq(y: i32) -> (&'static str, LiParams) {
        wei_qi_params_for_year(y).unwrap()
    }
    vec![
        ("蜀", 221, 263, shu),
        ("吴", 222, 280, wu),
        ("后秦", 384, 416, later_qin),
        ("北凉", 398, 439, northern_liang),
        ("北魏/西魏/北周/隋", 398, 590, wzs),
        ("东魏/北齐", 534, 577, wq),
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

/// 历术预设：明克让（失传，借正光+节气小余+0.4）（北周 565–578）
pub fn mingkerang_params() -> LiParams {
    LiParams {
        lunar: 29.0 + 39769.0 / 74952.0,
        solar: 365.0 + 1477.0 / 6060.0,
        jd_epoch: 1911671.5 + 6427.0 / 9369.0,
        jdw: 1721048.5 + 569.0 / 1515.0 + 0.4,
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
        ("明克让历", mingkerang_params())
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

/// 公历 (y,m,d)（前推格里高利历，前端约定）下所有推步政权（不含主表）的农历月日。
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
        assert!(n > 4900, "比对月数 {n}");
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
