//! 太阳引力光线偏折（广义相对论一阶，PPN γ = 1）。
//!
//! 公式同 SOFA `iauLd`/`iauLdsun`（其所据：Urban & Seidelmann 编《Explanatory Supplement to the Astronomical Almanac》
//! 第 3 版 §7.2.4；Klioner 2003, AJ 125, 1580）；IERS Conventions (2010) 所列 IAU 2006 天文常数与本模块同为 PPN γ = 1 的一阶式
//! （IERS 章节号未核）：
//!
//! $$\mathbf p_1 = \mathbf p + \frac{2GM_\odot}{c^2 E}\,\frac{\mathbf p\times(\mathbf e\times\mathbf q)}{\max(1+\mathbf q\cdot\mathbf e,\ \delta)}$$
//!
//! - p：观测者 → 天体的方向（单位矢量，偏折前）；
//! - q：太阳 → 天体的方向（单位矢量；恒星取 q = p）；
//! - e：太阳 → 观测者的方向（单位矢量），E 为日地距离（AU）；
//! - δ：近合日时防止分母为 0 的下限。SOFA `iauLdsun` 取 δ = 10⁻⁶ / max(E², 1)，本模块同。
//!
//! 量级：恒星或远天体在距角 ψ 处偏折 ≈ (2GM/c²E)·cot(ψ/2) ≈ 0.00407″·cot(ψ/2)；日面边缘（ψ ≈ 0.267°）1.75″，
//! 离日 1° 处 0.47″、10° 处 0.047″、90° 处 0.004″。结果不再归一化（与 SOFA 同；|p₁|² − 1 为偏折角平方，日面边缘约 7×10⁻¹¹）。
//!
//! 天体在日面之后（被日遮掩）时公式无物理意义，但 δ 保证有限；调用方不必另判。

use crate::math::real::{real, real_const, Real};

/// 太阳 Schwarzschild 半径 2GM☉/c²，单位 AU（SOFA `ERFA_SRS` = 1.97412574336e-8；GM☉ = 1.32712440041×10²⁰ m³/s²，
/// TDB 相容值，IAU 2009 系统）。
pub const SUN_SCHWARZSCHILD_RADIUS_AU: Real = real_const(1.97412574336e-8);

