//! DE406 BSP 历表：视黄经与定朔示例。
//!
//! BSP 路径与项目约定一致：`data/jpl/de406.bsp` 或 `data/jpl/de406/de406.bsp`（相对仓库根）。
//!
//! 运行（仓库根下）：
//!   cargo run -p lunar-core --example de406_apparent --no-default-features --features twofloat
//! 或指定路径 / 环境变量：
//!   cargo run -p lunar-core --example de406_apparent --no-default-features --features twofloat -- data/jpl/de406.bsp
//!   DE406_BSP=data/jpl/de406.bsp cargo run -p lunar-core --example de406_apparent --no-default-features --features twofloat
//!
//! 输出：J2000.5 太阳/月球视黄经（弧度与度）、该时段内一个定朔 JD(TT)。

/// 与 tests_jpl_python / data/jpl 约定一致：在 data/jpl 下找 de406.bsp 或 de406/de406.bsp。
const BSP_CANDIDATES: &[&str] = &[
    "data/jpl/de406.bsp",
    "data/jpl/de406/de406.bsp",
    "../data/jpl/de406.bsp",
    "../data/jpl/de406/de406.bsp",
];

fn main() {
    use lunar_core::astronomy::apparent::{
        moon_apparent_ecliptic_longitude_de406, sun_apparent_ecliptic_longitude_de406,
    };
    use lunar_core::astronomy::aspects::new_moon_jds_in_range_de406;
    use lunar_core::astronomy::ephemeris::De406Kernel;
    use lunar_core::astronomy::time::{TimePoint, TimeScale};
    use lunar_core::math::real::{real, RealOps};
    use lunar_core::quantity::angle::PlaneAngle;

    let bsp_path = std::env::args()
        .nth(1)
        .filter(|p| std::path::Path::new(p).exists())
        .or_else(|| std::env::var("DE406_BSP").ok().filter(|p| std::path::Path::new(p).exists()))
        .or_else(|| {
            BSP_CANDIDATES
                .iter()
                .find(|p| std::path::Path::new(p).exists())
                .map(|s| (*s).to_string())
        });

    let Some(bsp_path) = bsp_path else {
        eprintln!("未找到 BSP 文件。本项目约定路径（相对仓库根）：");
        eprintln!("  data/jpl/de406.bsp  或  data/jpl/de406/de406.bsp");
        eprintln!("也可：传参  -- <路径>  或设置环境变量  DE406_BSP");
        eprintln!("BSP 须为 NAIF SPK 格式（非 lnx*.406 的 JPL 原始二进制）");
        std::process::exit(1);
    };

    let kernel = match De406Kernel::open(&bsp_path) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("打开 BSP 失败: {}", e);
            eprintln!("文件: {} ({} 字节)", bsp_path, std::fs::metadata(&bsp_path).map(|m| m.len()).unwrap_or(0));
            std::process::exit(1);
        }
    };
    let deg = 180.0 / core::f64::consts::PI;

    // 太阳、月球视黄经 @ J2000.5 TT
    let t = TimePoint::new(TimeScale::TT, real(2451545.0 + 0.5));
    let sun_lam = sun_apparent_ecliptic_longitude_de406(&kernel, t);
    let moon_lam = moon_apparent_ecliptic_longitude_de406(&kernel, t);
    println!("JD(TT) = 2451545.5");
    println!("  太阳视黄经 λ☉ = {:.6} rad = {:.4}°", sun_lam.rad().as_f64(), sun_lam.rad().as_f64() * deg);
    println!("  月球视黄经 λ☽ = {:.6} rad = {:.4}°", moon_lam.rad().as_f64(), moon_lam.rad().as_f64() * deg);

    // 定朔：2026 年一个朔
    let jd_start = real(2457388.5);
    let jd_end = real(2457418.5);
    let tolerance = PlaneAngle::from_rad(real(1e-8));
    let new_moons = new_moon_jds_in_range_de406(&kernel, jd_start, jd_end, tolerance, 30);
    if let Some(jd) = new_moons.first() {
        println!("\n定朔（DE406）[JD {}..{}] 内首个合朔:", jd_start.as_f64(), jd_end.as_f64());
        println!("  JD(TT) = {:.6}", jd.as_f64());
    }
}
