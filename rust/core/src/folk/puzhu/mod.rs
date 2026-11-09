//! 铺注：月表之上、随年份节气朔望而变的诸日。
//!
//! 《欽定協紀辨方書》卷十：「見義例以下諸日皆每年不同，亦月表萬年書所不能載，臨時選用詳萬年書鋪註條例」。
//! 月表（[`super::yiji`]）只依月建与日干支；本模块在月表一日之上，按卷十各条加注：
//!
//! | 日 | 定法 | 卷十所注 |
//! |---|---|---|
//! | 反支 | 卷九：子丑朔六日、寅卯朔五日……戌亥朔一日 | 忌上冊進表章陳詞訟；與德合赦願併猶忌 |
//! | 上朔 | 卷九：甲年癸亥、乙年己巳……癸年丁巳 | 止不忌祭祀解除沐浴整容剃頭整手足甲補垣塞穴掃舍宇修飾垣牆平治道塗破屋壞垣伐木，餘事皆忌 |
//! | 四離、四絕 | 卷九：二分二至、四立各前一日 | 同上朔 |
//! | 晦日 | 月盡之日 | 止宜祭祀等事，餘事雖不忌亦不註宜（卷十上朔條按語） |
//! | 二至二分 | 冬至、夏至、春分、秋分當日 | 不註上冊進表章……捕捉畋獵取魚；冬至又不註伐木 |
//! | 土王用事 | 四立前十八日 | 忌營建宮室……栽種破土（義同土府） |
//! | 伏、社 | 三伏；二分前後近戊為社（卷十二引歷例） | 忌沐浴 |
//! | 朔弦望 | 朔、上弦、望、下弦所在日 | 忌求醫療病 |
//! | 月忌日 | 每月初五、十四、二十三（卷六） | 止註祭祀宴會沐浴整容剃頭整手足甲求醫療病補垣掃舍宇修飾垣牆平治道塗，餘事不註 |
//! | 十五日 | | 忌求醫療病（義同望日） |
//! | 人神所在 | 十二日在髮際、十五日在遍身忌剃頭；一日、六日、十五日、十九日、二十一日、二十三日忌整手足甲 | |
//! | 長星、短星 | 卷九：按月取日數 | 忌進人口裁製經絡開市立券交易納財納畜 |
//! | 氣往亡 | 卷九：立春後七日……小寒後三十日 | 同往亡（卷十「徃亡 氣徃亡」同條），與德合赦願併猶忌 |
//! | 嫁娶周堂 | 卷三十五：大月從夫順數，小月從婦逆數（夫姑堂翁第竈婦廚） | 「遇第堂厨竈日用之，如遇翁姑而無翁姑者亦可用」：值夫、婦之日不註宜嫁娶 |
//! | 交節之日 | 当日交节 | 时宪书以新旧两月之宜相交（见 [`jiao_jie_base`]） |
//! | 畋獵、取魚的节令 | 卷十一：畋獵「霜降後立春前」、取魚「雨水後立夏前」 | 月表按月建自戌月、寅月起注；逐年铺注于戌月霜降前、寅月雨水前不註宜 |
//!
//! 卷十：上朔以下「與德合赦願併猶忌，以下並同」，所以本层诸条一律不因德神而解。
//!
//! 無祿日（十惡大敗）月表只注癸亥；其余九日卷十明言「月表與萬年書俱不論無祿日」（因歲德、交中氣等每年不同），
//! 「臨時選用」。逐年颁行的时宪书则照注：乾隆三十年时宪书正月二十六日壬申（無祿，無解）月表所宜之療病不注，
//! 只注祭祀沐浴掃舍宇破屋壞垣。故本层并入無祿，解与不解按卷十無祿日條（见 [`wulu`]）。
//!
//! 事目沿用月表的通书六十事（合并显示单位 58 项，[`SHI`]）；卷十所列御用事目按月表引擎的对应关系折合
//! （上冊進表章 → 上冊受封、上表章；宴會 → 會親友；行幸遣使 → 出行；般移 → 移徙；整容剃頭 → 剃頭；裁製 → 裁衣；
//! 營建宮室、修宮室、繕城郭、興造動土 → 修造動土；補垣 → 補垣塞穴）。時憲書所注为民用三十七事，见 [`minyong`]。
//! 依据、口径与核对见 `doc/folk/shensha.md`「铺注」一节。

use super::yiji::YueBiaoDay;

pub mod context;
pub use context::{puzhu_days, PuzhuDay, QiWangWangCount, TermTable};

/// 月表所用通书六十事（卷十一「通書選擇六十事」，立券交易、開倉庫出貨財合并显示，共 58 项），按原书次第
pub const SHI: [&str; 58] = [
    "祭祀", "祈福", "求嗣", "上冊受封", "上表章", "襲爵受封", "會親友", "入學", "冠帶", "出行", "上官赴任", "臨政親民",
    "結婚姻", "納采問名", "嫁娶", "進人口", "移徙", "遠迴", "安牀", "解除", "沐浴", "剃頭", "整手足甲", "求醫療病",
    "療目", "針刺", "裁衣", "築隄防", "修造動土", "豎柱上梁", "修倉庫", "鼓鑄", "苫蓋", "經絡", "醞釀", "開市",
    "立券交易", "納財", "開倉庫出貨財", "修置產室", "開渠穿井", "安碓磑", "補垣塞穴", "掃舍宇", "修飾垣墻", "平治道塗",
    "破屋壞垣", "伐木", "捕捉", "畋獵", "取魚", "乘船渡水", "栽種", "牧養", "納畜", "破土", "安葬", "啟攢",
];

