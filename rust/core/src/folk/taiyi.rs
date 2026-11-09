//! 太乙（岁计、月计、日计、时计）与三元九运。
//!
//! 起例据唐王希明《太乙金鏡式經》（四库本）卷一，及《古今圖書集成·藝術典》卷六百八十七《太乙淘金歌》、卷六百八十八《太乙局》、
//! 《太乙秘書》；各家积年并列为选项（[`JiNian`]）。逐例检验见 doc/folk/taiyi.md。
//!
//! 一局由「入局数」r（1—360，即积数以周纪法三百六十去之之余，余零作三百六十）定：
//! - 元：r 以元法七十二除，甲子、丙子、戊子、庚子、壬子五元；局：元内第几（1—72）；纪：r 以六十除，六纪。
//! - 太乙：r 以太乙小周法二十四去之，「以三約之為宫數，不滿為入宫以來年數」，阳遁「命起一宫順行八宫不遊中五」，
//!   阴遁「命起九宫逆行」（金鏡卷一）。宫：一乾、二離、三艮、四震、六兌、七坤、八坎、九巽（淘金歌）。
//! - 天目（文昌）：r 以天目周法十八去之，阳遁命起武德顺行十六神，遇陰德（乾）、大武（坤）重留一筭（金鏡卷一）；
//!   阴遁命起呂申，遇艮、巽重留（淘金歌「陰局艮巽亦重輪」；金鏡卷一阴遁六纪表合）。
//! - 计神：r 以十二去之，阳遁起寅、阴遁起申，逆行十二辰（金鏡卷一；太乙秘書）。
//! - 始击（客目）：以计神加和德（艮），视天目所临之下（淘金歌「求客目」）。
//! - 主算、客算：自天目、始击所在起，正宫依宫数、间神起一，顺数所过正宫，至太乙前一宫止；二目临太乙宫只算本宫
//!   （淘金歌「求主客筭」）。大将：算满十去之余零；整十取十位（集成卷六八八「客筭四十，客大將四宮」）；参将：大将三因去十。
//!   得五为杜（「主客大小將俱無所主」）。

/// 十六神（自子起顺行）：位序 0 子 … 15 亥，含四维
pub const SHEN16: [&str; 16] = [
    "地主", "陽德", "和德", "呂申", "高叢", "太陽", "大炅", "大神", "大威", "天道", "大武", "武德", "太簇", "陰主", "陰德", "大義",
];
/// 十六神所在之方（与 [`SHEN16`] 同序）
pub const POS16: [&str; 16] = ["子", "丑", "艮", "寅", "卯", "辰", "巽", "巳", "午", "未", "坤", "申", "酉", "戌", "乾", "亥"];
/// 五元名
pub const YUAN: [&str; 5] = ["甲子", "丙子", "戊子", "庚子", "壬子"];

pub mod bamen;
pub mod geju;
pub mod shijing;
pub mod suanshu;
pub mod zhushen;

/// 支序（子 = 0）在十六神中的位序
pub fn zhi_pos(z: usize) -> usize {
    [0, 1, 3, 4, 5, 7, 8, 9, 11, 12, 13, 15][z % 12]
}
/// 宫数在十六神中的位序（正宫）
pub fn gong_pos(g: u8) -> usize {
    match g {
        1 => 14, // 乾
        2 => 8,  // 午（離）
        3 => 2,  // 艮
        4 => 4,  // 卯（震）
        6 => 12, // 酉（兌）
        7 => 10, // 坤
        8 => 0,  // 子（坎）
        9 => 6,  // 巽
        _ => unreachable!("太乙不入中五"),
    }
}
/// 十六神位序所属正宫之数；间神为 None
pub fn pos_gong(p: usize) -> Option<u8> {
    [Some(8), None, Some(3), None, Some(4), None, Some(9), None, Some(2), None, Some(7), None, Some(6), None, Some(1), None][p % 16]
}

/// 阴阳遁
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dun {
    Yang,
    Yin,
}

/// 积年之法（各家上元不同；只影响岁计，月计因 60×12 ≡ 0 (mod 360) 各家同）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiNian {
    /// 金鏡卷一：自上元甲子至唐开元十二年甲子积 1,937,281 筭；集成卷六八八「上古天皇上元甲子」至明天启四年甲子
    /// 积 10,155,541 年、太乙秘書至万历戊子积 10,155,505 筭（并「截得祖數」1,938,145，即金鏡之数），
    /// 三者以周纪法 360 去之同余（公元年 + 117）。缺省。
    JinJing,
    /// 淘金歌：「求上元者，截至元世祖至元元年甲子，為上元第一紀起筭」，又言与黄帝元年起算之解「差一元六十年」；
    /// 所举汉至隋岁计十例皆合此（公元年 + 177）。
    TaoJinGe,
}

