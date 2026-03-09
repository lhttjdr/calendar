//! ELP-MPP02 主问题与摄动解析：ELP_MAIN.S1/S2/S3、ELP_PERT.S1/S2/S3。
//! ELPMPP02 表解析。

use crate::platform::{DataLoader, LoadError};

use super::{parse_constants::ParseConstants, Elpmpp02Correction, Elpmpp02Term};
use crate::math::angle::wrap_to_2pi;
use crate::math::real::{real, RealOps};
use crate::quantity::angle::PlaneAngle;

/// 主问题行：4i3,2x,f13.5,5f12.2 → ilu(1:4), A, B(1:5)
pub fn split_fortran_main(line: &str) -> Vec<f64> {
    let s = line.replace('D', "E").replace('d', "E");
    let main_offsets = [0, 3, 6, 9, 14, 27, 39, 51, 63, 75];
    let main_lens = [3, 3, 3, 3, 13, 12, 12, 12, 12, 12];
    let mut out = Vec::with_capacity(10);
    for (off, len) in main_offsets.iter().zip(main_lens.iter()) {
        let end = (*off + len).min(s.len());
        let seg = if *off < s.len() { s[*off..end].trim() } else { "" };
        let v: f64 = if seg.is_empty() { 0.0 } else { seg.parse().unwrap_or(0.0) };
        out.push(v);
    }
    out
}

/// 摄动行：S(20), C(20), 13×i3
const PERT_OFFSETS: [usize; 15] = [5, 25, 45, 48, 51, 54, 57, 60, 63, 66, 69, 72, 75, 78, 81];
const PERT_LENS: [usize; 15] = [20, 20, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3];

fn split_fortran_pert(line: &str) -> Vec<f64> {
    let s = line.replace('D', "E").replace('d', "E");
    let mut out = Vec::with_capacity(15);
    for (off, len) in PERT_OFFSETS.iter().zip(PERT_LENS.iter()) {
        let end = (*off + len).min(s.len());
        let seg = if *off < s.len() { s[*off..end].trim() } else { "" };
        let v: f64 = if seg.is_empty() { 0.0 } else { seg.parse().unwrap_or(0.0) };
        out.push(v);
    }
    out
}

/// Ci = A + (B1 + rSMA2drMM3*B5)*(-m*dnu + dnp) + (B2*dG + B3*dE + B4*de)
fn ci_main_sin(a: f64, b1: f64, b2: f64, b3: f64, b4: f64, b5: f64, c: &ParseConstants) -> f64 {
    a + (b1 + c.r_sma2dr_mm3 * b5) * (-c.ratio_mean_motion * c.delta_nu + c.delta_np)
        + (b2 * c.delta_gamma + b3 * c.delta_e + b4 * c.delta_ep)
}

/// 主问题 Sin：每行 → Elpmpp02Term (Ci, Fi, 0, ilu)。按固定列解析，不 trim 行内容。
pub fn parse_main_sin(lines: &[String], c: &ParseConstants) -> Vec<Elpmpp02Term> {
    let mut out = Vec::new();
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let items = split_fortran_main(line);
        if items.len() < 9 {
            continue;
        }
        let ilu0 = items[0] as i32;
        let ilu1 = items[1] as i32;
        let ilu2 = items[2] as i32;
        let ilu3 = items[3] as i32;
        let a = items[4];
        let b1 = items[5];
        let b2 = items[6];
        let b3 = items[7];
        let b4 = items[8];
        let b5 = if items.len() > 9 { items[9] } else { 0.0 };
        let ci = ci_main_sin(a, b1, b2, b3, b4, b5, c);
        let fi: Vec<_> = (0..5)
            .map(|k| {
                real(
                    ilu0 as f64 * c.delaunay[0][k]
                        + ilu1 as f64 * c.delaunay[1][k]
                        + ilu2 as f64 * c.delaunay[2][k]
                        + ilu3 as f64 * c.delaunay[3][k],
                )
            })
            .collect();
        out.push(Elpmpp02Term {
            ci: PlaneAngle::from_arcsec(real(ci)),
            fi,
            alpha: 0,
            ilu: [ilu0, ilu1, ilu2, ilu3],
        });
    }
    out
}

