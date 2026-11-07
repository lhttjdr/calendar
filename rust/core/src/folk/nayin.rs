//! 六十甲子纳音。
//!
//! 每两个相连的干支（甲子乙丑、丙寅丁卯……）同一纳音，共三十个名目。五行顺序与名目据
//! 《三命通會》卷一《論納音取象》（四库本），用字从原书（路傍土、劒鋒金、白鑞金、井泉水、金泊金）。
//! 三种推法（沈括隔八相生、扬雄《太玄》起数与《考原》大衍数、朱子五音所生）推出同一张表，
//! 见《欽定協紀辨方書》卷一「納音」「納音干支起數合五行」，测试逐一复现。

/// 五行
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WuXing {
    Jin,
    Mu,
    Shui,
    Huo,
    Tu,
}

impl WuXing {
    pub fn name(self) -> &'static str {
        match self {
            WuXing::Jin => "金",
            WuXing::Mu => "木",
            WuXing::Shui => "水",
            WuXing::Huo => "火",
            WuXing::Tu => "土",
        }
    }
}

/// 三十纳音名目，下标 = 干支序 / 2（甲子乙丑 = 0）。
pub const NAYIN_NAMES: [&str; 30] = [
    "海中金", "爐中火", "大林木", "路傍土", "劒鋒金", "山頭火", // 甲子—乙亥
    "澗下水", "城頭土", "白鑞金", "楊柳木", "井泉水", "屋上土", // 丙子—丁亥
    "霹靂火", "松柏木", "長流水", "沙中金", "山下火", "平地木", // 戊子—己亥
    "壁上土", "金泊金", "覆燈火", "天河水", "大驛土", "釵釧金", // 庚子—辛亥
    "桑柘木", "大溪水", "沙中土", "天上火", "石榴木", "大海水", // 壬子—癸亥
];

/// 干支序（甲子 = 0）的纳音名目
pub fn nayin_name(gan_zhi_index: u8) -> &'static str {
    NAYIN_NAMES[(gan_zhi_index % 60 / 2) as usize]
}

/// 干支序的纳音五行（名目末字）
pub fn nayin_wuxing(gan_zhi_index: u8) -> WuXing {
    match NAYIN_NAMES[(gan_zhi_index % 60 / 2) as usize].chars().last() {
        Some('金') => WuXing::Jin,
        Some('木') => WuXing::Mu,
        Some('水') => WuXing::Shui,
        Some('火') => WuXing::Huo,
        _ => WuXing::Tu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 扬雄《太玄》数：甲己子午九、乙庚丑未八、丙辛寅申七、丁壬卯酉六、戊癸辰戌五、巳亥四。
    fn xuan_shu_gan(g: u8) -> i32 {
        [9, 8, 7, 6, 5, 9, 8, 7, 6, 5][g as usize]
    }
    fn xuan_shu_zhi(z: u8) -> i32 {
        [9, 8, 7, 6, 5, 4, 9, 8, 7, 6, 5, 4][z as usize]
    }

    /// 《考原》大衍数法：两干两支之数合计，以四十九减之，余数去十，一六水、二七火、三八木、四九金、五十土，
    /// 纳音取其所生之五行。
    #[test]
    fn dayan_method_reproduces_table() {
        for k in 0..30u8 {
            let (a, b) = (2 * k, 2 * k + 1);
            let sum = xuan_shu_gan(a % 10) + xuan_shu_zhi(a % 12) + xuan_shu_gan(b % 10) + xuan_shu_zhi(b % 12);
            let r = (49 - sum).rem_euclid(10);
            let base = match r {
                1 | 6 => WuXing::Shui,
                2 | 7 => WuXing::Huo,
                3 | 8 => WuXing::Mu,
                4 | 9 => WuXing::Jin,
                _ => WuXing::Tu,
            };
            let sheng = match base {
                WuXing::Shui => WuXing::Mu,
                WuXing::Mu => WuXing::Huo,
                WuXing::Huo => WuXing::Tu,
                WuXing::Tu => WuXing::Jin,
                WuXing::Jin => WuXing::Shui,
            };
            assert_eq!(sheng, nayin_wuxing(a), "{}", NAYIN_NAMES[k as usize]);
        }
    }

    /// 沈括《夢溪筆談》隔八相生：甲子金之仲，娶乙丑，隔八生壬申（孟），壬申娶癸酉，隔八生庚辰（季），
    /// 金三元终传火（戊子），次木、水、土；甲午起又如甲子之法。
    #[test]
    fn shen_kuo_method_reproduces_table() {
        let order = [WuXing::Jin, WuXing::Huo, WuXing::Mu, WuXing::Shui, WuXing::Tu];
        for half in 0..2u8 {
            let mut gz = 30 * half; // 甲子 或 甲午
            for &wx in &order {
                for _ in 0..3 {
                    assert_eq!(nayin_wuxing(gz), wx);
                    assert_eq!(nayin_wuxing(gz + 1), wx, "同位娶妻");
                    gz = (gz + 8) % 60; // 隔八生子
                }
            }
        }
    }

    /// 朱子五音法：宫商角徵羽依次纳甲丙戊庚壬，系以五子，纳音为本音所生；宫土生金、商金生水、角木生火、
    /// 徵火生土、羽水生木。次二起商、次三起角，以下各推。
    #[test]
    fn zhuxi_method_reproduces_table() {
        let yin_sheng = [WuXing::Jin, WuXing::Shui, WuXing::Huo, WuXing::Tu, WuXing::Mu]; // 宫商角徵羽所生
        for round in 0..6u8 {
            // 支：子丑、寅卯、辰巳、午未、申酉、戌亥
            for (i, gan) in [0u8, 2, 4, 6, 8].iter().enumerate() {
                let zhi = 2 * round;
                let gz = ((6 * *gan as i32 - 5 * zhi as i32).rem_euclid(60)) as u8;
                let yin = (i + (round % 3) as usize) % 5; // 次四「復以宫商角徵羽」，自午起重新从宫
                assert_eq!(nayin_wuxing(gz), yin_sheng[yin], "干序{gan} 支序{zhi}");
            }
        }
    }

    #[test]
    fn spot_values() {
        assert_eq!(nayin_name(0), "海中金");
        assert_eq!(nayin_name(31), "沙中金"); // 乙未
        assert_eq!(nayin_name(36), "壁上土"); // 庚子
        assert_eq!(nayin_name(59), "大海水");
    }
}
