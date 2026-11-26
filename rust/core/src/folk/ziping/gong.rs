//! 胎元、命宫、身宫、胎息、变（基础层，只依赖干支）。起例皆据《淵海子平》卷一《諸家起例》（明萬曆二十八年喬山堂本书影）
//! 与《三命通會》卷二《論胎元》《論坐命宫》。生前三百日之胎元要用节气，在 [super::pan]。

use super::{gan_of, gz_of, zhi_of};

/// 胎元（月前四位）：《淵海子平》卷一《論起胎法》「其法但從本生月前四位是。如己巳月，則前申上是胎，却數退午一位于未上，
/// 將生月天干己字與起，己未數至庚申是胎也」——月干进一、月支进三。
pub fn tai_yuan(month_gz: u8) -> u8 {
    gz_of((gan_of(month_gz) + 1) % 10, (zhi_of(month_gz) + 3) % 12)
}

/// 胎息：《淵海子平》卷一《論起息法》「取日上天干合處，地支合處是。假如甲子日，取天干甲與己合，又取地支子與丑合是息」
pub fn tai_xi(day_gz: u8) -> u8 {
    he(day_gz)
}

/// 变：《淵海子平》卷一《論起變法》「取時上天干合處地支合處是。假如丙寅時，取天干丙與辛合，地支寅與亥合，即辛亥是變也」
pub fn bian(hour_gz: u8) -> u8 {
    he(hour_gz)
}

/// 干五合、支六合（子丑、寅亥、卯戌、辰酉、巳申、午未）
fn he(gz: u8) -> u8 {
    gz_of((gan_of(gz) + 5) % 10, (13 - zhi_of(gz)) % 12)
}

/// 命宫所用之「生月」
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MingGongMonth {
    /// 以节定月（与月柱同）：《三命通會》《淵海子平》皆只言「本生月」，子平生月以节为界（《淵海》卷一《論起大運法》）：缺省
    #[default]
    Jie,
    /// 以中气定月（过中气作下月）：今人一说，原典未见
    ZhongQi,
}

/// 命宫之支：《淵海子平》卷一《論安身命法》「每從子上起正月，逆數至本生月止，從生月上起生時，順數遇卯即安命也」；
/// 《三命通會》卷二《論坐命宫》同（「天輪轉出地輪上，卯上分明是命宫」）。
/// `month_no` 正月 = 1 … 十二月 = 12；`hour_zhi` 子 = 0。即命宫 = 4 − 月 − 时 (mod 12)。
pub fn ming_gong_zhi(month_no: u8, hour_zhi: u8) -> u8 {
    (4 - month_no as i32 - hour_zhi as i32).rem_euclid(12) as u8
}

/// 命宫干支：干由年干五虎遁（《三命通會》「仍隨甲子年起，亦如起月之法」；《淵海》卷一《論起限天干法》「將甲己起丙寅數至子上是丙子」）
pub fn ming_gong(year_gz: u8, month_no: u8, hour_zhi: u8) -> u8 {
    let z = ming_gong_zhi(month_no, hour_zhi);
    gz_of(wu_hu_gan(gan_of(year_gz), z), z)
}

/// 五虎遁：年干 `year_gan` 之年，支 `zhi` 之干（寅起）
pub fn wu_hu_gan(year_gan: u8, zhi: u8) -> u8 {
    let yin = (year_gan * 2 + 2) % 10;
    (yin + (zhi + 10) % 12) % 10
}

/// 身宫：《淵海子平》卷一同节「再數至酉即安身也」；《神峰通考》卷一「原取逢卯安命，逢酉安身」（维基文库录文）。
/// 依此读法，所起之时数到酉之处为身宫，恒在命宫对冲。干亦由年干五虎遁。
pub fn shen_gong(year_gz: u8, month_no: u8, hour_zhi: u8) -> u8 {
    let z = (ming_gong_zhi(month_no, hour_zhi) + 6) % 12;
    gz_of(wu_hu_gan(gan_of(year_gz), z), z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::folk::ziping::{gz_name, parse_gz};

    fn g(s: &str) -> u8 {
        parse_gz(s).unwrap()
    }

    #[test]
    fn yuan_hai_examples() {
        // 淵海：己巳月胎庚申；甲子日息己丑；丙寅时变辛亥
        assert_eq!(gz_name(tai_yuan(g("己巳"))), "庚申");
        assert_eq!(gz_name(tai_xi(g("甲子"))), "己丑");
        assert_eq!(gz_name(bian(g("丙寅"))), "辛亥");
        // 淵海：四月寅时，正月子、二月亥、三月戌、四月酉，酉上起寅时，顺数至戌上卯，即戌宫坐命
        assert_eq!(ming_gong_zhi(4, 2), 10);
        // 三命：甲子年三月戌时，命坐卯宫，甲己之年丙作首，乃丁卯宫
        assert_eq!(gz_name(ming_gong(g("甲子"), 3, 10)), "丁卯");
        // 淵海《論起限天干法》：甲生人子上安命，甲己起丙寅数至子上是丙子
        assert_eq!(wu_hu_gan(0, 0), 2);
    }

    #[test]
    fn san_ming_tai_yuan_cases() {
        // 《三命通會》卷五、卷六所记带胎之命：章子厚丁亥戊子丁未壬寅己卯胎；李廷相辛丑甲午丙申戊戌乙酉胎；
        // 何洛文丙申辛丑壬子辛亥壬辰胎；許運使癸巳癸亥辛亥壬辰甲寅胎
        for (m, t) in [("戊子", "己卯"), ("甲午", "乙酉"), ("辛丑", "壬辰"), ("癸亥", "甲寅")] {
            assert_eq!(gz_name(tai_yuan(g(m))), t);
        }
    }

    #[test]
    fn shen_opposite() {
        for m in 1..=12 {
            for h in 0..12 {
                assert_eq!((ming_gong_zhi(m, h) + 6) % 12, zhi_of(shen_gong(0, m, h)));
            }
        }
    }
}
