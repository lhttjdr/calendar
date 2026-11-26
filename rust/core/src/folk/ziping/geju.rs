//! 月令取格（派生层）：只做《子平真詮》取格中可以机械判定的一条——以日干配月令所藏之干，看何者透出天干。
//!
//! 依据（《子平真詮》，上海世界圖書館排印本，胡焜倬乾隆四十一年序）：
//! - 《論用神》：「八字用神，專求月令，以日干配月令地支，而生剋不同，格局分焉」。
//! - 《論用神變化》：「月令所藏不一，而用神遂有變化。如十二支中，除子午卯酉外，餘皆有藏……即以寅論，甲為本主……丙其長生……
//!   戊亦長生……假使寅月為提，不透甲而透丙，則……得以作主」。
//! - 《論雜氣如何取用》：「辰本藏戊，而又為水庫，為乙餘氣……透干會支，取其清者用之……如甲生三月，透戊則用偏財，透癸則用正印，
//!   透乙則用月劫」；「一透則一用，兼透則兼用」。
//! - 《論建祿月劫》（目录）与《論用神成敗救應》前文：「是建祿月劫之格，非用而即用神也」——月令为日干之禄、劫时，另寻用神。
//!
//! 机械化的读法（本项目所定，书中无条文化的次序）：月令为日干之禄 → 建禄格；阳干月令为刃 → 阳刃格；阴干月令为同五行阳干之禄（如乙日寅月）
//! → 月劫格。否则：本气透出年、月、时干 → 本气之格；本气不透而他藏干透 → 取所透者（多透取表中在前者，并列其余）；皆不透 → 取本气。
//! 所透（或不透时之本气）为比劫者作月劫格（「透乙則用月劫」）。会支成局、透干之清浊、用神之成败救应、外格（从化、一行
//! 得气诸格）皆不机械判断。

use super::canggan::{ben_qi, cang_gan, CangGanTable};
use super::shensha::{lu, yang_ren, YangRenRule};
use super::shishen::{shi_shen, ShiShen};
use super::{gan_is_yang, gan_of, zhi_of, SiZhu};

/// 取格所由
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeYou {
    /// 月令本气透出天干
    BenQiTou,
    /// 本气不透，他藏干透出（《論用神變化》）
    TaGanTou,
    /// 皆不透，取本气
    BuTou,
    /// 月令为日干之禄
    JianLu,
    /// 阳干月令为刃
    YangRen,
    /// 月令为日干之劫财（含本气为比劫之四库而他干不透）
    YueJie,
}

/// 月令之格
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeJu {
    /// 格名，如「正官格」「建祿格」
    pub name: String,
    /// 取格之干（建禄、阳刃、月劫为月令本气）
    pub gan: u8,
    pub you: GeYou,
    /// 同透之他干（「兼透則兼用」）
    pub jian_tou: Vec<u8>,
}

fn ge_name(s: ShiShen) -> String {
    format!("{}格", s.name())
}

/// 月令取格
pub fn yue_ling_ge(sz: &SiZhu, table: CangGanTable) -> GeJu {
    let dg = sz.day_gan();
    let mz = zhi_of(sz.month);
    let b = ben_qi(mz);
    if mz == lu(dg) {
        return GeJu { name: "建祿格".into(), gan: b, you: GeYou::JianLu, jian_tou: vec![] };
    }
    if yang_ren(dg, YangRenRule::YangOnly) == Some(mz) {
        return GeJu { name: "陽刃格".into(), gan: b, you: GeYou::YangRen, jian_tou: vec![] };
    }
    if !gan_is_yang(dg) && mz == lu(dg - 1) {
        return GeJu { name: "月劫格".into(), gan: b, you: GeYou::YueJie, jian_tou: vec![] };
    }
    let tian: Vec<u8> = [sz.year, sz.month, sz.hour].iter().map(|&g| gan_of(g)).collect();
    let bi_jie = |g: u8| matches!(shi_shen(dg, g), ShiShen::BiJian | ShiShen::JieCai);
    let tou: Vec<u8> = cang_gan(mz, table).into_iter().filter(|g| tian.contains(g)).collect();
    if let Some(&g) = tou.first() {
        let jian_tou = tou[1..].to_vec();
        if bi_jie(g) {
            // 「透乙則用月劫」
            return GeJu { name: "月劫格".into(), gan: g, you: GeYou::YueJie, jian_tou };
        }
        let you = if g == b { GeYou::BenQiTou } else { GeYou::TaGanTou };
        return GeJu { name: ge_name(shi_shen(dg, g)), gan: g, you, jian_tou };
    }
    if bi_jie(b) {
        return GeJu { name: "月劫格".into(), gan: b, you: GeYou::YueJie, jian_tou: vec![] };
    }
    GeJu { name: ge_name(shi_shen(dg, b)), gan: b, you: GeYou::BuTou, jian_tou: vec![] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ge(s: &str) -> GeJu {
        yue_ling_ge(&SiZhu::parse(s).unwrap(), CangGanTable::TongXing)
    }

    #[test]
    fn zhen_quan_rules() {
        // 寅月为提，不透甲而透丙：辛日寅月透丙 → 正官（《論用神變化》「辛生寅月，逢丙而化格愈美」）
        let g = ge("丙寅庚寅辛酉戊子");
        assert_eq!((g.name.as_str(), g.you), ("正官格", GeYou::TaGanTou));
        // 甲生三月（辰）透戊用偏财，透癸用正印，透乙为月劫
        assert_eq!(ge("戊辰丙辰甲子甲子").name, "偏財格");
        assert_eq!(ge("癸亥丙辰甲子甲子").name, "正印格");
        assert_eq!(ge("乙亥庚辰甲子甲子").you, GeYou::YueJie);
        // 兼透
        let g = ge("戊辰丙辰甲子癸酉");
        assert_eq!((g.name.as_str(), g.jian_tou.len()), ("偏財格", 1));
        // 建禄、阳刃、月劫
        assert_eq!(ge("甲子丙寅甲子甲子").you, GeYou::JianLu);
        assert_eq!(ge("甲子丁卯甲子甲子").you, GeYou::YangRen);
        assert_eq!(ge("甲子丙寅乙丑丙子").you, GeYou::YueJie);
        // 子月癸本气不透：丙日为正官
        let g = ge("甲辰丙子丙午甲午");
        assert_eq!((g.name.as_str(), g.you), ("正官格", GeYou::BuTou));
    }
}
