//! 七政四余与《大清道光十三年七政經緯躔度時憲書》（国家图书馆藏，维基共享资源 NLC892-411999011532-17590）比对。
//! 书末「諸曜俱從子正起算」：取北京地方平时子正。宿度用清代距星（仪象考成，`qing`），黄道宿度，周天 360°。
//! 用法（仓库根）：cargo run --release -p lunar-core --no-default-features --features real-f64 --example qizheng_1833
use lunar_core::astronomy::apparent::ApparentPipelineOptions;
use lunar_core::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use lunar_core::astronomy::ephemeris::vsop87_patch::PatchTarget;
use lunar_core::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
use lunar_core::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use lunar_core::astronomy::pipeline::{Planet, Vsop87Elp, Vsop87Planets};
use lunar_core::astronomy::sky_culture::chinese::{load_lunar_mansion_systems_from_repo, MansionFrame, MANSIONS};
use lunar_core::astronomy::stars::load_hipparcos_bright_from_repo;
use lunar_core::astronomy::time::{TimePoint, TimeScale};
use lunar_core::folk::qizheng::{chart, BeiKind, NodeKind, QizhengOptions};
use lunar_core::math::real::{real, RealOps};

/// 书中四余逐旬宿度（月孛、罗睺、计都、紫气），(距正月初一日数, [(宿, 度, 分); 4])；转录自 PDF 第 40 页（索引 39）左半
const BOOK: [(i64, [(&str, f64, f64); 4]); 12] = [
    (0, [("女", 4., 44.), ("斗", 15., 44.), ("井", 20., 36.), ("角", 2., 38.)]),
    (10, [("女", 5., 53.), ("斗", 15., 13.), ("井", 20., 5.), ("角", 2., 59.)]),
    (20, [("女", 7., 1.), ("斗", 14., 31.), ("井", 19., 23.), ("角", 3., 20.)]),
    (29, [("女", 8., 2.), ("斗", 13., 43.), ("井", 18., 35.), ("角", 3., 39.)]),
    (39, [("女", 9., 9.), ("斗", 12., 43.), ("井", 17., 35.), ("角", 4., 0.)]),
    (49, [("女", 10., 16.), ("斗", 11., 39.), ("井", 16., 31.), ("角", 4., 22.)]),
    (59, [("女", 11., 22.), ("斗", 10., 35.), ("井", 15., 27.), ("角", 4., 43.)]),
    (69, [("虛", 0., 47.), ("斗", 9., 37.), ("井", 14., 29.), ("角", 5., 4.)]),
    (79, [("虛", 1., 52.), ("斗", 8., 46.), ("井", 13., 38.), ("角", 5., 25.)]),
    (88, [("虛", 2., 50.), ("斗", 8., 10.), ("井", 13., 2.), ("角", 5., 44.)]),
    (98, [("虛", 3., 55.), ("斗", 7., 40.), ("井", 12., 32.), ("角", 6., 5.)]),
    (108, [("虛", 4., 58.), ("斗", 7., 21.), ("井", 12., 13.), ("角", 6., 26.)]),
];
const MON: [&str; 4] = ["正", "二", "三", "四"];

