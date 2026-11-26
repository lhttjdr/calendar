//! 排盘（聚合层）：由出生时刻与节气时刻排四柱，起大运、小运，求命宫、胎元、人元司令。
//!
//! 节气时刻取天文域 `astronomy::aspects::solar_term_jds_for_year_cached`（定气，TT）；平气按当时行用历推算
//! （`calendar::gan_zhi_options::pingqi_jds_from_dingqi`，表外年代仍用定气）。年以立春、月以十二节换，皆比到交节时刻；
//! 日、时以钟点（`clock_utc_offset_hours` 所指时区的民用时）定，日界依 [DayBoundary]。
//!
//! 起运的折算诸说见 [QiYunRule]；节气时刻精确到秒，所以「数至未来/过去交节」可以精确到时辰以下（[QiYunPrecision]）；
//! 交节当日之「又說」见 [JiaoJieRi]；起运岁数落到日历上之法见 [JiaoYunRule]。

use super::canggan::si_ling_with;
use super::gong::{self, MingGongMonth};
use super::{gan_is_yang, gan_of, gz_of, zhi_of, SiZhu, ZipingOptions};
use crate::astronomy::aspects::solar_term_jds_for_year_cached;
use crate::astronomy::ephemeris::Vsop87;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::calendar::gan_zhi_options::{jd_to_gan_zhi_index_with_options, pingqi_jds_from_dingqi, DayBoundary, SolarTermMode};
use crate::calendar::gregorian::Gregorian;
use crate::math::real::{real, Real, RealOps};

/// 一个回归年的日数：[JiaoYunRule::HuiGuiNian] 折起运岁数、两法每运「十週年」之长
const TROPICAL_YEAR: f64 = 365.2422;
/// 运岁之日数（十二月、月作三十日）：[JiaoYunRule::LiYue]
const YUN_SUI_DAYS: f64 = 360.0;

/// 十二节在 24 气中的序（0 春分 … 23 驚蟄）：立春、驚蟄、清明……小寒；下标即月序（寅 = 0）
const JIE: [usize; 12] = [21, 23, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
/// 十二中气：雨水（正月中）、春分……大寒；下标 + 1 即月数
const ZHONG: [usize; 12] = [22, 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20];
/// 二十四气名（序同天文域：0 春分）
pub const QI_NAMES: [&str; 24] = [
    "春分", "清明", "穀雨", "立夏", "小滿", "芒種", "夏至", "小暑", "大暑", "立秋", "處暑", "白露",
    "秋分", "寒露", "霜降", "立冬", "小雪", "大雪", "冬至", "小寒", "大寒", "立春", "雨水", "驚蟄",
];

/// 子正换日时，23—24 时（夜子时）之时干依何日起
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum YeZiShi {
    /// 依次日日干起（袁树珊《命理探源》卷五「作辛酉日庚子時推（用壬日起庚子時）」，《命理集成》卷一照录）：缺省
    #[default]
    CiRi,
    /// 依当日日干起
    BenRi,
}

/// 起运岁数的折算
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QiYunRule {
    /// 三日一岁、一日四月、一时辰十日，实历折除不取整：《三命通會》卷二《論大運》「一辰十嵗折除以三日為年……實歷過六十三日（时），
    /// 折除……計六百三十日，乃一嵗竒九月」，并斥「今人行運多用約法……一嵗竒八月起運便以二嵗，殊不明折除實厯之數」；
    /// 《淵海子平》卷一《論起大運法》「又法零借之說……若春在寅時則多一時，乃零一旬也」。缺省
    #[default]
    ShiLi,
    /// 只取整岁：《淵海子平》卷一同节「以三日為一歲……或只零二日，可借一日收作一歲用；或零一日則去之，不足取用」
    ZhengSui,
}

/// 起运日数的计法
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QiYunPrecision {
    /// 以生时与交节时刻之差精确计（节气时刻由天文计算，精确到秒）：缺省
    #[default]
    Exact,
    /// 以时辰计（《三命通會》例「自二十四日巳時……至二十九申時，正得五日三時」；《淵海》「若學者不知生刻，獨知生時，則以生時扣算交節之時」）
    Shichen,
}

/// 交节当日生者之起运（《淵海子平》卷一《論起大運法》「又說」）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JiaoJieRi {
    /// 照常折算（一时辰十日）：《淵海》正文与《三命通會》卷二只有「一辰十嵗」之折除，不另立交节当日之例。缺省
    #[default]
    ZhaoSuan,
    /// 所数之节在生日当日者作一岁运（生于交节之时辰者，顺逆皆作一岁）：《淵海》同节「又說：如本生之日子時生者，就子時交節，不論逆順，皆起一歲運也；若丑時交節，
    /// 順者以一時辰就作一歲運也，逆者而作十歲運也」。顺者所数即当日之节，不足一岁作一岁；逆者所数为上一节（约三十日前），
    /// 照常约十岁，书中「十歲」即此，不另作处理
    YiSui,
}

