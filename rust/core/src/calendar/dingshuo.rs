//! 唐至明定朔诸历推步（历法域第 1 层·历谱生成；按历名供第 4 层 [crate::calendar::lifa::resolve_year] 取用）。
//!
//! 由 `scripts/calendar_data/*_dingshuo.py` 的复原移植，逐历对照：戊寅（定朔、平朔）、麟德、大衍、五纪、正元、宣明、崇玄、
//! 宋诸历（崇天、明天、观天、纪元、统元、乾道、淳熙、会元、统天、开禧、成天）、授时、大统。常数、表与校改的出处见各脚本
//! 文档与 `doc/calendar/calendar-compendium.md`；宋诸历之表由 `export_dingshuo_reference.py` 自维基文库底本解析、校改后导出为
//! `data/calendar/dingshuo_song_tables.csv`。移植正确性以 `data/calendar/dingshuo_reference.csv`（Python 所算定朔，精确分数）
//! 逐月比对（测试 `matches_python_reference`）。
//!
//! 排月（Python 复原只算定朔，此处新增）：定朔所在日，小余过进朔限者进一日；恒气（常气）中气，十一月为含冬至之月，
//! 两冬至间 13 月者以第一个无中气之月为闰。进朔之外的细则（[Paiyue]，各历行用年段前后各留二岁一次排定）：
//! - 「使不過三大二小」（[Rules::runs]）：崇天历文「隨其所近而進退之，不過三大二小」（《宋史》卷七十一），明天以后宋诸历同；
//!   唐大衍术注「使不過三大三小」（钱宝琮校作「三大二小」，本项目取原文，见 [TANG_RUNS]），五纪、正元、宣明承之。
//!   逐月推：进朔会成连大超限则不进，不进会成连小超限则进，只改当月。
//! - 麟德正月进朔（[Rules::zhengyue]）：684 年「詔二年元日用甲申，故進以癸未晦」，此后正月定朔过戌初（19 时）者进；
//!   常月 709 年起四分之三进朔（[PaiyueOptions::linde_changyue]），此前不进。
//! - 交会（[PaiyueOptions]，2026-11-18 起）：照各历「步交會」求定朔入交定（[Jiaohui]，常数见 [JiaoConst] 诸处与 `song_jiao`），
//!   宣明、崇天另照志文日蚀术判有蚀与食甚（[Rishi]）。宣明「或有交，應見虧初，則否」、宋诸历「有食應見者，其朔不進」（[JiaoBujin]）：
//!   过进朔限而有食、亏初不晚于日入者不进；大衍术注「正月朔有交、加時正見者……令虧在晦、二」只在史载有据之年
//!   （[TANG_ZHENGYUE_JIAO]、[WUDAI_ZHENGYUE_JIAO]）。检验见 `examples/jiaohui_eclipse_check.rs`、`examples/jiaohui_options_eval.rs`，
//!   Python 复原 `scripts/calendar_data/jiaohui.py`（逐月比对 `data/calendar/jiaohui_reference.csv`）。
//! - 史载例外（`data/calendar/dingshuo_exceptions.csv`）：诏令改朔、中原移闰，及交会规则推不出之史载月（宣明 824、836 年夜食不进，
//!   崇天 1044 年十一月）。
//!
//! 旧做法（2026-11-13 前）只按进朔限，不加上述细则，仍可由 [reckon_year_shuwen] 取得（麟德「不過頻三」照旧）。
//! 全部用有理数（日法之分），进朔判定无浮点误差。

use super::gregorian;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};

type Q = BigRational;

fn q(n: i64) -> Q {
    Q::from_integer(BigInt::from(n))
}
fn qr(n: i64, d: i64) -> Q {
    Q::new(BigInt::from(n), BigInt::from(d))
}
fn qbig(n: i128, d: i128) -> Q {
    Q::new(BigInt::from(n), BigInt::from(d))
}
fn fl(x: &Q) -> i64 {
    x.floor().to_integer().to_i64().expect("floor")
}
/// x mod m（m > 0），结果在 [0, m)
fn md(x: &Q, m: &Q) -> Q {
    x - m * Q::from_integer((x / m).floor().to_integer())
}
fn parse_q(s: &str) -> Q {
    let (a, b) = s.split_once('/').unwrap_or((s, "1"));
    Q::new(a.trim().parse::<BigInt>().unwrap(), b.trim().parse::<BigInt>().unwrap())
}

/// 上元日序 → JDN 之差：取 approx_jdn 前后 40 日内干支与日序相合者（与 `dingshuo_check.align_offset` 同）。
fn align_offset(day_index: i64, approx_jdn: i64) -> i64 {
    (-40..40)
        .map(|k| approx_jdn + k)
        .filter(|j| (j + 49).rem_euclid(60) == day_index.rem_euclid(60))
        .min_by_key(|j| (j - approx_jdn).abs())
        .unwrap()
        - day_index
}

/// 一部定朔历。单位为日法之分，自上元甲子夜半起算。
pub trait DingshuoLi: Send + Sync {
    fn f(&self) -> i64;
    /// 上元日序 → JDN 之差
    fn off(&self) -> i64;
    /// 岁实（分）
    fn sui(&self) -> Q;
    /// 冬至岁 y（y−1 年十一月至 y 年十月）的天正冬至（常气，分）
    fn dongzhi(&self, y: i32) -> Q;
    /// 冬至岁 y 自天正经朔起第 m 月（m ≥ 0）：(经朔, 定朔)
    fn month(&self, y: i32, m: i64) -> (Q, Q);
    /// 定朔 t（属冬至岁 y）的进朔限：小余（分）不小于此者进一日；None 为不进朔
    fn jinshuo_limit(&self, _y: i32, _t: &Q) -> Option<Q> {
        None
    }
    /// 注历所用定朔：缺省同 [DingshuoLi::month]；麟德历定小余近夜半者以本术重算
    fn ding_for_calendar(&self, y: i32, m: i64) -> Q {
        self.month(y, m).1
    }
    /// 已按历法另有规定（如麟德「不過頻三」）调整过的全部朔日 JDN（升序）；None 则逐岁现排
    fn fixed_days(&self) -> Option<&[i64]> {
        None
    }
    /// 史载须置闰之月的朔日 JDN（[Paiyue] 据例外表给出）；排月时两冬至间 13 月者以此月为闰
    fn forced_leaps(&self) -> &[i64] {
        &[]
    }
    /// 冬至岁 y 的十三中气（恒气，天正冬至起至下一冬至）所在日 JDN，排月定闰用。缺省按本历冬至、岁实十二等分
    fn zhongqi(&self, y: i32) -> Vec<i64> {
        zhongqi_hengqi(self, y)
    }
    /// 冬至岁 y 第 m 月定朔加时之入交（[Jiaohui]）；None 为本历未复原步交会
    fn jiao(&self, _y: i32, _m: i64) -> Option<Jiaohui> {
        None
    }
    /// 时刻 t（属冬至岁 y）所在日之日出（占一日之比，夜半起算）；None 为未复原
    fn sunrise(&self, _y: i32, _t: &Q) -> Option<Q> {
        None
    }
    /// 冬至岁 y 第 m 月定朔之日食（照本历日蚀术加蚀差等）；None 为未复原，排月以 [Jiaohui::shixian] 代之
    fn rishi(&self, _y: i32, _m: i64) -> Option<Rishi> {
        None
    }
    /// 冬至岁 y 第 m 月经朔所入日躔、月离表段（供逐月考证，见 [Biaoduan]）；None 为未提供
    fn biaoduan(&self, _y: i32, _m: i64) -> Option<Biaoduan> {
        None
    }
}

/// 经朔所入表段（供逐月考证，`examples/dingshuo_zhangbiao_diffs.rs`）：定朔所用日躔、月离表的哪一格，
/// 以查临界之月是否落在分段处、所用格是否为校改或疑讹之格。时刻皆按经朔（定朔即由此二格的损益加减而得）。
#[derive(Clone, Debug, PartialEq)]
pub struct Biaoduan {
    /// 月离：(入转（入历、入变）第几日，1 起；入余（占一日之比）；分段之日的初数（占一日之比，非分段日 None）)。
    /// 宣明前后半周两表，后半周日序加 14；授时、明天等以公式求者亦给入转日序（无表格可查）
    pub yue: Option<(u32, f64, Option<f64>)>,
    /// 日躔：(所入气序，0 = 冬至；入气日数（含余）；本气日数)。定气之历按定气，恒气之历按恒气
    pub ri: (usize, f64, f64),
}

/// 冬至岁 y 的十三恒气中气所在日 JDN：天正冬至加岁实 k/12（k = 0..=12）
fn zhongqi_hengqi<L: DingshuoLi + ?Sized>(li: &L, y: i32) -> Vec<i64> {
    let f = q(li.f());
    let (dz, sui) = (li.dongzhi(y), li.sui());
    (0..=12).map(|k| fl(&((&dz + &sui * qr(k, 12)) / &f)) + li.off()).collect()
}

/// 照本历日蚀术判得之日食
#[derive(Clone, Debug, PartialEq)]
pub struct Rishi {
    /// 有蚀
    pub shi: bool,
    /// 蚀在阴历（加减蚀差之后；阳历「不足減，反減蝕差」入阴历者亦为阴历）
    pub yin: bool,
    /// 食甚（蚀定余，占一日之比）
    pub shen: Q,
    /// 定用（食甚至亏初，占一日之比）；None 为本历未复原，排月取 [JiaoBujin::wan_min]
    pub dingyong: Option<Q>,
}

/// 步交会常数（日法之分）。唐宋诸历同一术：经朔入交泛 = 朔积分 mod 交终；「以其日入氣朓朒定數，朓減、朒加交汎，為入交常」；
/// 「以交率乘其日入轉朓朒定數，如交數而一，以朓減、朒加入交常，為入交定」；入交定在交中已下为阳历，已上去之为阴历；
/// 「入交定如望差已下，交限已上，為入蝕限」（大衍，宋诸历「後限以下、前限已上」同）。
#[derive(Clone)]
struct JiaoConst {
    /// 交终（分）
    zhong: Q,
    /// 望差（宋诸历「後限」，宣明「後準」）：入食限之半宽（分）
    xian: Q,
    /// 交率、交数
    lv: i64,
    shu: i64,
    /// 入交泛之应（分，自上元起算者 0；统天「交差」）
    cha: Q,
}

impl JiaoConst {
    fn new(zhong: Q, xian: Q, lv: i64, shu: i64) -> Self {
        JiaoConst { zhong, xian, lv, shu, cha: Q::zero() }
    }
    /// 经朔 jing（分，自上元）及其入气、入转朓朒定数（朒 +）→ 入交定（分，[0, 交终)）
    fn ru(&self, jing: &Q, sun: &Q, moon: &Q, f: i64) -> Jiaohui {
        let ru = md(&(md(&(jing - &self.cha), &self.zhong) + sun + moon * qr(self.lv, self.shu)), &self.zhong);
        Jiaohui::new(ru, &self.zhong, &self.xian, f)
    }
}

/// 定朔加时之入交（照各历「步交會」之简法：只到入交定日与入食限，不算气差、刻差等蚀差与食分）
#[derive(Clone, Debug, PartialEq)]
pub struct Jiaohui {
    /// 入交定（日）
    pub ru: Q,
    /// 去交（日）：距最近交点（交初或交中）
    pub qu: Q,
    /// 月在阴历（入交定在交中已上）
    pub yin: bool,
    /// 交前（将至交点）；否则交后
    pub qian: bool,
    /// 入食限：去交不过望差（后限）
    pub shixian: bool,
}

impl Jiaohui {
    fn new(ru: Q, zhong: &Q, xian: &Q, f: i64) -> Self {
        let half = zhong / q(2);
        let yin = ru >= half;
        let x = if yin { &ru - &half } else { ru.clone() };
        let qian = &x * q(2) > half;
        let qu = if qian { &half - &x } else { x };
        let fq = q(f);
        Jiaohui { shixian: qu <= *xian, ru: ru / &fq, qu: qu / &fq, yin, qian }
    }
}

/// 定朔 t 所在日（上元日序）与是否过进朔限
fn day_jin(li: &dyn DingshuoLi, y: i32, t: &Q) -> (i64, bool) {
    let f = q(li.f());
    let day = fl(&(t / &f));
    let xiao = t - q(day) * &f;
    (day, li.jinshuo_limit(y, t).is_some_and(|l| xiao >= l))
}

fn day_of(li: &dyn DingshuoLi, y: i32, t: &Q) -> i64 {
    let (day, jin) = day_jin(li, y, t);
    day + i64::from(jin)
}

/// 冬至岁 y0..=y1 的各月朔日：(JDN, 定小余占一日之比)，升序。每个经朔只由其所在冬至岁算一次（经朔 < 下一冬至岁天正经朔），
/// 免得相邻两岁所算定朔微异而重出。
fn month_starts(li: &dyn DingshuoLi, y0: i32, y1: i32) -> Vec<(i64, Q)> {
    let off = li.off();
    let f = q(li.f());
    let mut out = Vec::new();
    for yy in y0..=y1 {
        let next0 = li.month(yy + 1, 0).0;
        for m in 0..15 {
            let (jing, _) = li.month(yy, m);
            if jing >= next0 {
                break;
            }
            let ding = li.ding_for_calendar(yy, m);
            let frac = md(&ding, &f) / &f;
            out.push((day_of(li, yy, &ding) + off, frac));
        }
    }
    out.sort_by_key(|x| x.0);
    out.dedup_by_key(|x| x.0);
    out
}

/// 「盈朒之極，不過頻三。其或過者，觀定小餘近夜半者量之」（《旧唐书·历志二》麟德历）：连大（30 日）或连小（29 日）
/// 超过 max_run 月时，在该段首尾两朔中取定小余距夜半较近者挪一日——段首之朔连大则后移、连小则前移，段尾反之——
/// 使段断开；反复至无超限之段。「量之」原文未定细则，此取最近夜半者，属读法。
fn limit_runs(starts: &mut [(i64, Q)], max_run: usize) {
    for _ in 0..10_000 {
        let lens: Vec<i64> = starts.windows(2).map(|w| w[1].0 - w[0].0).collect();
        let mut i = 0;
        let mut fixed = false;
        while i < lens.len() {
            let mut j = i;
            while j + 1 < lens.len() && lens[j + 1] == lens[i] {
                j += 1;
            }
            if j - i + 1 > max_run {
                let (s, e) = (i, j + 1);
                let big = lens[i] == 30;
                let one = q(1);
                // (距夜半, 朔序, 挪动)
                let cand = if big {
                    [(&one - &starts[s].1, s, 1i64), (starts[e].1.clone(), e, -1)]
                } else {
                    [(starts[s].1.clone(), s, -1i64), (&one - &starts[e].1, e, 1)]
                };
                let (_, b, d) = cand.into_iter().min_by(|a, b| a.0.cmp(&b.0)).unwrap();
                starts[b].0 += d;
                fixed = true;
                break;
            }
            i = j + 1;
        }
        if !fixed {
            return;
        }
    }
}

/// 一月的定朔：定朔所在日（JDN）、是否过进朔限、定小余占一日之比
struct RawMonth {
    base: i64,
    jin: bool,
    frac: Q,
    /// 入交（本历复原步交会者）
    jiao: Option<Jiaohui>,
    /// 定朔所在日之日出（占一日之比）
    sunrise: Option<Q>,
    /// 本历日蚀术所判（复原者）
    rishi: Option<Rishi>,
}

impl RawMonth {
    /// 加时在昼：日出 ≤ 定小余 ≤ 日入（日入 = 1 − 日出）
    fn zhou(&self) -> bool {
        self.sunrise.as_ref().is_some_and(|sr| self.frac >= *sr && self.frac <= q(1) - sr)
    }
}

/// 冬至岁 y0..=y1 的各月定朔，升序；每个经朔只由其所在冬至岁算一次（同 [month_starts]）
fn raw_months(li: &dyn DingshuoLi, y0: i32, y1: i32) -> Vec<RawMonth> {
    let off = li.off();
    let f = q(li.f());
    let mut out = Vec::new();
    for yy in y0..=y1 {
        let next0 = li.month(yy + 1, 0).0;
        for m in 0..15 {
            let (jing, _) = li.month(yy, m);
            if jing >= next0 {
                break;
            }
            let ding = li.ding_for_calendar(yy, m);
            let (day, jin) = day_jin(li, yy, &ding);
            out.push(RawMonth { base: day + off, jin, frac: md(&ding, &f) / &f, jiao: li.jiao(yy, m), sunrise: li.sunrise(yy, &ding), rishi: li.rishi(yy, m) });
        }
    }
    out.sort_by_key(|x| x.base);
    out.dedup_by_key(|x| x.base);
    out
}

/// 史载例外之调整（`data/calendar/dingshuo_exceptions.csv`「调整」列）
#[derive(Clone, Copy, PartialEq, Debug)]
enum Liwai {
    /// 朔日 = 定朔所在日 + k（进 1、不进 0、退 −1）
    Shift(i64),
    /// 本月为闰
    Leap,
}

/// 史载例外表：(历名, 调整后朔日 JDN, 调整)
fn liwai_table() -> &'static [(String, i64, Liwai)] {
    use std::sync::OnceLock;
    static T: OnceLock<Vec<(String, i64, Liwai)>> = OnceLock::new();
    T.get_or_init(|| {
        let csv = include_str!("../../../../data/calendar/dingshuo_exceptions.csv");
        csv.lines()
            .filter(|l| !l.starts_with('#') && !l.starts_with("历,") && !l.trim().is_empty())
            .map(|l| {
                let c: Vec<&str> = l.split(',').collect();
                let jdn: i64 = c[4].parse().expect("朔日JDN");
                assert_eq!(crate::calendar::gan_zhi::GAN_ZHI_60[(jdn + 49).rem_euclid(60) as usize], c[5], "例外表 {l}");
                let adj = match c[6] {
                    "进" => Liwai::Shift(1),
                    "不进" => Liwai::Shift(0),
                    "退" => Liwai::Shift(-1),
                    "置闰" => Liwai::Leap,
                    x => panic!("例外表调整「{x}」"),
                };
                assert!(matches!(c[7], "他书" | "本纪"), "例外表类别「{}」", c[7]);
                (c[0].to_string(), jdn, adj)
            })
            .collect()
    })
}

/// 史载例外表各条：(历名, 年, 调整后朔日 JDN, 是否置闰, 类别「他书」/「本纪」)。供定朔诸历以外的推步（宋初诸历、清初新法）
/// 在推步结果上应用（[super::push_calendar::reckon_lifa_year]），及主序列检验另计「从本纪修正」。
pub fn shizai_liwai() -> &'static [(String, i32, i64, bool, String)] {
    use std::sync::OnceLock;
    static T: OnceLock<Vec<(String, i32, i64, bool, String)>> = OnceLock::new();
    T.get_or_init(|| {
        include_str!("../../../../data/calendar/dingshuo_exceptions.csv")
            .lines()
            .filter(|l| !l.starts_with('#') && !l.starts_with("历,") && !l.trim().is_empty())
            .map(|l| {
                let c: Vec<&str> = l.splitn(9, ',').collect();
                (c[0].to_string(), c[1].parse().expect("年"), c[4].parse().expect("朔日JDN"), c[6] == "置闰", c[7].to_string())
            })
            .collect()
    })
}

/// 「消息前後一兩月，以定月之大小」（大衍「正月朔有交」条，麟德同）：第 i 月朔按史载移日后，相邻月长若出 29、30 日，
/// 顺次移后月（向后）或前月（向前）一日，至月长复归 29、30 日为止。如 937 年正月退作甲寅，二月随退作甲申。
fn xiaoxi(days: &mut [i64], i: usize) {
    let mut j = i + 1;
    while j < days.len() {
        match days[j] - days[j - 1] {
            l if l > 30 => days[j] -= 1,
            l if l < 29 => days[j] += 1,
            _ => break,
        }
        j += 1;
    }
    let mut j = i;
    while j >= 1 {
        match days[j] - days[j - 1] {
            l if l > 30 => days[j - 1] += 1,
            l if l < 29 => days[j - 1] -= 1,
            _ => break,
        }
        j -= 1;
    }
}

/// 进朔之外的排月细则
#[derive(Clone, Copy, Default)]
struct Rules {
    /// 「使不過三大二小」：(连大至多, 连小至多)。逐月推，进朔会使连大超限则不进，不进会使连小超限则进，只改当月
    runs: Option<(usize, usize)>,
    /// 麟德「不過頻三」（[limit_runs]，旧有）
    pin_san: bool,
    /// 正月进朔：(起农历年, 限时)。正月（含雨水中气之月）定小余过限者进一日
    zhengyue: Option<(i32, i64)>,
    /// 有交不进（[JiaoBujin]）
    jiao_bujin: Option<JiaoBujin>,
    /// 「其正月朔有交、加時正見者，消息前後一兩月，以定大小，令虧在晦、二」（大衍术注）：(起农历年, 止农历年, 移日：
    /// +1 进令亏在晦、−1 退令亏在二日)。只在史载有据之年段启用（见 [TANG_ZHENGYUE_JIAO]）
    zhengyue_jiao: &'static [(i32, i32, i64)],
}

/// 「有交不进」：定小余过进朔限而本月有日食者不进。有食与否：本历复原日蚀术者（[DingshuoLi::rishi]）照之，否则以入食限
/// （[Jiaohui::shixian]）代之。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JiaoBujin {
    /// 只取阴历之食（唐宋诸历「朔入蝕限，月在陰曆，則日蝕」；宣明阳历「不足減」入阴历者亦算）
    pub yin: bool,
    /// 「應見虧初」：亏初（食甚减半定用，此取定值若干分钟，一日 1440 分）不晚于日入者方不进；None 为夜食亦不进。
    /// 食甚：复原日蚀术者用其蚀定余（[Rishi::shen]），否则用定朔加时
    pub wan_min: Option<i64>,
}

impl JiaoBujin {
    fn applies(&self, m: &RawMonth) -> bool {
        let (shi, yin) = match (&m.rishi, &m.jiao) {
            (Some(r), _) => (r.shi, r.yin),
            (None, Some(j)) => (j.shixian, j.yin),
            _ => return false,
        };
        if !shi || (self.yin && !yin) {
            return false;
        }
        match (self.wan_min, &m.sunrise) {
            (None, _) => true,
            (Some(w), Some(sr)) => {
                let kuichu = match &m.rishi {
                    Some(Rishi { shen, dingyong: Some(dy), .. }) => shen - dy,
                    Some(r) => &r.shen - qr(w, 1440),
                    None => &m.frac - qr(w, 1440),
                };
                kuichu <= q(1) - sr
            }
            (Some(_), None) => false,
        }
    }
}

/// 一部定朔历连同其排月细则：各历行用年段（[REGISTRY] 中 `names` 诸名之年段）前后各留二岁，一次排定全部朔日。
struct Paiyue {
    li: Box<dyn DingshuoLi>,
    /// 此历在 [REGISTRY] 与例外表中的历名
    names: &'static [&'static str],
    rules: Rules,
    /// 定闰另用他历恒气中气之冬至岁段：(他历, [(起冬至岁, 止冬至岁)])。五代「宣明、崇玄參而成之」：朔用宣明定朔，中气用崇玄（[WudaiDingqi]）
    zhongqi_li: Option<(Box<dyn DingshuoLi>, &'static [(i32, i32)])>,
    /// 细则全用：(朔日, 置闰之月)
    full: std::sync::OnceLock<(Vec<i64>, Vec<i64>)>,
    /// 只按术文进朔（旧做法，麟德「不過頻三」照旧）
    plain: std::sync::OnceLock<(Vec<i64>, Vec<i64>)>,
}

impl Paiyue {
    fn new(li: impl DingshuoLi + 'static, names: &'static [&'static str], rules: Rules) -> Self {
        Paiyue { li: Box::new(li), names, rules, zhongqi_li: None, full: Default::default(), plain: Default::default() }
    }

    /// 冬至岁段 spans 内定闰改用 qi 历之恒气中气
    fn with_zhongqi(mut self, qi: impl DingshuoLi + 'static, spans: &'static [(i32, i32)]) -> Self {
        if !spans.is_empty() {
            self.zhongqi_li = Some((Box::new(qi), spans));
        }
        self
    }

    fn span(&self) -> (i32, i32) {
        let spans = REGISTRY.iter().chain(REGIME_REGISTRY).filter(|(n, _)| self.names.contains(n)).flat_map(|(_, s)| s.iter());
        let (a, b) = spans.fold((i32::MAX, i32::MIN), |(a, b), &(x, y)| (a.min(x), b.max(y)));
        (a - 2, b + 2)
    }

    fn days(&self, full: bool) -> &(Vec<i64>, Vec<i64>) {
        let cell = if full { &self.full } else { &self.plain };
        cell.get_or_init(|| self.compute(full))
    }

    fn compute(&self, full: bool) -> (Vec<i64>, Vec<i64>) {
        let li = self.li.as_ref();
        let (y0, y1) = self.span();
        let raw = raw_months(li, y0, y1);
        let mut days: Vec<i64> = raw.iter().map(|m| m.base + i64::from(m.jin)).collect();
        let mut locked = vec![false; days.len()];
        let mut leaps = Vec::new();
        let f = q(li.f());
        if let (true, Some((from, h))) = (full, self.rules.zhengyue) {
            for y in y0.max(from)..=y1 {
                // 正月：含雨水（冬至后第二中气）之月
                let yushui = fl(&((li.dongzhi(y) + li.sui() * qr(2, 12)) / &f)) + li.off();
                if let Some(i) = days.iter().rposition(|&d| d <= yushui) {
                    if days[i] == raw[i].base && raw[i].frac >= qr(h, 24) {
                        days[i] += 1;
                    }
                }
            }
        }
        if self.rules.pin_san {
            let mut s: Vec<(i64, Q)> = days.iter().zip(&raw).map(|(&d, m)| (d, m.frac.clone())).collect();
            limit_runs(&mut s, 3);
            days = s.into_iter().map(|x| x.0).collect();
        }
        if full {
            if let Some(jb) = self.rules.jiao_bujin {
                for (i, m) in raw.iter().enumerate() {
                    if m.jin && jb.applies(m) {
                        days[i] = m.base;
                        locked[i] = true;
                    }
                }
            }
            if !self.rules.zhengyue_jiao.is_empty() {
                for y in y0..=y1 {
                    let Some(&(_, _, k)) = self.rules.zhengyue_jiao.iter().find(|r| (r.0..=r.1).contains(&y)) else { continue };
                    let yushui = fl(&((li.dongzhi(y) + li.sui() * qr(2, 12)) / &f)) + li.off();
                    let Some(i) = days.iter().rposition(|&d| d <= yushui) else { continue };
                    let m = &raw[i];
                    let shi = m.rishi.as_ref().map_or_else(|| m.jiao.as_ref().is_some_and(|j| j.shixian), |r| r.shi);
                    if shi && m.zhou() {
                        days[i] = m.base + k;
                        locked[i] = true;
                        xiaoxi(&mut days, i);
                    }
                }
            }
            for (name, jdn, adj) in liwai_table().iter().filter(|r| self.names.contains(&r.0.as_str())) {
                let Some(i) = raw.iter().position(|m| (m.base - jdn).abs() <= 1) else { continue };
                match adj {
                    Liwai::Shift(k) => {
                        days[i] = raw[i].base + k;
                        assert_eq!(days[i], *jdn, "{name} 例外 {jdn}：定朔所在日 {}", raw[i].base);
                        locked[i] = true;
                        xiaoxi(&mut days, i);
                    }
                    Liwai::Leap => {
                        assert_eq!(days[i], *jdn, "{name} 置闰 {jdn}");
                        leaps.push(*jdn);
                        locked[i] = true;
                    }
                }
            }
            if let Some((max_big, max_small)) = self.rules.runs {
                for i in 1..days.len() {
                    if locked[i] || !(raw[i].base..=raw[i].base + 1).contains(&days[i]) {
                        continue;
                    }
                    // 本月朔前已连续之大（30 日）、小（29 日）月数
                    let run = |len: i64| (1..i).rev().take_while(|&j| days[j] - days[j - 1] == len).count();
                    let len = days[i] - days[i - 1];
                    if days[i] > raw[i].base && len == 30 && run(30) >= max_big {
                        days[i] -= 1;
                    } else if days[i] == raw[i].base && len == 29 && run(29) >= max_small {
                        days[i] += 1;
                    }
                }
            }
        }
        (days, leaps)
    }
}

impl DingshuoLi for Paiyue {
    fn f(&self) -> i64 {
        self.li.f()
    }
    fn off(&self) -> i64 {
        self.li.off()
    }
    fn sui(&self) -> Q {
        self.li.sui()
    }
    fn dongzhi(&self, y: i32) -> Q {
        self.li.dongzhi(y)
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        self.li.month(y, m)
    }
    fn jinshuo_limit(&self, y: i32, t: &Q) -> Option<Q> {
        self.li.jinshuo_limit(y, t)
    }
    fn jiao(&self, y: i32, m: i64) -> Option<Jiaohui> {
        self.li.jiao(y, m)
    }
    fn sunrise(&self, y: i32, t: &Q) -> Option<Q> {
        self.li.sunrise(y, t)
    }
    fn rishi(&self, y: i32, m: i64) -> Option<Rishi> {
        self.li.rishi(y, m)
    }
    fn ding_for_calendar(&self, y: i32, m: i64) -> Q {
        self.li.ding_for_calendar(y, m)
    }
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        self.li.biaoduan(y, m)
    }
    fn fixed_days(&self) -> Option<&[i64]> {
        Some(&self.days(true).0)
    }
    fn forced_leaps(&self) -> &[i64] {
        &self.days(true).1
    }
    fn zhongqi(&self, y: i32) -> Vec<i64> {
        match &self.zhongqi_li {
            Some((qi, spans)) if spans.iter().any(|&(a, b)| (a..=b).contains(&y)) => qi.zhongqi(y),
            _ => self.li.zhongqi(y),
        }
    }
}

