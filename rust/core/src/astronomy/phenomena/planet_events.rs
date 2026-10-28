//! 行星会合现象：合（内行星分上合、下合）、冲、留、大距。
//!
//! 定义（《天文年历》说明书与 Meeus《天文算法》通行定义）：均用地心视位置，黄经取当日真黄道与真春分点（含章动）。
//! - 合、冲：行星与太阳视黄经之差为 0°、180°；内行星的合按地心距离分下合（行星近于太阳）、上合；
//! - 留：行星视黄经变化率为零；由顺行转逆行为「留（转逆）」，由逆行转顺行为「留（转顺）」；
//! - 大距：内行星与太阳的角距（大圆距离）极大；行星在太阳之东为东大距（昏见），之西为西大距（晨见）。
//!
//! 搜索：按步长取样找符号变化或局部极大，再二分 / 黄金分割求精（时刻约 1 s，留与大距因函数平缓，时刻约分钟级即足）。
//! 日与行星用同一提供者与同一管线（[`body_apparent_ecliptic`]），历表可为 VSOP87（`Vsop87Planets`）或 DE406/DE441。

use crate::astronomy::apparent::{body_apparent_ecliptic, body_apparent_equatorial, ApparentPipelineOptions};
use crate::astronomy::pipeline::{Body, EphemerisProvider, Planet};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};

use super::search::{bisect_root, extremum_by_derivative_root, minimize_golden, seconds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanetEventKind {
    /// 外行星合日
    Conjunction,
    /// 内行星下合（行星在日地之间）
    InferiorConjunction,
    /// 内行星上合
    SuperiorConjunction,
    /// 冲
    Opposition,
    /// 留（顺行转逆行）
    StationRetrograde,
    /// 留（逆行转顺行）
    StationDirect,
    /// 东大距（昏见）
    GreatestElongationEast,
    /// 西大距（晨见）
    GreatestElongationWest,
}

/// 一次会合现象（时刻 TT）。
#[derive(Clone, Copy, Debug)]
pub struct PlanetEvent {
    pub kind: PlanetEventKind,
    pub jd_tt: Real,
    /// 事件时刻行星与太阳的角距（度）
    pub elongation_deg: Real,
    /// 事件时刻行星视黄经（度，[0, 360)）
    pub longitude_deg: Real,
}

struct Ctx<'a, P> {
    eph: &'a P,
    planet: Planet,
    opts: ApparentPipelineOptions,
}

impl<P: EphemerisProvider> Ctx<'_, P> {
    fn t(&self, jd: Real) -> TimePoint {
        TimePoint::new(TimeScale::TT, jd)
    }

    fn lon(&self, body: Body, jd: Real) -> Real {
        body_apparent_ecliptic(self.eph, body, self.t(jd), &self.opts).0.rad()
    }

    /// λ行星 − λ太阳，归到 (−π, π]。
    fn dlon(&self, jd: Real) -> Real {
        (self.lon(Body::Planet(self.planet), jd) - self.lon(Body::Sun, jd)).wrap_to_signed_pi()
    }

    /// 行星视黄经变化率（rad/日），中心差分 h = 0.01 日。
    fn lon_rate(&self, jd: Real) -> Real {
        let h = real(0.01);
        let p = Body::Planet(self.planet);
        (self.lon(p, jd + h) - self.lon(p, jd - h)).wrap_to_signed_pi() / (real(2.0) * h)
    }

    /// 行星与太阳角距（rad）。
    fn elongation(&self, jd: Real) -> Real {
        let a = body_apparent_equatorial(self.eph, Body::Planet(self.planet), self.t(jd), &self.opts).to_meters();
        let b = body_apparent_equatorial(self.eph, Body::Sun, self.t(jd), &self.opts).to_meters();
        let cr = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
        let n = (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt();
        n.atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
    }

    fn event(&self, kind: PlanetEventKind, jd: Real) -> PlanetEvent {
        let deg = real(180.0 / core::f64::consts::PI);
        PlanetEvent {
            kind,
            jd_tt: jd,
            elongation_deg: self.elongation(jd) * deg,
            longitude_deg: self.lon(Body::Planet(self.planet), jd).wrap_to_2pi() * deg,
        }
    }

    fn distance_m(&self, body: Body, jd: Real) -> Real {
        body_apparent_ecliptic(self.eph, body, self.t(jd), &self.opts).2.meters()
    }
}

