//! VSOP87B → DE406 黄道改正（Chapront 2000 混合项形式）：在 **J2000 平黄道 (L,B,R)** 上施用。
//! 模型：Δσ(T) = (a0+a1*T+a2*T²+a3*T³) + Σ [(C0_i+C1_i*T)cos(f_i*T) + (S0_i+S1_i*T)sin(f_i*T)]。
//! T = (JD − J2000)/365250 儒略千年。ΔL、ΔB 角秒，ΔR AU。系数从外部文件加载，见 [`try_init_de406_ecliptic_patch`]；未加载时无改正（Δ=0）。
//! 文档 §5.3：黄道补丁用于验证或备用；pipeline 当前采用赤道补丁。

use crate::astronomy::constant::AU_METERS;
use crate::astronomy::time::{jd_to_t, TimePoint, TimeScale};
use crate::math::real::{real, RealOps};
use crate::math::series::power_series_at;
use crate::platform::DataLoader;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::length::Length;
use once_cell::sync::Lazy;
use std::sync::RwLock;

fn eval_mixed(
    t: f64,
    secular: &[f64; 4],
    terms: &[(f64, f64, f64, f64, f64)],
) -> f64 {
    let poly = power_series_at(secular, t).as_f64();
    let sum: f64 = terms
        .iter()
        .map(|&(freq, c0, s0, c1, s1)| {
            let angle = freq * t;
            let amp_c = c0 + c1 * t;
            let amp_s = s0 + s1 * t;
            amp_c * angle.cos() + amp_s * angle.sin()
        })
        .sum();
    poly + sum
}

/// 黄道 patch 数据（L/B/R 各 4 个长期项 + 若干周期项）。
pub struct EclipticPatchData {
    pub l_secular: [f64; 4],
    pub l_terms: Vec<(f64, f64, f64, f64, f64)>,
    pub b_secular: [f64; 4],
    pub b_terms: Vec<(f64, f64, f64, f64, f64)>,
    pub r_secular: [f64; 4],
    pub r_terms: Vec<(f64, f64, f64, f64, f64)>,
}

/// 默认黄道 patch 文件路径（相对项目根或资源根）。
pub const DEFAULT_ECLIPTIC_PATCH_PATH: &str = crate::repo::paths::FIT_VSOP87_DE406_ECLIPTIC;

static ECLIPTIC_PATCH_CACHE: Lazy<RwLock<Option<EclipticPatchData>>> = Lazy::new(|| RwLock::new(None));

/// 从 DataLoader 加载黄道 patch 并设为当前系数（注入用）。成功返回 true；失败保留原状。
pub fn try_init_de406_ecliptic_patch(loader: &dyn DataLoader, path: &str) -> bool {
    let lines = match loader.read_lines(path) {
        Ok(l) => l,
        Err(_) => return false,
    };
    try_init_de406_ecliptic_patch_with_lines(&lines)
}

/// 从「repo」读取并加载黄道 patch（Native=本地文件，Wasm=宿主 set_loader 注入）。
pub fn try_init_de406_ecliptic_patch_from_repo() -> bool {
    let lines = match crate::repo::read_lines(crate::repo::paths::FIT_VSOP87_DE406_ECLIPTIC) {
        Ok(l) => l,
        Err(_) => return false,
    };
    try_init_de406_ecliptic_patch_with_lines(&lines)
}

/// 用已读入的文本行设置黄道 patch 系数（仅 parser 结果写缓存）。
fn try_init_de406_ecliptic_patch_with_lines(lines: &[String]) -> bool {
    match parse_ecliptic_patch_lines(lines) {
        Some(data) => {
            *ECLIPTIC_PATCH_CACHE.write().unwrap() = Some(data);
            true
        }
        None => false,
    }
}

