//! 奇门遁甲定局（时家为主，附日家、月家）：阴阳遁、上中下三元、局数。
//!
//! 依明程道生《遁甲演義》（四库本）：卷三「竒遁佈局法」「冬至陽遁時竒起例」「夏至隂遁時竒起例」，
//! 卷一「超神接氣置閏訣」「超接引例」「又引證」「日家竒門」「月家竒門」。各条原文、各派差异与逐例检验见
//! `doc/folk/qimen.md`。
//!
//! 定局两法并列（[`DingJu`]）：
//! - **置闰**（超神接气）：每节气三元十五日，自上元符头起排；芒种、大雪超过旬日，叠芒种（大雪）三元为闰奇（界见 [`RunRule`]）。
//! - **拆补**：节气以交节时辰为界，元以当日符头定（交节当日起即用新节气，所在元可能是中元、下元，即「殘局」「補局」）。
//!
//! 节气来源（[`JieQiSource`]）：本项目定气，或当时行用历（明大统历等平气，表外与清以后用定气）。
//! 日界：子初（23 时）换日、换局（「凡換竒皆子時換也」）；时刻取东八区民用时。
//!
//! 布盘（地盘、直符直使、天盘九星与干、八门、八神；转盘与飞盘并列，中五寄宫三说）见 [`pai_pan`]；
//! 格局（演义卷二）见 [`ge_ju`]；年家奇门（三元年遁）见 [`nian_jia_pan`]。

mod pan;
pub use pan::*;
mod geju;
pub use geju::*;
mod nianjia;
pub use nianjia::*;

use crate::astronomy::aspects::solar_term_jds_for_year_cached;
use crate::astronomy::ephemeris::Vsop87;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::calendar::gregorian::Gregorian;
use crate::calendar::pingqi;
use crate::math::real::{real, Real, RealOps};

/// 定局之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DingJu {
    /// 超神接气、芒种大雪置闰；置闰之界见 [`RunRule`]
    ZhiRun(RunRule),
    /// 拆局补局：节气以交节时辰换，元以符头定（卷一「又引證」万历二十四年例）
    ChaiBu,
}

/// 置闰之界（两书不同，见 doc/folk/qimen.md）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunRule {
    /// 《遁甲演義》卷一：芒种（大雪）本节超日（节日 − 符头日）≥ 10 置闰（「超過九日即置閏」；万历二十六年芒种超 9 不闰、大雪超 11 闰，二十九年芒种超 10 闰）。缺省
    YanYi,
    /// 《奇門遁甲統宗》「置闰法」：芒种（大雪）三元满后，下一至交节日比下一符头晚 ≥ 8 日置闰（康熙五十七年晚 8 日闰，五十六年冬至晚 6 日不闰）
    TongZong,
}

impl RunRule {
    fn is_run(self, own_chao: i64, next_zhi_minus_next_fu: i64) -> bool {
        match self {
            RunRule::YanYi => own_chao >= 10,
            RunRule::TongZong => next_zhi_minus_next_fu >= 8,
        }
    }
}

/// 缺省定局：置闰、演义之界
pub const DEFAULT_DING_JU: DingJu = DingJu::ZhiRun(RunRule::YanYi);

/// 节气来源
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JieQiSource {
    /// 本项目历表定气
    DingQi,
    /// 当时行用历：平气表内的年代用平气（如明大统历），否则定气
    XingYong,
}

/// 三元
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Yuan {
    Shang,
    Zhong,
    Xia,
}

impl Yuan {
    pub fn name(self) -> &'static str {
        match self {
            Yuan::Shang => "上元",
            Yuan::Zhong => "中元",
            Yuan::Xia => "下元",
        }
    }
    fn index(self) -> i32 {
        match self {
            Yuan::Shang => 0,
            Yuan::Zhong => 1,
            Yuan::Xia => 2,
        }
    }
}

/// 二十四气名，序同 [`solar_term_jds_for_year_cached`]（0 春分 … 23 驚蟄）
pub const JIE_QI: [&str; 24] = [
    "春分", "清明", "穀雨", "立夏", "小滿", "芒種", "夏至", "小暑", "大暑", "立秋", "處暑", "白露", "秋分", "寒露", "霜降",
    "立冬", "小雪", "大雪", "冬至", "小寒", "大寒", "立春", "雨水", "驚蟄",
];
const MANG_ZHONG: usize = 5;
const DA_XUE: usize = 17;

