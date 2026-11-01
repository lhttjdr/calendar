//! 折射的逐条校验：写 data/phenomena/checks/refraction_*.csv（PHENOMENA_CHECKS=1 时）。

use super::*;
use crate::math::angle::deg2rad;
use crate::math::real::{real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::length::Length;
use crate::quantity::unit::LengthUnit;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};
use crate::test_util::CheckTable;

fn atm(p_mmhg: f64, t_c: f64, h_m: f64, lapse: f64) -> Atmosphere {
    Atmosphere {
        pressure: Pressure::from_pa(real(p_mmhg * 101_325.0 / 760.0)),
        temperature: ThermodynamicTemperature::from_celsius(real(t_c)),
        height: Length::from_value(real(h_m), LengthUnit::Meter),
        lapse_k_per_m: real(lapse),
        inversion: None,
    }
}

fn trace(h_app_deg: f64, a: &Atmosphere, c: &RayTraceConstants) -> f64 {
    ray_trace_refraction_apparent(deg2rad(h_app_deg), a, c).unwrap().as_f64().to_degrees() * 3600.0
}

/// 拟合式在视高度 h 的折射 [″]：真高度式解 h_t + R(h_t) = h。
fn formula(h_app_deg: f64, model: RefractionModel, a: &Atmosphere) -> f64 {
    let mut h_t = h_app_deg;
    let mut r = 0.0;
    for _ in 0..60 {
        r = refraction_from_geometric(PlaneAngle::from_rad(deg2rad(h_t)), model, a).rad().as_f64().to_degrees();
        h_t = h_app_deg - r;
    }
    r * 3600.0
}

const FITS: [(RefractionModel, &str); 3] = [(RefractionModel::Bennett, "Bennett"), (RefractionModel::BennettImproved, "Bennett 改进"), (RefractionModel::Saemundsson, "Saemundsson")];

fn rms(v: &[f64]) -> f64 {
    (v.iter().map(|x| x * x).sum::<f64>() / v.len() as f64).sqrt()
}

/// Garfinkel 1967 表 III、IV、V、VI 逐值对照：本实现的光线追迹与他的理论（表 VI 同时是数值积分）。
#[test]
fn refraction_ray_trace_vs_garfinkel() {
    let revised = RayTraceConstants::default();
    let old = RayTraceConstants { refractivity: real(2.9429e-4), gravity: real(9.81), gas_constant: real(287.0), earth_radius_m: real(6_378_400.0), ground_m: real(-100.0), ..revised };
    let mut tab = CheckTable::new("refraction_ray_trace_vs_garfinkel", &["表", "海拔 m", "气压 mmHg", "气温 °C", "递减率 K/km", "天顶距 °", "Garfinkel ″", "光线追迹 ″", "差 ″", "差 %"]);
    tab.meta("Garfinkel 1967（AJ 72, 235）折射表对本实现的分层大气光线追迹；表 IV、VI 用修订常数，表 III、V 用 1944 旧常数");
    let mut rows: Vec<(&str, f64, f64, f64, f64, f64, f64, &RayTraceConstants)> = vec![];
    for (z, r) in [(85.0, 614.0), (86.0, 732.0), (87.0, 898.0), (88.0, 1142.0), (89.0, 1524.0), (90.0, 2163.0)] {
        rows.push(("IV", 0.0, 760.0, 0.0, -6.5, z, r, &revised));
    }
    for (z, r) in [(85.0, 555.0), (86.0, 659.0), (87.0, 805.0), (88.0, 1015.0), (89.0, 1337.0), (90.0, 1859.0)] {
        rows.push(("VI", 0.0, 762.0, 26.67, -6.5, z, r, &revised));
    }
    for (h, z, p, t, r) in [
        (5000.0, 87.9202, 392.570, -28.462, 661.0),
        (5000.0, 92.0798, 392.570, -28.462, 3751.8),
        (0.0, 90.0, 760.0, 0.0, 2206.4),
    ] {
        rows.push(("III", h, p, t, -5.694, z, r, &old));
    }
    for (h, z, p, t, r) in [
        (1000.0, 90.9142, 669.687, -5.696, 2980.0),
        (2000.0, 91.2990, 588.404, -11.390, 3269.0),
        (3000.0, 91.5977, 515.746, -17.083, 3470.0),
        (4000.0, 91.8530, 450.653, -22.773, 3627.0),
        (6000.0, 92.2869, 340.879, -34.149, 3853.0),
        (7000.0, 92.4790, 295.003, -39.834, 3938.0),
        (8000.0, 92.6592, 254.443, -45.517, 4011.0),
        (9000.0, 92.8295, 218.583, -51.199, 4070.0),
        (10000.0, 92.9916, 187.073, -56.880, 4122.0),
    ] {
        rows.push(("V", h, p, t, -5.694, z, r, &old));
    }
    for (name, h, p, t, l, z, want, c) in rows {
        let got = trace(90.0 - z, &atm(p, t, h, l / 1000.0), c);
        let d = got - want;
        tab.row(vec![name.into(), format!("{h:.0}"), format!("{p:.3}"), format!("{t:.3}"), format!("{l:.3}"), format!("{z:.4}"), format!("{want:.1}"), format!("{got:.1}"), format!("{d:+.1}"), format!("{:+.2}", d / want * 100.0)]);
        assert!(d.abs() / want < 0.0025, "表 {name} h={h} z={z}：{got} vs {want}");
    }
    tab.write();
}

