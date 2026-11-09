//! 七政四余（星命）：日、月、五星（七政）与罗睺、计都、月孛、紫炁（四余）的宫度、宿度，命宫与十二宫。
//!
//! - 七政：本项目历表的地心视黄经、视黄纬（当日真黄道、真春分点；`body_apparent_ecliptic`）。
//! - 罗睺、计都：白道交点。哪一个是升交点（正交），各家相反，见 [`LuoJi`]；交点取平交点或真（瞬时）交点，见 [`NodeKind`]。
//! - 月孛：月之最高（远地点），见 [`BeiKind`]。
//! - 紫炁：天无其象的虚点，约二十八年一周，见 [`ZiQiRule`]。
//! - 宫：以中气分宫（太阳过宫），黄经 0°—30° 为戌宫（降娄）……330°—360° 为亥宫（娵訾），与时宪书同。
//! - 宿度：黄道宿度——二十八宿距星的视黄经为各宿起点，天体视黄经减其西侧最近之距星，周天 360°（清时宪书即此）。
//! - 命宫：以太阳所躔之宫加生时，卯上所临之宫为命宫（《星學大成》卷一「天盤轉出地盤上，卯上分明是命宫」），
//!   或取天文上升点所在宫（选项）；十二宫自命宫逆布（通例）。
//! - 星命（[`XingMing`]）：宫主、度主（宿主）、命主、身宫身主、十二宫主、廟樂旺喜怒、七强五弱、
//!   恩難（五行生克）、命度（量天尺，[`liang_tian_chi`]）、童限大限、童限逐岁宫（[`tong_xian_gong`]）、
//!   小限（[`xiao_xian`]、按生日 [`xiao_xian_sui`]）、月限（[`yue_xian`]）、流年變曜（[`liu_nian_bian_yao`]）；
//!   以宿度分宫的旧法见 [`GongRule`]。
//!
//! 原典、实测核对与各说出处见 doc/folk/qizheng.md。

use crate::astronomy::apparent::{body_apparent_ecliptic, ApparentPipelineOptions};
use crate::astronomy::frame::nutation::{nutation_for_model, NutationModel};
use crate::astronomy::frame::precession::mean_obliquity;
use crate::astronomy::pipeline::{Body, EphemerisProvider, Planet};
use crate::astronomy::sky_culture::chinese::{ecliptic_longitude, MansionFrame, MANSIONS};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};

/// 十一曜之序：太阳、太阴、水、金、火、木、土、罗睺、计都、月孛、紫炁
pub const YAO: [&str; 11] = ["太陽", "太陰", "水星", "金星", "火星", "木星", "土星", "羅睺", "計都", "月孛", "紫炁"];

/// 十二宫（次）名，下标为地支序（子 0 … 亥 11）
pub const GONG_CI: [&str; 12] = ["玄枵", "星紀", "析木", "大火", "壽星", "鶉尾", "鶉火", "鶉首", "實沈", "大梁", "降婁", "娵訾"];

/// 十二宫（人事），自命宫逆布
pub const PALACES: [&str; 12] = ["命宮", "財帛", "兄弟", "田宅", "男女", "奴僕", "妻妾", "疾厄", "遷移", "官祿", "福德", "相貌"];

/// 罗睺、计都何者为升交点（正交）。两说在中国前后更替（实物实测见 doc/folk/qizheng.md）：
/// 明回回历《七政推步》以计都为升交；崇祯新法（汤若望）、清初时宪书以罗睺为升交（杨光先斥之「以羅為計，以計為羅」）；
/// 清中期以后时宪书又以计都为升交。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuoJi {
    /// 计都为升交点、罗睺为降交点：回回历《七政推步》卷一「計都與月相離度……在初宮至五宮為黃道北」；
    /// 道光十三年（1833）七政经纬躔度时宪书实测（罗计差 25′ 内）。缺省
    JiAscending,
    /// 罗睺为升交点（正交）、计都为降交点：《明史》卷三十一崇祯改历议「羅喉、計都即正交、中交行度」；
    /// 康熙九年（1670）七政经纬躔度时宪历实测（罗睺差 30′、计都差 36′）；杨光先《不得已·摘謬十論》第八謬
    /// 「顛倒羅計之新……不過以羅為計，以計為羅」即斥此（相对回回旧法）。印度 Rāhu 亦为北交点
    LuoAscending,
}

/// 交点、最高取平行或真（瞬时）位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// 平交点：ELP 平均根数（Meeus《Astronomical Algorithms》式 47.7）。古今历家「為平行」（《明史》卷三十一）
    Mean,
    /// 真交点：瞬时轨道面与黄道之交（由月视位置与速度求角动量方向）。崇祯改历「設一加減，為交行均數」，
    /// 道光十三年时宪书罗计逐旬行度不匀，与真交点合（差 ±25′ 内）、与平交点差至 74′，故缺省
    True,
}

/// 月孛
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeiKind {
    /// 平远地点（ELP 平均根数，近地点 + 180°）。缺省：道光十三年时宪书月孛与之相差约 0.3°
    MeanApogee,
    /// 瞬时远地点（由月视位置速度求偏心率矢量）
    TrueApogee,
}

/// 紫炁
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZiQiRule {
    /// 清时宪书：自道光十三年七政经纬躔度时宪书逐旬紫炁宿度反推之匀速运动（周期 28 年、历元黄经见 [`ZIQI_SHIXIAN`]）。
    /// 推法原文未见，属据实物反推。缺省
    ShiXian,
    /// 不用紫炁（汤若望新法删之；《明史》卷三十一「天行無紫氣……知為妄增」）
    None,
}

/// 时宪书紫炁：(历元 JD(TT), 该时刻视黄经 °, 周期 日)。由道光十三年（1833）正月至四月逐旬宿度反推（见文档）
pub const ZIQI_SHIXIAN: (f64, f64, f64) = (2390598.5, 204.10, 28.0 * 365.2422);

/// 命宫之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MingRule {
    /// 太阳所躔之宫加生时，卯上所临为命宫（《星學大成》卷一）。缺省
    SunOnHour,
    /// 天文上升点（东方地平与黄道之交）所在之宫，需观测地经纬度
    Ascendant,
}

/// 分宫之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GongRule {
    /// 以中气分宫（黄经每 30° 一宫，太阳过宫即中气），清时宪书。缺省
    ZhongQi,
    /// 以宿度分宫：宫界在某宿某度，《張果星宗》卷一「度數所在」（女二子、斗四丑、尾三寅、氐一卯、軫十辰、張十五巳……），
    /// 见 [`GONG_JIE_XINGZONG`]。需距星（无则退回中气分宫）
    XiuDuXingZong,
    /// 以宿度分宫：《星學大成》卷一「氐一過卯尾三寅，斗三度兮丑中陳，女至一度方過子……張十四巳軫九辰」，
    /// 与星宗差一度者四（斗、女、張、軫），见 [`GONG_JIE_DACHENG`]
    XiuDuDaCheng,
}

/// 命度之法（卯上起命 [`MingRule::SunOnHour`] 时用；上升点法命度即上升点）。
///
/// 量天尺即「同位相對」之尺：太阳在其宫第几度，命度即在命宫同一第几度。
/// 古今圖書集成·藝術典卷五百六十七《張果星宗一·安命度法》「以太陽之度對著命宮之度，即是命度也。如太陽躔子宮虛六度，
/// 對著午宮星五度，即為命度。如日躔女三度，對著午宮柳四度是也」；同書卷五百六十九《欽天監校正授時過宮度圖》
/// 横列一度至三十度、竖列十二宫，同列即相對之度（书影）。中气分宫各宫等宽，二法相同；宿度分宫宫宽不等时有别。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MingDuRule {
    /// 自宫首（黄经小之一端，与相貌宫相接）量同度。安命度法二例用授时赤道宿度：虛六距子宫首（女二）15.35 度、
    /// 星五距午宫首（柳三）15.30 度，女三、柳四各 1 度，皆合（差 0.05 度）。缺省
    FromStart,
    /// 自宫尾（黄经大之一端）量同度：授時過宮度圖每宫「一度」在宫尾（子宫一度危十一、午宫一度張十四……）。
    /// 安命度法二例皆差 1.3 度
    FromEnd,
}

/// 月限起宫：《星學大成》卷三两说
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YueXianRule {
    /// 「小限宫中起生月……依宫逐月逆週十二宫」；《張果星宗》定小限例（集成本）「本人生于五月，就從戌上起，五月、六月酉……
    /// 所謂小限宮中起生月」。缺省
    XiaoXian,
    /// 「從小限行到太隂所泊之宫起生月逆數」：自太阴所在之宫（身宫）起生月
    TaiYin,
}

/// 命主取何者
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MingZhuRule {
    /// 命度主（命度所在宿之主曜）：《星命溯源》卷二《果憕問答·評人生禀賦分金論》「方知專用宫主為非，度主為是」；
    /// 《張果星宗》卷四先天心法「先觀主曜，即命度主也」。缺省（无距星则退回宫主）
    DuZhu,
    /// 命宫主（命宫之宫主）：分金论所斥「專用宫主」之法
    GongZhu,
}

/// 身主取何者（身宫恒为太阴所躔之宫）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShenZhuRule {
    /// 身度主（太阴所躔宿之主）：《張果星宗》有「身度主」之称（卷四「水為身度主」），与命主取度主同理。缺省
    DuZhu,
    /// 身宫主（太阴所躔宫之宫主）：《星學大成》卷二《論學堂》「凡身宫主星是學堂星，假令太隂星在人馬宫是身宫也，即木為宫主星」
    GongZhu,
}

