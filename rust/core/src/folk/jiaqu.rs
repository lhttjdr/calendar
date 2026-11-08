//! 婚嫁择吉：按新人之命择嫁娶年月日时。
//!
//! 两系并列（见 doc/folk/jiaqu.md）：
//! - **协纪**（缺省，清官定）：嫁娶宜忌取月表加卷十铺注与嫁娶周堂（[`super::puzhu::yiji_for_day`]），
//!   不将取卷四所引《歷例》逐月表；女命行嫁大利月诸说，协纪卷三十六辨訛斥为「惑世誣民之尤甚者」，「今已奏准刪除」。
//! - **类编**（题元宋魯珍《類編曆法通書大全》卷七「結婚嫁娶」，北大藏本，`02094406.cn` PDF 第 30—65 页）：
//!   女命大利月与妨亲属月（出呂才《大義婚書》）、阴阳不将起例、嫁娶凶神表、白虎入中宫、纳婿周堂、行嫁白虎。
//!
//! 只录规则，不录应验之说。

use super::puzhu::{yiji_for_day, zhou_tang, PuzhuDay};
use super::zhiri::xiu_index;

fn gz_index(name: &str) -> u8 {
    const G: &str = "甲乙丙丁戊己庚辛壬癸";
    let cs: Vec<char> = name.chars().collect();
    let g = G.chars().position(|c| c == cs[0]).unwrap() as u8;
    let z = "子丑寅卯辰巳午未申酉戌亥".chars().position(|c| c == cs[1]).unwrap() as u8;
    (0..60u8).find(|k| k % 10 == g && k % 12 == z).unwrap()
}

// ───────────────────────── 年月层：女命行嫁月 ─────────────────────────

/// 女命行嫁月之类（类编卷七表，PDF 第 31 页；协纪卷三十六述其法而斥之）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NvMingYue {
    /// 大利月
    DaLi,
    /// 妨媒氏首子（俗称小利月，名称出处未核）
    FangMeiShi,
    FangWengGu,
    FangNvFuMu,
    FangFu,
    FangNvShen,
}

impl NvMingYue {
    pub fn name(self) -> &'static str {
        match self {
            NvMingYue::DaLi => "大利月",
            NvMingYue::FangMeiShi => "妨媒氏首子",
            NvMingYue::FangWengGu => "妨翁姑",
            NvMingYue::FangNvFuMu => "妨女父母",
            NvMingYue::FangFu => "妨夫主",
            NvMingYue::FangNvShen => "妨女身",
        }
    }
}

/// 女命（生年地支，年以立春分）在某月（月建序，寅月 0）之类。
///
/// 协纪卷三十六：「以女命為主，子寅辰午申戌六陽年自本命前一月向前順數，丑夘巳未酉亥六隂年自本命後一月向後逆數，
/// 第一月為大利，第二月妨媒氏首子，第三月妨翁姑，第四月妨女父母，第五月妨夫，第六月妨本身，至第七月又復一轉」。
/// 与类编卷七表逐格相合（见测试）。月以月建（节）计。
pub fn nv_ming_yue(nv_zhi: u8, month_ord: u8) -> NvMingYue {
    let z = (nv_zhi % 12) as i32;
    let month_zhi = (month_ord as i32 + 2).rem_euclid(12);
    // 阳年本命前一月起顺数，阴年本命后一月起逆数
    let (start, dir) = if z % 2 == 0 { (z + 1, 1) } else { (z - 1, -1) };
    let k = ((month_zhi - start) * dir).rem_euclid(6) as usize;
    [NvMingYue::DaLi, NvMingYue::FangMeiShi, NvMingYue::FangWengGu, NvMingYue::FangNvFuMu, NvMingYue::FangFu, NvMingYue::FangNvShen][k]
}

// ───────────────────────── 阴阳不将 ─────────────────────────

/// 一日相对月厌的将否（类编卷七「隂陽不將例日」图与「論隂陽不將日」，PDF 第 32、50 页；协纪卷四引《天寶厯》同法）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jiang {
    /// 厌前干配厌后支：阴阳不将（吉）
    BuJiang,
    /// 厌前干支自相配：阳将（「陽將男亡」）
    YangJiang,
    /// 厌后干支自相配：阴将
    YinJiang,
    /// 厌后干配厌前支：阴阳俱将
    JuJiang,
    /// 支当月厌或厌对，不入前后
    Neither,
}

/// 二十四山自子顺行：子 癸 丑 艮 寅 甲 卯 乙 辰 巽 巳 丙 午 丁 未 坤 申 庚 酉 辛 戌 乾 亥 壬
fn shan_of_gan(gan: u8) -> Option<i32> {
    // 甲 5、乙 7、丙 11、丁 13、庚 17、辛 19、壬 23、癸 1；戊己无方位
    [Some(5), Some(7), Some(11), Some(13), None, None, Some(17), Some(19), Some(23), Some(1)][gan as usize % 10]
}

