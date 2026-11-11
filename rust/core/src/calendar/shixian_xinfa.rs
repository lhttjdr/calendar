//! 清初时宪历「西洋新法」（汤若望，天聪戊辰元；顺治二年至雍正三年行用，康熙五至八年复用大统除外）定朔、节气、排月。
//!
//! 由 `scripts/calendar_data/shixian_xinfa.py` 移植。日躔照《新法算书》卷二十五、二十六日躔表说复原；月离照表：
//! 第一均数查卷三十三「太陰自行加減表」，太阴用时以「月離日差表」均之（卷三十二「月離表用法」：「太隂亦有日差表亦用以均其時，
//! 其加減則用本號」），朔望「止用一均數，故白道上所得實平行度即為視經度」。太阴平行、自行以《清史稿·时宪志一》所载日行、根数
//! 线性计算（恒年、日、时三表数字未转录）。依据与检验见 doc/calendar/calendar-compendium.md 第十一节。
//! 日序：t = JDN + 日内比例，以京师子正为日首；中法宫度自冬至起。

use crate::math::real::{real, Real, RealOps};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn r(x: f64) -> Real {
    real(x)
}
fn dms(deg: f64, m: f64, s: f64, t: f64, q: f64, w: f64) -> f64 {
    deg + m / 60.0 + s / 3600.0 + t / 216_000.0 + q / 12_960_000.0 + w / 777_600_000.0
}

// ---------------------------------------------------------------- 日躔常数（新法算书卷二十五、二十六；清史稿卷四十五）
const SUI: f64 = 365.242_188_6 + 0.4e-8;
const ROOT_JDN: i64 = 2_315_666;
const ZUIGAO_YEAR: f64 = 45.0 / 3600.0;
const SUN_R: f64 = 100_000.0;
const SUN_A: f64 = 1792.0;

fn sun_daily() -> f64 {
    dms(0.0, 59.0, 8.0, 19.0, 49.0, 36.0)
}
fn root_l() -> f64 {
    dms(0.0, 53.0, 35.0, 39.0, 0.0, 0.0)
}
fn root_gao() -> f64 {
    dms(5.0, 59.0, 59.0, 0.0, 0.0, 0.0)
}
fn huang_chi() -> f64 {
    dms(23.0, 31.0, 30.0, 0.0, 0.0, 0.0)
}
// 太阴（清史稿卷四十五）
fn moon_daily() -> f64 {
    dms(13.0, 10.0, 35.0, 1.0, 0.0, 0.0)
}
fn moon_zixing() -> f64 {
    dms(13.0, 3.0, 53.0, 56.0, 0.0, 0.0)
}
fn root_moon() -> f64 {
    180.0 + dms(1.0, 50.0, 54.0, 46.0, 0.0, 0.0)
}
fn root_zixing() -> f64 {
    180.0 + dms(25.0, 32.0, 15.0, 34.0, 0.0, 0.0)
}

fn rad(d: Real) -> Real {
    d * Real::pi() / r(180.0)
}
fn sind(d: Real) -> Real {
    rad(d).sin()
}
fn cosd(d: Real) -> Real {
    rad(d).cos()
}
fn deg(x: Real) -> Real {
    x * r(180.0) / Real::pi()
}
fn norm(d: Real) -> Real {
    let x = d - (d / r(360.0)).floor() * r(360.0);
    if x >= r(360.0) { x - r(360.0) } else { x }
}
fn signed(d: Real) -> Real {
    norm(d + r(180.0)) - r(180.0)
}
fn fold(d: Real) -> Real {
    let d = norm(d);
    if d <= r(180.0) { d } else { r(360.0) - d }
}

/// 加减均数（度，正值）：phi 为距最高之度（0—180）。卷二十六两三角形之角相并
fn sun_jun_from_gao(phi: Real) -> Real {
    let p = r(SUN_A) * sind(phi);
    let q = r(SUN_A) * cosd(phi);
    let a1 = deg((p / r(SUN_R)).asin());
    let yi_xin = (r(SUN_R * SUN_R) - p * p).sqrt() + q;
    let a2 = deg(p.atan2(yi_xin + q));
    a1 + a2
}

