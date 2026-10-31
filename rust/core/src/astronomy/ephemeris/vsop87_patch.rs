//! VSOP87B → JPL 数值历表日心 L、B、R 改正（Chapront 2000 方法，前 3000 年—3000 年）。
//!
//! 目标历表可选 DE406 或 DE441（[`PatchTarget`]），两套系数并存：由 `scripts/ephemeris/vsop87_de_planet_fit.py <de406|de441>`
//! 拟合，存于 `data/fit/vsop87-de406-planets.txt`、`data/fit/vsop87-de441-planets.txt`。
//! 每段 `[体 变量]`：首行长期项 a0..a_NSEC；其后每行 `f C0 S0 C1 S1 …`（k = 0..MIXT 的 t^k 混合项）。
//! Δ = Σ a_k t^k + Σ_f Σ_k t^k (C_k cos f t + S_k sin f t)，t 为儒略千年（TDB）；L、B 角秒，R AU。
//! NSEC、MIXT 由各段行长自动确定，各行星可不同。改正挂在 [`Vsop87::patch`](super::Vsop87) 上，位置与速度（解析导数）都施加。
//! wasm 用二进制（[`Vsop87DePatch::to_binary`]，`examples/patch_to_bin.rs` 生成，通常只含地球 `ear`）。

use crate::math::real::{real, Real, RealOps};
use crate::platform::LoadError;

#[derive(Clone, Debug, Default)]
struct Series {
    secular: Vec<Real>,
    /// (频率 rad/千年, [C0, S0, C1, S1, …])
    periodic: Vec<(Real, Vec<Real>)>,
}

impl Series {
    fn eval(&self, t: Real) -> Real {
        let mut sum = real(0.0);
        let mut tk = real(1.0);
        for a in &self.secular {
            sum = sum + *a * tk;
            tk = tk * t;
        }
        for (f, cs) in &self.periodic {
            let arg = *f * t;
            let (s, c) = arg.sin_cos();
            let mut tk = real(1.0);
            for pair in cs.chunks(2) {
                sum = sum + tk * (pair[0] * c + pair[1] * s);
                tk = tk * t;
            }
        }
        sum
    }

    /// d/dt（t 儒略千年）。
    fn eval_derivative(&self, t: Real) -> Real {
        let mut sum = real(0.0);
        let mut tk = real(1.0); // t^(k−1)
        for (k, a) in self.secular.iter().enumerate().skip(1) {
            sum = sum + real(k as f64) * *a * tk;
            tk = tk * t;
        }
        for (f, cs) in &self.periodic {
            let arg = *f * t;
            let (s, c) = arg.sin_cos();
            let mut tk = real(1.0); // t^k
            let mut tkm1 = real(0.0); // k·t^(k−1)
            for (k, pair) in cs.chunks(2).enumerate() {
                let (ck, sk) = (pair[0], pair[1]);
                // d/dt [t^k (C cos ft + S sin ft)] = k t^(k−1)(C cos + S sin) + t^k f (−C sin + S cos)
                sum = sum + tkm1 * (ck * c + sk * s) + tk * *f * (sk * c - ck * s);
                tkm1 = real((k + 1) as f64) * tk;
                tk = tk * t;
            }
        }
        sum
    }
}

/// 一个天体的 L、B、R 改正。
#[derive(Clone, Debug, Default)]
pub struct LbrPatch {
    lbr: [Series; 3],
}

impl LbrPatch {
    /// t 为儒略千年（TDB）；返回 (ΔL 角秒, ΔB 角秒, ΔR AU)。
    pub fn correction(&self, t: Real) -> (Real, Real, Real) {
        (self.lbr[0].eval(t), self.lbr[1].eval(t), self.lbr[2].eval(t))
    }

    /// 改正对 t（儒略千年）的导数：(″/千年, ″/千年, AU/千年)。
    pub fn derivative(&self, t: Real) -> (Real, Real, Real) {
        (self.lbr[0].eval_derivative(t), self.lbr[1].eval_derivative(t), self.lbr[2].eval_derivative(t))
    }
}

/// 改正所拟合的目标历表。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatchTarget {
    /// JPL DE406（1998，前 3000—3000 年）
    De406,
    /// JPL DE441（Park et al. 2021，前 13200—17191 年）
    De441,
}