/// 是否阳遁（冬至至芒种十二气）
pub fn is_yang(term: usize) -> bool {
    !(6..=17).contains(&term)
}

/// 节气上元局数：各卦统三气，自卦宫数依次而下（卷三「坎則冬至小寒大寒，艮則立春雨水驚蟄……」）。
/// 阳遁坎 1、艮 8、震 3、巽 4 依次加一；阴遁离 9、坤 2、兑 7、乾 6 依次减一。
pub fn shang_yuan_ju(term: usize) -> u8 {
    // 自冬至（18）起数
    let q = (term + 24 - 18) % 24;
    const T: [u8; 24] = [1, 2, 3, 8, 9, 1, 3, 4, 5, 4, 5, 6, 9, 8, 7, 2, 1, 9, 7, 6, 5, 6, 5, 4];
    T[q]
}

/// 局数：上元之后「間六宫而行」为中、下元；阳遁顺（+6），阴遁逆（−6）
pub fn ju(term: usize, yuan: Yuan) -> u8 {
    let u = shang_yuan_ju(term) as i32 - 1;
    let step = if is_yang(term) { 6 } else { -6 };
    ((u + step * yuan.index()).rem_euclid(9) + 1) as u8
}

/// 日干支序（0 甲子 … 59 癸亥）
pub fn day_gz(jdn: i64) -> u8 {
    (jdn + 49).rem_euclid(60) as u8
}

/// 某日所在之元：以甲、己日为符头，五日一元；符头支子午卯酉上元、寅申巳亥中元、辰戌丑未下元（卷一「閏訣」）
pub fn yuan_of_day(jdn: i64) -> Yuan {
    let g = day_gz(jdn) as i64;
    let fu = g - g % 5;
    match fu % 12 {
        0 | 3 | 6 | 9 => Yuan::Shang,
        2 | 5 | 8 | 11 => Yuan::Zhong,
        _ => Yuan::Xia,
    }
}

/// 时辰序（自 JD 0 起的绝对时辰数）：东八区民用时，子时始于 23 时。所在日 = 时辰序 div 12。
pub fn shichen_index_from_ut(jd_ut: Real) -> i64 {
    let hours = (jd_ut.as_f64() + 8.0 / 24.0 + 0.5) * 24.0;
    ((hours + 1.0) / 2.0).floor() as i64
}

/// 由 JD(TT) 得绝对时辰序
pub fn shichen_index_from_tt(jd_tt: Real) -> i64 {
    shichen_index_from_ut(TimePoint::new(TimeScale::TT, jd_tt).to_scale(TimeScale::UTC).jd)
}

/// 时家一局
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShiJu {
    /// 阳遁为 true
    pub yang: bool,
    /// 局数 1—9
    pub ju: u8,
    /// 所用节气（0 春分 … 23 驚蟄）
    pub term: usize,
    pub yuan: Yuan,
    /// 置闰法之闰奇（叠用芒种或大雪）
    pub run: bool,
    /// 置闰法：本节上元符头日距该节气交节日的日数（节日 − 符头日；正为超、负为接、0 为正授；闰奇为相对所叠之节）。拆补法为 0
    pub chao: i64,
}

impl ShiJu {
    pub fn describe(&self) -> String {
        let run = if self.run { "閏" } else { "" };
        let dun = if self.yang { "陽" } else { "隂" };
        format!("{run}{}{} {}遁{}局", JIE_QI[self.term], self.yuan.name(), dun, self.ju)
    }
}

/// 一段时间的节气表（按来源）：(交节时辰序, 交节民用日, 节气序)，时间升序。
/// 交节日取民用日（子正为界）：原典数超接日数按历书所载交节之日（万历二十六年大雪「夜子初」交仍记庚寅，「已過十一日」自己卯数起）。
#[derive(Clone, Debug)]
pub struct QiTable {
    terms: Vec<(i64, i64, usize)>,
}

