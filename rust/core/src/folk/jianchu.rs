//! 建除十二值、黄道黑道十二神、百忌日。
//!
//! - 建除：《欽定協紀辨方書》卷四引《厯書》：「厯家以建除滿平定執破危成收開閉凡十二日周而復始……每月交節則疊兩值日，
//!   其法從月建上起建……如正月建寅則寅日起建，順行十二辰是也」。月建按节（交节日即属新月，故与前一日同值，
//!   即「疊兩值日」）。值 = (日支 − 月支) mod 12。
//! - 黄道黑道：同书卷七引《考原》：「一青龍、二明堂、三天刑、四朱雀、五金匱、六天德、七白虎、八玉堂、九天牢、
//!   十元武、十一司命、十二勾陳，其法則寅申青龍起子，卯酉起寅，辰戌起辰，巳亥起午，子午起申，丑未起戌，順行十二辰。
//!   月起日……日起時」。卷七按语考定「以六辰為黄道，餘則曰黑道」：司命、明堂、青龍、金匱、天德、玉堂。
//!   原书「元武」避康熙讳，即玄武。
//! - 百忌日：同书卷三十五附录「百忌日」：「甲不開倉，乙不栽植……亥不嫁娶」。协纪称百忌日，不冠彭祖之名。

/// 建除十二值，建 = 0
pub const JIAN_CHU: [&str; 12] = ["建", "除", "滿", "平", "定", "執", "破", "危", "成", "收", "開", "閉"];

/// 日支、月支（月建，按节）→ 建除值序
pub fn jian_chu(day_zhi: u8, month_zhi: u8) -> u8 {
    ((day_zhi as i32 - month_zhi as i32).rem_euclid(12)) as u8
}

/// 黄道黑道十二神（原书「元武」作玄武）
pub const HUANG_HEI_SHEN: [&str; 12] = ["青龍", "明堂", "天刑", "朱雀", "金匱", "天德", "白虎", "玉堂", "天牢", "玄武", "司命", "勾陳"];

/// 黄道六神在 [HUANG_HEI_SHEN] 中的序：青龍、明堂、金匱、天德、玉堂、司命
pub const HUANG_DAO: [u8; 6] = [0, 1, 4, 5, 7, 10];

/// 黄黑道神序。择日：`base_zhi` 为月支（月建），`zhi` 为日支；择时：`base_zhi` 为日支，`zhi` 为时支。
/// 起例「寅申青龍起子，卯酉起寅……」即青龍所在支 = 2 × base − 4 (mod 12)。
pub fn huang_hei(zhi: u8, base_zhi: u8) -> u8 {
    let qinglong = (2 * base_zhi as i32 - 4).rem_euclid(12);
    ((zhi as i32 - qinglong).rem_euclid(12)) as u8
}

pub fn is_huang_dao(shen: u8) -> bool {
    HUANG_DAO.contains(&shen)
}

/// 百忌日（《協紀》卷三十五附录），下标为日干序、日支序
pub const BAI_JI_GAN: [&str; 10] = ["甲不開倉", "乙不栽植", "丙不修竈", "丁不剃頭", "戊不受田", "己不破券", "庚不經絡", "辛不合醬", "壬不決水", "癸不詞訟"];
pub const BAI_JI_ZHI: [&str; 12] = ["子不問卜", "丑不冠帶", "寅不祭祀", "卯不穿井", "辰不哭泣", "巳不遠行", "午不苫蓋", "未不服藥", "申不安牀", "酉不會客", "戌不乞狗", "亥不嫁娶"];

#[cfg(test)]
mod tests {
    use super::*;

    /// 乾隆三十年（1765）时宪书二月（卯月，惊蛰已过、清明未到）：十九日乙未定、二十日丙申執、
    /// 二十四日庚子收、二十九日乙巳滿。
    #[test]
    fn qianlong_30_jianchu() {
        let mao = 3;
        assert_eq!(JIAN_CHU[jian_chu(7, mao) as usize], "定"); // 乙未
        assert_eq!(JIAN_CHU[jian_chu(8, mao) as usize], "執"); // 丙申
        assert_eq!(JIAN_CHU[jian_chu(0, mao) as usize], "收"); // 庚子
        assert_eq!(JIAN_CHU[jian_chu(5, mao) as usize], "滿"); // 乙巳
    }

    #[test]
    fn jiaojie_repeats_value() {
        // 交节日月支进一、日支亦进一：与前一日同值
        assert_eq!(jian_chu(5, 2), jian_chu(6, 3));
    }

    /// 起例与协纪卷七按语所举：寅月日罡……「申月……寅天刑、卯朱雀……子青龍、丑明堂」；
    /// 「午月……午司命、未勾陳、申青龍、酉明堂」；「卯月……寅青龍、卯明堂」。
    #[test]
    fn huang_hei_examples() {
        let name = |z, b| HUANG_HEI_SHEN[huang_hei(z, b) as usize];
        assert_eq!(name(0, 2), "青龍"); // 寅月子日
        assert_eq!(name(2, 8), "天刑"); // 申月寅日
        assert_eq!(name(0, 8), "青龍"); // 申月子日
        assert_eq!(name(6, 6), "司命"); // 午月午日
        assert_eq!(name(8, 6), "青龍"); // 午月申日
        assert_eq!(name(2, 3), "青龍"); // 卯月寅日
        assert_eq!(name(3, 3), "明堂");
        assert_eq!(name(8, 0), "青龍"); // 子日申时
        // 六黄道恰为 子午卯酉寅未 所值之神（协纪按语：司命即子……天德即未）
        let mut names: Vec<_> = HUANG_DAO.iter().map(|&i| HUANG_HEI_SHEN[i as usize]).collect();
        names.sort();
        let mut want = vec!["司命", "金匱", "明堂", "玉堂", "青龍", "天德"];
        want.sort();
        assert_eq!(names, want);
    }
}
