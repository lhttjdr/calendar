//! 数据提供层：EphemerisProvider（文档 3.1）。泛型于 R: Real，不在本层指定 f64。

use crate::astronomy::time::TimePoint;
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
pub trait EphemerisProvider {
    fn compute_state(&self, body: Body, epoch: TimePoint) -> State6;
}

/// VSOP87 太阳（地心）。
impl EphemerisProvider for Vsop87 {
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

/// ELPMPP02 月球（地心）。
impl EphemerisProvider for Elpmpp02Data {
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
