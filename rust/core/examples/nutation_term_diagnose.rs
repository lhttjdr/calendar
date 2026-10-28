//! 诊断用：逐项累加 tab5.3a 完整表的 Δψ/Δε，对照 pyerfa（SOFA nut00a 的 Python 移植）参考值，
//! 找出若有发散从第几项开始。参考值需要另装 pyerfa：
//! `uv run --project scripts --with pyerfa python3 -c "import erfa; print(erfa.nut00a(...))"`
//! 运行：cargo run -p lunar-core --example nutation_term_diagnose --no-default-features --features twofloat
//!
//! 2026-09 用它核实过 doc/case-studies/de406-solar-term-failure-analysis.md 里"完整表 78+ 项有 ~9s
//! 系统误差"的旧结论是错的——完整表在参考春分处与 pyerfa 只差 0.0001″，问题不在章动，详见该文档更新记录。

use lunar_core::math::real::RealOps;

fn main() {
    let iau = lunar_core::astronomy::frame::nutation::load_iau2000a_from_repo()
        .expect("加载章动表失败");
    println!("总项数: {}", iau.term_count());

    // 参考春分 JD(TDB)=2461120.116049，t = 儒略世纪数（J2000 TT）
    let jd = 2461120.116049_f64;
    let t = (jd - 2451545.0) / 36525.0;
    println!("t = {t}");

    // pyerfa (SOFA iauNut00a) 参考值，见 doc/case-studies/de406-solar-term-failure-analysis.md 诊断
    let ref_dpsi_arcsec = 6.222493378593895_f64;
    let ref_deps_arcsec = 9.134801871488445_f64;
    println!("参考 (pyerfa nut00a): dpsi={ref_dpsi_arcsec:.4}\" deps={ref_deps_arcsec:.4}\"");

    for n in [1usize, 10, 50, 77, 78, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, iau.term_count()] {
        let (dpsi, deps) = iau.nutation_first_n(t, n);
        let dpsi_as = dpsi.rad().as_f64() * (180.0 / std::f64::consts::PI) * 3600.0;
        let deps_as = deps.rad().as_f64() * (180.0 / std::f64::consts::PI) * 3600.0;
        println!(
            "n={n:4}  dpsi={dpsi_as:9.4}\"  deps={deps_as:9.4}\"   (diff dpsi={:+.4}\" diff deps={:+.4}\")",
            dpsi_as - ref_dpsi_arcsec,
            deps_as - ref_deps_arcsec
        );
    }
}
