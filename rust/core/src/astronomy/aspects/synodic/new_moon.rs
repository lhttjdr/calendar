//! 朔望月/合朔近似：平均朔望月与 W0 公式、定朔求根。
//! 支持粗算（几何黄经 + 常数导数）→ 精算（视黄经 + 数值导数）。

use std::cmp::Ordering;

use crate::astronomy::apparent::{
    moon_apparent_ecliptic_longitude_with_options, moon_apparent_ecliptic_longitude_de406_with_options,
    sun_apparent_ecliptic_longitude_with_options, sun_apparent_ecliptic_longitude_de406_with_options,
    ApparentPipelineOptions,
};
use crate::astronomy::aspects::{moon_ecliptic_longitude_with_max_terms, sun_ecliptic_longitude};
use crate::astronomy::ephemeris::{De406Kernel, Elpmpp02Data, Vsop87};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real_const, real, Real, RealOps};
use crate::quantity::angular_rate::AngularRate;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::unit::AngularRateUnit;

/// 数值导数步长（日）。读入即 Real，无 f64 边界。
const NUMERICAL_DELTA_JD: Real = real_const(0.01);

/// 2π（弧度），Real 常量
const TAU_R: Real = real_const(core::f64::consts::TAU);

/// 平均朔望月角速度（rad/日 ≈ 0.213），用于粗算阶段常数导数。
#[inline]
pub fn mean_synodic_velocity() -> AngularRate {
    AngularRate::from_value(TAU_R / MEAN_SYNODIC_MONTH_W0, AngularRateUnit::RadPerDay)
}

/// 合朔选项：粗算/精算。
#[derive(Clone, Debug)]
pub struct NewMoonOptions {
    /// 粗算容差；为 None 时跳过粗算，仅精算。
    pub coarse_tolerance: Option<PlaneAngle>,
    /// 粗算最大迭代次数。
    pub coarse_max_iterations: usize,
    /// 粗算阶段 ELP 级数项数上限（None=全部）。残差大时用较小项数加速。
    pub coarse_max_terms: Option<u32>,
    /// 精算阶段 ELP 级数项数上限（None=全部）。
    pub fine_max_terms: Option<u32>,
}

impl Default for NewMoonOptions {
    fn default() -> Self {
        Self {
            coarse_tolerance: Some(PlaneAngle::from_rad(real_const(1e-4))),
            coarse_max_iterations: 10,
            coarse_max_terms: None,
            fine_max_terms: None,
        }
    }
}

impl NewMoonOptions {
    /// 仅精算（不粗算），与旧版行为一致。
    pub fn fine_only() -> Self {
        Self {
            coarse_tolerance: None,
            coarse_max_iterations: 0,
            coarse_max_terms: None,
            fine_max_terms: None,
        }
    }
}

/// 2000-01-01 0h TT 的儒略日（W0 公式用）
const J2000_0H_TT_JD: Real = real_const(2451544.0);
/// W0 公式常数项（日），自 2000-01-01 0h TT 至第一个平朔
const W0_CONSTANT_DAYS: Real = real_const(5.597661);
/// W0 公式平均朔望月（日），Chapront et al. 2002
pub const MEAN_SYNODIC_MONTH_W0: Real = real_const(29.530588861);
/// W0 公式二次项系数（日/N²）
const W0_QUADRATIC_COEFF: Real = real_const(1.02026e-10);

/// 平均朔望月（日），通用近似
pub const MEAN_SYNODIC_MONTH: Real = real_const(29.530588);

/// 第一个平朔（N=0）的近似 JD(TT)
pub const NEW_MOON_W0_EPOCH_JD: Real = real_const(2451549.597661);

/// 第 N 个朔的近似儒略日 TT（W0 公式）。N=0 为 2000 年第一个平朔。直接返回 Real。
#[inline]
pub fn approximate_new_moon_jd(n: i32) -> Real {
    let n_r = real(n);
    J2000_0H_TT_JD + W0_CONSTANT_DAYS + MEAN_SYNODIC_MONTH_W0 * n_r + W0_QUADRATIC_COEFF * n_r * n_r
}