/// 交运时刻（起运岁数如何落到日历上）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JiaoYunRule {
    /// 按历月逐月累加，小月进一日：《淵海子平》卷一《論起大運法》「行運之法，假如甲子年正月初一日子時正一刻生，行運筭至乙丑年
    /// 正月初一日子時正一刻乃作一歲。其年內小六個月，即進六日，在初七日子時正一刻方作一歲。只要筭定十二月，却為本年有閏月，
    /// 乃多一月矣，當退還本年十二月初七日子時正一刻交運。從此筭後十週年方換一運」；《三命通會》卷二《論大運》同（作「本年有閏四月」），
    /// 并举「一嵗竒九月……必自十二月生日後實經厯過二十有一之日月運方移宫」。数满十二个朔望月（闰月亦数）再以小月补日，
    /// 即每月作三十日、一岁三百六十日：交运 = 出生 + 起运日数 × 120 日（「一辰十日」之运日原数）。此后「十週年方換一運」，
    /// 週年取回归年（农历同月同日之週年与之相差不过半月，排盘不取月历，未按农历月日）。缺省
    #[default]
    LiYue,
    /// 起运岁数 × 回归年（365.2422 日）加于出生时刻，每运再加十个回归年：今人排盘常法，两书无此说
    HuiGuiNian,
}

/// 小运起法
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum XiaoYunRule {
    /// 男一岁丙寅顺行，女一岁壬申逆行，不论阴阳：《淵海子平》卷一《論起小運法》「其法不論陰陽二命，男一歲起丙寅，順行二歲丁卯……
    /// 女一歲起壬申，逆行二歲辛未」；《三命通會》卷二《論小運》「男起丙寅順行、女起壬申逆行，一定而不可易也」。缺省
    #[default]
    BingRen,
    /// 自时柱起，顺逆同大运：《淵海》同节「諸書小運順逆，皆自時生而行之，順逆亦以年定，如陽命陽年甲子時生墮地，則一歲行乙丑」；
    /// 《三命通會》引醉醒子「男女小運皆由時生而行之……嘗試用之屢驗」
    ShiZhu,
    /// 按生年之旬：甲子旬男丙寅女壬申，甲申旬男丙戌女壬辰……（旬首之后二位、八位）。《三命通會》引「或者」之说而斥为「非聖人原立起年運之義」
    XunShou,
}

/// 胎元求法
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TaiYuanRule {
    /// 月前四位（月干进一、月支进三）：《淵海子平》卷一《論起胎法》。缺省
    #[default]
    YueQianSi,
    /// 生前三百日（与生日同干支之日）所在之月：《三命通會》卷二《論胎元》「惟有一法以當生前三百日為十月之氣，乃是受胎之正」
    ShengQianSanBai,
}

/// 出生时刻
#[derive(Clone, Copy, Debug)]
pub struct BirthInput {
    /// 出生时刻 JD(TT)
    pub jd_tt: Real,
    /// 钟点所在时区（时柱、日柱按此时区的民用钟点定），东八区为 8。用当地平太阳时或视太阳时者，自行换算后填 0 并以对应 UT 给出 jd_tt
    pub clock_utc_offset_hours: Real,
    pub male: bool,
}

