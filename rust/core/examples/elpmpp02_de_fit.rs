//! 按 Chapront & Francou (2003) 的两步法，把 ELP/MPP02 的常数拟合到 JPL 数值历表（DE406、DE441 等）。
//!
//! 论文 ELP/MPP02(405) 的做法（elpmpp02.pdf §4.2、§4.3.3、§7）：
//! 1. 在 1950—2060 年以 DE405 为「观测模型」拟合 Table 3 的 13 个常数改正
//!    （ΔW1⁽⁰⁾、ΔW2⁽⁰⁾、ΔW3⁽⁰⁾、Δν、ΔW2⁽¹⁾、ΔW3⁽¹⁾、ΔW1⁽²⁾、ΔΓ、ΔE、ΔT⁽⁰⁾、Δn′、Δϖ′⁽⁰⁾、Δe′）；
//! 2. 常数固定，在 6000 年（前 3000—3000 年）上拟合 W1、W2、W3 长期项的加性改正（Table 6：W1 的 T³、T⁴，W2、W3 的 T²、T³）。
//!
//! 本程序同法实现：模型即本仓库的完整 ELP/MPP02 求值（与官方 Fortran 校验值一致到 5 cm），目标为 BSP 内核的地心月球
//! 几何位置，经 Table 7（ICRS 行）转到 J2000 平黄道；残差取直角坐标差（km）。两步交替迭代（Gauss–Newton，数值偏导，
//! Householder QR 解最小二乘），至参数改变量小于偏导步长的千分之一。
//! 第一步另带两个小旋转（绕 J2000 平黄道 x、y 轴）作冗余参数，检验目标历表的架与 Table 7 是否一致；不写入结果。
//!
//! 用法（在 rust 目录，release 构建）：
//!   cargo run -p lunar-core --release --example elpmpp02_de_fit -- <de406|de441|de431> [输出文件] [--stage-b 名,名,...]
//! 缺省输出 `data/fit/elpmpp02-<目标>.txt`（相对仓库根）；`--stage-b` 改第二步参数（缺省 Table 6 的六个，列出的参数不再在
//! 第一步拟合）；`--no-fit` 不拟合，只输出论文常数集对目标历表之差（结果即论文常数集）。
//! 输出文件头为注释（拟合区间、样本数、形式误差、残差），正文为 `Elpmpp02Fit::parse` 可读的「名称 值」。

use lunar_core::astronomy::ephemeris::elpmpp02::{Elpmpp02Correction, Elpmpp02Fit, Elpmpp02Raw};
use lunar_core::astronomy::ephemeris::De406Kernel;

#[path = "support/elp_fit.rs"]
mod elp_fit;
use elp_fit::*;

/// 参数：常数改正集中的一个（按名），或冗余的小旋转。
#[derive(Clone)]
enum Param {
    Fit(String),
    RotX,
    RotY,
}

impl Param {
    fn name(&self) -> String {
        match self {
            Param::Fit(s) => s.clone(),
            Param::RotX => "rot_x".into(),
            Param::RotY => "rot_y".into(),
        }
    }
    /// T 的幂次（决定偏导步长）。
    fn power(&self) -> i32 {
        match self {
            Param::Fit(s) if s.starts_with("sw") => s[4..].parse().unwrap(),
            Param::Fit(s) if s.ends_with("_1") => 1,
            Param::Fit(s) if s.ends_with("_2") => 2,
            _ => 0,
        }
    }
}

struct Stage {
    name: &'static str,
    jds: Vec<f64>,
    target: Vec<[f64; 3]>,
    params: Vec<Param>,
    /// 区间内 |T| 的最大值（儒略世纪），定偏导步长
    t_max: f64,
}

/// 当前状态：常数改正集 + 冗余旋转（角秒）。
#[derive(Clone)]
struct State {
    fit: Elpmpp02Fit,
    rot: [f64; 2],
}

