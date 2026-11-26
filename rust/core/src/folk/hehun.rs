//! 合婚（通书说，缺省不启用）：按男女生年月日时配命，不择日。
//!
//! 出处与逐格核对见 `doc/folk/jiaqu.md`「合婚」一节与 `data/folk/hehun_qili.csv`：
//! - **类编**：题（元）宋魯珍《類編曆法通書大全》卷七「男女合婚」（北大藏本 `02094406.cn` PDF 第 53—64 页），
//!   诸法出呂才《大義婚書》；**鳌头**：《鰲頭通書大全》卷三「男女合婚」（第 4 册第 4—7 页），与类编同源，
//!   只男命木之益財起月、诸「益（退）某家若干年」之数有异文。
//! - **协纪**：卷三十六「男女合婚大利月」述其法而逐条斥之，卷首奏议「男女合婚……應請刪去」，「今已奏准刪除」，
//!   官历（时宪书）不用。卷三十五附錄「男女九宫」引《三元經》男命上元起坎一，谓世俗「誤以上元為中元」；
//!   卷三十六指出鐵掃帚火局取死月、益財金命十月起、土命七月起、女命火木金早一月、死墓絕妨夫誤與男命同等
//!   「傳寫之譌」。本模块「协纪」选项即按协纪所述之法（命宫依《三元經》，五行月按长生十二位）推，不是协纪所用。
//! - **星命诸煞**：陰錯陽差（类编、《星學大成》卷三、《三命通會》卷三同十二日）、孤鸞（《星學大成》卷三、
//!   《三命通會》卷三八日；卷十二注四日）、曲脚、平頭（类编）。
//!
//! 分层：第 1 层为各法的纯函数（只依赖干支序数、纳音五行、月序）；第 2 层 [`he_hun`] 汇成一对男女的结果与通书判语；
//! 第 4 层为 [`HeHunOptions`]（缺省 `enabled = false`，从协纪删除之断；启用时按所选之书）。
//! 女命行嫁大利月复用 [`super::jiaqu::nv_ming_yue`]。
//!
//! 与子平模块（`folk::ziping`）的关系：子平神煞中也有孤辰寡宿（以年支查四柱诸支）、陰錯陽差、孤鸞，判法不同；
//! 本模块只收通书合婚之说，所有名字都在 `hehun::` 之下（`gu_chen_month`、`yin_cuo_yang_cha_ri` 等），不与子平共用符号。
//!
//! 约定：干支序 0 甲子 … 59 癸亥；支 0 子 … 11 亥；月序 0 正月（寅）… 11 十二月（丑），以节为界、年以立春为界
//! （类编第 62 页「論男女生月看節氣……交立春節以後生命並作次年正月」）。月集合以 `u16` 位掩码表示，第 k 位为月序 k。

use super::jiaqu::{nv_ming_yue, NvMingYue};
use super::nayin::{nayin_wuxing, WuXing};

/// 起例表（转录），测试逐格核对
pub const QILI_CSV: &str = include_str!("../../../../data/folk/hehun_qili.csv");

// ───────────────────────── 第 4 层：选项 ─────────────────────────

/// 依何书
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeHunShu {
    /// 《類編曆法通書大全》卷七（表照录，含协纪所斥之「傳寫之譌」）
    LeiBian,
    /// 《鰲頭通書大全》卷三（同类编，唯男命木益財八月起）
    AoTou,
    /// 协纪卷三十五、三十六所述之法（三元經起宫，五行月按长生十二位推）
    XieJi,
}

/// 孤鸞之日
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuLuanShuo {
    /// 八日：乙巳丁巳辛亥甲寅丙午戊午壬子戊申（《星學大成》卷三、《三命通會》卷三）
    BaRi,
    /// 四日：甲寅丁巳戊申辛亥（《三命通會》卷十二注「等日」）
    SiRi,
}

/// 合婚选项。缺省不启用：协纪卷首奏议「男女合婚……應請刪去」，卷三十六「今已奏准刪除」，官历不用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeHunOptions {
    pub enabled: bool,
    pub shu: HeHunShu,
    pub gu_luan: GuLuanShuo,
}

impl HeHunOptions {
    pub const DEFAULT: Self = Self { enabled: false, shu: HeHunShu::LeiBian, gu_luan: GuLuanShuo::BaRi };
    /// 启用并按某书
    pub const fn tongshu(shu: HeHunShu) -> Self {
        Self { enabled: true, shu, gu_luan: GuLuanShuo::BaRi }
    }
}

impl Default for HeHunOptions {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ───────────────────────── 输入 ─────────────────────────

/// 男、女
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Xing {
    Nan,
    Nv,
}

/// 三元（六十年一元，1864 年甲子为上元）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SanYuan {
    Shang,
    Zhong,
    Xia,
}

impl SanYuan {
    fn idx(self) -> usize {
        match self {
            SanYuan::Shang => 0,
            SanYuan::Zhong => 1,
            SanYuan::Xia => 2,
        }
    }
    pub fn name(self) -> &'static str {
        ["上元", "中元", "下元"][self.idx()]
    }
}

/// 立春为界之年（天文年号）所属三元：1864、1684……为上元甲子，1924 中元，1984 下元
pub fn san_yuan(year: i32) -> SanYuan {
    match (year - 1864).rem_euclid(180) / 60 {
        0 => SanYuan::Shang,
        1 => SanYuan::Zhong,
        _ => SanYuan::Xia,
    }
}

/// 一人之命：年干支序、三元、四柱干支序（年、月、日、时）。
/// 四柱由历法域取：`calendar::gan_zhi_options::ganzhi_from_jd_solar` 配 `folk::ganzhi_presets::zi_ping_ba_zi()`
/// （立春换年、十二节换月，与类编「看節氣」合）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeHunMing {
    pub yuan: SanYuan,
    /// 年、月、日、时干支序
    pub si_zhu: [u8; 4],
}

impl HeHunMing {
    /// 由干支年之公历（天文）年号与月、日、时干支：年干支 = (year + 56) mod 60
    pub fn new(year: i32, month_gz: u8, day_gz: u8, hour_gz: u8) -> Self {
        let year_gz = (year + 56).rem_euclid(60) as u8;
        Self { yuan: san_yuan(year), si_zhu: [year_gz, month_gz % 60, day_gz % 60, hour_gz % 60] }
    }
    /// 只有四柱时须另给三元
    pub fn from_si_zhu(si_zhu: [u8; 4], yuan: SanYuan) -> Self {
        Self { yuan, si_zhu: si_zhu.map(|g| g % 60) }
    }
    pub fn year_gz(&self) -> u8 {
        self.si_zhu[0]
    }
    pub fn year_zhi(&self) -> u8 {
        self.si_zhu[0] % 12
    }
    /// 生月月序（0 正月寅 … 11 十二月丑），由月柱之支
    pub fn month_ord(&self) -> u8 {
        (self.si_zhu[1] % 12 + 10) % 12
    }
}

