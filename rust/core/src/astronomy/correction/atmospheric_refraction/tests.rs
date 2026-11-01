use super::*;
use crate::math::angle::deg2rad;
use crate::math::real::{real, RealOps};
use crate::math::series::arcsec_to_rad;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::length::Length;
use crate::quantity::unit::LengthUnit;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

const PI: f64 = core::f64::consts::PI;

fn std_pressure() -> Pressure {
    Pressure::from_kpa(real(101.0))
}
fn std_temperature() -> ThermodynamicTemperature {
    ThermodynamicTemperature::from_celsius(real(10.0))
}

#[test]
fn bennett_positive_altitude() {
    let alt = PlaneAngle::from_rad(deg2rad(30.0));
    let r = bennett_refraction_default(alt);
    assert!(r.rad() > real(0) && r.rad() < deg2rad(1.0));
}

#[test]
fn bennett_at_90_deg_abs_r_leq_0_9_arcsec() {
    let alt = PlaneAngle::parse("90°").unwrap();
    let r = bennett_refraction_default(alt);
    let bound_rad = arcsec_to_rad(0.9);
    assert!(r.rad().abs() <= bound_rad);
}

#[test]
fn bennett_improved_at_90_deg_abs_r_leq_0_9_arcsec() {
    let alt = PlaneAngle::parse("90°").unwrap();
    let r = bennett_improved_refraction_default(alt);
    let bound_rad = arcsec_to_rad(0.9);
    assert!(r.rad().abs() <= bound_rad);
}

#[test]
fn xjw_positive_altitude() {
    let alt = PlaneAngle::from_rad(deg2rad(30.0));
    let r = xjw_refraction(alt);
    assert!(r.rad() > real(0) && r.rad() < deg2rad(1.0));
}

fn assert_near_rad(a: crate::math::real::Real, b: crate::math::real::Real, eps_rad: f64) {
    assert!(a.is_near(b, eps_rad), "{:?} vs {:?} (eps {})", a, b, eps_rad);
}

#[test]
fn saemundsson_consistent_with_bennett_within_0_1_arcmin_at_23_deg() {
    let ha = PlaneAngle::parse("23°").unwrap();
    let r1 = bennett_refraction_default(ha);
    let h = PlaneAngle::from_rad(ha.rad() - r1.rad());
    let r2 = saemundsson_refraction(h, std_pressure(), std_temperature());
    let ha1 = h.rad() + r2.rad();
    let eps = 0.1 * (PI / 180.0 / 60.0);
    assert_near_rad(ha.rad(), ha1, eps);
}

#[test]
fn saemundsson_consistent_with_bennett_within_0_1_arcmin_dms() {
    let ha = PlaneAngle::parse("56°34'23\".5").unwrap();
    let r1 = bennett_refraction_default(ha);
    let h = PlaneAngle::from_rad(ha.rad() - r1.rad());
    let r2 = saemundsson_refraction(h, std_pressure(), std_temperature());
    let ha1 = h.rad() + r2.rad();
    let eps = 0.1 * (PI / 180.0 / 60.0);
    assert_near_rad(ha.rad(), ha1, eps);
}

#[test]
fn smart_vs_bennett_within_1_9_arcsec() {
    let eps_rad = arcsec_to_rad(1.9).as_f64();
    for s in &["45°", "75°", "95°"] {
        let ha = PlaneAngle::parse(s).unwrap();
        let b = bennett_refraction_default(ha);
        let sm = smart_refraction(ha, std_pressure(), std_temperature());
        assert_near_rad(b.rad(), sm.rad(), eps_rad);
    }
}

#[test]
fn meeus_consistent_with_smart_within_0_2_arcsec() {
    let ha = PlaneAngle::parse("56°34'23\".5").unwrap();
    let r1 = smart_refraction(ha, std_pressure(), std_temperature());
    let h = PlaneAngle::from_rad(ha.rad() - r1.rad());
    let r2 = meeus_refraction(h, std_pressure(), std_temperature());
    let ha1 = h.rad() + r2.rad();
    let eps_rad = arcsec_to_rad(0.2).as_f64();
    assert_near_rad(ha.rad(), ha1, eps_rad);
}

fn deg(d: f64) -> PlaneAngle {
    PlaneAngle::from_rad(deg2rad(d))
}

fn r_arcmin(h_deg: f64, m: RefractionModel) -> f64 {
    refraction_from_geometric(deg(h_deg), m, &Atmosphere::default()).rad().as_f64().to_degrees() * 60.0
}