/// 事目在 [`SHI`] 中的序；月表的省写（補垣、開渠、穿井）归入全称
pub fn shi_index(s: &str) -> Option<usize> {
    let full = match s {
        "補垣" => "補垣塞穴",
        "開渠" | "穿井" => "開渠穿井",
        _ => s,
    };
    SHI.iter().position(|x| *x == full)
}

/// 民用三十七事（卷十一），時憲書所注：(民用事名, 所据通书事目)
pub const MINYONG: [(&str, &str); 37] = [
    ("祭祀", "祭祀"), ("上表章", "上表章"), ("上官", "上官赴任"), ("入學", "入學"), ("冠帶", "冠帶"),
    ("結婚姻", "結婚姻"), ("會親友", "會親友"), ("嫁娶", "嫁娶"), ("進人口", "進人口"), ("出行", "出行"),
    ("移徙", "移徙"), ("安牀", "安牀"), ("沐浴", "沐浴"), ("剃頭", "剃頭"), ("療病", "求醫療病"), ("裁衣", "裁衣"),
    ("修造動土", "修造動土"), ("豎柱上梁", "豎柱上梁"), ("經絡", "經絡"), ("開市", "開市"), ("立券", "立券交易"),
    ("交易", "立券交易"), ("納財", "納財"), ("修置產室", "修置產室"), ("開渠穿井", "開渠穿井"), ("安碓磑", "安碓磑"),
    ("掃舍宇", "掃舍宇"), ("平治道塗", "平治道塗"), ("破屋壞垣", "破屋壞垣"), ("伐木", "伐木"), ("捕捉", "捕捉"),
    ("畋獵", "畋獵"), ("栽種", "栽種"), ("牧養", "牧養"), ("破土", "破土"), ("安葬", "安葬"), ("啟攢", "啟攢"),
];

/// 二分二至
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FenZhi {
    ChunFen,
    XiaZhi,
    QiuFen,
    DongZhi,
}

impl FenZhi {
    pub fn name(self) -> &'static str {
        match self {
            FenZhi::ChunFen => "春分",
            FenZhi::XiaZhi => "夏至",
            FenZhi::QiuFen => "秋分",
            FenZhi::DongZhi => "冬至",
        }
    }
}

/// 一日的年月、节气、朔望条件（由历法、天文层算好传入；本层不碰天文）
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PuzhuInput {
    /// 日干支序（甲子 = 0）
    pub day_gz: u8,
    /// 年干序（甲 = 0），上朔用。年以立春分（同年神）
    pub year_gan: u8,
    /// 农历月数 1—12（閏月取所閏之月）、日 1—30、本月日数
    pub lunar_month: u8,
    pub lunar_day: u8,
    pub days_in_month: u8,
    /// 次日交二分二至（四離）、次日交四立（四絕）
    pub si_li: bool,
    pub si_jue: bool,
    /// 当日交二分二至
    pub fen_zhi: Option<FenZhi>,
    /// 在土王用事期内（四立前十八日至四立前一日）
    pub tu_wang: bool,
    /// 三伏日、社日
    pub fu: bool,
    pub she: bool,
    /// 朔、上弦、望、下弦所在日（名称）
    pub phase: Option<&'static str>,
    /// 氣往亡
    pub qi_wang_wang: bool,
    /// 节气月序（寅 = 0）与当日是否在本月中气之前（畋獵、取魚的节令用）
    pub month_ord: u8,
    pub before_zhong_qi: bool,
    /// 值無祿而不解（[`wulu`] 为 `Some(WuLu::Yes)`；癸亥月表已注，此处亦为真）
    pub wulu: bool,
}

/// 上朔、四離、四絕「止不忌」者；晦日「止宜」者亦同（卷十上朔條）
const ZHI_BU_JI: [&str; 11] =
    ["祭祀", "解除", "沐浴", "剃頭", "整手足甲", "補垣塞穴", "掃舍宇", "修飾垣墻", "平治道塗", "破屋壞垣", "伐木"];
/// 無祿日「止註」者（祭祀解除沐浴整容剃頭整手足甲掃舍宇修飾垣墻平治道塗破屋壞垣伐木）
const WULU_ZHI_ZHU: [&str; 10] =
    ["祭祀", "解除", "沐浴", "剃頭", "整手足甲", "掃舍宇", "修飾垣墻", "平治道塗", "破屋壞垣", "伐木"];
/// 二至二分「不註」者（上冊進表章慶賜賞賀宴會行幸遣使安撫邊境選將訓兵出師上官赴任臨政親民結婚姻納采問名嫁娶進人口般移開市立券交易捕捉畋獵取魚）
const FEN_ZHI_BU_ZHU: [&str; 16] = [
    "上冊受封", "上表章", "會親友", "出行", "上官赴任", "臨政親民", "結婚姻", "納采問名", "嫁娶", "進人口", "移徙",
    "開市", "立券交易", "捕捉", "畋獵", "取魚",
];
/// 土王用事所忌（營建宮室修宮室繕城郭築隄防興造動土修倉庫修置產室開渠穿井安碓磑補垣修飾垣墻平治道塗破屋壞垣栽種破土）
const TU_WANG_JI: [&str; 12] = [
    "築隄防", "修造動土", "修倉庫", "修置產室", "開渠穿井", "安碓磑", "補垣", "修飾垣墻", "平治道塗", "破屋壞垣", "栽種",
    "破土",
];
/// 月忌日「止註」者（祭祀宴會沐浴整容剃頭整手足甲求醫療病補垣掃舍宇修飾垣牆平治道塗）
const YUE_JI_ZHI_ZHU: [&str; 10] =
    ["祭祀", "會親友", "沐浴", "剃頭", "整手足甲", "求醫療病", "補垣塞穴", "掃舍宇", "修飾垣墻", "平治道塗"];
