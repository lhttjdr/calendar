//! 婚嫁择吉：按新人之命择嫁娶年月日时。
//!
//! 两系并列（见 doc/folk/jiaqu.md）：
//! - **协纪**（缺省，清官定）：嫁娶宜忌取月表加卷十铺注与嫁娶周堂（[`super::puzhu::yiji_for_day`]），
//!   不将取卷四所引《歷例》逐月表；女命行嫁大利月诸说，协纪卷三十六辨訛斥为「惑世誣民之尤甚者」，「今已奏准刪除」。
//! - **类编**（题元宋魯珍《類編曆法通書大全》卷七「結婚嫁娶」，北大藏本，`02094406.cn` PDF 第 30—65 页）：
//!   女命大利月与妨亲属月（出呂才《大義婚書》）、阴阳不将起例、嫁娶凶神表、白虎入中宫、纳婿周堂、行嫁白虎。
//!
//! 只录规则，不录应验之说。

pub use super::jiaqu_catalog::{hun_shen_catalog, rule_info, RuleInfo, View, RULE_CATALOG};
use super::jiaqu_tongshu::{jiu_ben_yi_shen, huang_hei_dao, sha_gong_zhi_xing_ren_zhuan, tongshu_hits, TongshuInput, TONGSHU_SHEN};
use super::puzhu::{apply_skipping, puzhu_names, yiji_for_day, yuebiao_base_for_day, zhou_tang, PuzhuDay};
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
    // 雷霆白虎入中宮日：按月建之干。类编（第 39 页）、鳌头卷三（第 4 册 PDF 第 14 页）皆作「甲己月」「乙庚月」「丙辛戊癸月」「丁壬月」，
    // 而同表上朔作「甲年丁亥乙年己巳……」，一表之内年、月分书，故「月」是月干而非年干之讹（两书诸日全同）。
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
/// 雷霆白虎入中宫日、人民離日。赤口按农历月日起（[`xiao_liu_ren`]），不在此表，另以 [`ChiKouRule`] 取舍。
/// 受死书云「今人多不忌」，月忌日书云「不忌嫁娶」，只作附注。
pub const LEIBIAN_XIONG_RI: [&str; 10] = ["月厭", "厭對", "歸忌", "天賊", "月破", "亥日", "大殺白虎入中宮", "雷霆白虎入中宮", "人民離", "披麻殺"];

// ───────────────────────── 赤口（小六壬） ─────────────────────────

/// 小六壬六位（类编卷十五「六壬赤口日」：大安百無不利、留連淹延久遠、速喜利濟欣順、赤口是非口舌、小吉無凶有吉、空亡萬事無成）
pub const XIAO_LIU_REN: [&str; 6] = ["大安", "留連", "速喜", "赤口", "小吉", "空亡"];

/// 农历某月某日所值小六壬之位。
///
/// 类编卷十五「六壬赤口日」（`02094410.cn` PDF 第 90—91 页）：「正七初一起留連，二八却從速喜先，三九月初逢赤口，
/// 四十之月小吉傳，五十一月空亡上，六十二月大安前。假如正月初一日起留連，初二日速喜，初三日赤口，初四日小吉，
/// 初五日空亡，初六日大安，逐日順數去，其餘倣此」。即位序 = (月 + 日 − 1) mod 6（大安 0）。
/// 《田家五行》卷下「赤口日」：「以月建上起初一日，順數加辰戌二位是」，所推全同（正月寅起初一，初三至辰、初九至戌……）。
/// 闰月从本月之数：卷十五、《田家五行》皆未言；类编卷二「六輪兀」（同为小六壬之轮，见 [`liu_lun_wu`]）明言「逢閏月亦於本月上起日」，
/// 他说（如闰月从下月）未见原典。
pub fn xiao_liu_ren(lunar_month: u8, lunar_day: u8) -> &'static str {
    XIAO_LIU_REN[(lunar_month as usize + lunar_day as usize + 5) % 6]
}

/// 赤口之用
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChiKouRule {
    /// 不用（缺省）：协纪不立赤口；卷三十六「上兀下兀」谓兀日「不過小六壬之留連赤口日耳……毫無深義」
    BuYong,
    /// 类编：「論嫁娶凶日」忌赤口，凡赤口日否决
    LeiBian,
    /// 儲泳《祛疑説》「赤口煞」：「用之亦活法，不可以此小害遽廢良日，如赤口值寅巳酉戌則不可用，餘皆無害」。
    /// 「值寅巳酉戌」读作赤口日之日支（推读，未核）
    QuYi,
    /// 协纪卷三十六「上兀下兀」所述起法：「陽年正月初一日起小吉，隂年正月初一日起留連，按日順數」，兀日即小六壬之
    /// 留連（上兀）、赤口（下兀）。即类编卷二「六輪兀」（沈括本，[`liu_lun_wu`]）：值下兀（赤口）之日否决，上兀只作附注。
    /// 阴年与 [`ChiKouRule::LeiBian`] 逐日全同，阳年赤口移后三位
    XieJiWu,
}

/// 类编卷二「六輪兀」六位（后天八卦去震兑，顺行）：乾、坎、艮三白日、離九紫日吉，巽四緑上兀日凶，坤二黑下兀日凶
pub const LIU_LUN: [&str; 6] = ["乾", "坎", "艮", "巽", "離", "坤"];

/// 类编卷二上官门「六輪兀」（`02094404.cn` PDF 第 27—29 页，起例与陽年、陰年上下兀日两表）：
/// 「右甲丙戊庚壬五陽年正月起乾，乙丁己辛癸五陰年正月起巽，並順行一月一位到所用之月，月上起初一亦順行，
/// 如遇閏月仍就本月上起初一數到所用之日。艮坎乾為三白日，離為九紫日吉，巽為四緑上兀日凶，坤為二黑下兀日凶」；
/// 又「年上起月，月上起日，大抵只取本年月分起初一，卻不論節氣，逢閏月亦於本月上起日」；「右六輪兀用沈學士存中本」。
/// `yang_nian`：农历年之干为阳（甲丙戊庚壬）；`lunar_month` 闰月取本月之数。返回六位之一。
///
/// 协纪卷三十六所述（阳年起小吉、阴年起留連）与此一一对应：乾↔小吉、坎↔空亡、艮↔大安、巽↔留連、離↔速喜、坤↔赤口。
pub fn liu_lun_wu(yang_nian: bool, lunar_month: u8, lunar_day: u8) -> &'static str {
    let start = if yang_nian { 0 } else { 3 };
    LIU_LUN[(start + lunar_month as usize - 1 + lunar_day as usize - 1) % 6]
}

/// 协纪卷三十六所述兀日起法下的小六壬之位：阳年正月初一起小吉、阴年起留連，月进一位，按日顺数（闰月从本月，据类编卷二）。
/// 值「留連」为上兀、值「赤口」为下兀
pub fn xiao_liu_ren_wu(yang_nian: bool, lunar_month: u8, lunar_day: u8) -> &'static str {
    let k = LIU_LUN.iter().position(|x| *x == liu_lun_wu(yang_nian, lunar_month, lunar_day)).unwrap();
    // 乾 0 ↔ 小吉 4
    XIAO_LIU_REN[(k + 4) % 6]
}

/// 农历年干支序（正月初一为界）。`PuzhuDay::year_gz` 以立春为界，此按农历月与节月之差校正：
/// 农历十一、十二月而节月已入寅卯（立春后、春节前）属上一农历年；农历正、二月而节月尚在子丑（春节后、立春前）属下一农历年
pub fn lunar_year_gz(year_gz_lichun: u8, lunar_month: u8, month_ord: u8) -> u8 {
    if lunar_month >= 11 && month_ord <= 1 {
        (year_gz_lichun + 59) % 60
    } else if lunar_month <= 2 && month_ord >= 10 {
        (year_gz_lichun + 1) % 60
    } else {
        year_gz_lichun % 60
    }
}

/// 鳌头卷三嫁娶门（第 4 册 PDF 第 14 页）「橫天朱雀」：「每月初一日是，若值初一行嫁主再娶」（农历初一，类编卷七不载）
pub fn heng_tian_zhu_que(lunar_day: u8) -> bool {
    lunar_day == 1
}

