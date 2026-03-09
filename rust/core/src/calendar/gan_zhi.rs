//! 干支历：干支纪日（天干地支 60 日循环）。
//!
//! 历元：公历 2000-01-01 为 庚辰日（序 16）。日界按 0h UT，JD 取整为 floor(JD + 0.5)。

use crate::math::real::{real, RealOps, ToReal};

/// 60 干支名称（甲子=0 … 癸亥=59），。
pub const GAN_ZHI_60: [&str; 60] = [
    "甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己巳", "庚午", "辛未", "壬申", "癸酉",
    "甲戌", "乙亥", "丙子", "丁丑", "戊寅", "己卯", "庚辰", "辛巳", "壬午", "癸未",
    "甲申", "乙酉", "丙戌", "丁亥", "戊子", "己丑", "庚寅", "辛卯", "壬辰", "癸巳",
    "甲午", "乙未", "丙申", "丁酉", "戊戌", "己亥", "庚子", "辛丑", "壬寅", "癸卯",
    "甲辰", "乙巳", "丙午", "丁未", "戊申", "己酉", "庚戌", "辛亥", "壬子", "癸丑",
    "甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥",
];

/// 2000-01-01 0h UT 的「日号」= floor(JD+0.5) = 2451545，该日为庚辰（序 16）。
const EPOCH_DAY_2000_01_01: i32 = 2451545;
const EPOCH_GAN_ZHI_INDEX: i32 = 16;

/// 儒略日（0h UT）对应的干支日序（0..60）。JD 支持 Real。
#[inline]
pub fn jd_to_gan_zhi_index(jd: impl ToReal) -> u8 {
    let day = (real(jd) + real(0.5)).floor().to_i32_floor();
    ((day - EPOCH_DAY_2000_01_01 + EPOCH_GAN_ZHI_INDEX).rem_euclid(60)) as u8
}

/// 儒略日对应的干支日名称。JD 支持 Real。
#[inline]
pub fn jd_to_gan_zhi_day(jd: impl ToReal) -> &'static str {
    GAN_ZHI_60[jd_to_gan_zhi_index(jd) as usize]
}

/// 公历 (年, 月, 日) → 干支日名称。
#[inline]
pub fn gregorian_to_gan_zhi_day(year: i32, month: i32, day: i32) -> &'static str {
    let jd = crate::calendar::gregorian::Gregorian::to_julian_day(year, month, day);
    jd_to_gan_zhi_day(jd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gregorian::Gregorian;

    #[test]
    fn year_2000_jan_1_is_geng_chen() {
        let jd = Gregorian::to_julian_day(2000, 1, 1);
        let idx = jd_to_gan_zhi_index(jd);
        assert_eq!(idx, 16, "2000-01-01 应为庚辰(16)");
        assert_eq!(jd_to_gan_zhi_day(jd), "庚辰");
    }
}
