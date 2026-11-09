//! 大六壬课体：九宗门（卷一入手法九法）、本门课名（卷五各课），与可由盘面直接判定的常见课格（卷五至卷八）。
//!
//! [`ke_ge`] 只依日干支、四课、三传（及可选的十二天将、月建）判定；[`ke_ge_ctx`] 另收须论旺相、太岁月建、年命、
//! 神将吉凶的课格（三光、三陽、三陰、時泰、龍徳、官爵、富貴、軒蓋、鑄印、斵輪、引從、亨通、徳慶、斬闗、冲破、淫泆、侵害、
//! 刑傷、天獄、天網、龍戰、殃咎、鬼墓、勵徳、盤珠、六純），及须夫妻行年、节气日、月宿者（繁昌、榮華、合歡、和美、天禍、天寇、
//! 二煩、死奇、魄化、灾厄），以原文能判定者为限；未做者（迍福、物類）见 doc/folk/liuren.md。

use super::shen_sha::{ding_ma, gan_mu, ri_ben, ri_de, tian_ma, xun_shou, zhi_mu, MuRule};
use super::{ke, m12, xing, Fa, KeShi, TianJiang, GAN_JI, GAN_WX, ZHI_WX};

/// 九宗门（卷一「入手法」九法之名）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZongMen {
    ZeiKe,
    BiYong,
    SheHai,
    YaoKe,
    MaoXing,
    BieZe,
    BaZhuan,
    FuYin,
    FanYin,
}

impl ZongMen {
    pub fn name(&self) -> &'static str {
        match self {
            ZongMen::ZeiKe => "賊尅",
            ZongMen::BiYong => "比用",
            ZongMen::SheHai => "渉害",
            ZongMen::YaoKe => "遙尅",
            ZongMen::MaoXing => "昴星",
            ZongMen::BieZe => "别責",
            ZongMen::BaZhuan => "八專",
            ZongMen::FuYin => "伏吟",
            ZongMen::FanYin => "返吟",
        }
    }
}

impl Fa {
    /// 所属宗门
    pub fn zong_men(&self) -> ZongMen {
        match self {
            Fa::ChongShen | Fa::YuanShou => ZongMen::ZeiKe,
            Fa::ZhiYi => ZongMen::BiYong,
            Fa::SheHai | Fa::JianJi | Fa::ChaWei | Fa::ZhuiXia | Fa::BiYongGe => ZongMen::SheHai,
            Fa::HaoShi | Fa::DanShe => ZongMen::YaoKe,
            Fa::MaoXing => ZongMen::MaoXing,
            Fa::BieZe => ZongMen::BieZe,
            Fa::BaZhuan => ZongMen::BaZhuan,
            Fa::FuYin => ZongMen::FuYin,
            Fa::FanYin | Fa::JingLan => ZongMen::FanYin,
        }
    }
}

/// 本门课名（卷五）：昴星刚日「虎視轉蓬」、柔日「冬蛇掩目」；八專三传同为「獨足」；伏吟无克刚日「自任」柔日「自信」，
/// 初传自刑而中传取干支者「杜傳」（訂訛「發用干自刑則次傳取支，發用支自刑則次傳取干」）；返吟有克「無依」（訂訛「返吟亦名無依」）、无克「井欄」；
/// 余同 [`Fa::name`]
pub fn ke_ming(k: &KeShi, day_gz: u8) -> &'static str {
    let gang = (day_gz % 10) % 2 == 0;
    let [c0, c1, c2] = k.san_chuan;
    match k.fa {
        Fa::MaoXing => {
            if gang {
                "虎視"
            } else {
                "冬蛇掩目"
            }
        }
        Fa::BaZhuan if c0 == c1 && c1 == c2 => "獨足",
        Fa::FuYin => {
            if xing(c0) == c0 && c1 != c0 {
                "杜傳"
            } else if k.si_ke.iter().any(|x| x.zei() || x.ke_xia()) {
                "伏吟"
            } else if gang {
                "自任"
            } else {
                "自信"
            }
        }
        Fa::FanYin => "無依",
        _ => k.fa.name(),
    }
}

/// 一个课格：名与判定依据（卷、篇）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeGe {
    pub name: &'static str,
    pub note: &'static str,
}

/// 仲（子午卯酉）、孟（寅申巳亥）、季（辰戌丑未）
fn is_zhong(z: u8) -> bool {
    z % 3 == 0
}
fn is_meng(z: u8) -> bool {
    z % 3 == 2
}
fn is_ji(z: u8) -> bool {
    z % 3 == 1
}

/// 课格判定所需的盘外之数（无者为 None，则凡须之者不判）
#[derive(Clone, Copy, Debug)]
pub struct KeGeCtx<'a> {
    pub tj: Option<&'a TianJiang>,
    /// 月建地支（旺相、天马、天月德）
    pub yue_jian: Option<u8>,
    /// 太岁（占年地支）
    pub tai_sui: Option<u8>,
    /// 本命之支、行年之支
    pub ben_ming: Option<u8>,
    pub xing_nian: Option<u8>,
    pub mu: MuRule,
    /// 当令之行（[`super::shen_sha::dang_ling`]）；None 则依月建、季月全月土旺
    pub ling: Option<u8>,
    /// 本人行年、配偶行年之干支序（繁昌德孕须干合支合；旺孕只用支）
    pub xing_nian_gz: Option<u8>,
    pub pei_xing_nian_gz: Option<u8>,
    /// 今日为四立（立春、立夏、立秋、立冬）之日（天禍）
    pub si_li: bool,
    /// 今日为二分二至之日（天寇；離辰为前一日之支）
    pub fen_zhi: bool,
    /// 月宿所在之宫（地支序，[`yue_su_zhi`]；二煩、天寇、死奇）
    pub yue_su: Option<u8>,
}

impl Default for KeGeCtx<'_> {
    fn default() -> Self {
        KeGeCtx {
            tj: None,
            yue_jian: None,
            tai_sui: None,
            ben_ming: None,
            xing_nian: None,
            mu: MuRule::WuXing,
            ling: None,
            xing_nian_gz: None,
            pei_xing_nian_gz: None,
            si_li: false,
            fen_zhi: false,
            yue_su: None,
        }
    }
}

/// 二十八宿（角起）
pub const XIU28: [&str; 28] = [
    "角", "亢", "氐", "房", "心", "尾", "箕", "斗", "牛", "女", "虛", "危", "室", "壁", "奎", "婁", "胃", "昴", "畢", "觜", "參", "井", "鬼", "柳", "星", "張", "翼", "軫",
];

/// 起月宿法每月初一所起之宿（正月至十二月）。卷七二煩課訂訛与「起月宿法」歌「正室二奎三在胃四畢五參六鬼期七張八角九月氐十心一斗十二虚」；
/// 二煩課正文作「十尾」，与訂訛、歌异，从訂訛与歌
pub const YUE_SU_QI: [usize; 12] = [12, 14, 16, 18, 20, 22, 25, 0, 2, 4, 7, 10];

/// 起月宿法（卷七二煩課）：农历 `month`（1—12）`day`（1—30）月宿之宿序（角 0）。初一起本月之宿，「每日一宿遇奎張井翼氐斗重留一日」。
/// 书中诸例：正月十四柳、六月初二柳（二煩訂訛）、八月初五房（天寇訂訛）、九月初三房（二煩正文）、二月初五昴（天寇觀月經注）。
/// 闰月从本月数，未核
pub fn yue_su_qi_li(month: u8, day: u8) -> usize {
    let mut x = YUE_SU_QI[((month.clamp(1, 12)) - 1) as usize];
    let mut stay = false;
    for _ in 1..day.max(1) {
        if !stay && [14, 21, 25, 26, 2, 7].contains(&x) {
            stay = true;
        } else {
            stay = false;
            x = (x + 1) % 28;
        }
    }
    x
}

/// 月宿所屬十二辰（卷七「辰角亢　夘氐房心　寅尾箕　丑斗牛　子女虚危亥室壁　戌奎婁　酉胃昴畢　申觜參　未井鬼午栁星張　已翼軫」）
pub fn yue_su_zhi(xiu: usize) -> u8 {
    const Z: [u8; 28] = [4, 4, 3, 3, 3, 2, 2, 1, 1, 0, 0, 0, 11, 11, 10, 10, 9, 9, 9, 8, 8, 7, 7, 6, 6, 6, 5, 5];
    Z[xiu % 28]
}

/// 十二天将之五行（卷二各将注：贵人己丑土、螣蛇丁巳火、朱雀丙午火、六合乙卯木、勾陈戊辰土、青龙甲寅木、天空戊戌土、
/// 白虎庚申金、太常己未土、玄武癸亥水、太阴辛酉金、天后壬子水）
pub const JIANG_WX: [u8; 12] = [2, 1, 1, 0, 2, 0, 2, 3, 2, 4, 3, 4];
/// 吉将（卷二：贵人、六合、青龙、太常、太阴、天后为吉将，余为凶将）
pub fn ji_jiang(j: u8) -> bool {
    matches!(j, 0 | 3 | 5 | 8 | 10 | 11)
}