// ───────────────────────── 类编逐月吉日表 ─────────────────────────

/// 类编卷七逐月吉日表录文（`02094406.cn` PDF 第 42—48 页，书影目读）：求婚、下定、嫁娶、嫁娶結親備急用
const JIRI_CSV: &str = include_str!("../../../../data/folk/leibian_jiaqu_jiri.csv");

/// 类编卷七之吉日表
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiriBiao {
    /// 「逐月求婚吉日」（第 42—43 页）；表末「右吉日不犯魁罡約[勾]絞月破受死九空九土鬼人隔陰陽錯日」
    QiuHun,
    /// 「逐月下定吉日」（第 43—45 页）
    XiaDing,
    /// 「逐月嫁娶吉日」（第 45—46 页），注上吉、係不將、合季分、人民合等；表末「右吉日不犯魁罡月厭厭對月破天賊受死紅沙吟神人民離四忌五窮陰陽錯往亡亥日」
    JiaQu,
    /// 「嫁娶結親吉日備急用」（第 46—48 页），注不將、三合、五合、六合、月德合等；表末「依大全曆取其三合五合月德合不將六合日為吉，以備卒急用」
    BeiJi,
}

impl JiriBiao {
    pub const ALL: [JiriBiao; 4] = [JiriBiao::QiuHun, JiriBiao::XiaDing, JiriBiao::JiaQu, JiriBiao::BeiJi];
    pub fn name(self) -> &'static str {
        match self {
            JiriBiao::QiuHun => "求婚",
            JiriBiao::XiaDing => "下定",
            JiriBiao::JiaQu => "嫁娶",
            JiriBiao::BeiJi => "嫁娶備急",
        }
    }
}

/// 吉日表一条：(表, 月 1—12, 干支序, 注, 页, 校)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JiriEntry {
    pub biao: JiriBiao,
    pub month: u8,
    pub gz: u8,
    pub zhu: &'static str,
    pub page: u8,
    pub jiao: &'static str,
}

/// 吉日表全部条目（同月重出者照录）
pub fn jiri_entries() -> &'static [JiriEntry] {
    static T: std::sync::OnceLock<Vec<JiriEntry>> = std::sync::OnceLock::new();
    T.get_or_init(|| {
        JIRI_CSV
            .lines()
            .skip(1)
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                let f: Vec<&'static str> = l.split(',').collect();
                let biao = *JiriBiao::ALL.iter().find(|b| b.name() == f[0]).expect("表名");
                JiriEntry { biao, month: f[1].parse().unwrap(), gz: gz_index(f[3]), zhu: f[4], page: f[5].parse().unwrap(), jiao: f[6] }
            })
            .collect()
    })
}

/// 某月（月建序，寅月 0）某干支见于何表：(表, 注)。表以月为目，书不言节气抑月分；
/// 其注不將、披麻皆以月建论，故按节气月查（未核）
pub fn leibian_jiri(month_ord: u8, day_gz: u8) -> Vec<(JiriBiao, &'static str)> {
    let mut v: Vec<(JiriBiao, &'static str)> = Vec::new();
    for e in jiri_entries() {
        if e.month == month_ord % 12 + 1 && e.gz == day_gz && !v.iter().any(|(b, _)| *b == e.biao) {
            v.push((e.biao, e.zhu));
        }
    }
    v
}

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
    /// 不看之神（位掩码，位义见 [`super::yiji_derive::HUN_SHEN`]）：略去后按协纪推法重推婚事宜忌。缺省 0 即月表
    pub ignore_shen: u64,
    /// 用女命行嫁月（类编；协纪奏准删除）——缺省 false：只列出不据以否决
    pub nv_ming_yue: bool,
    pub zhou_tang: ZhouTangRule,
    /// 用类编嫁娶凶神表否决——缺省 false：只列出。为 true 而 `leibian_mask` 为 0 时用 [`LEIBIAN_DEFAULT_VETO`] 诸神
    pub leibian_xiong: bool,
    /// 类编凶神逐神取舍（位义见 [`LEIBIAN_SHEN`]，置位者否决）；非 0 时优先于 `leibian_xiong`
    pub leibian_mask: u64,
    /// 楊公忌（通书；协纪卷三十六辨讹斥之）否决——缺省 false：只作附注
    pub yang_gong: bool,
    /// 紅沙（协纪卷三十六引《轉神厯》及「又一本」；协纪不用）——缺省不用：只作附注
    pub hong_sha: HongShaRule,
    /// 天狗頭、尾、口、腹（通书；协纪卷三十六「天狗」條斥之）否决——缺省 false：只作附注
    pub tian_gou: bool,
    /// 章光（《堪輿經》月厭前一辰；协纪卷三十六斥之）否决——缺省 false：只作附注
    pub zhang_guang: bool,
    /// 行嫁白虎须禳之位（启用类编凶神时据以否决）
    pub bai_hu: BaiHuRule,
    /// 用制化：可制之否决移入 `jie`，判「经制化可用」——缺省 false。
    /// 理由：协纪于嫁娶凶神之解只取诸德（卷十宜忌等第表「遇德从宜」，已含于月表），不录禳、避之法；
    /// 《董公選擇日要覽》斥「或書一符、或書一字以為鎮魔，欺惑世人」，于白虎入中宫谓「縱有……天德月德可以化解……莫若不用」
    pub zhi_hua: bool,
    /// 阴阳不将之戊己寄法（仅作附注；协纪逐月表另列）
    pub wuji: WuJiJi,
    /// 赤口（小六壬）之用——缺省不用，只作附注
    pub chi_kou: ChiKouRule,
    /// 用鳌头「橫天朱雀」（初一）否决——缺省 false：只作附注
    pub heng_tian_zhu_que: bool,
    /// 协纪铺注婚事诸日逐项「不看」（位义见 [`super::puzhu::PUZHU_HUN`]）：上朔、四離、四絕、晦日、二分二至、月忌日、氣往亡、無祿、
    /// 周堂值夫婦。缺省 0 即时宪书体例
    pub ignore_puzhu: u16,
    /// 协纪改删之旧说与类编、鳌头、董公诸神逐神否决（位义见 [`super::jiaqu_tongshu::TONGSHU_SHEN`]）——缺省 0：只作附注
    pub tongshu_mask: u64,
    /// 董公逐日婚姻宜忌之用——缺省只作附注
    pub dong_gong: DongGongHun,
    /// 鳌头卷三婚姻四表之用——缺省只作附注
    pub aotou_jiri: AoTouJiRiRule,
}

/// 董公（《董公選擇日要覽》逐月 × 建除）婚姻宜忌之用。所取事目：婚姻、納采，及泛言之「諸事」
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DongGongHun {
    /// 只作附注（缺省）：董公与协纪神煞体系、吉凶相抵之法皆异，作并列之说（见 `doc/folk/tongshu_yiji.md` 第九节）
    BuYong,
    /// 董公忌婚姻（或諸事忌）之日否决
    BiJi,
    /// 须董公宜婚姻（或諸事宜）方可用，并避其忌
    XuYi,
}

/// 鳌头卷三婚姻四表（求婚、下定結婚、嫁娶、嫁娶結婚備急用，`data/folk/aotou_jiaqu_days.csv`）之用
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AoTouJiRiRule {
    /// 只作附注（缺省）：鳌头只出吉日表，表中未收之日不等于忌
    BuYong,
    /// 须见于鳌头嫁娶表或嫁娶結婚備急表（正选或「外」日）
    XuJiaQu,
}

impl Default for JiaquOptions {
    fn default() -> Self {
        Self {
            xieji_yiji: XieJiYiJi::XuYi,
            ignore_shen: 0,
            nv_ming_yue: false,
            zhou_tang: ZhouTangRule::XieJi,
            leibian_xiong: false,
            leibian_mask: 0,
            yang_gong: false,
            hong_sha: HongShaRule::BuYong,
            tian_gou: false,
            zhang_guang: false,
            bai_hu: BaiHuRule::LeiBian,
            zhi_hua: false,
            wuji: WuJiJi::LeiBian,
            chi_kou: ChiKouRule::BuYong,
            heng_tian_zhu_que: false,
            ignore_puzhu: 0,
            tongshu_mask: 0,
            dong_gong: DongGongHun::BuYong,
            aotou_jiri: AoTouJiRiRule::BuYong,
        }
    }
}