impl QiTable {
    /// 覆盖公历 y0—y1 年（各向外多取一年）
    pub fn new(vsop: &Vsop87, y0: i32, y1: i32, source: JieQiSource) -> Self {
        let mut terms = Vec::new();
        for y in (y0 - 1)..=(y1 + 1) {
            for (k, jd) in solar_term_jds_for_year_cached(vsop, y).iter().enumerate() {
                let jd = match source {
                    JieQiSource::DingQi => *jd,
                    JieQiSource::XingYong => {
                        let q = (k as i32 + 24 - 18) % 24;
                        pingqi::pingqi_tt(*jd, q).map(|(_, t)| t).unwrap_or(*jd)
                    }
                };
                let s = shichen_index_from_tt(jd);
                let ut = TimePoint::new(TimeScale::TT, jd).to_scale(TimeScale::UTC).jd.as_f64();
                terms.push((s, (ut + 8.0 / 24.0 + 0.5).floor() as i64, k));
            }
        }
        terms.sort();
        terms.dedup();
        Self { terms }
    }

    /// 由日期（儒略日数，东八区民用日）造表
    pub fn around(vsop: &Vsop87, jdn: i64, source: JieQiSource) -> Self {
        let (y, _, _) = Gregorian::from_julian_day(real(jdn as f64));
        Self::new(vsop, y - 8, y, source)
    }

    /// 时辰 s 所在节气（交节时辰起属新节气）
    fn term_at(&self, s: i64) -> usize {
        self.terms.iter().filter(|(t, _, _)| *t <= s).last().expect("节气表不足").2
    }

    /// 拆补法定局
    pub fn chai_bu(&self, shichen: i64) -> ShiJu {
        let term = self.term_at(shichen);
        let yuan = yuan_of_day(shichen.div_euclid(12));
        ShiJu { yang: is_yang(term), ju: ju(term, yuan), term, yuan, run: false, chao: 0 }
    }

    /// 置闰法：自 `anchor_before` 日以前最近一个「正授」（上元符头日恰逢交节或交气）起，逐元顺推至 `jdn`。
    /// 返回所在上元符头日、节气在表中的位置、是否闰奇。
    fn zhi_run_block(&self, jdn: i64, anchor_before: i64, rule: RunRule) -> (i64, usize, bool) {
        let shang = |d: i64| day_gz(d) % 15 == 0; // 甲子、己卯、甲午、己酉
        let days: Vec<i64> = self.terms.iter().map(|t| t.1).collect();
        // 正授起点
        let mut b = anchor_before - (day_gz(anchor_before) as i64 % 15);
        let (mut block, mut idx) = loop {
            if let Some(i) = days.iter().position(|d| *d == b) {
                break (b, i);
            }
            b -= 15;
            assert!(b > days[0], "节气表内找不到正授起点");
        };
        debug_assert!(shang(block));
        let mut run = false;
        while block + 15 <= jdn {
            let next = block + 15;
            let t = self.terms[idx].2;
            if !run && (t == MANG_ZHONG || t == DA_XUE) && rule.is_run(days[idx] - block, days[idx + 1] - next) {
                run = true; // 叠作本节，为闰奇
            } else {
                run = false;
                idx += 1;
            }
            block = next;
        }
        (block, idx, run)
    }

    /// 置闰法定局（时辰序）。正授起点取所求日前约两年半之内最早者，由其顺推（不同起点的一致性见测试）。
    pub fn zhi_run(&self, shichen: i64, rule: RunRule) -> ShiJu {
        let jdn = shichen.div_euclid(12);
        self.zhi_run_from(jdn, jdn - 900, rule)
    }

    fn zhi_run_from(&self, jdn: i64, anchor_before: i64, rule: RunRule) -> ShiJu {
        let (block, idx, run) = self.zhi_run_block(jdn, anchor_before, rule);
        let (_, term_day, term) = self.terms[idx];
        let yuan = match (jdn - block) / 5 {
            0 => Yuan::Shang,
            1 => Yuan::Zhong,
            _ => Yuan::Xia,
        };
        ShiJu { yang: is_yang(term), ju: ju(term, yuan), term, yuan, run, chao: term_day - block }
    }

    pub fn shi_ju(&self, shichen: i64, method: DingJu) -> ShiJu {
        match method {
            DingJu::ZhiRun(rule) => self.zhi_run(shichen, rule),
            DingJu::ChaiBu => self.chai_bu(shichen),
        }
    }
}

