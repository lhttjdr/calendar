//! 太乙八门：直使门、布门（八门所临之宫）与三门具不具。
//!
//! - 直使（岁月日计）：金鏡卷一「推八門占嵗計法」、卷四「推三門具不具」：积数以二百四十去之，不尽以三十约之为直门数，
//!   不尽为直门所入年，命起开门，以次休生，左行八门；「假令今開元十二年甲子即開門為直使至三十一年甲午嵗即休門為直使」；
//!   「年月日時直使同此法」。积年取演纪积年（「與小遊同」，以七百二十去之与卷一积年同余），故岁计八门不随淘金歌积年而变。
//! - 直使（时计）：金鏡卷一「推八門用法」引张良：阳遁冬至甲日夜半甲子以后开门直使、丙日日中甲午以后生门、己日夜半甲子以后
//!   惊门、辛日日中甲午以后休门，「三十時移門」；阴遁夏至杜、死、伤、景同。即时计积数以一百二十去之、三十约之。
//!   王希明改以冬至、夏至加时起算（「不拘其甲丙己辛日」），作选项 [`ShiMenFa::WangXiMing`]（[`zhi_shi_shi_wang`]，须交至时刻）。
//! - 阴遁（岁月日计）：金鏡未言，取阳遁直门之冲（开↔杜、休↔景、生↔死、伤↔惊），与张良阴阳二遁四门互冲、淘金歌
//!   「陰遁皆以杜門為直事」相应（**未核**）。
//! - 布门：[`BuMenFa`]。
//! - 门具：[`MenJuFa`]。

use super::{pos_gong, Dun, TaiyiJu};

/// 八门（自开门起之序）
pub const MEN8: [&str; 8] = ["開門", "休門", "生門", "傷門", "杜門", "景門", "死門", "驚門"];
/// 八门顺行所经之宫：乾一、坎八、艮三、震四、巽九、離二、坤七、兌六（淘金歌「二遁皆順行八三四」）；
/// 亦为八门定向（金鏡卷二「推八門所主法」开乾、休坎、生艮、伤震、杜巽、景離、死坤、惊兌）
pub const MEN_GONG: [u8; 8] = [1, 8, 3, 4, 9, 2, 7, 6];

/// 直使门与入门第几（年、月、日、时）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZhiShi {
    /// 门序 0—7（[`MEN8`]）
    pub men: u8,
    /// 入门第几，自 1 起
    pub ru: u8,
}

/// 岁月日计直使：积数（岁计为演纪积年 [`super::ji_nian`]`(·, JinJing)`，月计积月，日计积日）以二百四十去之、三十约之
pub fn zhi_shi(ji: i64, dun: Dun) -> ZhiShi {
    let k = (ji - 1).rem_euclid(240);
    let men = (k / 30) as u8;
    ZhiShi { men: if dun == Dun::Yin { (men + 4) % 8 } else { men }, ru: (k % 30 + 1) as u8 }
}

/// 时计直使（张良）：时计积数（[`super::shi_ji_shu`]）以一百二十去之、三十约之；阳遁开生惊休、阴遁杜死伤景
pub fn zhi_shi_shi(shi_ji_shu: i64, dun: Dun) -> ZhiShi {
    let k = (shi_ji_shu - 1).rem_euclid(120);
    let seq: [u8; 4] = match dun {
        Dun::Yang => [0, 2, 7, 1], // 开、生、惊、休
        Dun::Yin => [4, 6, 3, 5],  // 杜、死、伤、景
    };
    ZhiShi { men: seq[(k / 30) as usize], ru: (k % 30 + 1) as u8 }
}

/// 时计直使之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiMenFa {
    /// 张良：时计积数以一百二十去之、三十约之（门界恒在甲、丙、己、辛日之夜半、日中）。缺省
    ZhangLiang,
    /// 王希明：「臣今太乙冬至夏至初臨之時門則隨數起時盡則移不拘其甲丙己辛日也」；「推冬至加時所直門法」：
    /// 「置天正時實以二百四十去之不盡以一百二十去之不盡以三十約之為門數命起開門以次而行即天正冬至加時所直門也」；
    /// 夏至同（「以三十約之為門數」下阙数字，阴遁命起杜门**未核**）。见 [`zhi_shi_shi_wang`]
    WangXiMing,
}

