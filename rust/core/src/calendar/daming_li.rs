//! 金重修大明历（赵知微，大定二十一年 1181 校验后行用）定朔推步：经朔 + 日躔朓朒 + 月离朓朒 → 定朔，进朔，恒气无中气置闰。
//!
//! 术文据《金史·历志》卷二十一、二十二；维基文库本日躔、月离两表「表略」，表取《元史·历志》卷五十六庚午元历
//! （耶律楚材「因金大明历」，日法、岁实、朔实、转终与诸表同，点校本校勘记屡以「金大明曆與驗算合」定字）。
//! 杨级大明历（1137—1181 行用）术不传，以本历代推：《金史》本纪明书朔日 1137—1181 年 144 条合 143
//! （scripts/calendar_data/daming_dingshuo.py，doc/calendar/historical-calendar.md）。
//!
//! 全部用有理数（日法 5230 分、秒母一万），进朔判定无浮点误差。
//! 进朔限校准：纯按术文推步，《金史》本纪明书朔日、閏月与天文志在临限之月显示实际进朔限比术文高 0.14—0.36 时
//! （1178 年十月定朔超限 0.14 时而不进、1162 年超限 0.10 时而不进致閏月在二月后；1152 年四月超 0.36 时、1214 年正月超 0.37 时
//! 而进），原因未明（庚午元历表与金表或有小异）。缺省取区间中点 0.25 时（[JINSHUO_CALIB]），[year_months_with] 可取 0 用术文原限。
//! 不实现：「有交，亏初在日入前者不进」（不算日食），「使不过三大二小」（只在连大连小时调整，本历 1130—1235 年无此情形）。

use super::gregorian;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};

type Q = BigRational;

const F: i64 = 5230; // 日法
const SUI: i64 = 1_910_224; // 岁实
const SHUO: i64 = 154_445; // 朔实
const JI_1180: i64 = 88_639_656; // 上元甲子距大定庚子（1180）积年
const ZHUAN_WAN: i64 = 1_441_106_066; // 转终分 × 10000（转终分一十四万四千一百一十，秒六千六十六）

/// 进朔限校准（日法分）：0.25 时 = 日法/96，据本纪证据区间（0.14, 0.36）时取中点，见模块文档
pub const JINSHUO_CALIB: (i64, i64) = (F, 96);

/// 日躔：冬至起十二恒气损益率（日法分；正为朒、加），夏至后同数异名
const SUN_L: [i64; 12] = [276, 232, 185, 135, 83, 29, -29, -83, -135, -185, -232, -276];
/// 初、末定率（万分之一分）；庚午元历表霜降初末率讹，同穀雨；芒种末率同冬至初率
const SUN_A: [i64; 12] = [194864, 166874, 136911, 104670, 71114, 35631, 2480, 38576, 73359, 107136, 138940, 167852];
const SUN_B: [i64; 12] = [167852, 138019, 106214, 72745, 37963, 2480, 35631, 70501, 104056, 135991, 165952, 194864];
/// 冬至至夏至十三恒气初日出分（百分之一分），夏至后对称
const SUNRISE: [i64; 13] = [156792, 155752, 152879, 148523, 143400, 136614, 129696, 123227, 117318, 112234, 108248, 105642, 104707];
/// 月离：入转一至二十八日损益率（日法分；正为朓、减）。七、十四、二十一日分初末，见 [SPLIT]
const MOON_L: [i64; 28] = [
    513, 469, 411, 332, 243, 141, 0, -63, -164, -258, -352, -427, -481, 0, -505, -462, -395, -309, -219, -117, 0, 86, 184, 278, 368, 438, 493, 293,
];
/// (日序, 初数, 初率, 末数, 末率)；二十八日只有初数 2901
const SPLIT: [(usize, i64, i64, i64, i64); 3] = [(6, 4648, 43, 582, -4), (13, 4065, -403, 1165, -117), (20, 3483, -27, 1747, 11)];

fn q(n: i64) -> Q {
    Q::from_integer(BigInt::from(n))
}

fn qr(n: i64, d: i64) -> Q {
    Q::new(BigInt::from(n), BigInt::from(d))
}

fn floor_i64(x: &Q) -> i64 {
    x.floor().to_integer().to_i64().unwrap()
}

/// 气策（分）
fn qi() -> Q {
    qr(SUI, 24)
}

/// 距天正冬至 t（分，[0, 岁实)）→ 日躔朓朒定数（分，朒 +）。恒气内每日损益率自初定率起逐日等差（日差 = (初 − 末)/(气日 − 1)），
/// 朓朒积按日累计，入气余分乘该日损益率（《金史》「求每日盈缩朓朒」「求经朔弦望入气朓朒定数」）。
fn sun_def(t: &Q) -> Q {
    let qi = qi();
    let k = floor_i64(&(t / &qi));
    let x = t - &qi * q(k);
    let (half, kk) = ((k / 12) % 2, (k % 12) as usize);
    let sign = if half == 0 { 1 } else { -1 };
    let ji: i64 = sign * SUN_L[..kk].iter().sum::<i64>();
    let (a, b) = (qr(SUN_A[kk], 10000), qr(SUN_B[kk], 10000));
    let q_days = &qi / q(F);
    let cha = (&a - &b) / (&q_days - q(1));
    let day = floor_i64(&(&x / q(F)));
    let r = &x - q(day * F);
    let rate = |i: i64| &a - &cha * q(i);
    let mut acc = Q::zero();
    for i in 0..day {
        acc += rate(i);
    }
    acc += rate(day) * &r / q(F);
    let s = SUN_L[kk].signum();
    q(ji) + q(sign * s) * acc
}

