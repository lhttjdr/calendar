//! 地支藏干与人元司令（基础层）。
//!
//! 诸家藏干表（逐支所藏之干）：
//! - [CangGanTable::YuanHai]：《淵海子平》卷一《論支中所藏》歌诀（明萬曆二十八年喬山堂本书影）：「子宮單癸水，丑癸己辛逢，
//!   寅還見甲丙，卯宮獨乙木，辰藏癸戊乙，巳丙戊庚中，午宮丁己土，未內乙己丁，申宮庚壬戊，酉內獨辛金，戌宮辛丁戊，亥宮壬甲從」。
//!   寅只甲丙、无戊（寅字残损为版片之损，台湾图书馆藏同版印本同）。
//! - [CangGanTable::SanMing]：《三命通會》卷二《論人元司事》首段，逐月用事之干与日数（「正月建寅，寅中有艮土用事五日，丙火長生五日，
//!   甲木二十日……」）。四库之墓取阳干：辰乙壬戊、戌辛丙戊、丑癸庚己、未丁甲己；四仲含前月余气：卯甲乙、午丙丁、酉庚辛、子壬癸。
//! - [CangGanTable::TongXing]（缺省）：今通行表，十二支所藏逐支同明崇禎刻楊淙增校《新刊合併官板音義評註淵海子平》卷一
//!   《又地支藏遁歌》（「子宮癸水在其中，丑癸辛金己土同，寅宮甲木兼丙戊……亥藏壬甲是真踪」）；寅甲丙戊又见《神峰通考》卷四
//!   《十二支中所藏法》「寅藏甲丙戊木火」、《星平會海》（增補本卷六、萬曆積善堂本卷五）「寅宮戊甲丙」。寅、辰合《子平真詮》（《論用神變化》「以寅論，甲為本主……
//!   丙其長生……戊亦長生」；《論雜氣如何取用》「辰本藏戊，而又為水庫，為乙餘氣」），真詮取格依此，故作缺省。
//!
//! 人元司令（某支之月逐日何干用事）只有《三命通會》首段有完整日数（[si_ling]）；同卷所列逐月小注表（「正月寅，己土七日丙火五日甲木十八日」
//! 等）日数多不足或溢出三十日、所藏之干亦异，醉醒子已驳「淵源淵海……以幾日為限」之说，只录于文档，不实现。

use crate::math::real::{real, Real};
#[cfg(test)]
use super::{gan_wuxing, zhi_wuxing};

/// 藏干表之家
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CangGanTable {
    /// 《淵海子平》卷一歌诀
    YuanHai,
    /// 《三命通會》卷二《論人元司事》
    SanMing,
    /// 今通行表（楊淙本《淵海》七言藏遁歌；寅、辰合《子平真詮》，四库合《神峰通考》）：缺省
    #[default]
    TongXing,
}

const YUAN_HAI: [&[u8]; 12] = [
    &[9],       // 子 癸
    &[9, 5, 7], // 丑 癸己辛
    &[0, 2],    // 寅 甲丙
    &[1],       // 卯 乙
    &[9, 4, 1], // 辰 癸戊乙
    &[2, 4, 6], // 巳 丙戊庚
    &[3, 5],    // 午 丁己
    &[1, 5, 3], // 未 乙己丁
    &[6, 8, 4], // 申 庚壬戊
    &[7],       // 酉 辛
    &[7, 3, 4], // 戌 辛丁戊
    &[8, 0],    // 亥 壬甲
];

/// 《三命通會》人元司事：(干, 日数)，按用事先后（月初在前）
const SAN_MING: [&[(u8, u8)]; 12] = [
    &[(8, 7), (9, 23)],          // 子 壬七 癸二十三
    &[(9, 7), (6, 5), (5, 18)],  // 丑 癸七 庚墓庫五 己十八
    &[(4, 5), (2, 5), (0, 20)],  // 寅 艮土五 丙長生五 甲二十
    &[(0, 7), (1, 23)],          // 卯 甲七 乙二十三
    &[(1, 7), (8, 5), (4, 18)],  // 辰 乙七 壬墓庫五 戊十八
    &[(4, 7), (6, 5), (2, 18)],  // 巳 戊七 庚長生五 丙十八
    &[(2, 7), (3, 23)],          // 午 丙七 丁二十三
    &[(3, 7), (0, 5), (5, 18)],  // 未 丁七 甲墓庫五 己十八
    &[(4, 5), (8, 5), (6, 20)],  // 申 坤土五 壬長生五 庚二十
    &[(6, 7), (7, 23)],          // 酉 庚七 辛二十三
    &[(7, 7), (2, 5), (4, 18)],  // 戌 辛七 丙墓庫五 戊十八
    &[(4, 5), (0, 5), (8, 20)],  // 亥 戊五 甲長生五 壬二十
];