/// [Paiyue] 只按术文进朔之视图（旧做法）
struct Shuwen(&'static Paiyue);

impl DingshuoLi for Shuwen {
    fn f(&self) -> i64 {
        self.0.f()
    }
    fn off(&self) -> i64 {
        self.0.off()
    }
    fn sui(&self) -> Q {
        self.0.sui()
    }
    fn dongzhi(&self, y: i32) -> Q {
        self.0.dongzhi(y)
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        self.0.month(y, m)
    }
    fn jinshuo_limit(&self, y: i32, t: &Q) -> Option<Q> {
        self.0.jinshuo_limit(y, t)
    }
    fn jiao(&self, y: i32, m: i64) -> Option<Jiaohui> {
        self.0.jiao(y, m)
    }
    fn sunrise(&self, y: i32, t: &Q) -> Option<Q> {
        self.0.sunrise(y, t)
    }
    fn rishi(&self, y: i32, m: i64) -> Option<Rishi> {
        self.0.rishi(y, m)
    }
    fn fixed_days(&self) -> Option<&[i64]> {
        Some(&self.0.days(false).0)
    }
}

/// 冬至岁 y 的月：(朔日 JDN, 月序, 闰否)，自含冬至之月（十一月）起至下一含冬至之月前。
fn sui_months(li: &dyn DingshuoLi, y: i32) -> Vec<(i64, u8, bool)> {
    // 每个经朔只由其所在冬至岁算一次（经朔 < 下一冬至岁天正经朔），免得相邻两岁所算定朔微异而重出
    let mut days: Vec<i64> = match li.fixed_days() {
        Some(all) => all.to_vec(),
        None => month_starts(li, y - 1, y + 1).into_iter().map(|(d, _)| d).collect(),
    };
    days.sort_unstable();
    days.dedup();
    let zq = li.zhongqi(y);
    let (dz0, dz1) = (zq[0], zq[12]);
    let contains = |i: usize, d: i64| days[i] <= d && d < days[i + 1];
    let i0 = (0..days.len() - 1).find(|&i| contains(i, dz0)).expect("含冬至之月");
    let i1 = (0..days.len() - 1).find(|&i| contains(i, dz1)).expect("含下一冬至之月");
    assert!(i1 - i0 == 12 || i1 - i0 == 13, "冬至岁 {y}：两冬至间 {} 月", i1 - i0);
    let forced = (i0 + 1..i1).find(|&i| li.forced_leaps().contains(&days[i]));
    let leap = if i1 - i0 == 13 { forced.or_else(|| (i0 + 1..i1).find(|&i| !zq.iter().any(|&z| contains(i, z)))) } else { None };
    let mut out: Vec<(i64, u8, bool)> = Vec::with_capacity(13);
    let mut label = 11u8;
    for i in i0..i1 {
        if Some(i) == leap {
            out.push((days[i], out.last().map_or(11, |e| e.1), true));
            continue;
        }
        out.push((days[i], label, false));
        label = label % 12 + 1;
    }
    out
}

/// 某历下农历年 N 各月 (朔日 JDN, 月序, 闰否)，正月至十二月（含闰）。
pub fn year_months(li: &dyn DingshuoLi, lunar_year: i32) -> Vec<(i64, u8, bool)> {
    let head = sui_months(li, lunar_year);
    let k = head.iter().position(|e| e.1 == 1 && !e.2).expect("正月");
    let mut out = head[k..].to_vec();
    out.extend(sui_months(li, lunar_year + 1).into_iter().take_while(|e| e.1 >= 11));
    out
}

// ---------------------------------------------------------------- 大衍一系（大衍、五纪、正元、宣明、崇玄）

/// 月离表：入转（或入历半周）第 i 日损益率，七、十四、二十一、二十八日等按初数、末数分段。
struct MoonTable {
    f: i64,
    /// 各日损益率；分段之日为 None
    rates: &'static [Option<i64>],
    /// (日序, 初数, 初率, 末数, 末率)；末率 0 者末段不增减
    split: Vec<(usize, Q, i64, Q, i64)>,
}

impl MoonTable {
    fn cum(&self, i: usize) -> i64 {
        (0..i).map(|j| self.split.iter().find(|s| s.0 == j).map_or_else(|| self.rates[j].unwrap(), |s| s.2 + s.4)).sum()
    }
    fn eval(&self, x: &Q) -> Q {
        let f = q(self.f);
        let i = (fl(&(x / &f)) as usize).min(self.rates.len() - 1);
        let r = x - q(i as i64) * &f;
        let ji = q(self.cum(i));
        if let Some((_, n0, r0, n1, r1)) = self.split.iter().find(|s| s.0 == i) {
            if &r <= n0 {
                return ji + q(*r0) * &r / n0;
            }
            return ji + q(*r0) + if *r1 != 0 { q(*r1) * (&r - n0) / n1 } else { Q::zero() };
        }
        ji + q(self.rates[i].unwrap()) * r / f
    }

    /// 大衍「朔望有交者」月离精法（《新唐书·历志四上》步月离术，「此術本出皇極曆」），读法据张培瑜等《中国古代历法》§8.4
    /// 式（8-21）—（8-28）（书页 509—511）：
    /// - 转率（入余 t 日内所损益）：前多者（应益）以本日、后日损益率 Δ₁、Δ₂ 二次内插，I(t) = Δ₁t + (Δ₁−Δ₂)(t − t²)/2；
    ///   前少者（应损）「后率」从李善兰作前一日 Δ₀，I(t) = Δ₁t − (Δ₁−Δ₀)(t − t²)/2（皆日内积分为 Δ₁）；
    /// - 第二次改正：t₀ = |转率|/通法，「以朓減、朒加轉率，為定率」，定率 = I(t ± t₀)（朒加、朓减）。张书只推前多朓、前少朒
    ///   两种，即 I(t − t₀)、I(t + t₀)；前多朒（转日 1—6）、前少朓（22—27）术文同一句「朓減、朒加」，按同式取向。
    /// - 「其後無同率者，亦因前率」：所需之邻日为初末分段日或异号者，改用另一侧邻日。
    /// - 「其損益入餘進退日，分為二日，隨餘初末，如法求之。所得並以損益轉率」（七、十四、二十一、二十八日，损益在日内进退）：
    ///   张书未及此句（书页 509 只录术文）。读作：入余在初数以下者以初段为一日（长 = 初数，损益率 = 初率），已上者以末段为一日
    ///   （长 = 末数，损益率 = 末率），「如法」即在此段内照上式二次内插并作第二次改正（通法换作段长）。邻率照术注后文
    ///   「如初數已下者……因循前率；如初數以上，則反其衰，歸于後率」：初段取前一日、末段取后一日之损益率，按段长折算
    ///   （邻率 × 段长 / 通法，使两率同为段内之量）；邻日异号或缺、末率为 0（二十八日末「益入後」）者，该段仍用简法。
    ///   「所得並以損益轉率」：段内所得加于前段之积（末段 = 本日积 + 初率 + 段内所得）。读法为本项目所定，见
    ///   `doc/calendar/dingshuo-zhangbiao-diffs.md`「二次差精法与麟德步交会」。
    fn eval_jing(&self, x: &Q) -> Q {
        let f = q(self.f);
        let i = (fl(&(x / &f)) as usize).min(self.rates.len() - 1);
        let r = x - q(i as i64) * &f;
        let ji = q(self.cum(i));
        if let Some((_, n0, r0, n1, r1)) = self.split.iter().find(|s| s.0 == i) {
            // 分为二日：初段以前一日为邻，末段以后一日为邻
            let (len, rate, base, u, nb) = if &r <= n0 {
                (n0.clone(), *r0, ji.clone(), &r / n0, i.checked_sub(1).and_then(|j| self.rates.get(j).copied().flatten()))
            } else {
                (n1.clone(), *r1, &ji + q(*r0), (&r - n0) / n1, self.rates.get(i + 1).copied().flatten())
            };
            let nb = nb.filter(|v| rate != 0 && v.signum() == rate.signum());
            let Some(nb) = nb else { return self.eval(x) };
            let nb = q(nb) * &len / &f;
            let c = if &r <= n0 { -(q(rate) - nb) / q(2) } else { (q(rate) - nb) / q(2) };
            return Self::jing_step(q(rate), c, u, &len, base, rate > 0);
        }
        let r1 = self.rates[i].unwrap();
        let same = |j: Option<usize>| j.and_then(|j| self.rates.get(j).copied().flatten()).filter(|r| r.signum() == r1.signum() && *r != 0);
        let (next, prev) = (same(Some(i + 1)), same(i.checked_sub(1)));
        // c：I(u) = Δ₁u + c(u − u²) 之系数（带号）
        let c_next = next.map(|r2| qr(r1 - r2, 2));
        let c_prev = prev.map(|r0| -qr(r1 - r0, 2));
        let qianduo = next.is_some_and(|r2| r2.abs() <= r1.abs()) || prev.is_some_and(|r0| r0.abs() > r1.abs());
        let c = if qianduo { c_next.or(c_prev) } else { c_prev.or(c_next) };
        let Some(c) = c else { return self.eval(x) };
        Self::jing_step(q(r1), c, &r / &f, &f, ji, r1 > 0)
    }

    /// 一「日」（长 len 分）内之精法：转率 I(u) = Δ₁u + c(u − u²)，t₀ = |转率|/len，朒加朓减，定率 = I(u ± t₀)，加于 base
    fn jing_step(d1: Q, c: Q, u: Q, len: &Q, base: Q, yi: bool) -> Q {
        let integ = |v: &Q| &d1 * v + &c * (v - v * v);
        let zhuan = integ(&u);
        let t0 = zhuan.abs() / len;
        // 朒（迟，积为正）加、朓（疾）减
        let nu = (&base + &zhuan).is_positive() || (base.is_zero() && yi);
        let tt = if nu { &u + t0 } else { &u - t0 };
        base + integ(&tt)
    }
}

enum Jinshuo {
    /// 小余不小于 时/24 日 进
    Hours(i64),
    /// 五纪、正元：距次日夜半不过（晨初余数 − 十刻）者进；夜半漏按大衍步轨漏逐日递推（五纪「寫大衍舊術」，正元轨漏「悉如五紀法」）
    Lou,
    /// 宣明：秋分后 3/4；春分后按张培瑜等《中国古代历法》页 570 所算限值，气间线性
    Xuanming,
    /// 崇玄（《新唐书·历志六下》）：「其定朔小餘，如日法四十分之二十九已上，以定朔小餘減日法，餘如晨初餘數已下，進一日」；
    /// 晨初余数按志文晷漏术逐日推（[Dayan::chongxuan_chenchu]）
    Chongxuan,
}

struct Dayan {
    f: i64,
    /// 岁实（策实，分）
    ce: i64,
    /// 朔实（揲法，分）
    she: i64,
    /// 转终（分）
    zhuan: Q,
    ji: i64,
    y_ji: i32,
    /// 定气先后数（先 +），恒气者全 0；24 气
    xianhou: Vec<i64>,
    sun: Vec<i64>,
    /// 宣明分前后半周两表（半周 = 转终/2）
    moon: Vec<MoonTable>,
    /// 进朔限：(起农历年, 限)，按农历年取最后一段起年不晚于者
    jinshuo: Vec<(i32, Jinshuo)>,
    off: i64,
    /// 步交会
    jiao: Option<JiaoConst>,
    /// 求日出所用之漏：None 用大衍步轨漏（[Dayan::night_lou]），崇玄用其晷漏术
    chongxuan_lou: bool,
    /// 宣明日蚀术（[Dayan::xuanming_rishi]）
    xuanming_shi: bool,
    /// 朔望有交之月用日躔、月离二次差精法（[Dayan::sun_def_jing]、[MoonTable::eval_jing]）；否则一律简法
    jingfa: bool,
}

impl Dayan {
    fn qi(&self) -> Q {
        qr(self.ce, 24)
    }
    fn ding_qi(&self, k: i64) -> Q {
        self.qi() * q(k) - q(self.xianhou[k.rem_euclid(24) as usize])
    }
    fn seg(&self, t: &Q) -> i64 {
        let mut k = fl(&(t / self.qi())) - 1;
        while self.ding_qi(k + 1) <= *t {
            k += 1;
        }
        k
    }
    fn sun_def(&self, t: &Q) -> Q {
        let k = self.seg(t);
        let kk = k.rem_euclid(24) as usize;
        let ji: i64 = self.sun[..kk].iter().sum();
        let (a, b) = (self.ding_qi(k), self.ding_qi(k + 1));
        q(ji) + q(self.sun[kk]) * (t - &a) / (b - a)
    }
    fn moon_def(&self, x: &Q) -> Q {
        if self.moon.len() == 1 {
            return self.moon[0].eval(x);
        }
        let zhong = &self.zhuan / q(2);
        if *x < zhong {
            self.moon[0].eval(x)
        } else {
            self.moon[1].eval(&(x - zhong))
        }
    }
    /// 月离精法（[MoonTable::eval_jing]）
    fn moon_def_jing(&self, x: &Q) -> Q {
        if self.moon.len() == 1 {
            return self.moon[0].eval_jing(x);
        }
        let zhong = &self.zhuan / q(2);
        if *x < zhong {
            self.moon[0].eval_jing(x)
        } else {
            self.moon[1].eval_jing(&(x - zhong))
        }
    }

    /// 日躔精法（《新唐书·历志四上》步日躔术「以所入氣幷後氣盈縮分……其求朓朒倣此」），读法据张培瑜等《中国古代历法》§8.3
    /// 式（8-8）—（8-15）（书页 499—501）。Δ₁、Δ₂ 为所入气、后气损益率之绝对值，t₁、t₂ 为两定气日数：
    /// 末率 = (Δ₁+Δ₂)/(t₁+t₂)，气差 = |Δ₁/t₁ − Δ₂/t₂|，初率 = 末率 ± 气差（至后加、分后减），日差 = 2·气差/(t₁+t₂)，
    /// 气初定率 = 初率 ∓ 日差/2（至后减、分后加），入气 n 日（含余）之积 = n·[气初定率 ∓ (n−1)·日差/2]。
    /// 「距四正前一氣，在陰陽變革之際，不可相幷，皆因前末為初率。以氣差至前加之，分前減之，為末率」：二分、二至前一气
    /// （惊蛰、芒种、白露、大雪）以前一对之末率为初率、前一对之气差与日差。
    fn sun_def_jing(&self, t: &Q) -> Q {
        let k = self.seg(t);
        let kk = k.rem_euclid(24) as usize;
        let ji: i64 = self.sun[..kk].iter().sum();
        let f = q(self.f);
        let len = |k: i64| (self.ding_qi(k + 1) - self.ding_qi(k)) / &f;
        let a = |k: i64| q(self.sun[k.rem_euclid(24) as usize].abs());
        // 至后（冬至至惊蛰、夏至至白露）损益率递减
        let dec = kk % 12 < 6;
        let (c0, cha, ri) = if kk % 6 == 5 {
            let (a0, a1, l0, l1) = (a(k - 1), a(k), len(k - 1), len(k));
            let mo = (&a0 + &a1) / (&l0 + &l1);
            let cha = (&a0 / &l0 - &a1 / &l1).abs();
            let ri = q(2) * &cha / (&l0 + &l1);
            (mo, cha, ri)
        } else {
            let (a1, a2, l1, l2) = (a(k), a(k + 1), len(k), len(k + 1));
            let mo = (&a1 + &a2) / (&l1 + &l2);
            let cha = (&a1 / &l1 - &a2 / &l2).abs();
            let ri = q(2) * &cha / (&l1 + &l2);
            (if dec { &mo + &cha } else { &mo - &cha }, cha, ri)
        };
        let _ = cha;
        let n = (t - self.ding_qi(k)) / &f;
        let half = qr(1, 2);
        let s = if dec {
            let a0 = &c0 - &ri * &half;
            &n * (a0 - (&n - q(1)) * &ri * &half)
        } else {
            let a0 = &c0 + &ri * &half;
            &n * (a0 + (&n - q(1)) * &ri * &half)
        };
        q(ji) + q(self.sun[kk].signum()) * s
    }

    /// 冬至岁 y 第 m 月经朔之入交是否入食限（「朔望有交者」，以简法朓朒求入交）
    fn youjiao(&self, y: i32, m: i64) -> bool {
        let Some(jc) = self.jiao.as_ref() else { return false };
        let tong = self.tong(y);
        let jing = tong - tong.rem_euclid(self.she) + m * self.she;
        jc.ru(&q(jing), &self.sun_def(&q(jing - tong)), &self.moon_def(&md(&q(jing), &self.zhuan)), self.f).shixian
    }
    fn tong(&self, y: i32) -> i64 {
        self.ce * (self.ji + (y - self.y_ji) as i64)
    }

    /// 崇玄历晨初余数（通法之分）。志文「晷漏」：计二至加时已来至其日昏后夜半日数及余，冬至后为息、夏至后为消；
    /// 一象（91 日 3131 分）已下为初，已上反减二至限（182 日 62 分小分 22 半，日母一百）为末；令自相乘，进二位，以消息法
    /// （1667.5）除为分，副之；与五百分先相减、后相乘，千八百而一，以加副，为消息数。春分后加 1752、秋分后以减 2748，
    /// 为晷漏母；百三十五乘晷漏母，百约为分，得晨初余数。`t` 为定朔（分，自上元），`y` 为其冬至岁。
    fn chongxuan_chenchu(&self, y: i32, t: &Q) -> Q {
        let f = q(self.f);
        let erzhi = q(182) + qr(62225, 100000);
        let yixiang = q(91) + qr(3131, 13500);
        // 其日昏后夜半（次日夜半）距天正冬至加时之日数
        let night = q(fl(&(t / &f)) + 1) * &f;
        let mut d = (night - q(self.tong(y))) / &f;
        let year = q(self.ce) / &f;
        while d.is_negative() {
            d += &year;
        }
        while d >= year {
            d -= &year;
        }
        let dongzhi_half = d < erzhi;
        if !dongzhi_half {
            d -= &erzhi;
        }
        let chu = d <= yixiang;
        let x = if chu { d.clone() } else { &erzhi - &d };
        let p = &x * &x * q(100) / qr(3335, 2);
        let xiaoxi = (q(500) - &p) * &p / q(1800) + &p;
        // 春分后：冬至后已过一象，或夏至后未过一象
        let chunfen_hou = (dongzhi_half && !chu) || (!dongzhi_half && chu);
        let guilou = if chunfen_hou { q(1752) + xiaoxi } else { q(2748) - xiaoxi };
        guilou * q(135) / q(100)
    }

    /// 时刻 t（属冬至岁 y）所在日的夜半漏（刻）：大衍步轨漏，自其定气初日逐日递推（[dayan_night_lou]）
    fn night_lou(&self, y: i32, t: &Q) -> Q {
        let f = q(self.f);
        let tong = q(self.tong(y));
        let day = fl(&(t / &f));
        let d = md(&(q(day) * &f - &tong), &q(self.ce));
        let k = self.seg(&d);
        let qi_day = fl(&((&tong + self.ding_qi(k) + q(self.ce) * q(fl(&((q(day) * &f - &tong) / q(self.ce))))) / &f));
        dayan_night_lou(k.rem_euclid(24) as usize, (day - qi_day).max(0))
    }

    /// 宣明历日蚀（《新唐书·历志六上》卷三十上）。时差：「以定朔日出入辰刻距午正刻數，約百四十七，為時差。視定朔小餘如半法
    /// 已下，以減半法，為初率；已上，減去半法，餘為末率。以乘時差，如刻法而一，初率以減，末率倍之，以加定朔小餘，為蝕定餘」。
    /// 蚀差三项（单位统法之分，加减去交分）：
    /// - 气差：二至之初 2350，距二至每日损 26，至二分而空；「以日出沒辰刻距午正刻數，約其朔日氣差，以乘食甚距午正刻數。所得以減
    ///   氣差，為定數。春分後，陰曆加之，陽曆減之；秋分後，陰曆減之，陽曆加之」；
    /// - 刻差：二至初日无，每日益 2.10（差分二、小分十），至 94.5 止（立春至立夏、立秋至立冬），复日损至二至；乘食甚距午刻数；
    ///   「冬至後食甚在午正前，夏至後食甚在午正後，陰曆以減，陽曆以加」，反之陰加陽減；
    /// - 加差：小雪 17、大雪 34、冬至 51、小寒 34、大寒 17（「立冬初日後，每氣增差十七……終于大寒，損盡」），食甚在午后者
    ///   每刻累益，阴历减、阳历加。
    /// 「同名相從，異名相銷」为蚀差，加减去交分为定分。阴历不足减者入阳历，不蚀；阳历不足减者入阴历，蚀。余者定分在阳历蚀限
    /// 2640 与阴历蚀限 6060 之和 8700 已下者蚀（「如陽曆蝕限已下，為陽曆蝕……已上者，以陽曆蝕限減之，餘為陰曆蝕，以陰曆定法約之，
    /// 以減十五」，即定分 < 8700 蚀分为正）。此读法与史载日食 36 条合 33（以阴阳历各用本限读只合 24），见 doc。
    /// 日出用大衍步轨漏代宣明晷漏（近似）；距二至、二分之日数按恒气连续计。
    fn xuanming_rishi(&self, y: i32, m: i64) -> Option<Rishi> {
        let j = self.jiao(y, m)?;
        let ding = self.month(y, m).1;
        let fq = q(self.f);
        let frac = md(&ding, &fq) / &fq;
        let sr = self.sunrise(y, &ding)?;
        let half = qr(1, 2);
        let h = (&half - &sr) * q(100);
        let shicha = q(147) / &h;
        let shen = if frac <= half { &frac - (&half - &frac) * &shicha / q(84) } else { &frac + q(2) * (&frac - &half) * &shicha / q(84) };
        let ju_wu = (&shen - &half).abs() * q(100);
        let sui = q(self.ce) / &fq;
        let d = md(&((&ding - q(self.tong(y))) / &fq), &sui);
        let l = &sui / q(2);
        let x = md(&d, &l);
        let dz = if x <= &l / q(2) { x } else { &l - x };
        let yin_s = |b: bool| if j.yin == b { q(1) } else { q(-1) };
        // 气差
        let qi = (q(2350) - q(26) * &dz).max(Q::zero());
        let qi_d = &qi - &qi * &ju_wu / &h;
        let chun = d >= &sui / q(4) && d < &sui * qr(3, 4);
        let s_qi = yin_s(true) * if chun { q(1) } else { q(-1) } * qi_d;
        // 刻差
        let ke = (qr(21, 10) * &dz).min(qr(189, 2));
        let am = shen < half;
        let dzh = d < l;
        let s_ke = if dzh == am { -yin_s(true) } else { yin_s(true) } * ke * &ju_wu;
        // 加差
        let k = fl(&(&d / (&sui / q(24))));
        let jia = match k.rem_euclid(24) {
            22 | 2 => 17,
            23 | 1 => 34,
            0 => 51,
            _ => 0,
        };
        let s_jia = if am { Q::zero() } else { -yin_s(true) * q(jia) * (&shen - &half) * q(100) };
        let ding_fen = &j.qu * &fq + s_qi + s_ke + s_jia;
        let (shi, yin) = if ding_fen.is_negative() { (!j.yin, !j.yin) } else { (ding_fen < q(8700), j.yin) };
        Some(Rishi { shi, yin, shen, dingyong: None })
    }
}

/// 大衍历定气初夜半漏（刻），冬至至夏至，夏至后对称；春分、芒种从校勘记
fn dayan_lou(i: usize) -> Q {
    const A: [(i64, i64); 13] =
        [(27, 230), (27, 135), (26, 380), (25, 475), (24, 470), (23, 360), (22, 240), (21, 120), (20, 10), (19, 5), (18, 100), (17, 345), (17, 250)];
    qr(A[i].0 * 480 + A[i].1, 480)
}

/// 大衍历各定气初消息衰（百分之一分为单位，「息空（六十四）」作 64），冬至起；霜降「消廿四」从校勘记作三十四
const DY_SHUAI: [i64; 24] = [64, 1191, 2242, 3025, 3578, 3950, 3965, 3889, 3356, 2838, 2012, 1012, 52, 1076, 2075, 2890, 3455, 3890, 3966, 3950, 3498, 2972, 2170, 1113];
/// 陟降率（降 +、陟 −）；雨水、清明、处暑、寒露为初日之率，其后按 [DY_XIAN] 逐限损益
const DY_ZHIJIANG: [i64; 24] = [78, 72, 53, 34, 78, 1, -5, -1, -32, -52, -63, -64, 64, 63, 52, 32, 99, 5, -1, -1, -34, -53, -72, -78];
/// 「其距二分前後各一氣之外，陟降不等，皆以三日為限」：(气序, 各限每日损益)；雨水、处暑日损，清明、寒露日益（就绝对值言）
const DY_XIAN: [(usize, [i64; 5]); 4] = [(4, [-12, -8, -3, -2, -1]), (7, [1, 2, 3, 8, 19]), (16, [-19, -8, -3, -2, -1]), (19, [1, 2, 3, 8, 12])];

/// 大衍历步轨漏：定气 kk 初日后第 n 日的夜半漏（刻）。「各置其氣消息衰，依定氣所有日，每以陟降率陟減、降加其分，滿百從衰，
/// 各得每日消息定衰」；「置消息定衰，滿象積為刻，不滿為分。各遞以息減、消加其氣初夜半漏，得每日夜半漏定數」（《新唐书·历志四上》）。
/// 五纪「寫大衍舊術」，正元轨漏「悉如五紀法」，故二历同用。
fn dayan_night_lou(kk: usize, n: i64) -> Q {
    let base = dayan_lou(if kk <= 12 { kk } else { 24 - kk });
    let xi = kk < 12;
    let xian = DY_XIAN.iter().find(|x| x.0 == kk).map(|x| x.1);
    let mut shuai = DY_SHUAI[kk];
    let mut rate = DY_ZHIJIANG[kk].abs();
    let sign = DY_ZHIJIANG[kk].signum();
    let mut acc = 0i64;
    for j in 0..n {
        acc += shuai;
        if let Some(d) = xian {
            if j >= 1 {
                rate = (rate + d[(((j - 1) / 3) as usize).min(4)]).max(0);
            }
        }
        shuai += sign * rate;
    }
    // acc 以百分之一分计，象积 480 分为一刻
    let delta = qr(acc, 100 * 480);
    if xi { base - delta } else { base + delta }
}

impl DingshuoLi for Dayan {
    fn f(&self) -> i64 {
        self.f
    }
    fn off(&self) -> i64 {
        self.off
    }
    fn sui(&self) -> Q {
        q(self.ce)
    }
    fn dongzhi(&self, y: i32) -> Q {
        q(self.tong(y))
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        let tong = self.tong(y);
        let t0 = tong - tong.rem_euclid(self.she);
        let jing = t0 + m * self.she;
        // 「若非朔望有交者」用简法；有交之月用精法
        let ding = if self.jingfa && self.youjiao(y, m) {
            q(jing) + self.sun_def_jing(&q(jing - tong)) + self.moon_def_jing(&md(&q(jing), &self.zhuan))
        } else {
            q(jing) + self.sun_def(&q(jing - tong)) + self.moon_def(&md(&q(jing), &self.zhuan))
        };
        (q(jing), ding)
    }
    fn jiao(&self, y: i32, m: i64) -> Option<Jiaohui> {
        let jc = self.jiao.as_ref()?;
        let tong = self.tong(y);
        let jing = tong - tong.rem_euclid(self.she) + m * self.she;
        Some(jc.ru(&q(jing), &self.sun_def(&q(jing - tong)), &self.moon_def(&md(&q(jing), &self.zhuan)), self.f))
    }
    /// 日出 = 晨初（夜半漏）+ 昏明二刻半。大衍、五纪、正元用大衍步轨漏；宣明志「晷漏、交會，則稍增損之」而漏表未复原，
    /// 亦以大衍步轨漏代之（近似）；崇玄用其晷漏术之晨初余数
    fn sunrise(&self, y: i32, t: &Q) -> Option<Q> {
        if self.chongxuan_lou {
            return Some(self.chongxuan_chenchu(y, t) / q(self.f) + qr(25, 1000));
        }
        Some((self.night_lou(y, t) + qr(5, 2)) / q(100))
    }
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        let tong = self.tong(y);
        let jing = tong - tong.rem_euclid(self.she) + m * self.she;
        let f = q(self.f);
        let x = md(&q(jing), &self.zhuan);
        let (tab, x, add) = if self.moon.len() == 1 {
            (&self.moon[0], x, 0)
        } else {
            let zhong = &self.zhuan / q(2);
            if x < zhong { (&self.moon[0], x, 0) } else { (&self.moon[1], x - zhong, 14) }
        };
        let i = (fl(&(&x / &f)) as usize).min(tab.rates.len() - 1);
        let yu = (&x - q(i as i64) * &f) / &f;
        let chu = tab.split.iter().find(|s| s.0 == i).map(|s| (&s.1 / &f).to_f64().unwrap());
        let t = q(jing - tong);
        let k = self.seg(&t);
        let (a, b) = (self.ding_qi(k), self.ding_qi(k + 1));
        Some(Biaoduan {
            yue: Some((i as u32 + 1 + add, yu.to_f64().unwrap(), chu)),
            ri: (k.rem_euclid(24) as usize, ((&t - &a) / &f).to_f64().unwrap(), ((b - a) / &f).to_f64().unwrap()),
        })
    }
    fn rishi(&self, y: i32, m: i64) -> Option<Rishi> {
        if self.xuanming_shi {
            self.xuanming_rishi(y, m)
        } else {
            None
        }
    }
    fn jinshuo_limit(&self, y: i32, t: &Q) -> Option<Q> {
        let f = q(self.f);
        // 定朔所属农历年：过大寒（冬至后第一中气）者属冬至岁 y，否则属 y − 1（十一、十二月）
        let ly = if t - q(self.tong(y)) >= q(self.ce) / q(12) { y } else { y - 1 };
        let js = self.jinshuo.iter().rev().find(|(from, _)| *from <= ly).unwrap_or(&self.jinshuo[0]);
        Some(match js.1 {
            Jinshuo::Hours(h) => &f * qr(h, 24),
            Jinshuo::Lou => {
                // 定朔所在日的夜半漏：自其定气初日逐日递推（[dayan_night_lou]）
                let lou = self.night_lou(y, t);
                // 24 − (漏 − 十刻) × 0.24 时
                &f * (q(24) - (lou - q(10)) * qr(24, 100)) / q(24)
            }
            Jinshuo::Chongxuan => {
                let a = &f * qr(29, 40);
                let b = &f - self.chongxuan_chenchu(y, t);
                if a > b { a } else { b }
            }
            Jinshuo::Xuanming => {
                // 清明、白露一格书作 74924，疑「7」误作「9」：与春分只差 0.00076 而谷雨骤降 0.00445，不合二分前后漏变最快；
                // 按大衍步轨漏推清明初应约 0.7475，书中余格比大衍推值系统低约 0.0002，故取 74724（zhongguo-gudai-lifa-checks.md 第三节续查）
                const LIM: [i64; 13] = [75000, 74724, 74479, 74276, 74124, 74031, 74005, 74031, 74124, 74276, 74479, 74724, 75000];
                let d = md(&(t - q(self.tong(y))), &q(self.ce));
                let k = fl(&(&d / self.qi()));
                if !(6..18).contains(&k) {
                    &f * qr(3, 4)
                } else {
                    let x = (&d - self.qi() * q(k)) / self.qi();
                    let (a, b) = (qr(LIM[(k - 6) as usize], 100000), qr(LIM[(k - 5) as usize], 100000));
                    &f * (&a + (b - &a) * x)
                }
            }
        })
    }
}