/// 平时 t 之 (均数, 实行)
fn sun(t: Real) -> (Real, Real) {
    let n = t - r(ROOT_JDN as f64);
    let l = norm(r(root_l()) + r(sun_daily()) * n);
    let chong = r(root_gao()) + r(ZUIGAO_YEAR) * n / r(SUI);
    let yin = norm(l - chong);
    let e = sun_jun_from_gao(r(180.0) - fold(yin));
    let e = if norm(yin) <= r(180.0) { e } else { -e };
    (e, norm(l + e))
}

fn ri_cha_raw(t: Real) -> Real {
    let (e, lam) = sun(t);
    let lm = norm(lam + r(270.0));
    let ra = norm(deg((cosd(r(huang_chi())) * sind(lm)).atan2(cosd(lm))));
    (signed(lm - ra) - e) / r(360.0)
}

fn chunfen_scan(t: Real) -> Real {
    let mut d = t.to_i64_floor();
    for _ in 0..400 {
        let a = signed(sun(r(d as f64)).1 - r(90.0));
        let b = signed(sun(r((d + 1) as f64)).1 - r(90.0));
        if a < r(0.0) && b >= r(0.0) {
            return r(d as f64) - a / (b - a);
        }
        d -= 1;
    }
    panic!("春分")
}

fn chunfen_of(i: i64) -> Real {
    static C: OnceLock<Mutex<HashMap<i64, Real>>> = OnceLock::new();
    let m = C.get_or_init(Default::default);
    if let Some(v) = m.lock().unwrap().get(&i) {
        return *v;
    }
    let base = ROOT_JDN as f64 + 98.0 + round_half_even(i as f64 * SUI) + 8.0;
    let v = chunfen_scan(r(base));
    m.lock().unwrap().insert(i, v);
    v
}

fn chunfen_near(t: Real) -> Real {
    let mut i = ((t - r(ROOT_JDN as f64) - r(88.0)) / r(SUI)).to_i64_floor();
    while chunfen_of(i) > t {
        i -= 1;
    }
    while chunfen_of(i + 1) <= t {
        i += 1;
    }
    chunfen_of(i)
}

/// 日差（日）：「從春分起算」，加于平时得用时
fn ri_cha(t: Real) -> Real {
    ri_cha_raw(t) - ri_cha_raw(chunfen_near(t))
}

/// Python round：四舍六入五成双
fn round_half_even(x: f64) -> f64 {
    let f = x.floor();
    let d = x - f;
    if d > 0.5 || (d == 0.5 && (f as i64) % 2 != 0) { f + 1.0 } else { f }
}

// ---------------------------------------------------------------- 月离表
struct Tabs {
    /// 太阴自行加减表：自行 0′—10800′ 每 10′（秒）
    zixing: Vec<f64>,
    /// 月离日差表：0°—90°（秒，时）
    richa: Vec<f64>,
}

fn rows(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines().filter(|l| !l.starts_with('#')).skip(1).map(|l| l.split(',').collect())
}

fn tabs() -> &'static Tabs {
    static T: OnceLock<Tabs> = OnceLock::new();
    T.get_or_init(|| {
        let p1 = include_str!("../../../../data/calendar/xinfa_yueli_tables/zixing_part1.csv");
        let mut v: HashMap<i64, f64> = HashMap::new();
        let mut doubt = vec![];
        for f in rows(p1) {
            let k = f[3].parse::<i64>().unwrap() * 1800 + f[4].parse::<i64>().unwrap() * 60 + f[5].parse::<i64>().unwrap();
            let val = f[8].parse::<f64>().unwrap() * 3600.0 + f[9].parse::<f64>().unwrap() * 60.0 + f[10].parse::<f64>().unwrap();
            v.insert(k, val);
            if f[12] == "1" {
                doubt.push(k);
            }
        }
        // 存疑格以同行相邻两格之中数代
        for k in doubt {
            let m = round_half_even((v[&(k - 10)] + v[&(k + 10)]) / 2.0);
            v.insert(k, m);
        }
        let zixing = (0..=1080).map(|i| v[&(i * 10)]).collect();
        let p2 = include_str!("../../../../data/calendar/xinfa_yueli_tables/zixing_part2.csv");
        let mut rc: HashMap<i64, f64> = HashMap::new();
        for f in rows(p2) {
            if f[0] != "月离日差表" {
                continue;
            }
            let g = match f[3] {
                "星纪/鹑首" => 0,
                "玄枵/鹑火" => 1,
                "娵訾/鹑尾" => 2,
                _ => continue,
            };
            let x = g * 30 + f[4].parse::<i64>().unwrap();
            rc.insert(x, f[9].parse::<f64>().unwrap() * 60.0 + f[10].parse::<f64>().unwrap());
        }
        let richa = (0..=90).map(|x| rc[&x]).collect();
        Tabs { zixing, richa }
    })
}

