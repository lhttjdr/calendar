//! 太乙算数：阴阳和不和、算名（上和次和下和、重阳重阴等）、三才（无天、无地、无人、三才足）。
//!
//! 诸家说法：
//! - 金鏡卷二「推隂陽和不和」引张良：「下目立正宫為陽立間神為隂立陽筭得竒為重陽立隂筭得偶為重隂則不和……若在陽筭得偶
//!   隂筭得竒為隂陽和」；太乙「八三四九為陽宫……二七六一為隂宫」，阳宫算奇、阴宫算偶亦不和。同篇王希明列算名。
//! - 淘金歌「數主陰陽」（集成卷六八七）：「一三五七九數為陽，二四六八十數為陰。若太乙主客二目，在陽宮數得陰為和，在陰宮
//!   數得陽為和」；并列上和、次和、下和、孤阳、孤阴之数。
//! - 太乙秘書「主客算」注：「太乙在八三四九为阳宫……二目在干离艮震兑坤坎巽为阳宫……在亥丑寅辰巳未申戌为阴神……若太乙、
//!   二目在阳算得偶，在阴得奇，是谓算和」；并列上和至孤阴、三才诸数。
//! - 三才：金鏡卷六「筭中無十者無天……無五者為無地……一十二十三十四十為無人」；淘金歌「天變」「地變」「人變」「三才足」；
//!   秘書「自一至九，为无天；自一至四，为无地；一、十、二十、三十、四十，为无人；十六……三十八，为三才具足之算」。
//!
//! 算之奇偶以整数论（个位之奇偶）。

use super::pos_gong;

/// 阴阳和不和之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeFa {
    /// 金鏡卷二张良：以目（主算用文昌、客算用始击）论：目立正宫为阳、间神为阴，阳算偶、阴算奇为和。缺省
    JinJing,
    /// 淘金歌：以太乙宫论：太乙在阳宫（八三四九）算偶、阴宫（二七六一）算奇为和
    TaoJinGe,
    /// 太乙秘書：太乙宫与目二者俱和方为算和
    MiShu,
}

/// 太乙宫之阴阳：八三四九为阳
pub fn gong_yang(g: u8) -> bool {
    matches!(g, 8 | 3 | 4 | 9)
}

/// 阴阳和否：`suan` 算数，`mu` 所用之目（十六神位），`gong` 太乙宫
pub fn he(suan: u8, mu: usize, gong: u8, fa: HeFa) -> bool {
    let ou = suan % 2 == 0;
    let mu_he = pos_gong(mu).is_some() == ou;
    let gong_he = gong_yang(gong) == ou;
    match fa {
        HeFa::JinJing => mu_he,
        HeFa::TaoJinGe => gong_he,
        HeFa::MiShu => mu_he && gong_he,
    }
}

/// 算名所据之书
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MingFa {
    /// 金鏡卷二王希明所列。缺省
    JinJing,
    /// 淘金歌「數主陰陽」
    TaoJinGe,
    /// 太乙秘書「主客算」注
    MiShu,
}

