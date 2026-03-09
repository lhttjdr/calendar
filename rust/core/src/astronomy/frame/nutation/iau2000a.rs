//! IAU 2000A 完整章动（从 data/IAU2000/tab5.3a 加载月日项）。标量统一 Real。

use crate::astronomy::frame::nutation::{fundamental_arguments, table_parser};
use crate::math::real::{real, zero, RealOps, ToReal};
use crate::math::series::arcsec_to_rad;
use crate::quantity::angle::PlaneAngle;

/// 单行四元组：ψ, ψ率, ε, ε率
type Quad = [table_parser::ParsedTerm; 4];

/// 从 tab5.3a 解析结果构建的完整月日章动（SOFA Δε 序）。
pub struct Iau2000a {
    terms: Vec<Quad>,
}

impl Iau2000a {
    /// 从 load_tab53a 得到的四元组列表构建。ε 第 78 项起取反。
    pub fn from_quads(quads: Vec<Quad>) -> Self {
        let terms = quads
            .into_iter()
            .enumerate()
            .map(|(i, mut q)| {
                if i >= 78 {
                    q[2].1 = (-q[2].1.0, -q[2].1.1);
                    q[3].1 = (-q[3].1.0, -q[3].1.1);
                }
                q
            })
            .collect();
        Self { terms }
    }

    /// 月日项数量（用于诊断）
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// 章动 (Δψ, Δε)；t 儒略世纪。行星项固定 -0.135″、+0.388″。弧度。全程 Real。
    pub fn nutation(&self, t: impl ToReal) -> (PlaneAngle, PlaneAngle) {
        let t_r = real(t);
        let f = fundamental_arguments(t_r);
        let mut dpsi_arcsec = zero();
        let mut deps_arcsec = zero();
        for q in &self.terms {
            let c: &[i32] = &q[0].0;
            if c.len() < 5 {
                continue;
            }
            let theta = (0..5)
                .map(|i| real(c[i]) * f[i].rad())
                .fold(zero(), |a, b| a + b)
                .wrap_to_2pi();
            let (s, c_th) = (theta.sin(), theta.cos());
            let (psi_in, psi_out) = q[0].1;
            let (d_psi_in, d_psi_out) = q[1].1;
            let (eps_in, eps_out) = q[2].1;
            let (d_eps_in, d_eps_out) = q[3].1;
            dpsi_arcsec = dpsi_arcsec
                + (real(psi_in) + real(d_psi_in) * t_r) * s
                + (real(psi_out) + real(d_psi_out) * t_r) * c_th;
            deps_arcsec = deps_arcsec
                + (real(eps_in) + real(d_eps_in) * t_r) * s
                + (real(eps_out) + real(d_eps_out) * t_r) * c_th;
        }
        dpsi_arcsec = dpsi_arcsec + real(-0.135e-3);
        deps_arcsec = deps_arcsec + real(0.388e-3);
        (
            PlaneAngle::from_rad(arcsec_to_rad(dpsi_arcsec)),
            PlaneAngle::from_rad(arcsec_to_rad(deps_arcsec)),
        )
    }
}