fn apply_rot(p: [f64; 3], rot: [f64; 2]) -> [f64; 3] {
    // 小旋转 ω × v，ω = (rx, ry, 0)
    let (rx, ry) = (rot[0] * ARCSEC, rot[1] * ARCSEC);
    [p[0] + ry * p[2], p[1] - rx * p[2], p[2] + rx * p[1] - ry * p[0]]
}

fn residuals(raw: &Elpmpp02Raw, st: &State, stage: &Stage) -> Vec<f64> {
    let data = raw.build_with_patch(&st.fit, Elpmpp02Correction::DE406, None);
    let pos = elp_positions(&data, &stage.jds);
    let mut r = Vec::with_capacity(3 * pos.len());
    for (p, t) in pos.iter().zip(&stage.target) {
        let q = apply_rot(*p, st.rot);
        r.extend((0..3).map(|k| q[k] - t[k]));
    }
    r
}

/// 一步 Gauss–Newton 迭代至收敛，返回各参数的形式误差（角秒/世纪^k）与残差 RMS（km）。
fn run_stage(raw: &Elpmpp02Raw, st: &mut State, stage: &Stage) -> (Vec<f64>, f64) {
    let mut sigma = vec![0.0; stage.params.len()];
    let mut post = 0.0;
    for it in 0..6 {
        let r0 = residuals(raw, st, stage);
        let steps: Vec<f64> = stage.params.iter().map(|p| 0.01 / stage.t_max.powi(p.power()).max(1e-9)).collect();
        let mut cols = Vec::new();
        for (p, &h) in stage.params.iter().zip(&steps) {
            let mut s1 = st.clone();
            match p {
                Param::Fit(name) => *s1.fit.param_mut(name).unwrap() += h,
                Param::RotX => s1.rot[0] += h,
                Param::RotY => s1.rot[1] += h,
            }
            let r1 = residuals(raw, &s1, stage);
            cols.push(r1.iter().zip(&r0).map(|(a, b)| (a - b) / h).collect::<Vec<_>>());
        }
        let neg: Vec<f64> = r0.iter().map(|x| -x).collect();
        let (dx, var) = lstsq(cols, neg);
        for (p, d) in stage.params.iter().zip(&dx) {
            match p {
                Param::Fit(name) => *st.fit.param_mut(name).unwrap() += d,
                Param::RotX => st.rot[0] += d,
                Param::RotY => st.rot[1] += d,
            }
        }
        let m = r0.len() as f64;
        let n = stage.params.len() as f64;
        let r_after = residuals(raw, st, stage);
        post = rms(&r_after);
        let s0 = (r_after.iter().map(|x| x * x).sum::<f64>() / (m - n)).sqrt();
        sigma = var.iter().map(|v| s0 * v.sqrt()).collect();
        // 收敛判据：各参数改变量都小于其形式误差之半。残差以理论截断的系统差为主，形式误差很小，
        // 收敛后 Gauss–Newton 仍因非线性在约 0.1—1σ 内抖动（残差 RMS 只变 1e-5 km），再迭代无意义
        let rel = dx.iter().zip(&sigma).map(|(d, s)| (d / s).abs()).fold(0.0_f64, f64::max);
        eprintln!(
            "  [{}] 迭代 {}：残差 RMS {:.5} → {:.5} km，改变量/形式误差 最大 {:.2e}",
            stage.name,
            it + 1,
            rms(&r0),
            post,
            rel
        );
        if rel < 0.5 {
            break;
        }
    }
    (sigma, post)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let target = args.first().map(String::as_str).unwrap_or("de441").to_lowercase();
    let root = lunar_core::repo::repo_root();
    let bsp = match target.as_str() {
        "de406" => root.join(lunar_core::repo::paths::DE406_BSP_CANDIDATES[0]),
        "de441" => root.join(lunar_core::repo::paths::DE441_BSP),
        "de431" => root.join(lunar_core::repo::paths::DE431_BSP),
        t => panic!("未知目标历表 {t}（de406 | de441 | de431）"),
    };
    let out_path = args
        .get(1)
        .filter(|s| !s.starts_with("--"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join(format!("data/fit/elpmpp02-{target}.txt")));
    let stage_b_names: Vec<String> = match args.iter().position(|a| a == "--stage-b") {
        Some(i) => args[i + 1].split(',').map(str::to_string).collect(),
        None => ["sw1_3", "sw1_4", "sw2_2", "sw2_3", "sw3_2", "sw3_3"].iter().map(|s| s.to_string()).collect(),
    };

    let kernel = De406Kernel::open(&bsp).unwrap_or_else(|e| panic!("打不开 {}：{e}", bsp.display()));
    let raw = Elpmpp02Raw::load(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02).unwrap();

    // 第一步：1950—2060（同论文），6000 个时刻；第二步：前 3000—3000 年（同论文 Table 6），4000 个时刻
    let (a0, a1) = (1950.0, 2060.0);
    let (b0, b1) = (-2999.0, 2999.0);
    let jds_a = sample_jds(a0, a1, 6000, 20261106);
    let jds_b = sample_jds(b0, b1, 4000, 20261107);
    let stage_a = Stage {
        name: "第一步 1950—2060",
        target: target_positions(&kernel, &jds_a),
        jds: jds_a,
        // 第二步要拟合的参数（如 dw1_2）不在第一步重复拟合
        params: ["dw1_0", "dw2_0", "dw3_0", "dw1_1", "dw2_1", "dw3_1", "dw1_2", "dgam", "de", "deart_0", "deart_1", "dperi", "dep"]
            .iter()
            .filter(|s| !stage_b_names.iter().any(|b| b == *s))
            .map(|s| Param::Fit(s.to_string()))
            .chain([Param::RotX, Param::RotY])
            .collect(),
        t_max: 0.6,
    };
    let stage_b = Stage {
        name: "第二步 前3000—3000",
        target: target_positions(&kernel, &jds_b),
        jds: jds_b,
        params: stage_b_names.iter().map(|s| Param::Fit(s.clone())).collect(),
        t_max: 50.0,
    };

    // 初值：论文 DE405 + Table 6
    let mut st = State { fit: Elpmpp02Fit::de405(), rot: [0.0, 0.0] };
    let (mut sig_a, mut sig_b, mut rms_a, mut rms_b) = (vec![], vec![], 0.0, 0.0);
    // --no-fit：不拟合，只输出论文常数集对目标历表之差（作基线）
    let rounds = if args.iter().any(|a| a == "--no-fit") { 0 } else { 4 };
    for round in 1..=rounds {
        eprintln!("第 {round} 轮");
        let before = st.fit.clone();
        (sig_a, rms_a) = run_stage(&raw, &mut st, &stage_a);
        // 冗余旋转只用于检验，不进第二步
        let rot = st.rot;
        st.rot = [0.0, 0.0];
        (sig_b, rms_b) = run_stage(&raw, &mut st, &stage_b);
        st.rot = rot;
        // 轮间收敛：本轮各参数改变量都小于其形式误差之半
        let change = stage_a
            .params
            .iter()
            .zip(&sig_a)
            .chain(stage_b.params.iter().zip(&sig_b))
            .filter_map(|(p, s)| match p {
                Param::Fit(n) => Some((st.fit.param(n).unwrap() - before.param(n).unwrap()).abs() / s),
                _ => None,
            })
            .fold(0.0_f64, f64::max);
        eprintln!("  本轮改变量/形式误差 最大 {change:.2e}");
        if round > 1 && change < 0.5 {
            break;
        }
    }

    // 残差统计：同论文 §7 的三个区间
    let paper = raw.build_with_patch(&Elpmpp02Fit::de405(), Elpmpp02Correction::DE406, None);
    let fitted = raw.build_with_patch(&st.fit, Elpmpp02Correction::DE406, None);
    let mut stats = String::new();
    for (y0, y1, n, seed) in [(1950.0, 2060.0, 3000, 1u64), (1500.0, 2500.0, 3000, 2), (-2999.0, 2999.0, 3000, 3)] {
        let jds = sample_jds(y0, y1, n, 99 + seed);
        let tg = target_positions(&kernel, &jds);
        for (label, d) in [("论文 DE405+Table6", &paper), ("本拟合", &fitted)] {
            let (ml, mb, mr, sl) = sph_stats(&elp_positions(d, &jds), &tg);
            stats.push_str(&format!(
                "# {y0:>6}—{y1:<6} {label:<18} max|Δλ| {ml:8.4}″  max|Δβ| {mb:7.4}″  max|Δr| {mr:8.4} km  RMS Δλ {sl:7.4}″\n"
            ));
        }
    }
    eprint!("{stats}");

    // 逐世纪残差表：前 3000—3000 年每世纪 60 个时刻，论文常数集与本拟合各对目标历表
    let csv_path = out_path.with_file_name(format!(
        "{}-residuals.csv",
        out_path.file_stem().unwrap().to_string_lossy()
    ));
    let mut csv = String::from("century_start,constants,n,max_abs_dlon_arcsec,rms_dlon_arcsec,max_abs_dlat_arcsec,max_abs_dr_km\n");
    for c in -30..30 {
        let y0 = (c * 100) as f64;
        let jds = sample_jds(y0.max(-2999.0), (y0 + 100.0).min(2999.0), 60, 1000 + (c + 30) as u64);
        let tg = target_positions(&kernel, &jds);
        for (label, d) in [("DE405+Table6", &paper), ("fit", &fitted)] {
            let (ml, mb, mr, sl) = sph_stats(&elp_positions(d, &jds), &tg);
            csv.push_str(&format!("{},{label},{},{ml:.4},{sl:.4},{mb:.4},{mr:.4}\n", c * 100, jds.len()));
        }
    }
    std::fs::write(&csv_path, &csv).unwrap();
    eprintln!("写出 {}", csv_path.display());

    let mut text = String::new();
    text.push_str(&format!(
        "# ELP/MPP02 常数改正集：按 Chapront & Francou (2003) 两步法拟合 JPL {}\n",
        target.to_uppercase()
    ));
    text.push_str("# 生成：cargo run -p lunar-core --release --example elpmpp02_de_fit -- ");
    text.push_str(&args.join(" "));
    text.push('\n');
    text.push_str(&format!(
        "# 第一步 {a0}—{a1} 年 {} 个时刻，拟合 Table 3 的 13 个常数（另带冗余旋转 rot_x {:+.5}″、rot_y {:+.5}″，不写入）；残差 RMS {rms_a:.4} km\n",
        stage_a.jds.len(),
        st.rot[0],
        st.rot[1]
    ));
    text.push_str(&format!(
        "# 第二步 {b0}—{b1} 年 {} 个时刻，拟合长期项 {}；残差 RMS {rms_b:.4} km\n",
        stage_b.jds.len(),
        stage_b_names.join("、")
    ));
    text.push_str("# 形式误差（1σ）：");
    for (p, s) in stage_a.params.iter().zip(&sig_a).chain(stage_b.params.iter().zip(&sig_b)) {
        text.push_str(&format!("{} {:.1e}  ", p.name(), s));
    }
    text.push_str("\n# 与目标历表之差（J2000 平黄道几何位置）：\n");
    text.push_str(&stats);
    text.push_str("# 单位：角秒，角秒/世纪^k（_1、_2 与 swi_k 的 k）\n");
    text.push_str(&st.fit.to_text());
    std::fs::write(&out_path, &text).unwrap();
    eprintln!("写出 {}", out_path.display());
    print!("{text}");
}
