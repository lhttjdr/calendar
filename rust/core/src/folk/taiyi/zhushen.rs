//! 太乙诸神：四神、天乙、地乙、直符（三年一移、十二宫）；君基、臣基、民基（三基）；五福；大游（并大游天目）；
//! 小游；定计（定目）及定计大将参将。
//!
//! 起例依《太乙金鏡式經》卷五（四库本）、《太乙秘書》「太乙总图」「五福太乙」「三基」「四神天乙地乙直符」
//! 「定计目计算大将参将」、《古今圖書集成·藝術典》卷六百八十七《太乙淘金歌》注「主客定計」「五福」「大遊」
//! 「三基」「四神」「天乙」「地乙」「直符」。诸家异说并列为选项，逐例检验见 doc/folk/taiyi.md。
//!
//! 各神所用积年：
//! - 四神、天乙、地乙、直符、小游：金鏡卷五「推小遊太乙積年法（與四神天乙地乙直符同用）」：演纪上元甲子至开元十二年
//!   甲子积 3,361 筭，与卷一积年 1,937,281 以三百六十去之同余，即本项目 [`ji_nian`]（[`JiNian::JinJing`]）。
//! - 君基、臣基、民基：金鏡卷五「推積年法」：上元甲寅至开元十二年积 285,011 筭（[`JIA_YIN_JI_NIAN_724`]）。
//! - 五福、大游：金鏡卷五「推五福太乙法」「推大遊太乙入紀年」：上元甲寅至开元十二年积 13,331 筭
//!   （[`WU_FU_JI_NIAN_724`]；大游又言以晋永和十年甲寅为上元至开元十二年 371 年，以 360 去之同余）。

use super::{gong_pos, ji_nian, jiang, pos_gong, shen16_walk, suan, zhi_pos, JiNian, TaiyiJu};

/// 四神等所行十二宫：1—9 为九宫，10 絳宫、11 明堂、12 玉堂（金鏡卷五「命起一宫順行盡九宫又以絳宫明堂玉堂接之」）
pub const GONG12: [&str; 12] = ["一宮", "二宮", "三宮", "四宮", "五宮", "六宮", "七宮", "八宮", "九宮", "絳宮", "明堂", "玉堂"];
/// 太乙秘書「四神天乙地乙直符」图所配十二支（一宫冀州寅 … 九宫扬州戌、绛宫交州亥、明堂益州子、玉堂幽州丑）
pub const GONG12_ZHI: [&str; 12] = ["寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥", "子", "丑"];
/// 十二支名（子 = 0）
pub const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

/// 所在（宫或邦）与入宫第几年
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zai {
    /// 四神等为十二宫序 1—12（[`GONG12`]）；五福、大游、小游为九宫数 1—9；三基为支序 0—11（[`ZHI`]）
    pub wei: u8,
    /// 入宫（邦）以来第几年，自 1 起
    pub ru_nian: u8,
}

/// 金鏡卷一积年的零起序（上元甲子之岁为 0）
fn m_jia_zi(year: i32) -> i64 {
    ji_nian(year, JiNian::JinJing) - 1
}

// ---------------------------------------------------------------- 四神、天乙、地乙、直符

/// 水火金土四太乙（金鏡卷五「推四神太乙法」「推天乙太乙法」「推直符太乙法」及「四神三元五紀立成」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiTaiYi {
    /// 四神（水神）：命起一宫
    SiShen,
    /// 天乙（金神）：命起六宫
    TianYi,
    /// 地乙（土神）：命起九宫（金鏡正文佚，据卷五立成与集成卷六八七「上元甲子九宮」、太乙秘書「上元起九宫」）
    DiYi,
    /// 直符（火神）：命起五宫
    ZhiFu,
}

impl SiTaiYi {
    /// 上元甲子所起之十二宫零起序
    fn qi(self) -> i64 {
        match self {
            SiTaiYi::SiShen => 0,
            SiTaiYi::TianYi => 5,
            SiTaiYi::DiYi => 8,
            SiTaiYi::ZhiFu => 4,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            SiTaiYi::SiShen => "四神",
            SiTaiYi::TianYi => "天乙",
            SiTaiYi::DiYi => "地乙",
            SiTaiYi::ZhiFu => "直符",
        }
    }
}

