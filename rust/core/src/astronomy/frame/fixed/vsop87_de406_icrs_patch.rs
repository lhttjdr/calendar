//! VSOP87B → DE406 轨道改正（Chapront 2000 混合项形式）：在 VSOP→FK5→ICRS 之后施用。
//! 模型：Δσ(T) = (a0+a1*T+a2*T²+a3*T³) + Σ [(C0_i+C1_i*T)cos(f_i*T) + (S0_i+S1_i*T)sin(f_i*T)]。
//! T = (JD − J2000)/365250 儒略千年。系数从外部文件加载，见 [`try_init_de406_patch`]；未加载时无改正（Δ=0）。

use crate::astronomy::constant::AU_METERS;
use crate::astronomy::time::{jd_to_t, TimePoint, TimeScale};
use crate::math::real::{real, zero, RealOps};
use crate::math::series::{power_series_at, power_series_derivative_at};
use crate::platform::DataLoader;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{length::Length, position::Position, reference_frame::ReferenceFrame};
use once_cell::sync::Lazy;
use std::sync::RwLock;

/// 混合项求值：secular(a0..a3) + Σ (C0+C1*T)cos(f*T) + (S0+S1*T)sin(f*T)
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

/// 混合项对 T 的导数（T 儒略千年）
fn eval_mixed_derivative(
    t: f64,
    secular: &[f64; 4],
    terms: &[(f64, f64, f64, f64, f64)],
) -> f64 {
    let poly_deriv = power_series_derivative_at(secular, t).as_f64();
    let sum: f64 = terms
        .iter()
        .map(|&(freq, c0, s0, c1, s1)| {
            let f = freq;
            let angle = f * t;
            let amp_c = c0 + c1 * t;
            let amp_s = s0 + s1 * t;
            (c1 + amp_s * f) * angle.cos() + (s1 - amp_c * f) * angle.sin()
        })
        .sum();
    poly_deriv + sum
}

/// 外部 patch 数据（RA/Dec/R 各 4 个长期项 + 若干周期项）。
pub struct PatchData {
    pub ra_secular: [f64; 4],
    pub ra_terms: Vec<(f64, f64, f64, f64, f64)>,
    pub dec_secular: [f64; 4],
    pub dec_terms: Vec<(f64, f64, f64, f64, f64)>,
    pub r_secular: [f64; 4],
    pub r_terms: Vec<(f64, f64, f64, f64, f64)>,
}

/// 默认 patch 文件路径（相对项目根或资源根）。
pub const DEFAULT_PATCH_PATH: &str = crate::repo::paths::FIT_VSOP87_DE406_ICRS;

static PATCH_CACHE: Lazy<RwLock<Option<PatchData>>> = Lazy::new(|| RwLock::new(None));

/// 从 DataLoader 加载 patch 文件并设为当前系数（注入用）。成功返回 true；失败保留原状。
pub fn try_init_de406_patch(loader: &dyn DataLoader, path: &str) -> bool {
    let lines = match loader.read_lines(path) {
        Ok(l) => l,
        Err(_) => return false,
    };
    try_init_de406_patch_with_lines(&lines)
}

/// 从「repo」读取并加载 patch（Native=本地文件，Wasm=宿主 set_loader 注入）。
pub fn try_init_de406_patch_from_repo() -> bool {
    let lines = match crate::repo::read_lines(crate::repo::paths::FIT_VSOP87_DE406_ICRS) {
        Ok(l) => l,
        Err(_) => return false,
    };
    try_init_de406_patch_with_lines(&lines)
}

/// 用已读入的文本行设置 patch 系数（仅 parser 结果写缓存）。
fn try_init_de406_patch_with_lines(lines: &[String]) -> bool {
    match parse_patch_lines(lines) {
        Some(data) => {
            *PATCH_CACHE.write().unwrap() = Some(data);
            true
        }
        None => false,
    }
}

/// 二进制格式：魔数 "PICR"、版本 u32=1，随后 [RA] 4×f64 + u32 n_ra + n_ra×5×f64，[Dec] 同，[R] 同。小端。
const PATCH_BIN_MAGIC: &[u8; 4] = b"PICR";
const PATCH_BIN_VERSION: u32 = 1;

fn read_u32_le(b: &[u8], i: usize) -> Option<u32> {
    if i + 4 > b.len() {
        return None;
    }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&b[i..i + 4]);
    Some(u32::from_le_bytes(arr))
}

