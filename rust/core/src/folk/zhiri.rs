//! 二十八宿值日与七曜。
//!
//! 每日一宿，二十八日一周，从角到轸顺行，与干支纪日并行，时宪书逐日载之。起点由两条原典条件唯一确定：
//! - 《欽定協紀辨方書》卷一引《考原》：「一元甲子日起虚……二元甲子起奎，三元甲子起畢，四元甲子起鬼，
//!   五元甲子起翼，六元甲子起氐，七元甲子起箕」，七元四百二十日一周；
//! - 同条引《宿曜經》：「虚昴星房屬日，危畢張心屬月，室觜翼尾屬火，壁參軫箕屬水，奎井角斗屬木，
//!   婁鬼亢牛屬金，胃柳氐女屬土」；卷三十六：「密日者乃房虚星昴四宿，七政屬日，西語曰密」。
//!   七曜即七日一周的星期，日曜（密日）为星期日。
//!
//! 甲子日为虚宿且为星期日，在 JDN 中每 420 日出现一次，由此得 宿序 = (JDN + 11) mod 28。
//! 以乾隆三十年（1765）时宪书二月下半逐日所载核对（乙未昴、庚子鬼、乙巳軫等）全合。

/// 二十八宿，角 = 0
pub const XIU_28: [&str; 28] = [
    "角", "亢", "氐", "房", "心", "尾", "箕", // 东方
    "斗", "牛", "女", "虚", "危", "室", "壁", // 北方
    "奎", "婁", "胃", "昴", "畢", "觜", "參", // 西方
    "井", "鬼", "柳", "星", "張", "翼", "軫", // 南方
];

/// 七曜，下标 = 星期（日 = 0）
pub const QI_YAO: [&str; 7] = ["日", "月", "火", "水", "木", "金", "土"];

/// 儒略日数（民用日）对应的值日宿序（角 = 0）
pub fn xiu_index(jdn: i64) -> u8 {
    (jdn + 11).rem_euclid(28) as u8
}

/// 值日宿名
pub fn xiu_name(jdn: i64) -> &'static str {
    XIU_28[xiu_index(jdn) as usize]
}

/// 七曜序（日 = 0，即星期日）
pub fn qi_yao_index(jdn: i64) -> u8 {
    (jdn + 1).rem_euclid(7) as u8
}

/// 宿所属七曜（《宿曜經》，见模块文档）
pub fn xiu_qi_yao(xiu: u8) -> u8 {
    // 角(0)属木：角宿序 0 → 木(4)；宿序每进一，七曜进一（角木 亢金 氐土 房日 心月 尾火 箕水）
    ((xiu as u32 + 4) % 7) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gan_zhi::GAN_ZHI_60;

    fn gz(jdn: i64) -> &'static str {
        GAN_ZHI_60[(jdn + 49).rem_euclid(60) as usize]
    }

    #[test]
    fn qi_element_table_matches_suyao_jing() {
        let groups = [("虚昴星房", 0), ("危畢張心", 1), ("室觜翼尾", 2), ("壁參軫箕", 3), ("奎井角斗", 4), ("婁鬼亢牛", 5), ("胃柳氐女", 6)];
        for (names, yao) in groups {
            for c in names.chars() {
                let i = XIU_28.iter().position(|x| x.starts_with(c)).unwrap() as u8;
                assert_eq!(xiu_qi_yao(i), yao, "{c}");
            }
        }
    }

    /// 七元：连续七个甲子日依次值虚、奎、畢、鬼、翼、氐、箕；值日宿所属七曜即星期。
    #[test]
    fn seven_yuan_and_weekday() {
        let j0 = (2451545..2451545 + 420).find(|&j| gz(j) == "甲子" && xiu_name(j) == "虚").unwrap();
        let seq: Vec<_> = (0..8).map(|k| xiu_name(j0 + 60 * k)).collect();
        assert_eq!(seq, ["虚", "奎", "畢", "鬼", "翼", "氐", "箕", "虚"]);
        for j in 2400000..2400000 + 3000 {
            assert_eq!(xiu_qi_yao(xiu_index(j)), qi_yao_index(j));
        }
        assert_eq!(qi_yao_index(2451545), 6); // 2000-01-01 星期六
    }

    /// 乾隆三十年（1765）时宪书二月：十九日乙未金昴定、二十日丙申火畢執……二十四日庚子土鬼收、
    /// 二十九日乙巳火軫滿（台湾图书馆藏本影像）。二月初一为公历 1765-02-20。
    #[test]
    fn qianlong_30_shixianshu() {
        let day19 = crate::calendar::gregorian::Gregorian::to_julian_day(1765, 3, 10);
        let j = (day19.as_f64() + 0.5).floor() as i64;
        use crate::math::real::RealOps;
        let expect = [("乙未", "昴"), ("丙申", "畢"), ("丁酉", "觜"), ("戊戌", "參"), ("己亥", "井"), ("庚子", "鬼"), ("辛丑", "柳"), ("壬寅", "星"), ("癸卯", "張"), ("甲辰", "翼"), ("乙巳", "軫")];
        for (k, (g, x)) in expect.iter().enumerate() {
            assert_eq!(gz(j + k as i64), *g);
            assert_eq!(xiu_name(j + k as i64), *x);
        }
    }
}