fn split4(v: [(usize, i64, i64, i64, i64); 4]) -> Vec<(usize, Q, i64, Q, i64)> {
    v.iter().map(|&(i, n0, r0, n1, r1)| (i, q(n0), r0, q(n1), r1)).collect()
}

fn sym(half: &[i64]) -> Vec<i64> {
    half.iter().copied().chain(half.iter().map(|x| -x)).collect()
}

#[allow(clippy::too_many_arguments)]
fn make_dayan(f: i64, ce: i64, she: i64, zhuan: Q, ji: i64, y_ji: i32, xianhou: Vec<i64>, sun: Vec<i64>, moon: Vec<MoonTable>, jinshuo: Vec<(i32, Jinshuo)>, approx: (i32, i32, i32)) -> Dayan {
    let mut d = Dayan { f, ce, she, zhuan, ji, y_ji, xianhou, sun, moon, jinshuo, off: 0, jiao: None, chongxuan_lou: false, xuanming_shi: false, jingfa: false };
    let dz = d.tong(y_ji).div_euclid(f);
    d.off = align_offset(dz, gregorian::civil_to_jdn(approx.0, approx.1, approx.2));
    d
}

const DAYAN_MOON: [Option<i64>; 28] = [
    Some(297), Some(259), Some(220), Some(180), Some(139), Some(97), None, Some(-64), Some(-106), Some(-148), Some(-189), Some(-229), Some(-267), None,
    Some(-289), Some(-250), Some(-211), Some(-171), Some(-130), Some(-87), None, Some(73), Some(116), Some(157), Some(198), Some(237), Some(276), None,
];
const WUJI_MOON: [Option<i64>; 28] = [
    Some(-135), Some(-117), Some(-99), Some(-78), Some(-56), Some(-33), None, Some(14), Some(38), Some(62), Some(85), Some(103), Some(118), None,
    Some(128), Some(115), Some(95), Some(74), Some(52), Some(28), None, Some(-20), Some(-42), Some(-65), Some(-89), Some(-109), Some(-125), None,
];
const ZHENGYUAN_MOON: [Option<i64>; 28] = [
    Some(-110), Some(-96), Some(-81), Some(-64), Some(-46), Some(-27), None, Some(12), Some(31), Some(51), Some(68), Some(85), Some(96), None,
    Some(107), Some(94), Some(78), Some(61), Some(42), Some(23), None, Some(-16), Some(-35), Some(-53), Some(-71), Some(-88), Some(-102), None,
];
const XUANMING_A: [Option<i64>; 14] =
    [Some(830), Some(726), Some(606), Some(471), Some(337), Some(202), None, Some(-82), Some(-224), Some(-366), Some(-509), Some(-643), Some(-748), None];
const XUANMING_B: [Option<i64>; 14] =
    [Some(-830), Some(-726), Some(-598), Some(-464), Some(-329), Some(-195), None, Some(82), Some(225), Some(366), Some(501), Some(628), Some(740), None];
const CHONGXUAN_MOON: [Option<i64>; 28] = [
    Some(1319), Some(1150), Some(978), Some(799), Some(617), Some(431), None, Some(-285), Some(-471), Some(-650), Some(-840), Some(-1017), Some(-1185), None,
    Some(-1284), Some(-1110), Some(-941), Some(-757), Some(-578), Some(-386), None, Some(324), Some(516), Some(697), Some(879), Some(1053), Some(1223), None,
];

fn dayan() -> Dayan {
    let mut d = make_dayan(
        3040, 1110343, 89773, qr(6701279, 80), 96961740, 724,
        vec![0, 2353, 4198, 5588, 6564, 7152, 7366, 7152, 6564, 5588, 4198, 2353, 0, -2353, -4198, -5588, -6564, -7152, -7366, -7152, -6564, -5588, -4198, -2353],
        vec![176, 138, 104, 73, 44, 16, -16, -44, -73, -104, -138, -176, -176, -138, -104, -73, -44, -16, 16, 44, 73, 104, 138, 176],
        vec![MoonTable { f: 3040, rates: &DAYAN_MOON, split: split4([(6, 2701, 48, 339, -6), (13, 2363, -231, 677, -66), (20, 2024, -36, 1016, 18), (27, 1686, 165, 1354, 0)]) }],
        // 大衍历志无进朔明文；《新唐书·历志五》述五纪历「依大衍戌初進初朔」。此为晨初限之构造（[DayanJinshuo::Chenchu]）：
        // 729—757 随季节晨初限（同五纪「晨初餘數減十刻」，大衍步轨漏），758 起至德历戌初 19 时；排月时按
        // [PaiyueOptions::dayan_jinshuo] 改写，缺省全程戌初（[DayanJinshuo::Xuchu]，2026-11-28 据 737、738、742 年当时册文改回）。
        vec![(0, Jinshuo::Lou), (758, Jinshuo::Hours(19))],
        (723, 12, 21),
    );
    // 《新唐书·历志四下》步交會術：終數 827251322（秒法一萬）；望差一日四百八十三、秒九千三百三十九；交率 343，交數 4369。
    // 「以交數去朔積分」之「交數」从校勘记作「終數」（點校本卷二十八下校勘记，舊志同）
    d.jiao = Some(JiaoConst::new(qr(827251322, 10000), qr(35239339, 10000), 343, 4369));
    d
}

fn wuji() -> Dayan {
    let mut d = make_dayan(
        1340, 489428, 39571, qr(1366156, 37), 269978, 762,
        sym(&[0, 1037, 1850, 2463, 2893, 3152, 3246, 3152, 2893, 2463, 1850, 1037]),
        sym(&[78, 61, 46, 32, 19, 7, -7, -19, -32, -46, -61, -78]),
        vec![MoonTable { f: 1340, rates: &WUJI_MOON, split: split4([(6, 1191, -8, 149, 1), (13, 1042, 105, 298, 30), (20, 892, 6, 448, -3), (27, 743, -75, 597, 0)]) }],
        // 五纪晨初限自 765 年起；763、764 年仍行至德历戌初 19 时（763 年六月、764 年二月本纪合 19 时，不合晨初限）。
        // 旧做法五纪全程晨初限。
        vec![(0, Jinshuo::Hours(19)), (765, Jinshuo::Lou)],
        (761, 12, 19),
    );
    // 《新唐书·历志五》五紀：交終三億六千四百六十四萬三千七百六十七（秒法一萬）；望差一日二百一十三、秒三千一百一十六半；交率 61，交數 777
    d.jiao = Some(JiaoConst::new(qr(364643767, 10000), qr(155331165, 100000), 61, 777));
    d
}

fn zhengyuan() -> Dayan {
    let mut d = make_dayan(
        1095, 399943, 32336, qr(301720132, 10000), 402900, 784,
        sym(&[0, 848, 1512, 2013, 2364, 2576, 2653, 2576, 2364, 2013, 1512, 848]),
        sym(&[63, 50, 37, 26, 16, 6, -6, -16, -26, -37, -50, -63]),
        vec![MoonTable { f: 1095, rates: &ZHENGYUAN_MOON, split: split4([(6, 973, -7, 122, 1), (13, 851, 87, 244, 25), (20, 729, 5, 366, -2), (27, 607, -68, 488, 0)]) }],
        vec![(0, Jinshuo::Lou)],
        (783, 12, 19),
    );
    // 《新唐书·历志五》正元：交終分二億九千七百九十七萬三千八百一十五（秒法一萬）；望差一日百七十四、秒三千九十二半；交率 61，交數 777
    d.jiao = Some(JiaoConst::new(qr(297973815, 10000), qr(126930925, 100000), 61, 777));
    d
}

fn xuanming() -> Dayan {
    let half = |rates: &'static [Option<i64>], s6: (i64, i64), s13: i64| MoonTable {
        f: 8400,
        rates,
        split: vec![(6, q(7465), s6.0, q(935), s6.1), (13, q(6529), s13, q(1), 0)],
    };
    let mut d = make_dayan(
        8400, 3068055, 248057, qr(23145819, 100), 7070138, 822,
        [0, 60, 110, 150, 180, 198, 204, 198, 180, 150, 110, 60, 0, -60, -110, -150, -180, -198, -204, -198, -180, -150, -110, -60].iter().map(|x| 100 * x).collect(),
        vec![449, 374, 299, 224, 135, 45, -45, -135, -224, -299, -374, -449, -449, -374, -299, -224, -135, -45, 45, 135, 224, 299, 374, 449],
        vec![half(&XUANMING_A, (53, -7), -646), half(&XUANMING_B, (-53, 7), 646)],
        vec![(0, Jinshuo::Xuanming)],
        (821, 12, 19),
    );
    // 《新唐书·历志六上》宣明：終率二十二萬八千五百八十二、秒六千五百一十二；後準（望差）一日千三百三十七、秒千七百四十四；
    // 交率 202，交數 2573
    d.jiao = Some(JiaoConst::new(qr(2285826512, 10000), qr(97371744, 10000), 202, 2573));
    d.xuanming_shi = true;
    d
}

fn chongxuan() -> Dayan {
    let mut d = make_dayan(
        13500, 4930801, 398663, qr(37198697, 100), 53947308, 892,
        vec![0; 24],
        vec![782, 613, 462, 328, 200, 67, -67, -200, -328, -462, -613, -782, -782, -613, -462, -328, -200, -67, 67, 200, 328, 462, 613, 782],
        vec![MoonTable {
            f: 13500,
            rates: &CHONGXUAN_MOON,
            split: vec![
                (6, qr(47987, 4), 213, qr(6013, 4), -27),
                (13, qr(20987, 2), -1032, qr(6013, 2), -293),
                (20, qr(35961, 4), -160, qr(18039, 4), 80),
                (27, q(7487), 737, q(6013), 0),
            ],
        }],
        // 志文进朔：小余 ≥ 29/40 日法，且距次日夜半不过晨初余数
        vec![(0, Jinshuo::Chongxuan)],
        (891, 12, 17),
    );
    // 《新唐书·历志六下》崇玄：交終分三十六萬七千三百六十四、秒九千六百七十三；望差一日二千一百四十九、秒百六十三半；
    // 交率 262，交數 3350
    d.jiao = Some(JiaoConst::new(qr(3673649673, 10000), qr(1564901635, 100000), 262, 3350));
    d.chongxuan_lou = true;
    d
}

// ---------------------------------------------------------------- 麟德

struct Linde {
    off: i64,
    /// 步交会读法（只供交会检验，不入排月）
    jiaohui: LindeJiaohui,
    /// 常月进朔（定小余过日法四分之三者进）之起始冬至岁；None 为常月不进（志文无进朔）
    changyue_jin: Option<i32>,
    /// 常月停进朔之民用日（JDN）区间 [起, 止)：开元五至七年当时文字皆不进（[PaiyueOptions::linde_tingjin]）
    tingjin: Option<(i64, i64)>,
    /// 本术（定小余近夜半者覆算）十四日、二十八日「分為二日」（[Linde::chisu_fen]）；false 则此二日仍用速要
    fen_erri: bool,
}

/// 麟德步交会读法（[Linde::rishi_ld]）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum LindeJiaohui {
    /// 新志（缺省）：变交分、差準皆用，食定小余照新志象限序与加减
    #[default]
    Xinzhi,
    /// 旧志：变交分、差準同，食定小余照旧志「艮、坤、巽、乾為次」「初則減法」「艮坤加副，巽乾減副」
    Jiuzhi,
    /// 新志而不用变交分与差準（2026-11-26 前做法），只供对照
    Jian,
    /// 新志，变交分出日道表者一律按外道诸准判蚀（「依術消息」之另一读），只供对照
    BianWaidao,
}

/// 麟德历气序（立春后启蛰在雨水前）中与后气先后率同数之八气：大寒、雨水、谷雨、芒种、大暑、白露、霜降、大雪
const LD_EQUAL_NEXT: [usize; 8] = [2, 5, 8, 11, 14, 17, 20, 23];

const LD_F: i64 = 1340;
const LD_QI: i64 = 489428;
const LD_SHUO: i64 = 39571;
const LD_JI_664: i64 = 269880;
const LD_BIAN_ZHOU: i64 = 443077;
const LD_BIAN_QI: i64 = 12;
const LD_XIANHOU: [i64; 24] = [54, 46, 38, 38, 46, 54, -54, -46, -38, -38, -46, -54, 54, 46, 38, 38, 46, 54, -54, -46, -38, -38, -46, -54];
const LD_YINGNU: [i64; 24] = [0, 54, 100, 138, 176, 222, 276, 222, 176, 138, 100, 54, 0, -54, -100, -138, -176, -222, -276, -222, -176, -138, -100, -54];
const LD_XIAOXI: [i64; 24] =
    [0, 722, 1340, 1854, 2368, 2986, 3708, 2986, 2368, 1854, 1340, 722, 0, -722, -1340, -1854, -2368, -2986, -3708, -2986, -2368, -1854, -1340, -722];
const LD_ZENG: [i64; 28] =
    [134, 117, 99, 78, 56, 33, 9, -14, -38, -62, -85, -104, -121, -102, 128, 115, 95, 74, 52, 28, 4, -20, -44, -68, -89, -108, -125, -71];
/// 第 i 日增减率折成迟速积（速 +）之逐日变化：志文「增減」就迟速积之绝对值言（「以定率增減遲速積」），速段（一至十四日）增为 +，
/// 迟段（十五至二十八日）增为迟积加大即 −。LD_ZENG 照志文记增 +、减 −，故迟段反号（2026-11-26 前迟段误作同号，见
/// `doc/calendar/dingshuo-zhangbiao-diffs.md`「麟德迟段增减号」）。
fn ld_zeng(i: usize) -> i64 {
    if i >= 14 { -LD_ZENG[i] } else { LD_ZENG[i] }
}
const LD_CHISU: [i64; 28] = [
    0, 134, 251, 350, 428, 484, 517, 526, 512, 474, 412, 327, 223, 102, -29, -157, -272, -367, -441, -493, -521, -525, -505, -461, -393, -304, -196, -71,
];

/// 盈朒定积取整：「凡分餘不成全而更不復須者，過半從一，無半棄之」（《旧唐书·历志二》，后句从严敦杰校；按绝对值）
fn ld_round(x: Q) -> Q {
    let half = qr(1, 2);
    if x < q(0) { -q(fl(&(-x + half))) } else { q(fl(&(x + half))) }
}

impl Linde {
    /// 入变 x（日）→ 迟速积（分，速 +）：日内线性（「速要」）；十四日初减 102 后末增 29 入迟；二十八日只 743 分
    fn chisu(x: &Q) -> Q {
        let i = fl(x);
        let frac = x - q(i);
        if i >= 27 {
            return q(LD_CHISU[27]) + q(ld_zeng(27)) * frac * q(LD_F) / q(743);
        }
        if i == 13 {
            let chu = qr(1042, LD_F);
            if frac <= chu {
                return q(LD_CHISU[13]) - q(102) * frac / chu;
            }
            return -(q(29) * (frac - &chu) / (q(1) - chu));
        }
        q(LD_CHISU[i as usize]) + q(ld_zeng(i as usize)) * frac
    }
    /// 速要：冬至岁 y 第 m 月之 (常朔, 入气盈朒定积, 入变迟速定数（速 +）)
    fn suyao(&self, y: i32, m: i64) -> (Q, Q, Q) {
        let qizong = Self::qizong(y);
        let t0 = qizong - qizong.rem_euclid(LD_SHUO);
        let zongshi = t0 - t0.rem_euclid(LD_F);
        let bian0 = qr((zongshi * LD_BIAN_QI).rem_euclid(LD_BIAN_ZHOU), LD_BIAN_QI) / q(LD_F);
        let qi_len = qr(LD_QI, 24);
        let dingqi = |k: i64| q(qizong) + &qi_len * q(k) - q(LD_XIAOXI[k.rem_euclid(24) as usize]);
        let ch = t0 + m * LD_SHUO;
        let chq = q(ch);
        let mut k = fl(&((&chq - q(qizong)) / &qi_len)) - 2;
        while dingqi(k + 1) <= chq {
            k += 1;
        }
        let kk = k.rem_euclid(24) as usize;
        // 速要：「以所入定氣日算乘先後率，十五而一……入氣日十五算者，十六而一」（《旧唐书·历志二》小注；
        // 日算含分，从刘金沂、赵澄秋《〈麟德历〉定朔计算法》）
        let t = (&chq - dingqi(k)) / q(LD_F);
        let div = if t >= q(15) { q(16) } else { q(15) };
        let yn = ld_round(q(LD_YINGNU[kk]) + q(LD_XIANHOU[kk]) * t / div);
        let bian = md(&(bian0 + qr(t0.rem_euclid(LD_F), LD_F) + qr(m * LD_SHUO, LD_F) + &yn / q(LD_F)), &(qr(LD_BIAN_ZHOU, LD_BIAN_QI) / q(LD_F)));
        let chisu = Self::chisu(&bian);
        (chq, yn, chisu)
    }
    /// 速要所入：(定气序 k（可为负，取模 24 为气序）, 入定气日（含余）, 本气日数, 入变（日，含余）)，同 [Linde::suyao]
    fn suyao_ru(&self, y: i32, m: i64) -> (i64, Q, Q, Q) {
        let qizong = Self::qizong(y);
        let t0 = qizong - qizong.rem_euclid(LD_SHUO);
        let zongshi = t0 - t0.rem_euclid(LD_F);
        let bian0 = qr((zongshi * LD_BIAN_QI).rem_euclid(LD_BIAN_ZHOU), LD_BIAN_QI) / q(LD_F);
        let qi_len = qr(LD_QI, 24);
        let dingqi = |k: i64| q(qizong) + &qi_len * q(k) - q(LD_XIAOXI[k.rem_euclid(24) as usize]);
        let chq = q(t0 + m * LD_SHUO);
        let mut k = fl(&((&chq - q(qizong)) / &qi_len)) - 2;
        while dingqi(k + 1) <= chq {
            k += 1;
        }
        let t = (&chq - dingqi(k)) / q(LD_F);
        let len = (dingqi(k + 1) - dingqi(k)) / q(LD_F);
        let yn = self.suyao(y, m).1;
        let bian = md(&(bian0 + qr(t0.rem_euclid(LD_F), LD_F) + qr(m * LD_SHUO, LD_F) + &yn / q(LD_F)), &(qr(LD_BIAN_ZHOU, LD_BIAN_QI) / q(LD_F)));
        (k, t, len, bian)
    }

