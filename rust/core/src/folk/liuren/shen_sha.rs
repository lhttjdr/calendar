//! 大六壬盘面标注：旬（旬首、旬空）、遁干、日之神煞（驿马、丁马、天马、日德、日禄、羊刃、日鬼）、年命（本命、行年）。
//!
//! 依《六壬大全》（四库本）：
//! - 旬空：卷三「甲子旬中河魁登明是孤辰寡宿……十干後二辰是也又名空亡神也」；卷七孤寡訂訛「發用值旬空」。
//! - 遁干：缺省以日所在之旬遁（卷九「壬申癸酉二日見夘是丁」「壬午癸未二日見丑是丁」，即旬丁；卷五元首課「午上遁得庚金」）；
//!   另一说以日干五子遁（卷三占財「乙庚遁庚辰」，即乙庚日丙子起），见 [`DunRule`]。
//! - 十天干神煞（卷一「神煞」）：日徳「寅申巳亥巳寅申巳亥巳」、日禄「寅夘巳午巳午申酉亥子」、羊刃「夘辰午未午未酉戌子丑」。
//! - 支馬（驿马，卷一「十二地支神煞」）：「寅亥申巳寅亥申巳寅亥申巳」（子至亥）。
//! - 天馬（卷一）：「午順六陽」，卷三「天馬正月午順行六陽」「五月占天馬在寅」，以月建（节月）论。
//! - 丁馬：卷六遊子課「旬丁者每旬丁干所值之神」。
//! - 日鬼：克日干之支（卷一「鬼」；书中例「壬辰日……三傳戌丑辰上下六鬼」「己丑日支上寅發用作鬼」，阴阳俱论）。
//! - 行年：卷六繁昌課觀月經注「男行年一嵗起丙寅……四十九在甲寅；女行年一嵗起壬申……三十四在己亥」。

use super::{ke, m12, TianPan, GAN_WX, ZHI_WX};

/// 遁干之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DunRule {
    /// 以日所在之旬遁：旬首甲加旬首支顺布十干，旬空二支无干。缺省（卷九旬丁诸例、卷五元首課例）
    XunDun,
    /// 以日干五子遁：甲己日甲子、乙庚日丙子、丙辛日戊子、丁壬日庚子、戊癸日壬子起，十二支皆有干
    /// （卷三占財注「乙庚遁庚辰」；卷三白虎注有「時遁」之名，所指未核）
    WuZiDun,
}

/// 旬首之支：日干支 g（0 甲子…59 癸亥）所在之旬，甲所值之支
pub fn xun_shou(day_gz: u8) -> u8 {
    m12((day_gz % 60) as i32 % 12 - (day_gz % 60) as i32 % 10)
}

/// 旬空二支：旬首支前十、十一位（甲子旬戌亥）
pub fn xun_kong(day_gz: u8) -> [u8; 2] {
    let s = xun_shou(day_gz) as i32;
    [m12(s + 10), m12(s + 11)]
}

/// 地支 z 所遁之干（甲 0…癸 9）；旬遁时旬空二支为 None
pub fn dun_gan(day_gz: u8, z: u8, rule: DunRule) -> Option<u8> {
    match rule {
        DunRule::XunDun => {
            let k = m12(z as i32 - xun_shou(day_gz) as i32);
            if k < 10 {
                Some(k)
            } else {
                None
            }
        }
        DunRule::WuZiDun => {
            let start = ((day_gz % 10) % 5) * 2;
            Some((start + z % 12) % 10)
        }
    }
}

/// 驿马（卷一支馬）：申子辰寅、巳酉丑亥、寅午戌申、亥卯未巳
pub fn yi_ma(day_zhi: u8) -> u8 {
    super::yi_ma(day_zhi % 12)
}

/// 丁马：旬中丁干所值之支（旬首支 + 3）
pub fn ding_ma(day_gz: u8) -> u8 {
    m12(xun_shou(day_gz) as i32 + 3)
}

/// 天马（卷一「午順六陽」）：寅月午、卯月申、辰月戌、巳月子、午月寅、未月辰，七月以后同。`yue_jian` 月建地支
pub fn tian_ma(yue_jian: u8) -> u8 {
    m12(2 * (yue_jian as i32) + 2)
}

/// 日德（卷一）：甲寅 乙申 丙巳 丁亥 戊巳 己寅 庚申 辛巳 壬亥 癸巳
pub fn ri_de(day_gan: u8) -> u8 {
    [2, 8, 5, 11, 5, 2, 8, 5, 11, 5][(day_gan % 10) as usize]
}

/// 日禄（卷一）：甲寅 乙卯 丙巳 丁午 戊巳 己午 庚申 辛酉 壬亥 癸子
pub fn ri_lu(day_gan: u8) -> u8 {
    [2, 3, 5, 6, 5, 6, 8, 9, 11, 0][(day_gan % 10) as usize]
}

/// 羊刃（卷一）：禄前一位
pub fn yang_ren(day_gan: u8) -> u8 {
    m12(ri_lu(day_gan) as i32 + 1)
}

