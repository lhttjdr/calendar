//! 宋初应天历（964—982）、乾元历（983—1000）、仪天历（1001—1023）定朔与排月。
//!
//! 由 `scripts/calendar_data/song_early_dingshuo.py` 移植，照《宋史》卷六十八、六十九（三历同表分栏）复原，读法参王应伟
//! 《中国古历通解》第六章第一节。要点：应天岁盈作 730635（王应伟书页 511）；乾元转分作 16202.1204（同书页 526 校订者按）；
//! 「下以損益率展」读作乘其补数（日躔补至百、月离进位后补至千），据表闭合；诸表讹字据逐日损益与积闭合校正；
//! 进朔限 = 日法 3/4 −（该日晨分与二分晨分差之绝对值）/2；中气用常气。依据与检验见脚本文首与 `data/calendar/song_early_check.csv`。

use crate::math::real::{real, Real, RealOps};

fn r(x: f64) -> Real {
    real(x)
}

fn jul_jdn(y: i32, m: i32, d: i32) -> i64 {
    let a = (14 - m) / 12;
    let (yy, mm) = ((y + 4800 - a) as i64, (m + 12 * a - 3) as i64);
    d as i64 + (153 * mm + 2) / 5 + 365 * yy + yy / 4 - 32083
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Yingtian,
    Qianyuan,
    Yitian,
}

/// 应天先历、后历逐日损益率（负为损：|积|增 1000−率；正为益：|积|减），七日、十四日 (初, 末)
#[derive(Clone, Copy)]
enum Rate {
    One(i64),
    Two(i64, Option<i64>),
}
use Rate::{One, Two};

const YT_DING: [f64; 12] = [14.5045, 14.6236, 14.7425, 14.8616, 15.0042, 15.1470, 15.2899, 15.4328, 15.5757, 15.6947, 15.8136, 15.9327];
const YT_SUN_RATE: [i64; 24] = [-64, -69, -76, -82, -89, -97, 97, 89, 83, 78, 72, 66, -65, -72, -77, -83, -89, -97, 97, 89, 82, 75, 70, 64];
const YT_SUN_JI: [i64; 24] =
    [-20, 529, 975, 1335, 1606, 1771, 1819, 1780, 1605, 1350, 995, 541, 5, -549, -985, -1346, -1611, -1780, -1831, -1786, -1621, -1357, -988, -550];
const YT_XIAN: [Rate; 14] =
    [One(-12), One(-136), One(-278), One(-439), One(-599), One(-760), Two(-937, Some(992)), One(900), One(733), One(565), One(394), One(235), One(110), Two(231, None)];
const YT_HOU: [Rate; 14] =
    [One(-12), One(-136), One(-288), One(-448), One(-608), One(-768), Two(-937, Some(992)), One(900), One(732), One(564), One(404), One(252), One(120), Two(231, None)];
const YT_CHEN: [f64; 24] = [
    2748.0, 2735.0, 2688.0, 2612.0, 2508.0, 2388.0, 2350.0, 2212.0, 1992.0, 1888.0, 1812.0, 1765.0, 1752.0, 1765.0, 1812.0, 1888.0, 1992.0, 2212.0, 2350.0,
    2388.0, 2508.0, 2612.0, 2688.0, 2735.0,
];
const QY_YY: [i64; 12] = [0, 170, 303, 404, 475, 518, 532, 518, 475, 404, 303, 170];
const QY_RATE: [i64; 12] = [170, 133, 101, 71, 43, 14, -14, -43, -71, -101, -133, -170];
const QY_MOON: [Rate; 28] = [
    One(287), One(250), One(213), One(173), One(134), One(93), Two(46, Some(-6)), One(-62), One(-102), One(-141), One(-193), One(-211), One(-256),
    Two(-225, Some(-63)), One(-280), One(-242), One(-205), One(-165), One(-126), One(-84), Two(-35, Some(17)), One(71), One(112), One(150), One(191),
    One(229), One(266), Two(161, None),
];
const QY_CHEN: [f64; 24] = [
    808.0, 802.0, 786.0, 761.0, 732.0, 699.0, 666.0, 624.0, 589.0, 558.0, 534.0, 519.0, 515.0, 519.0, 534.0, 558.0, 585.0, 624.0, 666.0, 699.0, 730.0,
    762.0, 786.0, 803.0,
];
const YI_UP: [f64; 7] = [1086.0, 916.0, 746.0, 576.0, 406.0, 236.0, 75.0];
const YI_DOWN: [f64; 7] = [86.0, 236.0, 406.0, 576.0, 746.0, 916.0, 1200.0];
const YI_L1: f64 = 88.0 + 8899.5 / 10100.0;
const YI_L2: f64 = 93.0 + 7485.5 / 10100.0;
const YI_QIAN: f64 = 182.62228;

