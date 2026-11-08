//! 奇门遁甲布盘（时家）：地盘六仪三奇、旬首、直符直使、天盘九星与干、八门、八神。
//!
//! 两派并列（[`PanMethod`]）：
//! - **转盘**：九星、八门、八神沿八宫圆周（坎艮震巽离坤兑乾）旋转，中五寄宫。依《遁甲演義》卷一「時家竒門」
//!   「八門九星逐時移宫訣」、卷三「竒遁佈局法」「冬至陽遁時竒起例」「夏至隂遁時竒起例」「天乙直符吉凶神説」，
//!   《太白陰經》卷九「課式」，《古今圖書集成·藝術典》卷七百八「凡例」「口訣」。《集成》卷七百九至七百十二
//!   阴阳十八局逐时表逐项检验（见 doc/folk/qimen.md「布盘」）。缺省。
//! - **飞盘**（飞宫）：明陶仲文《陶真人遁甲神書》所创（原书未见），据《御定奇門寶鑑》「遁甲总论」转述：地盘阳一局
//!   一戊二己……九乙、「五還中宫」不寄；天盘九星与干按洛书数序（一至九，含中五）整体平移，使旬首之仪落于时干之宫；
//!   六甲时时干作戊（寶鑑以甲戌时与乙丑时同列第二图）；直符直使同加时干之宫（「以蓬休符使加于離九宮」）。
//!   阴遁之布法与平移方向、八门平移后无门之宫与中五之门、八神：寶鑑未载，出处未核（八门照数平移、八神不布为缺省）。
//!
//! 中五寄宫（[`JiGong`]，只用于转盘；飞盘直符直使在五时取其门亦用之）：寄坤（缺省）、阳坤阴艮、阳巽阴震。

use super::ShiJu;

/// 洛书九宫名（1—9）
pub const GONG: [&str; 10] = ["", "坎", "坤", "震", "巽", "中", "乾", "兌", "艮", "離"];
/// 九星本位（洛书宫 1—9）：蓬芮冲辅禽心柱任英
pub const XING: [&str; 10] = ["", "天蓬", "天芮", "天衝", "天輔", "天禽", "天心", "天柱", "天任", "天英"];
/// 八门本位（中五无门）
pub const MEN: [&str; 10] = ["", "休門", "死門", "傷門", "杜門", "", "開門", "驚門", "生門", "景門"];
/// 六仪三奇布局之序：戊己庚辛壬癸丁丙乙（阳遁自局数宫顺布，阴遁逆布）
pub const YI_QI: [&str; 9] = ["戊", "己", "庚", "辛", "壬", "癸", "丁", "丙", "乙"];
/// 八宫圆周（自坎顺行：坎艮震巽离坤兑乾），转盘九星、八门、八神沿此旋转
pub const RING: [u8; 8] = [1, 8, 3, 4, 9, 2, 7, 6];
const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
const XUN: [&str; 6] = ["甲子", "甲戌", "甲申", "甲午", "甲辰", "甲寅"];

/// 布盘之派
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanMethod {
    /// 转盘（缺省）
    ZhuanPan,
    /// 飞盘（飞宫）：陶氏法，据《奇門寶鑑》转述；阴遁、八门细节、八神出处未核
    FeiPan,
}

/// 中五寄宫
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiGong {
    /// 寄坤二：演义卷三「佈五宫則寄坤土」，卷一「甲辰五乃寄在坤二宫」；《集成》卷七百八凡例「天禽無位寄在坤二宮」；
    /// 《奇門寶鑑》「中五所以合於坤二……旧本多从前说，故遵之」。缺省
    Kun,
    /// 阳遁寄坤、阴遁寄艮：《奇門寶鑑》「一本阳局中五合於二，阴局中五合於八」；《元靈經》「五宮位中，寄生西南，陰寄東北」
    KunGen,
    /// 阳遁寄巽、阴遁寄震：《奇門寶鑑》所述先天说（「冬至……用阳局，而以中五寄巽……夏至……用阴局，而以中五寄震」）
    XunZhen,
}