/// 廟樂旺喜好怒之表
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiaoWangRule {
    /// 《星學大成》卷十四至二十二《三辰通載》各曜总论句（如卷十八太陽「其星廟戌樂巳好辰喜寅旺午怒酉」），十一曜俱全。缺省
    Lun,
    /// 同书各卷「某星廟樂旺順留伏逆」节（只五星：木廟亥樂寅旺未、火廟卯樂戌旺丑、土子丑廟旺樂、金辰廟旺樂酉、水巳廟旺樂申）；
    /// 日月四余无此节，仍用总论句
    Jie,
}

/// 七强五弱
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QiangRuoRule {
    /// 《星學大成》卷三《論十二宫管庫星歌》「命宫官禄最髙位，福徳妻宫次强宗，近强男女并田宅，第九遷移合照宫；
    /// 五弱疾宫并相貌，兄弟奴僕八宫中」：遷移为强、財帛为弱。缺省
    DaCheng,
    /// 《張果星宗》卷一入门看法「强宫者，謂命宫、官禄、田宅、妻妾、男女、福德、財帛」（又云财帛次弱、迁移近强）：財帛为强、遷移为弱
    XingZong,
}

/// 大限各宫年数
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DaXianYears {
    /// 《張果星宗》卷三「定行限度法」：相貌十年、官禄十五、福德妻妾各十一、遷移八、疾厄七、奴僕男女田宅各四年半、兄弟財帛各五。
    /// 《星學大成》卷七《洞微百六限起例》「命宫十五貌宫十福徳妻宫十一詳官禄十五最髙位遷移止有八年粮疾厄七兮共六六
    /// 財帛兄弟五年强田宅子孫并奴僕四年之半」同（四库本，故作缺省）
    XingZong,
    /// 《星學大成》卷三限例「十三嵗出童限入寅，二十三卯，三十三辰，四十八巳，五十七午，六十四未，七十五申」所含年数：
    /// 福德十、遷移九，余同星宗（例只到妻妾，其后各宫借星宗）
    DaChengLi,
}

/// 宿度分宫宫界（地支序 子…亥）：该宫起于（宿下标，入宿古度）。《張果星宗》卷一「度數所在」
pub const GONG_JIE_XINGZONG: [(usize, f64); 12] =
    [(9, 2.0), (7, 4.0), (5, 3.0), (2, 1.0), (27, 10.0), (25, 15.0), (23, 3.0), (21, 8.0), (18, 6.0), (16, 3.0), (14, 1.0), (11, 12.0)];
/// 宿度分宫宫界，《星學大成》卷一（宫界歌）
pub const GONG_JIE_DACHENG: [(usize, f64); 12] =
    [(9, 1.0), (7, 3.0), (5, 3.0), (2, 1.0), (27, 9.0), (25, 14.0), (23, 3.0), (21, 8.0), (18, 6.0), (16, 3.0), (14, 1.0), (11, 12.0)];

/// 古度 / 度（周天 365¼ 古度）
const GU_DU: f64 = 365.25 / 360.0;

/// 十一曜五行（0 木 1 火 2 土 3 金 4 水）：日火之精、月水之精（《星學大成》卷十八「太陽日宿火之精」、卷十九「太隂月宿水之精」）；
/// 四余：炁木、孛水、羅火、計土（《張果星宗》卷四《緯克經論》「木星有用忌金……土星有用忌木炁……金星有用忌火羅……水星有用忌土計」；
/// 《星學大成》卷三月限「見木炁」「見火羅」并称）
pub const YAO_XING: [u8; 11] = [1, 4, 4, 3, 1, 0, 2, 1, 2, 4, 0];
/// 五行名
pub const XING: [&str; 5] = ["木", "火", "土", "金", "水"];

/// 宫主（地支序 → 十一曜下标）：子丑土、寅亥木、卯戌火、辰酉金、巳申水、午日、未月。
/// 《星學大成》卷二「午屬太陽未屬月，申巳水宿辰酉金，卯戌火星寅亥木，子丑遇土來」；《張果星宗》卷一同
pub const GONG_ZHU: [usize; 12] = [6, 6, 5, 4, 3, 2, 0, 1, 2, 3, 4, 5];

/// 宿主（度主），七曜周而复始：角木、亢金、氐土、房日、心月、尾火、箕水……
/// 《星學大成》卷三月限「亢牛婁鬼金度……斗奎井角木度……參箕軫壁水度……心危畢張月度……女氐柳胃土度……尾翼觜室火度」（房虛昴星日度由此推）；
/// 《張果星宗》卷一「度主者……房虛昴星度日」俱全
const XIU_ZHU7: [usize; 7] = [5, 3, 6, 0, 1, 4, 2];

/// 地支所主之宫主（十一曜下标）
pub fn gong_zhu(zhi: u8) -> usize {
    GONG_ZHU[(zhi % 12) as usize]
}
/// 宿（0 角 … 27 轸）之主（十一曜下标）
pub fn xiu_zhu(xiu: usize) -> usize {
    XIU_ZHU7[xiu % 7]
}

/// 《三辰通載》总论句（卷十四至二十二），按十一曜之序原文照录（「正廟」作廟）
const MIAO_LUN: [&str; 11] = [
    "廟戌樂巳好辰喜寅旺午怒酉",     // 太陽 卷十八
    "廟戌旺酉喜巳樂申順亥丑怒寅",   // 太隂 卷十九「順於亥丑二位，怒於寅」
    "廟午旺巳喜辰好亥子",           // 水 卷十七
    "廟辰旺亥樂酉好巳喜午",         // 金 卷十六
    "廟卯旺丑好戌樂未喜申好寅",     // 火 卷十五
    "廟亥樂寅好申喜未",             // 木 卷十四
    "廟丑樂子旺辰好戌喜午",         // 土 卷十六「正廟在丑」
    "廟午樂卯旺辰好酉喜戌怒申",     // 羅㬋 卷二十一
    "廟亥旺戌樂寅好未喜卯巳怒辰",   // 計都 卷二十二
    "廟未旺寅好巳樂酉喜申順子怒戌", // 月孛 卷二十
    "廟丑旺亥好寅樂申喜午戌怒未",   // 紫炁 卷十九「午戌宫安皆喜，惟未為怒」
];
/// 「廟樂旺順留伏逆」节（五星；空串表示无此节、用总论句）
const MIAO_JIE: [&str; 11] = ["", "", "廟巳旺巳樂申", "廟辰旺辰樂酉", "廟卯樂戌旺丑", "廟亥樂寅旺未", "廟子丑旺子丑樂子丑", "", "", "", ""];
const MIAO_KIND: [&str; 7] = ["廟", "樂", "旺", "喜", "好", "怒", "順"];
const ZHI_CH: &str = "子丑寅卯辰巳午未申酉戌亥";

/// 一曜之廟樂旺喜好怒（宫，地支序）
pub fn miao_wang(yao: usize, rule: MiaoWangRule) -> Vec<(&'static str, u8)> {
    let src = match rule {
        MiaoWangRule::Jie if !MIAO_JIE[yao].is_empty() => MIAO_JIE[yao],
        _ => MIAO_LUN[yao],
    };
    let mut out = Vec::new();
    let mut kind = "";
    for c in src.chars() {
        if let Some(k) = MIAO_KIND.iter().find(|k| k.starts_with(c)) {
            kind = k;
        } else if let Some(z) = ZHI_CH.chars().position(|x| x == c) {
            out.push((kind, z as u8));
        }
    }
    out
}

/// 十二宫（PALACES 下标）是否强宫
pub fn palace_strong(palace: usize, rule: QiangRuoRule) -> bool {
    match rule {
        QiangRuoRule::DaCheng => matches!(palace, 0 | 3 | 4 | 6 | 8 | 9 | 10),
        QiangRuoRule::XingZong => matches!(palace, 0 | 1 | 3 | 4 | 6 | 9 | 10),
    }
}

/// 大限各宫年数（PALACES 下标；命宫由童限定，此处为 0）
pub fn da_xian_years(palace: usize, rule: DaXianYears) -> f64 {
    const XZ: [f64; 12] = [0.0, 5.0, 5.0, 4.5, 4.5, 4.5, 11.0, 7.0, 8.0, 15.0, 11.0, 10.0];
    match (rule, palace) {
        (DaXianYears::DaChengLi, 10) => 10.0,
        (DaXianYears::DaChengLi, 8) => 9.0,
        _ => XZ[palace % 12],
    }
}

/// 童限（出童限之岁）：命度在命宫内距宫首（与相貌宫相接之界）`d` 古度，《星學大成》卷三
/// 「遇交宫隨宫交三度起一嵗，如立命在子，女二女三女四共三度起十一嵗，女五女六女七共三度起十二嵗，女八女九女十起十三嵗」，
/// 即 11 + ⌊d／3⌋（宫中十五度约十五六岁，合「命宫十五為例」）
pub fn tong_xian(d_gudu: Real) -> Real {
    real(11.0) + crate::math::real::from_i32((d_gudu / real(3.0)).to_i32_floor().max(0))
}

/// 小限：生年支 `birth_zhi` 起命宫，逐年逆行一宫，`year_zhi` 年所在之宫。《星學大成》卷三
/// 「從命宫起逆轉，如亥生人立命在丑，就丑上數亥，子年在子，丑年在亥，寅年在戌……」；
/// 《張果星宗》定小限例（集成本）「以生年支加在命宮，逆數至太歲宮是」。此按年支换宫；交宫在生日者见 [`xiao_xian_sui`]
pub fn xiao_xian(ming: u8, birth_zhi: u8, year_zhi: u8) -> u8 {
    ((ming as i32 - (year_zhi as i32 - birth_zhi as i32)).rem_euclid(12)) as u8
}

/// 小限按生日换宫：`sui` 为岁数（初生一岁，每过一生日加一），一岁在命宫，逐岁逆行一宫。
/// 《星學大成》卷七《論洞微行年小限》注「自一嵗從命宫起逆行一年移一宫遇生日即交也，不論男女皆同」；
/// 卷三「小限者是流年生日後交過宫」。与 [`xiao_xian`] 宫序相同，只差生日前后（年支已换而生日未到时，此仍在上一宫）
pub fn xiao_xian_sui(ming: u8, sui: u32) -> u8 {
    ((ming as i64 - (sui.max(1) as i64 - 1)).rem_euclid(12)) as u8
}

