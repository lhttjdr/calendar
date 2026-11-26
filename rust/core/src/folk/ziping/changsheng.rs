//! 十二长生（基础层）：长生、沐浴、冠带、临官、帝旺、衰、病、死、墓、绝、胎、养。
//!
//! 两说：
//! - [ChangShengRule::YangShunYinNi]（缺省）：阳生阴死、阳顺阴逆。《三命通會》卷二《論天干隂陽生死》引賦「陽生陰死，陽死陰生，
//!   循環逆順，變化見矣」，逐干说甲生亥死午、乙生午死亥……，篇末「看命者當以前説為是」；《子平真詮·論陰陽生死》「陽主聚以進為進，
//!   故主順；陰主散以退為退，故主逆。此長生沐浴等項，所以有陽順陰逆之殊也」。
//! - [ChangShengRule::YinYangTongSheng]：阴阳同生同死、皆顺行（「古人只有四大長生之説」；同篇引《廣錄》「甲乙一木而分隂陽……
//!   皆同死同生」、蔡氏「甲不必主乙不必生」）。三命录此说而不从。
//!
//! 土之长生（[TuChangSheng]）：《三命通會》同篇「戊土生於寅……巳土生於酉明矣」，土随火（缺省）；又录「陰陽家之説土生於申」
//! （「水土原不相離」），即土随水。

/// 十二长生名
pub const STAGES: [&str; 12] = ["長生", "沐浴", "冠帶", "臨官", "帝旺", "衰", "病", "死", "墓", "絕", "胎", "養"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChangShengRule {
    /// 阳生阴死、阳顺阴逆（《三命通會》卷二、《子平真詮》）：缺省
    #[default]
    YangShunYinNi,
    /// 阴阳同生、皆顺行（四大长生，《三命通會》所录而不从）
    YinYangTongSheng,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TuChangSheng {
    /// 土随火：戊生寅（阴阳同生时己亦寅；阳顺阴逆时己生酉）。《三命通會》卷二：缺省
    #[default]
    SuiHuo,
    /// 土随水：戊生申（阳顺阴逆时己生卯）。「陰陽家之説土生於申」
    SuiShui,
}

/// 阳干长生之支：甲亥、丙寅、戊寅（随火）或申（随水）、庚巳、壬申
fn yang_sheng(gan_yang: u8, tu: TuChangSheng) -> u8 {
    match gan_yang {
        0 => 11,
        2 => 2,
        4 => match tu {
            TuChangSheng::SuiHuo => 2,
            TuChangSheng::SuiShui => 8,
        },
        6 => 5,
        _ => 8,
    }
}

/// 长生之支
pub fn sheng_zhi(gan: u8, rule: ChangShengRule, tu: TuChangSheng) -> u8 {
    let yang = gan - gan % 2;
    let ys = yang_sheng(yang, tu);
    if gan % 2 == 0 || rule == ChangShengRule::YinYangTongSheng {
        ys
    } else {
        // 阴干长生 = 同类阳干之死地（阳死阴生）：阳死在长生后七位（亥生午死）
        (ys + 7) % 12
    }
}

/// 某干临某支为十二长生第几位（0 长生 … 11 养）
pub fn chang_sheng(gan: u8, zhi: u8, rule: ChangShengRule, tu: TuChangSheng) -> u8 {
    let s = sheng_zhi(gan, rule, tu) as i32;
    let z = zhi as i32;
    let forward = gan % 2 == 0 || rule == ChangShengRule::YinYangTongSheng;
    (if forward { z - s } else { s - z }).rem_euclid(12) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::folk::ziping::{GAN, ZHI};

    fn at(g: u8, stage: usize, rule: ChangShengRule) -> &'static str {
        let z = (0..12).find(|&z| chang_sheng(g, z, rule, TuChangSheng::SuiHuo) as usize == stage).unwrap();
        ZHI[z as usize]
    }

    #[test]
    fn san_ming_sheng_si() {
        let r = ChangShengRule::YangShunYinNi;
        // 《三命通會》卷二逐干生死：甲生亥死午，乙生午死亥，丙生寅死酉，丁生酉死寅，戊生寅死酉，己生酉死寅，
        // 庚生巳死子，辛生子死巳，壬生申死卯，癸生卯死申
        let exp = [("亥", "午"), ("午", "亥"), ("寅", "酉"), ("酉", "寅"), ("寅", "酉"), ("酉", "寅"), ("巳", "子"), ("子", "巳"), ("申", "卯"), ("卯", "申")];
        for g in 0..10u8 {
            assert_eq!((at(g, 0, r), at(g, 7, r)), exp[g as usize], "{}", GAN[g as usize]);
        }
        // 臨官即祿：甲寅乙卯丙戊巳丁己午庚申辛酉壬亥癸子
        let lu: String = (0..10).map(|g| at(g, 3, r)).collect();
        assert_eq!(lu, "寅卯巳午巳午申酉亥子");
        // 帝旺：阳干之旺为刃（甲卯丙午戊午庚酉壬子）
        let wang: String = (0..10).step_by(2).map(|g| at(g, 4, r)).collect();
        assert_eq!(wang, "卯午午酉子");
    }

    #[test]
    fn tong_sheng_and_tu() {
        let r = ChangShengRule::YinYangTongSheng;
        for g in (1..10).step_by(2) {
            for z in 0..12 {
                assert_eq!(
                    chang_sheng(g, z, r, TuChangSheng::SuiHuo),
                    chang_sheng(g - 1, z, r, TuChangSheng::SuiHuo)
                );
            }
        }
        // 土随水：戊生申、己生卯
        assert_eq!(sheng_zhi(4, ChangShengRule::YangShunYinNi, TuChangSheng::SuiShui), 8);
        assert_eq!(sheng_zhi(5, ChangShengRule::YangShunYinNi, TuChangSheng::SuiShui), 3);
        assert_eq!(STAGES[chang_sheng(4, 0, ChangShengRule::YangShunYinNi, TuChangSheng::SuiShui) as usize], "帝旺");
    }
}