fn read_f64_le(b: &[u8], i: usize) -> Option<f64> {
    if i + 8 > b.len() {
        return None;
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&b[i..i + 8]);
    Some(f64::from_le_bytes(arr))
}

fn read_section_bin(bytes: &[u8], pos: &mut usize) -> Option<([f64; 4], Vec<(f64, f64, f64, f64, f64)>)> {
    let mut secular = [0.0_f64; 4];
    for s in &mut secular {
        *s = read_f64_le(bytes, *pos)?;
        *pos += 8;
    }
    let n = read_u32_le(bytes, *pos)? as usize;
    *pos += 4;
    let mut terms = Vec::with_capacity(n);
    for _ in 0..n {
        let f0 = read_f64_le(bytes, *pos)?;
        *pos += 8;
        let f1 = read_f64_le(bytes, *pos)?;
        *pos += 8;
        let f2 = read_f64_le(bytes, *pos)?;
        *pos += 8;
        let f3 = read_f64_le(bytes, *pos)?;
        *pos += 8;
        let f4 = read_f64_le(bytes, *pos)?;
        *pos += 8;
        terms.push((f0, f1, f2, f3, f4));
    }
    Some((secular, terms))
}

/// 从二进制加载 patch 数据（.bin 或解压后的 .br）。格式见本模块常量与 `to_binary`。
pub fn from_binary(bytes: &[u8]) -> Option<PatchData> {
    if bytes.len() < 4 + 4 {
        return None;
    }
    if &bytes[0..4] != PATCH_BIN_MAGIC {
        return None;
    }
    let version = read_u32_le(bytes, 4)?;
    if version != PATCH_BIN_VERSION {
        return None;
    }
    let mut pos = 8;
    let (ra_secular, ra_terms) = read_section_bin(bytes, &mut pos)?;
    let (dec_secular, dec_terms) = read_section_bin(bytes, &mut pos)?;
    let (r_secular, r_terms) = read_section_bin(bytes, &mut pos)?;
    if ra_terms.len() >= 20 && dec_terms.len() >= 20 && r_terms.len() >= 20 {
        Some(PatchData {
            ra_secular,
            ra_terms,
            dec_secular,
            dec_terms,
            r_secular,
            r_terms,
        })
    } else {
        None
    }
}

/// 序列化为二进制（供构建脚本生成 .bin；.br 由前端或构建时压缩）。
pub fn to_binary(data: &PatchData) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(PATCH_BIN_MAGIC);
    out.extend_from_slice(&PATCH_BIN_VERSION.to_le_bytes());
    fn write_section(out: &mut Vec<u8>, secular: &[f64; 4], terms: &[(f64, f64, f64, f64, f64)]) {
        for &s in secular {
            out.extend_from_slice(&s.to_le_bytes());
        }
        out.extend_from_slice(&(terms.len() as u32).to_le_bytes());
        for &(f0, f1, f2, f3, f4) in terms {
            out.extend_from_slice(&f0.to_le_bytes());
            out.extend_from_slice(&f1.to_le_bytes());
            out.extend_from_slice(&f2.to_le_bytes());
            out.extend_from_slice(&f3.to_le_bytes());
            out.extend_from_slice(&f4.to_le_bytes());
        }
    }
    write_section(&mut out, &data.ra_secular, &data.ra_terms);
    write_section(&mut out, &data.dec_secular, &data.dec_terms);
    write_section(&mut out, &data.r_secular, &data.r_terms);
    out
}

/// 从二进制 buffer 加载并设为当前 patch（.bin 或解压后的 .br）。成功返回 true。
pub fn try_init_de406_patch_from_binary(bytes: &[u8]) -> bool {
    if let Some(data) = from_binary(bytes) {
        *PATCH_CACHE.write().unwrap() = Some(data);
        true
    } else {
        false
    }
}

/// 当前是否已加载 patch（文本或二进制初始化后为 true）。
pub fn is_de406_patch_loaded() -> bool {
    PATCH_CACHE.read().unwrap().is_some()
}