fn ord_of_zhi(zhi: u8) -> u8 {
    (zhi % 12 + 10) % 12
}

fn bit(ord: u8) -> u16 {
    1u16 << (ord % 12)
}

/// 自月序 start 起连续 n 月之掩码
fn run(start: u8, n: u8) -> u16 {
    (0..n).fold(0u16, |m, k| m | bit((start + k) % 12))
}

// ───────────────────────── 第 1 层：三元命宫与命卦 ─────────────────────────

/// 起宫（甲子年所在宫）：类编、鳌头男上元七、中元一、下元四；协纪引《三元經》男上元一、中元四、下元七。女三家同：上元五、中元二、下元八
pub fn qi_gong(xing: Xing, yuan: SanYuan, shu: HeHunShu) -> u8 {
    let i = yuan.idx();
    match (xing, shu) {
        (Xing::Nan, HeHunShu::XieJi) => [1, 4, 7][i],
        (Xing::Nan, _) => [7, 1, 4][i],
        (Xing::Nv, _) => [5, 2, 8][i],
    }
}

/// 三元命宫（1—9，五不寄）：男自起宫逆行、女顺行，至本命年干支
pub fn ming_gong(xing: Xing, yuan: SanYuan, year_gz: u8, shu: HeHunShu) -> u8 {
    let s = qi_gong(xing, yuan, shu) as i32 - 1;
    let k = (year_gz % 60) as i32;
    let g = match xing {
        Xing::Nan => (s - k).rem_euclid(9),
        Xing::Nv => (s + k).rem_euclid(9),
    };
    g as u8 + 1
}

/// 五宫寄：男（阳）寄坤二，女（阴）寄艮八（类编第 54 页「陰艮陽坤，左右相分」）
pub fn ji_gong(xing: Xing, gong: u8) -> u8 {
    match (gong, xing) {
        (5, Xing::Nan) => 2,
        (5, Xing::Nv) => 8,
        (g, _) => g,
    }
}

/// 宫之卦名（一坎二坤三震四巽六乾七兌八艮九離）
pub fn gong_gua_name(gong: u8) -> &'static str {
    ["", "坎", "坤", "震", "巽", "中", "乾", "兌", "艮", "離"][gong as usize % 10]
}

/// 宫之三爻（位 0 初爻、位 1 中爻、位 2 上爻，阳为 1）
fn gua_bits(gong: u8) -> u8 {
    match gong {
        1 => 0b010,
        2 => 0b000,
        3 => 0b001,
        4 => 0b110,
        6 => 0b111,
        7 => 0b011,
        8 => 0b100,
        9 => 0b101,
        _ => panic!("五宫须先寄"),
    }
}

/// 两宫合卦（小遊年变卦：协纪卷二「天父卦從乾翻起……上爻變為兌為貪狼」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuaPei {
    /// 上爻变（貪狼）
    ShengQi,
    /// 中上爻变（巨門）
    TianYi,
    /// 三爻皆变（祿存）
    JueTi,
    /// 初上爻变（文曲）
    YouHun,
    /// 初爻变（廉貞）
    WuGui,
    /// 初中爻变（武曲）
    FuDe,
    /// 中爻变（破軍）
    JueMing,
    /// 不变（輔弼，本宫）
    GuiHun,
}

/// 命卦等第（类编第 55—56 页引呂才）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuaDeng {
    /// 生氣、天醫、福德
    ShangJi,
    /// 絕體、遊魂、歸魂
    ZhongDeng,
    /// 五鬼（口舌）
    WuGuiXiong,
    /// 絕命（禍必深重）
    JueMingXiong,
}

impl GuaPei {
    pub fn name(self) -> &'static str {
        match self {
            GuaPei::ShengQi => "生氣",
            GuaPei::TianYi => "天醫",
            GuaPei::JueTi => "絕體",
            GuaPei::YouHun => "遊魂",
            GuaPei::WuGui => "五鬼",
            GuaPei::FuDe => "福德",
            GuaPei::JueMing => "絕命",
            GuaPei::GuiHun => "歸魂",
        }
    }
    pub fn deng(self) -> GuaDeng {
        match self {
            GuaPei::ShengQi | GuaPei::TianYi | GuaPei::FuDe => GuaDeng::ShangJi,
            GuaPei::JueTi | GuaPei::YouHun | GuaPei::GuiHun => GuaDeng::ZhongDeng,
            GuaPei::WuGui => GuaDeng::WuGuiXiong,
            GuaPei::JueMing => GuaDeng::JueMingXiong,
        }
    }
}

impl GuaDeng {
    pub fn name(self) -> &'static str {
        match self {
            GuaDeng::ShangJi => "上吉",
            GuaDeng::ZhongDeng => "中等",
            GuaDeng::WuGuiXiong => "五鬼",
            GuaDeng::JueMingXiong => "絕命",
        }
    }
}

/// 男宫、女宫（各已寄）合卦
pub fn gua_pei(nan_gong: u8, nv_gong: u8) -> GuaPei {
    match gua_bits(ji_gong(Xing::Nan, nan_gong)) ^ gua_bits(ji_gong(Xing::Nv, nv_gong)) {
        0b000 => GuaPei::GuiHun,
        0b100 => GuaPei::ShengQi,
        0b110 => GuaPei::TianYi,
        0b111 => GuaPei::JueTi,
        0b101 => GuaPei::YouHun,
        0b001 => GuaPei::WuGui,
        0b011 => GuaPei::FuDe,
        _ => GuaPei::JueMing,
    }
}

// ───────────────────────── 第 1 层：年月诸煞 ─────────────────────────

/// 孤辰（男忌）之月序：亥子丑年正月、寅卯辰四月、巳午未七月、申酉戌十月（方后之孟）
pub fn gu_chen_month(year_zhi: u8) -> u8 {
    let g = ((year_zhi % 12 + 1) % 12) / 3;
    ord_of_zhi((g * 3 + 2) % 12)
}

