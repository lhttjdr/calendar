//! DE406（及兼容）BSP (SPK) 历表读取：通过 NAIF DAF/SPK 内核计算日、月地心状态，与管线对接。
//!
//! 输出架为 **ICRS (GCRF)**，位置米、速度 m/s。与 Vsop87/ELPMPP02 的 MeanEcliptic(J2000) 不同，
//! 使用本历表时管线起点已是 ICRS，无需 Frame bias 或 DE406 patch；可直接交给
//! [`TransformGraph`](crate::astronomy::pipeline::TransformGraph) 转到 ApparentEcliptic 等目标架。
//!
//! # 与管线对接
//!
//! 实现 [`EphemerisProvider`](crate::astronomy::pipeline::EphemerisProvider)，可与
//! [`LightTimeCorrector`](crate::astronomy::pipeline::LightTimeCorrector)、
//! [`TransformGraph`](crate::astronomy::pipeline::TransformGraph) 等共用：
//! 用 `kernel.compute_state(Body::Sun, t)` 得到 ICRS 下地心日/月状态后，
//! 经 `TransformGraph::transform_to` 转到视黄道等目标架即可。

mod daf;
mod segment;

use crate::astronomy::pipeline::{Body, EphemerisProvider, State6};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};
use crate::quantity::reference_frame::ReferenceFrame;
use daf::{read_header, iter_spk_summaries, Endian, SpkSummary};
use segment::{load_segment_data, evaluate_segment, SegmentData};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

const J2000_JD: f64 = 2451545.0;
const SEC_PER_DAY: f64 = 86400.0;

/// NAIF 天体 ID：SSB 0，地心 399，太阳 10，月球 301，地月质心 3。
pub const NAIF_SSB: i32 = 0;
pub const NAIF_EMB: i32 = 3;
pub const NAIF_EARTH: i32 = 399;
pub const NAIF_SUN: i32 = 10;
pub const NAIF_MOON: i32 = 301;

/// DE406（或兼容）BSP 内核：可对给定 (center, target) 与 TDB 时刻求状态。
pub struct De406Kernel {
    summaries: Vec<SpkSummary>,
    segments: HashMap<(i32, i32), SegmentData>,
    endian: Endian,
    file_data: Vec<u8>,
}

