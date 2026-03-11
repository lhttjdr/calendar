//! IAU 2000A 完整章动（从 data/IAU2000/tab5.3a 加载月日项）。标量统一 Real。
//! 支持文本（DataLoader）与二进制（.bin / 解压后的 .br）加载，与星历表一致。

use crate::astronomy::frame::nutation::{fundamental_arguments, table_parser};
use crate::math::real::{real, zero, RealOps, ToReal};
use crate::math::series::arcsec_to_rad;
use crate::platform::LoadError;
use crate::quantity::angle::PlaneAngle;

/// 单行四元组：ψ, ψ率, ε, ε率
type Quad = [table_parser::ParsedTerm; 4];

/// 每项二进制：14×i32 LE + 2×f64 LE
const TERM_BYTES: usize = 14 * 4 + 2 * 8;
/// 每行 4 项
const ROW_BYTES: usize = 4 * TERM_BYTES;

fn read_i32_le(b: &[u8], i: usize) -> Result<i32, LoadError> {
    if i + 4 > b.len() {
        return Err(LoadError::Io("tab5.3a binary: read i32 out of bounds".into()));
    }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&b[i..i + 4]);
    Ok(i32::from_le_bytes(arr))
}

fn read_f64_le(b: &[u8], i: usize) -> Result<f64, LoadError> {
    if i + 8 > b.len() {
        return Err(LoadError::Io("tab5.3a binary: read f64 out of bounds".into()));
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&b[i..i + 8]);
    Ok(f64::from_le_bytes(arr))
}

fn read_u32_le(b: &[u8], i: usize) -> Result<u32, LoadError> {
    if i + 4 > b.len() {
        return Err(LoadError::Io("tab5.3a binary: read u32 out of bounds".into()));
    }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&b[i..i + 4]);
    Ok(u32::from_le_bytes(arr))
}

/// 从 tab5.3a 解析结果构建的完整月日章动（SOFA Δε 序）。
pub struct Iau2000a {
    terms: Vec<Quad>,
}

impl Iau2000a {
    /// 从 load_tab53a 得到的四元组列表构建。ε 第 78 项起取反。
    pub fn from_quads(quads: Vec<Quad>) -> Self {
        let terms = quads
            .into_iter()
            .enumerate()
            .map(|(i, mut q)| {
                if i >= 78 {
                    q[2].1 = (-q[2].1.0, -q[2].1.1);
                    q[3].1 = (-q[3].1.0, -q[3].1.1);
                }
                q
            })
            .collect();
        Self { terms }
    }

    /// 月日项数量（用于诊断）
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// 从二进制格式加载（与星历表 .bin / 解压后 .br 一致）。格式：魔数 N53A、版本 u32=1、行数 u32、每行 4 项×（14×i32 LE + 2×f64 LE）。
    pub fn from_binary(bytes: &[u8]) -> Result<Self, LoadError> {
        const MAGIC: &[u8; 4] = b"N53A";
        const HEADER: usize = 4 + 4 + 4;
        if bytes.len() < HEADER {
            return Err(LoadError::Io("tab5.3a binary too short (header)".into()));
        }
        if &bytes[0..4] != MAGIC {
            return Err(LoadError::Io("tab5.3a binary bad magic".into()));
        }
        let version = read_u32_le(bytes, 4)?;
        if version != 1 {
            return Err(LoadError::Io(format!("tab5.3a binary unsupported version {}", version)));
        }
        let num_rows = read_u32_le(bytes, 8)? as usize;
        let mut terms = Vec::with_capacity(num_rows);
        let mut pos = HEADER;
        for _ in 0..num_rows {
            if pos + ROW_BYTES > bytes.len() {
                return Err(LoadError::Io("tab5.3a binary row truncated".into()));
            }
            let mut row: Quad = [
                (Vec::new(), (0.0, 0.0)),
                (Vec::new(), (0.0, 0.0)),
                (Vec::new(), (0.0, 0.0)),
                (Vec::new(), (0.0, 0.0)),
            ];
            for term in &mut row {
                let mut c14 = Vec::with_capacity(14);
                for _ in 0..14 {
                    c14.push(read_i32_le(bytes, pos)?);
                    pos += 4;
                }
                let a1 = read_f64_le(bytes, pos)?;
                pos += 8;
                let a2 = read_f64_le(bytes, pos)?;
                pos += 8;
                *term = (c14, (a1, a2));
            }
            terms.push(row);
        }
        if terms.is_empty() {
            return Err(LoadError::Io("tab5.3a binary no rows".into()));
        }
        Ok(Self::from_quads(terms))
    }