/// 类编卷七嫁娶凶神（与行嫁白虎）之名，下标即 `leibian_mask` 位
pub const LEIBIAN_SHEN: [&str; 26] = [
    "歸忌", "月厭", "厭對", "天賊", "月破", "受死", "天寡", "地寡", "紅沙殺", "披麻殺", "天罡勾絞", "河魁勾絞", "吟神勾絞",
    "天雄", "地雌", "往亡", "無翹", "荒蕪", "陰錯", "陽錯", "伏斷", "人民離", "亥日", "大殺白虎入中宮", "雷霆白虎入中宮", "行嫁白虎",
];
/// 「用类编凶神」缺省所否决者：「論嫁娶凶日」所列（[`LEIBIAN_XIONG_RI`]）与天寡地寡、紅沙、勾絞、往亡、無翹、陰陽錯、伏斷、行嫁白虎。
/// 受死（书云「今人多不忌」）、吟神（曆忌）、天雄地雌、荒蕪只列
pub const LEIBIAN_DEFAULT_VETO: [&str; 21] = [
    "月厭", "厭對", "歸忌", "天賊", "月破", "亥日", "大殺白虎入中宮", "雷霆白虎入中宮", "人民離", "披麻殺",
    "天寡", "地寡", "紅沙殺", "天罡勾絞", "河魁勾絞", "往亡", "無翹", "陰錯", "陽錯", "伏斷", "行嫁白虎",
];

/// 楊公忌（协纪卷三十六）：「元旦起角宿依二十八宿次序順數值室宿之日……每月遞退二日，故正月十三、二月十一以至七月初一、
/// 二十九而終于十二月十九，凡十三日」——通书按农历月日。闰月：书未言，从本月之数（未核）
pub fn yang_gong_ji(lunar_month: u8, lunar_day: u8) -> bool {
    let m = lunar_month as i32;
    let d = lunar_day as i32;
    (m <= 7 && d == 15 - 2 * m) || (m >= 7 && d == 43 - 2 * m)
}

/// 天狗诸位（协纪卷三十六「天狗」條所述俗说）：按季取「正位」——
/// 天狗頭＝月厭正位（春酉夏午秋卯冬子，「犯之者小姑無子」）、天狗尾＝厭對正位（春卯夏子秋酉冬午，「妨夫主」）、
/// 天狗口＝章光正位（春申夏巳秋寅冬亥，「忌行嫁」）、天狗腹＝天喜正位（春午夏卯秋子冬酉，「主當年無子」）。
/// 同條天狗背（紅鸞正位）、足、後足主有子，不是忌，不列。季以月建（节）分：寅卯辰为春
pub fn tian_gou(month_ord: u8, day_zhi: u8) -> Option<(&'static str, &'static str)> {
    let season = (month_ord % 12 / 3) as usize; // 0 春 1 夏 2 秋 3 冬
    const T: [(&str, &str, [u8; 4]); 4] = [
        ("天狗頭", "小姑無子", [9, 6, 3, 0]),
        ("天狗尾", "妨夫主", [3, 0, 9, 6]),
        ("天狗口", "忌行嫁", [8, 5, 2, 11]),
        ("天狗腹", "當年無子", [6, 3, 0, 9]),
    ];
    T.iter().find(|(_, _, z)| z[season] == day_zhi % 12).map(|(n, w, _)| (*n, *w))
}

/// 天狗之吉位（同條）：天狗背＝紅鸞正位（春子夏酉秋午冬卯，「主三年有子」）、天狗足＝六合正位（春戌夏未秋辰冬丑，「主六年生子」）、
/// 天狗後足＝月害正位（春辰夏丑秋戌冬未，「主九年生子」）
pub fn tian_gou_ji(month_ord: u8, day_zhi: u8) -> Option<(&'static str, &'static str)> {
    let season = (month_ord % 12 / 3) as usize;
    const T: [(&str, &str, [u8; 4]); 3] = [
        ("天狗背", "三年有子", [0, 9, 6, 3]),
        ("天狗足", "六年生子", [10, 7, 4, 1]),
        ("天狗後足", "九年生子", [4, 1, 10, 7]),
    ];
    T.iter().find(|(_, _, z)| z[season] == day_zhi % 12).map(|(n, w, _)| (*n, *w))
}

/// 五合五離之俗名（协纪卷三十六「五合五離」條所述俗术）：寅卯日为合、申酉日为離，干分日月（甲乙）、陰陽（丙丁）、
/// 人民（戊己）、金石（庚辛）、江河（壬癸）。世俗以戊寅己卯（人民合）为大吉、戊申己酉（人民離）为大凶
pub fn wu_he_li_su(day_gz: u8) -> Option<(String, bool)> {
    let z = day_gz % 12;
    let he = match z {
        2 | 3 => true,
        8 | 9 => false,
        _ => return None,
    };
    // 寅卯、申酉之干须为阳寅阴卯：甲寅乙卯……（干支同阴阳自然成立）
    let name = ["日月", "陰陽", "人民", "金石", "江河"][(day_gz % 10 / 2) as usize];
    Some((format!("{name}{}", if he { "合" } else { "離" }), he))
}

/// 章光（《堪輿經》「月厭前一辰為章光，後一辰為無翹」）：正月酉，逐月逆行
pub fn zhang_guang(month_ord: u8, day_zhi: u8) -> bool {
    (21 - month_ord as i32 % 12) % 12 == day_zhi as i32 % 12
}

/// 紅沙之说（协纪卷三十六）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HongShaRule {
    /// 不用（协纪：「通書萬年書皆不用，故今仍之，不以轉神為據」）
    BuYong,
    /// 《轉神厯》：孟月酉、仲月巳、季月丑，「其日忌嫁娶」
    ZhuanShen,
    /// 「又一本孟月取巳、仲月取酉」，季月丑
    YouYiBen,
}

/// 某月（月建序，寅 0）某日支是否紅沙
pub fn hong_sha(rule: HongShaRule, month_ord: u8, day_zhi: u8) -> bool {
    let k = month_ord % 3; // 0 孟 1 仲 2 季
    let z = match (rule, k) {
        (HongShaRule::BuYong, _) => return false,
        (_, 2) => 1, // 丑
        (HongShaRule::ZhuanShen, 0) | (HongShaRule::YouYiBen, 1) => 9, // 酉
        _ => 5, // 巳
    };
    day_zhi % 12 == z
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

/// 协纪月表嫁娶宜忌之依据：月表由卷十一「嫁娶」宜神、忌神，经卷十相併相抵与宜忌等第表推出，再加卷十铺注
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct XieJiBasis {
    /// 月表（铺注前）于嫁娶：「宜」「忌」「两不注」「諸事不宜」
    pub yuebiao: &'static str,
    /// 加铺注后（即时宪书所注）
    pub final_: &'static str,
    /// 当日所值之嫁娶宜神（卷十一：天德、月德、天德合、月德合、天赦、天願、三合、天喜、六合、不將）
    pub yi_shen: Vec<String>,
    /// 当日所值之嫁娶忌神（卷十一：月破、平日、收日、閉日、劫煞……八專、亥日）
    pub ji_shen: Vec<String>,
    /// 卷十宜忌等第表之用（月表 CSV「等第」：主凶神最重之等第→從宜／相抵／從忌／皆忌）
    pub deng_di: String,
    /// 铺注所值（铺注改动嫁娶之注时列出）
    pub puzhu: Vec<String>,
}

fn jiaqu_status(d: &super::yiji::YueBiaoDay) -> &'static str {
    match &d.yi {
        None => "諸事不宜",
        Some(v) if v.iter().any(|x| x == "嫁娶") => "宜",
        _ if d.ji.iter().any(|x| x == "嫁娶") => "忌",
        _ => "两不注",
    }
}

