//! 《古今圖書集成》卷五百八十四、五百八十五「鄭氏星案」命盘核算（转录见 data/folk/zheng_xing_an.csv，检验见 doc/folk/qizheng.md §4.8.1）。
//! 每盘只给四柱：年柱（立春换年）、月柱（交节换月）、日柱、时柱。在 1100—1400 年间逐日找出四柱全合之日，取时支正中
//! （东八区民用时，子正 0 时），按本项目缺省七政四余（中气分宫、计都升交、真交点、平远地点、卯上起命、量天尺自宫尾）
//! 求命宫、命度与诸曜，与盘中所书命度、书中明言之曜位比较；每盘列 1100—1400 间全部候选日，「最近」取书言曜位合者最多、次命度同宿度差最小者。距星：1270 年前宋皇祐、此后元（郭守敬）系统。
//! 用法（仓库根）：cargo run --release -p lunar-core --no-default-features --features real-f64 --example zheng_xing_an
use lunar_core::astronomy::apparent::ApparentPipelineOptions;
use lunar_core::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use lunar_core::astronomy::ephemeris::vsop87_patch::PatchTarget;
use lunar_core::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
use lunar_core::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use lunar_core::astronomy::pipeline::{Planet, Vsop87Elp, Vsop87Planets};
use lunar_core::astronomy::sky_culture::chinese::{load_lunar_mansion_systems_from_repo, MansionFrame, MANSIONS};
use lunar_core::astronomy::stars::load_hipparcos_bright_from_repo;
use lunar_core::astronomy::time::{TimePoint, TimeScale};
use lunar_core::folk::qizheng::{chart, QizhengChart, QizhengOptions};
use lunar_core::math::real::{real, RealOps};

