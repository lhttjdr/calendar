//! 大六壬课体：九宗门（卷一入手法九法）、本门课名（卷五各课），与可由盘面直接判定的常见课格（卷五至卷八）。
//!
//! 只收依日干支、四课、三传（及可选的十二天将、月建）即可判定者；须论旺相、岁月、年命、神煞吉凶的课格
//! （三光、三陽、時泰、龍徳、官爵、富貴、軒蓋、鑄印、榮華、徳慶、天獄、天寇、殃咎等）未做，见 doc/folk/liuren.md。

use super::shen_sha::{ding_ma, tian_ma, xun_shou};
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
            Fa::SheHai | Fa::JianJi | Fa::ChaWei | Fa::ZhuiXia => ZongMen::SheHai,
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

/// 由盘面判定常见课格。`tj` 十二天将（閉口、三交须用；无则不判此二者之将）；`yue_jian` 月建地支（遊子之天马、稼穡须用）
pub fn ke_ge(k: &KeShi, day_gz: u8, tj: Option<&TianJiang>, yue_jian: Option<u8>) -> Vec<KeGe> {
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
    v
}

#[cfg(test)]
mod tests {
    use super::super::{qi_ke, tian_jiang, GuiRule, GAN, ZHI};
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
        // 三传皆土无丁马者为稼穡：乙巳日若非辰月（天马不在传）而旬丁未在传，仍为遊子
        let k = qi_ke(gz("乙巳"), z("酉"), z("午"));
        assert!(names(&ke_ge(&k, gz("乙巳"), None, Some(z("巳")))).contains(&"遊子"));
    }
}
