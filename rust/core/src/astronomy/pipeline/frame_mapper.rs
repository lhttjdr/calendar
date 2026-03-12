//! 非线性映射器：跨架拟合修正（文档 3.1 FrameMapper）。泛型于 R: Real，内部用 f64 计算再转回 R。
//!
//! **时间尺度**：FK5↔ICRS 为架旋转无时间；DE406 patch 公式规定入参 **TT**，调用方应传 TT 的 `epoch`。

use crate::astronomy::frame::fk5_icrs;
use crate::astronomy::time::TimePoint;
use crate::astronomy::frame::vsop87_de406_icrs_patch;
use crate::quantity::{position::Position, reference_frame::ReferenceFrame, velocity::Velocity};
use super::state::State6;

/// 将状态从一个架映射到另一个架；R 由顶层选择，本层不指定 f64。
pub trait FrameMapper {
    fn apply(&self, state: State6, epoch: TimePoint) -> State6;
}

/// FK5 → ICRS 仅做 Frame bias（无 patch）。原子操作一。
pub struct Fk5ToIcrsBias;

impl FrameMapper for Fk5ToIcrsBias {
    fn apply(&self, state: State6, _epoch: TimePoint) -> State6 {
        let (pos_m, vel_m) = state.to_meters_and_m_per_s();
        let (x_icrs, y_icrs, z_icrs) = fk5_icrs::rotate_equatorial(
            pos_m[0],
            pos_m[1],
            pos_m[2],
        );
        let (vx, vy, vz) = fk5_icrs::rotate_equatorial(
            vel_m[0],
            vel_m[1],
            vel_m[2],
        );
        let position = Position::from_si_meters_in_frame(
            ReferenceFrame::ICRS,
            x_icrs, y_icrs, z_icrs,
        );
        let velocity = Velocity::from_si_m_per_s_in_frame(
            ReferenceFrame::ICRS,
            vx, vy, vz,
        );
        State6::new(position, velocity)
    }
}

/// ICRS 地心太阳：Vsop87 拟合到 DE406 赤道（位置+速度改正）。入参、出参均为 ICRS。原子操作二。
pub struct Vsop87FitDe406Equatorial;

impl FrameMapper for Vsop87FitDe406Equatorial {
    fn apply(&self, state: State6, epoch: TimePoint) -> State6 {
        let (pos_m, vel_m) = state.to_meters_and_m_per_s();
        let pos = Position::from_si_meters_in_frame(
            ReferenceFrame::ICRS,
            pos_m[0],
            pos_m[1],
            pos_m[2],
        );
        let (pos_c, vel_c) = vsop87_de406_icrs_patch::apply_patch_velocity_to_equatorial_for_geocentric_sun(
            pos,
            [vel_m[0], vel_m[1], vel_m[2]],
            &epoch,
        );
        let velocity = Velocity::from_si_m_per_s_in_frame(
            ReferenceFrame::ICRS,
            vel_c[0],
            vel_c[1],
            vel_c[2],
        );
        State6::new(pos_c, velocity)
    }
}

/// 顺序组合两个 FrameMapper：先 first 再 second。
pub struct Compose<A, B>(pub A, pub B);

