//! 历法域第 2 层（聚合/实算层）：**实物历日**——当时颁行的历书原件所载的整年历日（资料可靠性第 1 级）。
//!
//! 与查今人历表（[crate::calendar::historical]）、按术推步（[crate::calendar::push_calendar]）同层，何时取用由第 4 层
//! [crate::calendar::lifa::resolve_year] 仲裁：实物历日在中原一线优先于一切推算。
//!
//! 目前收（只收月表完整、正月起十二月止之年；同年多本须逐月相同，测试钉住）：
//! - 南宋《大宋寶祐四年丙辰歲會天萬年具注曆》（1256，宛委别藏影印清钞本），转录 `data/calendar/会天历_1256.csv`
//!   （各月一日干支与月大小，见 `doc/calendar/historical-calendar.md`「宝祐四年会天历」）；
//! - 明钦天监刊本《大统历》（台北「国家图书馆」藏，9 年），`data/calendar/datong_shiwu_months.csv`；
//! - 《国家图书馆藏明代大统历日汇编》（北京图书馆出版社 2007，6 册），`data/calendar/datong_huibian_months_v12.csv`、`_v34.csv`、`_v56.csv`；
//! - 清《时宪书》（国家图书馆等藏），`data/calendar/shixianshu_months.csv`、`shixianshu_netdisk_months.csv`。
//! 转录与逐月比较见 `doc/calendar/calendar-compendium.md`「《国家图书馆藏明代大统历日汇编》」「时宪书」各节。
//! 南明（永历）大统历、民国时期民间时宪书不属中原一线官历，不收。敦煌具注历日是敦煌政权之历，见 [crate::calendar::dunhuang]。

use super::gan_zhi::GAN_ZHI_60;
use super::gregorian::civil_to_jdn;
use super::historical::MonthStart;

/// 一部整年的实物历日：名称、农历年、各月。
#[derive(Clone, Debug)]
pub struct ArtifactYear {
    pub name: &'static str,
    pub lunar_year: i32,
    pub months: Vec<MonthStart>,
}

const HUITIAN_1256_NAME: &str = "《宝祐四年会天历》";

/// JDN 的日干支序（甲子 = 0）。
fn ganzhi_index(jdn: i64) -> usize {
    (jdn + 49).rem_euclid(60) as usize
}

/// 由各月「一日干支、大小」排出各月朔日 JDN。正月一日取 `[window_from, window_to]`（民用历日期，窗口须短于 60 日）
/// 内干支相符之日；其后逐月按大小累加，并核对每月一日干支——原件自相矛盾时返回 Err。
fn months_from_ganzhi(
    rows: &[(u8, bool, &str, bool)],
    window_from: (i32, i32, i32),
    window_to: (i32, i32, i32),
) -> Result<Vec<MonthStart>, String> {
    let (a, b) = (civil_to_jdn(window_from.0, window_from.1, window_from.2), civil_to_jdn(window_to.0, window_to.1, window_to.2));
    debug_assert!(b - a < 60);
    let first = rows.first().ok_or("无月")?;
    let mut jdn = (a..=b).find(|&j| GAN_ZHI_60[ganzhi_index(j)] == first.2).ok_or("窗口内无此干支")?;
    let mut out = Vec::with_capacity(rows.len());
    for (k, &(month, leap, gz, big)) in rows.iter().enumerate() {
        if GAN_ZHI_60[ganzhi_index(jdn)] != gz {
            return Err(format!("第 {} 月一日应为 {}，按前月大小推得 {}", k + 1, gz, GAN_ZHI_60[ganzhi_index(jdn)]));
        }
        out.push((jdn, month, leap));
        jdn += if big { 30 } else { 29 };
    }
    Ok(out)
}

/// 解析会天历转录表（类=月 的行：名, 月, 日, 干支, 加时, 大小, 书页）。
fn huitian_rows(csv: &str) -> Vec<(u8, bool, &str, bool)> {
    csv.lines()
        .filter(|l| l.starts_with("月,"))
        .filter_map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            let name = *f.get(1)?;
            let month: u8 = f.get(2)?.trim().parse().ok()?;
            Some((month, name.starts_with('閏') || name.starts_with('闰'), *f.get(4)?, *f.get(6)? == "大"))
        })
        .collect()
}