struct Li {
    kind: Kind,
    /// 日法
    f: i64,
    ji_base: i64,
    y_base: i32,
}

const LIS: [(&str, Li, i32, i32); 3] = [
    ("应天历", Li { kind: Kind::Yingtian, f: 10002, ji_base: 4_825_558, y_base: 962 }, 964, 983),
    ("乾元历", Li { kind: Kind::Qianyuan, f: 2940, ji_base: 30_543_977, y_base: 981 }, 984, 1000),
    ("仪天历", Li { kind: Kind::Yitian, f: 10100, ji_base: 716_497, y_base: 1001 }, 1001, 1023),
];

impl Li {
    fn ji(&self, y: i32) -> i64 {
        self.ji_base + (y - self.y_base) as i64
    }
    /// (天正冬至日 mod 60, 闰余日, 天正经朔入转日)
    fn epoch(&self, y: i32) -> (Real, Real, Real) {
        let n = self.ji(y);
        let f = r(self.f as f64);
        match self.kind {
            Kind::Yingtian => {
                let yj = n * 730_635;
                let dz = r(((yj % 840_168) * 5) as f64) / f;
                let tong = yj % 59_073;
                // (元积 − 通余) mod 离总 55120.1242：以万分整数算
                let z = ((yj - tong) as i128 * 10_000) % 551_201_242;
                (fmod(dz, 60.0), r((tong * 5) as f64) / f, r(z as f64) / r(10_000.0) * r(5.0) / f)
            }
            Kind::Qianyuan => {
                let s = n * 214_764;
                let dz = r(((s % 70_560) * 5) as f64) / f;
                let shuo_yu = s % 17_364;
                let z = ((s - shuo_yu) as i128 * 10_000) % 162_021_204;
                (fmod(dz, 60.0), r((shuo_yu * 5) as f64) / f, r(z as f64) / r(10_000.0) * r(5.0) / f)
            }
            Kind::Yitian => {
                let s = n * 3_688_970;
                let dz = r((s % (60 * 10_100)) as f64) / f;
                let run = s % 298_259;
                let z = ((s - run) as i128 * 10_000) % 2_783_010_165;
                (dz, r(run as f64) / f, r(z as f64) / r(10_000.0) / f)
            }
        }
    }
    fn shuo(&self) -> Real {
        match self.kind {
            Kind::Yingtian => r(5.0 * 59_073.0 / 10_002.0),
            Kind::Qianyuan => r(29.0 + 1560.0 / 2940.0),
            Kind::Yitian => r(298_259.0 / 10_100.0),
        }
    }
    fn zhuan(&self) -> Real {
        match self.kind {
            Kind::Yingtian => r(55_120.1242 * 5.0 / 10_002.0),
            Kind::Qianyuan => r(16_202.1204 * 5.0 / 2940.0),
            Kind::Yitian => r(278_301.0165 / 10_100.0),
        }
    }
    fn year(&self) -> Real {
        match self.kind {
            Kind::Yingtian => r(5.0 * 730_635.0 / 10_002.0),
            Kind::Qianyuan => r(5.0 * 214_764.0 / 2940.0),
            Kind::Yitian => r(3_688_970.0 / 10_100.0),
        }
    }

