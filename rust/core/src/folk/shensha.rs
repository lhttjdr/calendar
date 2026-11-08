//! 日神煞（月家吉凶神）。
//!
//! 起例据《欽定協紀辨方書》卷五、卷六，月份相关者由 `scripts/folk/gen_shensha_month.py` 展开成
//! `data/folk/shensha_month.csv`（每神十二个月的取值与原文出处）；与月份无关的列为常量（见下）。
//! 月以节为界，正月 = 寅月；取值「依干」者比日干、「依支」者比日支、「依干支」者比日干支。
//! 本模块只列出当日所值之神，不推宜忌（宜忌见协纪卷十、卷十一，另行实现）。

use once_cell::sync::Lazy;

const CSV: &str = include_str!("../../../../data/folk/shensha_month.csv");
const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";

/// 依什么比对
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Yi {
    Gan,
    Zhi,
    GanZhi,
}

#[derive(Clone, Debug)]
pub struct MonthShen {
    pub name: String,
    pub ji: bool,
    pub yi: Yi,
    /// 正月（寅）至十二月（丑）各月取值（可多个；空表示该月无）
    pub vals: [Vec<String>; 12],
    pub source: String,
}

pub static MONTH_SHEN: Lazy<Vec<MonthShen>> = Lazy::new(|| {
    CSV.lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.splitn(5, ',').collect();
            let yi = match f[2] {
                "干" => Yi::Gan,
                "支" => Yi::Zhi,
                _ => Yi::GanZhi,
            };
            let ms: Vec<&str> = f[3].split('|').collect();
            assert_eq!(ms.len(), 12, "{}", f[0]);
            let vals = core::array::from_fn(|k| {
                if ms[k] == "-" { Vec::new() } else { ms[k].split('+').map(str::to_string).collect() }
            });
            MonthShen { name: f[0].to_string(), ji: f[1] == "吉", yi, vals, source: f[4].trim_matches('"').to_string() }
        })
        .collect()
});

/// 与月份无关、按日干支固定的神：(名, 吉, 干支列表, 出处)
pub const FIXED_SHEN: [(&str, bool, &[&str], &str); 8] = [
    ("天恩", true, &["甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己卯", "庚辰", "辛巳", "壬午", "癸未", "己酉", "庚戌", "辛亥", "壬子", "癸丑"],
        "卷五歷例：常以甲子至戊辰、己卯至癸未、己酉至癸丑凡一十五日"),
    ("五合", true, &["甲寅", "乙卯", "丙寅", "丁卯", "戊寅", "己卯", "庚寅", "辛卯", "壬寅", "癸卯"], "卷五歷例：五合者寅卯日也"),
    ("重日", false, &["乙巳", "丁巳", "己巳", "辛巳", "癸巳", "乙亥", "丁亥", "己亥", "辛亥", "癸亥"], "卷五歷例：重日者巳亥二日也"),
    ("五離", false, &["甲申", "丙申", "戊申", "庚申", "壬申", "乙酉", "丁酉", "己酉", "辛酉", "癸酉"], "卷五歷例：五離者申酉日是也"),
    ("觸水龍", false, &["丙子", "癸丑", "癸未"], "卷五歷例：觸水龍者丙子癸丑癸未也"),
    // 协纪按语据《神煞起例》改定：甲丙庚壬配午申（避戊），乙丁己辛癸配酉，共十三日；
    // 「通書誤以庚寅壬寅為鳴吠，甲午為鳴吠對」。卷九立成同此。
    ("鳴吠", true, &["甲午", "丙午", "庚午", "壬午", "甲申", "丙申", "庚申", "壬申", "乙酉", "丁酉", "己酉", "辛酉", "癸酉"],
        "卷五按：以甲丙庚壬四干配午申得八日避戊土不用，以乙丁己辛癸五干配酉得五日，共十三日為鳴吠日（通書十四日含庚寅壬寅，協紀謂誤）；卷九立成同"),
    ("鳴吠對", true, &["丙子", "庚子", "壬子", "甲寅", "丙寅", "庚寅", "壬寅", "乙卯", "丁卯", "辛卯", "癸卯"],
        "卷五按：丙庚壬三干配子，甲丙庚壬四干配寅，乙丁辛癸四干配卯，共十一日為鳴吠對日（通書十日含甲午，協紀謂誤）；卷九立成同"),
    ("無祿", false, &["甲辰", "乙巳", "庚辰", "辛巳", "丙申", "戊戌", "丁亥", "己丑", "壬申", "癸亥"],
        "卷五：甲辰乙巳庚辰辛巳丙申戊戌丁亥己丑壬申癸亥，此十日名曰無祿日，又曰十惡大敗日（協紀按：惟隨各年及本命避之）"),
];