fn main() {
    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::DEFAULT).unwrap();
    let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
        .iter()
        .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::DEFAULT).unwrap()))
        .collect();
    let elp = load_all(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let sys = load_lunar_mansion_systems_from_repo().unwrap().into_iter().find(|s| s.id == "qing").unwrap();
    let opts = ApparentPipelineOptions::pipeline_default();
    let xiu_idx = |n: &str| { let n = if n == "虛" { "虚" } else { n }; MANSIONS.iter().position(|m| *m == n).unwrap() };
    // 正月初一癸酉 = JDN 2390600；北京地方平时子正 = UT 2390599.5 − 116.4°/360°；ΔT 约 7 s
    let jd0_ut = 2390599.5 - 116.4 / 360.0;
    println!("日期,曜,书,推算(平),差(平),推算(真),差(真)");
    let mut worst = [[0.0f64; 2]; 4];
    for (dd, row) in BOOK {
        let t = TimePoint::new(TimeScale::TT, real(jd0_ut + dd as f64 + 7.0 / 86400.0));
        let frame = MansionFrame::new(&eph, &cat, &sys, t, &opts).unwrap();
        let mean = chart(&eph, t, Some(&frame), 0, None, &QizhengOptions { node: NodeKind::Mean, ..Default::default() }, &opts);
        let tru = chart(&eph, t, Some(&frame), 0, None, &QizhengOptions { node: NodeKind::True, bei: BeiKind::MeanApogee, ..Default::default() }, &opts);
        let m = if dd < 29 { 0 } else if dd < 59 { 1 } else if dd < 88 { 2 } else { 3 };
        let d0 = [0, 29, 59, 88][m];
        let label = format!("{}月{}", MON[m], ["初一", "十一", "二十一"][((dd - d0) / 10) as usize]);
        for (k, name) in ["月孛", "羅睺", "計都", "紫炁"].iter().enumerate() {
            let (bx, bd, bm) = row[k];
            let book = bd + bm / 60.0;
            let got = |c: &lunar_core::folk::qizheng::QizhengChart| {
                let y = c.yao.iter().find(|y| y.name == *name).unwrap();
                let (xi, du) = y.xiu.unwrap();
                // 与书同宿则比度；异宿则换算到书之宿
                let off = if xi == xiu_idx(bx) { 0.0 } else { f64::NAN };
                (format!("{}{:.2}", MANSIONS[xi], du.as_f64()), du.as_f64() + off)
            };
            let (gm, vm) = got(&mean);
            let (gt, vt) = got(&tru);
            let (em, et) = ((vm - book) * 60.0, (vt - book) * 60.0);
            worst[k][0] = worst[k][0].max(em.abs());
            worst[k][1] = worst[k][1].max(et.abs());
            println!("{label},{name},{bx}{book:.2},{gm},{em:+.0}′,{gt},{et:+.0}′");
        }
    }
    // 七政：正月初一癸酉，书 PDF 第 4 页（索引 3）右半首栏；宫度换黄经（戌宫 0°，逆支每宫 30°），纬度南负北正
    const SEVEN: [(&str, f64, Option<f64>); 7] = [
        ("太陽", 330.0 + 51.0 / 60.0, None),
        ("太陰", 330.0 + 8.0 / 60.0, Some(-(3.0 + 5.0 / 60.0))),
        ("土星", 150.0 + 27.0 + 10.0 / 60.0, Some(2.0 + 36.0 / 60.0)),
        ("木星", 2.0 + 5.0 / 60.0, Some(-(1.0 + 9.0 / 60.0))),
        ("火星", 60.0 + 6.0 + 47.0 / 60.0, Some(2.0 + 2.0 / 60.0)),
        ("金星", 16.0 + 35.0 / 60.0, Some(54.0 / 60.0)),
        ("水星", 300.0 + 26.0 + 24.0 / 60.0, Some(-(1.0 + 33.0 / 60.0))),
    ];
    let t = TimePoint::new(TimeScale::TT, real(jd0_ut + 7.0 / 86400.0));
    let c = chart(&eph, t, None, 0, None, &QizhengOptions::default(), &opts);
    println!("\n七政（正月初一子正）,书黄经,推算,差,书纬,推算纬,差");
    for (name, lon, lat) in SEVEN {
        let y = c.yao.iter().find(|y| y.name == name).unwrap();
        let d = ((y.lon.as_f64() - lon + 540.0).rem_euclid(360.0) - 180.0) * 60.0;
        let (lt, dl) = match (lat, y.lat) {
            (Some(b), Some(g)) => (format!("{:.2}", g.as_f64()), format!("{:+.0}′", (g.as_f64() - b) * 60.0)),
            _ => ("—".into(), "—".into()),
        };
        println!("{name},{lon:.3},{:.3},{d:+.0}′,{},{lt},{dl}", y.lon.as_f64(), lat.map(|b| format!("{b:.2}")).unwrap_or("—".into()));
    }
    // 康熙九年（1670）七政时宪历（台湾图书馆 NCL-06340）正月初一己丑（JDN 2331036）：月孛張七度三十六分、罗睺奎八度五十六分、计都角七度三十四分
    let t = TimePoint::new(TimeScale::TT, real(2331035.5 - 116.4 / 360.0 + 10.0 / 86400.0));
    let frame = MansionFrame::new(&eph, &cat, &sys, t, &opts).unwrap();
    println!("\n康熙九年正月初一,书,罗睺为升交（推算）,计都为升交（推算）");
    for (lj, tag) in [(lunar_core::folk::qizheng::LuoJi::LuoAscending, "罗升"), (lunar_core::folk::qizheng::LuoJi::JiAscending, "计升")] {
        let c = chart(&eph, t, Some(&frame), 0, None, &QizhengOptions { luo_ji: lj, ..Default::default() }, &opts);
        let f = |n: &str| {
            let y = c.yao.iter().find(|y| y.name == n).unwrap();
            let (xi, du) = y.xiu.unwrap();
            format!("{}{:.2}", MANSIONS[xi], du.as_f64())
        };
        println!("{tag},月孛 書張7.60/算{},羅睺 書奎8.93/算{},計都 書角7.57/算{}", f("月孛"), f("羅睺"), f("計都"));
    }
    for (k, name) in ["月孛", "羅睺", "計都", "紫炁"].iter().enumerate() {
        eprintln!("{name}：最大差 平 {:.0}′ 真 {:.0}′", worst[k][0], worst[k][1]);
    }
}