/// 各拟合式对光线追迹：三种气温，看公式形状与 p/T 因子。
#[test]
fn refraction_models_vs_ray_trace() {
    let c = RayTraceConstants::default();
    let mut header: Vec<&'static str> = vec!["气温 °C", "视高度 °", "光线追迹 ″"];
    for (_, n) in FITS {
        header.push(Box::leak(format!("{n}−追迹 ″").into_boxed_str()));
    }
    let mut tab = CheckTable::new("refraction_models_vs_ray_trace", &header);
    tab.meta("101 kPa、−6.5 K/km；入参统一为视高度。追迹已对 Garfinkel 表验证（refraction_ray_trace_vs_garfinkel）");
    for t in [-20.0, 0.0, 10.0, 30.0] {
        let a = Atmosphere { temperature: ThermodynamicTemperature::from_celsius(real(t)), ..Atmosphere::default() };
        let mut errs: Vec<Vec<f64>> = vec![vec![]; FITS.len()];
        let mut errs5: Vec<Vec<f64>> = vec![vec![]; FITS.len()];
        for h in [0.0, 0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 20.0, 30.0, 45.0, 60.0, 75.0, 85.0] {
            let r = trace(h, &a, &c);
            let mut cells = vec![format!("{t:.0}"), format!("{h}"), format!("{r:.1}")];
            for (i, (m, _)) in FITS.iter().enumerate() {
                let e = formula(h, *m, &a) - r;
                errs[i].push(e);
                if h >= 5.0 {
                    errs5[i].push(e);
                }
                cells.push(format!("{e:+.1}"));
            }
            tab.row(cells);
        }
        let line: Vec<String> = FITS.iter().enumerate().map(|(i, (_, n))| format!("{n}: 全段 {:.1}″、≥5° {:.1}″", rms(&errs[i]), rms(&errs5[i]))).collect();
        println!("{t}°C rms {}", line.join("；"));
    }
    tab.write();
}

struct Timing {
    no: String,
    site: String,
    alt: f64,
    date: String,
    note: String,
    z: f64,
    r0: f64,
}

fn timings() -> Vec<Timing> {
    let text = std::fs::read_to_string(crate::test_util::base().join("data/phenomena/schaefer_liller1990_horizon.csv")).unwrap();
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            Timing { no: f[0].into(), site: f[1].into(), alt: f[4].parse().unwrap(), date: f[5].into(), note: f[7].into(), z: f[8].parse().unwrap(), r0: f[9].parse().unwrap() }
        })
        .collect()
}

