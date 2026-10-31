//! 按时段列出日月食：逐月在平朔、平望附近搜索（平朔望与真朔望差不足 1 日，在各搜索函数 ±1.5 日的范围内）。
//!
//! 一年二十多个朔望里有食的只有几个，而建搜索窗要 12 次完整历表求值。故先按平朔望时月亮的平升交角距 F 初筛：离交点太远的
//! 不可能有食，约六成朔望无需碰历表。阈值不能照搬 Meeus《天文算法》第 54 章的 |sin F| > 0.36：掠食实测可到 0.45 以上
//! （如 220 年 2 月 21 日半影日食 0.4518），这里取 0.55。安全性由 `tests_eclipse_sweep::node_screen_never_drops_an_eclipse`
//! 在前 3000—3000 年逐朔望保证。

use crate::astronomy::aspects::approximate_new_moon_jd;
use crate::math::real::{real, Real, RealOps};

use super::lunar_eclipse::{lunar_eclipse_in, LunarEclipse};
use super::solar_eclipse::{eclipse_window, solar_eclipse_global_in, EclipseEphemeris, SolarEclipseGlobal};

const SYNODIC_MONTH: f64 = 29.530588853;
/// 初筛阈值，见模块说明。
pub(super) const NODE_LIMIT_SIN_F: f64 = 0.55;

/// 覆盖 [start, end) 的朔望序号（两端各多取两个月，平朔序号与时刻的换算只是近似）。
fn lunation_range(start: f64, end: f64) -> std::ops::Range<i32> {
    let origin = approximate_new_moon_jd(0).as_f64();
    let n0 = ((start - origin) / SYNODIC_MONTH).floor() as i32 - 2;
    let n1 = ((end - origin) / SYNODIC_MONTH).ceil() as i32 + 2;
    n0..n1
}

fn collect<T>(start: Real, end: Real, offset: f64, find: impl Fn(Real) -> Option<T>, time: impl Fn(&T) -> f64) -> Vec<T> {
    let (a, b) = (start.as_f64(), end.as_f64());
    let mut out: Vec<T> = vec![];
    for n in lunation_range(a, b) {
        let guess = approximate_new_moon_jd(n) + real(offset);
        if guess.as_f64() < a - 2.0 || guess.as_f64() > b + 2.0 {
            continue;
        }
        let Some(e) = find(guess) else { continue };
        let t = time(&e);
        // 相邻两月的搜索范围不相交（±1.5 日），这里的去重只防万一
        if t >= a && t < b && !out.iter().any(|x| (time(x) - t).abs() < 5.0) {
            out.push(e);
        }
    }
    out.sort_by(|x, y| time(x).partial_cmp(&time(y)).unwrap());
    out
}

/// 月亮平升交角距 F（Simon et al. 1994，度）的正弦绝对值。这只是初筛，不走 Real。
pub(super) fn abs_sin_mean_f(jd_tt: f64) -> f64 {
    let t = (jd_tt - 2451545.0) / 36525.0;
    (93.272_090 + 483_202.017_538 * t - 0.003_654 * t * t).to_radians().sin().abs()
}

/// 初筛：此朔望离黄白交点是否近到可能有食。
fn near_node(guess: Real) -> bool {
    abs_sin_mean_f(guess.as_f64()) <= NODE_LIMIT_SIN_F
}

/// [start, end)（TT 儒略日）内极大食落入的日食，按时序。
pub fn solar_eclipses_between(eph: &EclipseEphemeris, start: Real, end: Real) -> Vec<SolarEclipseGlobal> {
    collect(start, end, 0.0, |g| if near_node(g) { solar_eclipse_global_in(&eclipse_window(eph, g), g) } else { None }, |e| e.greatest_tt.as_f64())
}

/// [start, end)（TT 儒略日）内食甚落入的月食，按时序。
pub fn lunar_eclipses_between(eph: &EclipseEphemeris, start: Real, end: Real) -> Vec<LunarEclipse> {
    collect(start, end, SYNODIC_MONTH / 2.0, |g| if near_node(g) { lunar_eclipse_in(&eclipse_window(eph, g), g) } else { None }, |e| e.greatest_tt.as_f64())
}