/// 王希明二至加时起门：`zhi_shi_shu` 为交至时刻所在时辰之时计积数（即「天正時實」「夏至時實」，以 [`super::shi_ji_shu`]
/// 同法计，1 起），`since` 为自交至之时辰起已过之时辰数（交至之时辰为 0）。
/// 至时之门依时实以一百二十去之、三十约之（阳遁开生惊休、阴遁杜死伤景，同张良之序）；门「隨數起」——自交至之时辰起
/// 入门第一时，满三十时（「時盡」）移次门，故门界随交至之时而定，「不拘其甲丙己辛日」。
/// 「隨數起」读作至时重起入门之数，系本项目之解读（若读作仍依时实之余数，则门界与张良全同，王氏之改无所施），**未核**。
pub fn zhi_shi_shi_wang(zhi_shi_shu: i64, since: i64, dun: Dun) -> ZhiShi {
    let k0 = (zhi_shi_shu - 1).rem_euclid(120) / 30;
    let seq: [u8; 4] = match dun {
        Dun::Yang => [0, 2, 7, 1],
        Dun::Yin => [4, 6, 3, 5],
    };
    let s = since.max(0);
    ZhiShi { men: seq[((k0 + s / 30) % 4) as usize], ru: (s % 30 + 1) as u8 }
}

/// 布门之法：八门各临何宫
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuMenFa {
    /// 直使门加太乙宫，余门依开休生伤杜景死惊之序顺行八宫（金鏡卷一张良「常以直門加太乙及主客大將隨數而行」）。缺省
    ZhiShi,
    /// 阳遁开门、阴遁杜门加太乙宫顺行（金鏡卷一引李淳風「常以開門加太乙即太乙之八門也」；淘金歌「八門起例」
    /// 「凡陽遁皆以開門，陰遁皆以杜門為直事，常加太乙宮」，辛丑年太乙一宫例）
    KaiMen,
    /// 八门定向：开乾、休坎……（淘金歌「又法，二遁直事所加則異，八門定向則同」；金鏡卷二「推八門所主法」）
    DingXiang,
}

/// 八宫所临之门：下标为宫数（0、5 不用，为 None），值为门序（[`MEN8`]）
pub fn bu_men(zs: ZhiShi, gong: u8, dun: Dun, fa: BuMenFa) -> [Option<u8>; 10] {
    let t = MEN_GONG.iter().position(|g| *g == gong).unwrap_or(0);
    let (start, first) = match fa {
        BuMenFa::ZhiShi => (t, zs.men),
        BuMenFa::KaiMen => (t, if dun == Dun::Yin { 4 } else { 0 }),
        BuMenFa::DingXiang => (0, 0),
    };
    let mut out = [None; 10];
    for i in 0..8 {
        out[MEN_GONG[(start + i) % 8] as usize] = Some(((first as usize + i) % 8) as u8);
    }
    out
}

/// 十六神位所属之宫（门所临）：正宫依本宫；间神依后天八卦所辖之支（丑寅属艮、辰巳属巽、未申属坤、戌亥属乾）。
/// 间神之门原书未言（**未核**）。
pub fn pos_men_gong(p: usize) -> u8 {
    pos_gong(p).unwrap_or(match p % 16 {
        1 | 3 => 3,
        5 | 7 => 9,
        9 | 11 => 7,
        _ => 1,
    })
}

/// 三门具不具之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenJuFa {
    /// 金鏡卷四「推三門具不具」：「若太乙天目在開生二門下為兩門不具若在休門下為三門不具」，否则三门具；
    /// 卷一引李淳風「太乙天目在主客三門下者閉塞不通」。宜与 [`BuMenFa::ZhiShi`] 同用（开门加太乙则太乙恒在开门下）。缺省
    JinJing,
    /// 淘金歌「八門起例」：「視主將在開、休、生門下者，吉」；「門發將具」注「如主將發，又會三門之下，此大吉也」：
    /// 主、客大将所在宫之门为开休生者，主、客门具
    TaoJinGe,
    /// 太乙秘書：「五、十五、二十五、三十五，为八门杜塞」，七十二局注中「门具」皆与将不杜相随：算不得五者门具
    MiShu,
}

