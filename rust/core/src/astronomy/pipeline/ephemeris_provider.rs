//! 数据提供层：EphemerisProvider（文档 3.1）。泛型于 R: Real，不在本层指定 f64。
//!
//! **时间尺度**：每一步按模型要求用 TT 或 TDB，无约定处才可任选。历表求值：VSOP87、DE406 规定用 **TDB**；ELPMPP02 规定用 **TT**。调用方应在边界转为对应尺度再传入。

use crate::astronomy::time::{TimePoint, TimeScale};
use crate::astronomy::ephemeris::{position_velocity, Elpmpp02Data, Vsop87};
use crate::quantity::{epoch::Epoch, position::Position, reference_frame::ReferenceFrame, velocity::Velocity};
use super::state::State6;

/// 天体标识，用于从历表取状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Sun,
    Moon,
    Planet(Planet),
}

/// 大行星（不含地球）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Planet {
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

impl Planet {
    pub const ALL: [Planet; 7] =
        [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn, Planet::Uranus, Planet::Neptune];

    /// NAIF 系统质心编号（DE406 BSP 只给各行星系质心，水星、金星无卫星，质心即行星本体）。
    pub fn naif_barycenter(self) -> i32 {
        match self {
            Planet::Mercury => 1,
            Planet::Venus => 2,
            Planet::Mars => 4,
            Planet::Jupiter => 5,
            Planet::Saturn => 6,
            Planet::Uranus => 7,
            Planet::Neptune => 8,
        }
    }

    /// VSOP87 数据文件扩展名（`VSOP87B.mer` 等）。
    pub fn vsop87_suffix(self) -> &'static str {
        match self {
            Planet::Mercury => "mer",
            Planet::Venus => "ven",
            Planet::Mars => "mar",
            Planet::Jupiter => "jup",
            Planet::Saturn => "sat",
            Planet::Uranus => "ura",
            Planet::Neptune => "nep",
        }
    }

    /// 是否为地内行星（有大距、无冲）。
    pub fn is_inferior(self) -> bool {
        matches!(self, Planet::Mercury | Planet::Venus)
    }
}

/// 历表提供者：在给定时刻返回天体的 6D 状态（文档 3.1）。标量 Real。
///
/// 入参 `epoch` 应为该历表**规定的时间尺度**（[`evaluation_time_scale()`]），调用方在边界做 TT↔TDB 转换。
pub trait EphemerisProvider {
    /// 历表公式规定的时间尺度：求值时应传入该尺度的 `TimePoint`。
    fn evaluation_time_scale(&self) -> TimeScale {
        TimeScale::TDB
    }

    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6;
}

/// VSOP87 太阳（地心）。
///
/// **日心→地心平移**：VSOP87 历表给出的是**地球**在日心架下的位置与速度（`position_and_velocity_jd` = 地球）。
/// 地心太阳 = 太阳 − 地心 = 0 − 地心（日心架下原点为太阳），故对地球位置/速度取负即得地心太阳状态。
/// 该平移在此处完成，之后管线（岁差、章动等）的起点即为地心坐标。见 doc/astronomy/reference-frames.md §历表对齐到 ICRS 与地心坐标。
impl EphemerisProvider for Vsop87 {
    // VSOP87 约定：历表时间 TDB。
    fn evaluation_time_scale(&self) -> TimeScale {
        TimeScale::TDB
    }

    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6 {
        match body {
            Body::Sun => {
                // 地心太阳 = -地心（日心架），见上方 doc 注释
                let (p, v) = self.heliocentric_rectangular(epoch.jd_tdb());
                let frame = ReferenceFrame::MeanEcliptic(Epoch::j2000());
                let position = Position::from_si_meters_in_frame(frame, -p[0], -p[1], -p[2]);
                let velocity = Velocity::from_si_m_per_s_in_frame(frame, -v[0], -v[1], -v[2]);
                State6::new(position, velocity)
            }
            Body::Moon => {
                panic!("Vsop87 does not provide Moon; use Elpmpp02Data with MoonProvider")
            }
            Body::Planet(p) => panic!("单个 Vsop87（地球）不提供 {p:?}；用 Vsop87Planets"),
        }
    }
}

/// ELPMPP02 月球（地心）；公式用 TT 儒略世纪。
impl EphemerisProvider for Elpmpp02Data {
    fn evaluation_time_scale(&self) -> TimeScale {
        TimeScale::TT
    }

    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6 {
        match body {
            Body::Moon => {
                let (pos, vel) = position_velocity(self, epoch);
                State6::new(pos, vel)
            }
            Body::Sun => panic!("Elpmpp02Data does not provide Sun; use Vsop87"),
            Body::Planet(p) => panic!("Elpmpp02Data does not provide {p:?}; use Vsop87Planets"),
        }
    }
}

/// VSOP87 行星（地心）：地心行星 = 日心行星 − 日心地球，均为 VSOP87B 的 J2000 动力学黄道与春分点（与地球同一架）；
/// 各 [`Vsop87`] 若带改正（[`Vsop87::patch`]，拟合到 DE406/DE441，Chapront 2000 方法）则自动施加。
/// 光行时由 [`LightTimeCorrector`](super::LightTimeCorrector) 对该地心矢量迭代（行星在 t−τ、地球在 t−τ，即一阶含周年光行差
/// 的「行星光行差」做法，与现有太阳管线一致）。也提供 `Body::Sun`（同 [`Vsop87`]），便于同一提供者算距角。
pub struct Vsop87Planets<'a> {
    pub earth: &'a Vsop87,
    pub planets: Vec<(Planet, &'a Vsop87)>,
}

impl Vsop87Planets<'_> {
    fn planet(&self, p: Planet) -> &Vsop87 {
        self.planets.iter().find(|(q, _)| *q == p).map(|(_, v)| *v).unwrap_or_else(|| panic!("未加载 {p:?} 的 VSOP87"))
    }
}

impl EphemerisProvider for Vsop87Planets<'_> {
    fn evaluation_time_scale(&self) -> TimeScale {
        TimeScale::TDB
    }

    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6 {
        match body {
            Body::Planet(p) => {
                let jd = epoch.jd_tdb();
                let (pp, pv) = self.planet(p).heliocentric_rectangular(jd);
                let (ep, ev) = self.earth.heliocentric_rectangular(jd);
                let frame = ReferenceFrame::MeanEcliptic(Epoch::j2000());
                State6::new(
                    Position::from_si_meters_in_frame(frame, pp[0] - ep[0], pp[1] - ep[1], pp[2] - ep[2]),
                    Velocity::from_si_m_per_s_in_frame(frame, pv[0] - ev[0], pv[1] - ev[1], pv[2] - ev[2]),
                )
            }
            Body::Sun => self.earth.compute_state(Body::Sun, epoch),
            Body::Moon => panic!("Vsop87Planets does not provide Moon"),
        }
    }
}

/// VSOP87 日、行星 + ELPMPP02 月的组合提供者：同一管线下算日、月、行星（如月与行星的近距）。ELPMPP02 求值内部自换 TT，
/// 故整体按 TDB 约定传入即可。
pub struct Vsop87Elp<'a> {
    pub planets: Vsop87Planets<'a>,
    pub moon: &'a Elpmpp02Data,
}

impl EphemerisProvider for Vsop87Elp<'_> {
    fn evaluation_time_scale(&self) -> TimeScale {
        TimeScale::TDB
    }

    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6 {
        match body {
            Body::Moon => self.moon.compute_state(Body::Moon, epoch),
            _ => self.planets.compute_state(body, epoch),
        }
    }
}