/// 寡宿（女忌）之月序：亥子丑年九月、寅卯辰十二月、巳午未三月、申酉戌六月（方前之季）
pub fn gua_su_month(year_zhi: u8) -> u8 {
    let g = ((year_zhi % 12 + 1) % 12) / 3;
    ord_of_zhi((g * 3 + 10) % 12)
}

/// 胞胎相衝（穿胎殺）：生年支前后各三辰之月（男年对女月、女年对男月）
pub fn bao_tai_mask(year_zhi: u8) -> u16 {
    bit(ord_of_zhi(year_zhi + 3)) | bit(ord_of_zhi(year_zhi + 9))
}

/// 骨髓破（男破女家，女破男家）：本人生年对本人生月，「或平或收」（类编第 57 页表）
pub fn gu_sui_po_month(year_zhi: u8) -> u8 {
    const T: [u8; 12] = [2, 3, 10, 5, 12, 1, 8, 9, 4, 11, 6, 7]; // 子…亥 → 月数
    T[year_zhi as usize % 12] - 1
}

/// 鐵掃帚（男掃女家、女掃男家）：本人生年三合局对本人生月。
/// 类编、鳌头：男 水局正、金局六、火局四、木局二月；女 水局十二、金局九、火局七、木局八月。
/// 协纪述其法「火局取臨官與死月」，女火局作八月（死在酉）。
pub fn tie_sao_zhou_month(xing: Xing, year_zhi: u8, shu: HeHunShu) -> u8 {
    let ju = (year_zhi % 12) % 4; // 0 申子辰 1 巳酉丑 2 寅午戌 3 亥卯未
    let m = match xing {
        Xing::Nan => [1, 6, 4, 2][ju as usize],
        Xing::Nv => match (ju, shu) {
            (2, HeHunShu::XieJi) => 8,
            _ => [12, 9, 7, 8][ju as usize],
        },
    };
    m - 1
}

/// 年月六害（六親刑剋偏不和）：本人生年支所害之月
pub fn liu_hai_month(year_zhi: u8) -> u8 {
    // 子未、丑午、寅巳、卯辰、申亥、酉戌：两支之和 ≡ 7
    ord_of_zhi((19 - year_zhi % 12) % 12)
}

/// 四敗（四敗下生；「男女生月犯之，雖見成婚，每多啾唧」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiBai {
    DaBai,
    LangJi,
    FeiTianLangJi,
    BaBai,
}

impl SiBai {
    pub fn name(self) -> &'static str {
        match self {
            SiBai::DaBai => "大敗",
            SiBai::LangJi => "狼籍",
            SiBai::FeiTianLangJi => "飛天狼籍",
            SiBai::BaBai => "八敗",
        }
    }
}

/// 四敗诸月：生年支分四组（子辰巳、丑申酉、寅卯午、未戌亥），类编第 58 页表（协纪云「其月分更無例可推」）
pub fn si_bai_masks(year_zhi: u8) -> [(SiBai, u16); 4] {
    let grp = match year_zhi % 12 {
        0 | 4 | 5 => 0,
        1 | 8 | 9 => 1,
        2 | 3 | 6 => 2,
        _ => 3,
    };
    // 月数（1—12）
    const T: [[&[u8]; 4]; 4] = [
        [&[4], &[5], &[2, 3], &[6]],
        [&[7], &[8], &[1, 7], &[9]],
        [&[10], &[11], &[5, 6], &[12]],
        [&[1], &[2], &[11], &[3]],
    ];
    let m = |ms: &[u8]| ms.iter().fold(0u16, |a, &x| a | bit(x - 1));
    let r = T[grp];
    [(SiBai::DaBai, m(r[0])), (SiBai::LangJi, m(r[1])), (SiBai::FeiTianLangJi, m(r[2])), (SiBai::BaBai, m(r[3]))]
}

/// 孤、虛
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuXu {
    Gu,
    Xu,
}

/// 孤虛：生年所在旬之空亡二支之月为孤，其衝为虛
pub fn gu_xu_masks(year_gz: u8) -> (u16, u16) {
    let xun_start_zhi = (year_gz % 60 - year_gz % 60 % 10) % 12; // 旬首（甲某）之支
    let k1 = (xun_start_zhi + 10) % 12;
    let k2 = (xun_start_zhi + 11) % 12;
    let gu = bit(ord_of_zhi(k1)) | bit(ord_of_zhi(k2));
    let xu = bit(ord_of_zhi(k1 + 6)) | bit(ord_of_zhi(k2 + 6));
    (gu, xu)
}

// ───────────────────────── 第 1 层：纳音五行诸月 ─────────────────────────

fn wx_idx(w: WuXing) -> usize {
    match w {
        WuXing::Jin => 0,
        WuXing::Mu => 1,
        WuXing::Shui => 2,
        WuXing::Huo => 3,
        WuXing::Tu => 4,
    }
}

/// 男取纳音所尅为妻，女取尅纳音者为夫（索引 金木水火土）
fn target_wx(xing: Xing, w: WuXing) -> usize {
    // 金尅木、木尅土、水尅火、火尅金、土尅水
    const KE: [usize; 5] = [1, 4, 3, 0, 2];
    // 尅金者火、尅木者金、尅水者土、尅火者水、尅土者木
    const KE_BY: [usize; 5] = [3, 0, 4, 2, 1];
    match xing {
        Xing::Nan => KE[wx_idx(w)],
        Xing::Nv => KE_BY[wx_idx(w)],
    }
}

/// 五行长生之月序：金巳、木亥、水申、火寅、土申（水土同）
fn chang_sheng_ord(t: usize) -> u8 {
    ord_of_zhi([5, 11, 8, 2, 8][t])
}

/// 自长生起之十二位（0 長生 … 5 衰 6 病 7 死 8 墓 9 絕 10 胎 11 養）之掩码
fn stage_mask(t: usize, stages: std::ops::RangeInclusive<u8>) -> u16 {
    stages.fold(0u16, |m, s| m | bit(chang_sheng_ord(t) + s))
}

/// 一人生月所值：益財、退財、望門鰥（寡）、妻（夫）多厄、死墓絕妨妻（夫）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CaiYueMasks {
    pub yi_cai: u16,
    pub tui_cai: u16,
    pub wang_men: u16,
    pub duo_e: u16,
    pub si_mu_jue: u16,
}

