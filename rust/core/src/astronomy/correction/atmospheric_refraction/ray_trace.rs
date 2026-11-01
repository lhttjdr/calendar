//! 分层大气光线追迹：不靠拟合式，直接积分折射积分。
//!
//! 球对称大气中 n·r·sin z = C（Garfinkel 1967 式 2），折射 R = ∫ tan z · (−d ln n)（式 3）。转折点处 tan z 发散，
//! 换元 x = √(r − r_t) 消去。空气取干空气理想气体、静力平衡、Gladstone–Dale（n − 1 ∝ 密度），温度分段线性；
//! 高度视同位势高度，与 Garfinkel 的表相符到 0.2% 内（见 tests）。

use crate::math::real::{real, real_const, zero, Real, RealOps};

use super::apparent_altitude::Atmosphere;

/// 8 点 Gauss–Legendre 节点与权（[−1, 1]，对称取半）。
const GAUSS_X: [f64; 4] = [0.183_434_642_495_649_8, 0.525_532_409_916_329, 0.796_666_477_413_626_7, 0.960_289_856_497_536_3];
const GAUSS_W: [f64; 4] = [0.362_683_783_378_362, 0.313_706_645_877_887_3, 0.222_381_034_453_374_5, 0.101_228_536_290_376_3];
/// 每层在 x 上等分的段数，平滑性见 `ray_trace_quadrature_converges` 测试。
const SEGMENTS: usize = 8;
/// 折射积分的上界 [m]：其上密度已可忽略。
const TOP_HEIGHT_M: f64 = 120_000.0;
/// 从观测站往上最多：逆温层、对流层、平流层。
const MAX_LAYERS: usize = 3;

/// 物理常数。缺省取 Garfinkel 1967 表 II 修订值（578 nm，1962 美国标准大气）。
#[derive(Clone, Copy, Debug)]
pub struct RayTraceConstants {
    /// 0°C、760 mmHg 干空气的 n − 1。
    pub refractivity: Real,
    pub gravity: Real,
    pub gas_constant: Real,
    pub earth_radius_m: Real,
    pub tropopause_m: Real,
    /// 视线撞地的高度 [m]，地平以下的光线到此为止。
    pub ground_m: Real,
}

impl Default for RayTraceConstants {
    fn default() -> Self {
        Self {
            refractivity: real_const(2.9241e-4),
            gravity: real_const(9.80655),
            gas_constant: real_const(287.053),
            earth_radius_m: real_const(6_378_390.0),
            tropopause_m: real_const(11_019.0),
            ground_m: zero(),
        }
    }
}

#[derive(Clone, Copy)]
struct Layer {
    base_m: Real,
    base_k: Real,
    base_pa: Real,
    /// 温度梯度 [K/m]，0 为等温。
    gradient: Real,
}

struct Profile {
    layers: [Layer; MAX_LAYERS],
    count: usize,
    constants: RayTraceConstants,
    station_r: Real,
    ground_r: Real,
}

impl Profile {
    fn new(atmosphere: &Atmosphere, constants: &RayTraceConstants) -> Self {
        let h0 = atmosphere.height.meters();
        let first = Layer { base_m: h0, base_k: atmosphere.temperature.kelvin(), base_pa: atmosphere.pressure.pa(), gradient: zero() };
        let mut profile = Self {
            layers: [first; MAX_LAYERS],
            count: 0,
            constants: *constants,
            station_r: constants.earth_radius_m + h0,
            ground_r: constants.earth_radius_m + h0.min(constants.ground_m),
        };
        let mut bottom = first;
        if let Some(inv) = atmosphere.inversion {
            bottom.gradient = inv.gradient_k_per_m;
            bottom = profile.push(bottom, h0 + inv.depth.meters());
        }
        if bottom.base_m < constants.tropopause_m {
            bottom.gradient = atmosphere.lapse_k_per_m;
            bottom = profile.push(bottom, constants.tropopause_m);
        }
        bottom.gradient = zero();
        profile.layers[profile.count] = bottom;
        profile.count += 1;
        // 向下外推的最低点也要查：观测站高于地面时光线可下到地面
        for r in [profile.ground_r, profile.station_r, constants.earth_radius_m + bottom.base_m] {
            assert!(profile.nr_slope(r) > zero(), "大气出现波导（n·r 随高度减小，光线被困）：逆温过强，折射积分不适用");
        }
        profile
    }