/// 長星短星所忌（進人口裁製經絡開市立券交易納財納畜）
const CHANG_DUAN_XING_JI: [&str; 7] = ["進人口", "裁衣", "經絡", "開市", "立券交易", "納財", "納畜"];
/// 往亡所忌（卷十：上冊進表章頒詔詔命公卿招賢宣政事行幸遣使安撫邊境選將訓兵出師上官赴任臨政親民嫁娶進人口般移
/// 求醫療病捕捉畋獵取魚），折合六十事；與月表引擎据卷十一所取者同
const WANG_WANG_JI: [&str; 12] = [
    "上冊受封", "上表章", "出行", "上官赴任", "臨政親民", "嫁娶", "進人口", "移徙", "求醫療病", "捕捉", "畋獵", "取魚",
];

/// 反支（卷九：子丑朔六日，寅卯朔五日，辰巳朔四日，午未朔三日，申酉朔二日，戌亥朔一日）
pub fn is_fan_zhi(shuo_gz: u8, lunar_day: u8) -> bool {
    lunar_day == 6 - (shuo_gz % 12) / 2
}

/// 上朔日干支序（卷九：甲年癸亥、乙年己巳、丙年乙亥、丁年辛巳、戊年丁亥、己年癸巳、庚年己亥、辛年乙巳、壬年辛亥、癸年丁巳）
pub fn shang_shuo_gz(year_gan: u8) -> u8 {
    ((59 + 6 * year_gan as u32) % 60) as u8
}

/// 長星、短星（卷九「日神按月取日數者」）：农历某月某日是否值
pub fn chang_xing(lunar_month: u8, lunar_day: u8) -> bool {
    const T: [&[u8]; 12] = [&[7], &[4], &[1], &[9], &[15], &[10], &[8], &[2, 5], &[3, 4], &[1], &[12], &[9]];
    (1..=12).contains(&lunar_month) && T[lunar_month as usize - 1].contains(&lunar_day)
}
pub fn duan_xing(lunar_month: u8, lunar_day: u8) -> bool {
    const T: [&[u8]; 12] = [&[21], &[19], &[16], &[25], &[25], &[20], &[22], &[18, 19], &[16, 17], &[14], &[22], &[25]];
    (1..=12).contains(&lunar_month) && T[lunar_month as usize - 1].contains(&lunar_day)
}

/// 氣往亡：交节后第几日（卷六引歷例「立春後七日……皆自交節日數之」），按节序 0 立春 … 11 小寒
pub const QI_WANG_WANG_DAYS: [u8; 12] = [7, 14, 21, 8, 16, 24, 9, 18, 27, 10, 20, 30];

/// 一日所值的铺注诸日（名称，按卷十次第）
pub fn puzhu_names(p: &PuzhuInput, shuo_gz: u8) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    if is_fan_zhi(shuo_gz, p.lunar_day) {
        v.push("反支".into());
    }
    if p.day_gz == shang_shuo_gz(p.year_gan) {
        v.push("上朔".into());
    }
    if p.si_li {
        v.push("四離".into());
    }
    if p.si_jue {
        v.push("四絕".into());
    }
    if p.lunar_day == p.days_in_month {
        v.push("晦日".into());
    }
    if let Some(f) = p.fen_zhi {
        v.push(f.name().into());
    }
    if p.tu_wang {
        v.push("土王用事".into());
    }
    if p.fu {
        v.push("伏".into());
    }
    if p.she {
        v.push("社".into());
    }
    if let Some(ph) = p.phase {
        v.push(ph.into());
    }
    if [5, 14, 23].contains(&p.lunar_day) {
        v.push("月忌".into());
    }
    if p.lunar_day == 15 {
        v.push("十五日".into());
    }
    match p.lunar_day {
        12 => v.push("人神在髮際".into()),
        15 => v.push("人神在遍身".into()),
        1 => v.push("人神在足大指".into()),
        6 => v.push("人神在手".into()),
        19 => v.push("人神在足".into()),
        21 => v.push("人神在手小指".into()),
        23 => v.push("人神在肝及足".into()),
        _ => {}
    }
    if chang_xing(p.lunar_month, p.lunar_day) {
        v.push("長星".into());
    }
    if duan_xing(p.lunar_month, p.lunar_day) {
        v.push("短星".into());
    }
    if p.qi_wang_wang {
        v.push("氣往亡".into());
    }
    if p.wulu {
        v.push("無祿".into());
    }
    if let Some(z) = zhou_tang(p.days_in_month, p.lunar_day) {
        if z == "夫" || z == "婦" {
            v.push(format!("周堂值{z}"));
        }
    }
    v
}

/// 嫁娶周堂（卷三十五）：大月從夫順數，小月從婦逆數。周堂圖乾翁、坎第、艮竈、震婦、巽厨、離夫、坤姑、兌堂，
/// 后天八卦自離顺行为夫姑堂翁第竈婦厨
pub fn zhou_tang(days_in_month: u8, lunar_day: u8) -> Option<&'static str> {
    const DA: [&str; 8] = ["夫", "姑", "堂", "翁", "第", "竈", "婦", "廚"];
    const XIAO: [&str; 8] = ["婦", "竈", "第", "翁", "堂", "姑", "夫", "廚"];
    if lunar_day == 0 {
        return None;
    }
    let k = (lunar_day as usize - 1) % 8;
    Some(if days_in_month >= 30 { DA[k] } else { XIAO[k] })
}