impl JiGong {
    /// 所寄之宫
    pub fn gong(self, yang: bool) -> u8 {
        match (self, yang) {
            (JiGong::Kun, _) => 2,
            (JiGong::KunGen, true) => 2,
            (JiGong::KunGen, false) => 8,
            (JiGong::XunZhen, true) => 4,
            (JiGong::XunZhen, false) => 3,
        }
    }
}

/// 八神名目：演义卷三「天乙直符吉凶神説」作白虎、玄武；《集成》卷七百七「八詐門者，直符、螣蛇、太陰、六合、勾陳、朱雀、九地、九天」
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShenMing {
    /// 直符、螣蛇、太陰、六合、白虎、玄武、九地、九天（缺省）
    BaiHuXuanWu,
    /// 直符、螣蛇、太陰、六合、勾陳、朱雀、九地、九天
    GouChenZhuQue,
}

impl ShenMing {
    pub fn names(self) -> [&'static str; 8] {
        match self {
            ShenMing::BaiHuXuanWu => ["直符", "螣蛇", "太陰", "六合", "白虎", "玄武", "九地", "九天"],
            ShenMing::GouChenZhuQue => ["直符", "螣蛇", "太陰", "六合", "勾陳", "朱雀", "九地", "九天"],
        }
    }
}

/// 布盘选项
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PanOptions {
    pub method: PanMethod,
    pub ji_gong: JiGong,
    pub shen_ming: ShenMing,
}

/// 缺省：转盘、寄坤、白虎玄武
pub const DEFAULT_PAN: PanOptions = PanOptions { method: PanMethod::ZhuanPan, ji_gong: JiGong::Kun, shen_ming: ShenMing::BaiHuXuanWu };

/// 一宫
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GongCell {
    /// 地盘干（转盘中五之干并入所寄之宫时，在寄宫另列于第二项；中五本宫仍列）
    pub di_pan: Vec<&'static str>,
    /// 天盘干（随九星而来；转盘寄宫之星带中五之干）
    pub tian_pan: Vec<&'static str>,
    /// 天盘九星（转盘寄宫之星带天禽）
    pub xing: Vec<&'static str>,
    /// 八门
    pub men: Option<&'static str>,
    /// 八神（飞盘不布）
    pub shen: Option<&'static str>,
}

/// 一盘
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QimenPan {
    pub ju: ShiJu,
    pub options: PanOptions,
    /// 时干支序（0 甲子 … 59 癸亥）
    pub hour_gz: u8,
    /// 旬首（甲子……甲寅）与所遁之仪（戊……癸）
    pub xun_shou: &'static str,
    pub xun_yi: &'static str,
    /// 旬首所在地盘宫（1—9，可为五）
    pub xun_gong: u8,
    /// 直符（九星，旬首宫之星；在五为天禽）、直使（八门，旬首宫之门；在五取寄宫之门）
    pub zhi_fu: &'static str,
    pub zhi_shi: &'static str,
    /// 时干所在地盘宫（直符所加，1—9，可为五；时干为甲则同旬首）
    pub zhi_fu_gong: u8,
    /// 直使所加之宫：自旬首宫逐时数去（阳顺阴逆，可为五）
    pub zhi_shi_gong: u8,
    /// 九宫，下标 0—8 对应洛书 1—9 宫
    pub gong: [GongCell; 9],
}

impl QimenPan {
    /// 洛书 p 宫（1—9）
    pub fn at(&self, p: u8) -> &GongCell {
        &self.gong[p as usize - 1]
    }
}

fn wrap9(p: i32) -> u8 {
    (p - 1).rem_euclid(9) as u8 + 1
}

fn ring_idx(p: u8) -> i32 {
    RING.iter().position(|&x| x == p).expect("八宫") as i32
}

fn ring_at(i: i32) -> u8 {
    RING[i.rem_euclid(8) as usize]
}