/// 解析 patch 文本：段 [RA]/[Dec]/[R]，每段一行 4 数（长期）再多行 (freq c0 s0 c1 s1)。供 example patch_to_bin 使用。
pub fn parse_patch_lines(lines: &[String]) -> Option<PatchData> {
    let mut ra_secular = [0.0_f64; 4];
    let mut ra_terms = Vec::new();
    let mut dec_secular = [0.0_f64; 4];
    let mut dec_terms = Vec::new();
    let mut r_secular = [0.0_f64; 4];
    let mut r_terms = Vec::new();
    let mut section: Option<&str> = None;
    let mut secular_done = false;
    for line in lines {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        if s == "[RA]" {
            section = Some("RA");
            secular_done = false;
            continue;
        }
        if s == "[Dec]" {
            section = Some("Dec");
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
                Some("RA") => {
                    ra_secular.copy_from_slice(&parts[..4]);
                    secular_done = true;
                }
                Some("Dec") => {
                    dec_secular.copy_from_slice(&parts[..4]);
                    secular_done = true;
                }
                Some("R") => {
                    r_secular.copy_from_slice(&parts[..4]);
                    secular_done = true;
                }
                _ => {}
            }
        } else if parts.len() == 5 {
            let term = (parts[0], parts[1], parts[2], parts[3], parts[4]);
            match section {
                Some("RA") => ra_terms.push(term),
                Some("Dec") => dec_terms.push(term),
                Some("R") => r_terms.push(term),
                _ => {}
            }
        }
    }
    if ra_terms.len() >= 20 && dec_terms.len() >= 20 && r_terms.len() >= 20 {
        Some(PatchData {
            ra_secular,
            ra_terms,
            dec_secular,
            dec_terms,
            r_secular,
            r_terms,
        })
    } else {
        None
    }
}

const ARCSEC_TO_RAD: f64 = std::f64::consts::PI / 648000.0; // π/(180*3600)
/// 1/儒略千年（日⁻¹），用于 T 对 JD 的导数
const DT_DJD: f64 = 1.0 / 365250.0;

/// 未加载 patch 时使用：零长期项 + 空周期项，即无改正。
static ZERO_SECULAR: [f64; 4] = [0.0; 4];
const EMPTY_TERMS: &[(f64, f64, f64, f64, f64)] = &[];

/// ΔRA、ΔDec、ΔR 对 JD 的导数：(弧度/日, 弧度/日, AU/日)。T = (JD−J2000)/365250。
pub fn correction_rad_dec_r_derivative_per_day(t: &TimePoint) -> (f64, f64, f64) {
    let jd = t.to_scale(TimeScale::TT).jd;
    let tt = jd_to_t(jd).0.as_f64();
    let cache = PATCH_CACHE.read().unwrap();
    let (ra_s, ra_t, dec_s, dec_t, r_s, r_t) = if let Some(ref p) = *cache {
        (
            &p.ra_secular,
            p.ra_terms.as_slice(),
            &p.dec_secular,
            p.dec_terms.as_slice(),
            &p.r_secular,
            p.r_terms.as_slice(),
        )
    } else {
        (&ZERO_SECULAR, EMPTY_TERMS, &ZERO_SECULAR, EMPTY_TERMS, &ZERO_SECULAR, EMPTY_TERMS)
    };
    let d_ra_arcsec_per_day = eval_mixed_derivative(tt, ra_s, ra_t) * DT_DJD;
    let d_dec_arcsec_per_day = eval_mixed_derivative(tt, dec_s, dec_t) * DT_DJD;
    let d_r_au_per_day = eval_mixed_derivative(tt, r_s, r_t) * DT_DJD;
    (
        d_ra_arcsec_per_day * ARCSEC_TO_RAD,
        d_dec_arcsec_per_day * ARCSEC_TO_RAD,
        d_r_au_per_day,
    )
}

/// ΔRA、ΔDec、ΔR（AU）。公式规定 **TT** 儒略日，入参须为 TT。(PlaneAngle, PlaneAngle, Length)。
pub fn correction_rad_dec_r(t: &TimePoint) -> (PlaneAngle, PlaneAngle, Length) {
    let jd = t.to_scale(TimeScale::TT).jd;
    let tt = jd_to_t(jd).0.as_f64();
    let cache = PATCH_CACHE.read().unwrap();
    let (ra_s, ra_t, dec_s, dec_t, r_s, r_t) = if let Some(ref p) = *cache {
        (
            &p.ra_secular,
            p.ra_terms.as_slice(),
            &p.dec_secular,
            p.dec_terms.as_slice(),
            &p.r_secular,
            p.r_terms.as_slice(),
        )
    } else {
        (&ZERO_SECULAR, EMPTY_TERMS, &ZERO_SECULAR, EMPTY_TERMS, &ZERO_SECULAR, EMPTY_TERMS)
    };
    let d_ra_arcsec = eval_mixed(tt, ra_s, ra_t);
    let d_dec_arcsec = eval_mixed(tt, dec_s, dec_t);
    let d_r_au = eval_mixed(tt, r_s, r_t);
    (
        PlaneAngle::from_rad(real(d_ra_arcsec * ARCSEC_TO_RAD)),
        PlaneAngle::from_rad(real(d_dec_arcsec * ARCSEC_TO_RAD)),
        Length::from_value(real(d_r_au) * AU_METERS, crate::quantity::unit::LengthUnit::Meter),
    )
}

