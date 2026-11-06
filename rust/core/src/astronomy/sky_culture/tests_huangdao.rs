//! 古代黄道宿度的量法检验：史志黄道宿度（data/sky_culture/ecliptic_lodge_widths.csv）与两种模型逐宿比较。
//!
//! - 投影：沿过天极的赤经圈把距星投到黄道（tan λ′ = tan α / cos ε），取相邻投影点的黄道弧长。
//! - 黄经：过黄极的大圆投影，即现代黄经差（清时宪历的量法）。
//!
//! 两者只在距星离黄道时不同，所以离黄道远的距星（如觜、参、井、柳）最能区分。

use crate::astronomy::apparent::{right_ascension_declination, stars_mean_equatorial, ApparentPipelineOptions};
use crate::astronomy::frame::precession::mean_obliquity;
use crate::astronomy::sky_culture::chinese::{load_lunar_mansion_systems_from_repo, MANSIONS};
use crate::astronomy::stars::{load_hipparcos_bright_from_repo, CatalogStar};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};

fn wrap180(x: f64) -> f64 {
    (x + 540.0).rem_euclid(360.0) - 180.0
}

fn rms(v: &[f64]) -> f64 {
    (v.iter().map(|x| x * x).sum::<f64>() / v.len() as f64).sqrt()
}

#[test]
fn ecliptic_lodge_widths_projection_vs_longitude() {
    let lines = crate::repo::read_lines("data/sky_culture/ecliptic_lodge_widths.csv").unwrap();
    let rows: Vec<Vec<String>> = lines.iter().filter(|l| !l.starts_with('#')).skip(1).map(|l| l.split(',').map(String::from).collect()).collect();
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let systems = load_lunar_mansion_systems_from_repo().unwrap();
    let mut tab = crate::test_util::CheckTable::new(
        "ecliptic_lodge_widths",
        &["宿", "距星黄纬°（元）", "唐724 史载", "投影差", "黄经差", "元1280 史载", "投影差", "黄经差"],
    );
    tab.meta("史志黄道宿度（古度）与两种模型逐宿比较，差 = 推算 − 史载：投影为沿赤经圈投到黄道，黄经为过黄极投影（现代黄经差）");
    tab.meta("唐 724 用宋皇祐体系距星、周天 365.25；元 1280 用元体系距星、周天 365.2575；平位置，当日平黄赤交角");
    let mut cells: Vec<Vec<String>> = (0..28).map(|i| vec![MANSIONS[i].to_string()]).collect();
    let mut summary = crate::test_util::CheckTable::new("ecliptic_lodge_widths_summary", &["历", "投影 均方根", "投影 最大差", "黄经 均方根", "黄经 最大差"]);
    summary.meta("史志黄道宿度与两种模型的差（古度）");
    for (col, name, year, sid, zt) in [(1usize, "唐大衍 724", 724.0, "song_huangyou", 365.25), (2, "元授时 1280", 1280.0, "yuan", 365.2575)] {
        let det = systems.iter().find(|s| s.id == sid).unwrap().determinatives;
        let stars: Vec<CatalogStar> = det.iter().map(|h| *cat.by_hip(*h).unwrap()).collect();
        let jd = 2451545.0 + (year - 2000.0) * 365.25;
        let eps = mean_obliquity(real((jd - 2451545.0) / 36525.0)).rad().as_f64();
        let pos = stars_mean_equatorial(&stars, TimePoint::new(TimeScale::TT, real(jd)), &ApparentPipelineOptions::default());
        let ad: Vec<(f64, f64)> = pos.iter().map(|p| { let (a, d) = right_ascension_declination(p); (a.rad().as_f64(), d.rad().as_f64()) }).collect();
        let proj: Vec<f64> = ad.iter().map(|(a, _)| a.sin().atan2(a.cos() * eps.cos()).to_degrees()).collect();
        let lon: Vec<f64> = ad.iter().map(|(a, d)| (a.sin() * eps.cos() + d.tan() * eps.sin()).atan2(a.cos()).to_degrees()).collect();
        let k = zt / 360.0;
        let (mut dp, mut dl) = (vec![], vec![]);
        for i in 0..28 {
            let j = (i + 1) % 28;
            assert_eq!(rows[i][0], MANSIONS[i]);
            let rec: f64 = rows[i][col].parse().unwrap();
            let (p, l) = (wrap180(proj[j] - proj[i]) * k - rec, wrap180(lon[j] - lon[i]) * k - rec);
            dp.push(p);
            dl.push(l);
            if col == 2 {
                let (a, d) = ad[i];
                let beta = (d.sin() * eps.cos() - d.cos() * eps.sin() * a.sin()).asin().to_degrees();
                cells[i].insert(1, format!("{beta:+.1}"));
            }
            cells[i].extend([rows[i][col].clone(), format!("{p:+.2}"), format!("{l:+.2}")]);
        }
        let max = |v: &[f64]| v.iter().fold(0.0f64, |m, x| m.max(x.abs()));
        println!("{name}：投影 均方根 {:.3} 最大 {:.3}；黄经 均方根 {:.3} 最大 {:.3}", rms(&dp), max(&dp), rms(&dl), max(&dl));
        summary.row(vec![name.into(), format!("{:.3}", rms(&dp)), format!("{:.3}", max(&dp)), format!("{:.3}", rms(&dl)), format!("{:.3}", max(&dl))]);
        // 投影须明显优于黄经差；容差按各代宿度分辨率
        assert!(rms(&dp) * 2.0 < rms(&dl), "{name}");
        assert!(max(&dp) < if col == 1 { 2.0 } else { 0.4 }, "{name}");
    }
    for r in cells {
        tab.row(r);
    }
    tab.write();
    summary.write();
}