    fn push(&mut self, layer: Layer, top_m: Real) -> Layer {
        self.layers[self.count] = layer;
        self.count += 1;
        let (top_k, top_pa) = Self::at(&layer, top_m, &self.constants);
        Layer { base_m: top_m, base_k: top_k, base_pa: top_pa, gradient: zero() }
    }

    fn at(layer: &Layer, h_m: Real, c: &RayTraceConstants) -> (Real, Real) {
        let dz = h_m - layer.base_m;
        if layer.gradient == zero() {
            (layer.base_k, layer.base_pa * (-(c.gravity * dz) / (c.gas_constant * layer.base_k)).exp())
        } else {
            let t = layer.base_k + layer.gradient * dz;
            (t, layer.base_pa * ((t / layer.base_k).ln() * (-c.gravity / (c.gas_constant * layer.gradient))).exp())
        }
    }

    fn layer_of(&self, h_m: Real) -> &Layer {
        // 观测站之下外推最低一层
        let mut i = 0;
        while i + 1 < self.count && self.layers[i + 1].base_m <= h_m {
            i += 1;
        }
        &self.layers[i]
    }

    /// (n·r, d ln n / dr)。
    fn nr_and_dlnn(&self, r: Real) -> (Real, Real) {
        let h = r - self.constants.earth_radius_m;
        let layer = self.layer_of(h);
        let (t, p) = Self::at(layer, h, &self.constants);
        let c = &self.constants;
        let refr = c.refractivity * (p / real_const(101_325.0)) * (real_const(273.15) / t);
        let n = Real::one() + refr;
        (n * r, refr * (-(c.gravity / (c.gas_constant * t)) - layer.gradient / t) / n)
    }

    /// d(n·r)/dr。
    fn nr_slope(&self, r: Real) -> Real {
        let (nr, dlnn) = self.nr_and_dlnn(r);
        nr / r + nr * dlnn
    }

    /// r 从 lo 到 hi、转折点（真实或虚拟）在 rt 的一支 ∫ tan z · (−d ln n)。
    fn branch(&self, lo: Real, hi: Real, rt: Real, invariant: Real) -> Real {
        let mut total = zero();
        let mut a = lo;
        for i in 1..=self.count {
            let b = if i < self.count {
                let edge = self.constants.earth_radius_m + self.layers[i].base_m;
                if edge <= a {
                    continue;
                }
                edge.min(hi)
            } else {
                hi
            };
            if b > a {
                total += self.branch_segment(a, b, rt, invariant);
                a = b;
            }
        }
        total
    }

    fn branch_segment(&self, a: Real, b: Real, rt: Real, invariant: Real) -> Real {
        let (xa, xb) = ((a - rt).max(zero()).sqrt(), (b - rt).sqrt());
        let half = (xb - xa) / real(2.0 * SEGMENTS as f64);
        let mut total = zero();
        for k in 0..SEGMENTS {
            let mid = xa + half * real((2 * k + 1) as f64);
            for (&gx, &gw) in GAUSS_X.iter().zip(GAUSS_W.iter()) {
                for sign in [-1.0, 1.0] {
                    let x = mid + half * real(sign * gx);
                    let r = rt + x * x;
                    let (nr, dlnn) = self.nr_and_dlnn(r);
                    // tan z = C/√((nr)² − C²)，dr = 2x dx
                    let q = (nr - invariant) * (nr + invariant);
                    total += half * real(gw) * real(2.0) * x * invariant / q.sqrt() * (-dlnn);
                }
            }
        }
        total
    }