const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";
/// 繁体宿名 → MANSIONS（简体）
fn xiu_idx(n: &str) -> Option<usize> {
    let n = match n {
        "虛" => "虚",
        "婁" => "娄",
        "畢" => "毕",
        "參" => "参",
        "張" => "张",
        "軫" => "轸",
        x => x,
    };
    MANSIONS.iter().position(|m| *m == n)
}
fn gz(s: &str) -> (usize, usize) {
    let c: Vec<char> = s.chars().collect();
    (GAN.chars().position(|x| x == c[0]).unwrap(), ZHI.chars().position(|x| x == c[1]).unwrap())
}
/// 儒略历日期之儒略日（正午）
fn jdn_julian(y: i32, m: i32, d: i32) -> i64 {
    let a = (14 - m) / 12;
    let yy = y as i64 + 4800 - a as i64;
    let mm = (m + 12 * a - 3) as i64;
    d as i64 + (153 * mm + 2) / 5 + 365 * yy + yy / 4 - 32083
}
fn julian_of_jdn(j: i64) -> (i64, i64, i64) {
    let c = j + 32082;
    let d = (4 * c + 3) / 1461;
    let e = c - 1461 * d / 4;
    let m = (5 * e + 2) / 153;
    (d - 4800 + m / 10, m + 3 - 12 * (m / 10), e - (153 * m + 2) / 5 + 1)
}
const YAO_CH: [char; 11] = ['日', '月', '水', '金', '火', '木', '土', '羅', '計', '孛', '炁'];

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
    let systems = load_lunar_mansion_systems_from_repo().unwrap();
    let opts = ApparentPipelineOptions::pipeline_default();
    let csv = std::fs::read_to_string("data/folk/zheng_xing_an.csv").unwrap();
    println!("序|标题|四柱|书命度|候选（日期 命宫 命度；书言曜位 合/不合）|最近");
    let (mut n_ming, mut n_claim, mut ok_claim) = (0, 0, 0);
    for l in csv.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let f: Vec<&str> = l.split(',').collect();
        let (no, title) = (f[0], f[3]);
        let (yg, yz) = gz(f[5]);
        let (_, mz) = gz(f[6]);
        let (dg, dz) = gz(f[7]);
        let (_, hz) = gz(f[8]);
        let book_ming = f[9].trim_end_matches('?');
        let bm_xiu: String = book_ming.chars().take_while(|c| !c.is_ascii_digit()).collect();
        let bm_deg: f64 = book_ming[bm_xiu.len()..].parse().unwrap();
        let claims: Vec<&str> = f[10].split(' ').filter(|s| !s.is_empty()).collect();
        let mut cands = Vec::new();
        for y in 1100..=1400 {
            if (y - 4i32).rem_euclid(10) as usize != yg || (y - 4i32).rem_euclid(12) as usize != yz {
                continue;
            }
            // 立春（约二月四日）起至次年立春前
            let j0 = jdn_julian(y, 1, 20);
            for j in j0..j0 + 380 {
                let g60 = (j + 49).rem_euclid(60) as usize;
                if g60 % 10 != dg || g60 % 12 != dz {
                    continue;
                }
                // 时支正中（东八区），子时取当日 0 时
                let jd_ut = j as f64 - 0.5 + (2.0 * hz as f64 - 8.0) / 24.0;
                let t = TimePoint::new(TimeScale::UT1, real(jd_ut)).to_scale(TimeScale::TT);
                let sid = if y < 1270 { "song_huangyou" } else { "yuan" };
                let sys = systems.iter().find(|s| s.id == sid).unwrap();
                let frame = MansionFrame::new(&eph, &cat, sys, t, &opts).unwrap();
                let c = chart(&eph, t, Some(&frame), hz as u8, None, &QizhengOptions::default(), &opts);
                let sun = c.yao[0].lon.as_f64();
                // 节月：寅月始于黄经 315°
                let month_zhi = (((sun - 315.0).rem_euclid(360.0) / 30.0).floor() as usize + 2) % 12;
                // 年柱：立春前属上一年
                let (cy, cm, _) = julian_of_jdn(j);
                let gy = if (sun - 315.0).rem_euclid(360.0) >= 300.0 && cm <= 3 { cy as i32 - 1 } else { cy as i32 };
                if gy != y || month_zhi != mz {
                    continue;
                }
                cands.push((j, y, c));
            }
        }
        let desc = |c: &QizhengChart| {
            let (xi, du) = c.xing_ming.ming_xiu.unwrap();
            (xi, du.as_f64())
        };
        let mut best: Option<(f64, usize)> = None;
        let mut parts = Vec::new();
        for (i, (j, _y, c)) in cands.iter().enumerate() {
            let (xi, du) = desc(c);
            let (yy, mm, dd) = julian_of_jdn(*j);
            // 命度差：同宿则度差，异宿以黄经差计（命度为量天尺推得之宿度，此处取宿名是否同 + 度差）
            let same = xiu_idx(&bm_xiu) == Some(xi);
            let mut cl = Vec::new();
            for cm in &claims {
                let ch: Vec<char> = cm.chars().collect();
                let k = YAO_CH.iter().position(|&x| x == ch[0]).unwrap();
                let y0 = &c.yao[k];
                let rest: String = ch[2..].iter().collect();
                let good = match ch[1] {
                    '@' => xiu_idx(&rest) == y0.xiu.map(|x| x.0),
                    ':' => ZHI.chars().position(|x| x == ch[2]) == Some(y0.gong as usize),
                    _ => {
                        if rest == "命" {
                            y0.gong == c.ming
                        } else {
                            let k2 = YAO_CH.iter().position(|&x| x == ch[2]).unwrap();
                            y0.gong == c.yao[k2].gong
                        }
                    }
                };
                let got = format!(
                    "{}{}{:.0}{}",
                    ch[0],
                    y0.xiu.map(|x| MANSIONS[x.0]).unwrap_or("?"),
                    y0.xiu.map(|x| x.1.as_f64()).unwrap_or(0.0),
                    &ZHI[y0.gong as usize * 3..y0.gong as usize * 3 + 3]
                );
                cl.push(format!("{cm}{}({got})", if good { "合" } else { "否" }));
            }
            // 取法：书言曜位合者多者优先，次取命度同宿而度差小者
            let fails = cl.iter().filter(|x| x.contains("否(")).count() as f64;
            let score = fails * 1000.0 + if same { (du - bm_deg).abs() } else { 100.0 + (du - bm_deg).abs() };
            if best.map_or(true, |b| score < b.0) {
                best = Some((score, i));
            }
            parts.push(format!(
                "{yy}-{mm:02}-{dd:02} 命{}{}{:.1}{}",
                &ZHI[c.ming as usize * 3..c.ming as usize * 3 + 3],
                MANSIONS[xi],
                du,
                if cl.is_empty() { String::new() } else { format!(" {}", cl.join(" ")) }
            ));
        }
        let bs = best.map(|(s, i)| {
            let (j, _, c) = &cands[i];
            let (yy, mm, dd) = julian_of_jdn(*j);
            let (xi, du) = desc(c);
            let same = s.rem_euclid(1000.0) < 100.0;
            if same {
                n_ming += 1;
            }
            let mut cl_ok = 0;
            for cm in &claims {
                if parts[i].contains(&format!("{cm}合")) {
                    cl_ok += 1;
                }
            }
            n_claim += claims.len();
            ok_claim += cl_ok;
            let pos: Vec<String> = c
                .yao
                .iter()
                .enumerate()
                .map(|(k, y)| format!("{}{}{:.0}", YAO_CH[k], y.xiu.map(|x| MANSIONS[x.0]).unwrap_or("?"), y.xiu.map(|x| x.1.as_f64()).unwrap_or(0.0)))
                .collect();
            format!("{yy}-{mm:02}-{dd:02} 命{}{:.1}（差{:+.1}°）书言曜位{cl_ok}/{}｜诸曜 {}", MANSIONS[xi], du, if same { du - bm_deg } else { f64::NAN }, claims.len(), pos.join(" "))
        });
        println!("{no}|{title}|{} {} {} {}|{}|{}|{}", f[5], f[6], f[7], f[8], f[9], parts.join("；"), bs.unwrap_or_default());
    }
    println!("命度同宿 {n_ming}/40；书言曜位（取合者最多、次命度最近之候选）合 {ok_claim}/{n_claim}");
}
