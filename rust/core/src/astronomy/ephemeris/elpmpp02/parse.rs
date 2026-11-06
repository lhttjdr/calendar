//! ELP/MPP02 级数：解析 ELP_MAIN.S1/S2/S3、ELP_PERT.S1/S2/S3 的原始系数，再按所选常数集生成求值项。
//!
//! 两步分开，同 Fortran `READFILE`：原始系数（主问题 A、B1—B5；摄动 S、C 与幅角整数系数）与常数无关，
//! 只解析一次；振幅改正与相位多项式依赖常数集（[ParseConstants]），每换一组常数（LLR、DE405、DE441 拟合等）重新生成。
//! 二进制（供前端零解析加载）存原始系数，故一套 .bin 适用于全部常数集，格式见 doc/engineering/ephemeris-binary-format.md。

use crate::platform::{DataLoader, LoadError};

use super::fit::Elpmpp02Fit;
use super::{parse_constants::ParseConstants, Elpmpp02Correction, Elpmpp02Term};
use crate::math::angle::wrap_to_2pi;
use crate::math::real::real;
use crate::quantity::angle::PlaneAngle;

/// 主问题原始项（Fortran 格式 4i3,2x,f13.5,5f12.2）：Delaunay 幅角 (D, F, l, l′) 的整数系数、振幅 A
/// （经纬度角秒，距离 km）、A 对常数 (ν, Γ, E, e′, n′) 的偏导 B1—B5。
#[derive(Clone, Debug, PartialEq)]
pub struct MainTermRaw {
    pub ilu: [i8; 4],
    pub a: f64,
    pub b: [f64; 5],
}

/// 摄动原始项（Fortran 格式 i5,2d20.13,16i3）：sin、cos 系数 S、C，前 13 个幅角整数系数
/// （D, F, l, l′，水星至海王星 8 个行星平黄经，ζ），所在 T 的幂次（0—3，由分段标题决定）。
#[derive(Clone, Debug, PartialEq)]
pub struct PertTermRaw {
    pub s: f64,
    pub c: f64,
    pub ifi: [i8; 13],
    pub power: u8,
}

/// 六个文件的原始系数：`main[iv]`、`pert[iv]`，iv = 0 经度、1 纬度、2 距离。
#[derive(Clone, Debug, PartialEq)]
pub struct Elpmpp02Raw {
    pub main: [Vec<MainTermRaw>; 3],
    pub pert: [Vec<PertTermRaw>; 3],
}

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

fn to_i8(v: f64, what: &str) -> Result<i8, LoadError> {
    if v.fract() != 0.0 || v < i8::MIN as f64 || v > i8::MAX as f64 {
        return Err(LoadError::Io(format!("ELPMPP02 {what} 整数系数超出范围：{v}")));
    }
    Ok(v as i8)
}

/// 解析主问题文件（首行为标题）。按固定列解析，不 trim 行内容。
pub fn parse_main_raw(lines: &[String]) -> Result<Vec<MainTermRaw>, LoadError> {
    let mut out = Vec::new();
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let it = split_fortran_main(line);
        out.push(MainTermRaw {
            ilu: [to_i8(it[0], "主问题")?, to_i8(it[1], "主问题")?, to_i8(it[2], "主问题")?, to_i8(it[3], "主问题")?],
            a: it[4],
            b: [it[5], it[6], it[7], it[8], it[9]],
        });
    }
    Ok(out)
}

/// 解析摄动文件：分 4 段（T⁰—T³），每段以含 `PERTURBATIONS` 的标题行开头。
pub fn parse_pert_raw(lines: &[String]) -> Result<Vec<PertTermRaw>, LoadError> {
    let mut out = Vec::new();
    let mut power: i32 = -1;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if line.contains("PERTURBATIONS") {
            power += 1;
            continue;
        }
        if power < 0 {
            return Err(LoadError::Io("ELPMPP02 摄动文件缺分段标题".into()));
        }
        let it = split_fortran_pert(line);
        let mut ifi = [0i8; 13];
        for (k, v) in ifi.iter_mut().enumerate() {
            *v = to_i8(it[2 + k], "摄动")?;
        }
        out.push(PertTermRaw { s: it[0], c: it[1], ifi, power: power as u8 });
    }
    Ok(out)
}