/// 在 ICRS 赤道直角位置上施加地心太阳改正：角 (ra+ΔRA, dec−ΔDec) 再径向 (r+ΔR)/r。
/// 入参、返回均为 ICRS 架下 Position（坐标米）。
pub fn apply_patch_to_equatorial_for_geocentric_sun(pos: Position, t: &TimePoint) -> Position {
    let (d_ra, d_dec, d_r) = correction_rad_dec_r(t);
    let d_r_au = d_r.meters() / AU_METERS;
    let x = pos.x.meters() / AU_METERS;
    let y = pos.y.meters() / AU_METERS;
    let z = pos.z.meters() / AU_METERS;
    let r = (x * x + y * y + z * z).sqrt();
    if r <= zero() {
        return pos;
    }
    let ra = y.atan2(x);
    let dec = (z / r).asin();
    let ra1 = ra + d_ra.rad();
    let dec1 = dec - d_dec.rad();
    let (cd, sd) = (dec1.cos(), dec1.sin());
    let (cr, sr) = (ra1.cos(), ra1.sin());
    let angular = (r * cd * cr, r * cd * sr, r * sd);
    let r1 = r + d_r_au;
    let scale = r1 / r;
    let x_c = angular.0 * scale * AU_METERS;
    let y_c = angular.1 * scale * AU_METERS;
    let z_c = angular.2 * scale * AU_METERS;
    Position::from_si_meters_in_frame(ReferenceFrame::ICRS, x_c, y_c, z_c)
}

/// 对地心太阳 ICRS 位置与速度施加 patch 改正：位置同 `apply_patch_to_equatorial_for_geocentric_sun`，
/// 速度加上因 (ΔRA, ΔDec, ΔR) 随时间变化带来的改正。pos/vel 为米、m/s；返回 (改正后位置, 改正后速度)。
pub fn apply_patch_velocity_to_equatorial_for_geocentric_sun(
    pos: Position,
    vel: [crate::math::real::Real; 3],
    t: &TimePoint,
) -> (Position, [crate::math::real::Real; 3]) {
    let pos_c = apply_patch_to_equatorial_for_geocentric_sun(pos, t);
    let (d_ra_per_day, d_dec_per_day, d_r_au_per_day) = correction_rad_dec_r_derivative_per_day(t);
    let x_c = pos_c.x.meters();
    let y_c = pos_c.y.meters();
    let z_c = pos_c.z.meters();
    let r_c = (x_c * x_c + y_c * y_c + z_c * z_c).sqrt();
    if r_c.as_f64() <= 0.0 {
        return (pos_c, vel);
    }
    let rho = (x_c * x_c + y_c * y_c).sqrt();
    let sec_per_day = 86400.0;

    let zero = real(0);
    let dpos_dra = [-y_c, x_c, zero];
    let dpos_ddec = if rho.as_f64() > 0.0 {
        [x_c * z_c / rho, y_c * z_c / rho, -rho]
    } else {
        [zero, zero, zero]
    };
    let unit = [x_c / r_c, y_c / r_c, z_c / r_c];
    let dpos_dr_au = [unit[0] * AU_METERS, unit[1] * AU_METERS, unit[2] * AU_METERS];

    let corr_m_per_day_0 = dpos_dra[0] * real(d_ra_per_day) + dpos_ddec[0] * real(d_dec_per_day) + dpos_dr_au[0] * real(d_r_au_per_day);
    let corr_m_per_day_1 = dpos_dra[1] * real(d_ra_per_day) + dpos_ddec[1] * real(d_dec_per_day) + dpos_dr_au[1] * real(d_r_au_per_day);
    let corr_m_per_day_2 = dpos_dra[2] * real(d_ra_per_day) + dpos_ddec[2] * real(d_dec_per_day) + dpos_dr_au[2] * real(d_r_au_per_day);

    let sec_per_day_r = real(sec_per_day);
    let vel_c = [
        vel[0] + corr_m_per_day_0 / sec_per_day_r,
        vel[1] + corr_m_per_day_1 / sec_per_day_r,
        vel[2] + corr_m_per_day_2 / sec_per_day_r,
    ];
    (pos_c, vel_c)
}