/// 四神、天乙、地乙、直符所在：积年以一百八十去之、又以三十六除之为纪（三十六年一纪，即三十六年一周），
/// 入纪年以三约之为宫数，命起本宫顺行九宫、接絳宫明堂玉堂（金鏡卷五）。一百八十为三十六之五倍，故等于积年径以三十六去之
/// （集成卷六八七「以三十六累除之」）。`year` 为干支年（立春分）。
pub fn si_tai_yi(year: i32, which: SiTaiYi) -> Zai {
    let k = m_jia_zi(year).rem_euclid(36);
    Zai { wei: ((k / 3 + which.qi()) % 12 + 1) as u8, ru_nian: (k % 3 + 1) as u8 }
}

// ---------------------------------------------------------------- 三基

/// 金鏡卷五「推積年法」：上元甲寅至唐开元十二年甲子积 285,011 筭（君基、臣基、民基用之）
pub const JIA_YIN_JI_NIAN_724: i64 = 285_011;
/// 金鏡卷五「推五福太乙法」「推大遊太乙入紀年」：上元甲寅至开元十二年积 13,331 筭（五福、大游用之）
pub const WU_FU_JI_NIAN_724: i64 = 13_331;

/// 君基、臣基、民基
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SanJi {
    /// 君基：三十年一邦，三百六十年一周
    Jun,
    /// 臣基：三年一邦，三十六年一周
    Chen,
    /// 民基：一年一邦，十二年一周
    Min,
}

/// 三基起法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SanJiFa {
    /// 金鏡卷五：甲寅积年 285,011（开元十二年），三基皆「命起戍邦順行十二邦」。
    /// 太乙秘書「三基」以甲寅岁起午（君臣起午、民起戌）、其纪年合 13,331 之数，两者相差 240 年，
    /// 君臣二基恰差八邦（戌 + 8 = 午）、民基差二十周，所推全同（见 doc）。缺省。
    JiaYin,
    /// 集成卷六八七淘金歌注：「自上元甲子起午宮」（君、臣；正文君基作「起卯邦」，与其所举天启四年入辰不合，
    /// 歌诀「君臣午上為初說」作午）、民基「自上元甲子起戌」，用上元甲子积年（同金鏡卷一）。
    JiaZi,
}

/// 三基所在（支序 0—11）。`year` 为干支年
pub fn san_ji(year: i32, which: SanJi, fa: SanJiFa) -> Zai {
    let (m, qi) = match fa {
        SanJiFa::JiaYin => ((JIA_YIN_JI_NIAN_724 - 1 + (year as i64 - 724)).rem_euclid(360), [10, 10, 10]),
        SanJiFa::JiaZi => (m_jia_zi(year).rem_euclid(360), [6, 6, 10]),
    };
    let (q, ru, i) = match which {
        SanJi::Jun => (m / 30, m % 30, 0),
        SanJi::Chen => ((m % 36) / 3, m % 3, 1),
        SanJi::Min => (m % 12, 0, 2),
    };
    Zai { wei: ((qi[i] + q) % 12) as u8, ru_nian: (ru + 1) as u8 }
}

// ---------------------------------------------------------------- 五福

/// 五福行宫之序：乾一、艮三、巽九、坤七、中五（金鏡卷五「命起乾艮巽坤中宫」）
pub const WU_FU_XU: [u8; 5] = [1, 3, 9, 7, 5];
/// 五福五宫之名（与 [`WU_FU_XU`] 同序）
pub const WU_FU_GONG: [&str; 5] = ["黃秘宮", "黃始宮", "黃室宮", "黃庭宮", "玄室宮"];

/// 五福起法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WuFuFa {
    /// 金鏡卷五：积年 13,331（开元十二年）以二百二十五去之、四十五约之，命起乾；「今開元十二年甲子在遼東十一年也」。缺省
    JinJing,
    /// 太乙秘書「自万历己卯年入黄秘干一宫」、集成卷六八七淘金歌注「貞觀八年甲午五福入中宮」「自天啟甲子起五福入艮宮」：
    /// 二书相合（乾宫始于 679 + 225k 年），比金鏡晚十年。二书所用积年未明载，此由其所举之年反推（未核）。
    MiShu,
}