/// 解析黄道 patch 文本：段 [L]/[B]/[R]，每段一行 4 数（长期）再多行 (freq c0 s0 c1 s1)。
fn parse_ecliptic_patch_lines(lines: &[String]) -> Option<EclipticPatchData> {
    let mut l_secular = [0.0_f64; 4];
    let mut l_terms = Vec::new();
    let mut b_secular = [0.0_f64; 4];
    let mut b_terms = Vec::new();
    let mut r_secular = [0.0_f64; 4];
    let mut r_terms = Vec::new();
    let mut section: Option<&str> = None;
    let mut secular_done = false;
    let mut got_l = false;
    let mut got_b = false;
    let mut got_r = false;
    for line in lines {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        if s == "[L]" {
            section = Some("L");
            secular_done = false;
            continue;
        }
        if s == "[B]" {
            section = Some("B");
            secular_done = false;
            continue;
        }
        if s == "[R]" {
            section = Some("R");
            secular_done = false;
            continue;
        }
        let parts: Vec<f64> = s
            .split_whitespace()
            .filter_map(|p| p.replace('D', "E").replace('d', "e").parse().ok())
            .collect();
        if parts.len() == 4 && !secular_done {
            match section {
                Some("L") => {
                    l_secular.copy_from_slice(&parts[..4]);
                    secular_done = true;
                    got_l = true;
                }
                Some("B") => {
                    b_secular.copy_from_slice(&parts[..4]);
                    secular_done = true;
                    got_b = true;
                }
                Some("R") => {
                    r_secular.copy_from_slice(&parts[..4]);
                    secular_done = true;
                    got_r = true;
                }
                _ => {}
            }
        } else if parts.len() == 5 {
            let term = (parts[0], parts[1], parts[2], parts[3], parts[4]);
            match section {
                Some("L") => l_terms.push(term),
                Some("B") => b_terms.push(term),
                Some("R") => r_terms.push(term),
                _ => {}
            }
        }
    }
    if got_l && got_b && got_r {
        Some(EclipticPatchData {
            l_secular,
            l_terms,
            b_secular,
            b_terms,
            r_secular,
            r_terms,
        })
    } else {
        None
    }
}

const ARCSEC_TO_RAD: f64 = std::f64::consts::PI / 648000.0;

static ZERO_SECULAR: [f64; 4] = [0.0; 4];
const EMPTY_TERMS: &[(f64, f64, f64, f64, f64)] = &[];

/// ΔL、ΔB（弧度）、ΔR（AU）。公式规定 **TT** 儒略日。未加载时返回 (0, 0, 0)。
pub fn correction_l_b_r(t: &TimePoint) -> (PlaneAngle, PlaneAngle, Length) {
    let jd = t.to_scale(TimeScale::TT).jd;
    let tt = jd_to_t(jd).0.as_f64();
    let cache = ECLIPTIC_PATCH_CACHE.read().unwrap();
    let (l_s, l_t, b_s, b_t, r_s, r_t) = if let Some(ref p) = *cache {
        (
            &p.l_secular,
            p.l_terms.as_slice(),
            &p.b_secular,
            p.b_terms.as_slice(),
            &p.r_secular,
            p.r_terms.as_slice(),
        )
    } else {
        (
            &ZERO_SECULAR,
            EMPTY_TERMS,
            &ZERO_SECULAR,
            EMPTY_TERMS,
            &ZERO_SECULAR,
            EMPTY_TERMS,
        )
    };
    let d_l_arcsec = eval_mixed(tt, l_s, l_t);
    let d_b_arcsec = eval_mixed(tt, b_s, b_t);
    let d_r_au = eval_mixed(tt, r_s, r_t);
    (
        PlaneAngle::from_rad(real(d_l_arcsec * ARCSEC_TO_RAD)),
        PlaneAngle::from_rad(real(d_b_arcsec * ARCSEC_TO_RAD)),
        Length::from_value(real(d_r_au) * AU_METERS, crate::quantity::unit::LengthUnit::Meter),
    )
}

/// 在 J2000 平黄道球面 (L,B,R) 上施加地心太阳改正：(L+ΔL, B+ΔB, R+ΔR)。
/// L、B 为弧度，R 为 AU；返回 (L', B', R') 弧度与 AU。
pub fn apply_patch_to_ecliptic_for_geocentric_sun(
    l: PlaneAngle,
    b: PlaneAngle,
    r_au: f64,
    t: &TimePoint,
) -> (PlaneAngle, PlaneAngle, f64) {
    let (d_l, d_b, d_r_len) = correction_l_b_r(t);
    let l_out = l.rad() + d_l.rad();
    let b_out = b.rad() + d_b.rad();
    let r_out_au = r_au + (d_r_len.meters() / real(AU_METERS)).as_f64();
    (
        PlaneAngle::from_rad(l_out),
        PlaneAngle::from_rad(b_out),
        r_out_au,
    )
}