    /// 麟德日蚀：入交、蚀否与食定小余（食甚）。读法与两志异文见 `doc/calendar/calendar-compendium.md` 麟德节「步交会」。
    /// - 变交分（新志注「其朔，月在日道裏者，以所入限數減遲速定數，餘以速減、遲加其定交分。而出日道表者，為變交分。不出表者，
    ///   依定交分。其變交分三時半內者，依術消息，以定蝕不」，旧志同而末作「檢其前後月望入交分數多少，依月虧初復末定蝕術，注消息」）：
    ///   内道之朔以（迟速定数 − 限数）再速减迟加，即定交分改正迟速全数；越出交点入外道者为变交分。出表在三时半（去交 392 分）外者
    ///   不蚀；三时半内「依術消息」是斟酌之文（旧志要查前后望之入交与亏初复末），缺省仍依定交分判（内道入蚀限即蚀），
    ///   [LindeJiaohui::BianWaidao] 按外道诸准判（史载 666—728 年三次实食 674、692、721 年因而判为不蚀，故不取）；不出者依定交分。
    /// - 蚀否：「朔在內道則日蝕」——内道（入交定在交中已上）且入蝕限者蚀（「去交在變準已上、加時在準內，或不蝕」之「或」
    ///   不作判定）；外道「朔不應蝕」，唯四例蚀：夏至初日初準 248，距夏至每日损 2（94 日止，末準 60）为变準，去交在变準以内
    ///   且加时在午正前后七刻内者蚀；加时在时準（七刻并刻準，刻準 = (变準 − 60)/18）内、去交在末準以内者蚀；「又置末準，
    ///   每一刻加十八，為差準」——加时在七刻外、时準内者，差準 = 60 + 18 ×（时準 − 距午刻数），去交在差準以内者蚀（自时準边
    ///   之末準至七刻边之变準线性相接；新志「刻準已上」从旧志及钱校作「已下」，点校本校勘记以钱校为是）；秋分至春分去交在
    ///   末準以内、加时在巳午未者蚀。
    /// - 食定小余：「百一十二約前後分，為去交時。置定朔小餘，副之。辰率約之，以艮、巽、坤、乾為次……其餘，半法已下為初；
    ///   已上者，去之，為末。初則因餘，末則減法，各為差率」——差率 = 象限内余（初）或半法减末余（末），象限 335 分（辰率）、
    ///   半法 167.5；内道「益去交時十而三除之。以乘差率，十四而一，為差」，二分前后一气内即为定差，近冬至以距寒露/雨水、
    ///   近夏至以距清明/白露之气数倍之、又加去交时三分之一增之，「近冬至艮巽以加、坤乾以減，近夏至艮巽以減、坤乾以加」，
    ///   「艮、巽加副，坤、乾減副」；外道「三除去交時數，以乘差率，十四而一，為差。艮、坤以減副，巽、乾以加副」。
    /// - 《旧唐书·历志二》（[LindeJiaohui::Jiuzhi]）此段作「以艮、坤、巽、乾為次，命退算外。不滿法者，半法減之。無可減者，為初；
    ///   所減之餘，為末。初則減法」「艮坤加副，巽乾減副」：象限序第二为坤、第三为巽，差率为半法减初余（初）或末余（末），
    ///   内道加减副以艮坤/巽乾分。缺省从新志（[LindeJiaohui::Xinzhi]）。
    fn rishi_ld(&self, y: i32, m: i64) -> Option<Rishi> {
        let j0 = self.jiao(y, m)?;
        let f = q(LD_F);
        let jiu = self.jiaohui == LindeJiaohui::Jiuzhi;
        let full = self.jiaohui != LindeJiaohui::Jian;
        // 变交分：内道之朔再以（迟速定数 − 限数）速减迟加，出日道表（入外道）者为变交分。出表在三时半（392 分）外者不蚀；
        // 三时半内「依術消息」：缺省仍依定交分（内道）判，[LindeJiaohui::BianWaidao] 则按外道诸准判
        let mut chubiao_wai = false;
        let j = if full && j0.yin {
            let (_, _, chisu) = self.suyao(y, m);
            let jc = ld_jiao();
            let bian = &j0.ru * &f - chisu * qr(777 - 60, 777);
            let jb = Jiaohui::new(md(&bian, &jc.zhong), &jc.zhong, &jc.xian, LD_F);
            if jb.yin {
                j0
            } else if self.jiaohui == LindeJiaohui::BianWaidao {
                jb
            } else {
                chubiao_wai = &jb.qu * &f > q(392);
                j0
            }
        } else {
            j0
        };
        let ding = self.ding_for_calendar(y, m);
        let xiao = md(&ding, &f);
        let qu_fen = &j.qu * &f;
        let qu_shi = &qu_fen / q(112);
        // 象限与差率；卦序 艮 0、巽 1、坤 2、乾 3（新志象限序同此，旧志象限序 艮、坤、巽、乾）
        let xian = q(335);
        let half = qr(335, 2);
        let quad = fl(&(&xiao / &xian)).clamp(0, 3);
        let gua = if jiu { [0, 2, 1, 3][quad as usize] } else { quad };
        let (gen_xun, gen_kun) = (gua <= 1, gua == 0 || gua == 2);
        let rem = &xiao - q(quad) * &xian;
        let chalv = if jiu {
            (&rem - &half).abs()
        } else if rem < half {
            rem.clone()
        } else {
            &xian - &rem
        };
        let fu = if j.yin {
            let mut cha = (&qu_shi + q(10)) / q(3) * &chalv / q(14);
            // 距冬至（日）与节气序（冬至 0）
            let sui = q(LD_QI) / &f;
            let d = md(&((&ding - q(Self::qizong(y))) / &f), &sui);
            let qi = fl(&(&d / (&sui / q(24))));
            // 二分前后一气内（惊蛰—清明、白露—寒露：气序 5—7、17—19）即以差为定
            if !matches!(qi, 5..=7 | 17..=19) {
                let near_dongzhi = !(6..18).contains(&qi);
                // 距寒露（19）/雨水（麟德气序启蛰在前，雨水为 5）、清明（7）/白露（17）之气数
                let n = if near_dongzhi {
                    if qi >= 18 { qi - 19 } else { 5 - qi }
                } else if qi < 12 {
                    qi - 7
                } else {
                    17 - qi
                };
                let zeng = q(2 * n.max(0)) + &qu_shi / q(3);
                // 近冬至：艮巽加、坤乾减；近夏至反之
                let jia = gen_xun == near_dongzhi;
                cha = if jia { cha + zeng } else { cha - zeng };
            }
            // 新志艮、巽加副，坤、乾减副；旧志艮、坤加副，巽、乾减副
            if (jiu && gen_kun) || (!jiu && gen_xun) { &xiao + cha } else { &xiao - cha }
        } else {
            let cha = &qu_shi / q(3) * &chalv / q(14);
            // 艮、坤减副，巽、乾加副
            if gen_kun { &xiao - cha } else { &xiao + cha }
        };
        let shen = &fu / &f;
        // 蚀否
        let ke = (&shen - qr(1, 2)).abs() * q(100);
        let shi = if chubiao_wai {
            false
        } else if j.yin {
            j.shixian
        } else {
            let sui = q(LD_QI) / &f;
            let d = md(&((&ding - q(Self::qizong(y))) / &f), &sui);
            let xiazhi = (&d - &sui / q(2)).abs().min(q(94));
            let bian = q(248) - q(2) * xiazhi;
            let kezhun = (&bian - q(60)) / q(18);
            let shizhun = q(7) + &kezhun;
            let qiu_chun = d < &sui / q(4) || d >= &sui * qr(3, 4);
            (qu_fen <= bian && ke <= q(7))
                || (qu_fen <= q(60) && ke <= shizhun)
                || (full && ke > q(7) && ke <= shizhun && qu_fen <= q(60) + q(18) * (&shizhun - &ke))
                || (qiu_chun && qu_fen <= q(60) && shen >= qr(9, 24) && shen < qr(15, 24))
        };
        Some(Rishi { shi, yin: j.yin, shen, dingyong: None })
    }

    fn qizong(y: i32) -> i64 {
        LD_QI * (LD_JI_664 + (y - 664) as i64)
    }

    /// 本术日躔（定气内等间距二次内插）：入定气 kk 第 t 日 → 盈朒积增量（先 +、后 −）。
    /// 读法据张培瑜等《中国古代历法》§7.6（书页 475—477）：半年内气长 L 取平均（秋分后 11×16/12、春分后 11×17/12 日），
    /// 前多 S = t(Δ1+Δ2)/2L + t(Δ1−Δ2)/L − t²(Δ1−Δ2)/2L²，前少 S = t(Δ1+Δ2)/2L − t(Δ1−Δ2)/L + t²(Δ1−Δ2)/2L²；
    /// 与后气同数之八气以前气、本气定初率与别差。Δ 为先后率之绝对值。
    fn sun_benshu(kk: usize, t: &Q) -> Q {
        let l = if kk >= 18 || kk < 6 { qr(44, 3) } else { qr(187, 12) };
        let a1 = q(LD_XIANHOU[kk].abs());
        let a2 = q(LD_XIANHOU[(kk + 1) % 24].abs());
        let sg = q(LD_XIANHOU[kk].signum());
        let two = q(2);
        let s = if LD_EQUAL_NEXT.contains(&kk) {
            let ap = q(LD_XIANHOU[(kk + 23) % 24].abs());
            let chu = (&ap + &a1) / (&two * &l);
            let bie = (&a1 - &ap).abs() / (&l * &l);
            let quad = t * t / &two * bie;
            if ap < a1 { t * chu + quad } else { t * chu - quad }
        } else {
            let lin = t * (&a1 + &a2) / (&two * &l);
            let d = &a1 - &a2;
            if a1 > a2 {
                lin + t * &d / &l - t * t * &d / (&two * &l * &l)
            } else {
                lin - t * &d / &l + t * t * &d / (&two * &l * &l)
            }
        };
        sg * s
    }

    /// 本术月离：入变 x（日）→ 迟速积（速 +）。读法据同书书页 479—481：经辰变率 v = tΔ1 + ½t(t−1)(Δ2−Δ1)，
    /// t0 = v/总法，T = t ∓ t0（速减、迟加），定率 = TΔ1 + ½T(T−1)(Δ2−Δ1)。Δ1、Δ2 为本日、次日增减率。
    /// 十四日、二十八日「其曆率損益入餘進退日者，分為二日，隨餘初末，如法求之，所得并以加減變率為定」（《旧唐书·历志二》）：
    /// `fen` 为真时按 [Linde::chisu_fen] 分段求之，否则仍用速要（日内线性）。
    fn chisu_benshu(x: &Q, fen: bool) -> Q {
        let i = fl(x);
        if i >= 27 || i == 13 {
            return if fen { Self::chisu_fen(x) } else { Self::chisu(x) };
        }
        let (a1, a2) = (q(LD_ZENG[i as usize]), q(LD_ZENG[i as usize + 1]));
        let c = (&a1 - a2) / q(2);
        Self::benshu_step(q(LD_CHISU[i as usize]), a1, c, x - q(i), &q(LD_F), i < 14)
    }

    /// 本术一「日」（长 len 分）内：变率 v(u) = Δ1u + c(u − u²)（c = (Δ1−Δ2)/2 即 ½u(u−1)(Δ2−Δ1)），t0 = v/len，
    /// U = u ∓ t0（速减、迟加），定率 v(U)；增减就迟速积之绝对值言，迟段增为迟积加大（[ld_zeng]），加于 base（速 +）
    fn benshu_step(base: Q, d1: Q, c: Q, u: Q, len: &Q, su: bool) -> Q {
        let rate = |v: &Q| v * &d1 + &c * (v - v * v);
        let t0 = rate(&u) / len;
        let uu = if su { &u - t0 } else { &u + t0 };
        let v = rate(&uu);
        base + if su { v } else { -v }
    }

    /// 本术十四日、二十八日「分為二日，隨餘初末，如法求之」：读法同大衍一系（[MoonTable::eval_jing]），入余在初数以下者以初段为一日
    /// （长 = 初数，率 = 初率），已上者以末段为一日（长 = 末数，率 = 末率），段内照本术二次内插并作第二次改正（总法换作段长）。
    /// 邻率照同条「其後無同率，亦因前率」：十四日初段（減百二）之后为末段增二十九，不同率，因前率十三日減百二十一；
    /// 末段（增二十九）之后十五日增百二十八，同率，用后率；二十八日（初减七十一，末「增入後」）因前率二十七日減百二十五。
    /// 邻率按段长折算（× 段长/总法），以前率者 c = (Δ0−Δ1)/2（段首瞬时率 = (Δ0+Δ1)/2，与前日之末相接）。
    /// 「所得并以加減變率為定」：段内所得加于段首之积（十四日末段之首 = 速 102 − 102 = 0，入迟）。读法为本项目所定，
    /// 见 `doc/calendar/dingshuo-zhangbiao-diffs.md`「二次差精法与麟德步交会」。
    fn chisu_fen(x: &Q) -> Q {
        let f = q(LD_F);
        let i = fl(x).min(27);
        let r = (x - q(i)) * &f;
        let (chu, chu_lv, qian) = if i == 13 { (q(1042), -102, LD_ZENG[12]) } else { (q(743), LD_ZENG[27], LD_ZENG[26]) };
        let su = i == 13;
        if r <= chu || i >= 27 {
            let nb = q(qian) * &chu / &f;
            let d1 = q(chu_lv);
            let c = (nb - &d1) / q(2);
            return Self::benshu_step(q(LD_CHISU[i as usize]), d1, c, &r / &chu, &chu, su);
        }
        // 十四日末段（二十八日入余不过初数 743）：增二十九入迟，后率十五日增百二十八
        let mo = &f - &chu;
        let nb = q(LD_ZENG[14]) * &mo / &f;
        let d1 = q(29);
        let c = (&d1 - nb) / q(2);
        Self::benshu_step(q(0), d1, c, (&r - &chu) / &mo, &mo, false)
    }

    /// 速要（`month`）所得定小余距夜半不过 24 分者，以本术重算（《旧唐书·历志二》：「其定小餘二十四已下，一千三百一十六已上者，
    /// 其入氣盈朒、入曆遲速，皆須覆依本術推算，不得從粗舉速要之限」）
    fn ding_benshu(&self, y: i32, m: i64) -> Q {
        let qizong = Self::qizong(y);
        let t0 = qizong - qizong.rem_euclid(LD_SHUO);
        let zongshi = t0 - t0.rem_euclid(LD_F);
        let bian0 = qr((zongshi * LD_BIAN_QI).rem_euclid(LD_BIAN_ZHOU), LD_BIAN_QI) / q(LD_F);
        let qi_len = qr(LD_QI, 24);
        let dingqi = |k: i64| q(qizong) + &qi_len * q(k) - q(LD_XIAOXI[k.rem_euclid(24) as usize]);
        let chq = q(t0 + m * LD_SHUO);
        let mut k = fl(&((&chq - q(qizong)) / &qi_len)) - 2;
        while dingqi(k + 1) <= chq {
            k += 1;
        }
        let kk = k.rem_euclid(24) as usize;
        let yn = ld_round(q(LD_YINGNU[kk]) + Self::sun_benshu(kk, &((&chq - dingqi(k)) / q(LD_F))));
        let bian = md(&(bian0 + qr(t0.rem_euclid(LD_F), LD_F) + qr(m * LD_SHUO, LD_F) + &yn / q(LD_F)), &(qr(LD_BIAN_ZHOU, LD_BIAN_QI) / q(LD_F)));
        &chq + yn - Self::chisu_benshu(&bian, self.fen_erri)
    }
}

impl DingshuoLi for Linde {
    fn f(&self) -> i64 {
        LD_F
    }
    fn off(&self) -> i64 {
        self.off
    }
    fn sui(&self) -> Q {
        q(LD_QI)
    }
    fn dongzhi(&self, y: i32) -> Q {
        q(Self::qizong(y))
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        let (chq, yn, chisu) = self.suyao(y, m);
        let ding = &chq + yn - chisu;
        (chq, ding)
    }
    /// 麟德步交会（《新唐书·历志二》）：「置總實，以奇率乘之，滿終率去之……為入交分。加天正常朔小餘，得朔汎交分」，
    /// 「各以朔望入氣盈朒定積，盈加、朒減之；又六十乘遲速定數，七百七十七除，為限數；以速減、遲加，為定交分」。
    /// 约终 36464 奇 113（奇率 300），交中其半；後準 1553 奇 93.5（前準 = 交中 − 後準）：去交在後準以內為入蝕限。
    /// 「其朔，月在日道裏者……出日道表者，為變交分」之变交分只用于判蚀（[Linde::rishi_ld]），此处为定交分。
    fn jiao(&self, y: i32, m: i64) -> Option<Jiaohui> {
        let (chq, yn, chisu) = self.suyao(y, m);
        Some(ld_jiao().ru(&chq, &yn, &-chisu, LD_F))
    }
    /// 麟德日蚀（《新唐书·历志二》「求日蝕所在辰」及「月在外道，朔不應蝕」「月在內道，朔應蝕」诸准），见 [Linde::rishi_ld]
    fn rishi(&self, y: i32, m: i64) -> Option<Rishi> {
        self.rishi_ld(y, m)
    }
    // 两唐书麟德历皆无进朔条文；684 年起中原正月进朔见 [Rules::zhengyue]。常月进朔：709 年起当时历日五见进朔（709 年五月
    // 敦煌伯 2347《十戒經》盟文「五月丁巳朔」，710 年四月义净译经题记「四月壬午朔」、六月中宗哀册「六月辛巳朔」、十一月同册
    // 「十一月二日乙〔己〕酉」并本纪十月丁未，713 年四月本纪，726 年四月岐王哀册「四月己酉朔」），仅 726 年十一月本纪一条不进；
    // 705 年三月岱岳观题记「三月庚辰朔」、五月册文「五月己卯朔」、706 年三月本纪纪日皆不进。排月取四分之三（18 时）进朔
    // （[PaiyueOptions::linde_changyue]）。开元五至七年（717—719）当时文字七月皆不进（墓志、碑、册、祭文，见
    // [PaiyueOptions::linde_tingjin]），其前 716 年闰十二月、其后 721 年正月墓志皆进，此段停常月进朔。
    fn jinshuo_limit(&self, y: i32, t: &Q) -> Option<Q> {
        let jdn = fl(&(t / q(LD_F))) + self.off;
        if self.tingjin.is_some_and(|(a, b)| (a..b).contains(&jdn)) {
            return None;
        }
        self.changyue_jin.filter(|&from| y >= from).map(|_| qr(3 * LD_F, 4))
    }
    /// 月离为入变（速要所用之日，十四日初数 1042、二十八日 743 分）；日躔为入定气
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        let (k, t, len, bian) = self.suyao_ru(y, m);
        let i = fl(&bian).min(27);
        let yu = (&bian - q(i)).to_f64().unwrap();
        let chu = match i {
            13 => Some(1042.0 / LD_F as f64),
            27 => Some(743.0 / LD_F as f64),
            _ => None,
        };
        Some(Biaoduan { yue: Some((i as u32 + 1, yu, chu)), ri: (k.rem_euclid(24) as usize, t.to_f64().unwrap(), len.to_f64().unwrap()) })
    }
    fn ding_for_calendar(&self, y: i32, m: i64) -> Q {
        let ding = self.month(y, m).1;
        let xiao = md(&ding, &q(LD_F));
        if xiao <= q(24) || xiao >= q(1316) {
            self.ding_benshu(y, m)
        } else {
            ding
        }
    }
}

/// 麟德步交会常数：约终 36464 奇 113（奇率 300，即游交终率 10939313 ÷ 300）；後準 1553 奇 93.5；限数 = 60 × 迟速定数 ÷ 777
fn ld_jiao() -> JiaoConst {
    JiaoConst::new(qr(10939313, 300), q(1553) + qr(935, 3000), 60, 777)
}

fn linde() -> Linde {
    let dz_day = (LD_QI * LD_JI_664).div_euclid(LD_F);
    Linde { off: align_offset(dz_day, gregorian::civil_to_jdn(663, 12, 20)), jiaohui: LindeJiaohui::Xinzhi, changyue_jin: None, tingjin: None, fen_erri: true }
}

// ---------------------------------------------------------------- 戊寅（崔善为所较）

struct Wuyin {
    /// 平朔（646—664 年行）：定朔即平朔
    ping: bool,
    off: i64,
}

const WY_RI: i64 = 13006;
const WY_YUE: i64 = 384075;
const WY_QIFA: i64 = 9464;
const WY_SUI: i64 = 3456675;
const WY_LIZHOU: i64 = 798200;
const WY_LIFA: i64 = 28968;
const WY_JI_626: i64 = 164348;
const WY_SUN: [i64; 24] = [896, 398, 400, 228, 341, 450, -500, -455, -355, -555, -848, -739, -626, -456, -288, -40, -342, -455, 682, 625, 570, 513, 456, 100];
const WY_MOON: [i64; 28] =
    [392, 347, 295, 236, 169, 103, 36, -38, -112, -178, -238, -290, -341, -386, -371, -326, -275, -216, -156, -90, -23, 51, 118, 184, 243, 302, 347, 383];
const WY_XING: [i64; 28] =
    [9909, 9810, 9695, 9563, 9414, 9266, 9118, 8953, 8788, 8640, 8508, 8392, 8277, 8178, 8211, 8310, 8425, 8557, 8689, 8837, 8986, 9151, 9299, 9447, 9578, 9710, 9809, 9891];

impl Wuyin {
    fn mji(i: usize) -> i64 {
        WY_XING[..i].iter().map(|x| (x - 9037) * WY_RI).sum()
    }
    /// 距冬至 t（日）→ 定盈缩分（日法分，盈 +）
    fn sun_def(t: &Q) -> Q {
        let qd = qr(WY_SUI, WY_QIFA * 24);
        let k = fl(&(t / &qd));
        let kk = k.rem_euclid(24) as usize;
        q(WY_SUN[..kk].iter().sum()) + q(WY_SUN[kk]) * (t - q(k) * qd) / q(15)
    }
    /// 入历 x（历法分）→ 定盈缩积分 / 差法（日法分，盈 +）
    fn moon_def(x: &Q) -> Q {
        let i = (fl(&(x / q(WY_LIFA))) as usize).min(27);
        let r = x - q(i as i64 * WY_LIFA);
        (q(Self::mji(i)) + r * q(WY_MOON[i])) / q(WY_XING[i] - 676)
    }
    fn nn(y: i32) -> i64 {
        WY_JI_626 + (y - 626) as i64
    }
}

impl DingshuoLi for Wuyin {
    fn f(&self) -> i64 {
        WY_RI
    }
    fn off(&self) -> i64 {
        self.off
    }
    fn sui(&self) -> Q {
        qr(WY_SUI * WY_RI, WY_QIFA)
    }
    fn dongzhi(&self, y: i32) -> Q {
        qr(Self::nn(y) * WY_SUI, WY_QIFA) * q(WY_RI)
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        let nn = Self::nn(y);
        let jiyue = (nn * 8361).div_euclid(676);
        let dz = qr(nn * WY_SUI, WY_QIFA);
        let p = qr((jiyue + m) * WY_YUE, WY_RI);
        let pf = q(fl(&p));
        let x = md(&(q((fl(&p) * WY_LIFA).rem_euclid(WY_LIZHOU)) + (&p - &pf) * q(WY_LIFA)), &q(WY_LIZHOU));
        let jing = &p * q(WY_RI);
        if self.ping {
            return (jing.clone(), jing);
        }
        let ding = &jing + Self::sun_def(&(&p - dz)) - Self::moon_def(&x);
        (jing, ding)
    }
    /// 月离为入历（历法 28968 分为一日）；日躔为入恒气
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        if self.ping {
            return None;
        }
        let nn = Self::nn(y);
        let jiyue = (nn * 8361).div_euclid(676);
        let dz = qr(nn * WY_SUI, WY_QIFA);
        let p = qr((jiyue + m) * WY_YUE, WY_RI);
        let pf = q(fl(&p));
        let x = md(&(q((fl(&p) * WY_LIFA).rem_euclid(WY_LIZHOU)) + (&p - &pf) * q(WY_LIFA)), &q(WY_LIZHOU));
        let i = fl(&(&x / q(WY_LIFA))).min(27);
        let yu = ((&x - q(i * WY_LIFA)) / q(WY_LIFA)).to_f64().unwrap();
        let qd = qr(WY_SUI, WY_QIFA * 24);
        let t = &p - dz;
        let k = fl(&(&t / &qd));
        Some(Biaoduan { yue: Some((i as u32 + 1, yu, None)), ri: (k.rem_euclid(24) as usize, (&t - q(k) * &qd).to_f64().unwrap(), qd.to_f64().unwrap()) })
    }
}

fn wuyin(ping: bool) -> Wuyin {
    let z = qr(WY_JI_626 * WY_SUI, WY_QIFA);
    Wuyin { ping, off: align_offset(fl(&z), gregorian::civil_to_jdn(625, 12, 20)) }
}

// ---------------------------------------------------------------- 宋诸历

enum SongSun {
    /// 恒气每气 (朏朒积, 符号, 定初率, 日差)
    Table(Vec<(i64, i64, Q, Q)>),
    /// 观天历：盈缩限公式
    Guantian,
}

struct SongMoon {
    /// (朏朒积, 损益率或分段 (初数, 初率, 末率))
    days: Vec<(i64, Result<i64, (Q, i64, i64)>)>,
}

struct SongCha {
    ju: i64,
    qi: i64,
    run: i64,
    zhuan: i64,
}

struct Song {
    f: i64,
    sui: i64,
    shuo: Q,
    zhuan: Q,
    ji: i64,
    y_ji: i32,
    rinru: bool,
    cha: Option<SongCha>,
    sun: SongSun,
    moon: SongMoon,
    /// 进朔用的日出分之术
    lou: SongLou,
    off: i64,
    /// 步交会
    jiao: Option<JiaoConst>,
}

/// 宋诸历求日出分之术（进朔限用）。北宋四历各据本历《宋史》步晷漏；南宋诸历志文「法與前曆同，此不載」，以纪元历之术代用。
#[derive(Clone, Copy, PartialEq)]
enum SongLou {
    /// 崇天（卷七十二）：消息定数，x 为距二至平日（「所入氣日及加其氣下中積」，中积为常气积日）
    Chongtian,
    /// 观天（卷七十七）：消息定数，x 为午中定积日（加盈缩分）
    Guantian,
    /// 纪元（卷七十九）：午中日行积度 → 赤道内外度 → 日出分
    Jiyuan,
    /// 南宋诸历：纪元之术代用（按一日之比）
    JiyuanProxy,
}

/// 消息定数法（崇天、明天、观天）：距冬至 x（日或度）→ 日出分占一日之比。
/// 常数 = a·u²/fa（u 为距最近二至），定数 = 常数 + (b − 常数)·常数/c；春分后晨分 = p + 定数，秋分后 = q − 定数；日出分 = 晨分 + 昏明。
struct XiaoxiLou {
    l: Q,
    a: i64,
    fa: i64,
    b: Q,
    c: i64,
    p: Q,
    q: Q,
    hm: Q,
    f: i64,
}