/// 各书算名表：（名，数）
fn ming_biao(fa: MingFa) -> &'static [(&'static str, &'static [u8])] {
    match fa {
        // 金鏡卷二：「三十三三十九為重陽二十二二十六為重隂二十四二十八為雜隂十三十九三十一三十七為雜陽……
        // 若筭得十四十八三十三為上和二十三二十九三十二為次和十二十六二十七三十四三十八為下和」。
        // 上和「三十三」与同段重阳相犯，且秘書、淘金歌上和皆无之，不取（**讹字未核**，或当作「二十三」而与次和重出）。
        MingFa::JinJing => &[
            ("上和", &[14, 18]),
            ("次和", &[23, 29, 32]),
            ("下和", &[12, 16, 27, 34, 38]),
            ("重陽", &[33, 39]),
            ("重陰", &[22, 26]),
            ("雜陰", &[24, 28]),
            ("雜陽", &[13, 19, 31, 37]),
        ],
        // 淘金歌：「十四、十八、二十三為上和……二十九、三十二、三十六為次和。十二十六、二十七、三十四、三十八為下和」；
        // 「十一、十三、三十七、為孤陽，不利主人。二十四為孤陰，不利為客」
        MingFa::TaoJinGe => &[
            ("上和", &[14, 18, 23]),
            ("次和", &[29, 32, 36]),
            ("下和", &[12, 16, 27, 34, 38]),
            ("孤陽", &[11, 13, 37]),
            ("孤陰", &[24]),
        ],
        // 秘書（「四十四、四十八，为阳中重阴」之数逾算之所及，照录）
        MingFa::MiShu => &[
            ("上和", &[14, 18]),
            ("次和", &[23, 29, 32, 36]),
            ("下和", &[12, 16, 21, 27, 34, 38]),
            ("重陽", &[11, 13, 17, 19, 31, 33, 37, 39]),
            ("重陰", &[22, 24, 26, 28]),
            ("純陽", &[33, 39]),
            ("純陰", &[22, 26]),
            ("陰中重陽", &[11, 17]),
            ("雜重陽", &[13, 19, 31, 37]),
            ("陽中重陰", &[44, 48]),
            ("雜重陰", &[24, 28]),
            ("單陽", &[10, 30]),
            ("孤陰", &[20, 40]),
        ],
    }
}

/// 算名（可有数个，如秘書「重陽、純陽」）
pub fn ming(suan: u8, fa: MingFa) -> Vec<&'static str> {
    ming_biao(fa).iter().filter(|(_, v)| v.contains(&suan)).map(|(n, _)| *n).collect()
}

/// 依算名表论和：上和、次和、下和为和
pub fn ming_he(suan: u8, fa: MingFa) -> bool {
    ming(suan, fa).iter().any(|n| n.ends_with('和'))
}

/// 三才之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SanCaiFa {
    /// 金鏡卷六：无十（一至九）无天；无五（个位一至四）无地；一十、二十、三十、四十无人。三才足金鏡未明言，
    /// 本项目以三无俱不犯且非杜（个位非五）为足。缺省
    JinJing,
    /// 淘金歌「天變」「地變」「人變」「三才足」所列之数
    TaoJinGe,
    /// 太乙秘書所列之数
    MiShu,
}

/// 三才
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SanCai {
    pub wu_tian: bool,
    pub wu_di: bool,
    pub wu_ren: bool,
    pub zu: bool,
}

pub fn san_cai(s: u8, fa: SanCaiFa) -> SanCai {
    let g = s % 10;
    match fa {
        SanCaiFa::JinJing => {
            let (wu_tian, wu_di, wu_ren) = (s < 10, (1..=4).contains(&g), g == 0);
            SanCai { wu_tian, wu_di, wu_ren, zu: !(wu_tian || wu_di || wu_ren) && g != 5 }
        }
        SanCaiFa::TaoJinGe => SanCai {
            wu_tian: s < 10,
            wu_di: s >= 10 && (1..=4).contains(&g),
            wu_ren: g == 0,
            zu: [16, 26, 36, 17, 27, 18, 28, 38, 19, 29, 39].contains(&s),
        },
        SanCaiFa::MiShu => SanCai {
            wu_tian: s < 10,
            wu_di: s <= 4,
            wu_ren: [1, 10, 20, 30, 40].contains(&s),
            zu: [16, 26, 36, 17, 27, 37, 18, 28, 38].contains(&s),
        },
    }
}

/// 三才判语，如「無天」「無地、無人」「三才足」；皆不犯亦不足则空
pub fn san_cai_yu(c: SanCai) -> String {
    let mut v = Vec::new();
    if c.wu_tian {
        v.push("無天");
    }
    if c.wu_di {
        v.push("無地");
    }
    if c.wu_ren {
        v.push("無人");
    }
    if c.zu {
        v.push("三才足");
    }
    v.join("、")
}

#[cfg(test)]
mod tests {
    use super::super::{ju, sui_ji, Dun, JiNian, SHEN16, YUAN};
    use super::*;