fn dot(a: [Real; 3], b: [Real; 3]) -> Real {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// 单位化；零矢量原样返回。
pub fn unit(v: [Real; 3]) -> [Real; 3] {
    let n = dot(v, v).sqrt();
    if n <= real(0.0) {
        return v;
    }
    [v[0] / n, v[1] / n, v[2] / n]
}

/// 单个引力体的光线偏折（SOFA `iauLd` 同式）。`bm` 为引力体质量与太阳质量之比（太阳取 1），
/// p、q、e 为单位矢量（见模块说明），`em_au` 为引力体到观测者距离（AU），`dlim` 为分母下限。
pub fn light_deflection(bm: Real, p: [Real; 3], q: [Real; 3], e: [Real; 3], em_au: Real, dlim: Real) -> [Real; 3] {
    let qpe = [q[0] + e[0], q[1] + e[1], q[2] + e[2]];
    let qdqpe = dot(q, qpe);
    let den = if qdqpe > dlim { qdqpe } else { dlim };
    let w = bm * SUN_SCHWARZSCHILD_RADIUS_AU / em_au / den;
    let peq = cross(p, cross(e, q));
    [p[0] + w * peq[0], p[1] + w * peq[1], p[2] + w * peq[2]]
}

/// 分母下限：SOFA `iauLdsun` 的 10⁻⁶ / max(E², 1)。
fn solar_dlim(em_au: Real) -> Real {
    let em2 = em_au * em_au;
    real(1e-6) / if em2 > real(1.0) { em2 } else { real(1.0) }
}

/// 太阳对恒星（无限远天体，q = p）的光线偏折（SOFA `iauLdsun`）。p、e 为单位矢量，`em_au` 为日地距离（AU）。
pub fn solar_deflection_star(p: [Real; 3], e: [Real; 3], em_au: Real) -> [Real; 3] {
    light_deflection(real(1.0), p, p, e, em_au, solar_dlim(em_au))
}

/// 太阳对有限距离天体（行星）的光线偏折。p、q、e 为单位矢量，`em_au` 为日地距离（AU）；分母下限同恒星。
pub fn solar_deflection_body(p: [Real; 3], q: [Real; 3], e: [Real; 3], em_au: Real) -> [Real; 3] {
    light_deflection(real(1.0), p, q, e, em_au, solar_dlim(em_au))
}

/// 由（地心）位置矢量直接施太阳偏折：`body` 为观测者 → 天体（米），`sun_at_tr` 为观测者 → 太阳（推迟时 tr 的太阳，米），
/// `sun_at_t` 为观测时刻 t 的观测者 → 太阳（米）。三者须在同一惯性架（ICRS）。返回偏折后的观测者 → 天体矢量（保持原长）。
///
/// q = body − sun_at_tr（太阳 → 天体，同在 tr），e = −sun_at_t（太阳 → 观测者，在 t）。
pub fn solar_deflection_of_position(body: [Real; 3], sun_at_tr: [Real; 3], sun_at_t: [Real; 3], au_meters: Real) -> [Real; 3] {
    let r = dot(body, body).sqrt();
    let p = unit(body);
    let q = unit([body[0] - sun_at_tr[0], body[1] - sun_at_tr[1], body[2] - sun_at_tr[2]]);
    let e_vec = [-sun_at_t[0], -sun_at_t[1], -sun_at_t[2]];
    let em = dot(e_vec, e_vec).sqrt() / au_meters;
    let p1 = solar_deflection_body(p, q, unit(e_vec), em);
    [p1[0] * r, p1[1] * r, p1[2] * r]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::RealOps;

    /// pyerfa（SOFA ERFA 2.0.1）`erfa.ldsun` / `erfa.ld` 算例，由 `scripts/phenomena/sofa_light_deflection.py` 生成：
    /// (p, q, e, em, dlim, ERFA 结果 p1)。前 12 行为 `ldsun`（q = p，dlim 由 em 定，此处列出备查），后 6 行为 `ld`
    /// （有限距离天体；dlim 同 `ldsun` 规则）。
    #[allow(clippy::type_complexity)]
    const ERFA_CASES: &[([f64; 3], [f64; 3], [f64; 3], f64, f64, [f64; 3])] = include!("light_deflection_erfa_cases.in");

    fn sep_arcsec(a: [Real; 3], b: [f64; 3]) -> f64 {
        let c = [
            a[1].as_f64() * b[2] - a[2].as_f64() * b[1],
            a[2].as_f64() * b[0] - a[0].as_f64() * b[2],
            a[0].as_f64() * b[1] - a[1].as_f64() * b[0],
        ];
        (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt() * 206264.80624709636
    }

    /// 与 SOFA 逐例一致（差 < 1 μas；所列 dlim 与本模块按 em 算出的相同）。
    #[test]
    fn matches_erfa_ld_and_ldsun() {
        let mut tab = crate::test_util::CheckTable::new(
            "light_deflection_vs_erfa",
            &["算例", "函数", "距角ψ°", "E/AU", "ERFA偏折″", "本实现偏折″", "差μas"],
        );
        tab.meta("太阳引力光线偏折对 SOFA（pyerfa 2.0.1.5，erfa.ldsun / erfa.ld）逐例对照；scripts/phenomena/sofa_light_deflection.py 生成");
        let r = |v: [f64; 3]| [real(v[0]), real(v[1]), real(v[2])];
        let mut worst: f64 = 0.0;
        for (k, &(p, q, e, em, dlim, want)) in ERFA_CASES.iter().enumerate() {
            let is_star = p == q;
            assert!((solar_dlim(real(em)).as_f64() - dlim).abs() <= 1e-20, "算例 {k} dlim");
            let got = if is_star { solar_deflection_star(r(p), r(e), real(em)) } else { solar_deflection_body(r(p), r(q), r(e), real(em)) };
            let d = sep_arcsec(got, want) * 1e6;
            let pd = [real(p[0]), real(p[1]), real(p[2])];
            let defl_erfa = sep_arcsec(pd, want);
            let defl_ours = sep_arcsec(pd, [got[0].as_f64(), got[1].as_f64(), got[2].as_f64()]);
            // 距角：太阳方向为 −e
            let psi = (-(p[0] * e[0] + p[1] * e[1] + p[2] * e[2])).clamp(-1.0, 1.0).acos().to_degrees();
            worst = worst.max(d);
            tab.row(vec![
                k.to_string(),
                if is_star { "ldsun".into() } else { "ld".into() },
                format!("{psi:.4}"),
                format!("{em:.4}"),
                format!("{defl_erfa:.6}"),
                format!("{defl_ours:.6}"),
                format!("{d:.4}"),
            ]);
            assert!(d < 1.0, "算例 {k}: 差 {d} μas");
        }
        tab.write();
        println!("对 ERFA 最大差 {worst:.4} μas");
    }

    /// 量级核对：E = 1 AU，距角 ψ 处恒星偏折 ≈ 0.00407″·cot(ψ/2)（日面边缘 1.75″）。
    #[test]
    fn magnitude_matches_cot_half_angle() {
        for psi_deg in [0.267_f64, 1.0, 10.0, 45.0, 90.0, 135.0] {
            let psi = psi_deg.to_radians();
            // 太阳在 +x，星在 xy 平面距角 ψ；e = 太阳 → 地 = −x
            let p = [real(psi.cos()), real(psi.sin()), real(0.0)];
            let e = [real(-1.0), real(0.0), real(0.0)];
            let p1 = solar_deflection_star(p, e, real(1.0));
            let d = sep_arcsec(p1, [psi.cos(), psi.sin(), 0.0]);
            let want = 1.97412574336e-8 * 206264.80624709636 / (psi / 2.0).tan();
            assert!((d - want).abs() < 1e-6 * want.max(1e-3), "ψ = {psi_deg}: {d} vs {want}");
            // 偏折方向背离太阳（距角增大）
            let psi1 = (p1[0].as_f64() / (p1[0].as_f64().hypot(p1[1].as_f64()))).acos();
            assert!(psi1 > psi);
        }
    }
}
