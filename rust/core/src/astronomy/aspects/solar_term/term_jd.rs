//! 定气：太阳视黄经反算时刻（对标 SolarTerm）。
//! 提供按公历年的 24 节气缓存，供农历（取中气）与节气派干支历（取十二节）共用，避免重复计算。

use crate::astronomy::apparent::{
    sun_apparent_ecliptic_longitude, sun_apparent_ecliptic_longitude_velocity,
    sun_apparent_ecliptic_longitude_with_options, ApparentPipelineOptions,
};
use crate::astronomy::ephemeris::Vsop87;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::calendar::gregorian::Gregorian;
use crate::math::real::{real_const, real, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::angular_rate::AngularRate;
use crate::quantity::unit::AngularRateUnit;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;
use std::f64::consts::PI;

/// 平均回归年（日），用于近似步进。直接 Real，无 f64 边界。
pub const MEAN_TROPICAL_YEAR_DAYS: Real = real_const(365.2422);

/// 平均太阳黄经角速度（rad/日 ≈ 2π/365.2422）
#[inline]
pub fn mean_solar_longitude_velocity() -> AngularRate {
    AngularRate::from_value(real(core::f64::consts::TAU) / MEAN_TROPICAL_YEAR_DAYS, AngularRateUnit::RadPerDay)
}

/// 数值导数用 JD 步长（日）
const NUMERICAL_DELTA_JD: Real = real_const(0.01);

/// 定气：求太阳视黄经 = target 的 JD(TT)。返回 R，标量全程 Real。
pub fn solar_longitude_jd(
    vsop: &Vsop87,
    target: PlaneAngle,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Real {
    const EARLY_RESIDUAL_RAD: Real = real_const(1e-5);
    const EARLY_DELTA_JD: Real = real_const(1e-6);
    let target_rad = target.rad();
    let tol_rad = tolerance.rad();

    let mut jd = t_approx.to_scale(TimeScale::TT).jd;
    let mut prev_residual: Option<Real> = None;
    for _ in 0..max_iterations {
        let t = TimePoint::new(TimeScale::TT, jd);
        let lam = sun_apparent_ecliptic_longitude(vsop, t).rad();
        let residual = (lam - target_rad).wrap_to_signed_pi();
        if residual.abs() <= tol_rad {
            return jd;
        }
        let t_lo = TimePoint::new(TimeScale::TT, jd - NUMERICAL_DELTA_JD);
        let t_hi = TimePoint::new(TimeScale::TT, jd + NUMERICAL_DELTA_JD);
        let lam_lo = sun_apparent_ecliptic_longitude(vsop, t_lo).rad();
        let lam_hi = sun_apparent_ecliptic_longitude(vsop, t_hi).rad();
        let mut dlam = lam_hi - lam_lo;
        if dlam > real(core::f64::consts::PI) {
            dlam = dlam - real(core::f64::consts::TAU);
        } else if dlam < real(-core::f64::consts::PI) {
            dlam = dlam + real(core::f64::consts::TAU);
        }
        let two_delta: Real = real_const(2.0) * NUMERICAL_DELTA_JD;
        let velocity = dlam / two_delta;
        let mean_rad_per_day = mean_solar_longitude_velocity().rad_per_day();
        let safe_velocity = if velocity.abs() < real_const(0.01) {
            if velocity >= real(0) {
                mean_rad_per_day
            } else {
                -mean_rad_per_day
            }
        } else {
            velocity
        };
        let mut step = residual / safe_velocity;
        if let Some(pr) = prev_residual {
            if (pr > real(0) && residual < real(0)) || (pr < real(0) && residual > real(0)) {
                step = step * real_const(0.5);
            }
        }
        prev_residual = Some(residual);
        if residual.abs() < EARLY_RESIDUAL_RAD && step.abs() < EARLY_DELTA_JD {
            return jd;
        }
        jd -= step;
    }
    jd
}

/// 同上，可指定 pipeline 选项（如岁差模型）。用于 TDB 对照等测试。
pub fn solar_longitude_jd_with_options(
    vsop: &Vsop87,
    target: PlaneAngle,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
    options: &ApparentPipelineOptions,
) -> Real {
    const EARLY_RESIDUAL_RAD: Real = real_const(1e-5);
    const EARLY_DELTA_JD: Real = real_const(1e-6);
    let target_rad = target.rad();
    let tol_rad = tolerance.rad();

    let mut jd = t_approx.to_scale(TimeScale::TT).jd;
    let mut prev_residual: Option<Real> = None;
    for _ in 0..max_iterations {
        let t = TimePoint::new(TimeScale::TT, jd);
        let lam = sun_apparent_ecliptic_longitude_with_options(vsop, t, options).rad();
        let residual = (lam - target_rad).wrap_to_signed_pi();
        if residual.abs() <= tol_rad {
            return jd;
        }
        let velocity = sun_apparent_ecliptic_longitude_velocity(vsop, t, options);
        let mean_rad_per_day = mean_solar_longitude_velocity().rad_per_day();
        let safe_velocity = if velocity.abs() < real_const(0.01) {
            if velocity >= real(0) {
                mean_rad_per_day
            } else {
                -mean_rad_per_day
            }
        } else {
            velocity
        };
        let mut step = residual / safe_velocity;
        if let Some(pr) = prev_residual {
            if (pr > real(0) && residual < real(0)) || (pr < real(0) && residual > real(0)) {
                step = step * real_const(0.5);
            }
        }
        prev_residual = Some(residual);
        if residual.abs() < EARLY_RESIDUAL_RAD && step.abs() < EARLY_DELTA_JD {
            return jd;
        }
        jd -= step;
    }
    jd
}

/// 节气序号 0..23 对应的目标黄经：0=春分(0°)，6=夏至(90°)，12=秋分(180°)，18=冬至(270°) 等。
#[inline]
pub fn solar_term_longitude(term_index: usize) -> PlaneAngle {
    let rad = real(term_index as f64) * real(15.0)
        * real(PI / 180.0);
    PlaneAngle::from_rad(rad)
}

/// 求太阳视黄经 = 第 term_index 节气目标值的 JD(TT)。term_index 0=春分,…,18=冬至。
pub fn solar_term_jd(
    vsop: &Vsop87,
    term_index: usize,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Real {
    solar_longitude_jd(
        vsop,
        solar_term_longitude(term_index.min(23)),
        t_approx,
        tolerance,
        max_iterations,
    )
}

/// 用当前视黄经近似目标黄经对应的 JD。
fn approximate_solar_longitude_jd(jd_ref: Real, longitude_ref: PlaneAngle, target: PlaneAngle) -> Real {
    let diff = (target.rad() - longitude_ref.rad()).wrap_to_signed_pi();
    jd_ref + diff / mean_solar_longitude_velocity().rad_per_day()
}

/// 求 [jd_start, jd_end] 内所有 24 节气时刻：(节气序号, JD) 列表，按 JD 升序。
/// 初值用 pipeline 视黄经近似，再牛顿求精。
pub fn solar_term_jds_in_range(
    vsop: &Vsop87,
    jd_start: Real,
    jd_end: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Vec<(usize, Real)> {
    if jd_end < jd_start {
        return vec![];
    }
    let mut out: Vec<(usize, Real)> = Vec::new();
    for k in 0..24 {
        let target = solar_term_longitude(k);
        let mut jd_approx = jd_start;
        while jd_approx <= jd_end {
            let t_approx = TimePoint::new(TimeScale::TT, jd_approx);
            let lam_ref = sun_apparent_ecliptic_longitude(vsop, t_approx);
            jd_approx = approximate_solar_longitude_jd(jd_approx, lam_ref, target);
            if jd_approx > jd_end {
                break;
            }
            let t_approx = TimePoint::new(TimeScale::TT, jd_approx);
            let jd = solar_longitude_jd(vsop, target, t_approx, tolerance, max_iterations);
            if jd >= jd_start && jd <= jd_end {
                out.push((k, jd));
            }
            jd_approx = jd + MEAN_TROPICAL_YEAR_DAYS;
        }
    }
    out.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// 同上，可指定 pipeline 选项（如岁差模型）。
pub fn solar_term_jds_in_range_with_options(
    vsop: &Vsop87,
    jd_start: Real,
    jd_end: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
    options: &ApparentPipelineOptions,
) -> Vec<(usize, Real)> {
    if jd_end < jd_start {
        return vec![];
    }
    let mut out: Vec<(usize, Real)> = Vec::new();
    for k in 0..24 {
        let target = solar_term_longitude(k);
        let mut jd_approx = jd_start;
        while jd_approx <= jd_end {
            let t_approx = TimePoint::new(TimeScale::TT, jd_approx);
            let lam_ref = sun_apparent_ecliptic_longitude_with_options(vsop, t_approx, options);
            jd_approx = approximate_solar_longitude_jd(jd_approx, lam_ref, target);
            if jd_approx > jd_end {
                break;
            }
            let t_approx = TimePoint::new(TimeScale::TT, jd_approx);
            let jd = solar_longitude_jd_with_options(
                vsop, target, t_approx, tolerance, max_iterations, options,
            );
            if jd >= jd_start && jd <= jd_end {
                out.push((k, jd));
            }
            jd_approx = jd + MEAN_TROPICAL_YEAR_DAYS;
        }
    }
    out.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// 某年 24 节气对应的 JD(TT) 列表。索引 0=春分(0°)，1=清明(15°)，…，18=冬至(270°)，19=小寒(285°)，…，23=惊蛰(345°)。
/// 用 solar_term_jds_in_range 得全年节气（初值由 pipeline 视黄经近似），再按 k 填回。
pub fn solar_term_jds_for_year(vsop: &Vsop87, year: i32) -> Vec<Real> {
    let jd_start = Gregorian::to_julian_day(year, 1, 1);
    let jd_end = Gregorian::to_julian_day(year, 12, 31) + real(1.0);
    let pairs = solar_term_jds_in_range(
        vsop,
        jd_start,
        jd_end,
        PlaneAngle::from_rad(real(1e-8)),
        25,
    );
    let mut jds: Vec<Real> = vec![real(0.0); 24];
    let mut filled = [false; 24];
    for (k, jd) in pairs {
        if k < 24 && jd >= jd_start && jd <= jd_end && !filled[k] {
            jds[k] = jd;
            filled[k] = true;
        }
    }
    for k in 0..24 {
        if !filled[k] {
            jds[k] = jd_start
                + real_const(79.0)
                + real_const(k as f64) * MEAN_TROPICAL_YEAR_DAYS / real_const(24.0);
        }
    }
    jds
}

/// 同上，可指定 pipeline 选项（如岁差模型）。用于 TDB 对照等测试。
pub fn solar_term_jds_for_year_with_options(
    vsop: &Vsop87,
    year: i32,
    options: &ApparentPipelineOptions,
) -> Vec<Real> {
    let jd_start = Gregorian::to_julian_day(year, 1, 1);
    let jd_end = Gregorian::to_julian_day(year, 12, 31) + real(1.0);
    let pairs = solar_term_jds_in_range_with_options(
        vsop,
        jd_start,
        jd_end,
        PlaneAngle::from_rad(real(1e-8)),
        25,
        options,
    );
    let mut jds: Vec<Real> = vec![real(0.0); 24];
    let mut filled = [false; 24];
    for (k, jd) in pairs {
        if k < 24 && jd >= jd_start && jd <= jd_end && !filled[k] {
            jds[k] = jd;
            filled[k] = true;
        }
    }
    for k in 0..24 {
        if !filled[k] {
            jds[k] = jd_start
                + real_const(79.0)
                + real_const(k as f64) * MEAN_TROPICAL_YEAR_DAYS / real_const(24.0);
        }
    }
    jds
}

/// 按公历年的 24 节气缓存，农历岁数据与节气派干支历共用，避免定气重复计算。
/// 使用 RwLock 以满足 static 的 Sync，便于 wasm 与 native 共用。
static SOLAR_TERMS_CACHE: Lazy<RwLock<HashMap<i32, Vec<Real>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 某年 24 节气 JD(TT)，带缓存。返回 Vec<R>，不写死 f64。
pub fn solar_term_jds_for_year_cached(vsop: &Vsop87, year: i32) -> Vec<Real> {
    let cache = SOLAR_TERMS_CACHE.read().unwrap();
    if let Some(jds) = cache.get(&year) {
        jds.clone()
    } else {
        drop(cache);
        let jds = solar_term_jds_for_year(vsop, year);
        let mut cache = SOLAR_TERMS_CACHE.write().unwrap();
        cache.insert(year, jds.clone());
        jds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::ephemeris::load_earth_vsop87;
    use crate::astronomy::frame::nutation;
    use crate::astronomy::frame::nutation::iau2000a::Iau2000a;
    use crate::astronomy::frame::nutation::table_parser;
    use crate::astronomy::time::{TimePoint, TimeScale};
    use crate::platform::DataLoaderNative;
    use std::io::BufRead;
    use std::path::Path;

    /// 从 data/TDBtimes.txt 加载指定年份的 24 节气 JD(TDB)。
    /// 格式依《月相和二十四节气的计算》§7.4：第 1 栏年、第 2 栏 jd0（该年 1 月 0 日 TDB+8 零时）、
    /// 第 3 栏 Z11a 为最接近 jd0 的冬至（前一岁冬至），第 4–27 栏为 Z11a 以后的二十四节气：J12(小寒) 到 Z11b(冬至)。
    /// 故 24 节气占 tokens[3]..tokens[26]（0-based）。本实现顺序为 春分(0)..惊蛰(23)，对应文件列 春分(8)、清明(9)、…、大雪(25)、冬至(26)、小寒(3)、…、惊蛰(7)。
    /// idx = 3 + (k<=17 ? 5+k : (k+5)%24)。
    fn load_tdbtimes_solar_terms(base_path: &Path, year: i32) -> Option<Vec<f64>> {
        let path = base_path.join("data/TDBtimes.txt");
        let f = std::fs::File::open(path).ok()?;
        let mut lines = std::io::BufReader::new(f).lines();
        lines.next(); // skip header
        for line in lines {
            let line = line.ok()?;
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() < 27 {
                continue;
            }
            let row_year: i32 = tokens[0].parse().ok()?;
            if row_year != year {
                continue;
            }
            let jd0: f64 = tokens[1].parse().ok()?;
            let mut jds = Vec::with_capacity(24);
            for k in 0..24 {
                let idx = 3 + if k <= 17 { 5 + k } else { (k + 5) % 24 };
                let offset: f64 = tokens[idx].parse().ok()?;
                jds.push(jd0 + offset);
            }
            return Some(jds);
        }
        None
    }

    /// 定气测试：2026 年 24 节气 vs data/TDBtimes.txt。
    #[test]
    fn solar_term_2026_vs_tdbtimes() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let loader = DataLoaderNative::new(&base);
        let vsop = match load_earth_vsop87(&loader, "data/vsop87/VSOP87B.ear") {
            Ok(v) => v,
            Err(_) => {
                println!("solar_term_2026_vs_tdbtimes: skipped (data/vsop87/VSOP87B.ear not found)");
                return;
            }
        };
        // 加载 data/IAU2000 完整章动后容差 30 s；未加载则用 77 项（残差约 300 s）
        match table_parser::load_tab53a(&loader, "data/IAU2000/tab5.3a.txt") {
            Ok(quads) if !quads.is_empty() => {
                let n = quads.len();
                let iau = Iau2000a::from_quads(quads);
                nutation::set_nutation_override(Some(Box::new(move |t| iau.nutation(t))));
                println!("  [章动] 已加载 data/IAU2000/tab5.3a.txt ({} 项)，定气用完整 IAU2000A", n);
            }
            Ok(_) => println!("  [章动] data/IAU2000/tab5.3a.txt 为空，用 77 项"),
            Err(e) => println!("  [章动] 未加载 data/IAU2000/tab5.3a.txt ({})，用 77 项", e),
        }
        // doc §9：方案二 VSOP87 + P03 岁差 + IAU 2000A 章动；默认 pipeline 即可。
        let our_jds = solar_term_jds_for_year(&vsop, 2026);
        assert_eq!(our_jds.len(), 24, "应得 24 个节气");

        const NAMES: [&str; 24] = [
            "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑", "立秋", "处暑", "白露",
            "秋分", "寒露", "霜降", "立冬", "小雪", "大雪", "冬至", "小寒", "大寒", "立春", "雨水", "惊蛰",
        ];
        /// 容差 30 s（TDBtimes 为 IAU2006/2000A）。算法需与参考一致，不得放宽容差掩盖错误。
        const TOLERANCE_SEC: f64 = 30.0;

        if let Some(ref_tdb) = load_tdbtimes_solar_terms(&base, 2026) {
            // 诊断：在参考春分时刻 ref_tt 处视黄经及中间量（默认 pipeline）
            let ref_tt_spring = TimePoint::new(TimeScale::TDB, real(ref_tdb[0])).to_scale(TimeScale::TT).jd;
            let t_ref = TimePoint::new(TimeScale::TT, ref_tt_spring);
            let (_lam, diag) = crate::astronomy::apparent::sun_apparent_ecliptic_longitude_diagnostic(&vsop, t_ref);
            let lam_dev_rad = diag.lambda.rad().wrap_to_signed_pi().as_f64();
            let lam_dev_arcsec = lam_dev_rad * 648000.0 / std::f64::consts::PI;
            let dpsi_sec = diag.dpsi.rad() * 648000.0 / std::f64::consts::PI;
            let deps_sec = diag.deps.rad() * 648000.0 / std::f64::consts::PI;
            let lam_mean_rad = diag.lambda_mean_ecliptic.rad().wrap_to_signed_pi().as_f64();
            let lam_mean_arcsec = lam_mean_rad * 648000.0 / std::f64::consts::PI;
            println!("  [诊断] 参考春分 JD(TT)={:.4} 处 λ 偏离 0° = {:.4} rad = {:.2}″ (负=本实现偏晚)", ref_tt_spring, lam_dev_rad, lam_dev_arcsec);
            println!("         平黄经(仅岁差)偏离 0° = {:.2}″，视−平 ≈ {:.2}″（章动等）",
                lam_mean_arcsec, lam_dev_arcsec - lam_mean_arcsec);
            println!("         t_cent={:.8}, Δψ={:.4}″, Δε={:.4}″, P_diag=[{:.8}, {:.8}, {:.8}], ε_mean={:.6} rad, ε_true={:.6} rad",
                diag.t_cent, dpsi_sec, deps_sec,
                diag.precession_diag[0], diag.precession_diag[1], diag.precession_diag[2],
                diag.eps_mean.rad(), diag.eps_true.rad());
            // 与定朔一致：均在 TDB 下比较（our_jds 为 TT，转为 TDB 后与 ref_tdb 比较）
            println!("  节气    本实现−TDB(s)");
            for k in 0..24 {
                let our_jd_tdb = TimePoint::new(TimeScale::TT, our_jds[k]).to_scale(TimeScale::TDB).jd;
                let diff_sec = (our_jd_tdb.as_f64() - ref_tdb[k]) * 86400.0;
                let name = if k < NAMES.len() { NAMES[k] } else { "?" };
                println!("  {}  {:+.1}", name, diff_sec);
                assert!(
                    diff_sec.abs() <= TOLERANCE_SEC,
                    "2026 节气 {} (k={}): 本实现−TDB = {:.1} s，超过容差 {} s",
                    name,
                    k,
                    diff_sec.abs(),
                    TOLERANCE_SEC
                );
            }
        } else {
            println!("solar_term_2026_vs_tdbtimes: data/TDBtimes.txt 无 2026 行，仅校验 24 节气计算完成");
        }
        nutation::set_nutation_override(None);
    }
}