    /// 日躔先后定数（日法分，先 +）；t 为距天正冬至日数
    fn sun_def(&self, t: Real) -> Real {
        match self.kind {
            Kind::Yingtian => {
                let t = fmod(t, self.year().as_f64());
                let ding: Vec<f64> = YT_DING.iter().copied().chain(YT_DING.iter().rev().copied()).collect();
                let mut acc = 0.0;
                let mut k = 0;
                while k < 23 && acc + ding[k] <= t.as_f64() {
                    acc += ding[k];
                    k += 1;
                }
                let rem = r(acc + ding[k]) - t;
                let rate = YT_SUN_RATE[k];
                let adj = rem * r((100 - rate.abs()) as f64);
                let nxt = r(YT_SUN_JI[(k + 1) % 24] as f64);
                let loss = rate < 0;
                if (k < 12) == loss { nxt - adj } else { nxt + adj }
            }
            Kind::Qianyuan => {
                let qi = self.year() / r(24.0);
                let t = fmod(t, self.year().as_f64());
                let k = (t / qi).to_i64_floor() as usize;
                let fr = (t - qi * r(k as f64)) / qi;
                let kk = k % 12;
                let v = r(QY_YY[kk] as f64) + fr * r(QY_RATE[kk] as f64);
                if k < 12 { v } else { -v }
            }
            Kind::Yitian => {
                let half = YI_L1 + YI_L2;
                let t = fmod(t, 2.0 * half);
                let neg = t >= r(half);
                let u = if neg { t - r(half) } else { t };
                let (la, lb) = if neg { (YI_L2, YI_L1) } else { (YI_L1, YI_L2) };
                let a = 1836.0;
                let init = |n: f64, l: f64| a * (2.0 * n / l - n * n / (l * l));
                let end = |n: f64, l: f64| a * (1.0 - (n / l) * (n / l));
                let interp = |g: &dyn Fn(f64, f64) -> f64, n: Real, l: f64| {
                    let i = n.floor().as_f64();
                    let (p, q) = (g(i, l), g((i + 1.0).min(l), l));
                    r(p) + r(q - p) * (n - r(i))
                };
                let v = if u < r(la) { interp(&init, u, la) } else { interp(&end, u - r(la), lb) };
                if neg { -v } else { v }
            }
        }
    }

    /// 月离先后定数（日法分，先 +）；x 为入转日
    fn moon_def(&self, x: Real) -> Real {
        let x = fmod(x, self.zhuan().as_f64());
        match self.kind {
            Kind::Yingtian => {
                let lz = 551_201_242.0 / 10_000.0 * 5.0 / 10_002.0 / 2.0;
                let hou = x >= r(lz);
                let y = if hou { x - r(lz) } else { x };
                let rates = if hou { &YT_HOU } else { &YT_XIAN };
                let change = |v: i64| r(((1000 - v.abs()) * if v < 0 { 1 } else { -1 }) as f64);
                let k = (y.to_i64_floor() as usize).min(13);
                let mut m = r(0.0);
                for (j, rt) in rates.iter().enumerate().take(k) {
                    let _ = j;
                    m = m + match *rt {
                        One(v) => change(v),
                        Two(a, b) => change(a) + b.map_or(r(0.0), change),
                    };
                }
                let fr = y - r(k as f64);
                m = m + match rates[k] {
                    One(v) => change(v) * fr,
                    Two(a, b) => {
                        let chu = if k == 6 { 8888.0 / 10_002.0 } else { 7774.0 / 10_002.0 };
                        if fr <= r(chu) {
                            change(a) * fr / r(chu)
                        } else {
                            change(a) + b.map_or(r(0.0), |b| change(b) * (fr - r(chu)) / r(1.0 - chu))
                        }
                    }
                };
                if hou { -m } else { m }
            }
            Kind::Qianyuan => {
                let k = (x.to_i64_floor() as usize).min(27);
                let fr = x - r(k as f64);
                let mut v = r(0.0);
                for rt in QY_MOON.iter().take(k) {
                    v = v + match *rt {
                        One(a) => r(a as f64),
                        Two(a, b) => r((a + b.unwrap_or(0)) as f64),
                    };
                }
                let chu = |k: usize| match k {
                    6 => 2612.0,
                    13 => 2285.0,
                    20 => 1958.0,
                    _ => 1632.0,
                } / 2940.0;
                v + match QY_MOON[k] {
                    One(a) => r(a as f64) * fr,
                    Two(a, b) => {
                        let c = chu(k);
                        if fr <= r(c) {
                            r(a as f64) * fr / r(c)
                        } else {
                            r(a as f64) + r(b.unwrap_or(0) as f64) * (fr - r(c)) / r(1.0 - c)
                        }
                    }
                }
            }
            Kind::Yitian => {
                let lz = 2_783_010_165.0 / 10_000.0 / 2.0 / 10_100.0;
                let xiang = lz / 2.0;
                let chi = x >= r(lz);
                let y = if chi { x - r(lz) } else { x };
                let mo = y >= r(xiang);
                let z = if mo { y - r(xiang) } else { y };
                let j = (z.to_i64_floor() as usize).min(6);
                let fr = z - r(j as f64);
                let v = if !mo {
                    r(YI_UP[..j].iter().sum::<f64>()) + r(YI_UP[j]) * fr
                } else {
                    r(4031.0 - YI_DOWN[..j].iter().sum::<f64>()) - r(YI_DOWN[j]) * fr
                };
                if chi { -v } else { v }
            }
        }
    }