/// 月限：自起宫 `start`（小限宫或太阴宫，见 [`YueXianRule`]）起生月 `birth_month`，逐月逆行一宫，`month`（1—12）所在之宫。
/// 《星學大成》卷三「小限宫中起生月……如小限在申……正月看申宫二月未三月午四月巳依宫逐月逆週十二宫」
pub fn yue_xian(start: u8, birth_month: u8, month: u8) -> u8 {
    ((start as i32 - (month as i32 - birth_month as i32)).rem_euclid(12)) as u8
}

/// 月限起宫
pub fn yue_xian_start(rule: YueXianRule, xiao: u8, shen_gong: u8) -> u8 {
    match rule {
        YueXianRule::XiaoXian => xiao,
        YueXianRule::TaiYin => shen_gong,
    }
}

/// 童限逐岁所行之宫（`sui` 岁，至出童限止）。《星學大成》卷七《論童限》「一命二財三疾厄，四妻五福順行流，
/// 六嵗却從官祿起，看看移到命宫休」；《張果星宗》定童限例歌（集成本）「一命二財三疾厄，四妻五福各宮值，六歲官祿順行流，
/// 十五還歸本命宅，十九起限住三年，二十行限四年畢」。一至五岁命、財帛、疾厄、妻妾、福德；六岁官祿，此后顺支逐岁一宫，
/// 十五岁正回命宫（官祿 = 命宫顺三，六岁起九年至命，与歌合），其后住命宫至出童限
pub fn tong_xian_gong(ming: u8, sui: u32) -> u8 {
    let m = ming as i32;
    let z = match sui {
        0 | 1 => m,
        2 => m - 1,  // 財帛
        3 => m - 7,  // 疾厄
        4 => m - 6,  // 妻妾
        5 => m - 10, // 福德
        6..=15 => m + 3 + (sui as i32 - 6),
        _ => m,
    };
    z.rem_euclid(12) as u8
}

/// 流年十二神（《星學大成》卷三「十二神煞」歌：一太嵗、二太陽、三喪門、四太隂、五官符、六死符、七嵗破、八龍徳、
/// 九白虎、十天徳、十一吊客、十二病符，順行位）
pub const LIU_NIAN_SHEN: [&str; 12] = ["太嵗", "太陽", "喪門", "太隂", "官符", "死符", "嵗破", "龍徳", "白虎", "天徳", "吊客", "病符"];