impl<A: FrameMapper, B: FrameMapper> FrameMapper for Compose<A, B> {
    fn apply(&self, state: State6, epoch: TimePoint) -> State6 {
        self.1.apply(self.0.apply(state, epoch), epoch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::constant::{AU_METERS, J2000};
    use crate::astronomy::ephemeris::load_earth_vsop87_from_repo;
    use crate::astronomy::ephemeris::spk::De406Kernel;
    use crate::astronomy::frame::vsop87_de406_ecliptic_patch;
    use crate::astronomy::pipeline::{Body, EphemerisProvider, TransformGraph, Via};
    use crate::math::real::{real, RealOps};
    use crate::astronomy::time::TimeScale;
    use crate::quantity::angle::PlaneAngle;
    use crate::quantity::epoch::Epoch;
    use std::path::Path;

    /// 图语义：赤道 patch 路径 = 最短路 MeanEcliptic→FK5→ICRS，mapper=Compose(Bias, 赤道fit)。
    fn graph_equatorial_patch() -> TransformGraph {
        TransformGraph::default_graph().with_fk5_to_icrs_mapper(|s, jd_tt| {
            let t = TimePoint::new(TimeScale::TT, jd_tt);
            Compose(Fk5ToIcrsBias, Vsop87FitDe406Equatorial).apply(s, t)
        })
    }

    /// 图语义：黄道 patch 路径 = 受限最短路 MeanEcliptic→Patch→FK5→ICRS，mapper=仅Bias（无赤道fit）。
    fn graph_ecliptic_patch(loader: &dyn crate::platform::DataLoader) -> TransformGraph {
        let ok = vsop87_de406_ecliptic_patch::try_init_de406_ecliptic_patch(
            loader,
            vsop87_de406_ecliptic_patch::DEFAULT_ECLIPTIC_PATCH_PATH,
        );
        if !ok {
            return TransformGraph::default_graph();
        }
        TransformGraph::default_graph()
            .with_fk5_to_icrs_mapper(|s, jd_tt| Fk5ToIcrsBias.apply(s, TimePoint::new(TimeScale::TT, jd_tt)))
            .with_ecliptic_patch_mapper(|s, jd_tt| {
                let epoch = Epoch::new(jd_tt);
                let tr = TimePoint::new(TimeScale::TT, jd_tt);
                let [x, y, z] = s.position.to_meters();
                let r = (x.clone() * x + y.clone() * y + z.clone() * z).sqrt();
                let r_au = (r.clone() / AU_METERS).as_f64();
                let l = y.clone().atan2(x.clone());
                let b = if r > real(0) { (z / r).asin() } else { real(0) };
                let (lp, bp, r_au_p) = vsop87_de406_ecliptic_patch::apply_patch_to_ecliptic_for_geocentric_sun(
                    PlaneAngle::from_rad(l),
                    PlaneAngle::from_rad(b),
                    r_au,
                    &tr,
                );
                let (vx, vy, vz) = (s.velocity.vx.m_per_s(), s.velocity.vy.m_per_s(), s.velocity.vz.m_per_s());
                let cp = lp.rad().cos() * bp.rad().cos();
                let sp = lp.rad().sin() * bp.rad().cos();
                let zp = bp.rad().sin();
                let r_m = real(r_au_p) * AU_METERS;
                super::State6::from_si_in_frame(
                    ReferenceFrame::MeanEclipticEclipticPatch(epoch),
                    r_m.clone() * cp,
                    r_m.clone() * sp,
                    r_m * zp,
                    vx, vy, vz,
                )
            })
    }

    /// 比较两条 patch 路径得到的 ICRS 地心太阳位置：图语义下赤道 path（最短路→ICRS+赤道fit）与
    /// 黄道 path（最短路 MeanEcliptic→Patch→FK5→ICRS，仅Bias）应在同一历元给出相近结果。
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn icrs_patch_vs_ecliptic_patch_agree() {
        let vsop = match load_earth_vsop87_from_repo() {
            Ok(v) => v,
            Err(_) => {
                println!("icrs_patch_vs_ecliptic_patch_agree: skipped (VSOP87B.ear not found)");
                return;
            }
        };
        let ok_icrs = vsop87_de406_icrs_patch::try_init_de406_patch_from_repo();
        let ok_ecl = vsop87_de406_ecliptic_patch::try_init_de406_ecliptic_patch_from_repo();
        if !ok_icrs || !ok_ecl {
            println!(
                "icrs_patch_vs_ecliptic_patch_agree: skipped (patch data not loaded: icrs={}, ecl={})",
                ok_icrs, ok_ecl
            );
            return;
        }

        let graph_a = graph_equatorial_patch();
        let graph_b = graph_ecliptic_patch(&crate::repo::default_loader());

        let jds: [f64; 3] = [
            J2000.as_f64(),
            J2000.as_f64() + 365.25,
            J2000.as_f64() + 3652.5,
        ];
        const TOLERANCE_M: f64 = 5e7;

        for jd in jds {
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let jd_tt = t.to_scale(TimeScale::TT).jd;
            let state = vsop.compute_state(Body::Sun, t);

            let pos_a = graph_a.transform_to(state.clone(), ReferenceFrame::ICRS, jd_tt).position;
            let via_patch = [Via::Node(ReferenceFrame::MeanEclipticEclipticPatch(Epoch::new(jd_tt)))];
            let pos_b = graph_b.transform_to_via(state, ReferenceFrame::ICRS, &via_patch, jd_tt).position;

            let dx = (pos_a.x.meters() - pos_b.x.meters()).as_f64();
            let dy = (pos_a.y.meters() - pos_b.y.meters()).as_f64();
            let dz = (pos_a.z.meters() - pos_b.z.meters()).as_f64();
            let dist_m = (dx * dx + dy * dy + dz * dz).sqrt();
            let dist_au = dist_m / 149_597_870_700.0;
            println!(
                "  JD {:.1}: 残差 = {:.3} km = {:.2e} AU（容差 {:.0e} m ≈ {:.4} AU）",
                jd,
                dist_m / 1000.0,
                dist_au,
                TOLERANCE_M,
                TOLERANCE_M / 149_597_870_700.0
            );
            assert!(
                dist_m <= TOLERANCE_M,
                "JD {}: 两路径 ICRS 位置差 {:.3} m（允许 {} m）",
                jd,
                dist_m,
                TOLERANCE_M
            );
        }
    }

    /// 赤道 patch 路径（VSOP→黄道→赤道→ICRS→patch）与 DE406 地心太阳 ICRS 对比。
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn equatorial_patch_path_vs_de406() {
        let base = crate::repo::repo_root();
        let vsop = match load_earth_vsop87_from_repo() {
            Ok(v) => v,
            Err(_) => {
                println!("equatorial_patch_path_vs_de406: skipped (VSOP87B.ear not found)");
                return;
            }
        };
        if !vsop87_de406_icrs_patch::try_init_de406_patch_from_repo() {
            println!("equatorial_patch_path_vs_de406: skipped (patch data not loaded)");
            return;
        }
        let bsp_path = std::env::var("DE406_BSP")
            .ok()
            .filter(|p| Path::new(p).is_file())
            .or_else(|| {
                let p = base.join(crate::repo::paths::DE406_BSP_CANDIDATES[0]);
                if p.is_file() {
                    Some(p.to_string_lossy().into_owned())
                } else {
                    base.join(crate::repo::paths::DE406_BSP_CANDIDATES[1])
                        .is_file()
                        .then(|| base.join(crate::repo::paths::DE406_BSP_CANDIDATES[1]).to_string_lossy().into_owned())
                }
            })
            .unwrap_or_else(|| base.join(crate::repo::paths::JPL_DATA_DIR).to_string_lossy().into_owned());
        if !Path::new(&bsp_path).is_file() {
            println!(
                "equatorial_patch_path_vs_de406: skipped (no DE406 BSP)，已尝试: {}",
                bsp_path
            );
            return;
        }
        let kernel = match De406Kernel::open(&bsp_path) {
            Ok(k) => k,
            Err(e) => {
                println!("equatorial_patch_path_vs_de406: skipped (open BSP: {})", e);
                return;
            }
        };

        let graph = graph_equatorial_patch();
        let jds: [f64; 3] = [
            J2000.as_f64(),
            J2000.as_f64() + 365.25,
            J2000.as_f64() + 3652.5,
        ];
        const TOLERANCE_DE406_M: f64 = 2e8;

        for jd in jds {
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let jd_tt = t.to_scale(TimeScale::TT).jd;
            let jd_tdb = t.jd_tdb().as_f64();
            let (de406_pos, _) = kernel.geocentric_sun(jd_tdb).expect("DE406 Sun");
            let state = vsop.compute_state(Body::Sun, t);
            let pos = graph.transform_to(state, ReferenceFrame::ICRS, jd_tt).position;
            let dx = pos.x.meters().as_f64() - de406_pos[0];
            let dy = pos.y.meters().as_f64() - de406_pos[1];
            let dz = pos.z.meters().as_f64() - de406_pos[2];
            let dist_m = (dx * dx + dy * dy + dz * dz).sqrt();
            println!(
                "  JD {:.1}: 赤道 patch 路径 − DE406 = {:.3} km = {:.2e} AU（容差 {:.0e} m）",
                jd,
                dist_m / 1000.0,
                dist_m / 149_597_870_700.0,
                TOLERANCE_DE406_M
            );
            assert!(
                dist_m <= TOLERANCE_DE406_M,
                "JD {}: 赤道 patch 路径与 DE406 差 {:.3} m（允许 {} m）",
                jd,
                dist_m,
                TOLERANCE_DE406_M
            );
        }
    }

    /// 黄道 patch 路径（图语义：MeanEcliptic→Patch→FK5→ICRS）与 DE406 地心太阳 ICRS 对比。
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn ecliptic_patch_path_vs_de406() {
        let base = crate::repo::repo_root();
        let vsop = match load_earth_vsop87_from_repo() {
            Ok(v) => v,
            Err(_) => {
                println!("ecliptic_patch_path_vs_de406: skipped (VSOP87B.ear not found)");
                return;
            }
        };
        if !vsop87_de406_ecliptic_patch::try_init_de406_ecliptic_patch_from_repo() {
            println!("ecliptic_patch_path_vs_de406: skipped (ecliptic patch data not loaded)");
            return;
        }
        let bsp_path = std::env::var("DE406_BSP")
            .ok()
            .filter(|p| Path::new(p).is_file())
            .or_else(|| {
                let p = base.join("data/jpl/de406/de406.bsp");
                if p.is_file() {
                    Some(p.to_string_lossy().into_owned())
                } else {
                    base.join("data/jpl/de406.bsp")
                        .is_file()
                        .then(|| base.join("data/jpl/de406.bsp").to_string_lossy().into_owned())
                }
            })
            .unwrap_or_else(|| base.join("data/jpl").to_string_lossy().into_owned());
        if !Path::new(&bsp_path).is_file() {
            println!(
                "ecliptic_patch_path_vs_de406: skipped (no DE406 BSP)，已尝试: {}",
                bsp_path
            );
            return;
        }
        let kernel = match De406Kernel::open(&bsp_path) {
            Ok(k) => k,
            Err(e) => {
                println!("ecliptic_patch_path_vs_de406: skipped (open BSP: {})", e);
                return;
            }
        };

        let graph = graph_ecliptic_patch(&crate::repo::default_loader());
        let jds: [f64; 3] = [
            J2000.as_f64(),
            J2000.as_f64() + 365.25,
            J2000.as_f64() + 3652.5,
        ];
        const TOLERANCE_DE406_M: f64 = 2e8;

        for jd in jds {
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let jd_tt = t.to_scale(TimeScale::TT).jd;
            let jd_tdb = t.jd_tdb().as_f64();
            let (de406_pos, _) = kernel.geocentric_sun(jd_tdb).expect("DE406 Sun");
            let state = vsop.compute_state(Body::Sun, t);
            let via_patch = [Via::Node(ReferenceFrame::MeanEclipticEclipticPatch(Epoch::new(jd_tt)))];
            let pos = graph.transform_to_via(state, ReferenceFrame::ICRS, &via_patch, jd_tt).position;
            let dx = pos.x.meters().as_f64() - de406_pos[0];
            let dy = pos.y.meters().as_f64() - de406_pos[1];
            let dz = pos.z.meters().as_f64() - de406_pos[2];
            let dist_m = (dx * dx + dy * dy + dz * dz).sqrt();
            println!(
                "  JD {:.1}: 黄道 patch 路径 − DE406 = {:.3} km = {:.2e} AU（容差 {:.0e} m）",
                jd,
                dist_m / 1000.0,
                dist_m / 149_597_870_700.0,
                TOLERANCE_DE406_M
            );
            assert!(
                dist_m <= TOLERANCE_DE406_M,
                "JD {}: 黄道 patch 路径与 DE406 差 {:.3} m（允许 {} m）",
                jd,
                dist_m,
                TOLERANCE_DE406_M
            );
        }
    }
}