/// 益財诸月。协纪：自长生至衰为益財，病至養为退財，絕为望門，死墓为多厄，死墓絕为妨。
/// 类编、鳌头照表：男金益財七月起、土五月起，女火木金早一月；女死墓絕妨夫与男命同行（协纪斥为傳寫之譌）；
/// 鳌头男木益財八月至十二月（七月不入益、退）。
pub fn cai_yue_masks(xing: Xing, nayin: WuXing, shu: HeHunShu) -> CaiYueMasks {
    let t = target_wx(xing, nayin);
    let rule = CaiYueMasks {
        yi_cai: stage_mask(t, 0..=5),
        tui_cai: stage_mask(t, 6..=11),
        wang_men: stage_mask(t, 9..=9),
        duo_e: stage_mask(t, 7..=8),
        si_mu_jue: stage_mask(t, 7..=9),
    };
    if shu == HeHunShu::XieJi {
        return rule;
    }
    let w = wx_idx(nayin);
    // 表之益財、退財（起月序、月数），金木水火土
    let (yi, tui): ((u8, u8), (u8, u8)) = match xing {
        Xing::Nan => {
            const YI: [(u8, u8); 5] = [(6, 6), (6, 6), (0, 6), (3, 6), (4, 6)];
            const TUI: [(u8, u8); 5] = [(0, 6), (0, 6), (6, 6), (9, 6), (10, 6)];
            if shu == HeHunShu::AoTou && w == 1 {
                ((7, 5), (0, 6))
            } else {
                (YI[w], TUI[w])
            }
        }
        Xing::Nv => {
            const YI: [(u8, u8); 5] = [(11, 6), (2, 6), (6, 6), (5, 6), (9, 6)];
            const TUI: [(u8, u8); 5] = [(5, 6), (8, 6), (0, 6), (11, 6), (3, 6)];
            (YI[w], TUI[w])
        }
    };
    let si_mu_jue = match xing {
        Xing::Nan => rule.si_mu_jue,
        // 女命此行与男命同：以本命所尅者（男命之妻）之死墓絕
        Xing::Nv => stage_mask(target_wx(Xing::Nan, nayin), 7..=9),
    };
    CaiYueMasks { yi_cai: run(yi.0, yi.1), tui_cai: run(tui.0, tui.1), si_mu_jue, ..rule }
}

// ───────────────────────── 第 1 层：日柱诸煞 ─────────────────────────

fn in_list(gz: u8, list: &[u8]) -> bool {
    list.contains(&(gz % 60))
}

/// 陰錯陽差十二日：丙子丁丑戊寅辛卯壬辰癸巳丙午丁未戊申辛酉壬戌癸亥（类编第 61 页、《星學大成》卷三、《三命通會》卷三）
pub fn yin_cuo_yang_cha_ri(gz: u8) -> bool {
    in_list(gz, &[12, 13, 14, 27, 28, 29, 42, 43, 44, 57, 58, 59])
}

/// 孤鸞日
pub fn gu_luan_ri(gz: u8, shuo: GuLuanShuo) -> bool {
    match shuo {
        // 乙巳丁巳辛亥甲寅丙午戊午壬子戊申
        GuLuanShuo::BaRi => in_list(gz, &[41, 53, 47, 50, 42, 54, 48, 44]),
        // 甲寅丁巳戊申辛亥
        GuLuanShuo::SiRi => in_list(gz, &[50, 53, 44, 47]),
    }
}

/// 曲脚殺（又為縮脚殺）：己巳乙巳丁巳日（类编第 61 页「人命日家遇之主尅頭妻」）
pub fn qu_jiao_ri(gz: u8) -> bool {
    in_list(gz, &[5, 41, 53])
}

/// 平頭殺：四柱天干依次甲乙丙丁「一路無間」（类编第 61—62 页只举甲子年乙亥月丙寅日丁酉時一例）
pub fn ping_tou(si_zhu: [u8; 4]) -> bool {
    si_zhu.iter().map(|g| g % 10).eq([0u8, 1, 2, 3])
}

// ───────────────────────── 第 2 层：一对男女 ─────────────────────────

/// 一人所犯（只依本人年月日时者）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeRenSha {
    pub gong: u8,
    /// 男犯孤辰、女犯寡宿
    pub gu_gua: bool,
    pub gu_sui_po: bool,
    pub tie_sao_zhou: bool,
    pub liu_hai: bool,
    pub si_bai: Vec<SiBai>,
    pub gu_xu: Option<GuXu>,
    pub cai: CaiYueMasks,
    /// 益財、退財、望門、多厄、死墓絕（生月是否值）
    pub yi_cai: bool,
    pub tui_cai: bool,
    pub wang_men: bool,
    pub duo_e: bool,
    pub si_mu_jue: bool,
    /// 陰錯陽差：年、月、日、时柱各是否值（类编「月日時兩重或三重犯之極重，只日家犯之尤重」）
    pub yin_cuo: [bool; 4],
    pub gu_luan: bool,
    pub qu_jiao: bool,
    pub ping_tou: bool,
}

/// 合婚结果
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeHunResult {
    pub shu: HeHunShu,
    pub nan: GeRenSha,
    pub nv: GeRenSha,
    pub gua: GuaPei,
    /// 男年衝女月
    pub bao_tai_nan_nian: bool,
    /// 女年衝男月
    pub bao_tai_nv_nian: bool,
    /// 女命逐月（月序 0—11）行嫁之类：大利、妨媒氏首子、妨翁姑、妨女父母、妨夫主、妨女身
    pub nv_jia_yue: [NvMingYue; 12],
    /// 通书判语（照录书中之辞）
    pub pan_yu: Vec<String>,
}

fn ge_ren(xing: Xing, m: &HeHunMing, opts: &HeHunOptions) -> GeRenSha {
    let yz = m.year_zhi();
    let mo = m.month_ord();
    let mb = bit(mo);
    let cai = cai_yue_masks(xing, nayin_wuxing(m.year_gz()), opts.shu);
    let (gu, xu) = gu_xu_masks(m.year_gz());
    let [_, _, day, _] = m.si_zhu;
    GeRenSha {
        gong: ming_gong(xing, m.yuan, m.year_gz(), opts.shu),
        gu_gua: mo == if xing == Xing::Nan { gu_chen_month(yz) } else { gua_su_month(yz) },
        gu_sui_po: mo == gu_sui_po_month(yz),
        tie_sao_zhou: mo == tie_sao_zhou_month(xing, yz, opts.shu),
        liu_hai: mo == liu_hai_month(yz),
        si_bai: si_bai_masks(yz).iter().filter(|(_, mk)| mk & mb != 0).map(|(s, _)| *s).collect(),
        gu_xu: if gu & mb != 0 {
            Some(GuXu::Gu)
        } else if xu & mb != 0 {
            Some(GuXu::Xu)
        } else {
            None
        },
        cai,
        yi_cai: cai.yi_cai & mb != 0,
        tui_cai: cai.tui_cai & mb != 0,
        wang_men: cai.wang_men & mb != 0,
        duo_e: cai.duo_e & mb != 0,
        si_mu_jue: cai.si_mu_jue & mb != 0,
        yin_cuo: m.si_zhu.map(yin_cuo_yang_cha_ri),
        gu_luan: gu_luan_ri(day, opts.gu_luan),
        qu_jiao: qu_jiao_ri(day),
        ping_tou: ping_tou(m.si_zhu),
    }
}

