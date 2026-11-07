//! 紫白九星（年、月）。
//!
//! 据《欽定協紀辨方書》卷八「三元九星」「三元月九星入中宫」：
//! - 九星：一白、二黑、三碧、四綠、五黃、六白、七赤、八白、九紫（《黄帝遯甲經》以休門為一白……景門為九紫）。
//! - 年：「康熙二十三年甲子為上元」；「上元甲子中宫起一白，中元甲子起四綠，下元甲子起七赤，三元一百八十年而一週
//!   ……逐年逆轉，如甲子年中宫起一白，乙丑年中宫起九紫」。年按太岁，以立春交替（卷三十四「歳官交承」）。
//! - 月：「子午卯酉年為上元，正月起八白；辰戌丑未年為中元，正月起五黃；寅申巳亥年為下元，正月起二黑，
//!   逐月逆轉」。月按节（寅月为正月）。
//!
//! 日、时九星（冬至、夏至后甲子起例）协纪未收，原典待考，本模块不提供。

/// 九星名，下标 = 星数 − 1
pub const JIU_XING: [&str; 9] = ["一白", "二黑", "三碧", "四綠", "五黃", "六白", "七赤", "八白", "九紫"];

/// 康熙二十三年（1684）甲子为上元，一白入中宫
pub const SHANG_YUAN_JIAZI_YEAR: i32 = 1684;

fn wrap9(n: i32) -> u8 {
    let r = n.rem_euclid(9);
    if r == 0 { 9 } else { r as u8 }
}

/// 年九星（入中宫之星，1—9）。`ganzhi_year` 为干支年（立春换年）所在的公元年，天文纪年。
pub fn year_star(ganzhi_year: i32) -> u8 {
    wrap9(1 - (ganzhi_year - SHANG_YUAN_JIAZI_YEAR))
}

/// 月九星（1—9）。`year_zhi`：年支序（子 = 0）；`month_ord`：节月序，寅月（正月）= 0 … 丑月 = 11。
pub fn month_star(year_zhi: u8, month_ord: u8) -> u8 {
    let start = match year_zhi % 12 {
        0 | 6 | 3 | 9 => 8,  // 子午卯酉：八白
        4 | 10 | 1 | 7 => 5, // 辰戌丑未：五黃
        _ => 2,              // 寅申巳亥：二黑
    };
    wrap9(start - month_ord as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_yuan_years() {
        assert_eq!(year_star(1684), 1); // 上元甲子 一白
        assert_eq!(year_star(1685), 9); // 乙丑 九紫
        assert_eq!(year_star(1744), 4); // 中元甲子 四綠
        assert_eq!(year_star(1804), 7); // 下元甲子 七赤
        assert_eq!(year_star(1864), 1); // 一百八十年复为上元
        for y in 1500..2100 {
            assert_eq!(year_star(y), year_star(y + 180));
        }
    }

    #[test]
    fn month_starts() {
        assert_eq!(month_star(0, 0), 8); // 子年正月八白
        assert_eq!(month_star(4, 0), 5); // 辰年正月五黃
        assert_eq!(month_star(2, 0), 2); // 寅年正月二黑
        assert_eq!(month_star(0, 1), 7); // 逐月逆转
        // 三年一循环、月星连续：子年丑月之后接丑年寅月
        assert_eq!(wrap9(month_star(0, 11) as i32 - 1), month_star(1, 0));
    }
}