/// 一日铺注前之月表；略去 `ignore` 诸神者婚事按协纪推法重推（交节之日新旧两月各推后取其交）
fn base_ignoring(d: &PuzhuDay, ignore: u64) -> super::yiji::YueBiaoDay {
    use super::puzhu::{jiao_jie_base, JIAO_JIE_EARLY_HOUR};
    use super::yiji_derive::{ignore_names, yuebiao_day_ignoring};
    if ignore == 0 {
        return yuebiao_base_for_day(d);
    }
    let names = ignore_names(ignore);
    let now = yuebiao_day_ignoring(d.month_ord, d.input.day_gz, &names);
    if d.jiao_jie && d.jiao_jie_hour.is_some_and(|h| h >= JIAO_JIE_EARLY_HOUR) {
        jiao_jie_base(&now, &yuebiao_day_ignoring((d.month_ord + 11) % 12, d.input.day_gz, &names))
    } else {
        now
    }
}

/// 一日协纪嫁娶宜忌之依据
pub fn xieji_basis(d: &PuzhuDay) -> XieJiBasis {
    xieji_basis_ignoring(d, 0, 0)
}

/// 同 [`xieji_basis`]，略去 `ignore` 诸神（位义见 [`super::yiji_derive::HUN_SHEN`]）、不加 `skip` 诸铺注（位义见 [`super::puzhu::PUZHU_HUN`]）
pub fn xieji_basis_ignoring(d: &PuzhuDay, ignore: u64, skip: u16) -> XieJiBasis {
    const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
    const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";
    let base = base_ignoring(d, ignore);
    let fin = apply_skipping(&base, &d.input, d.shuo_gz, skip);
    let gz = d.input.day_gz as usize;
    let mut names: Vec<String> = base.ji_shen.iter().chain(base.xiong_shen.iter()).cloned().collect();
    names.push(format!("{}日", base.jian_chu));
    names.push(format!("{}日", GAN.chars().nth(gz % 10).unwrap()));
    names.push(format!("{}日", ZHI.chars().nth(gz % 12).unwrap()));
    let (yi_s, ji_s) = super::yiji::shi_shen("嫁娶");
    let ign = super::yiji_derive::ignore_names(ignore);
    names.retain(|n| !ign.contains(n));
    let pick = |list: &[&str]| -> Vec<String> { list.iter().filter(|n| names.iter().any(|x| x == *n)).map(|n| n.to_string()).collect() };
    let (a, b) = (jiaqu_status(&base), jiaqu_status(&fin));
    XieJiBasis {
        yuebiao: a,
        final_: b,
        yi_shen: pick(&yi_s),
        ji_shen: pick(&ji_s),
        deng_di: base.deng_di.clone(),
        puzhu: if a != b {
            // 不看之铺注不列（周堂值夫、值婦同一位）
            let off = |n: &str| {
                let k = match n {
                    "月忌" => "月忌日",
                    "周堂值夫" | "周堂值婦" => "周堂值夫婦",
                    "春分" | "秋分" | "夏至" | "冬至" => "二分二至",
                    x => x,
                };
                skip & super::puzhu::puzhu_hun_bit(k) != 0
            };
            puzhu_names(&d.input, d.shuo_gz).into_iter().filter(|n| !off(n)).collect()
        } else {
            Vec::new()
        },
    }
}