const TONG_XING: [&[u8]; 12] = [
    &[9],       // 子 癸
    &[5, 9, 7], // 丑 己癸辛
    &[0, 2, 4], // 寅 甲丙戊
    &[1],       // 卯 乙
    &[4, 1, 9], // 辰 戊乙癸
    &[2, 6, 4], // 巳 丙庚戊
    &[3, 5],    // 午 丁己
    &[5, 3, 1], // 未 己丁乙
    &[6, 8, 4], // 申 庚壬戊
    &[7],       // 酉 辛
    &[4, 7, 3], // 戌 戊辛丁
    &[8, 0],    // 亥 壬甲
];

/// 本气之干：与支同五行；寅申巳亥、子午卯酉取同阴阳者（寅甲、巳丙、申庚、亥壬、子癸、午丁、卯乙、酉辛），四库取土（辰戌戊、丑未己）
pub fn ben_qi(zhi: u8) -> u8 {
    [9, 5, 0, 1, 4, 2, 3, 5, 6, 7, 4, 8][(zhi % 12) as usize]
}

/// 某支所藏之干，本气在前，余按原书次序
pub fn cang_gan(zhi: u8, table: CangGanTable) -> Vec<u8> {
    let raw: Vec<u8> = match table {
        CangGanTable::YuanHai => YUAN_HAI[(zhi % 12) as usize].to_vec(),
        CangGanTable::TongXing => TONG_XING[(zhi % 12) as usize].to_vec(),
        CangGanTable::SanMing => SAN_MING[(zhi % 12) as usize].iter().rev().map(|x| x.0).collect(),
    };
    let b = ben_qi(zhi);
    let mut v = vec![b];
    v.extend(raw.into_iter().filter(|&g| g != b));
    v
}

/// 人元司令日数之家
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SiLingTable {
    /// 《三命通會》卷二《論人元司事》首段，每月三十日：缺省
    #[default]
    SanMing,
    /// 陈素庵《命理約言》卷一《看月令法》所列「舊例」（精選本书页一三—一四），日下有分（十分一日）、錢，每支共约三十一日。
    /// 陈氏列而驳之：「各干分日，萬不可拘矣」。申月首段原作「戊己七日二分一錢」，此以戊计
    JiuLi,
}

/// 《命理約言》舊例：(干, 日数 × 100)，按用事先后
const JIU_LI: [&[(u8, u16)]; 12] = [
    &[(8, 1035), (9, 2065)],             // 子 壬十日三分半 癸二十日六分半
    &[(9, 930), (7, 310), (5, 1860)],    // 丑 癸九日三分 辛三日一分 己十八日六分
    &[(5, 725), (2, 735), (0, 1650)],    // 寅 己七日二分半 丙七日三分半 甲十六日五分
    &[(0, 1031), (1, 2065)],             // 卯 甲十日三分一錢 乙二十日六分半
    &[(1, 930), (9, 310), (4, 1860)],    // 辰 乙九日三分 癸三日一分 戊十八日六分
    &[(4, 725), (6, 725), (2, 1655)],    // 巳 戊七日二分半 庚七日二分半 丙十六日五分半
    &[(2, 1035), (5, 930), (3, 1135)],   // 午 丙十日三分半 己九日三分 丁十一日三分半
    &[(3, 930), (1, 310), (5, 1860)],    // 未 丁九日三分 乙三日一分 己十八日六分
    &[(4, 721), (8, 725), (6, 1630)],    // 申 戊己七日二分一錢 壬七日二分半 庚十六日三分
    &[(6, 1035), (7, 2065)],             // 酉 庚十日三分半 辛二十日六分半
    &[(7, 930), (3, 310), (4, 1860)],    // 戌 辛九日三分 丁三日一分 戊十八日六分
    &[(4, 735), (0, 725), (8, 1650)],    // 亥 戊七日三分半 甲七日二分半 壬十六日五分
];

