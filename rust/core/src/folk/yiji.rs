//! 宜忌：《欽定協紀辨方書》月表（卷二十至三十一）。
//!
//! 月表按「月 × 日干支」逐日列吉神、凶神、宜、忌，与年份无关。本模块读
//! `data/folk/xieji_yuebiao.csv`：该表由 `scripts/folk/xieji_yuebiao.py` 按卷十（各神宜忌、相併相抵）、
//! 卷十一（以事为纲的宜神忌神）、卷十末宜忌等第表与鋪註條例推出，并与月表影印本逐页核对
//!（核对记录 `data/folk/xieji_yuebiao_checks.csv`）。
//! 事目用月表所用的通书六十事（上冊受封、會親友、出行、移徙、裁衣、修造動土等）。
//! 与年份、节气、朔望有关者（無祿、上朔、四離四絕、氣往亡、二至二分、土王用事、伏社、朔望、月忌、
//! 長星短星等，卷十所谓「月表萬年書所不能載，臨時選用」）不在月表之内，另行处理。

use once_cell::sync::Lazy;

const CSV: &str = include_str!("../../../../data/folk/xieji_yuebiao.csv");
const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";

/// 月表一日
#[derive(Clone, Debug)]
pub struct YueBiaoDay {
    /// 建除（建、除、滿……）
    pub jian_chu: String,
    pub ji_shen: Vec<String>,
    pub xiong_shen: Vec<String>,
    /// 宜；`None` 表示月表作「諸事不宜」
    pub yi: Option<Vec<String>>,
    pub ji: Vec<String>,
}

fn words(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_string).collect()
}

/// 720 日：下标 = 月序（寅月 = 0）× 60 + 干支序
static TABLE: Lazy<Vec<YueBiaoDay>> = Lazy::new(|| {
    let mut v: Vec<Option<YueBiaoDay>> = vec![None; 720];
    for line in CSV.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let f: Vec<&str> = line.split(',').collect();
        assert!(f.len() >= 7, "{line}");
        let m: usize = f[0].parse().expect("月");
        let g = GAN.chars().position(|c| f[1].starts_with(c)).expect("干");
        let z = ZHI.chars().position(|c| f[1].ends_with(c)).expect("支");
        let k = (0..60).find(|k| k % 10 == g && k % 12 == z).expect("干支");
        v[(m - 1) * 60 + k] = Some(YueBiaoDay {
            jian_chu: f[2].to_string(),
            ji_shen: words(f[3]),
            xiong_shen: words(f[4]),
            yi: if f[5] == "諸事不宜" { None } else { Some(words(f[5])) },
            ji: words(f[6]),
        });
    }
    v.into_iter().map(|d| d.expect("月表缺日")).collect()
});

/// 某月（`month_ord`：寅月 = 0，月以节为界）某日（干支序 `day_gz`，甲子 = 0）的月表
pub fn yuebiao_day(month_ord: u8, day_gz: u8) -> &'static YueBiaoDay {
    &TABLE[(month_ord % 12) as usize * 60 + (day_gz % 60) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gz(name: &str) -> u8 {
        let g = GAN.chars().position(|c| name.starts_with(c)).unwrap();
        let z = ZHI.chars().position(|c| name.ends_with(c)).unwrap();
        (0..60u8).find(|k| *k as usize % 10 == g && *k as usize % 12 == z).unwrap()
    }

    /// 以影印本核过的几日（浙大藏四库本月表）
    #[test]
    fn checked_pages() {
        let d = yuebiao_day(0, gz("甲子"));
        assert_eq!(d.jian_chu, "開");
        assert_eq!(d.yi.as_deref().unwrap(), ["祭祀", "入學", "沐浴"]);
        assert_eq!(d.ji.first().map(String::as_str), Some("冠帶"));
        assert!(d.xiong_shen.iter().any(|s| s == "災煞"));
        assert!(yuebiao_day(0, gz("乙丑")).yi.is_none(), "正月乙丑諸事不宜");
        let d = yuebiao_day(9, gz("壬子"));
        assert_eq!(d.yi.as_deref().unwrap(), ["沐浴", "掃舍宇"]);
        let d = yuebiao_day(1, gz("壬辰"));
        assert!(d.ji.iter().any(|s| s == "開渠"), "壬日止忌開渠");
    }

    #[test]
    fn table_complete() {
        assert_eq!(TABLE.len(), 720);
        let none = TABLE.iter().filter(|d| d.yi.is_none()).count();
        assert!(none > 0 && none < 120, "{none}");
    }
}
