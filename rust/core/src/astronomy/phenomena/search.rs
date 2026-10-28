//! 天象时刻搜索：单峰函数求极小（黄金分割）、区间求根（二分）。时间变量为 TT 儒略日。

use crate::math::real::{real_const, Real, RealOps};

/// 在 [a, b] 上求单峰函数 f 的极小点，区间缩到 `tol_days` 以内。返回 (t, f(t))。
pub fn minimize_golden(f: impl Fn(Real) -> Real, mut a: Real, mut b: Real, tol_days: Real) -> (Real, Real) {
    let g = real_const(0.6180339887498949);
    let mut c = b - g * (b - a);
    let mut d = a + g * (b - a);
    let (mut fc, mut fd) = (f(c), f(d));
    while (b - a).abs() > tol_days {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - g * (b - a);
            fc = f(c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + g * (b - a);
            fd = f(d);
        }
    }
    let t = (a + b) / real_const(2.0);
    (t, f(t))
}

/// 在 [a, b] 上求 f 的根（要求 f(a)、f(b) 异号），精度 `tol_days`。端点不异号时返回 None。
pub fn bisect_root(f: impl Fn(Real) -> Real, mut a: Real, mut b: Real, tol_days: Real) -> Option<Real> {
    let mut fa = f(a);
    let fb = f(b);
    if (fa < real_const(0.0)) == (fb < real_const(0.0)) {
        return None;
    }
    while (b - a).abs() > tol_days {
        let m = (a + b) / real_const(2.0);
        let fm = f(m);
        if (fm < real_const(0.0)) == (fa < real_const(0.0)) {
            a = m;
            fa = fm;
        } else {
            b = m;
        }
    }
    Some((a + b) / real_const(2.0))
}

/// 在 [a, b] 上求光滑函数 f 的极值点：以五点中心差分导数
/// f′(t) ≈ [−f(t+2h) + 8f(t+h) − 8f(t−h) + f(t−2h)] / 12h 的根（二分，精度 `tol_days`）。
/// 与黄金分割比较 f 两值大小不同，极值附近 f 很平，两值之差可小到 1e-13 rad，f 的数值噪声（如 f64 构建儒略日约 40 µs 的
/// 时间量化，行星方向约 3e-12 rad）会使比较出错、极值时刻抖动数十秒；导数取相距 2h 的四点，噪声影响降到约
/// σ / (h·|f″|)，截断误差 O(h⁴)。端点导数不异号（区间内无极值）时返回 None。
pub fn extremum_by_derivative_root(f: impl Fn(Real) -> Real, a: Real, b: Real, h_days: Real, tol_days: Real) -> Option<Real> {
    let d = |t: Real| -f(t + h_days * real_const(2.0)) + real_const(8.0) * f(t + h_days) - real_const(8.0) * f(t - h_days) + f(t - h_days * real_const(2.0));
    bisect_root(d, a, b, tol_days)
}

/// 秒 → 日。
pub fn seconds(s: f64) -> Real {
    real_const(s) / real_const(86400.0)
}