/// 戊己之寄
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WuJiJi {
    /// 类编：「如戊巳日干亦寄月厭之前」
    LeiBian,
    /// 协纪卷四引《天寶厯》：「戊為陽土寄於艮，己為隂土寄於坤……春冬己不將，秋夏戊不將」
    XieJi,
}

/// 类编之法：以月厌（正月起戌逆行）为界，二十四山上厌前十一位为前（阳），厌后十一位为后（阴）。
pub fn jiang(month_ord: u8, day_gz: u8, wuji: WuJiJi) -> Jiang {
    let yan_zhi = (10 - month_ord as i32).rem_euclid(12); // 月厌：正戌、二酉……
    let y = yan_zhi * 2;
    // 前：y−11 … y−1；后：y+1 … y+11
    let side = |p: i32| -> Option<bool> {
        let d = (p - y).rem_euclid(24);
        if d == 0 || d == 12 {
            None
        } else {
            Some(d > 12) // true = 前（阳）
        }
    };
    let gan = day_gz % 10;
    let zhi_side = side((day_gz % 12) as i32 * 2);
    let gan_side = match shan_of_gan(gan) {
        Some(p) => side(p),
        None => match wuji {
            WuJiJi::LeiBian => Some(true),
            WuJiJi::XieJi => side(if gan == 4 { 3 } else { 15 }), // 戊寄艮、己寄坤
        },
    };
    match (gan_side, zhi_side) {
        (Some(true), Some(false)) => Jiang::BuJiang,
        (Some(true), Some(true)) => Jiang::YangJiang,
        (Some(false), Some(false)) => Jiang::YinJiang,
        (Some(false), Some(true)) => Jiang::JuJiang,
        _ => Jiang::Neither,
    }
}

/// 协纪卷四所引《歷例》逐月不将日（月表「不將」所据；八月「癸己」作癸巳）
pub fn bu_jiang_xieji(month_ord: u8, day_gz: u8) -> bool {
    super::yiji::yuebiao_day(month_ord, day_gz).ji_shen.iter().any(|s| s == "不將")
}

// ───────────────────────── 类编嫁娶凶神（按月） ─────────────────────────

/// 一日所犯类编卷七之嫁娶凶神：(神名, 所忌)。月以月建（节）计（`month_ord` 寅月 0），`month_gan` 为月建之干。
/// 表见 PDF 第 36—39 页（凶日、往亡以下、四离四绝、白虎入中宫）；「論嫁娶凶日」见第 49 页。
pub fn leibian_xiong(month_ord: u8, month_gan: u8, day_gz: u8, jdn: i64) -> Vec<(&'static str, &'static str)> {
    let m = (month_ord % 12) as usize;
    let z = day_gz % 12;
    let zi = |s: &str| "子丑寅卯辰巳午未申酉戌亥".chars().position(|c| s.starts_with(c)).unwrap() as u8;
    let col = |s: &str| -> u8 { zi(&s.chars().nth(m).unwrap().to_string()) };
    let mut v = Vec::new();
    // 第 36 页（以月为列，正月至十二月）
    const TAB: [(&str, &str, &str); 17] = [
        ("歸忌", "娶婦", "丑寅子丑寅子丑寅子丑寅子"),
        ("月厭", "殺公", "戌酉申未午巳辰卯寅丑子亥"),
        ("厭對", "殺姑", "辰卯寅丑子亥戌酉申未午巳"),
        // 第 37 页
        ("天賊", "行嫁", "辰酉寅未子巳戌卯申丑午亥"),
        ("月破", "娶婦", "申酉戌亥子丑寅卯辰巳午未"),
        ("受死", "娶婦", "戌辰亥巳子午丑未寅申卯酉"),
        ("天寡", "嫁女", "卯卯卯午午午酉酉酉子子子"),
        ("地寡", "嫁女", "酉酉酉子子子卯卯卯午午午"),
        // 书作四月一换；协纪卷三十六引《轉神厯》作孟酉仲巳季丑，互异
        ("紅沙殺", "娶婦", "酉酉酉酉巳巳巳巳丑丑丑丑"),
        ("披麻殺", "嫁娶", "子酉午卯子酉午卯子酉午卯"),
        ("天罡勾絞", "娶婦", "巳子未寅酉辰亥午丑申卯戌"),
        ("河魁勾絞", "娶婦", "亥午丑申卯戌巳子未寅酉辰"),
        ("吟神勾絞", "曆忌", "酉巳丑酉巳丑酉巳丑酉巳丑"),
        ("天雄", "", "戌亥子丑寅卯辰巳午未申酉"),
        ("地雌", "", "辰巳午未申酉戌亥子丑寅卯"),
        // 第 38 页
        ("往亡", "行嫁", "寅巳申亥卯午酉子辰未戌丑"),
        ("無翹", "婚姻", "亥戌酉申未午巳辰卯寅丑子"),
    ];
    for (n, ji, s) in TAB {
        if col(s) == z {
            v.push((n, ji));
        }
    }
    // 荒蕪（书作每月一支：巳酉丑辰申子卯未亥寅戌午）
    if col("巳酉丑辰申子卯未亥寅戌午") == z {
        v.push(("荒蕪", ""));
    }
    // 陰錯、陽錯（娶婦）：按干支
    const YIN_CUO: [&str; 12] = ["庚戌", "辛酉", "庚申", "丁未", "丙午", "丁巳", "甲辰", "乙卯", "甲寅", "癸丑", "壬子", "癸亥"];
    const YANG_CUO: [&str; 12] = ["甲寅", "乙卯", "甲辰", "丁巳", "丙午", "丁未", "庚申", "辛酉", "庚戌", "癸亥", "壬子", "癸丑"];
    if gz_index(YIN_CUO[m]) == day_gz {
        v.push(("陰錯", "娶婦"));
    }
    if gz_index(YANG_CUO[m]) == day_gz {
        v.push(("陽錯", "娶婦"));
    }
    // 伏斷：日支与值日宿（子虛丑斗寅室卯女辰箕巳房午角未張申鬼酉觜戌胃亥壁）
    const FU_DUAN_XIU: [u8; 12] = [10, 7, 12, 9, 6, 3, 0, 25, 22, 19, 16, 13];
    if xiu_index(jdn) == FU_DUAN_XIU[z as usize] {
        v.push(("伏斷", ""));
    }
    // 人民離（戊申己酉）、五亥日（主生離）：第 39 页
    if day_gz == gz_index("戊申") || day_gz == gz_index("己酉") {
        v.push(("人民離", ""));
    }
    if z == 11 {
        v.push(("亥日", "嫁娶")); // 「五亥日主生離」；百忌日「亥不嫁娶」
    }
    // 大殺白虎入中宮日：戊辰丁丑丙戌乙未甲辰癸丑壬戌（每九日一位）
    if ["戊辰", "丁丑", "丙戌", "乙未", "甲辰", "癸丑", "壬戌"].iter().any(|g| gz_index(g) == day_gz) {
        v.push(("大殺白虎入中宮", ""));
    }
    // 雷霆白虎入中宮日：按月干（书作「甲己月」「乙庚月」「丙辛戊癸月」「丁壬月」；「月」指月干抑年干，原文未明，此取月建之干，未核）
    // 干合五组：甲己 0、乙庚 1、丙辛 2、丁壬 3、戊癸 4；书以丙辛、戊癸同一行
    let lt: &[&str] = match month_gan % 5 {
        0 => &["丁卯", "丙子", "乙酉", "甲午", "癸卯", "壬子", "辛酉"],
        1 => &["戊辰", "丁丑", "丙戌", "乙未", "甲辰", "癸丑", "壬戌"],
        3 => &["乙丑", "甲戌", "癸未", "壬辰", "辛丑", "庚戌", "己未"],
        _ => &["辛未", "庚辰", "己丑", "戊戌", "丁未", "丙辰"],
    };
    if lt.iter().any(|g| gz_index(g) == day_gz) {
        v.push(("雷霆白虎入中宮", ""));
    }
    v
}

