//! ELP/MPP02 残差改正：在已拟合常数的 ELP/MPP02 之上，对 J2000 平黄道下的月亮黄经、黄纬、距离各加一组
//! 长期多项式 + 混合项（同 VSOP87 的 Chapront (2000) 改正形式），吃掉常数拟合后余下的周期差。
//!
//! 每项：Δ = τᵖ·(C·cos φ + S·sin φ)，τ = T/10（儒略千年，T 为 J2000 起儒略世纪），φ(T) 为 Delaunay 幅角组合
//! i_D·D + i_F·F + i_l·l + i_l′·l′（相位多项式取所配常数集的 Delaunay 多项式，与主问题级数同）；整数系数全为 0 时 φ = 0，
//! 即长期多项式项。黄经、黄纬单位角秒，距离单位 km。系数由 `examples/elpmpp02_residual_fit.rs` 拟合，
//! 存 `data/fit/elpmpp02-<目标>-patch.txt`，与该目标的常数集一一对应（编译时嵌入）。
//! 方法、选频与检验见 doc/astronomy/ephemerides-and-de-align.md §5.4.7。

use crate::math::real::{real, Real, RealOps};
use crate::platform::LoadError;

use super::parse_constants::ParseConstants;

/// 改正作用的变量：J2000 平黄道黄经、黄纬（″）、地心距（km）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatchVar {
    Lon,
    Lat,
    Dist,
}

/// 一项改正（文本一行）。
#[derive(Clone, Debug, PartialEq)]
pub struct PatchTermRaw {
    pub var: PatchVar,
    /// τ 的幂次
    pub power: u8,
    /// Delaunay 幅角 D、F、l、l′ 的整数系数
    pub ilu: [i8; 4],
    pub c: f64,
    pub s: f64,
}

/// 改正的原始系数（与常数集无关的文本内容）。
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Elpmpp02PatchRaw {
    pub terms: Vec<PatchTermRaw>,
    /// 拟合区间（天文纪年）。区间外 τ 固定在端点：多项式与振幅因子不外推（保持端点值），周期项相位照常。
    pub years: Option<(f64, f64)>,
}

impl Elpmpp02PatchRaw {
    /// 解析：每行 `变量 幂次 iD iF il il′ C S`（变量为 L、B、R），或 `Y 起年 止年`（拟合区间），`#` 起为注释。
    pub fn parse(text: &str) -> Result<Self, LoadError> {
        let mut terms = Vec::new();
        let mut years = None;
        for (ln, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split_whitespace().collect();
            let bad = || LoadError::Io(format!("ELPMPP02 残差改正第 {} 行格式错：{line}", ln + 1));
            if f[0] == "Y" && f.len() == 3 {
                years = Some((f[1].parse().map_err(|_| bad())?, f[2].parse().map_err(|_| bad())?));
                continue;
            }
            if f.len() != 8 {
                return Err(bad());
            }
            let var = match f[0] {
                "L" => PatchVar::Lon,
                "B" => PatchVar::Lat,
                "R" => PatchVar::Dist,
                _ => return Err(bad()),
            };
            let int = |s: &str| s.parse::<i8>().map_err(|_| bad());
            terms.push(PatchTermRaw {
                var,
                power: f[1].parse().map_err(|_| bad())?,
                ilu: [int(f[2])?, int(f[3])?, int(f[4])?, int(f[5])?],
                c: f[6].parse().map_err(|_| bad())?,
                s: f[7].parse().map_err(|_| bad())?,
            });
        }
        Ok(Self { terms, years })
    }