impl De406Kernel {
    /// 从路径打开 .bsp 文件并加载日、月地心段（399,10）、（399,301）。
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let f = File::open(path.as_ref()).map_err(|e| e.to_string())?;
        Self::from_reader(BufReader::new(f))
    }

    /// 从已打开的 DAF/SPK 流构建；会读入整个文件到内存再解析。
    pub fn from_reader<R: Read + Seek>(mut r: R) -> Result<Self, String> {
        let mut file_data = Vec::new();
        r.read_to_end(&mut file_data).map_err(|e| e.to_string())?;
        if file_data.len() < 1024 {
            if file_data.starts_with(b"version https://git-lfs.github.com") {
                return Err(
                    "该文件是 Git LFS 指针，未拉取真实内容。请在仓库根执行: git lfs pull".to_string(),
                );
            }
            return Err(format!(
                "BSP 文件过短（{} 字节，DAF 至少需 1024 字节首记录）；可能非 NAIF SPK、文件损坏或为 Git LFS 指针（可试 git lfs pull）",
                file_data.len()
            ));
        }
        if file_data.len() >= 8 {
            let head = &file_data[..8];
            if head.starts_with(b"JPL PLAN") || head.starts_with(b"JPL EPHEM") {
                return Err(
                    "文件为 JPL 原始二进制格式（JPL PLAN/EPHEM），不是 NAIF BSP/SPK。\
                    请使用 de406.bsp（SPK 格式，如 NAIF 或 jplephem 提供的 .bsp）"
                        .to_string(),
                );
            }
        }
        let mut cursor = std::io::Cursor::new(&file_data);
        let header = read_header(&mut cursor)?;
        let summaries = iter_spk_summaries(&mut cursor, &header, Some(file_data.len() as u64))?;
        let mut kernel = Self {
            summaries,
            segments: HashMap::new(),
            endian: header.endian,
            file_data,
        };
        kernel.load_segments_for_bodies()?;
        Ok(kernel)
    }

    fn load_segments_for_bodies(&mut self) -> Result<(), String> {
        // 优先：地心日 (399,10) 或反向；地心月 (399,301) 或反向。
        // 备选：SSB 下 (0,10)/(0,399) 相减得地心日；(3,301)/(3,399) 相减得地心月。
        let want = [
            (NAIF_EARTH, NAIF_SUN),
            (NAIF_EARTH, NAIF_MOON),
            (NAIF_SSB, NAIF_SUN),
            (NAIF_SSB, NAIF_EARTH),
            (NAIF_SSB, NAIF_EMB),
            (NAIF_EMB, NAIF_MOON),
            (NAIF_EMB, NAIF_EARTH),
        ];
        for (center, target) in want {
            let summary = self
                .summaries
                .iter()
                .find(|s| s.center == center && s.target == target)
                .or_else(|| self.summaries.iter().find(|s| s.center == target && s.target == center));
            if let Some(summary) = summary {
                let key = (summary.center, summary.target);
                let seg = {
                    let mut cursor = std::io::Cursor::new(&self.file_data);
                    load_segment_data(&mut cursor, summary, self.endian)?
                };
                self.segments.insert(key, seg);
            }
        }
        Ok(())
    }

    /// 计算 target 相对 center 的状态：位置 km，速度 km/s，TDB 儒略日。
    /// 若仅有反向段 (target, center) 则取负；地心日/月可来自两段相减（如 (0,10)-(0,399)、(3,301)-(3,399)）。
    pub fn compute_state_km(
        &self,
        center: i32,
        target: i32,
        jd_tdb: f64,
    ) -> Result<([f64; 3], [f64; 3]), String> {
        let tdb_seconds = (jd_tdb - J2000_JD) * SEC_PER_DAY;
        if let Some(seg) = self.segments.get(&(center, target)) {
            return evaluate_segment(seg, tdb_seconds);
        }
        if let Some(seg) = self.segments.get(&(target, center)) {
            let (pos, vel) = evaluate_segment(seg, tdb_seconds)?;
            return Ok(([-pos[0], -pos[1], -pos[2]], [-vel[0], -vel[1], -vel[2]]));
        }
        // 地心日 = 日相对 SSB - 地相对 SSB。地相对 SSB = (0,399) 或 (0,3)+(3,399)
        if center == NAIF_EARTH && target == NAIF_SUN {
            if let (Some(sun), Some(earth)) = (
                self.segments.get(&(NAIF_SSB, NAIF_SUN)),
                self.segments.get(&(NAIF_SSB, NAIF_EARTH)),
            ) {
                let (pa, va) = evaluate_segment(sun, tdb_seconds)?;
                let (pb, vb) = evaluate_segment(earth, tdb_seconds)?;
                return Ok((
                    [pa[0] - pb[0], pa[1] - pb[1], pa[2] - pb[2]],
                    [va[0] - vb[0], va[1] - vb[1], va[2] - vb[2]],
                ));
            }
            if let (Some(sun), Some(emb), Some(earth_emb)) = (
                self.segments.get(&(NAIF_SSB, NAIF_SUN)),
                self.segments.get(&(NAIF_SSB, NAIF_EMB)),
                self.segments.get(&(NAIF_EMB, NAIF_EARTH)),
            ) {
                let (ps, vs) = evaluate_segment(sun, tdb_seconds)?;
                let (pe, ve) = evaluate_segment(emb, tdb_seconds)?;
                let (pb, vb) = evaluate_segment(earth_emb, tdb_seconds)?;
                let earth_x = pe[0] + pb[0];
                let earth_y = pe[1] + pb[1];
                let earth_z = pe[2] + pb[2];
                let earth_vx = ve[0] + vb[0];
                let earth_vy = ve[1] + vb[1];
                let earth_vz = ve[2] + vb[2];
                return Ok((
                    [ps[0] - earth_x, ps[1] - earth_y, ps[2] - earth_z],
                    [vs[0] - earth_vx, vs[1] - earth_vy, vs[2] - earth_vz],
                ));
            }
        }
        // 地心月 = 月相对 EMB - 地相对 EMB
        if center == NAIF_EARTH && target == NAIF_MOON {
            if let (Some(a), Some(b)) = (
                self.segments.get(&(NAIF_EMB, NAIF_MOON)),
                self.segments.get(&(NAIF_EMB, NAIF_EARTH)),
            ) {
                let (pa, va) = evaluate_segment(a, tdb_seconds)?;
                let (pb, vb) = evaluate_segment(b, tdb_seconds)?;
                return Ok((
                    [pa[0] - pb[0], pa[1] - pb[1], pa[2] - pb[2]],
                    [va[0] - vb[0], va[1] - vb[1], va[2] - vb[2]],
                ));
            }
        }
        let available: Vec<String> = self
            .segments
            .keys()
            .map(|(c, t)| format!("({},{})", c, t))
            .collect();
        Err(format!(
            "no segment for center={} target={} (nor reverse). 本文件已加载的段: {}",
            center,
            target,
            if available.is_empty() {
                "无（未找到 (399,10)/(10,399)、(399,301)/(301,399) 或 (0,10)/(0,399)/(3,301)/(3,399)）".to_string()
            } else {
                available.join(", ")
            }
        ))
    }

    /// 地心太阳状态（ICRS，米、m/s）。
    pub fn geocentric_sun(&self, jd_tdb: f64) -> Result<([f64; 3], [f64; 3]), String> {
        let (pos_km, vel_km_s) = self.compute_state_km(NAIF_EARTH, NAIF_SUN, jd_tdb)?;
        let to_m = 1000.0;
        Ok((
            [pos_km[0] * to_m, pos_km[1] * to_m, pos_km[2] * to_m],
            [vel_km_s[0] * to_m, vel_km_s[1] * to_m, vel_km_s[2] * to_m],
        ))
    }

    /// 地心月球状态（ICRS，米、m/s）。
    pub fn geocentric_moon(&self, jd_tdb: f64) -> Result<([f64; 3], [f64; 3]), String> {
        let (pos_km, vel_km_s) = self.compute_state_km(NAIF_EARTH, NAIF_MOON, jd_tdb)?;
        let to_m = 1000.0;
        Ok((
            [pos_km[0] * to_m, pos_km[1] * to_m, pos_km[2] * to_m],
            [vel_km_s[0] * to_m, vel_km_s[1] * to_m, vel_km_s[2] * to_m],
        ))
    }
}