/// 类编「論嫁娶凶日」所列（第 49 页）：月厭、厭對、歸忌、赤口、天賊、月破、四離、四絶、五亥日、大殺白虎入中宫日、
/// 雷霆白虎入中宫日、人民離日。赤口（小六壬留連赤口）未实现。受死书云「今人多不忌」，月忌日书云「不忌嫁娶」，只作附注。
pub const LEIBIAN_XIONG_RI: [&str; 10] = ["月厭", "厭對", "歸忌", "天賊", "月破", "亥日", "大殺白虎入中宮", "雷霆白虎入中宮", "人民離", "披麻殺"];

// ───────────────────────── 周堂 ─────────────────────────

/// 行嫁白虎（第 40 页）：大月从竈顺数，小月从厨逆数；「只論月分大小不問節氣」。书论「如其日值路……若值其餘亦不必忌」
pub fn xing_jia_bai_hu(days_in_month: u8, lunar_day: u8) -> Option<&'static str> {
    const DA: [&str; 8] = ["竈", "堂", "床", "死", "睡", "門", "路", "厨"];
    const XIAO: [&str; 8] = ["厨", "路", "門", "睡", "死", "床", "堂", "竈"];
    (lunar_day >= 1).then(|| if days_in_month >= 30 { DA } else { XIAO }[(lunar_day as usize - 1) % 8])
}

/// 纳婿周堂（第 41 页）：大月从夫顺至姑，小月从户逆至厨。书论「值姑第翁可出外少避，值夫别擇日」
pub fn na_xu_zhou_tang(days_in_month: u8, lunar_day: u8) -> Option<&'static str> {
    const DA: [&str; 8] = ["夫", "姑", "第", "翁", "門", "竈", "厨", "戸"];
    const XIAO: [&str; 8] = ["戸", "厨", "竈", "門", "翁", "第", "姑", "夫"];
    (lunar_day >= 1).then(|| if days_in_month >= 30 { DA } else { XIAO }[(lunar_day as usize - 1) % 8])
}

