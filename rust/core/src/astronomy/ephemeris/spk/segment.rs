//! SPK 段：Type 2/3 Chebyshev 求值。
//! 与 jplephem Segment (Type 2/3) 行为一致；时间 TDB 秒（J2000 起），输出 km、km/s。
//!
//! 段数据保持文件中的原始记录格式（每条记录 = MID、RADIUS、各分量的 Chebyshev 系数 T0..Tn），求值时只取所需的一条记录，
//! 记录可来自内存（整段读入）或文件（按需 seek 读取，供 DE441 这类 GB 级文件）。

use super::daf::{read_double_array, Endian, SpkSummary};
use std::io::{Read, Seek};

/// 段的记录布局（取自段尾 4 个双精度：INIT、INTLEN、RSIZE、N）。
#[derive(Clone, Debug)]
pub struct SegmentLayout {
    /// 第一条记录起始时间（TDB 秒，相对 J2000）
    pub init: f64,
    /// 每条记录覆盖时长（秒）
    pub intlen: f64,
    /// 每条记录双精度个数（含 MID、RADIUS）
    pub rsize: usize,
    /// 记录条数
    pub n: usize,
    /// 分量数：Type 2 为 3（仅位置），Type 3 为 6（位置 + 速度）
    pub components: usize,
    /// 段数据在文件中的起始字（1 起算，DAF 地址）
    pub start_word: i32,
}

impl SegmentLayout {
    /// 读段尾，得布局。
    pub fn read<R: Read + Seek>(r: &mut R, summary: &SpkSummary, endian: Endian) -> Result<Self, String> {
        let components = match summary.data_type {
            2 => 3,
            3 => 6,
            t => return Err(format!("SPK type {t} not supported")),
        };
        if summary.end_i <= summary.start_i {
            return Err("segment end_i not set (DAF ni=5?); use BSP with NI=6 or patch summaries".to_string());
        }
        let trailer = read_double_array(r, summary.end_i - 3, summary.end_i, endian)?;
        let layout = Self {
            init: trailer[0],
            intlen: trailer[1],
            rsize: trailer[2] as usize,
            n: trailer[3] as usize,
            components,
            start_word: summary.start_i,
        };
        let words = (summary.end_i - 4 - summary.start_i + 1) as usize;
        if words != layout.n * layout.rsize {
            return Err(format!("segment length mismatch: {} != {}*{}", words, layout.n, layout.rsize));
        }
        Ok(layout)
    }

    fn coefficients_per_component(&self) -> usize {
        (self.rsize - 2) / self.components
    }

    /// 定位记录：返回 (记录号, 记录内偏移秒)。越出段覆盖范围时报错（段末端点归最后一条记录，同 jplephem）。
    /// 时刻为两段和 hi + lo（秒，自 J2000）。记录内偏移按 (hi − 记录起点) + lo 求，前一步精确，
    /// 误差只有一次舍入（约 1e-10 s，记录长约 1e6 s），与时刻离段起点多远无关。
    pub fn locate_split(&self, t: TdbSeconds) -> Result<(usize, f64), String> {
        let tdb_seconds = t.hi;
        let offset = tdb_seconds - self.init + t.lo;
        let end = self.intlen * self.n as f64;
        if offset < 0.0 {
            return Err("epoch before segment start".to_string());
        }
        if offset > end {
            return Err("epoch after segment end".to_string());
        }
        let index = ((offset / self.intlen).floor() as usize).min(self.n - 1);
        // 记录起点 init + index·intlen 为整秒之和，f64 精确；hi 与之相距不到一条记录长，相减亦精确。
        // 不可先算 hi − init：段起点可远在前 3000 年（约 −1.6e11 s），该差在 f64 里只到约 3e-5 s。
        let record_start = self.init + index as f64 * self.intlen;
        Ok((index, (t.hi - record_start) + t.lo))
    }

    /// 第 index 条记录的起止字（DAF 地址，含）。
    pub fn record_words(&self, index: usize) -> (i32, i32) {
        let start = self.start_word + (index * self.rsize) as i32;
        (start, start + self.rsize as i32 - 1)
    }
}

/// TDB 时刻（秒，自 J2000），两段和 hi + lo：单个 f64 表示儒略日时只到约 40 µs（行星位置抖动约 1e-11 rad，
/// 使平缓极值的时刻抖动数十秒），故由 double-double 时刻拆成两段传入。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TdbSeconds {
    pub hi: f64,
    pub lo: f64,
}

/// 用一条记录求值：返回 (位置 km, 速度 km/s)；Type 2 时速度由位置 Chebyshev 级数的导数得到。
pub fn evaluate_record(layout: &SegmentLayout, record: &[f64], offset_in_int: f64) -> ([f64; 3], [f64; 3]) {
    let ncoef = layout.coefficients_per_component();
    let s = 2.0 * offset_in_int / layout.intlen - 1.0;
    let comp = |c: usize| &record[2 + c * ncoef..2 + (c + 1) * ncoef];
    let mut pos = [0.0; 3];
    let mut vel = [0.0; 3];
    for c in 0..3 {
        pos[c] = clenshaw(comp(c), s);
        vel[c] = if layout.components == 6 {
            clenshaw(comp(c + 3), s)
        } else {
            // d/dt = (2/intlen)·d/ds
            clenshaw_derivative(comp(c), s) * 2.0 / layout.intlen
        };
    }
    (pos, vel)
}

/// Σ c_k T_k(s)。
fn clenshaw(c: &[f64], s: f64) -> f64 {
    let (mut b1, mut b2) = (0.0, 0.0);
    for &ck in c.iter().skip(1).rev() {
        let b0 = 2.0 * s * b1 - b2 + ck;
        b2 = b1;
        b1 = b0;
    }
    s * b1 - b2 + c[0]
}

/// Σ c_k T_k'(s)，用 T_k' = k U_{k−1}。
fn clenshaw_derivative(c: &[f64], s: f64) -> f64 {
    // Σ_{k≥1} k c_k U_{k−1}(s)，U 的 Clenshaw 递推
    let (mut b1, mut b2) = (0.0, 0.0);
    for k in (1..c.len()).rev() {
        let b0 = 2.0 * s * b1 - b2 + k as f64 * c[k];
        b2 = b1;
        b1 = b0;
    }
    b1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chebyshev 求值与导数对照显式多项式：c = [1, 2, 3, 4] → 1 + 2s + 3(2s²−1) + 4(4s³−3s)。
    #[test]
    fn clenshaw_matches_explicit() {
        let c = [1.0, 2.0, 3.0, 4.0];
        for s in [-1.0, -0.3, 0.0, 0.7, 1.0] {
            let v = 1.0 + 2.0 * s + 3.0 * (2.0 * s * s - 1.0) + 4.0 * (4.0 * s * s * s - 3.0 * s);
            let dv = 2.0 + 12.0 * s + 4.0 * (12.0 * s * s - 3.0);
            assert!((clenshaw(&c, s) - v).abs() < 1e-12);
            assert!((clenshaw_derivative(&c, s) - dv).abs() < 1e-12);
        }
    }
}
