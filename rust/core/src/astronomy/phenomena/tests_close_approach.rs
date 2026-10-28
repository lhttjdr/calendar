//! 近距事件对照 Skyfield 1.55（同读 de406.bsp）：行星之间 2020—2023 年、月与行星 2023 年，角距极小 ≤ 5°。参考数据由
//! `scripts/ephemeris/skyfield_close_approaches.py` 生成（2026-10-26，Skyfield `find_minima`）。另以 VSOP87（DE406 版改正）+
//! ELPMPP02 的组合提供者复算，与 DE406 比较。

use super::*;
use crate::astronomy::apparent::ApparentPipelineOptions;
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo, De406Kernel};
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::frame::precession::PrecessionModel;
use crate::astronomy::pipeline::{Body, Planet, Vsop87Elp, Vsop87Planets};
use crate::math::real::{real, RealOps};

/// (JD TT, 天体 A, 天体 B, 最小角距 °)
#[rustfmt::skip]
const SKYFIELD_CLOSE: [(f64, Body, Body, f64); 49] = [
    (2458987.6966910595, Body::Planet(Planet::Jupiter), Body::Planet(Planet::Saturn), 4.697033238448901),
    (2459205.265328438, Body::Planet(Planet::Jupiter), Body::Planet(Planet::Saturn), 0.10176913598608904),
    (2459257.1139869047, Body::Planet(Planet::Venus), Body::Planet(Planet::Jupiter), 0.43015961330001284),
    (2459700.3732301956, Body::Planet(Planet::Venus), Body::Planet(Planet::Jupiter), 0.2301496120020159),
    (2460005.712829673, Body::Planet(Planet::Venus), Body::Planet(Planet::Jupiter), 0.4891772204166127),
    (2458928.943556657, Body::Planet(Planet::Mars), Body::Planet(Planet::Jupiter), 0.7072619441234894),
    (2459728.93593751, Body::Planet(Planet::Mars), Body::Planet(Planet::Jupiter), 0.5824809629252945),
    (2458991.9059624234, Body::Planet(Planet::Mercury), Body::Planet(Planet::Venus), 0.8804789272779002),
    (2459258.897611358, Body::Planet(Planet::Mercury), Body::Planet(Planet::Venus), 4.561168904583696),
    (2459330.2177431975, Body::Planet(Planet::Mercury), Body::Planet(Planet::Venus), 1.1592150986203746),
    (2459363.63602903, Body::Planet(Planet::Mercury), Body::Planet(Planet::Venus), 0.4008846582990557),
    (2459577.703374553, Body::Planet(Planet::Mercury), Body::Planet(Planet::Venus), 4.216660356821002),
    (2459849.4933631676, Body::Planet(Planet::Mercury), Body::Planet(Planet::Venus), 3.1761423756066334),
    (2459904.674356296, Body::Planet(Planet::Mercury), Body::Planet(Planet::Venus), 1.273638558041431),
    (2459942.810651493, Body::Planet(Planet::Mercury), Body::Planet(Planet::Venus), 1.4017572833960017),
    (2459967.9368545464, Body::Moon, Body::Planet(Planet::Venus), 3.217529951988303),
    (2459997.904380448, Body::Moon, Body::Planet(Planet::Venus), 1.8436061736320979),
    (2460027.939639173, Body::Moon, Body::Planet(Planet::Venus), 0.09964071176971737),
    (2460058.0226155366, Body::Moon, Body::Planet(Planet::Venus), 1.2877860824205982),
    (2460088.0263858545, Body::Moon, Body::Planet(Planet::Venus), 2.1949081933941184),
    (2460117.6351394155, Body::Moon, Body::Planet(Planet::Venus), 3.5178965334360357),
    (2460257.941304774, Body::Moon, Body::Planet(Planet::Venus), 0.8861118056522287),
    (2460288.0727120545, Body::Moon, Body::Planet(Planet::Venus), 3.3093877137085568),
    (2459970.6469279164, Body::Moon, Body::Planet(Planet::Jupiter), 1.607365560311888),
    (2459998.4570687287, Body::Moon, Body::Planet(Planet::Jupiter), 1.0523440022001196),
    (2460026.348880426, Body::Moon, Body::Planet(Planet::Jupiter), 0.4733624129615123),
    (2460054.2269426165, Body::Moon, Body::Planet(Planet::Jupiter), 0.10821389532573608),
    (2460082.0286394646, Body::Moon, Body::Planet(Planet::Jupiter), 0.7177459471826452),
    (2460109.7275102395, Body::Moon, Body::Planet(Planet::Jupiter), 1.3748192739550638),
    (2460137.322964473, Body::Moon, Body::Planet(Planet::Jupiter), 2.0537000003929786),
    (2460164.824321908, Body::Moon, Body::Planet(Planet::Jupiter), 2.6666606733974585),
    (2460192.2354330025, Body::Moon, Body::Planet(Planet::Jupiter), 3.074462359401056),
    (2460219.5487692687, Body::Moon, Body::Planet(Planet::Jupiter), 3.146397026288957),
    (2460246.7584495796, Body::Moon, Body::Planet(Planet::Jupiter), 2.8883208907149713),
    (2460273.8896755762, Body::Moon, Body::Planet(Planet::Jupiter), 2.5315698674449147),
    (2460301.022999231, Body::Moon, Body::Planet(Planet::Jupiter), 2.375770715384274),
    (2459948.328621325, Body::Moon, Body::Planet(Planet::Mars), 0.522989826800289),
    (2459975.686747693, Body::Moon, Body::Planet(Planet::Mars), 0.10256237294436703),
    (2460003.675586208, Body::Moon, Body::Planet(Planet::Mars), 1.056765474965787),
    (2460032.0454860507, Body::Moon, Body::Planet(Planet::Mars), 2.28851383966041),
    (2460060.6229719524, Body::Moon, Body::Planet(Planet::Mars), 3.2183449488427684),
    (2460089.3061010633, Body::Moon, Body::Planet(Planet::Mars), 3.6651229728196233),
    (2460118.0380357164, Body::Moon, Body::Planet(Planet::Mars), 3.572154128597728),
    (2460146.7904097997, Body::Moon, Body::Planet(Planet::Mars), 2.9651726747196525),
    (2460175.554071032, Body::Moon, Body::Planet(Planet::Mars), 1.927208448929017),
    (2460204.333743172, Body::Moon, Body::Planet(Planet::Mars), 0.5843545332651511),
    (2460233.143095095, Body::Moon, Body::Planet(Planet::Mars), 0.9020391123830781),
    (2460261.9975856943, Body::Moon, Body::Planet(Planet::Mars), 2.338789982125011),
    (2460290.9050647444, Body::Moon, Body::Planet(Planet::Mars), 3.5038812263575765),
];