impl Elpmpp02Raw {
    /// 从文本加载；base_path 下需有 ELP_MAIN.S1/S2/S3、ELP_PERT.S1/S2/S3。
    pub fn load(loader: &dyn DataLoader, base_path: &str) -> Result<Self, LoadError> {
        let main = |n: u8| -> Result<Vec<MainTermRaw>, LoadError> {
            parse_main_raw(&loader.read_lines(&format!("{base_path}/ELP_MAIN.S{n}"))?)
        };
        let pert = |n: u8| -> Result<Vec<PertTermRaw>, LoadError> {
            parse_pert_raw(&loader.read_lines(&format!("{base_path}/ELP_PERT.S{n}"))?)
        };
        Ok(Self { main: [main(1)?, main(2)?, main(3)?], pert: [pert(1)?, pert(2)?, pert(3)?] })
    }

    /// 按常数集生成求值项（Fortran `READFILE` 的振幅改正与相位多项式）。
    /// 按常数集生成求值项；若常数集即该目标的内置常数集，一并挂上该目标的残差改正（[super::patch]）。
    pub fn build(&self, fit: &Elpmpp02Fit, correction: Elpmpp02Correction) -> super::Elpmpp02Data {
        let patch = if correction.builtin_fit().as_ref() == Some(fit) { correction.builtin_patch() } else { None };
        self.build_with_patch(fit, correction, patch.as_ref())
    }

    /// 同 [Self::build]，残差改正由调用方给出（`None` 为不加改正，拟合工具用）。
    pub fn build_with_patch(
        &self,
        fit: &Elpmpp02Fit,
        correction: Elpmpp02Correction,
        patch: Option<&super::patch::Elpmpp02PatchRaw>,
    ) -> super::Elpmpp02Data {
        let c = ParseConstants::from_fit(fit);
        super::Elpmpp02Data {
            patch: patch.map(|p| p.build(&c)).unwrap_or_default(),
            period_v: build_main(&self.main[0], &c, false),
            period_u: build_main(&self.main[1], &c, false),
            period_r: build_main(&self.main[2], &c, true),
            poisson_v: build_pert(&self.pert[0], &c),
            poisson_u: build_pert(&self.pert[1], &c),
            poisson_r: build_pert(&self.pert[2], &c),
            constants: super::Elpmpp02Constants::from_parse_constants(&c),
            correction,
            fit: fit.clone(),
        }
    }
}

/// 主问题项：Ci = A + (B1 + 2α/(3m)·B5)(δn′ − m·δν) + B2·δΓ + B3·δE + B4·δe′；距离级数先 A ← A − 2A·δν/3，
/// 相位加 π/2（cos 级数）。相位多项式 Fi = Σ ilu_i · Delaunay_i（含长期项改正）。
pub fn build_main(raw: &[MainTermRaw], c: &ParseConstants, distance: bool) -> Vec<Elpmpp02Term> {
    raw.iter()
        .map(|t| {
            let mut a = t.a;
            if distance {
                a -= 2.0 * a * c.delta_nu / 3.0;
            }
            let b = &t.b;
            let ci = a
                + (b[0] + c.r_sma2dr_mm3 * b[4]) * (-c.ratio_mean_motion * c.delta_nu + c.delta_np)
                + (b[1] * c.delta_gamma + b[2] * c.delta_e + b[3] * c.delta_ep);
            let mut fi: Vec<_> = (0..5)
                .map(|k| (0..4).map(|i| t.ilu[i] as f64 * c.delaunay[i][k]).sum::<f64>())
                .collect();
            if distance {
                fi[0] = wrap_to_2pi(fi[0] + core::f64::consts::FRAC_PI_2);
            }
            Elpmpp02Term {
                ci: PlaneAngle::from_arcsec(real(ci)),
                fi: fi.into_iter().map(real).collect(),
                alpha: 0,
                ilu: [t.ilu[0] as i32, t.ilu[1] as i32, t.ilu[2] as i32, t.ilu[3] as i32],
            }
        })
        .collect()
}

