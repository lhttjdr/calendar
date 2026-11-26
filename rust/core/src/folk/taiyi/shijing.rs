//! 太乙十精（天皇、帝符、天时、太尊、飞鸟、五行、八风、五风、三风、太乙数）与阳九、百六。
//!
//! 十精起例依金鏡卷七「推天皇太乙法」至「推太乙數法」；十精之目依集成卷六八八「太乙十精定風晴雨露」（一天皇、二帝符、
//! 三天时、四大尊、五飞鸟、六五行、七八风、八五风、九三风、十太乙五元七十二局之数）。
//! - 积数：金鏡卷七「推太乙積年法」演纪上元甲子至宋景祐元年甲戌「一千一十五萬四十九百五十」（当读 10,154,950），
//!   并言「若月日時並置其積同法求之」。十精诸周（二十、十二、四、九、五、七十二）皆整除周纪法三百六十，
//!   故以四计入局数 r 推之即得。景祐元年按本项目积年余 71、按书数余 70，差一；书又言「若求小遊太乙并天目所在義見嵗計」，
//!   秘書至万历戊子积 10,155,505 亦合本项目，故从岁计积年，书数当有一筭之讹（**未核**）。
//! - 太尊、五行之序：金鏡文「命起八宫順行四正宫」「命起一宫以三五七九二四六八」读不通，以集成卷六八八「十精定局」立成
//!   （一百八十年逐年列太乙、天目与九精，影印本逐格核，见 doc/folk/taiyi.md 第十三节）定为太尊八六二四、五行一八三九七
//!   （[`ShiJingFa::DingJu`]，缺省）；旧按字读之法（太尊八四二六、五行一三五七九）留作 [`ShiJingFa::JinJingWen`]。
//! - 衰旺：集成卷六八八「衰旺立成」八节各宫旺相胎沒死囚休廢（[`shuai_wang`]）。
//! - 阴遁：「十精以陽遁所衝即各得隂遁十精所在也」：十六神冲为对位（+8），十二辰冲为对支，九宫冲为十减宫数（中五自冲）。
//! - 阳九、百六：金鏡卷七「推太嵗有陽九之災法」「推太乙有百六之厄法」，积年为上元甲寅至开元十二年 13,331
//!   （[`super::zhushen::WU_FU_JI_NIAN_724`]）；集成卷六八七「陽九」「陰六」「自始有甲子，積至所用之年」（[`ELiFa::HuangDi`]）。

use super::zhushen::WU_FU_JI_NIAN_724;
use super::Dun;

/// 十精
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiJing {
    TianHuang,
    DiFu,
    TianShi,
    TaiZun,
    FeiNiao,
    WuXing,
    BaFeng,
    WuFeng,
    SanFeng,
    TaiYiShu,
}

/// 十精（集成卷六八八之序）
pub const SHI_JING: [ShiJing; 10] = [
    ShiJing::TianHuang,
    ShiJing::DiFu,
    ShiJing::TianShi,
    ShiJing::TaiZun,
    ShiJing::FeiNiao,
    ShiJing::WuXing,
    ShiJing::BaFeng,
    ShiJing::WuFeng,
    ShiJing::SanFeng,
    ShiJing::TaiYiShu,
];

impl ShiJing {
    pub fn name(self) -> &'static str {
        match self {
            ShiJing::TianHuang => "天皇",
            ShiJing::DiFu => "帝符",
            ShiJing::TianShi => "天時",
            ShiJing::TaiZun => "太尊",
            ShiJing::FeiNiao => "飛鳥",
            ShiJing::WuXing => "五行",
            ShiJing::BaFeng => "八風",
            ShiJing::WuFeng => "五風",
            ShiJing::SanFeng => "三風",
            ShiJing::TaiYiShu => "太乙數",
        }
    }
}

/// 十精所在
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wei {
    /// 十六神位序（天皇、帝符）
    Shen(usize),
    /// 支序，子 = 0（天时）
    Zhi(u8),
    /// 九宫数（太尊、飞鸟、五行、八风、五风、三风）
    Gong(u8),
    /// 太乙数 1—72
    Shu(u8),
}