/// 「斗二十度」「營室九度」「七星初」「氐5.0」→（宿序，度）。宿名繁简与全称皆可；初为 0，少半太为 ¼ ½ ¾。
fn parse_lodge(s: &str) -> Option<(usize, f64)> {
    let mut t = s.trim().to_string();
    for (a, b) in [("南斗", "斗"), ("東井", "井"), ("輿鬼", "鬼"), ("牽牛", "牛"), ("婺女", "女"), ("營室", "室"), ("東壁", "壁"), ("七星", "星"),
        ("虛", "虚"), ("婁", "娄"), ("畢", "毕"), ("參", "参"), ("張", "张"), ("軫", "轸")] {
        t = t.replace(a, b);
    }
    let first = t.chars().next()?;
    let li = MANSIONS.iter().position(|m| m.starts_with(first))?;
    let rest: String = t.chars().skip(1).collect();
    let rest = rest.trim_start_matches('宿');
    if let Ok(v) = rest.parse::<f64>() {
        return Some((li, v));
    }
    if rest.starts_with('初') {
        return Some((li, 0.0));
    }
    let (num, tail) = rest.split_once('度')?;
    let digit = |c: char| "〇一二三四五六七八九".chars().position(|d| d == c);
    let mut v = 0.0;
    let mut cur = 0.0;
    for c in num.chars() {
        match c {
            '十' => { v += if cur == 0.0 { 10.0 } else { cur * 10.0 }; cur = 0.0; }
            '百' => { v += if cur == 0.0 { 100.0 } else { cur * 100.0 }; cur = 0.0; }
            _ => cur = digit(c)? as f64,
        }
    }
    v += cur;
    v += if tail.starts_with('少') { 0.25 } else if tail.starts_with('半') { 0.5 } else if tail.starts_with('太') { 0.75 } else { 0.0 };
    Some((li, v))
}

