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
    /// 卷十宜忌等第表之用（如「上次→相抵」：所值主凶神最重之等第→從宜／相抵／從忌／皆忌）；无主凶神者空
    pub deng_di: String,
}

const SHI_CSV: &str = include_str!("../../../../data/folk/yiji_shi.csv");

/// 卷十一（以事为纲）：某事之宜神、忌神（`data/folk/yiji_shi.csv`）。月表即由此与卷十相併相抵、等第推出
pub fn shi_shen(shi: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    let (mut yi, mut ji) = (Vec::new(), Vec::new());
    for line in SHI_CSV.lines().skip(1) {
        let f: Vec<&'static str> = line.splitn(4, ',').collect();
        if f.len() < 3 || f[0] != shi {
            continue;
        }
        let v: Vec<&'static str> = f[2].split('|').filter(|x| !x.is_empty()).collect();
        if f[1] == "宜" { yi.extend(v) } else { ji.extend(v) }
    }
    (yi, ji)
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
            deng_di: f.get(7).map(|x| x.trim().to_string()).unwrap_or_default(),
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

    /// 月表 720 日之嫁娶：注宜者必值卷十一嫁娶宜神，注忌者必值嫁娶忌神（或諸事不宜），即依据可由宜神、忌神说明
    #[test]
    fn jiaqu_yuebiao_explained_by_shi_shen() {
        let (yi_s, ji_s) = shi_shen("嫁娶");
        assert!(yi_s.contains(&"不將") && ji_s.contains(&"月厭") && ji_s.contains(&"亥日"));
        let (mut n_yi, mut n_ji) = (0, 0);
        for m in 0..12u8 {
            for k in 0..60u8 {
                let d = yuebiao_day(m, k);
                let mut names: Vec<String> = d.ji_shen.iter().chain(d.xiong_shen.iter()).cloned().collect();
                names.push(format!("{}日", d.jian_chu));
                names.push(format!("{}日", ZHI.chars().nth(k as usize % 12).unwrap()));
                let has = |l: &[&str]| l.iter().any(|n| names.iter().any(|x| x == n));
                if d.yi.as_ref().is_some_and(|v| v.iter().any(|x| x == "嫁娶")) {
                    n_yi += 1;
                    assert!(has(&yi_s), "{m} {k} 注宜嫁娶而无宜神：{:?}", names);
                }
                if d.yi.is_some() && d.ji.iter().any(|x| x == "嫁娶") {
                    n_ji += 1;
                    assert!(has(&ji_s), "{m} {k} 注忌嫁娶而无忌神：{:?}", names);
                }
            }
        }
        assert!(n_yi > 50 && n_ji > 200, "{n_yi} {n_ji}");
    }

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
