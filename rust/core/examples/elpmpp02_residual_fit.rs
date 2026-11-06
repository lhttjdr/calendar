//! ELP/MPP02 残差改正拟合：在某目标的内置常数集（DE406 为论文常数集，DE431、DE441 为本项目两步法所拟）之上，
//! 对 J2000 平黄道下月亮黄经、黄纬、距离的残差 ρ = 目标 − ELP 各拟合一组 Chapront (2000) 式改正：
//!   ρ(τ) = Σ_{P0≤k≤NSEC} a_k τᵏ + Σ_j Σ_{P0≤p≤MIXT} τᵖ (C_jp cos φ_j + S_jp sin φ_j)，τ = T/10（儒略千年）。
//! 频率 φ_j 取该变量 ELP 主问题级数中振幅最大的 M 个幅角组合（相位多项式与理论本身一致），故对系数线性，
//! 用 Householder QR 解线性最小二乘，不加人为边界。
//!
//! 取样：前 2999—2999 年 30000 个随机时刻，另在 1950—2060 年加 3000 个，使现代精度不因远端拟合而变差。
//! 检验用另一组随机种子的独立时刻（每世纪 60 个）。
//!
//! 用法（仓库根，release）：
//!   cargo run -p lunar-core --release --example elpmpp02_residual_fit --no-default-features --features real-f64 -- \
//!     <de406|de431|de441> [--freqs M] [--pmin P0] [--nsec K] [--mixt P] [--out 路径]
//! 缺省 M = 60、幂次自 P0 = 2 起（多项式到 τ¹⁰、混合项到 τ³；按独立检验在 τ⁴—τ¹² 与 τ³—τ⁴ 中选定，见知识库 §5.4.7）：常数拟合已使现代（1950—2060）贴合到 0.006″，τ⁰、τ¹ 项会把远端
//! 残差带进现代，故改正从 τ² 起、在 J2000 附近按 τ² 消失。拟合区间外 τ 固定在端点（`Y` 行），高次多项式不外推。输出 `data/fit/elpmpp02-<目标>-patch.txt` 与 `…-patch-residuals.csv`。

#[path = "support/elp_fit.rs"]
mod elp_fit;
use elp_fit::*;

use lunar_core::astronomy::ephemeris::elpmpp02::parse_constants::ParseConstants;
use lunar_core::astronomy::ephemeris::elpmpp02::patch::{Elpmpp02PatchRaw, PatchTermRaw, PatchVar};
use lunar_core::astronomy::ephemeris::elpmpp02::{Elpmpp02Correction, Elpmpp02Raw, Elpmpp02Term};
use lunar_core::astronomy::ephemeris::De406Kernel;
use lunar_core::math::real::RealOps;

/// 球坐标（λ、β 弧度，r km）。
fn sph(p: &[f64; 3]) -> (f64, f64, f64) {
    let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    (p[1].atan2(p[0]), (p[2] / r).asin(), r)
}