fn pairs() -> Vec<(Body, Body, f64, f64)> {
    let p = |x| Body::Planet(x);
    let (y20, y23, y24) = (2458849.5, 2459945.5, 2460310.5);
    vec![
        (p(Planet::Jupiter), p(Planet::Saturn), y20, y24),
        (p(Planet::Venus), p(Planet::Jupiter), y20, y24),
        (p(Planet::Mars), p(Planet::Jupiter), y20, y24),
        (p(Planet::Mercury), p(Planet::Venus), y20, y24),
        (Body::Moon, p(Planet::Venus), y23, y24),
        (Body::Moon, p(Planet::Jupiter), y23, y24),
        (Body::Moon, p(Planet::Mars), y23, y24),
    ]
}

/// DE406（P03 岁差，同 Skyfield）：事件一一对应，时刻差 ≤ 30 s（极小处角距变化平缓），最小角距差 ≤ 0.1″。
#[test]
fn close_approaches_vs_skyfield() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let kernel = De406Kernel::open(&bsp).unwrap();
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let mut opts = ApparentPipelineOptions::pipeline_default();
    opts.precession_model = Some(PrecessionModel::P03);
    let mut failures = vec![];
    let mut tab = crate::test_util::CheckTable::new("close_approaches_vs_skyfield", &["天体对", "本实现 TT", "时刻差 s", "Skyfield 最小角距°", "角距差″"]);
    tab.meta("近距事件（地心视角距 ≤ 5° 的极小）对 Skyfield（DE406，P03，scripts/ephemeris/skyfield_close_approaches.py）逐次对照");
    for (a, b, j0, j1) in pairs() {
        let ours = close_approaches(&kernel, a, b, real(j0), real(j1), real(5.0), &opts);
        let refs: Vec<_> = SKYFIELD_CLOSE.iter().filter(|r| r.1 == a && r.2 == b).collect();
        if ours.len() != refs.len() {
            failures.push(format!("{a:?}–{b:?}: 本仓库 {} 次，Skyfield {} 次", ours.len(), refs.len()));
            continue;
        }
        let (mut dt, mut ds) = (0.0_f64, 0.0_f64);
        for (o, r) in ours.iter().zip(&refs) {
            dt = dt.max((o.jd_tt.as_f64() - r.0).abs() * 86400.0);
            ds = ds.max((o.separation_deg.as_f64() - r.3).abs() * 3600.0);
            tab.row(vec![
                format!("{a:?}–{b:?}"),
                crate::test_util::jd_to_calendar(o.jd_tt.as_f64()),
                format!("{:+.1}", (o.jd_tt.as_f64() - r.0) * 86400.0),
                format!("{:.4}", r.3),
                format!("{:+.3}", (o.separation_deg.as_f64() - r.3) * 3600.0),
            ]);
        }
        println!("{a:?}–{b:?}: {} 次，最大时刻差 {dt:.1} s，最小角距差 {ds:.3}″", ours.len());
        if dt > 30.0 || ds > 0.1 {
            failures.push(format!("{a:?}–{b:?}: 时刻差 {dt:.1} s，角距差 {ds:.3}″"));
        }
    }
    tab.write();
    assert!(failures.is_empty(), "{failures:?}");
}

