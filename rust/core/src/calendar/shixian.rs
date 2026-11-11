//! 清时宪历《历象考成后编》（雍正癸卯元法）推步：日躔、月离、节气、合朔、置闰。
//!
//! 由 `scripts/calendar_data/shixian_houbian.py` 移植，照《清史稿·时宪志》卷五十（雍正癸卯元法）与卷四十八
//! （康熙甲子元法，后编言「同甲子元法」诸条）逐步复原；依据、检验见 doc/calendar/calendar-compendium.md 第十一节。
//! 角度以度计；中法宫度自冬至起（= 现代黄经 − 270°）。日以京师平子正为界；合朔、中气化为用时（视太阳时）定日。

use crate::math::real::{real, Real, RealOps};

// ---------------------------------------------------------------- 用数（卷五十）
const ZHOU_SUI: f64 = 365.242_334_42;
const SUN_DAILY: f64 = 3548.329_089_7 / 3600.0;
const ZUIBEI_YEAR: f64 = 62.9975 / 3600.0;
const ZUIBEI_DAY: f64 = 0.17248 / 3600.0;
const SUN_E: f64 = 169_000.0;
const SUN_B: f64 = 9_998_571.85;
const ZUIBEI_YING: f64 = 8.0 + 7.0 / 60.0 + 32.0 / 3600.0 + 22.0 / 216_000.0;
const QI_YING: f64 = 32.122_54;
const HUANG_CHI: f64 = 23.0 + 29.0 / 60.0;

const MOON_DAILY: f64 = 47_435.023_408_6 / 3600.0;
const GAO_DAILY: f64 = 401.070_226 / 3600.0;
const JIAO_DAILY: f64 = 190.638_63 / 3600.0;
const SUN_MAX_JUN: f64 = 6973.0 / 3600.0;
const YI_PING: f64 = 710.0 / 3600.0;
const GAO_PING: f64 = 1196.0 / 3600.0;
const JIAO_PING: f64 = 570.0 / 3600.0;
const CUBE_GAO: f64 = 1_051_562.0;
const CUBE_DA: f64 = 101_410.0;
const ER_PING_GAO: f64 = 214.0 / 3600.0;
const ER_PING_BEI: f64 = 236.0 / 3600.0;
const SAN_PING: f64 = 47.0 / 3600.0;
const GAO_BENLUN: f64 = 550_505.0;
const GAO_JUNLUN: f64 = 117_315.0;
const ER_JUN_GAO: f64 = 1994.0 / 3600.0;
const ER_JUN_BEI: f64 = 2231.0 / 3600.0;
const SAN_JUN: f64 = 145.0 / 3600.0;
/// 两最高相距 10°…90°，两弦最大末均（秒）
const MO_JUN: [f64; 9] = [61.0, 67.0, 76.0, 88.0, 103.0, 120.0, 139.0, 159.0, 180.0];
const JIAO_BENLUN: f64 = 57.5 / 60.0;
const JIAO_JUNLUN: f64 = 1.5 / 60.0;
const DA_JU_MAX: f64 = 5.0 + 17.0 / 60.0 + 20.0 / 3600.0;
const DA_JU_BANJIAO: f64 = 532.5 / 3600.0;
const JU_RI_JIAFEN_MAX: f64 = 163.0 / 3600.0;
const MOON_YING: f64 = 5.0 * 30.0 + 26.0 + 27.0 / 60.0 + 48.0 / 3600.0 + 53.0 / 216_000.0;
const GAO_YING: f64 = 8.0 * 30.0 + 1.0 + 15.0 / 60.0 + 45.0 / 3600.0 + 38.0 / 216_000.0;
const JIAO_YING: f64 = 5.0 * 30.0 + 22.0 + 57.0 / 60.0 + 37.0 / 3600.0 + 33.0 / 216_000.0;

/// 雍正元年癸卯：天正冬至在壬寅年（1722）十一月
const YUAN_YEAR: i32 = 1723;
/// 雍正癸卯天正冬至丙申日（1722-12-22）儒略日数；日序以其前 32 日之甲子日子正为 0
const EPOCH_DZ_JDN: i64 = 2_350_363;

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
/// 入 0—180°（与全周相减用其余）
fn fold(d: Real) -> Real {
    let d = norm(d);
    if d <= r(180.0) { d } else { r(360.0) - d }
}
/// 入 (−180°, 180°]
fn signed(d: Real) -> Real {
    norm(d + r(180.0)) - r(180.0)
}
/// 平三角形：两边 a、b 夹角 C，求对 b 之角（度）与第三边
fn tri_opp(a: Real, b: Real, c: Real) -> (Real, Real) {
    let ang = atan2d(b * sind(c), a - b * cosd(c));
    let side = (a * a + b * b - r(2.0) * a * b * cosd(c)).sqrt();
    (ang, side)
}