impl PatchTarget {
    pub fn repo_path(self) -> &'static str {
        match self {
            PatchTarget::De406 => crate::repo::paths::FIT_VSOP87_DE406_PLANETS,
            PatchTarget::De441 => crate::repo::paths::FIT_VSOP87_DE441_PLANETS,
        }
    }
}

/// 全部天体的改正表（键为 VSOP87 文件后缀：`mer`、`ven`、`ear`、`mar`、`jup`、`sat`、`ura`、`nep`）。
#[derive(Clone, Debug, Default)]
pub struct Vsop87DePatch {
    bodies: Vec<(String, LbrPatch)>,
}

impl Vsop87DePatch {
    pub fn parse(text: &str) -> Result<Self, LoadError> {
        let mut out = Self::default();
        let mut current: Option<(String, usize)> = None;
        let mut first_line = false;
        for (no, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(head) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                let mut it = head.split_whitespace();
                let (body, var) = (it.next().unwrap_or(""), it.next().unwrap_or(""));
                let vi = match var {
                    "L" => 0,
                    "B" => 1,
                    "R" => 2,
                    _ => return Err(LoadError::Parse(format!("第 {} 行：未知变量 {var}", no + 1))),
                };
                if !out.bodies.iter().any(|(b, _)| b == body) {
                    out.bodies.push((body.to_string(), LbrPatch::default()));
                }
                current = Some((body.to_string(), vi));
                first_line = true;
                continue;
            }
            let (body, vi) = current.clone().ok_or_else(|| LoadError::Parse(format!("第 {} 行：段头之前有数据", no + 1)))?;
            let nums: Vec<Real> = line
                .split_whitespace()
                .map(|x| x.parse::<f64>().map(real).map_err(|e| LoadError::Parse(format!("第 {} 行：{e}", no + 1))))
                .collect::<Result<_, _>>()?;
            let series = &mut out.bodies.iter_mut().find(|(b, _)| *b == body).unwrap().1.lbr[vi];
            if first_line {
                series.secular = nums;
                first_line = false;
            } else {
                if nums.len() < 3 || (nums.len() - 1) % 2 != 0 {
                    return Err(LoadError::Parse(format!("第 {} 行：周期项列数 {} 不合", no + 1, nums.len())));
                }
                series.periodic.push((nums[0], nums[1..].to_vec()));
            }
        }
        Ok(out)
    }

    /// 从 repo 读指定目标历表的改正系数。
    pub fn from_repo(target: PatchTarget) -> Result<Self, LoadError> {
        let lines = crate::repo::read_lines(target.repo_path())?;
        Self::parse(&lines.join("\n"))
    }

    pub fn body(&self, suffix: &str) -> Option<&LbrPatch> {
        self.bodies.iter().find(|(b, _)| b == suffix).map(|(_, p)| p)
    }

    /// 只保留指定天体（如 wasm 只需地球 `ear`）。
    pub fn retain(&mut self, suffixes: &[&str]) {
        self.bodies.retain(|(b, _)| suffixes.contains(&b.as_str()));
    }

    /// 二进制：`VDP1`、u32 天体数；每体 4 字节名（后缀补 0）、3 个变量各为 u32 长期项数 + f64…、u32 周期项数、u32 每项系数数
    /// （不含频率）+ 每项 f64 频率与系数。全部小端。
    pub fn to_binary(&self) -> Vec<u8> {
        let mut out = b"VDP1".to_vec();
        out.extend_from_slice(&(self.bodies.len() as u32).to_le_bytes());
        for (name, p) in &self.bodies {
            let mut nm = [0u8; 4];
            nm[..name.len().min(4)].copy_from_slice(&name.as_bytes()[..name.len().min(4)]);
            out.extend_from_slice(&nm);
            for s in &p.lbr {
                out.extend_from_slice(&(s.secular.len() as u32).to_le_bytes());
                for a in &s.secular {
                    out.extend_from_slice(&a.as_f64().to_le_bytes());
                }
                let width = s.periodic.first().map(|(_, cs)| cs.len()).unwrap_or(0);
                out.extend_from_slice(&(s.periodic.len() as u32).to_le_bytes());
                out.extend_from_slice(&(width as u32).to_le_bytes());
                for (f, cs) in &s.periodic {
                    out.extend_from_slice(&f.as_f64().to_le_bytes());
                    for c in cs {
                        out.extend_from_slice(&c.as_f64().to_le_bytes());
                    }
                }
            }
        }
        out
    }

    pub fn from_binary(bytes: &[u8]) -> Result<Self, LoadError> {
        let mut pos = 0usize;
        let mut take = |n: usize| -> Result<&[u8], LoadError> {
            let s = bytes.get(pos..pos + n).ok_or_else(|| LoadError::Parse("VDP1 二进制截断".into()))?;
            pos += n;
            Ok(s)
        };
        if take(4)? != b"VDP1" {
            return Err(LoadError::Parse("VDP1 二进制魔数不符".into()));
        }
        let u32_at = |s: &[u8]| u32::from_le_bytes(s.try_into().unwrap()) as usize;
        let f64_at = |s: &[u8]| real(f64::from_le_bytes(s.try_into().unwrap()));
        let n_bodies = u32_at(take(4)?);
        let mut out = Self::default();
        for _ in 0..n_bodies {
            let nm = take(4)?;
            let name = String::from_utf8_lossy(nm).trim_end_matches('\0').to_string();
            let mut patch = LbrPatch::default();
            for s in patch.lbr.iter_mut() {
                let ns = u32_at(take(4)?);
                for _ in 0..ns {
                    s.secular.push(f64_at(take(8)?));
                }
                let np = u32_at(take(4)?);
                let width = u32_at(take(4)?);
                for _ in 0..np {
                    let f = f64_at(take(8)?);
                    let cs = (0..width).map(|_| take(8).map(f64_at)).collect::<Result<Vec<_>, _>>()?;
                    s.periodic.push((f, cs));
                }
            }
            out.bodies.push((name, patch));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 解析导数与数值差分一致；二进制往返不变。
    #[test]
    fn derivative_and_binary_roundtrip() {
        let Ok(p) = Vsop87DePatch::from_repo(PatchTarget::De406) else {
            println!("skipped: 无改正系数文件");
            return;
        };
        for name in ["ear", "jup", "sat"] {
            let b = p.body(name).unwrap();
            let tol = if cfg!(feature = "real-f64") { 2e-4 } else { 1e-5 };
            let mut wr = 0.0_f64;
            for t in [-0.48, -0.2, 0.0, 0.13, 0.29] {
                // 中心差分截断误差 ~ (f·h)²/6；改正项频率达 ~8×10⁴ rad/千年，h 须足够小（TwoFloat 精度下 1e-9 无舍入问题）
                // f64 构建 1e-9 的舍入过大：改用五点差分（截断 ~ (f·h)⁴/30）配较大步长
                let f64_build = cfg!(feature = "real-f64");
                let h = if f64_build { 1e-6 } else { 1e-9 };
                let c = |x: f64| b.correction(real(x));
                let (a0, b0, c0) = c(t - h);
                let (a1, b1, c1) = c(t + h);
                let (m2, p2) = (c(t - 2.0 * h), c(t + 2.0 * h));
                let diff = |lo2: Real, lo: Real, hi: Real, hi2: Real| {
                    if f64_build {
                        (lo2 - hi2 + real(8.0) * (hi - lo)) / real(12.0 * h)
                    } else {
                        (hi - lo) / real(2.0 * h)
                    }
                };
                let (da, db, dc) = b.derivative(real(t));
                for (num, ana) in [(diff(m2.0, a0, a1, p2.0), da), (diff(m2.1, b0, b1, p2.1), db), (diff(m2.2, c0, c1, p2.2), dc)] {
                    let (num, ana) = (num.as_f64(), ana.as_f64());
                    wr = wr.max((num - ana).abs() / (1.0 + ana.abs()));
                    assert!((num - ana).abs() <= tol * (1.0 + ana.abs()), "{name} t={t}: 数值 {num} 解析 {ana}");
                }
            }
            println!("{name}：差分对解析导数最大相对差 {wr:.3e}");
        }
        let q = Vsop87DePatch::from_binary(&p.to_binary()).unwrap();
        for name in ["mer", "ear", "nep"] {
            for t in [-0.5, 0.0, 0.3] {
                assert_eq!(p.body(name).unwrap().correction(real(t)), q.body(name).unwrap().correction(real(t)));
            }
        }
    }
}