    /// 淘金歌（集成卷六八七）「數主陰陽」「天變」「地變」「人變」「三才足」「評十數」所举诸年：
    /// （公元年（天文纪年），元序，局，太乙宫，天目或 None，主算）
    pub(crate) const TAOJINGE_SUAN_LI: [(i32, &str, u8, u8, u8, Option<&str>, u8); 12] = [
        (-178, "漢文帝元年壬戌：次和", 4, 71, 9, Some("大武"), 29),
        (1048, "宋仁宗慶曆八年戊子：天變", 2, 1, 1, None, 7), // 天目「在申」即武德
        (-548, "魯襄公二十四年壬子：無地", 4, 60, 6, None, 23), // 天目「在子」即地主
        (528, "魏孝昌四年戊申：人變", 4, 57, 3, Some("陰主"), 10),
        (-199, "漢高祖七年辛丑：三才足", 4, 50, 1, None, 16),
        (330, "後趙建平元年庚寅：主筭單一", 2, 3, 1, Some("陰主"), 1),
        (11, "新莽始建國三年辛未：主筭二十三", 2, 44, 8, Some("陽德"), 23),
        (-193, "漢惠帝元年丁未：主筭十五杜塞", 4, 56, 3, Some("太簇"), 15),
        (710, "唐睿宗景雲元年庚戌：主筭十六", 2, 23, 9, Some("陰德"), 16),
        (707, "唐中宗景龍元年丁未：主筭單七", 2, 20, 8, Some("太簇"), 7),
        (352, "晉穆帝永和八年壬子：主筭三十九不和", 2, 35, 1, Some("地主"), 39),
        (-23, "（对照）漢成帝陽朔元年丁酉", 2, 10, 4, Some("呂申"), 1),
    ];

    /// 逐例：（例，书之元局宫目算，推之元局宫目算）
    pub(crate) fn taojinge_suan_check() -> Vec<(&'static str, String, String)> {
        TAOJINGE_SUAN_LI
            .iter()
            .map(|(y, n, yuan, jn, g, tm, s)| {
                let j = ju(sui_ji(*y, JiNian::TaoJinGe), Dun::Yang);
                let book = format!("{}元{}局，太乙{}宫，天目{}，主算{}", YUAN[*yuan as usize], jn, g, tm.unwrap_or("—"), s);
                let got = format!(
                    "{}元{}局，太乙{}宫，天目{}，主算{}",
                    YUAN[j.yuan as usize],
                    j.ju,
                    j.gong,
                    if tm.is_some() { SHEN16[j.tian_mu] } else { "—" },
                    j.zhu_suan
                );
                (*n, book, got)
            })
            .collect()
    }

    /// 不合之例（鲁襄公二十四年：书言六十局、主算二十三，推得六十一局、主算三十三，差一局，未得其解）
    pub(crate) const BAD_SUAN_LI: [&str; 3] = ["魯襄公二十四年壬子：無地", "新莽始建國三年辛未：主筭二十三", "晉穆帝永和八年壬子：主筭三十九不和"];