impl XiaoxiLou {
    fn sunrise(&self, x: &Q) -> Q {
        let l = &self.l;
        let m = md(x, &(q(2) * l));
        let half = l / q(2);
        let chunfen_hou = m >= half && m < &half * q(3);
        let s = md(x, l);
        let u = if &s * q(2) <= *l { s } else { l - s };
        let chang = q(self.a) * &u * &u / q(self.fa);
        let ding = &chang + (&self.b - &chang) * &chang / q(self.c);
        let chen = if chunfen_hou { &self.p + ding } else { &self.q - ding };
        (chen + &self.hm) / q(self.f)
    }
    /// 春分之日（距冬至一象）日出分之比
    fn sunrise_chunfen(&self) -> Q {
        self.sunrise(&(&self.l / q(2)))
    }
}

fn chongtian_lou() -> XiaoxiLou {
    XiaoxiLou { l: qr(18262, 100), a: 500, fa: 7873, b: qr(1059, 2), c: 2350, p: qr(7413, 4), q: qr(11649, 4), hm: qr(1059, 4), f: 10590 }
}

/// 观天「春分後加二千一百少」从陈美东《古历新探》第七章（书页 197—198）校作二千一百五少：夏至日出恰 0.2 日
fn guantian_lou() -> XiaoxiLou {
    XiaoxiLou { l: qr(18262, 100), a: 700, fa: 9703, b: qr(1203, 2), c: 2670, p: qr(8421, 4), q: qr(13233, 4), hm: qr(1203, 4), f: 12030 }
}

fn mingtian_lou() -> XiaoxiLou {
    XiaoxiLou { l: qr(182 * MT_F + 24250, MT_F), a: 2500, fa: 10689, b: q(1950), c: 8650, p: q(6825), q: q(10725), hm: q(975), f: MT_F }
}

/// 纪元历（卷七十九）：午中日行积度 x（度，距冬至）→ 日出分占一日之比。
/// 「並置之於上，列象限於下……冬至前後五百一十七而一，夏至前後四百而一……如三十四萬八千八百五十六而一」得赤道内外度 δ，
/// 日出分 = 1822½ ± 3630δ/239（赤道外加、内减），日法 7290。
fn jiyuan_sunrise(x: &Q) -> Q {
    let l = qr(182 * 7290 + 4533, 7290);
    let xiang = &l / q(2);
    let m = md(x, &(q(2) * &l));
    let near_dongzhi = m < xiang || m >= &xiang * q(3);
    let s = md(x, &l);
    let u = if s <= xiang { s } else { &l - s };
    let a = &u + &u * (&xiang - &u) / q(if near_dongzhi { 517 } else { 400 });
    let b = &xiang - a;
    let delta = q(1000) * &b * (&l - &b) / q(348856);
    let d = q(3630) * delta / q(239);
    let sr = if near_dongzhi { qr(3645, 2) + d } else { qr(3645, 2) - d };
    sr / q(7290)
}

impl Song {
    fn qi(&self) -> Q {
        qr(self.sui, 24)
    }
    fn tong(&self, y: i32) -> Q {
        let n = self.ji + (y - self.y_ji) as i64;
        match &self.cha {
            None => q(self.sui * n),
            Some(c) => {
                let ju = n - c.ju;
                q(self.sui * n - c.qi) - qr(127 * ju * ju, 10000)
            }
        }
    }
    fn first_shuo(&self, y: i32) -> Q {
        let tong = self.tong(y);
        match &self.cha {
            None => &tong - md(&tong, &self.shuo),
            Some(c) => {
                let ju = self.ji + (y - self.y_ji) as i64 - c.ju;
                &tong - md(&(&tong - q(c.run)), &self.shuo) - qr(105 * ju, 10)
            }
        }
    }
    fn sun_def(&self, t: &Q) -> Q {
        match &self.sun {
            SongSun::Guantian => guantian_sun(&md(t, &q(self.sui)), self.f),
            SongSun::Table(tab) => {
                let qi = self.qi();
                let k = fl(&(t / &qi));
                let (ji, sign, a, cha) = &tab[k.rem_euclid(24) as usize];
                let x = t - q(k) * &qi;
                let f = q(self.f);
                let day = fl(&(&x / &f));
                let r = &x - q(day) * &f;
                let acc = a * q(day) + cha * q(day * (day - 1) / 2) + (a + cha * q(day)) * r / f;
                q(*ji) + q(*sign) * acc
            }
        }
    }
    fn moon_def(&self, x: &Q) -> Q {
        let f = q(self.f);
        let i = fl(&(x / &f)) as usize;
        let r = x - q(i as i64) * &f;
        let (ji, v) = &self.moon.days[i];
        match v {
            Ok(rate) => q(*ji) + q(*rate) * r / f,
            Err((n0, r0, r1)) => {
                if &r <= n0 {
                    q(*ji) + q(*r0) * r / n0
                } else if *r1 != 0 {
                    q(*ji) + q(*r0) + q(*r1) * (&r - n0) / (f - n0)
                } else {
                    q(*ji) + q(*r0)
                }
            }
        }
    }
}

/// 观天历：入盈缩限（距天正冬至 t 分）→ 朏朒定数（朒 +）。积 = x(2L − x) × 100 / D，x 为距二至或距二分之全日，日内线性。
fn guantian_sun(t: &Q, f: i64) -> Q {
    guantian_curve(t, f, qr(100, 366), qr(100, 407))
}

/// 观天历盈缩分（度，盈 +）：「相減相乘……在盈初、縮末，以三千二百九十四除之；在盈末、縮初，以三千六百五十九除之，皆爲度」
fn guantian_deg(t: &Q, f: i64) -> Q {
    guantian_curve(t, f, qr(1, 3294), qr(1, 3659))
}

/// 观天历日躔曲线 x(2L − x) × k（盈初缩末 k1、盈末缩初 k2），x 为距二至或距二分之全日，日内线性
fn guantian_curve(t: &Q, f: i64, k1: Q, k2: Q) -> Q {
    let hz = qr(182 * f + 7480, f);
    let (l1, l2) = (qr(88 * f + 10958, f), qr(93 * f + 8552, f));
    let d = t / q(f);
    let ying = d < hz;
    let x = if ying { d } else { d - &hz };
    let val = |u: &Q| -> Q {
        if ying {
            if *u <= l1 {
                u * (q(2) * &l1 - u) * &k1
            } else {
                let v = &hz - u;
                &v * (q(2) * &l2 - &v) * &k2
            }
        } else if *u <= l2 {
            u * (q(2) * &l2 - u) * &k2
        } else {
            let v = &hz - u;
            &v * (q(2) * &l1 - &v) * &k1
        }
    };
    let i = fl(&x);
    let lo = val(&q(i));
    let hi_u = if q(i + 1) < hz { q(i + 1) } else { hz.clone() };
    let s = &lo + (val(&hi_u) - &lo) * (&x - q(i));
    if ying {
        s
    } else {
        -s
    }
}

impl DingshuoLi for Song {
    fn f(&self) -> i64 {
        self.f
    }
    fn off(&self) -> i64 {
        self.off
    }
    fn sui(&self) -> Q {
        q(self.sui)
    }
    fn dongzhi(&self, y: i32) -> Q {
        self.tong(y)
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        let tong = self.tong(y);
        let t0 = self.first_shuo(y);
        let base = match &self.cha {
            Some(c) => md(&(&t0 - q(c.zhuan)), &self.zhuan),
            None => md(&t0, &self.zhuan),
        };
        let jing = &t0 + q(m) * &self.shuo;
        let ding = &jing + self.sun_def(&(&jing - &tong)) + self.moon_def(&md(&(base + q(m) * &self.shuo), &self.zhuan));
        (jing, ding)
    }
    /// 月离为入转（分段之日初数照表）；日躔为入恒气（观天以公式，亦按恒气记）
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        let tong = self.tong(y);
        let t0 = self.first_shuo(y);
        let base = match &self.cha {
            Some(c) => md(&(&t0 - q(c.zhuan)), &self.zhuan),
            None => md(&t0, &self.zhuan),
        };
        let jing = &t0 + q(m) * &self.shuo;
        let f = q(self.f);
        let x = md(&(base + q(m) * &self.shuo), &self.zhuan);
        let i = (fl(&(&x / &f)) as usize).min(self.moon.days.len() - 1);
        let yu = ((&x - q(i as i64) * &f) / &f).to_f64().unwrap();
        let chu = match &self.moon.days[i].1 {
            Err((n0, _, _)) => Some((n0 / &f).to_f64().unwrap()),
            Ok(_) => None,
        };
        let t = md(&(&jing - &tong), &q(self.sui));
        let qi = self.qi();
        let k = fl(&(&t / &qi));
        Some(Biaoduan { yue: Some((i as u32 + 1, yu, chu)), ri: (k.rem_euclid(24) as usize, ((&t - q(k) * &qi) / &f).to_f64().unwrap(), (qi / f).to_f64().unwrap()) })
    }
    fn jiao(&self, y: i32, m: i64) -> Option<Jiaohui> {
        let jc = self.jiao.as_ref()?;
        let tong = self.tong(y);
        let t0 = self.first_shuo(y);
        let base = match &self.cha {
            Some(c) => md(&(&t0 - q(c.zhuan)), &self.zhuan),
            None => md(&t0, &self.zhuan),
        };
        let jing = &t0 + q(m) * &self.shuo;
        Some(jc.ru(&jing, &self.sun_def(&(&jing - &tong)), &self.moon_def(&md(&(base + q(m) * &self.shuo), &self.zhuan)), self.f))
    }
    /// 宋诸历日食（简法）：有食 = 入食限（[Jiaohui::shixian]），阴阳历按入交定；食甚 = 定朔小余以时差加减（「求食定餘」）：
    /// 午前时差 =（半法 − 午前分）× 午前分 ÷ A，以减；午后 =（半法 − 午后分）× 午后分 ÷ B，以加。A、B（日法之分）崇天
    /// 31770、13885（卷七十三），观天 36090、18045（卷七十八），纪元 10935、日法（卷八十；南宋诸历「法同前曆」，按一日之比
    /// 用纪元之数）。不算气差、刻差与食分。
    fn rishi(&self, y: i32, m: i64) -> Option<Rishi> {
        if self.lou == SongLou::Chongtian {
            return self.chongtian_rishi(y, m);
        }
        let j = self.jiao(y, m)?;
        let fq = q(self.f);
        let frac = md(&self.month(y, m).1, &fq) / &fq;
        let (a, b) = match self.lou {
            SongLou::Chongtian => (qr(31770, 10590), qr(13885, 10590)),
            SongLou::Guantian => (qr(36090, 12030), qr(18045, 12030)),
            SongLou::Jiyuan | SongLou::JiyuanProxy => (qr(10935, 7290), q(1)),
        };
        let half = qr(1, 2);
        let shen = if frac <= half {
            let x = &half - &frac;
            &frac - (&half - &x) * &x / a
        } else {
            let x = &frac - &half;
            &frac + (&half - &x) * &x / b
        };
        // 汎用分（食甚距亏初），以去交分代三差后之交前后分（近似）；定用 = 汎用 × 1337 / 转定分，转定分取其平 1337，即以汎用为定用
        let qu = &j.qu * &fq;
        let dingyong = match self.lou {
            // 崇天：交前后分 ≤ 阳历食限 4200 为阳历食定分，已上覆减 11200 为阴历食定分；百约为 a，阳历 (84 − a)·a·100/185，
            // 阴历 (140 − a)·a·100/514（枢法之分）
            SongLou::Chongtian => {
                let (a, k, d) = if qu <= q(4200) { (&qu / q(100), 84, 185) } else { ((q(11200) - &qu).max(Q::zero()) / q(100), 140, 514) };
                (q(k) - &a) * &a * q(100) / q(d) / &fq
            }
            // 观天：阳历食限 4900，阴历覆减 12800；退二位为 a，阳 (98 − a)·a/250，阴 (158 − a)·a/650（约分，百分日）
            SongLou::Guantian => {
                let (a, k, d) = if qu <= q(4900) { (&qu / q(100), 98, 250) } else { ((q(12800) - &qu).max(Q::zero()) / q(100), 158, 650) };
                (q(k) - &a) * &a / q(d) / q(100)
            }
            // 纪元：交前后分自相乘，退二位，阳历 198、阴历 317 而一，用减 583（日法 7290 之分）；南宋诸历按一日之比
            SongLou::Jiyuan | SongLou::JiyuanProxy => {
                let u = &qu * q(7290) / &fq;
                ((q(583) - &u * &u / q(100) / q(if j.yin { 317 } else { 198 })) / q(7290)).max(Q::zero())
            }
        };
        Some(Rishi { shi: j.shixian, yin: j.yin, shen, dingyong: Some(dingyong) })
    }
    /// 定朔所在日午中之日出（同进朔限所用，[song_jinshuo_limit]）
    fn sunrise(&self, y: i32, t: &Q) -> Option<Q> {
        let fq = q(self.f);
        let noon = (q(fl(&(t / &fq))) + qr(1, 2)) * &fq;
        let d = self.since_dongzhi(y, &noon);
        Some(Song::sunrise(self, &(&d / &fq + self.yingsuo_deg(&d))))
    }
    fn jinshuo_limit(&self, y: i32, t: &Q) -> Option<Q> {
        Some(song_jinshuo_limit(
            self.f,
            &q(self.sui),
            |noon| self.since_dongzhi(y, noon),
            t,
            self.rinru,
            |x| self.sunrise(x),
            self.sunrise_chunfen(),
            |d| self.yingsuo_deg(d),
        ))
    }
}

impl Song {
    /// 崇天历日食（《宋史》卷七十三步交會，枢法 10590）：
    /// - 食定余：午前分 p =（半法 − 小余），时差 =（半法 − p）·p / 31770，以减；午后分 p =（小余 − 半法），时差 =（半法 − p）·p / 13885，
    ///   以加。距午定分 = 午前后分 + 时差。
    /// - 气差：中积（定朔距天正冬至，常气日）满二至限去之，距二至 u 日，气差 = 3533 − u²·100/236；定数 = 气差 − 气差 × 距午定分 / 半晝分
    ///   （半晝分 = 半法 − 日出分）；春分后交初减、交中加，秋分后交初加、交中减。
    /// - 刻差：x = 中积 mod 二至限，刻差 = (二至限 − x)·x·100/236；定数 = 刻差 × 距午定分 × 4 / 枢法；冬至后午前、夏至后午后交初加、交中减，
    ///   反之交初减、交中加（「陽初陰末爲交初，陰初陽末爲交中」）。
    /// - 时差：随食定余加减入交（午前减、午后加）。
    /// - 日入食限：入交定加减三差，在中日已下不食；已上去中日，后限已下为交后分，前限已上覆减中日为交前分。交前后分 ≤ 阳历食限 4200
    ///   为阳历食定分，已上覆减 11200 为阴历食定分，不足减不食。
    /// - 汎用分：食定分百约为 a，阳历 (84 − a)·a·100/185，阴历 (140 − a)·a·100/514；定用分 = 汎用 × 1337 / 转定分（转定分由月离损益率
    ///   折算，志表之转定分未入本项目之表）。
    fn chongtian_rishi(&self, y: i32, m: i64) -> Option<Rishi> {
        let jc = self.jiao.as_ref()?;
        let j = self.jiao(y, m)?;
        let fq = q(self.f);
        let ding = self.month(y, m).1;
        let xiao = md(&ding, &fq);
        let banfa = &fq / q(2);
        let (shicha, pm, p) = if xiao <= banfa {
            let p = &banfa - &xiao;
            ((&banfa - &p) * &p / q(31770), false, p)
        } else {
            let p = &xiao - &banfa;
            ((&banfa - &p) * &p / q(13885), true, p)
        };
        let shen_fen = if pm { &xiao + &shicha } else { &xiao - &shicha };
        let juwu = &p + &shicha;
        let sr = <Song as DingshuoLi>::sunrise(self, y, &ding)? * &fq;
        let banzhou = &banfa - &sr;
        let zhongji = md(&(&ding - self.tong(y)), &q(self.sui)) / &fq;
        let erzhi = qr(18262, 100);
        let x = md(&zhongji, &erzhi);
        let u = if x <= &erzhi / q(2) { x.clone() } else { &erzhi - &x };
        let jiaozhong = j.yin != j.qian; // 阴初（阴历交后）、阳末（阳历交前）为交中
        let jz = |v: Q| if jiaozhong { v } else { -v };
        let qicha = q(3533) - &u * &u * q(100) / q(236);
        let qi_d = &qicha - &qicha * &juwu / &banzhou;
        let sui = q(self.sui) / &fq;
        let chunfen_hou = zhongji >= &sui / q(4) && zhongji < &sui * qr(3, 4);
        let d_qi = if chunfen_hou { jz(qi_d) } else { -jz(qi_d) };
        let kecha = (&erzhi - &x) * &x * q(100) / q(236);
        let ke_d = &kecha * &juwu * q(4) / &fq;
        let dongzhi_hou = zhongji < erzhi;
        let d_ke = if dongzhi_hou != pm { -jz(ke_d) } else { jz(ke_d) };
        let d_shi = if pm { shicha.clone() } else { -shicha.clone() };
        let zhong = &jc.zhong;
        let zhongri = zhong / q(2);
        let xr = md(&(&j.ru * &fq + d_qi + d_ke + d_shi), zhong);
        let (houxian, qianxian) = (jc.xian.clone(), &zhongri - &jc.xian);
        let fen = if xr < zhongri {
            None
        } else {
            let w = &xr - &zhongri;
            if w <= houxian {
                Some(w)
            } else if w >= qianxian {
                Some(&zhongri - w)
            } else {
                None
            }
        };
        let (shi, yang, a) = match fen {
            Some(f) if f <= q(4200) => (true, true, f / q(100)),
            Some(f) if f < q(11200) => (true, false, (q(11200) - f) / q(100)),
            _ => (false, false, Q::zero()),
        };
        let fanyong = if yang { (q(84) - &a) * &a * q(100) / q(185) } else { (q(140) - &a) * &a * q(100) / q(514) };
        // 转定分（月行一日之分，平 1337）：由入转损益率折算，月行 = 13.37 度 − 损益率 × 12.37 / 枢法（朒向益为月迟）
        let t0 = self.first_shuo(y);
        let x = md(&(md(&t0, &self.zhuan) + q(m) * &self.shuo), &self.zhuan);
        let half_day = &fq / q(2);
        let rate = self.moon_def(&md(&(&x + &half_day), &self.zhuan)) - self.moon_def(&md(&(&x - &half_day + &self.zhuan), &self.zhuan));
        let zhuanding = q(1337) - rate * q(1237) / &fq;
        let dingyong = fanyong * q(1337) / zhuanding;
        Some(Rishi { shi, yin: shi, shen: shen_fen / &fq, dingyong: Some(dingyong / &fq) })
    }

    /// 进朔限所用之日（定朔所在日午中）距天正冬至的盈缩改正（度）：崇天中积用常气，不加；观天按本历盈缩分之术；
    /// 纪元先后数与朏朒积成比例（春分先二萬四千、朒一千三百九，单位万分度）；南宋诸历以纪元之比折算。
    fn yingsuo_deg(&self, d: &Q) -> Q {
        match self.lou {
            SongLou::Chongtian => q(0),
            SongLou::Guantian => guantian_deg(&md(d, &q(self.sui)), self.f),
            SongLou::Jiyuan | SongLou::JiyuanProxy => self.sun_def(d) / q(self.f) * q(7290) * q(24000) / q(1309 * 10000),
        }
    }
    fn sunrise(&self, x: &Q) -> Q {
        match self.lou {
            SongLou::Chongtian => chongtian_lou().sunrise(x),
            SongLou::Guantian => guantian_lou().sunrise(x),
            SongLou::Jiyuan | SongLou::JiyuanProxy => jiyuan_sunrise(x),
        }
    }
    fn sunrise_chunfen(&self) -> Q {
        match self.lou {
            SongLou::Chongtian => chongtian_lou().sunrise_chunfen(),
            SongLou::Guantian => guantian_lou().sunrise_chunfen(),
            SongLou::Jiyuan | SongLou::JiyuanProxy => jiyuan_sunrise(&qr(182 * 7290 + 4533, 2 * 7290)),
        }
    }
}

/// 宋诸历进朔（纪元历注文，明天、观天同）：秋分后小余 ≥ 3/4 日法；春分后限减（春分日出分 − 定朔日出分）/3。崇天：小余不小于日入分者进。
/// 日出分按各历步晷漏于定朔所在日午中求之（`SongLou`），南宋诸历代用纪元之术。
fn song_jinshuo_limit(f: i64, sui: &Q, d_at: impl Fn(&Q) -> Q, t: &Q, rinru: bool, sunrise: impl Fn(&Q) -> Q, chunfen: Q, yingsuo: impl Fn(&Q) -> Q) -> Q {
    let fq = q(f);
    let noon = (q(fl(&(t / &fq))) + qr(1, 2)) * &fq;
    let d = d_at(&noon);
    let x = &d / &fq + yingsuo(&d);
    let sr = sunrise(&x);
    if rinru {
        return &fq * (q(1) - sr);
    }
    let qi = sui / q(24);
    if d >= &qi * q(6) && d < &qi * q(18) {
        return &fq * (qr(3, 4) - (chunfen - sr) / q(3));
    }
    &fq * qr(3, 4)
}

impl Song {
    /// 时刻 t 所属岁距该岁天正冬至（分）
    fn since_dongzhi(&self, y: i32, t: &Q) -> Q {
        let sui = q(self.sui);
        (y - 1..=y + 2).map(|yy| t - self.tong(yy)).find(|d| !d.is_negative() && *d < sui).unwrap_or_else(|| md(&(t - self.tong(y)), &sui))
    }
}

/// 奉元历（1075—1093）补表复原：经朔、恒气用清李锐《补修宋奉元术》所补步气朔常数（`data/calendar/lirui_buxiu_constants.csv`：
/// 元法 23700、岁周 8656273、朔实 699875、演纪上元甲子距熙宁七年甲寅 83185070 算外）；奉元日躔、月离与进朔术不传，
/// 定朔改正（定朔 − 经朔）、进朔限、日出与交会借观天历（同一时刻折算到观天日法）。奉元「視《明天曆》數減二刻」
/// （《古今图书集成·历法典》第七十六卷），借明天、崇天改正亦试过，本纪合数不及观天（doc/calendar/calendar-compendium.md「奉元」）。
struct FengyuanBuxiu {
    base: Song,
    off: i64,
}

const FY_F: i64 = 23700;
const FY_SUI: i64 = 8656273;
const FY_SHUO: i64 = 699875;
const FY_JI: i64 = 83185070;
const FY_Y_JI: i32 = 1074;

impl FengyuanBuxiu {
    fn new() -> Self {
        let base = song("观天", 12030, 4393880, q(355253), qr(3314820389, 10000), 5944808, 1092, 0, false, None, (1091, 12, 16));
        let dz = fl(&(Self::tong(FY_Y_JI) / q(FY_F)));
        let off = align_offset(dz, gregorian::civil_to_jdn(1073, 12, 15));
        FengyuanBuxiu { base, off }
    }
    fn tong(y: i32) -> Q {
        q(FY_SUI * (FY_JI + (y - FY_Y_JI) as i64))
    }
    /// 本历之分 → 观天之分（同一时刻）
    fn to_base(&self, t: &Q) -> Q {
        (t / q(FY_F) + q(self.off - self.base.off)) * q(self.base.f)
    }
    /// 冬至岁 y 第 m 月对应的观天月序（经朔最近者）
    fn base_m(&self, y: i32, m: i64) -> i64 {
        let jing = self.to_base(&self.month_jing(y, m));
        (m - 1..=m + 1)
            .filter(|&k| k >= 0)
            .min_by_key(|&k| (DingshuoLi::month(&self.base, y, k).0 - &jing).abs())
            .unwrap()
    }
    fn month_jing(&self, y: i32, m: i64) -> Q {
        let tong = Self::tong(y);
        &tong - md(&tong, &q(FY_SHUO)) + q(m * FY_SHUO)
    }
}

impl DingshuoLi for FengyuanBuxiu {
    fn f(&self) -> i64 {
        FY_F
    }
    fn off(&self) -> i64 {
        self.off
    }
    fn sui(&self) -> Q {
        q(FY_SUI)
    }
    fn dongzhi(&self, y: i32) -> Q {
        Self::tong(y)
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        let jing = self.month_jing(y, m);
        let (bj, bd) = DingshuoLi::month(&self.base, y, self.base_m(y, m));
        let ding = &jing + (bd - bj) * q(FY_F) / q(self.base.f);
        (jing, ding)
    }
    fn jinshuo_limit(&self, y: i32, t: &Q) -> Option<Q> {
        DingshuoLi::jinshuo_limit(&self.base, y, &self.to_base(t)).map(|l| l * q(FY_F) / q(self.base.f))
    }
    fn sunrise(&self, y: i32, t: &Q) -> Option<Q> {
        DingshuoLi::sunrise(&self.base, y, &self.to_base(t))
    }
    fn jiao(&self, y: i32, m: i64) -> Option<Jiaohui> {
        DingshuoLi::jiao(&self.base, y, self.base_m(y, m))
    }
    fn rishi(&self, y: i32, m: i64) -> Option<Rishi> {
        DingshuoLi::rishi(&self.base, y, self.base_m(y, m))
    }
    /// 借观天日月改正：观天之格
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        DingshuoLi::biaoduan(&self.base, y, self.base_m(y, m))
    }
}

/// 明天历（周琮）：日月皆用公式，不用立成表（《宋史》卷七十四、七十五）
struct Mingtian {
    off: i64,
}

const MT_F: i64 = 39000;
const MT_SUI: i64 = 14244500;
const MT_SHUO: i64 = 1151693;
const MT_JI: i64 = 711760;

impl Mingtian {
    fn tong(y: i32) -> i64 {
        MT_SUI * (MT_JI + (y - 1064) as i64)
    }
    fn sun_def(t: &Q) -> Q {
        let l = qr(182 * MT_F + 24250, MT_F);
        let d = md(t, &q(MT_SUI)) / q(MT_F);
        let ying = d < l;
        let mut x = if ying { d } else { d - &l };
        let y = &l - &x;
        if y < x {
            x = y;
        }
        // 底本「列二至於下」，「至」为「百」之讹，作 x(200 − x)：周琮《岳臺晷景新書》同式「列二百於下……滿四千一百三十五除之」
        // （《宋史》卷七十六），明天历议「新曆以七千一為盈縮之極」（卷七十四；x = L/2 时 400·x(200 − x)/567 = 7001.4，
        // 作二至限则 5882），月离同例「列二百一度九分」；张培瑜等《中国古代历法》书页 108 亦校作「二百」
        let v = qr(400, 567) * &x * (q(200) - &x);
        if ying {
            v
        } else {
            -v
        }
    }
    fn moon_of(jiyue: i64) -> Q {
        let r = qbig((jiyue as i128 * 2142887000).rem_euclid(29882242251), 81120000);
        let zhong = q(184) + qr(15041125 * 2 + 1, 2 * 81120000);
        let fast = r < zhong;
        let mut x = if fast { r } else { r - &zhong };
        let y = &zhong - &x;
        if y < x {
            x = y;
        }
        let v = qr(10000 * 2, 13547) * &x * (qr(20109, 100) - &x);
        // 从注文「疾加、遲減」
        if fast {
            v
        } else {
            -v
        }
    }
}