/// 交节在此钟点（东八区）以前者，全日从新月，不取两月之交。乾隆三十年时宪书十月二十五日大雪交节在丑时
///（推算 2 时 45 分），书照注新月之宜「祭祀平治道塗」；小暑（8 时 37 分）、寒露（9 时 34 分）以后各节皆取两月之交。
/// 界限在 2 时 45 分与 8 时 37 分之间，暂取卯初（5 时），待更多交节日验证。
pub const JIAO_JIE_EARLY_HOUR: f64 = 5.0;

/// 交节之日的月表：时宪书于交节当日只注新旧两月皆宜之事（乾隆三十年閏二月十五日清明，旧月宜沐浴掃舍宇捕捉、
/// 新月宜祭祀沐浴掃舍宇，书注「沐浴掃舍宇」；三月十六日立夏，旧月无民用之宜，书不注宜）。忌取两月之并。
/// 协纪无明文，据时宪书归纳；建除、诸神仍从新月（卷四「每月交節則疊兩值日」）。
pub fn jiao_jie_base(new: &YueBiaoDay, old: &YueBiaoDay) -> YueBiaoDay {
    let mut d = new.clone();
    d.yi = match (&new.yi, &old.yi) {
        (Some(a), Some(b)) => Some(a.iter().filter(|x| b.iter().any(|y| shi_index(y) == shi_index(x))).cloned().collect()),
        _ => None,
    };
    if d.yi.is_some() {
        for x in &old.ji {
            if !d.ji.iter().any(|y| shi_index(y) == shi_index(x)) {
                d.ji.push(x.clone());
            }
        }
        sort_shi(&mut d.ji);
    }
    d
}

fn sort_shi(v: &mut Vec<String>) {
    v.sort_by_key(|s| shi_index(s).unwrap_or(usize::MAX));
    v.dedup();
}

/// 忌某事：不註宜，註忌（諸事不宜之日不动）
fn ji(d: &mut YueBiaoDay, items: &[&str]) {
    let Some(yi) = d.yi.as_mut() else { return };
    yi.retain(|s| !items.iter().any(|x| shi_index(x) == shi_index(s)));
    for x in items {
        let k = shi_index(x);
        // 已注全称或省写者不再添；所添为全称而原注为省写者，改为全称
        if let Some(pos) = d.ji.iter().position(|s| shi_index(s) == k) {
            if d.ji[pos] != *x && SHI.contains(x) {
                d.ji[pos] = x.to_string();
            }
        } else {
            d.ji.push(x.to_string());
        }
    }
    sort_shi(&mut d.ji);
}

/// 不註宜（不添忌）
fn bu_zhu_yi(d: &mut YueBiaoDay, items: &[&str]) {
    if let Some(yi) = d.yi.as_mut() {
        yi.retain(|s| !items.iter().any(|x| shi_index(x) == shi_index(s)));
    }
}

/// 止註（止宜）某些事，餘事不註宜
fn zhi_yi(d: &mut YueBiaoDay, allowed: &[&str]) {
    if let Some(yi) = d.yi.as_mut() {
        yi.retain(|s| allowed.iter().any(|x| shi_index(x) == shi_index(s)));
    }
}

/// 一日铺注以前的月表（交节之日按 [`jiao_jie_base`] 取新旧两月之交）
pub fn yuebiao_base_for_day(d: &PuzhuDay) -> YueBiaoDay {
    use super::yiji::yuebiao_day;
    let base = yuebiao_day(d.month_ord, d.input.day_gz);
    if d.jiao_jie && d.jiao_jie_hour.is_some_and(|h| h >= JIAO_JIE_EARLY_HOUR) {
        jiao_jie_base(base, yuebiao_day((d.month_ord + 11) % 12, d.input.day_gz))
    } else {
        base.clone()
    }
}

/// 一日的宜忌：月表（交节之日按 [`jiao_jie_base`]）加铺注，即时宪书体例下的通书六十事宜忌。
pub fn yiji_for_day(d: &PuzhuDay) -> YueBiaoDay {
    apply(&yuebiao_base_for_day(d), &d.input, d.shuo_gz)
}

/// 时宪书底栏朱字所用之神（以乾隆三十年时宪书归纳，见 doc/folk/shensha.md「时宪书朱字」）：
/// 月表诸神中的往亡、月厭、劫煞、天吏、月刑、大耗、小耗，铺注的反支、四離、四絕、氣往亡（印作往亡），
/// 以及协纪不载、钦天监沿用通书之神：荒蕪（即五虛，《選擇宗鏡》）、枯焦（与九焦同位）、
/// 四不祥（农历初四、初七、十六、十九、二十八；协纪卷三十六辨訛主删而书仍印）、
/// 天獄（通书旧法：正月起子顺行四仲；协纪卷六已改从災煞而书仍用旧法）。
/// 交节之日新旧两月之神并列（书于立夏、立秋、寒露、立冬、小寒诸日印旧月之神）。
/// 书上每日实际印出几位还受版面所限（宜事长者栏底无空，常不印），此处给出应值之全集。
pub const ZHU_ZI: [&str; 14] = [
    "往亡", "月厭", "劫煞", "天吏", "月刑", "大耗", "小耗", "反支", "四離", "四絕", "荒蕪", "枯焦", "四不祥", "天獄",
];

