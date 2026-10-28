//! 等距取样的地心视位置与星等，四点 Lagrange 插值（伏见、月掩星的粗搜共用）。

use crate::math::real::{real, Real, RealOps};

/// 取样步长（日）。四点 Lagrange 插值误差约 (2πh/P)⁴/24：月（P ≈ 27.3 日）h = 1 日时 < 0.01°，
/// 远低于天空亮度模型所需（作者称模型约 20%）；日、行星与恒星更小。
pub(crate) const TRACK_STEP: f64 = 1.0;

/// 四点 Lagrange 插值的等距序列。
pub(crate) struct Grid<T> {
    t0: Real,
    pts: Vec<T>,
}

fn grid_times(t0: Real, t1: Real) -> (Real, usize) {
    let n = ((t1 - t0) / real(TRACK_STEP)).floor().as_f64() as usize + 4;
    (t0 - real(TRACK_STEP), n)
}

fn lagrange4(s: Real) -> [Real; 4] {
    let mut w = [real(1.0); 4];
    for (j, wj) in w.iter_mut().enumerate() {
        for m in 0..4 {
            if m != j {
                *wj = *wj * (s - real(m as f64)) / real(j as f64 - m as f64);
            }
        }
    }
    w
}

impl<T> Grid<T> {
    pub(crate) fn build(t0: Real, t1: Real, f: impl Fn(Real) -> T) -> Self {
        let (start, n) = grid_times(t0, t1);
        Self { t0: start, pts: (0..n).map(|i| f(start + real(TRACK_STEP * i as f64))).collect() }
    }

    /// 插值基点下标 i 与四点权重。
    pub(crate) fn locate(&self, t: Real) -> (usize, [Real; 4]) {
        let u = (t - self.t0) / real(TRACK_STEP);
        let i = (u.floor().as_f64().max(0.0) as usize).clamp(1, self.pts.len() - 3) - 1;
        (i, lagrange4(u - real(i as f64)))
    }

    pub(crate) fn covers(&self, t0: Real, t1: Real) -> bool {
        t0 >= self.t0 + real(TRACK_STEP) && t1 <= self.t0 + real(TRACK_STEP * (self.pts.len() as f64 - 2.0))
    }
}

/// 地心视位置（当日真赤道，米）。
pub(crate) type Track = Grid<[Real; 3]>;

impl Track {
    pub(crate) fn at(&self, t: Real) -> [Real; 3] {
        let (i, w) = self.locate(t);
        let mut out = [real(0.0); 3];
        for (j, wj) in w.iter().enumerate() {
            for c in 0..3 {
                out[c] = out[c] + *wj * self.pts[i + j][c];
            }
        }
        out
    }
}

/// 星等序列；超出行星实测相角范围处为 None，插值时四点有一缺即 None。
pub(crate) type MagTrack = Grid<Option<Real>>;

impl MagTrack {
    pub(crate) fn mag_at(&self, t: Real) -> Option<Real> {
        let (i, w) = self.locate(t);
        let mut out = real(0.0);
        for (j, wj) in w.iter().enumerate() {
            out = out + *wj * self.pts[i + j]?;
        }
        Some(out)
    }
}

