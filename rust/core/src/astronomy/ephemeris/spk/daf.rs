//! NAIF DAF (Double Precision Array File) 解析。
//! 仅实现读取 SPK 所需：文件头、摘要区、按摘要取双精度数组。
//! 参考：NAIF DAF Required Reading；jplephem/daf.py。

use std::io::{Read, Seek, SeekFrom};

const RECORD_SIZE: usize = 1024;
const J2000_JD: f64 = 2451545.0;
const SEC_PER_DAY: f64 = 86400.0;

/// DAF 文件记录号从 1 开始。
fn record_offset(record_number: u32) -> u64 {
    (record_number as u64 - 1) * (RECORD_SIZE as u64)
}

/// NAIF DAF 首记录布局（与 jplephem 一致）：8s II 60s III 8s ...，即 fward/bward/free 在 76、80、84。
const DAF_HEADER_ND_OFF: usize = 8;
const DAF_HEADER_NI_OFF: usize = 12;
const DAF_HEADER_FWARD_OFF: usize = 76;
const DAF_HEADER_BWARD_OFF: usize = 80;
const DAF_HEADER_FREE_OFF: usize = 84;
const DAF_HEADER_LOCFMT_OFF: usize = 88;

/// 从首记录按给定字节序读 nd, ni, fward（用于 NAIF/DAF locfmt 全零时推断字节序）。
fn read_header_u32s(rec: &[u8], endian: Endian) -> (u32, u32, u32) {
    let read_u32 = |i: usize| -> u32 {
        if endian == Endian::Little {
            u32::from_le_bytes(rec[i..i + 4].try_into().unwrap())
        } else {
            u32::from_be_bytes(rec[i..i + 4].try_into().unwrap())
        }
    };
    (read_u32(DAF_HEADER_ND_OFF), read_u32(DAF_HEADER_NI_OFF), read_u32(DAF_HEADER_FWARD_OFF))
}

