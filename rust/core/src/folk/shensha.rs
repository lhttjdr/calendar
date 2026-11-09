//! 日神煞（月家吉凶神）。
//!
//! 起例据《欽定協紀辨方書》卷五、卷六，月份相关者由 `scripts/folk/gen_shensha_month.py` 展开成
//! `data/folk/shensha_month.csv`（每神十二个月的取值与原文出处）；与月份无关的列为常量（见下）。
//! 月以节为界，正月 = 寅月；取值「依干」者比日干、「依支」者比日支、「依干支」者比日干支。
//! 本模块只列出当日所值之神，不推宜忌（宜忌见协纪卷十、卷十一，另行实现）。
//!
//! **起例选项**（[`ShenshaOptions`]，缺省全依协纪）：元末本《類編曆法通書大全》（题宋魯珍，北大藏，archive.org
//! `02094403.cn`—`02094417.cn`，下称类编）与协纪起例不同、而类编自身前后一致的几神，可逐神改依类编：
//! 天德天德合四仲月、天火顺行、鳴吠（十四日、安葬十二日）、鳴吠對（十一日、十日两表）、復日四说、荒蕪（协纪月家不列；
//! 按季三支、类编表序一月一支、类编新增起例一月一支）。年神蠶命见 [`super::nianshen::NianShenOptions`]。
//! 逐神出处与逐日检验见 `doc/folk/tongshu.md`「类编选项」一节。选项只改所列之神，不改协纪月表宜忌（[`super::yiji`]）。

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

/// 神煞起例所依之书
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum QiLiYi {
    /// 《欽定協紀辨方書》（缺省）
    #[default]
    XieJi,
    /// 《類編曆法通書大全》
    LeiBian,
}

/// 鳴吠日之说
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MingFeiRule {
    /// 协纪卷五按语据《神煞起例》改定之十三日（缺省）
    #[default]
    XieJi,
    /// 类编卷二十八「鳴吠吉日」十四日：庚午壬申癸酉壬午甲申乙酉庚寅丙申丁酉壬寅丙午己酉庚申辛酉
    ///（02094417.cn PDF 第 25 页）；协纪卷五引一行同此十四日，按语谓「通書誤以庚寅壬寅為鳴吠」
    LeiBian14,
    /// 类编卷二十八「論安葬鳴吠日」十二日：壬申癸酉壬午甲申乙酉丙申丁酉壬寅丙午己酉庚申辛酉（同册第 37 页）
    LeiBianAnZang12,
}

/// 鳴吠對日之说
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MingFeiDuiRule {
    /// 协纪卷五按语之十一日（缺省）
    #[default]
    XieJi,
    /// 类编卷二十八「鳴吠對日」十一日：甲子丙寅丁卯丙子戊寅庚寅辛卯癸卯壬子甲寅乙卯（02094417.cn PDF 第 26 页）
    LeiBian11,
    /// 类编卷二十八所引另表十日：丙寅丁卯丙子辛卯甲午庚子癸卯壬子甲寅乙卯（同册第 47 页，表题首字漫漶，所引曆名未核）；
    /// 协纪卷五引一行同此十日，按语谓「甲午為鳴吠對」误
    LeiBian10,
}

/// 復日之说（孟仲月诸说皆同：正甲二乙四丙五丁七庚八辛十壬十一癸；异在季月）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FuRiRule {
    /// 协纪卷五：取与月建同气之一干，季月三戊六己九戊十二己（缺省）
    #[default]
    XieJi,
    /// 类编卷二十九「論劒鋒殺并重賻日」引《集正》：三、六月戊，九、十二月己（02094417.cn PDF 第 81 页）
    JiZheng,
    /// 同条引《百忌》：辰戌丑未月戊己俱忌（同页）
    BaiJi,
    /// 同条引《陰陽備用》：辰戌丑未月只忌己（第 81—82 页）
    BeiYong,
    /// 同条引百忌歌「正七連庚甲，二八乙辛當，丁癸五十一，四十丙壬方，辰戌丑未月戊己」两干并取（第 80 页；
    /// 类编名重喪、劒鋒殺）。协纪卷首奏议所斥「通書因誤以正月之庚日七月之甲日皆為復日」即此
    BaiJiGe,
}