/// 入转 x（分，[0, 转终分)）→ 月离朓朒定数（分，朓 +）。
fn moon_def(x: &Q) -> Q {
    let i = floor_i64(&(x / q(F))) as usize;
    let r = x - q(i as i64 * F);
    let ji = |i: usize| -> i64 {
        (0..i)
            .map(|j| SPLIT.iter().find(|s| s.0 == j).map_or(MOON_L[j], |s| s.2 + s.4))
            .sum()
    };
    if let Some(&(_, n0, r0, n1, r1)) = SPLIT.iter().find(|s| s.0 == i) {
        return if r <= q(n0) { q(ji(i)) + q(r0) * &r / q(n0) } else { q(ji(i) + r0) + q(r1) * (&r - q(n0)) / q(n1) };
    }
    if i >= 27 {
        let r = if r > q(2901) { q(2901) } else { r };
        return q(ji(27)) + q(MOON_L[27]) * r / q(2901);
    }
    q(ji(i)) + q(MOON_L[i]) * r / q(F)
}

/// 距天正冬至 t（分）→ 日出分（恒气间线性）
fn sunrise(t: &Q) -> Q {
    let qi = qi();
    let k = floor_i64(&(t / &qi)).rem_euclid(24) as usize;
    let x = t - &qi * q(floor_i64(&(t / &qi)));
    let (i0, i1) = if k < 12 { (k, k + 1) } else { (24 - k, 23 - k) };
    let (s0, s1) = (qr(SUNRISE[i0], 100), qr(SUNRISE[i1], 100));
    &s0 + (s1 - &s0) * x / qi
}

/// 天正冬至在 y−1 年之岁的通积分（自上元甲子夜半起，分）
fn tongji(y: i32) -> BigInt {
    BigInt::from(SUI) * BigInt::from(JI_1180 + (y as i64 - 1180))
}

/// 上元日序 → JDN：1180 年天正冬至己巳，儒略历 1179 年 12 月 15 日
fn epoch_offset() -> i64 {
    let dz = (tongji(1180) / BigInt::from(F)).to_i64().unwrap();
    gregorian::civil_to_jdn(1179, 12, 15) - dz
}

/// 冬至在 y−1 年之岁，自天正经朔起第 m 月的进朔后定朔日（上元日序）
fn dingshuo_day(y: i32, m: i64, calib: &Q) -> i64 {
    let tong = tongji(y);
    let run = &tong % BigInt::from(SHUO);
    let t0 = &tong - &run;
    let jing = &t0 + BigInt::from(m * SHUO);
    let year = q(SUI);
    let wrap = |v: Q, m: &Q| &v - m * Q::from_integer((&v / m).floor().to_integer());
    let dt = wrap(Q::from_integer(&jing - &tong), &year);
    let sd = sun_def(&dt);
    let zhuan = qr(ZHUAN_WAN, 10000);
    let ru = (&t0 * BigInt::from(10000)) % BigInt::from(ZHUAN_WAN);
    let md = moon_def(&wrap(Q::new(ru, BigInt::from(10000)) + q(m * SHUO), &zhuan));
    let day0 = (&jing / BigInt::from(F)).to_i64().unwrap();
    let frac = Q::from_integer(&jing % BigInt::from(F)) + &sd - &md;
    let carry = floor_i64(&(&frac / q(F)));
    let xiao = &frac - q(carry * F);
    // 进朔：秋分后（至春分）小余 ≥ 3/4 日法；春分后（至秋分）限减（春分日出分 − 定朔日出分）/3
    let ts = wrap(dt + &sd - &md, &year);
    let three_q = qr(3 * F, 4);
    let lim = if ts >= qi() * q(6) && ts < qi() * q(18) {
        &three_q - (qr(SUNRISE[6], 100) - sunrise(&ts)) / q(3)
    } else {
        three_q
    };
    day0 + carry + if xiao >= lim + calib { 1 } else { 0 }
}