/// 五福所在宫（1、3、9、7、5）与入宫年
pub fn wu_fu(year: i32, fa: WuFuFa) -> Zai {
    let m = match fa {
        WuFuFa::JinJing => (WU_FU_JI_NIAN_724 - 1 + (year as i64 - 724)).rem_euclid(225),
        WuFuFa::MiShu => (year as i64 - 4).rem_euclid(225),
    };
    Zai { wei: WU_FU_XU[(m / 45) as usize], ru_nian: (m % 45 + 1) as u8 }
}

// ---------------------------------------------------------------- 大游、小游

/// 大游行宫之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DaYouFa {
    /// 金鏡卷五：积年 13,331 以元法 4320 去之、以大游小周 288 去之、三十六除之为宫数，「命起七宫順行八宫不遊中五」
    /// （七八九一二三四六）。太乙秘書「长兴元年庚寅岁，初起七宫」「万历丁亥岁大游太乙入巽九宫」，
    /// 集成卷六八七引《太乙统宗》「自後唐明宗長興元年庚寅歲大遊入七宮……順行，則在乾一宮已十年」，皆合。缺省
    Shun,
    /// 集成卷六八七淘金歌：「入宮逆數……七六四三二一九八」，「隋煬帝大業元年乙丑，是年大遊入三宮」，
    /// 天启四年在八宫；注者自言「俱與《淘金歌》順逆不同，俟攷」。起点由大业元年入三宫反推（497 + 288k 年入七宫）。
    Ni,
}

/// 大游所在宫与入宫年
pub fn da_you(year: i32, fa: DaYouFa) -> Zai {
    let (m, xu) = match fa {
        DaYouFa::Shun => ((WU_FU_JI_NIAN_724 - 1 + (year as i64 - 724)).rem_euclid(288), [7, 8, 9, 1, 2, 3, 4, 6]),
        DaYouFa::Ni => ((year as i64 - 497).rem_euclid(288), [7, 6, 4, 3, 2, 1, 9, 8]),
    };
    Zai { wei: xu[(m / 36) as usize], ru_nian: (m % 36 + 1) as u8 }
}

/// 大游天目（十六神位序）：金鏡卷五「置上元以來積年以天目元法七十二去之不盡以天目周法十八去之命起天道順行十六神
/// 遇大武隂徳重留一筭外」；「上元甲寅嵗……天道為天目」。积年用 13,331 之数（大游同章）。
pub fn da_you_tian_mu(year: i32) -> usize {
    shen16_walk(9, [10, 14], WU_FU_JI_NIAN_724 + (year as i64 - 724))
}

/// 小游：金鏡卷五「推小遊太乙積年法」与岁计太乙同法（命起一宫順行八宫不遊中五），「小遊主事見在嵗計中」；
/// 太乙秘書「小游太乙阳遁九宫阴一」即岁计太乙之宫。此返回阳遁岁计太乙宫。
pub fn xiao_you(year: i32) -> u8 {
    super::ju(super::sui_ji(year, JiNian::JinJing), super::Dun::Yang).gong
}

// ---------------------------------------------------------------- 定计

/// 定计算之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DingJiFa {
    /// 集成卷六八七淘金歌「主客定計」：「依數筭來太乙前，亦如主客推之法」，至太乙前一宫止；
    /// 建安五年例「起筭太乙前十六……計大將在六宮，計參將在八宮」合。缺省
    TaoJinGe,
    /// 太乙秘書：「计算与主客算同，分正间，数过太乙一宫而止」，即连太乙宫及其后一宫并数；
    /// 小淘金歌「就从定计起计算，谕过太乙宫一陈」。万历戊子阳遁计算三十四合此（阴遁书作二十五，不合，见 doc）。
    MiShu,
}

/// 定计目与定计大将、参将
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DingJi {
    /// 定计目所在十六神位序
    pub mu: usize,
    pub suan: u8,
    /// 大将、参将所在宫；得五为杜（None）
    pub da: Option<u8>,
    pub can: Option<u8>,
}

/// 支之合神（子丑、寅亥、卯戌、辰酉、巳申、午未；金鏡卷一「推太嵗支合神法」「太嵗在子合神在丑逆行十二辰」）
pub fn he_shen(zhi: u8) -> u8 {
    ((13 - zhi as i32 % 12) % 12) as u8
}