/// 时家定局：某时刻（JD(UT)）。缺省宜用 [`DEFAULT_DING_JU`] 与 [`JieQiSource::XingYong`]（见 doc/folk/qimen.md）
pub fn shi_jia_ju(vsop: &Vsop87, jd_ut: Real, method: DingJu, source: JieQiSource) -> ShiJu {
    let s = shichen_index_from_ut(jd_ut);
    QiTable::around(vsop, s.div_euclid(12), source).shi_ju(s, method)
}

/// 日家奇门：休门所在之卦（卷一「日家竒門」：自坎宫起甲子，每三日一换，顺飞八方不入中五；甲子复自坎起）
pub fn ri_jia_xiu_men(day_gz: u8) -> &'static str {
    ["坎", "坤", "震", "巽", "乾", "兑", "艮", "离"][((day_gz / 3) % 8) as usize]
}

/// 月家奇门：所在之元与局（卷一「月家竒門」：甲己遇四孟为上元一宫起甲子、四仲为中元七宫、四季为下元四宫，阴遁，每一元管五年）
pub fn yue_jia_ju(year_gz: u8) -> (Yuan, u8) {
    let fu = year_gz - year_gz % 5;
    match fu % 12 {
        2 | 5 | 8 | 11 => (Yuan::Shang, 1),
        0 | 3 | 6 | 9 => (Yuan::Zhong, 7),
        _ => (Yuan::Xia, 4),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::ephemeris::load_earth_vsop87_from_repo;

    const GZ: [&str; 60] = [
        "甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己巳", "庚午", "辛未", "壬申", "癸酉", "甲戌", "乙亥", "丙子", "丁丑", "戊寅",
        "己卯", "庚辰", "辛巳", "壬午", "癸未", "甲申", "乙酉", "丙戌", "丁亥", "戊子", "己丑", "庚寅", "辛卯", "壬辰", "癸巳",
        "甲午", "乙未", "丙申", "丁酉", "戊戌", "己亥", "庚子", "辛丑", "壬寅", "癸卯", "甲辰", "乙巳", "丙午", "丁未", "戊申",
        "己酉", "庚戌", "辛亥", "壬子", "癸丑", "甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥",
    ];
    fn gz(s: &str) -> u8 {
        GZ.iter().position(|g| *g == s).unwrap() as u8
    }
    fn term(s: &str) -> usize {
        JIE_QI.iter().position(|g| *g == s).unwrap()
    }

    #[test]
    fn ju_table() {
        // 《奇門遁甲統宗》「陰陽二遁順逆起例」歌诀（仅作对照）
        let song = [
            ("冬至", [1, 7, 4]), ("驚蟄", [1, 7, 4]), ("小寒", [2, 8, 5]), ("大寒", [3, 9, 6]), ("春分", [3, 9, 6]),
            ("芒種", [6, 3, 9]), ("穀雨", [5, 2, 8]), ("小滿", [5, 2, 8]), ("立春", [8, 5, 2]), ("清明", [4, 1, 7]),
            ("立夏", [4, 1, 7]), ("雨水", [9, 6, 3]), ("夏至", [9, 3, 6]), ("白露", [9, 3, 6]), ("小暑", [8, 2, 5]),
            ("大暑", [7, 1, 4]), ("秋分", [7, 1, 4]), ("立秋", [2, 5, 8]), ("霜降", [5, 8, 2]), ("小雪", [5, 8, 2]),
            ("大雪", [4, 7, 1]), ("處暑", [1, 4, 7]), ("立冬", [6, 9, 3]), ("寒露", [6, 9, 3]),
        ];
        for (t, v) in song {
            let k = term(t);
            assert_eq!([ju(k, Yuan::Shang), ju(k, Yuan::Zhong), ju(k, Yuan::Xia)], v, "{t}");
        }
    }

    #[test]
    fn yuan_and_ri_jia() {
        // 卷一：甲子至戊辰上元，己巳至癸酉中元，甲戌至戊寅下元（万历二十五年小满例）
        let d0 = 2_451_545i64 - day_gz(2_451_545) as i64; // 某甲子日
        assert_eq!(day_gz(d0), 0);
        assert_eq!(yuan_of_day(d0 + 4), Yuan::Shang);
        assert_eq!(yuan_of_day(d0 + 5), Yuan::Zhong);
        assert_eq!(yuan_of_day(d0 + 10), Yuan::Xia);
        assert_eq!(yuan_of_day(d0 + 15), Yuan::Shang); // 己卯
        // 卷一日家：甲子乙丑丙寅在坎……辛酉壬戌癸亥在巽，甲子又复自坎
        assert_eq!(ri_jia_xiu_men(gz("丁卯")), "坤");
        assert_eq!(ri_jia_xiu_men(gz("己卯")), "兑");
        assert_eq!(ri_jia_xiu_men(gz("戊子")), "坎");
        assert_eq!(ri_jia_xiu_men(gz("辛酉")), "巽");
        // 卷一月家：甲己遇四孟上元一宫，四仲中元七宫，四季下元四宫
        assert_eq!(yue_jia_ju(gz("甲寅")), (Yuan::Shang, 1));
        assert_eq!(yue_jia_ju(gz("戊午")), (Yuan::Shang, 1)); // 甲寅元管五年
        assert_eq!(yue_jia_ju(gz("己卯")), (Yuan::Zhong, 7));
        assert_eq!(yue_jia_ju(gz("甲辰")), (Yuan::Xia, 4));
    }

    /// 以「节气 + 干支」定日：取该节气（行用历）交节民用日前后 25 日内同干支之日
    fn locate(vsop: &Vsop87, year: i32, t: &str, g: &str) -> i64 {
        let tt = solar_term_jds_for_year_cached(vsop, year)[term(t)];
        let tt = pingqi::pingqi_tt(tt, (term(t) as i32 + 6) % 24).map(|x| x.1).unwrap_or(tt);
        let d = (TimePoint::new(TimeScale::TT, tt).to_scale(TimeScale::UTC).jd.as_f64() + 8.0 / 24.0 + 0.5).floor() as i64;
        let c: Vec<i64> = (d - 25..=d + 25).filter(|x| day_gz(*x) == gz(g)).collect();
        assert_eq!(c.len(), 1, "{year}{t}{g}");
        c[0]
    }
    /// 某日某时辰（子 0 … 亥 11）的绝对时辰序；子时属当日（子初换日）
    fn sc(d: i64, shi: i64) -> i64 {
        d * 12 + shi
    }

    fn check(table: &QiTable, s: i64, m: DingJu, want: &str, chao: Option<i64>) {
        let j = table.shi_ju(s, m);
        assert_eq!(j.describe(), want, "{} {}时", GZ[day_gz(s.div_euclid(12)) as usize], s.rem_euclid(12));
        if let Some(c) = chao {
            assert_eq!(j.chao, c, "{want}");
        }
    }

    /// 《遁甲演義》卷一「又引證」：萬厯二十四年丙申立春、雨水、驚蟄（拆补）
    #[test]
    fn wanli24_chai_bu() {
        let v = load_earth_vsop87_from_repo().unwrap();
        let t = QiTable::new(&v, 1594, 1597, JieQiSource::XingYong);
        let lc = locate(&v, 1596, "立春", "丙子");
        let m = DingJu::ChaiBu;
        check(&t, sc(lc, 0), m, "大寒下元 陽遁6局", None); // 丙子日戊子時還是先年大寒下局
        check(&t, sc(lc, 1), m, "立春下元 陽遁2局", None); // 自丑時借用立春下局，陽二遁坤二起甲子
        check(&t, sc(lc + 3, 0), m, "立春上元 陽遁8局", None); // 十二己夘日符頭纔到……立春上局陽遁八宫
        check(&t, sc(lc + 8, 0), m, "立春中元 陽遁5局", None); // 十七甲申日……立春中局陽五遁
        check(&t, sc(lc + 13, 0), m, "立春下元 陽遁2局", None); // 二十二日己丑……補足前立春下局
        let ys = locate(&v, 1596, "雨水", "辛卯");
        check(&t, sc(ys, 3), m, "立春下元 陽遁2局", None); // 至辛夘日夘時止
        check(&t, sc(ys, 4), m, "雨水下元 陽遁3局", None); // 辛夘日辰時雨水……借雨水下局陽三遁
        check(&t, sc(ys + 3, 0), m, "雨水上元 陽遁9局", None); // 二十七甲午日……雨水上局陽九遁
        check(&t, sc(ys + 8, 0), m, "雨水中元 陽遁6局", None); // 二月初二己亥……雨水中局陽六遁
        let jz = locate(&v, 1596, "驚蟄", "丙午");
        check(&t, sc(jz, 5), m, "雨水下元 陽遁3局", None); // 初九丙午日巳時上共三十時俱補足前雨水下局
        check(&t, sc(jz, 8), m, "驚蟄下元 陽遁4局", None); // 自午時起……驚蟄下局陽四（書午時交，本项目平气推得未時）
        check(&t, sc(jz + 3, 0), m, "驚蟄上元 陽遁1局", None); // 十二己酉日……驚蟄上局陽一遁
        check(&t, sc(jz + 8, 0), m, "驚蟄中元 陽遁7局", None); // 十七甲寅日……驚蟄中局陽七遁
    }

    /// 《遁甲演義》卷三「冬至陽遁時竒起例」「夏至隂遁時竒起例」：萬厯二十五年（置闰法）
    #[test]
    fn wanli25_zhi_run() {
        let v = load_earth_vsop87_from_repo().unwrap();
        let t = QiTable::new(&v, 1590, 1598, JieQiSource::XingYong);
        let m = DEFAULT_DING_JU;
        // 四月十二壬申日丙午時：初四甲子符頭已到，初七丁夘戌時小滿，超；壬申為小滿中局陽遁二
        let d = locate(&v, 1597, "小滿", "壬申");
        check(&t, sc(d, 6), m, "小滿中元 陽遁2局", Some(3));
        // 五月十一辛丑日丙申時：初四甲午符頭，初八戊戌夏至，超；夏至中局隂遁三
        let d = locate(&v, 1597, "夏至", "辛丑");
        check(&t, sc(d, 8), m, "夏至中元 隂遁3局", Some(4));
    }

    /// 《遁甲演義》卷一「又引證」：萬厯二十六年至二十九年的超、閏、接、正授（置闰法）
    #[test]
    fn wanli26_29_zhi_run() {
        let v = load_earth_vsop87_from_repo().unwrap();
        let t = QiTable::new(&v, 1592, 1602, JieQiSource::XingYong);
        let m = DEFAULT_DING_JU;
        // 二十六年正月初八甲午符頭已到，十五日戌時雨水：超
        check(&t, sc(locate(&v, 1598, "雨水", "甲午"), 0), m, "雨水上元 陽遁9局", Some(7));
        // 十月二十七己夘日超用大雪；十一月初九大雪已過十一日；十三甲午日疊作大雪（閏竒）
        check(&t, sc(locate(&v, 1598, "大雪", "己卯"), 0), m, "大雪上元 隂遁4局", Some(11));
        check(&t, sc(locate(&v, 1598, "大雪", "甲午"), 0), m, "閏大雪上元 隂遁4局", None);
        // 自十一月十三接起……至二十七年五月十七甲子日子時正交小暑：正授
        check(&t, sc(locate(&v, 1599, "小暑", "甲子"), 0), m, "小暑上元 隂遁8局", Some(0));
        // 八月初三己夘符頭到，初四庚辰丑時秋分：超
        check(&t, sc(locate(&v, 1599, "秋分", "己卯"), 0), m, "秋分上元 隂遁7局", Some(1));
        // 二十九年四月二十七甲午超用芒種，五月初七甲辰丑時交芒種已超過十日；五月十二己酉疊作芒種上局陽六遁（閏竒）
        check(&t, sc(locate(&v, 1601, "芒種", "甲午"), 0), m, "芒種上元 陽遁6局", Some(10));
        check(&t, sc(locate(&v, 1601, "芒種", "己酉"), 0), m, "閏芒種上元 陽遁6局", None);
    }

    /// 《遁甲演義》卷一「超接引例」：丙午（维基文库编者标作淳祐六年）立夏超，丁未清明接、立秋正授
    #[test]
    fn chunyou_zhi_run() {
        let v = load_earth_vsop87_from_repo().unwrap();
        let t = QiTable::new(&v, 1240, 1248, JieQiSource::XingYong);
        let m = DEFAULT_DING_JU;
        // 四月十三日壬申立夏而本月初五日是甲子……借用立夏上局；初十日己巳立夏中局；十六日甲戌立夏下局
        let d = locate(&v, 1246, "立夏", "甲子");
        check(&t, sc(d, 0), m, "立夏上元 陽遁4局", Some(8));
        check(&t, sc(d + 5, 0), m, "立夏中元 陽遁1局", None);
        check(&t, sc(d + 10, 0), m, "立夏下元 陽遁7局", None);
        // 丁未二月二十三日雖交清明，至二十五日是己酉始用清明上局：接
        check(&t, sc(locate(&v, 1247, "清明", "己酉"), 0), m, "清明上元 陽遁4局", Some(-2));
        // 其年六月二十八日己酉立秋，正值節與日辰同到，即立秋上局：正授
        check(&t, sc(locate(&v, 1247, "立秋", "己酉"), 0), m, "立秋上元 隂遁2局", Some(0));
    }

    /// 《奇門遁甲統宗》「置闰法」康熙五十六至五十八年（统宗为简体录文，仅作线索；与定气、清时宪历同）
    #[test]
    fn kangxi_zhi_run() {
        let v = load_earth_vsop87_from_repo().unwrap();
        let t = QiTable::new(&v, 1712, 1720, JieQiSource::XingYong);
        let m = DingJu::ZhiRun(RunRule::TongZong);
        // 五十六年五月十一日甲子為夏至符頭，已超三日（十三日丙寅夜子初交）
        // 书「丙寅夜子初二刻」交；本项目推得已过东八区子正（丁卯），超日差一，局同
        check(&t, sc(locate(&v, 1717, "夏至", "甲子"), 0), m, "夏至上元 隂遁9局", None);
        // 十一月二十日庚午寅正交冬至，「去符頭甲午日共超七日」（不置閏）。甲午距庚午三十六日，当是甲子之讹（甲子至庚午差六日，连头尾七日）
        check(&t, sc(locate(&v, 1717, "冬至", "甲子"), 0), m, "冬至上元 陽遁1局", Some(6));
        // 五十七年五月初一己酉芒種上局……十六日甲子不作夏至上局而為芒種閏竒上局；六月初二己卯始為夏至上局（接）
        check(&t, sc(locate(&v, 1718, "芒種", "己酉"), 0), m, "芒種上元 陽遁6局", None);
        check(&t, sc(locate(&v, 1718, "夏至", "甲子"), 0), m, "閏芒種上元 陽遁6局", None);
        check(&t, sc(locate(&v, 1718, "夏至", "己卯"), 0), m, "夏至上元 隂遁9局", Some(-7));
        // 五十八年六月二十三日立秋，甲子符頭恰當日，是為正授，本日即是隂遁二局
        check(&t, sc(locate(&v, 1719, "立秋", "甲子"), 0), m, "立秋上元 隂遁2局", Some(0));
        // 上元至七月初九日庚辰處暑即超一日
        check(&t, sc(locate(&v, 1719, "處暑", "己卯"), 0), m, "處暑上元 隂遁1局", Some(1));
    }

    /// 置闰法与正授起点无关：自不同起点顺推，结果一致
    #[test]
    fn zhi_run_anchor_independent() {
        let v = load_earth_vsop87_from_repo().unwrap();
        let t = QiTable::new(&v, 2010, 2027, JieQiSource::DingQi);
        let start = Gregorian::to_julian_day(2024, 1, 1).as_f64().round() as i64;
        for d in (start..start + 730).step_by(3) {
            for rule in [RunRule::YanYi, RunRule::TongZong] {
                let a = t.zhi_run_from(d, d - 900, rule);
                for back in [1300, 2000, 3000] {
                    assert_eq!(t.zhi_run_from(d, d - back, rule), a, "日 {d} 起点前推 {back}");
                }
                // 超接日数有界：超不过十二日，接不过七日
                assert!((-7..=12).contains(&a.chao) || a.run, "日 {d}：{a:?}");
            }
        }
    }
}