fn pan_yu_of(nan: &GeRenSha, nv: &GeRenSha, gua: GuaPei, bt: (bool, bool)) -> Vec<String> {
    let mut v = Vec::new();
    let gp = format!("命卦{}（男{}宫{}、女{}宫{}）", gua.name(), nan.gong, gong_gua_name(ji_gong(Xing::Nan, nan.gong)), nv.gong, gong_gua_name(ji_gong(Xing::Nv, nv.gong)));
    v.push(match gua.deng() {
        GuaDeng::ShangJi => format!("{gp}：上吉，子孫昌盛，不避衝刑害絕勾絞及胞胎有犯月內諸凶，並無忌也"),
        GuaDeng::ZhongDeng => format!("{gp}：中等，可以較量輕重而言之；命卦通和、月中少忌，然後可以成婚"),
        GuaDeng::WuGuiXiong => format!("{gp}：五鬼之婚，於男女多主撓攪，口舌相連"),
        GuaDeng::JueMingXiong => format!("{gp}：絕命之婚，禍必深重，於男女各有憂亡；使命卦和悅，又得九吉相當，亦不宜為其婚也"),
    });
    for (x, r, who) in [(Xing::Nan, nan, "男"), (Xing::Nv, nv, "女")] {
        if r.gu_gua {
            v.push(if x == Xing::Nan { "男犯孤辰：男忌孤神".into() } else { "女犯寡宿：女忌寡宿，主嫁別鄉".into() });
        }
        if r.gu_sui_po {
            v.push(format!("{who}犯骨髓破：{}", if x == Xing::Nan { "男破女家" } else { "女破男家" }));
        }
        if r.tie_sao_zhou {
            v.push(format!("{who}犯鐵掃帚：{}", if x == Xing::Nan { "男掃女家" } else { "女掃男家" }));
        }
        if r.liu_hai {
            v.push(format!("{who}犯年月六害：六親刑剋偏不和"));
        }
        if !r.si_bai.is_empty() {
            let ns: Vec<&str> = r.si_bai.iter().map(|s| s.name()).collect();
            v.push(format!("{who}犯{}：雖見成婚，每多啾唧", ns.join("、")));
        }
        if let Some(g) = r.gu_xu {
            v.push(format!("{who}犯{}：男女孤虛，浪蕩難居，男當傷女，女必妨夫", if g == GuXu::Gu { "孤" } else { "虛" }));
        }
        let jia = if x == Xing::Nan { "女家" } else { "夫家" };
        if r.yi_cai {
            v.push(format!("{who}生月益財：益{jia}"));
        }
        if r.tui_cai {
            v.push(format!("{who}生月退財：退{jia}"));
        }
        if r.wang_men {
            v.push(format!("{who}犯望門{}", if x == Xing::Nan { "守鰥" } else { "守寡" }));
        }
        if r.duo_e {
            v.push(format!("{who}犯{}", if x == Xing::Nan { "妻多厄" } else { "夫多厄" }));
        }
        if r.si_mu_jue {
            v.push(format!("{who}犯死墓絕：{}", if x == Xing::Nan { "男命犯之妨三妻" } else { "女命犯之妨三夫" }));
        }
        let n = r.yin_cuo.iter().filter(|&&b| b).count();
        if n > 0 {
            let s = if x == Xing::Nan { "主退外家，亦與妻家是非寡合" } else { "翁姑寡合，妯娌不足，外家冷退" };
            let heavy = if n >= 2 { "；兩重或三重犯之極重" } else if r.yin_cuo[2] { "；日家犯之尤重" } else { "" };
            v.push(format!("{who}犯陰錯陽差：{s}{heavy}"));
        }
        if r.gu_luan {
            v.push(format!("{who}日犯孤鸞：{}", if x == Xing::Nan { "男尅妻" } else { "女尅夫" }));
        }
        if r.qu_jiao {
            v.push(format!("{who}日犯曲脚殺：主尅頭妻"));
        }
        if r.ping_tou {
            v.push(format!("{who}犯平頭殺：主陷害，{}", if x == Xing::Nan { "男尅妻" } else { "女尅夫" }));
        }
    }
    if bt.0 {
        v.push("男年衝女月，犯胞胎相衝（穿胎殺）：犯之多產厄".into());
    }
    if bt.1 {
        v.push("女年衝男月，犯胞胎相衝（穿胎殺）：犯之多產厄".into());
    }
    if (bt.0 || bt.1) && gua.deng() == GuaDeng::ShangJi {
        v.push("命卦生氣天醫福德，胞胎相衝即不忌".into());
    }
    v
}

/// 合婚：未启用（缺省）返回 `None`。
pub fn he_hun(nan: &HeHunMing, nv: &HeHunMing, opts: &HeHunOptions) -> Option<HeHunResult> {
    if !opts.enabled {
        return None;
    }
    let a = ge_ren(Xing::Nan, nan, opts);
    let b = ge_ren(Xing::Nv, nv, opts);
    let gua = gua_pei(a.gong, b.gong);
    let bt = (bao_tai_mask(nan.year_zhi()) & bit(nv.month_ord()) != 0, bao_tai_mask(nv.year_zhi()) & bit(nan.month_ord()) != 0);
    let nv_jia_yue = std::array::from_fn(|k| nv_ming_yue(nv.year_zhi(), k as u8));
    let pan_yu = pan_yu_of(&a, &b, gua, bt);
    Some(HeHunResult { shu: opts.shu, nan: a, nv: b, gua, bao_tai_nan_nian: bt.0, bao_tai_nv_nian: bt.1, nv_jia_yue, pan_yu })
}