/// 起运
#[derive(Clone, Copy, Debug)]
pub struct QiYun {
    /// 大运顺行（阳男阴女）
    pub forward: bool,
    /// 所数之节（24 气序）及其时刻 JD(TT)
    pub jie: usize,
    pub jie_jd_tt: Real,
    /// 出生至所数之节的日数（按 [QiYunPrecision]）
    pub days: Real,
    /// 起运岁数（实岁，可带小数）
    pub age: Real,
    /// 起运岁数的岁、月、日（一岁 360 日、一月 30 日之折算）
    pub years: i32,
    pub months: i32,
    pub day_part: i32,
    /// 交运时刻 JD(TT)：按 [JiaoYunRule]
    pub start_jd_tt: Real,
}

/// 一步大运
#[derive(Clone, Copy, Debug)]
pub struct DaYun {
    pub gz: u8,
    /// 起于实岁
    pub age: Real,
    pub start_jd_tt: Real,
}

/// 命盘
#[derive(Clone, Debug)]
pub struct BaziPan {
    pub sizhu: SiZhu,
    pub male: bool,
    /// 出生所在节气月之节（24 气序）与时刻、下一节
    pub prev_jie: (usize, Real),
    pub next_jie: (usize, Real),
    /// 交节后日数与人元司令之干（日数之家依 [ZipingOptions::si_ling]）
    pub days_since_jie: Real,
    pub si_ling: u8,
    pub qi_yun: QiYun,
    pub da_yun: Vec<DaYun>,
    pub tai_yuan: u8,
    pub ming_gong: u8,
    pub shen_gong: u8,
    pub tai_xi: u8,
    pub bian: u8,
}

/// 某段时间内的节气表 (JD(TT), 24 气序)，升序
fn terms_around(vsop: &Vsop87, jd_tt: Real, mode: SolarTermMode) -> Vec<(Real, usize)> {
    let (y, _, _) = Gregorian::from_julian_day(jd_tt);
    let mut v = Vec::with_capacity(96);
    for yy in (y - 2)..=(y + 1) {
        let dq = solar_term_jds_for_year_cached(vsop, yy);
        let t = match mode {
            SolarTermMode::Pingqi => pingqi_jds_from_dingqi(&dq),
            _ => dq,
        };
        v.extend(t.into_iter().enumerate().map(|(k, jd)| (jd, k)));
    }
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    v
}

/// 时刻 `jd_tt` 的年柱、月柱（立春换年、十二节换月）及所在节、下一节
fn year_month_at(terms: &[(Real, usize)], jd_tt: Real) -> (u8, u8, (usize, Real), (usize, Real)) {
    let jie: Vec<&(Real, usize)> = terms.iter().filter(|t| JIE.contains(&t.1)).collect();
    let i = jie.iter().rposition(|t| t.0 <= jd_tt).expect("节气表不足");
    let (prev, next) = (*jie[i], *jie[i + 1]);
    let lichun = jie[..=i].iter().rev().find(|t| t.1 == JIE[0]).expect("节气表不足").0;
    let (gy, _, _) = Gregorian::from_julian_day(lichun);
    let year = (gy + 56).rem_euclid(60) as u8;
    let ord = JIE.iter().position(|&k| k == prev.1).unwrap() as u8; // 寅 = 0
    let month = gz_of((gan_of(year) * 2 + 2 + ord) % 10, (ord + 2) % 12);
    (year, month, (prev.1, prev.0), (next.1, next.0))
}

/// 以中气定的月数（雨水后为正月）
fn zhongqi_month_no(terms: &[(Real, usize)], jd_tt: Real) -> u8 {
    let k = terms.iter().filter(|t| ZHONG.contains(&t.1) && t.0 <= jd_tt).last().expect("节气表不足").1;
    ZHONG.iter().position(|&z| z == k).unwrap() as u8 + 1
}

/// TT → 钟点民用 JD
fn clock_jd(jd_tt: Real, offset_h: Real) -> Real {
    TimePoint::new(TimeScale::TT, jd_tt).to_scale(TimeScale::UTC).jd + offset_h / real(24.0)
}

/// 钟点民用 JD 的时辰流水号（子时跨日：23 时起入下一时辰号）
fn shichen_serial(clock: Real) -> i64 {
    ((clock + real(0.5) + real(1.0 / 24.0)) * real(12.0)).floor().to_i64_floor()
}

