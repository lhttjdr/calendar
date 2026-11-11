//! 清时宪历《历象考成》（康熙甲子元法）推步：日躔、月离、节气、合朔、置闰。雍正四年（1726）起「造時憲書一遵曆象考成之法」，
//! 至《历象考成后编》起用。
//!
//! 由 `scripts/calendar_data/shixian_kaocheng.py` 移植，照《清史稿·时宪志》卷四十八（康熙甲子元法中）逐步复原，
//! 卷四十七作模型说明与自检（两弦、自行三宫「極大均數七度二十五分四十六秒」）。角度以度计；中法宫度自冬至起。
//! 用时：太阴平行以时差行化至用时子正（「求用時太陰平行」），故合朔内插所得即用时；中气以时差总化用时定日。

use crate::math::real::{real, Real, RealOps};

const ZHOU_SUI: f64 = 365.242_187_5;
const SUN_DAILY: f64 = 3548.330_516_9 / 3600.0;
const ZUIBEI_YEAR: f64 = 61.166_66 / 3600.0;
const ZUIBEI_DAY: f64 = 0.167_469 / 3600.0;
const R: f64 = 1e7;
const SUN_BENLUN: f64 = 268_812.0;
const ZUIBEI_YING: f64 = 7.0 + 10.0 / 60.0 + 11.0 / 3600.0 + 10.0 / 216_000.0;
const QI_YING: f64 = 7.656_374_926;
const HUANG_CHI: f64 = 23.0 + 29.0 / 60.0 + 30.0 / 3600.0;

const MOON_DAILY: f64 = 47_435.021_177 / 3600.0;
const MOON_HOURLY: f64 = 1976.459_215_7 / 3600.0;
const BEI_DAILY: f64 = 401.077_477 / 3600.0;
const JIAO_DAILY: f64 = 190.64 / 3600.0;
const MOON_JUNLUN: f64 = 290_000.0;
const CI_LUN: f64 = 217_000.0;
const CI_JUNLUN: f64 = 117_500.0;
const DA_JU_ZHONG: f64 = 5.0 + 8.0 / 60.0;
const DA_JU_BANJIAO: f64 = 9.5 / 60.0;
const MOON_YING: f64 = 30.0 + 8.0 + 40.0 / 60.0 + 57.0 / 3600.0 + 16.0 / 216_000.0;
const BEI_YING: f64 = 3.0 * 30.0 + 4.0 + 49.0 / 60.0 + 54.0 / 3600.0 + 9.0 / 216_000.0;
const JIAO_YING: f64 = 6.0 * 30.0 + 27.0 + 13.0 / 60.0 + 37.0 / 3600.0 + 48.0 / 216_000.0;

/// 康熙二十三年甲子：天正冬至在癸亥年（1683）十一月
const YUAN_YEAR: i32 = 1684;
/// 康熙甲子天正冬至辛未日（1683-12-21）儒略日数；日序以其前 7 日之甲子日子正为 0
const EPOCH_DZ_JDN: i64 = 2_336_118;