/// 嫁娶周堂值位之用
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZhouTangRule {
    /// 不用嫁娶周堂
    BuYong,
    /// 协纪卷三十五附录：「遇第堂厨竈日用之，如遇翁姑而無翁姑者亦可用」；时宪书于值夫、婦之日不注嫁娶（乾隆三十年核）。
    /// 故值夫婦不用，值翁姑则有翁姑在者不用
    XieJi,
    /// 类编：值婦「別擇日」；值翁姑及夫「時俗有從權出外少避」（视为可制，见 [`JiaquOptions::zhi_hua`]）
    LeiBian,
}

/// 行嫁白虎须禳之位
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaiHuRule {
    /// 类编卷七（第 51 页）：「如其日值路，先就門外路斬牲以禳之，若值其餘亦不必忌」
    LeiBian,
    /// 鳌头卷三（第 4 册第 14 页）：「如在死睡厨竈吉，如值門床堂路則祈禳之吉」
    AoTou,
}

// ───────────────────────── 制化 ─────────────────────────

/// 类编卷七白虎入中宫之禳（第 50 页「論大殺白虎雷霆白虎」）
pub const JIE_BAI_HU_ZHONG_GONG: &str = "禳：新人入門時於廳中斬牲以魘禳，然後入廳下轎；不在正堂，忌廳堂大宴；鼓樂從門外遠處起動行入（类编卷七第 50 页「論大殺白虎雷霆白虎」「論白虎日娶婦用皷樂」）";
/// 行嫁白虎之禳
pub const JIE_XING_JIA_BAI_HU_LEIBIAN: &str = "禳：先就門外路斬牲以禳之（类编卷七第 51 页「論行嫁白虎」）";
pub const JIE_XING_JIA_BAI_HU_AOTOU: &str = "祈禳之（鳌头通书卷三第 4 册第 14 页「行嫁周堂」）";
/// 周堂值翁姑夫之避
pub const JIE_ZHOU_TANG_BI: &str = "回避：新人入門時翁姑及夫從權出外少避，候新人坐床方可廻家（类编卷七第 51 页「論嫁娶周堂」；鳌头卷三「如值翁姑宜出宅少避」）";
/// 纳婿周堂值姑第翁之避
pub const JIE_NA_XU_BI: &str = "回避：可出外少避（类编卷七第 51 页「論納婿周堂」）";
/// 受死之禳
pub const JIE_SHOU_SI: &str = "禳：新人入門時斬牲於地以魘禳（类编卷七第 49 页「論受死日」，并云「今人多不忌」）";

/// 判定三态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// 无否决
    KeYong,
    /// 否决皆可制，经制化可用（须选项「用制化」）
    ZhiHuaKeYong,
    /// 有不可制之否决
    BuKeYong,
}

// ───────────────────────── 逐日判定 ─────────────────────────

/// 新人与家人
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JiaquInput {
    /// 女命生年干支序（年以立春分：类编「論男女生月看節氣……交立春節以後生命並作次年正月」）
    pub nv_year_gz: u8,
    /// 有翁姑（夫之父母）在
    pub has_weng_gu: bool,
    /// 女父母在
    pub has_nv_fu_mu: bool,
    /// 纳婿（男就女家）：用纳婿周堂
    pub na_xu: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JiaquOptions {
    /// 协纪月表（加铺注）之嫁娶宜忌如何用：须宜嫁娶（缺省）、只避忌嫁娶、不用
    pub xieji_yiji: XieJiYiJi,
    /// 用女命行嫁月（类编；协纪奏准删除）——缺省 false：只列出不据以否决
    pub nv_ming_yue: bool,
    pub zhou_tang: ZhouTangRule,
    /// 用类编嫁娶凶神表否决——缺省 false：只列出
    pub leibian_xiong: bool,
    /// 行嫁白虎须禳之位（启用类编凶神时据以否决）
    pub bai_hu: BaiHuRule,
    /// 用制化：可制之否决移入 `jie`，判「经制化可用」——缺省 false。
    /// 理由：协纪于嫁娶凶神之解只取诸德（卷十宜忌等第表「遇德从宜」，已含于月表），不录禳、避之法；
    /// 《董公選擇日要覽》斥「或書一符、或書一字以為鎮魔，欺惑世人」，于白虎入中宫谓「縱有……天德月德可以化解……莫若不用」
    pub zhi_hua: bool,
    /// 阴阳不将之戊己寄法（仅作附注；协纪逐月表另列）
    pub wuji: WuJiJi,
}

impl Default for JiaquOptions {
    fn default() -> Self {
        Self {
            xieji_yiji: XieJiYiJi::XuYi,
            nv_ming_yue: false,
            zhou_tang: ZhouTangRule::XieJi,
            leibian_xiong: false,
            bai_hu: BaiHuRule::LeiBian,
            zhi_hua: false,
            wuji: WuJiJi::LeiBian,
        }
    }
}

/// 协纪月表嫁娶宜忌之用法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XieJiYiJi {
    /// 须月表（加铺注）宜嫁娶方用（时宪书注宜之日；缺省）
    XuYi,
    /// 只避忌嫁娶：宜忌两不注之日亦可
    BiJi,
    /// 不用协纪月表
    BuYong,
}

