//! 石氏星表度数检验：《开元占经》四库本所载石氏中外官入宿度、去极度与二十八宿距星去极度（data/sky_culture/shishi_xingjing.csv），
//! 认星后与 Hipparcos 推算的平位置（当日平赤道，空间运动 + 岁差）比较，并按岁差估观测年代。
//!
//! - 认星：二十八宿用陈卓体系距星；中外官的候选为其所对陈卓星官（data/sky_culture/sanjia_schools.csv，拆分者为诸官之并）的成员。
//!   单星者即是；注文言方位（东西南北，可复合）者取该方向最远之星；言「大星」者取最亮星。其余（「柄星」「門右星」、无限定语的多星
//!   星官等）取在参考年代与记载最近之星，记为「就近」，只列不用：以记载选星再用其估年代会循环。
//! - 年代：前 800 至公元 400 年每 5 年，求入宿度、去极度残差（记载 − 推算，度；古度按 360/365.25 换算）。两量各允许一个整体系统差
//!   （如仪器零点），各自去均值。先以中位绝对偏差（稳健）取初值并剔除离群（偏离中位数超过 3 倍稳健标准差 1.4826·MAD），
//!   再求保留条目去均值残差平方和随年代的极小，三点抛物线细化。整个流程对记录作 1000 次 bootstrap 重抽样，给 16%—84% 区间。
//! - 入宿度依赖所入之宿的距星赤经，用陈卓体系距星；二十八宿本身的入宿度恒为 0，不参与。

use std::collections::HashMap;

use crate::astronomy::apparent::{right_ascension_declination, stars_mean_equatorial, ApparentPipelineOptions};
use crate::astronomy::sky_culture::chinese::{
    load_asterisms_from_repo, load_lunar_mansion_systems_from_repo, load_school_attributions_from_repo, AsterismMember, MANSIONS,
};
use crate::astronomy::stars::{load_hipparcos_bright_from_repo, CatalogStar};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};

const GU: f64 = 360.0 / 365.25;

#[derive(Clone, Debug)]
struct Rec {
    reference: String,
    title: String,
    qualifier: String,
    lodge: usize,
    ru_du: f64,
    qu_ji: f64,
    is_lodge: bool,
}

fn load_records() -> Vec<Rec> {
    let lines = crate::repo::read_lines("data/sky_culture/shishi_xingjing.csv").unwrap();
    lines
        .iter()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            let lodge = MANSIONS.iter().position(|m| *m == f[3]).unwrap_or_else(|| panic!("宿名 {}", f[3]));
            Rec {
                reference: f[0].into(),
                title: f[1].into(),
                qualifier: f[2].into(),
                lodge,
                ru_du: f[4].parse().unwrap(),
                qu_ji: f[5].parse().unwrap(),
                is_lodge: f[0].contains("二十八宿"),
            }
        })
        .collect()
}

/// 某年（天文纪年，年中）一批星的平赤经、赤纬（度）。
fn positions(stars: &[CatalogStar], year: f64) -> HashMap<u32, (f64, f64)> {
    let jd = 2451545.0 + (year - 2000.0) * 365.25;
    let t = TimePoint::new(TimeScale::TT, real(jd));
    stars_mean_equatorial(stars, t, &ApparentPipelineOptions::default())
        .iter()
        .zip(stars)
        .map(|(p, s)| {
            let (a, d) = right_ascension_declination(p);
            (s.hip, (a.rad().as_f64().to_degrees().rem_euclid(360.0), d.rad().as_f64().to_degrees()))
        })
        .collect()
}

fn wrap180(x: f64) -> f64 {
    (x + 540.0).rem_euclid(360.0) - 180.0
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Method {
    Determinative,
    Single,
    Direction,
    Brightest,
    Nearest,
}

impl Method {
    fn label(self) -> &'static str {
        match self {
            Method::Determinative => "距星（陈卓）",
            Method::Single => "单星",
            Method::Direction => "方位",
            Method::Brightest => "大星",
            Method::Nearest => "就近（不用）",
        }
    }
    fn used(self) -> bool {
        self != Method::Nearest
    }
}

