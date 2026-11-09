//! 太乙十精（天皇、帝符、天时、太尊、飞鸟、五行、八风、五风、三风、太乙数）与阳九、百六。
//!
//! 十精起例依金鏡卷七「推天皇太乙法」至「推太乙數法」；十精之目依集成卷六八八「太乙十精定風晴雨露」（一天皇、二帝符、
//! 三天时、四大尊、五飞鸟、六五行、七八风、八五风、九三风、十太乙五元七十二局之数）。
//! - 积数：金鏡卷七「推太乙積年法」演纪上元甲子至宋景祐元年甲戌「一千一十五萬四十九百五十」（当读 10,154,950），
//!   并言「若月日時並置其積同法求之」。十精诸周（二十、十二、四、九、五、七十二）皆整除周纪法三百六十，
//!   故以四计入局数 r 推之即得。景祐元年按本项目积年余 71、按书数余 70，差一；书又言「若求小遊太乙并天目所在義見嵗計」，
//!   秘書至万历戊子积 10,155,505 亦合本项目，故从岁计积年，书数当有一筭之讹（**未核**）。
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

/// 十精所在。`ji` 为四计积数或入局数 r（诸周皆整除三百六十，二者同）。
pub fn shi_jing(ji: i64, which: ShiJing, dun: Dun) -> Wei {
    let w = match which {
        // 「以天皇小周二十去之不盡者命起武徳順行十六神至隂徳大炅大武和徳重留一筭外」
        ShiJing::TianHuang => Wei::Shen(walk(11, &[14, 6, 10, 2], zhou(ji, 20))),
        // 「以帝符小周二十去之不盡從隂主順行十六神至地主髙叢大威太蔟重留一筭外」
        ShiJing::DiFu => Wei::Shen(walk(13, &[0, 4, 8, 12], zhou(ji, 20))),
        // 「以天時小周十二去之不盡命起吕申順行十二辰」
        ShiJing::TianShi => Wei::Zhi(((zhou(ji, 12) - 1 + 2) % 12) as u8),
        // 「以太尊小周四除之不盡命起八宫順行四正宫」：坎八、震四、離二、兌六（四正依十六神顺行之序，**未核**）
        ShiJing::TaiZun => Wei::Gong([8, 4, 2, 6][zhou(ji, 4) as usize - 1]),
        // 「以飛鳥小周九除之不盡命起一宫順行九宫」
        ShiJing::FeiNiao => Wei::Gong(zhou(ji, 9) as u8),
        // 「以五行小周五除之不盡命起一宫以三五七九二四六八」：五除只得五位，取一三五七九（「二四六八」或涉五风而衍，**未核**）
        ShiJing::WuXing => Wei::Gong([1, 3, 5, 7, 9][zhou(ji, 5) as usize - 1]),
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
}
