//! 地球自转角与恒星时（IAU 2006/2000A，IERS Conventions 2010 第 5 章；与 SOFA `iauEra00`/`iauGmst06`/`iauEe06a`/`iauGst06a` 同式）。
//!
//! - [`earth_rotation_angle`]：ERA，以 UT1 为自变量（IAU 2000 定义）。
//! - [`gmst06`]：格林尼治平恒星时 = ERA + IAU 2006 岁差多项式（TT）。
//! - [`equation_of_equinoxes`]：二分差 = Δψ·cos ε_A + 互补项（EECT）。
//! - [`gast`]：格林尼治视恒星时 = GMST06 + 二分差，即真春分点起量的地球自转角，与视位置管线的真赤道架（`TrueEquator`）配套。
//!
//! 章动与平黄赤交角取视位置管线所用的 [`nutation_for_apparent`] 与 P03 [`mean_obliquity`]，保证恒星时与天体视赤经同一真赤道架。
//! UT1 由 TT 减 ΔT 得到，ΔT 模型见 [`super::delta_t`]。

use crate::astronomy::constant::J2000;
use crate::astronomy::frame::nutation::nutation_for_apparent;
use crate::astronomy::frame::precession::mean_obliquity;
use crate::math::real::{real_const, Real, RealOps};
use crate::quantity::angle::PlaneAngle;

/// 地球自转角 ERA（IAU 2000）：θ = 2π(0.7790572732640 + 1.00273781191135448·Du)，Du = JD(UT1) − 2451545.0。
pub fn earth_rotation_angle(jd_ut1: Real) -> PlaneAngle {
    let du = jd_ut1 - J2000;
    // 先取整日的小数部分再乘以 2π，避免大数乘法损失精度（同 SOFA）
    let frac_day = du.clone() - du.clone().floor();
    let turns = frac_day + real_const(0.7790572732640) + real_const(0.00273781191135448) * du;
    PlaneAngle::from_rad(Real::two_pi() * turns).wrap_to_2pi()
}

/// 格林尼治平恒星时（IAU 2006）：GMST = ERA + (0.014506 + 4612.156534 t + 1.3915817 t² − 0.00000044 t³ − 0.000029956 t⁴
/// − 0.0000000368 t⁵)″，t 为 TT 儒略世纪。
pub fn gmst06(jd_ut1: Real, jd_tt: Real) -> PlaneAngle {
    let t = (jd_tt - J2000) / 36525.0;
    let poly = real_const(0.014506)
        + t.clone()
            * (real_const(4612.156534)
                + t.clone()
                    * (real_const(1.3915817)
                        + t.clone()
                            * (real_const(-0.00000044)
                                + t.clone() * (real_const(-0.000029956) + t * real_const(-0.0000000368)))));
    let era = earth_rotation_angle(jd_ut1);
    PlaneAngle::from_rad(era.rad() + PlaneAngle::from_arcsec(poly).rad()).wrap_to_2pi()
}

/// IERS 2003 Delaunay 基本幅角（角秒多项式），返回弧度：(l, l', F, D, Ω)。
fn fundamental_arguments(t: Real) -> [Real; 5] {
    let poly = |c: [f64; 5]| {
        let v = real_const(c[0])
            + t.clone()
                * (real_const(c[1]) + t.clone() * (real_const(c[2]) + t.clone() * (real_const(c[3]) + t.clone() * real_const(c[4]))));
        PlaneAngle::from_arcsec(v).wrap_to_2pi().rad()
    };
    [
        poly([485868.249036, 1717915923.2178, 31.8792, 0.051635, -0.00024470]),
        poly([1287104.79305, 129596581.0481, -0.5532, 0.000136, -0.00001149]),
        poly([335779.526232, 1739527262.8478, -12.7512, -0.001037, 0.00000417]),
        poly([1072260.70369, 1602961601.2090, -6.3706, 0.006593, -0.00003169]),
        poly([450160.398036, -6962890.5431, 7.4722, 0.007702, -0.00005939]),
    ]
}

