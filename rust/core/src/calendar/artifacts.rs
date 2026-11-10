//! 历法域第 2 层（聚合/实算层）：**实物历日**——当时颁行的历书原件所载的整年历日（资料可靠性第 1 级）。
//!
//! 与查今人历表（[crate::calendar::historical]）、按术推步（[crate::calendar::push_calendar]）同层，何时取用由第 4 层
//! [crate::calendar::lifa::resolve_year] 仲裁：实物历日在中原一线优先于一切推算。
//!
//! 目前收：南宋《大宋寶祐四年丙辰歲會天萬年具注曆》（1256，宛委别藏影印清钞本），转录 `data/calendar/会天历_1256.csv`
//! （各月一日干支与月大小，见 `doc/calendar/historical-calendar.md`「宝祐四年会天历」）。敦煌具注历日是敦煌政权之历，
//! 不属中原一线，见 [crate::calendar::dunhuang]。

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

/// 中原一线的实物历日（目前只有 1256 年）。
pub fn zhongyuan_artifacts() -> &'static [ArtifactYear] {
    use std::sync::OnceLock;
    static ALL: OnceLock<Vec<ArtifactYear>> = OnceLock::new();
    ALL.get_or_init(|| {
        let rows = huitian_rows(include_str!("../../../../data/calendar/会天历_1256.csv"));
        // 正月一日（癸巳）必在 1256 年 1 月 20 日至 2 月 21 日之间（正月为含雨水之月）
        let months = months_from_ganzhi(&rows, (1256, 1, 20), (1256, 2, 21)).expect("会天历转录表自相矛盾");
        vec![ArtifactYear { name: HUITIAN_1256_NAME, lunar_year: 1256, months }]
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
}