/// 荒蕪日之说
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HuangWuRule {
    /// 协纪月家不列（卷三十四按「荒蕪日即五虚日」，五虛已在月神表中）（缺省）
    #[default]
    BuLie,
    /// 按季三支：春巳酉丑、夏申子辰、秋亥卯未、冬寅午戌（协纪卷三十四引《選擇宗鏡》与按语；时宪书朱字所用）
    AnJi,
    /// 类编卷十五「天地荒蕪」表（02094410.cn PDF 第 61 页；卷十种植门 02094407.cn 第 93 页同）：
    /// 巳酉丑辰申子卯未亥寅戌午，一月一支
    LeiBianBiao,
    /// 类编卷十五「天地荒蕪日」新增起例「孟平仲破季逢收」（02094410.cn PDF 第 66—67 页）：巳酉丑申子辰亥卯未寅午戌，
    /// 一月一支；与协纪卷三十四所引宗镜「正月止忌巳日二月止忌酉日三月止忌丑日」同，协纪按「謂一月止忌一字者非」
    LeiBianQiLi,
}

/// 月家日神煞起例选项。缺省全依协纪；[`ShenshaOptions::leibian`] 全依类编（各神取类编正文所主之说）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ShenshaOptions {
    /// 天德、天德合四仲月。类编卷十五日表（02094410.cn PDF 第 52 页）二、五、八、十一月天德作申、亥、寅、巳，
    /// 天德合作巳、寅、亥、申；协纪卷五按「四維無十干，則不得有其日」，四仲无
    pub tian_de: QiLiYi,
    /// 天火。类编卷十五（02094410.cn PDF 第 59 页）正月子顺行四仲（子卯午酉），即协纪卷六所引《歷例》旧文；
    /// 协纪按语改逆行，与災煞同位
    pub tian_huo: QiLiYi,
    pub ming_fei: MingFeiRule,
    pub ming_fei_dui: MingFeiDuiRule,
    pub fu_ri: FuRiRule,
    pub huang_wu: HuangWuRule,
}

impl ShenshaOptions {
    /// 全依类编：復日取所列三曆之首《集正》，鳴吠取十四日，鳴吠對取「鳴吠對日」正表十一日，荒蕪取表序
    pub fn leibian() -> Self {
        Self {
            tian_de: QiLiYi::LeiBian,
            tian_huo: QiLiYi::LeiBian,
            ming_fei: MingFeiRule::LeiBian14,
            ming_fei_dui: MingFeiDuiRule::LeiBian11,
            fu_ri: FuRiRule::JiZheng,
            huang_wu: HuangWuRule::LeiBianBiao,
        }
    }
}

const MING_FEI_14: [&str; 14] =
    ["庚午", "壬申", "癸酉", "壬午", "甲申", "乙酉", "庚寅", "丙申", "丁酉", "壬寅", "丙午", "己酉", "庚申", "辛酉"];
const MING_FEI_12: [&str; 12] = ["壬申", "癸酉", "壬午", "甲申", "乙酉", "丙申", "丁酉", "壬寅", "丙午", "己酉", "庚申", "辛酉"];
const MING_FEI_DUI_11: [&str; 11] = ["甲子", "丙寅", "丁卯", "丙子", "戊寅", "庚寅", "辛卯", "癸卯", "壬子", "甲寅", "乙卯"];
const MING_FEI_DUI_10: [&str; 10] = ["丙寅", "丁卯", "丙子", "辛卯", "甲午", "庚子", "癸卯", "壬子", "甲寅", "乙卯"];

/// 復日之干（干序）。孟仲月诸说同；季月（月序 2、5、8、11）依说
fn fu_ri_gan(month_ord: u8, rule: FuRiRule) -> Vec<u8> {
    let m = (month_ord % 12) as usize;
    // 正甲二乙三戊四丙五丁六己七庚八辛九戊十壬十一癸十二己（协纪）
    const XIEJI: [u8; 12] = [0, 1, 4, 2, 3, 5, 6, 7, 4, 8, 9, 5];
    let ji_yue = m % 3 == 2;
    match rule {
        FuRiRule::XieJi => vec![XIEJI[m]],
        _ if !ji_yue && rule != FuRiRule::BaiJiGe => vec![XIEJI[m]],
        // 集正：三六月戊，九十二月己
        FuRiRule::JiZheng => vec![if m < 6 { 4 } else { 5 }],
        FuRiRule::BaiJi => vec![4, 5],
        FuRiRule::BeiYong => vec![5],
        // 百忌歌：孟仲月取本月与对冲之月之干，季月戊己
        FuRiRule::BaiJiGe => {
            if ji_yue {
                vec![4, 5]
            } else {
                let mut v = vec![XIEJI[m], XIEJI[(m + 6) % 12]];
                v.sort();
                v
            }
        }
    }
}