/// 三门具不具
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenJu {
    /// 主、客门具否（金鏡之法主客同）
    pub zhu: bool,
    pub ke: bool,
    /// 判语：金鏡「三門具」「兩門不具」「三門不具」；淘金歌、秘書「主門具」「客門不具」之类
    pub yu: String,
}

/// 依布门求三门具不具；`men` 为 [`bu_men`] 所得
pub fn men_ju(j: &TaiyiJu, men: &[Option<u8>; 10], fa: MenJuFa) -> MenJu {
    let ji = |g: Option<u8>| g.and_then(|g| men[g as usize]).is_some_and(|m| m <= 2);
    match fa {
        MenJuFa::JinJing => {
            let ms = [men[j.gong as usize], men[pos_men_gong(j.tian_mu) as usize]];
            let (yu, ok) = if ms.contains(&Some(1)) {
                ("三門不具", false)
            } else if ms.contains(&Some(0)) || ms.contains(&Some(2)) {
                ("兩門不具", false)
            } else {
                ("三門具", true)
            };
            MenJu { zhu: ok, ke: ok, yu: yu.into() }
        }
        MenJuFa::TaoJinGe | MenJuFa::MiShu => {
            let (zhu, ke) = if fa == MenJuFa::TaoJinGe { (ji(j.zhu_da), ji(j.ke_da)) } else { (j.zhu_da.is_some(), j.ke_da.is_some()) };
            let w = |b: bool| if b { "具" } else { "不具" };
            MenJu { zhu, ke, yu: format!("主門{}、客門{}", w(zhu), w(ke)) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ji_nian, ju, shi_ji_shu, sui_ji, JiNian};
    use super::*;

    /// 金鏡卷一「推八門占嵗計法」：开元十二年甲子开门直使，三十一年甲午休门；卷四「上元甲子開門為直使滿三十年即休門」
    #[test]
    fn jinjing_zhi_shi() {
        assert_eq!(zhi_shi(ji_nian(724, JiNian::JinJing), Dun::Yang), ZhiShi { men: 0, ru: 1 });
        assert_eq!(zhi_shi(ji_nian(753, JiNian::JinJing), Dun::Yang), ZhiShi { men: 0, ru: 30 });
        assert_eq!(zhi_shi(ji_nian(754, JiNian::JinJing), Dun::Yang), ZhiShi { men: 1, ru: 1 });
        // 卷五演纪积年 3,361（开元十二年）以二百四十去之亦余一
        assert_eq!(zhi_shi(3361, Dun::Yang), zhi_shi(ji_nian(724, JiNian::JinJing), Dun::Yang));
        assert_eq!(zhi_shi(1, Dun::Yang).men, 0);
        assert_eq!(zhi_shi(31, Dun::Yang).men, 1);
    }

    /// 金鏡卷一张良：阳遁甲日夜半甲子开门、丙日日中甲午生门、己日夜半甲子惊门、辛日日中甲午休门；阴遁杜死伤景
    #[test]
    fn zhang_liang_shi_men() {
        let at = |gz: u8, h: u8, d: Dun| zhi_shi_shi(shi_ji_shu(gz, h), d).men;
        assert_eq!([at(0, 0, Dun::Yang), at(2, 6, Dun::Yang), at(5, 0, Dun::Yang), at(7, 6, Dun::Yang)], [0, 2, 7, 1]);
        assert_eq!([at(0, 0, Dun::Yin), at(2, 6, Dun::Yin), at(5, 0, Dun::Yin), at(7, 6, Dun::Yin)], [4, 6, 3, 5]);
        // 甲戌日夜半复为开门（一百二十时一周）
        assert_eq!(at(10, 0, Dun::Yang), 0);
        assert_eq!(at(2, 5, Dun::Yang), 0); // 丙寅日巳时仍为开门
    }

    /// 王希明二至加时起门：至时之门依时实，入门自至时起第一时，三十时一移，门界随至时
    #[test]
    fn wang_xi_ming_shi_men() {
        // 冬至加时在甲子日子时：与张良同，开门第一时
        assert_eq!(zhi_shi_shi_wang(shi_ji_shu(0, 0), 0, Dun::Yang), ZhiShi { men: 0, ru: 1 });
        // 冬至加时在乙丑日卯时（时实 16）：门数 0（开门），张良入开门第十六时，王氏自此为开门第一时
        let s0 = shi_ji_shu(1, 3);
        assert_eq!(zhi_shi_shi(s0, Dun::Yang), ZhiShi { men: 0, ru: 16 });
        assert_eq!(zhi_shi_shi_wang(s0, 0, Dun::Yang), ZhiShi { men: 0, ru: 1 });
        // 张良丙寅日午时（至后十五时）已移生门，王氏至后三十时（丁卯日酉时）方移
        assert_eq!(zhi_shi_shi(s0 + 15, Dun::Yang).men, 2);
        assert_eq!(zhi_shi_shi_wang(s0, 15, Dun::Yang).men, 0);
        assert_eq!(zhi_shi_shi_wang(s0, 29, Dun::Yang), ZhiShi { men: 0, ru: 30 });
        assert_eq!(zhi_shi_shi_wang(s0, 30, Dun::Yang), ZhiShi { men: 2, ru: 1 });
        // 四门一周一百二十时
        assert_eq!(zhi_shi_shi_wang(s0, 120, Dun::Yang), ZhiShi { men: 0, ru: 1 });
        // 夏至加时在己巳日子时（时实 61，门数 2）：阴遁伤门
        assert_eq!(zhi_shi_shi_wang(shi_ji_shu(5, 0), 0, Dun::Yin), ZhiShi { men: 3, ru: 1 });
    }

    /// 淘金歌「八門起例」：第三丙子元辛丑年二十六局，太乙一宫，开门加乾，休坎、生艮、伤震、杜巽、景離、死坤、惊兌
    #[test]
    fn taojinge_ba_men() {
        // 辛丑二十六局：丙子元为第二元（r = 72 + 26 = 98）
        let j = ju(98, Dun::Yang);
        assert_eq!(j.gong, 1);
        let m = bu_men(ZhiShi { men: 0, ru: 1 }, j.gong, Dun::Yang, BuMenFa::KaiMen);
        let got: Vec<&str> = MEN_GONG.iter().map(|g| MEN8[m[*g as usize].unwrap() as usize]).collect();
        assert_eq!(got, ["開門", "休門", "生門", "傷門", "杜門", "景門", "死門", "驚門"]);
        // 定向与太乙一宫开门加太乙同
        assert_eq!(bu_men(ZhiShi { men: 3, ru: 1 }, 6, Dun::Yang, BuMenFa::DingXiang), m);
        // 直使加太乙：直使休门、太乙六宫，则兌休、乾生……
        let z = bu_men(ZhiShi { men: 1, ru: 1 }, 6, Dun::Yang, BuMenFa::ZhiShi);
        assert_eq!((z[6], z[1], z[8]), (Some(1), Some(2), Some(3)));
        let _ = sui_ji(724, JiNian::JinJing);
    }

    /// 金鏡卷四门具：太乙、天目在开生门下两门不具，在休门下三门不具
    #[test]
    fn jinjing_men_ju() {
        let j = ju(1, Dun::Yang); // 太乙一宫，天目武德（申，属坤七）
        let m = bu_men(ZhiShi { men: 0, ru: 1 }, j.gong, Dun::Yang, BuMenFa::ZhiShi);
        assert_eq!(men_ju(&j, &m, MenJuFa::JinJing).yu, "兩門不具");
        let m = bu_men(ZhiShi { men: 1, ru: 1 }, j.gong, Dun::Yang, BuMenFa::ZhiShi);
        assert_eq!(men_ju(&j, &m, MenJuFa::JinJing).yu, "三門不具");
        let m = bu_men(ZhiShi { men: 5, ru: 1 }, j.gong, Dun::Yang, BuMenFa::ZhiShi);
        // 太乙乾景门；坤七在乾后第六位（乾坎艮震巽離坤）：景死惊开休生伤 → 坤伤门
        assert_eq!(m[7], Some(3));
        assert_eq!(men_ju(&j, &m, MenJuFa::JinJing).yu, "三門具");
    }
}