/// 某干支年（立春分年，天文纪年）的积年
pub fn ji_nian(year: i32, school: JiNian) -> i64 {
    match school {
        JiNian::JinJing => 1_937_281 + (year as i64 - 724),
        JiNian::TaoJinGe => year as i64 - 1263,
    }
}

fn r360(n: i64) -> u16 {
    let r = n.rem_euclid(360) as u16;
    if r == 0 {
        360
    } else {
        r
    }
}

/// 岁计入局数
pub fn sui_ji(year: i32, school: JiNian) -> u16 {
    r360(ji_nian(year, school))
}

/// 月计入局数：`year` 为干支年（立春分），`month_ord` 为节月序（寅月 = 0 … 子月 = 10、丑月 = 11）。
/// 月以节换（金鏡卷一「若有閏月只以逐月節氣時刻為正」），自天正十一月（子月）起数：积月 =（积年 − 1）× 12 + 第几月
/// （集成卷六八八：天启四年正月丙寅，「另加十一月、十二月、正月三數」）。子、丑月属下一干支年之天正。
pub fn yue_ji(year: i32, month_ord: u8, school: JiNian) -> u16 {
    r360(yue_ji_shu(year, month_ord, school))
}

/// 月计积月（未以周纪法去之之数；八门直使用之，见 [`bamen`]）
pub fn yue_ji_shu(year: i32, month_ord: u8, school: JiNian) -> i64 {
    let (t, k) = match month_ord {
        10 => (year + 1, 1),
        11 => (year + 1, 2),
        m => (year, m as i64 + 3),
    };
    (ji_nian(t, school) - 1) * 12 + k
}

// ---------------------------------------------------------------- 金鏡章岁法（经朔积月、积日）

/// 金鏡卷一「推太乙月計差法」：章歲六百五十七
pub const ZHANG_SUI: i64 = 657;
/// 章月：录文作「八千七百二十六」，以十九年七闰之率 657 × (12 + 7/19) ≈ 8126 校正（8726/657 ≈ 13.28，一年十三月有余，不可通）
pub const ZHANG_YUE: i64 = 8126;
/// 金鏡卷一「推日計差法」：日法四十九
pub const RI_FA: i64 = 49;
/// 月法：录文作「二千四百四十七」，以朔策二十九日有余校正为 1447（1447/49 = 29.5306）；
/// 同条「小餘三十六」亦当作二十六（1447 − 29 × 49 = 26）
pub const YUE_FA: i64 = 1447;

/// 章岁法所求年天正十一月经朔之积月与闰余：「置積年所求年以章月乘之章嵗而一為積月不盡為閏餘」。
/// 积年取（积年 − 1），即所求年之前已过之年（与梁天监三年例相合，见 doc/folk/taiyi.md）。
pub fn zhang_sui_ji_yue(year: i32, school: JiNian) -> (i64, i64) {
    let a = (ji_nian(year, school) - 1) * ZHANG_YUE;
    (a.div_euclid(ZHANG_SUI), a.rem_euclid(ZHANG_SUI))
}

/// 经朔积日与小余：「置積月以月法乘之日法而一為積日不盡為小餘」；积日以六十去之命甲子即经朔日干支（甲子 = 0）
pub fn jing_shuo_ji_ri(ji_yue: i64) -> (i64, i64) {
    let a = ji_yue * YUE_FA;
    (a.div_euclid(RI_FA), a.rem_euclid(RI_FA))
}

/// 某日（儒略日数）在章岁法经朔之第几月：返回（已过之积月 m，月内第几日 1—30）。
/// 日计积日 = 经朔积日 + 日（与 [`RI_JI_OFFSET`] 同：梁天监三年六月经朔积日 707,501,052 丙子，合实历朔日）。
pub fn jing_shuo_yue(jdn: i64) -> (i64, i64) {
    let d = jdn + RI_JI_OFFSET;
    let m = (d * RI_FA - 1).div_euclid(YUE_FA);
    (m, d - jing_shuo_ji_ri(m).0)
}

/// 月计入局数之章岁法：积月（含闰月）+ 1，以周纪法去之（金鏡卷一「推太乙月計所在」）。
/// 与缺省「逐月節氣」之法不同：章岁积月计入闰月，故与集成卷六八八天启四年正月例不合（见 doc）。
pub fn yue_ji_zhang_sui(jdn: i64) -> u16 {
    r360(jing_shuo_yue(jdn).0 + 1)
}