/// Schaefer & Liller 1990 表 1 的 116 次目视海地平计时：观测站标准大气下预测「地平处的未折射天顶距 Z」。
/// 几何法为原文的归算：Z = 90° + 几何俯角 + 海平面地平折射；追迹法让光线从观测站下到海平面再出大气，俯角与折射一起算。
#[test]
fn refraction_horizon_schaefer_liller1990() {
    let rows = timings();
    assert_eq!(rows.len(), 116);
    let c = RayTraceConstants::default();
    let sea = Atmosphere::default();
    let bennett_r0 = formula(0.0, RefractionModel::Bennett, &sea) / 3600.0;
    let trace_r0 = trace(0.0, &sea, &c) / 3600.0;
    let mut tab = CheckTable::new("refraction_horizon_schaefer_liller1990", &["序", "地点", "海拔 m", "日期", "注", "实测 Z °", "原文 R0 °", "几何俯角+Bennett Z °", "差 °", "追迹 Z °", "差 °"]);
    tab.meta("Schaefer & Liller 1990（PASP 102, 796）表 1；大气取各站海拔的标准大气（海平面 101 kPa、10°C），未用当日气象");
    tab.meta("差 = 实测 − 预测。原文 #64 的 R0 与其 Z 差 0.020°、#76 差 0.002°，照印刷值列出，本检验只用 Z");
    let mut by_site: Vec<(String, f64, Vec<f64>, Vec<f64>)> = vec![];
    for r in &rows {
        let dip = (1.0 / (1.0 + r.alt / 6_378_000.0)).acos().to_degrees();
        let geo = 90.0 + dip + bennett_r0;
        let a = Atmosphere::standard_at_height(Length::from_value(real(r.alt), LengthUnit::Meter));
        let (h, refr) = ray_trace_sea_horizon(&a, &c);
        let z = 90.0 - (h - refr).as_f64().to_degrees();
        let (dg, dt) = (r.z - geo, r.z - z);
        tab.row(vec![r.no.clone(), r.site.clone(), format!("{:.0}", r.alt), r.date.clone(), r.note.clone(), format!("{:.3}", r.z), format!("{:.3}", r.r0), format!("{geo:.3}"), format!("{dg:+.3}"), format!("{z:.3}"), format!("{dt:+.3}")]);
        let key = if r.alt >= 2000.0 { r.site.clone() } else if r.alt >= 150.0 { r.site.clone() } else { "夏威夷低站".to_string() };
        match by_site.iter_mut().find(|s| s.0 == key) {
            Some(s) => {
                s.2.push(dg);
                s.3.push(dt);
            }
            None => by_site.push((key, r.alt, vec![dg], vec![dt])),
        }
    }
    tab.write();
    println!("海平面地平折射：Bennett {bennett_r0:.3}°，追迹 {trace_r0:.3}°；144 次总平均 0.551°");
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    for (k, alt, g, t) in &by_site {
        println!("{k:14} {alt:5.0} m n={:3} 几何法均差 {:+.3}° rms {:.3}°；追迹均差 {:+.3}° rms {:.3}°", g.len(), mean(g), rms(g), mean(t), rms(t));
    }
    let all_g: Vec<f64> = by_site.iter().flat_map(|s| s.2.clone()).collect();
    let all_t: Vec<f64> = by_site.iter().flat_map(|s| s.3.clone()).collect();
    println!("全部 116：几何法 rms {:.3}°，追迹 rms {:.3}°", rms(&all_g), rms(&all_t));
}

/// 同一批实测里，Seidelmann 1968 年六分仪的日出日没（原文表 2 带当时气温气压）：R(2°)、R(1°)、R0。
#[test]
fn refraction_seidelmann1968() {
    let c = RayTraceConstants::default();
    let mut tab = CheckTable::new("refraction_seidelmann1968", &["日期", "地点", "时", "气温 °C", "气压 inHg", "视高度 °", "实测 °", "Bennett °", "差 °", "Saemundsson °", "差 °", "追迹 °", "差 °"]);
    tab.meta("Schaefer & Liller 1990 表 2（Seidelmann 等六分仪）；R0 为原文外推。原文归因的日出/日没差 40% 为低层温度结构，气温气压解释不了");
    let rows = [
        ("1968-11-01", "Kitty Hawk", "日没", 16.0, 30.08, [Some(0.25), Some(0.35), Some(0.49)]),
        ("1968-11-02", "Nag's Head", "日出", 14.0, 30.02, [Some(0.36), None, Some(0.64)]),
        ("1968-11-02", "Kitty Hawk", "日没", 19.0, 29.98, [Some(0.27), Some(0.34), Some(0.50)]),
        ("1968-11-03", "Nag's Head", "日出", 14.0, 30.02, [Some(0.38), Some(0.47), Some(0.66)]),
    ];
    for (d, s, w, t, p, vals) in rows {
        let a = atm(p * 25.4, t, 0.0, -0.0065);
        for (h, v) in [2.0, 1.0, 0.0].into_iter().zip(vals) {
            let Some(v) = v else { continue };
            let (b, sa, tr) = (formula(h, RefractionModel::Bennett, &a) / 3600.0, formula(h, RefractionModel::Saemundsson, &a) / 3600.0, trace(h, &a, &c) / 3600.0);
            tab.row(vec![d.into(), s.into(), w.into(), format!("{t:.0}"), format!("{p:.2}"), format!("{h:.0}"), format!("{v:.2}"), format!("{b:.3}"), format!("{:+.3}", v - b), format!("{sa:.3}"), format!("{:+.3}", v - sa), format!("{tr:.3}"), format!("{:+.3}", v - tr)]);
        }
    }
    tab.write();
}