/// VSOP87（DE406 版改正）+ ELPMPP02 对 DE406：同一批事件（月的配对只取 2023 年前 120 日），最小角距差 ≤ 1″（行星改正 ≤ 0.36″；月 ELPMPP02 与 DE406 差约 1″ 内）。
/// 耗时约 20 分钟（TwoFloat，VSOP87 + ELPMPP02 全级数），默认不跑：`cargo test --release ... -- --ignored close_approaches_vsop_elp_vs_de406`。
#[test]
#[ignore]
fn close_approaches_vsop_elp_vs_de406() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let kernel = De406Kernel::open(&bsp).unwrap();
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let loaded: Vec<_> = Planet::ALL.iter().map(|p| (*p, load_planet_vsop87_corrected_from_repo(*p, PatchTarget::De406).unwrap())).collect();
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: loaded.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
    let opts = ApparentPipelineOptions::pipeline_default();
    let mut failures = vec![];
    let mut tab = crate::test_util::CheckTable::new("close_approaches_vsop_elp_vs_de406", &["天体对", "DE406 TT", "时刻差 s", "DE406 最小角距°", "角距差″"]);
    tab.meta("近距事件：VSOP87（DE406 版改正）+ ELPMPP02 对 DE406 逐次对照（月的配对取 2023 年前 120 日）");
    for (a, b, j0, j1) in pairs() {
        // VSOP87 + ELPMPP02 全级数逐 0.2 日取样较慢，月的配对只取前 120 日
        let j1 = if a == Body::Moon { j0 + 120.0 } else { j1 };
        let x = close_approaches(&kernel, a, b, real(j0), real(j1), real(5.0), &opts);
        let y = close_approaches(&eph, a, b, real(j0), real(j1), real(5.0), &opts);
        if x.len() != y.len() {
            failures.push(format!("{a:?}–{b:?}: DE406 {} 次，VSOP87+ELP {} 次", x.len(), y.len()));
            continue;
        }
        let (mut dt, mut ds) = (0.0_f64, 0.0_f64);
        for (o, r) in x.iter().zip(&y) {
            dt = dt.max((o.jd_tt - r.jd_tt).as_f64().abs() * 86400.0);
            ds = ds.max((o.separation_deg - r.separation_deg).as_f64().abs() * 3600.0);
            tab.row(vec![
                format!("{a:?}–{b:?}"),
                crate::test_util::jd_to_calendar(o.jd_tt.as_f64()),
                format!("{:+.1}", (r.jd_tt - o.jd_tt).as_f64() * 86400.0),
                format!("{:.4}", o.separation_deg.as_f64()),
                format!("{:+.3}", (r.separation_deg - o.separation_deg).as_f64() * 3600.0),
            ]);
        }
        println!("{a:?}–{b:?}: {} 次，VSOP87+ELP 对 DE406 最大时刻差 {dt:.1} s，最小角距差 {ds:.3}″", x.len());
        if ds > 1.0 {
            failures.push(format!("{a:?}–{b:?}: 角距差 {ds:.3}″"));
        }
    }
    tab.write();
    assert!(failures.is_empty(), "{failures:?}");
}