/// 中比例：tab[i] 为引数 i·step 之值
fn interp(tab: &[f64], x: Real, step: f64) -> Real {
    let i = ((x / r(step)).to_i64_floor() as usize).min(tab.len() - 2);
    let f = x / r(step) - r(i as f64);
    r(tab[i]) + r(tab[i + 1] - tab[i]) * f
}

/// 第一均数（度）：顺数○至五宫减，逆数六至十一宫加
fn moon_jun1(a: Real) -> Real {
    let v = interp(&tabs().zixing, fold(a) * r(60.0), 10.0) / r(3600.0);
    if norm(a) <= r(180.0) { -v } else { v }
}

/// 月离日差（日）：引数太阳实行；冬至至春分、夏至至秋分为顺数，加；余为逆数，减
fn moon_richa(lam: Real) -> Real {
    let x = norm(lam);
    let q = (x / r(90.0)).to_i64_floor();
    let rem = x - r(q as f64 * 90.0);
    let shun = q.rem_euclid(2) == 0;
    let arg = if shun { rem } else { r(90.0) - rem };
    let sec = interp(&tabs().richa, arg, 1.0);
    (if shun { sec } else { -sec }) / r(86400.0)
}

/// 用时 t 之 (太阴实平行, 太阳实行)
fn moon_sun_yong(t: Real) -> (Real, Real) {
    let lam = sun(t - ri_cha(t)).1;
    let tm = t + moon_richa(lam);
    let n = tm - r(ROOT_JDN as f64);
    let l = norm(r(root_moon()) + r(moon_daily()) * n);
    let a = norm(r(root_zixing()) + r(moon_zixing()) * n);
    (norm(l + moon_jun1(a)), lam)
}

// ---------------------------------------------------------------- 节气、定朔、排月
fn cross_sun(target: f64, mut d: i64) -> Real {
    for _ in 0..800 {
        let a = signed(sun(r(d as f64)).1 - r(target));
        let b = signed(sun(r((d + 1) as f64)).1 - r(target));
        if a < r(0.0) && b >= r(0.0) {
            return r(d as f64) - a / (b - a);
        }
        d += 1;
    }
    panic!("交气")
}

fn dongzhi(y: i32) -> Real {
    let j = ROOT_JDN + round_half_even((y - 1628) as f64 * SUI) as i64;
    cross_sun(0.0, j - 8)
}

fn shuo_times(d0: i64, d1: i64) -> Vec<Real> {
    let el = |d: i64| {
        let (m, s) = moon_sun_yong(r(d as f64));
        signed(m - s)
    };
    let mut out = vec![];
    let mut e_d = el(d0);
    for d in d0..d1 {
        let e_n = el(d + 1);
        if e_d < r(0.0) && e_n >= r(0.0) {
            out.push(r(d as f64) - e_d / (e_n - e_d));
        }
        e_d = e_n;
    }
    out
}