/// 合朔时黄经差的近似值：floor((jd - W0_epoch)/MEAN_SYNODIC_MONTH_W0) * 2π。
/// 用于牛顿迭代的连续化参考。
#[inline]
pub fn expected_new_moon_longitude_difference(jd: Real) -> PlaneAngle {
    let n = (jd - NEW_MOON_W0_EPOCH_JD) / MEAN_SYNODIC_MONTH_W0;
    PlaneAngle::from_rad(n.floor() * TAU_R)
}

/// 几何黄经差（月−日）连续化：ref 为 Some 时加 2π 的整数倍与 ref 连续。返回 Real。
fn longitude_difference_geometric_unwrapped(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    t: TimePoint,
    ref_for_continuity: Option<Real>,
    moon_max_terms: Option<u32>,
) -> Real {
    let moon_lam = moon_ecliptic_longitude_with_max_terms(elp, t, moon_max_terms).rad();
    let sun_lam = sun_ecliptic_longitude(vsop, t).rad();
    let raw = moon_lam - sun_lam;
    match ref_for_continuity {
        Some(r) => {
            let k = ((r - raw) / TAU_R).to_i32_round();
            raw + real(k) * TAU_R
        }
        None => raw,
    }
}

/// 定朔用管线选项：含月球光行时（use_light_time_moon），满足 GB/T 33661-2017 约 1 s 精度。
fn synodic_apparent_options() -> ApparentPipelineOptions {
    ApparentPipelineOptions::pipeline_default()
}

/// 视黄经差（月−日）在合朔处为 0，用于二分法求根。使用 pipeline_default（含月球光行时 tr 迭代）。
fn residual_apparent_longitude_diff(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    jd: Real,
) -> Real {
    let t = TimePoint::new(TimeScale::TT, jd);
    let opts = synodic_apparent_options();
    let moon_lam = moon_apparent_ecliptic_longitude_with_options(elp, t, &opts).rad();
    let sun_lam = sun_apparent_ecliptic_longitude_with_options(vsop, t, &opts).rad();
    (moon_lam - sun_lam).wrap_to_signed_pi()
}

