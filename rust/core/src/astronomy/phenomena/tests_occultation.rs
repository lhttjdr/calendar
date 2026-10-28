//! 月掩星检验。

use super::*;
use crate::astronomy::apparent::ApparentPipelineOptions;
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::geodesy::Observer;
use crate::astronomy::pipeline::Planet;
use crate::astronomy::stars::load_hipparcos_bright_from_repo;
use crate::math::real::{real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;
use crate::quantity::length::Length;
use crate::quantity::unit::LengthUnit;

/// Skyfield（DE441，`scripts/phenomena/skyfield_occultations.py`）：北京（116.3912°E、39.9067°N、50 m），2023—2026 年，
/// 轩辕十四、角宿一、心宿二、毕宿五、昴宿六与金、火、木、土的全部月掩（不论昼夜、地平上下）。(天体, 入掩 TT, 出掩 TT, ΔT s)。
const SKYFIELD_OCC: &[(&str, f64, f64, f64)] = &[
    ("HIP49669", 2461019.807327549, 2461019.833842597, 69.107),
    ("HIP49669", 2461047.158506589, 2461047.187384858, 69.110),
    ("HIP49669", 2461101.9617529064, 2461102.0002588835, 69.123),
    ("HIP65474", 2460532.944701462, 2460532.9737444306, 69.150),
    ("HIP65474", 2460560.214582635, 2460560.2460546102, 69.130),
    ("HIP65474", 2460642.0068660574, 2460642.036793149, 69.134),
    ("HIP65474", 2460669.2545534642, 2460669.2952197585, 69.138),
    ("HIP80763", 2460208.818366858, 2460208.8763923077, 69.173),
    ("HIP80763", 2460290.705369506, 2460290.760119601, 69.173),
    ("HIP80763", 2460372.87023116, 2460372.8980919, 69.187),
    ("HIP80763", 2460400.0514847226, 2460400.086647949, 69.198),
    ("HIP80763", 2460454.635212362, 2460454.6631582556, 69.205),
    ("HIP80763", 2460481.8911319813, 2460481.9252884146, 69.196),
    ("HIP17702", 2460603.3869085684, 2460603.4338123295, 69.129),
    ("HIP17702", 2460658.2832835354, 2460658.322581971, 69.136),
    ("HIP17702", 2460712.7465079334, 2460712.7847759416, 69.138),
    ("HIP17702", 2460876.964959829, 2460876.9930415954, 69.127),
    ("HIP17702", 2460904.1348720053, 2460904.1654402716, 69.111),
    ("HIP17702", 2460958.7480171127, 2460958.7754693134, 69.092),
    ("HIP17702", 2461013.636015419, 2461013.6617986932, 69.107),
    ("HIP17702", 2461150.2161056856, 2461150.238722531, 69.147),
    ("HIP17702", 2461205.0770213036, 2461205.1026530107, 69.146),
    ("HIP17702", 2461368.9410647303, 2461368.951796661, 69.099),
    ("HIP17702", 2461396.493888098, 2461396.509041325, 69.096),
    ("Venus", 2460257.933873902, 2460257.958805398, 69.172),
    ("Venus", 2460938.006443533, 2460938.0382590564, 69.094),
    ("Mars", 2460435.6271160096, 2460435.6576228784, 69.204),
    ("Mars", 2460662.8384953057, 2460662.8614214137, 69.135),
    ("Mars", 2460716.355341306, 2460716.389905042, 69.137),
    ("Mars", 2460856.4754528757, 2460856.5033375826, 69.141),
    ("Mars", 2461318.7846433455, 2461318.8031918937, 69.090),
    ("Jupiter", 2460082.0527207134, 2460082.0786530646, 69.229),
    ("Saturn", 2460516.3799895514, 2460516.425723927, 69.163),
    ("Saturn", 2460543.6386893624, 2460543.6662194086, 69.139),
    ("Saturn", 2460598.305751823, 2460598.3389464645, 69.125),
    ("Saturn", 2460707.6346287252, 2460707.6797027113, 69.137),
];

/// 与 Skyfield 逐次对照：次数一致，入掩、出掩时刻差 < 1 s。DE441（缺文件则跳过）。
#[test]
fn occultations_vs_skyfield() {
    let _ = try_init_full_nutation_from_repo();
    let Ok(k) = crate::astronomy::ephemeris::De406Kernel::open(crate::repo::repo_root().join(crate::repo::paths::DE441_BSP)) else {
        eprintln!("跳过：无 DE441");
        return;
    };
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let obs = Observer::new(PlaneAngle::from_deg(real(116.3912)), PlaneAngle::from_deg(real(39.9067)), Length::from_value(real(50.0), LengthUnit::Meter));
    let opts = ApparentPipelineOptions::pipeline_default();
    let (j0, j1) = (2459945.5, 2461406.5);
    let targets: Vec<(&str, OccultedBody)> = vec![
        ("HIP49669", OccultedBody::Star(*cat.by_hip(49669).unwrap())),
        ("HIP65474", OccultedBody::Star(*cat.by_hip(65474).unwrap())),
        ("HIP80763", OccultedBody::Star(*cat.by_hip(80763).unwrap())),
        ("HIP21421", OccultedBody::Star(*cat.by_hip(21421).unwrap())),
        ("HIP17702", OccultedBody::Star(*cat.by_hip(17702).unwrap())),
        ("Venus", OccultedBody::Planet(Planet::Venus)),
        ("Mars", OccultedBody::Planet(Planet::Mars)),
        ("Jupiter", OccultedBody::Planet(Planet::Jupiter)),
        ("Saturn", OccultedBody::Planet(Planet::Saturn)),
    ];
    let mut worst: f64 = 0.0;
    let mut tab = crate::test_util::CheckTable::new(
        "occultations_vs_skyfield",
        &["天体", "入掩 TT", "入掩差 s", "出掩差 s", "边", "位置角°", "尖角°", "月高°", "日高°", "亮面比例"],
    );
    tab.meta("北京（116.3912°E、39.9067°N、50 m）2023—2026 月掩星对 Skyfield（DE441，scripts/phenomena/skyfield_occultations.py）逐次对照；不论昼夜、地平上下");
    for (name, body) in targets {
        let want: Vec<_> = SKYFIELD_OCC.iter().filter(|r| r.0 == name).collect();
        let dt = want.first().map(|r| r.3).unwrap_or(69.15);
        let t = std::time::Instant::now();
        let got = lunar_occultations(&k, body, &obs, Duration::in_seconds(real(dt)), real(j0), real(j1), &opts);
        assert_eq!(got.len(), want.len(), "{name}: 次数");
        for (g, w) in got.iter().zip(&want) {
            let di = (g.immersion.jd_tt.as_f64() - w.1) * 86400.0;
            let de = (g.emersion.jd_tt.as_f64() - w.2) * 86400.0;
            worst = worst.max(di.abs()).max(de.abs());
            println!(
                "{name:9} 入 {:.6} 差 {di:+.2} s  出 差 {de:+.2} s  {}边 PA {:5.1}° 尖角 {:4.1}° 月高 {:5.1}° 日高 {:6.1}° 亮面 {:.2}",
                g.immersion.jd_tt.as_f64(),
                if g.immersion.bright_limb { "亮" } else { "暗" },
                g.immersion.position_angle_deg.as_f64(),
                g.immersion.cusp_angle_deg.as_f64(),
                g.immersion.moon_altitude_deg.as_f64(),
                g.immersion.sun_altitude_deg.as_f64(),
                g.immersion.sunlit_fraction.as_f64()
            );
            tab.row(vec![
                name.to_string(),
                crate::test_util::jd_to_calendar(g.immersion.jd_tt.as_f64()),
                format!("{di:+.2}"),
                format!("{de:+.2}"),
                if g.immersion.bright_limb { "亮".into() } else { "暗".into() },
                format!("{:.1}", g.immersion.position_angle_deg.as_f64()),
                format!("{:.1}", g.immersion.cusp_angle_deg.as_f64()),
                format!("{:.1}", g.immersion.moon_altitude_deg.as_f64()),
                format!("{:.1}", g.immersion.sun_altitude_deg.as_f64()),
                format!("{:.2}", g.immersion.sunlit_fraction.as_f64()),
            ]);
            assert!(di.abs() < 1.0 && de.abs() < 1.0, "{name} {}: 入 {di:.2} s 出 {de:.2} s", w.1);
        }
        println!("{name}: {} 次，{:.1} s", got.len(), t.elapsed().as_secs_f64());
    }
    tab.write();
    println!("最大时刻差 {worst:.2} s");
}