fn sheng(a: u8, b: u8) -> bool {
    (a + 1) % 5 == b
}

/// 破（卷一「破」：陽日破後四辰隂日破前四辰；午破卯、酉破子、辰破丑、亥破寅、申破巳……）
fn po(z: u8) -> u8 {
    if z % 2 == 0 {
        m12(z as i32 - 3)
    } else {
        m12(z as i32 + 3)
    }
}

/// 帝旺之神（亨通課「甲申日干上酉乃支之旺神支上卯乃干之旺神」）：木卯、火午、金酉、水子；土原书未举，不论
fn wang_shen(wx: u8) -> Option<u8> {
    match wx {
        0 => Some(3),
        1 => Some(6),
        3 => Some(9),
        4 => Some(0),
        _ => None,
    }
}

/// 由盘面判定常见课格。`tj` 十二天将（閉口、三交须用；无则不判此二者之将）；`yue_jian` 月建地支（遊子之天马、稼穡须用）
pub fn ke_ge(k: &KeShi, day_gz: u8, tj: Option<&TianJiang>, yue_jian: Option<u8>) -> Vec<KeGe> {
    ke_ge_ctx(k, day_gz, &KeGeCtx { tj, yue_jian, ..Default::default() })
}

/// 课格全判：[`ke_ge`] 诸格，加须论旺相、岁月、年命、神将吉凶者
pub fn ke_ge_ctx(k: &KeShi, day_gz: u8, ctx: &KeGeCtx) -> Vec<KeGe> {
    let tj = ctx.tj;
    let yue_jian = ctx.yue_jian;
    let mut v: Vec<KeGe> = Vec::new();
    let mut push = |name: &'static str, note: &'static str| v.push(KeGe { name, note });
    let g = day_gz % 10;
    let zhi = day_gz % 12;
    let pan = k.pan;
    let sc = k.san_chuan;
    let chu = sc[0];
    let in_sc = |z: u8| sc.contains(&z);
    let xs = xun_shou(day_gz);
    let gw = GAN_WX[g as usize];
    let ji = GAN_JI[g as usize];
    let gan_shang = k.si_ke[0].shang;
    let zhi_shang = k.si_ke[2].shang;

    // 三奇（卷五三竒課訂訛）：旬奇發用或入傳。甲子甲戌旬丑、甲申甲午旬子、甲辰甲寅旬亥；止有干奇不名三奇
    let xun_qi = match xs {
        0 | 10 => 1u8,
        8 | 6 => 0,
        _ => 11,
    };
    if in_sc(xun_qi) {
        push("三奇", "卷五三竒課：旬奇（甲子甲戌旬丑、甲申甲午旬子、甲辰甲寅旬亥）發用或入傳");
    }
    // 六儀（卷五六儀課訂訛）：旬儀（旬首）發用或入傳；止有支儀不名六儀
    if in_sc(xs) {
        push("六儀", "卷五六儀課：旬首之儀發用或入傳");
    }
    // 閉口（卷六閉口課）：旬尾加旬首發用；或天盤旬首乘玄武發用；或地盤旬首上神乘玄武發用
    let xun_wei = m12(xs as i32 + 9);
    let xuan_wu = |s: u8| tj.map_or(false, |t| t.at[s as usize] == 9);
    if (chu == xun_wei && pan.below(chu) == xs) || (chu == xs && xuan_wu(chu)) || (chu == pan.above(xs) && xuan_wu(chu)) {
        push("閉口", "卷六閉口課：旬尾加旬首發用，或旬首（天盤、地盤上神）乘玄武發用");
    }
    // 三传皆土：遊子（见旬丁或天马）、稼穡（无丁马）
    if sc.iter().all(|&s| is_ji(s)) {
        let dm = ding_ma(day_gz);
        let tm = yue_jian.map(tian_ma);
        if in_sc(dm) || tm.map_or(false, |t| in_sc(t)) {
            push("遊子", "卷六遊子課：三傳皆土，見旬丁或天馬");
        } else if tm.is_some() {
            push("稼穡", "卷八全局課稼穡訂訛：三傳辰戌丑未無丁馬");
        }
    }
    // 全局（卷八）：三传三合
    let mut s3 = sc;
    s3.sort();
    if s3[0] != s3[1] && s3[1] != s3[2] && sc.iter().all(|&s| s % 4 == sc[0] % 4) {
        let name = match sc[0] % 4 {
            0 => "潤下",
            1 => "從革",
            2 => "炎上",
            _ => "曲直",
        };
        push(name, "卷八全局課：三傳三合（申子辰潤下、寅午戌炎上、亥卯未曲直、巳酉丑從革）");
    }
    // 玄胎（卷八）：三传皆四孟
    if sc.iter().all(|&s| is_meng(s)) {
        push("玄胎", "卷八𤣥胎課：三傳皆四孟");
    }
    // 連珠（卷八）：三传相连（进连茹、退连茹）
    if sc[1] == m12(sc[0] as i32 + 1) && sc[2] == m12(sc[1] as i32 + 1) {
        push("進連茹", "卷八連珠課：三傳一方相連（順）");
    } else if sc[1] == m12(sc[0] as i32 - 1) && sc[2] == m12(sc[1] as i32 - 1) {
        push("退連茹", "卷八連珠課：三傳一方相連（逆，又名失友格）");
    }
    // 三交（卷六三交課）：四仲日，仲神加支，三传皆仲，传逢太阴、六合
    if is_zhong(zhi) && is_zhong(zhi_shang) && sc.iter().all(|&s| is_zhong(s)) {
        let yin_he = tj.map_or(true, |t| sc.iter().any(|&s| matches!(t.at[s as usize], 3 | 10)));
        if yin_he {
            push("三交", "卷六三交課：四仲日四仲加支，三傳皆仲，逢太陰六合");
        }
    }
    // 亂首、贅壻（卷一課目、卷六）：干支互加，论其克
    let zw = ZHI_WX[zhi as usize];
    if zhi_shang == ji || gan_shang == zhi {
        // 干加支上（支上神为干寄宫）或支加干上（干上神为日支）
        if ke(zw, gw) {
            push("亂首", "卷一課目：支加干尅干，干加支上被尅");
        } else if ke(gw, zw) {
            push("贅壻", "卷一課目、卷六贅壻課：支臨干被尅，干加支上尅支");
        }
    }
    // 蕪淫（卷七）：四课不备（三课）而有克，或干上神克支、支上神克干（日辰交互相克）
    let mut distinct: Vec<(u8, u8)> = Vec::new();
    for x in &k.si_ke {
        if !distinct.contains(&(x.xia_zhi, x.shang)) {
            distinct.push((x.xia_zhi, x.shang));
        }
    }
    let you_ke = k.si_ke.iter().any(|x| x.zei() || x.ke_xia());
    let jiao_ke = ke(ZHI_WX[gan_shang as usize], zw) && ke(ZHI_WX[zhi_shang as usize], gw);
    if (distinct.len() == 3 && you_ke) || jiao_ke {
        push("蕪淫", "卷七蕪淫課：四課不備而有克，或日辰交互相克");
    }
    // 度厄、無祿、絶嗣（卷七）
    let n_ke = k.si_ke.iter().filter(|x| x.ke_xia()).count();
    let n_zei = k.si_ke.iter().filter(|x| x.zei()).count();
    if n_ke == 4 {
        push("無祿", "卷一課目、卷七正文：四上克下（卷七訂訛作絶嗣，与課目、正文相反）");
    } else if n_ke == 3 {
        push("幼度厄", "卷七度厄課：三上克下");
    }
    if n_zei == 4 {
        push("絶嗣", "卷一課目、卷七正文：四下賊上（卷七訂訛作無祿，与課目、正文相反）");
    } else if n_zei == 3 {
        push("長度厄", "卷七度厄課：三下賊上");
    }
    // 九醜（卷八）：乙戊己辛壬配子午卯酉之十日，丑加四仲发用，或丑临支上
    if [1u8, 4, 5, 7, 8].contains(&g) && is_zhong(zhi) && ((chu == 1 && is_zhong(pan.below(1))) || zhi_shang == 1) {
        push("九醜", "卷八九醜課：戊子戊午壬子壬午乙卯乙酉己卯己酉辛卯辛酉日，丑臨四仲發用或臨支上");
    }

    // ———— 以下须论旺相、岁月、年命、神将 ————
    let lin = pan.below(chu);
    let mo = sc[2];
    let cw = ZHI_WX[chu as usize];
    let gang = g % 2 == 0;
    let at = |s: u8| tj.map(|t| t.at[s as usize]);
    // 旺相：当令之行由调用方给（土旺十八日之说）则用之，否则依月建
    let ling = ctx.ling.or(yue_jian.map(|y| super::shen_sha::dang_ling(y, super::shen_sha::WangXiangRule::JiYue, false)));
    let ws = |w: u8| ling.map(|l| super::shen_sha::wang_shuai_ling(w, l));
    let qi = |w: u8| ws(w).map(|x| x <= 1);
    // 贵人前后：顺布者地盘顺数、逆布者逆数，距贵人所临 1—5 为前、7—11 为后（卷二「前有五位……後有五位」）
    let dist = |d: u8| tj.map(|t| if t.shun { m12(d as i32 - t.gui_lin as i32) } else { m12(t.gui_lin as i32 - d as i32) });
    let front = |d: u8| dist(d).map_or(false, |x| (1..=5).contains(&x));
    let behind = |d: u8| dist(d).map_or(false, |x| (7..=11).contains(&x));

    // 三光（卷五三光課訂訛「日辰發用者旺相乘吉將」）：日、辰、初传旺相，初传乘吉将。日辰之旺相，觀月經注以干支本身论
    // （「如春甲乙夏丙丁之類」），袖中金例（六月戊寅日午時寅將）寅本身不旺而书仍名三光；此处干支本身或其上神旺相即算
    let ri_qi = qi(gw) == Some(true) || qi(ZHI_WX[gan_shang as usize]) == Some(true);
    let chen_qi = qi(zw) == Some(true) || qi(ZHI_WX[zhi_shang as usize]) == Some(true);
    if ri_qi && chen_qi && qi(cw) == Some(true) && at(chu).map_or(false, ji_jiang) {
        push("三光", "卷五三光課訂訛：日、辰（干支或其上神）、發用三處旺相，乘吉將");
    }
    // 三陽（卷五三陽課訂訛「天乙順行日辰居前發用旺相」）
    if tj.map_or(false, |t| t.shun) && front(ji) && front(zhi) && qi(cw) == Some(true) {
        push("三陽", "卷五三陽課訂訛：天乙順行，日辰（干寄宫、日支）居貴人前，發用旺相");
    }
    // 三陰（卷七三陰課訂訛「貴人逆治日辰在後一也發用傳終名帶囚死二也將乘𤣥虎時克行年三也」）：须行年
    if let (Some(t), Some(xn)) = (tj, ctx.xing_nian) {
        let qs = ws(cw).map_or(false, |x| x == 3 || x == 4);
        if !t.shun && behind(ji) && behind(zhi) && qs && matches!(t.at[chu as usize], 7 | 9) && ke(ZHI_WX[pan.shi as usize], ZHI_WX[xn as usize]) {
            push("三陰", "卷七三陰課訂訛：貴人逆治、日辰在後，發用囚死乘玄武白虎，占時克行年");
        }
    }
    // 時泰（卷五時泰課「用起太嵗月建乘青龍六合又帶財徳之神」；訂訛「太嵗月建作日財徳太嵗發用更佳入傳亦可」）
    {
        let jian = [ctx.tai_sui, yue_jian];
        let hit = sc.iter().any(|&c| {
            jian.contains(&Some(c)) && (ke(gw, ZHI_WX[c as usize]) || c == ri_de(g)) && matches!(at(c), Some(3) | Some(5))
        });
        if hit {
            push("時泰", "卷五時泰課：太嵗或月建作日財、日徳入傳，乘青龍六合");
        }
    }
    // 龍徳（卷五龍徳課「太嵗月將乘貴人發用」；心鏡「太嵗今朝作貴人立用還須月將神」）
    if ctx.tai_sui == Some(chu) && at(chu) == Some(0) && in_sc(pan.jiang) {
        push("龍徳", "卷五龍徳課：太嵗乘貴人發用，三傳見月將");
    }
    // 官爵（卷六官爵課「嵗月年命驛馬發用又天魁太常入傳」；訂訛「或驛馬發用印綬入傳或印綬發用驛馬入傳」）
    if tj.is_some() {
        let mut ma: Vec<u8> = vec![super::shen_sha::yi_ma(zhi)];
        for x in [ctx.tai_sui, yue_jian, ctx.ben_ming, ctx.xing_nian].into_iter().flatten() {
            ma.push(super::shen_sha::yi_ma(x));
        }
        let chang = sc.iter().any(|&c| at(c) == Some(8));
        let a = ma.contains(&chu) && in_sc(10) && chang;
        let b = chu == 10 && sc[1..].iter().any(|c| ma.contains(c)) && chang;
        if a || b {
            push("官爵", "卷六官爵課：驛馬（日、嵗、月、年命之馬）發用，天魁、太常入傳；或天魁發用、驛馬入傳");
        }
    }
    // 富貴（卷六富貴課「天乙乘旺相氣上下相生更臨日辰年命發用」）
    if at(chu) == Some(0) && qi(cw) == Some(true) {
        let lw = ZHI_WX[lin as usize];
        let lin_ok = lin == ji || lin == zhi || ctx.ben_ming == Some(lin) || ctx.xing_nian == Some(lin);
        if (sheng(cw, lw) || sheng(lw, cw)) && lin_ok {
            push("富貴", "卷六富貴課：天乙發用乘旺相，上下相生，臨日辰年命");
        }
    }
    // 軒蓋（卷六「勝光為用遇太冲神后」；卷一「軒蓋三傳午夘子」）
    if chu == 6 && sc[1..].contains(&3) && sc[1..].contains(&0) {
        push("軒蓋", "卷六軒盖課、卷一課目：勝光發用，傳遇太冲神后（午卯子）");
    }
    // 鑄印（卷六鑄印課訂訛「三傳巳戌卯」）
    if sc == [5, 10, 3] {
        push("鑄印", "卷六鑄印課訂訛：三傳巳戌卯");
    }
    // 斵輪（卷六斵輪課訂訛「卯加庚辛申酉發用」：庚寄申、辛寄戌）
    if chu == 3 && matches!(lin, 8 | 9 | 10) {
        push("斵輪", "卷六斵輪課：卯加庚辛（申、戌）或申酉發用");
    }
    // 引從（卷六引從課訂訛「初末傳拱干……初末傳拱支」：初传临干支前一位、末传临后一位，如庚辰日初寅加酉、末子加未）
    let lin_mo = pan.below(mo);
    if lin == m12(ji as i32 + 1) && lin_mo == m12(ji as i32 - 1) {
        push("引從", "卷六引從課：初傳臨干前、末傳臨干後，拱干");
    }
    if lin == m12(zhi as i32 + 1) && lin_mo == m12(zhi as i32 - 1) {
        push("引從", "卷六引從課：初傳臨支前、末傳臨支後，拱支");
    }
    // 亨通（卷六亨通課訂訛：遞生、俱生、互生、俱旺、互旺）
    {
        let [w0, w1, w2] = sc.map(|c| ZHI_WX[c as usize]);
        let gsw = ZHI_WX[gan_shang as usize];
        let zsw = ZHI_WX[zhi_shang as usize];
        let di = (sheng(w0, w1) && sheng(w1, w2) && sheng(w2, gw)) || (sheng(w2, w1) && sheng(w1, w0) && sheng(w0, gw));
        let ju = sheng(gsw, gw) && sheng(zsw, zw);
        let hu = sheng(gsw, zw) && sheng(zsw, gw);
        let ju_w = wang_shen(gw) == Some(gan_shang) && wang_shen(zw) == Some(zhi_shang);
        let hu_w = wang_shen(zw) == Some(gan_shang) && wang_shen(gw) == Some(zhi_shang);
        let sub = if di {
            Some("卷六亨通課：三傳遞生日干")
        } else if ju {
            Some("卷六亨通課：干上生干、支上生支（俱生）")
        } else if hu {
            Some("卷六亨通課：干上生支、支上生干（互生）")
        } else if ju_w {
            Some("卷六亨通課：干上干之旺神、支上支之旺神（俱旺）")
        } else if hu_w {
            Some("卷六亨通課：干上支之旺神、支上干之旺神（互旺）")
        } else {
            None
        };
        if let Some(n) = sub {
            push("亨通", n);
        }
    }
    // 徳慶（卷六徳慶課「日辰干支徳神及天月二徳發用……乘吉將」）
    if let Some(a) = at(chu) {
        let mut de = vec![ri_de(g), m12(zhi as i32 + 5)];
        if let Some(y) = yue_jian {
            let yi = m12(y as i32 - 2) as usize; // 正月 0
            // 天德：正丁二坤三壬四辛五乾六甲七癸八艮九丙十乙十一巽十二庚（干取寄宫）
            de.push([7, 8, 11, 10, 11, 2, 1, 2, 5, 4, 5, 8][yi]);
            // 月德：寅午戌月丙、亥卯未月甲、申子辰月壬、巳酉丑月庚（取寄宫）
            de.push([11, 8, 5, 2][(y % 4) as usize]);
        }
        if de.contains(&chu) && ji_jiang(a) {
            push("徳慶", "卷六徳慶課：干徳、支徳（子日起巳順行）、天徳、月徳發用，乘吉將");
        }
    }
    // 斬闗（卷六斬闗課「魁罡加日辰發用」）
    if (chu == 4 || chu == 10) && (lin == ji || lin == zhi) {
        push("斬闗", "卷六斬闗課：魁罡加日辰發用");
    }
    // 冲破（卷六冲破課「日辰之冲神加破為用」：如庚子日午加卯，午冲子，卯為午破）
    if chu == m12(zhi as i32 + 6) && lin == po(chu) {
        push("冲破", "卷六冲破課：日支之冲神加其破發用");
    }
    // 淫泆（卷六淫泆課「初傳卯酉為用將乘后合」；又「用起六合終於天后為狡童格」「用起天后神終傳六合神為泆女格」，
    // 书举泆女例戊戌日辰時午將子加戌發用，不必卯酉）
    match (at(chu), at(mo)) {
        (Some(3), Some(11)) => push("淫泆", "卷六淫泆課：初傳六合、末傳天后（狡童格）"),
        (Some(11), Some(3)) => push("淫泆", "卷六淫泆課：初傳天后、末傳六合（泆女格）"),
        (Some(3) | Some(11), _) if chu == 3 || chu == 9 => push("淫泆", "卷六淫泆課：卯酉發用，乘天后六合"),
        _ => {}
    }
    // 侵害（卷七侵害課訂訛「干支害神上下相加發用臨行年更的」）：初传与所临六害
    if (chu as i32 + lin as i32) % 12 == 7 {
        push("侵害", "卷七侵害課：六害上下相加發用（臨行年更的）");
    }
    // 刑傷（卷七刑傷課「三刑發用併行年」）：初传与所临相刑
    if chu != lin && (xing(chu) == lin || xing(lin) == chu) {
        push("刑傷", "卷七刑傷課：三刑上下相加發用（併行年更的）");
    }
    // 天獄（卷七天獄課「囚死墓神發用斗係日本」；訂訛「用死囚斗繫日本」）
    {
        let qs = ws(cw).map_or(false, |x| x == 3 || x == 4);
        if (qs || chu == gan_mu(g, ctx.mu)) && pan.above(ri_ben(g)) == 4 {
            push("天獄", "卷七天獄課：囚死或日墓發用，天罡加日本（日干長生）");
        }
    }
    // 天網（卷一課目「天網時用俱尅日」）
    if ke(cw, gw) && ke(ZHI_WX[pan.shi as usize], gw) {
        push("天網", "卷一課目、卷七：占時與發用俱克日干");
    }
    // 龍戰（卷七龍戰課訂訛「夘酉日占夘酉上神發用」）
    if (zhi == 3 || zhi == 9) && (chu == 3 || chu == 9 || lin == 3 || lin == 9) {
        push("龍戰", "卷七龍戰課：卯酉日卯酉發用或卯酉上神發用（行年卯酉更的）");
    }
    // 殃咎（卷八殃咎課訂訛「遞克夾克外戰内戰乘墓坐墓皆是」）
    {
        let [w0, w1, w2] = sc.map(|c| ZHI_WX[c as usize]);
        let di = (ke(w0, w1) && ke(w1, w2) && ke(w2, gw)) || (ke(w2, w1) && ke(w1, w0) && ke(w0, gw));
        let gm = gan_mu(g, ctx.mu);
        let zm = zhi_mu(zhi);
        let cheng = gan_shang == gm && zhi_shang == zm;
        let zuo = pan.below(ji) == gm && pan.below(zhi) == zm;
        let jw = |c: u8| at(c).map(|j| JIANG_WX[j as usize]);
        let jia = jw(chu).map_or(false, |j| ke(ZHI_WX[lin as usize], cw) && ke(j, cw));
        let wai = sc.iter().all(|&c| jw(c).map_or(false, |j| ke(j, ZHI_WX[c as usize])));
        let nei = sc.iter().all(|&c| jw(c).map_or(false, |j| ke(ZHI_WX[c as usize], j)));
        for (ok, n) in [
            (di, "卷八殃咎課：三傳遞克日干"),
            (jia, "卷八殃咎課訂訛：夾克（用神坐下克上又受將克）"),
            (wai, "卷八殃咎課訂訛：外戰（三傳皆將克神）"),
            (nei, "卷八殃咎課訂訛：内戰（三傳皆神克將）"),
            (cheng, "卷八殃咎課訂訛：干支乘墓（干上干墓、支上支墓）"),
            (zuo, "卷八殃咎課訂訛：干支坐墓（干支各臨墓神）"),
        ] {
            if ok {
                push("殃咎", n);
                break;
            }
        }
    }
    // 鬼墓（卷八鬼墓課訂訛「干支上神發用或作干鬼干墓或作支鬼支墓」；鬼「陽見陽陰見陰」）
    if chu == gan_shang || chu == zhi_shang {
        let gui_of = |w: u8, yang: bool| ke(cw, w) && ((chu % 2 == 0) == yang);
        let gm = chu == gan_mu(g, ctx.mu) || chu == zhi_mu(zhi);
        if gm || gui_of(gw, gang) || gui_of(zw, zhi % 2 == 0) {
            push("鬼墓", "卷八鬼墓課：干支上神發用，作干支之鬼（陽見陽陰見陰）或墓");
        }
    }
    // 勵徳（卷八勵徳課「天乙立夘酉」）
    if tj.map_or(false, |t| t.gui_lin == 3 || t.gui_lin == 9) {
        push("勵徳", "卷八勵徳課：貴人臨卯酉");
    }
    // 盤珠（卷八盤珠課「太嵗月建及日時并三傳皆在四課之中」）
    if let (Some(ts), Some(yj)) = (ctx.tai_sui, yue_jian) {
        let mut ks: Vec<u8> = vec![zhi];
        for x in &k.si_ke {
            ks.push(x.shang);
            ks.push(x.xia_zhi);
        }
        if [ts, yj, zhi, pan.shi].iter().chain(sc.iter()).all(|z| ks.contains(z)) {
            push("盤珠", "卷八盤珠課：太嵗、月建、日、時與三傳皆在四課之中");
        }
    }
    // 六純（卷八六純課「四課三傳俱陽……俱陰」）：日干、日支、四课上神、三传阴阳皆同
    {
        let par = g % 2;
        let mut all: Vec<u8> = vec![zhi % 2];
        all.extend(k.si_ke.iter().map(|x| x.shang % 2));
        all.extend(sc.iter().map(|c| c % 2));
        if all.iter().all(|&p| p == par) {
            push("六純", if par == 0 { "卷八六純課：四課三傳俱陽" } else { "卷八六純課：四課三傳俱陰" });
        }
    }

    // ———— 以下须年命、行年干支、节气日、月宿 ————
    let liu_he = |a: u8, b: u8| (a + b) % 12 == 1;
    let san_he = |a: u8, b: u8| a != b && a % 4 == b % 4;
    let he = |a: u8, b: u8| liu_he(a, b) || san_he(a, b);
    let gan_he = |a: u8, b: u8| (a as i32 - b as i32).rem_euclid(10) == 5;
    let ming_shang: Vec<u8> = [ctx.ben_ming, ctx.xing_nian].into_iter().flatten().map(|d| pan.above(d)).collect();
    // 繁昌（卷六繁昌課「夫妻行年乘本命旺相氣又值干支德合或年立時令旺相之鄉」）：
    // 德孕格——夫妻行年干合、支六合，二年上神俱旺相（例壬申日未時巳將，夫行年甲寅上子、妻行年己亥上酉，七月金旺水相）；
    // 旺孕格——夫妻行年三合異位而俱旺相（心鏡「春占……妻午夫年立在寅」、曾門「夫妻之年同類異位」）
    if let (Some(a), Some(b)) = (ctx.xing_nian_gz, ctx.pei_xing_nian_gz) {
        let (za, zb) = (a % 12, b % 12);
        let shang_qi = qi(ZHI_WX[pan.above(za) as usize]) == Some(true) && qi(ZHI_WX[pan.above(zb) as usize]) == Some(true);
        if gan_he(a % 10, b % 10) && liu_he(za, zb) && shang_qi {
            push("繁昌", "卷六繁昌課德孕格：夫妻行年干合支合，年上神俱旺相");
        } else if san_he(za, zb) && qi(ZHI_WX[za as usize]) == Some(true) && qi(ZHI_WX[zb as usize]) == Some(true) {
            push("繁昌", "卷六繁昌課旺孕格（心鏡、曾門）：夫妻行年三合異位，俱立旺相之鄉");
        }
    }
    // 榮華（卷六榮華課「禄馬貴人臨干支年命併旺相氣發用入傳更乘吉將」；例丙申日卯時子將干上寅馬、支上巳禄，巳相氣發用，末傳亥貴人）：
    // 日禄、驿马俱见于干上、支上、年命上；初传为禄马贵人之一而旺相；贵人入传或临干支年命。贵人自为吉将；书例初传巳乘天空，
    // 「乘吉將」不作初传之必要条件
    if let Some(t) = tj {
        let lu = super::shen_sha::ri_lu(g);
        let ma = super::shen_sha::yi_ma(zhi);
        let mut wei: Vec<u8> = vec![gan_shang, zhi_shang];
        wei.extend(ming_shang.iter().copied());
        let gui = t.gui;
        if wei.contains(&lu) && wei.contains(&ma) && [lu, ma, gui].contains(&chu) && qi(cw) == Some(true) && (in_sc(gui) || wei.contains(&gui)) {
            push("榮華", "卷六榮華課：禄馬臨干支年命，禄馬貴人旺相發用，貴人入傳或臨干支年命");
        }
    }
    // 合歡（卷一課目「合歡日上遁干合　吉将三六合用兼」；卷六「日辰遇天干作合及支三合六合發用併占人年命俱乘吉將」，
    // 例戊申日子時申將：干上丑遁癸合戊，子與干上丑六合發用、與支申三合，乘青龍；亥命上未貴人、行年辰上子青龍）
    if let Some(t) = tj {
        let dun = super::shen_sha::dun_gan(day_gz, gan_shang, super::shen_sha::DunRule::XunDun);
        let dun_he = dun.map_or(false, |d| gan_he(d, g));
        let yong_he = liu_he(chu, gan_shang) || liu_he(chu, zhi_shang) || san_he(chu, zhi);
        let ming_ji = ming_shang.iter().all(|&s| ji_jiang(t.at[s as usize]));
        if dun_he && yong_he && ji_jiang(t.at[chu as usize]) && ming_ji {
            push("合歡", "卷六合歡課：干上神遁干與日干合，發用與干支上神六合或與日支三合，乘吉將（年命上神亦乘吉將）");
        }
    }
    // 和美（卷六和美課「干支遇三合六合上下遞互相合」；卷一「和美專言四課事　各合互合皆為歡」）：干上神與干寄宫、支上神與日支（各合），
    // 或干上神與日支、支上神與干寄宫（互合），或干支上神相合，三合六合俱論。书例辛卯日卯加辛、壬寅日亥加寅、甲申日干上亥支上巳、
    // 丁丑己丑日干上午、乙丑日干上子、乙酉丙申日伏吟、壬午日巳時丑將
    if he(gan_shang, ji) || he(zhi_shang, zhi) || he(gan_shang, zhi) || he(zhi_shang, ji) || he(gan_shang, zhi_shang) {
        push("和美", "卷六和美課：干支與上神各合、互合或干支上神相合（三合六合）");
    }
    // 天禍（卷七天禍課「四立日占得今日干支臨昨日干支或昨日干支臨今日干支」；卷一「昨日之干加今干」）：干以寄宫论
    if ctx.si_li {
        let y = (day_gz + 59) % 60;
        let (jy, zy) = (GAN_JI[(y % 10) as usize], y % 12);
        if pan.below(ji) == jy || pan.below(jy) == ji || pan.below(zhi) == zy || pan.below(zy) == zhi {
            push("天禍", "卷七天禍課：四立日，今日干支臨昨日（四絶日）干支或昨日干支臨今日干支");
        }
    }
    // 天寇（卷七天冦課訂訛「分至日占得月宿臨離辰……分至前一日為離」；例八月初五丁酉秋分辰將酉時，月宿房在卯加申）
    if let (true, Some(ys)) = (ctx.fen_zhi, ctx.yue_su) {
        if pan.below(ys) == m12(zhi as i32 - 1) {
            push("天寇", "卷七天冦課：分至日，月宿加離辰（前一日之支）；發用更甚");
        }
    }
    // 二煩（卷七二煩課訂訛「日宿臨仲斗繫丑未曰天煩月宿臨仲斗繫丑未曰地煩日月宿俱臨仲斗繫丑未日天地二煩」）：日宿即月将；
    // 斗不繫丑未者名杜傳（「日月宿臨仲斗不繫丑未又名杜傳」）。訂訛又云「日月宿不發用者不真」，此处不以發用为必要，注明之
    {
        let dou = pan.below(4);
        let tian = is_zhong(pan.below(pan.jiang));
        let di = ctx.yue_su.map_or(false, |ys| is_zhong(pan.below(ys)));
        let fa = chu == pan.jiang || ctx.yue_su == Some(chu);
        if (tian || di) && (dou == 1 || dou == 7) {
            push(
                "二煩",
                match (tian, di, fa) {
                    (true, true, true) => "卷七二煩課：日宿、月宿俱臨仲，斗繫丑未（天地二煩），宿發用",
                    (true, true, false) => "卷七二煩課：日宿、月宿俱臨仲，斗繫丑未（天地二煩）；宿不發用，訂訛謂不真",
                    (true, false, true) => "卷七二煩課：日宿（月將）臨仲，斗繫丑未（天煩），發用",
                    (true, false, false) => "卷七二煩課：日宿（月將）臨仲，斗繫丑未（天煩）；宿不發用，訂訛謂不真",
                    (false, _, true) => "卷七二煩課：月宿臨仲，斗繫丑未（地煩），發用",
                    _ => "卷七二煩課：月宿臨仲，斗繫丑未（地煩）；宿不發用，訂訛謂不真",
                },
            );
        } else if tian && chu == pan.jiang {
            push("天煩杜傳", "卷七二煩課：日宿臨仲發用，斗不繫丑未（杜傳）");
        } else if di && fa {
            push("地煩杜傳", "卷七二煩課：月宿臨仲發用，斗不繫丑未（杜傳）");
        }
    }
    // 死奇（卷七死奇課「斗罡係日辰隂陽發用」；訂訛注「天罡發用月行度到角亢之分或月宿臨太嵗日辰皆是」）
    if chu == 4 {
        let ke_shang = k.si_ke.iter().any(|x| x.shang == 4);
        let yue = ctx.yue_su.map_or(false, |ys| ys == 4 || [Some(ji), Some(zhi), ctx.tai_sui].contains(&Some(pan.below(ys))));
        if ke_shang || yue {
            push("死奇", "卷七死奇課：天罡加日辰陰陽發用，或天罡發用而月行角亢、月宿臨嵗日辰");
        }
    }
    // 魄化（卷七魄化課「白虎帶死神死氣臨日辰行年發用」，死神正月起巳、死氣正月起午俱順行；訂訛又「及囚死」「墓乘虎作鬼加日亦是」；
    // 卷一「魄化死囚帶白虎　干支年用凶禍連」）：白虎所乘之神为死神、死氣或時令囚死，在日辰陰陽（四课上神）、行年上或發用
    if let (Some(t), Some(yj)) = (tj, yue_jian) {
        if let Some(s) = (0..12u8).find(|&s| t.at[s as usize] == 7) {
            let mi = m12(yj as i32 - 2) as i32;
            let si_shen = m12(5 + mi);
            let si_qi = m12(6 + mi);
            let qs = ws(ZHI_WX[s as usize]).map_or(false, |x| x == 3 || x == 4);
            let wei = k.si_ke.iter().any(|x| x.shang == s) || ctx.xing_nian.map_or(false, |x| pan.above(x) == s) || chu == s;
            let mu_gui = s == gan_mu(g, ctx.mu) && ke(ZHI_WX[s as usize], gw) && s == gan_shang;
            if ((s == si_shen || s == si_qi || qs) && wei) || mu_gui {
                push("魄化", "卷七魄化課：白虎乘死神、死氣或囚死之神，臨日辰陰陽、行年或發用（或日墓作鬼乘虎加干）");
            }
        }
    }
    // 灾厄（卷七灾厄課「喪車逰魂伏殃病符喪弔丘墓嵗虎發用」）：喪車正月起未逆行四季、逰魂正月起亥順行、伏殃正月起酉逆行四仲、
    // 病符舊太嵗、喪門嵗前二辰、弔客嵗後二辰、丘墓日干之墓、嵗虎嵗後四辰（例亥年正月乙亥日卯時，未為喪車、嵗虎發用）
    if let Some(yj) = yue_jian {
        let mi = m12(yj as i32 - 2) as i32;
        let mut shen: Vec<(u8, &'static str)> = vec![
            (m12(7 - 3 * mi), "卷七灾厄課：喪車（正月起未逆行四季）發用"),
            (m12(11 + mi), "卷七灾厄課：逰魂（正月起亥順行）發用"),
            (m12(9 - 3 * mi), "卷七灾厄課：伏殃（正月起酉逆行四仲）發用"),
            (gan_mu(g, ctx.mu), "卷七灾厄課：丘墓（日干之墓）發用"),
        ];
        if let Some(ts) = ctx.tai_sui {
            shen.push((m12(ts as i32 - 1), "卷七灾厄課：病符（舊太嵗）發用"));
            shen.push((m12(ts as i32 + 2), "卷七灾厄課：喪門（嵗前二辰）發用"));
            shen.push((m12(ts as i32 - 2), "卷七灾厄課：弔客（嵗後二辰）發用"));
            shen.push((m12(ts as i32 - 4), "卷七灾厄課：嵗虎（嵗後四辰）發用"));
        }
        if let Some((_, n)) = shen.iter().find(|(z, _)| *z == chu) {
            push("灾厄", n);
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::super::{qi_ke, tian_jiang, GuiRule, GAN, ZHI};
    use super::super::shen_sha::MuRule;
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
    fn sc(s: &str) -> [u8; 3] {
        let v: Vec<u8> = s.chars().map(|c| z(&c.to_string())).collect();
        [v[0], v[1], v[2]]
    }
    fn names(v: &[KeGe]) -> Vec<&'static str> {
        v.iter().map(|x| x.name).collect()
    }

    /// 本门课名：卷五各课之例
    #[test]
    fn ke_ming_examples() {
        let cases: &[(&str, &str, &str, &str, &str, &str)] = &[
            // (出处, 日, 时, 将, 三传, 课名)
            ("卷五昴星虎視", "戊申", "卯", "辰", "戌酉午", "虎視"),
            ("卷五昴星冬蛇掩目", "丁丑", "辰", "丑", "子辰戌", "冬蛇掩目"),
            ("卷五八專獨足", "己未", "未", "酉", "酉酉酉", "獨足"),
            ("卷五八專", "甲寅", "辰", "丑", "丑亥亥", "八專"),
            ("卷五伏吟（六癸日有克）", "癸巳", "午", "午", "丑戌未", "伏吟"),
            ("卷五伏吟自任格", "丙辰", "申", "申", "巳申寅", "自任"),
            ("卷五伏吟自信（丁丑用丑）", "丁丑", "未", "未", "丑戌未", "自信"),
            ("卷五杜傳格", "壬辰", "酉", "酉", "亥辰戌", "杜傳"),
            ("卷五返吟無依", "庚戌", "寅", "申", "寅申寅", "無依"),
            ("卷五返吟井欄射", "辛丑", "巳", "亥", "亥未辰", "井欄"),
            ("卷五元首課", "甲子", "卯", "子", "午卯子", "元首"),
            ("卷五重審課", "丙戌", "巳", "申", "申亥寅", "重審"),
            ("卷五知一課", "壬辰", "巳", "辰", "戌酉申", "知一"),
            ("卷五蒿矢", "壬辰", "巳", "申", "戌丑辰", "蒿矢"),
            ("卷五彈射", "壬申", "申", "亥", "巳申亥", "彈射"),
        ];
        for (src, d, s, j, want_sc, name) in cases {
            let k = qi_ke(gz(d), z(j), z(s));
            assert_eq!(k.san_chuan, sc(want_sc), "{src}");
            assert_eq!(ke_ming(&k, gz(d)), *name, "{src}");
        }
        // 杜傳：卷五「壬午日三傳亥午子、乙亥日辰亥巳、乙酉日辰酉卯」
        for (d, want) in [("壬午", "亥午子"), ("乙亥", "辰亥巳"), ("乙酉", "辰酉卯")] {
            let k = qi_ke(gz(d), 0, 0);
            assert_eq!(k.san_chuan, sc(want), "{d}");
            assert_eq!(ke_ming(&k, gz(d)), "杜傳", "{d}");
        }
        // 宗门
        assert_eq!(Fa::JingLan.zong_men().name(), "返吟");
        assert_eq!(Fa::ChaWei.zong_men().name(), "渉害");
        assert_eq!(Fa::YuanShou.zong_men().name(), "賊尅");
    }

    /// 课格：卷五至卷八各课所举之例。(出处, 日, 时, 将, 三传（空串不核）, 书中昼夜（None 不用天将）, 月建, 应有之格)
    #[test]
    fn ke_ge_examples() {
        type Case = (&'static str, &'static str, &'static str, &'static str, &'static str, Option<(GuiRule, bool)>, Option<&'static str>, &'static str);
        let cases: &[Case] = &[
            ("卷五三竒課（三傳亥子丑，甲申旬子）", "乙酉", "未", "申", "亥子丑", None, None, "三奇"),
            ("卷五六儀課（寅為旬儀）", "丙辰", "寅", "未", "寅未子", None, None, "六儀"),
            // 閉口：甲申日巳加申、丙辰日亥加寅（旬尾加旬首）；丁酉日午加酉夜將（天盤旬首乘玄武）；甲子日辰加子晝夜皆（地盤旬首上神乘玄武）
            ("卷六閉口課 甲申日巳加申", "甲申", "申", "巳", "", None, None, "閉口"),
            ("卷六閉口課 丙辰日亥加寅", "丙辰", "寅", "亥", "", None, None, "閉口"),
            // 书言「夜將」，俗例须以昼贵得之、《考原》夜贵得之（俗例与考原除戊日外昼夜对调）
            ("卷六閉口課 丁酉日午加酉", "丁酉", "酉", "午", "", Some((GuiRule::KaoYuan, false)), None, "閉口"),
            ("卷六閉口課 甲子日辰加子（昼）", "甲子", "子", "辰", "", Some((GuiRule::SuLi, true)), None, "閉口"),
            ("卷六閉口課 甲子日辰加子（夜）", "甲子", "子", "辰", "", Some((GuiRule::SuLi, false)), None, "閉口"),
            ("卷六遊子課（三月將、乙巳日午時）", "乙巳", "午", "酉", "未戌丑", None, Some("辰"), "遊子"),
            ("卷六三交課（卯乘太陰）", "戊子", "午", "酉", "卯午酉", Some((GuiRule::SuLi, true)), None, "三交"),
            ("卷六亂首（正月酉時庚午日）", "庚午", "酉", "亥", "", None, None, "亂首"),
            ("卷六贅壻（甲戌日戌加甲）", "甲戌", "卯", "亥", "戌午寅", None, None, "贅壻"),
            ("卷六贅壻（丙申日巳加申）", "丙申", "辰", "丑", "巳寅亥", None, None, "贅壻"),
            ("卷七蕪淫（乙卯日不備）", "乙卯", "午", "未", "", None, None, "蕪淫"),
            ("卷七蕪淫（乙亥日不備）", "乙亥", "巳", "子", "", None, None, "蕪淫"),
            ("卷七蕪淫（甲子日交克）", "甲子", "卯", "亥", "", None, None, "蕪淫"),
            ("卷七度厄（甲子日三上克下）", "甲子", "丑", "申", "", None, None, "幼度厄"),
            ("卷七無祿（三月己巳日）", "己巳", "寅", "酉", "", None, None, "無祿"),
            ("卷七絶嗣（正月庚辰日）", "庚辰", "辰", "亥", "", None, None, "絶嗣"),
            ("卷八九醜（二月將乙卯日子時）", "乙卯", "子", "戌", "", None, None, "九醜"),
            ("卷八𤣥胎（甲寅日寅時巳將）", "甲寅", "寅", "巳", "申亥寅", None, None, "玄胎"),
            ("卷八連珠（乙丑日酉時戌將）", "乙丑", "酉", "戌", "寅卯辰", None, None, "進連茹"),
        ];
        for (src, d, s, j, want_sc, gr, yj, ge) in cases {
            let day = gz(d);
            let k = qi_ke(day, z(j), z(s));
            if !want_sc.is_empty() {
                assert_eq!(k.san_chuan, sc(want_sc), "{src}");
            }
            let tj = gr.map(|(r, zhou)| tian_jiang(&k.pan, day % 10, zhou, r));
            let got = ke_ge(&k, day, tj.as_ref(), yj.map(z));
            assert!(names(&got).contains(ge), "{src}: {:?} 三传 {:?}", names(&got), k.san_chuan);
        }
    }

    /// 须论旺相、岁月、年命、神将之课格：卷五至卷八各课所举之例（贵人用书中实用表、卯至酉为昼）。
    /// (出处, 日, 时, 将, 月建, 太岁, 本命, 行年, 应有之格)
    #[test]
    fn ke_ge_ctx_examples() {
        use super::super::{is_zhou_mao_zhi_you, tian_jiang, GuiRule, TianPan};
        type C = (&'static str, &'static str, &'static str, &'static str, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, &'static str);
        let cases: &[C] = &[
            ("卷五三光袖中金 六月戊寅日午時寅將", "戊寅", "午", "寅", Some("未"), None, None, None, "三光"),
            ("卷五三陽 乙丑日酉時戌將（天乙子臨亥順行）", "乙丑", "酉", "戌", Some("卯"), None, None, None, "三陽"),
            ("卷五時泰 子年戌月戊寅日戌時卯將", "戊寅", "戌", "卯", Some("戌"), Some("子"), None, None, "時泰"),
            ("卷五龍徳 癸巳年七月癸酉日酉時巳將", "癸酉", "酉", "巳", Some("申"), Some("巳"), None, None, "龍徳"),
            ("卷六官爵 未年二月丁亥日巳時戌將（亥命行年午）", "丁亥", "巳", "戌", Some("卯"), Some("未"), Some("亥"), Some("午"), "官爵"),
            ("卷六富貴 二月辛巳日丑時戌將（寅命行年巳）", "辛巳", "丑", "戌", Some("卯"), None, Some("寅"), Some("巳"), "富貴"),
            ("卷六軒盖 甲子日卯時子將", "甲子", "卯", "子", None, None, None, None, "軒蓋"),
            ("卷六鑄印 丙子日未時子將", "丙子", "未", "子", None, None, None, None, "鑄印"),
            ("卷六斵輪 辛丑日辰時亥將（卯加申）", "辛丑", "辰", "亥", None, None, None, None, "斵輪"),
            ("卷六引從 壬子日巳時戌將（巳加子、卯加戌）", "壬子", "巳", "戌", None, None, None, None, "引從"),
            ("卷六亨通 丙戌日申時亥將（申亥寅遞生）", "丙戌", "申", "亥", None, None, None, None, "亨通"),
            ("卷六徳慶 戊子日戌時卯將（巳為徳加子）", "戊子", "戌", "卯", Some("寅"), None, None, None, "徳慶"),
            ("卷六斬闗 甲寅日亥時未將（戌加寅）", "甲寅", "亥", "未", None, None, None, None, "斬闗"),
            ("卷六冲破 子年庚子日午加卯", "庚子", "卯", "午", None, Some("子"), None, None, "冲破"),
            ("卷六淫泆 戊戌日辰時午將（泆女）", "戊戌", "辰", "午", None, None, None, None, "淫泆"),
            ("卷七天獄 七月乙丑日巳加子", "乙丑", "子", "巳", Some("申"), None, None, None, "天獄"),
            ("卷七龍戰 丁卯日辰時戌將（卯加酉）", "丁卯", "辰", "戌", None, None, None, None, "龍戰"),
            ("卷八勵徳 戊子日申時午將（晝丑加卯）", "戊子", "申", "午", None, None, None, None, "勵徳"),
        ];
        let mut bad = Vec::new();
        for (src, d, s, j, yj, ts, bm, xn, want) in cases {
            let day = gz(d);
            let k = qi_ke(day, z(j), z(s));
            let tj = tian_jiang(&TianPan { jiang: z(j), shi: z(s) }, day % 10, is_zhou_mao_zhi_you(z(s)), GuiRule::DaQuan);
            let ctx = KeGeCtx { tj: Some(&tj), yue_jian: yj.map(z), tai_sui: ts.map(z), ben_ming: bm.map(z), xing_nian: xn.map(z), mu: MuRule::WuXing, ..Default::default() };
            let got = ke_ge_ctx(&k, day, &ctx);
            if !names(&got).contains(want) {
                bad.push(format!("{src}: {:?} 三传 {:?}", names(&got), k.san_chuan));
            }
        }
        assert!(bad.is_empty(), "{bad:#?}");
    }

    /// 鬼墓訂訛「假如壬日辰加亥發用既作日鬼又作日墓」：六壬日中辰加亥而辰發用者皆判鬼墓
    #[test]
    fn gui_mu_example() {
        let ctx = KeGeCtx { tj: None, yue_jian: None, tai_sui: None, ben_ming: None, xing_nian: None, mu: MuRule::WuXing, ..Default::default() };
        let mut n = 0;
        for d in ["壬子", "壬寅", "壬辰", "壬午", "壬申", "壬戌"] {
            let k = qi_ke(gz(d), z("辰"), z("亥"));
            if k.san_chuan[0] == z("辰") {
                n += 1;
                assert!(names(&ke_ge_ctx(&k, gz(d), &ctx)).contains(&"鬼墓"), "{d}");
            }
        }
        assert!(n > 0);
    }

    /// 殃咎：卷八所举三传（时将未举，于 144 盘中取得此三传者判之）
    #[test]
    fn yang_jiu_examples() {
        for (d, want) in [("己巳", "巳申寅"), ("丙子", "子未寅"), ("辛酉", "未子巳")] {
            let day = gz(d);
            let mut found = false;
            for j in 0..12u8 {
                for s in 0..12u8 {
                    let k = qi_ke(day, j, s);
                    if k.san_chuan == sc(want) {
                        found = true;
                        let ctx = KeGeCtx { tj: None, yue_jian: None, tai_sui: None, ben_ming: None, xing_nian: None, mu: MuRule::WuXing, ..Default::default() };
                        assert!(names(&ke_ge_ctx(&k, day, &ctx)).contains(&"殃咎"), "{d}");
                    }
                }
            }
            assert!(found, "{d}");
        }
        // 殃咎訂訛：丙寅日丙上戌寅上未為乘墓
        let day = gz("丙寅");
        let k = qi_ke(day, z("戌"), z("巳")); // 戌加巳：干（巳）上戌、支（寅）上未
        assert_eq!((k.si_ke[0].shang, k.si_ke[2].shang), (z("戌"), z("未")));
        let ctx = KeGeCtx { tj: None, yue_jian: None, tai_sui: None, ben_ming: None, xing_nian: None, mu: MuRule::WuXing, ..Default::default() };
        assert!(names(&ke_ge_ctx(&k, day, &ctx)).contains(&"殃咎"));
        // 十干墓：丙墓亦戌，寅墓（支）未——两说同
        let ctx2 = KeGeCtx { mu: MuRule::ShiGan, ..ctx };
        assert!(names(&ke_ge_ctx(&k, day, &ctx2)).contains(&"殃咎"));
        // 坐墓：丙寅日巳加戌、寅加未
        let k = qi_ke(day, z("巳"), z("戌"));
        assert!(names(&ke_ge_ctx(&k, day, &ctx)).contains(&"殃咎"));
    }

    /// 起月宿法：卷七书中诸例
    #[test]
    fn yue_su_examples() {
        let x = |m, d| XIU28[yue_su_qi_li(m, d)];
        assert_eq!(x(1, 14), "柳"); // 二煩訂訛「正月初一起室數至十四日為桺」
        assert_eq!(x(6, 2), "柳"); // 「六月初一起鬼初二栁」
        assert_eq!(x(8, 5), "房"); // 天冦訂訛「初一角初二亢初三氐氐該重留一日初四仍在氐初五房」
        assert_eq!(x(9, 3), "房"); // 二煩正文九月初三「月宿躔氐五度……夘宫」（氐、房皆在卯）
        assert_eq!(x(2, 5), "昴"); // 天冦觀月經注「初五日……從奎數至昴」
        assert_eq!(yue_su_zhi(yue_su_qi_li(1, 14)), z("午"));
        assert_eq!(yue_su_zhi(yue_su_qi_li(8, 5)), z("卯"));
    }

    /// 繁昌、榮華、合歡、和美、天禍、天寇、二煩、死奇、魄化、灾厄：卷六、卷七所举之例（贵人用书中实用表、卯至酉为昼）。
    #[test]
    fn ke_ge_ctx2_examples() {
        use super::super::{is_zhou_mao_zhi_you, tian_jiang, GuiRule, TianPan};
        struct C {
            src: &'static str,
            d: &'static str,
            s: &'static str,
            j: &'static str,
            ctx: KeGeCtx<'static>,
            want: &'static str,
        }
        // 个别书例之贵人与书中实用表（卯至酉为昼）不合者，另给（贵人之法, 昼）
        let over: &[(&str, GuiRule, bool)] = &[("卷七魄化心鏡 二月甲戌日寅時（戌將）", GuiRule::KaoYuan, false)];
        let base = KeGeCtx::default();
        let ys = |m, d| Some(yue_su_zhi(yue_su_qi_li(m, d)));
        let cases = [
            C { src: "卷六繁昌德孕 壬申日未時巳將，夫行年甲寅、妻行年己亥", d: "壬申", s: "未", j: "巳", ctx: KeGeCtx { yue_jian: Some(z("申")), xing_nian_gz: Some(gz("甲寅")), pei_xing_nian_gz: Some(gz("己亥")), ..base }, want: "繁昌" },
            C { src: "卷六繁昌旺孕心鏡 春占夫年寅妻年午", d: "甲子", s: "子", j: "子", ctx: KeGeCtx { yue_jian: Some(z("卯")), xing_nian_gz: Some(gz("丙寅")), pei_xing_nian_gz: Some(gz("庚午")), ..base }, want: "繁昌" },
            C { src: "卷六榮華 丙申日卯時子將，寅命行年巳", d: "丙申", s: "卯", j: "子", ctx: KeGeCtx { yue_jian: Some(z("寅")), ben_ming: Some(z("寅")), xing_nian: Some(z("巳")), ..base }, want: "榮華" },
            C { src: "卷六合歡 戊申日子時申將，亥命行年辰", d: "戊申", s: "子", j: "申", ctx: KeGeCtx { ben_ming: Some(z("亥")), xing_nian: Some(z("辰")), ..base }, want: "合歡" },
            C { src: "卷六和美 壬午日巳時丑將", d: "壬午", s: "巳", j: "丑", ctx: base, want: "和美" },
            C { src: "卷六和美 辛卯日卯加辛（戌）", d: "辛卯", s: "戌", j: "卯", ctx: base, want: "和美" },
            C { src: "卷六和美 壬寅日亥加寅", d: "壬寅", s: "寅", j: "亥", ctx: base, want: "和美" },
            C { src: "卷六和美 甲申日干上亥", d: "甲申", s: "寅", j: "亥", ctx: base, want: "和美" },
            C { src: "卷六和美 丁丑日干上午", d: "丁丑", s: "未", j: "午", ctx: base, want: "和美" },
            C { src: "卷六和美 己丑日干上午", d: "己丑", s: "未", j: "午", ctx: base, want: "和美" },
            C { src: "卷六和美 乙丑日干上子", d: "乙丑", s: "辰", j: "子", ctx: base, want: "和美" },
            C { src: "卷六和美 乙酉日伏吟", d: "乙酉", s: "子", j: "子", ctx: base, want: "和美" },
            C { src: "卷六和美 丙申日伏吟", d: "丙申", s: "子", j: "子", ctx: base, want: "和美" },
            C { src: "卷七天禍 立春甲申日寅加癸", d: "甲申", s: "丑", j: "寅", ctx: KeGeCtx { si_li: true, ..base }, want: "天禍" },
            C { src: "卷七天禍觀月經 癸丑日丑上見亥", d: "癸丑", s: "丑", j: "亥", ctx: KeGeCtx { si_li: true, ..base }, want: "天禍" },
            C { src: "卷七天禍觀月經 庚日干上小吉（庚午）", d: "庚午", s: "申", j: "未", ctx: KeGeCtx { si_li: true, ..base }, want: "天禍" },
            C { src: "卷七天禍觀月經 丁日未上見太乙（丁卯）", d: "丁卯", s: "未", j: "巳", ctx: KeGeCtx { si_li: true, ..base }, want: "天禍" },
            C { src: "卷七天禍心鏡 乙酉日立春子將戌時", d: "乙酉", s: "戌", j: "子", ctx: KeGeCtx { si_li: true, ..base }, want: "天禍" },
            C { src: "卷七天冦訂訛 八月初五丁酉秋分辰將酉時", d: "丁酉", s: "酉", j: "辰", ctx: KeGeCtx { fen_zhi: true, yue_su: ys(8, 5), ..base }, want: "天寇" },
            C { src: "卷七天冦正文 癸卯日春分月宿辰加寅", d: "癸卯", s: "寅", j: "辰", ctx: KeGeCtx { fen_zhi: true, yue_su: Some(z("辰")), ..base }, want: "天寇" },
            C { src: "卷七二煩正文 九月初三丙午日午時卯將", d: "丙午", s: "午", j: "卯", ctx: KeGeCtx { yue_su: ys(9, 3), ..base }, want: "二煩" },
            C { src: "卷七二煩訂訛 壬子日午將卯時正月十四（地煩）", d: "壬子", s: "卯", j: "午", ctx: KeGeCtx { yue_su: ys(1, 14), ..base }, want: "二煩" },
            C { src: "卷七二煩訂訛 壬子日午將卯時六月初二（天地二煩）", d: "壬子", s: "卯", j: "午", ctx: KeGeCtx { yue_su: ys(6, 2), ..base }, want: "二煩" },
            C { src: "卷七二煩 三月十五己卯日子時酉將（天煩）", d: "己卯", s: "子", j: "酉", ctx: base, want: "二煩" },
            C { src: "卷七二煩 戌將卯時己卯日（天煩杜傳）", d: "己卯", s: "卯", j: "戌", ctx: base, want: "天煩杜傳" },
            C { src: "卷七二煩 四月丙午日申將午時（天煩杜傳）", d: "丙午", s: "午", j: "申", ctx: base, want: "天煩杜傳" },
            C { src: "卷七二煩 六月癸酉日未將酉時（天煩杜傳）", d: "癸酉", s: "酉", j: "未", ctx: base, want: "天煩杜傳" },
            C { src: "卷七死奇 甲子日丑時巳將", d: "甲子", s: "丑", j: "巳", ctx: base, want: "死奇" },
            C { src: "卷七魄化心鏡 二月甲戌日寅時（戌將）", d: "甲戌", s: "寅", j: "戌", ctx: KeGeCtx { yue_jian: Some(z("卯")), ..base }, want: "魄化" },
            C { src: "卷七魄化心鏡 六月壬戌日未時（午將）", d: "壬戌", s: "未", j: "午", ctx: KeGeCtx { yue_jian: Some(z("未")), ..base }, want: "魄化" },
            C { src: "卷七灾厄 亥年正月將乙亥日卯時", d: "乙亥", s: "卯", j: "亥", ctx: KeGeCtx { yue_jian: Some(z("寅")), tai_sui: Some(z("亥")), ..base }, want: "灾厄" },
        ];
        let mut bad = Vec::new();
        let mut rows = Vec::new();
        for c in &cases {
            let day = gz(c.d);
            let k = qi_ke(day, z(c.j), z(c.s));
            let (gr, zhou) = over.iter().find(|o| o.0 == c.src).map_or((GuiRule::DaQuan, is_zhou_mao_zhi_you(z(c.s))), |o| (o.1, o.2));
            let tj = tian_jiang(&TianPan { jiang: z(c.j), shi: z(c.s) }, day % 10, zhou, gr);
            let ctx = KeGeCtx { tj: Some(&tj), ..c.ctx };
            let got = ke_ge_ctx(&k, day, &ctx);
            let sc: String = k.san_chuan.iter().map(|x| ZHI[*x as usize]).collect();
            let hit = got.iter().find(|g| g.name == c.want);
            rows.push(format!("| {} | {} | {} | {} |", c.src, sc, c.want, hit.map_or("不合".to_string(), |g| format!("合（{}）", g.note))));
            if hit.is_none() {
                bad.push(format!("{}: {:?} 三传 {}", c.src, names(&got), sc));
            }
        }
        eprintln!("{}", rows.join("\n"));
        assert!(bad.is_empty(), "{bad:#?}");
    }

    /// 土旺十八日：辰月土王用事之前木旺（辰中余木），其间土旺；缺省季月全月土旺
    #[test]
    fn tu_wang_18() {
        use super::super::shen_sha::{dang_ling, wang_shuai, wang_shuai_ling, WangXiangRule};
        assert_eq!(dang_ling(z("辰"), WangXiangRule::JiYue, false), 2);
        assert_eq!(dang_ling(z("辰"), WangXiangRule::TuWang18, false), 0);
        assert_eq!(dang_ling(z("辰"), WangXiangRule::TuWang18, true), 2);
        assert_eq!(dang_ling(z("丑"), WangXiangRule::TuWang18, false), 4);
        assert_eq!(dang_ling(z("寅"), WangXiangRule::TuWang18, false), 0);
        for y in 0..12u8 {
            for w in 0..5u8 {
                assert_eq!(wang_shuai(w, y), wang_shuai_ling(w, dang_ling(y, WangXiangRule::JiYue, false)));
            }
        }
    }

    /// 反例：不应判出之格
    #[test]
    fn ke_ge_negative() {
        // 三奇：止有干奇不名三奇。甲子日卯時子將三傳午卯子，午为甲之干奇，旬奇丑不在传
        let k = qi_ke(gz("甲子"), z("子"), z("卯"));
        assert!(!names(&ke_ge(&k, gz("甲子"), None, None)).contains(&"三奇"));
        // 閉口須玄武：丁酉日午加酉，俗例夜贵午乘白虎，不成閉口
        let k = qi_ke(gz("丁酉"), z("午"), z("酉"));
        let tj = tian_jiang(&k.pan, 3, false, GuiRule::SuLi);
        assert!(!names(&ke_ge(&k, gz("丁酉"), Some(&tj), None)).contains(&"閉口"));
        // 天禍、天寇须四立、分至之日：同盘而非其日不判
        let k = qi_ke(gz("甲申"), z("寅"), z("丑"));
        assert!(!names(&ke_ge_ctx(&k, gz("甲申"), &KeGeCtx::default())).contains(&"天禍"));
        let k = qi_ke(gz("丁酉"), z("辰"), z("酉"));
        let ys = Some(yue_su_zhi(yue_su_qi_li(8, 5)));
        assert!(!names(&ke_ge_ctx(&k, gz("丁酉"), &KeGeCtx { yue_su: ys, ..Default::default() })).contains(&"天寇"));
        // 繁昌：夫妻行年干合支合而年上神不旺相者不成（甲寅、己亥，寅月：子水休、酉金囚）
        let k = qi_ke(gz("壬申"), z("巳"), z("未"));
        let c = KeGeCtx { yue_jian: Some(z("寅")), xing_nian_gz: Some(gz("甲寅")), pei_xing_nian_gz: Some(gz("己亥")), ..Default::default() };
        assert!(!names(&ke_ge_ctx(&k, gz("壬申"), &c)).contains(&"繁昌"));
        // 三传皆土无丁马者为稼穡：乙巳日若非辰月（天马不在传）而旬丁未在传，仍为遊子
        let k = qi_ke(gz("乙巳"), z("酉"), z("午"));
        assert!(names(&ke_ge(&k, gz("乙巳"), None, Some(z("巳")))).contains(&"遊子"));
    }
}