/// 冬至岁 y：(朔日 JDN, 月序, 闰否)，自天正至次年天正前；中气、定朔皆以用时定日，无中气之月为闰
pub fn sui_months(y: i32) -> Vec<(i64, u8, bool)> {
    let (dz0, dz1) = (dongzhi(y), dongzhi(y + 1));
    let f0 = dz0.to_i64_floor();
    let mut zday: Vec<i64> = (0..12)
        .map(|k| {
            let t = cross_sun(30.0 * k as f64, f0 + (30.4 * k as f64) as i64 - 3);
            (t + ri_cha(t)).to_i64_floor()
        })
        .collect();
    zday.push((dz1 + ri_cha(dz1)).to_i64_floor());
    let days: Vec<i64> = shuo_times(f0 - 32, dz1.to_i64_floor() + 32).iter().map(|t| t.to_i64_floor()).collect();
    let i0 = days.iter().rposition(|&d| d <= zday[0]).expect("天正");
    let i1 = days.iter().rposition(|&d| d <= zday[12]).expect("次年天正");
    let ms = &days[i0..=i1];
    let n = i1 - i0;
    let has: Vec<bool> = (0..n).map(|i| zday.iter().any(|&z| ms[i] <= z && z < ms[i + 1])).collect();
    let leap = if n == 13 { (1..n).find(|&i| !has[i]) } else { None };
    let mut out: Vec<(i64, u8, bool)> = Vec::with_capacity(n);
    let mut mo = 11u8;
    for i in 0..n {
        if Some(i) == leap {
            let prev = out.last().map_or(mo, |e| e.1);
            out.push((ms[i], prev, true));
            continue;
        }
        out.push((ms[i], mo, false));
        mo = mo % 12 + 1;
    }
    out
}

/// 农历年 N 各月 (朔日 JDN, 月序, 闰否)，正月至十二月（含闰）
pub fn year_months(lunar_year: i32) -> Vec<(i64, u8, bool)> {
    let head = sui_months(lunar_year);
    let k = head.iter().position(|e| e.1 == 1 && !e.2).expect("正月");
    let mut out = head[k..].to_vec();
    out.extend(sui_months(lunar_year + 1).into_iter().take_while(|e| e.1 >= 11));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 卷三十二月离假如：第一例（崇祯四年十月十五日乙卯夜望，食甚丙辰用时 6:51）日差 24 分加、太阴日差 9′56″ 减、日月相望；
    /// 第二例（崇祯五年三月癸丑夜望，用时 20:14）日差 11 分加、太阴日差 10 分减，月经「大火十四度二十七分」
    #[test]
    fn xinfa_jiaru_examples() {
        let t1 = r(2_317_083.0) + r((6.0 + 51.0 / 60.0) / 24.0);
        assert!((ri_cha(t1).as_f64() * 1440.0 - 24.0).abs() < 0.5);
        let (m1, s1) = moon_sun_yong(t1);
        assert!((moon_richa(s1).as_f64() * 86400.0 + 596.0).abs() < 3.0, "9′56″ 减");
        assert!((fold(m1 - s1).as_f64() - 180.0).abs() < 0.02, "望");
        let t2 = r(2_317_260.0) + r((20.0 + 14.0 / 60.0) / 24.0);
        assert!((ri_cha(t2).as_f64() * 1440.0 - 11.0).abs() < 0.7);
        let (m2, _) = moon_sun_yong(t2);
        let want = 300.0 + 14.0 + 27.0 / 60.0;
        assert!((m2.as_f64() - want).abs() < 1.0 / 60.0, "{}", m2.as_f64());
    }

    /// 与 Python 复原（data/calendar/shixian_xinfa_check.csv 用时朔日）逐月相同：冬至岁 1645—1729
    #[test]
    fn matches_python_check() {
        let txt = include_str!("../../../../data/calendar/shixian_xinfa_check.csv");
        let mut want: std::collections::BTreeMap<i32, Vec<String>> = Default::default();
        for f in rows(txt) {
            want.entry(f[0].parse().unwrap()).or_default().push(f[4].to_string());
        }
        for (y, days) in want {
            let got: Vec<String> = sui_months(y)
                .iter()
                .map(|e| {
                    let (yy, mm, dd) = crate::calendar::gregorian::civil_from_jdn(e.0);
                    format!("{yy}-{mm:02}-{dd:02}")
                })
                .collect();
            assert_eq!(got, days, "冬至岁 {y}");
        }
    }
}