/// 冬至岁 y（天正冬至在 y−1 年十一月）的天正冬至日序与积年：「置周歲，以距元年數減一得積年乘之，得中積分，加氣應得通積分」
fn dongzhi(y: i32) -> (Real, i32) {
    let ji = y - YUAN_YEAR;
    (r(ZHOU_SUI) * r(ji as f64) + r(QI_YING), ji)
}

fn jdn_of(daynum: i64) -> i64 {
    EPOCH_DZ_JDN - 32 + daynum
}
#[cfg(test)]
fn daynum_of(jdn: i64) -> i64 {
    jdn - (EPOCH_DZ_JDN - 32)
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

/// 日躔：(平行, 引数, 均数, 实行, 日距地心/千万)
struct Sun {
    ping: Real,
    yin: Real,
    jun: Real,
    shi: Real,
    dist: Real,
}

/// 日躔均数（卷五十「求實行」）：二千万、倍两心差、引数解平三角形，对倍两心差之角倍之为橢圓界角；
/// 以小、大半径与引数正切得橢圓差角；界角加减差角为均数；引数初宫至五宫为加，六宫至十一宫为减。
fn sun_jun(yin: Real) -> Real {
    let m = fold(yin);
    let (alpha, _) = tri_opp(r(2e7), r(2.0 * SUN_E), m);
    let x = atan2d(r(1e7) * sind(m), r(SUN_B) * cosd(m));
    let jun = r(2.0) * alpha + (x - m);
    if norm(yin) <= r(180.0) { jun } else { -jun }
}

fn sun(t: Real) -> Sun {
    let y = sui_of(t);
    let (dz, ji) = dongzhi(y);
    let ping = norm(r(SUN_DAILY) * (t - dz));
    let next_zi = dz.floor() + r(1.0);
    let zuibei = r(ZUIBEI_YING) + r(ZUIBEI_YEAR) * r(ji as f64) + r(ZUIBEI_DAY) * (t - next_zi);
    let yin = norm(ping - zuibei);
    let jun = sun_jun(yin);
    let shi = norm(ping + jun);
    // 日距地心（「求用平行」条）：实引 = 引数加减均数；分股、勾、股弦和求弦，转与全径相减
    let shiyin = norm(yin + jun);
    let d2 = r(2.0 * SUN_E);
    let s = r(2e7) + d2 * cosd(shiyin);
    let gou = d2 * sind(shiyin);
    let xian = (s * s + gou * gou) / (r(2.0) * s);
    Sun { ping, yin, jun, shi, dist: (r(2e7) - xian) / r(1e7) }
}

/// 两弦最大末均：两最高相距 10°…90° 列表用中比例；十度以下与零度连线（原表未列，本项目假定）
fn mo_jun_max(h: Real) -> Real {
    let x = fold(h);
    let x = if x <= r(90.0) { x } else { r(180.0) - x };
    if x <= r(10.0) {
        return r(MO_JUN[0] / 3600.0) * x / r(10.0);
    }
    let i = ((x / r(10.0)).to_i64_floor() as usize).min(8);
    let (lo, hi) = (MO_JUN[i - 1], MO_JUN[i.min(8)]);
    (r(lo) + r(hi - lo) * (x - r(10.0 * i as f64)) / r(10.0)) / r(3600.0)
}

/// 月离：时刻 t（日序，平时）的黄道实行（度）与同时日躔实行
fn moon(t: Real) -> (Real, Real) {
    let n = t - (dongzhi(YUAN_YEAR).0.floor() + r(1.0));
    let ping = norm(r(MOON_YING) + r(MOON_DAILY) * n);
    let gao = norm(r(GAO_YING) + r(GAO_DAILY) * n);
    let jiao = norm(r(JIAO_YING) - r(JIAO_DAILY) * n);
    let s = sun(t);
    // 求用平行：一平均（太阴、正交与太阳相反，最高与太阳同）、二平均、三平均
    let k = s.jun.abs() / r(SUN_MAX_JUN);
    let sg = if s.jun >= r(0.0) { r(1.0) } else { r(-1.0) };
    let ping2 = ping - r(YI_PING) * k * sg;
    let gao_u = gao + r(GAO_PING) * k * sg;
    let jiao_u = jiao - r(JIAO_PING) * k * sg;
    let ri_gao = norm(s.shi - gao_u);
    let ri_jiao = norm(s.shi - jiao_u);
    let cube = r(CUBE_GAO) - s.dist * s.dist * s.dist * r(1e6);
    let s2 = sind(r(2.0) * ri_gao).abs();
    let (eg, eb) = (r(ER_PING_GAO) * s2, r(ER_PING_BEI) * s2);
    let er_ping = eg + (eb - eg) * cube / r(CUBE_DA);
    let er_ping = if norm(r(2.0) * ri_gao) < r(180.0) { -er_ping } else { er_ping };
    let san_ping = r(SAN_PING) * sind(r(2.0) * ri_jiao).abs();
    let san_ping = if norm(r(2.0) * ri_jiao) < r(180.0) { -san_ping } else { san_ping };
    let yong = norm(ping2 + er_ping + san_ping);
    // 求初实行：最高本轮、均轮得最高实均与本时两心差；借平圆引数求实引
    let two = norm(r(2.0) * ri_gao);
    let (shi_jun, e_m) = tri_opp(r(GAO_BENLUN), r(GAO_JUNLUN), r(180.0) - fold(two));
    let shi_jun = if two < r(180.0) { shi_jun } else { -shi_jun };
    let gao_shi = norm(gao_u + shi_jun);
    let yin = norm(yong - gao_shi);
    let m = fold(yin);
    let c = r(180.0) - m;
    let (beta, _) = tri_opp(r(1e7), e_m, c);
    let c2 = c + beta;
    let (q_ang, _) = tri_opp(r(1e7), e_m, c2);
    let ping_yuan = r(180.0) - c2 - q_ang;
    let ratio = (r(1.0) - (e_m / r(1e7)) * (e_m / r(1e7))).sqrt();
    let shi_yin = atan2d(ratio * sind(ping_yuan), cosd(ping_yuan));
    let chu_jun = m - shi_yin;
    let chu = if norm(yin) <= r(180.0) { norm(yong - chu_jun) } else { norm(yong + chu_jun) };
    // 求白道实行：二均、三均、末均
    let ju_ri = norm(chu - s.shi);
    let s2r = sind(r(2.0) * ju_ri).abs();
    let (jg, jb) = (r(ER_JUN_GAO) * s2r, r(ER_JUN_BEI) * s2r);
    let er_jun = jg + (jb - jg) * cube / r(CUBE_DA);
    let er_jun = if norm(r(2.0) * ju_ri) < r(180.0) { er_jun } else { -er_jun };
    let shi_ju_ri = norm(ju_ri + er_jun);
    let ri_zuigao = norm(s.ping - s.yin + r(180.0));
    let h = norm(gao_shi - ri_zuigao);
    let zong = norm(shi_ju_ri + h);
    let san_jun = r(SAN_JUN) * sind(zong).abs();
    let san_jun = if zong < r(180.0) { san_jun } else { -san_jun };
    let mo = mo_jun_max(h) * sind(shi_ju_ri).abs();
    let mo = if shi_ju_ri < r(180.0) { -mo } else { mo };
    let bai = norm(chu + er_jun + san_jun + mo);
    // 求黄道实行：正交实均（半较法）、交角减分、距日加分得黄白大距，求升度差
    let two_j = norm(r(2.0) * ri_jiao);
    let ext = if two_j < r(180.0) { two_j } else { two_j - r(180.0) };
    let kk = r((JIAO_BENLUN - JIAO_JUNLUN) / (JIAO_BENLUN + JIAO_JUNLUN));
    let half = ext / r(2.0);
    let half_diff = atan2d(kk * sind(half), cosd(half));
    let shijun = half - half_diff;
    let jiao_shi = if two_j < r(180.0) { norm(jiao_u + shijun) } else { norm(jiao_u - shijun) };
    let ju_jiao = norm(bai - jiao_shi);
    let zs = r(1.0) - cosd(two_j);
    let jian = r(DA_JU_BANJIAO) * zs;
    let jia_cha = r(JU_RI_JIAFEN_MAX / 2.0) * zs;
    let jia_fen = jia_cha / r(2.0) * (r(1.0) - cosd(r(2.0) * shi_ju_ri));
    let da_ju = r(DA_JU_MAX) - jian + jia_fen;
    let hd = atan2d(cosd(da_ju) * sind(ju_jiao), cosd(ju_jiao));
    let sheng = signed(hd - ju_jiao);
    (norm(bai + sheng), s.shi)
}

/// 时差总（日）：均数时差（均数变时，反其加减）+ 升度时差（黄、赤经差变时，二分后加、二至后减）。用时 = 平时 + 时差总
fn shi_cha(t: Real) -> Real {
    let s = sun(t);
    let lam = norm(s.shi + r(270.0));
    let ra = norm(atan2d(cosd(r(HUANG_CHI)) * sind(lam), cosd(lam)));
    (-s.jun + signed(lam - ra)) / r(360.0)
}

/// 冬至岁 y 的二十四气平时时刻（k = 0…24，日躔实行 k × 15°），子正实行内插
fn jieqi_times(y: i32) -> Vec<Real> {
    let mut out = Vec::with_capacity(25);
    let mut d = dongzhi(y).0.to_i64_floor() - 2;
    let mut lam_d = sun(r(d as f64)).shi;
    for k in 0..25 {
        let target = r(((k * 15) % 360) as f64);
        loop {
            let lam_n = sun(r((d + 1) as f64)).shi;
            let a = signed(lam_d - target);
            let b = signed(lam_n - target);
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

/// [d0, d1) 日内各合朔平时时刻：本日、次日子正日月黄道实行内插
fn shuo_times(d0: i64, d1: i64) -> Vec<Real> {
    let mut out = Vec::new();
    let elong = |d: i64| {
        let (mo, su) = moon(r(d as f64));
        signed(mo - su)
    };
    let mut e_d = elong(d0);
    for d in d0..d1 {
        let e_n = elong(d + 1);
        if e_d < r(0.0) && e_n >= r(0.0) {
            out.push(r(d as f64) - e_d / (e_n - e_d));
        }
        e_d = e_n;
    }
    out
}

fn yong_day(t: Real) -> i64 {
    (t + shi_cha(t)).to_i64_floor()
}

/// 冬至岁 y：(朔日 JDN, 月序, 闰否)，自天正（含冬至之月）至次年天正前。
/// 「以前後兩年有冬至之月為準。中積十三月者，以無中氣之月，從前月置閏。一歲中兩無中氣者，置在前無中氣之月為閏。」
pub fn sui_months(y: i32) -> Vec<(i64, u8, bool)> {
    let (q0, q1) = (jieqi_times(y), jieqi_times(y + 1));
    let mut zhong: Vec<i64> = (0..24).step_by(2).map(|k| yong_day(q0[k])).collect();
    zhong.push(yong_day(q1[0]));
    let sh = shuo_times(q0[0].to_i64_floor() - 32, q1[0].to_i64_floor() + 32);
    let days: Vec<i64> = sh.iter().map(|&t| yong_day(t)).collect();
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

    /// 志文所载极值：日躔最大均数 1°56′12″；雍正癸卯天正冬至丙申日丑正三刻十一分
    #[test]
    fn houbian_constants() {
        let mx = (0..3600).map(|a| sun_jun(r(a as f64 / 10.0)).abs().as_f64()).fold(0.0, f64::max);
        assert!((mx * 60.0 - 116.2).abs() < 0.05, "{mx}");
        let (dz, _) = dongzhi(YUAN_YEAR);
        assert_eq!((jdn_of(dz.to_i64_floor()) + 49).rem_euclid(60), 32, "丙申");
        let h = (dz - dz.floor()).as_f64() * 24.0;
        assert!((2.93..2.95).contains(&h), "{h}");
        assert_eq!(daynum_of(jdn_of(5)), 5);
    }

    /// 《三年历书·凡例》：1914 年旧法（后编，用时）九大十小
    #[test]
    fn houbian_1914_old_rule() {
        let ms = year_months(1914);
        let nine = ms.iter().position(|e| e.1 == 9 && !e.2).unwrap();
        assert_eq!(ms[nine + 1].0 - ms[nine].0, 30, "九月大");
        let ten_next = sui_months(1915)[0].0;
        assert_eq!(ten_next - ms[nine + 1].0, 29, "十月小");
    }

    /// 与 Python 复原（data/calendar/shixian_houbian_check.csv）逐月相同：冬至岁 1880—1913 用时朔日
    #[test]
    fn matches_python_check() {
        let txt = std::fs::read_to_string(crate::repo::repo_root().join("data/calendar/shixian_houbian_check.csv")).unwrap();
        let mut want: std::collections::BTreeMap<i32, Vec<String>> = Default::default();
        for line in txt.lines().filter(|l| !l.starts_with('#')).skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            let y: i32 = f[0].parse().unwrap();
            if (1880..=1913).contains(&y) {
                want.entry(y).or_default().push(f[3].to_string());
            }
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