/// 残差（度）：(入宿度差, 去极度差)；二十八宿入宿度差为 None。
fn residual(r: &Rec, hip: u32, det: &[u32; 28], pos: &HashMap<u32, (f64, f64)>) -> (Option<f64>, f64) {
    let (ra, dec) = pos[&hip];
    let dq = r.qu_ji * GU - (90.0 - dec);
    let dr = if r.is_lodge { None } else { Some(wrap180(r.ru_du * GU - (ra - pos[&det[r.lodge]].0).rem_euclid(360.0))) };
    (dr, dq)
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

/// 离散度：残差减中位数后的中位绝对偏差（允许整体系统差）。
fn spread(v: &[f64]) -> f64 {
    let mut a = v.to_vec();
    let m = median(&mut a);
    let mut d: Vec<f64> = v.iter().map(|x| (x - m).abs()).collect();
    median(&mut d)
}

#[test]
fn shishi_catalog_epoch() {
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let recs = load_records();
    let det: [u32; 28] = load_lunar_mansion_systems_from_repo().unwrap().into_iter().find(|s| s.id == "chenzhuo").unwrap().determinatives;
    let attr = load_school_attributions_from_repo().unwrap();
    let asterisms: HashMap<String, Vec<u32>> = load_asterisms_from_repo()
        .unwrap()
        .into_iter()
        .filter(|a| a.system == "chenzhuo")
        .map(|a| (a.id.clone(), a.members.iter().filter_map(|m| if let AsterismMember::Hip(h) = m { Some(*h) } else { None }).collect()))
        .collect();
    // 条目 → 候选星
    let candidates = |r: &Rec| -> Vec<u32> {
        if r.is_lodge {
            return vec![det[r.lodge]];
        }
        let mut c: Vec<u32> =
            attr.iter().filter(|a| a.source == r.reference).flat_map(|a| asterisms[&a.id].iter().copied()).filter(|h| cat.by_hip(*h).is_some()).collect();
        c.sort();
        c.dedup();
        c
    };
    let mut all: Vec<u32> = recs.iter().flat_map(|r| candidates(r)).chain(det.iter().copied()).collect();
    all.sort();
    all.dedup();
    let stars: Vec<CatalogStar> = all.iter().map(|h| *cat.by_hip(*h).unwrap()).collect();

    // 认星（方位、亮度在参考年代判断；星官形状数百年间几乎不变）
    const REF_YEAR: f64 = -200.0;
    let pos_ref = positions(&stars, REF_YEAR);
    let ident: Vec<Option<(u32, Method)>> = recs
        .iter()
        .map(|r| {
            let c = candidates(r);
            if c.is_empty() {
                return None;
            }
            if r.is_lodge {
                return Some((c[0], Method::Determinative));
            }
            if c.len() == 1 {
                return Some((c[0], Method::Single));
            }
            let q = &r.qualifier;
            let simple = !q.contains('第') && !q.contains('柄') && !q.contains('門') && !q.contains('台') && !q.contains('前');
            let dx = if q.contains('西') { -1.0 } else if q.contains('東') { 1.0 } else { 0.0 };
            let dy = if q.contains('北') { 1.0 } else if q.contains('南') { -1.0 } else { 0.0 };
            if simple && (dx != 0.0 || dy != 0.0) {
                let (ra0, de0) = pos_ref[&c[0]];
                let score = |h: &u32| {
                    let (ra, de) = pos_ref[h];
                    dx * wrap180(ra - ra0) * de0.to_radians().cos() + dy * (de - de0)
                };
                let best = *c.iter().max_by(|a, b| score(a).partial_cmp(&score(b)).unwrap()).unwrap();
                return Some((best, Method::Direction));
            }
            if simple && q.contains('大') {
                let best = *c.iter().min_by(|a, b| cat.by_hip(**a).unwrap().vmag.partial_cmp(&cat.by_hip(**b).unwrap().vmag).unwrap()).unwrap();
                return Some((best, Method::Brightest));
            }
            let cost = |h: &u32| {
                let (dr, dq) = residual(r, *h, &det, &pos_ref);
                dr.unwrap_or(0.0).powi(2) + dq.powi(2)
            };
            Some((*c.iter().min_by(|a, b| cost(a).partial_cmp(&cost(b)).unwrap()).unwrap(), Method::Nearest))
        })
        .collect();

    // 历元扫描
    let years: Vec<f64> = (0..=240).map(|k| -800.0 + 5.0 * k as f64).collect();
    let scan: Vec<HashMap<u32, (f64, f64)>> = years.iter().map(|y| positions(&stars, *y)).collect();
    let used: Vec<usize> = (0..recs.len()).filter(|&i| ident[i].is_some_and(|(_, m)| m.used())).collect();
    // 残差分组：(去极度残差, 入宿度残差)；k = 0 只用去极度，1 只用入宿度，2 两者（各去均值）
    let groups = |idx: &[usize], k: usize, pos: &HashMap<u32, (f64, f64)>| -> [Vec<(usize, f64)>; 2] {
        let (mut q, mut r) = (vec![], vec![]);
        for &i in idx {
            let (h, _) = ident[i].unwrap();
            let (dr, dq) = residual(&recs[i], h, &det, pos);
            if k != 1 {
                q.push((i, dq));
            }
            if k != 0 {
                if let Some(x) = dr {
                    r.push((i, x));
                }
            }
        }
        [q, r]
    };
    let ss = |g: &[(usize, f64)], keep: &dyn Fn(usize, usize) -> bool, gi: usize| -> (f64, usize) {
        let v: Vec<f64> = g.iter().filter(|(i, _)| keep(gi, *i)).map(|(_, x)| *x).collect();
        if v.is_empty() {
            return (0.0, 0);
        }
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - m).powi(2)).sum(), v.len())
    };
    // 返回 (最佳年代, 均方根, 保留条数, 剔除集)
    let fit = |idx: &[usize], k: usize| -> (f64, f64, usize, Vec<(usize, usize)>) {
        // 初值：两组 MAD 之和最小
        let mut y0 = (0usize, f64::INFINITY);
        for (n, pos) in scan.iter().enumerate() {
            let g = groups(idx, k, pos);
            let s: f64 = g.iter().filter(|x| !x.is_empty()).map(|x| spread(&x.iter().map(|p| p.1).collect::<Vec<_>>())).sum();
            if s < y0.1 {
                y0 = (n, s);
            }
        }
        // 剔除离群（按初值年代）
        let g0 = groups(idx, k, &scan[y0.0]);
        let mut out: Vec<(usize, usize)> = vec![];
        for (gi, g) in g0.iter().enumerate() {
            if g.is_empty() {
                continue;
            }
            let v: Vec<f64> = g.iter().map(|p| p.1).collect();
            let mut c = v.clone();
            let med = median(&mut c);
            let sig = 1.4826 * spread(&v);
            out.extend(g.iter().filter(|p| (p.1 - med).abs() > 3.0 * sig).map(|p| (gi, p.0)));
        }
        let keep = |gi: usize, i: usize| !out.contains(&(gi, i));
        let cost: Vec<(f64, usize)> = scan
            .iter()
            .map(|pos| {
                let g = groups(idx, k, pos);
                let (a, na) = ss(&g[0], &keep, 0);
                let (b, nb) = ss(&g[1], &keep, 1);
                (a + b, na + nb)
            })
            .collect();
        let n = (0..cost.len()).min_by(|a, b| cost[*a].0.partial_cmp(&cost[*b].0).unwrap()).unwrap();
        let mut y = years[n];
        if n > 0 && n + 1 < cost.len() {
            let (a, b, c) = (cost[n - 1].0, cost[n].0, cost[n + 1].0);
            let den = a - 2.0 * b + c;
            if den > 0.0 {
                y += 5.0 * (a - c) / (2.0 * den);
            }
        }
        let used_n = cost[n].1;
        (y, (cost[n].0 / used_n.max(1) as f64).sqrt(), used_n, out)
    };
    // bootstrap：确定性线性同余伪随机
    let mut seed: u64 = 20261031;
    let mut rand = move |n: usize| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % n
    };
    let mut summary = crate::test_util::CheckTable::new("shishi_epoch", &["量", "条数", "最佳年代", "16%—84%", "均方根（去系统差、剔除离群后）°", "系统差（均值）°"]);
    summary.meta("石氏星表（《开元占经》四库本）按岁差估观测年代：认星依据见逐条表，「就近」者不用；两量各去系统差，剔除偏离中位数 3 倍稳健标准差者");
    summary.meta("年代为天文纪年（0 = 前 1 年），前 800 至公元 400 年每 5 年扫描、抛物线细化；区间为对记录 1000 次 bootstrap 重抽样的最佳年代分位数");
    let mut best_all = 0.0;
    let mut outliers: Vec<(usize, usize)> = vec![];
    // 末行为稳健性检验：只用认星最确定的距星、单星（只含去极度，因距星无入宿度、单星不足以单独定入宿）
    for (k, name, strict) in [(0usize, "去极度", false), (1, "入宿度", false), (2, "合（两量各去系统差）", false), (0, "去极度（仅距星、单星）", true)] {
        let idx: Vec<usize> = used
            .iter()
            .copied()
            .filter(|&i| k != 1 || !recs[i].is_lodge)
            .filter(|&i| !strict || matches!(ident[i].unwrap().1, Method::Determinative | Method::Single))
            .collect();
        let (y, rms, n_used, out) = fit(&idx, k);
        let mut boots: Vec<f64> = (0..1000)
            .map(|_| {
                let pick: Vec<usize> = (0..idx.len()).map(|_| idx[rand(idx.len())]).collect();
                fit(&pick, k).0
            })
            .collect();
        boots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (lo, hi) = (boots[160], boots[840]);
        // 系统差：最近扫描年代上保留条目的均值
        let near = (0..years.len()).min_by(|a, b| (years[*a] - y).abs().partial_cmp(&(years[*b] - y).abs()).unwrap()).unwrap();
        let g = groups(&idx, k, &scan[near]);
        let mean = |gi: usize| {
            let v: Vec<f64> = g[gi].iter().filter(|p| !out.contains(&(gi, p.0))).map(|p| p.1).collect();
            if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
        };
        let sys = match k {
            0 => format!("{:+.3}", mean(0)),
            1 => format!("{:+.3}", mean(1)),
            _ => format!("去极 {:+.3}；入宿 {:+.3}", mean(0), mean(1)),
        };
        println!("{name}：{} 条（残差 {n_used} 个，剔除 {}），最佳年代 {y:.0}（{lo:.0}—{hi:.0}），均方根 {rms:.3}°，系统差 {sys}", idx.len(), out.len());
        summary.row(vec![name.into(), format!("{}（剔除 {}）", idx.len(), out.len()), format!("{y:.0}"), format!("{lo:.0}—{hi:.0}"), format!("{rms:.3}"), sys]);
        if k == 2 {
            best_all = y;
            outliers = out;
        }
    }
    summary.write();

    // 逐条（取合并最佳年代）
    let near_all = (0..years.len()).min_by(|a, b| (years[*a] - best_all).abs().partial_cmp(&(years[*b] - best_all).abs()).unwrap()).unwrap();
    let pos = &scan[near_all];
    let best_all = years[near_all];
    let mut tab = crate::test_util::CheckTable::new(
        "shishi_xingjing",
        &["条目", "星官", "所测", "认星 HIP", "认星依据", "入宿", "记载入宿度", "推算", "差°", "记载去极度", "推算", "差°", "剔除"],
    );
    tab.meta(format!("石氏星表逐条（《开元占经》四库本）：记载与推算（平位置，{best_all} 年，即合并最佳年代）；度数为古度，差为度（记载 − 推算）"));
    tab.meta("认星：二十八宿用陈卓体系距星；中外官按注文方位、「大星」或单星认定；「就近」者为在参考年代与记载最近之星，不参与估年代");
    let mut n_none = 0;
    for (i, r) in recs.iter().enumerate() {
        let Some((h, m)) = ident[i] else {
            n_none += 1;
            tab.row(vec![r.reference.clone(), r.title.clone(), r.qualifier.clone(), "—".into(), "无候选星".into(), MANSIONS[r.lodge].into(),
                format!("{:.2}", r.ru_du), "".into(), "".into(), format!("{:.2}", r.qu_ji), "".into(), "".into(), "".into()]);
            continue;
        };
        let (ra, dec) = pos[&h];
        let calc_ru = (ra - pos[&det[r.lodge]].0).rem_euclid(360.0) / GU;
        let calc_qj = (90.0 - dec) / GU;
        let (dr, dq) = residual(r, h, &det, pos);
        tab.row(vec![
            r.reference.clone(),
            r.title.clone(),
            r.qualifier.clone(),
            h.to_string(),
            m.label().into(),
            MANSIONS[r.lodge].into(),
            format!("{:.2}", r.ru_du),
            if r.is_lodge { "".into() } else { format!("{calc_ru:.2}") },
            dr.map(|x| format!("{x:+.2}")).unwrap_or_default(),
            format!("{:.2}", r.qu_ji),
            format!("{calc_qj:.2}"),
            format!("{dq:+.2}"),
            [(1usize, "入宿"), (0, "去极")].iter().filter(|(gi, _)| outliers.contains(&(*gi, i))).map(|(_, n)| *n).collect::<Vec<_>>().join("、"),
        ]);
    }
    tab.write();
    let count = |m: Method| ident.iter().filter(|x| x.is_some_and(|(_, mm)| mm == m)).count();
    println!(
        "认星：距星 {} 单星 {} 方位 {} 大星 {} 就近 {} 无候选 {n_none}",
        count(Method::Determinative),
        count(Method::Single),
        count(Method::Direction),
        count(Method::Brightest),
        count(Method::Nearest)
    );
    assert!(used.len() >= 60, "可用于估年代的条目太少：{}", used.len());
}