/// 摄动项：Ci = √(S² + C²)，Fi⁽⁰⁾ = atan2(C, S) + Σ ifi·幅角⁽⁰⁾，Fi⁽ᵏ⁾ = Σ ifi·幅角⁽ᵏ⁾；
/// 幅角依次为 4 个 Delaunay、8 个行星平黄经、ζ（均含长期项改正）。
pub fn build_pert(raw: &[PertTermRaw], c: &ParseConstants) -> Vec<Elpmpp02Term> {
    raw.iter()
        .map(|t| {
            let ci = (t.s * t.s + t.c * t.c).sqrt();
            let fi0 = wrap_to_2pi(t.c.atan2(t.s));
            let fi: Vec<_> = (0..5)
                .map(|a| {
                    let args = (0..4)
                        .map(|i| c.delaunay[i][a])
                        .chain((0..8).map(|i| c.planetary[i][a]))
                        .chain(std::iter::once(c.longitude_lunar_zeta[a]));
                    let dot: f64 = t.ifi.iter().zip(args).map(|(&x, y)| x as f64 * y).sum();
                    real(if a == 0 { wrap_to_2pi(fi0 + dot) } else { dot })
                })
                .collect();
            Elpmpp02Term {
                ci: PlaneAngle::from_arcsec(real(ci)),
                fi,
                alpha: t.power as i32,
                ilu: [t.ifi[0] as i32, t.ifi[1] as i32, t.ifi[2] as i32, t.ifi[3] as i32],
            }
        })
        .collect()
}

/// 从数据目录加载完整 ELPMPP02；base_path 下需有 ELP_MAIN.S1/S2/S3、ELP_PERT.S1/S2/S3。
/// 常数集由 `correction` 决定（全部内置）。
pub fn load_all(
    loader: &dyn DataLoader,
    base_path: &str,
    correction: Elpmpp02Correction,
) -> Result<super::Elpmpp02Data, LoadError> {
    let fit = correction.fit(loader)?;
    Ok(Elpmpp02Raw::load(loader, base_path)?.build(&fit, correction))
}

// --------------- 二进制序列化（零解析加载用；存原始系数，与常数集无关） ---------------

const MAIN_MAGIC: &[u8; 4] = b"ELPM";
const PERT_MAGIC: &[u8; 4] = b"ELPP";
const BINARY_VERSION: u32 = 2;
/// 主问题项：ilu 4×i8，A 与 B1—B5 共 6×f64
const MAIN_TERM_SIZE: usize = 4 + 6 * 8;
/// 摄动项：S、C 2×f64，ifi 13×i8，幂次 u8
const PERT_TERM_SIZE: usize = 2 * 8 + 13 + 1;

fn header(magic: &[u8; 4], n: usize, term_size: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + n * term_size);
    out.extend_from_slice(magic);
    out.extend_from_slice(&BINARY_VERSION.to_le_bytes());
    out.extend_from_slice(&(n as u32).to_le_bytes());
    out
}