/// 一日之判
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JiaquDay {
    pub jdn: i64,
    /// 三态：可用、经制化可用、不可用
    pub verdict: Verdict,
    /// 协纪月表 + 铺注：宜嫁娶 / 忌嫁娶 / 两不注
    pub xieji_yi: bool,
    pub xieji_ji: bool,
    /// 宜結婚姻、納采問名（议婚、下定）
    pub yi_jie_hun_yin: bool,
    pub yi_na_cai: bool,
    pub nv_ming_yue: NvMingYue,
    pub zhou_tang: Option<&'static str>,
    pub xing_jia_bai_hu: Option<&'static str>,
    pub na_xu_zhou_tang: Option<&'static str>,
    /// 协纪《歷例》不将、类编法将否
    pub bu_jiang_xieji: bool,
    pub jiang: Jiang,
    /// 类编凶神 (名, 所忌)
    pub leibian_xiong: Vec<(&'static str, &'static str)>,
    /// 不可制（或未用制化）之否决
    pub reasons: Vec<String>,
    /// 经制化之否决：(所制之忌, 制法与出处)。仅「用制化」时有
    pub jie: Vec<(String, String)>,
    /// 附注（不否决）
    pub notes: Vec<String>,
}

impl JiaquDay {
    /// 可用（含经制化）
    pub fn usable(&self) -> bool {
        self.verdict != Verdict::BuKeYong
    }
}