/// 荒蕪之支（支序）
fn huang_wu_zhi(month_ord: u8, rule: HuangWuRule) -> Vec<u8> {
    let m = (month_ord % 12) as usize;
    // 按季三支（五虛同位）：春巳酉丑、夏申子辰、秋亥卯未、冬寅午戌
    const AN_JI: [[u8; 3]; 4] = [[5, 9, 1], [8, 0, 4], [11, 3, 7], [2, 6, 10]];
    // 表序：巳酉丑辰申子卯未亥寅戌午
    const BIAO: [u8; 12] = [5, 9, 1, 4, 8, 0, 3, 7, 11, 2, 10, 6];
    match rule {
        HuangWuRule::BuLie => vec![],
        HuangWuRule::AnJi => AN_JI[m / 3].to_vec(),
        HuangWuRule::LeiBianBiao => vec![BIAO[m]],
        // 孟平仲破季收：孟月取季内三合第一支（平日）、仲月第二支（破日）、季月第三支（收日）
        HuangWuRule::LeiBianQiLi => vec![AN_JI[m / 3][m % 3]],
    }
}

/// 某月（`month_ord`：寅月 = 0）某日（干支序 `day_gz`）所值之神：(名, 吉)。起例全依协纪
pub fn shen_for_day(month_ord: u8, day_gz: u8) -> Vec<(String, bool)> {
    shen_for_day_with(month_ord, day_gz, &ShenshaOptions::default())
}

