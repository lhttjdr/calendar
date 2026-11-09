//! 紫微斗数排盘（十四主星一派）：命宫、身宫、十二宫、五行局、紫微天府两系十四主星、辅佐煞曜、四化、大限小限。
//!
//! 起例以《紫微斗數全書》卷二「安身命例」以下诸诀为主（维基文库简体录文，未注底本；见 doc/folk/ziwei.md），
//! 各派差异做成选项（[`ZiweiOptions`]），缺省取全书或证据最强者，理由见文档。
//! 《續道藏》本《紫微斗數》（題陳摶）为十八飞星一派（紫、虛、貴、印、壽……），星系与安法皆不同，不在本模块。
//!
//! 宫位以地支序表示：0 子 … 11 亥。时辰亦以地支序：0 子时 … 11 亥时。
//! 农历日期（月、闰、日）由调用方给出（实历优先的换算见 calendar::lifa / chinese_lunar）。

/// 十二支
pub const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
/// 十干
pub const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
/// 十二宫名（自命宫逆布；全书卷二「安十二宫例」：男女俱从逆转）
pub const PALACES: [&str; 12] = ["命宮", "兄弟", "妻妾", "子女", "財帛", "疾厄", "遷移", "奴僕", "官祿", "田宅", "福德", "父母"];
/// 十四主星（紫微系六星、天府系八星）
pub const MAIN_STARS: [&str; 14] =
    ["紫微", "天機", "太陽", "武曲", "天同", "廉貞", "天府", "太陰", "貪狼", "巨門", "天相", "天梁", "七殺", "破軍"];

fn m12(k: i32) -> u8 {
    k.rem_euclid(12) as u8
}

/// 闰月之月如何起命宫
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeapRule {
    /// 作下月：全书卷二「閏月正月生者要在二月內起安身命，凡有閏月具要依此為例」（缺省）
    NextMonth,
    /// 作本月（近人一说，出处未核）
    SameMonth,
    /// 十五日以前作本月、十六日以后作下月（近人一说，出处未核）
    SplitAt15,
}

/// 辛年天魁天钺之配
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XinKuiYue {
    /// 魁午、钺寅（通行；缺省）
    KuiWuYueYin,
    /// 魁寅、钺午（照全书诀「六辛逢虎馬」字序）
    KuiYinYueWu,
}

/// 丙丁年天钺之位
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BingDingYue {
    /// 钺在酉（「丙丁豬雞位」，与天乙贵人丙丁亥酉同；缺省）
    You,
    /// 钺在戌（全书录文作「丙丁猪狗位」，疑录文之误，待刻本核）
    Xu,
}

/// 四化表之家
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiHuaTable {
    /// 全书卷二「安祿權科忌四星變化訣」：庚日武陰同、壬梁紫府武、戊貪月弼機（缺省）
    QuanShu,
    /// 近人通行之一：庚太陽武曲天府天同，壬天梁紫微左輔武曲（出处未核）
    Modern,
    /// 韩国国立中央图书馆藏写本《紫微斗數方書》第 10 页「定祿權科忌四化星法」：庚日武月同（同全书），壬梁紫左武（科作左輔，异于全书之天府）
    KoreanMs,
}

/// 长生十二神（及博士十二神以外者）的顺逆
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangShengDir {
    /// 全书卷二：「男命順數、女命逆數」（缺省）
    MaleForward,
    /// 阳男阴女顺、阴男阳女逆（韩国藏写本《紫微斗數方書》第 10 页「定胞胎」：「陽男陰女順，陰男陽女逆」）
    YangMaleYinFemaleForward,
}

#[derive(Clone, Copy, Debug)]
pub struct ZiweiOptions {
    pub leap: LeapRule,
    pub xin_kui_yue: XinKuiYue,
    pub bing_ding_yue: BingDingYue,
    pub si_hua: SiHuaTable,
    pub chang_sheng: ChangShengDir,
}

impl Default for ZiweiOptions {
    fn default() -> Self {
        Self {
            leap: LeapRule::NextMonth,
            xin_kui_yue: XinKuiYue::KuiWuYueYin,
            bing_ding_yue: BingDingYue::You,
            si_hua: SiHuaTable::QuanShu,
            chang_sheng: ChangShengDir::MaleForward,
        }
    }
}

/// 巳酉丑年火星、铃星之起宫
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiYouChouHuoLing {
    /// 火卯铃戌：全书「巳酉丑人卯戌位」，依他组「丑卯」「寅戌」「酉戌」先火后铃之序（缺省）
    QuanShu,
    /// 火戌铃卯：韩国藏写本第 9 页「天火天令」：「巳酉丑生戌火卯令」
    KoreanMs,
}

/// 身主之表
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShenZhuTable {
    /// 全书「子午人火玲星……辰戌人文昌星」：子午读作铃星（「火玲星」即属火之铃星；韩国藏写本作「子午令」可证），辰戌文昌（缺省）
    QuanShu,
    /// 韩国藏写本第 11 页「身主」：子午令（铃星）、丑未相、寅申梁、卯酉同、辰戌曲（文曲）、巳亥機
    KoreanMs,
    /// 子午火星、辰戌文昌（近人通行之读法，出处未核）
    HuoXing,
}

/// 补充选项（不入 [`ZiweiOptions`]，以免改动既有结构）：巳酉丑火铃、身主
#[derive(Clone, Copy, Debug)]
pub struct ZiweiExtraOptions {
    pub huo_ling: SiYouChouHuoLing,
    pub shen_zhu: ShenZhuTable,
    /// 加排韩国藏写本第 9、11 页诸星（全书无，见 [`xie_ben_stars`]）；缺省不排
    pub xie_ben_xing: bool,
}

impl Default for ZiweiExtraOptions {
    fn default() -> Self {
        Self { huo_ling: SiYouChouHuoLing::QuanShu, shen_zhu: ShenZhuTable::QuanShu, xie_ben_xing: false }
    }
}

/// 排盘输入
#[derive(Clone, Copy, Debug)]
pub struct ZiweiInput {
    /// 生年干支序（0 甲子 … 59 癸亥；以农历正月初一换年，斗数通例）
    pub year_gz: u8,
    /// 农历月 1—12
    pub lunar_month: u8,
    pub is_leap_month: bool,
    /// 农历日 1—30
    pub lunar_day: u8,
    /// 时辰地支序（0 子 … 11 亥）
    pub hour_zhi: u8,
    pub male: bool,
}

/// 一颗星及其所在宫（地支序）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Star {
    pub name: &'static str,
    pub zhi: u8,
}

