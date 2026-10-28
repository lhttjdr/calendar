//! 行星会合现象对照 Skyfield 1.55（同读 de406.bsp，2020—2025 年）。参考数据由 `scripts/ephemeris/skyfield_planet_events.py` 生成
//! （2026-10-26）：合、冲取 `almanac.oppositions_conjunctions`；留、大距用 Skyfield 的 `find_discrete` / `find_maxima` 独立求得。

use super::*;
use crate::astronomy::apparent::ApparentPipelineOptions;
use crate::astronomy::ephemeris::De406Kernel;
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::frame::precession::PrecessionModel;
use crate::astronomy::pipeline::Planet;
use crate::math::real::{real, RealOps};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum C {
    Conj,
    Opp,
    Stat,
    Elong,
}

fn class(k: PlanetEventKind) -> C {
    match k {
        PlanetEventKind::Conjunction | PlanetEventKind::InferiorConjunction | PlanetEventKind::SuperiorConjunction => C::Conj,
        PlanetEventKind::Opposition => C::Opp,
        PlanetEventKind::StationRetrograde | PlanetEventKind::StationDirect => C::Stat,
        PlanetEventKind::GreatestElongationEast | PlanetEventKind::GreatestElongationWest => C::Elong,
    }
}

#[rustfmt::skip]
const SKYFIELD_EVENTS: [(f64, Planet, C); 241] = [
    (2458859.1392741213, Planet::Mercury, C::Conj),
    (2458905.57353295, Planet::Mercury, C::Conj),
    (2458974.404614743, Planet::Mercury, C::Conj),
    (2459031.6206707805, Planet::Mercury, C::Conj),
    (2459079.1308785267, Planet::Mercury, C::Conj),
    (2459148.2668294953, Planet::Mercury, C::Conj),
    (2459203.6436608587, Planet::Mercury, C::Conj),
    (2459254.075695719, Planet::Mercury, C::Conj),
    (2459323.5768625955, Planet::Mercury, C::Conj),
    (2459376.551411226, Planet::Mercury, C::Conj),
    (2459428.089352643, Planet::Mercury, C::Conj),
    (2459497.18011779, Planet::Mercury, C::Conj),
    (2459547.6948381383, Planet::Mercury, C::Conj),
    (2459602.936994367, Planet::Mercury, C::Conj),
    (2459672.466631658, Planet::Mercury, C::Conj),
    (2459721.304955651, Planet::Mercury, C::Conj),
    (2459777.3186201644, Planet::Mercury, C::Conj),
    (2459845.785424185, Planet::Mercury, C::Conj),
    (2459892.1971514532, Planet::Mercury, C::Conj),
    (2459952.040365601, Planet::Mercury, C::Conj),
    (2460020.9484477313, Planet::Mercury, C::Conj),
    (2460066.4782861248, Planet::Mercury, C::Conj),
    (2460126.7131102225, Planet::Mercury, C::Conj),
    (2460193.965461871, Planet::Mercury, C::Conj),
    (2460237.7355364445, Planet::Mercury, C::Conj),
    (2460301.2883024504, Planet::Mercury, C::Conj),
    (2460368.863906468, Planet::Mercury, C::Conj),
    (2460412.4611621206, Planet::Mercury, C::Conj),
    (2460476.190053156, Planet::Mercury, C::Conj),
    (2460541.582777723, Planet::Mercury, C::Conj),
    (2460584.3822955037, Planet::Mercury, C::Conj),
    (2460650.5964890174, Planet::Mercury, C::Conj),
    (2460716.0064282375, Planet::Mercury, C::Conj),
    (2460759.3260326707, Planet::Mercury, C::Conj),
    (2460825.6763266455, Planet::Mercury, C::Conj),
    (2460888.4878043868, Planet::Mercury, C::Conj),
    (2460931.953424712, Planet::Mercury, C::Conj),
    (2460999.8919365136, Planet::Mercury, C::Conj),
    (2458896.5383091713, Planet::Mercury, C::Stat),
    (2458918.659534632, Planet::Mercury, C::Stat),
    (2459018.708345418, Planet::Mercury, C::Stat),
    (2459042.8524509813, Planet::Mercury, C::Stat),
    (2459136.5459380737, Planet::Mercury, C::Stat),
    (2459157.2436546204, Planet::Mercury, C::Stat),
    (2459245.1617061645, Planet::Mercury, C::Stat),
    (2459266.536908089, Planet::Mercury, C::Stat),
    (2459364.4411076196, Planet::Mercury, C::Stat),
    (2459388.417499776, Planet::Mercury, C::Stat),
    (2459484.7162037543, Planet::Mercury, C::Stat),
    (2459506.137492549, Planet::Mercury, C::Stat),
    (2459593.987950664, Planet::Mercury, C::Stat),
    (2459614.676456253, Planet::Mercury, C::Stat),
    (2459709.992130214, Planet::Mercury, C::Stat),
    (2459733.8343321923, Planet::Mercury, C::Stat),
    (2459832.652234913, Planet::Mercury, C::Stat),
    (2459854.880938156, Planet::Mercury, C::Stat),
    (2459942.8979080524, Planet::Mercury, C::Stat),
    (2459963.05074007, Planet::Mercury, C::Stat),
    (2460055.8583836863, Planet::Mercury, C::Stat),
    (2460079.6374075236, Planet::Mercury, C::Stat),
    (2460180.3337659827, Planet::Mercury, C::Stat),
    (2460203.3488413338, Planet::Mercury, C::Stat),
    (2460291.7987933387, Planet::Mercury, C::Stat),
    (2460311.6310554235, Planet::Mercury, C::Stat),
    (2460402.427536289, Planet::Mercury, C::Stat),
    (2460426.0383866783, Planet::Mercury, C::Stat),
    (2460527.706421691, Planet::Mercury, C::Stat),
    (2460551.3855009945, Planet::Mercury, C::Stat),
    (2460640.613573989, Planet::Mercury, C::Stat),
    (2460660.373238666, Planet::Mercury, C::Stat),
    (2460749.7828131095, Planet::Mercury, C::Stat),
    (2460772.964426164, Planet::Mercury, C::Stat),
    (2460874.698725061, Planet::Mercury, C::Stat),
    (2460898.8132110303, Planet::Mercury, C::Stat),
    (2460989.2936513126, Planet::Mercury, C::Stat),
    (2461009.2358329096, Planet::Mercury, C::Stat),
    (2458890.0813214146, Planet::Mercury, C::Elong),
    (2458932.5885517946, Planet::Mercury, C::Elong),
    (2459005.0473675765, Planet::Mercury, C::Elong),
    (2459053.134251162, Planet::Mercury, C::Elong),
    (2459124.1713744663, Planet::Mercury, C::Elong),
    (2459164.2114031916, Planet::Mercury, C::Elong),
    (2459238.5819609226, Planet::Mercury, C::Elong),
    (2459279.9745621653, Planet::Mercury, C::Elong),
    (2459351.746390564, Planet::Mercury, C::Elong),
    (2459400.3238012935, Planet::Mercury, C::Elong),
    (2459471.6845079944, Planet::Mercury, C::Elong),
    (2459512.7297724467, Planet::Mercury, C::Elong),
    (2459586.9617242482, Planet::Mercury, C::Elong),
    (2459627.380560376, Planet::Mercury, C::Elong),
    (2459698.840387407, Planet::Mercury, C::Elong),
    (2459747.1227262574, Planet::Mercury, C::Elong),
    (2459819.1774303545, Planet::Mercury, C::Elong),
    (2459861.385184335, Planet::Mercury, C::Elong),
    (2459935.1476018084, Planet::Mercury, C::Elong),
    (2459974.746440227, Planet::Mercury, C::Elong),
    (2460046.4247004734, Planet::Mercury, C::Elong),
    (2460093.7326786676, Planet::Mercury, C::Elong),
    (2460166.574861462, Planet::Mercury, C::Elong),
    (2460210.0535926614, Planet::Mercury, C::Elong),
    (2460283.1039293897, Planet::Mercury, C::Elong),
    (2460322.110179852, Planet::Mercury, C::Elong),
    (2460394.441134256, Planet::Mercury, C::Elong),
    (2460440.3961863285, Planet::Mercury, C::Elong),
    (2460513.777724413, Planet::Mercury, C::Elong),
    (2460558.6051571146, Planet::Mercury, C::Elong),
    (2460630.84062647, Planet::Mercury, C::Elong),
    (2460669.6051183664, Planet::Mercury, C::Elong),
    (2460742.757261561, Planet::Mercury, C::Elong),
    (2460787.284920093, Planet::Mercury, C::Elong),
    (2460860.6943865623, Planet::Mercury, C::Elong),
    (2460906.909193701, Planet::Mercury, C::Elong),
    (2460978.419015146, Planet::Mercury, C::Elong),
    (2461017.3778208536, Planet::Mercury, C::Elong),
    (2459004.239429159, Planet::Venus, C::Conj),
    (2459299.7908871765, Planet::Venus, C::Conj),
    (2459588.5338117285, Planet::Venus, C::Conj),
    (2459875.3879000423, Planet::Venus, C::Conj),
    (2460169.9699777905, Planet::Venus, C::Conj),
    (2460466.149028007, Planet::Venus, C::Conj),
    (2460757.5476753493, Planet::Venus, C::Conj),
    (2458982.7821343346, Planet::Venus, C::Stat),
    (2459025.784262713, Planet::Venus, C::Stat),
    (2459567.9423495894, Planet::Venus, C::Stat),
    (2459608.8660051487, Planet::Venus, C::Stat),
    (2460148.56525471, Planet::Venus, C::Stat),
    (2460191.5564394533, Planet::Venus, C::Stat),
    (2460736.525881335, Planet::Venus, C::Stat),
    (2460778.544083544, Planet::Venus, C::Stat),
    (2458933.426868588, Planet::Venus, C::Elong),
    (2459074.510656328, Planet::Venus, C::Elong),
    (2459517.3701957013, Planet::Venus, C::Elong),
    (2459658.893232037, Planet::Venus, C::Elong),
    (2460099.9596971786, Planet::Venus, C::Elong),
    (2460241.4691756545, Planet::Venus, C::Elong),
    (2460685.710248823, Planet::Venus, C::Elong),
    (2460827.6456165463, Planet::Venus, C::Elong),
    (2459136.477135428, Planet::Mars, C::Opp),
    (2459495.6681583393, Planet::Mars, C::Conj),
    (2459921.738018283, Planet::Mars, C::Opp),
    (2460266.7380267563, Planet::Mars, C::Conj),
    (2460691.6109306575, Planet::Mars, C::Opp),
    (2459102.433001385, Planet::Mars, C::Stat),
    (2459167.5256152833, Planet::Mars, C::Stat),
    (2459883.0603004564, Planet::Mars, C::Stat),
    (2459957.373249713, Planet::Mars, C::Stat),
    (2460651.4821575433, Planet::Mars, C::Stat),
    (2460730.584016109, Planet::Mars, C::Stat),
    (2459044.8330665664, Planet::Jupiter, C::Opp),
    (2459243.569940581, Planet::Jupiter, C::Conj),
    (2459446.5206092335, Planet::Jupiter, C::Opp),
    (2459644.0886246515, Planet::Jupiter, C::Conj),
    (2459849.3154957145, Planet::Jupiter, C::Opp),
    (2460046.4224040816, Planet::Jupiter, C::Conj),
    (2460251.710841667, Planet::Jupiter, C::Opp),
    (2460449.282352768, Planet::Jupiter, C::Conj),
    (2460652.374435427, Planet::Jupiter, C::Opp),
    (2460851.137731707, Planet::Jupiter, C::Conj),
    (2458984.106337581, Planet::Jupiter, C::Stat),
    (2459105.52908321, Planet::Jupiter, C::Stat),
    (2459386.1292840336, Planet::Jupiter, C::Stat),
    (2459505.7300059446, Planet::Jupiter, C::Stat),
    (2459789.3601631457, Planet::Jupiter, C::Stat),
    (2459907.4605655842, Planet::Jupiter, C::Stat),
    (2460192.091777325, Planet::Jupiter, C::Stat),
    (2460309.612174222, Planet::Jupiter, C::Stat),
    (2460592.7957056, Planet::Jupiter, C::Stat),
    (2460710.903858769, Planet::Jupiter, C::Stat),
    (2460991.196256295, Planet::Jupiter, C::Stat),
    (2458862.1367268683, Planet::Saturn, C::Conj),
    (2459051.436727104, Planet::Saturn, C::Opp),
    (2459238.626733203, Planet::Saturn, C::Conj),
    (2459428.7607976864, Planet::Saturn, C::Opp),
    (2459615.295725499, Planet::Saturn, C::Conj),
    (2459806.216476633, Planet::Saturn, C::Opp),
    (2459992.2010777043, Planet::Saturn, C::Conj),
    (2460183.8536269674, Planet::Saturn, C::Opp),
    (2460369.39351898, Planet::Saturn, C::Conj),
    (2460561.6919142283, Planet::Saturn, C::Opp),
    (2460746.9375929004, Planet::Saturn, C::Conj),
    (2460939.740833851, Planet::Saturn, C::Opp),
    (2458980.6738851005, Planet::Saturn, C::Stat),
    (2459121.7169918017, Planet::Saturn, C::Stat),
    (2459357.889012631, Planet::Saturn, C::Stat),
    (2459498.5960046565, Planet::Saturn, C::Stat),
    (2459735.408341756, Planet::Saturn, C::Stat),
    (2459875.672690239, Planet::Saturn, C::Stat),
    (2460113.2281597108, Planet::Saturn, C::Stat),
    (2460252.7945292974, Planet::Saturn, C::Stat),
    (2460491.2968743183, Planet::Saturn, C::Stat),
    (2460630.098103405, Planet::Saturn, C::Stat),
    (2460869.67213169, Planet::Saturn, C::Stat),
    (2461007.6617833283, Planet::Saturn, C::Stat),
    (2458965.8764766594, Planet::Uranus, C::Conj),
    (2459154.162721701, Planet::Uranus, C::Opp),
    (2459335.329899081, Planet::Uranus, C::Conj),
    (2459523.4992230465, Planet::Uranus, C::Opp),
    (2459704.807407479, Planet::Uranus, C::Conj),
    (2459892.8523923745, Planet::Uranus, C::Opp),
    (2460074.3313136706, Planet::Uranus, C::Conj),
    (2460262.2234717007, Planet::Uranus, C::Opp),
    (2460443.8851835122, Planet::Uranus, C::Conj),
    (2460631.6152205807, Planet::Uranus, C::Opp),
    (2460813.481563925, Planet::Uranus, C::Conj),
    (2461001.0184931983, Planet::Uranus, C::Opp),
    (2458859.5759315123, Planet::Uranus, C::Stat),
    (2459077.1026804, Planet::Uranus, C::Stat),
    (2459228.8592465282, Planet::Uranus, C::Stat),
    (2459446.5703224028, Planet::Uranus, C::Stat),
    (2459598.1440178268, Planet::Uranus, C::Stat),
    (2459816.0796849267, Planet::Uranus, C::Stat),
    (2459967.4580700155, Planet::Uranus, C::Stat),
    (2460185.611080225, Planet::Uranus, C::Stat),
    (2460336.817083928, Planet::Uranus, C::Stat),
    (2460555.138121589, Planet::Uranus, C::Stat),
    (2460706.1830692063, Planet::Uranus, C::Stat),
    (2460924.7030913085, Planet::Uranus, C::Stat),
    (2458917.016785529, Planet::Neptune, C::Conj),
    (2459104.351978409, Planet::Neptune, C::Opp),
    (2459284.501412398, Planet::Neptune, C::Conj),
    (2459471.8903321633, Planet::Neptune, C::Opp),
    (2459651.9890058697, Planet::Neptune, C::Conj),
    (2459839.431933398, Planet::Neptune, C::Opp),
    (2460019.486206389, Planet::Neptune, C::Conj),
    (2460206.97124095, Planet::Neptune, C::Opp),
    (2460386.9746619337, Planet::Neptune, C::Conj),
    (2460574.512396029, Planet::Neptune, C::Opp),
    (2460754.4764841544, Planet::Neptune, C::Conj),
    (2460942.0379530517, Planet::Neptune, C::Opp),
    (2459023.689238878, Planet::Neptune, C::Stat),
    (2459182.5258440604, Planet::Neptune, C::Stat),
    (2459391.3072059094, Planet::Neptune, C::Stat),
    (2459550.057941622, Planet::Neptune, C::Stat),
    (2459758.8306757505, Planet::Neptune, C::Stat),
    (2459917.5110121267, Planet::Neptune, C::Stat),
    (2460126.380743257, Planet::Neptune, C::Stat),
    (2460285.0578965163, Planet::Neptune, C::Stat),
    (2460493.9461104046, Planet::Neptune, C::Stat),
    (2460652.489224588, Planet::Neptune, C::Stat),
    (2460861.3990921853, Planet::Neptune, C::Stat),
    (2461020.017288982, Planet::Neptune, C::Stat),
];