/// 日柱、时柱
pub fn day_hour(clock: Real, day_boundary: DayBoundary, ye_zi: YeZiShi) -> (u8, u8) {
    let day = jd_to_gan_zhi_index_with_options(clock, day_boundary);
    let hours = ((clock + real(0.5)) - (clock + real(0.5)).floor()) * real(24.0);
    let hz = (((hours + real(1.0)) / real(2.0)).floor().to_i32_floor().rem_euclid(12)) as u8;
    let ye_zi_shi = hours >= real(23.0);
    let base = if ye_zi_shi && day_boundary == DayBoundary::Hour0 && ye_zi == YeZiShi::CiRi { (day + 1) % 60 } else { day };
    let hg = (gan_of(base) * 2 + hz) % 10;
    (day, gz_of(hg, hz))
}

/// 起运：数到下一节（顺）或上一节（逆）
fn qi_yun(
    birth: Real,
    clock_off: Real,
    forward: bool,
    prev: (usize, Real),
    next: (usize, Real),
    o: &ZipingOptions,
) -> QiYun {
    let (jie, jie_jd) = if forward { next } else { prev };
    let days = match o.qi_yun_precision {
        QiYunPrecision::Exact => {
            if forward {
                jie_jd - birth
            } else {
                birth - jie_jd
            }
        }
        QiYunPrecision::Shichen => {
            let a = shichen_serial(clock_jd(birth, clock_off));
            let b = shichen_serial(clock_jd(jie_jd, clock_off));
            real(((b - a).abs()) as f64) / real(12.0)
        }
    };
    // 又說：所数之节在生日当日（钟点民用日）；或生于交节之时辰（「子時生者就子時交節，不論逆順」），则不论所数为哪一节
    let jiao_jie_ri = o.jiao_jie_ri == JiaoJieRi::YiSui && {
        let day = |t: Real| (clock_jd(t, clock_off) + real(0.5)).floor().to_i64_floor();
        let sc = |t: Real| shichen_serial(clock_jd(t, clock_off));
        let other = if forward { prev.1 } else { next.1 };
        day(birth) == day(jie_jd) || sc(birth) == sc(other)
    };
    let (age, years, months, day_part) = if jiao_jie_ri {
        (real(1.0), 1, 0, 0)
    } else {
        match o.qi_yun {
            QiYunRule::ShiLi => {
                // 一日 = 120 日（四月），一岁 = 360 日
                let life = days * real(120.0);
                let y = (life / real(360.0)).floor().to_i32_floor();
                let rem = life - real(360.0 * y as f64);
                let m = (rem / real(30.0)).floor().to_i32_floor();
                let d = (rem - real(30.0 * m as f64)).floor().to_i32_floor();
                (days / real(3.0), y, m, d)
            }
            QiYunRule::ZhengSui => {
                let n = days.to_i32_round();
                let y = n / 3 + if n % 3 == 2 { 1 } else { 0 };
                (real(y as f64), y, 0, 0)
            }
        }
    };
    let per_sui = match o.jiao_yun {
        JiaoYunRule::LiYue => YUN_SUI_DAYS,
        JiaoYunRule::HuiGuiNian => TROPICAL_YEAR,
    };
    QiYun { forward, jie, jie_jd_tt: jie_jd, days, age, years, months, day_part, start_jd_tt: birth + age * real(per_sui) }
}