/// 日食所记日躔（「日有食之，在某宿某度」）：合朔时太阳位置按六种算法与记载比较，以区分各代日躔用赤道度还是黄道度、真太阳还是平行。
///
/// - 赤道（真）：真太阳赤经 − 距星赤经；赤道（平行）：平黄经当作赤道度（「日行一度」沿赤道，冬至点赤经 270° 与平黄经 270° 重合）。
/// - 投影黄道：太阳黄经 − 距星沿赤经圈投到黄道之点（tan λ′ = tan α / cos ε）；真、平行两种太阳。
/// - 黄经：太阳黄经 − 距星真黄经；真、平行两种太阳。
///
/// 差 = 推算 − 记载（古度，周天 365.25），以所记之宿的距星为起点，不论推算落在哪一宿。合朔取地心视黄经相合之时（记载朔日附近）。
/// 古历日躔有各自的冬至点与岁差处理，故整体系统差按历分组另计，判别看去系统差后的离散。
#[test]
#[ignore]
fn eclipse_sun_lodge_degrees() {
    use crate::astronomy::apparent::body_apparent_equatorial;
    use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
    use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
    use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
    use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
    use crate::astronomy::pipeline::{Body, Vsop87Elp, Vsop87Planets};
    use crate::astronomy::time::delta_t::{DeltaTModel, MorrisonStephenson2021};

    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::DEFAULT).unwrap();
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: vec![] }, moon: &elp };
    let opts = ApparentPipelineOptions::default();
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let systems = load_lunar_mansion_systems_from_repo().unwrap();
    let det = systems.iter().find(|s| s.id == "chenzhuo").unwrap().determinatives;
    let stars: Vec<CatalogStar> = det.iter().map(|h| *cat.by_hip(*h).unwrap()).collect();
    const PERIODS: [(i64, i64, &str); 13] = [
        (-220, -105, "颛顼历"), (-104, 84, "太初/三统历"), (85, 220, "后汉四分历"), (221, 618, "其他"), (619, 664, "戊寅元历"),
        (665, 728, "麟德历"), (729, 761, "大衍历"), (762, 783, "五纪历"), (784, 806, "正元历"), (807, 821, "观象历"),
        (822, 892, "宣明历"), (893, 907, "崇玄历"), (908, 2000, "其他"),
    ];
    let names = ["赤道·真", "赤道·平行", "投影黄道·真", "投影黄道·平行", "黄经·真", "黄经·平行"];
    let mut header = vec!["书", "年", "公历", "历", "记载"];
    header.extend(names.iter().copied());
    let mut tab = crate::test_util::CheckTable::new("eclipse_sun_lodges", &header);
    tab.meta("史书日食所记日躔与合朔时太阳位置六种算法逐条比较；差 = 推算 − 记载（古度，周天 365.25，自所记之宿的陈卓距星起量）");
    tab.meta("赤道·平行：平黄经当作赤道度（日行一度沿赤道）；投影黄道：距星沿赤经圈投到黄道；黄经：现代黄经差。历按年份所属行用历分组");
    let mut groups: Vec<(String, Vec<[f64; 6]>)> = vec![];
    let mut n_rec = 0;
    for (book, title) in [("hanshu", "《汉书》"), ("houhanshu", "《后汉书》"), ("xintangshu", "《新唐书》")] {
        let lines = crate::repo::read_lines(&format!("data/phenomena/{book}_solar_eclipses.csv")).unwrap();
        let rows: Vec<&String> = lines.iter().filter(|l| !l.starts_with('#')).collect();
        let head: Vec<&str> = rows[0].split(',').collect();
        let col = |n: &str| head.iter().position(|h| *h == n).unwrap();
        let (cj, cl, cy, cu) = (col("jdn"), col("lodge"), col("year"), col("julian"));
        for r in &rows[1..] {
            let f: Vec<&str> = r.split(',').collect();
            let (Ok(jdn), Some((li, deg))) = (f[cj].parse::<f64>(), parse_lodge(f[cl])) else { continue };
            let year: i64 = f[cy].parse().unwrap();
            // 合朔：自记载日正午（UT）起割线迭代 Δ(视黄经) = 0
            let dt = MorrisonStephenson2021.estimate(real(jdn)).value.seconds().as_f64() / 86400.0;
            let lon_of = |t: f64, b: Body| {
                let p = body_apparent_equatorial(&eph, b, TimePoint::new(TimeScale::TT, real(t)), &opts);
                let (a, d) = right_ascension_declination(&p);
                let (a, d) = (a.rad().as_f64(), d.rad().as_f64());
                let eps = mean_obliquity(real((t - 2451545.0) / 36525.0)).rad().as_f64();
                ((a.sin() * eps.cos() + d.tan() * eps.sin()).atan2(a.cos()).to_degrees(), a.to_degrees(), d)
            };
            let mut t = jdn + dt;
            for _ in 0..6 {
                let diff = wrap180(lon_of(t, Body::Moon).0 - lon_of(t, Body::Sun).0);
                t -= diff / 12.19;
            }
            let (lam, ra, _) = lon_of(t, Body::Sun);
            let tc = (t - 2451545.0) / 36525.0;
            let lmean = 280.46646 + 36000.76983 * tc + 0.0003032 * tc * tc;
            let eps = mean_obliquity(real(tc)).rad().as_f64();
            let pos = stars_mean_equatorial(&stars, TimePoint::new(TimeScale::TT, real(t)), &opts);
            let (a0, d0) = right_ascension_declination(&pos[li]);
            let (a0, d0) = (a0.rad().as_f64(), d0.rad().as_f64());
            let det_ra = a0.to_degrees();
            let det_proj = a0.sin().atan2(a0.cos() * eps.cos()).to_degrees();
            let det_lon = (a0.sin() * eps.cos() + d0.tan() * eps.sin()).atan2(a0.cos()).to_degrees();
            let k = 365.25 / 360.0;
            let res = |x: f64, base: f64| wrap180(x - base) * k - deg;
            let v = [res(ra, det_ra), res(lmean, det_ra), res(lam, det_proj), res(lmean, det_proj), res(lam, det_lon), res(lmean, det_lon)];
            let per = PERIODS.iter().find(|p| p.0 <= year && year <= p.1).map(|p| p.2).unwrap_or("其他");
            let mut row = vec![title.to_string(), year.to_string(), f[cu].to_string(), per.to_string(), f[cl].to_string()];
            row.extend(v.iter().map(|x| format!("{x:+.2}")));
            tab.row(row);
            match groups.iter_mut().find(|g| g.0 == per) {
                Some(g) => g.1.push(v),
                None => groups.push((per.to_string(), vec![v])),
            }
            n_rec += 1;
        }
    }
    tab.write();
    let mut sum_header = vec!["历", "条数"];
    let cols: Vec<String> = names.iter().flat_map(|n| [format!("{n} 系统差"), format!("{n} 离散")]).collect();
    sum_header.extend(cols.into_iter().map(|s| &*Box::leak(s.into_boxed_str())));
    let mut summary = crate::test_util::CheckTable::new("eclipse_sun_lodges_summary", &sum_header);
    summary.meta("按行用历分组：系统差为中位数，离散为去中位数后的中位绝对差（古度）；离散最小者最可能是该历日躔的算法");
    let med = |mut v: Vec<f64>| { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); let n = v.len(); if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 } };
    for (per, v) in &groups {
        let mut row = vec![per.clone(), v.len().to_string()];
        let mut line = String::new();
        for m in 0..6 {
            let c: Vec<f64> = v.iter().map(|x| x[m]).collect();
            let s = med(c.clone());
            let disp = med(c.iter().map(|x| (x - s).abs()).collect());
            row.extend([format!("{s:+.2}"), format!("{disp:.2}")]);
            line += &format!(" {}:{s:+.2}/{disp:.2}", names[m]);
        }
        println!("{per} {}条{line}", v.len());
        // 十条以上的组：最佳算法的离散须明显小于真黄经（距星黄纬使黄经差与古代度数系统不合）
        if v.len() >= 10 {
            let disp = |m: usize| { let c: Vec<f64> = v.iter().map(|x| x[m]).collect(); let s = med(c.clone()); med(c.iter().map(|x| (x - s).abs()).collect()) };
            let best = (0..4).map(disp).fold(f64::INFINITY, f64::min);
            assert!(best * 1.4 < disp(4).min(disp(5)), "{per}");
        }
        summary.row(row);
    }
    summary.write();
    assert!(n_rec > 100, "{n_rec}");
}

