//! 近距事件：两天体（月、行星、太阳）地心视位置角距的极小（时刻与最小角距）。
//!
//! 只输出客观量；史书术语（「犯」「守」「合」「入」等）的阈值与判定留给下游（天文域不含星占或历法术语，见设计文档 §三）。
//! 角距取当日真赤道下两天体视方向的大圆距离（与黄道/赤道选取无关）；月为地心，站心（含视差，可达 1°）另见
//! [`crate::astronomy::apparent::topocentric`]。

use crate::astronomy::apparent::{body_apparent_equatorial, ApparentPipelineOptions};
use crate::astronomy::pipeline::{Body, EphemerisProvider};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};

use super::search::{extremum_by_derivative_root, minimize_golden, seconds};

/// 一次近距事件（时刻 TT）。
#[derive(Clone, Copy, Debug)]
pub struct CloseApproach {
    pub jd_tt: Real,
    /// 最小角距（度）
    pub separation_deg: Real,
}

/// 两天体地心视方向角距（度）。
pub fn angular_separation_deg<P: EphemerisProvider>(eph: &P, a: Body, b: Body, jd_tt: Real, options: &ApparentPipelineOptions) -> Real {
    let t = TimePoint::new(TimeScale::TT, jd_tt);
    let u = body_apparent_equatorial(eph, a, t, options).to_meters();
    let v = body_apparent_equatorial(eph, b, t, options).to_meters();
    let cr = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
    let n = (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt();
    n.atan2(u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) * real(180.0 / core::f64::consts::PI)
}

/// 取样步长（日）：须远小于两天体相对运动走过一个「近距窗口」的时间。有月参与取 0.2 日（月日行约 13°），否则 1 日。
fn step_days(a: Body, b: Body) -> f64 {
    if a == Body::Moon || b == Body::Moon {
        0.2
    } else {
        1.0
    }
}

/// [jd_start, jd_end]（TT）内两天体角距的各次极小，只保留最小角距 ≤ `max_separation_deg` 者，按时刻排序。
pub fn close_approaches<P: EphemerisProvider>(
    eph: &P,
    a: Body,
    b: Body,
    jd_start: Real,
    jd_end: Real,
    max_separation_deg: Real,
    options: &ApparentPipelineOptions,
) -> Vec<CloseApproach> {
    let step = real(step_days(a, b));
    let n = ((jd_end - jd_start) / step).floor().as_f64().max(2.0) as usize;
    let ts: Vec<Real> = (0..=n).map(|k| jd_start + step * real(k as f64)).collect();
    let sep: Vec<Real> = ts.iter().map(|&t| angular_separation_deg(eph, a, b, t, options)).collect();
    let mut out = vec![];
    for k in 1..n {
        if sep[k] < sep[k - 1] && sep[k] <= sep[k + 1] {
            // 极小处平缓：以导数之根定时刻（见 search::extremum_by_derivative_root），无根时退回黄金分割
            let sep = |t| angular_separation_deg(eph, a, b, t, options);
            let t = extremum_by_derivative_root(sep, ts[k - 1], ts[k + 1], seconds(1800.0), seconds(0.01))
                .unwrap_or_else(|| minimize_golden(sep, ts[k - 1], ts[k + 1], seconds(1.0)).0);
            let s = sep(t);
            if s <= max_separation_deg {
                out.push(CloseApproach { jd_tt: t, separation_deg: s });
            }
        }
    }
    out
}