/// 积数以周法去之，余零作周法
fn zhou(n: i64, p: i64) -> i64 {
    match n.rem_euclid(p) {
        0 => p,
        k => k,
    }
}

/// 十六神顺行，遇 `stay` 诸位各重留一筭；第 `n` 筭所在
fn walk(start: usize, stay: &[usize], n: i64) -> usize {
    let (mut p, mut stayed) = (start, false);
    for _ in 1..n {
        if stay.contains(&p) && !stayed {
            stayed = true;
        } else {
            p = (p + 1) % 16;
            stayed = false;
        }
    }
    p
}

/// 太尊、五行二精之序
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShiJingFa {
    /// 集成卷六八八「十精定局」立成：太尊坎八、兌六、離二、震四；五行一、八、三、九、七（一百八十年 2160 格，
    /// 除第 135 年三格错位外全合本项目推算）。缺省
    #[default]
    DingJu,
    /// 金鏡卷七按字读：太尊八、四、二、六（四正依十六神顺行）；五行一、三、五、七、九（「二四六八」视为衍文）。
    /// 二者皆为旧时推读，立成不合
    JinJingWen,
}

/// 十精所在（太尊、五行依立成，[`ShiJingFa::DingJu`]）。`ji` 为四计积数或入局数 r（诸周皆整除三百六十，二者同）。
pub fn shi_jing(ji: i64, which: ShiJing, dun: Dun) -> Wei {
    shi_jing_with(ji, which, dun, ShiJingFa::DingJu)
}

/// 十精所在，太尊、五行之序依 `fa`
pub fn shi_jing_with(ji: i64, which: ShiJing, dun: Dun, fa: ShiJingFa) -> Wei {
    let ding = fa == ShiJingFa::DingJu;
    let w = match which {
        // 「以天皇小周二十去之不盡者命起武徳順行十六神至隂徳大炅大武和徳重留一筭外」
        ShiJing::TianHuang => Wei::Shen(walk(11, &[14, 6, 10, 2], zhou(ji, 20))),
        // 「以帝符小周二十去之不盡從隂主順行十六神至地主髙叢大威太蔟重留一筭外」
        ShiJing::DiFu => Wei::Shen(walk(13, &[0, 4, 8, 12], zhou(ji, 20))),
        // 「以天時小周十二去之不盡命起吕申順行十二辰」
        ShiJing::TianShi => Wei::Zhi(((zhou(ji, 12) - 1 + 2) % 12) as u8),
        // 「以太尊小周四除之不盡命起八宫順行四正宫」
        // 立成作八、六、二、四（坎、兌、離、震）
        ShiJing::TaiZun => Wei::Gong(if ding { [8, 6, 2, 4] } else { [8, 4, 2, 6] }[zhou(ji, 4) as usize - 1]),
        // 「以飛鳥小周九除之不盡命起一宫順行九宫」
        ShiJing::FeiNiao => Wei::Gong(zhou(ji, 9) as u8),
        // 「以五行小周五除之不盡命起一宫以三五七九二四六八」：五除只得五位
        // 立成作一、八、三、九、七
        ShiJing::WuXing => Wei::Gong(if ding { [1, 8, 3, 9, 7] } else { [1, 3, 5, 7, 9] }[zhou(ji, 5) as usize - 1]),
        // 「上元甲子嵗起二宫次順行九宫……命起二宮順行二三五六七八九一宫」：后句脱「四」，依前句九宫顺行
        ShiJing::BaFeng => Wei::Gong([2, 3, 4, 5, 6, 7, 8, 9, 1][zhou(ji, 9) as usize - 1]),
        // 「上元甲子嵗起一宫次行三五七九二四六八」
        ShiJing::WuFeng => Wei::Gong([1, 3, 5, 7, 9, 2, 4, 6, 8][zhou(ji, 9) as usize - 1]),
        // 「以三風小周九除之不盡命起三宫次行七二六一五九四八」
        ShiJing::SanFeng => Wei::Gong([3, 7, 2, 6, 1, 5, 9, 4, 8][zhou(ji, 9) as usize - 1]),
        // 「以太乙局法七十二去之不盡命起一筭數外」
        ShiJing::TaiYiShu => Wei::Shu(zhou(ji, 72) as u8),
    };
    if dun == Dun::Yang {
        return w;
    }
    match w {
        Wei::Shen(p) => Wei::Shen((p + 8) % 16),
        Wei::Zhi(z) => Wei::Zhi((z + 6) % 12),
        Wei::Gong(g) => Wei::Gong(10 - g),
        s => s,
    }
}