/// 逐日判定。`d` 由 [`super::puzhu::puzhu_days`] 或 `TermTable::day` 得来。
pub fn judge_day(d: &PuzhuDay, inp: &JiaquInput, o: &JiaquOptions) -> JiaquDay {
    use super::yiji::yuebiao_day;
    let yb = yiji_for_day(d);
    let has = |v: &Option<Vec<String>>, s: &str| v.as_ref().is_some_and(|v| v.iter().any(|x| x == s));
    let xieji_yi = has(&yb.yi, "嫁娶");
    let xieji_ji = yb.yi.is_none() || yb.ji.iter().any(|x| x == "嫁娶");
    let gz = d.input.day_gz;
    let (dim, ld) = (d.input.days_in_month, d.input.lunar_day);
    let yue = nv_ming_yue(inp.nv_year_gz % 12, d.month_ord);
    let zt = zhou_tang(dim, ld);
    let bh = xing_jia_bai_hu(dim, ld);
    let nx = na_xu_zhou_tang(dim, ld);
    let lx = leibian_xiong(d.month_ord, d.month_gz % 10, gz, d.jdn);
    // 否决：(由, 可制之法)
    let mut veto: Vec<(String, Option<&'static str>)> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // 协纪：须宜嫁娶（月表 + 铺注；铺注已于周堂值夫婦去宜）。
    // 月表本宜嫁娶而只因周堂值夫去宜者，类编以为可避（值婦则不可）
    if o.xieji_yiji != XieJiYiJi::BuYong && !xieji_yi && (o.xieji_yiji == XieJiYiJi::XuYi || xieji_ji) {
        let base_yi = yuebiao_day(d.month_ord, gz).yi.as_ref().is_some_and(|v| v.iter().any(|x| x == "嫁娶"));
        let by_zhou_tang_fu = zt == Some("夫") && base_yi && !xieji_ji;
        if xieji_ji {
            veto.push(("协纪：忌嫁娶".into(), None));
        } else if by_zhou_tang_fu {
            veto.push(("协纪：嫁娶周堂值夫不注宜".into(), Some(JIE_ZHOU_TANG_BI)));
        } else {
            veto.push(("协纪：不注宜嫁娶".into(), None));
        }
    }
    // 周堂
    match (o.zhou_tang, zt) {
        (ZhouTangRule::XieJi, Some(z @ ("翁" | "姑"))) if inp.has_weng_gu => {
            veto.push((format!("协纪：嫁娶周堂值{z}而有翁姑"), Some(JIE_ZHOU_TANG_BI)));
        }
        (ZhouTangRule::LeiBian, Some("婦")) => veto.push(("类编：嫁娶周堂值婦，别擇日".into(), None)),
        (ZhouTangRule::LeiBian, Some(z @ ("翁" | "姑" | "夫"))) => {
            // 类编之用：值夫亦可避，故协纪因周堂值夫不注宜者不否决
            veto.retain(|(r, _)| r != "协纪：嫁娶周堂值夫不注宜");
            if z != "夫" && inp.has_weng_gu || z == "夫" {
                notes.push(format!("嫁娶周堂值{z}：{JIE_ZHOU_TANG_BI}"));
            }
        }
        (_, Some(z)) => notes.push(format!("嫁娶周堂值{z}")),
        _ => {}
    }
    if inp.na_xu {
        match nx {
            Some("夫") => veto.push(("纳婿周堂值夫，别擇日".into(), None)),
            Some(z @ ("姑" | "第" | "翁")) => notes.push(format!("纳婿周堂值{z}：{JIE_NA_XU_BI}")),
            _ => {}
        }
    }
    // 行嫁白虎
    let bh_bad = match o.bai_hu {
        BaiHuRule::LeiBian => bh == Some("路"),
        BaiHuRule::AoTou => matches!(bh, Some("門" | "床" | "堂" | "路")),
    };
    let bh_jie = if o.bai_hu == BaiHuRule::LeiBian { JIE_XING_JIA_BAI_HU_LEIBIAN } else { JIE_XING_JIA_BAI_HU_AOTOU };
    if bh_bad {
        if o.leibian_xiong {
            veto.push((format!("行嫁白虎值{}", bh.unwrap()), Some(bh_jie)));
        } else {
            notes.push(format!("行嫁白虎值{}（{bh_jie}）", bh.unwrap()));
        }
    }
    // 女命行嫁月（不可制）
    let bad_yue = match yue {
        NvMingYue::DaLi | NvMingYue::FangMeiShi => false,
        NvMingYue::FangWengGu => inp.has_weng_gu,
        NvMingYue::FangNvFuMu => inp.has_nv_fu_mu,
        NvMingYue::FangFu | NvMingYue::FangNvShen => true,
    };
    if o.nv_ming_yue && bad_yue {
        veto.push((format!("类编：女命{}", yue.name()), None));
    } else if yue != NvMingYue::DaLi {
        notes.push(format!("女命{}（类编；协纪奏准删除）", yue.name()));
    }
    // 类编凶神：白虎入中宫可禳；披麻殺书云「俗有厭𭫲之法，詳見諸公口訣」而法未载，不作可制；余不可制
    const VETO_EXTRA: [&str; 10] = ["天寡", "地寡", "紅沙殺", "天罡勾絞", "河魁勾絞", "往亡", "無翹", "陰錯", "陽錯", "伏斷"];
    let lx_veto: Vec<&(&str, &str)> = lx.iter().filter(|(n, _)| LEIBIAN_XIONG_RI.contains(n) || VETO_EXTRA.contains(n)).collect();
    if o.leibian_xiong {
        for (n, _) in &lx_veto {
            let jie = match *n {
                "大殺白虎入中宮" | "雷霆白虎入中宮" => Some(JIE_BAI_HU_ZHONG_GONG),
                _ => None,
            };
            veto.push((format!("类编凶神：{n}"), jie));
        }
    } else if !lx.is_empty() {
        notes.push(format!("类编所列：{}", lx.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(" ")));
    }
    if lx.iter().any(|(n, _)| *n == "披麻殺") {
        notes.push("披麻殺：类编云「俗有厭𭫲之法，詳見諸公口訣」，法未载".into());
    }
    if lx.iter().any(|(n, _)| *n == "受死") {
        notes.push(format!("受死：{JIE_SHOU_SI}"));
    }
    let bjx = bu_jiang_xieji(d.month_ord, gz);
    let jg = jiang(d.month_ord, gz, o.wuji);
    if bjx || jg == Jiang::BuJiang {
        notes.push(format!("不將（{}）", match (bjx, jg == Jiang::BuJiang) {
            (true, true) => "协纪、类编",
            (true, false) => "协纪",
            _ => "类编",
        }));
    }
    // 三态
    let mut reasons = Vec::new();
    let mut jie = Vec::new();
    for (r, j) in veto {
        match j {
            Some(j) if o.zhi_hua => jie.push((r, j.to_string())),
            Some(j) => reasons.push(format!("{r}（可制：{j}；未用制化）")),
            None => reasons.push(r),
        }
    }
    let verdict = if !reasons.is_empty() {
        Verdict::BuKeYong
    } else if !jie.is_empty() {
        Verdict::ZhiHuaKeYong
    } else {
        Verdict::KeYong
    };
    JiaquDay {
        jdn: d.jdn,
        verdict,
        xieji_yi,
        xieji_ji,
        yi_jie_hun_yin: has(&yb.yi, "結婚姻"),
        yi_na_cai: has(&yb.yi, "納采問名"),
        nv_ming_yue: yue,
        zhou_tang: zt,
        xing_jia_bai_hu: bh,
        na_xu_zhou_tang: nx,
        bu_jiang_xieji: bjx,
        jiang: jg,
        leibian_xiong: lx,
        reasons,
        jie,
        notes,
    }
}

/// 吉时参照：当日黄道时辰（协纪卷七黄道黑道：以日支起时）。
/// 类编只言「女家惟擇吉日吉時出門，男家亦擇吉時入門」而无定法，此以黄道时为参照，出处为推，未核。
pub fn huang_dao_shi(day_gz: u8) -> Vec<(u8, &'static str)> {
    use super::jianchu::{huang_hei, is_huang_dao, HUANG_HEI_SHEN};
    (0..12u8)
        .filter_map(|h| {
            let s = huang_hei(h, day_gz % 12);
            is_huang_dao(s).then(|| (h, HUANG_HEI_SHEN[s as usize]))
        })
        .collect()
}

/// 在 [j0, j1] 内逐日判定（农历实历优先，见 [`super::puzhu::puzhu_days`]）
pub fn choose(
    vsop: &crate::astronomy::ephemeris::Vsop87,
    elp: &crate::astronomy::ephemeris::Elpmpp02Data,
    historical: Option<&crate::calendar::historical::HistoricalCalendar>,
    j0: i64,
    j1: i64,
    inp: &JiaquInput,
    o: &JiaquOptions,
) -> Vec<JiaquDay> {
    super::puzhu::puzhu_days(vsop, elp, historical, j0, j1, super::puzhu::QiWangWangCount::Inclusive)
        .iter()
        .map(|d| judge_day(d, inp, o))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 类编卷七女命表（PDF 第 31 页），每格两月（相隔六月）：行为女命，列为大利、妨媒氏首子、妨翁姑、妨女父母、妨夫主、妨女身
    #[test]
    fn nv_ming_table_leibian() {
        // 农历月数（正 1 … 十二 12）→ 月建序 m-1
        let tab: [(&str, [[u8; 2]; 6]); 6] = [
            ("子午", [[6, 12], [1, 7], [2, 8], [3, 9], [4, 10], [5, 11]]),
            ("丑未", [[5, 11], [4, 10], [3, 9], [2, 8], [1, 7], [6, 12]]),
            ("寅申", [[2, 8], [3, 9], [4, 10], [5, 11], [6, 12], [1, 7]]),
            ("卯酉", [[1, 7], [6, 12], [5, 11], [4, 10], [3, 9], [2, 8]]),
            ("辰戌", [[4, 10], [5, 11], [6, 12], [1, 7], [2, 8], [3, 9]]),
            ("巳亥", [[3, 9], [2, 8], [1, 7], [6, 12], [5, 11], [4, 10]]),
        ];
        let kinds = [NvMingYue::DaLi, NvMingYue::FangMeiShi, NvMingYue::FangWengGu, NvMingYue::FangNvFuMu, NvMingYue::FangFu, NvMingYue::FangNvShen];
        for (pair, row) in tab {
            for zc in pair.chars() {
                let z = "子丑寅卯辰巳午未申酉戌亥".chars().position(|c| c == zc).unwrap() as u8;
                for (k, months) in row.iter().enumerate() {
                    for &mo in months {
                        assert_eq!(nv_ming_yue(z, mo - 1), kinds[k], "女命{zc} {mo}月");
                    }
                }
            }
        }
    }

    /// 类编「論正月陰陽不將日」与陽將、陰將、陰陽俱將诸例（PDF 第 32—33 页）
    #[test]
    fn jiang_zheng_yue_leibian() {
        for g in ["丁卯", "辛卯", "丙寅", "丙子", "己卯", "戊子", "庚寅", "庚子", "辛亥", "辛丑"] {
            assert_eq!(jiang(0, gz_index(g), WuJiJi::LeiBian), Jiang::BuJiang, "{g}");
        }
        for g in ["辛酉", "辛未", "辛巳", "庚申", "庚午", "戊午", "丙午", "己酉", "己未", "己巳", "丁酉", "丁未", "丁巳"] {
            assert_eq!(jiang(0, gz_index(g), WuJiJi::LeiBian), Jiang::YangJiang, "{g}");
        }
        for g in ["壬子", "壬寅", "癸卯", "癸亥", "甲子", "甲寅", "乙卯", "乙丑", "乙亥"] {
            assert_eq!(jiang(0, gz_index(g), WuJiJi::LeiBian), Jiang::YinJiang, "{g}");
        }
        for g in ["壬午", "壬申", "癸巳", "癸未", "癸酉", "甲午", "甲申", "乙未", "乙巳", "乙酉"] {
            assert_eq!(jiang(0, gz_index(g), WuJiJi::LeiBian), Jiang::JuJiang, "{g}");
        }
        // 「論隂陽不將日」：二月厭前取乙丙丁庚為日干，厭後取戌亥子丑寅為日支
        for g in ["乙亥", "丙子", "丁丑", "庚寅", "丙戌", "庚子"] {
            assert_eq!(jiang(1, gz_index(g), WuJiJi::LeiBian), Jiang::BuJiang, "二月 {g}");
        }
    }

    /// 协纪卷四《天寶厯》戊己寄艮坤之法，所推不将日与协纪所引《歷例》逐月表（月表「不將」）比对
    #[test]
    fn jiang_xieji_vs_liliu_table() {
        let mut diff = Vec::new();
        for m in 0..12u8 {
            for gz in 0..60u8 {
                let rule = jiang(m, gz, WuJiJi::XieJi) == Jiang::BuJiang;
                if rule != bu_jiang_xieji(m, gz) {
                    diff.push((m, gz, rule));
                }
            }
        }
        // 规则与歷例表全合（若有出入，见 doc/folk/jiaqu.md 记录）
        assert!(diff.is_empty(), "不合 {} 处：{:?}", diff.len(), diff);
        // 表非空：正月《歷例》十三日（辛亥辛丑辛夘庚子庚寅己亥己丑己夘丁亥丁丑丁夘丙子丙寅）
        assert_eq!((0..60u8).filter(|&g| bu_jiang_xieji(0, g)).count(), 13);
        // 两家之异：类编戊己皆寄厌前，正月戊子为不将；协纪春月戊寄艮（厌后），戊子非不将
        assert_eq!(jiang(0, gz_index("戊子"), WuJiJi::LeiBian), Jiang::BuJiang);
        assert_ne!(jiang(0, gz_index("戊子"), WuJiJi::XieJi), Jiang::BuJiang);
    }

    #[test]
    fn zhou_tang_variants() {
        // 行嫁白虎：大月初一竈、初七路；小月初一厨、初二路
        assert_eq!(xing_jia_bai_hu(30, 1), Some("竈"));
        assert_eq!(xing_jia_bai_hu(30, 7), Some("路"));
        assert_eq!(xing_jia_bai_hu(29, 2), Some("路"));
        assert_eq!(xing_jia_bai_hu(29, 9), Some("厨"));
        // 纳婿周堂：大月初一夫，小月初一户、初八夫
        assert_eq!(na_xu_zhou_tang(30, 1), Some("夫"));
        assert_eq!(na_xu_zhou_tang(29, 1), Some("戸"));
        assert_eq!(na_xu_zhou_tang(29, 8), Some("夫"));
    }

    #[test]
    fn leibian_xiong_examples() {
        // 披麻殺：正五九月子日（「論披麻殺日」）
        let pm = |m: u8, g: &str| leibian_xiong(m, 0, gz_index(g), 0).iter().any(|(n, _)| *n == "披麻殺");
        assert!(pm(0, "甲子") && pm(4, "丙子") && pm(1, "癸酉") && pm(3, "丁卯"));
        assert!(!pm(0, "丁卯"));
        // 月厭正戌、厭對正辰
        let has = |m: u8, g: &str, n: &str| leibian_xiong(m, 0, gz_index(g), 0).iter().any(|(x, _)| *x == n);
        assert!(has(0, "甲戌", "月厭") && has(0, "丙辰", "厭對"));
        assert!(has(0, "庚戌", "陰錯") && has(0, "甲寅", "陽錯"));
        assert!(has(5, "戊申", "人民離") && has(5, "乙亥", "亥日"));
    }

    /// 制化：以 2026 年逐日实算检验三态之间的关系
    #[test]
    fn zhi_hua_three_states() {
        use crate::astronomy::ephemeris::{load_all, load_earth_vsop87_from_repo, Elpmpp02Correction};
        use crate::calendar::gregorian::Gregorian;
        use crate::math::real::RealOps;
        let vsop = load_earth_vsop87_from_repo().unwrap();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
        let j0 = (Gregorian::to_julian_day(2026, 1, 1).as_f64() + 0.5).floor() as i64;
        let inp = JiaquInput { nv_year_gz: 41, has_weng_gu: true, has_nv_fu_mu: true, na_xu: false };
        let base = JiaquOptions { leibian_xiong: true, bai_hu: BaiHuRule::AoTou, ..JiaquOptions::default() };
        let off = choose(&vsop, &elp, None, j0, j0 + 364, &inp, &base);
        let on = choose(&vsop, &elp, None, j0, j0 + 364, &inp, &JiaquOptions { zhi_hua: true, ..base });
        let mut n_zh = 0;
        for (a, b) in off.iter().zip(&on) {
            assert_eq!(a.jdn, b.jdn);
            // 未用制化：无「经制化」态，jie 恒空
            assert_ne!(a.verdict, Verdict::ZhiHuaKeYong);
            assert!(a.jie.is_empty());
            match b.verdict {
                Verdict::ZhiHuaKeYong => {
                    n_zh += 1;
                    assert!(b.reasons.is_empty() && !b.jie.is_empty());
                    // 不用制化时同日不可用，且由皆注「可制」
                    assert_eq!(a.verdict, Verdict::BuKeYong);
                    assert!(a.reasons.iter().all(|r| r.contains("可制")), "{:?}", a.reasons);
                }
                Verdict::KeYong => assert_eq!(a.verdict, Verdict::KeYong),
                Verdict::BuKeYong => assert_eq!(a.verdict, Verdict::BuKeYong),
            }
        }
        assert!(n_zh > 0, "一年中应有经制化可用之日");
        // 协纪：周堂值翁姑而有翁姑在者否决，可避而制
        assert!(off.iter().any(|d| d.reasons.iter().any(|r| r.contains("值翁而有翁姑") || r.contains("值姑而有翁姑"))));
        // 无翁姑则不因此否决
        let free = choose(&vsop, &elp, None, j0, j0 + 364, &JiaquInput { has_weng_gu: false, ..inp }, &JiaquOptions::default());
        assert!(free.iter().all(|d| !d.reasons.iter().any(|r| r.contains("有翁姑"))));
    }

    #[test]
    fn huang_dao_hours() {
        // 子日：青龍起申（2×0−4 ≡ 8），黄道申酉子丑卯午
        let h: Vec<u8> = huang_dao_shi(0).iter().map(|(z, _)| *z).collect();
        assert_eq!(h, vec![0, 1, 3, 6, 8, 9]);
    }
}