/// 同 [`shen_for_day`]，起例按选项
pub fn shen_for_day_with(month_ord: u8, day_gz: u8, o: &ShenshaOptions) -> Vec<(String, bool)> {
    let gz = name60(day_gz);
    let g = gz.chars().next().unwrap().to_string();
    let z = gz.chars().nth(1).unwrap().to_string();
    let m = (month_ord % 12) as usize;
    let (gan_i, zhi_i) = (day_gz % 10, day_gz % 12);
    let zhong = m % 3 == 1;
    let mut out = Vec::new();
    for s in MONTH_SHEN.iter() {
        let key = match s.yi {
            Yi::Gan => &g,
            Yi::Zhi => &z,
            Yi::GanZhi => &gz,
        };
        let on = match s.name.as_str() {
            // 类编四仲：天德申亥寅巳，天德合巳寅亥申（依支）
            "天德" if o.tian_de == QiLiYi::LeiBian && zhong => zhi_i == [8, 11, 2, 5][m / 3],
            "天德合" if o.tian_de == QiLiYi::LeiBian && zhong => zhi_i == [5, 2, 11, 8][m / 3],
            // 类编顺行四仲：正子二卯三午四酉
            "天火" if o.tian_huo == QiLiYi::LeiBian => zhi_i as usize == (3 * m) % 12,
            "復日" if o.fu_ri != FuRiRule::XieJi => fu_ri_gan(month_ord, o.fu_ri).contains(&gan_i),
            _ => s.vals[m].iter().any(|v| v == key),
        };
        if on {
            out.push((s.name.clone(), s.ji));
        }
    }
    if huang_wu_zhi(month_ord, o.huang_wu).contains(&zhi_i) {
        out.push(("荒蕪".to_string(), false));
    }
    for (name, ji, list, _) in FIXED_SHEN.iter() {
        let hit = match *name {
            "鳴吠" => match o.ming_fei {
                MingFeiRule::XieJi => list.contains(&gz.as_str()),
                MingFeiRule::LeiBian14 => MING_FEI_14.contains(&gz.as_str()),
                MingFeiRule::LeiBianAnZang12 => MING_FEI_12.contains(&gz.as_str()),
            },
            "鳴吠對" => match o.ming_fei_dui {
                MingFeiDuiRule::XieJi => list.contains(&gz.as_str()),
                MingFeiDuiRule::LeiBian11 => MING_FEI_DUI_11.contains(&gz.as_str()),
                MingFeiDuiRule::LeiBian10 => MING_FEI_DUI_10.contains(&gz.as_str()),
            },
            _ => list.contains(&gz.as_str()),
        };
        if hit {
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

    // ───────── 类编选项：与类编原表逐日比对 ─────────

    const TONGSHU: &str = include_str!("../../../../data/folk/tongshu_leibian_vs_xieji.csv");

    /// 对照表某神一行：(协纪起例, 类编起例)
    fn tongshu_row(name: &str) -> (String, String) {
        let l = TONGSHU.lines().find(|l| l.split(',').next() == Some(name)).unwrap_or_else(|| panic!("{name}"));
        let f: Vec<&str> = l.split(',').collect();
        (f[1].to_string(), f[2].to_string())
    }
    /// 「a|b|c…」十二月取值，「-」为无；取「：」之后、「（」之前
    fn twelve(s: &str) -> Vec<Vec<String>> {
        let s = s.rsplit('：').next().unwrap();
        let s = s.split('（').next().unwrap();
        let v: Vec<Vec<String>> =
            s.split('|').map(|x| if x == "-" { vec![] } else { x.split('+').map(str::to_string).collect() }).collect();
        assert_eq!(v.len(), 12, "{s}");
        v
    }
    /// 干支串（两字一日）
    fn gz_list(s: &str) -> Vec<String> {
        let cs: Vec<char> = s.chars().collect();
        cs.chunks(2).map(|c| c.iter().collect()).collect()
    }
    fn has(v: &[(String, bool)], n: &str) -> bool {
        v.iter().any(|(x, _)| x == n)
    }
    /// 逐日（12 月 × 60 日）比对实现与原表：表值为干、支或干支，日之干、支、干支有一与之同即值。返回 (比对日数, 合)
    fn compare_month_table(name: &str, table: &[Vec<String>], o: &ShenshaOptions) -> (usize, usize) {
        let mut ok = 0;
        for m in 0..12u8 {
            for d in 0..60u8 {
                let gz = name60(d);
                let (g, z) = (gz.chars().next().unwrap().to_string(), gz.chars().nth(1).unwrap().to_string());
                let want = table[m as usize].iter().any(|v| *v == g || *v == z || *v == gz);
                let got = has(&shen_for_day_with(m, d, o), name);
                if want == got {
                    ok += 1;
                } else {
                    eprintln!("{name} {}月 {gz}：表 {want} 实现 {got}", m + 1);
                }
            }
        }
        (720, ok)
    }
    fn compare_day_list(name: &str, list: &[String], o: &ShenshaOptions) -> (usize, usize) {
        let mut ok = 0;
        for m in 0..12u8 {
            for d in 0..60u8 {
                let want = list.contains(&name60(d));
                if want == has(&shen_for_day_with(m, d, o), name) {
                    ok += 1;
                }
            }
        }
        (720, ok)
    }

    /// 缺省（协纪）与对照表协纪栏逐日相同；选类编后与类编原表（对照表类编栏，均以影像核过）逐日相同。
    /// `cargo test … leibian_options -- --nocapture` 打印逐神结果表（写入 doc/folk/tongshu.md）
    #[test]
    fn leibian_options() {
        let xj = ShenshaOptions::default();
        let lb = ShenshaOptions::leibian();
        let mut rows: Vec<(String, (usize, usize))> = Vec::new();
        for n in ["天德", "天德合", "天火"] {
            let (x, l) = tongshu_row(n);
            rows.push((format!("{n}（协纪）"), compare_month_table(n, &twelve(&x), &xj)));
            rows.push((format!("{n}（类编）"), compare_month_table(n, &twelve(&l), &lb)));
        }
        // 鳴吠：协纪十三日、类编十四日与安葬十二日
        let (_, l) = tongshu_row("鳴吠");
        let parts: Vec<&str> = l.split('；').map(|p| p.rsplit('：').next().unwrap()).collect();
        rows.push(("鳴吠（协纪十三日）".into(), compare_day_list("鳴吠", &gz_list(tongshu_row("鳴吠").0.rsplit('：').next().unwrap()), &xj)));
        for (k, r) in [(0, MingFeiRule::LeiBian14), (1, MingFeiRule::LeiBianAnZang12)] {
            let o = ShenshaOptions { ming_fei: r, ..xj };
            rows.push((format!("鳴吠（类编{}日）", gz_list(parts[k]).len()), compare_day_list("鳴吠", &gz_list(parts[k]), &o)));
        }
        let (x, l) = tongshu_row("鳴吠對");
        let parts: Vec<&str> = l.split('；').map(|p| p.rsplit('：').next().unwrap()).collect();
        rows.push(("鳴吠對（协纪十一日）".into(), compare_day_list("鳴吠對", &gz_list(x.rsplit('：').next().unwrap()), &xj)));
        for (k, r) in [(0, MingFeiDuiRule::LeiBian11), (1, MingFeiDuiRule::LeiBian10)] {
            let o = ShenshaOptions { ming_fei_dui: r, ..xj };
            rows.push((format!("鳴吠對（类编{}日）", gz_list(parts[k]).len()), compare_day_list("鳴吠對", &gz_list(parts[k]), &o)));
        }
        // 荒蕪：类编表序、新增起例
        let (_, l) = tongshu_row("荒蕪");
        let parts: Vec<&str> = l.split('；').collect();
        for (k, r) in [(0, HuangWuRule::LeiBianBiao), (1, HuangWuRule::LeiBianQiLi)] {
            let o = ShenshaOptions { huang_wu: r, ..xj };
            rows.push((format!("荒蕪（{}）", ["类编表序", "类编新增起例"][k]), compare_month_table("荒蕪", &twelve(parts[k]), &o)));
        }
        // 荒蕪按季三支与五虛同位（协纪卷三十四按「荒蕪日即五虚日」）
        let wu_xu = get("五虛").vals.to_vec();
        rows.push(("荒蕪（按季三支＝五虛）".into(), compare_month_table("荒蕪", &wu_xu, &ShenshaOptions { huang_wu: HuangWuRule::AnJi, ..xj })));
        // 復日：类编卷二十九「論劒鋒殺并重賻日」原文（02094417.cn PDF 第 80—82 页，以影像录）
        // 「三曆孟仲月同……同者皆云正甲二乙四丙五丁七庚八辛十壬十一癸」
        let base: [&str; 12] = ["甲", "乙", "", "丙", "丁", "", "庚", "辛", "", "壬", "癸", ""];
        let fill = |ji: [&str; 4]| -> Vec<Vec<String>> {
            (0..12).map(|m| {
                let v = if m % 3 == 2 { ji[m / 3] } else { base[m] };
                v.chars().map(|c| c.to_string()).collect()
            }).collect()
        };
        let (x, _) = tongshu_row("復日");
        rows.push(("復日（协纪）".into(), compare_month_table("復日", &twelve(&x), &xj)));
        // 集正「三六月戊日九與十二月己日」；百忌「辰戌丑未月皆忌戊己二日」；陰陽備用「辰戌丑未月……只忌己日」
        for (label, r, ji) in [
            ("集正", FuRiRule::JiZheng, ["戊", "戊", "己", "己"]),
            ("百忌", FuRiRule::BaiJi, ["戊己", "戊己", "戊己", "戊己"]),
            ("陰陽備用", FuRiRule::BeiYong, ["己", "己", "己", "己"]),
        ] {
            rows.push((format!("復日（类编引{label}）"), compare_month_table("復日", &fill(ji), &ShenshaOptions { fu_ri: r, ..xj })));
        }
        // 百忌歌「正七連庚甲，二八乙辛當，丁癸五十一，四十丙壬方，辰戌丑未月戊己」
        let ge: Vec<Vec<String>> = ["庚甲", "乙辛", "戊己", "丙壬", "丁癸", "戊己", "庚甲", "乙辛", "戊己", "丙壬", "丁癸", "戊己"]
            .iter()
            .map(|s| s.chars().map(|c| c.to_string()).collect())
            .collect();
        rows.push(("復日（百忌歌两干）".into(), compare_month_table("復日", &ge, &ShenshaOptions { fu_ri: FuRiRule::BaiJiGe, ..xj })));
        // 缺省不列荒蕪
        assert!((0..12u8).all(|m| (0..60u8).all(|d| !has(&shen_for_day(m, d), "荒蕪"))));

        println!("| 神（所依） | 比对日数 | 合 | 不合 |\n|---|---|---|---|");
        for (n, (t, ok)) in &rows {
            println!("| {n} | {t} | {ok} | {} |", t - ok);
        }
        for (n, (t, ok)) in &rows {
            assert_eq!(t, ok, "{n}");
        }
    }

    /// 类编选项只动所选之神，其余诸神与协纪逐日相同
    #[test]
    fn leibian_options_isolated() {
        let lb = ShenshaOptions::leibian();
        let touched = ["天德", "天德合", "天火", "復日", "荒蕪", "鳴吠", "鳴吠對"];
        for m in 0..12u8 {
            for d in 0..60u8 {
                let strip = |v: Vec<(String, bool)>| -> Vec<String> {
                    v.into_iter().map(|(n, _)| n).filter(|n| !touched.contains(&n.as_str())).collect()
                };
                assert_eq!(strip(shen_for_day(m, d)), strip(shen_for_day_with(m, d, &lb)), "{m} {d}");
            }
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

    /// 类编卷十一逐月日表（`data/folk/leibian_juan11_days.csv`，02094408.cn 逐页转录）与本模块起例逐日互校。
    /// 卷十一名 → 本模块名（同位异名）；比对 720 日，打印各神合与不合，并断言不合者只是已在 CSV 备注中注明的刻漏。
    /// `cargo test -p lunar-core leibian_juan11 -- --nocapture` 打印比对表（doc/folk/tongshu.md 第七节）。
    #[test]
    fn leibian_juan11() {
        const DAYS: &str = include_str!("../../../../data/folk/leibian_juan11_days.csv");
        // (卷十一名, 本模块名, 依类编选项 true / 依协纪缺省 false)
        let pairs: [(&str, &str, bool); 27] = [
            ("月德", "月德", false), ("月德合", "月德合", false), ("月空", "月空", false), ("月恩", "月恩", false),
            ("天德", "天德", true), ("天德合", "天德合", true), ("天赦", "天赦", false), ("母倉", "母倉", false),
            ("天恩", "天恩", false), ("要安", "要安", false), ("玉堂", "玉宇", false), ("金堂", "金堂", false),
            ("敬心", "敬安", false), ("普護", "普護", false), ("福生", "福生", false), ("聖心", "聖心", false),
            ("益後", "益後", false), ("續世", "續世", false), ("六合", "六合", false), ("游禍", "遊禍", false),
            ("月殺", "月煞", false), ("天火", "天火", true), ("焦坎", "九坎", false), ("飛廉", "大煞", false),
            ("歸忌", "歸忌", false), ("荒蕪", "荒蕪", true), ("正四廢", "四廢", false),
        ];
        // 卷十一刻漏（CSV 备注已注）：(月, 干支, 卷十一名)
        let known_omit = [(3u8, "丁亥", "刀砧"), (6, "庚子", "荒蕪"), (9, "辛亥", "荒蕪"), (9, "庚子", "土瘟")];
        let gz_idx = |s: &str| (0..60u8).find(|&k| name60(k) == s).unwrap();
        let lb = ShenshaOptions::leibian();
        let xj = ShenshaOptions::default();
        let mut stat = vec![(0usize, 0usize, Vec::<String>::new()); pairs.len()];
        let mut rows = 0;
        for l in DAYS.lines().skip(1).filter(|l| !l.trim().is_empty()) {
            let f: Vec<&str> = l.split(',').collect();
            let m: u8 = f[0].parse().unwrap();
            let d = gz_idx(f[1]);
            let names: Vec<&str> = f[4].split(' ').chain(f[5].split(' ')).filter(|s| !s.is_empty()).collect();
            rows += 1;
            let ours_lb: Vec<String> = shen_for_day_with(m - 1, d, &lb).into_iter().map(|x| x.0).collect();
            let ours_xj: Vec<String> = shen_for_day_with(m - 1, d, &xj).into_iter().map(|x| x.0).collect();
            for (k, (a, b, use_lb)) in pairs.iter().enumerate() {
                let ours = if *use_lb { &ours_lb } else { &ours_xj };
                let theirs = names.contains(a);
                let mine = ours.iter().any(|n| n == b);
                if theirs == mine {
                    stat[k].0 += 1;
                } else {
                    stat[k].1 += 1;
                    stat[k].2.push(format!("{m}月{}{}", f[1], if theirs { "卷十一有" } else { "卷十一無" }));
                }
            }
        }
        assert_eq!(rows, 720);
        println!("| 卷十一 | 本项目 | 起例 | 合 | 不合 | 不合之日 |");
        println!("|---|---|---|---|---|---|");
        for (k, (a, b, use_lb)) in pairs.iter().enumerate() {
            println!("| {a} | {b} | {} | {} | {} | {} |", if *use_lb { "类编" } else { "协纪" }, stat[k].0, stat[k].1, stat[k].2.join(" "));
            if *a == "荒蕪" {
                // 不合只许是 CSV 已注的两处漏刻
                for e in &stat[k].2 {
                    assert!(known_omit.iter().any(|(mm, g, n)| *n == "荒蕪" && e == &format!("{mm}月{g}卷十一無")), "{e}");
                }
            } else if *a == "天德" || *a == "天德合" || *a == "天火" {
                assert_eq!(stat[k].1, 0, "{a}");
            }
        }
        // 協紀起例之神除已知异说外逐日全合
        for (k, (a, _, use_lb)) in pairs.iter().enumerate() {
            if !use_lb {
                assert_eq!(stat[k].1, 0, "{a}: {:?}", stat[k].2);
            }
        }
    }
}