#[derive(Debug)]
pub struct DafHeader {
    pub endian: Endian,
    /// 摘要中双精度个数（SPK 为 2）
    pub nd: u32,
    /// 摘要中整数个数（SPK 为 5 或 6）
    pub ni: u32,
    /// 第一个摘要记录号
    pub fward: u32,
    /// 最后一个摘要记录号（向后链）
    pub bward: u32,
    /// 第一个空闲字索引（数据区在此前）
    pub free: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

impl Endian {
    fn from_locfmt(locfmt: &[u8]) -> Option<Self> {
        if locfmt.starts_with(b"LTL-IEEE") || locfmt.starts_with(b"LTL") {
            Some(Endian::Little)
        } else if locfmt.starts_with(b"BIG-IEEE") || locfmt.starts_with(b"BIG") {
            Some(Endian::Big)
        } else {
            None
        }
    }
}

/// 单个摘要：SPK 段描述。2 双精度 + 5 整数（NAIF 标准）；部分文件为 2+6。
#[derive(Clone, Debug)]
pub struct SpkSummary {
    pub start_second: f64,
    pub end_second: f64,
    pub target: i32,
    pub center: i32,
    pub frame: i32,
    pub data_type: i32,
    pub start_i: i32,
    pub end_i: i32,
}

impl SpkSummary {
    pub fn start_jd(&self) -> f64 {
        J2000_JD + self.start_second / SEC_PER_DAY
    }
    pub fn end_jd(&self) -> f64 {
        J2000_JD + self.end_second / SEC_PER_DAY
    }
}

/// 从记录 1 解析 DAF 文件头；支持 NAIF/DAF 与 DAF/ 两种魔数。
pub fn read_header<R: Read + Seek>(r: &mut R) -> Result<DafHeader, String> {
    let mut rec = [0u8; RECORD_SIZE];
    r.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    r.read_exact(&mut rec).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("fill whole buffer") {
            format!("DAF 文件过短（需至少 {} 字节的首记录）或非 NAIF BSP: {}", RECORD_SIZE, msg)
        } else {
            msg
        }
    })?;

    let locidw = &rec[0..8];
    let endian = if locidw == b"NAIF/DAF" || locidw == b"naif/daf" {
        let locfmt = &rec[DAF_HEADER_LOCFMT_OFF..DAF_HEADER_LOCFMT_OFF + 8];
        match Endian::from_locfmt(locfmt) {
            Some(e) => e,
            None => {
                // NAIF/DAF 常见 locfmt 未填（全零）：按 jplephem 用两种字节序试，取 nd==2 者
                let (nd_le, ni_le, fward_le) = read_header_u32s(&rec, Endian::Little);
                let (nd_be, ni_be, fward_be) = read_header_u32s(&rec, Endian::Big);
                if nd_le == 2 && ni_le >= 4 && ni_le <= 8 && fward_le >= 1 {
                    Endian::Little
                } else if nd_be == 2 && ni_be >= 4 && ni_be <= 8 && fward_be >= 1 {
                    Endian::Big
                } else {
                    return Err(format!("NAIF/DAF 但 locfmt 全零且无法推断字节序（小端 nd={} ni={} fward={}，大端 nd={} ni={} fward={}）", nd_le, ni_le, fward_le, nd_be, ni_be, fward_be));
                }
            }
        }
    } else if locidw.starts_with(b"DAF/") || locidw.starts_with(b"daf/") {
        let locfmt = &rec[DAF_HEADER_LOCFMT_OFF..DAF_HEADER_LOCFMT_OFF + 8];
        Endian::from_locfmt(locfmt)
            .ok_or_else(|| format!("unknown DAF locfmt: {:?}", locfmt))?
    } else {
        return Err(format!("not a DAF file, locidw: {:?}", locidw));
    };

    let read_u32 = |i: usize| -> u32 {
        if endian == Endian::Little {
            u32::from_le_bytes(rec[i..i + 4].try_into().unwrap())
        } else {
            u32::from_be_bytes(rec[i..i + 4].try_into().unwrap())
        }
    };

    let nd = read_u32(DAF_HEADER_ND_OFF);
    let ni = read_u32(DAF_HEADER_NI_OFF);
    let fward = read_u32(DAF_HEADER_FWARD_OFF);
    let bward = read_u32(DAF_HEADER_BWARD_OFF);
    let free = read_u32(DAF_HEADER_FREE_OFF);

    Ok(DafHeader {
        endian,
        nd,
        ni,
        fward,
        bward,
        free,
    })
}

/// 摘要控制：每条摘要记录前 24 字节为 3 个 double（next, prev, n_summaries 以 double 存）。
fn read_summary_control(rec: &[u8], endian: Endian) -> (u32, u32, u32) {
    let read_f64 = |i: usize| -> f64 {
        if endian == Endian::Little {
            f64::from_le_bytes(rec[i..i + 8].try_into().unwrap())
        } else {
            f64::from_be_bytes(rec[i..i + 8].try_into().unwrap())
        }
    };
    let next = read_f64(0) as u32;
    let prev = read_f64(8) as u32;
    let n_sum = read_f64(16) as u32;
    (next, prev, n_sum)
}

/// 读一个 double（8 字节）。
fn read_f64(buf: &[u8], i: usize, endian: Endian) -> f64 {
    if endian == Endian::Little {
        f64::from_le_bytes(buf[i..i + 8].try_into().unwrap())
    } else {
        f64::from_be_bytes(buf[i..i + 8].try_into().unwrap())
    }
}

/// 读一个 i32（4 字节）。
fn read_i32(buf: &[u8], i: usize, endian: Endian) -> i32 {
    if endian == Endian::Little {
        i32::from_le_bytes(buf[i..i + 4].try_into().unwrap())
    } else {
        i32::from_be_bytes(buf[i..i + 4].try_into().unwrap())
    }
}