// ───────────────────────── 逐格核对 ─────────────────────────

/// 起例表一行
#[derive(Clone, Debug)]
pub struct QiliRow {
    pub fa: String,
    pub shu: String,
    pub ye: String,
    pub jian: String,
    pub zhi: String,
}

/// 解析 `hehun_qili.csv`（跳过 # 注释头与列名行；原文、校两列不取）
pub fn qili_rows() -> Vec<QiliRow> {
    QILI_CSV
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.splitn(8, ',').collect();
            QiliRow { fa: c[0].into(), shu: c[1].into(), ye: c[3].into(), jian: c[4].into(), zhi: c[5].into() }
        })
        .collect()
}

const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";
const GAN: &str = "甲乙丙丁戊己庚辛壬癸";

fn zhi_idx(c: char) -> u8 {
    ZHI.chars().position(|x| x == c).unwrap() as u8
}

fn gz_parse(s: &str) -> u8 {
    let cs: Vec<char> = s.chars().collect();
    let g = GAN.chars().position(|c| c == cs[0]).unwrap() as u8;
    let z = zhi_idx(cs[1]);
    (0..60u8).find(|k| k % 10 == g && k % 12 == z).unwrap()
}

fn wx_parse(s: &str) -> WuXing {
    match s {
        "金" => WuXing::Jin,
        "木" => WuXing::Mu,
        "水" => WuXing::Shui,
        "火" => WuXing::Huo,
        _ => WuXing::Tu,
    }
}

fn mask_str(m: u16) -> String {
    // 按月数升序；跨年者自最小缺口后起
    let ms: Vec<u8> = (0..12u8).filter(|k| m & bit(*k) != 0).collect();
    if ms.is_empty() {
        return String::new();
    }
    let start = (0..12u8).find(|k| m & bit(*k) != 0 && m & bit((k + 11) % 12) == 0).unwrap_or(0);
    (0..12u8).map(|i| (start + i) % 12).filter(|k| m & bit(*k) != 0).map(|k| (k + 1).to_string()).collect::<Vec<_>>().join(" ")
}

fn shu_of(name: &str) -> HeHunShu {
    match name {
        "鳌头" => HeHunShu::AoTou,
        "协纪" => HeHunShu::XieJi,
        _ => HeHunShu::LeiBian,
    }
}

/// 一行起例按某书之说推出之值（与表「值」同格式）；不依书者 shu 不起作用
pub fn qili_compute(row: &QiliRow, shu: HeHunShu) -> String {
    let fa = row.fa.as_str();
    let k = row.jian.as_str();
    let first_zhi = || zhi_idx(k.chars().next().unwrap());
    let zhis = || k.chars().map(zhi_idx).collect::<Vec<u8>>();
    let ms = |f: &dyn Fn(u8) -> u16| -> String {
        let all: Vec<u16> = zhis().into_iter().map(f).collect();
        if all.iter().all(|m| *m == all[0]) {
            mask_str(all[0])
        } else {
            "（组内不一）".into()
        }
    };
    let xing_of = |fa: &str| if fa.ends_with('女') || fa.starts_with("望門寡") || fa.starts_with("夫") || fa.ends_with("妨夫") { Xing::Nv } else { Xing::Nan };
    match fa {
        "三元起宫" => {
            let x = if k.starts_with('男') { Xing::Nan } else { Xing::Nv };
            let y = match k.chars().nth(1) {
                Some('上') => SanYuan::Shang,
                Some('中') => SanYuan::Zhong,
                _ => SanYuan::Xia,
            };
            qi_gong(x, y, shu).to_string()
        }
        "五宫寄" => ji_gong(if k == "男" { Xing::Nan } else { Xing::Nv }, 5).to_string(),
        "三元命宫" => {
            let p: Vec<&str> = k.split('|').collect();
            let y = match p[0] {
                "上元" => SanYuan::Shang,
                "中元" => SanYuan::Zhong,
                _ => SanYuan::Xia,
            };
            let x = if p[2] == "男" { Xing::Nan } else { Xing::Nv };
            let cs: Vec<char> = p[1].chars().collect();
            let vals: Vec<u8> = cs.chunks(2).map(|c| ming_gong(x, y, gz_parse(&c.iter().collect::<String>()), shu)).collect();
            if vals.iter().all(|v| *v == vals[0]) {
                vals[0].to_string()
            } else {
                "（行内不一）".into()
            }
        }
        "卦配" => {
            let p: Vec<u8> = k.split('-').map(|x| x.parse().unwrap()).collect();
            gua_pei(p[0], p[1]).name().into()
        }
        "孤辰" => ms(&|z| bit(gu_chen_month(z))),
        "寡宿" => ms(&|z| bit(gua_su_month(z))),
        "胞胎相衝" => ms(&bao_tai_mask),
        "骨髓破" => mask_str(bit(gu_sui_po_month(first_zhi()))),
        "鐵掃帚男" => mask_str(bit(tie_sao_zhou_month(Xing::Nan, first_zhi(), shu))),
        "鐵掃帚女" => mask_str(bit(tie_sao_zhou_month(Xing::Nv, first_zhi(), shu))),
        "鐵掃帚" => ms(&|z| bit(tie_sao_zhou_month(Xing::Nan, z, shu)) | bit(tie_sao_zhou_month(Xing::Nv, z, shu))),
        "六害" => mask_str(bit(liu_hai_month(first_zhi()))),
        "大敗" | "狼籍" | "飛天狼籍" | "八敗" => {
            let i = ["大敗", "狼籍", "飛天狼籍", "八敗"].iter().position(|x| *x == fa).unwrap();
            ms(&|z| si_bai_masks(z)[i].1)
        }
        "孤" | "虛" => {
            let y = gz_parse(&k.chars().take(2).collect::<String>());
            let (g, x) = gu_xu_masks(y);
            mask_str(if fa == "孤" { g } else { x })
        }
        "益財男" | "退財男" | "望門鰥" | "妻多厄" | "死墓絕妨妻" | "益財女" | "退財女" | "望門寡" | "夫多厄" | "死墓絕妨夫" => {
            let c = cai_yue_masks(xing_of(fa), wx_parse(k), shu);
            mask_str(match fa {
                "益財男" | "益財女" => c.yi_cai,
                "退財男" | "退財女" => c.tui_cai,
                "望門鰥" | "望門寡" => c.wang_men,
                "妻多厄" | "夫多厄" => c.duo_e,
                _ => c.si_mu_jue,
            })
        }
        "陰錯陽差" | "孤鸞" | "曲脚" => {
            let gl = if row.ye == "丁巳孤鸞" { GuLuanShuo::SiRi } else { GuLuanShuo::BaRi };
            let f = |g: u8| match fa {
                "陰錯陽差" => yin_cuo_yang_cha_ri(g),
                "孤鸞" => gu_luan_ri(g, gl),
                _ => qu_jiao_ri(g),
            };
            // 依表所列之序，列出六十日中所值者；与表同集合则照表序
            let want: Vec<u8> = row.zhi.split_whitespace().map(gz_parse).collect();
            let got: Vec<u8> = (0..60u8).filter(|g| f(*g)).collect();
            let mut ws = want.clone();
            ws.sort();
            if ws == got {
                row.zhi.clone()
            } else {
                got.iter().map(|g| format!("{}{}", GAN.chars().nth((g % 10) as usize).unwrap(), ZHI.chars().nth((g % 12) as usize).unwrap())).collect::<Vec<_>>().join(" ")
            }
        }
        "平頭" => {
            let sz: Vec<u8> = row.zhi.split_whitespace().map(gz_parse).collect();
            if ping_tou([sz[0], sz[1], sz[2], sz[3]]) { row.zhi.clone() } else { "不犯".into() }
        }
        "女命行嫁月" => {
            let p: Vec<&str> = k.split('|').collect();
            let kind = ["大利月", "妨媒氏首子", "妨翁姑", "妨女父母", "妨夫主", "妨女身"].iter().position(|x| *x == p[1]).unwrap();
            let kinds = [NvMingYue::DaLi, NvMingYue::FangMeiShi, NvMingYue::FangWengGu, NvMingYue::FangNvFuMu, NvMingYue::FangFu, NvMingYue::FangNvShen];
            let f = |z: u8| (0..12u8).filter(|m| nv_ming_yue(z, *m) == kinds[kind]).fold(0u16, |a, m| a | bit(m));
            let all: Vec<u16> = p[0].chars().map(|c| f(zhi_idx(c))).collect();
            if all.iter().all(|m| *m == all[0]) { mask_str(all[0]) } else { "（组内不一）".into() }
        }
        _ => panic!("未知之法 {fa}"),
    }
}