/// 日计积日与儒略日数之差：金鏡卷一举梁天监三年六月八日积日 707,501,061；以「积日减一、六十去之、命甲子」为日干支
/// 定其日为甲申（JDN 1,905,331，儒略历 504-07-06）。集成卷六八八天启三年十一月初八甲子（1623-12-29）得 181 与此合。
pub const RI_JI_OFFSET: i64 = 705_595_730;

/// 日计入局数（`jdn` 为该日儒略日数）
pub fn ri_ji(jdn: i64) -> u16 {
    r360(jdn + RI_JI_OFFSET)
}

/// 时计积数（日干支序 × 12 + 时序，1—720；八门直使用之）
pub fn shi_ji_shu(day_gz: u8, hour_zhi: u8) -> i64 {
    day_gz as i64 % 60 * 12 + hour_zhi as i64 % 12 + 1
}

/// 时计入局数：日干支序（甲子 = 0）× 12 + 时支序（子 = 1 … 亥 = 12）。冬至后用阳遁、夏至后用阴遁（调用方定 [`Dun`]）。
/// 集成卷六八八：冬至后甲子日甲子时为阳遁第一局；金鏡卷一例：庚申日寅时阴遁壬子元二十七局（r = 315）。
pub fn shi_ji(day_gz: u8, hour_zhi: u8) -> u16 {
    r360(day_gz as i64 * 12 + hour_zhi as i64 + 1)
}

/// 一局诸项
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaiyiJu {
    /// 入局数 1—360
    pub r: u16,
    pub dun: Dun,
    /// 元序 0—4（[`YUAN`]）、元内局 1—72、纪 1—6
    pub yuan: u8,
    pub ju: u8,
    pub ji: u8,
    /// 太乙所在宫（1—9，不入五）与入宫第几（1—3）
    pub gong: u8,
    pub ru_gong: u8,
    /// 天目（文昌）、计神、始击：十六神位序（[`SHEN16`]、[`POS16`]）
    pub tian_mu: usize,
    pub ji_shen: usize,
    pub shi_ji: usize,
    pub zhu_suan: u8,
    pub ke_suan: u8,
    /// 大将、参将所在宫；得五为杜（None）
    pub zhu_da: Option<u8>,
    pub zhu_can: Option<u8>,
    pub ke_da: Option<u8>,
    pub ke_can: Option<u8>,
}

fn tian_mu(r: u16, dun: Dun) -> usize {
    let (start, stay) = match dun {
        Dun::Yang => (11, [14, 10]), // 武德起，陰德、大武重留
        Dun::Yin => (3, [2, 6]),     // 呂申起，和德（艮）、大炅（巽）重留
    };
    shen16_walk(start, stay, r as i64)
}

/// 十六神顺行、遇 `stay` 二位各重留一筭：积数 `n` 以十八去之（余零作十八），自 `start` 起第一筭，
/// 返回所在位序。天目（金鏡卷一）与大游天目（金鏡卷五）共用。
pub fn shen16_walk(start: usize, stay: [usize; 2], n: i64) -> usize {
    let n = match n.rem_euclid(18) {
        0 => 18,
        k => k,
    };
    let mut p = start;
    let mut step = 1;
    let mut stayed = false;
    while step < n {
        if stay.contains(&p) && !stayed {
            stayed = true;
        } else {
            p = (p + 1) % 16;
            stayed = false;
        }
        step += 1;
    }
    p
}

/// 自十六神位 `from` 起算，至太乙前一宫止
pub fn suan(from: usize, gong: u8) -> u8 {
    let tp = gong_pos(gong);
    let mut s = pos_gong(from).unwrap_or(1) as u32;
    if from == tp {
        return s as u8;
    }
    let mut p = (from + 1) % 16;
    while p != tp {
        if let Some(g) = pos_gong(p) {
            s += g as u32;
        }
        p = (p + 1) % 16;
    }
    s as u8
}

/// 由算求将：满十去之余零；整十取十位；得五为杜
pub fn jiang(s: u32) -> Option<u8> {
    let g = if s % 10 == 0 { (s / 10) % 10 } else { s % 10 };
    (g != 5 && g != 0).then_some(g as u8)
}