/// 日鬼：五行克日干之支（阴阳俱论）
pub fn ri_gui(day_gan: u8) -> Vec<u8> {
    let gw = GAN_WX[(day_gan % 10) as usize];
    (0..12u8).filter(|&z| ke(ZHI_WX[z as usize], gw)).collect()
}

/// 行年（干支序）：男一岁丙寅顺行，女一岁壬申逆行。`sui` 虚岁（≥1）
pub fn xing_nian(sui: u32, male: bool) -> u8 {
    let n = ((sui.max(1) - 1) % 60) as i32;
    if male {
        (2 + n).rem_euclid(60) as u8
    } else {
        (8 - n).rem_euclid(60) as u8
    }
}

/// 由本命年干支与占年干支求虚岁（模 60，行年以 60 为周，足用）
pub fn xu_sui(ben_ming_gz: u8, zhan_nian_gz: u8) -> u32 {
    ((zhan_nian_gz as i32 - ben_ming_gz as i32).rem_euclid(60) + 1) as u32
}

/// 一日之盘面标注
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanBiao {
    pub xun_shou: u8,
    pub xun_kong: [u8; 2],
    /// `dun[d]`：地盘 d 上天盘神所遁之干
    pub dun: [Option<u8>; 12],
    pub yi_ma: u8,
    pub ding_ma: u8,
    /// 须月建
    pub tian_ma: Option<u8>,
    pub ri_de: u8,
    pub ri_lu: u8,
    pub yang_ren: u8,
    pub ri_gui: Vec<u8>,
}