/// 解析「年,月,闰,大小,朔干支」式月表（列按表头名取；`year_col` 为农历年所在列）。只返回月表完整之年：正月起、十二月止、
/// 逐月衔接、每月有朔干支与大小。
fn month_table_years<'a>(csv: &'a str, year_col: &str) -> Vec<(i32, Vec<(u8, bool, &'a str, bool)>)> {
    let mut lines = csv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    let Some(head) = lines.next() else { return vec![] };
    let cols: Vec<&str> = head.split(',').collect();
    let idx = |n: &str| cols.iter().position(|c| *c == n).unwrap_or_else(|| panic!("月表缺列 {n}"));
    let (iy, im, il, ib, ig) = (idx(year_col), idx("月"), idx("闰"), idx("大小"), idx("朔干支"));
    let mut years: Vec<(i32, Vec<(u8, bool, &str, bool)>, bool)> = Vec::new();
    for l in lines {
        let f: Vec<&str> = l.split(',').collect();
        let Some(y) = f.get(iy).and_then(|v| v.trim().parse::<i32>().ok()) else { continue };
        let m = f.get(im).and_then(|v| v.trim().parse::<u8>().ok());
        let leap = matches!(f.get(il).map(|v| v.trim()), Some("1" | "闰" | "是"));
        let (big, gz) = (f.get(ib).map(|v| v.trim()), f.get(ig).map(|v| v.trim()).unwrap_or(""));
        if years.last().map(|e| e.0) != Some(y) {
            years.push((y, Vec::new(), true));
        }
        let e = years.last_mut().unwrap();
        match (m, big) {
            (Some(m), Some(b @ ("大" | "小"))) if !gz.is_empty() => {
                // 同年多本（如 1532 两本逐月交错）：同月再出须与前条相同，去重
                let row = (m, leap, gz, b == "大");
                match e.1.iter().find(|r| (r.0, r.1) == (m, leap)) {
                    Some(prev) => assert_eq!(*prev, row, "{y} 年 {m} 月各本不同"),
                    None => e.1.push(row),
                }
            }
            _ => e.2 = false,
        }
    }
    years
        .into_iter()
        .filter(|(_, ms, ok)| {
            // 月序逐月衔接（下一月为次月，或同月之闰），残本（中缺数月）不收
            let contiguous = ms.windows(2).all(|w| (w[1].0 == w[0].0 + 1 && !w[1].1) || (w[1].0 == w[0].0 && w[1].1 && !w[0].1));
            *ok && contiguous && ms.first().map(|r| (r.0, r.1)) == Some((1, false)) && ms.last().map(|r| r.0) == Some(12)
        })
        .map(|(y, ms, _)| (y, ms))
        .collect()
}

/// 中原一线的实物历日。
pub fn zhongyuan_artifacts() -> &'static [ArtifactYear] {
    use std::sync::OnceLock;
    static ALL: OnceLock<Vec<ArtifactYear>> = OnceLock::new();
    ALL.get_or_init(|| {
        let rows = huitian_rows(include_str!("../../../../data/calendar/会天历_1256.csv"));
        // 正月一日（癸巳）必在 1256 年 1 月 20 日至 2 月 21 日之间（正月为含雨水之月）
        let months = months_from_ganzhi(&rows, (1256, 1, 20), (1256, 2, 21)).expect("会天历转录表自相矛盾");
        let mut all = vec![ArtifactYear { name: HUITIAN_1256_NAME, lunar_year: 1256, months }];
        let tables: [(&str, &str, &str); 6] = [
            ("明钦天监刊本《大统历》", include_str!("../../../../data/calendar/datong_shiwu_months.csv"), "年"),
            ("《国家图书馆藏明代大统历日汇编》", include_str!("../../../../data/calendar/datong_huibian_months_v12.csv"), "年"),
            ("《国家图书馆藏明代大统历日汇编》", include_str!("../../../../data/calendar/datong_huibian_months_v34.csv"), "年"),
            ("《国家图书馆藏明代大统历日汇编》", include_str!("../../../../data/calendar/datong_huibian_months_v56.csv"), "年"),
            ("清《时宪书》", include_str!("../../../../data/calendar/shixianshu_months.csv"), "年"),
            ("清《时宪书》", include_str!("../../../../data/calendar/shixianshu_netdisk_months.csv"), "农历年"),
        ];
        for (name, csv, ycol) in tables {
            for (y, rows) in month_table_years(csv, ycol) {
                // 正月一日在民用历（1582 年前儒略历）1 月 5 日至 2 月 28 日之间：明清正月含雨水，格里历 1 月 21 日—2 月 20 日，儒略历早约十日
                let months = months_from_ganzhi(&rows, (y, 1, 5), (y, 2, 28))
                    .unwrap_or_else(|e| panic!("{name} {y} 年月表自相矛盾：{e}"));
                match all.iter().find(|a| a.lunar_year == y) {
                    Some(a) => assert_eq!(a.months, months, "{y} 年 {} 与 {name} 不同", a.name),
                    None => all.push(ArtifactYear { name, lunar_year: y, months }),
                }
            }
        }
        all.sort_by_key(|a| a.lunar_year);
        all
    })
}

/// 某农历年的实物历日（中原一线）。
pub fn zhongyuan_artifact_year(lunar_year: i32) -> Option<&'static ArtifactYear> {
    zhongyuan_artifacts().iter().find(|a| a.lunar_year == lunar_year)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gregorian::civil_from_jdn;

    /// 会天历 12 个月一日干支与大小自洽；正月一日为 1256 年 1 月 29 日（儒略历）
    #[test]
    fn huitian_1256_self_consistent() {
        let a = zhongyuan_artifact_year(1256).unwrap();
        assert_eq!(a.months.len(), 12);
        assert_eq!(civil_from_jdn(a.months[0].0), (1256, 1, 29));
        assert_eq!(civil_from_jdn(a.months[11].0), (1256, 12, 19));
        assert!(a.months.iter().enumerate().all(|(k, &(_, m, leap))| m as usize == k + 1 && !leap));
    }

    /// 明大统历、清时宪书实物：月表完整之年 115 个（同年多本逐月相同，否则 zhongyuan_artifacts 内 assert 失败）；
    /// 万历三十八年（1610）二月丁未朔（月表「二月大 丁未」，证立成推步）。
    #[test]
    fn ming_qing_artifacts_loaded() {
        let all = zhongyuan_artifacts();
        assert_eq!(all.len(), 1 + 115);
        let a = zhongyuan_artifact_year(1610).unwrap();
        let feb = a.months.iter().find(|m| m.1 == 2 && !m.2).unwrap();
        assert_eq!(GAN_ZHI_60[ganzhi_index(feb.0)], "丁未");
        for y in [1646, 1658, 1676, 1679, 1714] {
            assert_eq!(zhongyuan_artifact_year(y).unwrap().name, "清《时宪书》");
        }
    }
}