/// 视高度式由真高度求得的折射，代回「视高度 = 真高度 + 折射」须自洽。
#[test]
fn apparent_basis_models_are_inverted() {
    for h in [0.0, 0.5, 3.0, 10.0, 45.0, 80.0] {
        let b = r_arcmin(h, RefractionModel::Bennett);
        let direct = bennett_refraction_default(deg(h + b / 60.0)).rad().as_f64().to_degrees() * 60.0;
        assert!((b - direct).abs() < 1e-6, "h={h}：{b}′ vs {direct}′");
    }
    for h in [20.0, 45.0, 80.0] {
        let s = r_arcmin(h, RefractionModel::Smart);
        let direct = smart_refraction(deg(h + s / 60.0), Pressure::from_kpa(real(101.0)), ThermodynamicTemperature::from_celsius(real(10.0)))
            .rad().as_f64().to_degrees() * 60.0;
        assert!((s - direct).abs() < 1e-6, "h={h}：{s}′ vs {direct}′");
    }
}

/// 视高度 0 处 Bennett 折射 34.5′：真高度约 −34.5′ 的天体应正好在视地平上。
#[test]
fn object_below_geometric_horizon_is_lifted_to_apparent_horizon() {
    let (r0, cot) = (bennett::bennett_arcmin(real(0.0)).as_f64(), r_arcmin(-34.478 / 60.0, RefractionModel::Bennett));
    assert!((r0 - 34.478).abs() < 0.01, "视高度 0 折射 {r0}′");
    // 缺省 10°C = 283.15 K，公式参考 283 K，因子 0.9995，差约 0.016′
    assert!((cot - r0).abs() < 0.02, "{cot}′ vs {r0}′");
}

/// 过地平无跳变（旧公式在 0° 处从 34′ 跌到 0），视高度随真高度单调，−2° 以下为零。
#[test]
fn continuous_and_monotonic_through_horizon() {
    for m in [RefractionModel::Bennett, RefractionModel::BennettImproved, RefractionModel::Saemundsson] {
        let mut last = f64::NEG_INFINITY;
        let mut last_r = r_arcmin(-3.0, m);
        assert_eq!(last_r, 0.0);
        for i in 0..=9300 {
            let h = -3.0 + i as f64 * 0.01;
            let r = r_arcmin(h, m);
            let app = h + r / 60.0;
            assert!(app > last, "{m:?} h={h}：视高度不单调");
            assert!((r - last_r).abs() < 1.0, "{m:?} h={h}：折射跳变 {last_r}′ → {r}′");
            assert!(r >= 0.0);
            last = app;
            last_r = r;
        }
    }
}

#[test]
fn smart_and_meeus_agree_above_their_limit() {
    for h in [15.0, 30.0, 60.0] {
        let (s, m) = (r_arcmin(h, RefractionModel::Smart), r_arcmin(h, RefractionModel::Meeus));
        assert!((s - m).abs() < 0.05, "h={h}：Smart {s}′ vs Meeus {m}′");
    }
}

#[test]
#[should_panic(expected = "只适用于高度")]
fn polynomial_model_rejects_low_altitude() {
    r_arcmin(5.0, RefractionModel::Smart);
}

/// XJW 系数即 Saemundsson 式换算到弧度。
#[test]
fn xjw_is_saemundsson_at_reference_atmosphere() {
    for h in [0.5, 5.0, 30.0, 80.0] {
        let (x, s) = (xjw_refraction(deg(h)).rad().as_f64().to_degrees() * 60.0, r_arcmin(h, RefractionModel::Saemundsson));
        // 缺省大气因子 0.9995（见上），h = 0.5° 处差约 0.013′
        assert!((x - s).abs() < 0.02, "h={h}：XJW {x}′ vs Saemundsson {s}′");
    }
}

#[test]
fn pressure_temperature_scale_refraction() {
    let thin = Atmosphere { pressure: Pressure::from_kpa(real(50.5)), temperature: ThermodynamicTemperature::from_celsius(real(10.0)), ..Atmosphere::default() };
    let m = RefractionModel::Saemundsson;
    let half = refraction_from_geometric(deg(20.0), m, &thin).rad().as_f64().to_degrees() * 60.0;
    assert!((half - r_arcmin(20.0, m) / 2.0).abs() < 1e-9);
}