/// 一日之判
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JiaquDay {
    /// 协纪月表嫁娶宜忌之依据
    pub xieji_basis: XieJiBasis,
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
    /// 小六壬之位（类编卷十五；值「赤口」者为赤口日）
    pub xiao_liu_ren: &'static str,
    /// 协纪卷三十六所述兀日（类编卷二六輪兀）：值巽为「上兀」（留連）、值坤为「下兀」（赤口），余为 None
    pub wu_ri: Option<&'static str>,
    /// 类编卷七逐月吉日表所列：(表, 注)
    pub leibian_jiri: Vec<(JiriBiao, &'static str)>,
    /// 所值协纪改删之旧说与类编、鳌头、董公诸神（[`TONGSHU_SHEN`] 名）
    pub tongshu: Vec<&'static str>,
    /// 董公逐日婚姻宜忌：「宜（等第）：所据原句」等
    pub dong_gong: Vec<String>,
    /// 鳌头卷三婚姻四表所列：「表（正／外，注）」
    pub aotou_jiri: Vec<String>,
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
    let yb = if o.ignore_shen == 0 && o.ignore_puzhu == 0 {
        yiji_for_day(d)
    } else {
        apply_skipping(&base_ignoring(d, o.ignore_shen), &d.input, d.shuo_gz, o.ignore_puzhu)
    };
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
        let base_yi = base_ignoring(d, o.ignore_shen).yi.as_ref().is_some_and(|v| v.iter().any(|x| x == "嫁娶"));
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
    // 类编逐神取舍：位掩码优先，否则「用类编凶神」取缺省诸神
    let lb_veto = |n: &str| -> bool {
        if o.leibian_mask != 0 {
            LEIBIAN_SHEN.iter().position(|x| *x == n).is_some_and(|i| o.leibian_mask >> i & 1 == 1)
        } else {
            o.leibian_xiong && LEIBIAN_DEFAULT_VETO.contains(&n)
        }
    };
    if bh_bad {
        if lb_veto("行嫁白虎") {
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
    let (lx_veto, lx_note): (Vec<&(&str, &str)>, Vec<&(&str, &str)>) = lx.iter().partition(|(n, _)| lb_veto(n));
    for (n, _) in &lx_veto {
        let jie = match *n {
            "大殺白虎入中宮" | "雷霆白虎入中宮" => Some(JIE_BAI_HU_ZHONG_GONG),
            _ => None,
        };
        veto.push((format!("类编凶神：{n}"), jie));
    }
    if !lx_note.is_empty() {
        notes.push(format!("类编所列：{}", lx_note.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(" ")));
    }
    // 楊公忌（通书；协纪斥之）
    if yang_gong_ji(d.lunar.month, d.lunar.day) {
        if o.yang_gong {
            veto.push(("楊公忌（通书）".into(), None));
        } else {
            notes.push("楊公忌（通书载；协纪卷三十六斥为「邪説誣民」）".into());
        }
    }
    // 天狗诸位、章光（通书、堪輿經；协纪卷三十六斥之）
    if let Some((n, w)) = tian_gou(d.month_ord, gz % 12) {
        if o.tian_gou {
            veto.push((format!("{n}（通书：{w}）"), None));
        } else {
            notes.push(format!("{n}（通书：{w}；协纪卷三十六斥）"));
        }
    }
    if let Some((n, w)) = tian_gou_ji(d.month_ord, gz % 12) {
        notes.push(format!("{n}（通书：{w}）"));
    }
    if let Some((n, he)) = wu_he_li_su(gz) {
        // 人民離已在类编凶神内；余者俗名只作附注
        if n != "人民離" {
            notes.push(format!("{n}（俗术五{}之名；协纪卷三十六斥为附会）", if he { "合" } else { "離" }));
        }
    }
    if zhang_guang(d.month_ord, gz % 12) {
        if o.zhang_guang {
            veto.push(("章光（堪輿經：月厭前一辰）".into(), None));
        } else {
            notes.push("章光（堪輿經月厭前一辰；协纪卷三十六斥）".into());
        }
    }
    // 紅沙（协纪所引《轉神厯》等；协纪不用）
    if o.hong_sha != HongShaRule::BuYong && hong_sha(o.hong_sha, d.month_ord, gz % 12) {
        veto.push((format!("紅沙（{}）", if o.hong_sha == HongShaRule::ZhuanShen { "轉神厯：孟酉仲巳季丑" } else { "又一本：孟巳仲酉季丑" }), None));
    } else if hong_sha(HongShaRule::ZhuanShen, d.month_ord, gz % 12) {
        notes.push("紅沙（轉神厯；协纪不用）".into());
    }
    if lx.iter().any(|(n, _)| *n == "披麻殺") {
        notes.push("披麻殺：类编云「俗有厭𭫲之法，詳見諸公口訣」，法未载".into());
    }
    if lx.iter().any(|(n, _)| *n == "受死") {
        notes.push(format!("受死：{JIE_SHOU_SI}"));
    }
    // 赤口（类编卷十五小六壬；不可制）
    let lr = xiao_liu_ren(d.lunar.month, d.lunar.day);
    // 兀日（协纪卷三十六所述、类编卷二六輪兀）：按农历年阴阳起
    let yang_nian = lunar_year_gz(d.year_gz, d.lunar.month, d.month_ord) % 2 == 0;
    let wu_ri = match liu_lun_wu(yang_nian, d.lunar.month, d.lunar.day) {
        "巽" => Some("上兀"),
        "坤" => Some("下兀"),
        _ => None,
    };
    if o.chi_kou == ChiKouRule::XieJiWu {
        match wu_ri {
            Some("下兀") => veto.push(("赤口（协纪所述兀日起法：下兀）".into(), None)),
            Some(w) => notes.push(format!("{w}（留連；协纪所述兀日，六輪兀巽位）")),
            None => {}
        }
        if lr == "赤口" && wu_ri != Some("下兀") {
            notes.push("赤口（类编卷十五小六壬；兀日起法不值）".into());
        }
    } else if lr == "赤口" {
        let ck_bad = match o.chi_kou {
            ChiKouRule::BuYong | ChiKouRule::XieJiWu => false,
            ChiKouRule::LeiBian => true,
            ChiKouRule::QuYi => matches!(gz % 12, 2 | 5 | 9 | 10),
        };
        if ck_bad {
            veto.push((format!("赤口（{}）", if o.chi_kou == ChiKouRule::QuYi { "祛疑説：值寅巳酉戌" } else { "类编" }), None));
        } else {
            notes.push("赤口（类编卷十五小六壬；协纪不立）".into());
        }
    }
    // 橫天朱雀（鳌头卷三；不可制）
    if heng_tian_zhu_que(d.lunar.day) {
        if o.heng_tian_zhu_que {
            veto.push(("鳌头：橫天朱雀（初一）".into(), None));
        } else {
            notes.push("橫天朱雀（鳌头：初一行嫁主再娶）".into());
        }
    }
    // 协纪改删之旧说与类编、鳌头、董公诸神（清点篇第十一节待实现清单 4—21）：置位者否决（不可制），余只作附注
    let ti = TongshuInput {
        month_ord: d.month_ord,
        day_gz: gz,
        jdn: d.jdn,
        year_gz: d.year_gz,
        nv_year_gz: inp.nv_year_gz,
        phase: d.input.phase,
        hui: d.input.lunar_day == d.input.days_in_month,
    };
    let hits = tongshu_hits(&ti);
    for (i, txt) in &hits {
        let (_, grp, why) = TONGSHU_SHEN[*i];
        if o.tongshu_mask >> i & 1 == 1 {
            veto.push((format!("{grp}：{txt}"), None));
        } else {
            notes.push(format!("{txt}（{grp}：{why}）"));
        }
    }
    // 旧本宜婚之神（协纪改其所宜）与黄道：只作附注
    let jb = jiu_ben_yi_shen(d.month_ord, gz % 12);
    let (hd, is_hd) = huang_hei_dao(d.month_ord, gz % 12);
    if !jb.is_empty() || is_hd {
        let mut v: Vec<String> = jb.iter().map(|(n, a, b)| format!("{n}（{a}；协纪{b}）")).collect();
        if is_hd {
            v.push(format!("黃道{hd}（神樞經：宜結婚姻；协纪卷十「諸事皆宜而無專宜之事」）"));
        }
        notes.push(format!("旧说宜婚：{}", v.join("、")));
    }
    // 十二星斷（鳌头卷一）之宜：建平宜行嫁，滿定執成收開宜婚，閉宜娶婦（破危之忌、除日又說在上）
    let (xd_jc, xd_yj, xd_ju) = super::jiaqu_tongshu::shi_er_xing_duan(d.month_ord, gz % 12);
    if xd_yj == "宜" {
        notes.push(format!("十二星斷：{xd_jc}日「{xd_ju}」（鳌头卷一）"));
    }
    let sg = sha_gong_zhi_xing_ren_zhuan(d.month_ord, gz);
    if !sg.is_empty() {
        notes.push(format!("董公：{}（「能解凶星，唯不能解金神七煞」；略记又云白虎入中宮之日「莫若不用為上」）", sg.join("、")));
    }
    // 董公逐日婚姻宜忌（婚姻、納采与泛言之諸事）
    let dg = super::dong_gong::dong_gong_day((d.month_ord + 2) % 12, gz);
    let dg_rules: Vec<&super::dong_gong::DongGongRule> = dg.rules.iter().filter(|r| matches!(r.event, "婚姻" | "納采" | "諸事")).collect();
    let dg_txt: Vec<String> = dg_rules
        .iter()
        .map(|r| {
            let g = if r.grade.is_empty() { String::new() } else { format!("，{}", r.grade) };
            let c = if r.condition.is_empty() { String::new() } else { format!("，{}", r.condition) };
            format!("{}{}{g}{c}：{}", r.event, if r.yi { "宜" } else { "忌" }, r.basis)
        })
        .collect();
    let dg_ji = dg_rules.iter().any(|r| !r.yi);
    let dg_yi = dg_rules.iter().any(|r| r.yi);
    match o.dong_gong {
        DongGongHun::BiJi | DongGongHun::XuYi if dg_ji => veto.push(("董公：忌婚姻".into(), None)),
        DongGongHun::XuYi if !dg_yi => veto.push((format!("董公：不注宜婚姻{}", if dg.no_text() { "（董公无宜忌之文）" } else { "" }), None)),
        _ => {}
    }
    if !dg_txt.is_empty() {
        notes.push(format!("董公婚姻：{}", dg_txt.join("；")));
    }
    // 鳌头卷三婚姻四表
    let at: Vec<super::aotou::AoTouJiRi> = super::aotou::jiri_for_day(d.month_ord, gz).into_iter().filter(|r| r.table.starts_with("逐月") && matches!(r.shi.as_str(), "求婚" | "下定結婚" | "嫁娶") || r.table == "逐月嫁娶結婚吉日備急用").collect();
    let at_txt: Vec<String> = at
        .iter()
        .map(|r| {
            let name = if r.table == "逐月嫁娶結婚吉日備急用" { "嫁娶備急".to_string() } else { r.shi.clone() };
            let n = [r.deng.as_str(), r.note.as_str()].iter().filter(|x| !x.is_empty()).copied().collect::<Vec<_>>().join("，");
            if n.is_empty() { name } else { format!("{name}（{n}）") }
        })
        .collect();
    if o.aotou_jiri == AoTouJiRiRule::XuJiaQu && !at.iter().any(|r| r.shi == "嫁娶" || r.table == "逐月嫁娶結婚吉日備急用") {
        veto.push(("鳌头：不在嫁娶、嫁娶備急表".into(), None));
    }
    if !at_txt.is_empty() {
        notes.push(format!("鳌头婚姻表：{}", at_txt.join("、")));
    }
    let jiri = leibian_jiri(d.month_ord, gz);
    if !jiri.is_empty() {
        notes.push(format!(
            "类编吉日表：{}",
            jiri.iter().map(|(b, z)| if z.is_empty() { b.name().to_string() } else { format!("{}（{z}）", b.name()) }).collect::<Vec<_>>().join("、")
        ));
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
        xieji_basis: xieji_basis_ignoring(d, o.ignore_shen, o.ignore_puzhu),
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
        xiao_liu_ren: lr,
        wu_ri,
        leibian_jiri: jiri,
        tongshu: hits.iter().map(|(i, _)| TONGSHU_SHEN[*i].0).collect(),
        dong_gong: dg_txt,
        aotou_jiri: at_txt,
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
mod tests_tonshu_su {
    use super::*;

    /// 协纪卷三十六所举诸俗说之日（逐条与原文核）
    #[test]
    fn su_shuo_days() {
        // 楊公忌：正月十三、二月十一……七月初一、二十九……十二月十九，凡十三日
        let days: Vec<(u8, u8)> = (1..=12u8).flat_map(|m| (1..=30u8).map(move |d| (m, d))).filter(|&(m, d)| yang_gong_ji(m, d)).collect();
        assert_eq!(days.len(), 13);
        assert!(days.contains(&(1, 13)) && days.contains(&(2, 11)) && days.contains(&(7, 1)) && days.contains(&(7, 29)) && days.contains(&(12, 19)));
        // 紅沙：轉神厯孟酉仲巳季丑；又一本孟巳仲酉
        assert!(hong_sha(HongShaRule::ZhuanShen, 0, 9) && hong_sha(HongShaRule::ZhuanShen, 1, 5) && hong_sha(HongShaRule::ZhuanShen, 2, 1));
        assert!(hong_sha(HongShaRule::YouYiBen, 0, 5) && hong_sha(HongShaRule::YouYiBen, 1, 9));
        // 章光：正月酉（月厭戌之前一辰）、二月申……十二月戌
        assert!(zhang_guang(0, 9) && zhang_guang(1, 8) && zhang_guang(11, 10));
        // 天狗：春月厭正位酉为頭、厭對卯为尾、章光正位申为口、天喜正位午为腹
        assert_eq!(tian_gou(1, 9).map(|x| x.0), Some("天狗頭"));
        assert_eq!(tian_gou(1, 3).map(|x| x.0), Some("天狗尾"));
        assert_eq!(tian_gou(1, 8).map(|x| x.0), Some("天狗口"));
        assert_eq!(tian_gou(1, 6).map(|x| x.0), Some("天狗腹"));
        assert_eq!(tian_gou(10, 0).map(|x| x.0), Some("天狗頭")); // 冬子
        // 天狗背、足、後足：春子、戌、辰
        assert_eq!(tian_gou_ji(0, 0).map(|x| x.0), Some("天狗背"));
        assert_eq!(tian_gou_ji(0, 10).map(|x| x.0), Some("天狗足"));
        assert_eq!(tian_gou_ji(0, 4).map(|x| x.0), Some("天狗後足"));
        // 五合五離俗名：戊寅人民合、己酉人民離、甲寅日月合
        let gz = |n: &str| (0..60u8).find(|k| "甲乙丙丁戊己庚辛壬癸".chars().nth(*k as usize % 10) == n.chars().next() && "子丑寅卯辰巳午未申酉戌亥".chars().nth(*k as usize % 12) == n.chars().nth(1)).unwrap();
        assert_eq!(wu_he_li_su(gz("戊寅")), Some(("人民合".to_string(), true)));
        assert_eq!(wu_he_li_su(gz("己酉")), Some(("人民離".to_string(), false)));
        assert_eq!(wu_he_li_su(gz("甲寅")), Some(("日月合".to_string(), true)));
    }
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

    /// 赤口：类编卷十五歌诀与例，及《田家五行》「月建上起初一日，順數加辰戌二位」所推全同
    #[test]
    fn chi_kou_xiao_liu_ren() {
        assert_eq!(xiao_liu_ren(1, 1), "留連");
        assert_eq!(xiao_liu_ren(1, 2), "速喜");
        assert_eq!(xiao_liu_ren(1, 3), "赤口");
        assert_eq!(xiao_liu_ren(1, 6), "大安");
        // 正七留連、二八速喜、三九赤口、四十小吉、五十一空亡、六十二大安（初一）
        for (m, s) in [(1, "留連"), (7, "留連"), (2, "速喜"), (8, "速喜"), (3, "赤口"), (9, "赤口"), (4, "小吉"), (10, "小吉"), (5, "空亡"), (11, "空亡"), (6, "大安"), (12, "大安")] {
            assert_eq!(xiao_liu_ren(m, 1), s, "{m}月初一");
        }
        for m in 1..=12u8 {
            for d in 1..=30u8 {
                let tian_jia = (m as u32 + 1 + d as u32 - 1) % 12; // 月建（正月寅）上起初一顺数
                assert_eq!(xiao_liu_ren(m, d) == "赤口", tian_jia == 4 || tian_jia == 10, "{m}月{d}日");
            }
        }
        assert!(heng_tian_zhu_que(1) && !heng_tian_zhu_que(2));
    }

    /// 协纪卷三十六兀日起法 / 类编卷二「六輪兀」：与类编卷二起例图（02094404:27）、陽年陰年上下兀日两表（02094404:28—29）逐格比对。
    /// `cargo test -p lunar-core liu_lun_wu_tables -- --nocapture` 打印比对表（写入 doc/folk/jiaqu.md）
    #[test]
    fn liu_lun_wu_tables() {
        // 起例图：各卦所值之月（初一所起）。(卦, 陽年月, 陰年月)
        let tu: [(&str, [u8; 2], [u8; 2]); 6] = [
            ("乾", [1, 7], [4, 10]),
            ("坎", [2, 8], [5, 11]),
            ("艮", [3, 9], [6, 12]),
            ("巽", [4, 10], [1, 7]),
            ("離", [5, 11], [2, 8]),
            ("坤", [6, 12], [3, 9]),
        ];
        let mut tu_ok = 0;
        for (g, ym, nm) in tu {
            for m in ym {
                tu_ok += (liu_lun_wu(true, m, 1) == g) as usize;
            }
            for m in nm {
                tu_ok += (liu_lun_wu(false, m, 1) == g) as usize;
            }
        }
        // 两表：月组（正七、二八……六十二）上兀、下兀之首日，其后每六日一见（表中五日）
        let yang: [(u8, u8); 6] = [(4, 6), (3, 5), (2, 4), (1, 3), (6, 2), (5, 1)];
        let yin: [(u8, u8); 6] = [(1, 3), (6, 2), (5, 1), (4, 6), (3, 5), (2, 4)];
        let mut rows = Vec::new();
        for (label, yn, tab) in [("陽年", true, yang), ("陰年", false, yin)] {
            let (mut n, mut ok) = (0, 0);
            for m in 1..=12u8 {
                let (s, x) = tab[((m - 1) % 6) as usize];
                for d in 1..=30u8 {
                    let want = if (d as i32 - s as i32).rem_euclid(6) == 0 {
                        Some("巽")
                    } else if (d as i32 - x as i32).rem_euclid(6) == 0 {
                        Some("坤")
                    } else {
                        None
                    };
                    let got = match liu_lun_wu(yn, m, d) {
                        g @ ("巽" | "坤") => Some(g),
                        _ => None,
                    };
                    n += 1;
                    ok += (want == got) as usize;
                }
            }
            rows.push((label, n, ok));
        }
        // 协纪：阳年正月初一起小吉、阴年起留連；兀日即留連（上兀）赤口（下兀）
        assert_eq!(xiao_liu_ren_wu(true, 1, 1), "小吉");
        assert_eq!(xiao_liu_ren_wu(false, 1, 1), "留連");
        let mut map_ok = 0;
        let mut yin_same = 0;
        let mut yang_diff = 0;
        for m in 1..=12u8 {
            for d in 1..=30u8 {
                for yn in [true, false] {
                    let (g, x) = (liu_lun_wu(yn, m, d), xiao_liu_ren_wu(yn, m, d));
                    map_ok += ((g == "巽") == (x == "留連") && (g == "坤") == (x == "赤口")) as usize;
                }
                yin_same += (xiao_liu_ren_wu(false, m, d) == xiao_liu_ren(m, d)) as usize;
                yang_diff += ((xiao_liu_ren_wu(true, m, d) == "赤口") != (xiao_liu_ren(m, d) == "赤口")) as usize;
            }
        }
        println!("| 比对 | 格 | 合 | 不合 |\n|---|---|---|---|");
        println!("| 起例图（各卦所值月之初一） | 24 | {tu_ok} | {} |", 24 - tu_ok);
        for (l, n, ok) in &rows {
            println!("| {l}上下兀日表（12 月 × 30 日） | {n} | {ok} | {} |", n - ok);
        }
        println!("| 巽↔留連、坤↔赤口（协纪所述，两年 × 12 月 × 30 日） | 720 | {map_ok} | {} |", 720 - map_ok);
        println!("| 阴年兀日起法 vs 类编卷十五小六壬 | 360 | {yin_same} | {} |", 360 - yin_same);
        println!("| 阳年赤口日与类编卷十五赤口日相异之日 | 360 | — | {yang_diff} |");
        assert_eq!(tu_ok, 24);
        for (l, n, ok) in rows {
            assert_eq!(n, ok, "{l}");
        }
        assert_eq!(map_ok, 720);
        assert_eq!(yin_same, 360);
        // 阳年赤口移三位，与类编赤口无一日同：每月 5 日类编赤口 + 5 日兀日赤口
        assert_eq!(yang_diff, 120);
        // 农历年：2026-02-10 农历十二月廿三、节月寅（立春后），立春年丙午（42）→ 农历年乙巳（41）；
        // 2025-01-30 农历正月初二、节月丑，立春年甲辰（40）→ 农历年乙巳（41）
        assert_eq!(lunar_year_gz(42, 12, 0), 41);
        assert_eq!(lunar_year_gz(40, 1, 11), 41);
        assert_eq!(lunar_year_gz(42, 3, 1), 42);
    }

    /// 类编卷七逐月吉日表（录文 data/folk/leibian_jiaqu_jiri.csv）与 jiaqu 诸判据逐条比对（按月建、干支，不涉具体年日）。
    /// 设环境变量 `JIAQU_JIRI_CHECKS` 时写出逐条结果 data/folk/leibian_jiaqu_jiri_checks.csv（工作目录须为 rust/core）。
    #[test]
    fn leibian_jiri_vs_rules() {
        use super::super::yiji::yuebiao_day;
        let es = jiri_entries();
        let cnt = |b: JiriBiao| es.iter().filter(|e| e.biao == b).count();
        assert_eq!((cnt(JiriBiao::QiuHun), cnt(JiriBiao::XiaDing), cnt(JiriBiao::JiaQu), cnt(JiriBiao::BeiJi)), (113, 100, 54, 67));
        // 凶神：不随年变者（雷霆白虎随月干、伏斷随值宿，不入此比）
        const VETO: [&str; 19] = ["月厭", "厭對", "歸忌", "天賊", "月破", "亥日", "大殺白虎入中宮", "人民離", "披麻殺", "天寡", "地寡", "紅沙殺", "天罡勾絞", "河魁勾絞", "往亡", "無翹", "陰錯", "陽錯", "受死"];
        let mut out = String::from("表,月,干支,注,協紀月表宜,協紀歷例不將,類編法不將,類編凶神,類編所列他神,注與推算\n");
        let mut stat: Vec<(JiriBiao, [usize; 6])> = JiriBiao::ALL.iter().map(|b| (*b, [0; 6])).collect();
        for e in es {
            let m = e.month - 1;
            let ev = match e.biao {
                JiriBiao::QiuHun => "結婚姻",
                JiriBiao::XiaDing => "納采問名",
                _ => "嫁娶",
            };
            let yi = yuebiao_day(m, e.gz).yi.as_ref().is_some_and(|v| v.iter().any(|x| x == ev));
            let bjx = bu_jiang_xieji(m, e.gz);
            let bjl = jiang(m, e.gz, WuJiJi::LeiBian) == Jiang::BuJiang;
            let all: Vec<&str> = leibian_xiong(m, 0, e.gz, 0).into_iter().map(|(n, _)| n).filter(|n| !["雷霆白虎入中宮", "伏斷"].contains(n)).collect();
            let xs: Vec<&str> = all.iter().copied().filter(|n| VETO.contains(n)).collect();
            let other: Vec<&str> = all.iter().copied().filter(|n| !VETO.contains(n)).collect();
            // 注云不將者，推算是否不將（任一家）
            let zhu_bj = e.zhu.contains("不將");
            let agree = if zhu_bj { if bjx || bjl { "注不將，推算合" } else { "注不將，推算非不將" } } else { "" };
            let s = &mut stat.iter_mut().find(|(b, _)| *b == e.biao).unwrap().1;
            s[0] += 1;
            s[1] += yi as usize;
            s[2] += bjx as usize;
            s[3] += bjl as usize;
            s[4] += (!xs.is_empty()) as usize;
            s[5] += (zhu_bj && !(bjx || bjl)) as usize;
            out += &format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                e.biao.name(),
                e.month,
                GZ60[e.gz as usize],
                e.zhu,
                if yi { "宜" } else { "" },
                if bjx { "是" } else { "" },
                if bjl { "是" } else { "" },
                xs.join(" "),
                other.join(" "),
                agree
            );
        }
        for (b, s) in &stat {
            eprintln!("{}：{} 条，协纪月表宜{} {}，协纪不将 {}，类编法不将 {}，犯类编凶神 {}，注不将而推算非 {}", b.name(), s[0], match b {
                JiriBiao::QiuHun => "結婚姻",
                JiriBiao::XiaDing => "納采問名",
                _ => "嫁娶",
            }, s[1], s[2], s[3], s[4], s[5]);
        }
        if std::env::var("JIAQU_JIRI_CHECKS").is_ok() {
            std::fs::write("../../data/folk/leibian_jiaqu_jiri_checks.csv", out).unwrap();
        }
    }

    /// 2026 年逐日：类编嫁娶吉日表所列之日在诸规则下之判，及赤口、橫天朱雀之选项于可用日之影响（女命乙巳，翁姑、女父母俱在）
    #[test]
    fn jiri_and_chi_kou_2026() {
        use crate::astronomy::ephemeris::{load_all, load_earth_vsop87_from_repo, Elpmpp02Correction};
        use crate::calendar::gregorian::Gregorian;
        use crate::math::real::RealOps;
        let vsop = load_earth_vsop87_from_repo().unwrap();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
        let j0 = (Gregorian::to_julian_day(2026, 1, 1).as_f64() + 0.5).floor() as i64;
        let inp = JiaquInput { nv_year_gz: 41, has_weng_gu: true, has_nv_fu_mu: true, na_xu: false };
        let lb = JiaquOptions { leibian_xiong: true, zhou_tang: ZhouTangRule::LeiBian, ..JiaquOptions::default() };
        let cases = [
            ("协纪（缺省）", JiaquOptions::default()),
            ("协纪 + 赤口类编", JiaquOptions { chi_kou: ChiKouRule::LeiBian, ..JiaquOptions::default() }),
            ("协纪 + 赤口祛疑説", JiaquOptions { chi_kou: ChiKouRule::QuYi, ..JiaquOptions::default() }),
            ("协纪 + 赤口兀日起法", JiaquOptions { chi_kou: ChiKouRule::XieJiWu, ..JiaquOptions::default() }),
            ("协纪 + 橫天朱雀", JiaquOptions { heng_tian_zhu_que: true, ..JiaquOptions::default() }),
            ("类编凶神 + 类编周堂", lb),
            ("类编凶神 + 类编周堂 + 赤口类编 + 橫天朱雀", JiaquOptions { chi_kou: ChiKouRule::LeiBian, heng_tian_zhu_que: true, ..lb }),
            ("只用类编凶神（协纪月表、周堂不用）", JiaquOptions { xieji_yiji: XieJiYiJi::BuYong, zhou_tang: ZhouTangRule::BuYong, leibian_xiong: true, ..JiaquOptions::default() }),
        ];
        let mut ck_days = 0;
        for (name, o) in cases {
            let v = choose(&vsop, &elp, None, j0, j0 + 364, &inp, &o);
            ck_days = v.iter().filter(|d| d.xiao_liu_ren == "赤口").count();
            let ok = v.iter().filter(|d| d.usable()).count();
            let in_jq: Vec<&JiaquDay> = v.iter().filter(|d| d.leibian_jiri.iter().any(|(b, _)| *b == JiriBiao::JiaQu)).collect();
            let in_bj = v.iter().filter(|d| d.leibian_jiri.iter().any(|(b, _)| *b == JiriBiao::BeiJi)).count();
            let jq_ok = in_jq.iter().filter(|d| d.usable()).count();
            let ok_in_any = v.iter().filter(|d| d.usable() && d.leibian_jiri.iter().any(|(b, _)| matches!(b, JiriBiao::JiaQu | JiriBiao::BeiJi))).count();
            eprintln!("{name}：可用 {ok}；嫁娶表之日 {} 中可用 {jq_ok}；備急表之日 {in_bj}；可用日见于嫁娶或備急表 {ok_in_any}", in_jq.len());
        }
        eprintln!("2026 年赤口日 {ck_days}");
        // 兀日起法：春节（2026-02-17）前为乙巳阴年，下兀与类编赤口同日；春节后丙午阳年，无一日同
        let cj = (Gregorian::to_julian_day(2026, 2, 17).as_f64() + 0.5).floor() as i64;
        let v = choose(&vsop, &elp, None, j0, j0 + 364, &inp, &JiaquOptions { chi_kou: ChiKouRule::XieJiWu, ..JiaquOptions::default() });
        let xia: Vec<&JiaquDay> = v.iter().filter(|d| d.wu_ri == Some("下兀")).collect();
        eprintln!("2026 年下兀日 {}，上兀日 {}", xia.len(), v.iter().filter(|d| d.wu_ri == Some("上兀")).count());
        for d in &v {
            let ck = d.xiao_liu_ren == "赤口";
            let xw = d.wu_ri == Some("下兀");
            if d.jdn < cj {
                assert_eq!(ck, xw, "{}", d.jdn);
            } else {
                assert!(!(ck && xw), "{}", d.jdn);
            }
            if xw {
                assert!(d.reasons.iter().any(|r| r.contains("下兀")));
            }
        }
        assert!((55..=67).contains(&xia.len()), "{}", xia.len());
        // 赤口日约占六分之一
        assert!((55..=67).contains(&ck_days), "{ck_days}");
    }

    /// 2026 年逐日：新增诸选项（铺注逐项不看、协纪改删旧说与类编鳌头董公诸神逐神否决、董公婚姻、鳌头婚姻表）于可用日之影响。
    /// 缺省判定与诸选项全关时同；每项否决只落在所值之日；铺注全看时与 `yiji_for_day` 逐日全同。
    /// `cargo test -p lunar-core --lib tongshu_options_2026 -- --nocapture` 打印表（写入 doc/folk/jiaqu.md）
    #[test]
    fn tongshu_options_2026() {
        use super::super::jiaqu_tongshu::TONGSHU_SHEN;
        use super::super::puzhu::{puzhu_days, yiji_for_day, QiWangWangCount, PUZHU_HUN};
        use crate::astronomy::ephemeris::{load_all, load_earth_vsop87_from_repo, Elpmpp02Correction};
        use crate::calendar::gregorian::Gregorian;
        use crate::math::real::RealOps;
        let vsop = load_earth_vsop87_from_repo().unwrap();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
        let j0 = (Gregorian::to_julian_day(2026, 1, 1).as_f64() + 0.5).floor() as i64;
        let days = puzhu_days(&vsop, &elp, None, j0, j0 + 364, QiWangWangCount::Inclusive);
        let inp = JiaquInput { nv_year_gz: 41, has_weng_gu: true, has_nv_fu_mu: true, na_xu: false };
        let run = |o: &JiaquOptions| -> Vec<JiaquDay> { days.iter().map(|d| judge_day(d, &inp, o)).collect() };
        let base = run(&JiaquOptions::default());
        let n_ok = |v: &[JiaquDay]| v.iter().filter(|d| d.usable()).count();
        // 铺注全看：与 yiji_for_day 逐日全同
        for (d, j) in days.iter().zip(&base) {
            let y = yiji_for_day(d);
            assert_eq!(j.xieji_yi, y.yi.as_ref().is_some_and(|v| v.iter().any(|x| x == "嫁娶")));
        }
        println!("| 选项 | 所值之日 | 可用（缺省 {}） | 由否变可／由可变否 |\n|---|---|---|---|", n_ok(&base));
        let diff = |v: &[JiaquDay]| -> (usize, usize) {
            let a = v.iter().zip(&base).filter(|(x, y)| x.usable() && !y.usable()).count();
            let b = v.iter().zip(&base).filter(|(x, y)| !x.usable() && y.usable()).count();
            (a, b)
        };
        // 铺注逐项不看
        for (i, n) in PUZHU_HUN.iter().enumerate() {
            let v = run(&JiaquOptions { ignore_puzhu: 1 << i, ..JiaquOptions::default() });
            let (a, b) = diff(&v);
            assert_eq!(b, 0, "{n}：不看铺注不应使可用日变否");
            let hit = days.iter().zip(&v).zip(&base).filter(|((_, x), y)| x.xieji_yi != y.xieji_yi).count();
            println!("| 不看铺注「{n}」 | 改注宜 {hit} | {} | {a}／{b} |", n_ok(&v));
        }
        let all_off = run(&JiaquOptions { ignore_puzhu: 0x1ff, ..JiaquOptions::default() });
        println!("| 不看铺注九项 | — | {} | {}／{} |", n_ok(&all_off), diff(&all_off).0, diff(&all_off).1);
        // 诸神逐神否决
        for (i, (n, grp, _)) in TONGSHU_SHEN.iter().enumerate() {
            let v = run(&JiaquOptions { tongshu_mask: 1 << i, ..JiaquOptions::default() });
            let hit = base.iter().filter(|d| d.tongshu.contains(n)).count();
            let (a, b) = diff(&v);
            assert_eq!(a, 0, "{n}");
            // 否决只落在所值之日
            for (x, y) in v.iter().zip(&base) {
                if x.usable() != y.usable() {
                    assert!(y.tongshu.contains(n));
                    assert!(x.reasons.iter().any(|r| r.starts_with(grp)));
                }
            }
            println!("| {grp}：{n} | {hit} | {} | {a}／{b} |", n_ok(&v));
        }
        let all_on = run(&JiaquOptions { tongshu_mask: (1 << TONGSHU_SHEN.len()) - 1, ..JiaquOptions::default() });
        println!("| 诸神全否决 | — | {} | {}／{} |", n_ok(&all_on), diff(&all_on).0, diff(&all_on).1);
        for (name, o) in [
            ("董公：避忌婚姻", JiaquOptions { dong_gong: DongGongHun::BiJi, ..JiaquOptions::default() }),
            ("董公：须宜婚姻", JiaquOptions { dong_gong: DongGongHun::XuYi, ..JiaquOptions::default() }),
            ("鳌头：须见嫁娶或備急表", JiaquOptions { aotou_jiri: AoTouJiRiRule::XuJiaQu, ..JiaquOptions::default() }),
            ("只用鳌头嫁娶表（协纪月表、周堂不用）", JiaquOptions { xieji_yiji: XieJiYiJi::BuYong, zhou_tang: ZhouTangRule::BuYong, aotou_jiri: AoTouJiRiRule::XuJiaQu, ..JiaquOptions::default() }),
            ("只用董公须宜（协纪月表、周堂不用）", JiaquOptions { xieji_yiji: XieJiYiJi::BuYong, zhou_tang: ZhouTangRule::BuYong, dong_gong: DongGongHun::XuYi, ..JiaquOptions::default() }),
        ] {
            let v = run(&o);
            let hit = match o.dong_gong {
                DongGongHun::BuYong => base.iter().filter(|d| !d.aotou_jiri.is_empty()).count(),
                _ => base.iter().filter(|d| !d.dong_gong.is_empty()).count(),
            };
            let (a, b) = diff(&v);
            println!("| {name} | {hit} | {} | {a}／{b} |", n_ok(&v));
        }
        // 女嫁厄年：女命乙巳（巳）厄年为戌；2026 丙午非厄年，女命己酉（酉）厄年为午
        assert!(base.iter().all(|d| !d.tongshu.contains(&"女嫁厄年")));
        let inp2 = JiaquInput { nv_year_gz: 45, ..inp };
        let v2: Vec<JiaquDay> = days.iter().map(|d| judge_day(d, &inp2, &JiaquOptions::default())).collect();
        let cj = (Gregorian::to_julian_day(2026, 2, 4).as_f64() + 0.5).floor() as i64;
        for d in &v2 {
            if d.jdn > cj {
                assert!(d.tongshu.contains(&"女嫁厄年"), "{}", d.jdn);
            }
        }
    }

    const GZ60: [&str; 60] = [
        "甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己巳", "庚午", "辛未", "壬申", "癸酉", "甲戌", "乙亥", "丙子", "丁丑", "戊寅", "己卯", "庚辰", "辛巳", "壬午", "癸未", "甲申", "乙酉", "丙戌", "丁亥", "戊子", "己丑", "庚寅", "辛卯", "壬辰", "癸巳", "甲午", "乙未", "丙申", "丁酉", "戊戌", "己亥", "庚子", "辛丑", "壬寅", "癸卯", "甲辰", "乙巳", "丙午", "丁未", "戊申", "己酉", "庚戌", "辛亥", "壬子", "癸丑", "甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥",
    ];

    #[test]
    fn huang_dao_hours() {
        // 子日：青龍起申（2×0−4 ≡ 8），黄道申酉子丑卯午
        let h: Vec<u8> = huang_dao_shi(0).iter().map(|(z, _)| *z).collect();
        assert_eq!(h, vec![0, 1, 3, 6, 8, 9]);
    }
}