/// 人元司令：某支之月、交节后第 `days` 日（0 起，可带小数）何干用事；过末段仍归本气。
pub fn si_ling_with(month_zhi: u8, days: Real, table: SiLingTable) -> u8 {
    let segs: Vec<(u8, i32)> = match table {
        SiLingTable::SanMing => SAN_MING[(month_zhi % 12) as usize].iter().map(|&(g, d)| (g, d as i32 * 100)).collect(),
        SiLingTable::JiuLi => JIU_LI[(month_zhi % 12) as usize].iter().map(|&(g, d)| (g, d as i32)).collect(),
    };
    let mut acc = 0i32;
    for (g, d) in segs {
        acc += d;
        if days * real(100.0) < real(acc) {
            return g;
        }
    }
    ben_qi(month_zhi)
}

/// 人元司令，《三命通會》日数
pub fn si_ling(month_zhi: u8, days: Real) -> u8 {
    si_ling_with(month_zhi, days, SiLingTable::SanMing)
}

/// 人元司令的分段（干, 起日, 止日），供文档与网页列表
pub fn si_ling_segments(month_zhi: u8) -> Vec<(u8, u8, u8)> {
    let mut acc = 0u8;
    SAN_MING[(month_zhi % 12) as usize]
        .iter()
        .map(|&(g, d)| {
            let s = (g, acc, acc + d);
            acc += d;
            s
        })
        .collect()
}

/// 内部一致：本气与支同五行
#[cfg(test)]
fn ben_qi_consistent(z: u8) -> bool {
    gan_wuxing(ben_qi(z)) == zhi_wuxing(z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::folk::ziping::GAN;

    fn names(z: u8, t: CangGanTable) -> String {
        cang_gan(z, t).iter().map(|&g| GAN[g as usize]).collect()
    }

    #[test]
    fn tables() {
        for z in 0..12 {
            assert!(ben_qi_consistent(z), "本气 {z}");
            assert_eq!(cang_gan(z, CangGanTable::TongXing)[0], ben_qi(z));
        }
        // 淵海：寅只甲丙；三命：寅戊丙甲（艮土）；通行：寅甲丙戊
        assert_eq!(names(2, CangGanTable::YuanHai), "甲丙");
        assert_eq!(names(2, CangGanTable::SanMing), "甲丙戊");
        assert_eq!(names(2, CangGanTable::TongXing), "甲丙戊");
        // 四库：三命取阳干之墓
        assert_eq!(names(4, CangGanTable::SanMing), "戊壬乙");
        assert_eq!(names(4, CangGanTable::TongXing), "戊乙癸");
        assert_eq!(names(4, CangGanTable::YuanHai), "戊癸乙");
        assert_eq!(names(10, CangGanTable::SanMing), "戊丙辛");
        assert_eq!(names(1, CangGanTable::SanMing), "己庚癸");
        assert_eq!(names(7, CangGanTable::SanMing), "己甲丁");
        // 子午卯酉：三命含前月余气
        assert_eq!(names(3, CangGanTable::SanMing), "乙甲");
        assert_eq!(names(3, CangGanTable::TongXing), "乙");
        assert_eq!(names(6, CangGanTable::YuanHai), "丁己");
    }

    #[test]
    fn si_ling_days() {
        // 每月三十日
        for z in 0..12u8 {
            assert_eq!(si_ling_segments(z).last().unwrap().2, 30, "{z}");
        }
        // 寅月：艮土五日、丙火五日、甲木二十日
        assert_eq!(si_ling(2, real(0.0)), 4);
        assert_eq!(si_ling(2, real(4.9)), 4);
        assert_eq!(si_ling(2, real(5.0)), 2);
        assert_eq!(si_ling(2, real(10.0)), 0);
        assert_eq!(si_ling(2, real(31.0)), 0);
        // 丑月：癸七、庚五、己十八
        assert_eq!(si_ling(1, real(7.5)), 6);
        assert_eq!(si_ling(1, real(12.0)), 5);
        // 舊例：寅月己七日二分半、丙七日三分半；每支约三十一日
        assert_eq!(si_ling_with(2, real(7.0), SiLingTable::JiuLi), 5);
        assert_eq!(si_ling_with(2, real(8.0), SiLingTable::JiuLi), 2);
        for z in 0..12 {
            let t: u16 = JIU_LI[z].iter().map(|x| x.1).sum();
            assert!((3076..=3110).contains(&t), "{z} {t}");
        }
    }
}