#[test]
fn refract_enu_keeps_azimuth_and_distance() {
    let v = [real(3.0e8), real(4.0e8), real(1.0e8)];
    let out = refract_enu(v, RefractionModel::Bennett, &Atmosphere::default());
    let norm = |a: [crate::math::real::Real; 3]| (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().as_f64();
    assert!((norm(out) - norm(v)).abs() < 1e-4);
    assert!((out[0].atan2(out[1]) - v[0].atan2(v[1])).abs().as_f64() < 1e-12);
    assert!(out[2].as_f64() > v[2].as_f64());
}

/// 各模型逐高度对照，写入 doc/astronomy/celestial-phenomena.md。`cargo test ... refraction_model_table -- --nocapture`。
#[test]
fn refraction_model_table() {
    for h in [-0.5, 0.0, 1.0, 2.0, 5.0, 10.0, 15.0, 20.0, 30.0, 45.0, 60.0, 80.0] {
        let cell = |m: RefractionModel| {
            if m.min_altitude_deg().is_some_and(|x| h < x) { "—".to_string() } else { format!("{:.2}", r_arcmin(h, m)) }
        };
        let row: Vec<String> = RefractionModel::ALL.into_iter().map(cell).collect();
        println!("| {h} | {} |", row.join(" | "));
    }
    let (b, s) = (r_arcmin(0.0, RefractionModel::Bennett), r_arcmin(0.0, RefractionModel::Saemundsson));
    assert!((b - s).abs() < 0.5, "真高度 0：Bennett {b}′ vs Saemundsson {s}′");
}

fn mmhg(p: f64, t_c: f64, h_m: f64, lapse: f64) -> Atmosphere {
    Atmosphere {
        pressure: Pressure::from_pa(real(p * 101_325.0 / 760.0)),
        temperature: ThermodynamicTemperature::from_celsius(real(t_c)),
        height: Length::from_value(real(h_m), LengthUnit::Meter),
        lapse_k_per_m: real(lapse),
        inversion: None,
    }
}

fn trace_arcsec(z_deg: f64, a: &Atmosphere, c: &RayTraceConstants) -> f64 {
    ray_trace_refraction_apparent(deg2rad(90.0 - z_deg), a, c).unwrap().as_f64().to_degrees() * 3600.0
}

/// Garfinkel 1967 表 IV（760 mmHg、0°C、−6.5 K/km）与表 VI（762 mmHg、26.67°C）：后者是他与数值积分的对照。
#[test]
fn ray_trace_matches_garfinkel_sea_level() {
    let c = RayTraceConstants::default();
    let iv = [(85.0, 614.0), (86.0, 732.0), (87.0, 898.0), (88.0, 1142.0), (89.0, 1524.0), (90.0, 2163.0)];
    let a = mmhg(760.0, 0.0, 0.0, -0.0065);
    for (z, want) in iv {
        let got = trace_arcsec(z, &a, &c);
        assert!((got - want).abs() < 1.0, "表 IV z={z}：{got}″ vs {want}″");
    }
    let vi = [(85.0, 555.0), (86.0, 659.0), (87.0, 805.0), (88.0, 1015.0), (89.0, 1337.0), (90.0, 1859.0)];
    let a = mmhg(762.0, 26.67, 0.0, -0.0065);
    for (z, want) in vi {
        let got = trace_arcsec(z, &a, &c);
        assert!((got - want).abs() < 2.5, "表 VI z={z}：{got}″ vs {want}″");
    }
}

/// Garfinkel 1967 表 III、V（1944 旧常数）：高海拔观测站，含地平以下的掠射光线（下行再上行）。
#[test]
fn ray_trace_matches_garfinkel_grazing_at_height() {
    let c = RayTraceConstants {
        refractivity: real(2.9429e-4),
        gravity: real(9.81),
        gas_constant: real(287.0),
        earth_radius_m: real(6_378_400.0),
        // 表中掠射光线恰好切海平面，z 只有 4 位小数，不让舍入判成撞地
        ground_m: real(-100.0),
        ..RayTraceConstants::default()
    };
    let rows = [
        (0.0, 90.0, 760.0, 0.0, 2206.4),
        (1000.0, 90.9142, 669.687, -5.696, 2980.0),
        (2000.0, 91.2990, 588.404, -11.390, 3269.0),
        (3000.0, 91.5977, 515.746, -17.083, 3470.0),
        (4000.0, 91.8530, 450.653, -22.773, 3627.0),
        (5000.0, 92.0798, 392.570, -28.462, 3751.8),
        (5000.0, 87.9202, 392.570, -28.462, 661.0),
        (6000.0, 92.2869, 340.879, -34.149, 3853.0),
        (7000.0, 92.4790, 295.003, -39.834, 3938.0),
        (8000.0, 92.6592, 254.443, -45.517, 4011.0),
        (9000.0, 92.8295, 218.583, -51.199, 4070.0),
        (10000.0, 92.9916, 187.073, -56.880, 4122.0),
    ];
    for (h, z, p, t, want) in rows {
        let got = trace_arcsec(z, &mmhg(p, t, h, -0.005694), &c);
        assert!((got - want).abs() / want < 0.002, "h={h} z={z}：{got}″ vs {want}″");
    }
}

/// 高度较大时折射 ≈ (n−1)(1 − H/r)·tan z，H/r 约 0.13%；z = 0 为零。
#[test]
fn ray_trace_limits() {
    let c = RayTraceConstants::default();
    let a = mmhg(760.0, 0.0, 0.0, -0.0065);
    assert!(trace_arcsec(0.0, &a, &c).abs() < 1e-9);
    let got = trace_arcsec(45.0, &a, &c);
    let leading = 2.9241e-4_f64.to_degrees() * 3600.0;
    assert!((got / leading - 0.9987).abs() < 0.002, "{got}″ vs {leading}″");
}

/// 由视高度到真高度再回来须自洽，地平两侧连续。
#[test]
fn ray_trace_geometric_inverts_and_is_continuous() {
    let a = Atmosphere::default();
    let c = RayTraceConstants::default();
    for h in [-0.4, 0.0, 0.5, 3.0, 10.0, 45.0, 80.0] {
        let r = r_arcmin(h, RefractionModel::RayTrace) / 60.0;
        let direct = trace_arcsec(90.0 - (h + r), &a, &c) / 60.0;
        assert!((r * 60.0 - direct).abs() < 1e-3, "h={h}：{}′ vs {direct}′", r * 60.0);
    }
    // 海平面站掠地：真高度约 −0.564°；其下 1° 线性收到零
    let (above, below) = (r_arcmin(-0.5636, RefractionModel::RayTrace), r_arcmin(-0.5645, RefractionModel::RayTrace));
    assert!((above - below).abs() < 0.2, "{above}′ vs {below}′");
    assert_eq!(r_arcmin(-1.6, RefractionModel::RayTrace), 0.0);
}

/// 海拔越高地面折射越小，且高处站可见地平之下：掠地前不撞地、撞地返回 None。
#[test]
fn ray_trace_with_observer_height() {
    let c = RayTraceConstants::default();
    let high = Atmosphere::standard_at_height(Length::from_value(real(4000.0), LengthUnit::Meter));
    let sea = Atmosphere::default();
    assert!(high.pressure.kpa().as_f64() < 62.0 && high.pressure.kpa().as_f64() > 60.0);
    assert!(trace_arcsec(80.0, &high, &c) < trace_arcsec(80.0, &sea, &c));
    assert!(ray_trace_refraction_apparent(deg2rad(-1.0), &high, &c).is_some());
    assert!(ray_trace_refraction_apparent(deg2rad(-3.0), &high, &c).is_none());
    assert!(ray_trace_refraction_apparent(deg2rad(-0.1), &sea, &c).is_none());
}

/// 逆温主要抬高地平处折射，高处几乎不变（Schaefer & Liller 1990 §2.3）。过强的逆温成波导，拒绝计算。
#[test]
fn ray_trace_inversion() {
    let c = RayTraceConstants::default();
    let plain = Atmosphere::default();
    let inv = plain.with_inversion(Length::from_value(real(100.0), LengthUnit::Meter), real(0.05));
    let (r0, r0i) = (trace_arcsec(90.0, &plain, &c), trace_arcsec(90.0, &inv, &c));
    assert!(r0i > r0 * 1.2, "{r0}″ → {r0i}″");
    let (r5, r5i) = (trace_arcsec(85.0, &plain, &c), trace_arcsec(85.0, &inv, &c));
    assert!((r5i / r5 - 1.0).abs() < 0.01, "{r5}″ → {r5i}″");
    let duct = plain.with_inversion(Length::from_value(real(50.0), LengthUnit::Meter), real(0.2));
    assert!(std::panic::catch_unwind(|| ray_trace_refraction_apparent(deg2rad(1.0), &duct, &c)).is_err());
}

/// 积分足够细：若段数不足，折射曲线的二阶差分会出现锯齿。
#[test]
fn ray_trace_quadrature_converges() {
    let c = RayTraceConstants::default();
    let a = Atmosphere::default();
    let vals: Vec<f64> = (0..=40).map(|i| trace_arcsec(90.0 - i as f64 * 0.05, &a, &c)).collect();
    for w in vals.windows(3) {
        let second = w[0] - 2.0 * w[1] + w[2];
        assert!((0.0..3.0).contains(&second), "折射曲线应平滑下凸：{w:?}");
    }
}