/// 命盘
#[derive(Clone, Debug)]
pub struct Chart {
    /// 起命宫所用之月（闰月按 [`LeapRule`] 折算后）
    pub month_used: u8,
    pub ming: u8,
    pub shen: u8,
    /// 命宫干支序
    pub ming_gz: u8,
    /// 五行局数（水二、木三、金四、土五、火六）
    pub ju: u8,
    pub ju_name: &'static str,
    /// 十二宫：下标为地支，值为宫名
    pub palaces: [&'static str; 12],
    /// 十二宫天干（五虎遁）：下标为地支，值为干序
    pub palace_gan: [u8; 12],
    pub stars: Vec<Star>,
    /// 四化：(化祿/化權/化科/化忌, 星名)
    pub si_hua: Vec<(&'static str, &'static str)>,
    /// 长生十二神：下标为地支
    pub chang_sheng: [&'static str; 12],
    /// 博士十二神：下标为地支
    pub bo_shi: [&'static str; 12],
    /// 大限：(起岁, 止岁, 宫地支)，自命宫起
    pub da_xian: Vec<(u8, u8, u8)>,
    /// 小限一岁所在之宫，及顺逆（true 顺）
    pub xiao_xian_start: u8,
    pub xiao_xian_forward: bool,
    /// 命主、身主
    pub ming_zhu: &'static str,
    pub shen_zhu: &'static str,
}

impl Chart {
    /// 某星所在之宫
    pub fn star(&self, name: &str) -> Option<u8> {
        self.stars.iter().find(|s| s.name == name).map(|s| s.zhi)
    }
    /// 某宫（地支）中的星
    pub fn stars_at(&self, zhi: u8) -> Vec<&'static str> {
        self.stars.iter().filter(|s| s.zhi == zhi).map(|s| s.name).collect()
    }
}

/// 命宫、身宫：寅起正月顺数至生月，月上起子时逆数至生时安命、顺数至生时安身（全书卷二「安身命例」）
pub fn ming_shen(month: u8, hour_zhi: u8) -> (u8, u8) {
    let base = 2 + month as i32 - 1;
    (m12(base - hour_zhi as i32), m12(base + hour_zhi as i32))
}

/// 五虎遁：某年干之寅宫天干（甲己丙、乙庚戊、丙辛庚、丁壬壬、戊癸甲；全书「起五行寅例」）
pub fn yin_gan(year_gan: u8) -> u8 {
    ((year_gan % 5) * 2 + 2) % 10
}

/// 干序、支序合为干支序
fn gz_of(gan: u8, zhi: u8) -> u8 {
    (0..60u8).find(|k| k % 10 == gan % 10 && k % 12 == zhi % 12).expect("干支同阴阳")
}

/// 五行局：命宫干支纳音之五行（水二、木三、金四、土五、火六）
pub fn wu_xing_ju(ming_gz: u8) -> (u8, &'static str) {
    use super::nayin::{nayin_wuxing, WuXing};
    match nayin_wuxing(ming_gz) {
        WuXing::Shui => (2, "水二局"),
        WuXing::Mu => (3, "木三局"),
        WuXing::Jin => (4, "金四局"),
        WuXing::Tu => (5, "土五局"),
        WuXing::Huo => (6, "火六局"),
    }
}

/// 紫微所在之宫：以局数 n 除生日 d，不尽则借 k 日使整除，商 q；自寅起数 q 宫，借数奇则逆退 k 宫、偶则顺进 k 宫。
/// 与全书卷二五局安紫微图逐日相合（图中录文讹脱三日，见文档）。
pub fn zi_wei_pos(ju: u8, day: u8) -> u8 {
    let (n, d) = (ju as i32, day as i32);
    let k = (n - d % n) % n;
    let q = (d + k) / n;
    let p = 2 + q - 1;
    m12(if k % 2 == 1 { p - k } else { p + k })
}

/// 天府所在之宫：与紫微以寅申为轴相对（全书「安天府圖」：寅申二宫紫府同宫，余宫对位，如紫居丑则府居卯）
pub fn tian_fu_pos(zi_wei: u8) -> u8 {
    m12(16 - zi_wei as i32)
}

/// 十四主星（全书「安南北斗諸星訣」：紫微天機逆行旁，隔一陽武天同當，又隔二位廉貞地；
/// 天府太陰與貪狼，巨門天相及天梁，七殺空三破軍位，八星順數）
pub fn main_stars(zi_wei: u8) -> Vec<Star> {
    let z = zi_wei as i32;
    let f = tian_fu_pos(zi_wei) as i32;
    let ziwei_sys = [("紫微", 0), ("天機", -1), ("太陽", -3), ("武曲", -4), ("天同", -5), ("廉貞", -8)];
    let fu_sys = [("天府", 0), ("太陰", 1), ("貪狼", 2), ("巨門", 3), ("天相", 4), ("天梁", 5), ("七殺", 6), ("破軍", 10)];
    ziwei_sys
        .iter()
        .map(|(n, o)| Star { name: n, zhi: m12(z + o) })
        .chain(fu_sys.iter().map(|(n, o)| Star { name: n, zhi: m12(f + o) }))
        .collect()
}

/// 四化（化祿、化權、化科、化忌）所配之星，按生年干
pub fn si_hua(year_gan: u8, table: SiHuaTable) -> [&'static str; 4] {
    // 全书：甲廉破武陽，乙機梁紫月，丙同機昌廉，丁月同機巨，戊貪月弼機，己武貪梁曲，
    // 庚日武陰同，辛巨陽曲昌，壬梁紫府武，癸破巨陰貪
    let q = [
        ["廉貞", "破軍", "武曲", "太陽"],
        ["天機", "天梁", "紫微", "太陰"],
        ["天同", "天機", "文昌", "廉貞"],
        ["太陰", "天同", "天機", "巨門"],
        ["貪狼", "太陰", "右弼", "天機"],
        ["武曲", "貪狼", "天梁", "文曲"],
        ["太陽", "武曲", "太陰", "天同"],
        ["巨門", "太陽", "文曲", "文昌"],
        ["天梁", "紫微", "天府", "武曲"],
        ["破軍", "巨門", "太陰", "貪狼"],
    ];
    let g = (year_gan % 10) as usize;
    match (table, g) {
        (SiHuaTable::Modern, 6) => ["太陽", "武曲", "天府", "天同"],
        (SiHuaTable::Modern, 8) | (SiHuaTable::KoreanMs, 8) => ["天梁", "紫微", "左輔", "武曲"],
        _ => q[g],
    }
}

/// 天魁、天钺（按生年干；全书：甲戊庚牛羊，乙己鼠猴鄉，六辛逢虎馬，壬癸兔蛇藏，丙丁猪狗位）
pub fn kui_yue(year_gan: u8, o: &ZiweiOptions) -> (u8, u8) {
    match year_gan % 10 {
        0 | 4 | 6 => (1, 7),
        1 | 5 => (0, 8),
        2 | 3 => (11, if o.bing_ding_yue == BingDingYue::You { 9 } else { 10 }),
        7 => match o.xin_kui_yue {
            XinKuiYue::KuiWuYueYin => (6, 2),
            XinKuiYue::KuiYinYueWu => (2, 6),
        },
        _ => (3, 5),
    }
}

/// 禄存（按生年干：甲寅、乙卯、丙戊巳、丁己午、庚申、辛酉、壬亥、癸子）
pub fn lu_cun(year_gan: u8) -> u8 {
    [2, 3, 5, 6, 5, 6, 8, 9, 11, 0][(year_gan % 10) as usize]
}

/// 三合局首（寅午戌→寅 等）的分组序：0 寅午戌、1 申子辰、2 巳酉丑、3 亥卯未
fn san_he_group(zhi: u8) -> usize {
    match zhi % 4 {
        2 => 0, // 寅午戌
        0 => 1, // 申子辰
        1 => 2, // 巳酉丑
        _ => 3, // 亥卯未
    }
}

/// 起局并安诸星（补充选项取缺省）
pub fn chart(inp: &ZiweiInput, o: &ZiweiOptions) -> Chart {
    chart_with(inp, o, &ZiweiExtraOptions::default())
}

/// 起局并安诸星
pub fn chart_with(inp: &ZiweiInput, o: &ZiweiOptions, x: &ZiweiExtraOptions) -> Chart {
    let yg = inp.year_gz % 10;
    let yz = inp.year_gz % 12;
    let month = if inp.is_leap_month {
        match o.leap {
            LeapRule::NextMonth => inp.lunar_month % 12 + 1,
            LeapRule::SameMonth => inp.lunar_month,
            LeapRule::SplitAt15 => {
                if inp.lunar_day <= 15 {
                    inp.lunar_month
                } else {
                    inp.lunar_month % 12 + 1
                }
            }
        }
    } else {
        inp.lunar_month
    };
    let h = inp.hour_zhi % 12;
    let (ming, shen) = ming_shen(month, h);
    let yin = yin_gan(yg);
    let mut palace_gan = [0u8; 12];
    for z in 0..12u8 {
        palace_gan[z as usize] = ((yin as i32 + (z as i32 - 2).rem_euclid(12)) % 10) as u8;
    }
    let ming_gz = gz_of(palace_gan[ming as usize], ming);
    let (ju, ju_name) = wu_xing_ju(ming_gz);
    let mut palaces = [""; 12];
    for (k, name) in PALACES.iter().enumerate() {
        palaces[m12(ming as i32 - k as i32) as usize] = name;
    }

    let zw = zi_wei_pos(ju, inp.lunar_day);
    let mut stars = main_stars(zw);
    let mi = month as i32 - 1;
    let hi = h as i32;
    let (kui, yue) = kui_yue(yg, o);
    let lc = lu_cun(yg);
    let grp = san_he_group(yz);
    // 天马：寅午戌申、申子辰寅、巳酉丑亥、亥卯未巳
    let ma = [8u8, 2, 11, 5][grp];
    // 火星、铃星：寅午戌丑卯、申子辰寅戌、巳酉丑卯戌、亥卯未酉戌，子时起顺数至生时
    let (huo0, ling0) = match (grp, x.huo_ling) {
        (2, SiYouChouHuoLing::KoreanMs) => (10, 3),
        _ => [(1, 3), (2, 10), (3, 10), (9, 10)][grp],
    };
    let zuo = m12(4 + mi);
    let you = m12(10 - mi);
    let d = inp.lunar_day as i32 - 1;
    let yzi = yz as i32;
    let aux: [(&'static str, u8); 27] = [
        ("文昌", m12(10 - hi)),
        ("文曲", m12(4 + hi)),
        ("左輔", zuo),
        ("右弼", you),
        ("天魁", kui),
        ("天鉞", yue),
        ("祿存", lc),
        ("擎羊", m12(lc as i32 + 1)),
        ("陀羅", m12(lc as i32 - 1)),
        ("天馬", ma),
        ("火星", m12(huo0 + hi)),
        ("鈴星", m12(ling0 + hi)),
        ("地劫", m12(11 + hi)),
        ("天空", m12(11 - hi)),
        ("天刑", m12(9 + mi)),
        ("天姚", m12(1 + mi)),
        ("紅鸞", m12(3 - yzi)),
        ("天喜", m12(9 - yzi)),
        ("天哭", m12(6 - yzi)),
        ("天虛", m12(6 + yzi)),
        ("龍池", m12(4 + yzi)),
        ("鳳閣", m12(10 - yzi)),
        ("台輔", m12(6 + hi)),
        ("封誥", m12(2 + hi)),
        ("三台", m12(zuo as i32 + d)),
        ("八座", m12(you as i32 - d)),
        // 天伤在奴仆宫、天使在疾厄宫（全书「命前六位是天傷，命後六位天使當」，注「天傷安在奴僕宮，天使安在疾厄宮」）
        ("天傷", m12(ming as i32 - 7)),
    ];
    stars.extend(aux.iter().map(|(n, z)| Star { name: n, zhi: *z }));
    stars.push(Star { name: "天使", zhi: m12(ming as i32 - 5) });
    // 截路空亡（论本生年干）：甲己申酉、乙庚午未、丙辛辰巳、丁壬寅卯、戊癸子丑（全书卷二「安截路空亡訣」；写本第 10 页同）
    let jie = [8u8, 6, 4, 2, 0][(yg % 5) as usize];
    stars.push(Star { name: "截路", zhi: jie });
    stars.push(Star { name: "截路", zhi: jie + 1 });
    // 旬中空亡（论本生年）：生年所在之旬所空二支（全书「安旬中空亡訣」：甲子旬中空戌亥……）
    let xun = (inp.year_gz % 60) / 10; // 0 甲子旬 … 5 甲寅旬
    let k0 = m12(10 - 2 * xun as i32);
    stars.push(Star { name: "旬空", zhi: k0 });
    stars.push(Star { name: "旬空", zhi: m12(k0 as i32 + 1) });
    // 解神：从戌上起子，逆数至当生年太岁（全书「安天德月德解神訣」）
    stars.push(Star { name: "解神", zhi: m12(10 - yzi) });
    if x.xie_ben_xing {
        stars.extend(xie_ben_stars(inp.year_gz));
    }

    let sh = si_hua(yg, o.si_hua);
    let si_hua = vec![("化祿", sh[0]), ("化權", sh[1]), ("化科", sh[2]), ("化忌", sh[3])];

    let yang_year = yg % 2 == 0;
    // 长生十二神：火局寅、木局亥、水局申、金局巳、土局申起长生
    let cs0: i32 = match ju {
        6 => 2,
        3 => 11,
        2 | 5 => 8,
        _ => 5,
    };
    let cs_fwd = match o.chang_sheng {
        ChangShengDir::MaleForward => inp.male,
        ChangShengDir::YangMaleYinFemaleForward => inp.male == yang_year,
    };
    const CS: [&str; 12] = ["長生", "沐浴", "冠帶", "臨官", "帝旺", "衰", "病", "死", "墓", "絕", "胎", "養"];
    let mut chang_sheng = [""; 12];
    for (k, n) in CS.iter().enumerate() {
        let z = if cs_fwd { cs0 + k as i32 } else { cs0 - k as i32 };
        chang_sheng[m12(z) as usize] = n;
    }
    // 博士十二神：自禄存起，阳男阴女顺、阴男阳女逆（全书「安十二宮太歲殺祿訣」）
    const BS: [&str; 12] = ["博士", "力士", "青龍", "小耗", "將軍", "奏書", "飛廉", "喜神", "病符", "大耗", "伏兵", "官府"];
    let fwd = inp.male == yang_year;
    let mut bo_shi = [""; 12];
    for (k, n) in BS.iter().enumerate() {
        let z = if fwd { lc as i32 + k as i32 } else { lc as i32 - k as i32 };
        bo_shi[m12(z) as usize] = n;
    }
    // 大限：自命宫起，局数为起岁，每限十年；阳男阴女顺行（次限在命前一宫，即父母宫）、阴男阳女逆行（次限在兄弟宫）
    let da_xian = (0..12)
        .map(|k| {
            let z = if fwd { ming as i32 + k } else { ming as i32 - k };
            let a = ju + 10 * k as u8;
            (a, a + 9, m12(z))
        })
        .collect();
    // 小限：寅午戌辰、申子辰戌、巳酉丑未、亥卯未丑起一岁；男顺女逆
    let xiao_xian_start = [4u8, 10, 7, 1][grp];
    // 命主（按命宫支）、身主（按生年支）
    let ming_zhu = ["貪狼", "巨門", "祿存", "文曲", "廉貞", "武曲", "破軍", "武曲", "廉貞", "文曲", "祿存", "巨門"][ming as usize];
    let shen_zhu = match x.shen_zhu {
        ShenZhuTable::QuanShu => ["鈴星", "天相", "天梁", "天同", "文昌", "天機"],
        ShenZhuTable::KoreanMs => ["鈴星", "天相", "天梁", "天同", "文曲", "天機"],
        ShenZhuTable::HuoXing => ["火星", "天相", "天梁", "天同", "文昌", "天機"],
    }[(yz % 6) as usize];

    Chart {
        month_used: month,
        ming,
        shen,
        ming_gz,
        ju,
        ju_name,
        palaces,
        palace_gan,
        stars,
        si_hua,
        chang_sheng,
        bo_shi,
        da_xian,
        xiao_xian_start,
        xiao_xian_forward: inp.male,
        ming_zhu,
        shen_zhu,
    }
}

/// 流年诸星
#[derive(Clone, Debug)]
pub struct LiuNian {
    /// 流年干支序
    pub year_gz: u8,
    /// 斗君所在之宫（地支序）
    pub dou_jun: u8,
    pub stars: Vec<Star>,
}

impl LiuNian {
    pub fn star(&self, name: &str) -> Option<u8> {
        self.stars.iter().find(|s| s.name == name).map(|s| s.zhi)
    }
}

/// 岁前十二神（流年太岁起顺布），名目从韩国藏写本第 10 页「太歲」条：太歲 太陽 喪門 太陰 官符 死符 歲破 龍德 白虎 福德 吊客 病符。
/// 全书「安喪門白虎弔客官府四飛星訣」只载其四（岁前二丧门、后二吊客、丧门对宫白虎、吊客对宫官府——全书作「官府」，写本作「官符」），
/// 位置与写本逐一相合。（「太陽」「太陰」与主星同名；流年诸星另存于 [`LiuNian`]，不与命盘主星相混。）
pub const SUI_QIAN_12: [&str; 12] = ["太歲", "太陽", "喪門", "太陰", "官符", "死符", "歲破", "龍德", "白虎", "福德", "弔客", "病符"];

/// 某流年（干支序）的流年诸星。依据：
/// - 流禄、流羊、流陀：与禄存、擎羊、陀罗同法，按流年干（全书「安流祿流羊流陀訣」：「假如己丑流年流禄在午、流羊在未、流陀在巳」）。
/// - 丧门、吊客、白虎、官府：流年太岁前二、后二及其对宫（全书「安喪門白虎弔客官府四飛星訣」）；其余八神名目从写本。
/// - 斗君：「於流年太歲宮起正月，逆至本生月，又從本生月起子順數至本生時」（全书「安斗君訣」；写本第 10 页同）。
/// - 天德、月德：「天德星從酉上起子，順數至流年太歲；月德星從子上起子，順數至流年太歲」（全书录文如此，月德即与流年太岁同宫，录文或有脱误，未核）。
/// - 飞天三杀（奏书、将军、直符）：「寅午戌年飛入亥卯未宮，申子辰年飛入巳酉丑宮，亥卯未年飛入申子辰宮，巳酉丑年飛入寅午戌宮」；
///   三星与三宫之对应原文未分，此按所列字序（奏书居三合首宫），**未核**。
/// - 流昌、流曲：全书无安法；从韩国藏写本第 11 页：流昌「甲蛇乙馬報君知，丙戊猴宮丁己雞，庚豬辛鼠壬逢虎，癸人見兔步雲梯」，
///   流曲「甲年酉宮，乙年申，丙戊年午，丁己年巳，庚年卯宮，辛年寅，壬年子，癸年亥」。
pub fn liu_nian(inp: &ZiweiInput, c: &Chart, liu_year_gz: u8) -> LiuNian {
    let lg = liu_year_gz % 10;
    let lz = (liu_year_gz % 12) as i32;
    let lc = lu_cun(lg);
    let mut stars = vec![
        Star { name: "流祿", zhi: lc },
        Star { name: "流羊", zhi: m12(lc as i32 + 1) },
        Star { name: "流陀", zhi: m12(lc as i32 - 1) },
        Star { name: "流昌", zhi: [5u8, 6, 8, 9, 8, 9, 11, 0, 2, 3][lg as usize] },
        Star { name: "流曲", zhi: [9u8, 8, 6, 5, 6, 5, 3, 2, 0, 11][lg as usize] },
        Star { name: "天德", zhi: m12(9 + lz) },
        Star { name: "月德", zhi: m12(lz) },
    ];
    for (k, n) in SUI_QIAN_12.iter().enumerate() {
        stars.push(Star { name: n, zhi: m12(lz + k as i32) });
    }
    // 飞天三杀：流年三合局 → 所飞入之三合局（寅午戌→亥卯未、申子辰→巳酉丑、亥卯未→申子辰、巳酉丑→寅午戌），按 首、中、尾 配奏书、将军、直符
    let target = [[11u8, 3, 7], [5, 9, 1], [2, 6, 10], [8, 0, 4]][san_he_group(lz as u8)];
    for (n, z) in ["奏書", "將軍", "直符"].iter().zip(target) {
        stars.push(Star { name: n, zhi: z });
    }
    let dou_jun = m12(lz - (c.month_used as i32 - 1) + (inp.hour_zhi % 12) as i32);
    LiuNian { year_gz: liu_year_gz % 60, dou_jun, stars }
}

/// 星曜庙旺七等（全书卷二诸星庙旺表）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiaoWang {
    Miao,
    Wang,
    DeDi,
    LiYi,
    PingHe,
    BuDeDi,
    LuoXian,
}

impl MiaoWang {
    pub fn name(self) -> &'static str {
        ["廟", "旺", "得地", "利益", "平和", "不得地", "落陷"][self as usize]
    }
}

/// 全书卷二庙旺表（录文表格逐格转录）：每支七格，依次庙、旺、得地、利益、平和、不得地、落陷；
/// 星名简称：紫 機 日（太陽）武 同 廉 府 月（太陰）貪 巨 相 梁 殺 破 昌 曲 祿（祿存）羊 陀 火 鈴
const MIAO_WANG_TABLE: [[&str; 7]; 12] = [
    ["機府月相梁破祿", "武同貪巨殺", "昌曲", "", "紫廉", "", "日羊火鈴"],
    ["紫武府月貪相殺昌曲羊陀", "梁破", "火鈴", "廉", "", "日同巨", "機"],
    ["廉府巨相梁殺祿火鈴", "紫日月", "機武破", "同", "貪曲", "", "昌陀"],
    ["日巨梁祿", "紫機殺曲", "府", "武貪昌火鈴", "同廉", "", "月相破羊"],
    ["武府貪梁殺羊陀", "日破", "紫相昌曲", "機廉", "同", "", "月巨火鈴"],
    ["同昌曲祿", "紫日巨", "府相火鈴", "", "機武殺破", "", "廉月貪梁陀"],
    ["紫機相梁破祿火鈴", "日武府貪巨殺", "", "", "廉", "月", "同昌曲羊"],
    ["紫武府貪殺羊陀", "梁破曲", "日相", "廉昌火鈴", "", "同月巨", "機"],
    ["廉巨相殺祿", "紫同", "機日武府破昌曲", "月", "貪", "", "梁陀火鈴"],
    ["巨昌曲祿", "紫機府月殺", "梁火鈴", "武貪", "日同廉", "", "相破羊"],
    ["武府貪梁殺羊陀火鈴", "月破", "紫相", "機廉", "同", "日", "巨昌曲"],
    ["同月祿", "紫巨曲", "府相", "昌火鈴", "機武殺破", "", "日廉貪梁陀"],
];

fn star_abbr(name: &str) -> Option<char> {
    Some(match name {
        "紫微" => '紫',
        "天機" => '機',
        "太陽" => '日',
        "武曲" => '武',
        "天同" => '同',
        "廉貞" => '廉',
        "天府" => '府',
        "太陰" => '月',
        "貪狼" => '貪',
        "巨門" => '巨',
        "天相" => '相',
        "天梁" => '梁',
        "七殺" => '殺',
        "破軍" => '破',
        "文昌" => '昌',
        "文曲" => '曲',
        "祿存" => '祿',
        "擎羊" => '羊',
        "陀羅" => '陀',
        "火星" => '火',
        "鈴星" => '鈴',
        _ => return None,
    })
}

/// 某星在某宫（地支序）之庙旺（全书卷二表）；表所不载之星或位为 None
pub fn miao_wang(star: &str, zhi: u8) -> Option<MiaoWang> {
    const ALL: [MiaoWang; 7] =
        [MiaoWang::Miao, MiaoWang::Wang, MiaoWang::DeDi, MiaoWang::LiYi, MiaoWang::PingHe, MiaoWang::BuDeDi, MiaoWang::LuoXian];
    let c = star_abbr(star)?;
    MIAO_WANG_TABLE[(zhi % 12) as usize].iter().position(|cell| cell.contains(c)).map(|k| ALL[k])
}

impl Chart {
    /// 某星所在宫之庙旺（全书卷二表）
    pub fn brightness(&self, name: &str) -> Option<MiaoWang> {
        self.star(name).and_then(|z| miao_wang(name, z))
    }
}

/// 韩国藏写本《紫微斗數方書》第 9、11 页所载、全书所无之星（按生年干支）。逐字依写本：
/// - 第 9 页：天官「甲人未，乙人辰，丙人巳，丁人寅，戊人卯，己辛人酉，庚人亥，壬人戌，癸人午」；
///   天福「甲人酉，乙人申，丙人子，丁人亥，戊人卯，己人寅，庚壬人午，辛癸人巳」；
///   華蓋「申子辰居辰，寅午戌居戌，亥卯未在未，巳酉丑在丑」；桃花殺「寅午戌兔，申子辰鷄，亥卯未鼠，巳酉丑馬」；
///   孤辰寡宿「寅卯辰人安巳丑，巳午未人怕申辰，申酉戌人居亥未，亥子丑人寅戌嗔」（先孤后寡）；
///   破碎殺「子午卯酉蛇，寅申巳亥鷄，辰戌丑未牛」；劫殺（与天马同条）「申子辰生……劫巳，寅午戌生……劫亥，巳酉丑……劫寅，
///   亥卯未生……劫申」；大耗殺「鼠忌羊，牛嗔馬，虎嫌鷄，兔恐猴，龍憎猪，蛇驚犬，互見」。
/// - 第 11 页：天廚「甲丙生巳宮，乙丁生午，戊生申，己生酉，庚生亥，辛生子宮，壬生寅，癸生卯」；
///   太極貴人「甲年子宮，乙年午，丙年卯，丁年酉，戊年巳，己年午宮，庚年寅，辛年亥，壬年亥，癸年申」；
///   科名（「星即貴」）「甲乙寅，丙丁亥，戊己巳，庚辛申，壬癸巳」；文星貴「甲未，乙辰，丙巳，丁寅，戊丑，己戌，庚午，辛申，壬癸卯」；
///   山河節度「甲戊丙寅，乙己丁未，庚壬亥，辛癸丑」；福星貴「甲丙寅子，乙癸丑，丁亥，戊申，己未，庚午，辛巳，壬辰」
///   （「甲丙寅子」读作甲、丙俱寅子二宫，与《三命通會》卷八「甲丙相邀入虎鄉」、卷五「甲人見丙寅丙子」合；读作甲寅丙子亦通，**未核**）。
/// 写本页中另有流殺、祿倒、馬倒诸条，非本命之星，未取。
pub fn xie_ben_stars(year_gz: u8) -> Vec<Star> {
    let g = (year_gz % 10) as usize;
    let z = year_gz % 12;
    let grp = san_he_group(z); // 0 寅午戌 1 申子辰 2 巳酉丑 3 亥卯未
    let mut v = vec![
        Star { name: "天官", zhi: [7u8, 4, 5, 2, 3, 9, 11, 9, 10, 6][g] },
        Star { name: "天福", zhi: [9u8, 8, 0, 11, 3, 2, 6, 5, 6, 5][g] },
        Star { name: "華蓋", zhi: [10u8, 4, 1, 7][grp] },
        Star { name: "桃花殺", zhi: [3u8, 9, 6, 0][grp] },
        Star { name: "劫殺", zhi: [11u8, 5, 2, 8][grp] },
        Star { name: "破碎", zhi: [5u8, 9, 1][[0usize, 2, 1, 0, 2, 1][z as usize % 6]] },
        Star { name: "大耗", zhi: [7u8, 6, 9, 8, 11, 10, 1, 0, 3, 2, 5, 4][z as usize] },
        Star { name: "天廚", zhi: [5u8, 6, 5, 6, 8, 9, 11, 0, 2, 3][g] },
        Star { name: "太極貴人", zhi: [0u8, 6, 3, 9, 5, 6, 2, 11, 11, 8][g] },
        Star { name: "科名", zhi: [2u8, 2, 11, 11, 5, 5, 8, 8, 5, 5][g] },
        Star { name: "文星貴", zhi: [7u8, 4, 5, 2, 1, 10, 6, 8, 3, 3][g] },
        Star { name: "山河節度", zhi: [2u8, 7, 2, 7, 2, 7, 11, 1, 11, 1][g] },
    ];
    // 孤辰寡宿：寅卯辰（方）巳丑、巳午未申辰、申酉戌亥未、亥子丑寅戌
    let fang = ((z as usize + 10) % 12) / 3; // 寅卯辰 0、巳午未 1、申酉戌 2、亥子丑 3
    let (gu, gua) = [(5u8, 1u8), (8, 4), (11, 7), (2, 10)][fang];
    v.push(Star { name: "孤辰", zhi: gu });
    v.push(Star { name: "寡宿", zhi: gua });
    match g {
        0 | 2 => {
            v.push(Star { name: "福星貴", zhi: 2 });
            v.push(Star { name: "福星貴", zhi: 0 });
        }
        _ => v.push(Star { name: "福星貴", zhi: [0u8, 1, 0, 11, 8, 7, 6, 5, 4, 1][g] }),
    }
    v
}

/// 某宫之三方四正：本宫、对宫、两三合宫
pub fn san_fang_si_zheng(zhi: u8) -> [u8; 4] {
    let z = zhi as i32;
    [m12(z), m12(z + 6), m12(z + 4), m12(z + 8)]
}

/// 小限：某虚岁所在之宫（一岁为 `xiao_xian_start`，一年一宫）
pub fn xiao_xian_at(c: &Chart, age: u32) -> u8 {
    let k = (age.max(1) - 1) as i32;
    m12(c.xiao_xian_start as i32 + if c.xiao_xian_forward { k } else { -k })
}

/// 大限：某虚岁所在之宫（未及起限之岁为 None）
pub fn da_xian_at(c: &Chart, age: u32) -> Option<u8> {
    c.da_xian.iter().find(|(a, b, _)| (*a as u32..=*b as u32).contains(&age)).map(|x| x.2)
}

/// 流年限上之格（只取原书有明文、可据星位判定者）：
/// - 「羊陀迭並」（全书卷三「論羊陀迭並」）：命宫三方四正原有擎羊或陀羅，流年又遇流羊或流陀于命宫三方四正。
///   书例「庚年生人，命在卯宮，遷移在酉宮……流年亦庚祿居申，流羊在酉，流陀在未」。
/// - 「七殺重逢」（全书卷三「論七殺重逢」）：「命中三合原有七殺守照，而流年又遇流羊流陀沖照」——七杀与流羊或流陀俱在命宫三方四正。
/// - 「小限羊陀重逢」「重逢七殺」（写本第 11—12 页）：「如甲子生人安命在巳，小限又行在亥，擎羊又在卯宮，却是三方四正俱見羊陀，
///   又見重逢七殺，正遭毒禍」——小限宫三方四正兼见本命羊陀与流羊流陀为重逢，又见七杀则「正遭毒禍」。
/// - 「竹羅三限」（写本第 12 页「定竹羅三限」）：「大小二限三方四正遇七殺破軍貪狼三星并巨暗及流年太歲諸凶殺曜交會」——
///   此只判三星俱见（三星恒相三合，见一即见三），巨门与流年凶曜另列于判语之后。
/// 「三方四正」之「冲合」取本宫、对宫、三合（[`san_fang_si_zheng`]）。诸格判定方式为本项目之读法，**未核**。
pub fn liu_nian_ge(c: &Chart, liu: &LiuNian, age: u32) -> Vec<String> {
    let has = |zs: &[u8; 4], names: &[&str], liu_side: bool| -> bool {
        names.iter().any(|n| {
            let z = if liu_side { liu.star(n) } else { c.star(n) };
            z.is_some_and(|z| zs.contains(&z))
        })
    };
    let mut out = Vec::new();
    let ming = san_fang_si_zheng(c.ming);
    let ben_yt = has(&ming, &["擎羊", "陀羅"], false);
    let liu_yt = has(&ming, &["流羊", "流陀"], true);
    if ben_yt && liu_yt {
        out.push("羊陀迭並".to_string());
    }
    if has(&ming, &["七殺"], false) && liu_yt {
        out.push("七殺重逢".to_string());
    }
    let xx = san_fang_si_zheng(xiao_xian_at(c, age));
    if has(&xx, &["擎羊", "陀羅"], false) && has(&xx, &["流羊", "流陀"], true) {
        out.push(if has(&xx, &["七殺"], false) { "小限羊陀重逢七殺".to_string() } else { "小限羊陀重逢".to_string() });
    }
    let xiong = ["流羊", "流陀", "喪門", "白虎", "弔客", "官符"];
    for (nm, zx) in [("大限", da_xian_at(c, age)), ("小限", Some(xiao_xian_at(c, age)))] {
        let Some(zx) = zx else { continue };
        let zs = san_fang_si_zheng(zx);
        if has(&zs, &["七殺"], false) && has(&zs, &["破軍"], false) && has(&zs, &["貪狼"], false) {
            let mut extra: Vec<&str> = Vec::new();
            if has(&zs, &["巨門"], false) {
                extra.push("巨門");
            }
            extra.extend(xiong.iter().copied().filter(|n| has(&zs, &[n], true)));
            out.push(if extra.is_empty() { format!("{nm}竹羅三限") } else { format!("{nm}竹羅三限（并{}）", extra.join("、")) });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(c: &str) -> u8 {
        ZHI.iter().position(|x| *x == c).unwrap() as u8
    }

    /// 全书卷二「安身命例」：正月子时寅宫安身命；丑时命丑身卯；寅时命子身辰
    #[test]
    fn ming_shen_examples() {
        assert_eq!(ming_shen(1, 0), (z("寅"), z("寅")));
        assert_eq!(ming_shen(1, 1), (z("丑"), z("卯")));
        assert_eq!(ming_shen(1, 2), (z("子"), z("辰")));
    }

    /// 全书卷二：甲生人安命在寅，起丙寅，丙寅爐中火，火局；正月初一生者酉宫起紫微
    #[test]
    fn jia_ming_yin_fire() {
        let g = gz_of(yin_gan(0), z("寅"));
        assert_eq!(GAN[(g % 10) as usize], "丙");
        assert_eq!(wu_xing_ju(g), (6, "火六局"));
        assert_eq!(zi_wei_pos(6, 1), z("酉"));
    }

    /// 全书卷二五局安紫微图（解析录文所得，150 日中 147 日；录文讹脱三日除外，见文档）
    #[test]
    fn zi_wei_grids_quanshu() {
        // 每局 30 日紫微所在之支（初一至三十），由卷二图逐格读出
        let grids: [(u8, &str); 5] = [
            (2, "丑寅寅卯卯辰辰巳巳午午未未申申酉酉戌戌亥亥子子丑丑寅寅卯卯辰"),
            (3, "辰丑寅巳？卯午卯？未辰巳申巳午酉午未戌未申亥申酉子酉戌丑戌亥"),
            (4, "亥辰丑寅子巳寅卯丑午卯辰寅未辰巳卯申巳午辰酉午未巳戌未申午？"),
            (5, "午亥辰丑寅未子巳寅卯申丑午卯辰酉寅未辰巳戌卯申巳午亥辰酉午未"),
            (6, "酉午亥辰丑寅戌未子巳寅卯亥申丑午卯辰子酉寅未辰巳丑戌卯申巳午"),
        ];
        for (ju, s) in grids {
            for (i, c) in s.chars().enumerate() {
                if c == '？' {
                    continue; // 录文讹脱：木三局初五（寅格作「初九」，致初九两见）、金四局三十（亥格脱）
                }
                assert_eq!(ZHI[zi_wei_pos(ju, i as u8 + 1) as usize], c.to_string(), "{ju} 局 {} 日", i + 1);
            }
        }
    }

    /// 全书「安天府圖」：寅申紫府同宫，紫居丑则府居卯
    #[test]
    fn tian_fu() {
        assert_eq!(tian_fu_pos(z("寅")), z("寅"));
        assert_eq!(tian_fu_pos(z("申")), z("申"));
        assert_eq!(tian_fu_pos(z("丑")), z("卯"));
    }

    /// 全书卷二各主星条「某宫与某星同度」四十一条，紫微十二位遍历皆成立
    #[test]
    fn tong_du_statements() {
        let st: [(&str, &str, &str); 41] = [
            ("紫微", "卯酉", "貪狼"), ("紫微", "寅申", "天府"), ("紫微", "巳亥", "七殺"), ("紫微", "辰戌", "天相"),
            ("紫微", "丑未", "破軍"), ("天機", "卯酉", "巨門"), ("天機", "寅申", "太陰"), ("天機", "辰戌", "天梁"),
            ("太陽", "寅申", "巨門"), ("太陽", "丑未", "太陰"), ("武曲", "子午", "天府"), ("武曲", "卯酉", "七殺"),
            ("武曲", "寅申", "天相"), ("武曲", "巳亥", "破軍"), ("武曲", "丑未", "貪狼"), ("天同", "寅申", "天梁"),
            ("天同", "丑未", "巨門"), ("廉貞", "子午", "天相"), ("廉貞", "卯酉", "破軍"), ("廉貞", "辰戌", "天府"),
            ("廉貞", "丑未", "七殺"), ("天府", "子午", "武曲"), ("天府", "寅申", "紫微"), ("天府", "辰戌", "廉貞"),
            ("貪狼", "卯酉", "紫微"), ("貪狼", "巳亥", "廉貞"), ("貪狼", "丑未", "武曲"), ("巨門", "卯酉", "天機"),
            ("巨門", "寅申", "太陽"), ("天相", "子午", "廉貞"), ("天相", "辰戌", "紫微"), ("天相", "寅申", "武曲"),
            ("天梁", "卯酉", "太陽"), ("天梁", "辰戌", "天機"), ("天梁", "寅申", "天同"), ("七殺", "卯酉", "武曲"),
            ("七殺", "丑未", "廉貞"), ("七殺", "巳亥", "紫微"), ("破軍", "卯酉", "廉貞"), ("破軍", "丑未", "紫微"),
            ("破軍", "巳亥", "武曲"),
        ];
        for zw in 0..12u8 {
            let ms = main_stars(zw);
            let at = |n: &str| ms.iter().find(|s| s.name == n).unwrap().zhi;
            for (a, brs, b) in st {
                for c in brs.chars() {
                    if ZHI[at(a) as usize] == c.to_string() {
                        assert_eq!(at(a), at(b), "紫微在{}：{a}在{c}应与{b}同度", ZHI[zw as usize]);
                    }
                }
            }
        }
    }

    /// 全书卷二诸诀所举之例
    #[test]
    fn aux_examples() {
        // 甲生人廉貞化祿、破軍化權、武曲化科、太陽化忌
        assert_eq!(si_hua(0, SiHuaTable::QuanShu), ["廉貞", "破軍", "武曲", "太陽"]);
        // 癸禄在子，丑宫安擎羊，亥宫安陀罗
        assert_eq!(lu_cun(9), z("子"));
        // 己丑流年流禄在午（流禄与禄存同法）
        assert_eq!(lu_cun(5), z("午"));
        let inp = |h: u8| ZiweiInput { year_gz: 9, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: h, male: true };
        let o = ZiweiOptions::default();
        let c = chart(&inp(0), &o);
        assert_eq!(c.star("擎羊"), Some(z("丑")));
        assert_eq!(c.star("陀羅"), Some(z("亥")));
        // 子时劫空俱在亥；丑时劫子空戌；午时劫空俱在巳
        assert_eq!((c.star("地劫"), c.star("天空")), (Some(z("亥")), Some(z("亥"))));
        let c1 = chart(&inp(1), &o);
        assert_eq!((c1.star("地劫"), c1.star("天空")), (Some(z("子")), Some(z("戌"))));
        let c6 = chart(&inp(6), &o);
        assert_eq!((c6.star("地劫"), c6.star("天空")), (Some(z("巳")), Some(z("巳"))));
        // 文昌子时戌、丑时酉；文曲子时辰、丑时巳；左辅正月辰、二月巳；右弼正月戌、二月酉
        assert_eq!((c.star("文昌"), c.star("文曲")), (Some(z("戌")), Some(z("辰"))));
        assert_eq!((c1.star("文昌"), c1.star("文曲")), (Some(z("酉")), Some(z("巳"))));
        assert_eq!((c.star("左輔"), c.star("右弼")), (Some(z("辰")), Some(z("戌"))));
        // 天伤在奴仆宫、天使在疾厄宫
        assert_eq!(c.palaces[c.star("天傷").unwrap() as usize], "奴僕");
        assert_eq!(c.palaces[c.star("天使").unwrap() as usize], "疾厄");
        // 午宫安命命主破军，子宫安命命主贪狼
        let mut x = inp(0);
        x.lunar_month = 5; // 五月子时：寅起正月顺数至午，子时安命在午
        assert_eq!(chart(&x, &o).ming, z("午"));
        assert_eq!(chart(&x, &o).ming_zhu, "破軍");
    }

    /// 闰月：全书作下月；其余二说为选项
    #[test]
    fn leap_month_rules() {
        let inp = ZiweiInput { year_gz: 0, lunar_month: 1, is_leap_month: true, lunar_day: 20, hour_zhi: 0, male: true };
        let mut o = ZiweiOptions::default();
        assert_eq!(chart(&inp, &o).ming, z("卯")); // 作二月
        o.leap = LeapRule::SameMonth;
        assert_eq!(chart(&inp, &o).ming, z("寅"));
        o.leap = LeapRule::SplitAt15;
        assert_eq!(chart(&inp, &o).ming, z("卯"));
        let early = ZiweiInput { lunar_day: 10, ..inp };
        assert_eq!(chart(&early, &o).ming, z("寅"));
    }

    /// 大限、小限、十二宫
    #[test]
    fn limits_and_palaces() {
        // 甲子年（阳）男，正月初一子时：命寅，火六局，大限 6—15 在命宫，次限顺行至父母宫（卯）
        let inp = ZiweiInput { year_gz: 0, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true };
        let c = chart(&inp, &ZiweiOptions::default());
        assert_eq!(c.ju_name, "火六局");
        assert_eq!(c.da_xian[0], (6, 15, z("寅")));
        assert_eq!(c.da_xian[1].2, z("卯"));
        assert_eq!(c.palaces[z("卯") as usize], "父母");
        assert_eq!(c.palaces[z("丑") as usize], "兄弟");
        // 子年申子辰：小限一岁在戌，男顺
        assert_eq!(c.xiao_xian_start, z("戌"));
        // 女命大限逆行至兄弟宫
        let f = chart(&ZiweiInput { male: false, ..inp }, &ZiweiOptions::default());
        assert_eq!(f.da_xian[1].2, z("丑"));
    }

    /// 全书卷二庙旺表：每支十四主星与昌曲火铃各恰一见；禄存、擎羊、陀罗只见于其可落之支
    #[test]
    fn miao_wang_table_complete() {
        let mains = ["紫微", "天機", "太陽", "武曲", "天同", "廉貞", "天府", "太陰", "貪狼", "巨門", "天相", "天梁", "七殺", "破軍", "文昌", "文曲", "火星", "鈴星"];
        for zh in 0..12u8 {
            for n in mains {
                let c = star_abbr(n).unwrap();
                let k = MIAO_WANG_TABLE[zh as usize].iter().filter(|cell| cell.contains(c)).count();
                assert_eq!(k, 1, "{n} 在{}", ZHI[zh as usize]);
            }
            // 禄存只落 寅卯巳午申酉亥子；擎羊为禄前一位、陀罗为禄后一位
            let lu: Vec<u8> = (0..10).map(lu_cun).collect();
            let has = |c: char| MIAO_WANG_TABLE[zh as usize].iter().any(|cell| cell.contains(c));
            assert_eq!(has('祿'), lu.contains(&zh), "祿在{}", ZHI[zh as usize]);
            assert_eq!(has('羊'), lu.iter().any(|&l| m12(l as i32 + 1) == zh), "羊在{}", ZHI[zh as usize]);
            assert_eq!(has('陀'), lu.iter().any(|&l| m12(l as i32 - 1) == zh), "陀在{}", ZHI[zh as usize]);
        }
    }

    /// 全书卷二太阳条「子宮陷午宮旺……卯宮廟酉宮和平……寅宮旺申宮得地……巳宮旺……亥宮陷……辰宮旺……未宮得地」与表相合者
    #[test]
    fn miao_wang_tai_yang() {
        use MiaoWang::*;
        for (zh, g) in [("子", LuoXian), ("午", Wang), ("卯", Miao), ("酉", PingHe), ("寅", Wang), ("申", DeDi), ("巳", Wang), ("亥", LuoXian), ("辰", Wang), ("未", DeDi)] {
            assert_eq!(miao_wang("太陽", z(zh)), Some(g), "太陽在{zh}");
        }
        assert_eq!(miao_wang("左輔", 0), None);
    }

    /// 截路空亡、旬中空亡、解神（论本生年）
    #[test]
    fn birth_year_kong_wang() {
        let o = ZiweiOptions::default();
        let at = |c: &Chart, n: &str| -> Vec<u8> { c.stars.iter().filter(|s| s.name == n).map(|s| s.zhi).collect() };
        // 甲子年：截路申酉；旬空戌亥（甲子旬）；解神戌（戌上起子逆数至子）
        let c = chart(&ZiweiInput { year_gz: 0, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true }, &o);
        assert_eq!(at(&c, "截路"), vec![z("申"), z("酉")]);
        assert_eq!(at(&c, "旬空"), vec![z("戌"), z("亥")]);
        assert_eq!(c.star("解神"), Some(z("戌")));
        // 癸亥年（甲寅旬）：截路子丑；旬空子丑；解神亥（戌起子逆数至亥：戌子、酉丑……亥）
        let c = chart(&ZiweiInput { year_gz: 59, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true }, &o);
        assert_eq!(at(&c, "截路"), vec![z("子"), z("丑")]);
        assert_eq!(at(&c, "旬空"), vec![z("子"), z("丑")]);
        assert_eq!(c.star("解神"), Some(z("亥")));
        // 丙寅年（甲子旬）旬空戌亥；庚午年（甲子旬）亦戌亥；甲戌年（甲戌旬）申酉
        for (gz, k) in [(2u8, "戌"), (6, "戌"), (10, "申")] {
            let c = chart(&ZiweiInput { year_gz: gz, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true }, &o);
            assert_eq!(at(&c, "旬空")[0], z(k), "年 {gz}");
        }
    }

    /// 流年诸星：全书「己丑流年流禄在午、流羊在未、流陀在巳」；丧门吊客白虎官府之相对位置；斗君
    #[test]
    fn liu_nian_stars() {
        let inp = ZiweiInput { year_gz: 0, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true };
        let c = chart(&inp, &ZiweiOptions::default());
        let ln = liu_nian(&inp, &c, 25); // 己丑
        assert_eq!((ln.star("流祿"), ln.star("流羊"), ln.star("流陀")), (Some(z("午")), Some(z("未")), Some(z("巳"))));
        // 丑年：丧门卯（前二）、吊客亥（后二）、白虎酉（丧门对）、官符巳（吊客对）
        assert_eq!(ln.star("喪門"), Some(z("卯")));
        assert_eq!(ln.star("弔客"), Some(z("亥")));
        assert_eq!(ln.star("白虎"), Some(m12(ln.star("喪門").unwrap() as i32 + 6)));
        assert_eq!(ln.star("官符"), Some(m12(ln.star("弔客").unwrap() as i32 + 6)));
        // 斗君：丑年太岁宫起正月，正月生即在丑；子时生不移
        assert_eq!(ln.dou_jun, z("丑"));
        // 五月寅时生：丑逆四位至酉，酉起子顺至寅为亥
        let inp5 = ZiweiInput { lunar_month: 5, hour_zhi: 2, ..inp };
        assert_eq!(liu_nian(&inp5, &chart(&inp5, &ZiweiOptions::default()), 25).dou_jun, z("亥"));
        // 天德：酉起子顺至流年太岁（丑年在戌）；飞天三杀：巳酉丑年飞入寅午戌
        assert_eq!(ln.star("天德"), Some(z("戌")));
        assert_eq!((ln.star("奏書"), ln.star("將軍"), ln.star("直符")), (Some(z("寅")), Some(z("午")), Some(z("戌"))));
        // 写本流昌流曲：己年流昌酉、流曲巳；庚年流昌亥、流曲卯
        assert_eq!((ln.star("流昌"), ln.star("流曲")), (Some(z("酉")), Some(z("巳"))));
        let ln6 = liu_nian(&inp, &c, 6); // 庚午
        assert_eq!((ln6.star("流昌"), ln6.star("流曲")), (Some(z("亥")), Some(z("卯"))));
    }

    /// 写本四化、火铃、身主选项
    #[test]
    fn korean_ms_options() {
        assert_eq!(si_hua(6, SiHuaTable::KoreanMs), ["太陽", "武曲", "太陰", "天同"]);
        assert_eq!(si_hua(8, SiHuaTable::KoreanMs), ["天梁", "紫微", "左輔", "武曲"]);
        assert_eq!(si_hua(8, SiHuaTable::QuanShu), ["天梁", "紫微", "天府", "武曲"]);
        let o = ZiweiOptions::default();
        // 巳年（巳酉丑）子时：全书火卯铃戌，写本火戌铃卯
        let inp = ZiweiInput { year_gz: 5, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true };
        let c = chart(&inp, &o);
        assert_eq!((c.star("火星"), c.star("鈴星")), (Some(z("卯")), Some(z("戌"))));
        let k = chart_with(&inp, &o, &ZiweiExtraOptions { huo_ling: SiYouChouHuoLing::KoreanMs, ..Default::default() });
        assert_eq!((k.star("火星"), k.star("鈴星")), (Some(z("戌")), Some(z("卯"))));
        // 身主：子年全书（读作铃星）、火星说；辰年写本文曲
        let zi = ZiweiInput { year_gz: 0, ..inp };
        assert_eq!(chart(&zi, &o).shen_zhu, "鈴星");
        assert_eq!(chart_with(&zi, &o, &ZiweiExtraOptions { shen_zhu: ShenZhuTable::HuoXing, ..Default::default() }).shen_zhu, "火星");
        let chen = ZiweiInput { year_gz: 4, ..inp };
        assert_eq!(chart(&chen, &o).shen_zhu, "文昌");
        assert_eq!(chart_with(&chen, &o, &ZiweiExtraOptions { shen_zhu: ShenZhuTable::KoreanMs, ..Default::default() }).shen_zhu, "文曲");
    }


    /// 写本第 9、11 页诸星：逐条依写本（甲子、己丑、庚寅诸年）
    #[test]
    fn xie_ben_xing() {
        let at = |gz: u8, n: &str| -> Vec<u8> { xie_ben_stars(gz).iter().filter(|s| s.name == n).map(|s| s.zhi).collect() };
        // 甲子年：天官未、天福酉、華蓋辰、桃花酉、劫殺巳、破碎巳、大耗未、天廚巳、太極子、科名寅、文星貴未、山河寅、孤寅寡戌、福星寅子
        assert_eq!(at(0, "天官"), [z("未")]);
        assert_eq!(at(0, "天福"), [z("酉")]);
        assert_eq!(at(0, "華蓋"), [z("辰")]);
        assert_eq!(at(0, "桃花殺"), [z("酉")]);
        assert_eq!(at(0, "劫殺"), [z("巳")]);
        assert_eq!(at(0, "破碎"), [z("巳")]);
        assert_eq!(at(0, "大耗"), [z("未")]);
        assert_eq!(at(0, "天廚"), [z("巳")]);
        assert_eq!(at(0, "太極貴人"), [z("子")]);
        assert_eq!(at(0, "科名"), [z("寅")]);
        assert_eq!(at(0, "文星貴"), [z("未")]);
        assert_eq!(at(0, "山河節度"), [z("寅")]);
        assert_eq!((at(0, "孤辰"), at(0, "寡宿")), (vec![z("寅")], vec![z("戌")]));
        assert_eq!(at(0, "福星貴"), [z("寅"), z("子")]);
        // 己丑年（25）：破碎丑、大耗午、孤寅寡戌、華蓋丑、劫殺寅、桃花午
        assert_eq!(at(25, "破碎"), [z("丑")]);
        assert_eq!(at(25, "大耗"), [z("午")]);
        assert_eq!(at(25, "華蓋"), [z("丑")]);
        // 庚寅年（26）：破碎酉、孤巳寡丑、天官亥、福星午
        assert_eq!(at(26, "破碎"), [z("酉")]);
        assert_eq!((at(26, "孤辰"), at(26, "寡宿")), (vec![z("巳")], vec![z("丑")]));
        assert_eq!(at(26, "福星貴"), [z("午")]);
        // 缺省命盘不排写本诸星，选项排之
        let inp = ZiweiInput { year_gz: 0, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true };
        assert!(chart(&inp, &ZiweiOptions::default()).star("天官").is_none());
        let x = ZiweiExtraOptions { xie_ben_xing: true, ..Default::default() };
        assert_eq!(chart_with(&inp, &ZiweiOptions::default(), &x).star("天官"), Some(z("未")));
    }

    /// 流年限上之格：写本第 11 页甲子生人、小限亥、己丑流年例；全书卷三羊陀迭并庚年命卯例
    #[test]
    fn liu_nian_ge_examples() {
        // 甲子男：小限申子辰人自戌宫起、男顺；己丑年虚岁二十六，小限在亥（写本「小限又行在亥」）
        let o = ZiweiOptions::default();
        let mut found = None;
        for m in 1..=12u8 {
            for h in 0..12u8 {
                let inp = ZiweiInput { year_gz: 0, lunar_month: m, is_leap_month: false, lunar_day: 1, hour_zhi: h, male: true };
                let c = chart(&inp, &o);
                if c.ming == z("巳") {
                    found = Some((inp, c));
                }
            }
        }
        let (inp, c) = found.unwrap();
        assert_eq!(xiao_xian_at(&c, 26), z("亥"));
        assert_eq!(c.star("擎羊"), Some(z("卯")));
        let liu = liu_nian(&inp, &c, 25);
        assert_eq!((liu.star("流羊"), liu.star("流陀")), (Some(z("未")), Some(z("巳"))));
        let ge = liu_nian_ge(&c, &liu, 26);
        assert!(ge.iter().any(|g| g.starts_with("小限羊陀重逢")), "{ge:?}");
        // 全书例：庚年生、命卯，擎羊酉；庚年流羊酉、流陀未
        for m in 1..=12u8 {
            for h in 0..12u8 {
                let inp = ZiweiInput { year_gz: 6, lunar_month: m, is_leap_month: false, lunar_day: 1, hour_zhi: h, male: true };
                let c = chart(&inp, &o);
                if c.ming == z("卯") {
                    assert_eq!(c.star("擎羊"), Some(z("酉")));
                    let liu = liu_nian(&inp, &c, 6);
                    assert_eq!((liu.star("流羊"), liu.star("流陀")), (Some(z("酉")), Some(z("未"))));
                    assert!(liu_nian_ge(&c, &liu, 1).contains(&"羊陀迭並".to_string()));
                }
            }
        }
    }
}