    /// 写成 [Self::parse] 可读的文本（不含注释头）。
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        if let Some((a, b)) = self.years {
            s.push_str(&format!("Y {a} {b}\n"));
        }
        for t in &self.terms {
            let v = match t.var {
                PatchVar::Lon => "L",
                PatchVar::Lat => "B",
                PatchVar::Dist => "R",
            };
            s.push_str(&format!(
                "{v} {} {:3} {:3} {:3} {:3} {:+.10e} {:+.10e}\n",
                t.power, t.ilu[0], t.ilu[1], t.ilu[2], t.ilu[3], t.c, t.s
            ));
        }
        s
    }

    /// 按常数集生成可求值的改正（相位多项式取该常数集的 Delaunay 多项式）。
    pub fn build(&self, c: &ParseConstants) -> Elpmpp02Patch {
        // 年 → τ（儒略千年，J2000 起）
        let tau = |y: f64| (y - 2000.0) / 1000.0;
        Elpmpp02Patch {
            tau_range: self.years.map(|(a, b)| (tau(a), tau(b))),
            terms: self
                .terms
                .iter()
                .map(|t| PatchTerm {
                    var: t.var,
                    power: t.power as i32,
                    phase: (0..5).map(|k| (0..4).map(|i| t.ilu[i] as f64 * c.delaunay[i][k]).sum::<f64>()).collect::<Vec<_>>(),
                    periodic: t.ilu != [0; 4],
                    c: t.c,
                    s: t.s,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
struct PatchTerm {
    var: PatchVar,
    power: i32,
    /// 相位多项式 φ(T) 的系数（弧度，T 的 0—4 次）
    phase: Vec<f64>,
    periodic: bool,
    c: f64,
    s: f64,
}

/// 可求值的残差改正。
#[derive(Clone, Debug, Default)]
pub struct Elpmpp02Patch {
    terms: Vec<PatchTerm>,
    tau_range: Option<(f64, f64)>,
}

/// 改正值与其对 T（儒略世纪）的导数：(Δλ″, Δβ″, Δr km) 与 (Δλ″/cy, Δβ″/cy, Δr km/cy)。
pub type PatchValue = ([Real; 3], [Real; 3]);

impl Elpmpp02Patch {
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// 在 T（儒略世纪）处求改正及其导数。
    pub fn eval(&self, t_cy: Real) -> PatchValue {
        let tau_raw = t_cy / real(10.0);
        // 区间外：τ 固定在端点，振幅因子的导数为 0
        let (tau, inside) = match self.tau_range {
            Some((a, _)) if tau_raw < real(a) => (real(a), false),
            Some((_, b)) if tau_raw > real(b) => (real(b), false),
            _ => (tau_raw, true),
        };
        let mut v = [real(0.0), real(0.0), real(0.0)];
        let mut d = [real(0.0), real(0.0), real(0.0)];
        for t in &self.terms {
            let k = match t.var {
                PatchVar::Lon => 0,
                PatchVar::Lat => 1,
                PatchVar::Dist => 2,
            };
            let tp = if t.power == 0 { real(1.0) } else { tau.powi(t.power) };
            // dτᵖ/dT = p·τᵖ⁻¹ / 10
            let dtp = if t.power == 0 || !inside { real(0.0) } else { real(t.power as f64) * tau.powi(t.power - 1) / real(10.0) };
            if t.periodic {
                let mut phi = real(0.0);
                let mut dphi = real(0.0);
                let mut tk = real(1.0);
                for (j, a) in t.phase.iter().enumerate() {
                    phi = phi + real(*a) * tk;
                    if j + 1 < t.phase.len() {
                        dphi = dphi + real((j + 1) as f64 * t.phase[j + 1]) * tk;
                    }
                    tk = tk * t_cy;
                }
                let (sp, cp) = phi.sin_cos();
                let w = real(t.c) * cp + real(t.s) * sp;
                let dw = (real(t.s) * cp - real(t.c) * sp) * dphi;
                v[k] = v[k] + tp * w;
                d[k] = d[k] + dtp * w + tp * dw;
            } else {
                v[k] = v[k] + tp * real(t.c);
                d[k] = d[k] + dtp * real(t.c);
            }
        }
        (v, d)
    }
}

/// 把改正施加到 J2000 平黄道直角坐标的位置（m）与速度（m/s）上：转球坐标，加 Δ 与 Δ′，再转回。
pub(super) fn apply_to_cartesian(p: [Real; 3], v: [Real; 3], patch: &PatchValue) -> ([Real; 3], [Real; 3]) {
    const ARCSEC: f64 = std::f64::consts::PI / 648000.0;
    const CY_S: f64 = 36525.0 * 86400.0;
    let (dv, dd) = patch;
    let rho2 = p[0] * p[0] + p[1] * p[1];
    let rho = rho2.sqrt();
    let r = (rho2 + p[2] * p[2]).sqrt();
    let lon = p[1].atan2(p[0]);
    let lat = p[2].atan2(rho);
    let r_dot = (p[0] * v[0] + p[1] * v[1] + p[2] * v[2]) / r;
    let lon_dot = (p[0] * v[1] - p[1] * v[0]) / rho2;
    let lat_dot = (v[2] * rho - p[2] * (p[0] * v[0] + p[1] * v[1]) / rho) / (r * r);
    let lon = lon + dv[0] * real(ARCSEC);
    let lat = lat + dv[1] * real(ARCSEC);
    let r = r + dv[2] * real(1000.0);
    let lon_dot = lon_dot + dd[0] * real(ARCSEC / CY_S);
    let lat_dot = lat_dot + dd[1] * real(ARCSEC / CY_S);
    let r_dot = r_dot + dd[2] * real(1000.0 / CY_S);
    let (sl, cl) = lon.sin_cos();
    let (sb, cb) = lat.sin_cos();
    let pos = [r * cb * cl, r * cb * sl, r * sb];
    let vel = [
        r_dot * cb * cl - r * sb * cl * lat_dot - r * cb * sl * lon_dot,
        r_dot * cb * sl - r * sb * sl * lat_dot + r * cb * cl * lon_dot,
        r_dot * sb + r * cb * lat_dot,
    ];
    (pos, vel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trip_and_eval() {
        let raw = Elpmpp02PatchRaw::parse("# 注释\nY -2999 2999\nL 0 0 0 0 0 +1.5e-1 +0\nB 1 0 1 0 0 2.0e-2 -1.0e-2\nR 2 2 0 -1 0 0.3 0.1\n").unwrap();
        assert_eq!(raw.years, Some((-2999.0, 2999.0)));
        assert_eq!(Elpmpp02PatchRaw::parse(&raw.to_text()).unwrap(), raw);
        let c = ParseConstants::from_fit(&super::super::Elpmpp02Fit::de405());
        let p = raw.build(&c);
        let (v, d) = p.eval(real(0.0));
        assert!((v[0].as_f64() - 0.15).abs() < 1e-12 && d[0].as_f64() == 0.0);
        // 数值导数核对
        let h = 1e-7;
        let (a, _) = p.eval(real(-3.0 - h));
        let (b, _) = p.eval(real(-3.0 + h));
        let (_, dm) = p.eval(real(-3.0));
        for k in 0..3 {
            let num = (b[k].as_f64() - a[k].as_f64()) / (2.0 * h);
            assert!((num - dm[k].as_f64()).abs() < 1e-5 * num.abs().max(1.0), "{k}: {num} vs {}", dm[k].as_f64());
        }
    }

    /// 区间外多项式保持端点值，不外推
    #[test]
    fn clamped_outside_range() {
        let raw = Elpmpp02PatchRaw::parse("Y -2999 2999\nL 10 0 0 0 0 1.0 0\n").unwrap();
        let p = raw.build(&ParseConstants::from_fit(&super::super::Elpmpp02Fit::de405()));
        let edge = p.eval(real(-49.99)).0[0].as_f64();
        let far = p.eval(real(-80.0)).0[0].as_f64();
        assert!((edge - far).abs() < 1e-3, "{edge} vs {far}");
        assert_eq!(p.eval(real(-80.0)).1[0].as_f64(), 0.0);
    }

    #[test]
    fn apply_round_trip_velocity() {
        // 改正为 0 时位置、速度不变
        let p = [real(-3.0e8), real(2.5e8), real(1.0e7)];
        let v = [real(-600.0), real(-750.0), real(40.0)];
        let z = ([real(0.0), real(0.0), real(0.0)], [real(0.0), real(0.0), real(0.0)]);
        let (q, w) = apply_to_cartesian(p, v, &z);
        for k in 0..3 {
            assert!((q[k] - p[k]).as_f64().abs() < 1e-6);
            assert!((w[k] - v[k]).as_f64().abs() < 1e-9);
        }
    }
}