fn name60(gz: u8) -> String {
    let g = GAN.chars().nth((gz % 10) as usize).unwrap();
    let z = ZHI.chars().nth((gz % 12) as usize).unwrap();
    format!("{g}{z}")
}

/// 某月（`month_ord`：寅月 = 0）某日（干支序 `day_gz`）所值之神：(名, 吉)
pub fn shen_for_day(month_ord: u8, day_gz: u8) -> Vec<(String, bool)> {
    let gz = name60(day_gz);
    let g = gz.chars().next().unwrap().to_string();
    let z = gz.chars().nth(1).unwrap().to_string();
    let mut out = Vec::new();
    for s in MONTH_SHEN.iter() {
        let key = match s.yi {
            Yi::Gan => &g,
            Yi::Zhi => &z,
            Yi::GanZhi => &gz,
        };
        if s.vals[(month_ord % 12) as usize].iter().any(|v| v == key) {
            out.push((s.name.clone(), s.ji));
        }
    }
    for (name, ji, list, _) in FIXED_SHEN.iter() {
        if list.contains(&gz.as_str()) {
            out.push((name.to_string(), *ji));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(name: &str) -> &'static MonthShen {
        MONTH_SHEN.iter().find(|s| s.name == name).unwrap_or_else(|| panic!("{name}"))
    }
    fn gi(c: &str) -> i32 {
        GAN.chars().position(|x| x.to_string() == c).unwrap() as i32
    }
    fn zi(c: &str) -> i32 {
        ZHI.chars().position(|x| x.to_string() == c).unwrap() as i32
    }
    /// 月建支序：正月寅
    fn jian(k: usize) -> i32 {
        (k as i32 + 2) % 12
    }

    /// 内在关系检验，查抄录错误：天德合、月德合为天德、月德的五合（相差五干），月空与月德相冲（相差六干）
    #[test]
    fn de_he_kong_relations() {
        for k in 0..12 {
            for (a, b) in [("天德", "天德合"), ("月德", "月德合")] {
                let (x, y) = (&get(a).vals[k], &get(b).vals[k]);
                assert_eq!(x.len(), y.len(), "{a} {k}");
                if let (Some(x), Some(y)) = (x.first(), y.first()) {
                    assert_eq!((gi(x) + 5) % 10, gi(y), "{a}{k}");
                }
            }
            let (d, kong) = (&get("月德").vals[k][0], &get("月空").vals[k][0]);
            assert_eq!((gi(d) - gi(kong)).abs(), 6, "月空{k}");
        }
    }

    /// 三合与月建成三合局，六合与月建六合（支序和 ≡ 1 mod 12），月害与月建六害（支序和 ≡ 7 mod 12），
    /// 驛馬为三合局首之衝
    #[test]
    fn he_hai_relations() {
        for k in 0..12 {
            let j = jian(k);
            let sh: Vec<i32> = get("三合").vals[k].iter().map(|x| zi(x)).collect();
            for z in &sh {
                assert_eq!((z - j).rem_euclid(4), 0, "三合{k}");
            }
            assert_eq!((zi(&get("六合").vals[k][0]) + j) % 12, 1, "六合{k}");
            assert_eq!((zi(&get("月害").vals[k][0]) + j) % 12, 7, "月害{k}");
            // 三合局首（生地）：寅午戌→寅，局首之衝即驛馬
            let head = [2, 11, 8, 5].iter().copied().find(|h| (j - h).rem_euclid(4) == 0).unwrap();
            assert_eq!(zi(&get("驛馬").vals[k][0]), (head + 6) % 12, "驛馬{k}");
        }
    }

    #[test]
    fn day_lookup() {
        // 正月（寅）丁卯日：天德（丁）、五合（卯日）；卯日亦为官日
        let v = shen_for_day(0, 3);
        let names: Vec<&str> = v.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"天德") && names.contains(&"五合") && names.contains(&"官日"), "{names:?}");
        assert_eq!(MONTH_SHEN.len(), 63);
    }
}