pub fn zhu_zi(d: &PuzhuDay) -> Vec<&'static str> {
    use super::yiji::yuebiao_day;
    let mut months = vec![d.month_ord];
    if d.jiao_jie {
        months.push((d.month_ord + 11) % 12);
    }
    let names = puzhu_names(&d.input, d.shuo_gz);
    let zhi = d.input.day_gz % 12;
    let mut out: Vec<&'static str> = Vec::new();
    for &m in &months {
        let xs = &yuebiao_day(m, d.input.day_gz).xiong_shen;
        let has = |n: &str| xs.iter().any(|x| x == n);
        for n in ZHU_ZI {
            let on = match n {
                "往亡" => has("往亡") || names.iter().any(|x| x == "氣往亡"),
                "反支" | "四離" | "四絕" => names.iter().any(|x| x == n),
                "荒蕪" => has("五虛"),
                "枯焦" => has("九焦"),
                "四不祥" => matches!(d.input.lunar_day, 4 | 7 | 16 | 19 | 28),
                // 月序 m（寅月 0）：正月子、二月卯、三月午、四月酉……
                "天獄" => zhi == (3 * m) % 12,
                _ => has(n),
            };
            if on && !out.contains(&n) {
                out.push(n);
            }
        }
    }
    out
}

/// 时宪书「不宜某事」所用五事（乾隆三十年时宪书归纳）
pub const BU_YI: [&str; 5] = ["出行", "移徙", "針刺", "動土", "栽種"];

/// 交节之日「不宜」的取法
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum JiaoJieBuYi {
    /// 时宪书体例（缺省）：交节在 [`JIAO_JIE_EARLY_HOUR`] 以后者，「不宜」取新旧两月之交（两月各自推「不宜」，
    /// 只留两月皆有之事）；以前者从新月。协纪无明文（卷四只言建除「每月交節則疊兩值日」，卷十、卷十一不及交节），
    /// 据时宪书实测：乾隆三十年十三个交节日、乾隆二十一年小暑寒露立冬、三十一年清明，书印「不宜」皆合此法
    ///（见 doc/folk/shensha.md「他年时宪书」）
    #[default]
    ShiXianShu,
    /// 只从新月（建除、诸神皆从新月，忌亦只取新月）
    XinYue,
}

/// 时宪书「不宜」推算选项
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BuYiOptions {
    pub jiao_jie: JiaoJieBuYi,
}

/// 时宪书「不宜某事」应值之全集（以乾隆三十年时宪书归纳，见 doc/folk/shensha.md「时宪书「不宜」与朱字」）：
/// 某事在当日忌（月表加铺注之后，通书六十事）之内，且当日值该事的触发之神——
/// - 出行：往亡、氣往亡、天賊、月厭、四離、四絕，或巳日（百忌「巳不遠行」，卷十作「巳日忌出行」）；
/// - 移徙：歸忌、月厭；
/// - 針刺：血忌、血支；
/// - 動土（修造動土）：土府（建日）、土符、地囊；
/// - 栽種：九焦（时宪书朱字作枯焦）、月煞、月厭，或乙日（百忌「乙不栽植」）。
///
/// 触发神皆为协纪所载该事之忌神：往亡等见卷十各条；四離、四絕卷十同上朔「餘事皆忌」，而卷六四離條引李鼎祚
/// 「此日忌出行征伐」、四絕條引李鼎祚「此日忌出軍逺行」，专指出行（乾隆三十年四離四絕九日书全印不宜出行，
/// 二十一年閏九月十四日四絕亦印）。此集中书未印者多在宜事长、栏底无空之日，宜事短之日书亦常只印一二事，
/// 取舍未明，故只给全集，不模拟取舍。月表「諸事不宜」之日（忌不另列）不给。交节之日按 [`BuYiOptions`]。
pub fn bu_yi(d: &PuzhuDay) -> Vec<&'static str> {
    bu_yi_with(d, &BuYiOptions::default())
}

/// 同 [`bu_yi`]，交节之日的取法按选项
pub fn bu_yi_with(d: &PuzhuDay, o: &BuYiOptions) -> Vec<&'static str> {
    let new = bu_yi_month(d, d.month_ord);
    let late = d.jiao_jie && d.jiao_jie_hour.is_some_and(|h| h >= JIAO_JIE_EARLY_HOUR);
    if late && o.jiao_jie == JiaoJieBuYi::ShiXianShu {
        let old = bu_yi_month(d, (d.month_ord + 11) % 12);
        new.into_iter().filter(|x| old.contains(x)).collect()
    } else {
        new
    }
}

/// 以某月（月序 `m`）之月表加铺注推当日「不宜」
fn bu_yi_month(d: &PuzhuDay, m: u8) -> Vec<&'static str> {
    use super::yiji::yuebiao_day;
    let base = yuebiao_day(m, d.input.day_gz);
    let after = apply(base, &d.input, d.shuo_gz);
    let names = puzhu_names(&d.input, d.shuo_gz);
    let has = |n: &str| base.xiong_shen.iter().any(|x| x == n) || names.iter().any(|x| x == n);
    let ji = |e: &str| after.ji.iter().any(|x| x == e);
    let (gan, zhi) = (d.input.day_gz % 10, d.input.day_gz % 12);
    let mut out = Vec::new();
    if ji("出行") && (has("往亡") || has("氣往亡") || has("天賊") || has("月厭") || has("四離") || has("四絕") || zhi == 5) {
        out.push("出行");
    }
    if ji("移徙") && (has("歸忌") || has("月厭")) {
        out.push("移徙");
    }
    if ji("針刺") && (has("血忌") || has("血支")) {
        out.push("針刺");
    }
    if ji("修造動土") && (has("土府") || has("土符") || has("地囊")) {
        out.push("動土");
    }
    if ji("栽種") && (has("九焦") || has("月煞") || has("月厭") || gan == 1) {
        out.push("栽種");
    }
    out
}