/// 主问题原始项 → 二进制（与 [main_from_binary] 对称）。
pub fn main_to_binary(terms: &[MainTermRaw]) -> Vec<u8> {
    let mut out = header(MAIN_MAGIC, terms.len(), MAIN_TERM_SIZE);
    for t in terms {
        out.extend(t.ilu.iter().map(|&v| v as u8));
        out.extend_from_slice(&t.a.to_le_bytes());
        for v in t.b {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

/// 摄动原始项 → 二进制（与 [pert_from_binary] 对称）。
pub fn pert_to_binary(terms: &[PertTermRaw]) -> Vec<u8> {
    let mut out = header(PERT_MAGIC, terms.len(), PERT_TERM_SIZE);
    for t in terms {
        out.extend_from_slice(&t.s.to_le_bytes());
        out.extend_from_slice(&t.c.to_le_bytes());
        out.extend(t.ifi.iter().map(|&v| v as u8));
        out.push(t.power);
    }
    out
}

/// 校验文件头并返回项数与数据区。
fn check_header<'a>(bytes: &'a [u8], magic: &[u8; 4], term_size: usize) -> Result<(usize, &'a [u8]), LoadError> {
    if bytes.len() < 12 || &bytes[0..4] != magic {
        return Err(LoadError::Io(format!("ELP 二进制文件头不符（应为 {}）", String::from_utf8_lossy(magic))));
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != BINARY_VERSION {
        return Err(LoadError::Io(format!("ELP 二进制版本 {version} 不支持（应为 {BINARY_VERSION}），请重新生成 .bin")));
    }
    let n = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let body = &bytes[12..];
    if body.len() != n * term_size {
        return Err(LoadError::Io(format!("ELP 二进制长度不符：{} 项应 {} 字节，实为 {}", n, n * term_size, body.len())));
    }
    Ok((n, body))
}

fn f64_at(b: &[u8], i: usize) -> f64 {
    f64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}

/// 二进制 → 主问题原始项。
pub fn main_from_binary(bytes: &[u8]) -> Result<Vec<MainTermRaw>, LoadError> {
    let (_, body) = check_header(bytes, MAIN_MAGIC, MAIN_TERM_SIZE)?;
    Ok(body
        .chunks_exact(MAIN_TERM_SIZE)
        .map(|r| MainTermRaw {
            ilu: [r[0] as i8, r[1] as i8, r[2] as i8, r[3] as i8],
            a: f64_at(r, 4),
            b: [f64_at(r, 12), f64_at(r, 20), f64_at(r, 28), f64_at(r, 36), f64_at(r, 44)],
        })
        .collect())
}

/// 二进制 → 摄动原始项。
pub fn pert_from_binary(bytes: &[u8]) -> Result<Vec<PertTermRaw>, LoadError> {
    let (_, body) = check_header(bytes, PERT_MAGIC, PERT_TERM_SIZE)?;
    Ok(body
        .chunks_exact(PERT_TERM_SIZE)
        .map(|r| {
            let mut ifi = [0i8; 13];
            for (k, v) in ifi.iter_mut().enumerate() {
                *v = r[16 + k] as i8;
            }
            PertTermRaw { s: f64_at(r, 0), c: f64_at(r, 8), ifi, power: r[29] }
        })
        .collect())
}

impl Elpmpp02Raw {
    /// 从 6 个二进制 buffer 构造。顺序：MAIN.S1, MAIN.S2, MAIN.S3, PERT.S1, PERT.S2, PERT.S3。
    pub fn from_binary(bufs: [&[u8]; 6]) -> Result<Self, LoadError> {
        Ok(Self {
            main: [main_from_binary(bufs[0])?, main_from_binary(bufs[1])?, main_from_binary(bufs[2])?],
            pert: [pert_from_binary(bufs[3])?, pert_from_binary(bufs[4])?, pert_from_binary(bufs[5])?],
        })
    }
}

/// 从 6 个二进制 buffer 加载完整 ELPMPP02（零解析）。顺序：MAIN.S1, MAIN.S2, MAIN.S3, PERT.S1, PERT.S2, PERT.S3。
/// 常数集由 `correction` 决定（全部内置）；自定常数集用 [load_all_from_binary_with_fit]。
pub fn load_all_from_binary(
    elp_main_s1: &[u8],
    elp_main_s2: &[u8],
    elp_main_s3: &[u8],
    elp_pert_s1: &[u8],
    elp_pert_s2: &[u8],
    elp_pert_s3: &[u8],
    correction: Elpmpp02Correction,
) -> Result<super::Elpmpp02Data, LoadError> {
    let fit = correction.builtin_fit().expect("全部常数集均内置");
    load_all_from_binary_with_fit(
        [elp_main_s1, elp_main_s2, elp_main_s3, elp_pert_s1, elp_pert_s2, elp_pert_s3],
        &fit,
        correction,
    )
}

/// 同 [load_all_from_binary]，常数集由调用方给出（如由 `data/fit/elpmpp02-de441.txt` 解析）。
pub fn load_all_from_binary_with_fit(
    bufs: [&[u8]; 6],
    fit: &Elpmpp02Fit,
    correction: Elpmpp02Correction,
) -> Result<super::Elpmpp02Data, LoadError> {
    Ok(Elpmpp02Raw::from_binary(bufs)?.build(fit, correction))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn binary_round_trip_matches_text() {
        let raw = Elpmpp02Raw::load(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02).unwrap();
        assert_eq!(raw.main.iter().map(Vec::len).sum::<usize>(), 2645);
        assert_eq!(raw.pert.iter().map(Vec::len).sum::<usize>(), 33256);
        let mb: Vec<Vec<u8>> = raw.main.iter().map(|v| main_to_binary(v)).collect();
        let pb: Vec<Vec<u8>> = raw.pert.iter().map(|v| pert_to_binary(v)).collect();
        let back = Elpmpp02Raw::from_binary([&mb[0], &mb[1], &mb[2], &pb[0], &pb[1], &pb[2]]).unwrap();
        assert_eq!(raw, back);
    }
}