/// 主问题 Cos：A' = A - 2*A*delta_nu/3，Fi[0] += π/2 并 wrap。按固定列解析，不 trim 行内容。
pub fn parse_main_cos(lines: &[String], c: &ParseConstants) -> Vec<Elpmpp02Term> {
    let half_pi = core::f64::consts::FRAC_PI_2;
    let mut out = Vec::new();
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let items = split_fortran_main(line);
        if items.len() < 9 {
            continue;
        }
        let ilu0 = items[0] as i32;
        let ilu1 = items[1] as i32;
        let ilu2 = items[2] as i32;
        let ilu3 = items[3] as i32;
        let mut a = items[4];
        a -= 2.0 * a * c.delta_nu / 3.0;
        let b1 = items[5];
        let b2 = items[6];
        let b3 = items[7];
        let b4 = items[8];
        let b5 = if items.len() > 9 { items[9] } else { 0.0 };
        let ci = ci_main_sin(a, b1, b2, b3, b4, b5, c);
        let mut fi: Vec<_> = (0..5)
            .map(|k| {
                real(
                    ilu0 as f64 * c.delaunay[0][k]
                        + ilu1 as f64 * c.delaunay[1][k]
                        + ilu2 as f64 * c.delaunay[2][k]
                        + ilu3 as f64 * c.delaunay[3][k],
                )
            })
            .collect();
        fi[0] = real(wrap_to_2pi(fi[0].as_f64() + half_pi));
        out.push(Elpmpp02Term {
            ci: PlaneAngle::from_arcsec(real(ci)),
            fi,
            alpha: 0,
            ilu: [ilu0, ilu1, ilu2, ilu3],
        });
    }
    out
}

/// 摄动：Ci = sqrt(S²+C²)，fi0 = atan2(C,S)，Fi[a] = fi0 + dot(ifi, argsA) for a=0 else dot(ifi, argsA)。按固定列解析，不 trim 行内容。
pub fn parse_pert(lines: &[String], c: &ParseConstants) -> Vec<Elpmpp02Term> {
    let mut out = Vec::new();
    let mut alpha_t = 0i32;
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        if line.contains("PERTURBATIONS") {
            alpha_t += 1;
            continue;
        }
        let items = split_fortran_pert(line);
        if items.len() < 15 {
            continue;
        }
        let s = items[0];
        let c_val = items[1];
        let ci = (s * s + c_val * c_val).sqrt();
        let fi0 = c_val.atan2(s);
        let fi0 = wrap_to_2pi(fi0);
        let ifi: Vec<f64> = items[2..15].iter().copied().collect();
        if ifi.len() < 13 {
            continue;
        }
        let mut fi = Vec::with_capacity(5);
        for a in 0..5 {
            let args_a: Vec<f64> = (0..4)
                .map(|i| c.delaunay[i][a])
                .chain((0..8).map(|i| c.planetary[i][a]))
                .chain(std::iter::once(c.longitude_lunar_zeta[a]))
                .collect();
            let dot_ia: f64 = ifi.iter().zip(args_a.iter()).map(|(x, y)| x * y).sum();
            if a == 0 {
                fi.push(real(wrap_to_2pi(fi0 + dot_ia)));
            } else {
                fi.push(real(dot_ia));
            }
        }
        let ilu: [i32; 4] = [
            ifi[0] as i32,
            ifi.get(1).copied().unwrap_or(0.0) as i32,
            ifi.get(2).copied().unwrap_or(0.0) as i32,
            ifi.get(3).copied().unwrap_or(0.0) as i32,
        ];
        out.push(Elpmpp02Term {
            ci: PlaneAngle::from_arcsec(real(ci)),
            fi,
            alpha: alpha_t,
            ilu,
        });
    }
    out
}

/// 从数据目录加载完整 ELPMPP02；base_path 下需有 ELP_MAIN.S1/S2/S3、ELP_PERT.S1/S2/S3。
pub fn load_all(
    loader: &dyn DataLoader,
    base_path: &str,
    correction: Elpmpp02Correction,
) -> Result<super::Elpmpp02Data, LoadError> {
    let parse_constants = match correction {
        Elpmpp02Correction::DE405 | Elpmpp02Correction::DE406 => super::parse_constants::de405(),
        Elpmpp02Correction::LLR => {
            return Err(LoadError::Io(
                "ELPMPP02 LLR parse constants not yet implemented".to_string(),
            ));
        }
    };
    let period_v = parse_main_sin(&loader.read_lines(&format!("{}/ELP_MAIN.S1", base_path))?, &parse_constants);
    let period_u = parse_main_sin(&loader.read_lines(&format!("{}/ELP_MAIN.S2", base_path))?, &parse_constants);
    let period_r = parse_main_cos(&loader.read_lines(&format!("{}/ELP_MAIN.S3", base_path))?, &parse_constants);
    let poisson_v = parse_pert(&loader.read_lines(&format!("{}/ELP_PERT.S1", base_path))?, &parse_constants);
    let poisson_u = parse_pert(&loader.read_lines(&format!("{}/ELP_PERT.S2", base_path))?, &parse_constants);
    let poisson_r = parse_pert(&loader.read_lines(&format!("{}/ELP_PERT.S3", base_path))?, &parse_constants);

    let constants = super::Elpmpp02Constants::de405();
    Ok(super::Elpmpp02Data {
        period_v,
        period_u,
        period_r,
        poisson_v,
        poisson_u,
        poisson_r,
        constants,
        correction,
    })
}

// --------------- 二进制序列化（零解析加载用） ---------------

