//! DE406、DE441 等 BSP (SPK) 历表读取：通过 NAIF DAF/SPK 内核计算日、月地心状态，与管线对接。
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
//!
//! # 数据来源
//!
//! - [`De406Kernel::open`]：只读目录与段尾，求值时按需从文件读所需的一条记录（DE441 等 GB 级文件不必整体载入内存）；
//! - [`De406Kernel::from_reader`]：整个文件读入内存（wasm、或调用方已持有字节流时）。
//!
//! 同一 (center, target) 可有多段（如 DE441 每个天体分前后两段），按时刻选取覆盖它的段；时刻不在任何段内时报错。

mod daf;
mod segment;

use crate::astronomy::pipeline::{Body, EphemerisProvider, Planet, State6};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};
use crate::quantity::reference_frame::ReferenceFrame;
use daf::{iter_spk_summaries, read_double_array, read_header, Endian, SpkSummary};
use segment::{evaluate_record, SegmentLayout};
pub use segment::TdbSeconds;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

const J2000_JD: f64 = 2451545.0;
const SEC_PER_DAY: f64 = 86400.0;

/// NAIF 天体 ID：SSB 0，地心 399，太阳 10，月球 301，地月质心 3。
pub const NAIF_SSB: i32 = 0;
pub const NAIF_EMB: i32 = 3;
pub const NAIF_EARTH: i32 = 399;
pub const NAIF_SUN: i32 = 10;
pub const NAIF_MOON: i32 = 301;

/// 段记录的来源。
enum Source {
    Memory(Vec<u8>),
    File(Mutex<BufReader<File>>),
}

/// 一段：覆盖时间（TDB 秒）与记录布局。
struct Segment {
    start_second: f64,
    end_second: f64,
    layout: SegmentLayout,
}

/// DE406、DE441 等 NAIF SPK（BSP）内核：可对给定 (center, target) 与 TDB 时刻求状态。
pub struct De406Kernel {
    summaries: Vec<SpkSummary>,
    segments: HashMap<(i32, i32), Vec<Segment>>,
    endian: Endian,
    source: Source,
}