fn wrap(d: f64) -> f64 {
    (d + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

/// 残差（目标 − ELP）：Δλ、Δβ（″），Δr（km）。
fn residuals(elp: &[[f64; 3]], tg: &[[f64; 3]]) -> [Vec<f64>; 3] {
    let mut out = [vec![], vec![], vec![]];
    for (a, b) in elp.iter().zip(tg) {
        let (l1, b1, r1) = sph(a);
        let (l2, b2, r2) = sph(b);
        out[0].push(wrap(l2 - l1) / ARCSEC);
        out[1].push((b2 - b1) / ARCSEC);
        out[2].push(r2 - r1);
    }
    out
}

/// 振幅最大的 M 个不同幅角组合（整数系数全为 0 者除外）。
fn top_frequencies(terms: &[Elpmpp02Term], m: usize) -> Vec<[i8; 4]> {
    let mut v: Vec<(f64, [i8; 4])> = terms
        .iter()
        .filter(|t| t.ilu != [0; 4])
        .map(|t| (t.ci.arcsec().as_f64().abs(), [t.ilu[0] as i8, t.ilu[1] as i8, t.ilu[2] as i8, t.ilu[3] as i8]))
        .collect();
    v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let mut out: Vec<[i8; 4]> = Vec::new();
    for (_, k) in v {
        if !out.contains(&k) {
            out.push(k);
        }
        if out.len() == m {
            break;
        }
    }
    out
}

fn phase(c: &ParseConstants, ilu: &[i8; 4], t_cy: f64) -> f64 {
    let mut s = 0.0;
    let mut tk = 1.0;
    for k in 0..5 {
        s += (0..4).map(|i| ilu[i] as f64 * c.delaunay[i][k]).sum::<f64>() * tk;
        tk *= t_cy;
    }
    s
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opt = |name: &str, def: usize| -> usize {
        args.iter().position(|a| a == name).map(|i| args[i + 1].parse().unwrap()).unwrap_or(def)
    };
    let target = args.first().map(String::as_str).unwrap_or("de431").to_lowercase();
    let (m, nsec, mixt, pmin) = (opt("--freqs", 60), opt("--nsec", 10), opt("--mixt", 3), opt("--pmin", 2));
    let root = lunar_core::repo::repo_root();
    let (bsp, corr) = match target.as_str() {
        "de406" => (root.join(lunar_core::repo::paths::DE406_BSP_CANDIDATES[0]), Elpmpp02Correction::DE406),
        "de431" => (root.join(lunar_core::repo::paths::DE431_BSP), Elpmpp02Correction::DE431),
        "de441" => (root.join(lunar_core::repo::paths::DE441_BSP), Elpmpp02Correction::DE441),
        t => panic!("未知目标 {t}"),
    };
    let out_path = args
        .iter()
        .position(|a| a == "--out")
        .map(|i| std::path::PathBuf::from(&args[i + 1]))
        .unwrap_or_else(|| root.join(format!("data/fit/elpmpp02-{target}-patch.txt")));
    let kernel = De406Kernel::open(&bsp).unwrap_or_else(|e| panic!("打不开 {}：{e}", bsp.display()));
    let raw = Elpmpp02Raw::load(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02).unwrap();
    let fit = corr.builtin_fit().unwrap();
    let pc = ParseConstants::from_fit(&fit);
    let base = raw.build_with_patch(&fit, corr, None);

    let mut jds = sample_jds(-2999.0, 2999.0, 30000, 20261108);
    jds.extend(sample_jds(1950.0, 2060.0, 3000, 20261109));
    let tg = target_positions(&kernel, &jds);
    let res = residuals(&elp_positions(&base, &jds), &tg);
    let ts: Vec<f64> = jds.iter().map(|jd| (jd - J2000) / 36525.0).collect();

    let series: [(&[Elpmpp02Term], PatchVar, &str); 3] =
        [(&base.period_v, PatchVar::Lon, "黄经″"), (&base.period_u, PatchVar::Lat, "黄纬″"), (&base.period_r, PatchVar::Dist, "距离 km")];
    let mut terms: Vec<PatchTermRaw> = Vec::new();
    let mut summary = String::new();
    for (k, (ser, var, label)) in series.iter().enumerate() {
        let freqs = top_frequencies(ser, m);
        // 设计矩阵（列主序）：多项式 τ⁰..τ^NSEC，再每频率 τᵖ cos φ、τᵖ sin φ（p ≤ MIXT）
        let mut cols: Vec<Vec<f64>> = Vec::new();
        let mut keys: Vec<(u8, [i8; 4], bool)> = Vec::new();
        for p in pmin..=nsec {
            cols.push(ts.iter().map(|t| (t / 10.0).powi(p as i32)).collect());
            keys.push((p as u8, [0; 4], false));
        }
        for f in &freqs {
            let ph: Vec<f64> = ts.iter().map(|t| phase(&pc, f, *t)).collect();
            for p in pmin..=mixt {
                cols.push(ts.iter().zip(&ph).map(|(t, a)| (t / 10.0).powi(p as i32) * a.cos()).collect());
                keys.push((p as u8, *f, false));
                cols.push(ts.iter().zip(&ph).map(|(t, a)| (t / 10.0).powi(p as i32) * a.sin()).collect());
                keys.push((p as u8, *f, true));
            }
        }
        let n = cols.len();
        let (x, _) = lstsq(cols, res[k].clone());
        // 合并同一 (p, 频率) 的 cos、sin 系数
        let mut i = 0;
        while i < n {
            let (p, f, _) = keys[i];
            if f == [0; 4] {
                terms.push(PatchTermRaw { var: *var, power: p, ilu: f, c: x[i], s: 0.0 });
                i += 1;
            } else {
                terms.push(PatchTermRaw { var: *var, power: p, ilu: f, c: x[i], s: x[i + 1] });
                i += 2;
            }
        }
        summary.push_str(&format!("# {label}：{n} 个未知数，拟合前残差 RMS {:.4}\n", rms(&res[k])));
        eprintln!("{label}：{n} 个未知数，拟合前 RMS {:.4}", rms(&res[k]));
    }
    let patch = Elpmpp02PatchRaw { terms, years: Some((-2999.0, 2999.0)) };
    let patched = raw.build_with_patch(&fit, corr, Some(&patch));

    // 拟合样本上的残差
    let res2 = residuals(&elp_positions(&patched, &jds), &tg);
    for (k, label) in ["黄经″", "黄纬″", "距离 km"].iter().enumerate() {
        summary.push_str(&format!("# {label}：拟合后（拟合样本）RMS {:.4}\n", rms(&res2[k])));
    }

    // 独立检验：同 elpmpp02_de_fit 的三个区间，及逐世纪
    let mut stats = String::new();
    for (y0, y1, seed) in [(1950.0, 2060.0, 11u64), (1500.0, 2500.0, 12), (-2999.0, 2999.0, 13)] {
        let jj = sample_jds(y0, y1, 3000, 777 + seed);
        let tt = target_positions(&kernel, &jj);
        for (label, d) in [("常数集", &base), ("常数集+残差改正", &patched)] {
            let (ml, mb, mr, sl) = sph_stats(&elp_positions(d, &jj), &tt);
            stats.push_str(&format!(
                "# {y0:>6}—{y1:<6} {label:<14} max|Δλ| {ml:8.4}″  max|Δβ| {mb:7.4}″  max|Δr| {mr:8.4} km  RMS Δλ {sl:7.4}″\n"
            ));
        }
    }
    eprint!("{stats}");
    let csv_path = out_path.with_file_name(format!("{}-residuals.csv", out_path.file_stem().unwrap().to_string_lossy()));
    let mut csv = String::from("century_start,variant,n,max_abs_dlon_arcsec,rms_dlon_arcsec,max_abs_dlat_arcsec,max_abs_dr_km\n");
    for c in -30..30 {
        let y0 = (c * 100) as f64;
        let jj = sample_jds(y0.max(-2999.0), (y0 + 100.0).min(2999.0), 60, 5000 + (c + 30) as u64);
        let tt = target_positions(&kernel, &jj);
        for (label, d) in [("constants", &base), ("constants+patch", &patched)] {
            let (ml, mb, mr, sl) = sph_stats(&elp_positions(d, &jj), &tt);
            csv.push_str(&format!("{},{label},{},{ml:.4},{sl:.4},{mb:.4},{mr:.4}\n", c * 100, jj.len()));
        }
    }
    std::fs::write(&csv_path, &csv).unwrap();

    let mut text = format!(
        "# ELP/MPP02 残差改正：目标 JPL {}，常数集 {:?}（Chapront 2000 式，见 rust/core/src/astronomy/ephemeris/elpmpp02/patch.rs）\n",
        target.to_uppercase(),
        corr
    );
    text.push_str("# 生成：cargo run -p lunar-core --release --example elpmpp02_residual_fit --no-default-features --features real-f64 -- ");
    text.push_str(&args.join(" "));
    text.push('\n');
    text.push_str(&format!(
        "# 每变量频率 M = {m}（主问题级数振幅最大者）、τ 的幂次 {pmin}—{nsec}（多项式）、{pmin}—{mixt}（混合项）；样本前 2999—2999 年 30000 + 1950—2060 年 3000 个时刻\n"
    ));
    text.push_str(&summary);
    text.push_str("# 独立检验（每区间 3000 个随机时刻，J2000 平黄道几何位置）：\n");
    text.push_str(&stats);
    text.push_str("# 每行：变量（L 黄经″、B 黄纬″、R 距离 km） τ 的幂次 iD iF il il′ cos 系数 sin 系数\n");
    text.push_str(&patch.to_text());
    std::fs::write(&out_path, &text).unwrap();
    eprintln!("写出 {}、{}", out_path.display(), csv_path.display());
}