/// 取样步长（日）：须小于相邻两次同类事件间隔的一半；水星最快（会合周期 116 日、留与合相隔约 10 日），取 1 日，余 2 日。
fn step_days(planet: Planet) -> f64 {
    match planet {
        Planet::Mercury => 1.0,
        _ => 2.0,
    }
}

/// [jd_start, jd_end]（TT）内某行星的会合现象，按时刻排序。
pub fn planet_events<P: EphemerisProvider>(
    eph: &P,
    planet: Planet,
    jd_start: Real,
    jd_end: Real,
    options: &ApparentPipelineOptions,
) -> Vec<PlanetEvent> {
    let ctx = Ctx { eph, planet, opts: options.clone() };
    let step = real(step_days(planet));
    let n = ((jd_end - jd_start) / step).floor().as_f64().max(1.0) as usize;
    let ts: Vec<Real> = (0..=n).map(|k| jd_start + step * real(k as f64)).collect();
    let dl: Vec<Real> = ts.iter().map(|&t| ctx.dlon(t)).collect();
    let rate: Vec<Real> = ts.iter().map(|&t| ctx.lon_rate(t)).collect();
    let mut out = vec![];
    let zero = real(0.0);
    let half_pi = real(core::f64::consts::FRAC_PI_2);
    for k in 0..n {
        let (a, b) = (ts[k], ts[k + 1]);
        // 合：Δλ 过 0（两端都在 ±90° 内）；冲：Δλ 过 ±180°（两端都在 ±90° 外，符号由 + 跳到 −）
        let (d0, d1) = (dl[k], dl[k + 1]);
        if (d0 < zero) != (d1 < zero) {
            if d0.abs() < half_pi && d1.abs() < half_pi {
                if let Some(t) = bisect_root(|t| ctx.dlon(t), a, b, seconds(1.0)) {
                    let kind = if planet.is_inferior() {
                        if ctx.distance_m(Body::Planet(planet), t) < ctx.distance_m(Body::Sun, t) {
                            PlanetEventKind::InferiorConjunction
                        } else {
                            PlanetEventKind::SuperiorConjunction
                        }
                    } else {
                        PlanetEventKind::Conjunction
                    };
                    out.push(ctx.event(kind, t));
                }
            } else if !planet.is_inferior() && d0.abs() > half_pi && d1.abs() > half_pi {
                // sin(Δλ) 在 ±180° 处连续过零
                if let Some(t) = bisect_root(|t| ctx.dlon(t).sin(), a, b, seconds(1.0)) {
                    out.push(ctx.event(PlanetEventKind::Opposition, t));
                }
            }
        }
        // 留：视黄经变化率变号
        let (r0, r1) = (rate[k], rate[k + 1]);
        if (r0 < zero) != (r1 < zero) {
            if let Some(t) = bisect_root(|t| ctx.lon_rate(t), a, b, seconds(10.0)) {
                let kind = if r0 > zero { PlanetEventKind::StationRetrograde } else { PlanetEventKind::StationDirect };
                out.push(ctx.event(kind, t));
            }
        }
    }
    // 大距（内行星）：角距局部极大
    if planet.is_inferior() {
        let el: Vec<Real> = ts.iter().map(|&t| ctx.elongation(t)).collect();
        for k in 1..n {
            if el[k] > el[k - 1] && el[k] >= el[k + 1] {
                // 极大处平缓：以导数之根定时刻（见 search::extremum_by_derivative_root），无根时退回黄金分割
                let t = extremum_by_derivative_root(|t| ctx.elongation(t), ts[k - 1], ts[k + 1], seconds(1800.0), seconds(0.01))
                    .unwrap_or_else(|| minimize_golden(|t| -ctx.elongation(t), ts[k - 1], ts[k + 1], seconds(1.0)).0);
                let kind = if ctx.dlon(t) > zero {
                    PlanetEventKind::GreatestElongationEast
                } else {
                    PlanetEventKind::GreatestElongationWest
                };
                out.push(ctx.event(kind, t));
            }
        }
    }
    out.sort_by(|a, b| a.jd_tt.partial_cmp(&b.jd_tt).unwrap());
    out
}