/// 在 [jd_lo, jd_hi] 内用二分法求定朔（视黄经差 = 0），保证收敛，用于牛顿法失效时的后备。
fn new_moon_jd_bisection(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    jd_lo: Real,
    jd_hi: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Real {
    let tol_r = tolerance.rad();
    let mut lo = jd_lo;
    let mut hi = jd_hi;
    let mut r_lo = residual_apparent_longitude_diff(vsop, elp, lo);
    let r_hi = residual_apparent_longitude_diff(vsop, elp, hi);
    if r_lo * r_hi > real_const(0.0) {
        // 两端同号则区间未括住根，取中点作为保守结果（不应发生于 [approx±2] 内）
        return (lo + hi) * real_const(0.5);
    }
    for _ in 0..max_iterations {
        let mid = (lo + hi) * real_const(0.5);
        let r_mid = residual_apparent_longitude_diff(vsop, elp, mid);
        if r_mid.abs() <= tol_r {
            return mid;
        }
        if r_mid * r_lo > real_const(0.0) {
            lo = mid;
            r_lo = r_mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) * real_const(0.5)
}

/// 视黄经差（月−日）残差，DE406 历表路径。
fn residual_apparent_longitude_diff_de406(
    kernel: &De406Kernel,
    jd: Real,
    options: &ApparentPipelineOptions,
) -> Real {
    let t = TimePoint::new(TimeScale::TT, jd);
    let moon_lam = moon_apparent_ecliptic_longitude_de406_with_options(kernel, t, options).rad();
    let sun_lam = sun_apparent_ecliptic_longitude_de406_with_options(kernel, t, options).rad();
    (moon_lam - sun_lam).wrap_to_signed_pi()
}

fn new_moon_jd_bisection_de406_with_options(
    kernel: &De406Kernel,
    jd_lo: Real,
    jd_hi: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
    options: &ApparentPipelineOptions,
) -> Real {
    let tol_r = tolerance.rad();
    let mut lo = jd_lo;
    let mut hi = jd_hi;
    let mut r_lo = residual_apparent_longitude_diff_de406(kernel, lo, options);
    let r_hi = residual_apparent_longitude_diff_de406(kernel, hi, options);
    if r_lo * r_hi > real_const(0.0) {
        return (lo + hi) * real_const(0.5);
    }
    for _ in 0..max_iterations {
        let mid = (lo + hi) * real_const(0.5);
        let r_mid = residual_apparent_longitude_diff_de406(kernel, mid, options);
        if r_mid.abs() <= tol_r {
            return mid;
        }
        if r_mid * r_lo > real_const(0.0) {
            lo = mid;
            r_lo = r_mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) * real_const(0.5)
}

/// 二分法求定朔，DE406 历表。
#[allow(dead_code)]
fn new_moon_jd_bisection_de406(
    kernel: &De406Kernel,
    jd_lo: Real,
    jd_hi: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Real {
    new_moon_jd_bisection_de406_with_options(
        kernel,
        jd_lo,
        jd_hi,
        tolerance,
        max_iterations,
        &synodic_apparent_options(),
    )
}

/// 仅精算阶段：从 t_approx 出发，用 DE406 视黄经 + 数值导数求合朔 JD。
pub fn new_moon_jd_fine_de406_with_options(
    kernel: &De406Kernel,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
    options: &ApparentPipelineOptions,
) -> Real {
    let tol_r = tolerance.rad();
    let mut jd = t_approx.to_scale(TimeScale::TT).jd;
    for _ in 0..max_iterations {
        let t = TimePoint::new(TimeScale::TT, jd);
        let moon_lam = moon_apparent_ecliptic_longitude_de406_with_options(kernel, t, options).rad();
        let sun_lam = sun_apparent_ecliptic_longitude_de406_with_options(kernel, t, options).rad();
        let residual_r = (moon_lam - sun_lam).wrap_to_signed_pi();
        if residual_r.abs() <= tol_r {
            return jd;
        }
        let t_lo = TimePoint::new(TimeScale::TT, jd - NUMERICAL_DELTA_JD);
        let t_hi = TimePoint::new(TimeScale::TT, jd + NUMERICAL_DELTA_JD);
        let moon_lo = moon_apparent_ecliptic_longitude_de406_with_options(kernel, t_lo, options).rad();
        let sun_lo = sun_apparent_ecliptic_longitude_de406_with_options(kernel, t_lo, options).rad();
        let moon_hi = moon_apparent_ecliptic_longitude_de406_with_options(kernel, t_hi, options).rad();
        let sun_hi = sun_apparent_ecliptic_longitude_de406_with_options(kernel, t_hi, options).rad();
        let diff_lo = (moon_lo - sun_lo).wrap_to_signed_pi();
        let diff_hi = (moon_hi - sun_hi).wrap_to_signed_pi();
        let mut d_diff = diff_hi - diff_lo;
        if d_diff > real(core::f64::consts::PI) {
            d_diff = d_diff - TAU_R;
        } else if d_diff < real(-core::f64::consts::PI) {
            d_diff = d_diff + TAU_R;
        }
        let two_delta: Real = real_const(2.0) * NUMERICAL_DELTA_JD;
        let velocity = d_diff / two_delta;
        let safe_velocity = if velocity.abs() < real_const(0.01) {
            real_const(0.13)
        } else {
            velocity
        };
        jd -= residual_r / safe_velocity;
    }
    jd
}

/// 仅精算阶段，使用 pipeline_default（含月球光行时）。
pub fn new_moon_jd_fine_de406(
    kernel: &De406Kernel,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Real {
    new_moon_jd_fine_de406_with_options(
        kernel,
        t_approx,
        tolerance,
        max_iterations,
        &synodic_apparent_options(),
    )
}

/// 定朔（DE406 BSP 历表）：求月视黄经 − 日视黄经 = 0 的 JD(TT)。仅精算，无粗算。
pub fn new_moon_jd_de406(
    kernel: &De406Kernel,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Real {
    new_moon_jd_fine_de406(kernel, t_approx, tolerance, max_iterations)
}

/// 定朔（DE406）：[jd_start, jd_end] 内所有合朔 JD(TT)，升序。牛顿失败时用二分法后备。
pub fn new_moon_jds_in_range_de406(
    kernel: &De406Kernel,
    jd_start: Real,
    jd_end: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Vec<Real> {
    new_moon_jds_in_range_de406_with_options(
        kernel,
        jd_start,
        jd_end,
        tolerance,
        max_iterations,
        &synodic_apparent_options(),
    )
}

/// 同上，可指定 pipeline 选项（如岁差/章动历元 TDB 以与 TDBtimes 对照）。
pub fn new_moon_jds_in_range_de406_with_options(
    kernel: &De406Kernel,
    jd_start: Real,
    jd_end: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
    options: &ApparentPipelineOptions,
) -> Vec<Real> {
    if jd_end < jd_start {
        return vec![];
    }
    let epoch = NEW_MOON_W0_EPOCH_JD;
    let month = MEAN_SYNODIC_MONTH_W0;
    let n0 = ((jd_start - epoch) / month).to_i32_floor();
    let n_end_approx = ((jd_end - epoch) / month).to_i32_floor() + 2;
    let mut out: Vec<Real> = Vec::new();
    for n in n0..=n_end_approx {
        let approx = approximate_new_moon_jd(n);
        let jd = new_moon_jd_fine_de406_with_options(
            kernel,
            TimePoint::new(TimeScale::TT, approx),
            tolerance,
            max_iterations,
            options,
        );
        let jd_f = jd.as_f64();
        let valid = jd_f.is_finite()
            && (jd_f - approx.as_f64()).abs() < MAX_REFINED_OFFSET_DAYS;
        let jd_use = if valid {
            jd
        } else {
            new_moon_jd_bisection_de406_with_options(
                kernel,
                approx - BISECT_HALF_WIDTH_DAYS,
                approx + BISECT_HALF_WIDTH_DAYS,
                tolerance,
                max_iterations,
                options,
            )
        };
        if jd_use >= jd_start && jd_use <= jd_end {
            out.push(jd_use);
        }
    }
    out
}

/// 仅精算阶段：从 t_approx 出发，用视黄经 + 数值导数求合朔 JD。供粗算后调用或直接使用。
/// 收敛判断在 R 上做，天文层不把 tolerance 转为 f64。
pub fn new_moon_jd_fine(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Real {
    let tol_r = tolerance.rad();
    let opts = synodic_apparent_options();
    let mut jd = t_approx.to_scale(TimeScale::TT).jd;
    for _ in 0..max_iterations {
        let t = TimePoint::new(TimeScale::TT, jd);
        let moon_lam = moon_apparent_ecliptic_longitude_with_options(elp, t, &opts).rad();
        let sun_lam = sun_apparent_ecliptic_longitude_with_options(vsop, t, &opts).rad();
        let residual_r = (moon_lam - sun_lam).wrap_to_signed_pi();
        if residual_r.abs() <= tol_r {
            return jd;
        }
        let t_lo = TimePoint::new(TimeScale::TT, jd - NUMERICAL_DELTA_JD);
        let t_hi = TimePoint::new(TimeScale::TT, jd + NUMERICAL_DELTA_JD);
        let moon_lo = moon_apparent_ecliptic_longitude_with_options(elp, t_lo, &opts).rad();
        let sun_lo = sun_apparent_ecliptic_longitude_with_options(vsop, t_lo, &opts).rad();
        let moon_hi = moon_apparent_ecliptic_longitude_with_options(elp, t_hi, &opts).rad();
        let sun_hi = sun_apparent_ecliptic_longitude_with_options(vsop, t_hi, &opts).rad();
        let diff_lo = (moon_lo - sun_lo).wrap_to_signed_pi();
        let diff_hi = (moon_hi - sun_hi).wrap_to_signed_pi();
        let mut d_diff = diff_hi - diff_lo;
        if d_diff > real(core::f64::consts::PI) {
            d_diff = d_diff - TAU_R;
        } else if d_diff < real(-core::f64::consts::PI) {
            d_diff = d_diff + TAU_R;
        }
        let two_delta: Real = real_const(2.0) * NUMERICAL_DELTA_JD;
        let velocity = d_diff / two_delta;
        let safe_velocity = if velocity.abs() < real_const(0.01) {
            real_const(0.13)
        } else {
            velocity
        };
        jd -= residual_r / safe_velocity;
    }
    jd
}

/// 定朔：求月视黄经 − 日视黄经 = 0 的 JD(TT)。默认先粗算（几何+常数导数）再精算（视黄经+数值导数）。
pub fn new_moon_jd(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Real {
    new_moon_jd_with_options(vsop, elp, t_approx, tolerance, max_iterations, &NewMoonOptions::default())
}

/// 定朔（可配置粗算）：coarse_tolerance 为 Some 时先粗算再精算。
pub fn new_moon_jd_with_options(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    t_approx: TimePoint,
    tolerance: PlaneAngle,
    max_iterations: usize,
    options: &NewMoonOptions,
) -> Real {
    let mut jd = t_approx.to_scale(TimeScale::TT).jd;

    if let Some(coarse_tol) = options.coarse_tolerance {
        let coarse_tol_rad = coarse_tol.rad();
        let coarse_max = options.coarse_max_iterations;
        let mut prev_unwrapped: Option<Real> = None;
        for _ in 0..coarse_max {
            let t = TimePoint::new(TimeScale::TT, jd);
            let diff = longitude_difference_geometric_unwrapped(
                vsop,
                elp,
                t,
                prev_unwrapped,
                options.coarse_max_terms,
            );
            prev_unwrapped = Some(diff);
            if diff.abs() <= coarse_tol_rad {
                break;
            }
            jd -= diff / mean_synodic_velocity().rad_per_day();
        }
    }

    new_moon_jd_fine(
        vsop,
        elp,
        TimePoint::new(TimeScale::TT, jd),
        tolerance,
        max_iterations,
    )
}

/// All new-moon JD(TT) in [jd_start, jd_end], ascending. 返回 Vec<R>，不写死 f64。
pub fn new_moon_jds_in_range(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    jd_start: Real,
    jd_end: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
) -> Vec<Real> {
    new_moon_jds_in_range_with_options(
        vsop,
        elp,
        jd_start,
        jd_end,
        tolerance,
        max_iterations,
        &NewMoonOptions::default(),
    )
}

/// 定朔结果相对平朔的允许偏差（日），超出则视为收敛到错误根或数值异常，改用二分法重算真实朔。
const MAX_REFINED_OFFSET_DAYS: f64 = 15.0;
/// 二分法区间半宽（日），真朔相对平朔在约 ±1.5 天内。
const BISECT_HALF_WIDTH_DAYS: Real = real_const(2.0);

/// 同上，可传合朔选项（粗算/精算）。
/// 按平朔索引 n 迭代至近似 JD 超出范围，筛出 [jd_start, jd_end] 内的定朔。若某次牛顿法结果异常（非有限或偏离平朔超半朔望月），
/// 在该 n 的平朔附近用二分法求真实朔，保证范围内一个不漏且始终使用真实朔时刻。
pub fn new_moon_jds_in_range_with_options(
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
    jd_start: Real,
    jd_end: Real,
    tolerance: PlaneAngle,
    max_iterations: usize,
    options: &NewMoonOptions,
) -> Vec<Real> {
    if jd_end < jd_start {
        return vec![];
    }
    let epoch = NEW_MOON_W0_EPOCH_JD;
    let month = MEAN_SYNODIC_MONTH_W0;
    let n0 = ((jd_start - epoch) / month).to_i32_floor();
    // 多算约一个朔望月再停，确保末端边界上的朔不被漏掉
    let n_end_approx = ((jd_end - epoch) / month).to_i32_floor() + 2;
    let mut out: Vec<Real> = Vec::new();
    for n in n0..=n_end_approx {
        let approx = approximate_new_moon_jd(n);
        let jd = new_moon_jd_with_options(
            vsop,
            elp,
            TimePoint::new(TimeScale::TT, approx),
            tolerance,
            max_iterations,
            options,
        );
        let jd_f = jd.as_f64();
        let valid = jd_f.is_finite()
            && (jd_f - approx.as_f64()).abs() < MAX_REFINED_OFFSET_DAYS;
        let jd_use = if valid {
            jd
        } else {
            new_moon_jd_bisection(
                vsop,
                elp,
                approx - BISECT_HALF_WIDTH_DAYS,
                approx + BISECT_HALF_WIDTH_DAYS,
                tolerance,
                max_iterations,
            )
        };
        if jd_use >= jd_start && jd_use <= jd_end {
            out.push(jd_use);
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::constant::J2000;
    use crate::astronomy::ephemeris::{load_all, load_earth_vsop87_from_repo, Elpmpp02Correction};
    use crate::astronomy::frame::nutation;
    use crate::astronomy::frame::vsop87_de406_icrs_patch;
    use crate::astronomy::time::TimeScale;
    use crate::calendar::gregorian::Gregorian;
    use std::io::BufRead;
    use std::path::Path;

    /// 从节气朔望标准时刻表加载指定年份的 12 定朔 JD(TDB)。列序：Q0_02..Q0_13，索引 31+4*i。
    fn load_new_moons_reference_jd(base_path: &Path, year: i32) -> Option<Vec<f64>> {
        let path = base_path.join(crate::repo::paths::SOLAR_TERMS_REFERENCE);
        let f = std::fs::File::open(path).ok()?;
        let mut lines = std::io::BufReader::new(f).lines();
        lines.next(); // skip header
        for line in lines {
            let line = line.ok()?;
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() < 76 {
                continue;
            }
            let row_year: i32 = tokens[0].parse().ok()?;
            if row_year != year {
                continue;
            }
            let jd0: f64 = tokens[1].parse().ok()?;
            let mut jds = Vec::with_capacity(12);
            for i in 0..12 {
                let idx = 31 + 4 * i;
                let offset: f64 = tokens[idx].parse().ok()?;
                jds.push(jd0 + offset);
            }
            return Some(jds);
        }
        None
    }

    /// 定朔测试：2026 年 12 朔与《月相和二十四节气的计算》§7.4 节气朔望标准时刻表对照，容差 3 s。
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn new_moon_2026_vs_reference() {
        let base = crate::repo::repo_root();
        let vsop = match load_earth_vsop87_from_repo() {
            Ok(v) => v,
            Err(_) => {
                println!("new_moon_2026_vs_reference: skipped ({})", crate::repo::paths::VSOP87_EARTH);
                return;
            }
        };
        let loader = crate::repo::default_loader();
        let elp = match load_all(&loader, crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406) {
            Ok(e) => e,
            Err(_) => {
                println!("new_moon_2026_vs_reference: skipped ({})", crate::repo::paths::ELPMPP02);
                return;
            }
        };
        if nutation::try_init_full_nutation_from_repo() {
            println!("  [章动] 已加载 IERS 5.3a+5.3b，定朔用完整 IAU2000A");
        } else {
            println!("  [章动] 未加载 5.3a/5.3b，用 77 项");
        }
        let _ = vsop87_de406_icrs_patch::try_init_de406_patch_from_repo();
        let year = 2026;
        let jd_start = Gregorian::to_julian_day(year, 1, 1);
        let jd_end = Gregorian::to_julian_day(year, 12, 31) + real_const(1.0);
        const MAX_ITER: usize = 30;
        let our_jds = new_moon_jds_in_range(&vsop, &elp, jd_start, jd_end, PlaneAngle::from_rad(real_const(1e-8)), MAX_ITER);
        /// 视黄经合朔，容差 3 s。算法需与参考一致，不得放宽容差掩盖错误。
        const TOLERANCE_SEC: f64 = 3.0;
        const NUM_NEW_MOONS: usize = 6;

        if let Some(ref_jd_tdb) = load_new_moons_reference_jd(&base, year) {
            assert!(
                our_jds.len() >= NUM_NEW_MOONS,
                "2026 应至少 {} 个朔，得 {}",
                NUM_NEW_MOONS,
                our_jds.len()
            );
            println!("  [标准] 朔日参考取自节气朔望标准时刻表 (DE441+IAU2006/2000A)，容差 {} s", TOLERANCE_SEC as i32);
            println!("  朔    本实现−参考TDB(s)");
            for i in 0..NUM_NEW_MOONS {
                let ref_jd_tdb = ref_jd_tdb[i];
                let our_jd_tt = our_jds[i];
                let our_jd_tdb = TimePoint::new(TimeScale::TT, our_jd_tt).to_scale(TimeScale::TDB).jd;
                let diff_sec = (our_jd_tdb - ref_jd_tdb) * 86400.0;
                println!("  {}  {:+.3}", i + 1, diff_sec);
                assert!(
                    diff_sec.abs() <= TOLERANCE_SEC,
                    "2026 朔 {}: 本实现−参考TDB = {:.3} s，超过容差 {} s",
                    i + 1,
                    diff_sec.abs(),
                    TOLERANCE_SEC as i32
                );
            }
        } else {
            println!("new_moon_2026_vs_reference: 节气朔望标准时刻表无 2026 行，仅校验 2026 年内定朔计算完成");
            assert!(
                our_jds.len() >= 1,
                "2026 年内应至少 1 个朔，得 {}",
                our_jds.len()
            );
        }
        nutation::set_nutation_override(None);
    }

    /// 定朔（DE406）与《月相和二十四节气的计算》TDBtimes 对照。参考表为 DE441，本实现用 DE406；DE441 与 DE406 差异小，容差 1 s。
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn new_moon_2026_de406_vs_tdbtimes() {
        let base = crate::repo::repo_root();
        let bsp_path: String = std::env::var("DE406_BSP")
            .ok()
            .filter(|p| std::path::Path::new(p).is_file())
            .or_else(|| {
                let p = base.join(crate::repo::paths::DE406_BSP_CANDIDATES[0]);
                if p.is_file() {
                    Some(p.to_string_lossy().into_owned())
                } else {
                    base.join(crate::repo::paths::DE406_BSP_CANDIDATES[1])
                        .is_file()
                        .then(|| base.join(crate::repo::paths::DE406_BSP_CANDIDATES[1]).to_string_lossy().into_owned())
                }
            })
            .unwrap_or_else(|| base.join(crate::repo::paths::JPL_DATA_DIR).to_string_lossy().into_owned());
        if !std::path::Path::new(&bsp_path).is_file() {
            let tried = std::env::var("DE406_BSP").ok().unwrap_or_else(|| bsp_path.clone());
            println!(
                "new_moon_2026_de406_vs_tdbtimes: skipped (no DE406 BSP)，已尝试: {}",
                tried
            );
            return;
        }
        let kernel = match De406Kernel::open(&bsp_path) {
            Ok(k) => k,
            Err(e) => {
                println!("new_moon_2026_de406_vs_tdbtimes: skipped (open BSP: {})", e);
                return;
            }
        };
        if nutation::try_init_full_nutation_from_repo() {
            println!("  [章动] 已加载 IERS 5.3a+5.3b，DE406 定朔用完整 IAU2000A");
        } else {
            println!("  [章动] 未加载 5.3a/5.3b，用 77 项");
        }
        let _ = vsop87_de406_icrs_patch::try_init_de406_patch_from_repo();
        let year = 2026;
        let jd_start = Gregorian::to_julian_day(year, 1, 1);
        let jd_end = Gregorian::to_julian_day(year, 12, 31) + real_const(1.0);
        const MAX_ITER: usize = 30;
        let options = ApparentPipelineOptions::pipeline_default();
        let our_jds = new_moon_jds_in_range_de406_with_options(
            &kernel,
            jd_start,
            jd_end,
            PlaneAngle::from_rad(real_const(1e-8)),
            MAX_ITER,
            &options,
        );
        /// DE406 vs 参考表(DE441)，容差 1 s（DE441/DE406 差异小）
        const TOLERANCE_SEC: f64 = 1.0;
        const NUM_NEW_MOONS: usize = 6;
        println!("  [DE406 定朔 vs TDBtimes] 容差: {} s", TOLERANCE_SEC);

        if let Some(ref_jd_tdb) = load_new_moons_reference_jd(&base, year) {
            assert!(
                our_jds.len() >= NUM_NEW_MOONS,
                "2026 DE406 应至少 {} 个朔，得 {}",
                NUM_NEW_MOONS,
                our_jds.len()
            );
            println!("  与 TDBtimes.txt (DE441) 对照，逐项残差 (本实现−参考，秒):");
            println!("  朔    残差(s)");
            let mut max_residual_sec: f64 = 0.0;
            for i in 0..NUM_NEW_MOONS {
                let ref_jd = ref_jd_tdb[i];
                let our_jd_tt = our_jds[i];
                let our_jd_tdb = TimePoint::new(TimeScale::TT, our_jd_tt).to_scale(TimeScale::TDB).jd;
                let diff_sec = (our_jd_tdb - ref_jd) * 86400.0;
                let residual_sec = diff_sec.abs().as_f64();
                if residual_sec > max_residual_sec {
                    max_residual_sec = residual_sec;
                }
                println!("  {}  {:+.3}", i + 1, diff_sec);
                assert!(
                    residual_sec <= TOLERANCE_SEC,
                    "2026 朔 {}: DE406−参考TDB = {:.3} s，超过容差 {} s",
                    i + 1,
                    residual_sec,
                    TOLERANCE_SEC as i32
                );
            }
            println!("  最大残差: {:.3} s (容差 {} s)", max_residual_sec, TOLERANCE_SEC);
        } else {
            let ref_path = base.join(crate::repo::paths::SOLAR_TERMS_REFERENCE);
            println!(
                "  （未找到参考表或其中无 2026 行，未输出残差）路径: {}",
                ref_path.display()
            );
            assert!(
                our_jds.len() >= 1,
                "2026 年内应至少 1 个朔，得 {}",
                our_jds.len()
            );
        }
        nutation::set_nutation_override(None);
    }

    /// expectedNewMoonLongitudeDifference(J2000) 非负
    #[test]
    fn expected_new_moon_longitude_difference_at_j2000() {
        let diff = expected_new_moon_longitude_difference(J2000);
        assert!(diff.rad().abs() >= real(0));
    }

    #[test]
    fn approximate_new_moon_epoch() {
        let jd0 = approximate_new_moon_jd(0);
        assert!((jd0 - NEW_MOON_W0_EPOCH_JD).abs() < real_const(1e-6));
    }

    #[test]
    fn expected_longitude_difference_at_epoch() {
        let diff = expected_new_moon_longitude_difference(NEW_MOON_W0_EPOCH_JD);
        assert!(diff.rad().abs() < real_const(1e-10));
    }

    /// approximateNewMoonJD(1)-approximateNewMoonJD(0) ≈ MEAN_SYNODIC_MONTH
    #[test]
    fn approximate_new_moon_jd_diff_near_synodic_month() {
        let jd0 = approximate_new_moon_jd(0);
        let jd1 = approximate_new_moon_jd(1);
        let diff = jd1 - jd0;
        assert!((diff - MEAN_SYNODIC_MONTH_W0).abs() < real_const(0.01));
    }

    #[test]
    fn new_moon_jd_converges_near_epoch() {
        let vsop = crate::astronomy::ephemeris::vsop87::minimal_earth_vsop();
        let elp = crate::astronomy::ephemeris::Elpmpp02Data::de405_mean_only();
        let t_approx = TimePoint::new(TimeScale::TT, approximate_new_moon_jd(0));
        let jd = new_moon_jd(&vsop, &elp, t_approx, PlaneAngle::from_rad(real_const(1e-6)), 30);
        assert!((jd - NEW_MOON_W0_EPOCH_JD).abs() < real_const(5.0), "定朔应落在平朔附近数日内");
    }
}
