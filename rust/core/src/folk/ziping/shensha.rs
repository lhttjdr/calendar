//! 空亡与常用神煞（基础层）。起例出《三命通會》卷二、卷三、卷五（维基文库四库本录文），异说并列为选项。
//! 只列某柱值何神，不论吉凶（《子平真詮·論星辰無關格局》亦谓星辰不关格局）。
//!
//! 依干起者（天乙、太极、禄、羊刃、金舆）以年干或日干起，依支起者（驿马、咸池、华盖、将星、劫煞、亡神、孤辰寡宿）以年支或日支起，
//! 依 [ShaBase]；所起之柱本身不计，只看其余三柱。天德、月德依月支，看四柱天干（天德四维之月看地支）。

use super::{gan_is_yang, gan_of, zhi_of, SiZhu};

/// 四柱位置
pub const PILLARS: [&str; 4] = ["年", "月", "日", "時"];

/// 起例所依之柱
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShaBase {
    /// 只依年（《三命通會》多称「甲戊庚人」「寅午戌生人」，以生年为主）
    Year,
    /// 只依日（子平以日为主，今人多以日干起贵人禄刃）
    Day,
    /// 年、日各起一遍，逐条注明所依（缺省：两说并存，不作取舍）
    #[default]
    YearAndDay,
}

/// 天乙贵人之家
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TianYiRule {
    /// 「甲戊庚牛羊，乙己鼠猴鄉，丙丁豬雞位，壬癸兔蛇藏，六辛逢馬虎」：《三命通會》卷三《論天乙貴人》以先天坤、后天坤起阳贵阴贵推得
    /// 「甲戊庚牛羊六辛逢馬虎明矣」：缺省
    #[default]
    JiaWuGeng,
    /// 「甲戊兼牛羊……庚辛逢馬虎」：宋儲泳《祛疑説》记「或者謂當以甲戊在牛羊，庚辛逢馬虎為是」（儲泳自谓「當以古法為正」，不从）；
    /// 清《增刪卜易·星煞章》用此
    GengXinMaHu,
}

/// 羊刃之说
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum YangRenRule {
    /// 惟阳干有刃（甲卯丙午戊午庚酉壬子）：《三命通會》卷五《論陽刃》「惟甲丙戊庚壬五陽干有刃，乙丁己辛癸五陰干無刃，故曰陽刃」：缺省
    #[default]
    YangOnly,
    /// 禄前一辰，阴干亦有（乙辰丁未己未辛戌癸丑）：《三命通會》卷三《論羊刃》「祿前一辰為羊刃……辰者乙之正位為陰木之極」
    LuQian,
}

/// 空亡依何柱之旬
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KongWangBase {
    /// 日柱：今人通行，缺省。（《三命通會》卷三「命中犯者當以日上見之為是，其餘不論」在《論十惡大敗》，
    /// 说十恶大败日须见于日柱，非泛论空亡；萬曆六年刊本同。依日之书证待补，见 doc/folk/ziping.md 第九节）
    #[default]
    Day,
    /// 年柱（以生年之旬）
    Year,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShenshaOptions {
    pub base: ShaBase,
    pub tian_yi: TianYiRule,
    pub yang_ren: YangRenRule,
    pub kong_wang: KongWangBase,
}

/// 一条神煞：名、所依之柱（月德天德为月）、所在之柱
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShenshaHit {
    pub name: &'static str,
    pub base: usize,
    pub at: usize,
}

/// 十干禄：甲寅乙卯丙戊巳丁己午庚申辛酉壬亥癸子（《三命通會》卷三《論十干祿》）
pub fn lu(gan: u8) -> u8 {
    [2, 3, 5, 6, 5, 6, 8, 9, 11, 0][gan as usize % 10]
}

/// 羊刃：禄前一辰；[YangRenRule::YangOnly] 下阴干无刃
pub fn yang_ren(gan: u8, rule: YangRenRule) -> Option<u8> {
    match rule {
        YangRenRule::YangOnly if !gan_is_yang(gan) => None,
        _ => Some((lu(gan) + 1) % 12),
    }
}

/// 金舆：禄前二辰（《三命通會》卷三《論金轝》「金轝常居祿前二辰」）
pub fn jin_yu(gan: u8) -> u8 {
    (lu(gan) + 2) % 12
}

/// 天乙贵人二支
pub fn tian_yi(gan: u8, rule: TianYiRule) -> [u8; 2] {
    match (gan % 10, rule) {
        (0, _) | (4, _) => [1, 7],
        (6, TianYiRule::JiaWuGeng) => [1, 7],
        (6, TianYiRule::GengXinMaHu) => [6, 2],
        (1, _) | (5, _) => [0, 8],
        (2, _) | (3, _) => [11, 9],
        (7, _) => [6, 2],
        _ => [5, 3], // 壬癸兔蛇
    }
}

/// 太极贵人（《三命通會》卷三《論太極貴》「甲乙木先造乎子……後終乎午；丙丁……卯……酉；庚辛……寅……亥；壬癸……申……巳；戊己……辰戌丑未」）
pub fn tai_ji(gan: u8) -> &'static [u8] {
    match gan % 10 {
        0 | 1 => &[0, 6],
        2 | 3 => &[3, 9],
        4 | 5 => &[4, 10, 1, 7],
        6 | 7 => &[2, 11],
        _ => &[8, 5],
    }
}

