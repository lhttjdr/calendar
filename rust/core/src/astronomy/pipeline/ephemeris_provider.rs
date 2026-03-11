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
/// 该平移在此处完成，之后管线（岁差、章动等）的起点即为地心坐标。见 doc/2-reference-frames.md §历表对齐到 ICRS 与地心坐标。
impl EphemerisProvider for Vsop87 {
    // VSOP87 约定：历表时间 TDB。
    fn evaluation_time_scale(&self) -> TimeScale {
        TimeScale::TDB
    }

    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6 {
        match body {
            Body::Sun => {
                let (pos, vel) = self.position_and_velocity_jd(epoch.jd_tdb());
                let l = pos.L.rad();
                let b = pos.B.rad();
                let au_m = crate::astronomy::constant::AU_METERS;
                let r_au = pos.R.meters() / au_m;
                let (cl, sl) = (l.cos(), l.sin());
                let (cb, sb) = (b.cos(), b.sin());
                let x_earth = r_au * cb * cl;
                let y_earth = r_au * cb * sl;
                let z_earth = r_au * sb;
                let dl = vel.d_l.rad_per_day();
                let db = vel.d_b.rad_per_day();
                let dr = vel.d_r.au_per_day(au_m);
                let day_s = 86400.0;
                let vxe = (dr * cb * cl - r_au * sb * cl * db - r_au * cb * sl * dl) * au_m / day_s;
                let vye = (dr * cb * sl - r_au * sb * sl * db + r_au * cb * cl * dl) * au_m / day_s;
                let vze = (dr * sb + r_au * cb * db) * au_m / day_s;
                let frame = ReferenceFrame::MeanEcliptic(Epoch::j2000());
                // 地心太阳 = -地心（日心架），见上方 doc 注释
                let position = Position::from_si_meters_in_frame(
                    frame,
                    -x_earth * au_m,
                    -y_earth * au_m,
                    -z_earth * au_m,
                );
                let velocity = Velocity::from_si_m_per_s_in_frame(frame, -vxe, -vye, -vze);
                State6::new(position, velocity)
            }
            Body::Moon => {
                panic!("Vsop87 does not provide Moon; use Elpmpp02Data with MoonProvider")
            }
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
        }
    }
}