/// 入局数 r 与遁之一局
pub fn ju(r: u16, dun: Dun) -> TaiyiJu {
    let r0 = r as usize - 1;
    let k24 = match r % 24 {
        0 => 24,
        k => k,
    } as usize;
    let seq: [u8; 8] = match dun {
        Dun::Yang => [1, 2, 3, 4, 6, 7, 8, 9],
        Dun::Yin => [9, 8, 7, 6, 4, 3, 2, 1],
    };
    let gong = seq[(k24 - 1) / 3];
    let tm = tian_mu(r, dun);
    let n12 = (r0 % 12) as i32;
    let js_zhi = match dun {
        Dun::Yang => (2 - n12).rem_euclid(12),
        Dun::Yin => (8 - n12).rem_euclid(12),
    } as usize;
    let js = zhi_pos(js_zhi);
    let sj = (tm + 16 + 2 - js) % 16;
    let zs = suan(tm, gong);
    let ks = suan(sj, gong);
    let can = |d: Option<u8>| d.and_then(|d| jiang(d as u32 * 3));
    let zhu_da = jiang(zs as u32);
    let ke_da = jiang(ks as u32);
    TaiyiJu {
        r,
        dun,
        yuan: (r0 / 72) as u8,
        ju: (r0 % 72 + 1) as u8,
        ji: (r0 / 60 + 1) as u8,
        gong,
        ru_gong: (r0 % 3 + 1) as u8,
        tian_mu: tm,
        ji_shen: js,
        shi_ji: sj,
        zhu_suan: zs,
        ke_suan: ks,
        zhu_da,
        zhu_can: can(zhu_da),
        ke_da,
        ke_can: can(ke_da),
    }
}

/// 计神所在之支名
pub fn ji_shen_zhi(j: &TaiyiJu) -> &'static str {
    POS16[j.ji_shen]
}

// ---------------------------------------------------------------- 三元九运

/// 三元：上元、中元、下元
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SanYuan {
    Shang,
    Zhong,
    Xia,
}

/// 九运分法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiuYun {
    /// 每运二十年，三元一百八十年九运（玄空家通说；出处未核）。缺省
    Even20,
    /// 五运不独立：前十年寄四运、后十年寄六运（玄空另说；出处未核）
    WuJiSiLiu,
}

/// 上元甲子之年：协纪卷八「康熙二十三年甲子為上元」（1684），三元一百八十年一周，故 1864、2044 亦为上元甲子
pub const SHANG_YUAN_JIAZI: i32 = 1684;

