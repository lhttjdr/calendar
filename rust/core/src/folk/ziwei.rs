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
}

/// 长生十二神（及博士十二神以外者）的顺逆
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangShengDir {
    /// 全书卷二：「男命順數、女命逆數」（缺省）
    MaleForward,
    /// 阳男阴女顺、阴男阳女逆（近人通行，出处未核）
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
        (SiHuaTable::Modern, 8) => ["天梁", "紫微", "左輔", "武曲"],
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

/// 起局并安诸星
pub fn chart(inp: &ZiweiInput, o: &ZiweiOptions) -> Chart {
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
    let (huo0, ling0) = [(1, 3), (2, 10), (3, 10), (9, 10)][grp];
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
    let shen_zhu = ["火星", "天相", "天梁", "天同", "文昌", "天機"][(yz % 6) as usize];

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
}