/// 定计：以合神加该计之支（岁计太岁、月计月建、日计日支、时计时支），视天目所临之下为定计目，
/// 自目起算如主客算（集成卷六八七淘金歌注「用年則以年支合加太歲，用月則以月支合加月建，用日則以日支合加日支，
/// 用時則以時支合加時支……看文昌所臨之下」；太乙秘書「阴阳二目俱以合神加太岁，并月日时之辰，视文昌下即计目也」）。
/// 去十用零为大将，三因之为参将。
pub fn ding_ji(j: &TaiyiJu, zhi: u8, fa: DingJiFa) -> DingJi {
    let z = zhi % 12;
    let mu = (j.tian_mu + 16 + zhi_pos(z as usize) - zhi_pos(he_shen(z) as usize)) % 16;
    let mut s = suan(mu, j.gong) as u32;
    if fa == DingJiFa::MiShu {
        let tp = gong_pos(j.gong);
        if mu != tp {
            s += j.gong as u32;
        }
        // 太乙后一宫：十六神顺行下一正宫
        let next = pos_gong((tp + 2) % 16).expect("正宫");
        s += next as u32;
    }
    let da = jiang(s);
    DingJi { mu, suan: s as u8, da, can: da.and_then(|d| jiang(d as u32 * 3)) }
}

#[cfg(test)]
mod tests {
    use super::super::{ju, sui_ji, Dun, JiNian, POS16, SHEN16};
    use super::*;

    fn zhi_of(y: i32) -> u8 {
        (y - 4).rem_euclid(12) as u8
    }

    /// 金鏡卷五「四神三元五紀立成」逐年比对：自 vendor 录文解析六十年 × 上中下三元 × 四神天乙地乙直符
    #[test]
    fn jinjing_juan5_si_shen_li_cheng() {
        let path = crate::repo::repo_root().join("vendor/folk/wikisource/太乙金鏡式經/卷05.wikitext");
        let text = std::fs::read_to_string(path).expect("卷五录文");
        let rows = parse_si_shen_li_cheng(&text);
        assert_eq!(rows.len(), 60, "立成六十行");
        let kinds = [SiTaiYi::SiShen, SiTaiYi::TianYi, SiTaiYi::DiYi, SiTaiYi::ZhiFu];
        let mut bad = Vec::new();
        for (gz, cells) in rows.iter().enumerate() {
            for (ki, k) in kinds.iter().enumerate() {
                for yuan in 0..3 {
                    // 上元甲子取 1684 − 180·n 之类皆可：以金鏡积年，上元甲子 = 积年零起序 ≡ 0 (mod 180)
                    // 积年零起序 = 公元年 + 116，故上元甲子之岁为 64 + 180k（如 1864）
                    let y = 1864 + 60 * yuan as i32 + gz as i32;
                    let got = si_tai_yi(y, *k).wei;
                    let want = cells[ki * 3 + yuan];
                    if got != want {
                        bad.push((gz, k.name(), yuan, want, got));
                    }
                }
            }
        }
        // 唯直符中元辛卯至癸亥三十三年与上下文不连（每三年一宫之序整体错三宫，当是传抄错行），余 687 格全合
        assert!(bad.iter().all(|b| b.1 == "直符" && b.2 == 1 && b.0 >= 27), "{bad:?}");
        assert_eq!(bad.len(), 33, "{bad:?}");
    }

    /// 解析「四神三元五紀立成」：返回每行（甲子起）十二格（四神上中下、天乙上中下、地乙上中下、直符上中下），
    /// 絳、明、玉记 10、11、12
    pub(crate) fn parse_si_shen_li_cheng(text: &str) -> Vec<[u8; 12]> {
        let start = text.find("四神三元五紀立成").expect("立成");
        let body = &text[start..];
        let end = body.find("太乙金鏡式經卷五").unwrap_or(body.len());
        let body = &body[..end];
        let gz: Vec<String> = (0..60).map(|i| format!("{}{}", "甲乙丙丁戊己庚辛壬癸".chars().nth(i % 10).unwrap(), ZHI[i % 12])).collect();
        let norm = body.replace('夘', "卯");
        let val = |c: char| -> Option<u8> {
            "一二三四五六七八九絳明玉".chars().position(|x| x == c).map(|p| p as u8 + 1)
        };
        let mut out = Vec::new();
        for g in &gz {
            let i = norm.find(&format!("　{g}　")).unwrap_or_else(|| panic!("{g}"));
            let cells: Vec<u8> = norm[i + g.len() + 3..].chars().filter_map(val).take(12).collect();
            out.push(cells.try_into().unwrap());
        }
        out
    }