    /// 该日晨分（日法分）；t 为距天正冬至日数
    fn chen(&self, t: Real) -> Real {
        match self.kind {
            Kind::Yingtian => chen_interp(&YT_CHEN, t, self.year().as_f64() / 24.0),
            Kind::Qianyuan => chen_interp(&QY_CHEN, t, self.year().as_f64() / 24.0),
            Kind::Yitian => r(yitian_chen(t.as_f64())),
        }
    }
    fn erfen_chen(&self) -> Real {
        match self.kind {
            Kind::Yingtian => r(2350.0),
            Kind::Qianyuan => r(666.0),
            Kind::Yitian => r(yitian_chen(YI_L1)),
        }
    }
    /// 进朔限（日之分数）：3/4 − |二分晨分 − 该日晨分| / 2 / 日法
    fn jin_limit(&self, t_day: Real) -> Real {
        r(0.75) - (self.erfen_chen() - self.chen(t_day)).abs() / r(2.0) / r(self.f as f64)
    }
}

fn fmod(x: Real, m: f64) -> Real {
    x - (x / r(m)).floor() * r(m)
}

/// 应天、乾元晨分逐日：中率 = 相邻气差 / L（雨水至霜降前 16，余 15），合差 = 次中率 − 本中率，初率 = 中率 − 合差/2，日差 = 合差/L
fn chen_interp(tab: &[f64; 24], t: Real, qi: f64) -> Real {
    let t = fmod(t, qi * 24.0).as_f64();
    let k = (t / qi).floor() as usize;
    let n = (t - k as f64 * qi).floor() as usize;
    let l = |kk: usize| if (4..20).contains(&(kk % 24)) { 16.0 } else { 15.0 };
    let zl = |kk: usize| (tab[(kk + 1) % 24] - tab[kk % 24]) / l(kk);
    let he = zl(k + 1) - zl(k);
    let chu = zl(k) - he / 2.0;
    let rc = he / l(k);
    let mut v = tab[k];
    for j in 0..n {
        v += chu + j as f64 * rc;
    }
    r(v)
}

/// 仪天晷漏母（晨分）：入前后限日 u，(100u)² ÷ 昇法/平法，副之与 505 相减相乘、2850/5552 除，加副；春分后 1768 +，秋分后 2777 −
fn yitian_chen(t: f64) -> f64 {
    let t = t.rem_euclid(2.0 * YI_QIAN);
    let after_xia = t >= YI_QIAN;
    let x = if after_xia { t - YI_QIAN } else { t };
    let chu = if after_xia { YI_L2 } else { YI_L1 };
    let (u, fa, den, chunfen_hou) = if x <= chu {
        (x, if after_xia { 174_003.0 } else { 156_428.0 }, if after_xia { 5552.0 } else { 2850.0 }, after_xia)
    } else {
        (YI_QIAN - x, if after_xia { 156_428.0 } else { 174_003.0 }, if after_xia { 2850.0 } else { 5552.0 }, !after_xia)
    };
    let p = (100.0 * u) * (100.0 * u) / fa;
    let sy = p + (505.0 - p) * p / den;
    if chunfen_hou { 1768.0 + sy } else { 2777.0 - sy }
}

/// 冬至岁 y 天正冬至（儒略日数带分）：儒略历 y−1 年 12 月 16 日前后 30 日内干支相合之日
fn dongzhi_abs(li: &Li, y: i32) -> Real {
    let (d, _, _) = li.epoch(y);
    let di = d.to_i64_floor();
    let base = jul_jdn(y - 1, 12, 16);
    let j = (base - 30..=base + 30).find(|j| (j + 49).rem_euclid(60) == di).expect("冬至");
    r(j as f64) + (d - r(di as f64))
}