impl DingshuoLi for Mingtian {
    fn f(&self) -> i64 {
        MT_F
    }
    fn off(&self) -> i64 {
        self.off
    }
    fn sui(&self) -> Q {
        q(MT_SUI)
    }
    fn dongzhi(&self, y: i32) -> Q {
        q(Self::tong(y))
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        let tong = Self::tong(y);
        let jy = tong.div_euclid(MT_SHUO);
        let jing = (jy + m) * MT_SHUO;
        (q(jing), q(jing) + Self::sun_def(&q(jing - tong)) + Self::moon_of(jy + m))
    }
    /// 明天历注文：秋分后 3/4；「春分後，其定朔晨分差如春分之日者，三約之，以減四分之三」。晨分按本历步晷漏（卷七十五），
    /// 消息定数用「日中日度」：平日加盈缩差度（x(200 − x)/4135，即盈缩定差 × 567/(400 × 4135)）。
    fn jinshuo_limit(&self, y: i32, t: &Q) -> Option<Q> {
        let sui = q(MT_SUI);
        let lou = mingtian_lou();
        Some(song_jinshuo_limit(
            MT_F,
            &sui,
            |noon| {
                (y - 1..=y + 2).map(|yy| noon - q(Self::tong(yy))).find(|d| !d.is_negative() && *d < sui).unwrap_or_else(|| md(&(noon - q(Self::tong(y))), &sui))
            },
            t,
            false,
            |x| lou.sunrise(x),
            lou.sunrise_chunfen(),
            |d| Self::sun_def(d) * q(567) / q(400 * 4135),
        ))
    }
}

/// 宋诸历之表（`data/calendar/dingshuo_song_tables.csv`）
fn song_tables(name: &str) -> (Option<Vec<(i64, i64, Q, Q)>>, SongMoon) {
    let csv = include_str!("../../../../data/calendar/dingshuo_song_tables.csv");
    let mut sun = Vec::new();
    let mut days = Vec::new();
    for line in csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("历,")) {
        let c: Vec<&str> = line.split(',').collect();
        if c[0] != name {
            continue;
        }
        let n = |s: &str| s.parse::<i64>().unwrap();
        match c[1] {
            "日躔" => sun.push((n(c[3]), n(c[4]), parse_q(c[5]), parse_q(c[6]))),
            "月离" => days.push((n(c[3]), Ok(n(c[4])))),
            "月离分段" => days.push((n(c[3]), Err((parse_q(c[4]), n(c[5]), n(c[6]))))),
            _ => {}
        }
    }
    assert_eq!(days.len(), 28, "{name} 月离表");
    (if sun.is_empty() { None } else { assert_eq!(sun.len(), 24); Some(sun) }, SongMoon { days })
}

/// 宋诸历步交会常数（《宋史·律历志》）：交终分、后限（望差，= 朔差之半）、交率、交数，秒母皆一万。
/// 崇天卷七十三、观天卷七十八、纪元卷八十、统元乾道淳熙会元卷八十三（统元正文，三历注文）、统天开禧成天卷八十四（统天正文，二历注文）。
/// 后限志有明文者（崇天、观天、统元、乾道）用之，余以朔差之半；明天以交初度另立一术，未复原。
fn song_jiao(name: &str) -> Option<JiaoConst> {
    let j = |zhong: i64, shuocha: i64, lv: i64, shu: i64| Some(JiaoConst::new(qr(zhong, 10000), qr(shuocha, 20000), lv, shu));
    match name {
        // 交終分二十八萬八千一百七十七、秒四千二百七十七；後限日一、餘一千六百八十五、秒七千八百六十一半（朔差二日三千三百七十一、秒五千七百二十三之半）
        "崇天" => j(2881774277, 245515723, 141, 1796),
        // 交終分三十二萬七千三百六十一、秒九千九百四十四；後限日一、餘一千九百一十五、秒五千二十八（朔差二日三千八百三十一、秒五十六之半）
        "观天" => j(3273629944, 278900056, 183, 2331),
        // 交終分一十九萬八千三百七十七、秒八百八十；朔差二日二千三百二十、秒九千一百二十（志不列後限，取半）
        "纪元" => j(1983770880, 169009120, 324, 4127),
        // 交終分一十八萬八千五百八十、秒六千四百五十七；後限一日一千一百三、秒一千七百七十一半
        "统元" => j(1885806457, 160663543, 42, 535),
        // 乾道八十一萬六千三百六十六、秒六千三十四；後限一日四千七百七十五、秒五千七百八十三；交率八十、交數一千一十九
        "乾道" => j(8163666034, 695511566, 80, 1019),
        // 淳熙交實一十五萬三千四百七十六、秒九千五百四十三；朔差二日一千七百九十五、秒六千五十七；交率六十一、交數七百七十七
        "淳熙" => j(1534769543, 130756057, 61, 777),
        // 會元一百五萬三千一百一十三、秒二千一百四十（注文「交率」，即交終）；朔差二日一萬二千三百二十、秒七千八百六十；交率五百七。
        // 志不載會元交數，以交率 : 交數 = 朔差 : 朔實（他历皆然，统元 42/535 = 0.07850）推得 6458（507/6458 = 0.078507，朔差/朔實 0.078508）
        "会元" => j(10531132140, 897207860, 507, 6458),
        // 統天交實三十二萬六千五百四十七；交差八萬二百九十八（同轉差、閏差，自朔定積減之）；朔差二日三千八百二十一；交率十九、交數二百四十二
        "统天" => {
            let mut c = JiaoConst::new(q(326547), qr(27821, 2), 19, 242);
            c.cha = q(80298);
            Some(c)
        }
        // 開禧「交率」四十五萬九千八百八十六、秒四千八百二十五（即交終）；朔差二日五千三百八十、秒五千一百七十五
        "开禧" => j(4598864825, 391805175, 19, 242),
        // 成天交終分二十萬一千九百一十四、秒七千五十一；朔差二日二千三百六十二、秒二千九百四十九
        "成天" => j(2019147051, 172022949, 19, 242),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn song(name: &str, f: i64, sui: i64, shuo: Q, zhuan: Q, ji: i64, y_ji: i32, day0: i64, rinru: bool, cha: Option<SongCha>, approx: (i32, i32, i32)) -> Song {
    let (sun, moon) = song_tables(name);
    let sun = sun.map_or(SongSun::Guantian, SongSun::Table);
    let lou = match name {
        "崇天" => SongLou::Chongtian,
        "观天" => SongLou::Guantian,
        "纪元" => SongLou::Jiyuan,
        _ => SongLou::JiyuanProxy,
    };
    let jiao = song_jiao(name);
    let mut s = Song { f, sui, shuo, zhuan, ji, y_ji, rinru, cha, sun, moon, lou, off: 0, jiao };
    let dz = fl(&(s.tong(y_ji) / q(f)));
    s.off = align_offset(dz + day0, gregorian::civil_to_jdn(approx.0, approx.1, approx.2)) + day0;
    s
}

// ---------------------------------------------------------------- 授时、大统

struct Shoushi {
    run_ying: i64,
    zhuan_ying: i64,
    off: i64,
}

const SS_F: i64 = 10000;
const SS_SUI: i64 = 3652425;
const SS_ZHUAN: i64 = 275546;
const SS_QI_YING: i64 = 550600;

fn ss_shuo() -> Q {
    qr(29530593, 100)
}

fn ss_poly(x: &Q, a: i64, b: i64, c: i64) -> Q {
    (q(a) - (q(b) + q(c) * x) * x) * x / q(100_000_000)
}

/// 入盈缩历：t 为距冬至分。返回盈缩差（度，盈 +）
fn ss_sun(t: &Q) -> Q {
    let t = md(t, &q(SS_SUI));
    let half = qr(SS_SUI, 2);
    let f = q(SS_F);
    let ying = t < half;
    let x = if ying { &t / &f } else { (&t - &half) / &f };
    let l_ying = q(88 * SS_F + 9092) + qr(1, 4);
    let l_suo = q(93 * SS_F + 7120) + qr(1, 4);
    if ying {
        if &x * &f <= l_ying {
            return ss_poly(&x, 5133200, 24600, 31);
        }
        return ss_poly(&(&half / &f - &x), 4870600, 22100, 27);
    }
    if &x * &f <= l_suo {
        return -ss_poly(&x, 4870600, 22100, 27);
    }
    -ss_poly(&(&half / &f - &x), 5133200, 24600, 31)
}

/// 入转 r（分）→ 迟疾差（度，疾 +）
fn ss_moon(r: &Q) -> Q {
    let r = md(r, &q(SS_ZHUAN));
    let zhong = qr(SS_ZHUAN, 2);
    let fast = r < zhong;
    let d = if fast { r / q(SS_F) } else { (r - zhong) / q(SS_F) };
    let mut x = d * qr(122, 10);
    if x > q(84) {
        x = q(168) - x;
    }
    let v = (q(11110000) - (q(28100) + q(325) * &x) * &x) * &x / q(100_000_000);
    if fast {
        v
    } else {
        -v
    }
}

fn ss_speed(r: &Q) -> Q {
    let h = q(10);
    qr(1336875, 100000) + (ss_moon(&(r + &h)) - ss_moon(&(r - &h))) / (q(2) * h / q(SS_F))
}

impl DingshuoLi for Shoushi {
    fn f(&self) -> i64 {
        SS_F
    }
    fn off(&self) -> i64 {
        self.off
    }
    fn sui(&self) -> Q {
        q(SS_SUI)
    }
    fn dongzhi(&self, y: i32) -> Q {
        q((y - 1281) as i64 * SS_SUI + SS_QI_YING)
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        let zj = q((y - 1281) as i64 * SS_SUI);
        let dz = &zj + q(SS_QI_YING);
        let run = md(&(&zj + q(self.run_ying)), &ss_shuo());
        let t0 = &dz - &run;
        let ru0 = md(&(&zj + q(self.zhuan_ying) - &run), &q(SS_ZHUAN));
        let jing = &t0 + q(m) * ss_shuo();
        let r = &ru0 + q(m) * ss_shuo();
        let deg = ss_sun(&(&jing - &dz)) - ss_moon(&r);
        // 除数用月每限实行度，不减日行：《大统历日通轨》「以其下遲疾度分而一」，立成初限疾曆 679.3140、遲曆 832.0649 即 820 除以月实行度；
        // 永乐十五年、万历三十三年钦天监刊本三处朔日亦证。《明史·历志》「遲疾限行度內減去八百二十分」与颁历不合。
        let v = ss_speed(&r) * qr(82, 1000);
        let ding = &jing + deg / v * q(820);
        (jing, ding)
    }
    /// 公式（招差），无表格：月离记入转日序，日躔记入恒气
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        let zj = q((y - 1281) as i64 * SS_SUI);
        let dz = &zj + q(SS_QI_YING);
        let run = md(&(&zj + q(self.run_ying)), &ss_shuo());
        let ru0 = md(&(&zj + q(self.zhuan_ying) - &run), &q(SS_ZHUAN));
        let r = md(&(&ru0 + q(m) * ss_shuo()), &q(SS_ZHUAN)) / q(SS_F);
        let t = (&dz - &run + q(m) * ss_shuo() - &dz) / q(SS_F);
        Some(ss_biaoduan(&r, &t))
    }
}

/// 授时、大统：入转 r（日）、距冬至 t（日）→ 表段（入转日序、入恒气）
fn ss_biaoduan(r: &Q, t: &Q) -> Biaoduan {
    let i = fl(r);
    let qd = qr(SS_SUI, 24 * SS_F);
    let t = md(t, &qr(SS_SUI, SS_F));
    let k = fl(&(&t / &qd));
    Biaoduan { yue: Some((i as u32 + 1, (r - q(i)).to_f64().unwrap(), None)), ri: (k.rem_euclid(24) as usize, (&t - q(k) * &qd).to_f64().unwrap(), qd.to_f64().unwrap()) }
}

/// 大统历：照《大统暦日通轨》步气朔术文查立成定朔（`data/calendar/datong_tonggui_licheng_used.csv`，由
/// `scripts/calendar_data/datong_tonggui_dingshuo.py --export` 从两张转录表导出）。钦天监颁历用立成而非三次式：
/// 迟疾历日率按转终 ÷ 336 一限累计，积度在八十二至八十五限另有取舍，与公式定朔逐月差中位 1.9 分、最大 29 分；
/// 弘治十年十月、万历三十八年二月两月因此与《明实录》相合（公式推步差在子夜两侧）。
/// 截分（术文未言，假定）：盈缩差、迟疾差截至表末位，加减差截至秒；不截结果相同。
struct Tonggui {
    /// (日率分, 损益秒每日分（损负）, 积度秒, 疾历列, 迟历列)
    moon: Vec<(Q, Q, Q, Q, Q)>,
    /// 太阳 (冬至表?, 积日) → (加分, 积分)
    sun: std::collections::BTreeMap<(bool, i64), (Q, Q)>,
    off: i64,
}

fn trunc_to(x: &Q, unit: &Q) -> Q {
    (x / unit).trunc() * unit
}

impl Tonggui {
    fn new() -> Self {
        let csv = include_str!("../../../../data/calendar/datong_tonggui_licheng_used.csv");
        let mut moon = Vec::new();
        let mut sun = std::collections::BTreeMap::new();
        for line in csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("类,")) {
            let c: Vec<&str> = line.split(',').collect();
            match c[0] {
                "太阴" if !c[3].is_empty() => moon.push((parse_q(c[2]), parse_q(c[3]), parse_q(c[4]), parse_q(c[5]), parse_q(c[6]))),
                "太阴" => moon.push((parse_q(c[2]), Q::zero(), Q::zero(), Q::zero(), Q::zero())),
                "太阳" => {
                    sun.insert((c[1] == "冬", c[2].parse::<i64>().unwrap()), (parse_q(c[3]), parse_q(c[4])));
                }
                _ => {}
            }
        }
        assert_eq!(moon.len(), 169, "太阴立成");
        Tonggui { moon, sun, off: align_offset(SS_QI_YING / SS_F, gregorian::civil_to_jdn(1280, 12, 14)) }
    }

    /// 盈缩历 li（分，0—半岁周），ying 盈历否 → 盈缩差（度，盈正缩负）
    fn ying_suo(&self, li: &Q, ying: bool) -> Q {
        let half = qr(SS_SUI, 2);
        let first = if ying { li < &(q(88 * SS_F + 9092) + qr(1, 4)) } else { li < &(q(93 * SS_F + 7120) + qr(1, 4)) };
        // 盈初、缩末查冬至前后表；缩初、盈末查夏至前后表
        let winter = if ying { first } else { !first };
        let x = if first { li.clone() } else { &half - li };
        let day = fl(&(&x / q(SS_F)));
        let frac = &x - q(day * SS_F);
        let (a, j) = &self.sun[&(winter, day)];
        let part = trunc_to(&(a * frac / q(SS_F)), &qr(1, SS_F));
        let deg = (j + part) / q(SS_F);
        if ying {
            deg
        } else {
            -deg
        }
    }

    /// 迟疾历 li（分，0—小转中）→ (迟疾差（度，疾正迟负）, 限行度列)
    fn chi_ji(&self, li: &Q, fast: bool) -> (Q, Q) {
        let n = (0..168).filter(|&k| &self.moon[k].0 <= li).max().unwrap();
        let (rl, sy, jd, ji, ch) = &self.moon[n];
        let sec = trunc_to(&(jd + sy * (li - rl)), &qr(1, SS_F));
        let deg = sec / q(SS_F);
        if fast {
            (deg, ji.clone())
        } else {
            (-deg, ch.clone())
        }
    }
}

impl DingshuoLi for Tonggui {
    fn f(&self) -> i64 {
        SS_F
    }
    fn off(&self) -> i64 {
        self.off
    }
    fn sui(&self) -> Q {
        q(SS_SUI)
    }
    fn dongzhi(&self, y: i32) -> Q {
        q((y - 1281) as i64 * SS_SUI + SS_QI_YING)
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        let zj = q((y - 1281) as i64 * SS_SUI);
        let shuo = ss_shuo();
        let half = qr(SS_SUI, 2);
        let run = md(&(&zj + q(202050)), &shuo);
        let jing = &zj + q(SS_QI_YING) - &run + q(m) * &shuo;
        // 天正经朔入缩历 = 半岁周 − 闰余，累加朔策，满半岁周去之，盈缩互易
        let mut li_ys = &half - &run + q(m) * &shuo;
        let mut ying = false;
        while li_ys >= half {
            li_ys -= &half;
            ying = !ying;
        }
        let zz = md(&(&zj + q(130205) - &run + q(m) * &shuo), &q(SS_ZHUAN));
        let zhong = q(137773);
        let fast = zz < zhong;
        let li_cj = if fast { zz } else { zz - zhong };
        let ys = self.ying_suo(&li_ys, ying);
        let (cj, col) = self.chi_ji(&li_cj, fast);
        // 盈、迟为加，缩、疾为减；立成限行度列 ×100 即 820 ÷ 月每限行度
        let jjc = trunc_to(&((ys - cj) * col * q(100)), &qr(1, 100));
        let ding = &jing + jjc;
        (jing, ding)
    }
    /// 立成：月离记入转日序（立成按限，一日约 12.2 限），日躔记入恒气
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        let zj = q((y - 1281) as i64 * SS_SUI);
        let shuo = ss_shuo();
        let run = md(&(&zj + q(202050)), &shuo);
        let zz = md(&(&zj + q(130205) - &run + q(m) * &shuo), &q(SS_ZHUAN)) / q(SS_F);
        let t = (q(m) * &shuo - &run) / q(SS_F);
        Some(ss_biaoduan(&zz, &t))
    }
}

/// 明郑大统（1662—1683）：照《通轨》立成推步，定朔在夜子初（23 时）以后者置次日。
/// 依据：明郑颁历实物（Bodleian Sinica 58、88，剑桥 St John's S.14；永历二十五、三十、三十一年）36 月中，
/// 推步定朔 23:48、23:40 两月实物皆作次日，20:08 以前者不移；纪年页闰月注（1653、1675）限界于 22:43—23:10。
/// 节气交时全合北京子午线推步，故非台湾经度之差。见 doc/calendar/calendar-compendium.md「明郑颁历实物」。
struct TongguiZichu(Tonggui);

impl DingshuoLi for TongguiZichu {
    fn f(&self) -> i64 {
        self.0.f()
    }
    fn off(&self) -> i64 {
        self.0.off()
    }
    fn sui(&self) -> Q {
        self.0.sui()
    }
    fn dongzhi(&self, y: i32) -> Q {
        self.0.dongzhi(y)
    }
    fn month(&self, y: i32, m: i64) -> (Q, Q) {
        self.0.month(y, m)
    }
    fn jinshuo_limit(&self, _y: i32, _t: &Q) -> Option<Q> {
        Some(qr(23 * SS_F, 24))
    }
    fn biaoduan(&self, y: i32, m: i64) -> Option<Biaoduan> {
        self.0.biaoduan(y, m)
    }
}

fn shoushi() -> Shoushi {
    // 《明史》称《大统历法通轨》所载为授时续定之数：闰应 202050、转应 130205（与《元史》历经 201850、131904 异），授时亦用之
    Shoushi { run_ying: 202050, zhuan_ying: 130205, off: align_offset(SS_QI_YING / SS_F, gregorian::civil_to_jdn(1280, 12, 14)) }
}

// ---------------------------------------------------------------- 按历名取

/// 主序列定朔诸历：(历名（同 `main_series_calendar_periods.csv`）, 推步适用的农历年段)。
const REGISTRY: &[(&str, &[(i32, i32)])] = &[
    ("戊寅元历", &[(627, 645)]),
    ("戊寅元历（平朔）", &[(646, 665)]),
    ("麟德历", &[(666, 728)]),
    ("大衍历", &[(729, 762)]),
    ("五纪历", &[(763, 783)]),
    ("正元历", &[(784, 806)]),
    ("宣明历", &[(822, 892)]),
    ("崇玄历", &[(893, 938), (944, 955)]),
    // 五代实行：「當時歲曆，猶用《宣明》《崇元》二法，參而成之」（《旧五代史·历志》）；马重绩「以《宣明》氣朔，《崇元》星緯」。
    // 893—956 年本纪明书朔日 368 条：宣明推步合 360、崇玄术文 344。见 doc/calendar/calendar-compendium.md「崇玄」。
    ("崇玄历（宣明气朔）", &[(893, 938), (944, 956)]),
    ("调元历（宣明气朔）", &[(939, 943)]),
    ("崇天历", &[(1024, 1064), (1068, 1074)]),
    ("明天历", &[(1065, 1067)]),
    ("观天历", &[(1094, 1102)]),
    ("纪元历", &[(1106, 1127), (1133, 1135), (1151, 1166)]),
    ("统元历", &[(1136, 1150), (1167, 1167)]),
    ("乾道历", &[(1168, 1176)]),
    ("淳熙历", &[(1177, 1190)]),
    ("会元历", &[(1191, 1198)]),
    ("统天历", &[(1199, 1207)]),
    ("开禧历", &[(1208, 1251)]),
    ("成天历", &[(1271, 1276)]),
    ("授时历", &[(1281, 1367)]),
    ("大统历", &[(1368, 1644), (1666, 1668)]),
];

/// 主序列以外分政权所用之定朔历（不入 [REGISTRY]，只经 [reckon_year_any] 取）：(历名, 排月年段)。
const REGIME_REGISTRY: &[(&str, &[(i32, i32)])] = &[
    // 辽 947—1087：大同元年得后晋调元历（马重绩「以《宣明》氣朔」），统和十二年（994）贾俊大明历，术皆不传，以宣明代推。
    // 《辽史·历象志下》「朔考」序：宋元丰元年诏司天监考辽历，「遼己未歲（1079）氣朔與《宣明曆》合」。1088 年起本纪闰位合宋历、
    // 不合宣明（日本宣明历同年闰位全合推步），辽后期另有所改，术不传，不推。细则同宣明（三大三小、有交应见亏初不进）；
    // 五代 937、938 正月有交退日与例外表皆不及。见 doc/calendar/calendar-compendium.md「辽」。
    ("宣明历（辽）", &[(947, 1087)]),
    // 术不传之段的代推候选（2026-11-21，`examples/lost_proxy_eval.rs` 以本纪、实物检验择用）。
    // 观象历（807—821）：《新唐书·历志》「無蔀章之數。至於察斂啟閉之候，循用舊法」，《旧唐书·历志》「察斂啟閉之期，何殊舊法」——
    // 旧法即正元；宣明「氣朔……皆因大衍舊術」，同属大衍一系，并作候选。
    // 武德术不传（619—626）：志载为贞观初崔善为所较「復用上元積算」之术，日法、岁实、朔实同，代以此术推，与张表持平；
    // 南渡初 1128—1132 历术失载，代以纪元历推，1129 年四月本纪不合，不取（存作检验）
    ("戊寅元历（代推武德）", &[(619, 626)]),
    ("纪元历（代推南渡初）", &[(1128, 1132)]),
    ("正元历（代推观象）", &[(807, 821)]),
    ("宣明历（代推观象）", &[(807, 821)]),
    // 张表观象段（即刘羲叟《长历》）比正元定朔系统偏晚，试以宣明进朔、平四分之三进朔加于正元定朔：本纪明书朔 103、105，皆不及（存作检验）
    ("正元历（宣明进朔，代推观象）", &[(807, 821)]),
    ("正元历（四分之三进朔，代推观象）", &[(807, 821)]),
    // 会天历（1253—1270）：术不传；前开禧、后成天，并作候选，另有 1256 年实物历日可验。
    ("开禧历（代推会天）", &[(1253, 1270)]),
    ("成天历（代推会天）", &[(1253, 1270)]),
    // 奉元历（1075—1093）：术不传（《宋史·律历志》「惟奉元、會天二法不存」）；前崇天、后观天并作候选。
    ("崇天历（代推奉元）", &[(1075, 1093)]),
    ("观天历（代推奉元）", &[(1075, 1093)]),
    // 《古今图书集成·历法典》第七十六卷：奉元「視《明天曆》數減二刻」而成，明天亦作候选。
    ("明天历（代推奉元）", &[(1075, 1093)]),
    // 李锐《补修宋奉元术》补出步气朔常数（经朔、恒气），定朔改正与进朔借观天（[FengyuanBuxiu]）：七闰全合宋历，主序列取此
    ("奉元历（李锐补修气朔，借观天日月改正）", &[(1075, 1093)]),
    // 占天历（1103—1105）：术不传；前观天、后纪元并作候选。
    ("观天历（代推占天）", &[(1103, 1105)]),
    ("纪元历（代推占天）", &[(1103, 1105)]),
    // 淳祐历（1252）：陈垓言十一年历「依《開禧》新曆推算」，淳祐新历立春差六刻（《宋史》卷八十二）；以开禧代推。
    ("开禧历（代推淳祐）", &[(1252, 1252)]),
    // 本天历（1277—1279）：益王、卫王海上所作，「今亡」（《宋史》卷八十二）；以成天代推。
    ("成天历（代推本天）", &[(1277, 1279)]),
    // 钦天历（957—963）：术文在《新五代史》卷五十八，日躔、月离损益率表不存；前此气朔用宣明，作代推候选。
    ("宣明历（代推钦天）", &[(957, 963)]),
];

/// 「使不過三大二小」
/// 唐大衍一系「使不過三大三小」（大衍术注原文；钱宝琮校作「三大二小」）。取原文：二小之读使宣明（含五代宣明气朔）强进
/// 约二十余月，大衍本纪 61 不变、五纪 48→46、宣明 302→300、五代 332→329，张表同日亦减；三小、只限三大两读结果全同
/// （此期无连四小）。
const TANG_RUNS: Rules = Rules { runs: Some((3, 3)), pin_san: false, zhengyue: None, jiao_bujin: None, zhengyue_jiao: &[] };
const SAN_DA_ER_XIAO: Rules = Rules { runs: Some((3, 2)), pin_san: false, zhengyue: None, jiao_bujin: None, zhengyue_jiao: &[] };

