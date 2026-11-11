//! 唐至明定朔诸历推步（历法域第 1 层·历谱生成；按历名供第 4 层 [crate::calendar::lifa::resolve_year] 取用）。
//!
//! 由 `scripts/calendar_data/*_dingshuo.py` 的复原移植，逐历对照：戊寅（定朔、平朔）、麟德、大衍、五纪、正元、宣明、崇玄、
//! 宋诸历（崇天、明天、观天、纪元、统元、乾道、淳熙、会元、统天、开禧、成天）、授时、大统。常数、表与校改的出处见各脚本
//! 文档与 `doc/calendar/calendar-compendium.md`；宋诸历之表由 `export_dingshuo_reference.py` 自维基文库底本解析、校改后导出为
//! `data/calendar/dingshuo_song_tables.csv`。移植正确性以 `data/calendar/dingshuo_reference.csv`（Python 所算定朔，精确分数）
//! 逐月比对（测试 `matches_python_reference`）。
//!
//! 排月（Python 复原只算定朔，此处新增）：定朔所在日，小余过进朔限者进一日；恒气（常气）中气，十一月为含冬至之月，
//! 两冬至间 13 月者以第一个无中气之月为闰。不实现：「有交，亏初在日入前者不进」、「使不过三大二小」。
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
}

