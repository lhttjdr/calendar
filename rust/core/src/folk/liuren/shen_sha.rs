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

/// 日墓之法（卷一「墓」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MuRule {
    /// 五行墓：木墓未、火墓戌、金墓丑、水土墓辰（卷一「未為木墓戌為火墓丑為金墓辰為水圡墓此蓋單論五行土寄坤」）。
    /// 书中诸例皆用此：卷一墓条「甲乙日未臨亥丙丁日戌臨寅戊己壬癸日辰臨申庚辛日丑臨巳」、卷七天獄「七月乙丑日……小吉加寅是正墓」、
    /// 卷八鬼墓「丁夘日戌加夘為干墓臨支」、卷十「唯六辛日丑加戌旦將乘白虎作墓神……其餘干無此例」等（见文档）。缺省
    WuXing,
    /// 十干墓：甲癸未、乙丙戊戌、丁己庚丑、辛壬辰（卷一「未為甲癸之墓戌為丙戊乙之墓丑為庚丁己之墓辰為壬辛之墓……
    /// 壬課重在日惟従十干墓不□五行墓」）
    ShiGan,
}

/// 日干之墓（地支序）
pub fn gan_mu(day_gan: u8, rule: MuRule) -> u8 {
    match rule {
        MuRule::WuXing => [7, 7, 10, 10, 4, 4, 1, 1, 4, 4][(day_gan % 10) as usize],
        MuRule::ShiGan => [7, 10, 10, 1, 10, 1, 1, 4, 4, 7][(day_gan % 10) as usize],
    }
}

/// 日支之墓（五行墓；土支墓辰，卷一「坐墓如亥子并辰戌丑未臨辰寅夘臨未申酉臨丑巳午臨戌」）
pub fn zhi_mu(zhi: u8) -> u8 {
    [4, 4, 7, 7, 4, 10, 10, 4, 1, 1, 4, 4][(zhi % 12) as usize]
}

/// 日本（日干五行长生）：木亥、火寅、金巳、水土申（卷七天獄「日本者干長生也」，例「七月乙丑日……天罡臨亥是斗繫日本」；
/// 卷一「戊己壬癸日辰臨申」长生处同申）
pub fn ri_ben(day_gan: u8) -> u8 {
    [11, 2, 8, 5, 8][GAN_WX[(day_gan % 10) as usize] as usize]
}

/// 旺相休囚死（0 旺 1 相 2 休 3 囚 4 死）：以月建定四时，寅卯木、巳午火、申酉金、亥子水旺，辰未戌丑四季月土旺
/// （卷五觀月經注「春木旺夏火旺秋金旺冬水旺四季土旺」「春火相夏土相秋水相冬木相季月金相」「春土死夏金死秋木死冬火死」）。
/// 当令者旺、旺所生者相、生旺者休、克旺者囚、旺所克者死
pub fn wang_shuai(wx: u8, yue_jian: u8) -> u8 {
    let y = (yue_jian % 12) as usize;
    let ling: u8 = [4, 2, 0, 0, 2, 1, 1, 2, 3, 3, 2, 4][y];
    let d = (wx as i32 - ling as i32).rem_euclid(5);
    // d：0 同（旺）1 旺所生（相）4 生旺（休）2 旺所克（死）3 克旺（囚）
    [0u8, 1, 4, 3, 2][d as usize]
}

/// 有气：旺或相
pub fn you_qi(wx: u8, yue_jian: u8) -> bool {
    wang_shuai(wx, yue_jian) <= 1
}

/// 四时当令之行（旺相之所据）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WangXiangRule {
    /// 季月（辰未戌丑）全月土旺（卷五觀月經注「四季土旺」「季月金相」）。缺省
    JiYue,
    /// 土旺四季各十八日：四立前十八日土旺，季月其余日仍属本季（卷八雜狀課「土守中宫分旺四季毎季前各旺十八日總七十二日合而成嵗
    /// 故辰中有餘木未有餘火戌中有餘金丑中有餘水各十二日」，又「春夏秋冬辰戌丑未月各寄旺十八日」；卷二太常注「土旺四季各十八日」）。
    /// 起讫用土王用事（四立前十八日至四立前一日，与择日同，[`crate::folk::zajie::tu_wang`]），须调用方给出是否在其中
    TuWang18,
}

/// 当令之行（0 木 1 火 2 土 3 金 4 水）。`tu_wang` 是否在土王用事期内（只 [`WangXiangRule::TuWang18`] 用）
pub fn dang_ling(yue_jian: u8, rule: WangXiangRule, tu_wang: bool) -> u8 {
    let y = (yue_jian % 12) as usize;
    match rule {
        WangXiangRule::JiYue => [4, 2, 0, 0, 2, 1, 1, 2, 3, 3, 2, 4][y],
        WangXiangRule::TuWang18 if tu_wang => 2,
        // 季月余日从本季：辰木、未火、戌金、丑水
        WangXiangRule::TuWang18 => [4, 4, 0, 0, 0, 1, 1, 1, 3, 3, 3, 4][y],
    }
}

/// 以当令之行定旺相休囚死（0 旺 1 相 2 休 3 囚 4 死）
pub fn wang_shuai_ling(wx: u8, ling: u8) -> u8 {
    let d = (wx as i32 - ling as i32).rem_euclid(5);
    [0u8, 1, 4, 3, 2][d as usize]
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
    /// 日干之墓、日支之墓
    pub gan_mu: u8,
    pub zhi_mu: u8,
}