/// 事件一一对应（个数相同），时刻差：合、冲、大距 ≤ 2 s；留 ≤ 10 min。
/// 大距以角距导数之根求（search::extremum_by_derivative_root），实测与 Skyfield 差 ≤ 0.3 s（此前黄金分割约 30 s，见
/// doc/engineering/project-and-implementation.md §11.2.2）。留为黄经变化率之根，慢行星变化率随时间变化极缓，两边黄经约 0.01″ 的
/// 差（章动实现、太阳引力偏折等）即使留的时刻差数分钟（海王星实测 264 s），故放宽。
#[test]
fn planet_events_vs_skyfield() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let kernel = De406Kernel::open(&bsp).unwrap();
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    // Skyfield 用 IAU 2006（P03）岁差
    let mut opts = ApparentPipelineOptions::pipeline_default();
    opts.precession_model = Some(PrecessionModel::P03);
    let mut failures = vec![];
    let mut tab = crate::test_util::CheckTable::new("planet_events_vs_skyfield", &["行星", "类别", "本实现事件", "本实现 TT", "Skyfield TT", "时刻差 s"]);
    tab.meta("行星会合现象对 Skyfield（DE406，P03 岁差，scripts/ephemeris/skyfield_planet_events.py）逐次对照，2020-01-01—2025-12-31");
    for p in Planet::ALL {
        let ours = planet_events(&kernel, p, real(2458849.5), real(2461041.5), &opts);
        for c in [C::Conj, C::Opp, C::Stat, C::Elong] {
            let a: Vec<f64> = ours.iter().filter(|e| class(e.kind) == c).map(|e| e.jd_tt.as_f64()).collect();
            let kinds: Vec<_> = ours.iter().filter(|e| class(e.kind) == c).map(|e| e.kind).collect();
            let b: Vec<f64> = SKYFIELD_EVENTS.iter().filter(|e| e.1 == p && e.2 == c).map(|e| e.0).collect();
            if a.len() != b.len() {
                failures.push(format!("{p:?} {c:?}: 本仓库 {} 个，Skyfield {} 个", a.len(), b.len()));
                continue;
            }
            for ((x, y), k) in a.iter().zip(&b).zip(&kinds) {
                tab.row(vec![
                    format!("{p:?}"),
                    format!("{c:?}"),
                    format!("{k:?}"),
                    crate::test_util::jd_to_calendar(*x),
                    crate::test_util::jd_to_calendar(*y),
                    format!("{:+.1}", (x - y) * 86400.0),
                ]);
            }
            let worst = a.iter().zip(&b).map(|(x, y)| (x - y).abs() * 86400.0).fold(0.0_f64, f64::max);
            if !a.is_empty() {
                println!("{p:?} {c:?}: {} 个，最大时刻差 {worst:.1} s", a.len());
            }
            let tol = if matches!(c, C::Stat) { 600.0 } else { 2.0 };
            if worst > tol {
                failures.push(format!("{p:?} {c:?}: 最大时刻差 {worst:.1} s"));
            }
        }
    }
    tab.write();
    assert!(failures.is_empty(), "{failures:?}");
}