    /// 视高度 [rad] → 折射 [rad]。光线在视线方向撞地返回 None。
    fn refraction(&self, h_app: Real) -> Option<Real> {
        let r0 = self.station_r;
        let (nr0, _) = self.nr_and_dlnn(r0);
        let invariant = nr0 * h_app.cos();
        let top = self.constants.earth_radius_m + real(TOP_HEIGHT_M);
        if h_app >= zero() {
            // 没有真实转折点；线性外推出虚拟转折点，使 h_app → 0⁺ 时与下行支连续
            let rt = (r0 - (nr0 - invariant) / self.nr_slope(r0)).max(zero());
            return Some(self.branch(r0, top, rt, invariant));
        }
        let (mut lo, mut hi) = (self.ground_r, r0);
        // 掠地光线本身不算撞地：grazing_altitude 算出的视高度代回时只差舍入
        if self.nr_and_dlnn(lo).0 > invariant * real_const(1.0 + 1e-14) {
            return None;
        }
        for _ in 0..60 {
            let mid = (lo + hi) / real_const(2.0);
            if self.nr_and_dlnn(mid).0 < invariant {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some(self.branch(hi, r0, hi, invariant) + self.branch(hi, top, hi, invariant))
    }

    /// 掠过地面的光线在观测站的视高度 [rad]，站在地面时为 0。
    fn grazing_altitude(&self) -> Real {
        let cos_h = self.nr_and_dlnn(self.ground_r).0 / self.nr_and_dlnn(self.station_r).0;
        if self.ground_r >= self.station_r {
            return zero();
        }
        -(Real::one() - cos_h * cos_h).max(zero()).sqrt().atan2(cos_h)
    }
}

/// 视高度 [rad] → 折射 [rad]。视线撞地（观测站之下的地面挡住）返回 None。
pub fn ray_trace_refraction_apparent(h_app: Real, atmosphere: &Atmosphere, constants: &RayTraceConstants) -> Option<Real> {
    Profile::new(atmosphere, constants).refraction(h_app)
}

/// 掠过地面（缺省海平面）的光线：观测站看到的视高度与这条光线的折射 [rad]，即海地平的俯角与地平折射。
pub fn ray_trace_sea_horizon(atmosphere: &Atmosphere, constants: &RayTraceConstants) -> (Real, Real) {
    let profile = Profile::new(atmosphere, constants);
    let h = profile.grazing_altitude();
    (h, profile.refraction(h).expect("掠地光线应可追迹"))
}

/// 真高度 [rad] → 折射 [rad]。低于掠地光线的真高度时，掠地折射在其下 `fade` 内线性收到零，只为保持连续以便求根。
pub(super) fn ray_trace_refraction_geometric(h_geom: Real, fade: Real, atmosphere: &Atmosphere) -> Real {
    let profile = Profile::new(atmosphere, &RayTraceConstants::default());
    let h_graze = profile.grazing_altitude();
    let r_graze = profile.refraction(h_graze).expect("掠地光线应可追迹");
    let geom_graze = h_graze - r_graze;
    if h_geom <= geom_graze {
        return (r_graze * (h_geom - geom_graze + fade) / fade).max(zero());
    }
    // g(h_a) = h_a − R(h_a) − h_geom 在 [max(h_graze, h_geom), h_geom + R_graze] 上变号。Illinois 割线法
    let g = |h_a: Real| h_a - profile.refraction(h_a).expect("掠地光线之上不撞地") - h_geom;
    let (mut a, mut b) = (h_graze.max(h_geom), h_geom + r_graze);
    let (mut ga, mut gb) = (g(a), g(b));
    let tolerance = real_const(1e-9);
    for _ in 0..40 {
        if ga.abs() < tolerance {
            return a - h_geom;
        }
        if gb.abs() < tolerance || b - a < tolerance {
            return b - h_geom;
        }
        let c = b - gb * (b - a) / (gb - ga);
        let gc = g(c);
        if (gc < zero()) == (gb < zero()) {
            ga = ga / real_const(2.0);
        } else {
            (a, ga) = (b, gb);
        }
        (b, gb) = (c, gc);
        if a > b {
            ((a, ga), (b, gb)) = ((b, gb), (a, ga));
        }
    }
    b - h_geom
}