/// 冬至在 y−1 年之岁的月：(朔日 JDN, 月序, 闰否)，自含冬至之月（十一月）起至下一含冬至之月前。
fn sui_months(y: i32, calib: &Q) -> Vec<(i64, u8, bool)> {
    let off = epoch_offset();
    let days: Vec<i64> = (-2..16).map(|m| dingshuo_day(y, m, calib) + off).collect();
    // 恒气中气日（冬至起偶数气），覆盖本岁与下岁冬至
    let zq: Vec<i64> = (0..=26)
        .step_by(2)
        .map(|k| {
            let t = Q::from_integer(tongji(y)) + qi() * q(k);
            floor_i64(&(t / q(F))) + off
        })
        .collect();
    let (dz0, dz1) = (zq[0], zq[12]);
    let contains = |i: usize, d: i64| days[i] <= d && d < days[i + 1];
    let i0 = (0..days.len() - 1).find(|&i| contains(i, dz0)).unwrap();
    let i1 = (0..days.len() - 1).find(|&i| contains(i, dz1)).unwrap();
    let leap = if i1 - i0 == 13 { (i0 + 1..i1).find(|&i| !zq.iter().any(|&z| contains(i, z))) } else { None };
    let mut out = Vec::new();
    let mut label = 11u8;
    for i in i0..i1 {
        if Some(i) == leap {
            out.push((days[i], out.last().map_or(11, |e: &(i64, u8, bool)| e.1), true));
            continue;
        }
        out.push((days[i], label, false));
        label = label % 12 + 1;
    }
    out
}

/// 农历年 N 各月 (朔日 JDN, 月序, 闰否)，正月至十二月（含闰），按时间先后；进朔限取校准值 [JINSHUO_CALIB]。
pub fn year_months(lunar_year: i32) -> Vec<(i64, u8, bool)> {
    year_months_with(lunar_year, JINSHUO_CALIB)
}

/// 同 [year_months]，进朔限加 calib = (分子, 分母) 日法分；(0, 1) 即术文原限。
pub fn year_months_with(lunar_year: i32, calib: (i64, i64)) -> Vec<(i64, u8, bool)> {
    let calib = qr(calib.0, calib.1);
    let head: Vec<_> = sui_months(lunar_year, &calib);
    // 本岁十一、十二月（及其闰）属上一农历年
    let k = head.iter().position(|e| e.1 == 1 && !e.2).unwrap();
    let mut out = head[k..].to_vec();
    out.extend(sui_months(lunar_year + 1, &calib).into_iter().take_while(|e| e.1 >= 11));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gan_zhi;

    fn gz(jdn: i64) -> usize {
        ((jdn + 49).rem_euclid(60)) as usize
    }

    #[test]
    fn epoch_dongzhi_1180_is_jisi() {
        let dz = (tongji(1180) / BigInt::from(F)).to_i64().unwrap();
        assert_eq!(dz.rem_euclid(60), 5, "1180 年天正冬至应为己巳（上元甲子起第 5 日）");
        assert_eq!(gz(dz + epoch_offset()), 5);
    }

    fn month(y: i32, m: u8, leap: bool) -> i64 {
        year_months(y).into_iter().find(|e| e.1 == m && e.2 == leap).unwrap().0
    }

    fn gz_of(y: i32, m: u8, leap: bool) -> &'static str {
        gan_zhi::GAN_ZHI_60[gz(month(y, m, leap))]
    }

    /// 定校准区间的四条本纪：术文原限下 1162 年闰三月、1178 年十月辛卯，与本纪、天文志不合；校准后皆合，1152、1214 年仍进朔
    #[test]
    fn calibration_matches_benji_at_threshold() {
        let raw = |y: i32| year_months_with(y, (0, 1));
        assert!(raw(1162).iter().any(|e| e.1 == 3 && e.2), "术文原限 1162 年闰三月");
        assert!(year_months(1162).iter().any(|e| e.1 == 2 && e.2), "1162 年閏二月（本纪「閏月」在二月后，天文志「閏二月戊寅」）");
        assert_eq!(gz_of(1162, 3, false), "丁酉");
        assert_eq!(gz_of(1178, 10, false), "庚寅");
        assert_eq!(gz_of(1152, 4, false), "丙寅");
        assert_eq!(gz_of(1214, 1, false), "丁卯");
    }

    /// 《金史》本纪明书朔日中，本历（进朔）与主表（宋历）不同而本纪从本历者
    #[test]
    fn jinshi_new_moons_where_jin_differs_from_song() {
        // 天德四年（1152）四月丙寅朔、大定八年（1168）十月己丑朔、大定十六年（1176）正月戊申朔
        for (y, m, gzs) in [(1152, 4, "丙寅"), (1168, 10, "己丑"), (1176, 1, "戊申")] {
            let j = month(y, m, false);
            assert_eq!(gan_zhi::GAN_ZHI_60[gz(j)], gzs, "{y} 年 {m} 月");
        }
    }

    /// 1137—1234 年闰月（校准后与主表全同）
    #[test]
    fn leap_months_1137_1234() {
        let leaps: Vec<(i32, u8)> = (1137..=1234)
            .filter_map(|y| year_months(y).into_iter().find(|e| e.2).map(|e| (y, e.1)))
            .collect();
        assert_eq!(leaps.len(), 36);
        assert!(leaps.contains(&(1162, 2)));
        assert!(leaps.contains(&(1183, 11)) && leaps.contains(&(1232, 9)));
        for y in 1137..=1234 {
            let n = year_months(y).len();
            assert!(n == 12 || n == 13, "{y} 年 {n} 月");
        }
    }
}