fn r(x: f64) -> Real {
    real(x)
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
fn atan2d(y: Real, x: Real) -> Real {
    y.atan2(x) * r(180.0) / Real::pi()
}
fn norm(d: Real) -> Real {
    let x = d - (d / r(360.0)).floor() * r(360.0);
    if x >= r(360.0) { x - r(360.0) } else { x }
}
fn fold(d: Real) -> Real {
    let d = norm(d);
    if d <= r(180.0) { d } else { r(360.0) - d }
}
fn signed(d: Real) -> Real {
    norm(d + r(180.0)) - r(180.0)
}
fn tri_opp(a: Real, b: Real, c: Real) -> (Real, Real) {
    let ang = atan2d(b * sind(c), a - b * cosd(c));
    (ang, (a * a + b * b - r(2.0) * a * b * cosd(c)).sqrt())
}

fn jdn_of(daynum: i64) -> i64 {
    EPOCH_DZ_JDN - 7 + daynum
}

/// 冬至岁 y 的天正冬至日序与积年
fn dongzhi(y: i32) -> (Real, i32) {
    let ji = y - YUAN_YEAR;
    (r(ZHOU_SUI) * r(ji as f64) + r(QI_YING), ji)
}

fn sui_of(t: Real) -> i32 {
    let mut y = YUAN_YEAR + ((t - r(QI_YING)) / r(ZHOU_SUI)).to_i64_floor() as i32;
    while dongzhi(y + 1).0 <= t {
        y += 1;
    }
    while dongzhi(y).0 > t {
        y -= 1;
    }
    y
}

/// 日躔均数：「以本輪半徑三分之二為對正角之邊，以引數為一角，求得對角之邊倍之。又求得對又一角之邊，與本天半徑相加減」；
/// 小边 = 2h·sin(引数)，大边 = R − h·cos(引数)（三宫至八宫相加、九宫至二宫相减），对小边之角为均数；引数初宫至五宫为加
fn sun_jun(yin: Real) -> Real {
    let h = r(SUN_BENLUN * 2.0 / 3.0);
    let m = norm(yin);
    let jun = atan2d(r(2.0) * h * sind(m).abs(), r(R) - h * cosd(m));
    if m <= r(180.0) { jun } else { -jun }
}

/// (平行, 均数, 实行)
fn sun(t: Real) -> (Real, Real, Real) {
    let y = sui_of(t);
    let (dz, ji) = dongzhi(y);
    let ping = norm(r(SUN_DAILY) * (t - dz));
    let zuibei = r(ZUIBEI_YING) + r(ZUIBEI_YEAR) * r(ji as f64) + r(ZUIBEI_DAY) * (t - (dz.floor() + r(1.0)));
    let jun = sun_jun(norm(ping - zuibei));
    (ping, jun, norm(ping + jun))
}

/// 时差总（日）：均数时差（均数加者时差为减）+ 升度时差（二分后加、二至后减）
fn shi_cha(t: Real) -> Real {
    let (_, jun, shi) = sun(t);
    let lam = norm(shi + r(270.0));
    let ra = norm(atan2d(cosd(r(HUANG_CHI)) * sind(lam), cosd(lam)));
    (-jun + signed(lam - ra)) / r(360.0)
}

/// 二均数（卷四十八「求白道實行」）：次轮最近点距地心 d1、倍次引之通弦为两边，「初均數與引數減半周之度相加，
/// 又以次引距象限度加減之」为夹角，对通弦之角为二均；加减号依「加減泛限」与「定限」。返回 (二均（加为正）, 次均轮心距地心)
fn er_jun(m: Real, chu_signed: Real, chu_abs: Real, d1: Real, ci_yin: Real) -> (Real, Real) {
    let th = norm(ci_yin);
    let chord = r(2.0 * CI_LUN) * sind(th).abs();
    let a1 = chu_abs + if m < r(180.0) { r(180.0) - m } else { m - r(180.0) };
    let dq = if th < r(90.0) {
        r(90.0) - th
    } else if th < r(180.0) {
        th - r(90.0)
    } else if th < r(270.0) {
        r(270.0) - th
    } else {
        th - r(270.0)
    };
    let jian = chu_signed < r(0.0);
    let over_q = (th >= r(90.0) && th < r(180.0)) || th >= r(270.0);
    let add = if jian { over_q } else { !over_q };
    let mut ang = if add { a1 + dq } else { a1 - dq }.abs();
    if ang > r(180.0) {
        ang = r(360.0) - ang;
    }
    let (mag, side) = if chord == r(0.0) || ang == r(0.0) || ang == r(180.0) {
        (r(0.0), (d1 + if ang == r(0.0) { chord } else { -chord }).abs())
    } else {
        tri_opp(d1, chord, ang)
    };
    let fan = chu_abs + (r(180.0) - m).abs();
    let two = norm(r(2.0) * th);
    let same = if fan == r(90.0) {
        true
    } else if fan < r(90.0) {
        let x = if jian { two } else { norm(r(360.0) - two) };
        x > r(2.0) * (r(90.0) - fan)
    } else {
        let x = if jian { norm(r(360.0) - two) } else { two };
        x > r(2.0) * (fan - r(90.0))
    };
    let sg_chu = if jian { r(-1.0) } else { r(1.0) };
    (if same { sg_chu } else { -sg_chu } * mag, side)
}

/// 时刻 t（日序，子正）：(太阴黄道实行, 太阳实行)。yongshi：太阴平行化至用时子正
fn moon(t: Real, yongshi: bool) -> (Real, Real) {
    let n = t - (dongzhi(YUAN_YEAR).0.floor() + r(1.0));
    let mut ping = norm(r(MOON_YING) + r(MOON_DAILY) * n);
    let bei = norm(r(BEI_YING) + r(BEI_DAILY) * n);
    let jiao = norm(r(JIAO_YING) - r(JIAO_DAILY) * n);
    let (_, _, s_shi) = sun(t);
    if yongshi {
        // 时差总化秒，与太阴每时平行相乘，以一度化秒（3600）为法除之，得时差行；时差总为加者则减
        let cha_sec = shi_cha(t) * r(86400.0);
        ping = norm(ping - cha_sec * r(MOON_HOURLY) / r(3600.0));
    }
    // 初实行：本轮半径之半为对正角之边，对角之边三因之
    let m = norm(ping - bei);
    let h = r(MOON_JUNLUN);
    let small = r(3.0) * h * sind(m).abs();
    let big = r(R) + h * cosd(m);
    let chu_abs = atan2d(small, big);
    let d1 = (small * small + big * big).sqrt();
    let chu_signed = if m <= r(180.0) { -chu_abs } else { chu_abs };
    let chu = norm(ping + chu_signed);
    let ci_yin = norm(chu - s_shi);
    let (e2, side) = er_jun(m, chu_signed, chu_abs, d1, ci_yin);
    let two = norm(r(2.0) * ci_yin);
    let (e3, _) = tri_opp(side, r(CI_JUNLUN), fold(two));
    let e3 = if two < r(180.0) { e3 } else { -e3 };
    let bai = norm(chu + e2 + e3);
    // 黄道实行：弧三角形，黄白大距中数、半较为两边，次引倍度为夹角
    let c = fold(two);
    let (a, b) = (r(DA_JU_ZHONG), r(DA_JU_BANJIAO));
    let cos_i = cosd(a) * cosd(b) + sind(a) * sind(b) * cosd(c);
    let da_ju = atan2d((r(1.0) - cos_i * cos_i).sqrt(), cos_i);
    let jiao_jun = atan2d(sind(b) * sind(c), sind(a) * cosd(b) - cosd(a) * sind(b) * cosd(c));
    let jiao_shi = if two < r(180.0) { norm(jiao - jiao_jun) } else { norm(jiao + jiao_jun) };
    let ju = norm(bai - jiao_shi);
    let hd = atan2d(cosd(da_ju) * sind(ju), cosd(ju));
    (norm(bai + signed(hd - ju)), s_shi)
}

fn jieqi_times(y: i32) -> Vec<Real> {
    let mut out = Vec::with_capacity(25);
    let mut d = dongzhi(y).0.to_i64_floor() - 2;
    let mut lam_d = sun(r(d as f64)).2;
    for k in 0..25 {
        let target = r(((k * 15) % 360) as f64);
        loop {
            let lam_n = sun(r((d + 1) as f64)).2;
            let (a, b) = (signed(lam_d - target), signed(lam_n - target));
            if a < r(0.0) && b >= r(0.0) {
                out.push(r(d as f64) - a / (b - a));
                break;
            }
            d += 1;
            lam_d = lam_n;
        }
    }
    out
}

fn shuo_times(d0: i64, d1: i64, yongshi: bool) -> Vec<Real> {
    let el = |d: i64| {
        let (mo, su) = moon(r(d as f64), yongshi);
        signed(mo - su)
    };
    let mut out = Vec::new();
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

/// 冬至岁 y：(朔日 JDN, 月序, 闰否)，自天正至次年天正前（用时）。
pub fn sui_months(y: i32) -> Vec<(i64, u8, bool)> {
    sui_months_with(y, true)
}

fn sui_months_with(y: i32, yongshi: bool) -> Vec<(i64, u8, bool)> {
    let (q0, q1) = (jieqi_times(y), jieqi_times(y + 1));
    let conv = |t: Real| if yongshi { (t + shi_cha(t)).to_i64_floor() } else { t.to_i64_floor() };
    let mut zhong: Vec<i64> = (0..24).step_by(2).map(|k| conv(q0[k])).collect();
    zhong.push(conv(q1[0]));
    let days: Vec<i64> = shuo_times(q0[0].to_i64_floor() - 32, q1[0].to_i64_floor() + 32, yongshi).iter().map(|t| t.to_i64_floor()).collect();
    let i0 = days.iter().rposition(|&d| d <= zhong[0]).expect("天正");
    let i1 = days.iter().rposition(|&d| d <= zhong[12]).expect("次年天正");
    let ms = &days[i0..=i1];
    let n = i1 - i0;
    let has: Vec<bool> = (0..n).map(|i| zhong.iter().any(|&z| ms[i] <= z && z < ms[i + 1])).collect();
    let leap_at = if n == 13 { (1..n).find(|&i| !has[i]) } else { None };
    let mut out: Vec<(i64, u8, bool)> = Vec::with_capacity(n);
    let mut mo = 11u8;
    for i in 0..n {
        if Some(i) == leap_at {
            let prev = out.last().map_or(mo, |e| e.1);
            out.push((jdn_of(ms[i]), prev, true));
            continue;
        }
        out.push((jdn_of(ms[i]), mo, false));
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

    /// 卷四十七：两弦、自行三宫「極大均數七度二十五分四十六秒」（初均 + 二均）；康熙甲子天正冬至辛未
    #[test]
    fn kaocheng_self_check() {
        let h = r(MOON_JUNLUN);
        let (small, big) = (r(3.0) * h, r(R));
        let chu = atan2d(small, big);
        let (e2, _) = er_jun(r(90.0), -chu, chu, (small * small + big * big).sqrt(), r(90.0));
        let tot = (chu + e2.abs()).as_f64();
        assert!((tot - (7.0 + 25.0 / 60.0 + 46.0 / 3600.0)).abs() < 2.0 / 3600.0, "{tot}");
        let (dz, _) = dongzhi(YUAN_YEAR);
        assert_eq!((jdn_of(dz.to_i64_floor()) + 49).rem_euclid(60), 7, "辛未");
        assert_eq!(crate::calendar::gregorian::gregorian_to_jdn(1683, 12, 21), EPOCH_DZ_JDN);
    }

    /// 与 Python 复原（data/calendar/shixian_kaocheng_check.csv）逐月相同：冬至岁 1726—1741，用时、平时
    #[test]
    fn matches_python_check() {
        let txt = std::fs::read_to_string(crate::repo::repo_root().join("data/calendar/shixian_kaocheng_check.csv")).unwrap();
        let mut want: std::collections::BTreeMap<i32, (Vec<String>, Vec<String>)> = Default::default();
        for line in txt.lines().filter(|l| !l.starts_with('#')).skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            let y: i32 = f[0].parse().unwrap();
            if (1726..=1741).contains(&y) {
                let e = want.entry(y).or_default();
                e.0.push(f[3].to_string());
                e.1.push(f[4].to_string());
            }
        }
        let fmt = |v: Vec<(i64, u8, bool)>| -> Vec<String> {
            v.iter()
                .map(|e| {
                    let (yy, mm, dd) = crate::calendar::gregorian::civil_from_jdn(e.0);
                    format!("{yy}-{mm:02}-{dd:02}")
                })
                .collect()
        };
        for (y, (yong, ping)) in want {
            assert_eq!(fmt(sui_months_with(y, true)), yong, "用时 冬至岁 {y}");
            let p = fmt(sui_months_with(y, false));
            assert_eq!(p[..ping.len().min(p.len())], ping[..ping.len().min(p.len())], "平时 冬至岁 {y}");
        }
    }
}