impl De406Kernel {
    /// 从路径打开 .bsp：只读目录与所需各段的段尾，记录在求值时按需读取。
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let f = File::open(path.as_ref()).map_err(|e| e.to_string())?;
        let len = f.metadata().map_err(|e| e.to_string())?.len();
        let mut r = BufReader::new(f);
        let mut head = vec![0u8; 8.min(len as usize)];
        r.read_exact(&mut head).map_err(|e| e.to_string())?;
        Self::check_file_kind(len as usize, &head)?;
        r.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        let header = read_header(&mut r)?;
        let summaries = iter_spk_summaries(&mut r, &header, Some(len))?;
        let mut kernel = Self { summaries, segments: HashMap::new(), endian: header.endian, source: Source::File(Mutex::new(r)) };
        kernel.load_segments_for_bodies()?;
        Ok(kernel)
    }

    /// 从已打开的 DAF/SPK 流构建；整个文件读入内存。
    pub fn from_reader<R: Read + Seek>(mut r: R) -> Result<Self, String> {
        let mut file_data = Vec::new();
        r.read_to_end(&mut file_data).map_err(|e| e.to_string())?;
        Self::check_file_kind(file_data.len(), &file_data[..8.min(file_data.len())])?;
        let mut cursor = Cursor::new(&file_data);
        let header = read_header(&mut cursor)?;
        let summaries = iter_spk_summaries(&mut cursor, &header, Some(file_data.len() as u64))?;
        let mut kernel = Self { summaries, segments: HashMap::new(), endian: header.endian, source: Source::Memory(file_data) };
        kernel.load_segments_for_bodies()?;
        Ok(kernel)
    }

    fn check_file_kind(len: usize, head: &[u8]) -> Result<(), String> {
        if len < 1024 {
            if head.starts_with(b"version ") {
                return Err("该文件是 Git LFS 指针，未拉取真实内容。请在仓库根执行: git lfs pull".to_string());
            }
            return Err(format!(
                "BSP 文件过短（{len} 字节，DAF 至少需 1024 字节首记录）；可能非 NAIF SPK、文件损坏或为 Git LFS 指针（可试 git lfs pull）"
            ));
        }
        if head.starts_with(b"JPL PLAN") || head.starts_with(b"JPL EPHEM") {
            return Err("文件为 JPL 原始二进制格式（JPL PLAN/EPHEM），不是 NAIF BSP/SPK。\
                请使用 de406.bsp（SPK 格式，如 NAIF 或 jplephem 提供的 .bsp）"
                .to_string());
        }
        Ok(())
    }

    fn read_words(&self, start: i32, end: i32) -> Result<Vec<f64>, String> {
        match &self.source {
            Source::Memory(bytes) => read_double_array(&mut Cursor::new(bytes), start, end, self.endian),
            Source::File(f) => read_double_array(&mut *f.lock().unwrap(), start, end, self.endian),
        }
    }

    fn load_segments_for_bodies(&mut self) -> Result<(), String> {
        // 优先：地心日 (399,10)、地心月 (399,301)；备选：SSB 下 (0,10)/(0,399) 相减得地心日，(3,301)/(3,399) 相减得地心月。
        let want = [
            (NAIF_EARTH, NAIF_SUN),
            (NAIF_EARTH, NAIF_MOON),
            (NAIF_SSB, NAIF_SUN),
            (NAIF_SSB, NAIF_EARTH),
            (NAIF_SSB, NAIF_EMB),
            (NAIF_EMB, NAIF_MOON),
            (NAIF_EMB, NAIF_EARTH),
            // 行星系质心（相对 SSB）
            (NAIF_SSB, 1),
            (NAIF_SSB, 2),
            (NAIF_SSB, 4),
            (NAIF_SSB, 5),
            (NAIF_SSB, 6),
            (NAIF_SSB, 7),
            (NAIF_SSB, 8),
        ];
        let summaries = self.summaries.clone();
        for (center, target) in want {
            for summary in summaries.iter().filter(|s| (s.center, s.target) == (center, target)) {
                let layout = match &self.source {
                    Source::Memory(bytes) => SegmentLayout::read(&mut Cursor::new(bytes), summary, self.endian)?,
                    Source::File(f) => SegmentLayout::read(&mut *f.lock().unwrap(), summary, self.endian)?,
                };
                self.segments.entry((center, target)).or_default().push(Segment {
                    start_second: summary.start_second,
                    end_second: summary.end_second,
                    layout,
                });
            }
        }
        Ok(())
    }

    /// 按 (center, target) 段求值；无此段返回 None，时刻不在任何段内返回 Some(Err)。
    fn eval(&self, key: (i32, i32), t: TdbSeconds) -> Option<Result<([f64; 3], [f64; 3]), String>> {
        let tdb_seconds = t.hi;
        let segs = self.segments.get(&key)?;
        let Some(seg) = segs.iter().find(|s| s.start_second <= tdb_seconds && tdb_seconds <= s.end_second) else {
            let (lo, hi) = segs.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), s| (a.min(s.start_second), b.max(s.end_second)));
            return Some(Err(format!(
                "epoch JD {:.1} 超出 ({},{}) 段覆盖 [{:.1}, {:.1}]",
                J2000_JD + tdb_seconds / SEC_PER_DAY,
                key.0,
                key.1,
                J2000_JD + lo / SEC_PER_DAY,
                J2000_JD + hi / SEC_PER_DAY
            )));
        };
        Some((|| {
            let (index, offset) = seg.layout.locate_split(t)?;
            let (a, b) = seg.layout.record_words(index);
            let record = self.read_words(a, b)?;
            Ok(evaluate_record(&seg.layout, &record, offset))
        })())
    }

    /// 计算 target 相对 center 的状态：位置 km，速度 km/s，TDB 儒略日。
    /// 若仅有反向段 (target, center) 则取负；地心日/月可来自两段相减（如 (0,10)-(0,399)、(3,301)-(3,399)）。
    pub fn compute_state_km(
        &self,
        center: i32,
        target: i32,
        jd_tdb: f64,
    ) -> Result<([f64; 3], [f64; 3]), String> {
        self.compute_state_km_split(center, target, TdbSeconds { hi: (jd_tdb - J2000_JD) * SEC_PER_DAY, lo: 0.0 })
    }

    /// 同 [Self::compute_state_km]，时刻为两段秒数（见 [TdbSeconds]）。
    pub fn compute_state_km_split(&self, center: i32, target: i32, tdb_seconds: TdbSeconds) -> Result<([f64; 3], [f64; 3]), String> {
        let sub = |(pa, va): ([f64; 3], [f64; 3]), (pb, vb): ([f64; 3], [f64; 3])| {
            ([pa[0] - pb[0], pa[1] - pb[1], pa[2] - pb[2]], [va[0] - vb[0], va[1] - vb[1], va[2] - vb[2]])
        };
        if let Some(r) = self.eval((center, target), tdb_seconds) {
            return r;
        }
        if let Some(r) = self.eval((target, center), tdb_seconds) {
            let (p, v) = r?;
            return Ok(([-p[0], -p[1], -p[2]], [-v[0], -v[1], -v[2]]));
        }
        // 地心日 = 日相对 SSB − 地相对 SSB
        if center == NAIF_EARTH && target == NAIF_SUN {
            if let Some(sun) = self.eval((NAIF_SSB, NAIF_SUN), tdb_seconds) {
                return Ok(sub(sun?, self.earth_ssb_km_split(tdb_seconds)?));
            }
        }
        // 地心月 = 月相对 EMB − 地相对 EMB
        if center == NAIF_EARTH && target == NAIF_MOON {
            if let (Some(a), Some(b)) = (self.eval((NAIF_EMB, NAIF_MOON), tdb_seconds), self.eval((NAIF_EMB, NAIF_EARTH), tdb_seconds)) {
                return Ok(sub(a?, b?));
            }
        }
        let available: Vec<String> = self.segments.keys().map(|(c, t)| format!("({},{})", c, t)).collect();
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

    /// 文件中全部段的 (center, target)。
    pub fn segment_ids(&self) -> Vec<(i32, i32)> {
        self.summaries.iter().map(|s| (s.center, s.target)).collect()
    }

    /// 地球相对 SSB 的状态（km、km/s）：(0,399) 或 (0,3)+(3,399)。
    fn earth_ssb_km_split(&self, tdb_seconds: TdbSeconds) -> Result<([f64; 3], [f64; 3]), String> {
        if let Some(r) = self.eval((NAIF_SSB, NAIF_EARTH), tdb_seconds) {
            return r;
        }
        let (pa, va) = self.eval((NAIF_SSB, NAIF_EMB), tdb_seconds).ok_or("缺 (0,3) 段")??;
        let (pb, vb) = self.eval((NAIF_EMB, NAIF_EARTH), tdb_seconds).ok_or("缺 (3,399) 段")??;
        Ok(([pa[0] + pb[0], pa[1] + pb[1], pa[2] + pb[2]], [va[0] + vb[0], va[1] + vb[1], va[2] + vb[2]]))
    }

    /// 地心行星（系质心）状态（ICRS，米、m/s）。DE406 只有行星系质心段；火星以外行星本体与质心之差对地心方向
    /// 的影响 < 0.05″（木星系最大），此处不计。
    pub fn geocentric_planet(&self, planet: Planet, jd_tdb: f64) -> Result<([f64; 3], [f64; 3]), String> {
        self.geocentric_planet_split(planet, TdbSeconds { hi: (jd_tdb - J2000_JD) * SEC_PER_DAY, lo: 0.0 })
    }

    fn geocentric_planet_split(&self, planet: Planet, tdb_seconds: TdbSeconds) -> Result<([f64; 3], [f64; 3]), String> {
        let (pp, pv) = self
            .eval((NAIF_SSB, planet.naif_barycenter()), tdb_seconds)
            .ok_or_else(|| format!("缺 (0,{}) 段", planet.naif_barycenter()))??;
        let (ep, ev) = self.earth_ssb_km_split(tdb_seconds)?;
        Ok((
            [(pp[0] - ep[0]) * 1000.0, (pp[1] - ep[1]) * 1000.0, (pp[2] - ep[2]) * 1000.0],
            [(pv[0] - ev[0]) * 1000.0, (pv[1] - ev[1]) * 1000.0, (pv[2] - ev[2]) * 1000.0],
        ))
    }

    /// 地心太阳状态（ICRS，米、m/s）。
    pub fn geocentric_sun(&self, jd_tdb: f64) -> Result<([f64; 3], [f64; 3]), String> {
        self.geocentric_split(NAIF_SUN, TdbSeconds { hi: (jd_tdb - J2000_JD) * SEC_PER_DAY, lo: 0.0 })
    }

    /// 地心日、月（ICRS，米、m/s），时刻为两段秒数。
    fn geocentric_split(&self, target: i32, t: TdbSeconds) -> Result<([f64; 3], [f64; 3]), String> {
        let (pos_km, vel_km_s) = self.compute_state_km_split(NAIF_EARTH, target, t)?;
        let to_m = 1000.0;
        Ok((
            [pos_km[0] * to_m, pos_km[1] * to_m, pos_km[2] * to_m],
            [vel_km_s[0] * to_m, vel_km_s[1] * to_m, vel_km_s[2] * to_m],
        ))
    }

    /// 地心月球状态（ICRS，米、m/s）。
    pub fn geocentric_moon(&self, jd_tdb: f64) -> Result<([f64; 3], [f64; 3]), String> {
        self.geocentric_split(NAIF_MOON, TdbSeconds { hi: (jd_tdb - J2000_JD) * SEC_PER_DAY, lo: 0.0 })
    }
}