/// 遍历所有 SPK 摘要。若提供 file_len_bytes，则 NI=5 推断的 end_i 不会超过文件可容纳字数（避免 "failed to fill whole buffer"）。
pub fn iter_spk_summaries<R: Read + Seek>(
    r: &mut R,
    header: &DafHeader,
    file_len_bytes: Option<u64>,
) -> Result<Vec<SpkSummary>, String> {
    if header.nd != 2 {
        return Err(format!("DAF nd={} not supported for SPK", header.nd));
    }
    let summary_len = (header.nd as usize) * 8 + (header.ni as usize) * 4;
    let summary_step = (summary_len + 7) & !7;

    let mut out = Vec::new();
    let mut rec_num = header.fward;
    let mut rec = [0u8; RECORD_SIZE];

    while rec_num != 0 {
        let offset = record_offset(rec_num);
        if let Some(len) = file_len_bytes {
            if offset + (RECORD_SIZE as u64) > len {
                return Err(format!(
                    "DAF 摘要记录 {} 超出文件尾（偏移 {}，需 1024 字节，文件共 {} 字节）；文件可能被截断或非标准 BSP",
                    rec_num, offset, len
                ));
            }
        }
        r.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
        r.read_exact(&mut rec).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("fill whole buffer") {
                format!(
                    "读取 DAF 摘要记录 {} 时超出文件尾（文件可能被截断）；{}",
                    rec_num, msg
                )
            } else {
                msg
            }
        })?;

        let (next, _prev, n_sum) = read_summary_control(&rec, header.endian);
        let n = n_sum as usize;
        for i in 0..n {
            let base = 24 + i * summary_step;
            if base + summary_len > RECORD_SIZE {
                break;
            }
            let s0 = read_f64(&rec, base, header.endian);
            let s1 = read_f64(&rec, base + 8, header.endian);
            let t = read_i32(&rec, base + 16, header.endian);
            let c = read_i32(&rec, base + 20, header.endian);
            let f = read_i32(&rec, base + 24, header.endian);
            let dt = read_i32(&rec, base + 28, header.endian);
            let start_i = read_i32(&rec, base + 32, header.endian);
            let end_i = if header.ni >= 6 {
                read_i32(&rec, base + 36, header.endian)
            } else {
                start_i
            };
            out.push(SpkSummary {
                start_second: s0,
                end_second: s1,
                target: t,
                center: c,
                frame: f,
                data_type: dt,
                start_i,
                end_i,
            });
        }
        rec_num = next;
    }

    // NI=5 时摘要无 end_i，用下一段 start_i - 1 或 free - 1 推断（NAIF 常见）；且不超过文件字数
    if header.ni < 6 && !out.is_empty() {
        out.sort_by_key(|s| s.start_i);
        let max_word = file_len_bytes.map(|b| (b / 8) as i32);
        for i in 0..out.len() {
            if out[i].end_i <= out[i].start_i {
                let inferred = if i + 1 < out.len() {
                    out[i + 1].start_i - 1
                } else {
                    (header.free as i32) - 1
                };
                out[i].end_i = match max_word {
                    Some(m) if inferred > m => m,
                    _ => inferred,
                };
            }
        }
    }

    Ok(out)
}

/// 从文件中读取双精度数组 [start_word..=end_word]（DAF 字从 1 开始）。
pub fn read_double_array<R: Read + Seek>(
    r: &mut R,
    start_word: i32,
    end_word: i32,
    endian: Endian,
) -> Result<Vec<f64>, String> {
    let start = start_word.max(1) as u64;
    let end = end_word.max(start_word) as u64;
    let len = (end - start + 1) as usize;
    let byte_start = (start - 1) * 8;
    let byte_len = len * 8;

    let stream_len = r.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    r.seek(SeekFrom::Start(byte_start)).map_err(|e| e.to_string())?;
    if byte_start + (byte_len as u64) > stream_len {
        return Err(format!(
            "DAF 段 [word {}..{}] 超出文件尾（文件 {} 字节，需读到 {}）；可能 NI=5 推断的 end_i 过大或文件被截断",
            start_word,
            end_word,
            stream_len,
            byte_start + byte_len as u64
        ));
    }
    let mut buf = vec![0u8; byte_len];
    r.read_exact(&mut buf).map_err(|e| {
        format!(
            "读取 DAF 段 [word {}..{}] 失败: {}（文件可能被截断或非 NAIF BSP）",
            start_word,
            end_word,
            e
        )
    })?;

    let mut arr = Vec::with_capacity(len);
    for i in 0..len {
        arr.push(read_f64(&buf, i * 8, endian));
    }
    Ok(arr)
}