fn day_of(li: &dyn DingshuoLi, y: i32, t: &Q) -> i64 {
    let f = q(li.f());
    let day = fl(&(t / &f));
    let xiao = t - q(day) * &f;
    day + li.jinshuo_limit(y, t).map_or(0, |l| i64::from(xiao >= l))
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

/// 冬至岁 y 的月：(朔日 JDN, 月序, 闰否)，自含冬至之月（十一月）起至下一含冬至之月前。
fn sui_months(li: &dyn DingshuoLi, y: i32) -> Vec<(i64, u8, bool)> {
    let off = li.off();
    // 每个经朔只由其所在冬至岁算一次（经朔 < 下一冬至岁天正经朔），免得相邻两岁所算定朔微异而重出
    let mut days: Vec<i64> = match li.fixed_days() {
        Some(all) => all.to_vec(),
        None => month_starts(li, y - 1, y + 1).into_iter().map(|(d, _)| d).collect(),
    };
    days.sort_unstable();
    days.dedup();
    let sui = li.sui();
    let f = q(li.f());
    let zq: Vec<i64> = (0..=12).map(|k| fl(&((li.dongzhi(y) + &sui * qr(k, 12)) / &f)) + off).collect();
    let (dz0, dz1) = (zq[0], zq[12]);
    let contains = |i: usize, d: i64| days[i] <= d && d < days[i + 1];
    let i0 = (0..days.len() - 1).find(|&i| contains(i, dz0)).expect("含冬至之月");
    let i1 = (0..days.len() - 1).find(|&i| contains(i, dz1)).expect("含下一冬至之月");
    assert!(i1 - i0 == 12 || i1 - i0 == 13, "冬至岁 {y}：两冬至间 {} 月", i1 - i0);
    let leap = if i1 - i0 == 13 { (i0 + 1..i1).find(|&i| !zq.iter().any(|&z| contains(i, z))) } else { None };
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
    jinshuo: Jinshuo,
    off: i64,
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
        let ding = q(jing) + self.sun_def(&q(jing - tong)) + self.moon_def(&md(&q(jing), &self.zhuan));
        (q(jing), ding)
    }
    fn jinshuo_limit(&self, y: i32, t: &Q) -> Option<Q> {
        let f = q(self.f);
        Some(match self.jinshuo {
            Jinshuo::Hours(h) => &f * qr(h, 24),
            Jinshuo::Lou => {
                // 定朔所在日的夜半漏：自其定气初日逐日递推（[dayan_night_lou]）
                let tong = q(self.tong(y));
                let day = fl(&(t / &f));
                let d = md(&(q(day) * &f - &tong), &q(self.ce));
                let k = self.seg(&d);
                let qi_day = fl(&((&tong + self.ding_qi(k) + q(self.ce) * q(fl(&((q(day) * &f - &tong) / q(self.ce))))) / &f));
                let lou = dayan_night_lou(k.rem_euclid(24) as usize, (day - qi_day).max(0));
                // 24 − (漏 − 十刻) × 0.24 时
                &f * (q(24) - (lou - q(10)) * qr(24, 100)) / q(24)
            }
            Jinshuo::Chongxuan => {
                let a = &f * qr(29, 40);
                let b = &f - self.chongxuan_chenchu(y, t);
                if a > b { a } else { b }
            }
            Jinshuo::Xuanming => {
                const LIM: [i64; 13] = [75000, 74924, 74479, 74276, 74124, 74031, 74005, 74031, 74124, 74276, 74479, 74924, 75000];
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

fn make_dayan(f: i64, ce: i64, she: i64, zhuan: Q, ji: i64, y_ji: i32, xianhou: Vec<i64>, sun: Vec<i64>, moon: Vec<MoonTable>, jinshuo: Jinshuo, approx: (i32, i32, i32)) -> Dayan {
    let mut d = Dayan { f, ce, she, zhuan, ji, y_ji, xianhou, sun, moon, jinshuo, off: 0 };
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
    make_dayan(
        3040, 1110343, 89773, qr(6701279, 80), 96961740, 724,
        vec![0, 2353, 4198, 5588, 6564, 7152, 7366, 7152, 6564, 5588, 4198, 2353, 0, -2353, -4198, -5588, -6564, -7152, -7366, -7152, -6564, -5588, -4198, -2353],
        vec![176, 138, 104, 73, 44, 16, -16, -44, -73, -104, -138, -176, -176, -138, -104, -73, -44, -16, 16, 44, 73, 104, 138, 176],
        vec![MoonTable { f: 3040, rates: &DAYAN_MOON, split: split4([(6, 2701, 48, 339, -6), (13, 2363, -231, 677, -66), (20, 2024, -36, 1016, 18), (27, 1686, 165, 1354, 0)]) }],
        // 大衍历志无进朔明文；《新唐书·历志五》述五纪历「依大衍戌初進初朔」，取戌初 19 时
        Jinshuo::Hours(19),
        (723, 12, 21),
    )
}

fn wuji() -> Dayan {
    make_dayan(
        1340, 489428, 39571, qr(1366156, 37), 269978, 762,
        sym(&[0, 1037, 1850, 2463, 2893, 3152, 3246, 3152, 2893, 2463, 1850, 1037]),
        sym(&[78, 61, 46, 32, 19, 7, -7, -19, -32, -46, -61, -78]),
        vec![MoonTable { f: 1340, rates: &WUJI_MOON, split: split4([(6, 1191, -8, 149, 1), (13, 1042, 105, 298, 30), (20, 892, 6, 448, -3), (27, 743, -75, 597, 0)]) }],
        Jinshuo::Lou,
        (761, 12, 19),
    )
}

fn zhengyuan() -> Dayan {
    make_dayan(
        1095, 399943, 32336, qr(301720132, 10000), 402900, 784,
        sym(&[0, 848, 1512, 2013, 2364, 2576, 2653, 2576, 2364, 2013, 1512, 848]),
        sym(&[63, 50, 37, 26, 16, 6, -6, -16, -26, -37, -50, -63]),
        vec![MoonTable { f: 1095, rates: &ZHENGYUAN_MOON, split: split4([(6, 973, -7, 122, 1), (13, 851, 87, 244, 25), (20, 729, 5, 366, -2), (27, 607, -68, 488, 0)]) }],
        Jinshuo::Lou,
        (783, 12, 19),
    )
}

fn xuanming() -> Dayan {
    let half = |rates: &'static [Option<i64>], s6: (i64, i64), s13: i64| MoonTable {
        f: 8400,
        rates,
        split: vec![(6, q(7465), s6.0, q(935), s6.1), (13, q(6529), s13, q(1), 0)],
    };
    make_dayan(
        8400, 3068055, 248057, qr(23145819, 100), 7070138, 822,
        [0, 60, 110, 150, 180, 198, 204, 198, 180, 150, 110, 60, 0, -60, -110, -150, -180, -198, -204, -198, -180, -150, -110, -60].iter().map(|x| 100 * x).collect(),
        vec![449, 374, 299, 224, 135, 45, -45, -135, -224, -299, -374, -449, -449, -374, -299, -224, -135, -45, 45, 135, 224, 299, 374, 449],
        vec![half(&XUANMING_A, (53, -7), -646), half(&XUANMING_B, (-53, 7), 646)],
        Jinshuo::Xuanming,
        (821, 12, 19),
    )
}

fn chongxuan() -> Dayan {
    make_dayan(
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
        Jinshuo::Chongxuan,
        (891, 12, 17),
    )
}

// ---------------------------------------------------------------- 麟德

struct Linde {
    off: i64,
    /// 麟德历行用年段前后各留二岁的全部朔日（已按「不過頻三」调整）
    days: std::sync::OnceLock<Vec<i64>>,
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
            return q(LD_CHISU[27]) + q(LD_ZENG[27]) * frac * q(LD_F) / q(743);
        }
        if i == 13 {
            let chu = qr(1042, LD_F);
            if frac <= chu {
                return q(LD_CHISU[13]) - q(102) * frac / chu;
            }
            return -(q(29) * (frac - &chu) / (q(1) - chu));
        }
        q(LD_CHISU[i as usize]) + q(LD_ZENG[i as usize]) * frac
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
    /// 十四日、二十八日分初末（「分為二日，隨餘初末」），仍用速要。
    fn chisu_benshu(x: &Q) -> Q {
        let i = fl(x);
        if i >= 27 || i == 13 {
            return Self::chisu(x);
        }
        let t = x - q(i);
        let (a1, a2) = (q(LD_ZENG[i as usize]), q(LD_ZENG[i as usize + 1]));
        let rate = |u: &Q| u * &a1 + u * (u - q(1)) / q(2) * (&a2 - &a1);
        let t0 = rate(&t) / q(LD_F);
        let su = i < 14;
        let tt = if su { &t - t0 } else { &t + t0 };
        q(LD_CHISU[i as usize]) + rate(&tt)
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
        &chq + yn - Self::chisu_benshu(&bian)
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
        let ding = &chq + yn - Self::chisu(&bian);
        (chq, ding)
    }
    // 两唐书麟德历皆无进朔条文，按志文不进朔（缺省 None）。
    fn ding_for_calendar(&self, y: i32, m: i64) -> Q {
        let ding = self.month(y, m).1;
        let xiao = md(&ding, &q(LD_F));
        if xiao <= q(24) || xiao >= q(1316) {
            self.ding_benshu(y, m)
        } else {
            ding
        }
    }
    /// 「不過頻三」（[limit_runs]）。「其正月朔有定加時正月者，消息前後各一兩月」须推交食，未实现。
    fn fixed_days(&self) -> Option<&[i64]> {
        Some(self.days.get_or_init(|| {
            let mut s = month_starts(self, 663, 730);
            limit_runs(&mut s, 3);
            s.into_iter().map(|x| x.0).collect()
        }))
    }
}

fn linde() -> Linde {
    let dz_day = (LD_QI * LD_JI_664).div_euclid(LD_F);
    Linde { off: align_offset(dz_day, gregorian::civil_to_jdn(663, 12, 20)), days: std::sync::OnceLock::new() }
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
        let v = qr(400, 567) * &x * (&l - &x);
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
    /// 消息定数用「日中日度」：平日加盈缩差度（x(L − x)/4135，即盈缩定差 × 567/(400 × 4135)）。
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
    let mut s = Song { f, sui, shuo, zhuan, ji, y_ji, rinru, cha, sun, moon, lou, off: 0 };
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
}

fn shoushi() -> Shoushi {
    // 《明史》称《大统历法通轨》所载为授时续定之数：闰应 202050、转应 130205（与《元史》历经 201850、131904 异），授时亦用之
    Shoushi { run_ying: 202050, zhuan_ying: 130205, off: align_offset(SS_QI_YING / SS_F, gregorian::civil_to_jdn(1280, 12, 14)) }
}

// ---------------------------------------------------------------- 按历名取

/// 中原一线定朔诸历：(历名（同 `zhongyuan_calendar_periods.csv`）, 推步适用的农历年段)。
const REGISTRY: &[(&str, &[(i32, i32)])] = &[
    ("戊寅元历", &[(627, 645)]),
    ("戊寅元历（平朔）", &[(646, 665)]),
    ("麟德历", &[(666, 728)]),
    ("大衍历", &[(729, 761)]),
    ("五纪历", &[(763, 783)]),
    ("正元历", &[(784, 806)]),
    ("宣明历", &[(822, 892)]),
    ("崇玄历", &[(893, 938), (944, 955)]),
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

/// 历名 → 历（惰性构造，宋诸历读表）。
pub fn li_by_name(name: &str) -> Option<&'static dyn DingshuoLi> {
    use std::sync::OnceLock;
    macro_rules! once {
        ($e:expr) => {{
            static C: OnceLock<Box<dyn DingshuoLi>> = OnceLock::new();
            Some(C.get_or_init(|| Box::new($e)).as_ref())
        }};
    }
    match name {
        "戊寅元历" => once!(wuyin(false)),
        "戊寅元历（平朔）" => once!(wuyin(true)),
        "麟德历" => once!(linde()),
        "大衍历" => once!(dayan()),
        "五纪历" => once!(wuji()),
        "正元历" => once!(zhengyuan()),
        "宣明历" => once!(xuanming()),
        "崇玄历" => once!(chongxuan()),
        "崇天历" => once!(song("崇天", 10590, 3867940, q(312729), qr(2918030594, 10000), 97556340, 1024, 0, true, None, (1023, 12, 18))),
        "明天历" => once!({
            let dz = Mingtian::tong(1064).div_euclid(MT_F);
            Mingtian { off: align_offset(dz, gregorian::civil_to_jdn(1063, 12, 16)) }
        }),
        "观天历" => once!(song("观天", 12030, 4393880, q(355253), qr(3314820389, 10000), 5944808, 1092, 0, false, None, (1091, 12, 16))),
        "纪元历" => once!(song("纪元", 7290, 2662626, q(215278), qr(2008730990, 10000), 28613466, 1106, 15, false, None, (1105, 12, 16))),
        "统元历" => once!(song("统元", 6930, 2531138, q(204647), qr(1909532563, 10000), 94251591, 1135, 0, false, None, (1134, 12, 15))),
        "乾道历" => once!(song("乾道", 30000, 10957308, qr(88591776, 100), qr(8266377395, 10000), 91645823, 1167, 0, false, None, (1166, 12, 15))),
        "淳熙历" => once!(song("淳熙", 5640, 2059974, qr(16655256, 100), qr(1554079740, 10000), 52421972, 1176, 0, false, None, (1175, 12, 15))),
        "会元历" => once!(song("会元", 38700, 14134932, q(1142834), qr(10663617310, 10000), 25494767, 1191, 0, false, None, (1190, 12, 15))),
        "统天历" => once!(song(
            "统天", 12000, 4382910, q(354368), q(330655), 3835, 1199, 0, false,
            Some(SongCha { ju: 3830, qi: 237811, run: 21704, zhuan: 188800 }), (1198, 12, 15)
        )),
        "开禧历" => once!(song("开禧", 16900, 6172608, q(499067), qr(4656725396, 10000), 7848183, 1207, 0, false, None, (1206, 12, 15))),
        "成天历" => once!(song("成天", 7420, 2710101, q(219117), qr(2044551641, 10000), 71758147, 1271, 0, false, None, (1270, 12, 14))),
        "授时历" => once!(shoushi()),
        "大统历" => once!(Tonggui::new()),
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

/// 按历名推某农历年各月，不限中原一线行用年段：供分政权历日（如南明、明郑续用大统历，1645—1683）。
pub fn reckon_year_any(name: &str, lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    Some(year_months(li_by_name(name)?, lunar_year))
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
            let li = li_by_name(alias(c[0])).unwrap();
            assert_eq!(li.f().to_string(), c[1], "{} 日法", c[0]);
            assert_eq!(li.off().to_string(), c[2], "{} 上元", c[0]);
            let (y, m): (i32, i64) = (c[3].parse().unwrap(), c[4].parse().unwrap());
            assert_eq!(li.month(y, m).1, parse_q(c[5]), "{} {y} 岁第 {m} 月", c[0]);
            n += 1;
        }
        assert!(n > 4000);
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