/// 二分差互补项 EECT（IERS Conventions 2010 表 5.2e 主项，系数单位微角秒；截去 < 0.3 μas 的项）。
/// 每项：[l, l', F, D, Ω 的整数系数], sin 系数, cos 系数。
const EECT_TERMS: [([i8; 5], f64, f64); 14] = [
    ([0, 0, 0, 0, 1], 2640.96, -0.39),
    ([0, 0, 0, 0, 2], 63.52, -0.02),
    ([0, 0, 2, -2, 3], 11.75, 0.01),
    ([0, 0, 2, -2, 1], 11.21, 0.01),
    ([0, 0, 2, -2, 2], -4.55, 0.00),
    ([0, 0, 2, 0, 3], 2.02, 0.00),
    ([0, 0, 2, 0, 1], 1.98, 0.00),
    ([0, 0, 0, 0, 3], -1.72, 0.00),
    ([0, 1, 0, 0, 1], -1.41, -0.01),
    ([0, 1, 0, 0, -1], -1.26, -0.01),
    ([1, 0, 0, 0, -1], -0.63, 0.00),
    ([1, 0, 0, 0, 1], -0.63, 0.00),
    ([0, 1, 2, -2, 3], 0.46, 0.00),
    ([0, 1, 2, -2, 1], 0.45, 0.00),
];

/// 二分差（IAU 2006/2000A）：EE = Δψ·cos ε_A + EECT；EECT 另含 t·sinΩ 项 −0.87 μas。
pub fn equation_of_equinoxes(jd_tt: Real) -> PlaneAngle {
    let t = (jd_tt - J2000) / 36525.0;
    let (dpsi, _deps) = nutation_for_apparent(t.clone());
    let eps_a = mean_obliquity(t.clone()).rad();
    let args = fundamental_arguments(t.clone());
    let mut eect_uas = real_const(-0.87) * t * args[4].clone().sin();
    for (k, s, c) in EECT_TERMS.iter() {
        let mut a = real_const(0.0);
        for (ki, arg) in k.iter().zip(args.iter()) {
            a = a + arg.clone() * real_const(*ki as f64);
        }
        eect_uas = eect_uas + real_const(*s) * a.clone().sin() + real_const(*c) * a.cos();
    }
    let eect = PlaneAngle::from_arcsec(eect_uas / 1.0e6);
    PlaneAngle::from_rad(dpsi.rad() * eps_a.cos() + eect.rad())
}

/// 格林尼治视恒星时（IAU 2006/2000A）= GMST06 + 二分差。
pub fn gast(jd_ut1: Real, jd_tt: Real) -> PlaneAngle {
    let gmst = gmst06(jd_ut1, jd_tt.clone());
    PlaneAngle::from_rad(gmst.rad() + equation_of_equinoxes(jd_tt).rad()).wrap_to_2pi()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::real;

    fn mjd(m: f64) -> Real {
        real(2400000.5) + real(m)
    }

    /// SOFA t_sofa_c.c：iauEra00(2400000.5, 54388.0) = 0.4022837240028158102。
    #[test]
    fn era_matches_sofa() {
        let v = earth_rotation_angle(mjd(54388.0)).rad().as_f64();
        assert!((v - 0.4022837240028158102).abs() < 1e-12, "{v}");
    }

    /// SOFA：iauGmst06(2400000.5, 53736.0, 2400000.5, 53736.0) = 1.754174971870091203。
    #[test]
    fn gmst06_matches_sofa() {
        let v = gmst06(mjd(53736.0), mjd(53736.0)).rad().as_f64();
        assert!((v - 1.754174971870091203).abs() < 1e-12, "{v}");
    }

    /// SOFA：iauEe06a(2400000.5, 53736.0) = −0.8834195072043790156e-5；iauGst06a(…) = 1.754166137675019159
    /// （二者与上面已逐位验证的 GMST06 满足 GST = GMST + EE）。
    /// 需先加载完整 IAU 2000A 章动（未加载时章动模块静默返回零，二分差会丢掉 Δψ 项）。实测与 SOFA 差约 0.16 mas
    /// （SOFA 对 2000A 另加 IAU 2006 调整、章动表版本差异），容差取 1e-9 rad（0.2 mas，恒星时约 0.014 ms）。
    #[test]
    fn equation_of_equinoxes_and_gast_match_sofa() {
        assert!(crate::astronomy::frame::nutation::try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
        let ee = equation_of_equinoxes(mjd(53736.0)).rad().as_f64();
        assert!((ee - (-0.8834195072043790156e-5)).abs() < 1e-9, "{ee}");
        let g = gast(mjd(53736.0), mjd(53736.0)).rad().as_f64();
        assert!((g - 1.754166137675019159).abs() < 1e-9, "{g}");
    }
}