/// 盘面标注：`yue_jian` 月建地支（无则不出天马）
pub fn pan_biao(pan: &TianPan, day_gz: u8, rule: DunRule, yue_jian: Option<u8>, mu: MuRule) -> PanBiao {
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
        gan_mu: gan_mu(g, mu),
        zhi_mu: zhi_mu(day_gz % 12),
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

    /// 日墓两说：书中所举干墓之例（卷一、卷七至卷十）。(出处, 日干, 书中所称干墓)；十干墓与五行墓不同者（乙丁戊己辛癸）全依五行墓
    #[test]
    fn mu_examples() {
        let cases: &[(&str, &str, &str)] = &[
            ("卷七天獄 七月乙丑日巳加子「小吉加寅是正墓」", "乙", "未"),
            ("卷七白虎 正月癸巳日戌加癸「三傳又見天羅日之正墓」（三傳戌未辰）", "癸", "辰"),
            ("卷八鬼墓 丁夘日戌加夘為干墓臨支", "丁", "戌"),
            ("卷九 丁亥日干上戌「始棄干上之墓」", "丁", "戌"),
            ("卷九祿被玄奪 辛夘日初傳丑「又是日墓」", "辛", "丑"),
            ("卷九 辛亥日初傳丑「是日之墓神」", "辛", "丑"),
            ("卷九有始無終 乙未日末傳未加夘「為干墓」", "乙", "未"),
            ("卷十 己酉日干上辰「墓其干」", "己", "辰"),
            ("卷十互坐邱墓 戊申日申加辰（支坐干墓）", "戊", "辰"),
            ("卷十 乙亥日未為干墓臨支", "乙", "未"),
            ("卷十 唯六辛日丑加戌乘白虎作墓神", "辛", "丑"),
            ("卷八物類「以干之墓神為妾」戊已壬癸見辰", "己", "辰"),
            ("卷一墓「日之長生處乗墓」庚辛日丑臨巳", "辛", "丑"),
            ("卷一墓「日之長生處乗墓」甲乙日未臨亥", "甲", "未"),
            ("卷一墓「日之長生處乗墓」丙丁日戌臨寅", "丙", "戌"),
            ("卷一墓「日之長生處乗墓」壬癸日辰臨申", "壬", "辰"),
        ];
        let mut shi_gan = 0;
        for (src, gs, want) in cases {
            assert_eq!(gan_mu(g(gs), MuRule::WuXing), z(want), "{src}");
            if gan_mu(g(gs), MuRule::ShiGan) == z(want) {
                shi_gan += 1;
            }
        }
        assert_eq!(cases.len(), 16);
        // 十干墓只合甲、丙、壬三例（两说同墓之干），乙丁戊己辛癸十三例皆不合
        assert_eq!(shi_gan, 3);
        // 十干墓表：甲癸未、乙丙戊戌、丁己庚丑、辛壬辰
        let sg: Vec<u8> = (0..10).map(|x| gan_mu(x, MuRule::ShiGan)).collect();
        assert_eq!(sg, ["未", "戌", "戌", "丑", "戌", "丑", "丑", "辰", "辰", "未"].map(z).to_vec());
        // 支墓：卷一「坐墓如亥子并辰戌丑未臨辰寅夘臨未申酉臨丑巳午臨戌」；卷十丙子日辰加子「支墓臨支」、丁亥日辰加亥
        assert_eq!(zhi_mu(z("子")), z("辰"));
        assert_eq!(zhi_mu(z("亥")), z("辰"));
        assert_eq!(zhi_mu(z("未")), z("辰"));
        assert_eq!(zhi_mu(z("寅")), z("未"));
        assert_eq!(zhi_mu(z("酉")), z("丑"));
        assert_eq!(zhi_mu(z("午")), z("戌"));
        // 日本（长生）：卷七天獄乙日斗加亥
        assert_eq!(ri_ben(g("乙")), z("亥"));
        // 旺相：卯月木旺火相水休金囚土死；未月（季月）土旺金相火休木囚水死
        assert_eq!([0u8, 1, 2, 3, 4].map(|w| wang_shuai(w, z("卯"))), [0, 1, 4, 3, 2]);
        assert_eq!([2u8, 3, 1, 0, 4].map(|w| wang_shuai(w, z("未"))), [0, 1, 2, 3, 4]);
    }

    /// 卷十「干乘墓虎」：「唯六辛日丑加戌旦將乘白虎作墓神……外有六乙日晝貴順行雖是未乘白虎止有臨行年本命上者即無加干者也
    /// ……其餘干無此例」。干上神为干墓且乘白虎者，五行墓配书中实用贵人只有辛日昼，十干墓则无一日
    #[test]
    fn gan_cheng_mu_hu() {
        use super::super::{tian_jiang, GuiRule, TianPan, GAN_JI};
        let has = |mu: MuRule| -> Vec<(u8, bool)> {
            let mut v = Vec::new();
            for gi in 0..10u8 {
                let m = gan_mu(gi, mu);
                let ji = GAN_JI[gi as usize];
                let pan = TianPan { jiang: m12(m as i32 - ji as i32), shi: 0 };
                for zhou in [true, false] {
                    if tian_jiang(&pan, gi, zhou, GuiRule::DaQuan).name_of(m) == "白虎" {
                        v.push((gi, zhou));
                    }
                }
            }
            v
        };
        assert_eq!(has(MuRule::WuXing), vec![(g("辛"), true)]);
        assert!(has(MuRule::ShiGan).is_empty());
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