/// 流年變曜（按规则）：`year_zhi` 年十二神各在年支顺数之宫，所变之曜即该宫宫主。
/// 返回（神名，宫，十一曜下标）。卷三例「今年丙戌子為喪門寅官符午白虎申吊客酉病符戌太嵗……則流土為喪門流木為官符
/// 流水為吊客流陽為白虎流火為太嵗」、「如酉年流金為太嵗龍徳……流隂為吊客」十二神逐一合此规则；
/// 「並從冬至日為始」（冬至换年支，调用处定 `year_zhi`）。卷三逐年變曜表另含天喜、驛馬、劫煞等，见 [`BIAN_YAO_TABLE`]
pub fn liu_nian_bian_yao(year_zhi: u8) -> [(&'static str, u8, usize); 12] {
    core::array::from_fn(|k| {
        let g = ((year_zhi as usize + k) % 12) as u8;
        (LIU_NIAN_SHEN[k], g, gong_zhu(g))
    })
}

/// 《星學大成》卷三「流年十二變曜」逐年原表（子年……亥年），每项「曜:所化神煞」原文照录（四库录文有数项被排作锚点标题，已归回本年）。
/// 曜字：土木火金水，陽 = 太陽，隂 = 太隂
pub const BIAN_YAO_TABLE: [&str; 12] = [
    "土:太嵗太陽天空 木:喪門病符亡神驛 火:太隂吊狗勾神 金:天喜天徳官符披 水:月徳刼虎㢘的 陽:天哭 隂:天厄",
    "土:太嵗病的 木:太陽鸞驛空吊劫 火:天徳披喪 金:太隂天解虎㢘的 水:天喜厄哭官 陽:月徳死符 隂:地解嵗破",
    "木:太嵗披劫 火:太陽天空虎㢘 金:哭喪暴的 水:太隂驛解勾破血刃 陽:官符 隂:天喜月徳死符 土:鸞吊病",
    "火:太嵗哭暴 金:太陽天空 水:月徳地解喪㢘的劫 陽:天喜太隂勾 隂:天解官符 木:白虎病符 土:天徳鸞披吊",
    "金:太嵗月徳地解死符 水:太陽喜空官符 陽:天解喪㢘 隂:太隂勾 火:嵗破病符 木:鸞驛厄哭吊 土:虎披絞",
    "水:太嵗天解太隂 陽:太陽天空 隂:喪㢘 金:天喜地解病官的 火:鸞月徳吊死 木:天徳驛披破劫 土:厄虎哭",
    "陽:太嵗 隂:太陽天空 水:驛爵喪病的 金:太隂鸞吊天解 火:天喜天徳地解官披 木:月徳死虎㢘刼 土:破哭厄",
    "隂:太嵗 水:太陽驛鸞空吊刼 金:天徳披喪 火:太隂解神虎㢘 木:天喜官哭厄 土:月徳死的 陽:病符",
    "水:太嵗天徳披刼 金:太陽空虎㢘的 火:天馬哭喪厄 木:太隂解神驛勾 土:天喜月徳官死 陽:吊 隂:鸞病",
    "金:太嵗爵哭厄 火:太陽天空 木:驛地解月徳喪㢘死刼 土:天喜解神太隂勾官 水:虎病的 陽:鸞天徳絞 隂:披吊",
    "火:太嵗死符 木:太陽天空飛符刼 土:太隂喪㢘血刃勾的 金:爵破病 水:厄哭吊驛亡 陽:虎披 隂:絞",
    "木:太嵗勾 土:太陽天空喪㢘 火:太隂喜病官 金:鸞月徳死的 水:驛天徳披絞刼 陽:地解厄 隂:虎哭",
];

/// 逐年原表之一年：（十一曜下标，所化神煞原文）
pub fn bian_yao_table(year_zhi: u8) -> Vec<(usize, &'static str)> {
    BIAN_YAO_TABLE[(year_zhi % 12) as usize]
        .split(' ')
        .filter_map(|w| {
            let (y, s) = w.split_once(':')?;
            let k = match y {
                "陽" => 0,
                "隂" => 1,
                "水" => 2,
                "金" => 3,
                "火" => 4,
                "木" => 5,
                "土" => 6,
                _ => return None,
            };
            Some((k, s))
        })
        .collect()
}

/// 大限一周：命宫 0 岁至童限 `tong`，其后顺支（相貌、福德、官祿……財帛）各依 [`da_xian_years`]。
/// 《星學大成》卷三「以限度言則一命宫二相貌三福徳四官禄……十二財帛，順數左旋」
pub fn da_xian(ming: u8, tong: Real, rule: DaXianYears) -> Vec<DaXian> {
    let mut v = vec![DaXian { gong: ming % 12, palace: PALACES[0], from: real(0.0), to: tong }];
    let mut age = tong;
    for k in 1..12u8 {
        let z = (ming + k) % 12;
        let p = ((ming as i32 - z as i32).rem_euclid(12)) as usize;
        let to = age + real(da_xian_years(p, rule));
        v.push(DaXian { gong: z, palace: PALACES[p], from: age, to });
        age = to;
    }
    v
}

/// 一宫大限
#[derive(Clone, Debug, PartialEq)]
pub struct DaXian {
    /// 宫（地支序）
    pub gong: u8,
    /// 宫名（PALACES）
    pub palace: &'static str,
    /// 起止之岁
    pub from: Real,
    pub to: Real,
}

/// 星命诸项
#[derive(Clone, Debug)]
pub struct XingMing {
    /// 命度黄经（°）：上升点法即上升点；卯上起命法按量天尺（同位相對）取命宫中与太阳同位之度，见 [`MingDuRule`]
    pub ming_lon: Real,
    /// 命度所在宿与入宿度（°）
    pub ming_xiu: Option<(usize, Real)>,
    /// 命宫主、命度主（十一曜下标）；命主依 [`MingZhuRule`]
    pub ming_gong_zhu: usize,
    pub ming_du_zhu: Option<usize>,
    pub ming_zhu: usize,
    /// 身宫（太阴所躔之宫）、身宫主、身度主、身主（依 [`ShenZhuRule`]）
    pub shen_gong: u8,
    pub shen_gong_zhu: usize,
    pub shen_du_zhu: Option<usize>,
    pub shen_zhu: usize,
    /// 十二宫之宫主（地支序 → 十一曜下标）
    pub gong_zhu: [usize; 12],
    /// 各曜（与 `QizhengChart::yao` 同序）所居之廟樂旺喜好怒順，及「入垣」（居自主之宫）、「升殿」（居自主之宿）
    pub dignity: Vec<Vec<&'static str>>,
    /// 各宫（地支序）强弱：true 为强
    pub strong: [bool; 12],
    /// 各曜对命主之五行关系：恩（生命主）、難（克命主）、比（同行）、子（命主所生）、制（命主所克）；命主本身为「主」
    pub relation: Vec<&'static str>,
    /// 出童限之岁
    pub tong_xian: Real,
    /// 大限：命宫（童限）起，顺支历相貌、福德、官祿……財帛一周
    pub da_xian: Vec<DaXian>,
    /// 童限逐岁之宫（下标 0 为一岁，至出童限前一岁），见 [`tong_xian_gong`]
    pub tong_xian_gong: Vec<u8>,
}

/// 五行关系：a 对 b（b 为主）
fn relation_of(a: u8, b: u8) -> &'static str {
    match (a as i32 - b as i32).rem_euclid(5) {
        0 => "比",
        4 => "恩", // a 生 b：a + 1 = b
        1 => "子", // b 生 a
        3 => "難", // a 克 b：a + 2 = b
        _ => "制", // b 克 a
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QizhengOptions {
    pub luo_ji: LuoJi,
    pub node: NodeKind,
    pub bei: BeiKind,
    pub zi_qi: ZiQiRule,
    pub ming: MingRule,
    pub gong: GongRule,
    pub ming_zhu: MingZhuRule,
    pub shen_zhu: ShenZhuRule,
    pub miao_wang: MiaoWangRule,
    pub qiang_ruo: QiangRuoRule,
    pub da_xian: DaXianYears,
    pub ming_du: MingDuRule,
    pub yue_xian: YueXianRule,
}

impl Default for QizhengOptions {
    fn default() -> Self {
        Self {
            luo_ji: LuoJi::JiAscending,
            node: NodeKind::True,
            bei: BeiKind::MeanApogee,
            zi_qi: ZiQiRule::ShiXian,
            ming: MingRule::SunOnHour,
            gong: GongRule::ZhongQi,
            ming_zhu: MingZhuRule::DuZhu,
            shen_zhu: ShenZhuRule::DuZhu,
            miao_wang: MiaoWangRule::Lun,
            qiang_ruo: QiangRuoRule::DaCheng,
            da_xian: DaXianYears::XingZong,
            ming_du: MingDuRule::FromStart,
            yue_xian: YueXianRule::XiaoXian,
        }
    }
}

/// 一曜所在
#[derive(Clone, Debug, PartialEq)]
pub struct YaoPos {
    pub name: &'static str,
    /// 当日视黄经（°，0—360）
    pub lon: Real,
    /// 视黄纬（°）；四余为 None
    pub lat: Option<Real>,
    /// 所在宫（地支序）与入宫度（°，0—30）
    pub gong: u8,
    pub gong_deg: Real,
    /// 所在宿（0 角 … 27 轸）与入宿度（°，黄道宿度）；未给距星时为 None
    pub xiu: Option<(usize, Real)>,
    /// 逆行（七政按一日黄经变化判；四余之罗计恒逆，孛炁恒顺）
    pub retro: bool,
}

/// 命盘
#[derive(Clone, Debug)]
pub struct QizhengChart {
    pub yao: Vec<YaoPos>,
    /// 命宫（地支序）
    pub ming: u8,
    /// 十二宫：下标为地支，值为宫名
    pub palaces: [&'static str; 12],
    /// 星命诸项（宫主、度主、命主、身主、廟旺、强弱、恩難、限）
    pub xing_ming: XingMing,
}

fn deg(x: Real) -> Real {
    x * real(180.0 / core::f64::consts::PI)
}
fn rad(x: Real) -> Real {
    x * real(core::f64::consts::PI / 180.0)
}
fn norm360(x: Real) -> Real {
    deg(rad(x).wrap_to_2pi())
}

/// 黄经（°）所在之宫（地支序）：以中气分宫，0°—30° 戌宫，逆支而行
pub fn gong_of(lon: Real) -> (u8, Real) {
    let l = norm360(lon);
    let k = (l / real(30.0)).to_i32_floor();
    (((10 - k).rem_euclid(12)) as u8, l - real(30.0) * real(k))
}

/// 儒略世纪数（TT，J2000 起）
fn centuries(t: TimePoint) -> Real {
    (t.to_scale(TimeScale::TT).jd - real(2451545.0)) / real(36525.0)
}

/// 月球平升交点黄经（°，平春分点）：Meeus 式 47.7
pub fn mean_ascending_node(t: TimePoint) -> Real {
    let c = centuries(t);
    norm360(real(125.0445479) - real(1934.1362891) * c + real(0.0020754) * c * c + c * c * c / real(467441.0) - c * c * c * c / real(60616000.0))
}

/// 月球平近地点黄经（°）：Meeus 式 47.7；平远地点 = 此 + 180°
pub fn mean_perigee(t: TimePoint) -> Real {
    let c = centuries(t);
    norm360(real(83.3532465) + real(4069.0137287) * c - real(0.01032) * c * c - c * c * c / real(80053.0) + c * c * c * c / real(18999000.0))
}

/// 黄经章动（°）
fn delta_psi(t: TimePoint) -> Real {
    deg(nutation_for_model(centuries(t), NutationModel::IAU2000A).0.rad())
}

/// 真黄赤交角（rad）
fn true_obliquity(t: TimePoint) -> Real {
    let c = centuries(t);
    mean_obliquity(c).rad() + nutation_for_model(c, NutationModel::IAU2000A).1.rad()
}

/// 月之视位置（当日真黄道直角坐标，单位：地球半径不论，取 距离 m）
fn moon_vec<P: EphemerisProvider>(eph: &P, t: TimePoint, opts: &ApparentPipelineOptions) -> [Real; 3] {
    let (lon, lat, r) = body_apparent_ecliptic(eph, Body::Moon, t, opts);
    let (l, b, d) = (lon.rad(), lat.rad(), r.meters());
    [d * b.cos() * l.cos(), d * b.cos() * l.sin(), d * b.sin()]
}

fn cross(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// 月之瞬时升交点与远地点（当日真黄道视黄经，°）：由前后各半小时的视位置差分得速度，
/// 角动量 h = r × v 之黄道投影定升交点（n = ẑ × h），偏心率矢量 e = v × h / μ − r̂ 之反向为远地点。
pub fn osculating_node_apogee<P: EphemerisProvider>(eph: &P, t: TimePoint, opts: &ApparentPipelineOptions) -> (Real, Real) {
    let dt = real(1.0 / 48.0);
    let jd = t.to_scale(TimeScale::TT).jd;
    let p0 = moon_vec(eph, TimePoint::new(TimeScale::TT, jd - dt), opts);
    let p1 = moon_vec(eph, TimePoint::new(TimeScale::TT, jd + dt), opts);
    let r = moon_vec(eph, t, opts);
    let secs = real(2.0 * 86400.0) * dt;
    let v = [(p1[0] - p0[0]) / secs, (p1[1] - p0[1]) / secs, (p1[2] - p0[2]) / secs];
    let h = cross(r, v);
    let node = norm360(deg(h[0].atan2(-h[1])));
    // μ = G(M⊕ + M☾)
    let mu = real(3.986004418e14 * (1.0 + 1.0 / 81.30056));
    let vh = cross(v, h);
    let rn = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    let e = [vh[0] / mu - r[0] / rn, vh[1] / mu - r[1] / rn, vh[2] / mu - r[2] / rn];
    // e 指近地点；远地点反向。取其黄道投影之黄经
    let apogee = norm360(deg(e[1].atan2(e[0])) + real(180.0));
    (node, apogee)
}

/// 二十八宿距星的视黄经（°）：由距星视赤经赤纬与真黄赤交角求
pub fn mansion_longitudes(frame: &MansionFrame, t: TimePoint) -> [Real; 28] {
    let eps = true_obliquity(t);
    core::array::from_fn(|i| deg(ecliptic_longitude(frame.ra[i], frame.dec[i], eps)))
}

/// 黄道宿度：黄经 `lon`（°）所在之宿（西侧最近之距星）与入宿度（°）
pub fn xiu_of(mansion_lons: &[Real; 28], lon: Real) -> (usize, Real) {
    let mut best = (0usize, real(f64::MAX));
    for (i, m) in mansion_lons.iter().enumerate() {
        let d = norm360(lon - *m);
        if d < best.1 {
            best = (i, d);
        }
    }
    best
}

/// 以宿度分宫：黄经 `lon` 所在之宫（地支序）与距宫首之度（°）。宫首 = 宫界宿距星黄经 + 入宿古度（折为度）
pub fn gong_of_xiu(mansion_lons: &[Real; 28], lon: Real, jie: &[(usize, f64); 12]) -> (u8, Real) {
    let mut best = (0u8, real(f64::MAX));
    for (z, (x, d)) in jie.iter().enumerate() {
        let b = mansion_lons[*x] + real(*d / GU_DU);
        let dd = norm360(lon - b);
        if dd < best.1 {
            best = (z as u8, dd);
        }
    }
    best
}

/// 十二宫宫首黄经（°，黄经小端；地支序）：中气分宫各 30°，宿度分宫在宫界宿度（宫界宿距星黄经 + 古度）
pub fn gong_starts(jie: Option<&[(usize, f64); 12]>, mlon: Option<&[Real; 28]>) -> [Real; 12] {
    core::array::from_fn(|z| match (jie, mlon) {
        (Some(j), Some(ml)) => norm360(ml[j[z].0] + real(j[z].1 / GU_DU)),
        _ => real(30.0) * real(((10 - z as i32).rem_euclid(12)) as f64),
    })
}

/// 量天尺（同位相對）：太阳黄经 `sun_lon` 在 `sun_gong` 宫之位，移至 `ming` 宫同位，得命度黄经（°）。
/// `starts` 为各宫宫首（[`gong_starts`]）；宫宽 = 逆支下一宫宫首 − 本宫宫首。同位超出命宫宽者取命宫边上。见 [`MingDuRule`]
pub fn liang_tian_chi(starts: &[Real; 12], sun_lon: Real, sun_gong: u8, ming: u8, rule: MingDuRule) -> Real {
    let width = |z: u8| {
        let w = norm360(starts[((z + 11) % 12) as usize] - starts[z as usize]);
        if w < real(1e-9) { real(360.0) } else { w }
    };
    let eps = real(1e-9);
    let (ws, wm) = (width(sun_gong), width(ming));
    let d = norm360(sun_lon - starts[sun_gong as usize]).min(ws);
    let pos = match rule {
        MingDuRule::FromStart => d.min(wm - eps),
        MingDuRule::FromEnd => (wm - (ws - d)).max(real(0.0)).min(wm - eps),
    };
    norm360(starts[ming as usize] + pos)
}

/// 命宫（地支序）：太阳所躔之宫 `sun_gong` 加生时 `hour_zhi`，卯上所临之宫
pub fn ming_sun_on_hour(sun_gong: u8, hour_zhi: u8) -> u8 {
    ((sun_gong as i32 + 3 - hour_zhi as i32).rem_euclid(12)) as u8
}

/// 天文上升点黄经（°）：地方恒星时 `lst`（°）、纬度 `lat`（°）、真黄赤交角 `eps`（rad）
pub fn ascendant(lst_deg: Real, lat_deg: Real, eps: Real) -> Real {
    let th = rad(lst_deg);
    let phi = rad(lat_deg);
    // 黄道与地平之交有二（升点、降点）：tan λ = −cos θ ／（sin ε tan φ + cos ε sin θ）；取在东方者（时角 sin H < 0）
    let l1 = (-th.cos()).atan2(eps.sin() * phi.tan() + eps.cos() * th.sin());
    let east = |l: Real| {
        let ra = (l.sin() * eps.cos()).atan2(l.cos());
        (th - ra).sin() < real(0.0)
    };
    let l = if east(l1) { l1 } else { l1 + real(core::f64::consts::PI) };
    norm360(deg(l))
}

/// 排七政四余。`t` 为时刻；`frame` 为同历元二十八宿距星视位置（None 则不算宿度）；`hour_zhi` 为生时地支；
/// `lst_lat` 为（地方视恒星时 °，纬度 °），仅 [`MingRule::Ascendant`] 用。
pub fn chart<P: EphemerisProvider>(
    eph: &P,
    t: TimePoint,
    frame: Option<&MansionFrame>,
    hour_zhi: u8,
    lst_lat: Option<(Real, Real)>,
    o: &QizhengOptions,
    opts: &ApparentPipelineOptions,
) -> QizhengChart {
    let mlon = frame.map(|f| mansion_longitudes(f, t));
    let jd = t.to_scale(TimeScale::TT).jd;
    let jie = match (o.gong, mlon.is_some()) {
        (GongRule::XiuDuXingZong, true) => Some(&GONG_JIE_XINGZONG),
        (GongRule::XiuDuDaCheng, true) => Some(&GONG_JIE_DACHENG),
        _ => None,
    };
    let gong_fn = |lon: Real| match (jie, mlon.as_ref()) {
        (Some(j), Some(m)) => gong_of_xiu(m, lon, j),
        _ => gong_of(lon),
    };
    let mk = |name: &'static str, lon: Real, lat: Option<Real>, retro: bool| {
        let (gong, gong_deg) = gong_fn(lon);
        YaoPos { name, lon: norm360(lon), lat, gong, gong_deg, xiu: mlon.as_ref().map(|m| xiu_of(m, lon)), retro }
    };
    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Planet(Planet::Mercury),
        Body::Planet(Planet::Venus),
        Body::Planet(Planet::Mars),
        Body::Planet(Planet::Jupiter),
        Body::Planet(Planet::Saturn),
    ];
    let mut yao = Vec::with_capacity(11);
    for (k, b) in bodies.iter().enumerate() {
        let (lon, lat, _) = body_apparent_ecliptic(eph, *b, t, opts);
        let (lon2, _, _) = body_apparent_ecliptic(eph, *b, TimePoint::new(TimeScale::TT, jd + real(1.0)), opts);
        let dl = (lon2.rad() - lon.rad()).wrap_to_signed_pi();
        yao.push(mk(YAO[k], deg(lon.rad()), Some(deg(lat.rad())), dl < real(0.0)));
    }
    // 四余：平根数在平春分点，加黄经章动归当日真春分点，与七政同一基准
    let dpsi = delta_psi(t);
    let (asc_node, apogee) = match (o.node, o.bei) {
        (NodeKind::Mean, BeiKind::MeanApogee) => (mean_ascending_node(t) + dpsi, mean_perigee(t) + real(180.0) + dpsi),
        _ => {
            let (n, a) = osculating_node_apogee(eph, t, opts);
            (
                if o.node == NodeKind::Mean { mean_ascending_node(t) + dpsi } else { n },
                if o.bei == BeiKind::MeanApogee { mean_perigee(t) + real(180.0) + dpsi } else { a },
            )
        }
    };
    let desc_node = asc_node + real(180.0);
    let (luo, ji) = match o.luo_ji {
        LuoJi::JiAscending => (desc_node, asc_node),
        LuoJi::LuoAscending => (asc_node, desc_node),
    };
    yao.push(mk("羅睺", luo, None, true));
    yao.push(mk("計都", ji, None, true));
    yao.push(mk("月孛", apogee, None, false));
    if o.zi_qi == ZiQiRule::ShiXian {
        let (j0, l0, p) = ZIQI_SHIXIAN;
        yao.push(mk("紫炁", real(l0) + real(360.0) * (jd - real(j0)) / real(p), None, false));
    }
    let sun_gong = yao[0].gong;
    let (ming, ming_lon) = match (o.ming, lst_lat) {
        (MingRule::Ascendant, Some((lst, lat))) => {
            let a = ascendant(lst, lat, true_obliquity(t));
            (gong_fn(a).0, a)
        }
        _ => {
            let m = ming_sun_on_hour(sun_gong, hour_zhi);
            let starts = gong_starts(jie, mlon.as_ref());
            let lon = liang_tian_chi(&starts, yao[0].lon, sun_gong, m, o.ming_du);
            (m, lon)
        }
    };
    let palaces = core::array::from_fn(|z| PALACES[((ming as i32 - z as i32).rem_euclid(12)) as usize]);
    let xing_ming = xing_ming(&yao, ming, ming_lon, gong_fn(ming_lon).1, mlon.as_ref(), o);
    QizhengChart { yao, ming, palaces, xing_ming }
}

/// 由诸曜位置、命宫、命度推星命诸项
fn xing_ming(yao: &[YaoPos], ming: u8, ming_lon: Real, ming_gong_deg: Real, mlon: Option<&[Real; 28]>, o: &QizhengOptions) -> XingMing {
    let ming_xiu = mlon.map(|m| xiu_of(m, ming_lon));
    let ming_gong_zhu = gong_zhu(ming);
    let ming_du_zhu = ming_xiu.map(|(x, _)| xiu_zhu(x));
    let ming_zhu = match (o.ming_zhu, ming_du_zhu) {
        (MingZhuRule::DuZhu, Some(d)) => d,
        _ => ming_gong_zhu,
    };
    let moon = &yao[1];
    let shen_gong = moon.gong;
    let shen_gong_zhu = gong_zhu(shen_gong);
    let shen_du_zhu = moon.xiu.map(|(x, _)| xiu_zhu(x));
    let shen_zhu = match (o.shen_zhu, shen_du_zhu) {
        (ShenZhuRule::DuZhu, Some(d)) => d,
        _ => shen_gong_zhu,
    };
    let palace_of = |z: u8| ((ming as i32 - z as i32).rem_euclid(12)) as usize;
    let dignity = yao
        .iter()
        .enumerate()
        .map(|(k, y)| {
            let mut v: Vec<&'static str> = miao_wang(k, o.miao_wang).into_iter().filter(|(_, z)| *z == y.gong).map(|(kd, _)| kd).collect();
            if gong_zhu(y.gong) == k {
                v.push("入垣");
            }
            if y.xiu.is_some_and(|(x, _)| xiu_zhu(x) == k) {
                v.push("升殿");
            }
            v
        })
        .collect();
    let relation = (0..yao.len()).map(|k| if k == ming_zhu { "主" } else { relation_of(YAO_XING[k], YAO_XING[ming_zhu]) }).collect();
    let tong = tong_xian(ming_gong_deg * real(GU_DU));
    let da_xian = da_xian(ming, tong, o.da_xian);
    let tong_xian_gong = (1..tong.to_i32_floor().max(1) as u32).map(|s| tong_xian_gong(ming, s)).collect();
    XingMing {
        ming_lon,
        ming_xiu,
        ming_gong_zhu,
        ming_du_zhu,
        ming_zhu,
        shen_gong,
        shen_gong_zhu,
        shen_du_zhu,
        shen_zhu,
        gong_zhu: GONG_ZHU,
        dignity,
        strong: core::array::from_fn(|z| palace_strong(palace_of(z as u8), o.qiang_ruo)),
        relation,
        tong_xian: tong,
        da_xian,
        tong_xian_gong,
    }
}

/// 宿名
pub fn xiu_name(i: usize) -> &'static str {
    MANSIONS[i % 28]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gong_boundaries() {
        assert_eq!(gong_of(real(0.5)).0, 10); // 戌 降婁
        assert_eq!(gong_of(real(330.85)).0, 11); // 亥：道光十三年正月初一日躔亥宫初度五十一分
        assert_eq!(gong_of(real(275.0)).0, 1); // 丑 星紀（冬至后）
        assert!((gong_of(real(330.85)).1.as_f64() - 0.85).abs() < 1e-9);
    }

    #[test]
    fn ming_rule() {
        // 卯时生，命宫即太阳宫；午时生，命宫为太阳宫逆三（天文上升点约在太阳东 90°）
        assert_eq!(ming_sun_on_hour(11, 3), 11);
        assert_eq!(ming_sun_on_hour(11, 6), 8);
        assert_eq!(ming_sun_on_hour(0, 0), 3);
    }

    /// 宫主、宿主：《星學大成》卷二宫主歌、卷三月限宿度五行；《張果星宗》卷一宫度偏正垣「虛日坐命度，主是日，宫主是土」
    #[test]
    fn gong_xiu_zhu() {
        let z = |c: char| ZHI_CH.chars().position(|x| x == c).unwrap() as u8;
        for (zs, y) in [("午", 0), ("未", 1), ("巳申", 2), ("辰酉", 3), ("卯戌", 4), ("寅亥", 5), ("子丑", 6)] {
            for c in zs.chars() {
                assert_eq!(gong_zhu(z(c)), y, "{c}");
            }
        }
        let x = |c: &str| MANSIONS.iter().position(|m| *m == c).unwrap();
        for (xs, y) in [("亢牛娄鬼", 3), ("斗奎井角", 5), ("参箕轸壁", 2), ("心危毕张", 1), ("女氐柳胃", 6), ("尾翼觜室", 4), ("房虚昴星", 0)] {
            for c in xs.chars() {
                assert_eq!(xiu_zhu(x(&c.to_string())), y, "{c}");
            }
        }
        assert_eq!((xiu_zhu(x("虚")), gong_zhu(z('子'))), (0, 6));
        // 卷二《論學堂》：太隂在人馬宫（寅）是身宫，木為宫主星
        assert_eq!(YAO[gong_zhu(z('寅'))], "木星");
    }

    /// 《星學大成》卷三限例
    #[test]
    fn xian_examples() {
        // 童限：立命在子，女二至四起十一歲，女五至七十二歲，女八至十十三歲（距宫首古度 0—3、3—6、6—9）
        for (d, a) in [(0.5, 11.0), (2.9, 11.0), (3.5, 12.0), (6.5, 13.0), (15.0, 16.0)] {
            assert_eq!(tong_xian(real(d)).as_f64(), a, "{d}");
        }
        // 大限：「十三嵗出童限入寅，二十三卯，三十三辰，四十八巳，五十七午，六十四未，七十五申」（命宫丑）
        let book = [(2u8, 13.0), (3, 23.0), (4, 33.0), (5, 48.0), (6, 57.0), (7, 64.0), (8, 75.0)];
        let li = da_xian(1, real(13.0), DaXianYears::DaChengLi);
        let xz = da_xian(1, real(13.0), DaXianYears::XingZong);
        for (k, (g, a)) in book.iter().enumerate() {
            assert_eq!((li[k + 1].gong, li[k + 1].from.as_f64()), (*g, *a));
        }
        // 星宗年数：卯、辰两宫起岁各晚一年，巳以后复合
        let xz_from: Vec<f64> = xz[1..8].iter().map(|d| d.from.as_f64()).collect();
        assert_eq!(xz_from, vec![13.0, 23.0, 34.0, 49.0, 57.0, 64.0, 75.0]);
        assert_eq!(li[1].palace, "相貌");
        assert_eq!(li[3].palace, "官祿");
        // 小限：亥生人立命在丑，就丑上數亥，子年在子，丑年在亥……戌年在寅
        let z = |c: char| ZHI_CH.chars().position(|x| x == c).unwrap() as u8;
        let book = "亥丑 子子 丑亥 寅戌 卯酉 辰申 巳未 午午 未巳 申辰 酉卯 戌寅";
        for w in book.split(' ') {
            let c: Vec<char> = w.chars().collect();
            assert_eq!(xiao_xian(z('丑'), z('亥'), z(c[0])), z(c[1]), "{w}");
        }
    }

    /// 恩難：《星學大成》卷三月限「命坐巳未申三宫以月水為主，喜金水月木，畏火羅土計」；
    /// 「辰酉安命以金為主，流月喜逢胃柳女氐（土度），忌尾室翼觜（火度）」
    #[test]
    fn en_nan() {
        let r = |a: usize, b: usize| relation_of(YAO_XING[a], YAO_XING[b]);
        // 水为主（下标 2）：金恩、水月比、木子；火羅制、土計難
        assert_eq!([r(3, 2), r(1, 2), r(5, 2), r(4, 2), r(7, 2), r(6, 2), r(8, 2)], ["恩", "比", "子", "制", "制", "難", "難"]);
        // 金为主（3）：土計恩，火羅日難
        assert_eq!([r(6, 3), r(8, 3), r(4, 3), r(7, 3), r(0, 3)], ["恩", "恩", "難", "難", "難"]);
    }

    /// 廟樂旺：《三辰通載》总论句与「廟樂旺」节
    #[test]
    fn miao_wang_tables() {
        assert!(miao_wang(5, MiaoWangRule::Lun).contains(&("廟", 11))); // 木廟亥（卷十四）
        assert!(miao_wang(0, MiaoWangRule::Lun).contains(&("怒", 9))); // 太陽怒酉（卷十八）
        assert!(miao_wang(8, MiaoWangRule::Lun).contains(&("喜", 5))); // 計都喜卯巳（卷二十二）
        assert!(miao_wang(4, MiaoWangRule::Lun).contains(&("樂", 7))); // 火樂未（论句）
        assert!(miao_wang(4, MiaoWangRule::Jie).contains(&("樂", 10))); // 火樂戌（节「火入白羊戌上」）
        assert_eq!(miao_wang(6, MiaoWangRule::Jie).len(), 6); // 土子丑廟旺樂
        assert_eq!(miao_wang(0, MiaoWangRule::Jie), miao_wang(0, MiaoWangRule::Lun)); // 日无节，用论句
        assert!(palace_strong(8, QiangRuoRule::DaCheng) && !palace_strong(1, QiangRuoRule::DaCheng));
        assert!(!palace_strong(8, QiangRuoRule::XingZong) && palace_strong(1, QiangRuoRule::XingZong));
    }

    /// 宿度分宫：宫首在界宿距星 + 古度
    #[test]
    fn gong_by_xiu() {
        let m: [Real; 28] = core::array::from_fn(|i| real(i as f64 * 360.0 / 28.0));
        let b = |z: usize| (m[GONG_JIE_XINGZONG[z].0] + real(GONG_JIE_XINGZONG[z].1 / GU_DU)).as_f64();
        let (g, d) = gong_of_xiu(&m, real(b(0) + 0.1), &GONG_JIE_XINGZONG);
        assert_eq!(g, 0);
        assert!((d.as_f64() - 0.1).abs() < 1e-9);
        assert_eq!(gong_of_xiu(&m, real(b(0) - 0.1), &GONG_JIE_XINGZONG).0, 1); // 女二之前属丑
    }

    #[test]
    fn ascendant_sanity() {
        // 赤道上、恒星时 0（春分点上中天）：上升点为黄经 90°
        let eps = rad(real(23.44));
        assert!((ascendant(real(0.0), real(0.0), eps).as_f64() - 90.0).abs() < 1e-9);
        // 恒星时 90°（夏至点附近上中天）：上升点为秋分点 180°
        assert!((ascendant(real(90.0), real(0.0), eps).as_f64() - 180.0).abs() < 1e-9);
        // 北纬 40°、恒星时 0：上升点在巨蟹宫一带（90° 以上）
        assert!(ascendant(real(0.0), real(40.0), eps).as_f64() > 90.0);
    }

    /// 《大清道光十三年七政經緯躔度時憲書》逐旬四余宿度（转录见 examples/qizheng_1833.rs、doc/folk/qizheng.md）：
    /// 缺省（计都升交、真交点、平远地点）下罗计、月孛差 30′ 内；正月初一太阳差 2′ 内
    #[test]
    fn daoguang13_shixianshu() {
        use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
        use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
        use crate::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
        use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
        use crate::astronomy::pipeline::{Vsop87Elp, Vsop87Planets};
        use crate::astronomy::sky_culture::chinese::load_lunar_mansion_systems_from_repo;
        use crate::astronomy::stars::load_hipparcos_bright_from_repo;
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::DEFAULT).unwrap();
        let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
            .iter()
            .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::DEFAULT).unwrap()))
            .collect();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
        let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let sys = load_lunar_mansion_systems_from_repo().unwrap().into_iter().find(|s| s.id == "qing").unwrap();
        let opts = ApparentPipelineOptions::pipeline_default();
        // (距正月初一日数, 月孛, 罗睺, 计都)：(宿, 度+分/60)
        let book: [(i64, (&str, f64), (&str, f64), (&str, f64)); 6] = [
            (0, ("女", 4.733), ("斗", 15.733), ("井", 20.6)),
            (20, ("女", 7.017), ("斗", 14.517), ("井", 19.383)),
            (39, ("女", 9.15), ("斗", 12.717), ("井", 17.583)),
            (59, ("女", 11.367), ("斗", 10.583), ("井", 15.45)),
            (79, ("虚", 1.867), ("斗", 8.767), ("井", 13.633)),
            (108, ("虚", 4.967), ("斗", 7.35), ("井", 12.217)),
        ];
        let jd0 = 2390599.5 - 116.4 / 360.0;
        for (dd, bei, luo, ji) in book {
            let t = TimePoint::new(TimeScale::TT, real(jd0 + dd as f64));
            let frame = MansionFrame::new(&eph, &cat, &sys, t, &opts).unwrap();
            let c = chart(&eph, t, Some(&frame), 0, None, &QizhengOptions::default(), &opts);
            for (name, (x, d)) in [("月孛", bei), ("羅睺", luo), ("計都", ji)] {
                let y = c.yao.iter().find(|y| y.name == name).unwrap();
                let (xi, du) = y.xiu.unwrap();
                assert_eq!(MANSIONS[xi], x, "{dd} {name}");
                assert!((du.as_f64() - d).abs() * 60.0 < 30.0, "{dd} {name}: {} vs {d}", du.as_f64());
            }
            if dd == 0 {
                assert!((c.yao[0].lon.as_f64() - (330.0 + 51.0 / 60.0)).abs() * 60.0 < 2.0);
                assert_eq!(c.yao[0].gong, 11);
                // 星命：子时生，命宫 = 亥 + 3 − 0 = 寅；身宫为太阴之宫；大限一周十二宫
                let x = &c.xing_ming;
                assert_eq!(c.ming, 2);
                assert_eq!(x.ming_gong_zhu, 5);
                assert_eq!(x.ming_zhu, xiu_zhu(x.ming_xiu.unwrap().0));
                assert_eq!(x.shen_gong, c.yao[1].gong);
                assert_eq!(x.relation[x.ming_zhu], "主");
                assert_eq!(x.da_xian.len(), 12);
                assert_eq!(x.da_xian[1].palace, "相貌");
                // 宿度分宫：各曜宫与中气分宫至多差一宫
                for g in [GongRule::XiuDuXingZong, GongRule::XiuDuDaCheng] {
                    let c2 = chart(&eph, t, Some(&frame), 0, None, &QizhengOptions { gong: g, ..Default::default() }, &opts);
                    for (a, b) in c.yao.iter().zip(&c2.yao) {
                        let dz = (a.gong as i32 - b.gong as i32).rem_euclid(12);
                        assert!(dz == 0 || dz == 1 || dz == 11, "{:?} {} {} {}", g, a.name, a.gong, b.gong);
                        assert!(b.gong_deg.as_f64() < 40.0);
                    }
                    eprintln!("{g:?}: 太阳 {} 宫 {:.2}°，命 {}", c2.yao[0].gong, c2.yao[0].gong_deg.as_f64(), c2.ming);
                }
                eprintln!("命度 {:?} 命主 {} 身主 {} 童限 {} 廟旺 {:?}", x.ming_xiu, YAO[x.ming_zhu], YAO[x.shen_zhu], x.tong_xian.as_f64(), x.dignity);
            }
        }
    }

    /// 授时赤道宿度（古度，《元史·曆志》）造一套等效距星黄经，用于按书中宿度检验量天尺与童限
    fn shoushi_mansions() -> [Real; 28] {
        const W: [f64; 28] = [
            12.1, 9.2, 16.3, 5.6, 6.5, 19.1, 10.4, 25.2, 7.2, 11.35, 8.9575, 15.4, 17.1, 8.6, 16.6, 11.8, 15.6, 11.3, 17.4, 0.05, 11.1,
            33.3, 2.2, 13.3, 6.3, 17.25, 18.75, 17.3,
        ];
        let mut acc = 0.0;
        core::array::from_fn(|i| {
            let v = acc / GU_DU;
            acc += W[i];
            real(v)
        })
    }
    fn xiu_idx(c: &str) -> usize {
        MANSIONS.iter().position(|m| *m == c).unwrap()
    }

    /// 量天尺：集成本《張果星宗》安命度法二例；定限度法三例；《星學大成》卷三「八月十四日太陽在巳翼十度……十三嵗出童限」
    #[test]
    fn liang_tian_chi_examples() {
        let m = shoushi_mansions();
        let starts = gong_starts(Some(&GONG_JIE_XINGZONG), Some(&m));
        let at = |x: &str, d: f64| norm360(m[xiu_idx(x)] + real(d / GU_DU));
        let gu = |lon: Real| {
            let (x, d) = xiu_of(&m, lon);
            (MANSIONS[x], d.as_f64() * GU_DU)
        };
        // 虛六（子宫）對午宫星五；女三對柳四
        for ((sx, sd), (mx, md)) in [(("虚", 6.0), ("星", 5.0)), (("女", 3.0), ("柳", 4.0))] {
            let s = at(sx, sd);
            assert_eq!(gong_of_xiu(&m, s, &GONG_JIE_XINGZONG).0, 0);
            let (x, d) = gu(liang_tian_chi(&starts, s, 0, 6, MingDuRule::FromStart));
            assert_eq!(x, mx);
            assert!((d - md).abs() < 0.1, "{sx}{sd} → {x}{d:.2}");
            let (x2, d2) = gu(liang_tian_chi(&starts, s, 0, 6, MingDuRule::FromEnd));
            eprintln!("安命度法 {sx}{sd}：自宫首 {x}{d:.2}，自宫尾 {x2}{d2:.2}（书 {mx}{md}）");
        }
        // 定限度法（命在午）：柳三四五 → 十一歲；星五 → 书十五歲（授时宿度得十六）；張十一至十三 → 二十歲
        let tong_at = |x: &str, d: f64| {
            let lon = at(x, d);
            let (g, dd) = gong_of_xiu(&m, lon, &GONG_JIE_XINGZONG);
            assert_eq!(g, 6);
            tong_xian(dd * real(GU_DU)).as_f64()
        };
        // 某宿某度取该度之始（入宿整度）
        for d in [3.0, 4.0, 5.0] {
            assert_eq!(tong_at("柳", d), 11.0);
        }
        assert_eq!(tong_at("星", 5.0), 16.0); // 书十五，差一年（授时柳宽 13.3；书若柳宽十二则合，见文档）
        for d in [11.0, 12.0, 13.0] {
            assert_eq!(tong_at("张", d), 20.0);
        }
        assert_eq!(tong_at("张", 13.5), 21.0); // 張十三度之末已过 30 古度
        // 大成卷三：太陽巳宫翼十度，命丑，书十三嵗出童限
        let s = at("翼", 10.5);
        assert_eq!(gong_of_xiu(&m, s, &GONG_JIE_XINGZONG).0, 5);
        for (rule, jie) in [(MingDuRule::FromStart, &GONG_JIE_XINGZONG), (MingDuRule::FromEnd, &GONG_JIE_XINGZONG), (MingDuRule::FromStart, &GONG_JIE_DACHENG), (MingDuRule::FromEnd, &GONG_JIE_DACHENG)] {
            let st = gong_starts(Some(jie), Some(&m));
            let l = liang_tian_chi(&st, s, gong_of_xiu(&m, s, jie).0, 1, rule);
            let (g, dd) = gong_of_xiu(&m, l, jie);
            assert_eq!(g, 1);
            eprintln!("大成卷三例 {rule:?} {}：命度 {:?} 童限 {}", if jie[5].1 == 15.0 { "星宗宫界" } else { "大成宫界" }, gu(l), tong_xian(dd * real(GU_DU)).as_f64());
        }
        // 中气分宫：二法皆即「移整宫」
        let st = gong_starts(None, None);
        for r in [MingDuRule::FromStart, MingDuRule::FromEnd] {
            assert!((liang_tian_chi(&st, real(335.0), 11, 2, r).as_f64() - 245.0).abs() < 1e-9);
        }
    }

    /// 小限、月限、童限逐岁宫
    #[test]
    fn xiao_yue_tong_examples() {
        let z = |c: char| ZHI_CH.chars().position(|x| x == c).unwrap() as u8;
        // 星宗定小限例：甲子生、壬辰太歲、寅宮坐命 → 小限戌；生五月：五月戌、六月酉……四月亥
        let xiao = xiao_xian(z('寅'), z('子'), z('辰'));
        assert_eq!(xiao, z('戌'));
        let book = "戌酉申未午巳辰卯寅丑子亥";
        for (k, c) in book.chars().enumerate() {
            let month = ((5 + k as u8 - 1) % 12) + 1;
            assert_eq!(yue_xian(xiao, 5, month), z(c), "{month}月");
        }
        // 大成卷三：小限在申，正月申、二月未、三月午、四月巳（生正月）
        for (mo, c) in [(1, '申'), (2, '未'), (3, '午'), (4, '巳')] {
            assert_eq!(yue_xian(z('申'), 1, mo), z(c));
        }
        // 大成卷三：午生人午上安命，戌年小限在寅，三月生其月在寅、四月丑、五月子
        let x2 = xiao_xian(z('午'), z('午'), z('戌'));
        assert_eq!(x2, z('寅'));
        for (mo, c) in [(3, '寅'), (4, '丑'), (5, '子')] {
            assert_eq!(yue_xian(yue_xian_start(YueXianRule::XiaoXian, x2, 0), 3, mo), z(c));
        }
        // 按生日：一岁命宫，与年支法同序
        for sui in 1..30u32 {
            assert_eq!(xiao_xian_sui(z('丑'), sui), xiao_xian(z('丑'), z('亥'), ((z('亥') as u32 + sui - 1) % 12) as u8));
        }
        // 童限：一命二財三疾厄四妻五福，六官祿，十五還命
        let m = z('子');
        let p = |sui| PALACES[((m as i32 - tong_xian_gong(m, sui) as i32).rem_euclid(12)) as usize];
        assert_eq!([p(1), p(2), p(3), p(4), p(5), p(6)], ["命宮", "財帛", "疾厄", "妻妾", "福德", "官祿"]);
        assert_eq!(p(15), "命宮");
        assert_eq!(p(19), "命宮");
    }

    /// 流年變曜：十二神所在宫之宫主（规则）对卷三逐年原表
    #[test]
    fn bian_yao_rule_vs_table() {
        let abbr = ["太嵗", "太陽", "喪", "太隂", "官", "死", "破", "", "虎", "天徳", "吊", "病"];
        let (mut hit, mut miss, mut conflict) = (0, Vec::new(), Vec::new());
        for y in 0..12u8 {
            let tab = bian_yao_table(y);
            assert_eq!(tab.len(), 7);
            for (k, (name, _, yao)) in liu_nian_bian_yao(y).iter().enumerate() {
                if abbr[k].is_empty() {
                    continue;
                }
                let has = |s: &str| if abbr[k] == "太陽" || abbr[k] == "太隂" { s.contains(abbr[k]) } else { s.replace("太陽", "").replace("太隂", "").contains(abbr[k]) };
                if tab.iter().any(|(t, s)| t == yao && has(s)) {
                    hit += 1;
                } else if let Some((t, _)) = tab.iter().find(|(_, s)| has(s)) {
                    conflict.push(format!("{}年{name}：规则{} 表{}", &ZHI_CH[y as usize * 3..y as usize * 3 + 3], YAO[*yao], YAO[*t]));
                } else {
                    miss.push(format!("{}年{name}", &ZHI_CH[y as usize * 3..y as usize * 3 + 3]));
                }
            }
        }
        eprintln!("變曜：合 {hit}，表缺 {:?}，异 {:?}", miss, conflict);
        assert_eq!(hit + miss.len() + conflict.len(), 132);
        assert!(hit >= 118);
        assert_eq!(conflict.len(), 1);
        // 卷三丙戌例
        let v = liu_nian_bian_yao(10);
        assert_eq!([v[2].2, v[4].2, v[10].2, v[8].2, v[0].2], [6, 5, 2, 0, 4]);
    }

    /// 书中命例（《星學大成》卷七、十一、十二、二十三，《星命溯源》卷四、五）：干支年在 960—1644 间逐一推算，
    /// 看何年与书所记各曜之宫、命宫、命度之宿相合。结果表见 doc/folk/qizheng.md §4.8
    /// 逐年推算 685 年、十八例，release 下 6—9 分钟，故缺省不跑（覆盖率 CI 亦不跑）；
    /// 手动：cargo test --release -p lunar-core --no-default-features --features real-f64 --lib -- --ignored ming_li_cases
    #[test]
    #[ignore = "逐年推算，耗时数分钟；手动 --ignored 运行"]
    fn ming_li_cases() {
        use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
        use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
        use crate::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
        use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
        use crate::astronomy::pipeline::{Vsop87Elp, Vsop87Planets};
        use crate::astronomy::sky_culture::chinese::load_lunar_mansion_systems_from_repo;
        use crate::astronomy::stars::load_hipparcos_bright_from_repo;
        use crate::calendar::chinese_lunar::{compute_year_data, to_julian_day, tt_to_day_number_utc8, ChineseLunarDate};
        use crate::quantity::angle::PlaneAngle;
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::DEFAULT).unwrap();
        let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
            .iter()
            .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::DEFAULT).unwrap()))
            .collect();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
        let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let systems = load_lunar_mansion_systems_from_repo().unwrap();
        let opts = ApparentPipelineOptions::pipeline_default();
        let gz = "甲乙丙丁戊己庚辛壬癸";
        let z = |c: char| ZHI_CH.chars().position(|x| x == c).unwrap() as u8;
        // （名，出处，年：干支或公历，月，闰，日，时支，所记：「曜宫」如「金寅」、「曜@宿」如「命@室」）
        type Case = (&'static str, &'static str, &'static str, u8, bool, u8, char, &'static str);
        let cases: [Case; 18] = [
            ("淳熙十三年五星聚", "大成卷二十三", "1186", 7, true, 16, '巳', "土巳 木巳 火巳 金巳 水巳"),
            ("乾道四年連珠", "大成卷二十三", "1168", 12, false, 15, '午', "水申 金酉 火戌 木亥 土丑"),
            ("王待制", "大成卷十一", "甲申", 10, false, 5, '寅', "命辰 金寅 日卯 月丑"),
            ("虞學士", "大成卷十一", "乙未", 4, false, 29, '午', "命午 日酉 月戌 火戌"),
            ("蔡樞密", "大成卷十二", "己丑", 12, false, 27, '申', "日子 月寅 金寅 火丑"),
            ("許狀元", "大成卷十二", "戊子", 7, false, 1, '丑', "命申 月未 水巳 木亥 火戌 土子 羅巳 計亥 孛丑"),
            ("安墩御史", "大成卷十二", "壬午", 12, false, 26, '午', "命酉 日子 月丑 土巳 木午 金亥"),
            ("官均", "大成卷十二", "辛巳", 8, false, 1, '戌', "命戌 日巳 月巳 金巳 水巳 火巳"),
            ("戊子十二月初八卯時", "大成卷七", "戊子", 12, false, 8, '卯', "命丑"),
            ("柯監丞", "大成卷二十三", "壬子", 8, false, 27, '亥', "命申"),
            ("黄少卿", "溯源卷四", "甲戌", 1, false, 27, '卯', "命亥 日亥 水亥 木亥 命@室"),
            ("婦人丁巳（日未記）", "溯源卷四", "丁巳", 1, false, 15, '申', "木@亢"),
            ("乙卯九月十四", "溯源卷四", "乙卯", 9, false, 14, '戌', "命@胃"),
            ("丁酉十月（日未記）", "溯源卷四", "丁酉", 10, false, 15, '丑', "火@轸"),
            ("戊辰正月（日未記）", "溯源卷四", "戊辰", 1, false, 15, '酉', "金@危"),
            ("辛卯十月十五", "溯源卷四", "辛卯", 10, false, 15, '戌', "命@毕 土卯 月卯 水卯 日卯"),
            ("癸未八月初一", "溯源卷五", "癸未", 8, false, 1, '未', "命@斗 孛=命"),
            ("壬申五月二十", "溯源卷五", "壬申", 5, false, 20, '午', "命巳 土巳 計申 水申 日申"),
        ];
        let yao_idx = |c: char| match c {
            '日' => 0,
            '月' => 1,
            '水' => 2,
            '金' => 3,
            '火' => 4,
            '木' => 5,
            '土' => 6,
            '羅' => 7,
            '計' => 8,
            '孛' => 9,
            '炁' => 10,
            _ => 99, // 命
        };
        let mut full: Vec<(String, i32, &str)> = Vec::new();
        for (name, src, year, month, leap, day, hour, items) in cases {
            let years: Vec<i32> = if let Ok(y) = year.parse::<i32>() {
                vec![y]
            } else {
                let c: Vec<char> = year.chars().collect();
                let (g, zz) = (gz.chars().position(|x| x == c[0]).unwrap() as i32, z(c[1]) as i32);
                // 《星學大成》成书于嘉靖四十二年（1563），所举命例不晚于此；《星命溯源》四库本收，取至明末
                let last = if src.starts_with("大成") { 1563 } else { 1644 };
                (960..=last).filter(|y| (*y - 4i32).rem_euclid(10) == g && (*y - 4i32).rem_euclid(12) == zz).collect()
            };
            let mut line = Vec::new();
            for y in years {
                // 十一、十二月在下一农历年岁数据之首
                let ly = if month >= 11 { y + 1 } else { y };
                let Ok(yd) = compute_year_data(&earth, &elp, ly, PlaneAngle::from_rad(real(1e-9)), 30) else { continue };
                let d0 = ChineseLunarDate { year: ly, month: 1, day: 1, is_leap_month: false, days_in_month: 30 };
                let date = ChineseLunarDate { month, is_leap_month: leap, ..d0 };
                let Some(nm) = to_julian_day(date, &yd).or_else(|| to_julian_day(ChineseLunarDate { is_leap_month: false, ..date }, &yd)) else { continue };
                // 东八区民用日 + 时支正中（子正 0 时）
                let dn = tt_to_day_number_utc8(nm) + day as i64 - 1;
                let jd_ut = real(dn as f64 - 0.5 + (2.0 * z(hour) as f64 - 8.0) / 24.0);
                let t = TimePoint::new(TimeScale::UT1, jd_ut).to_scale(TimeScale::TT);
                let sid = if y < 1270 { "song_huangyou" } else if y < 1368 { "yuan" } else { "ming_xianglin" };
                let sys = systems.iter().find(|s| s.id == sid).unwrap();
                let frame = MansionFrame::new(&eph, &cat, sys, t, &opts).unwrap();
                let mut cols = Vec::new();
                for (tag, g) in [("中", GongRule::ZhongQi), ("宗", GongRule::XiuDuXingZong)] {
                    let o = QizhengOptions { gong: g, ..Default::default() };
                    let c = chart(&eph, t, Some(&frame), z(hour), None, &o, &opts);
                    let mut ok = 0;
                    let mut n = 0;
                    let mut bad = Vec::new();
                    for it in items.split(' ') {
                        let ch: Vec<char> = it.chars().collect();
                        n += 1;
                        let k = yao_idx(ch[0]);
                        let (got_gong, got_xiu) = if k == 99 {
                            (c.ming, c.xing_ming.ming_xiu.map(|x| x.0).unwrap())
                        } else {
                            (c.yao[k].gong, c.yao[k].xiu.map(|x| x.0).unwrap())
                        };
                        let good = match ch[1] {
                            '@' => MANSIONS[got_xiu] == ch[2..].iter().collect::<String>(),
                            '=' => got_gong == c.ming,
                            _ => got_gong == z(ch[1]),
                        };
                        if good {
                            ok += 1;
                        } else if ch[1] == '@' {
                            bad.push(format!("{}{}", ch[0], MANSIONS[got_xiu]));
                        } else {
                            bad.push(format!("{}{}", ch[0], &ZHI_CH[got_gong as usize * 3..got_gong as usize * 3 + 3]));
                        }
                    }
                    if ok == n {
                        full.push((name.to_string(), y, tag));
                    }
                    cols.push(format!("{tag}{ok}/{n}{}", if bad.is_empty() { String::new() } else { format!("({})", bad.join("")) }));
                }
                line.push(format!("{y}:{}", cols.join(",")));
            }
            eprintln!("命例|{name}|{src}|{year}{}{}月{day}日{hour}時|{items}|{}", if leap { "閏" } else { "" }, month, line.join(" "));
        }
        // 年份推定后全合之例：黄少卿（1394，含命度室）、壬申五月二十（1332，含計都在申）、王待制（1404）
        for (n, y) in [("黄少卿", 1394), ("壬申五月二十", 1332), ("王待制", 1404)] {
            assert!(full.iter().any(|(a, b, t)| a == n && *b == y && *t == "中") && full.iter().any(|(a, b, t)| a == n && *b == y && *t == "宗"), "{n} {y}");
        }
    }

    #[test]
    fn mean_elements_1833() {
        // 道光十三年正月初一（1833-02-19，JD 2390598.5）
        let t = TimePoint::new(TimeScale::TT, real(2390598.5));
        assert!((mean_ascending_node(t).as_f64() - 112.40).abs() < 0.01);
        assert!(((mean_perigee(t).as_f64() + 180.0).rem_euclid(360.0) - 313.67).abs() < 0.01);
    }
}