/// 三合局之序：申子辰 0、寅午戌 1、巳酉丑 2、亥卯未 3
fn san_he(zhi: u8) -> usize {
    match zhi % 12 {
        8 | 0 | 4 => 0,
        2 | 6 | 10 => 1,
        5 | 9 | 1 => 2,
        _ => 3,
    }
}

/// 驿马（《三命通會》卷三《論驛馬》：寅午戌人馬在申，申子辰人馬在寅，巳酉丑人馬在亥，亥卯未人馬在巳）
pub fn yi_ma(zhi: u8) -> u8 {
    [2, 8, 11, 5][san_he(zhi)]
}
/// 咸池（桃花）：三合长生后一位之沐浴（卷二《論咸池》：寅午戌卯，巳酉丑午，申子辰酉，亥卯未子）
pub fn xian_chi(zhi: u8) -> u8 {
    [9, 3, 6, 0][san_he(zhi)]
}
/// 华盖：三合之库（卷二《論將星華葢》）
pub fn hua_gai(zhi: u8) -> u8 {
    [4, 10, 1, 7][san_he(zhi)]
}
/// 将星：三合之中
pub fn jiang_xing(zhi: u8) -> u8 {
    [0, 6, 9, 3][san_he(zhi)]
}
/// 劫煞：三合五行之绝（卷三《論刼煞亡神》：申子辰巳、寅午戌亥、巳酉丑寅、亥卯未申）
pub fn jie_sha(zhi: u8) -> u8 {
    [5, 11, 2, 8][san_he(zhi)]
}
/// 亡神：三合五行之临官（同上：申子辰亥、寅午戌巳、巳酉丑申、亥卯未寅）
pub fn wang_shen(zhi: u8) -> u8 {
    [11, 5, 8, 2][san_he(zhi)]
}

/// 孤辰、寡宿（卷三《論孤辰寡宿》：亥子丑人寅孤戌寡，寅卯辰人巳孤丑寡，巳午未人申孤辰寡，申酉戌人亥孤未寡）
pub fn gu_gua(zhi: u8) -> (u8, u8) {
    match zhi % 12 {
        11 | 0 | 1 => (2, 10),
        2 | 3 | 4 => (5, 1),
        5 | 6 | 7 => (8, 4),
        _ => (11, 7),
    }
}

/// 元辰（卷三《論元辰》）：阳男阴女在冲前一位，阴男阳女在冲后一位（「甲子生男與甲午對衝，即乙未為正；乙丑生男與乙未對衝，即甲午為正」）
pub fn yuan_chen(year_gz: u8, male: bool) -> u8 {
    let z = zhi_of(year_gz);
    if gan_is_yang(gan_of(year_gz)) == male {
        (z + 7) % 12
    } else {
        (z + 5) % 12
    }
}

/// 天德：正丁、二坤、三壬、四辛、五乾、六甲、七癸、八艮、九丙、十乙、十一巽、十二庚；四维之月取其支（坤申、乾亥、艮寅、巽巳）。
/// 返回 (是否为干, 序)。命书与择日同（协纪卷五引《堪輿經》）
pub fn tian_de(month_zhi: u8) -> (bool, u8) {
    let ord = ((month_zhi + 10) % 12) as usize; // 寅 = 0
    [(true, 3), (false, 8), (true, 8), (true, 7), (false, 11), (true, 0), (true, 9), (false, 2), (true, 2), (true, 1), (false, 5), (true, 6)][ord]
}

/// 月德：寅午戌丙、申子辰壬、亥卯未甲、巳酉丑庚（《三命通會》卷三《論天月徳》）
pub fn yue_de(month_zhi: u8) -> u8 {
    [8, 2, 6, 0][san_he(month_zhi)]
}

/// 旬空二支
/// 旬空二支：甲子旬戌亥、甲戌旬申酉、甲申旬午未、甲午旬辰巳、甲辰旬寅卯、甲寅旬子丑（旬首之支前二位）
pub fn xun_kong(gz: u8) -> [u8; 2] {
    let a = (10 - 2 * ((gz % 60) / 10) as i32).rem_euclid(12) as u8;
    [a, (a + 1) % 12]
}