/// 冬至岁 y 自天正经朔起 14 月的进朔后朔日（JDN）
fn sui_days(li: &Li, y: i32) -> (Vec<i64>, Real) {
    let dz = dongzhi_abs(li, y);
    let (_, run, x0) = li.epoch(y);
    let jing0 = dz - run;
    let f = r(li.f as f64);
    let out = (0..14)
        .map(|m| {
            let mm = r(m as f64) * li.shuo();
            let jing = jing0 + mm;
            let ding = jing + (li.sun_def(jing - dz) + li.moon_def(x0 + mm)) / f;
            let day = ding.floor();
            let lim = li.jin_limit(day - dz);
            day.to_i64_floor() + i64::from(ding - day >= lim)
        })
        .collect();
    (out, dz)
}

fn sui_months(li: &Li, y: i32) -> Vec<(i64, u8, bool)> {
    let (a, _) = sui_days(li, y - 1);
    let (b, dz) = sui_days(li, y);
    let (c, dz2) = sui_days(li, y + 1);
    let qi = (dz2 - dz) / r(24.0);
    let zhong: Vec<i64> = (0..13).map(|k| (dz + r(2.0 * k as f64) * qi).to_i64_floor()).collect();
    let mut days: Vec<i64> = a.into_iter().chain(b).chain(c).collect();
    days.sort_unstable();
    days.dedup();
    let i0 = days.iter().rposition(|&d| d <= zhong[0]).expect("天正");
    let i1 = days.iter().rposition(|&d| d <= zhong[12]).expect("次年天正");
    let seq = &days[i0..=i1];
    let n = i1 - i0;
    let has: Vec<bool> = (0..n).map(|i| zhong.iter().any(|&z| seq[i] <= z && z < seq[i + 1])).collect();
    let leap = if n == 13 { (1..n).find(|&i| !has[i]) } else { None };
    let mut out: Vec<(i64, u8, bool)> = Vec::new();
    let mut mo = 11u8;
    for i in 0..n {
        if Some(i) == leap {
            let p = out.last().map_or(mo, |e| e.1);
            out.push((seq[i], p, true));
            continue;
        }
        out.push((seq[i], mo, false));
        mo = mo % 12 + 1;
    }
    out
}

/// 按历名推某农历年各月 (朔日 JDN, 月序, 闰否)，正月至十二月（含闰）；历名或年份不在适用年段返回 None
pub fn reckon_year(name: &str, lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    let (_, li, y0, y1) = LIS.iter().find(|(n, _, _, _)| *n == name)?;
    if !(*y0..=*y1).contains(&lunar_year) {
        return None;
    }
    let head = sui_months(li, lunar_year);
    let k = head.iter().position(|e| e.1 == 1 && !e.2)?;
    let mut out = head[k..].to_vec();
    out.extend(sui_months(li, lunar_year + 1).into_iter().take_while(|e| e.1 >= 11));
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 与 Python 复原逐月朔日相同（data/calendar/song_early_check.csv「月」行）
    #[test]
    fn matches_python_check() {
        let txt = std::fs::read_to_string(crate::repo::repo_root().join("data/calendar/song_early_check.csv")).unwrap();
        let mut want: std::collections::BTreeMap<(String, i32), Vec<(String, u8, bool)>> = Default::default();
        for line in txt.lines().filter(|l| !l.starts_with('#')).skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            if f[0] != "月" {
                continue;
            }
            want.entry((f[1].to_string(), f[2].parse().unwrap())).or_default().push((f[5].to_string(), f[3].parse().unwrap(), f[4] == "1"));
        }
        assert_eq!(want.len(), 60, "三历 20 + 17 + 23 年");
        for ((name, y), ms) in want {
            let got: Vec<(String, u8, bool)> = reckon_year(&name, y)
                .unwrap()
                .into_iter()
                .map(|(j, m, l)| {
                    let (yy, mm, dd) = crate::calendar::gregorian::civil_from_jdn(j);
                    (format!("{yy}-{mm:02}-{dd:02}"), m, l)
                })
                .collect();
            assert_eq!(got, ms, "{name} {y}");
        }
    }

    /// 应天天正冬至：冬至岁 964 为儒略历 963-12-17 丁丑（积年 4825558 为建隆三年壬戌之冬至岁）
    #[test]
    fn yingtian_epoch() {
        let dz = dongzhi_abs(&LIS[0].1, 964);
        let j = dz.to_i64_floor();
        assert_eq!(j, jul_jdn(963, 12, 17));
        assert_eq!((j + 49).rem_euclid(60), 13, "丁丑");
    }
}