    /// 打印金鏡卷五立成比对表（`-- --ignored --nocapture si_shen_dump`）：每格「书」，不合者「书→推」
    #[test]
    #[ignore]
    fn si_shen_dump() {
        let path = crate::repo::repo_root().join("vendor/folk/wikisource/太乙金鏡式經/卷05.wikitext");
        let rows = parse_si_shen_li_cheng(&std::fs::read_to_string(path).unwrap());
        let kinds = [SiTaiYi::SiShen, SiTaiYi::TianYi, SiTaiYi::DiYi, SiTaiYi::ZhiFu];
        let nm = |v: u8| ["", "一", "二", "三", "四", "五", "六", "七", "八", "九", "絳", "明", "玉"][v as usize];
        for (gz, cells) in rows.iter().enumerate() {
            let mut line = format!("| {}{} |", "甲乙丙丁戊己庚辛壬癸".chars().nth(gz % 10).unwrap(), ZHI[gz % 12]);
            for (ki, k) in kinds.iter().enumerate() {
                for yuan in 0..3 {
                    let got = si_tai_yi(1864 + 60 * yuan as i32 + gz as i32, *k).wei;
                    let want = cells[ki * 3 + yuan];
                    if got == want {
                        line += &format!(" {} |", nm(want));
                    } else {
                        line += &format!(" **{}→{}** |", nm(want), nm(got));
                    }
                }
            }
            println!("{line}");
        }
    }

    /// 太乙秘書「太乙总图」与「四神天乙地乙直符」所举之年
    #[test]
    fn mishu_si_shen_examples() {
        let zhi = |y, k| GONG12_ZHI[si_tai_yi(y, k).wei as usize - 1];
        // 四神：嘉靖壬午元年入申，戊午又入申，万历乙酉入巳，戊子入午
        assert_eq!(zhi(1522, SiTaiYi::SiShen), "申");
        assert_eq!(si_tai_yi(1522, SiTaiYi::SiShen).ru_nian, 1);
        assert_eq!(zhi(1558, SiTaiYi::SiShen), "申");
        assert_eq!(zhi(1585, SiTaiYi::SiShen), "巳");
        assert_eq!(si_tai_yi(1585, SiTaiYi::SiShen).ru_nian, 1);
        assert_eq!(zhi(1588, SiTaiYi::SiShen), "午");
        // 总图：万历乙酉天乙入戌（「万历乙酉年入九宫，丁亥年出」）、地乙入丑（玉堂）、直符入酉（八宫）；丁亥满
        assert_eq!((zhi(1585, SiTaiYi::TianYi), si_tai_yi(1585, SiTaiYi::TianYi).wei), ("戌", 9));
        assert_eq!((zhi(1585, SiTaiYi::DiYi), si_tai_yi(1585, SiTaiYi::DiYi).wei), ("丑", 12));
        assert_eq!((zhi(1585, SiTaiYi::ZhiFu), si_tai_yi(1585, SiTaiYi::ZhiFu).wei), ("酉", 8));
        for k in [SiTaiYi::SiShen, SiTaiYi::TianYi, SiTaiYi::DiYi, SiTaiYi::ZhiFu] {
            assert_eq!(si_tai_yi(1587, k).ru_nian, 3, "{} 丁亥满", k.name());
        }
        // 直符：「元辛卯三年入二宫……甲午三年入三宫……丁酉三年临四宫」（至正十一年 1351）；「戊午入明堂」（嘉靖 1558）
        assert_eq!((si_tai_yi(1351, SiTaiYi::ZhiFu).wei, si_tai_yi(1351, SiTaiYi::ZhiFu).ru_nian), (2, 1));
        assert_eq!((si_tai_yi(1354, SiTaiYi::ZhiFu).wei, si_tai_yi(1354, SiTaiYi::ZhiFu).ru_nian), (3, 1));
        assert_eq!((si_tai_yi(1357, SiTaiYi::ZhiFu).wei, si_tai_yi(1357, SiTaiYi::ZhiFu).ru_nian), (4, 1));
        assert_eq!(GONG12[si_tai_yi(1558, SiTaiYi::ZhiFu).wei as usize - 1], "明堂");
    }

