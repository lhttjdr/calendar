//! 十神（基础层）：以日干为我，比他干的五行生克与阴阳。
//!
//! 《淵海子平》卷一《論五行相生相尅訣》（明萬曆二十八年喬山堂本书影）：「生我者為正印偏印，我生者為傷官食神，
//! 尅我者為正官七殺，我尅者為偏財正財，比者為劫財敗財。其法陽見陰為正，陰見陽為正，陽見陽為偏，陰見陰為偏。」
//! 比和者同阴阳为比肩、异阴阳为劫财（《三命通會》卷五《論古人立印食官財名義》）。

use super::{gan_is_yang, gan_wuxing};

/// 十神
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShiShen {
    BiJian,
    JieCai,
    ShiShen,
    ShangGuan,
    PianCai,
    ZhengCai,
    QiSha,
    ZhengGuan,
    PianYin,
    ZhengYin,
}

impl ShiShen {
    pub const ALL: [ShiShen; 10] = [
        ShiShen::BiJian,
        ShiShen::JieCai,
        ShiShen::ShiShen,
        ShiShen::ShangGuan,
        ShiShen::PianCai,
        ShiShen::ZhengCai,
        ShiShen::QiSha,
        ShiShen::ZhengGuan,
        ShiShen::PianYin,
        ShiShen::ZhengYin,
    ];

    /// 名目（七杀一名偏官，偏印一名枭神、倒食，正印一名印绶）
    pub fn name(self) -> &'static str {
        match self {
            ShiShen::BiJian => "比肩",
            ShiShen::JieCai => "劫財",
            ShiShen::ShiShen => "食神",
            ShiShen::ShangGuan => "傷官",
            ShiShen::PianCai => "偏財",
            ShiShen::ZhengCai => "正財",
            ShiShen::QiSha => "七殺",
            ShiShen::ZhengGuan => "正官",
            ShiShen::PianYin => "偏印",
            ShiShen::ZhengYin => "正印",
        }
    }

    /// 简称（排盘习用一字）
    pub fn short(self) -> &'static str {
        match self {
            ShiShen::BiJian => "比",
            ShiShen::JieCai => "劫",
            ShiShen::ShiShen => "食",
            ShiShen::ShangGuan => "傷",
            ShiShen::PianCai => "才",
            ShiShen::ZhengCai => "財",
            ShiShen::QiSha => "殺",
            ShiShen::ZhengGuan => "官",
            ShiShen::PianYin => "梟",
            ShiShen::ZhengYin => "印",
        }
    }
}

/// 日干 `me` 视他干 `other` 为何神
pub fn shi_shen(me: u8, other: u8) -> ShiShen {
    let (a, b) = (gan_wuxing(me) as i32, gan_wuxing(other) as i32);
    let same = gan_is_yang(me) == gan_is_yang(other);
    // 相生次序 木火土金水：b − a = 0 同我，1 我生，2 我克，3 克我，4 生我
    match ((b - a).rem_euclid(5), same) {
        (0, true) => ShiShen::BiJian,
        (0, false) => ShiShen::JieCai,
        (1, true) => ShiShen::ShiShen,
        (1, false) => ShiShen::ShangGuan,
        (2, true) => ShiShen::PianCai,
        (2, false) => ShiShen::ZhengCai,
        (3, true) => ShiShen::QiSha,
        (3, false) => ShiShen::ZhengGuan,
        (4, true) => ShiShen::PianYin,
        _ => ShiShen::ZhengYin,
    }
}

/// 日干视某十神所在之干（与 [shi_shen] 互逆）
pub fn gan_of_shi_shen(me: u8, s: ShiShen) -> u8 {
    (0..10).find(|&g| shi_shen(me, g) == s).expect("十神必有一干")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table() {
        // 甲日：甲比 乙劫 丙食 丁傷 戊才 己財 庚殺 辛官 壬梟 癸印
        let names: Vec<_> = (0..10).map(|g| shi_shen(0, g).name()).collect();
        assert_eq!(names, ["比肩", "劫財", "食神", "傷官", "偏財", "正財", "七殺", "正官", "偏印", "正印"]);
        // 癸日：甲傷 乙食 丙財 丁才 戊官 己殺 庚印 辛梟 壬劫 癸比
        let names: Vec<_> = (0..10).map(|g| shi_shen(9, g).short()).collect();
        assert_eq!(names, ["傷", "食", "財", "才", "官", "殺", "印", "梟", "劫", "比"]);
        // 每日干十神各一
        for me in 0..10 {
            for s in ShiShen::ALL {
                assert_eq!(shi_shen(me, gan_of_shi_shen(me, s)), s);
            }
        }
        // 《三命通會》卷五：「如吕吉甫學士壬申己酉丁巳庚子，壬與巳（己）庚與丁為官」——丁日见壬为官
        assert_eq!(shi_shen(3, 8), ShiShen::ZhengGuan);
    }
}