/// 排月选项：交会诸规则（「有交不进」「正月朔有交」）与五代定闰所用中气（[WudaiDingqi]）。缺省 [PaiyueOptions::default]
/// 依本纪检验而定，见 `doc/calendar/calendar-compendium.md`「交会与进朔」「唐宋诸历本纪不合之考证」。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PaiyueOptions {
    /// 宣明（含五代宣明气朔）「或有交，應見虧初，則否」；None 不用
    pub xuanming: Option<JiaoBujin>,
    /// 宋诸历「有食應見者，其朔不進」；None 不用
    pub song: Option<JiaoBujin>,
    /// 大衍术注「正月朔有交、加時正見者……令虧在晦、二」
    pub zhengyue: ZhengyueJiao,
    /// 五代「崇玄历（宣明气朔）」定闰之中气
    pub wudai_dingqi: WudaiDingqi,
    /// 大衍一系（大衍、五纪、正元、宣明及其代推）「朔望有交者」日躔、月离用二次差精法（术文）；false 则一律简法（2026-11-25 前做法）
    pub dayan_jingfa: bool,
    /// 麟德步交会读法（只关交会检验，不入排月）
    pub linde_jiaohui: LindeJiaohui,
    /// 麟德常月进朔（四分之三）起始之冬至岁：缺省 Some(709)，据 709—726 年盟文、哀册、译经题记与本纪，705—706 年三条不进
    /// （见 [Linde::jinshuo_limit]）；Some(710) 为 709 年盟文未入前之做法；
    /// None 为常月不进（2026-11-26 前做法）
    pub linde_changyue: Option<i32>,
    /// 麟德常月停进朔之区间（民用历日 [起, 止)）：缺省 Some(((717, 2, 1), (720, 2, 1)))，即开元五年正月至七年十二月——
    /// 717 年四月、十二月，718 年五月、十二月，719 年四月当时文字（祭文、塔铭、哀册、神道碑、墓志）皆不进，前后 716 年闰十二月、
    /// 721 年正月墓志皆进；719 年五月至 720 年无文，止于开元七年末为本项目所定。None 为不停（2026-11-28 前做法，三月入例外表）
    pub linde_tingjin: Option<((i32, i32, i32), (i32, i32, i32))>,
    /// 麟德本术（定小余近夜半者覆算）十四日、二十八日「分為二日，隨餘初末，如法求之」（《旧唐书·历志二》，读法同大衍一系精法，
    /// 见 [Linde::chisu_fen]）；false 则此二日仍用速要（2026-11-26 前做法）
    pub linde_fen_erri: bool,
    /// 大衍历本历时期（729—757）进朔限（[DayanJinshuo]）；758 年起至德历皆戌初 19 时
    pub dayan_jinshuo: DayanJinshuo,
}

/// 大衍历（729—757）进朔限。大衍历志无进朔明文，唯《新唐书·历志五》述五纪历「依大衍戌初進初朔」。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DayanJinshuo {
    /// 戌初 19 时（据《新唐书》卷二十九「依大衍戌初進初朔」）：当时册文 737 年八月、738 年七月与闰八月、742 年五月（定朔 20.75—21.12 时，
    /// 皆在晨初限下）进朔皆合；与之相违者 729 年七月（宋人碑目转录一件，23.89 时不进）、750 年正月（本纪，19.98 时不进）列史载例外表
    Xuchu,
    /// 随季节之晨初限（同五纪「晨初餘數減十刻」，大衍步轨漏；2026-11-13 至 11-28 做法，据 750 年正月本纪）：上举当时册文四月皆须列例外
    Chenchu,
}

/// 五代宣明气朔诸段定闰所用之中气。《旧五代史·历志》：「當時歲曆，猶用《宣明》《崇元》二法，參而成之」；后晋马重绩调元历
/// 「以《宣明》氣朔，《崇元》星緯」。904 年（天祐元年）本纪、新唐书、旧五代史、《通鉴》（「夏四月辛巳……閏月丁酉」）皆闰四月，
/// 宣明恒气得闰三月（日本行宣明同），崇玄恒气得闰四月：朔用宣明定朔、中气用崇玄恒气，即「參而成之」。
/// 各段之冬至岁见 [WUDAI_CHONGXUAN_QI]、[WUDAI_CHONGXUAN_QI_ALL]；辽一路「宣明历（辽）」不在此列。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WudaiDingqi {
    /// 全用宣明恒气（旧做法，2026-11-18 前）
    Xuanming,
    /// 崇玄行用期 893—938 与复用期 944—956 用崇玄恒气，调元段 939—943 照马重绩「宣明氣朔」用宣明
    Chongxuan,
    /// 893—956 全程用崇玄恒气（调元段亦然），只供对照
    ChongxuanAll,
}

/// 五代崇玄行用期（893—938）与复用期（944—956）之冬至岁
pub const WUDAI_CHONGXUAN_QI: &[(i32, i32)] = &[(893, 938), (944, 956)];
/// 五代 893—956 全程之冬至岁
pub const WUDAI_CHONGXUAN_QI_ALL: &[(i32, i32)] = &[(893, 956)];

/// 「正月朔有交」之适用范围
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZhengyueJiao {
    /// 不用
    Off,
    /// 只在史载有据之年（[TANG_ZHENGYUE_JIAO]、[WUDAI_ZHENGYUE_JIAO]）
    Shizai,
    /// 大衍一系全程（唐 +1 进令亏在晦，五代宣明气朔 −1 退令亏在二日）：本纪 781、789、790、874、892、901、911 年元日有交昼见而未移，
    /// 全程用之与本纪不合，只供对照
    All,
}

impl Default for PaiyueOptions {
    fn default() -> Self {
        PaiyueOptions {
            // 阴历之蚀、亏初（食甚减 80 分钟）不晚于日入：宣明本纪不变，五代宣明气朔张表同日 +3；夜食亦不进则本纪 −5
            xuanming: Some(JiaoBujin { yin: true, wan_min: Some(80) }),
            // 有食（崇天照志文日蚀术，余以入食限代之）、亏初（食甚减定用）不晚于日入；不限阴阳历
            song: Some(JiaoBujin { yin: false, wan_min: Some(0) }),
            zhengyue: ZhengyueJiao::Shizai,
            wudai_dingqi: WudaiDingqi::Chongxuan,
            dayan_jingfa: true,
            linde_jiaohui: LindeJiaohui::Xinzhi,
            linde_changyue: Some(709),
            linde_tingjin: Some(((717, 2, 1), (720, 2, 1))),
            linde_fen_erri: true,
            dayan_jinshuo: DayanJinshuo::Xuchu,
        }
    }
}

/// 史载正月朔有交移日之年（唐）：780 年（大历十五年）元日丙寅 15.46 时有交昼见，天文志「十二月丙寅晦，日有食之」，正月作丁卯
pub const TANG_ZHENGYUE_JIAO: &[(i32, i32, i64)] = &[(780, 780, 1)];
/// 史载正月朔有交移日之年（后晋）：937、938 年元日食，司天奏「正月二日」食（《五代会要》卷十），正月退一日
pub const WUDAI_ZHENGYUE_JIAO: &[(i32, i32, i64)] = &[(937, 938, -1)];

/// 历名 → 连同排月细则的历（按交会选项构造，宋诸历读表）。
fn build_paiyue(name: &str, o: &PaiyueOptions) -> Option<Paiyue> {
    let none = Rules::default();
    let song_r = Rules { jiao_bujin: o.song, ..SAN_DA_ER_XIAO };
    let zy = |shizai: &'static [(i32, i32, i64)], all: &'static [(i32, i32, i64)]| match o.zhengyue {
        ZhengyueJiao::Off => &[][..],
        ZhengyueJiao::Shizai => shizai,
        ZhengyueJiao::All => all,
    };
    // 大衍一系有交之月用精法
    let jf = |mut d: Dayan| {
        d.jingfa = o.dayan_jingfa;
        d
    };
    Some(match name {
        "戊寅元历" => Paiyue::new(wuyin(false), &["戊寅元历"], none),
        // 崇玄（《新唐书·历志六下》卷三十下）：「氣朔……定朔弦望……皆大衍之舊」，「盈縮、朓朒，皆用常氣」——求定朔同大衍（日躔入常气），
        // 有交之月亦用大衍二次差精法（[Dayan::sun_def_jing] 于恒气等长，[MoonTable::eval_jing]）；术文进朔，无排月细则
        "崇玄历" => Paiyue::new(jf(chongxuan()), &["崇玄历"], none),
        "戊寅元历（代推武德）" => Paiyue::new(wuyin(false), &["戊寅元历（代推武德）"], none),
        "纪元历（代推南渡初）" => Paiyue::new(song("纪元", 7290, 2662626, q(215278), qr(2008730990, 10000), 28613466, 1106, 15, false, None, (1105, 12, 16)), &["纪元历（代推南渡初）"], song_r),
        "麟德历" => Paiyue::new(Linde { jiaohui: o.linde_jiaohui, changyue_jin: o.linde_changyue, tingjin: o.linde_tingjin.map(|((y0, m0, d0), (y1, m1, d1))| (gregorian::civil_to_jdn(y0, m0, d0), gregorian::civil_to_jdn(y1, m1, d1))), fen_erri: o.linde_fen_erri, ..linde() }, &["麟德历"], Rules { runs: None, pin_san: true, zhengyue: Some((684, 19)), jiao_bujin: None, zhengyue_jiao: &[] }),
        "大衍历" => Paiyue::new(
            jf(Dayan {
                jinshuo: match o.dayan_jinshuo {
                    DayanJinshuo::Xuchu => vec![(0, Jinshuo::Hours(19))],
                    DayanJinshuo::Chenchu => vec![(0, Jinshuo::Lou), (758, Jinshuo::Hours(19))],
                },
                ..dayan()
            }),
            &["大衍历"],
            Rules { zhengyue_jiao: zy(&[], &[(729, 762, 1)]), ..TANG_RUNS },
        ),
        "五纪历" => Paiyue::new(jf(wuji()), &["五纪历"], Rules { zhengyue_jiao: zy(TANG_ZHENGYUE_JIAO, &[(762, 783, 1)]), ..TANG_RUNS }),
        "正元历" => Paiyue::new(jf(zhengyuan()), &["正元历"], Rules { zhengyue_jiao: zy(&[], &[(784, 806, 1)]), ..TANG_RUNS }),
        "宣明历" | "崇玄历（宣明气朔）" | "调元历（宣明气朔）" => Paiyue::new(
            jf(xuanming()),
            &["宣明历", "崇玄历（宣明气朔）", "调元历（宣明气朔）"],
            Rules { jiao_bujin: o.xuanming, zhengyue_jiao: zy(WUDAI_ZHENGYUE_JIAO, &[(822, 906, 1), (907, 956, -1)]), ..TANG_RUNS },
        )
        .with_zhongqi(
            chongxuan(),
            match o.wudai_dingqi {
                WudaiDingqi::Xuanming => &[],
                WudaiDingqi::Chongxuan => WUDAI_CHONGXUAN_QI,
                WudaiDingqi::ChongxuanAll => WUDAI_CHONGXUAN_QI_ALL,
            },
        ),
        "宣明历（辽）" => Paiyue::new(jf(xuanming()), &["宣明历（辽）"], Rules { jiao_bujin: o.xuanming, ..TANG_RUNS }),
        "正元历（代推观象）" => Paiyue::new(jf(zhengyuan()), &["正元历（代推观象）"], Rules { zhengyue_jiao: zy(&[], &[(807, 821, 1)]), ..TANG_RUNS }),
        "宣明历（代推观象）" => Paiyue::new(jf(xuanming()), &["宣明历（代推观象）"], Rules { jiao_bujin: o.xuanming, ..TANG_RUNS }),
        "正元历（宣明进朔，代推观象）" => Paiyue::new(
            jf(Dayan { jinshuo: vec![(0, Jinshuo::Xuanming)], ..zhengyuan() }),
            &["正元历（宣明进朔，代推观象）"],
            Rules { zhengyue_jiao: zy(&[], &[(807, 821, 1)]), ..TANG_RUNS },
        ),
        "正元历（四分之三进朔，代推观象）" => Paiyue::new(
            jf(Dayan { jinshuo: vec![(0, Jinshuo::Hours(18))], ..zhengyuan() }),
            &["正元历（四分之三进朔，代推观象）"],
            Rules { zhengyue_jiao: zy(&[], &[(807, 821, 1)]), ..TANG_RUNS },
        ),
        "宣明历（代推钦天）" => Paiyue::new(jf(xuanming()), &["宣明历（代推钦天）"], Rules { jiao_bujin: o.xuanming, ..TANG_RUNS }),
        "开禧历（代推会天）" => Paiyue::new(
            song("开禧", 16900, 6172608, q(499067), qr(4656725396, 10000), 7848183, 1207, 0, false, None, (1206, 12, 15)),
            &["开禧历（代推会天）"],
            song_r,
        ),
        "崇天历（代推奉元）" => Paiyue::new(song("崇天", 10590, 3867940, q(312729), qr(2918030594, 10000), 97556340, 1024, 0, true, None, (1023, 12, 18)), &["崇天历（代推奉元）"], song_r),
        "观天历（代推奉元）" => Paiyue::new(song("观天", 12030, 4393880, q(355253), qr(3314820389, 10000), 5944808, 1092, 0, false, None, (1091, 12, 16)), &["观天历（代推奉元）"], song_r),
        "奉元历（李锐补修气朔，借观天日月改正）" => Paiyue::new(FengyuanBuxiu::new(), &["奉元历（李锐补修气朔，借观天日月改正）"], song_r),
        "观天历（代推占天）" => Paiyue::new(song("观天", 12030, 4393880, q(355253), qr(3314820389, 10000), 5944808, 1092, 0, false, None, (1091, 12, 16)), &["观天历（代推占天）"], song_r),
        "纪元历（代推占天）" => Paiyue::new(song("纪元", 7290, 2662626, q(215278), qr(2008730990, 10000), 28613466, 1106, 15, false, None, (1105, 12, 16)), &["纪元历（代推占天）"], song_r),
        "开禧历（代推淳祐）" => Paiyue::new(song("开禧", 16900, 6172608, q(499067), qr(4656725396, 10000), 7848183, 1207, 0, false, None, (1206, 12, 15)), &["开禧历（代推淳祐）"], song_r),
        "成天历（代推本天）" => Paiyue::new(song("成天", 7420, 2710101, q(219117), qr(2044551641, 10000), 71758147, 1271, 0, false, None, (1270, 12, 14)), &["成天历（代推本天）"], song_r),
        "成天历（代推会天）" => Paiyue::new(
            song("成天", 7420, 2710101, q(219117), qr(2044551641, 10000), 71758147, 1271, 0, false, None, (1270, 12, 14)),
            &["成天历（代推会天）"],
            song_r,
        ),
        "崇天历" => Paiyue::new(song("崇天", 10590, 3867940, q(312729), qr(2918030594, 10000), 97556340, 1024, 0, true, None, (1023, 12, 18)), &["崇天历"], song_r),
        "明天历" => {
            let dz = Mingtian::tong(1064).div_euclid(MT_F);
            Paiyue::new(Mingtian { off: align_offset(dz, gregorian::civil_to_jdn(1063, 12, 16)) }, &["明天历"], song_r)
        }
        "明天历（代推奉元）" => {
            let dz = Mingtian::tong(1064).div_euclid(MT_F);
            Paiyue::new(Mingtian { off: align_offset(dz, gregorian::civil_to_jdn(1063, 12, 16)) }, &["明天历（代推奉元）"], song_r)
        }
        "观天历" => Paiyue::new(song("观天", 12030, 4393880, q(355253), qr(3314820389, 10000), 5944808, 1092, 0, false, None, (1091, 12, 16)), &["观天历"], song_r),
        "纪元历" => Paiyue::new(song("纪元", 7290, 2662626, q(215278), qr(2008730990, 10000), 28613466, 1106, 15, false, None, (1105, 12, 16)), &["纪元历"], song_r),
        "统元历" => Paiyue::new(song("统元", 6930, 2531138, q(204647), qr(1909532563, 10000), 94251591, 1135, 0, false, None, (1134, 12, 15)), &["统元历"], song_r),
        "乾道历" => Paiyue::new(song("乾道", 30000, 10957308, qr(88591776, 100), qr(8266377395, 10000), 91645823, 1167, 0, false, None, (1166, 12, 15)), &["乾道历"], song_r),
        "淳熙历" => Paiyue::new(song("淳熙", 5640, 2059974, qr(16655256, 100), qr(1554079740, 10000), 52421972, 1176, 0, false, None, (1175, 12, 15)), &["淳熙历"], song_r),
        "会元历" => Paiyue::new(song("会元", 38700, 14134932, q(1142834), qr(10663617310, 10000), 25494767, 1191, 0, false, None, (1190, 12, 15)), &["会元历"], song_r),
        "统天历" => Paiyue::new(
            song("统天", 12000, 4382910, q(354368), q(330655), 3835, 1199, 0, false, Some(SongCha { ju: 3830, qi: 237811, run: 21704, zhuan: 188800 }), (1198, 12, 15)),
            &["统天历"],
            song_r,
        ),
        "开禧历" => Paiyue::new(song("开禧", 16900, 6172608, q(499067), qr(4656725396, 10000), 7848183, 1207, 0, false, None, (1206, 12, 15)), &["开禧历"], song_r),
        "成天历" => Paiyue::new(song("成天", 7420, 2710101, q(219117), qr(2044551641, 10000), 71758147, 1271, 0, false, None, (1270, 12, 14)), &["成天历"], song_r),
        _ => return None,
    })
}

/// 历名 + 交会选项 → 排月之历（惰性构造并缓存；宣明三名共一历）
fn paiyue_with(name: &str, o: &PaiyueOptions) -> Option<&'static Paiyue> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static C: OnceLock<Mutex<HashMap<(&'static str, PaiyueOptions), &'static Paiyue>>> = OnceLock::new();
    let key = match name {
        "崇玄历（宣明气朔）" | "调元历（宣明气朔）" => "宣明历",
        _ => name,
    };
    let key: &'static str = REGISTRY.iter().chain(REGIME_REGISTRY).map(|(n, _)| *n).find(|n| *n == key)?;
    let mut m = C.get_or_init(Default::default).lock().unwrap();
    if let Some(p) = m.get(&(key, *o)) {
        return Some(p);
    }
    let p: &'static Paiyue = Box::leak(Box::new(build_paiyue(key, o)?));
    m.insert((key, *o), p);
    Some(p)
}

/// 历名 → 连同排月细则的历（缺省交会选项）。
fn paiyue_by_name(name: &str) -> Option<&'static Paiyue> {
    paiyue_with(name, &PaiyueOptions::default())
}

/// 历名 → 历（惰性构造，宋诸历读表）。
pub fn li_by_name(name: &str) -> Option<&'static dyn DingshuoLi> {
    if let Some(p) = paiyue_by_name(name) {
        return Some(p);
    }
    use std::sync::OnceLock;
    macro_rules! once {
        ($e:expr) => {{
            static C: OnceLock<Box<dyn DingshuoLi>> = OnceLock::new();
            Some(C.get_or_init(|| Box::new($e)).as_ref())
        }};
    }
    match name {
        "戊寅元历（平朔）" => once!(wuyin(true)),
        "授时历" => once!(shoushi()),
        "大统历" => once!(Tonggui::new()),
        "大统历（明郑子初进朔）" => once!(TongguiZichu(Tonggui::new())),
        _ => None,
    }
}

/// 按历名推某农历年各月；该年不在此历推步适用的年段内（如崇玄历 939—943 年）返回 None。
pub fn reckon_year(name: &str, lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    let (_, spans) = REGISTRY.iter().find(|(n, _)| *n == name)?;
    if !spans.iter().any(|&(a, b)| (a..=b).contains(&lunar_year)) {
        return None;
    }
    Some(year_months(li_by_name(name)?, lunar_year))
}

/// 按历名推某农历年各月，只按术文定朔、进朔（旧做法：不加「三大二小」、麟德正月进朔与史载例外；麟德「不過頻三」照旧），
/// 供与 [reckon_year] 对照。不在 [paiyue_by_name] 诸历者同 [reckon_year]。
pub fn reckon_year_shuwen(name: &str, lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    let (_, spans) = REGISTRY.iter().find(|(n, _)| *n == name)?;
    if !spans.iter().any(|&(a, b)| (a..=b).contains(&lunar_year)) {
        return None;
    }
    match paiyue_by_name(name) {
        Some(p) => Some(year_months(&Shuwen(p), lunar_year)),
        None => Some(year_months(li_by_name(name)?, lunar_year)),
    }
}

/// 按历名推某农历年各月，交会诸规则按 `opts`（供检验各选项；缺省同 [reckon_year]）。不在 [paiyue_by_name] 诸历者同 [reckon_year]。
pub fn reckon_year_with(name: &str, lunar_year: i32, opts: &PaiyueOptions) -> Option<Vec<(i64, u8, bool)>> {
    let (_, spans) = REGISTRY.iter().find(|(n, _)| *n == name)?;
    if !spans.iter().any(|&(a, b)| (a..=b).contains(&lunar_year)) {
        return None;
    }
    match paiyue_with(name, opts) {
        Some(p) => Some(year_months(p, lunar_year)),
        None => Some(year_months(li_by_name(name)?, lunar_year)),
    }
}

/// 按历名推某农历年各月，不限主序列行用年段：供分政权历日（如南明、明郑续用大统历，1645—1683）。
pub fn reckon_year_any(name: &str, lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    Some(year_months(li_by_name(name)?, lunar_year))
}

/// 一月之定朔与入交（供检验，见 `examples/jiaohui_eclipse_check.rs`）
#[derive(Clone, Debug)]
pub struct JiaoMonth {
    /// 定朔所在日（JDN）
    pub ding_jdn: i64,
    /// 定小余（占一日之比）
    pub frac: f64,
    /// 定小余过进朔限
    pub jin: bool,
    /// 排月所得朔日（JDN；细则全用）
    pub day: i64,
    /// 入交
    pub jiao: Option<Jiaohui>,
    /// 日出（占一日之比）
    pub sunrise: Option<f64>,
    /// 加时在昼
    pub zhou: bool,
    /// 本历日蚀术所判
    pub rishi: Option<Rishi>,
}

/// 按历名列其推步年段（前后各二岁）各月之定朔与入交；本历未复原步交会者 `jiao` 为 None。
pub fn jiao_months(name: &str) -> Option<Vec<JiaoMonth>> {
    jiao_months_with(name, &PaiyueOptions::default())
}

/// [jiao_months]，按所给排月选项（如麟德步交会读法 [PaiyueOptions::linde_jiaohui]）
pub fn jiao_months_with(name: &str, o: &PaiyueOptions) -> Option<Vec<JiaoMonth>> {
    let li: &dyn DingshuoLi = match paiyue_with(name, o) {
        Some(p) => p,
        None => li_by_name(name)?,
    };
    let paiyue_by_name = |n: &str| paiyue_with(n, o);
    let names: Vec<&str> = match paiyue_by_name(name) {
        Some(p) => p.names.to_vec(),
        None => vec![name],
    };
    let spans = REGISTRY.iter().chain(REGIME_REGISTRY).filter(|(n, _)| names.contains(n)).flat_map(|(_, s)| s.iter());
    let (a, b) = spans.fold((i32::MAX, i32::MIN), |(a, b), &(x, y)| (a.min(x), b.max(y)));
    let (y0, y1) = (a - 2, b + 2);
    let raw = raw_months(li, y0, y1);
    let days: Vec<i64> = match paiyue_by_name(name) {
        Some(p) if p.span() == (y0, y1) => p.days(true).0.clone(),
        _ => raw.iter().map(|m| m.base + i64::from(m.jin)).collect(),
    };
    Some(
        raw.iter()
            .zip(days)
            .map(|(m, day)| JiaoMonth {
                ding_jdn: m.base,
                frac: m.frac.to_f64().unwrap(),
                jin: m.jin,
                day,
                jiao: m.jiao.clone(),
                sunrise: m.sunrise.as_ref().map(|x| x.to_f64().unwrap()),
                zhou: m.zhou(),
                rishi: m.rishi.clone(),
            })
            .collect(),
    )
}

/// 一月之经朔、定朔与进朔限（供逐月考证，见 `examples/dingshuo_zhangbiao_diffs.rs`）
#[derive(Clone, Debug)]
pub struct DingshuoMonth {
    /// 所属冬至岁
    pub dongzhi_year: i32,
    /// 冬至岁内自天正经朔起之月序
    pub index: i64,
    /// 经朔所在日（JDN）与小余（占一日之比）
    pub jing_jdn: i64,
    pub jing_frac: f64,
    /// 注历所用定朔所在日（JDN）与定小余（占一日之比）
    pub ding_jdn: i64,
    pub frac: f64,
    /// 本历进朔限（占一日之比）；None 为术文不进朔（麟德常月等）
    pub limit: Option<f64>,
    /// 定小余过进朔限
    pub jin: bool,
    /// 排月所得朔日（JDN；细则全用，同 [JiaoMonth::day]）
    pub day: i64,
    /// 经朔入交入食限（「朔望有交」；大衍一系此等月定朔用精法）；None 为本历未复原步交会
    pub youjiao: Option<bool>,
    /// 经朔所入日躔、月离表段（[DingshuoLi::biaoduan]）
    pub biaoduan: Option<Biaoduan>,
}

/// 按历名列其推步年段（前后各二岁）各月之经朔、定朔、进朔限与排月所得朔日。
pub fn dingshuo_months(name: &str) -> Option<Vec<DingshuoMonth>> {
    let li = li_by_name(name)?;
    let jm = jiao_months(name)?;
    let names: Vec<&str> = match paiyue_by_name(name) {
        Some(p) => p.names.to_vec(),
        None => vec![name],
    };
    let spans = REGISTRY.iter().chain(REGIME_REGISTRY).filter(|(n, _)| names.contains(n)).flat_map(|(_, s)| s.iter());
    let (a, b) = spans.fold((i32::MAX, i32::MIN), |(a, b), &(x, y)| (a.min(x), b.max(y)));
    let (y0, y1) = (a - 2, b + 2);
    let off = li.off();
    let f = q(li.f());
    // 与 [raw_months] 同序：每个经朔只由其所在冬至岁算一次，按定朔所在日排序去重
    let mut out = Vec::new();
    for yy in y0..=y1 {
        let next0 = li.month(yy + 1, 0).0;
        for m in 0..15 {
            let (jing, _) = li.month(yy, m);
            if jing >= next0 {
                break;
            }
            let ding = li.ding_for_calendar(yy, m);
            let (day, jin) = day_jin(li, yy, &ding);
            let jf = md(&jing, &f);
            let lim = li.jinshuo_limit(yy, &ding).map(|l| (l / &f).to_f64().unwrap());
            out.push(DingshuoMonth {
                dongzhi_year: yy,
                index: m,
                jing_jdn: fl(&(&jing / &f)) + off,
                jing_frac: (jf / &f).to_f64().unwrap(),
                ding_jdn: day + off,
                frac: (md(&ding, &f) / &f).to_f64().unwrap(),
                limit: lim,
                jin,
                day: 0,
                youjiao: None,
                biaoduan: li.biaoduan(yy, m),
            });
        }
    }
    out.sort_by_key(|x| x.ding_jdn);
    out.dedup_by_key(|x| x.ding_jdn);
    assert_eq!(out.len(), jm.len(), "{name}：与 jiao_months 月数不同");
    for (o, j) in out.iter_mut().zip(&jm) {
        assert_eq!(o.ding_jdn, j.ding_jdn);
        o.day = j.day;
        o.youjiao = j.jiao.as_ref().map(|x| x.shixian);
    }
    Some(out)
}