/// 干支年（立春分）所属之元与运（1—9）
pub fn san_yuan_jiu_yun(year: i32, scheme: JiuYun) -> (SanYuan, u8) {
    let k = (year - SHANG_YUAN_JIAZI).rem_euclid(180);
    let yuan = [SanYuan::Shang, SanYuan::Zhong, SanYuan::Xia][(k / 60) as usize];
    let mut yun = (k / 20 + 1) as u8;
    if scheme == JiuYun::WuJiSiLiu && yun == 5 {
        yun = if k % 20 < 10 { 4 } else { 6 };
    }
    (yuan, yun)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(p: usize) -> &'static str {
        SHEN16[p]
    }

    /// 淘金歌所举岁计诸例（集成卷六八七）：（公元年，元序，局，太乙宫，天目，计神或 None，始击或 None）
    #[test]
    fn taojinge_sui_ji_examples() {
        let cases: [(i32, u8, u8, u8, &str, Option<&str>, Option<&str>); 10] = [
            (200, 0, 17, 7, "大武", None, Some("大義")),      // 漢獻帝建安五年庚辰：第一甲子元十七局，太乙七宫，天目大武，始擊大義
            (219, 0, 36, 4, "大武", Some("高叢"), Some("大威")), // 建安二十四年己亥：一元三十六局，太乙四宫，計神高叢，始擊大威
            (554, 0, 11, 4, "高叢", None, None),             // 梁元帝承聖三年甲戌：一元十一局，太乙四宫，高叢為天目
            (551, 0, 8, 3, "陽德", Some("天道"), Some("大武")), // 梁簡文帝大寶二年辛未：一元八局，太乙三宫
            (304, 1, 49, 1, "大炅", None, None),             // 晉惠帝永興元年甲子：二丙子元四十九局，太乙一宫，天目大炅
            (611, 0, 68, 8, "大神", Some("天道"), Some("地主")), // 隋煬帝大業七年辛未：一元六十八局，太乙八宫
            (123, 4, 12, 4, "太陽", Some("高叢"), None),      // 漢安帝延光二年癸亥：五壬子元十二局，太乙四宫；始擊臨寅（间神，见下）
            (418, 3, 19, 8, "武德", Some("武德"), None),      // 晉安帝義熙十四年戊午：四庚子元十九局，太乙八宫；始擊艮（见下）
            (371, 2, 44, 8, "陽德", None, None),             // 晉簡文帝咸安元年辛未：三戊子元四十四局，太乙八宫
            (589, 0, 46, 9, "呂申", None, None),             // 陳後主禎明三年己酉：一元四十六局，太乙九宫，呂申為天目
        ];
        for (y, yuan, jun, gong, tm, js, sj) in cases {
            let j = ju(sui_ji(y, JiNian::TaoJinGe), Dun::Yang);
            assert_eq!((j.yuan, j.ju, j.gong, name(j.tian_mu)), (yuan, jun, gong, tm), "{y}");
            if let Some(js) = js {
                assert_eq!(name(j.ji_shen), js, "{y} 計神");
            }
            if let Some(sj) = sj {
                assert_eq!(name(j.shi_ji), sj, "{y} 始擊");
            }
        }
        let j = ju(sui_ji(123, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!(POS16[j.shi_ji], "寅");
        let j = ju(sui_ji(418, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!(POS16[j.shi_ji], "艮");
        // 建安五年：主筭單七、主大將七宫；客筭二十七、客大將七宫
        let j = ju(sui_ji(200, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!((j.zhu_suan, j.zhu_da, j.ke_suan, j.ke_da), (7, Some(7), 27, Some(7)));
        // 建安二十四年：客筭二十七，大將在七宫
        let j = ju(sui_ji(219, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!((j.ke_suan, j.ke_da), (27, Some(7)));
        // 咸安元年：主筭三十三，主大將三宫
        let j = ju(sui_ji(371, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!((j.zhu_suan, j.zhu_da), (33, Some(3)));
        // 禎明三年：主筭得單五，杜塞
        let j = ju(sui_ji(589, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!((j.zhu_suan, j.zhu_da), (5, None));
        // 淘金歌：至元元年甲子為上元第一紀起筭
        assert_eq!(sui_ji(1264, JiNian::TaoJinGe), 1);
    }

    /// 集成卷六八八《太乙局》四计算例（天启年间）
    #[test]
    fn jicheng_juan688_examples() {
        // 岁计：天啓四年甲子，积 10,155,541 年，餘 301，第五壬子元十三局，太乙兌六宫；
        // 計神寅，文昌大炅，主筭十八、主大將八、主參將四，始擊太陽，客筭十九、客大將九、客參將七
        assert_eq!(10_155_541i64.rem_euclid(360), sui_ji(1624, JiNian::JinJing) as i64);
        let j = ju(sui_ji(1624, JiNian::JinJing), Dun::Yang);
        assert_eq!((j.r, j.yuan, j.ju, j.gong, name(j.tian_mu), POS16[j.ji_shen], name(j.shi_ji)), (301, 4, 13, 6, "大炅", "寅", "太陽"));
        assert_eq!((j.zhu_suan, j.zhu_da, j.zhu_can, j.ke_suan, j.ke_da, j.ke_can), (18, Some(8), Some(4), 19, Some(9), Some(7)));
        // 月计：天啓四年正月丙寅，第一甲子元三局，太乙乾一宫；計神坎（子），文星陰主，主筭一、主大將一、主參將三，
        // 始擊大義，客筭四十、客大將四宫、客參將二宫
        let j = ju(yue_ji(1624, 0, JiNian::JinJing), Dun::Yang);
        assert_eq!((j.r, j.gong, POS16[j.ji_shen], name(j.tian_mu), name(j.shi_ji)), (3, 1, "子", "陰主", "大義"));
        assert_eq!((j.zhu_suan, j.zhu_da, j.zhu_can, j.ke_suan, j.ke_da, j.ke_can), (1, Some(1), Some(3), 40, Some(4), Some(2)));
        assert_eq!(yue_ji(1624, 0, JiNian::JinJing), yue_ji(1624, 0, JiNian::TaoJinGe));
        // 日计：天啓三年十一月初八甲子（1623-12-29，JDN 2314211），餘 181，第三戊子元三十七局，太乙兌宫；
        // 文昌武德，主筭一、主大將一、主參將三，始擊大武，客筭七、客大將七、客參將一
        let j = ju(ri_ji(2_314_211), Dun::Yang);
        assert_eq!((j.r, j.yuan, j.ju, j.gong, name(j.tian_mu), name(j.shi_ji)), (181, 2, 37, 6, "武德", "大武"));
        assert_eq!((j.zhu_suan, j.zhu_da, j.zhu_can, j.ke_suan, j.ke_da, j.ke_can), (1, Some(1), Some(3), 7, Some(7), Some(1)));
        // 时计：冬至後甲子日甲子時，陽遁第一甲子元第一局，太乙乾一宫；計神寅，文昌武德，主筭七、主大將七、主參將一，
        // 始擊大武，客筭十三、客大將三、客參將九
        let j = ju(shi_ji(0, 0), Dun::Yang);
        assert_eq!((j.r, j.gong, POS16[j.ji_shen], name(j.tian_mu), name(j.shi_ji)), (1, 1, "寅", "武德", "大武"));
        assert_eq!((j.zhu_suan, j.zhu_da, j.zhu_can, j.ke_suan, j.ke_da, j.ke_can), (7, Some(7), Some(1), 13, Some(3), Some(9)));
    }

    /// 金鏡卷一诸例
    #[test]
    fn jinjing_juan1_examples() {
        // 「自上元混沌甲子之嵗至今大唐開元十二年甲子嵗積得一百九十三萬七千二百八十一筭」；太岁：积年减一以六十去之命甲子
        assert_eq!(ji_nian(724, JiNian::JinJing), 1_937_281);
        assert_eq!((ji_nian(724, JiNian::JinJing) - 1).rem_euclid(60), 0);
        // 梁天監三年甲申六月八日：積日七億七百五十萬一千六十一，以周紀法除之不盡二百六十一，「入第五紀」；
        // 虞履：「今日太乙在八宫，和徳為天目」
        assert_eq!(ri_ji(1_905_331), (707_501_061i64 % 360) as u16);
        let j = ju(ri_ji(1_905_331), Dun::Yang);
        assert_eq!((j.r, j.ji, j.gong, name(j.tian_mu)), (261, 5, 8, "和德"));
        assert_eq!((1_905_331i64 + 49).rem_euclid(60), 20); // 甲申
        // 时计例：十月五日庚申時加戊寅，入立冬陰遁，第六紀壬子元二十七局，太乙九宫，大武為天目，午為計神
        let j = ju(shi_ji(56, 2), Dun::Yin);
        assert_eq!((j.r, j.ji, j.yuan, j.ju, j.gong, name(j.tian_mu), POS16[j.ji_shen]), (315, 6, 4, 27, 9, "大武", "午"));
        // 同例：「主筭得二十九主大將在九宫主參將在七宫以計神加和徳宫天目下臨髙叢卯客筭得單四客大將在四宫參將在二宫」
        assert_eq!((j.zhu_suan, j.zhu_da, j.zhu_can, name(j.shi_ji), j.ke_suan, j.ke_da, j.ke_can), (29, Some(9), Some(7), "高叢", 4, Some(4), Some(2)));
        // 六纪表：阳遁乾一至六纪夜半甲子太乙一、六、一、六、一、六宫，天目武德、地主、大炅、武德、地主、大炅，計神寅；
        // 阴遁巽一至六纪太乙九、四、九、四、九、四宫，天目呂申、大威、陰德、呂申、大威、陰德，計神申
        let yang = ["武德", "地主", "大炅", "武德", "地主", "大炅"];
        let yin = ["呂申", "大威", "陰德", "呂申", "大威", "陰德"];
        for k in 0..6u16 {
            let r = 1 + 60 * k;
            let a = ju(r, Dun::Yang);
            assert_eq!((a.gong, name(a.tian_mu), POS16[a.ji_shen]), ([1, 6][k as usize % 2], yang[k as usize], "寅"), "陽 {k}");
            let b = ju(r, Dun::Yin);
            assert_eq!((b.gong, name(b.tian_mu), POS16[b.ji_shen]), ([9, 4][k as usize % 2], yin[k as usize], "申"), "陰 {k}");
        }
    }

    /// 太乙秘書：万历戊子岁计，阳遁一宫、阴遁九宫；计神阳寅阴申；始击俱在辰；
    /// 阳遁主算二十四、客算二十五（客将杜）、主大四宫主参二宫；阴遁主算十六、客算单一、主大六宫主参八宫、客大一宫客参三宫
    #[test]
    fn taiyi_mishu_1588() {
        assert_eq!(sui_ji(1588, JiNian::JinJing), (10_155_505i64 % 360) as u16);
        assert_eq!(ji_nian(1588, JiNian::JinJing), 1_938_145);
        let r = sui_ji(1588, JiNian::JinJing);
        let a = ju(r, Dun::Yang);
        assert_eq!((a.gong, POS16[a.ji_shen], POS16[a.shi_ji], a.zhu_suan, a.ke_suan), (1, "寅", "辰", 24, 25));
        assert_eq!((a.zhu_da, a.zhu_can, a.ke_da), (Some(4), Some(2), None));
        let b = ju(r, Dun::Yin);
        assert_eq!((b.gong, POS16[b.ji_shen], POS16[b.shi_ji], b.zhu_suan, b.ke_suan), (9, "申", "辰", 16, 1));
        assert_eq!((b.zhu_da, b.zhu_can, b.ke_da, b.ke_can), (Some(6), Some(8), Some(1), Some(3)));
    }

    /// 金鏡卷三「陽局/隂局天目地目計神主客大小將立成」七十二局逐局比对（太乙、天目、主算、客目、客算、计神）
    #[test]
    fn jinjing_juan3_li_cheng() {
        let rows = jinjing_juan3_check();
        assert_eq!(rows.len(), 144, "阴阳各七十二局");
        let bad: Vec<_> = rows.iter().filter(|r| !r.2.is_empty()).collect();
        let n: usize = bad.iter().map(|r| r.2.len()).sum();
        assert!(n <= JINJING_JUAN3_KNOWN_BAD, "{n}：{bad:#?}");
    }

    /// 金鏡卷三立成已知不合项数（逐项见 doc/folk/taiyi.md）
    const JINJING_JUAN3_KNOWN_BAD: usize = 14;

    /// 返回（遁、局、不合项（项，书，推））
    pub(crate) fn jinjing_juan3_check() -> Vec<(Dun, u16, Vec<(&'static str, String, String)>)> {
        let path = crate::repo::repo_root().join("vendor/folk/wikisource/太乙金鏡式經/卷03.wikitext");
        let raw = std::fs::read_to_string(path).expect("卷三录文");
        // 去夹注 {{SK notes|…}}
        let mut text = String::new();
        let mut rest = raw.as_str();
        while let Some(i) = rest.find("{{SK notes|") {
            text.push_str(&rest[..i]);
            rest = &rest[i..];
            let e = rest.find("}}").map(|e| e + 2).unwrap_or(rest.len());
            rest = &rest[e..];
        }
        text.push_str(rest);
        let text = text
            .replace('夘', "卯")
            .replace('戊', "戌")
            .replace('吕', "呂")
            .replace('髙', "高")
            .replace('徳', "德")
            .replace('隂', "陰")
            .replace("太蔟", "太簇")
            .replace("太炅", "大炅");
        let yang = text.find("陽局天目地目計神主客大小將立成").unwrap();
        let yin = text.find("陰局天目地目計神主客大小將立成").unwrap();
        let end = text.rfind("五子元積年立成法").unwrap();
        #[derive(Clone, Debug, PartialEq)]
        enum U {
            N(u16),
            S(usize),
            Z(usize),
        }
        let units = |seg: &str| -> Vec<U> {
            let cs: Vec<char> = seg.chars().collect();
            let mut out = Vec::new();
            let mut i = 0;
            let d = |c: char| "一二三四五六七八九".chars().position(|x| x == c).map(|p| p as u16 + 1);
            while i < cs.len() {
                let c = cs[i];
                if d(c).is_some() || c == '十' {
                    let mut k = i;
                    while k < cs.len() && (d(cs[k]).is_some() || cs[k] == '十') {
                        k += 1;
                    }
                    let w: Vec<char> = cs[i..k].to_vec();
                    let v = match w.as_slice() {
                        [a] => d(*a).unwrap_or(10),
                        ['十', b] => 10 + d(*b).unwrap_or(0),
                        [a, '十'] => d(*a).unwrap_or(0) * 10,
                        [a, '十', b] => d(*a).unwrap_or(0) * 10 + d(*b).unwrap_or(0),
                        _ => 0,
                    };
                    out.push(U::N(v));
                    i = k;
                    continue;
                }
                if i + 1 < cs.len() {
                    let two: String = cs[i..i + 2].iter().collect();
                    if let Some(p) = SHEN16.iter().position(|n| *n == two) {
                        out.push(U::S(p));
                        i += 2;
                        continue;
                    }
                }
                if let Some(z) = "子丑寅卯辰巳午未申酉戌亥".chars().position(|x| x == c) {
                    out.push(U::Z(z));
                }
                i += 1;
            }
            out
        };
        let mut out = Vec::new();
        for (dun, seg) in [(Dun::Yang, &text[yang..yin]), (Dun::Yin, &text[yin..end])] {
            let us = units(seg);
            let mut i = 2;
            while i < us.len() {
                let (U::N(r), U::N(g), U::S(tm)) = (&us[i - 2], &us[i - 1], &us[i]) else {
                    i += 1;
                    continue;
                };
                let next = |from: usize, f: &dyn Fn(&U) -> bool| (from..us.len()).find(|&k| f(&us[k]));
                let Some(a) = next(i + 1, &|u| matches!(u, U::N(_))) else { break };
                let Some(b) = next(a + 1, &|u| matches!(u, U::S(_))) else { break };
                let Some(c) = next(b + 1, &|u| matches!(u, U::N(_))) else { break };
                let Some(e) = next(c + 1, &|u| matches!(u, U::Z(_))) else { break };
                let (U::N(zs), U::S(ke), U::N(ks), U::Z(js)) = (&us[a], &us[b], &us[c], &us[e]) else { unreachable!() };
                let j = ju(*r, dun);
                let mut diff = Vec::new();
                let mut cmp = |k: &'static str, book: String, got: String| {
                    if book != got {
                        diff.push((k, book, got));
                    }
                };
                cmp("太乙", g.to_string(), j.gong.to_string());
                cmp("天目", SHEN16[*tm].into(), SHEN16[j.tian_mu].into());
                cmp("主算", zs.to_string(), j.zhu_suan.to_string());
                cmp("客目", SHEN16[*ke].into(), SHEN16[j.shi_ji].into());
                cmp("客算", ks.to_string(), j.ke_suan.to_string());
                cmp("计神", POS16[zhi_pos(*js)].into(), POS16[j.ji_shen].into());
                out.push((dun, *r, diff));
                i = e + 1;
            }
        }
        out
    }

    /// 打印金鏡卷三立成比对不合项（`-- --ignored --nocapture jinjing_juan3_dump`）
    #[test]
    #[ignore]
    fn jinjing_juan3_dump() {
        let rows = jinjing_juan3_check();
        let cell = |d: Dun, r: u16| {
            let x = rows.iter().find(|x| x.0 == d && x.1 == r).expect("局");
            if x.2.is_empty() {
                "合".to_string()
            } else {
                x.2.iter().map(|(k, b, g)| format!("{k}書{b}推{g}")).collect::<Vec<_>>().join("；")
            }
        };
        for r in 1..=72u16 {
            println!("| {} | {} | {} |", r, cell(Dun::Yang, r), cell(Dun::Yin, r));
        }
    }

    /// 金鏡章岁法（录文讹字校正后）：梁天监三年六月经朔积日、干支合实历朔日；与月计节气法之差
    #[test]
    fn zhang_sui_fa() {
        let (m0, run) = zhang_sui_ji_yue(504, JiNian::JinJing);
        assert_eq!((m0, run), (23_958_218, 334));
        // 天正十一月起第七月（六月）经朔积日 707,501,052，以六十去之余 12 命甲子为丙子；实历朔日表此月朔丙子（JDN 1,905,323）
        let (d, _) = jing_shuo_ji_ri(m0 + 7);
        assert_eq!(d, 707_501_052);
        assert_eq!(d.rem_euclid(60), 12);
        assert_eq!((1_905_323 + 49) % 60, 12);
        // 初八（JDN 1,905,330，实历癸未）在第七月第八日；积日 707,501,060，书作 707,501,061（差一）
        assert_eq!(jing_shuo_yue(1_905_330), (m0 + 7, 8));
        assert_eq!(1_905_330 + RI_JI_OFFSET, 707_501_060);
        // 录文「章月八千七百二十六」「月法二千四百四十七」不可通
        assert!(8726 * 19 / 657 > 13 * 19 && YUE_FA * 100 / RI_FA == 2953);
        // 月计章岁法计入闰月：天启四年正月（1624-02-19 后，JDN 2,314,260）与节气法之 3 不同
        let r = yue_ji_zhang_sui(2_314_260);
        assert_ne!(r, 3);
        assert_eq!(r, 33);
    }

    #[test]
    fn san_yuan() {
        assert_eq!(san_yuan_jiu_yun(1684, JiuYun::Even20), (SanYuan::Shang, 1));
        assert_eq!(san_yuan_jiu_yun(1864, JiuYun::Even20), (SanYuan::Shang, 1));
        assert_eq!(san_yuan_jiu_yun(1924, JiuYun::Even20), (SanYuan::Zhong, 4));
        assert_eq!(san_yuan_jiu_yun(1944, JiuYun::Even20), (SanYuan::Zhong, 5));
        assert_eq!(san_yuan_jiu_yun(1944, JiuYun::WuJiSiLiu), (SanYuan::Zhong, 4));
        assert_eq!(san_yuan_jiu_yun(1954, JiuYun::WuJiSiLiu), (SanYuan::Zhong, 6));
        assert_eq!(san_yuan_jiu_yun(2004, JiuYun::Even20), (SanYuan::Xia, 8));
        assert_eq!(san_yuan_jiu_yun(2024, JiuYun::Even20), (SanYuan::Xia, 9));
        assert_eq!(san_yuan_jiu_yun(2044, JiuYun::Even20), (SanYuan::Shang, 1));
    }
}