    /// 三基：太乙秘書、金鏡、集成卷六八七诸例
    #[test]
    fn san_ji_examples() {
        let z = |y, k, f| ZHI[san_ji(y, k, f).wei as usize];
        let f = SanJiFa::JiaYin;
        // 秘書君基：「甲寅岁起午，至甲申入未，嘉靖甲申入酉，甲寅入戌，万历甲申入亥」；总图「君基—万历甲申入亥巳四年」（丁亥）
        assert_eq!(z(1524, SanJi::Jun, f), "酉");
        assert_eq!(san_ji(1524, SanJi::Jun, f).ru_nian, 1);
        assert_eq!(z(1523, SanJi::Jun, f), "申");
        assert_eq!(z(1554, SanJi::Jun, f), "戌");
        assert_eq!(z(1584, SanJi::Jun, f), "亥");
        assert_eq!(san_ji(1587, SanJi::Jun, f), Zai { wei: 11, ru_nian: 4 });
        // 秘書臣基：「嘉靖甲寅年入戌，丁巳入亥，万历丁亥入酉」；总图「臣基—万历丁亥入酉巳一年」
        assert_eq!(z(1554, SanJi::Chen, f), "戌");
        assert_eq!(z(1557, SanJi::Chen, f), "亥");
        assert_eq!(san_ji(1587, SanJi::Chen, f), Zai { wei: 9, ru_nian: 1 });
        // 秘書民基：「嘉靖丁巳入丑，万历丁亥入未」
        assert_eq!(z(1557, SanJi::Min, f), "丑");
        assert_eq!(z(1587, SanJi::Min, f), "未");
        // 金鏡卷五：积年 285,011 以三百六十去之余 251（开元十二年），三十除之得八邦、入邦十一年，命起戌：午
        assert_eq!((JIA_YIN_JI_NIAN_724 % 360, z(724, SanJi::Jun, f)), (251, "午"));
        assert_eq!(san_ji(724, SanJi::Jun, f).ru_nian, 11);
        // 淘金歌注（上元甲子起午）：「君基於天啟三年甲子，入辰宮」（天启四年甲子 1624）；「臣基於天啟三年甲子入戌宮，
        // 三年至丁卯入亥宮」；「民基卯年在丑」（丁卯 1627）
        let g = SanJiFa::JiaZi;
        assert_eq!((z(1624, SanJi::Jun, g), san_ji(1624, SanJi::Jun, g).ru_nian), ("辰", 1));
        assert_eq!((z(1624, SanJi::Chen, g), san_ji(1624, SanJi::Chen, g).ru_nian), ("戌", 1));
        assert_eq!((z(1627, SanJi::Chen, g), san_ji(1627, SanJi::Chen, g).ru_nian), ("亥", 1));
        assert_eq!(z(1627, SanJi::Min, g), "丑");
        // 两法不同：天启四年甲寅法君基子、臣基酉
        assert_eq!((z(1624, SanJi::Jun, f), z(1624, SanJi::Chen, f)), ("子", "酉"));
    }

    #[test]
    fn wu_fu_examples() {
        // 金鏡卷五：「今開元十二年甲子在遼東十一年也」（艮三宫黄始）
        assert_eq!(wu_fu(724, WuFuFa::JinJing), Zai { wei: 3, ru_nian: 11 });
        // 太乙秘書：「自万历己卯年入黄秘干一宫」；总图「五福—万历己卯年入黄秘宫巳九年」（丁亥）
        assert_eq!(wu_fu(1579, WuFuFa::MiShu), Zai { wei: 1, ru_nian: 1 });
        assert_eq!(wu_fu(1587, WuFuFa::MiShu), Zai { wei: 1, ru_nian: 9 });
        // 集成卷六八七：「唐太宗貞觀八年甲午，是年五福入中宮」；「自天啟甲子起五福入艮宮，至丁卯四年矣」
        assert_eq!(wu_fu(634, WuFuFa::MiShu), Zai { wei: 5, ru_nian: 1 });
        assert_eq!(wu_fu(1624, WuFuFa::MiShu), Zai { wei: 3, ru_nian: 1 });
        assert_eq!(wu_fu(1627, WuFuFa::MiShu).ru_nian, 4);
        // 二法相差十年：金鏡万历己卯在乾已十一年
        assert_eq!(wu_fu(1579, WuFuFa::JinJing), Zai { wei: 1, ru_nian: 11 });
    }