/// 已移植的定朔历名与适用年段（供检验与文档）。
pub fn registry() -> &'static [(&'static str, &'static [(i32, i32)])] {
    REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 宋诸历步晷漏：冬至日出 0.3 日、夏至 0.2 日（崇天、观天、明天按志文常数精确成立；观天从陈美东校「二千一百五少」），
    /// 春分前后连续；纪元历由赤道内外度求得，二至约 0.3/0.2 日、二分 0.25 日。
    /// 大衍步轨漏逐日递推：气初日即表值；雨水按「三日為限」逐限日损 12、8、3、2、1，十五日陟降率之和 372，
    /// 恰为雨水至惊蛰消息衰之差（35.78 → 39.50），读法得证；推满一气，夜半漏与次气初表值相差不过 30 分（象积 480 分为刻）。
    #[test]
    fn dayan_night_lou_recurrence() {
        assert_eq!(dayan_night_lou(4, 0), dayan_lou(4));
        // 雨水 15 日后之衰：直接复算
        let mut shuai = DY_SHUAI[4];
        let mut rate = 78;
        for j in 0..15 {
            if j >= 1 {
                rate += DY_XIAN[0].1[((j - 1) / 3) as usize];
            }
            shuai += rate;
        }
        assert_eq!(shuai, DY_SHUAI[5]);
        for (kk, n) in [(0usize, 14i64), (4, 15), (10, 16), (16, 15), (22, 15)] {
            let next = dayan_lou(if kk + 1 <= 12 { kk + 1 } else { 23 - kk });
            let diff = (dayan_night_lou(kk, n) - next).abs();
            assert!(diff < qr(30, 480), "{kk}");
        }
    }

    #[test]
    fn song_sunrise_models() {
        for lou in [chongtian_lou(), guantian_lou(), mingtian_lou()] {
            let l = lou.l.clone();
            assert_eq!(lou.sunrise(&q(0)), qr(3, 10), "冬至");
            assert_eq!(lou.sunrise(&l), qr(1, 5), "夏至");
            let half = &l / q(2);
            let eps = qr(1, 1_000_000);
            let (a, b) = (lou.sunrise(&(&half - &eps)), lou.sunrise(&(&half + &eps)));
            assert!((a - b).abs() < qr(1, 100_000), "春分连续");
        }
        let near = |x: Q, v: f64| {
            let got = x.to_f64().unwrap();
            assert!((got - v).abs() < 0.001, "{got} vs {v}");
        };
        let l = qr(182 * 7290 + 4533, 7290);
        near(jiyuan_sunrise(&q(0)), 0.3);
        near(jiyuan_sunrise(&l), 0.2);
        near(jiyuan_sunrise(&(&l / q(2))), 0.25);
    }

    /// 大衍一系一律用简法之历（Python 复原只有简法，供逐月比对）
    fn jianfa(name: &str) -> &'static dyn DingshuoLi {
        match paiyue_with(name, &PaiyueOptions { dayan_jingfa: false, ..PaiyueOptions::default() }) {
            Some(p) => p,
            None => li_by_name(name).unwrap(),
        }
    }

    /// 大衍日躔精法：张培瑜等《中国古代历法》书页 505—506 算例——七月平望入立秋 4.741255 日，气初定率 5.562231、日差 0.118994，
    /// 太阳改正朓 443.316583 分（书用《旧唐书》辰数 186.5、184.9，本项目用定气实长，差在十分之一分以内）；
    /// 各气初日精法、简法同为表值，气内二法之差不过数分
    #[test]
    fn dayan_sun_jingfa_example() {
        let d = dayan();
        let t = d.ding_qi(15) + qr(4741255, 1_000_000) * q(3040);
        let v = d.sun_def_jing(&t).to_f64().unwrap();
        assert!((v + 443.316583).abs() < 0.1, "{v}");
        for k in 0..24 {
            let t0 = d.ding_qi(k);
            assert_eq!(d.sun_def_jing(&t0), d.sun_def(&t0), "气 {k}");
            let mid = (d.ding_qi(k) + d.ding_qi(k + 1)) / q(2);
            let diff = (d.sun_def_jing(&mid) - d.sun_def(&mid)).abs().to_f64().unwrap();
            assert!(diff < 8.0, "气 {k}：{diff}");
        }
    }

    /// 大衍月离精法：转日 2（益 259，后日 220，前多、朒）入余半日，转率 I(½) = 259·½ + 39/2·¼ = 134.375，t₀ = 134.375/3040，
    /// 定率 = I(½ + t₀)；初末分段之日分为二日各照此法
    #[test]
    fn dayan_moon_jingfa() {
        let d = dayan();
        let mt = &d.moon[0];
        let x = q(3040) + q(1520);
        let i = |u: Q| q(259) * &u + qr(39, 2) * (&u - &u * &u);
        let t0 = i(qr(1, 2)) / q(3040);
        assert_eq!(mt.eval_jing(&x), q(297) + i(qr(1, 2) + t0));
        // 七日（初数 2701 初率 48，末数 339 末率 −6）「分為二日」：初段以前一日 97 折段长为邻率
        let x7 = q(6 * 3040) + q(100);
        let base = q(297 + 259 + 220 + 180 + 139 + 97);
        let nb = q(97) * q(2701) / q(3040);
        let c = -(q(48) - &nb) / q(2);
        let i7 = |u: &Q| q(48) * u + &c * (u - u * u);
        let u = qr(100, 2701);
        let t0 = i7(&u) / q(2701);
        assert_eq!(mt.eval_jing(&x7), &base + i7(&(&u + t0)));
        // 二十八日末段（末率 0，益入後）仍同简法
        let x28 = q(27 * 3040) + q(2000);
        assert_eq!(mt.eval_jing(&x28), mt.eval(&x28));
    }

    /// 移植与 Python 复原逐月相同：`data/calendar/dingshuo_reference.csv`（每历隔 3 岁，自天正经朔起 14 月的定朔分）
    #[test]
    fn matches_python_reference() {
        let csv = std::fs::read_to_string(crate::repo::repo_root().join("data/calendar/dingshuo_reference.csv")).unwrap();
        let alias = |n: &str| -> &'static str {
            match n {
                "戊寅定朔" => "戊寅元历",
                "戊寅平朔" => "戊寅元历（平朔）",
                "麟德" => "麟德历",
                "大衍" => "大衍历",
                "五纪" => "五纪历",
                "正元" => "正元历",
                "宣明" => "宣明历",
                "崇玄" => "崇玄历",
                "崇天" => "崇天历",
                "明天" => "明天历",
                "观天" => "观天历",
                "纪元" => "纪元历",
                "统元" => "统元历",
                "乾道" => "乾道历",
                "淳熙" => "淳熙历",
                "会元" => "会元历",
                "统天" => "统天历",
                "开禧" => "开禧历",
                "成天" => "成天历",
                "授时" => "授时历",
                "大统" => "大统历",
                _ => panic!("{n}"),
            }
        };
        let mut n = 0;
        for line in csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("历,")) {
            let c: Vec<&str> = line.split(',').collect();
            let li = jianfa(alias(c[0]));
            assert_eq!(li.f().to_string(), c[1], "{} 日法", c[0]);
            assert_eq!(li.off().to_string(), c[2], "{} 上元", c[0]);
            let (y, m): (i32, i64) = (c[3].parse().unwrap(), c[4].parse().unwrap());
            assert_eq!(li.month(y, m).1, parse_q(c[5]), "{} {y} 岁第 {m} 月", c[0]);
            n += 1;
        }
        assert!(n > 4000);
    }

    /// 史载例外表逐条落实于排月：该历该年该月（含闰）之朔即表中 JDN；并钉住各细则所解之月
    #[test]
    fn liwai_and_rules_applied() {
        let csv = include_str!("../../../../data/calendar/dingshuo_exceptions.csv");
        let mut n = 0;
        for line in csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("历,") && !l.trim().is_empty()) {
            let c: Vec<&str> = line.split(',').collect();
            let (y, m, leap, jdn): (i32, u8, bool, i64) = (c[1].parse().unwrap(), c[2].parse().unwrap(), c[3] == "1", c[4].parse().unwrap());
            // 代推之历（如「开禧历（代推会天）」）登记于 REGIME_REGISTRY，不在 reckon_year 之列
            // 宋初应天等与清初新法不在定朔排月之内，由 push_calendar::reckon_lifa_year 在推步结果上应用
            let months = reckon_year(c[0], y)
                .or_else(|| reckon_year_any(c[0], y))
                .or_else(|| crate::calendar::push_calendar::reckon_lifa_year(c[0], y))
                // 观象段古人长历（刘羲叟《长历》）的修正
                .or_else(|| (c[0] == crate::calendar::liu_changli::LIU_CHANGLI_NAME).then(|| crate::calendar::liu_changli::year_months(y)).flatten())
                // 分政权之修正：历名「政权：历名」
                .or_else(|| {
                    let (reg, li) = c[0].split_once('：')?;
                    crate::calendar::push_calendar::regime_year_months(reg, y).filter(|x| x.0 == li).map(|x| x.1)
                })
                .unwrap();
            assert!(months.contains(&(jdn, m, leap)), "{} {y} 年 {m} 月：{months:?}", c[0]);
            n += 1;
        }
        assert_eq!(n, liwai_table().len());
        let gz = |name: &str, y: i32, m: u8| {
            let j = reckon_year(name, y).unwrap().into_iter().find(|e| e.1 == m && !e.2).unwrap().0;
            crate::calendar::gan_zhi::GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
        };
        // 麟德正月进朔（本纪）
        assert_eq!(gz("麟德历", 684, 1), "甲申");
        assert_eq!(gz("麟德历", 711, 1), "丁未");
        assert_eq!(gz("麟德历", 712, 1), "辛未");
        // 麟德开元五至七年停常月进朔（当时文字）：717 年四月张说祭城隍文、苏颋悼王石塔铭，十二月李素等四志，718 年五月哀皇后哀册、
        // 十二月李氏墓志，719 年四月王仁皎神道碑；前后 716 年闰十二月、721 年正月墓志进
        assert_eq!(gz("麟德历", 717, 4), "庚午");
        assert_eq!(gz("麟德历", 717, 12), "丙寅");
        assert_eq!(gz("麟德历", 718, 5), "甲午");
        assert_eq!(gz("麟德历", 718, 12), "庚申");
        assert_eq!(gz("麟德历", 719, 4), "己未");
        assert_eq!(gz("麟德历", 721, 1), "己酉");
        assert_eq!(gz("麟德历", 721, 12), "甲戌");
        // 大衍 750 正月超戌初不进（史载例外表，本纪）、至德戌初限（763 六月、764 二月进）
        assert_eq!(gz("大衍历", 750, 1), "庚寅");
        assert_eq!(gz("五纪历", 763, 6), "癸酉");
        assert_eq!(gz("五纪历", 764, 2), "己巳");
        // 三大三小（779 正月）、三大二小（崇天 1027 正月、观天 1097 正月、成天 1274 六月、1275 七月）
        assert_eq!(gz("五纪历", 779, 1), "壬寅");
        assert_eq!(gz("崇天历", 1027, 1), "壬寅");
        assert_eq!(gz("观天历", 1097, 1), "丙戌");
        assert_eq!(gz("成天历", 1274, 6), "丙午");
        assert_eq!(gz("成天历", 1275, 7), "庚午");
        // 消息：937 正月退作甲寅，二月随退作甲申
        assert_eq!(gz("崇玄历（宣明气朔）", 937, 2), "甲申");
        // 五代「宣明、崇玄參而成之」：904 年宣明恒气闰三月，崇玄恒气闰四月（本纪、新唐书、旧五代史、通鉴同）
        let leap = |o: WudaiDingqi| reckon_year_with("崇玄历（宣明气朔）", 904, &PaiyueOptions { wudai_dingqi: o, ..PaiyueOptions::default() }).unwrap().into_iter().find(|e| e.2).map(|e| e.1);
        assert_eq!(leap(WudaiDingqi::Chongxuan), Some(4));
        assert_eq!(leap(WudaiDingqi::Xuanming), Some(3));
        // 旧做法（只按术文进朔）仍可取
        let old = reckon_year_shuwen("崇天历", 1027).unwrap();
        assert_eq!(crate::calendar::gan_zhi::GAN_ZHI_60[(old[0].0 + 49).rem_euclid(60) as usize], "癸卯");
    }

    /// 入交定与日食判定与 Python 复原逐月相同：`data/calendar/jiaohui_reference.csv`（scripts/calendar_data/jiaohui.py）
    #[test]
    fn jiaohui_matches_python_reference() {
        let csv = std::fs::read_to_string(crate::repo::repo_root().join("data/calendar/jiaohui_reference.csv")).unwrap();
        let mut n = 0;
        for line in csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("历,")) {
            let c: Vec<&str> = line.split(',').collect();
            let li = jianfa(&format!("{}历", c[0]));
            let (y, m): (i32, i64) = (c[1].parse().unwrap(), c[2].parse().unwrap());
            let j = li.jiao(y, m).unwrap();
            assert_eq!(j.ru * q(li.f()), parse_q(c[3]), "{} {y} 岁第 {m} 月入交定", c[0]);
            if !c[4].is_empty() {
                let r = li.rishi(y, m).unwrap();
                assert_eq!((u8::from(r.shi).to_string(), u8::from(r.yin).to_string()), (c[4].to_string(), c[5].to_string()), "{} {y} {m} 蚀", c[0]);
                assert_eq!(r.shen, parse_q(c[6]), "{} {y} {m} 食甚", c[0]);
                assert_eq!(r.dingyong.map(|d| d.to_string()).unwrap_or_default(), if c[7].is_empty() { String::new() } else { parse_q(c[7]).to_string() });
            }
            n += 1;
        }
        assert!(n > 2000);
    }

    /// 交会规则所推之月：正月有交（780 进、937—938 退）、有食應見者不进（1033 年六月）；崇天日蚀术判 1039 年八月不蚀而进
    #[test]
    fn jiaohui_rules_applied() {
        let gz = |name: &str, y: i32, m: u8| {
            let j = reckon_year(name, y).unwrap().into_iter().find(|e| e.1 == m && !e.2).unwrap().0;
            crate::calendar::gan_zhi::GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
        };
        assert_eq!(gz("五纪历", 780, 1), "丁卯");
        assert_eq!(gz("崇玄历（宣明气朔）", 937, 1), "甲寅");
        assert_eq!(gz("崇玄历（宣明气朔）", 938, 1), "戊申");
        assert_eq!(gz("崇玄历（宣明气朔）", 938, 2), "戊寅");
        assert_eq!(gz("崇天历", 1033, 6), "甲午");
        assert_eq!(gz("崇天历", 1039, 8), "庚申");
        // 规则不用时回到按术进朔
        let off = PaiyueOptions { xuanming: None, song: None, zhengyue: ZhengyueJiao::Off, ..PaiyueOptions::default() };
        let j = reckon_year_with("崇天历", 1033, &off).unwrap().into_iter().find(|e| e.1 == 6 && !e.2).unwrap().0;
        assert_eq!(crate::calendar::gan_zhi::GAN_ZHI_60[(j + 49).rem_euclid(60) as usize], "乙未");
    }

    /// 703 年（长安三年）九月：三书「九月庚寅朔，日有蝕之」，张表己丑。麟德速要定朔己丑 15.13 时、本术（二次内插）15.19 时，
    /// 经朔己丑 12.05 时，皆不及任何进朔之限（正月戌初、张表四分之三），按术不能得庚寅。680 年二月：迟段增减号改正（[ld_zeng]）前
    /// 定朔丁未 0.44 时，与张表丙午异；改正后丙午 22.27 时（本术 22.37 时），与张表同。665 年四月癸酉、闰四月壬寅、五月壬申合
    /// 李善兰为汪曰桢所步细草。见 doc/calendar/dingshuo-zhangbiao-diffs.md。
    #[test]
    fn linde_703_september_and_680_february() {
        let l = linde();
        let f = q(LD_F);
        let gz = |t: &Q| crate::calendar::gan_zhi::GAN_ZHI_60[(fl(&(t / &f)) + l.off + 49).rem_euclid(60) as usize];
        let hours = |t: &Q| (md(t, &f) / &f).to_f64().unwrap() * 24.0;
        // 冬至岁 703 第 11 月即九月
        let (jing, suyao) = l.month(703, 11);
        let ben = l.ding_benshu(703, 11);
        assert_eq!((gz(&jing), gz(&suyao), gz(&ben)), ("己丑", "己丑", "己丑"));
        assert!((hours(&jing) - 12.05).abs() < 0.01 && (hours(&suyao) - 15.13).abs() < 0.01 && (hours(&ben) - 15.19).abs() < 0.01);
        let y703 = reckon_year("麟德历", 703).unwrap();
        assert_eq!(y703.iter().find(|e| e.1 == 9 && !e.2).map(|e| crate::calendar::gan_zhi::GAN_ZHI_60[(e.0 + 49).rem_euclid(60) as usize]), Some("己丑"));
        // 680 年二月（冬至岁 680 第 3 月）
        let (_, suyao) = l.month(680, 3);
        let ben = l.ding_benshu(680, 3);
        assert_eq!((gz(&suyao), gz(&ben)), ("丙午", "丙午"));
        assert!((hours(&suyao) - 22.27).abs() < 0.01);
        // 665 年四月、闰四月、五月（冬至岁 665 第 5、6、7 月）
        let g: Vec<&str> = (5..=7).map(|m| gz(&l.ding_for_calendar(665, m))).collect();
        assert_eq!(g, ["癸酉", "壬寅", "壬申"]);
    }

    /// 麟德步交会：702 年九月乙丑（实有全食）内道、去交 0.76 日入蚀限而蚀；703 年三月壬戌（实有环食）去交 0.08 日入蚀限（外道，
    /// 加时 16.2 时，按外道诸准不蚀）；703 年九月（三书「庚寅朔，日有蝕之」，实无日食）去交 2.21 日，不入蚀限，食定小余 13.3 时
    /// 仍在己丑——此食不出麟德推步，庚寅亦不能由食甚得之。同年八月庚申定交分内道去交 0.05 日，变交分出日道表在三时半（392 分）外，
    /// 不蚀（改前按定交分判蚀，加时在夜）。
    /// 麟德本术十四日、二十八日「分為二日」（[Linde::chisu_fen]）：段首段末与速要表值相接；666—728 年覆本术之月只 702 年十月
    /// （冬至岁 702 第 11 月，乙未）入变恰在此二日，定小余 3.70→9.33 分，同在夜半后，朔日不变；排月与速要逐月全同
    #[test]
    fn linde_fen_erri() {
        let at = |d: i64, r: i64| q(d) + qr(r, LD_F);
        // 段首即表值：十四日初速 102，二十八日初迟 71（第二次改正在段首为 0）
        assert_eq!(Linde::chisu_fen(&at(13, 0)), q(102));
        assert_eq!(Linde::chisu_fen(&at(27, 0)), q(-71));
        // 十四日末段之首入迟 0；二十八日之末近 0（第二次改正 t₀ = v/段长 使段末不恰为表值，常日亦然，此处约迟 6 分）
        assert!(Linde::chisu_fen(&(at(13, 1042) + qr(1, 1_000_000))).abs() < qr(1, 1000));
        assert!(Linde::chisu_fen(&at(27, 743)).abs() < q(8));
        // 与速要之差至多约 10 分（0.2 时）
        for r in (0..1340).step_by(67) {
            let x = at(13, r);
            assert!((Linde::chisu_fen(&x) - Linde::chisu(&x)).abs() < q(12), "十四日 {r}");
        }
        let (on, off) = (Linde { fen_erri: true, ..linde() }, Linde { fen_erri: false, ..linde() });
        let mut changed = Vec::new();
        for y in 665..=729 {
            for m in 0..13 {
                if on.ding_for_calendar(y, m) != off.ding_for_calendar(y, m) {
                    changed.push((y, m, (md(&off.ding_for_calendar(y, m), &q(LD_F)) * q(100)).round().to_f64().unwrap(), (md(&on.ding_for_calendar(y, m), &q(LD_F)) * q(100)).round().to_f64().unwrap()));
                }
            }
        }
        assert_eq!(changed, vec![(702, 11, 370.0, 933.0)]);
        let o0 = PaiyueOptions { linde_fen_erri: false, ..PaiyueOptions::default() };
        for y in 666..=728 {
            assert_eq!(reckon_year_with("麟德历", y, &o0), reckon_year("麟德历", y), "{y}");
        }
    }

    #[test]
    fn linde_jiaohui_703() {
        let l = linde();
        let gz = |y: i32, m: i64| crate::calendar::gan_zhi::GAN_ZHI_60[(fl(&(l.ding_for_calendar(y, m) / q(LD_F))) + l.off + 49).rem_euclid(60) as usize];
        let (j, r) = (l.jiao(702, 10).unwrap(), l.rishi(702, 10).unwrap());
        assert_eq!(gz(702, 10), "乙丑");
        assert!(j.yin && j.shixian && r.shi);
        let (j, r) = (l.jiao(703, 4).unwrap(), l.rishi(703, 4).unwrap());
        assert_eq!(gz(703, 4), "壬戌");
        assert!(j.shixian && !j.yin && j.qu < qr(1, 10) && !r.shi);
        let (j, r) = (l.jiao(703, 11).unwrap(), l.rishi(703, 11).unwrap());
        assert_eq!(gz(703, 11), "己丑");
        assert!(!j.shixian && !r.shi && (j.qu.to_f64().unwrap() - 2.213).abs() < 0.01);
        assert!((r.shen.to_f64().unwrap() * 24.0 - 13.32).abs() < 0.01);
        let (j, r) = (l.jiao(703, 10).unwrap(), l.rishi(703, 10).unwrap());
        assert_eq!(gz(703, 10), "庚申");
        assert!(j.yin && j.shixian && r.yin && !r.shi && r.shen < qr(1, 4));
        // 变交分出日道表在三时半外，不蚀；按外道诸准判亦不蚀；不用变交分则内道当蚀
        let wai = Linde { jiaohui: LindeJiaohui::BianWaidao, ..linde() };
        let r = wai.rishi(703, 10).unwrap();
        assert!(!r.yin && !r.shi);
        let jian = Linde { jiaohui: LindeJiaohui::Jian, ..linde() };
        let r = jian.rishi(703, 10).unwrap();
        assert!(r.yin && r.shi);
    }

    /// 逐月细目（[dingshuo_months]）与排月一致：朔日即 [jiao_months] 所排，且按术进朔者朔日不早于定朔所在日
    #[test]
    fn dingshuo_months_consistent() {
        for name in ["麟德历", "大衍历", "宣明历", "崇天历", "授时历"] {
            let v = dingshuo_months(name).unwrap();
            assert!(v.windows(2).all(|w| (29..=30).contains(&(w[1].day - w[0].day))), "{name}");
            for m in &v {
                assert!((0.0..1.0).contains(&m.frac));
                if let Some(l) = m.limit {
                    assert_eq!(m.jin, m.frac >= l - 1e-12, "{name} {}", m.ding_jdn);
                }
            }
        }
        // 宣明 891 年十月：定朔丁丑 20.21 时，过限进戊寅（本纪丁丑，已考讹）
        let v = dingshuo_months("宣明历").unwrap();
        let m = v.iter().find(|m| crate::calendar::gan_zhi::GAN_ZHI_60[(m.ding_jdn + 49).rem_euclid(60) as usize] == "丁丑" && m.dongzhi_year == 891 && m.frac > 0.8).unwrap();
        assert!(m.jin && (m.frac * 24.0 - 20.21).abs() < 0.01 && m.day == m.ding_jdn + 1);
    }

    /// 表段（[Biaoduan]）：月离日序与入余、分段日之初数、日躔入气皆在范围内；宣明 874 年十二月钉住（经朔入转第七日初段，
    /// 见 `doc/calendar/dingshuo-zhangbiao-diffs.md`「逐月复核」）
    #[test]
    fn biaoduan_in_range() {
        for name in ["戊寅元历", "麟德历", "大衍历", "五纪历", "正元历", "宣明历", "崇天历", "纪元历", "开禧历", "授时历", "大统历"] {
            let v = dingshuo_months(name).unwrap();
            for m in &v {
                let b = m.biaoduan.as_ref().unwrap_or_else(|| panic!("{name} 无表段"));
                let (n, yu, chu) = b.yue.unwrap();
                assert!((1..=28).contains(&n), "{name} 日序 {n}");
                // 末日（入转不足整日）与宣明半周末日之余可过一日之比，不过 1.0 为常
                assert!(yu >= 0.0 && yu < 1.0 + 1e-9, "{name} {n} 入余 {yu}");
                if let Some(c) = chu {
                    assert!(c > 0.0 && c < 1.0, "{name} 初数 {c}");
                    if !name.starts_with("麟德") {
                        assert!([7, 14, 21, 28].contains(&n), "{name} 分段日 {n}");
                    }
                }
                let (k, d, len) = b.ri;
                assert!(k < 24 && d >= -1e-9 && d <= len + 1e-9 && (13.5..17.0).contains(&len), "{name} 气 {k} {d}/{len}");
            }
        }
        let v = dingshuo_months("宣明历").unwrap();
        let m = v.iter().find(|m| m.dongzhi_year == 875 && (m.frac * 24.0 - 18.03).abs() < 0.01).unwrap();
        let (n, yu, chu) = m.biaoduan.as_ref().unwrap().yue.unwrap();
        assert_eq!(n, 7);
        assert!((yu - 0.590).abs() < 0.001 && (chu.unwrap() - 7465.0 / 8400.0).abs() < 1e-9);
        assert_eq!(m.biaoduan.as_ref().unwrap().ri.0, 1, "小寒");
    }

    /// 各历在其年段内逐年排月：每年 12 或 13 月，月长 29、30 日，前后年首尾相接
    #[test]
    fn year_months_well_formed() {
        for (name, spans) in registry() {
            for &(a, b) in spans.iter() {
                for y in a..=b {
                    let m = reckon_year(name, y).unwrap();
                    assert!(m.len() == 12 || m.len() == 13, "{name} {y} 年 {} 月", m.len());
                    assert_eq!((m[0].1, m[0].2), (1, false), "{name} {y} 正月");
                    assert!(m.windows(2).all(|w| (29..=30).contains(&(w[1].0 - w[0].0))), "{name} {y} 月长 {m:?}");
                    if y < b {
                        let next = reckon_year(name, y + 1).unwrap();
                        let len = next[0].0 - m.last().unwrap().0;
                        assert!((29..=30).contains(&len), "{name} {y} 岁末月长 {len}");
                    }
                }
            }
        }
    }
}