impl EphemerisProvider for De406Kernel {
    /// JPL BSP 历表约定 TDB。
    fn evaluation_time_scale(&self) -> TimeScale {
        TimeScale::TDB
    }

    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6 {
        // 时刻自 J2000 的秒数按 double-double 求，拆成两段 f64 传入（见 TdbSeconds）
        let s = (epoch.jd_tdb() - real(J2000_JD)) * real(SEC_PER_DAY);
        let hi = s.as_f64();
        let t = TdbSeconds { hi, lo: (s - real(hi)).as_f64() };
        let (pos_m, vel_m_s) = match body {
            Body::Sun => self.geocentric_split(NAIF_SUN, t).expect("DE406 Sun segment missing or epoch out of range"),
            Body::Moon => self.geocentric_split(NAIF_MOON, t).expect("DE406 Moon segment missing or epoch out of range"),
            Body::Planet(p) => self.geocentric_planet_split(p, t).expect("DE406 planet segment missing or epoch out of range"),
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
            for p in ["vendor/jpl/de406.bsp", "../vendor/jpl/de406.bsp"] {
                if std::path::Path::new(p).exists() {
                    return Some(p.to_string());
                }
            }
            None
        });
        let Some(path) = path else {
            eprintln!("skip de406_open_and_compute: no DE406_BSP env or vendor/jpl/de406.bsp");
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

