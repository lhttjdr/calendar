//! SPK 段：Type 2/3 Chebyshev 求值。
//! 与 jplephem Segment (Type 2/3) 行为一致；时间 TDB 秒（J2000 起），输出 km、km/s。

use super::daf::{read_double_array, Endian, SpkSummary};
use std::io::{Read, Seek};


/// 段数据：Type 2（仅位置）或 Type 3（位置+速度），Chebyshev 系数。
pub struct SegmentData {
    /// 第一段起始时间（TDB 秒，相对 J2000）
    pub init: f64,
    /// 每段覆盖时长（秒）
    pub intlen: f64,
    /// 每条记录双精度个数（含 MID,RADIUS）
    pub rsize: f64,
    /// 记录条数
    pub n: i32,
    /// 系数 [component][degree][record]；component 3=pos, 6=pos+vel；degree 从高到低。
    pub coefficients: Vec<Vec<Vec<f64>>>,
}

/// 从文件中加载 Type 2 或 Type 3 段数据。
pub fn load_segment_data<R: Read + Seek>(
    r: &mut R,
    summary: &SpkSummary,
    endian: Endian,
) -> Result<SegmentData, String> {
    let data_type = summary.data_type;
    let component_count = match data_type {
        2 => 3,
        3 => 6,
        _ => return Err(format!("SPK type {} not supported", data_type)),
    };

    let (start_i, end_i) = (summary.start_i, summary.end_i);
    if end_i <= start_i {
        return Err("segment end_i not set (DAF ni=5?); use BSP with NI=6 or patch summaries".to_string());
    }

    let trailer = read_double_array(r, end_i - 3, end_i, endian)?;
    let init = trailer[0];
    let intlen = trailer[1];
    let rsize = trailer[2];
    let n = trailer[3] as i32;
    let rsize_i = rsize as i32;
    let coefficient_count = (rsize_i - 2) / component_count;

    let full = read_double_array(r, start_i, end_i - 4, endian)?;
    let n_usize = n as usize;
    let rsize_usize = rsize_i as usize;
    if full.len() != n_usize * rsize_usize {
        return Err(format!(
            "segment length mismatch: {} != {}*{}",
            full.len(),
            n_usize,
            rsize_usize
        ));
    }

    let coeff_per_comp = coefficient_count as usize;
    let comp_count = component_count as usize;
    let mut coefficients = vec![
        vec![vec![0.0; n_usize]; coeff_per_comp];
        comp_count
    ];
    for (rec, chunk) in full.chunks_exact(rsize_usize).enumerate() {
        let coeffs = &chunk[2..];
        for comp in 0..comp_count {
            for d in 0..coeff_per_comp {
                let k = coeff_per_comp - 1 - d;
                coefficients[comp][d][rec] = coeffs[comp * coeff_per_comp + k];
            }
        }
    }

    Ok(SegmentData {
        init,
        intlen,
        rsize,
        n,
        coefficients,
    })
}

/// 在段内用 Chebyshev 求值：返回 (位置 km, 速度 km/s)；Type 2 时速度为数值微分。
pub fn evaluate_segment(
    seg: &SegmentData,
    tdb_seconds: f64,
) -> Result<( [f64; 3], [f64; 3] ), String> {
    let init = seg.init;
    let intlen = seg.intlen;
    let n = seg.n as i32;
    let offset = tdb_seconds - init;
    let index_f = offset / intlen;
    let mut index = index_f.floor() as i32;
    let mut offset_in_int = offset - (index as f64) * intlen;
    if index >= n {
        index = n - 1;
        offset_in_int = intlen;
    } else if index < 0 {
        return Err("epoch before segment start".to_string());
    }

    let component_count = seg.coefficients.len();
    let pos_only = component_count == 3;
    let degree = seg.coefficients[0].len() - 1;

    let s = 2.0 * offset_in_int / intlen - 1.0;
    let s2 = 2.0 * s;

    let mut pos = [0.0; 3];
    for comp in 0..3 {
        let coeffs = &seg.coefficients[comp];
        let mut w0 = 0.0f64;
        let mut w1 = 0.0f64;
        for d in 0..degree {
            let c = coeffs[d][index as usize];
            let w2 = w1;
            w1 = w0;
            w0 = c + (s2 * w1 - w2);
        }
        pos[comp] = coeffs[degree][index as usize] + (s * w0 - w1);
    }

    let vel = if pos_only {
        let delta = 1e-6 * intlen;
        let (p0, _) = evaluate_segment_at_offset(seg, index, offset_in_int - delta)?;
        let (p1, _) = evaluate_segment_at_offset(seg, index, offset_in_int + delta)?;
        let scale = 0.5 / delta;
        [
            (p1[0] - p0[0]) * scale,
            (p1[1] - p0[1]) * scale,
            (p1[2] - p0[2]) * scale,
        ]
    } else {
        let mut vel = [0.0; 3];
        for comp in 0..3 {
            let coeffs = &seg.coefficients[comp + 3];
            let mut w0 = 0.0f64;
            let mut w1 = 0.0f64;
            for d in 0..degree {
                let c = coeffs[d][index as usize];
                let w2 = w1;
                w1 = w0;
                w0 = c + (s2 * w1 - w2);
            }
            vel[comp] = coeffs[degree][index as usize] + (s * w0 - w1);
        }
        vel
    };

    Ok((pos, vel))
}

fn evaluate_segment_at_offset(
    seg: &SegmentData,
    index: i32,
    offset_in_int: f64,
) -> Result<([f64; 3], ()), String> {
    let intlen = seg.intlen;
    let degree = seg.coefficients[0].len() - 1;
    let s = 2.0 * offset_in_int / intlen - 1.0;
    let s2 = 2.0 * s;
    let mut pos = [0.0; 3];
    for comp in 0..3 {
        let coeffs = &seg.coefficients[comp];
        let mut w0 = 0.0f64;
        let mut w1 = 0.0f64;
        for d in 0..degree {
            let c = coeffs[d][index as usize];
            let w2 = w1;
            w1 = w0;
            w0 = c + (s2 * w1 - w2);
        }
        pos[comp] = coeffs[degree][index as usize] + (s * w0 - w1);
    }
    Ok((pos, ()))
}