const ELP_BINARY_MAGIC: &[u8; 4] = b"ELP1";
const ELP_TERM_SIZE: usize = 8 + 5 * 8 + 4 + 4 * 4; // ci_arcsec f64, fi[5] f64, alpha i32, ilu[4] i32

/// 将一项列表序列化为二进制（与 terms_from_binary 对称）。格式见 doc/13-ephemeris-binary-format.md。
pub fn terms_to_binary(terms: &[Elpmpp02Term]) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + terms.len() * ELP_TERM_SIZE);
    out.extend_from_slice(ELP_BINARY_MAGIC);
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&(terms.len() as u32).to_le_bytes());
    for t in terms {
        out.extend_from_slice(&t.ci.arcsec().as_f64().to_le_bytes());
        for i in 0..5 {
            out.extend_from_slice(&t.fi.get(i).map(|r| r.as_f64()).unwrap_or(0.0).to_le_bytes());
        }
        out.extend_from_slice(&t.alpha.to_le_bytes());
        for &v in &t.ilu {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

fn read_u32_elp(b: &[u8], i: usize) -> Result<u32, LoadError> {
    if i + 4 > b.len() {
        return Err(LoadError::Io("ELP binary: read u32 out of bounds".into()));
    }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&b[i..i + 4]);
    Ok(u32::from_le_bytes(arr))
}

fn read_i32_elp(b: &[u8], i: usize) -> Result<i32, LoadError> {
    if i + 4 > b.len() {
        return Err(LoadError::Io("ELP binary: read i32 out of bounds".into()));
    }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&b[i..i + 4]);
    Ok(i32::from_le_bytes(arr))
}

fn read_f64_elp(b: &[u8], i: usize) -> Result<f64, LoadError> {
    if i + 8 > b.len() {
        return Err(LoadError::Io("ELP binary: read f64 out of bounds".into()));
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&b[i..i + 8]);
    Ok(f64::from_le_bytes(arr))
}

/// 从二进制反序列化一项列表（零解析）。
pub fn terms_from_binary(bytes: &[u8]) -> Result<Vec<Elpmpp02Term>, LoadError> {
    const HEADER_LEN: usize = 4 + 4 + 4;
    if bytes.len() < HEADER_LEN {
        return Err(LoadError::Io("ELP binary too short (header)".into()));
    }
    if &bytes[0..4] != ELP_BINARY_MAGIC {
        return Err(LoadError::Io("ELP binary bad magic".into()));
    }
    let version = read_u32_elp(bytes, 4)?;
    if version != 1 {
        return Err(LoadError::Io(format!("ELP binary unsupported version {}", version)));
    }
    let term_count = read_u32_elp(bytes, 8)? as usize;
    let mut terms = Vec::with_capacity(term_count);
    let mut pos = HEADER_LEN;
    for _ in 0..term_count {
        if pos + ELP_TERM_SIZE > bytes.len() {
            return Err(LoadError::Io("ELP binary term truncated".into()));
        }
        let ci_arcsec = read_f64_elp(bytes, pos)?;
        pos += 8;
        let mut fi = Vec::with_capacity(5);
        for _ in 0..5 {
            fi.push(real(read_f64_elp(bytes, pos)?));
            pos += 8;
        }
        let alpha = read_i32_elp(bytes, pos)?;
        pos += 4;
        let ilu = [
            read_i32_elp(bytes, pos)?,
            read_i32_elp(bytes, pos + 4)?,
            read_i32_elp(bytes, pos + 8)?,
            read_i32_elp(bytes, pos + 12)?,
        ];
        pos += 16;
        terms.push(Elpmpp02Term {
            ci: PlaneAngle::from_arcsec(real(ci_arcsec)),
            fi,
            alpha,
            ilu,
        });
    }
    Ok(terms)
}

/// 从 6 个二进制 buffer 加载完整 ELPMPP02（零解析）。顺序：MAIN.S1, MAIN.S2, MAIN.S3, PERT.S1, PERT.S2, PERT.S3。
pub fn load_all_from_binary(
    elp_main_s1: &[u8],
    elp_main_s2: &[u8],
    elp_main_s3: &[u8],
    elp_pert_s1: &[u8],
    elp_pert_s2: &[u8],
    elp_pert_s3: &[u8],
    correction: Elpmpp02Correction,
) -> Result<super::Elpmpp02Data, LoadError> {
    let period_v = terms_from_binary(elp_main_s1)?;
    let period_u = terms_from_binary(elp_main_s2)?;
    let period_r = terms_from_binary(elp_main_s3)?;
    let poisson_v = terms_from_binary(elp_pert_s1)?;
    let poisson_u = terms_from_binary(elp_pert_s2)?;
    let poisson_r = terms_from_binary(elp_pert_s3)?;
    let constants = super::Elpmpp02Constants::de405();
    Ok(super::Elpmpp02Data {
        period_v,
        period_u,
        period_r,
        poisson_v,
        poisson_u,
        poisson_r,
        constants,
        correction,
    })
}