fn same_value(fa: &str, a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    // 月集合不计次序
    fn set(s: &str) -> Vec<&str> {
        let mut v: Vec<&str> = s.split_whitespace().collect();
        v.sort();
        v
    }
    !matches!(fa, "卦配" | "三元起宫" | "三元命宫" | "五宫寄") && set(a) == set(b)
}

/// 逐格核对：每行依该书之说推、依协纪推法推，各与表比。返回 CSV 文本（`data/folk/hehun_qili_checks.csv`）
pub fn qili_checks_csv() -> String {
    let mut s = String::from("# 合婚起例逐格核对（由 folk::hehun::tests 生成，勿手改）：依书＝按该行之书的选项推；依协纪推法＝按 HeHunShu::XieJi 推\n法,书,页,键,表值,依书推值,依书,依协纪推法推值,依协纪推法\n");
    for r in qili_rows() {
        let own = qili_compute(&r, shu_of(&r.shu));
        let xj = qili_compute(&r, HeHunShu::XieJi);
        let ok = |v: &str| if same_value(&r.fa, v, &r.zhi) { "合" } else { "异" };
        s.push_str(&format!("{},{},{},{},{},{},{},{},{}\n", r.fa, r.shu, r.ye, r.jian, r.zhi, own, ok(&own), xj, ok(&xj)));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gz(s: &str) -> u8 {
        gz_parse(s)
    }

    /// 每行起例按其书之说推，与表逐格全合
    #[test]
    fn qili_table_all_rows_match_own_book() {
        let rows = qili_rows();
        assert!(rows.len() > 400, "{}", rows.len());
        let mut bad = Vec::new();
        for r in &rows {
            let v = qili_compute(r, shu_of(&r.shu));
            if !same_value(&r.fa, &v, &r.zhi) {
                bad.push(format!("{}|{}|{}", r.fa, r.shu, r.jian));
            }
        }
        // 唯一之异：类编遊魂「九五」误刻（以男九女五、女五寄艮推得五鬼；鳌头正作「九二」）
        assert_eq!(bad, vec!["卦配|类编|9-5".to_string()]);
    }

    /// 类编、鳌头之表与协纪推法相异之格，正是协纪所斥之「傳寫之譌」与「误以上元为中元」
    #[test]
    fn tongshu_vs_xieji_rule_differences() {
        let mut diff: Vec<String> = Vec::new();
        for r in qili_rows().iter().filter(|r| r.shu != "协纪") {
            let v = qili_compute(r, HeHunShu::XieJi);
            if !same_value(&r.fa, &v, &r.zhi) {
                diff.push(format!("{}|{}|{}", r.fa, r.shu, r.jian));
            }
        }
        let count = |fa: &str| diff.iter().filter(|d| d.starts_with(&format!("{fa}|"))).count();
        // 男命起宫（类编、鳌头各三元）与命宫立成（男 27 格 × 2 书）
        assert_eq!(count("三元起宫"), 6);
        assert_eq!(count("三元命宫"), 54);
        // 鐵掃帚女火局
        assert_eq!(count("鐵掃帚女"), 3);
        // 益財：男金、土（类编、鳌头）+ 鳌头男木；女火木金 × 2 书
        assert_eq!(count("益財男"), 5);
        assert_eq!(count("退財男"), 4);
        assert_eq!(count("益財女"), 6);
        assert_eq!(count("退財女"), 6);
        // 死墓絕妨夫：女命五行皆与男命同行，五格全异
        assert_eq!(count("死墓絕妨夫"), 5);
        // 卦配「九五」误刻（男五寄坤则合，女五寄艮则否）
        assert_eq!(count("卦配"), 1);
        let others: Vec<&String> = diff
            .iter()
            .filter(|d| !["三元起宫|", "三元命宫|", "鐵掃帚女|", "益財男|", "退財男|", "益財女|", "退財女|", "死墓絕妨夫|", "卦配|"].iter().any(|p| d.starts_with(p)))
            .collect();
        assert!(others.is_empty(), "{others:#?}");
    }

    /// 知识库逐格核对表与现算一致（`HEHUN_WRITE_CHECKS=1` 时重写）
    #[test]
    fn checks_csv_up_to_date() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/folk/hehun_qili_checks.csv");
        let s = qili_checks_csv();
        if std::env::var("HEHUN_WRITE_CHECKS").is_ok() {
            std::fs::write(path, &s).unwrap();
        }
        let on_disk = std::fs::read_to_string(path).unwrap_or_default();
        assert_eq!(on_disk, s, "hehun_qili_checks.csv 过期：HEHUN_WRITE_CHECKS=1 重跑");
    }

    #[test]
    fn san_yuan_years() {
        assert_eq!(san_yuan(1864), SanYuan::Shang);
        assert_eq!(san_yuan(1923), SanYuan::Shang);
        assert_eq!(san_yuan(1924), SanYuan::Zhong);
        assert_eq!(san_yuan(1984), SanYuan::Xia);
        assert_eq!(san_yuan(2043), SanYuan::Xia);
        assert_eq!(san_yuan(2044), SanYuan::Shang);
        assert_eq!(san_yuan(1324), SanYuan::Shang);
    }

    /// 类编第 53 页之例：「一命在一宫，一命在四宫，即合成生氣」
    #[test]
    fn gua_pei_example() {
        assert_eq!(gua_pei(1, 4), GuaPei::ShengQi);
        assert_eq!(gua_pei(4, 1), GuaPei::ShengQi);
        // 协纪卷二小遊年：乾上爻变兑为贪狼（生氣），兑中爻变震巨門（天醫）……
        assert_eq!(gua_pei(6, 7), GuaPei::ShengQi);
        assert_eq!(gua_pei(6, 3), GuaPei::TianYi);
        assert_eq!(gua_pei(6, 2), GuaPei::JueTi);
        assert_eq!(gua_pei(6, 1), GuaPei::YouHun);
        assert_eq!(gua_pei(6, 4), GuaPei::WuGui);
        assert_eq!(gua_pei(6, 8), GuaPei::FuDe);
        assert_eq!(gua_pei(6, 9), GuaPei::JueMing);
        assert_eq!(gua_pei(6, 6), GuaPei::GuiHun);
        // 五寄：男五寄坤、女五寄艮
        assert_eq!(gua_pei(5, 5), gua_pei(2, 8));
    }

    /// 类编第 64 页之例：甲子（海中金）男生五、六、七月死墓絕妨妻；娶丙寅（爐中火）女
    #[test]
    fn leibian_example_p64() {
        let c = cai_yue_masks(Xing::Nan, nayin_wuxing(gz("甲子")), HeHunShu::LeiBian);
        assert_eq!(mask_str(c.si_mu_jue), "5 6 7");
        assert_eq!(nayin_wuxing(gz("丙寅")), WuXing::Huo);
    }

    /// 协纪所举水命男女之例
    #[test]
    fn xieji_shui_examples() {
        let n = cai_yue_masks(Xing::Nan, WuXing::Shui, HeHunShu::XieJi);
        assert_eq!((mask_str(n.yi_cai), mask_str(n.tui_cai), mask_str(n.wang_men), mask_str(n.duo_e), mask_str(n.si_mu_jue)), ("1 2 3 4 5 6".into(), "7 8 9 10 11 12".into(), "10".into(), "8 9".into(), "8 9 10".into()));
        let f = cai_yue_masks(Xing::Nv, WuXing::Shui, HeHunShu::XieJi);
        assert_eq!((mask_str(f.yi_cai), mask_str(f.tui_cai), mask_str(f.wang_men), mask_str(f.duo_e), mask_str(f.si_mu_jue)), ("7 8 9 10 11 12".into(), "1 2 3 4 5 6".into(), "4".into(), "2 3".into(), "2 3 4".into()));
    }

    #[test]
    fn ping_tou_and_days() {
        assert!(ping_tou([gz("甲子"), gz("乙亥"), gz("丙寅"), gz("丁酉")]));
        assert!(!ping_tou([gz("乙丑"), gz("乙亥"), gz("丙寅"), gz("丁酉")]));
        assert_eq!((0..60u8).filter(|g| yin_cuo_yang_cha_ri(*g)).count(), 12);
        assert_eq!((0..60u8).filter(|g| gu_luan_ri(*g, GuLuanShuo::BaRi)).count(), 8);
        assert_eq!((0..60u8).filter(|g| gu_luan_ri(*g, GuLuanShuo::SiRi)).count(), 4);
        assert!((0..60u8).filter(|g| gu_luan_ri(*g, GuLuanShuo::SiRi)).all(|g| gu_luan_ri(g, GuLuanShuo::BaRi)));
    }

    /// 缺省不启用；启用后给出结果与判语
    #[test]
    fn he_hun_default_off_and_example() {
        // 男 1990 庚午年、女 1992 壬申年
        let nan = HeHunMing::new(1990, gz("戊寅"), gz("丙子"), gz("戊子"));
        let nv = HeHunMing::new(1992, gz("丁未"), gz("乙巳"), gz("丙子"));
        assert!(he_hun(&nan, &nv, &HeHunOptions::default()).is_none());
        let r = he_hun(&nan, &nv, &HeHunOptions::tongshu(HeHunShu::LeiBian)).unwrap();
        // 1990 庚午、1992 壬申皆下元（1984 起）：类编下元庚午男七，壬申女七
        assert_eq!(r.nan.gong, 7);
        assert_eq!(r.nv.gong, 7);
        assert_eq!(r.gua, GuaPei::GuiHun); // 兌兌
        let x = he_hun(&nan, &nv, &HeHunOptions::tongshu(HeHunShu::XieJi)).unwrap();
        // 三元經：下元男起兌七，庚午逆行六宫 → 一（坎）；女同类编
        assert_eq!(x.nan.gong, 1);
        assert_eq!(x.nv.gong, 7);
        assert_eq!(x.gua, GuaPei::WuGui); // 坎兌
        // 男年午衝女月：午前後三辰 酉、卯（八、二月）；女生六月（未），不犯
        assert!(!r.bao_tai_nan_nian);
        // 男生日丙子：陰錯陽差、日家
        assert!(r.nan.yin_cuo[2]);
        // 女日乙巳：孤鸞、曲脚
        assert!(r.nv.gu_luan && r.nv.qu_jiao);
        assert!(r.pan_yu.iter().any(|s| s.starts_with("命卦歸魂")));
        assert_eq!(r.nv_jia_yue.iter().filter(|y| **y == NvMingYue::DaLi).count(), 2);
    }
}
