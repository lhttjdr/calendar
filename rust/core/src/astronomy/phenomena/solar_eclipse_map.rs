//! 日食地图（食带图）：由贝塞尔基本面几何求各条界线的大地经纬度，供前端画平面图或地球图。
//!
//! 基本面坐标 (ξ, η, ζ) 以地球赤道半径为单位（见 [`super::solar_eclipse`]）。影锥母线上的点
//!   P(ζ) = (x + L(ζ)·cosθ, y + L(ζ)·sinθ, ζ)，L(ζ) = l − ζ·tan f，
//! 关于 ζ 是线性的，与地球椭球 X² + Y² + Z²/(1 − e²) = 1 求交是 ζ 的二次方程，取向日一侧的大根。由此：
//! - **中心线**：影轴（l = 0、tan f = 0）与椭球的交点，按时刻扫描；
//! - **影子轮廓**：某时刻 θ 走一周；
//! - **南北界**（本影、半影）：影子轮廓随时间的包络。地面固定点 G 到影轴的距离与该处影锥半径之差
//!   F(t) = |Δ(t)| − |L(t)|，轮廓上 F = 0；包络点另需 dF/dt = 0（该点只被影子边缘擦过）。对每个时刻在 θ 上求 dF/dt 的根，
//!   dF/dt 用 ±30 s 中心差分；
//! - **带食出没界线**：半影轮廓与地球边缘（沿 k 方向切椭球，即太阳中心几何高度 0°）的交点，随时刻连成曲线，是「日出带食」
//!   「日没带食」区域的边界。
//!
//! ΔT 只经格林尼治视恒星时进入（贝塞尔量只依赖 TT）。GAST = ERA(UT1) + 只依赖 TT 的项，ERA 对 UT1 是线性的，所以换一个
//! ΔT，全部曲线严格地整体沿经度平移 [`longitude_shift_for_delta_t`]；前端画 ΔT ± σ 时平移即可，不必重算。
//!
//! 依据：*Explanatory Supplement to the Astronomical Almanac* (2013) 第 11 章；Meeus, *Elements of Solar Eclipses 1951–2200*
//! (1989)。sxwnl `rsGS` 的界线作对照。