/// 月食实测宿度（data/phenomena/lunar_eclipse_lodges.csv，何承天、祖冲之、梁大同、唐麟德开元所引）：望时月亮位置按赤道、
/// 投影黄道、真黄经三种量法与记载比较。月食时月在黄道附近（|β| < 1.5°），三种量法的差主要来自距星黄纬。
/// 望取地心视黄经相冲之时（记载之日附近），与食既时刻相差一两小时，月行约 1 度，计入误差。
#[test]
#[ignore]
fn lunar_eclipse_lodge_observations() {
    use crate::astronomy::apparent::body_apparent_equatorial;
    use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
    use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
    use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
    use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
    use crate::astronomy::pipeline::{Body, Vsop87Elp, Vsop87Planets};

    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::DEFAULT).unwrap();
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: vec![] }, moon: &elp };
    let opts = ApparentPipelineOptions::default();
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let det = load_lunar_mansion_systems_from_repo().unwrap().into_iter().find(|s| s.id == "chenzhuo").unwrap().determinatives;
    let stars: Vec<CatalogStar> = det.iter().map(|h| *cat.by_hip(*h).unwrap()).collect();
    let lines = crate::repo::read_lines("data/phenomena/lunar_eclipse_lodges.csv").unwrap();
    let mut tab = crate::test_util::CheckTable::new("lunar_eclipse_lodges", &["年", "记载", "计入", "月黄纬°", "赤道 差", "投影黄道 差", "黄经 差", "原文"]);
    tab.meta("唐以前史志月食实测宿度与望时月亮位置：差 = 推算 − 记载（古度，周天 365.25，自所记之宿的陈卓距星起量）");
    let mut used: Vec<[f64; 3]> = vec![];
    for l in lines.iter().filter(|l| !l.starts_with('#')).skip(1) {
        let f: Vec<&str> = l.splitn(7, ',').collect();
        let jdn: f64 = f[1].parse().unwrap();
        let li = MANSIONS.iter().position(|m| *m == f[2]).unwrap();
        let deg: f64 = f[3].parse().unwrap();
        let ok = f[4] == "1";
        let lon_of = |t: f64, b: Body| {
            let p = body_apparent_equatorial(&eph, b, TimePoint::new(TimeScale::TT, real(t)), &opts);
            let (a, d) = right_ascension_declination(&p);
            let (a, d) = (a.rad().as_f64(), d.rad().as_f64());
            let eps = mean_obliquity(real((t - 2451545.0) / 36525.0)).rad().as_f64();
            ((a.sin() * eps.cos() + d.tan() * eps.sin()).atan2(a.cos()).to_degrees(), a, d)
        };
        let mut t = jdn;
        for _ in 0..8 {
            let diff = wrap180(lon_of(t, Body::Moon).0 - lon_of(t, Body::Sun).0 - 180.0);
            t -= diff / 12.19;
        }
        let (lam, a, d) = lon_of(t, Body::Moon);
        let tc = (t - 2451545.0) / 36525.0;
        let eps = mean_obliquity(real(tc)).rad().as_f64();
        let beta = (d.sin() * eps.cos() - d.cos() * eps.sin() * a.sin()).asin().to_degrees();
        let pos = stars_mean_equatorial(&stars, TimePoint::new(TimeScale::TT, real(t)), &opts);
        let (a0, d0) = right_ascension_declination(&pos[li]);
        let (a0, d0) = (a0.rad().as_f64(), d0.rad().as_f64());
        let k = 365.25 / 360.0;
        let proj = a.sin().atan2(a.cos() * eps.cos()).to_degrees();
        let r = [
            wrap180(a.to_degrees() - a0.to_degrees()) * k - deg,
            wrap180(proj - a0.sin().atan2(a0.cos() * eps.cos()).to_degrees()) * k - deg,
            wrap180(lam - (a0.sin() * eps.cos() + d0.tan() * eps.sin()).atan2(a0.cos()).to_degrees()) * k - deg,
        ];
        assert!((t - jdn).abs() < 2.0, "{l}: 望距记载日 {:.2}", t - jdn);
        println!("{} {}{} 赤道 {:+.2} 投影 {:+.2} 黄经 {:+.2} β {beta:+.2}", f[0], f[2], f[3], r[0], r[1], r[2]);
        tab.row(vec![f[0].into(), format!("{}{}", f[2], f[3]), if ok { "是".into() } else { "否".into() }, format!("{beta:+.2}"),
            format!("{:+.2}", r[0]), format!("{:+.2}", r[1]), format!("{:+.2}", r[2]), f[6].replace(',', "，")]);
        if ok {
            used.push(r);
        }
    }
    tab.write();
    let names = ["赤道", "投影黄道", "黄经"];
    let mut summary = crate::test_util::CheckTable::new("lunar_eclipse_lodges_summary", &["量法", "条数", "平均差", "中位绝对差", "均方根"]);
    summary.meta("月食实测宿度汇总（古度；只计「计入」者）");
    for m in 0..3 {
        let v: Vec<f64> = used.iter().map(|r| r[m]).collect();
        let mut a: Vec<f64> = v.iter().map(|x| x.abs()).collect();
        a.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        println!("{}：平均 {mean:+.2} 中位绝对 {:.2} 均方根 {:.2}", names[m], a[a.len() / 2], rms(&v));
        summary.row(vec![names[m].into(), v.len().to_string(), format!("{mean:+.2}"), format!("{:.2}", a[a.len() / 2]), format!("{:.2}", rms(&v))]);
        // 实测宿度按赤道度应无系统差、散布在实测精度内
        if m == 0 {
            assert!(mean.abs() < 1.0 && rms(&v) < 3.0, "赤道：平均 {mean:.2} 均方根 {:.2}", rms(&v));
        }
    }
    summary.write();
}