    #[test]
    fn taojinge_suan_examples() {
        let rows = taojinge_suan_check();
        let bad: Vec<&str> = rows.iter().filter(|(_, b, g)| b != g).map(|r| r.0).collect();
        // 新莽始建國三年：书言主筭二十三，而同为戊子元四十四局之咸安元年例书言三十三（推 33），二例自相违，当是此处讹；
        // 永和八年：书言三十五局，而太乙一宫、天目地主、主算三十九皆合二十五局（三十五局太乙当在四宫），「三十五」当作「二十五」
        assert_eq!(bad, BAD_SUAN_LI.to_vec(), "{rows:#?}");
        // 庆历八年天目在申、鲁襄公二十四年天目在子
        let j = ju(sui_ji(1048, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!(super::super::POS16[j.tian_mu], "申");
        // 算名与三才
        assert_eq!(ming(29, MingFa::TaoJinGe), ["次和"]);
        assert!(ming_he(29, MingFa::TaoJinGe));
        assert!(!ming_he(39, MingFa::TaoJinGe) && ming(39, MingFa::JinJing) == ["重陽"]);
        for fa in [SanCaiFa::JinJing, SanCaiFa::TaoJinGe] {
            assert!(san_cai(7, fa).wu_tian);
            assert!(san_cai(23, fa).wu_di);
            assert!(san_cai(10, fa).wu_ren);
            assert!(san_cai(16, fa).zu);
        }
        // 秘書：二十三不入无地（「自一至四」），十六三才足
        assert!(!san_cai(23, SanCaiFa::MiShu).wu_di && san_cai(16, SanCaiFa::MiShu).zu);
    }

    /// 张良之法：文昌在正宫算偶为和
    #[test]
    fn he_rules() {
        // 太乙九宫，大武（坤，正宫）算二十九：目阳算奇，重阳不和；太乙阳宫算奇亦不和
        assert!(!he(29, 10, 9, HeFa::JinJing) && !he(29, 10, 9, HeFa::TaoJinGe));
        assert!(he(14, 10, 9, HeFa::MiShu));
        assert!(he(13, 1, 2, HeFa::MiShu)); // 丑（间神）算奇、太乙二宫（阴）算奇
    }

    /// 比对之法：甲金鏡目法、乙淘金歌宫法、丙秘書兼法、丁金鏡算名、戊淘金歌算名、己秘書算名
    pub(crate) const FA_MING: [&str; 6] = ["甲", "乙", "丙", "丁", "戊", "己"];

    fn yu_ce(s: u8, mu: usize, gong: u8) -> [bool; 6] {
        [
            he(s, mu, gong, HeFa::JinJing),
            he(s, mu, gong, HeFa::TaoJinGe),
            he(s, mu, gong, HeFa::MiShu),
            ming_he(s, MingFa::JinJing),
            ming_he(s, MingFa::TaoJinGe),
            ming_he(s, MingFa::MiShu),
        ]
    }

    /// 秘書七十二局一项：（遁，局，主/客，算，书言和否，诸法推和否）
    pub(crate) type HeRow = (Dun, u16, &'static str, u8, bool, [bool; 6]);
    /// 秘書七十二局门具一项：（遁，局，主/客，大将宫，淘金歌法（开杜门加太乙、大将在开休生）推门具否）
    pub(crate) type MenRow = (Dun, u16, &'static str, Option<u8>, bool);

    fn hz_num(t: &str) -> Option<u16> {
        let d = |c: char| "一二三四五六七八九".chars().position(|x| x == c).map(|p| p as u16 + 1);
        let cs: Vec<char> = t.chars().filter(|c| *c != '單' && *c != '单').collect();
        match cs.as_slice() {
            [a] => d(*a).or((*a == '十').then_some(10)),
            ['十', b] => Some(10 + d(*b)?),
            ['一', '十', b] => Some(10 + d(*b)?),
            [a, '十'] => Some(d(*a)? * 10),
            [a, '十', b] => Some(d(*a)? * 10 + d(*b)?),
            _ => None,
        }
    }

    /// 解析秘書「定主客胜负阳局、阴局七十二局」所注主客算之「和」「不和」及「门具」
    pub(crate) fn mishu_he_check() -> (Vec<HeRow>, Vec<MenRow>) {
        use super::super::bamen::{bu_men, BuMenFa, ZhiShi};
        let path = crate::repo::repo_root().join("vendor/folk/wikisource/太乙秘書/全文.wikitext");
        let text = std::fs::read_to_string(path).expect("太乙秘書");
        let yang = text.find("○ 定主客胜负阳局七十二局").unwrap();
        let yin = text.find("○ 定主客胜负阴局七十二局").unwrap();
        let end = text[yin..].find("○ 太乙总图").map(|e| e + yin).unwrap();
        let (mut hes, mut mens) = (Vec::new(), Vec::new());
        for (dun, seg) in [(Dun::Yang, &text[yang..yin]), (Dun::Yin, &text[yin..end])] {
            for blk in seg.split("● 第").skip(1) {
                let jn: String = blk.chars().take_while(|c| *c != '局').collect();
                let Some(r) = hz_num(&jn) else { continue };
                let j = ju(r, dun);
                for (key, side, s, mu) in [("主算", "主", j.zhu_suan, j.tian_mu), ("客算", "客", j.ke_suan, j.shi_ji)] {
                    let Some(i) = blk.find(key) else { continue };
                    let rest = &blk[i + key.len()..];
                    let t: String = rest.chars().take_while(|c| "單单一二三四五六七八九十".contains(*c)).collect();
                    // 书数与推数不合者（见第十节表）不比
                    if hz_num(&t) != Some(s as u16) {
                        continue;
                    }
                    let tail: String = rest.chars().skip(t.chars().count()).take_while(|c| !"，,）)。﹔".contains(*c)).collect();
                    if tail.contains("不和") {
                        hes.push((dun, r, side, s, false, yu_ce(s, mu, j.gong)));
                    } else if tail.contains('和') {
                        hes.push((dun, r, side, s, true, yu_ce(s, mu, j.gong)));
                    }
                }
                if let Some(i) = blk.find("此局") {
                    let c = &blk[i..];
                    if c.contains("门具") {
                        let side = if c.contains("利为主") { "主" } else if c.contains("利为客") { "客" } else { continue };
                        let da = if side == "主" { j.zhu_da } else { j.ke_da };
                        let m = bu_men(ZhiShi { men: 0, ru: 1 }, j.gong, dun, BuMenFa::KaiMen);
                        let ok = da.and_then(|g| m[g as usize]).is_some_and(|x| x <= 2);
                        mens.push((dun, r, side, da, ok));
                    }
                }
            }
        }
        (hes, mens)
    }

    #[test]
    fn mishu_he_men_ju() {
        let (hes, mens) = mishu_he_check();
        assert!(hes.len() > 100, "{}", hes.len());
        let agree: Vec<usize> = (0..6).map(|k| hes.iter().filter(|h| h.5[k] == h.4).count()).collect();
        // 防回退：诸法与秘書所注之合数（逐局见 doc）
        assert_eq!(agree, MISHU_HE_AGREE.to_vec(), "共 {} 项", hes.len());
        assert_eq!((mens.len(), mens.iter().filter(|m| m.4).count()), MISHU_MEN_AGREE);
    }

    /// 秘書七十二局和否之合数（甲至己）
    const MISHU_HE_AGREE: [usize; 6] = [54, 52, 45, 53, 53, 53];
    /// 秘書七十二局「门具」：（项数，淘金歌法亦门具者）
    const MISHU_MEN_AGREE: (usize, usize) = (14, 1);

    /// 打印逐局表（`cargo test ... -- --ignored --nocapture mishu_he_dump`）
    #[test]
    #[ignore]
    fn mishu_he_dump() {
        let (hes, mens) = mishu_he_check();
        println!("总 {} 项", hes.len());
        for k in 0..6 {
            println!("{} 合 {}", FA_MING[k], hes.iter().filter(|h| h.5[k] == h.4).count());
        }
        for r in 1..=72u16 {
            let cell = |d: Dun| {
                let mut v: Vec<String> = hes
                    .iter()
                    .filter(|h| h.0 == d && h.1 == r)
                    .map(|h| {
                        let ok: String = (0..6).filter(|k| h.5[*k] == h.4).map(|k| FA_MING[k]).collect();
                        format!("{}{}書{}：{}合", h.2, h.3, if h.4 { "和" } else { "不和" }, if ok.is_empty() { "皆不".into() } else { ok })
                    })
                    .collect();
                v.extend(mens.iter().filter(|m| m.0 == d && m.1 == r).map(|m| {
                    format!("{}門具（大將{}宫）：淘金歌{}", m.2, m.3.map(|g| g.to_string()).unwrap_or("杜".into()), if m.4 { "合" } else { "不合" })
                }));
                if v.is_empty() {
                    "—".to_string()
                } else {
                    v.join("；")
                }
            };
            println!("| {} | {} | {} |", r, cell(Dun::Yang), cell(Dun::Yin));
        }
        println!("门具 {} 项，淘金歌法合 {}", mens.len(), mens.iter().filter(|m| m.4).count());
    }
}