/// 盘面标注：`yue_jian` 月建地支（无则不出天马）
pub fn pan_biao(pan: &TianPan, day_gz: u8, rule: DunRule, yue_jian: Option<u8>) -> PanBiao {
    let g = day_gz % 10;
    let mut dun = [None; 12];
    for d in 0..12u8 {
        dun[d as usize] = dun_gan(day_gz, pan.above(d), rule);
    }
    PanBiao {
        xun_shou: xun_shou(day_gz),
        xun_kong: xun_kong(day_gz),
        dun,
        yi_ma: yi_ma(day_gz % 12),
        ding_ma: ding_ma(day_gz),
        tian_ma: yue_jian.map(tian_ma),
        ri_de: ri_de(g),
        ri_lu: ri_lu(g),
        yang_ren: yang_ren(g),
        ri_gui: ri_gui(g),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{qi_ke, GAN, ZHI};
    use super::*;

    fn gz(s: &str) -> u8 {
        let c: Vec<char> = s.chars().collect();
        let g = GAN.iter().position(|x| x.starts_with(c[0])).unwrap() as u8;
        let z = ZHI.iter().position(|x| x.starts_with(c[1])).unwrap() as u8;
        (0..60u8).find(|k| k % 10 == g && k % 12 == z).unwrap()
    }
    fn z(s: &str) -> u8 {
        ZHI.iter().position(|x| *x == s).unwrap() as u8
    }
    fn g(s: &str) -> u8 {
        GAN.iter().position(|x| *x == s).unwrap() as u8
    }

    #[test]
    fn xun_and_kong() {
        assert_eq!(xun_shou(gz("甲子")), z("子"));
        assert_eq!(xun_shou(gz("癸酉")), z("子"));
        assert_eq!(xun_shou(gz("壬申")), z("子"));
        assert_eq!(xun_shou(gz("乙酉")), z("申"));
        // 卷三：甲子旬中河魁登明（戌亥）为孤寡，又名空亡神
        assert_eq!(xun_kong(gz("甲子")), [z("戌"), z("亥")]);
        assert_eq!(xun_kong(gz("癸巳")), [z("午"), z("未")]);
        assert_eq!(xun_kong(gz("甲寅")), [z("子"), z("丑")]);
        // 卷九：辛巳日干上酉为日之旺祿「奈是旬空」（甲戌旬空申酉）；癸亥日干上子「亦是旬空」（甲寅旬空子丑）
        assert!(xun_kong(gz("辛巳")).contains(&z("酉")));
        assert!(xun_kong(gz("癸亥")).contains(&z("子")));
        // 卷九：甲子日戌加子作初傳「雖曰日財乃是旬空」
        assert!(xun_kong(gz("甲子")).contains(&z("戌")));
        // 卷九：壬午日干上申「雖為日之長生奈是旬空」（甲戌旬空申酉）
        assert!(xun_kong(gz("壬午")).contains(&z("申")));
    }

    /// 遁干与丁马：书中例
    #[test]
    fn dun_examples() {
        // 卷五元首課：甲子日卯時子將，「午上遁得庚金為官星」——两说皆合
        for r in [DunRule::XunDun, DunRule::WuZiDun] {
            assert_eq!(dun_gan(gz("甲子"), z("午"), r), Some(g("庚")));
        }
        // 卷六合歡課：戊申日子時申將，「戊日天干上丑遁得癸」——两说皆合
        let k = qi_ke(gz("戊申"), z("申"), z("子"));
        assert_eq!(k.si_ke[0].shang, z("丑"));
        for r in [DunRule::XunDun, DunRule::WuZiDun] {
            assert_eq!(dun_gan(gz("戊申"), z("丑"), r), Some(g("癸")));
        }
        // 卷三占財注：「乙庚遁庚辰」——五子遁乙庚日皆然；旬遁只乙亥、庚辰等甲戌旬之日合
        assert_eq!(dun_gan(gz("乙丑"), z("辰"), DunRule::WuZiDun), Some(g("庚")));
        assert_eq!(dun_gan(gz("庚午"), z("辰"), DunRule::WuZiDun), Some(g("庚")));
        assert_eq!(dun_gan(gz("乙丑"), z("辰"), DunRule::XunDun), Some(g("戊")));
        // 卷九財乗丁馬格：壬申癸酉見卯是丁、壬午癸未見丑、壬辰癸巳見亥、壬寅癸卯見酉、壬子癸丑見未、壬戌癸亥見巳
        for (d, want) in [
            ("壬申", "卯"),
            ("癸酉", "卯"),
            ("壬午", "丑"),
            ("癸未", "丑"),
            ("壬辰", "亥"),
            ("癸巳", "亥"),
            ("壬寅", "酉"),
            ("癸卯", "酉"),
            ("壬子", "未"),
            ("癸丑", "未"),
            ("壬戌", "巳"),
            ("癸亥", "巳"),
        ] {
            assert_eq!(ding_ma(gz(d)), z(want), "{d}");
            assert_eq!(dun_gan(gz(d), z(want), DunRule::XunDun), Some(g("丁")), "{d}");
        }
        // 五子遁不合旬丁：壬申日五子遁庚子起，卯得癸、丁在未
        assert_eq!(dun_gan(gz("壬申"), z("卯"), DunRule::WuZiDun), Some(g("癸")));
        // 卷六遊子課：乙巳日三傳未戌丑「遇丁馬」——甲辰旬丁未
        assert_eq!(ding_ma(gz("乙巳")), z("未"));
        // 卷一天空下淚注：「甲子旬中傳出申酉二字即是下淚」——甲子旬壬申癸酉
        assert_eq!(dun_gan(gz("丙寅"), z("申"), DunRule::XunDun), Some(g("壬")));
        assert_eq!(dun_gan(gz("丙寅"), z("酉"), DunRule::XunDun), Some(g("癸")));
        // 旬空无干
        assert_eq!(dun_gan(gz("甲子"), z("戌"), DunRule::XunDun), None);
    }

    #[test]
    fn ri_shen_sha() {
        // 卷一十天干神煞表
        let de: Vec<u8> = (0..10).map(ri_de).collect();
        assert_eq!(de, ["寅", "申", "巳", "亥", "巳", "寅", "申", "巳", "亥", "巳"].map(z).to_vec());
        let lu: Vec<u8> = (0..10).map(ri_lu).collect();
        assert_eq!(lu, ["寅", "卯", "巳", "午", "巳", "午", "申", "酉", "亥", "子"].map(z).to_vec());
        let ren: Vec<u8> = (0..10).map(yang_ren).collect();
        assert_eq!(ren, ["卯", "辰", "午", "未", "午", "未", "酉", "戌", "子", "丑"].map(z).to_vec());
        // 卷一支馬：子至亥「寅亥申巳寅亥申巳寅亥申巳」
        let ma: Vec<u8> = (0..12).map(yi_ma).collect();
        assert_eq!(ma, ["寅", "亥", "申", "巳", "寅", "亥", "申", "巳", "寅", "亥", "申", "巳"].map(z).to_vec());
        // 天马：正月午，卷三「五月占天馬在寅」
        assert_eq!(tian_ma(z("寅")), z("午"));
        assert_eq!(tian_ma(z("午")), z("寅"));
        assert_eq!(tian_ma(z("辰")), z("戌"));
        assert_eq!(tian_ma(z("申")), z("午"));
        // 日鬼：壬日戌丑辰未（卷一「壬辰日……三傳戌丑辰上下六鬼」）；己日寅卯（「己丑日支上寅發用作鬼」）
        assert_eq!(ri_gui(g("壬")), ["丑", "辰", "未", "戌"].map(z).to_vec());
        assert_eq!(ri_gui(g("己")), ["寅", "卯"].map(z).to_vec());
        // 卷一：「如壬子日未加夘發用三傳木局……貴勾常為鬼」，壬日鬼土；「庚辰日午加己發用」為鬼
        assert!(ri_gui(g("庚")).contains(&z("午")));
    }

    /// 卷六繁昌課觀月經注：夫四十九嵗行年甲寅、妻三十四嵗行年己亥
    #[test]
    fn xing_nian_example() {
        assert_eq!(xing_nian(1, true), gz("丙寅"));
        assert_eq!(xing_nian(1, false), gz("壬申"));
        assert_eq!(xing_nian(49, true), gz("甲寅"));
        assert_eq!(xing_nian(34, false), gz("己亥"));
        assert_eq!(xing_nian(61, true), gz("丙寅"));
        assert_eq!(xu_sui(gz("甲子"), gz("甲子")), 1);
        assert_eq!(xu_sui(gz("甲子"), gz("壬申")), 9);
    }
}