/// 排盘：四柱、起运与大运（`n_yun` 步）、胎元、命宫身宫、胎息与变、人元司令
pub fn pai_pan(vsop: &Vsop87, b: &BirthInput, n_yun: usize, o: &ZipingOptions) -> BaziPan {
    let terms = terms_around(vsop, b.jd_tt, o.solar_term_mode);
    let (year, month, prev, next) = year_month_at(&terms, b.jd_tt);
    let clock = clock_jd(b.jd_tt, b.clock_utc_offset_hours);
    let (day, hour) = day_hour(clock, o.day_boundary, o.ye_zi_shi);
    let sizhu = SiZhu::new(year, month, day, hour);

    let forward = gan_is_yang(gan_of(year)) == b.male;
    let qy = qi_yun(b.jd_tt, b.clock_utc_offset_hours, forward, prev, next, o);
    let da_yun = (1..=n_yun as i32)
        .map(|k| {
            let gz = (month as i32 + if forward { k } else { -k }).rem_euclid(60) as u8;
            let age = qy.age + real(10.0 * (k - 1) as f64);
            // 首运交运后「十週年方換一運」（两法同）
            DaYun { gz, age, start_jd_tt: qy.start_jd_tt + real(10.0 * TROPICAL_YEAR * (k - 1) as f64) }
        })
        .collect();

    let tai_yuan = match o.tai_yuan {
        TaiYuanRule::YueQianSi => gong::tai_yuan(month),
        TaiYuanRule::ShengQianSanBai => {
            let t = b.jd_tt - real(300.0);
            let terms_t = terms_around(vsop, t, o.solar_term_mode);
            year_month_at(&terms_t, t).1
        }
    };
    let month_no = match o.ming_gong_month {
        MingGongMonth::Jie => (zhi_of(month) + 10) % 12 + 1,
        MingGongMonth::ZhongQi => zhongqi_month_no(&terms, b.jd_tt),
    };
    let hz = zhi_of(hour);
    let days_since_jie = b.jd_tt - prev.1;
    BaziPan {
        sizhu,
        male: b.male,
        prev_jie: prev,
        next_jie: next,
        days_since_jie,
        si_ling: si_ling_with(zhi_of(month), days_since_jie, o.si_ling),
        qi_yun: qy,
        da_yun,
        tai_yuan,
        ming_gong: gong::ming_gong(year, month_no, hz),
        shen_gong: gong::shen_gong(year, month_no, hz),
        tai_xi: gong::tai_xi(day),
        bian: gong::bian(hour),
    }
}

/// 小运：虚岁 `sui`（生年为一岁）之小运干支
pub fn xiao_yun(sz: &SiZhu, male: bool, sui: u32, rule: XiaoYunRule) -> u8 {
    let k = sui as i32 - 1;
    let forward = gan_is_yang(gan_of(sz.year)) == male;
    let start = |g: i32| g.rem_euclid(60) as u8;
    match rule {
        XiaoYunRule::BingRen => {
            if male {
                start(2 + k) // 丙寅
            } else {
                start(8 - k) // 壬申
            }
        }
        XiaoYunRule::ShiZhu => start(sz.hour as i32 + if forward { k + 1 } else { -(k + 1) }),
        XiaoYunRule::XunShou => {
            let xun = (sz.year / 10 * 10) as i32;
            if male {
                start(xun + 2 + k)
            } else {
                start(xun + 8 - k)
            }
        }
    }
}