use crate::astronomy::time::earth_rotation::gast;
use crate::math::real::{real, real_const, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;

use super::search::{bisect_root, minimize_golden, seconds};
use super::solar_eclipse::{
    besselian_with_axes, dot, norm, solar_eclipse_global_in, sub, Besselian, SolarEclipseGlobal, SunMoonSource, EARTH_E2, V3,
};

/// 地球自转角 ERA 对 UT1 的速率（圈/日，IERS Conventions 2010 式 5.15）。
pub const EARTH_ROTATION_TURNS_PER_UT1_DAY: f64 = 1.002_737_811_911_354_48;

/// ΔT 增加 `delta_dt` 时，日食地图上全部曲线的经度平移量（东正）：+2π·1.0027378…·δΔT/日。
/// ΔT 越大，同一 TT 对应的 UT1 越早、地球少转，食带偏东。
pub fn longitude_shift_for_delta_t(delta_dt: Duration) -> PlaneAngle {
    PlaneAngle::from_rad(
        real_const(2.0) * Real::pi() * real_const(EARTH_ROTATION_TURNS_PER_UT1_DAY) * delta_dt.seconds() / real_const(86400.0),
    )
}

/// 地图上的一点：TT 时刻与大地经纬度（经度东正，取值 (−π, π]）。
#[derive(Clone, Copy, Debug)]
pub struct MapPoint {
    pub jd_tt: Real,
    pub longitude: PlaneAngle,
    pub latitude: PlaneAngle,
}

/// 一条折线（按时刻或按轮廓角排列）。
pub type Polyline = Vec<MapPoint>;

/// 某时刻的影子轮廓（只含落在地面上的部分，可能断成几段）。
#[derive(Clone, Debug)]
pub struct ShadowOutline {
    pub jd_tt: Real,
    pub penumbra: Vec<Polyline>,
    /// 本影（全食）或伪本影（环食）。
    pub umbra: Vec<Polyline>,
}

/// 日食地图。所有时刻为 TT；经度对应所给 ΔT。
#[derive(Clone, Debug)]
pub struct SolarEclipseMap {
    pub global: SolarEclipseGlobal,
    pub delta_t: Duration,
    /// 半影始触、终离地球（P1、P4）。
    pub penumbra_contacts: (Real, Real),
    /// 影轴始触、终离地球（中心食的中心线两端）。
    pub central_contacts: Option<(Real, Real)>,
    /// 极大食点：中心食为极大食时影轴落地点；非中心食为地球边缘上离影轴最近的点。
    pub greatest_point: MapPoint,
    /// 极大食时的日下点（地心方向 k 与地面的交点，纬度为地心纬度）：画昼夜界线、从太阳方向看地球用。
    pub subsolar: MapPoint,
    pub central_line: Vec<Polyline>,
    pub umbra_north: Vec<Polyline>,
    pub umbra_south: Vec<Polyline>,
    /// 半影界（偏食南北界）：可见极大食分 0 等值线上、极大食不在地平处的部分。各点时刻为该地食甚（即擦边时刻）。
    pub penumbra_limits: Vec<Polyline>,
    /// 带食出没界线（初亏或复圆恰在日出、日没时）。
    pub rise_set: Vec<Polyline>,
    /// 可见极大食分等值线（按 [`SolarEclipseMapOptions::magnitude_levels`]）；各点时刻为该地可见极大食时刻。
    pub magnitude_contours: Vec<MagnitudeContour>,
    pub outlines: Vec<ShadowOutline>,
}

/// 一条可见极大食分等值线（可能分成多段）。
#[derive(Clone, Debug)]
pub struct MagnitudeContour {
    pub level: Real,
    pub lines: Vec<Polyline>,
}

/// 地图计算选项。
#[derive(Clone, Debug)]
pub struct SolarEclipseMapOptions {
    /// 时间扫描步长（中心线、本影界、带食出没界线的取点间隔；也是逐地求极大食分的时刻表间隔）。
    pub step: Duration,
    /// 影子轮廓的时间间隔（按 UT 整点对齐）；None 不画轮廓。
    pub outline_interval: Option<Duration>,
    /// 本影包络、影子轮廓在 θ 上的采样数。
    pub theta_samples: usize,
    /// 逐地求可见极大食分的经纬网格间距（半影界与食分等值线由此网格求等值线，再在网格边上精确二分）。
    pub grid_step: PlaneAngle,
    /// 要画的食分等值线（0 级总会算，用来得半影界）。
    pub magnitude_levels: Vec<Real>,
}

impl Default for SolarEclipseMapOptions {
    fn default() -> Self {
        Self {
            step: Duration::in_seconds(real_const(60.0)),
            outline_interval: Some(Duration::in_seconds(real_const(1800.0))),
            theta_samples: 180,
            grid_step: PlaneAngle::from_deg(real_const(1.0)),
            magnitude_levels: vec![real_const(0.2), real_const(0.4), real_const(0.6), real_const(0.8)],
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 基本面 ↔ 地固系

/// 某时刻的基本面与地球自转。
#[derive(Clone, Copy)]
struct Frame {
    t: Real,
    b: Besselian,
    axes: [V3; 3],
    /// GAST（弧度）。
    gast: Real,
}

impl Frame {
    fn new(src: &dyn SunMoonSource, t: Real, delta_t_days: Real) -> Self {
        let (b, axes) = besselian_with_axes(src, t);
        Self { t, b, axes, gast: gast(t - delta_t_days, t).rad() }
    }

    /// 基本面坐标 → 当日真赤道架。
    fn to_equatorial(&self, xi: Real, eta: Real, zeta: Real) -> V3 {
        let [i, j, k] = self.axes;
        std::array::from_fn(|n| xi * i[n] + eta * j[n] + zeta * k[n])
    }

    /// 当日真赤道架 → 地固系（绕 z 轴转 −GAST）。
    fn to_itrs(&self, v: V3) -> V3 {
        let (s, c) = (self.gast.sin(), self.gast.cos());
        [v[0] * c + v[1] * s, -v[0] * s + v[1] * c, v[2]]
    }

    /// 地固系点在本时刻基本面上的坐标 (ξ, η, ζ)。
    fn fundamental_of_itrs(&self, g: V3) -> V3 {
        let (s, c) = (self.gast.sin(), self.gast.cos());
        let v = [g[0] * c - g[1] * s, g[0] * s + g[1] * c, g[2]];
        let [i, j, k] = self.axes;
        [dot(v, i), dot(v, j), dot(v, k)]
    }

    fn cone(&self, kind: Cone) -> (Real, Real) {
        match kind {
            Cone::Penumbra => (self.b.l1, self.b.tan_f1),
            Cone::Umbra => (self.b.l2, self.b.tan_f2),
        }
    }

    /// 地球在基本面上投影（椭圆）的 η 半轴 ρ₁ = √(1 − e²cos²d)；ξ 半轴为 1。
    fn rho1(&self) -> Real {
        self.b.rho1()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cone {
    Penumbra,
    Umbra,
}

/// 椭球二次型 q(u, v) = uₓvₓ + u_yv_y + u_zv_z/(1 − e²)。
fn qform(u: V3, v: V3) -> Real {
    u[0] * v[0] + u[1] * v[1] + u[2] * v[2] / (real_const(1.0) - EARTH_E2)
}

/// 直线 A + ζ·D（赤道架，地球赤道半径为单位）与椭球的向日交点；不相交返回 None。
fn hit_ellipsoid(a: V3, d: V3) -> Option<V3> {
    let (qa, qb, qc) = (qform(d, d), qform(a, d), qform(a, a) - real_const(1.0));
    let disc = qb * qb - qa * qc;
    if disc < real_const(0.0) {
        return None;
    }
    let zeta = (-qb + disc.sqrt()) / qa;
    Some(std::array::from_fn(|n| a[n] + zeta * d[n]))
}

/// 影锥母线（方向角 θ）与地面的交点，地固系。
fn cone_ground_point(f: &Frame, kind: Cone, theta: Real) -> Option<V3> {
    let (l, tan_f) = f.cone(kind);
    let (s, c) = (theta.sin(), theta.cos());
    let a = f.to_equatorial(f.b.x + l * c, f.b.y + l * s, real_const(0.0));
    let d = f.to_equatorial(-tan_f * c, -tan_f * s, real_const(1.0));
    hit_ellipsoid(a, d).map(|v| f.to_itrs(v))
}

/// 影轴与地面的交点，地固系。
fn axis_ground_point(f: &Frame) -> Option<V3> {
    let a = f.to_equatorial(f.b.x, f.b.y, real_const(0.0));
    let d = f.to_equatorial(real_const(0.0), real_const(0.0), real_const(1.0));
    hit_ellipsoid(a, d).map(|v| f.to_itrs(v))
}

/// 地球边缘（基本面投影椭圆上参数 ψ 处）对应的地面切点，地固系。
fn limb_ground_point(f: &Frame, psi: Real) -> V3 {
    let a = f.to_equatorial(psi.cos(), f.rho1() * psi.sin(), real_const(0.0));
    let d = f.to_equatorial(real_const(0.0), real_const(0.0), real_const(1.0));
    // 切线：判别式理论上为 0，取重根
    let zeta = -qform(a, d) / qform(d, d);
    f.to_itrs(std::array::from_fn(|n| a[n] + zeta * d[n]))
}

/// 地面固定点（地固系）在某时刻的 F = |Δ| − |L|：到影轴距离减该处影锥半径；< 0 在影内。
fn shadow_excess(f: &Frame, kind: Cone, g: V3) -> Real {
    let [xi, eta, zeta] = f.fundamental_of_itrs(g);
    let (l, tan_f) = f.cone(kind);
    let (dx, dy) = (f.b.x - xi, f.b.y - eta);
    (dx * dx + dy * dy).sqrt() - (l - zeta * tan_f).abs()
}

/// 地固系椭球面上一点 → 大地经纬度。
fn to_map_point(t: Real, g: V3) -> MapPoint {
    let p = (g[0] * g[0] + g[1] * g[1]).sqrt();
    MapPoint {
        jd_tt: t,
        longitude: PlaneAngle::from_rad(g[1].atan2(g[0])),
        latitude: PlaneAngle::from_rad(g[2].atan2((real_const(1.0) - EARTH_E2) * p)),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 扫描与连线

/// θ 上的求根：h 在一周上的零点，h 返回 None 处（该方向的影锥母线不落地）跳过。
///
/// 先在 n 个等分点上取样；一端落地、一端不落地的区间二分出落地边界，并从落地端向边界按几何级数加密——靠近边界时
/// θ 到地面点的映射被急剧拉伸（边界处导数无穷大），θ 上很近的点地面上可隔上千公里。取样点按落地的连续段排成
/// θ 递增的序列，在每段上：相邻两点变号则二分求根；三点同号而中间 |h| 最小时，用黄金分割求极值，极值变号说明
/// 有一对根（尖点附近两根相向靠拢）同落一个小区间，两侧各二分一根。
fn roots_on_circle(n: usize, h: impl Fn(Real) -> Option<Real>) -> Vec<Real> {
    let two_pi = real_const(2.0) * Real::pi();
    let at = |k: usize| two_pi * real(k as f64) / real(n as f64);
    let vals: Vec<Option<Real>> = (0..n).map(|k| h(at(k))).collect();

    // 在 [hit, miss] 之间二分出落地边界（返回落地一侧）
    let edge = |mut x: Real, mut y: Real| {
        for _ in 0..50 {
            let m = (x + y) / real_const(2.0);
            if h(m).is_some() {
                x = m;
            } else {
                y = m;
            }
        }
        x
    };
    // 从 start 向边界 x 几何加密的点（不含 start），按离 start 由近到远排列，末项为 x
    let toward_edge = |start: Real, x: Real| -> Vec<(Real, Real)> {
        (1..=36)
            .map(|jj| x + (start - x) * real_const(0.5_f64.powi(jj)))
            .chain(std::iter::once(x))
            .filter_map(|th| h(th).map(|v| (th, v)))
            .collect()
    };

    let mut runs: Vec<Vec<(Real, Real)>> = Vec::new();
    if vals.iter().all(Option::is_some) {
        // 整圈落地：首尾各补一点，保证跨 0 的区间与三点组都被覆盖（成对检查从第 2 对起，避免重复）
        let mut run = vec![(at(n - 1) - two_pi, vals[n - 1].unwrap())];
        run.extend((0..n).map(|k| (at(k), vals[k].unwrap())));
        run.push((two_pi, vals[0].unwrap()));
        runs.push(run);
    } else if vals.iter().any(Option::is_some) {
        // 从一个「不落地 → 落地」处起走一圈；θ 展开为单调递增
        let k0 = (0..n).find(|&k| vals[k].is_some() && vals[(k + n - 1) % n].is_none()).unwrap();
        let mut cur: Vec<(Real, Real)> = Vec::new();
        for step in 0..n {
            let k = (k0 + step) % n;
            let th = at(k0) + two_pi * real(step as f64) / real(n as f64);
            let th_next = th + two_pi / real(n as f64);
            let (here, next) = (vals[k], vals[(k + 1) % n]);
            match (here, next) {
                (Some(v), Some(_)) => cur.push((th, v)),
                (Some(v), None) => {
                    cur.push((th, v));
                    cur.extend(toward_edge(th, edge(th, th_next)));
                    runs.push(std::mem::take(&mut cur));
                }
                (None, Some(_)) => {
                    let mut pts = toward_edge(th_next, edge(th_next, th));
                    pts.reverse();
                    cur.extend(pts);
                }
                (None, None) => {}
            }
        }
        if !cur.is_empty() {
            runs.push(cur);
        }
    }

    let neg = |v: Real| v < real_const(0.0);
    let mut roots = Vec::new();
    for run in &runs {
        let circular = runs.len() == 1 && vals.iter().all(Option::is_some);
        for (idx, w) in run.windows(2).enumerate() {
            if circular && idx == 0 {
                continue;
            }
            let ((l, vl), (r, vr)) = (w[0], w[1]);
            if neg(vl) != neg(vr) {
                let far = real_const(1e9) * if neg(vl) { real_const(-1.0) } else { real_const(1.0) };
                roots.extend(bisect_root(|t| h(t).unwrap_or(far), l, r, real_const(1e-12)));
            }
        }
        for w in run.windows(3) {
            let ((l, a), (_, m), (r, b)) = (w[0], w[1], w[2]);
            if neg(a) != neg(m) || neg(m) != neg(b) || m.abs() >= a.abs() || m.abs() >= b.abs() {
                continue;
            }
            let sgn = if neg(m) { real_const(-1.0) } else { real_const(1.0) };
            let f = |t: Real| h(t).map(|v| v * sgn).unwrap_or(real_const(1e9));
            // 容差下限 1e-13 rad：几何加密后区间可短到 1e-13 量级，再按比例取会低于 f64 在 θ≈2π 处的分辨率而死循环
            let tol = ((r - l) * real_const(1e-9)).max(real_const(1e-13));
            let (tm, vm) = minimize_golden(f, l, r, tol);
            if vm < real_const(0.0) {
                roots.extend(bisect_root(f, l, tm, tol));
                roots.extend(bisect_root(f, tm, r, tol));
            }
        }
    }
    roots.iter().map(|&t| t - two_pi * (t / two_pi).floor()).collect()
}

/// 按时刻逐步把点连成折线：新点接到上一步末端最近（地固系弦长 < `max_chord`）的折线上，否则另起一条。
struct TrackLinker {
    max_chord: Real,
    open: Vec<(V3, Polyline)>,
    closed: Vec<Polyline>,
}

impl TrackLinker {
    fn new(max_chord: f64) -> Self {
        Self { max_chord: real(max_chord), open: Vec::new(), closed: Vec::new() }
    }

    fn step(&mut self, t: Real, points: Vec<V3>) {
        let mut prev = std::mem::take(&mut self.open);
        let mut next: Vec<(V3, Polyline)> = Vec::new();
        for g in points {
            let best = prev
                .iter()
                .enumerate()
                .map(|(n, (e, _))| (n, norm(sub(*e, g))))
                .filter(|(_, d)| *d < self.max_chord)
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            match best {
                Some((n, _)) => {
                    let (_, mut line) = prev.swap_remove(n);
                    line.push(to_map_point(t, g));
                    next.push((g, line));
                }
                None => {
                    // 新起点：同一步另有新起且离得近的折线（分叉，如带食出没界线自 P1 起分两支），从其起点画起，免得断开
                    let mut line = Vec::new();
                    if let Some((_, l)) = next.iter().find(|(e, l)| l.len() == 1 && norm(sub(*e, g)) < self.max_chord) {
                        line.push(l[0]);
                    }
                    line.push(to_map_point(t, g));
                    next.push((g, line));
                }
            }
        }
        self.closed.extend(prev.into_iter().map(|(_, l)| l));
        self.open = next;
    }

    fn finish(mut self) -> Vec<Polyline> {
        self.closed.extend(self.open.into_iter().map(|(_, l)| l));
        self.closed.retain(|l| l.len() >= 2);
        self.closed
    }
}

/// 一条（组）曲线的时间扫描。点数在相邻两步间变化（曲线出现、终止或成对生灭）时递归细分该步：每层 8 等分，共 3 层，
/// 最内层再二分出变化时刻。边缘附近影子在地面上因投影缩短移动极快，界线还可能在边缘旁折出尖点、生出只存在十几秒的
/// 短支（地面上却长达数百公里）；只按 1 分钟步长会漏掉它们，在端点处留下缺口。
struct Sweep {
    linker: TrackLinker,
    prev: Option<(Real, Vec<V3>)>,
}

impl Sweep {
    fn new(max_chord: f64) -> Self {
        Self { linker: TrackLinker::new(max_chord), prev: None }
    }

    /// 在 (ta, tb) 内补点（不含两端），结果追加到 out。
    fn refine(probe: &dyn Fn(Real) -> Vec<V3>, ta: Real, na: usize, tb: Real, nb: usize, depth: u32, out: &mut Vec<(Real, Vec<V3>)>) {
        if depth == 0 {
            // 二分出变化时刻：a 端点数仍为 na
            let (mut a, mut b) = (ta, tb);
            let (mut pa, mut pb) = (None, None);
            for _ in 0..20 {
                let m = (a + b) / real_const(2.0);
                let pm = probe(m);
                if pm.len() == na {
                    a = m;
                    pa = Some(pm);
                } else {
                    b = m;
                    pb = Some(pm);
                }
            }
            out.extend(pa.map(|p| (a, p)));
            out.extend(pb.map(|p| (b, p)));
            let _ = nb;
            return;
        }
        const K: usize = 8;
        let ts: Vec<Real> = (1..K).map(|k| ta + (tb - ta) * real(k as f64 / K as f64)).collect();
        let samples: Vec<(Real, Vec<V3>)> = ts.into_iter().map(|t| (t, probe(t))).collect();
        let mut prev = (ta, na);
        for (t, pts) in samples.iter().map(|(t, p)| (*t, p.len())).chain(std::iter::once((tb, nb))) {
            if pts != prev.1 {
                Self::refine(probe, prev.0, prev.1, t, pts, depth - 1, out);
            }
            prev = (t, pts);
        }
        out.extend(samples);
    }

    fn advance(&mut self, t: Real, points: Vec<V3>, probe: &dyn Fn(Real) -> Vec<V3>) {
        if let Some((tp, pp)) = &self.prev {
            if pp.len() != points.len() {
                let mut extra = Vec::new();
                Self::refine(probe, *tp, pp.len(), t, points.len(), 3, &mut extra);
                extra.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
                extra.dedup_by(|x, y| x.0 == y.0);
                for (te, pts) in extra {
                    self.linker.step(te, pts);
                }
            }
        }
        self.linker.step(t, points.clone());
        self.prev = Some((t, points));
    }

    fn finish(self) -> Vec<Polyline> {
        self.linker.finish()
    }
}

/// 某时刻的南北界点：在影子轮廓上求 dF/dt = 0 的 θ，按在影轴运动方向左（北）右（南）分开。
fn limit_points(fm: &Frame, f: &Frame, fp: &Frame, kind: Cone, n: usize) -> (Vec<V3>, Vec<V3>) {
    let dfdt = |th: Real| {
        let g = cone_ground_point(f, kind, th)?;
        Some(shadow_excess(fp, kind, g) - shadow_excess(fm, kind, g))
    };
    // 影轴在基本面上的运动方向，其左法向记为北
    let (vx, vy) = (fp.b.x - fm.b.x, fp.b.y - fm.b.y);
    let (mut north, mut south) = (Vec::new(), Vec::new());
    for th in roots_on_circle(n, dfdt) {
        let Some(g) = cone_ground_point(f, kind, th) else { continue };
        let [xi, eta, _] = f.fundamental_of_itrs(g);
        let side = -vy * (xi - f.b.x) + vx * (eta - f.b.y);
        if side > real_const(0.0) {
            north.push(g);
        } else {
            south.push(g);
        }
    }
    (north, south)
}

/// 某时刻半影轮廓与地球边缘的交点（带食出没界线上的点）。
fn rise_set_points(f: &Frame, n: usize) -> Vec<V3> {
    let h = |psi: Real| Some(shadow_excess(f, Cone::Penumbra, limb_ground_point(f, psi)));
    roots_on_circle(n, h).into_iter().map(|psi| limb_ground_point(f, psi)).collect()
}

/// 影子轮廓：θ 一周取样，落地的连续段各成一条折线（首尾相接时合并）。
fn outline(f: &Frame, kind: Cone, n: usize) -> Vec<Polyline> {
    let two_pi = real_const(2.0) * Real::pi();
    let pts: Vec<Option<V3>> = (0..n).map(|k| cone_ground_point(f, kind, two_pi * real(k as f64) / real(n as f64))).collect();
    let mut runs: Vec<Polyline> = Vec::new();
    let mut cur: Polyline = Vec::new();
    for p in &pts {
        match p {
            Some(g) => cur.push(to_map_point(f.t, *g)),
            None if !cur.is_empty() => runs.push(std::mem::take(&mut cur)),
            None => {}
        }
    }
    if pts.iter().all(Option::is_some) {
        // 整圈落地：闭合
        let first = cur[0];
        cur.push(first);
        return vec![cur];
    }
    if !cur.is_empty() {
        // 末段与首段在 θ = 0 处相连
        if pts[0].is_some() && !runs.is_empty() {
            let mut head = runs.remove(0);
            cur.append(&mut head);
        }
        runs.push(cur);
    }
    runs.retain(|r| r.len() >= 2);
    runs
}

// ---------------------------------------------------------------------------------------------------------------------
// 逐地可见极大食分与等值线

/// 一个时刻的基本面，三轴已转到地固系：ξ = rᵢ·G 等（G 为地面点地固系坐标，地球赤道半径为单位）。
#[derive(Clone, Copy)]
struct FrameRow {
    t: Real,
    r: [V3; 3],
    x: Real,
    y: Real,
    l1: Real,
    l2: Real,
    tan_f1: Real,
    tan_f2: Real,
}

/// 某地某刻：(ζ, 食分)。食分按基本面定义 m = (L₁ − Δ)/(L₁ + L₂)，影外为负，连续可比；ζ > 0 太阳在（几何）地平上。
fn zeta_and_magnitude(row: &FrameRow, g: V3) -> (Real, Real) {
    let [xi, eta, zeta] = row.r.map(|r| dot(r, g));
    let (dx, dy) = (row.x - xi, row.y - eta);
    let big_l1 = row.l1 - zeta * row.tan_f1;
    let big_l2 = row.l2 - zeta * row.tan_f2;
    (zeta, (big_l1 - (dx * dx + dy * dy).sqrt()) / (big_l1 + big_l2))
}

/// 某地可见极大食分：(食分, 时刻, 是否在地平处取得)。全程太阳在地平下返回 None。
/// 时刻表内取最大，内部极大用三点抛物线细化；极大在可见时段端点（日出、日没）时按相邻两刻线性插值到 ζ = 0。
/// 这里是全图最热的循环（网格每点、二分每步都走一遍时刻表），故地平下的时刻不算食分，也不存中间结果。
fn visible_max_magnitude(rows: &[FrameRow], g: V3) -> Option<(Real, Real, bool)> {
    let zeta_of = |r: &FrameRow| dot(r.r[2], g);
    let vis = |k: usize| zeta_of(&rows[k]) > real_const(0.0);
    let mut best: Option<(usize, Real)> = None;
    for (k, row) in rows.iter().enumerate() {
        if zeta_of(row) > real_const(0.0) {
            let m = zeta_and_magnitude(row, g).1;
            if best.is_none_or(|(_, b)| m > b) {
                best = Some((k, m));
            }
        }
    }
    let (k, b) = best?;
    let n = rows.len();
    let inner = k > 0 && k + 1 < n && vis(k - 1) && vis(k + 1);
    if inner {
        let (a, c) = (zeta_and_magnitude(&rows[k - 1], g).1, zeta_and_magnitude(&rows[k + 1], g).1);
        let den = a - real_const(2.0) * b + c;
        let u = if den < real_const(0.0) { (a - c) / (real_const(2.0) * den) } else { real_const(0.0) };
        let mag = b - (a - c) * u / real_const(4.0);
        let dt = rows[k + 1].t - rows[k].t;
        return Some((mag, rows[k].t + u * dt, false));
    }
    // 端点：向不可见一侧插值到地平
    let j = if k + 1 < n && !vis(k + 1) {
        k + 1
    } else if k > 0 && !vis(k - 1) {
        k - 1
    } else {
        return Some((b, rows[k].t, false)); // 时刻表边界（不应发生：表覆盖 P1–P4 以外）
    };
    let (z0, mj) = (zeta_of(&rows[k]), zeta_and_magnitude(&rows[j], g));
    let s = z0 / (z0 - mj.0);
    let mag = b + (mj.1 - b) * s;
    // 地平处的值若反而小于时刻表内最大值（插值误差），取表内值
    let mag = if mag > b { mag } else { b };
    Some((mag, rows[k].t + (rows[j].t - rows[k].t) * s, true))
}

/// 大地经纬度 → 椭球面地固系坐标（地球赤道半径为单位）。
fn ground_point(lon: Real, lat: Real) -> V3 {
    let (sp, cp) = (lat.sin(), lat.cos());
    let n = real_const(1.0) / (real_const(1.0) - EARTH_E2 * sp * sp).sqrt();
    [n * cp * lon.cos(), n * cp * lon.sin(), n * (real_const(1.0) - EARTH_E2) * sp]
}

/// 可见极大食分网格与等值线。
struct MagnitudeGrid<'a> {
    rows: &'a [FrameRow],
    step: Real,
    nlon: usize,
    nlat: usize,
    /// [纬度序][经度序]：None 为全程在地平下
    vals: Vec<Vec<Option<Real>>>,
}

/// 全程在地平下的点在等值线计算中的取值（远低于任何食分）。
const NEVER_VISIBLE: f64 = -1.0e3;

impl<'a> MagnitudeGrid<'a> {
    fn new(rows: &'a [FrameRow], step_deg: f64) -> Self {
        let nlon = (360.0 / step_deg).round() as usize;
        let nlat = (180.0 / step_deg).round() as usize + 1;
        let step = PlaneAngle::from_deg(real(step_deg)).rad();
        let mut g = Self { rows, step, nlon, nlat, vals: Vec::new() };
        g.vals = (0..nlat)
            .map(|i| (0..nlon).map(|j| visible_max_magnitude(rows, ground_point(g.lon(j), g.lat(i))).map(|v| v.0)).collect())
            .collect();
        g
    }
    fn lon(&self, j: usize) -> Real {
        -Real::pi() + self.step * real(j as f64)
    }
    fn lat(&self, i: usize) -> Real {
        -Real::pi() / real_const(2.0) + self.step * real(i as f64)
    }
    fn val(&self, i: usize, j: usize, level: Real) -> Real {
        self.vals[i][j % self.nlon].unwrap_or(real_const(NEVER_VISIBLE)) - level
    }
    fn exact(&self, lon: Real, lat: Real) -> Option<(Real, Real, bool)> {
        visible_max_magnitude(self.rows, ground_point(lon, lat))
    }

    /// 网格边上的等值点：沿边二分（经度边或纬度边），返回 (经度, 纬度, 时刻, 是否地平处)。
    fn edge_point(&self, a: (usize, usize), b: (usize, usize), level: Real) -> (Real, Real, Real, bool) {
        let (la, lo_a) = (self.lat(a.0), self.lon(a.1));
        let (lb, lo_b) = (self.lat(b.0), self.lon(a.1) + self.step * real((b.1 as i64 - a.1 as i64) as f64));
        let f = |s: Real| {
            let (lo, la_) = (lo_a + (lo_b - lo_a) * s, la + (lb - la) * s);
            self.exact(lo, la_).map(|v| v.0).unwrap_or(real_const(NEVER_VISIBLE)) - level
        };
        // 容差以网格边长计：1e-4 × 1° 约 11 m，远小于 ΔT 不确定度造成的平移
        let s = bisect_root(f, real_const(0.0), real_const(1.0), real_const(1e-4)).unwrap_or(real_const(0.5));
        let (lo, la_) = (lo_a + (lo_b - lo_a) * s, la + (lb - la) * s);
        let (t, horizon) = self.exact(lo, la_).map(|v| (v.1, v.2)).unwrap_or((real_const(0.0), true));
        (lo, la_, t, horizon)
    }

    /// 等值线（0 级）上内部极大点 a 与地平处极大点 b 之间的交接点：沿 a→b 二分；每个试探点先沿 a→b 的法向
    /// 二分回到 0 等值线，再看极大是否在地平处。
    fn junction(&self, a: MapPoint, b: MapPoint) -> MapPoint {
        let (la0, lo0) = (a.latitude.rad(), a.longitude.rad());
        let mut dlo = b.longitude.rad() - lo0;
        if dlo > Real::pi() {
            dlo = dlo - real_const(2.0) * Real::pi();
        } else if dlo < -Real::pi() {
            dlo = dlo + real_const(2.0) * Real::pi();
        }
        let dla = b.latitude.rad() - la0;
        let c = la0.cos().abs().max(real_const(1e-3));
        // 局部东-北平面：切向 (dlo·c, dla)，法向 (−dla, dlo·c)
        let on_contour = |s: Real| -> (Real, Real, Option<(Real, Real, bool)>) {
            let (lo, la) = (lo0 + dlo * s, la0 + dla * s);
            let f = |u: Real| {
                let (lo_u, la_u) = (lo - dla * u / c, la + dlo * c * u);
                self.exact(lo_u, la_u).map(|v| v.0).unwrap_or(real_const(NEVER_VISIBLE))
            };
            let u = bisect_root(f, real_const(-0.5), real_const(0.5), real_const(1e-9)).unwrap_or(real_const(0.0));
            let (lo_u, la_u) = (lo - dla * u / c, la + dlo * c * u);
            (lo_u, la_u, self.exact(lo_u, la_u))
        };
        let (mut s0, mut s1) = (real_const(0.0), real_const(1.0));
        for _ in 0..30 {
            let m = (s0 + s1) / real_const(2.0);
            if on_contour(m).2.is_some_and(|v| !v.2) {
                s0 = m;
            } else {
                s1 = m;
            }
        }
        let (lo, la, v) = on_contour(s0);
        MapPoint {
            jd_tt: v.map(|v| v.1).unwrap_or(a.jd_tt),
            longitude: PlaneAngle::from_rad(lo).wrap_to_signed_pi(),
            latitude: PlaneAngle::from_rad(la),
        }
    }

    /// marching squares：返回等值线折线（每点附「是否地平处取得」）。经度方向首尾相接。
    fn contour(&self, level: Real) -> Vec<Vec<(MapPoint, bool)>> {
        use std::collections::HashMap;
        // 边的编号：(i, j, 0) 为 (i,j)-(i,j+1) 经度边，(i, j, 1) 为 (i,j)-(i+1,j) 纬度边
        type Edge = (usize, usize, u8);
        let mut adj: HashMap<Edge, Vec<Edge>> = HashMap::new();
        let pos = |v: Real| v >= real_const(0.0);
        for i in 0..self.nlat - 1 {
            for j in 0..self.nlon {
                let jn = (j + 1) % self.nlon;
                // 角点：0 (i,j)，1 (i,j+1)，2 (i+1,j+1)，3 (i+1,j)
                let c = [self.val(i, j, level), self.val(i, jn, level), self.val(i + 1, jn, level), self.val(i + 1, j, level)];
                let edges: [Edge; 4] = [(i, j, 0), (i, jn, 1), (i + 1, j, 0), (i, j, 1)]; // 下、右、上、左
                let crossing: Vec<usize> = (0..4).filter(|&e| pos(c[e]) != pos(c[(e + 1) % 4])).collect();
                let pairs: Vec<(usize, usize)> = match crossing.len() {
                    2 => vec![(crossing[0], crossing[1])],
                    4 => {
                        // 鞍点：按中心值决定连法
                        let center = (c[0] + c[1] + c[2] + c[3]) / real_const(4.0);
                        if pos(center) == pos(c[0]) { vec![(0, 1), (2, 3)] } else { vec![(0, 3), (1, 2)] }
                    }
                    _ => vec![],
                };
                for (e1, e2) in pairs {
                    adj.entry(edges[e1]).or_default().push(edges[e2]);
                    adj.entry(edges[e2]).or_default().push(edges[e1]);
                }
            }
        }
        let mut cache: HashMap<Edge, (MapPoint, bool)> = HashMap::new();
        let mut point = |e: Edge| -> (MapPoint, bool) {
            *cache.entry(e).or_insert_with(|| {
                let b = if e.2 == 0 { (e.0, e.1 + 1) } else { (e.0 + 1, e.1) };
                let (lo, la, t, hz) = self.edge_point((e.0, e.1), b, level);
                let lo = PlaneAngle::from_rad(lo).wrap_to_signed_pi();
                (MapPoint { jd_tt: t, longitude: lo, latitude: PlaneAngle::from_rad(la) }, hz)
            })
        };
        // 串成折线：先从端点（度为 1）出发，再处理闭合环
        let mut used: std::collections::HashSet<(Edge, Edge)> = std::collections::HashSet::new();
        let mut lines = Vec::new();
        let mut starts: Vec<Edge> = adj.iter().filter(|(_, v)| v.len() == 1).map(|(k, _)| *k).collect();
        starts.sort();
        let mut all: Vec<Edge> = adj.keys().copied().collect();
        all.sort();
        starts.extend(all);
        for s0 in starts {
            loop {
                let Some(&next) = adj[&s0].iter().find(|n| !used.contains(&(s0, **n))) else { break };
                let mut line = vec![point(s0)];
                let (mut prev, mut cur) = (s0, next);
                used.insert((prev, cur));
                used.insert((cur, prev));
                loop {
                    line.push(point(cur));
                    let Some(&nx) = adj[&cur].iter().find(|n| **n != prev && !used.contains(&(cur, **n))) else { break };
                    used.insert((cur, nx));
                    used.insert((nx, cur));
                    prev = cur;
                    cur = nx;
                }
                if line.len() >= 2 {
                    lines.push(line);
                }
            }
        }
        lines
    }
}

/// 0 级等值线中极大不在地平处的部分 = 半影界。闭合环先转到从地平处的点起算，免得在接缝处断开；每段两端在
/// 「内部极大」与「地平处极大」两相邻点之间二分出交接点（该处擦边恰在地平上，即与带食出没界线相接）。
fn penumbral_limits_from_contour(grid: &MagnitudeGrid, lines: &[Vec<(MapPoint, bool)>]) -> Vec<Polyline> {
    let mut out = Vec::new();
    for line in lines {
        let mut line = line.clone();
        let closed = line.len() > 2 && {
            let (a, b) = (line[0].0, line[line.len() - 1].0);
            (a.longitude.rad() - b.longitude.rad()).abs() < real_const(1e-9) && (a.latitude.rad() - b.latitude.rad()).abs() < real_const(1e-9)
        };
        if closed {
            line.pop();
            if let Some(k) = line.iter().position(|p| p.1) {
                line.rotate_left(k);
            }
            let first = line[0];
            line.push(first);
        }
        let n = line.len();
        let mut k = 0;
        while k < n {
            if line[k].1 {
                k += 1;
                continue;
            }
            let start = k;
            while k < n && !line[k].1 {
                k += 1;
            }
            let mut seg: Polyline = line[start..k].iter().map(|p| p.0).collect();
            if start > 0 {
                seg.insert(0, grid.junction(line[start].0, line[start - 1].0));
            }
            if k < n {
                seg.push(grid.junction(line[k - 1].0, line[k].0));
            }
            if seg.len() >= 2 {
                out.push(seg);
            }
        }
    }
    out
}

/// 求日食地图。`src` 通常是 [`super::eclipse_window`] 建好的插值窗；`jd_tt_guess` 为近朔时刻（误差 1 日内）；
/// `delta_t` 为 TT − UT1。无日食返回 None。
pub fn solar_eclipse_map(
    src: &dyn SunMoonSource,
    jd_tt_guess: Real,
    delta_t: Duration,
    options: &SolarEclipseMapOptions,
) -> Option<SolarEclipseMap> {
    let global = solar_eclipse_global_in(src, jd_tt_guess)?;
    let t0 = global.greatest_tt;
    let dt_days = delta_t.seconds() / real_const(86400.0);
    let frame = |t: Real| Frame::new(src, t, dt_days);

    // P1、P4：影轴到地心（按扁率缩放）距离 = 1 + l₁；中心线两端：= 1
    let m1 = |t: Real| {
        let (b, _) = besselian_with_axes(src, t);
        b.m1()
    };
    let span = real_const(0.35);
    let tol = seconds(0.5);
    let pen = |t: Real| m1(t) - real_const(1.0) - besselian_with_axes(src, t).0.l1;
    let p1 = bisect_root(pen, t0 - span, t0, tol)?;
    let p4 = bisect_root(pen, t0, t0 + span, tol)?;
    let central_contacts = if global.central {
        let axis = |t: Real| m1(t) - real_const(1.0);
        bisect_root(axis, t0 - span, t0, tol).zip(bisect_root(axis, t0, t0 + span, tol))
    } else {
        None
    };

    let f0 = frame(t0);
    let greatest_point = match axis_ground_point(&f0) {
        Some(g) if global.central => to_map_point(t0, g),
        _ => {
            let psi = (f0.b.y / f0.rho1()).atan2(f0.b.x);
            to_map_point(t0, limb_ground_point(&f0, psi))
        }
    };

    let k = f0.to_itrs(f0.axes[2]);
    let subsolar = MapPoint {
        jd_tt: t0,
        longitude: PlaneAngle::from_rad(k[1].atan2(k[0])),
        latitude: PlaneAngle::from_rad(k[2].asin()),
    };

    // 扫描：比 P1–P4 两端各多一步，保证界线端点不漏
    let step = options.step.seconds() / real_const(86400.0);
    let h = seconds(30.0);
    let n = options.theta_samples;
    // 南北两条本影界由同一次计算得出，记住上一刻的结果，免得每个时刻算两遍
    let umbra_memo: std::cell::RefCell<Option<(Real, (Vec<V3>, Vec<V3>))>> = std::cell::RefCell::new(None);
    let limits = |t: Real| {
        let mut memo = umbra_memo.borrow_mut();
        if let Some((mt, v)) = memo.as_ref() {
            if *mt == t {
                return v.clone();
            }
        }
        let (fm, f, fp) = (frame(t - h), frame(t), frame(t + h));
        let v = limit_points(&fm, &f, &fp, Cone::Umbra, n);
        *memo = Some((t, v.clone()));
        v
    };
    let probes: [Box<dyn Fn(Real) -> Vec<V3> + '_>; 4] = [
        Box::new(|t| axis_ground_point(&frame(t)).into_iter().collect()),
        Box::new(|t| limits(t).0),
        Box::new(|t| limits(t).1),
        Box::new(|t| rise_set_points(&frame(t), 2 * n)),
    ];
    let mut sweeps: Vec<Sweep> = [0.2, 0.2, 0.2, 0.3].iter().map(|&c| Sweep::new(c)).collect();
    let mut rows = Vec::new();
    let steps = ((p4 - p1) / step).as_f64().ceil() as i64 + 2;
    for s in -1..=steps {
        let t = p1 + step * real(s as f64);
        for (sw, probe) in sweeps.iter_mut().zip(probes.iter()) {
            sw.advance(t, probe(t), probe.as_ref());
        }
        let f = frame(t);
        rows.push(FrameRow {
            t,
            r: f.axes.map(|a| f.to_itrs(a)),
            x: f.b.x,
            y: f.b.y,
            l1: f.b.l1,
            l2: f.b.l2,
            tan_f1: f.b.tan_f1,
            tan_f2: f.b.tan_f2,
        });
    }
    let [central, un, us, rs]: [Vec<Polyline>; 4] =
        sweeps.into_iter().map(Sweep::finish).collect::<Vec<_>>().try_into().ok().unwrap();

    // 逐地可见极大食分：0 级等值线给半影界，其余给食分等值线
    let grid = MagnitudeGrid::new(&rows, options.grid_step.rad().as_f64().to_degrees());
    let zero = grid.contour(real_const(0.0));
    let penumbra_limits = penumbral_limits_from_contour(&grid, &zero);
    let magnitude_contours = options
        .magnitude_levels
        .iter()
        .map(|&level| MagnitudeContour {
            level,
            lines: grid.contour(level).into_iter().map(|l| l.into_iter().map(|p| p.0).collect()).collect(),
        })
        .collect();

    // 影子轮廓：按 UT 整 interval 对齐
    let mut outlines = Vec::new();
    if let Some(iv) = options.outline_interval {
        let iv = iv.seconds() / real_const(86400.0);
        let ut_start = ((p1 - dt_days) / iv).as_f64().ceil();
        let mut k = ut_start;
        loop {
            let t = real(k) * iv + dt_days;
            if t > p4 {
                break;
            }
            let f = frame(t);
            outlines.push(ShadowOutline { jd_tt: t, penumbra: outline(&f, Cone::Penumbra, n), umbra: outline(&f, Cone::Umbra, n / 2) });
            k += 1.0;
        }
    }

    Some(SolarEclipseMap {
        global,
        delta_t,
        penumbra_contacts: (p1, p4),
        central_contacts,
        greatest_point,
        subsolar,
        central_line: central,
        umbra_north: un,
        umbra_south: us,
        penumbra_limits,
        rise_set: rs,
        magnitude_contours,
        outlines,
    })
}