/// 地盘：各宫之干（1—9）
pub fn di_pan(yang: bool, ju: u8) -> [&'static str; 10] {
    let mut d = [""; 10];
    for (i, s) in YI_QI.iter().enumerate() {
        let p = wrap9(ju as i32 + if yang { i as i32 } else { -(i as i32) });
        d[p as usize] = s;
    }
    d
}

/// 布盘。`hour_gz` 为所用时之干支序（与 `ju` 同一时辰）。
pub fn pai_pan(ju: &ShiJu, hour_gz: u8, opts: PanOptions) -> QimenPan {
    let yang = ju.yang;
    let dp = di_pan(yang, ju.ju);
    let pos = |s: &str| (1..=9u8).find(|&p| dp[p as usize] == s).expect("干");
    let hour_gz = hour_gz % 60;
    let xun = (hour_gz / 10) as usize;
    let xun_yi = YI_QI[xun];
    let xun_gong = pos(xun_yi);
    let g = (hour_gz % 10) as usize;
    let fei = opts.method == PanMethod::FeiPan;
    // 时干为甲：转盘遁于本旬之仪；飞盘（陶氏）作戊——《奇門寶鑑》「遁甲总论」列甲戌时与乙丑时同为第二图，甲子时天地盘相同
    let zhi_fu_gong = if g == 0 {
        if fei { pos("戊") } else { xun_gong }
    } else {
        pos(GAN[g])
    };
    let k = (hour_gz % 10) as i32;
    // 直使：转盘自旬首宫逐时数宫（阳顺阴逆）；飞盘与直符同加时干之宫（寶鑑转述陶氏「以蓬休符使加于離九宮」）
    let zhi_shi_gong = if fei { zhi_fu_gong } else { wrap9(xun_gong as i32 + if yang { k } else { -k }) };
    let ji = opts.ji_gong.gong(yang);
    let eff = |p: u8| if p == 5 { ji } else { p };
    let zhi_fu = XING[xun_gong as usize];
    let zhi_shi = MEN[eff(xun_gong) as usize];
    let mut gong: [GongCell; 9] = Default::default();
    for p in 1..=9u8 {
        gong[p as usize - 1].di_pan.push(dp[p as usize]);
    }
    match opts.method {
        PanMethod::ZhuanPan => {
            gong[ji as usize - 1].di_pan.push(dp[5]);
            // 九星与天盘干：直符（旬首宫之星，在五则随寄宫）加时干宫（在五则寄宫）
            let src = eff(xun_gong);
            let tgt = eff(zhi_fu_gong);
            let sh = ring_idx(tgt) - ring_idx(src);
            for &t in RING.iter() {
                let s = ring_at(ring_idx(t) - sh);
                let c = &mut gong[t as usize - 1];
                c.xing.push(XING[s as usize]);
                c.tian_pan.push(dp[s as usize]);
                if s == ji {
                    c.xing.push(XING[5]);
                    c.tian_pan.push(dp[5]);
                }
            }
            // 八门：直使加时宫（在五则寄宫）
            let msh = ring_idx(eff(zhi_shi_gong)) - ring_idx(src);
            for &t in RING.iter() {
                gong[t as usize - 1].men = Some(MEN[ring_at(ring_idx(t) - msh) as usize]);
            }
            // 八神：自天盘直符所临之宫起，阳顺阴逆（演义卷三：前二太阴、前三六合、后一九天、后二九地）
            let i0 = ring_idx(tgt);
            for (i, n) in opts.shen_ming.names().iter().enumerate() {
                let t = ring_at(i0 + if yang { i as i32 } else { -(i as i32) });
                gong[t as usize - 1].shen = Some(n);
            }
        }
        PanMethod::FeiPan => {
            // 九星（含天禽）与天盘干按洛书数整体平移：直符自旬首宫至时干宫（出处未核）
            let d = zhi_fu_gong as i32 - xun_gong as i32;
            for q in 1..=9u8 {
                let s = wrap9(q as i32 - d);
                let c = &mut gong[q as usize - 1];
                c.xing.push(XING[s as usize]);
                c.tian_pan.push(dp[s as usize]);
            }
            // 八门按数平移：直使之门（旬首在五取寄宫之门）自其本宫至直使所加之宫（即时干之宫）；中五本无门，平移后所到之宫无门
            //（寶鑑问「直使仅有八门，又将何法以分九宫」，可知陶图亦必有一宫无门，其处置原书未见，出处未核）
            let msrc = eff(xun_gong);
            let md = zhi_shi_gong as i32 - msrc as i32;
            for q in 1..=9u8 {
                let s = wrap9(q as i32 - md);
                let m = MEN[s as usize];
                gong[q as usize - 1].men = (!m.is_empty()).then_some(m);
            }
        }
    }
    QimenPan {
        ju: ju.clone(),
        options: opts,
        hour_gz,
        xun_shou: XUN[xun],
        xun_yi,
        xun_gong,
        zhi_fu,
        zhi_shi,
        zhi_fu_gong,
        zhi_shi_gong,
        gong,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ShiJu, Yuan};
    use super::*;

    fn sj(yang: bool, ju: u8) -> ShiJu {
        ShiJu { yang, ju, term: if yang { 18 } else { 6 }, yuan: Yuan::Shang, run: false, chao: 0 }
    }
    const GZ: [&str; 60] = [
        "甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己巳", "庚午", "辛未", "壬申", "癸酉", "甲戌", "乙亥", "丙子", "丁丑", "戊寅",
        "己卯", "庚辰", "辛巳", "壬午", "癸未", "甲申", "乙酉", "丙戌", "丁亥", "戊子", "己丑", "庚寅", "辛卯", "壬辰", "癸巳",
        "甲午", "乙未", "丙申", "丁酉", "戊戌", "己亥", "庚子", "辛丑", "壬寅", "癸卯", "甲辰", "乙巳", "丙午", "丁未", "戊申",
        "己酉", "庚戌", "辛亥", "壬子", "癸丑", "甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥",
    ];
    fn gz(s: &str) -> u8 {
        GZ.iter().position(|x| *x == s).unwrap() as u8
    }
    fn zp(yang: bool, ju: u8, h: &str) -> QimenPan {
        pai_pan(&sj(yang, ju), gz(h), DEFAULT_PAN)
    }
    fn men(p: &QimenPan, g: u8) -> &'static str {
        p.at(g).men.unwrap()
    }
    fn has_tian(p: &QimenPan, g: u8, s: &str) -> bool {
        p.at(g).tian_pan.contains(&s)
    }

    /// 演义卷一「時家竒門」：阳一局六旬直符直使；阴九局同
    #[test]
    fn yanyi_shijia_xun() {
        let cases = [("甲子", "天蓬", "休門"), ("甲戌", "天芮", "死門"), ("甲申", "天衝", "傷門"), ("甲午", "天輔", "杜門"), ("甲辰", "天禽", "死門"), ("甲寅", "天心", "開門")];
        for (h, x, m) in cases {
            let p = zp(true, 1, h);
            assert_eq!((p.zhi_fu, p.zhi_shi), (x, m), "阳一 {h}");
        }
        let cases = [("甲子", "天英", "景門"), ("甲戌", "天任", "生門"), ("甲申", "天柱", "驚門"), ("甲午", "天心", "開門"), ("甲辰", "天禽", "死門"), ("甲寅", "天輔", "杜門")];
        for (h, x, m) in cases {
            let p = zp(false, 9, h);
            assert_eq!((p.zhi_fu, p.zhi_shi), (x, m), "阴九 {h}");
        }
        // 「乙竒離 丙竒艮 丁竒兊」；阴九「丁竒震 丙竒坤 乙竒坎」
        let d = di_pan(true, 1);
        assert_eq!((d[9], d[8], d[7]), ("乙", "丙", "丁"));
        let d = di_pan(false, 9);
        assert_eq!((d[3], d[2], d[1]), ("丁", "丙", "乙"));
    }

    /// 演义卷一「八門九星逐時移宫訣」：阳一局甲子、乙丑、丙寅三时
    #[test]
    fn yanyi_yishi_yigong() {
        // 甲子：蓬、休俱在一，「生門丙竒下有丙在艮」「乙竒在離」「丁竒在兑」
        let p = zp(true, 1, "甲子");
        assert_eq!(p.at(1).xing, vec!["天蓬"]);
        assert_eq!(men(&p, 1), "休門");
        assert!(has_tian(&p, 8, "丙") && men(&p, 8) == "生門");
        assert!(has_tian(&p, 9, "乙") && has_tian(&p, 7, "丁"));
        // 乙丑：蓬星在九、休門在二；「休門丙竒在二坤宫」「乙竒在坎無門」「丁竒在震無門」（無吉門）
        let p = zp(true, 1, "乙丑");
        assert_eq!((p.zhi_fu_gong, p.zhi_shi_gong), (9, 2));
        assert!(p.at(9).xing.contains(&"天蓬") && men(&p, 2) == "休門" && has_tian(&p, 2, "丙"));
        assert!(has_tian(&p, 1, "乙") && has_tian(&p, 3, "丁"));
        for g in [1, 3] {
            assert!(!["休門", "生門", "開門"].contains(&men(&p, g)));
        }
        // 丙寅：蓬在八、休在三；「休門丙竒在震宫」「乙竒在二宫丁竒在六宫俱無門」
        let p = zp(true, 1, "丙寅");
        assert_eq!((p.zhi_fu_gong, p.zhi_shi_gong), (8, 3));
        assert!(men(&p, 3) == "休門" && has_tian(&p, 3, "丙"));
        assert!(has_tian(&p, 2, "乙") && has_tian(&p, 6, "丁"));
        for g in [2, 6] {
            assert!(!["休門", "生門", "開門"].contains(&men(&p, g)));
        }
    }

    /// 演义卷三「冬至陽遁時竒起例」「夏至隂遁時竒起例」（万历二十五年）
    #[test]
    fn yanyi_juan3_examples() {
        // 小满中局阳二局，丙午时：心星直符加离；蓬乙到坤、任丁到兑、英丙到艮；开门加艮；杜门加坤、景门加兑
        let p = zp(true, 2, "丙午");
        assert_eq!((p.xun_shou, p.zhi_fu, p.zhi_shi), ("甲辰", "天心", "開門"));
        assert!(p.at(9).xing.contains(&"天心"));
        assert!(p.at(2).xing.contains(&"天蓬") && has_tian(&p, 2, "乙"));
        assert!(p.at(7).xing.contains(&"天任") && has_tian(&p, 7, "丁"));
        assert!(p.at(8).xing.contains(&"天英") && has_tian(&p, 8, "丙"));
        assert_eq!((men(&p, 8), men(&p, 2), men(&p, 7)), ("開門", "杜門", "景門"));
        // 「甲辰乾 乙巳兑 丙午時艮」：直使逐时数宫
        assert_eq!(p.zhi_shi_gong, 8);
        // 夏至中局阴三局，丙申时：英星直符，时干丙在中寄坤；禽丙到兑、心丁到坎、辅乙到离；景门加兑
        let p = zp(false, 3, "丙申");
        let d = di_pan(false, 3);
        assert_eq!((d[3], d[2], d[1], d[9], d[8], d[7], d[6], d[5], d[4]), ("戊", "己", "庚", "辛", "壬", "癸", "丁", "丙", "乙"));
        assert_eq!((p.zhi_fu, p.zhi_fu_gong), ("天英", 5));
        assert!(p.at(2).xing.contains(&"天英"));
        assert!(p.at(7).xing.contains(&"天禽") && has_tian(&p, 7, "丙"));
        assert!(p.at(1).xing.contains(&"天心") && has_tian(&p, 1, "丁"));
        assert!(p.at(9).xing.contains(&"天輔") && has_tian(&p, 9, "乙"));
        assert_eq!(men(&p, 7), "景門");
        // 「離兑坎三宫得竒不得門，艮震巽三宫得門不得竒」：艮開、震休、巽生
        assert_eq!((men(&p, 8), men(&p, 3), men(&p, 4)), ("開門", "休門", "生門"));
    }

    /// 《太白陰經》卷九「課式」：阳一局丁卯时开门临震、生门临离，震下有乙、离有丁；直符临九宫之八神
    #[test]
    fn taibai_yinjing() {
        let p = zp(true, 1, "丁卯");
        assert_eq!(p.zhi_shi_gong, 4);
        assert_eq!((men(&p, 3), men(&p, 9)), ("開門", "生門"));
        assert!(has_tian(&p, 3, "乙") && has_tian(&p, 9, "丁"));
        // 「陽遁直符臨九宫，則九天在四宫、九地在三宫、太陰在七宫、六合在六宫」（阳一局乙丑时直符在九）
        let p = zp(true, 1, "乙丑");
        assert_eq!(p.at(9).shen, Some("直符"));
        assert_eq!((p.at(4).shen, p.at(3).shen, p.at(7).shen, p.at(6).shen), (Some("九天"), Some("九地"), Some("太陰"), Some("六合")));
    }

    /// 《奇門寶鑑》「注释烟波钓叟歌」诸格之例（转盘、寄坤）
    #[test]
    fn baojian_examples() {
        // 立春下元阳二局，壬申时：天芮直符加乾，辅庚下临坤二宫（伏宫格）
        let p = zp(true, 2, "壬申");
        assert_eq!((p.zhi_fu, p.zhi_fu_gong), ("天芮", 6));
        assert!(p.at(2).xing.contains(&"天輔") && has_tian(&p, 2, "庚"));
        // 秋分下局阴四局，丙寅时：天辅直符加乾，六庚临艮八宫癸上（大格）
        let p = zp(false, 4, "丙寅");
        assert!(p.at(6).xing.contains(&"天輔"));
        assert!(has_tian(&p, 8, "庚") && p.at(8).di_pan[0] == "癸");
        // 大寒上局阳三局，丙寅时：天冲直符加坎，天禽六庚加巽己上（刑格）
        let p = zp(true, 3, "丙寅");
        assert!(p.at(1).xing.contains(&"天衝"));
        assert!(p.at(4).xing.contains(&"天禽") && has_tian(&p, 4, "庚") && p.at(4).di_pan[0] == "己");
        // 立春中局阳五局，癸酉时：中五天禽寄坤以天芮为直符加坎；天柱六庚临艮辛上（岁格）
        let p = zp(true, 5, "癸酉");
        assert!(p.at(1).xing.contains(&"天芮"));
        assert!(p.at(8).xing.contains(&"天柱") && has_tian(&p, 8, "庚") && p.at(8).di_pan[0] == "辛");
        // 立春上局阳八局，丁卯时：丁在中寄坤，天任直符加坤；蓬庚到离
        let p = zp(true, 8, "丁卯");
        assert_eq!(p.zhi_fu_gong, 5);
        assert!(p.at(2).xing.contains(&"天任"));
        assert!(p.at(9).xing.contains(&"天蓬") && has_tian(&p, 9, "庚"));
        // 小暑下局阴五局，丙寅时：天禽直符加兑；冲庚加巽己
        let p = zp(false, 5, "丙寅");
        assert_eq!(p.zhi_fu, "天禽");
        assert!(p.at(7).xing.contains(&"天禽"));
        assert!(p.at(4).xing.contains(&"天衝") && has_tian(&p, 4, "庚") && p.at(4).di_pan[0] == "己");
    }

    /// 《集成》卷七百九至七百十二阴阳十八局逐时表：直符星与所加之宫、直使门与所加之宫、九天九地太阴六合、
    /// 三奇所在宫之门，逐项比对（录文字讹之处见 doc/folk/qimen.md「布盘」）
    #[test]
    fn jicheng_tables() {
        let dir = crate::repo::repo_root().join("vendor/folk/wikisource/古今圖書集成藝術典");
        let mut txt = String::new();
        for v in 709..=712 {
            match std::fs::read_to_string(dir.join(format!("第{v}卷.txt"))) {
                Ok(s) => txt.push_str(&s),
                Err(_) => {
                    println!("skipped: 无《集成》卷{v}录文");
                    return;
                }
            }
        }
        let txt: String = txt.chars().filter(|c| !"《》〈〉「」『』，、。：；　 \n\r\t".contains(*c)).collect();
        let num = |c: &str| "一二三四五六七八九".chars().position(|x| x.to_string() == c).unwrap() as u8 + 1;
        let gong_of = |c: &str| match c {
            "坎" => 1,
            "坤" => 2,
            "震" => 3,
            "巽" | "㢲" => 4,
            "乾" => 6,
            "兌" => 7,
            "艮" => 8,
            "離" => 9,
            _ => unreachable!(),
        };
        let head = regex::Regex::new(r"(陽|陰)遁([一二三四五六七八九])局").unwrap();
        let hour = regex::Regex::new(&format!(r"({})([蓬內芮衝輔禽心柱任英])([一二三四五六七八九])([休死傷杜開驚生景])([一二三四五六七八九])", GZ.join("|"))).unwrap();
        let shen = regex::Regex::new(r"天([一二三四五六七八九])地([一二三四五六七八九])陰([一二三四五六七八九])(?:合|陽)?([一二三四五六七八九])").unwrap();
        let door = regex::Regex::new(r"([休死傷杜開驚生景])[乙丙丁]?([坎坤震巽㢲乾兌艮離])").unwrap();
        let heads: Vec<_> = head.captures_iter(&txt).map(|c| (c.get(0).unwrap().start(), c[1] == *"陽", num(&c[2]))).collect();
        let (mut n_hours, mut ok, mut tot) = (0, 0, 0);
        for (i, &(start, yang, ju)) in heads.iter().enumerate() {
            let end = heads.get(i + 1).map(|h| h.0).unwrap_or(txt.len());
            let body = &txt[start..end];
            let hs: Vec<_> = hour.captures_iter(body).collect();
            for (j, c) in hs.iter().enumerate() {
                n_hours += 1;
                let seg_end = hs.get(j + 1).map(|n| n.get(0).unwrap().start()).unwrap_or(body.len());
                let seg = &body[c.get(0).unwrap().end()..seg_end];
                let p = zp(yang, ju, &c[1]);
                let star = if &c[2] == "內" { "芮" } else { &c[2] };
                let mut check = |b: bool| {
                    tot += 1;
                    ok += b as i32;
                };
                check(p.zhi_fu.ends_with(star));
                check(p.zhi_fu_gong == num(&c[3]));
                check(p.zhi_shi.starts_with(&c[4]));
                check(p.zhi_shi_gong == num(&c[5]));
                if let Some(s) = shen.captures(seg) {
                    for (k, name) in ["九天", "九地", "太陰", "六合"].iter().enumerate() {
                        let g = num(&s[k + 1]);
                        check(p.at(g).shen == Some(name));
                    }
                }
                for d in door.captures_iter(seg) {
                    let g = gong_of(&d[2]);
                    check(p.at(g).men.is_some_and(|m| m.starts_with(&d[1])));
                }
            }
        }
        println!("集成十八局表：{n_hours} 时，{ok}/{tot} 项合");
        assert!(n_hours >= 970, "解析时数 {n_hours}");
        // 余为录文字讹（零星一二、二三之混），逐条见文档
        assert!(ok as f64 / tot as f64 > 0.99, "{ok}/{tot}");
    }

    /// 寄宫三说以《集成》表衡之：寄坤最合（其余见文档）
    #[test]
    fn ji_gong_variants() {
        assert_eq!(JiGong::Kun.gong(false), 2);
        assert_eq!(JiGong::KunGen.gong(false), 8);
        assert_eq!(JiGong::XunZhen.gong(true), 4);
        // 阳一局甲辰旬：寄坤直使死门，阳坤阴艮同；阳巽阴震为杜门
        let h = gz("乙巳");
        let a = pai_pan(&sj(true, 1), h, DEFAULT_PAN);
        let b = pai_pan(&sj(true, 1), h, PanOptions { ji_gong: JiGong::XunZhen, ..DEFAULT_PAN });
        assert_eq!((a.zhi_shi, b.zhi_shi), ("死門", "杜門"));
        // 阴九局甲辰旬：寄坤死门，阳坤阴艮为生门
        let c = pai_pan(&sj(false, 9), h, PanOptions { ji_gong: JiGong::KunGen, ..DEFAULT_PAN });
        assert_eq!(c.zhi_shi, "生門");
    }

    /// 飞盘：九星按数平移、中五不寄；伏吟时与地盘同
    #[test]
    fn fei_pan_basic() {
        let o = PanOptions { method: PanMethod::FeiPan, ..DEFAULT_PAN };
        // 甲子时伏吟：星门各在本宫
        let p = pai_pan(&sj(true, 1), gz("甲子"), o);
        for q in 1..=9u8 {
            assert_eq!(p.at(q).xing, vec![XING[q as usize]]);
            assert_eq!(p.at(q).men.unwrap_or(""), MEN[q as usize]);
        }
        // 阳一局乙丑：直符天蓬飞至离九（乙在九），诸星同移八宫（+8 ≡ −1）；休门数至二
        let p = pai_pan(&sj(true, 1), gz("乙丑"), o);
        assert_eq!(p.at(9).xing, vec!["天蓬"]);
        assert_eq!(p.at(1).xing, vec!["天芮"]);
        assert_eq!(p.at(4).xing, vec!["天禽"]);
        // 寶鑑「以蓬休符使加于離九宮」：直使与直符同加时干之宫
        assert_eq!((p.zhi_fu_gong, p.zhi_shi_gong), (9, 9));
        assert_eq!(p.at(9).men, Some("休門"));
        assert!(p.gong.iter().all(|c| c.shen.is_none()));
        // 八门八个、一宫无门
        assert_eq!(p.gong.iter().filter(|c| c.men.is_none()).count(), 1);
    }

    /// 《奇門寶鑑》「遁甲总论」转述陶氏阳一局：同甲子时者「共为一局第一图」，同乙丑时者「共为一局第二图」；
    /// 乙丑时「以天盘甲子戊加乙于九宫，再加甲戌己于一宫……加乙奇于八宫，以蓬休符使加于离九宫」
    #[test]
    fn fei_pan_baojian_tu() {
        let o = PanOptions { method: PanMethod::FeiPan, ..DEFAULT_PAN };
        let d = di_pan(true, 1);
        // 第一图：天盘同地盘、符使同在本宫
        for h in ["甲子", "戊辰", "己卯", "庚寅", "辛丑", "壬子", "癸亥"] {
            let p = pai_pan(&sj(true, 1), gz(h), o);
            for q in 1..=9u8 {
                assert_eq!(p.at(q).tian_pan, vec![d[q as usize]], "{h} {q}");
            }
        }
        // 第二图：一己二庚三辛四壬五癸六丁七丙八乙九戊；甲戌时（甲作戊）与之同
        let tu2 = ["", "己", "庚", "辛", "壬", "癸", "丁", "丙", "乙", "戊"];
        for h in ["乙丑", "甲戌", "戊寅", "己丑", "庚子", "辛亥", "壬戌"] {
            let p = pai_pan(&sj(true, 1), gz(h), o);
            for q in 1..=9u8 {
                assert_eq!(p.at(q).tian_pan, vec![tu2[q as usize]], "{h} {q}");
            }
            // 符使同加时干之宫
            assert_eq!(p.zhi_fu_gong, p.zhi_shi_gong, "{h}");
        }
        // 乙丑：蓬休符使加离九
        let p = pai_pan(&sj(true, 1), gz("乙丑"), o);
        assert_eq!((p.zhi_fu, p.zhi_shi, p.zhi_fu_gong), ("天蓬", "休門", 9));
        // 转盘不受影响：甲戌时仍伏吟（时干甲遁本旬之仪）
        let z = pai_pan(&sj(true, 1), gz("甲戌"), DEFAULT_PAN);
        assert_eq!(z.zhi_fu_gong, z.xun_gong);
    }
}
