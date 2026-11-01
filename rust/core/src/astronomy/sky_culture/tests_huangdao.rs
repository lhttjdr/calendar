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