/// 铺注诸日中改动婚事（結婚姻、納采問名、嫁娶）之注者，下标即 [`apply_skipping`] 之 `skip` 位（婚嫁择吉「不看」用）。
/// 反支、土王、伏社、朔弦望、人神、長短星等不及婚事，不列
pub const PUZHU_HUN: [&str; 9] = ["上朔", "四離", "四絕", "晦日", "二分二至", "月忌日", "氣往亡", "無祿", "周堂值夫婦"];

/// `skip` 中某名之位
pub fn puzhu_hun_bit(name: &str) -> u16 {
    PUZHU_HUN.iter().position(|x| *x == name).map_or(0, |i| 1 << i)
}

/// 一日的月表宜忌加铺注。`shuo_gz` 为本农历月朔日干支（反支用）。返回加注后的一日，凶神栏后附所值诸日名
pub fn apply(day: &YueBiaoDay, p: &PuzhuInput, shuo_gz: u8) -> YueBiaoDay {
    apply_skipping(day, p, shuo_gz, 0)
}

/// 同 [`apply`]，而 `skip` 置位之铺注（位义见 [`PUZHU_HUN`]）不加——其名仍列于凶神栏，只是不改宜忌。
/// `skip` 为 0 即 [`apply`]
pub fn apply_skipping(day: &YueBiaoDay, p: &PuzhuInput, shuo_gz: u8, skip: u16) -> YueBiaoDay {
    let mut d = day.clone();
    let names = puzhu_names(p, shuo_gz);
    let off = |n: &str| skip & puzhu_hun_bit(n) != 0;
    let has = |n: &str| {
        let k = match n {
            "月忌" => "月忌日",
            "周堂值夫" | "周堂值婦" => "周堂值夫婦",
            x => x,
        };
        !off(k) && names.iter().any(|x| x == n)
    };
    if has("反支") {
        ji(&mut d, &["上冊受封", "上表章"]);
    }
    if has("上朔") || has("四離") || has("四絕") {
        // 止不忌某某，餘事皆忌
        zhi_yi(&mut d, &ZHI_BU_JI);
        let rest: Vec<&str> = SHI.iter().copied().filter(|s| !ZHI_BU_JI.contains(s)).collect();
        if d.yi.is_some() {
            ji(&mut d, &rest);
        }
    }
    if has("晦日") {
        zhi_yi(&mut d, &ZHI_BU_JI);
    }
    if has("無祿") {
        zhi_yi(&mut d, &WULU_ZHI_ZHU);
    }
    if let Some(f) = p.fen_zhi.filter(|_| !off("二分二至")) {
        bu_zhu_yi(&mut d, &FEN_ZHI_BU_ZHU);
        if f == FenZhi::DongZhi {
            bu_zhu_yi(&mut d, &["伐木"]);
        }
    }
    if has("土王用事") {
        // 義同土府：止註忌補垣，亦不註宜塞穴
        bu_zhu_yi(&mut d, &["補垣塞穴"]);
        ji(&mut d, &TU_WANG_JI);
    }
    if has("伏") || has("社") {
        ji(&mut d, &["沐浴"]);
    }
    if p.phase.is_some() || has("十五日") {
        ji(&mut d, &["求醫療病"]);
    }
    if has("月忌") {
        zhi_yi(&mut d, &YUE_JI_ZHI_ZHU);
    }
    if [12, 15].contains(&p.lunar_day) {
        ji(&mut d, &["剃頭"]);
    }
    if [1, 6, 15, 19, 21, 23].contains(&p.lunar_day) {
        ji(&mut d, &["整手足甲"]);
    }
    if has("長星") || has("短星") {
        ji(&mut d, &CHANG_DUAN_XING_JI);
    }
    if has("氣往亡") {
        ji(&mut d, &WANG_WANG_JI);
        // 鋪註條例：凡忌詔命公卿招賢不註宜施恩封拜舉正直襲爵受封
        bu_zhu_yi(&mut d, &["襲爵受封"]);
    }
    // 卷十一畋獵「霜降後立春前」、取魚「雨水後立夏前」：月表自戌月、寅月起注，中气以前不註宜
    if p.before_zhong_qi && p.month_ord == 8 {
        bu_zhu_yi(&mut d, &["畋獵"]);
    }
    if p.before_zhong_qi && p.month_ord == 0 {
        bu_zhu_yi(&mut d, &["取魚"]);
    }
    if has("周堂值夫") || has("周堂值婦") {
        // 不註宜嫁娶（不添忌）
        bu_zhu_yi(&mut d, &["嫁娶"]);
    }
    for n in names {
        if !d.xiong_shen.contains(&n) {
            d.xiong_shen.push(n);
        }
    }
    d
}

/// 时宪书式：折成民用三十七事。返回 (宜, 忌)；宜为 None 表示諸事不宜
pub fn minyong(d: &YueBiaoDay) -> (Option<Vec<&'static str>>, Vec<&'static str>) {
    let pick = |v: &[String]| -> Vec<&'static str> {
        MINYONG
            .iter()
            .filter(|(_, src)| v.iter().any(|s| shi_index(s) == shi_index(src)))
            .map(|(m, _)| *m)
            .collect()
    };
    (d.yi.as_ref().map(|v| pick(v)), pick(&d.ji))
}

// ---------------- 無祿（臨時選用） ----------------

/// 日干之祿（卷四：甲祿在寅……癸祿在子），支序
fn lu_zhi(gan: u8) -> u8 {
    [2, 3, 5, 6, 5, 6, 8, 9, 11, 0][gan as usize % 10]
}