/// 宫类十精与太乙宫之合（同宫）、冲（「沖者相對，一九二八是也」）：返回如「飛鳥合太乙」「三風沖太乙」
pub fn he_chong(ji: i64, dun: Dun, gong: u8) -> Vec<String> {
    SHI_JING
        .iter()
        .filter_map(|s| match shi_jing(ji, *s, dun) {
            Wei::Gong(g) if g == gong => Some(format!("{}合太乙", s.name())),
            Wei::Gong(g) if g == 10 - gong => Some(format!("{}沖太乙", s.name())),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------- 衰旺

/// 衰旺八等（集成卷六八八「衰旺立成」表头之序）
pub const SHUAI_WANG: [&str; 8] = ["旺", "相", "胎", "沒", "死", "囚", "休", "廢"];
/// 八节（立成行序）
pub const BA_JIE: [&str; 8] = ["立春", "春分", "立夏", "夏至", "立秋", "秋分", "立冬", "冬至"];
/// 立春行之宫序：八、三、四、九、二、七、六、一；余节依次前移一位（立成八行皆如此，唯立冬、冬至二行坤位书作「三」）
const SHUAI_WANG_XU: [u8; 8] = [8, 3, 4, 9, 2, 7, 6, 1];

/// 某节（`jie`：0 立春 … 7 冬至）某宫之衰旺（[`SHUAI_WANG`] 序）；中五无。`gong` 为立成所书之宫数（照原文，不论读法）。
/// 《景祐太乙福應經》卷十末「衰旺立成」（明谈剑山居钞本）八行 64 格与本函数全合，立冬休、冬至囚皆作「二」，
/// 证集成二「三」字为刻误。
pub fn shuai_wang(jie: usize, gong: u8) -> Option<usize> {
    let i = SHUAI_WANG_XU.iter().position(|g| *g == gong)?;
    Some((i + 8 - jie % 8) % 8)
}

/// 衰旺立成宫数之读法
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShuaiWangDu {
    /// 依洛书数（坎一坤二震三巽四乾六兌七艮八離九）：立春艮旺、春分震旺……立冬乾旺、冬至坎旺，即八卦八节旺相之常说；
    /// 福應經卷二「九宫所主」引张良「一為陽九……至亥陰在一宫之地」「二午八子」，八节所当之支（冬至子、立冬亥）
    /// 依此读法与张良之配相应。缺省
    #[default]
    LuoShu,
    /// 依太乙九宫数（乾一離二艮三震四兌六坤七坎八巽九）：福應經「當以太乙立休囚王相決之」，太乙所在宫本以此数，
    /// 照数对表则此读；然冬至乾旺、立春坎旺、立冬兌旺，与张良二至二分之支配皆不相应（**无据待核**）
    TaiYiGong,
}

/// 太乙九宫数 → 洛书数（乾一→六、離二→九、艮三→八、震四→三、中五、兌六→七、坤七→二、坎八→一、巽九→四）
pub const TAI_YI_TO_LUO_SHU: [u8; 10] = [0, 6, 9, 8, 3, 5, 7, 2, 1, 4];

/// 太乙所立之宫（太乙九宫数）当某节之衰旺：福應經卷十「凡占萬物，當以太乙立休囚王相決之」
pub fn tai_yi_shuai_wang(jie: usize, tai_yi_gong: u8, du: ShuaiWangDu) -> Option<usize> {
    let g = match du {
        ShuaiWangDu::LuoShu => *TAI_YI_TO_LUO_SHU.get(tai_yi_gong as usize)?,
        ShuaiWangDu::TaiYiGong => tai_yi_gong,
    };
    shuai_wang(jie, g)
}

// ---------------------------------------------------------------- 阳九、百六

/// 阳九、百六之积年
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ELiFa {
    /// 金鏡卷七：上元甲寅积年（开元十二年 13,331）；所举武德元年入第十阳九、贞观十六年入第二百六诸年皆合。缺省
    JinJing,
    /// 集成卷六八七淘金歌注「自始有甲子，積至所用之年」：取黄帝元年甲子（前 2697）为第一年（淘金歌「即位元年之始，
    /// 正為上元甲子之首」）。所指之年**未核**
    HuangDi,
}

/// 厄会所在
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ELi {
    /// 一元之内第几阳九（1—10）或第几百六（1—15）
    pub di: u8,
    /// 入此阳九（或百六）第几年
    pub ru_nian: u16,
    /// 所临之邦（支序，子 = 0；命起寅邦顺行）
    pub bang: u8,
}

fn e_ji_nian(year: i32, fa: ELiFa) -> i64 {
    match fa {
        ELiFa::JinJing => WU_FU_JI_NIAN_724 + (year as i64 - 724),
        ELiFa::HuangDi => year as i64 + 2697,
    }
}

/// 阳九：「四千五百六十為一元四百五十六嵗為一陽九也十三年移一邦命起寅邦順行十二邦」。
/// 456 非 13 之倍，邦以入阳九以来年数计（每阳九复起寅，**未核**）。
pub fn yang_jiu(year: i32, fa: ELiFa) -> ELi {
    let k = (e_ji_nian(year, fa) - 1).rem_euclid(4560);
    let m = k % 456;
    ELi { di: (k / 456 + 1) as u8, ru_nian: (m + 1) as u16, bang: ((2 + m / 13) % 12) as u8 }
}

/// 百六：「以二百八十八年為一周十五周為一元共計四千三百三十（当作二十）為一元也二十四年移一邦命起寅邦順行」
pub fn bai_liu(year: i32, fa: ELiFa) -> ELi {
    let k = (e_ji_nian(year, fa) - 1).rem_euclid(4320);
    let m = k % 288;
    ELi { di: (k / 288 + 1) as u8, ru_nian: (m + 1) as u16, bang: ((2 + m / 24) % 12) as u8 }
}

#[cfg(test)]
mod tests {
    use super::super::{ji_nian, JiNian};
    use super::*;

    /// 金鏡卷七所举阳九、百六之年（公元年为天文纪年）
    pub(crate) const YANG_JIU_LI: [(i32, &str, u8); 5] = [
        (-750, "周平王宜臼二十年庚寅入第七陽九", 7),
        (-294, "周赧王延二十年丙寅入第八陽九", 8),
        (162, "漢桓帝延熹五年壬寅入第九陽九", 9),
        (618, "隋義寧二年即武德元年戊寅入第十陽九", 10),
        (1074, "開元十二年以後三百五十年方會陽九之數（第一）", 1),
    ];
    pub(crate) const BAI_LIU_LI: [(i32, &str, u8); 6] = [
        (-798, "周宣王二十九年壬寅入第十二百六", 12),
        (-510, "周敬王九年庚寅入第十三百六", 13),
        (-222, "秦始皇二十四年戊寅入第十四百六", 14),
        (66, "漢明帝永平九年丙寅入第十五百六", 15),
        (354, "東晉永和十年甲寅入第二元一百六", 1),
        (642, "大唐貞觀十六年壬寅入第二元二百六", 2),
    ];

    #[test]
    fn jinjing_yang_jiu_bai_liu() {
        for (y, n, di) in YANG_JIU_LI {
            let e = yang_jiu(y, ELiFa::JinJing);
            assert_eq!((e.di, e.ru_nian, e.bang), (di, 1, 2), "{n}");
        }
        for (y, n, di) in BAI_LIU_LI {
            let e = bai_liu(y, ELiFa::JinJing);
            assert_eq!((e.di, e.ru_nian, e.bang), (di, 1, 2), "{n}");
        }
        // 所举之年干支：庚寅、丙寅、壬寅、戊寅……皆寅年
        for y in YANG_JIU_LI.iter().map(|x| x.0).chain(BAI_LIU_LI.iter().map(|x| x.0)) {
            assert_eq!((y - 4).rem_euclid(12), 2, "{y}");
        }
        assert_eq!(bai_liu(354 + 24, ELiFa::JinJing).bang, 3);
    }

    #[test]
    fn shi_jing_cycles() {
        // 天皇、帝符二十筭一周，十六神各至一次，重留四位各二筭
        for s in [ShiJing::TianHuang, ShiJing::DiFu] {
            let mut cnt = [0; 16];
            for n in 1..=20 {
                if let Wei::Shen(p) = shi_jing(n, s, Dun::Yang) {
                    cnt[p] += 1;
                }
            }
            assert_eq!(cnt.iter().sum::<i32>(), 20);
            assert!(cnt.iter().all(|c| *c == 1 || *c == 2));
        }
        assert_eq!(shi_jing(1, ShiJing::TianHuang, Dun::Yang), Wei::Shen(11));
        assert_eq!(shi_jing(20, ShiJing::TianHuang, Dun::Yang), Wei::Shen(10)); // 终于大武重留
        assert_eq!(shi_jing(1, ShiJing::DiFu, Dun::Yang), Wei::Shen(13));
        assert_eq!(shi_jing(1, ShiJing::TianShi, Dun::Yang), Wei::Zhi(2));
        assert_eq!(shi_jing(1, ShiJing::TaiZun, Dun::Yang), Wei::Gong(8));
        assert_eq!(shi_jing(1, ShiJing::BaFeng, Dun::Yang), Wei::Gong(2));
        assert_eq!(shi_jing(1, ShiJing::SanFeng, Dun::Yang), Wei::Gong(3));
        assert_eq!(shi_jing(1, ShiJing::SanFeng, Dun::Yin), Wei::Gong(7));
        assert_eq!(shi_jing(1, ShiJing::TianHuang, Dun::Yin), Wei::Shen(3));
        // 积年与入局数同：十精诸周整除三百六十
        let j = ji_nian(1624, JiNian::JinJing);
        for s in SHI_JING {
            assert_eq!(shi_jing(j, s, Dun::Yang), shi_jing(j.rem_euclid(360), s, Dun::Yang));
        }
        // 景祐元年：书积 10,154,950，本项目余 71、书余 70
        assert_eq!(ji_nian(1034, JiNian::JinJing).rem_euclid(360), 71);
        assert_eq!(10_154_950i64.rem_euclid(360), 70);
    }

    /// 集成卷六八八「十精定局」立成（影印本逐格转录，`data/folk/taiyi_shijing_dingju.csv`）
    const DING_JU_CSV: &str = include_str!("../../../../../data/folk/taiyi_shijing_dingju.csv");

    fn cn(n: u8) -> &'static str {
        ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"][n as usize]
    }

    fn wei_str(w: Wei) -> String {
        match w {
            Wei::Shen(p) => super::super::POS16[p].to_string(),
            Wei::Zhi(z) => "子丑寅卯辰巳午未申酉戌亥".chars().nth(z as usize).unwrap().to_string(),
            Wei::Gong(g) => cn(g).to_string(),
            Wei::Shu(n) => n.to_string(),
        }
    }

    /// 立成一百八十年逐格：第 n 行即入局数 n（前三甲子）与 n + 180（後三甲子），阳遁
    #[test]
    fn ding_ju_li_cheng() {
        let jing = [
            ShiJing::TianHuang,
            ShiJing::DiFu,
            ShiJing::TianShi,
            ShiJing::TaiZun,
            ShiJing::FeiNiao,
            ShiJing::WuXing,
            ShiJing::BaFeng,
            ShiJing::WuFeng,
            ShiJing::SanFeng,
        ];
        let (mut he, mut bu) = (0, vec![]);
        let mut jw_bu = 0;
        let mut rows = 0;
        for line in DING_JU_CSV.lines().filter(|l| !l.starts_with('#')).skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            let n: u16 = f[0].parse().unwrap();
            rows += 1;
            let qian = super::super::ju(n, Dun::Yang);
            let hou = super::super::ju(n + 180, Dun::Yang);
            let mut tui = vec![cn(qian.gong).to_string(), cn(hou.gong).to_string(), super::super::POS16[qian.tian_mu].to_string()];
            assert_eq!(qian.tian_mu, hou.tian_mu);
            for s in jing {
                tui.push(wei_str(shi_jing(n as i64, s, Dun::Yang)));
                // 後三甲子同
                assert_eq!(shi_jing(n as i64, s, Dun::Yang), shi_jing(n as i64 + 180, s, Dun::Yang));
            }
            for (k, t) in tui.iter().enumerate() {
                if f[2 + k] == t {
                    he += 1;
                } else {
                    bu.push((n, k));
                }
            }
            // 旧读法（金鏡文按字）太尊、五行与立成不合之格
            for (k, s) in [(8usize, ShiJing::TaiZun), (10, ShiJing::WuXing)] {
                if wei_str(shi_jing_with(n as i64, s, Dun::Yang, ShiJingFa::JinJingWen)) != f[k] {
                    jw_bu += 1;
                }
            }
        }
        assert_eq!(rows, 180);
        // 2160 格合 2157；第 135 年太尊、飛鳥、五行三格书作「九七一」，推得「二九七」，错一格
        assert_eq!(he, 2157);
        assert_eq!(bu, vec![(135, 6), (135, 7), (135, 8)]);
        // 旧读法：太尊 180 年中 91 不合（90 + 第 135 年之讹）、五行 144 不合
        assert_eq!(jw_bu, 235);
    }

    /// 集成卷六八八「衰旺立成」：八节 × 八宫 64 格
    #[test]
    fn shuai_wang_li_cheng() {
        // 行：立春……冬至；列：旺相胎沒死囚休廢（影印本转录）
        let book: [[u8; 8]; 8] = [
            [8, 3, 4, 9, 2, 7, 6, 1],
            [3, 4, 9, 2, 7, 6, 1, 8],
            [4, 9, 2, 7, 6, 1, 8, 3],
            [9, 2, 7, 6, 1, 8, 3, 4],
            [2, 7, 6, 1, 8, 3, 4, 9],
            [7, 6, 1, 8, 3, 4, 9, 2],
            [6, 1, 8, 3, 4, 9, 3, 7], // 休位书作「三」，当作「二」
            [1, 8, 3, 4, 9, 3, 7, 6], // 囚位书作「三」，当作「二」
        ];
        let mut bu = vec![];
        for (jie, row) in book.iter().enumerate() {
            for (k, g) in row.iter().enumerate() {
                if shuai_wang(jie, *g) != Some(k) {
                    bu.push((BA_JIE[jie], SHUAI_WANG[k]));
                }
            }
        }
        assert_eq!(bu, vec![("立冬", "休"), ("冬至", "囚")]);
        assert_eq!(shuai_wang(0, 5), None);
        // 洛书读：冬至太乙在坎八宫（洛书一）为旺；太乙宫数读：冬至太乙在乾一宫为旺
        assert_eq!(tai_yi_shuai_wang(7, 8, ShuaiWangDu::LuoShu), Some(0));
        assert_eq!(tai_yi_shuai_wang(7, 1, ShuaiWangDu::TaiYiGong), Some(0));
        assert_eq!(tai_yi_shuai_wang(0, 3, ShuaiWangDu::LuoShu), Some(0)); // 立春艮旺
    }
}
