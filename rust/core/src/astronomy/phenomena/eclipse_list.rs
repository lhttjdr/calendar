//! 按时段列出日月食：逐月在平朔、平望附近搜索（平朔望与真朔望差不足 1 日，在各搜索函数 ±1.5 日的范围内）。
//!
//! 一年二十多个朔望里有食的只有几个，而精算窗要 32 次完整历表求值。故先用 8 节点粗窗初筛（插值误差远小于所留余量），
//! 只对可能有食者建精算窗。筛选的安全性由 `tests_eclipse_sweep::eclipse_lists_match_nasa_catalog` 对 NASA 目录逐条保证。

use crate::astronomy::aspects::approximate_new_moon_jd;
use crate::math::real::{real, Real, RealOps};

use super::lunar_eclipse::{lunar_eclipse_geometry, lunar_eclipse_in, LunarEclipse};
use super::search::{minimize_golden, seconds};
use super::solar_eclipse::{besselian, eclipse_window, solar_eclipse_global_in, EclipseEphemeris, InterpolatedSunMoon, SolarEclipseGlobal};

const SYNODIC_MONTH: f64 = 29.530588853;

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

fn coarse_window(eph: &EclipseEphemeris, guess: Real) -> InterpolatedSunMoon {
    InterpolatedSunMoon::new(eph, guess, real(2.0), 8)
}

/// 初筛：影轴到地心的最小距离是否够得着地球（留 0.1 地球半径余量）。
fn solar_possible(eph: &EclipseEphemeris, g: Real) -> bool {
    let w = coarse_window(eph, g);
    let (t, _) = minimize_golden(|t| besselian(&w, t).m1(), g - real(1.5), g + real(1.5), seconds(60.0));
    let b = besselian(&w, t);
    b.m1() < real(1.1) + b.l1
}

/// 初筛：最小角距时半影食分是否接近正值（留 0.2 余量）。
fn lunar_possible(eph: &EclipseEphemeris, g: Real) -> bool {
    let w = coarse_window(eph, g);
    let (t, _) = minimize_golden(|t| lunar_eclipse_geometry(&w, t).separation, g - real(1.5), g + real(1.5), seconds(60.0));
    lunar_eclipse_geometry(&w, t).penumbral_magnitude() > real(-0.2)
}

/// [start, end)（TT 儒略日）内极大食落入的日食，按时序。
pub fn solar_eclipses_between(eph: &EclipseEphemeris, start: Real, end: Real) -> Vec<SolarEclipseGlobal> {
    collect(start, end, 0.0, |g| if solar_possible(eph, g) { solar_eclipse_global_in(&eclipse_window(eph, g), g) } else { None }, |e| e.greatest_tt.as_f64())
}

/// [start, end)（TT 儒略日）内食甚落入的月食，按时序。
pub fn lunar_eclipses_between(eph: &EclipseEphemeris, start: Real, end: Real) -> Vec<LunarEclipse> {
    collect(start, end, SYNODIC_MONTH / 2.0, |g| if lunar_possible(eph, g) { lunar_eclipse_in(&eclipse_window(eph, g), g) } else { None }, |e| e.greatest_tt.as_f64())
}