    /// 序列化为二进制（供构建脚本生成 .bin；.br 由前端或构建时压缩）。
    pub fn to_binary(&self) -> Vec<u8> {
        const MAGIC: &[u8; 4] = b"N53A";
        let mut out = Vec::with_capacity(12 + self.terms.len() * ROW_BYTES);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&1u32.to_le_bytes());
        out.extend_from_slice(&(self.terms.len() as u32).to_le_bytes());
        for row in &self.terms {
            for term in row {
                for &c in term.0.iter().take(14) {
                    out.extend_from_slice(&c.to_le_bytes());
                }
                for _ in term.0.len()..14 {
                    out.extend_from_slice(&0i32.to_le_bytes());
                }
                out.extend_from_slice(&term.1.0.to_le_bytes());
                out.extend_from_slice(&term.1.1.to_le_bytes());
            }
        }
        out
    }

    /// 章动 (Δψ, Δε)；t 儒略世纪。行星项固定 -0.135″、+0.388″。弧度。全程 Real。
    pub fn nutation(&self, t: impl ToReal) -> (PlaneAngle, PlaneAngle) {
        let t_r = real(t);
        let f = fundamental_arguments(t_r);
        let mut dpsi_arcsec = zero();
        let mut deps_arcsec = zero();
        for q in &self.terms {
            let c: &[i32] = &q[0].0;
            if c.len() < 5 {
                continue;
            }
            let theta = (0..5)
                .map(|i| real(c[i]) * f[i].rad())
                .fold(zero(), |a, b| a + b)
                .wrap_to_2pi();
            let (s, c_th) = (theta.sin(), theta.cos());
            let (psi_in, psi_out) = q[0].1;
            let (d_psi_in, d_psi_out) = q[1].1;
            let (eps_in, eps_out) = q[2].1;
            let (d_eps_in, d_eps_out) = q[3].1;
            dpsi_arcsec = dpsi_arcsec
                + (real(psi_in) + real(d_psi_in) * t_r) * s
                + (real(psi_out) + real(d_psi_out) * t_r) * c_th;
            deps_arcsec = deps_arcsec
                + (real(eps_in) + real(d_eps_in) * t_r) * s
                + (real(eps_out) + real(d_eps_out) * t_r) * c_th;
        }
        dpsi_arcsec = dpsi_arcsec + real(-0.135e-3);
        deps_arcsec = deps_arcsec + real(0.388e-3);
        (
            PlaneAngle::from_rad(arcsec_to_rad(dpsi_arcsec)),
            PlaneAngle::from_rad(arcsec_to_rad(deps_arcsec)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_binary_to_binary_roundtrip() {
        let one_row: Quad = [
            ((0..14).collect(), (-17.2064161_f64, -6.798383)),
            ((0..14).collect(), (0.9086, 3.3386)),
            ((0..14).collect(), (9.2052331, 0.0029)),
            ((0..14).collect(), (1.5377, 0.0002)),
        ];
        let iau = Iau2000a::from_quads(vec![one_row]);
        let bin = iau.to_binary();
        let iau2 = Iau2000a::from_binary(&bin).unwrap();
        assert_eq!(iau2.term_count(), 1);
    }
}