/// 列出四柱所值神煞
pub fn shensha(sz: &SiZhu, male: bool, o: &ShenshaOptions) -> Vec<ShenshaHit> {
    let p = sz.as_array();
    let mut out = Vec::new();
    let bases: &[usize] = match o.base {
        ShaBase::Year => &[0],
        ShaBase::Day => &[2],
        ShaBase::YearAndDay => &[0, 2],
    };
    fn push_zhi(p: &[u8; 4], name: &'static str, base: usize, targets: &[u8], out: &mut Vec<ShenshaHit>) {
        for (i, &g) in p.iter().enumerate() {
            if i != base && targets.contains(&zhi_of(g)) {
                out.push(ShenshaHit { name, base, at: i });
            }
        }
    }
    for &b in bases {
        let g = gan_of(p[b]);
        let z = zhi_of(p[b]);
        push_zhi(&p, "天乙貴人", b, &tian_yi(g, o.tian_yi), &mut out);
        push_zhi(&p, "太極貴人", b, tai_ji(g), &mut out);
        push_zhi(&p, "祿", b, &[lu(g)], &mut out);
        if let Some(r) = yang_ren(g, o.yang_ren) {
            push_zhi(&p, "羊刃", b, &[r], &mut out);
        }
        push_zhi(&p, "金轝", b, &[jin_yu(g)], &mut out);
        push_zhi(&p, "驛馬", b, &[yi_ma(z)], &mut out);
        push_zhi(&p, "咸池", b, &[xian_chi(z)], &mut out);
        push_zhi(&p, "華蓋", b, &[hua_gai(z)], &mut out);
        push_zhi(&p, "將星", b, &[jiang_xing(z)], &mut out);
        push_zhi(&p, "劫煞", b, &[jie_sha(z)], &mut out);
        push_zhi(&p, "亡神", b, &[wang_shen(z)], &mut out);
        let (gu, gua) = gu_gua(z);
        push_zhi(&p, "孤辰", b, &[gu], &mut out);
        push_zhi(&p, "寡宿", b, &[gua], &mut out);
    }
    // 元辰：依年
    push_zhi(&p, "元辰", 0, &[yuan_chen(p[0], male)], &mut out);
    // 天德、月德：依月，看四柱
    let mz = zhi_of(p[1]);
    let (is_gan, td) = tian_de(mz);
    let yd = yue_de(mz);
    for (i, &g) in p.iter().enumerate() {
        if (is_gan && gan_of(g) == td) || (!is_gan && zhi_of(g) == td) {
            out.push(ShenshaHit { name: "天德", base: 1, at: i });
        }
        if gan_of(g) == yd {
            out.push(ShenshaHit { name: "月德", base: 1, at: i });
        }
    }
    // 空亡
    let kb = match o.kong_wang {
        KongWangBase::Day => 2,
        KongWangBase::Year => 0,
    };
    push_zhi(&p, "空亡", kb, &xun_kong(p[kb]), &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::folk::ziping::ZHI;

    fn zs(v: &[u8]) -> String {
        v.iter().map(|&z| ZHI[z as usize]).collect()
    }

    #[test]
    fn tables() {
        let t: Vec<String> = (0..10).map(|g| zs(&tian_yi(g, TianYiRule::JiaWuGeng))).collect();
        assert_eq!(t, ["丑未", "子申", "亥酉", "亥酉", "丑未", "子申", "丑未", "午寅", "巳卯", "巳卯"]);
        assert_eq!(zs(&tian_yi(6, TianYiRule::GengXinMaHu)), "午寅");
        let r: Vec<_> = (0..10).map(|g| yang_ren(g, YangRenRule::LuQian).map(|z| ZHI[z as usize])).collect();
        assert_eq!(r, [Some("卯"), Some("辰"), Some("午"), Some("未"), Some("午"), Some("未"), Some("酉"), Some("戌"), Some("子"), Some("丑")]);
        assert_eq!(yang_ren(1, YangRenRule::YangOnly), None);
        // 金轝：甲子人祿在寅，辰為金轝
        assert_eq!(jin_yu(0), 4);
        // 元辰两例
        assert_eq!(ZHI[yuan_chen(0, true) as usize], "未");
        assert_eq!(ZHI[yuan_chen(1, true) as usize], "午");
        // 天德、月德：三月壬、壬
        assert_eq!(tian_de(4), (true, 8));
        assert_eq!(yue_de(4), 8);
        // 旬空：甲子旬戌亥
        assert_eq!(zs(&xun_kong(0)), "戌亥");
    }

    #[test]
    fn san_ming_examples() {
        // 《三命通會》卷三《論天乙貴人》：「蘓東坡學士丙子辛丑癸亥己卯」年时互换见贵——丙贵在亥（日支），己贵在子（年支）
        let sz = SiZhu::parse("丙子辛丑癸亥己卯").unwrap();
        let h = shensha(&sz, true, &ShenshaOptions { base: ShaBase::Year, ..Default::default() });
        assert!(h.iter().any(|x| x.name == "天乙貴人" && x.at == 2));
        assert!(tian_yi(5, TianYiRule::JiaWuGeng).contains(&0));
        // 同节：「丙申己亥辛未己亥」「辛巳丙申壬午壬寅」俱互换见贵
        for (s, y, t) in [("丙申己亥辛未己亥", 2u8, 5u8), ("辛巳丙申壬午壬寅", 7, 8)] {
            let sz = SiZhu::parse(s).unwrap();
            assert!(tian_yi(y, TianYiRule::JiaWuGeng).contains(&zhi_of(sz.hour)), "{s}");
            assert!(tian_yi(t, TianYiRule::JiaWuGeng).contains(&zhi_of(sz.year)), "{s}");
        }
    }
}