/// 流年干支：公历（天文纪年）某年立春后之年干支
pub fn liu_nian(year: i32) -> u8 {
    (year + 56).rem_euclid(60) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::folk::ziping::{gz_name, parse_gz};

    fn o() -> ZipingOptions {
        ZipingOptions::default()
    }

    /// 合成起运：以给定节气前后日数检验折算
    fn qy(days: f64, o: &ZipingOptions) -> QiYun {
        let birth = real(2_451_545.0);
        qi_yun(birth, real(8.0), true, (21, birth - real(10.0)), (23, birth + real(days)), o)
    }

    #[test]
    fn qi_yun_rules() {
        // 《三命通會》卷二例：甲子阳男十二月二十四日巳时生，二十九日申时立春，「正得五日三時……乃一嵗竒九月」
        let q = qy(5.0 + 3.0 / 12.0, &o());
        assert_eq!((q.years, q.months, q.day_part), (1, 9, 0));
        // 淵海：正月初一丑时正一刻生，初四丑正一刻立春，「乃作一歲運全，若春在寅時則多一時，乃零一旬也」
        let q = qy(3.0, &o());
        assert_eq!((q.years, q.months, q.day_part), (1, 0, 0));
        let q = qy(3.0 + 1.0 / 12.0, &o());
        assert_eq!((q.years, q.months, q.day_part), (1, 0, 10));
        // 淵海整岁：零二日借一日作一岁，零一日去之
        let z = ZipingOptions { qi_yun: QiYunRule::ZhengSui, ..o() };
        assert_eq!(qy(8.0, &z).years, 3);
        assert_eq!(qy(7.0, &z).years, 2);
        assert_eq!(qy(6.0, &z).years, 2);
    }

    #[test]
    fn jiao_yun_rules() {
        let birth = real(2_451_545.0);
        // 三命例：一岁奇九月，「實經厯過二十有一之日月」，月作三十日：六百三十日
        let q = qy(5.0 + 3.0 / 12.0, &o());
        assert!((q.start_jd_tt - birth - real(630.0)).abs() < real(1e-6));
        // 淵海例：一岁运「其年內小六個月即進六日」：十二月共三百六十日
        assert!((qy(3.0, &o()).start_jd_tt - birth - real(360.0)).abs() < real(1e-6));
        let h = ZipingOptions { jiao_yun: JiaoYunRule::HuiGuiNian, ..o() };
        assert!((qy(3.0, &h).start_jd_tt - birth - real(TROPICAL_YEAR)).abs() < real(1e-6));
        // 交节当日「又說」：顺者一时辰作一岁；逆者数上一节照常（约十岁）；生于交节之时辰者逆亦一岁
        let y = ZipingOptions { jiao_jie_ri: JiaoJieRi::YiSui, ..o() };
        assert_eq!(qy(1.0 / 12.0, &o()).day_part, 10);
        let q = qy(1.0 / 12.0, &y);
        assert_eq!((q.years, q.months, q.day_part), (1, 0, 0));
        let ni = |next: f64, o: &ZipingOptions| qi_yun(birth, real(8.0), false, (21, birth - real(30.0)), (23, birth + real(next)), o);
        assert_eq!(ni(1.0 / 12.0, &y).years, 10);
        assert_eq!(ni(0.01, &y).years, 1);
        assert_eq!(ni(0.01, &o()).years, 10);
    }

    #[test]
    fn xiao_yun_rules() {
        let sz = SiZhu::parse("甲子丙寅甲子甲子").unwrap();
        // 淵海：男一岁丙寅、二岁丁卯；女一岁壬申、二岁辛未
        assert_eq!(gz_name(xiao_yun(&sz, true, 1, XiaoYunRule::BingRen)), "丙寅");
        assert_eq!(gz_name(xiao_yun(&sz, true, 2, XiaoYunRule::BingRen)), "丁卯");
        assert_eq!(gz_name(xiao_yun(&sz, false, 2, XiaoYunRule::BingRen)), "辛未");
        // 时柱起：阳命甲子时，一岁乙丑、二岁丙寅
        assert_eq!(gz_name(xiao_yun(&sz, true, 1, XiaoYunRule::ShiZhu)), "乙丑");
        assert_eq!(gz_name(xiao_yun(&sz, true, 2, XiaoYunRule::ShiZhu)), "丙寅");
        // 阴命（阳年女）逆：一岁癸亥
        assert_eq!(gz_name(xiao_yun(&sz, false, 1, XiaoYunRule::ShiZhu)), "癸亥");
        // 按旬：甲申旬男丙戌、女壬辰
        let sz2 = SiZhu::new(parse_gz("乙酉").unwrap(), 0, 0, 0);
        assert_eq!(gz_name(xiao_yun(&sz2, true, 1, XiaoYunRule::XunShou)), "丙戌");
        assert_eq!(gz_name(xiao_yun(&sz2, false, 1, XiaoYunRule::XunShou)), "壬辰");
    }

    #[test]
    fn hour_pillar() {
        // 2024-01-01（甲子日）各钟点；23 时夜子时：子正换日、时干依次日（乙丑日起丙子）
        let d = Gregorian::to_julian_day(2024, 1, 1); // 0 时
        let at = |h: f64, db, yz| {
            let (dd, hh) = day_hour(d + real(h / 24.0), db, yz);
            format!("{} {}", gz_name(dd), gz_name(hh))
        };
        assert_eq!(at(0.5, DayBoundary::Hour0, YeZiShi::CiRi), "甲子 甲子");
        assert_eq!(at(12.0, DayBoundary::Hour0, YeZiShi::CiRi), "甲子 庚午");
        assert_eq!(at(23.5, DayBoundary::Hour0, YeZiShi::CiRi), "甲子 丙子");
        assert_eq!(at(23.5, DayBoundary::Hour0, YeZiShi::BenRi), "甲子 甲子");
        assert_eq!(at(23.5, DayBoundary::Hour23, YeZiShi::CiRi), "乙丑 丙子");
    }

    #[test]
    fn liu_nian_gz() {
        assert_eq!(gz_name(liu_nian(1984)), "甲子");
        assert_eq!(gz_name(liu_nian(2026)), "丙午");
    }
}