    #[test]
    fn da_you_examples() {
        let s = DaYouFa::Shun;
        // 金鏡卷五：以晋永和十年甲寅（354）为上元，「入上元第一紀三百七十一年」（开元十二年）；上元甲寅太乙在七宫、天道为天目
        assert_eq!(da_you(354, s), Zai { wei: 7, ru_nian: 1 });
        assert_eq!(SHEN16[da_you_tian_mu(354)], "天道");
        assert_eq!(da_you(724, s), Zai { wei: 9, ru_nian: 11 });
        // 太乙秘書：「长兴元年庚寅岁，初起七宫」（930）；「万历丁亥岁，大游太乙入巽九宫」
        assert_eq!(da_you(930, s), Zai { wei: 7, ru_nian: 1 });
        assert_eq!(da_you(1587, s).wei, 9);
        // 集成卷六八七引太乙统宗：长兴元年入七宫起，至天启三年癸亥「順行，則在乾一宮已十年」
        assert_eq!(da_you(1623, s), Zai { wei: 1, ru_nian: 10 });
        // 淘金歌（逆行）：「隋煬帝大業元年乙丑，是年大遊入三宮」；天启四年「入八宮子地，至天啟七年丁卯十五年矣」
        let n = DaYouFa::Ni;
        assert_eq!(da_you(605, n), Zai { wei: 3, ru_nian: 1 });
        assert_eq!(da_you(1627, n), Zai { wei: 8, ru_nian: 15 });
        // 大游天目：天道起，十八年一周，大武、陰德重留
        let seq: Vec<&str> = (0..18).map(|k| SHEN16[da_you_tian_mu(354 + k)]).collect();
        assert_eq!(&seq[..4], &["天道", "大武", "大武", "武德"]);
        assert_eq!(da_you_tian_mu(354 + 18), da_you_tian_mu(354));
    }

    #[test]
    fn xiao_you_is_sui_ji() {
        // 太乙秘書：「万历丁亥岁……小游太乙阳遁九宫阴一」
        assert_eq!(xiao_you(1587), 9);
        assert_eq!(ju(sui_ji(1587, JiNian::JinJing), Dun::Yin).gong, 1);
    }

    #[test]
    fn ding_ji_examples() {
        assert_eq!((0..12).map(he_shen).collect::<Vec<_>>(), vec![1, 0, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2]);
        // 淘金歌注：建安五年庚辰，天目大武、太乙七宫；「以庚辰年合神酉加太歲辰位，天目臨呂申，起筭太乙前十六，
        // 乃為定計目也，計大將在六宮，計參將在八宮」
        let j = ju(sui_ji(200, JiNian::TaoJinGe), Dun::Yang);
        let d = ding_ji(&j, zhi_of(200), DingJiFa::TaoJinGe);
        assert_eq!((SHEN16[d.mu], d.suan, d.da, d.can), ("呂申", 16, Some(6), Some(8)));
        // 太乙秘書：万历戊子岁，阳遁定计辰宫，计算三十四，大将四宫，参将二宫
        let r = sui_ji(1588, JiNian::JinJing);
        let a = ju(r, Dun::Yang);
        let d = ding_ji(&a, zhi_of(1588), DingJiFa::MiShu);
        assert_eq!((POS16[d.mu], d.suan, d.da, d.can), ("辰", 34, Some(4), Some(2)));
        // 阴遁：定计戌宫（合），书言计算二十五、二将杜；按「数过太乙一宫」得二十八、大将八、参将四（不合，见 doc）
        let b = ju(r, Dun::Yin);
        let d = ding_ji(&b, zhi_of(1588), DingJiFa::MiShu);
        assert_eq!((POS16[d.mu], d.suan, d.da, d.can), ("戌", 28, Some(8), Some(4)));
        // 总图（万历丁亥）：「定计将—阳丑，阴未；计大将—阳九，阴三；计参将—阳九，阴七」
        let r = sui_ji(1587, JiNian::JinJing);
        let a = ding_ji(&ju(r, Dun::Yang), zhi_of(1587), DingJiFa::MiShu);
        let b = ding_ji(&ju(r, Dun::Yin), zhi_of(1587), DingJiFa::MiShu);
        assert_eq!((POS16[a.mu], a.da, POS16[b.mu], b.da), ("丑", Some(9), "未", Some(3)));
        // 参将：三因得阳七、阴九，与书阳九阴七互易
        assert_eq!((a.can, b.can), (Some(7), Some(9)));
    }
}