/// 無祿日的判定结果
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WuLu {
    /// 值無祿，「止註祭祀解除沐浴整容剃頭整手足甲掃舍宇修飾垣墻平治道塗破屋壞垣伐木，餘事不註」
    Yes,
    /// 值無祿日而按卷十「不以無祿論」，附理由
    Exempt(&'static str),
}

/// 無祿日（十惡大敗：甲辰乙巳丙申丁亥戊戌己丑庚辰辛巳壬申癸亥，干祿落旬空）及其解（卷十無祿日條）。
/// 非無祿日返回 None。`sun_palace_zhi` 为太阳所在宫的支序（雨水後在亥、春分後在戌……大寒後在子，以中氣交宮）；
/// `has_tian_yue_de` 为当日值天德或月德。月表与万年书只注癸亥（见模块说明），本函数供臨時選用。
pub fn wulu(day_gz: u8, year_gz: u8, month_gz: u8, sun_palace_zhi: u8, has_tian_yue_de: bool) -> Option<WuLu> {
    const DAYS: [u8; 10] = [40, 41, 32, 23, 34, 25, 16, 17, 8, 59];
    if !DAYS.contains(&day_gz) {
        return None;
    }
    // 癸亥干支俱盡：雖值天月二德、歲月太陽填實、歲德會合，仍以無祿論
    if day_gz == 59 {
        return Some(WuLu::Yes);
    }
    let dg = day_gz % 10;
    let lu = lu_zhi(dg);
    if has_tian_yue_de {
        return Some(WuLu::Exempt("與天德月德併"));
    }
    if year_gz % 12 == lu || month_gz % 12 == lu {
        return Some(WuLu::Exempt("歲月填實祿空"));
    }
    if sun_palace_zhi == lu {
        return Some(WuLu::Exempt("太陽填實祿空"));
    }
    let yg = year_gz % 10;
    // 歲德：陽年取本干，陰年取所合之陽干；歲德合為歲德所合之干
    let sui_de = if yg % 2 == 0 { yg } else { (yg + 5) % 10 };
    let sui_de_he = (sui_de + 5) % 10;
    // 月德（寅午戌月丙、亥卯未月甲、申子辰月壬、巳酉丑月庚）與月德合；天德合（以月建取）
    let mz = month_gz % 12;
    let yue_de = match mz {
        2 | 6 | 10 => 2,
        11 | 3 | 7 => 0,
        8 | 0 | 4 => 8,
        _ => 6,
    };
    let yue_de_he = (yue_de + 5) % 10;
    if dg == sui_de_he && dg == yue_de_he {
        return Some(WuLu::Exempt("歲德合月德合所會之辰"));
    }
    // 天德合（只取天干者）：卷十所列四日即亥月庚、巳月丙、寅月壬、申月戊
    let tian_de_he = match mz {
        11 => Some(6),
        5 => Some(2),
        2 => Some(8),
        8 => Some(4),
        _ => None,
    };
    if dg == sui_de && tian_de_he == Some(dg) {
        return Some(WuLu::Exempt("天德合歲德所會之辰"));
    }
    if yg == month_gz % 10 && yg == dg {
        return Some(WuLu::Exempt("歲德歲德合天干三朋"));
    }
    Some(WuLu::Yes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(yi: Option<&[&str]>, ji_: &[&str]) -> YueBiaoDay {
        YueBiaoDay {
            jian_chu: "成".into(),
            ji_shen: vec![],
            xiong_shen: vec![],
            yi: yi.map(|v| v.iter().map(|s| s.to_string()).collect()),
            ji: ji_.iter().map(|s| s.to_string()).collect(),
            deng_di: String::new(),
        }
    }

    #[test]
    fn tables() {
        assert_eq!(SHI.len(), 58);
        // 上朔：甲年癸亥(59)、乙年己巳(5)、癸年丁巳(53)
        assert_eq!((shang_shuo_gz(0), shang_shuo_gz(1), shang_shuo_gz(9)), (59, 5, 53));
        // 反支：子朔六日、亥朔一日
        assert!(is_fan_zhi(0, 6) && is_fan_zhi(11, 1) && !is_fan_zhi(11, 2));
        assert!(chang_xing(8, 2) && chang_xing(8, 5) && duan_xing(9, 17) && !duan_xing(1, 20));
    }

    #[test]
    fn rules() {
        let base = day(Some(&["祭祀", "上表章", "嫁娶", "沐浴", "求醫療病", "修造動土", "補垣塞穴"]), &["開渠"]);
        // 上朔：止不忌祭祀沐浴補垣塞穴等，餘事皆忌
        let p = PuzhuInput { day_gz: 59, year_gan: 0, lunar_day: 3, days_in_month: 30, ..Default::default() };
        let d = apply(&base, &p, 0);
        assert_eq!(d.yi.as_deref().unwrap(), ["祭祀", "沐浴", "補垣塞穴"]);
        assert!(d.ji.iter().any(|s| s == "嫁娶") && d.ji.iter().any(|s| s == "開渠穿井") && !d.ji.iter().any(|s| s == "開渠"));
        assert!(d.xiong_shen.iter().any(|s| s == "上朔"));
        // 晦日：止宜祭祀等，餘事不忌亦不註宜
        let p = PuzhuInput { day_gz: 1, lunar_day: 29, days_in_month: 29, ..Default::default() };
        let d = apply(&base, &p, 0);
        assert_eq!(d.yi.as_deref().unwrap(), ["祭祀", "沐浴", "補垣塞穴"]);
        assert_eq!(d.ji, ["開渠"]);
        // 土王用事：忌修造動土，補垣止註忌補垣
        let p = PuzhuInput { day_gz: 1, lunar_day: 3, days_in_month: 30, tu_wang: true, ..Default::default() };
        let d = apply(&base, &p, 0);
        assert!(!d.yi.as_ref().unwrap().iter().any(|s| s == "修造動土" || s == "補垣塞穴"));
        assert!(d.ji.iter().any(|s| s == "補垣") && d.ji.iter().any(|s| s == "修造動土"));
        // 望日、十五日：忌求醫療病；十五日人神在遍身忌剃頭整手足甲
        let p = PuzhuInput { day_gz: 1, lunar_day: 15, days_in_month: 30, phase: Some("望"), ..Default::default() };
        let d = apply(&base, &p, 0);
        assert!(!d.yi.as_ref().unwrap().iter().any(|s| s == "求醫療病"));
        assert!(["求醫療病", "剃頭", "整手足甲"].iter().all(|x| d.ji.iter().any(|s| s == x)));
        // 諸事不宜之日不再添忌
        let none = day(None, &[]);
        let p = PuzhuInput { day_gz: 1, lunar_day: 5, days_in_month: 30, ..Default::default() };
        let d = apply(&none, &p, 0);
        assert!(d.yi.is_none() && d.ji.is_empty());
    }

    /// 铺注九项逐项可不看：720 月表日 × 各项单值之输入，全看与 [`apply`] 同；只跳过所值之项则婚事之注回到月表，
    /// 跳过他项不变
    #[test]
    fn skipping_each_item() {
        use super::super::yiji::yuebiao_day;
        // 各项单值之输入（day_gz 另设）：上朔取年干使 shang_shuo_gz 恰为当日
        let mk = |k: &str, gz: u8| -> PuzhuInput {
            let mut p = PuzhuInput { day_gz: gz, year_gan: 0, lunar_month: 3, lunar_day: 3, days_in_month: 29, month_ord: 2, ..Default::default() };
            // 初三小月周堂值第（不值夫婦）；year_gan 取使上朔不值者
            p.year_gan = (0..10u8).find(|g| shang_shuo_gz(*g) != gz).unwrap();
            match k {
                "上朔" => p.year_gan = (0..10u8).find(|g| shang_shuo_gz(*g) == gz).unwrap_or(p.year_gan),
                "四離" => p.si_li = true,
                "四絕" => p.si_jue = true,
                "晦日" => p.lunar_day = 29,
                "二分二至" => p.fen_zhi = Some(FenZhi::ChunFen),
                "月忌日" => p.lunar_day = 5,
                "氣往亡" => p.qi_wang_wang = true,
                "無祿" => p.wulu = true,
                "周堂值夫婦" => p.lunar_day = 1, // 小月初一值婦
                _ => {}
            }
            p
        };
        let hun = |d: &YueBiaoDay| -> Vec<(bool, bool)> {
            ["結婚姻", "納采問名", "嫁娶"].iter().map(|s| (d.yi.as_ref().is_some_and(|v| v.iter().any(|x| x == s)), d.ji.iter().any(|x| x == s))).collect()
        };
        let mut changed = [0usize; 9];
        for m in 0..12u8 {
            for gz in 0..60u8 {
                let base = yuebiao_day(m, gz);
                for (i, k) in PUZHU_HUN.iter().enumerate() {
                    let p = mk(k, gz);
                    if *k == "上朔" && shang_shuo_gz(p.year_gan) != gz {
                        continue;
                    }
                    let full = apply(base, &p, 0);
                    assert_eq!(full, apply_skipping(base, &p, 0, 0));
                    // 跳过他项（全部他位）：不变
                    assert_eq!(full, apply_skipping(base, &p, 0, 0x1ff & !(1 << i)), "{k}");
                    // 跳过本项：婚事之注同无此项之日
                    let none = apply(base, &mk("", gz), 0);
                    let sk = apply_skipping(base, &p, 0, 1 << i);
                    assert_eq!(hun(&sk), hun(&none), "{k} {m} {gz}");
                    changed[i] += (hun(&full) != hun(&sk)) as usize;
                }
            }
        }
        // 每项皆有改动婚事之日
        assert!(changed.iter().all(|&n| n > 0), "{changed:?}");
    }

    #[test]
    fn wulu_rules() {
        // 非無祿日
        assert_eq!(wulu(0, 0, 2, 2, false), None);
        // 癸亥：雖值二德仍以無祿論
        assert_eq!(wulu(59, 0, 0, 0, true), Some(WuLu::Yes));
        // 寅年甲辰日：歲月填實
        assert_eq!(wulu(40, 50, 3, 9, false), Some(WuLu::Exempt("歲月填實祿空")));
        // 雨水後（太陽在亥）壬申日：太陽填實
        assert_eq!(wulu(8, 1, 3, 11, false), Some(WuLu::Exempt("太陽填實祿空")));
        // 丁壬年申子辰月丁亥日：歲德合月德合所會（丁年 = 3，子月丙子 = 12）
        assert_eq!(wulu(23, 3, 12, 9, false), Some(WuLu::Exempt("歲德合月德合所會之辰")));
        // 乙庚年亥月庚辰日：天德合歲德所會（乙丑年 = 1，亥月丁亥 = 23）
        assert_eq!(wulu(16, 1, 23, 9, false), Some(WuLu::Exempt("天德合歲德所會之辰")));
        // 丙辛年巳月丙申日：巳月已是歲月填實（丙祿在巳），卷十兩處並列
        assert_eq!(wulu(32, 2, 29, 9, false), Some(WuLu::Exempt("歲月填實祿空")));
        // 甲年甲月甲辰日：天干三朋（甲子年 0，甲戌月 10）
        assert_eq!(wulu(40, 0, 10, 9, false), Some(WuLu::Exempt("歲德歲德合天干三朋")));
        // 無解
        assert_eq!(wulu(40, 1, 12, 9, false), Some(WuLu::Yes));
    }
}