impl EphemerisProvider for De406Kernel {
    /// JPL BSP 历表约定 TDB。
    fn evaluation_time_scale(&self) -> TimeScale {
        TimeScale::TDB
    }

    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6 {
        let jd_tdb = epoch.jd_tdb().as_f64();
        let (pos_m, vel_m_s) = match body {
            Body::Sun => self
                .geocentric_sun(jd_tdb)
                .expect("DE406 Sun segment missing or epoch out of range"),
            Body::Moon => self
                .geocentric_moon(jd_tdb)
                .expect("DE406 Moon segment missing or epoch out of range"),
        };
        let frame = ReferenceFrame::ICRS;
        State6::from_si_in_frame(
            frame,
            real(pos_m[0]),
            real(pos_m[1]),
            real(pos_m[2]),
            real(vel_m_s[0]),
            real(vel_m_s[1]),
            real(vel_m_s[2]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::time::TimeScale;
    use crate::math::real::real;

    #[test]
    fn de406_open_and_compute_if_bsp_present() {
        let path = std::env::var("DE406_BSP").ok().or_else(|| {
            for p in ["data/jpl/de406.bsp", "../data/jpl/de406.bsp"] {
                if std::path::Path::new(p).exists() {
                    return Some(p.to_string());
                }
            }
            None
        });
        let Some(path) = path else {
            eprintln!("skip de406_open_and_compute: no DE406_BSP env or data/jpl/de406.bsp");
            return;
        };
        let kernel = De406Kernel::open(&path).expect("open BSP");
        let jd = 2451545.0 + 0.5;
        let t = TimePoint::new(TimeScale::TDB, real(jd));
        let state_sun = kernel.compute_state(Body::Sun, t);
        let state_moon = kernel.compute_state(Body::Moon, t);
        assert_eq!(state_sun.frame(), ReferenceFrame::ICRS);
        assert_eq!(state_moon.frame(), ReferenceFrame::ICRS);
        let [x, y, z] = state_sun.position.to_meters();
        assert!(x.as_f64().abs() < 2e11, "Sun ~1 AU in m");
        assert!(y.as_f64().abs() < 2e11);
        assert!(z.as_f64().abs() < 2e11);
    }
}
