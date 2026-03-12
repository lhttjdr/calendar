//! ELPMPP02/VSOP87 vs JPL DE406：Rust 测试内实时调用 Python/jplephem，内存传递，无需 CSV。
//! 运行：PYO3_PYTHON=.venv/bin/python cargo test elpmpp02_vs_jpl_de406_python
//!      PYO3_PYTHON=... cargo test vsop87_vs_jpl_de406_python
//!      PYO3_PYTHON=... cargo test de406_rust_vs_python_de406

#![cfg(all(test, not(target_arch = "wasm32"), feature = "python-test"))]

use super::*;
use crate::astronomy::apparent::sun_position_icrs;
use crate::astronomy::ephemeris::{load_earth_vsop87_from_repo, De406Kernel, Vsop87};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};
use pyo3::types::PyAnyMethods;

#[allow(dead_code)]
const PY_CODE: &str = r"
import math, os
def run(ephem_path, jd_list):
    try:
        from jplephem.spk import SPK
    except ImportError as e:
        return [], 'jplephem import failed: ' + str(e)
    p = ephem_path
    if os.path.isfile(p):
        pass
    elif os.path.isdir(p):
        # 优先用 SPK(.bsp)，再用 .406，避免选中同目录下的 JPL PLAN 格式 .406
        candidates = [
            'de406.bsp', os.path.join('de406', 'de406.bsp'),
            'linux_p1550p2650.406', 'unxp1550p2650.406', 'lnxm3000p3000.406',
            os.path.join('de406', 'linux_p1550p2650.406'), os.path.join('de406', 'unxp1550p2650.406'),
            os.path.join('de406', 'lnxp1550p2650.406'), os.path.join('de406', 'lnxm3000p3000.406'),
        ]
        for n in candidates:
            f = os.path.join(p, n)
            if os.path.isfile(f):
                p = f
                break
        else:
            return [], 'DE406 not found under ' + repr(ephem_path) + ' (need a .406/.bsp kernel file, e.g. lnxm3000p3000.406 or linux_p1550p2650.406 in data/jpl or data/jpl/de406/)'
    else:
        return [], 'DE406 path does not exist: ' + repr(ephem_path)
    try:
        with open(p, 'rb') as f:
            head = f.read(32)
        if head.startswith(b'JPL PLAN') or head.startswith(b'JPL EPHEM'):
            return [], ('file is legacy JPL planetary format (starts with JPL PLAN/EPHEM); '
                'jplephem SPK needs DAF/SPK format. Use de406.bsp from NAIF, or see data/jpl/README.txt')
    except Exception:
        pass
    try:
        kernel = SPK.open(p)
    except Exception as e:
        return [], 'SPK.open failed: ' + str(e)
    try:
        moon_emb = kernel[3, 301].compute(jd_list[0])
        earth_emb = kernel[3, 399].compute(jd_list[0])
    except (KeyError, TypeError) as e:
        return [], 'kernel segments (3,301)/(3,399) not found: ' + str(e)
    arcsec = math.pi / (180.0 * 3600.0)
    eps = (23*3600+26*60+21+0.41100) * arcsec
    phi = -0.05542 * arcsec
    out = []
    for jd in jd_list:
        moon_emb = kernel[3, 301].compute(jd)
        earth_emb = kernel[3, 399].compute(jd)
        x, y, z = (moon_emb - earth_emb)  # geocentric Moon, km, GCRF/ICRF
        cz, sz = math.cos(-phi), math.sin(-phi)
        x1, y1 = x*cz - y*sz, x*sz + y*cz
        ce, se = math.cos(eps), math.sin(eps)
        out.append([jd, x1, (y1*ce+z*se), (-y1*se+z*ce)])
    return out, ''
";

/// DE406 地心月球 in ICRS (GCRF) km，不转黄道。用于 ELPMPP02+Table7 与 DE406 在 ICRS 下对比。
const PY_CODE_MOON_ICRS: &str = r"
import os
def run_moon_icrs(ephem_path, jd_list):
    try:
        from jplephem.spk import SPK
    except ImportError as e:
        return [], 'jplephem import failed: ' + str(e)
    p = ephem_path
    if os.path.isfile(p):
        pass
    elif os.path.isdir(p):
        candidates = [
            'de406.bsp', os.path.join('de406', 'de406.bsp'),
            'linux_p1550p2650.406', 'unxp1550p2650.406', 'lnxm3000p3000.406',
            os.path.join('de406', 'linux_p1550p2650.406'), os.path.join('de406', 'unxp1550p2650.406'),
            os.path.join('de406', 'lnxm3000p2650.406'), os.path.join('de406', 'lnxm3000p3000.406'),
        ]
        for n in candidates:
            f = os.path.join(p, n)
            if os.path.isfile(f):
                p = f
                break
        else:
            return [], 'DE406 not found under ' + repr(ephem_path)
    else:
        return [], 'DE406 path does not exist: ' + repr(ephem_path)
    try:
        with open(p, 'rb') as f:
            head = f.read(32)
        if head.startswith(b'JPL PLAN') or head.startswith(b'JPL EPHEM'):
            return [], 'file is legacy JPL format; need de406.bsp (SPK)'
    except Exception:
        pass
    try:
        kernel = SPK.open(p)
        kernel[3, 301].compute(jd_list[0])
        kernel[3, 399].compute(jd_list[0])
    except Exception as e:
        return [], 'SPK or segments failed: ' + str(e)
    out = []
    for jd in jd_list:
        moon_emb = kernel[3, 301].compute(jd)
        earth_emb = kernel[3, 399].compute(jd)
        x, y, z = (moon_emb - earth_emb)
        out.append([jd, float(x), float(y), float(z)])
    return out, ''
";

/// DE406 地心太阳 in ICRS (GCRF) km，不转黄道。用于 VSOP87+patch 与 DE406 在 ICRS 下对比。
const PY_CODE_SUN_ICRS: &str = r"
import os
def run_sun_icrs(ephem_path, jd_list):
    try:
        from jplephem.spk import SPK
    except ImportError as e:
        return [], 'jplephem import failed: ' + str(e)
    p = ephem_path
    if os.path.isfile(p):
        pass
    elif os.path.isdir(p):
        candidates = [
            'de406.bsp', os.path.join('de406', 'de406.bsp'),
            'linux_p1550p2650.406', 'unxp1550p2650.406', 'lnxm3000p3000.406',
            os.path.join('de406', 'linux_p1550p2650.406'), os.path.join('de406', 'unxp1550p2650.406'),
            os.path.join('de406', 'lnxp1550p2650.406'), os.path.join('de406', 'lnxm3000p3000.406'),
        ]
        for n in candidates:
            f = os.path.join(p, n)
            if os.path.isfile(f):
                p = f
                break
        else:
            return [], 'DE406 not found under ' + repr(ephem_path)
    else:
        return [], 'DE406 path does not exist: ' + repr(ephem_path)
    try:
        with open(p, 'rb') as f:
            head = f.read(32)
        if head.startswith(b'JPL PLAN') or head.startswith(b'JPL EPHEM'):
            return [], 'file is legacy JPL format; need de406.bsp (SPK)'
    except Exception:
        pass
    try:
        kernel = SPK.open(p)
        sun_ssb = kernel[0, 10].compute(jd_list[0])
        emb_ssb = kernel[0, 3].compute(jd_list[0])
        earth_emb = kernel[3, 399].compute(jd_list[0])
    except Exception as e:
        return [], 'SPK.open or segments failed: ' + str(e)
    out = []
    for jd in jd_list:
        sun_ssb = kernel[0, 10].compute(jd)
        emb_ssb = kernel[0, 3].compute(jd)
        earth_emb = kernel[3, 399].compute(jd)
        earth_ssb = emb_ssb + earth_emb
        x, y, z = (sun_ssb - earth_ssb)
        out.append([jd, float(x), float(y), float(z)])
    return out, ''
";

/// DE406 地心日、地心月 ICRS (km, km/s)，用于 Rust DE406 与 Python/jplephem 直接核对（不经过 CSV）。
const PY_CODE_DE406_GEOCENTRIC: &str = r"
import os
def run_geocentric(bsp_path, jd_list):
    try:
        from jplephem.spk import SPK
    except ImportError as e:
        return [], 'jplephem import failed: ' + str(e)
    if not os.path.isfile(bsp_path):
        return [], 'BSP path is not a file: ' + repr(bsp_path)
    try:
        with open(bsp_path, 'rb') as f:
            head = f.read(8)
        if head.startswith(b'JPL PLAN') or head.startswith(b'JPL EPHEM'):
            return [], 'file is legacy JPL format; need de406.bsp (SPK)'
    except Exception:
        pass
    try:
        kernel = SPK.open(bsp_path)
        kernel[0, 10].compute_and_differentiate(jd_list[0])
        kernel[0, 3].compute_and_differentiate(jd_list[0])
        kernel[3, 399].compute_and_differentiate(jd_list[0])
        kernel[3, 301].compute_and_differentiate(jd_list[0])
    except Exception as e:
        return [], 'SPK.open or segments failed: ' + str(e)
    # jplephem compute_and_differentiate 返回速度为 km/day，转为 km/s 与 Rust 一致
    SEC_PER_DAY = 86400.0
    out = []
    for jd in jd_list:
        sun_pos, sun_vel = kernel[0, 10].compute_and_differentiate(jd)
        emb_pos, emb_vel = kernel[0, 3].compute_and_differentiate(jd)
        earth_emb_pos, earth_emb_vel = kernel[3, 399].compute_and_differentiate(jd)
        earth_pos = emb_pos + earth_emb_pos
        earth_vel = emb_vel + earth_emb_vel
        sx, sy, sz = (sun_pos - earth_pos)
        svx, svy, svz = (sun_vel - earth_vel) / SEC_PER_DAY
        moon_pos, moon_vel = kernel[3, 301].compute_and_differentiate(jd)
        earth_emb_pos, earth_emb_vel = kernel[3, 399].compute_and_differentiate(jd)
        mx, my, mz = (moon_pos - earth_emb_pos)
        mvx, mvy, mvz = (moon_vel - earth_emb_vel) / SEC_PER_DAY
        out.append([jd, float(sx), float(sy), float(sz), float(svx), float(svy), float(svz),
                    float(mx), float(my), float(mz), float(mvx), float(mvy), float(mvz)])
    return out, ''
";

/// DE406 地心太阳在 J2000 平黄道 (x,y,z) km；与 PY_CODE 同路径与旋转。
#[allow(dead_code)]
const PY_CODE_SUN: &str = r"
import math, os
def run_sun(ephem_path, jd_list):
    try:
        from jplephem.spk import SPK
    except ImportError as e:
        return [], 'jplephem import failed: ' + str(e)
    p = ephem_path
    if os.path.isfile(p):
        pass
    elif os.path.isdir(p):
        candidates = [
            'de406.bsp', os.path.join('de406', 'de406.bsp'),
            'linux_p1550p2650.406', 'unxp1550p2650.406', 'lnxm3000p3000.406',
            os.path.join('de406', 'linux_p1550p2650.406'), os.path.join('de406', 'unxp1550p2650.406'),
            os.path.join('de406', 'lnxp1550p2650.406'), os.path.join('de406', 'lnxm3000p3000.406'),
        ]
        for n in candidates:
            f = os.path.join(p, n)
            if os.path.isfile(f):
                p = f
                break
        else:
            return [], 'DE406 not found under ' + repr(ephem_path)
    else:
        return [], 'DE406 path does not exist: ' + repr(ephem_path)
    try:
        with open(p, 'rb') as f:
            head = f.read(32)
        if head.startswith(b'JPL PLAN') or head.startswith(b'JPL EPHEM'):
            return [], 'file is legacy JPL format; need de406.bsp (SPK)'
    except Exception:
        pass
    try:
        kernel = SPK.open(p)
        sun_ssb = kernel[0, 10].compute(jd_list[0])
        emb_ssb = kernel[0, 3].compute(jd_list[0])
        earth_emb = kernel[3, 399].compute(jd_list[0])
    except Exception as e:
        return [], 'SPK.open or segments failed: ' + str(e)
    arcsec = math.pi / (180.0 * 3600.0)
    eps = (23*3600+26*60+21+0.41100) * arcsec
    phi = -0.05542 * arcsec
    out = []
    for jd in jd_list:
        sun_ssb = kernel[0, 10].compute(jd)
        emb_ssb = kernel[0, 3].compute(jd)
        earth_emb = kernel[3, 399].compute(jd)
        earth_ssb = emb_ssb + earth_emb
        x, y, z = (sun_ssb - earth_ssb)  # geocentric Sun, km, ICRF
        cz, sz = math.cos(-phi), math.sin(-phi)
        x1, y1 = x*cz - y*sz, x*sz + y*cz
        ce, se = math.cos(eps), math.sin(eps)
        out.append([jd, x1, (y1*ce+z*se), (-y1*se+z*ce)])
    return out, ''
";

/// 在首次 Py_Initialize 前设置 Python 标准库路径（嵌入解释器常因 prefix 错误找不到 encodings）。
/// 需在 with_gil 之前调用；Py_SetPythonHome 接受 wchar_t*，Linux 上为 UTF-32。
fn set_python_home_if_requested() {
    let Ok(home) = std::env::var("PYTHONHOME") else { return };
    if home.is_empty() {
        return;
    }
    let wide: Vec<u32> = home.chars().map(|c| c as u32).chain(std::iter::once(0)).collect();
    let leaked: &'static [u32] = Box::leak(wide.into_boxed_slice());
    let ptr = leaked.as_ptr();
    unsafe {
        #[allow(deprecated)] // PyConfig 需更多 FFI 设置，暂用旧 API
        #[allow(clippy::cast_possible_wrap)]
        pyo3::ffi::Py_SetPythonHome(ptr as *const i32);
    }
}

#[test]
fn elpmpp02_vs_jpl_de406_python() {
    if std::env::var("PYO3_PYTHON").is_err() {
        println!("elpmpp02_vs_jpl_de406_python: skipped (PYO3_PYTHON not set)");
        return;
    }
    set_python_home_if_requested();
    let base = crate::repo::repo_root();
    let ephem_path: String = std::env::var("DE406_PATH")
        .unwrap_or_else(|_| base.join(crate::repo::paths::JPL_DATA_DIR).to_string_lossy().into_owned());
    let jds: Vec<f64> = vec![2433282.5, 2444239.5, 2451545.0, 2455000.0, 2473400.5];

    let (jpl_rows, _jpl_error): (Vec<Vec<f64>>, String) = match pyo3::Python::with_gil(|py| -> pyo3::PyResult<(Vec<Vec<f64>>, String)> {
        #[allow(deprecated)]
        let mod_ = pyo3::types::PyModule::from_code_bound(py, PY_CODE_MOON_ICRS, "jpl_moon_icrs.py", "jpl_moon_icrs")?;
        let func = mod_.getattr("run_moon_icrs")?;
        let tuple = func.call1((ephem_path.as_str(), jds))?;
        tuple.extract()
    }) {
        Ok((rows, _)) if !rows.is_empty() => (rows, String::new()),
        Ok((_, msg)) => {
            println!("elpmpp02_vs_jpl_de406_python: skipped ({})", if msg.is_empty() { "no JPL rows" } else { msg.trim() });
            return;
        }
        Err(e) => {
            println!("elpmpp02_vs_jpl_de406_python: skipped (Python error: {})", e);
            return;
        }
    };

    let loader = crate::repo::default_loader();
    let data = match load_all(&loader, crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406) {
        Ok(d) => d,
        Err(_) => {
            println!("elpmpp02_vs_jpl_de406_python: skipped (ELPMPP02 data not loaded)");
            return;
        }
    };

    fn tol_km(jd: f64) -> f64 {
        if jd >= 2433282.5 && jd <= 2473400.5 {
            0.2
        } else if jd >= 2268922.5 && jd <= 2637936.5 {
            1.0
        } else {
            10.0
        }
    }

    const RAD_TO_ARCSEC: f64 = 180.0 * 3600.0 / std::f64::consts::PI;
    fn xyz_to_lbr(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let r = (x * x + y * y + z * z).sqrt();
        let l = y.atan2(x);
        let b = if r > 0.0 { (z / r).asin() } else { 0.0 };
        (l, b, r)
    }

    println!("JD(TDB)     ELP+Table7 ICRS (km)    DE406 ICRS (km)          residual dx,dy,dz (km)   |dr|(km)  tol(km)");
    for row in &jpl_rows {
        if row.len() < 4 {
            continue;
        }
        let jd_tdb = row[0];
        let (x_jpl, y_jpl, z_jpl) = (row[1], row[2], row[3]);
        let jd_tt = TimePoint::new(TimeScale::TDB, real(jd_tdb)).to_scale(TimeScale::TT).jd;
        let (pos_m, _) = position_velocity(&data, TimePoint::new(TimeScale::TT, jd_tt));
        let pm = pos_m.to_meters();
        let (x_ecl, y_ecl, z_ecl) = (
            (pm[0] / 1000.0).as_f64(),
            (pm[1] / 1000.0).as_f64(),
            (pm[2] / 1000.0).as_f64(),
        );
        let (x_elp, y_elp, z_elp) = super::table7::ecliptic_j2000_to_icrs(x_ecl, y_ecl, z_ecl);
        let (dx, dy, dz) = (x_elp - x_jpl, y_elp - y_jpl, z_elp - z_jpl);
        let dr_km = (dx * dx + dy * dy + dz * dz).sqrt();
        let tol = tol_km(jd_tdb);
        println!(
            "{:.1}  ({:.3},{:.3},{:.3})  ({:.3},{:.3},{:.3})  ({:+.6},{:+.6},{:+.6})  {:.6}  {:.1}",
            jd_tdb, x_elp, y_elp, z_elp, x_jpl, y_jpl, z_jpl, dx, dy, dz, dr_km, tol
        );
        let (l_elp, b_elp, r_elp) = xyz_to_lbr(x_elp, y_elp, z_elp);
        let (l_jpl, b_jpl, r_jpl) = xyz_to_lbr(x_jpl, y_jpl, z_jpl);
        let dl_rad = l_elp - l_jpl;
        let dl_rad_wrap = if dl_rad > std::f64::consts::PI {
            dl_rad - 2.0 * std::f64::consts::PI
        } else if dl_rad < -std::f64::consts::PI {
            dl_rad + 2.0 * std::f64::consts::PI
        } else {
            dl_rad
        };
        println!(
            "           LBR: L={:.6},{:.6} rad  B={:.6},{:.6} rad  R={:.1},{:.1} km  => dL={:+.4}\" dB={:+.4}\" dR={:+.4} km",
            l_elp, l_jpl, b_elp, b_jpl, r_elp, r_jpl,
            dl_rad_wrap * RAD_TO_ARCSEC,
            (b_elp - b_jpl) * RAD_TO_ARCSEC,
            r_elp - r_jpl
        );
        assert!(
            dx.abs() <= tol && dy.abs() <= tol && dz.abs() <= tol,
            "JD {} ELP+Table7=({:.3},{:.3},{:.3}) DE406=({:.3},{:.3},{:.3}) tol={}",
            jd_tdb, x_elp, y_elp, z_elp, x_jpl, y_jpl, z_jpl, tol
        );
    }
}

/// VSOP87B + FK5→ICRS + DE406 patch 地心太阳 vs JPL DE406（ICRS km），容差 12 000 km。
#[test]
fn vsop87_vs_jpl_de406_python() {
    if std::env::var("PYO3_PYTHON").is_err() {
        println!("vsop87_vs_jpl_de406_python: skipped (PYO3_PYTHON not set)");
        return;
    }
    set_python_home_if_requested();
    let base = crate::repo::repo_root();
    let ephem_path: String = std::env::var("DE406_PATH")
        .unwrap_or_else(|_| base.join(crate::repo::paths::JPL_DATA_DIR).to_string_lossy().into_owned());
    let jds: Vec<f64> = vec![
        2444239.5, 2451545.0, 2455000.0, 2268922.5, 2637936.5,
        2300000.0, 2400000.0, 2500000.0, 2600000.0,
    ];

    let (jpl_rows, _err): (Vec<Vec<f64>>, String) = match pyo3::Python::with_gil(|py| -> pyo3::PyResult<(Vec<Vec<f64>>, String)> {
        #[allow(deprecated)]
        let mod_ = pyo3::types::PyModule::from_code_bound(py, PY_CODE_SUN_ICRS, "jpl_sun_icrs.py", "jpl_sun_icrs")?;
        let func = mod_.getattr("run_sun_icrs")?;
        let tuple = func.call1((ephem_path.as_str(), jds.clone()))?;
        tuple.extract()
    }) {
        Ok((rows, _)) if !rows.is_empty() => (rows, String::new()),
        Ok((_, msg)) => {
            println!("vsop87_vs_jpl_de406_python: skipped ({})", if msg.is_empty() { "no JPL Sun rows" } else { msg.trim() });
            return;
        }
        Err(e) => {
            println!("vsop87_vs_jpl_de406_python: skipped (Python error: {})", e);
            return;
        }
    };

    let vsop: Vsop87 = match load_earth_vsop87_from_repo() {
        Ok(v) => v,
        Err(_) => {
            println!("vsop87_vs_jpl_de406_python: skipped (VSOP87 data not loaded)");
            return;
        }
    };

    /// 容差 km：ICRS 单点 12 000 km
    const TOL_KM: f64 = 12_000.0;
    const RAD_TO_ARCSEC: f64 = 180.0 * 3600.0 / std::f64::consts::PI;

    /// ICRS (x,y,z) km → L(rad), B(rad), R(km)
    fn xyz_to_lbr(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let r = (x * x + y * y + z * z).sqrt();
        let l = y.atan2(x);
        let b = if r > 0.0 { (z / r).asin() } else { 0.0 };
        (l, b, r)
    }

    println!("JD(TDB)     VSOP87+patch ICRS (km)    DE406 ICRS (km)          residual dx,dy,dz (km)   |dr|(km)  tol(km)");
    for row in &jpl_rows {
        if row.len() < 4 {
            continue;
        }
        let jd = row[0];
        let (x_jpl, y_jpl, z_jpl) = (row[1], row[2], row[3]);
        let t = TimePoint::new(TimeScale::TDB, real(jd));
        let [x_m, y_m, z_m] = sun_position_icrs(&vsop, t).to_meters();
        let (x_vsop, y_vsop, z_vsop) = (
            (x_m / 1000.0).as_f64(),
            (y_m / 1000.0).as_f64(),
            (z_m / 1000.0).as_f64(),
        );
        let (dx, dy, dz): (f64, f64, f64) = (x_vsop - x_jpl, y_vsop - y_jpl, z_vsop - z_jpl);
        let dr_km = (dx * dx + dy * dy + dz * dz).sqrt();
        println!(
            "{:.1}  ({:.3},{:.3},{:.3})  ({:.3},{:.3},{:.3})  ({:+.2},{:+.2},{:+.2})  {:.2}  {:.0}",
            jd, x_vsop, y_vsop, z_vsop, x_jpl, y_jpl, z_jpl, dx, dy, dz, dr_km, TOL_KM
        );
        let (l_v, b_v, r_v) = xyz_to_lbr(x_vsop, y_vsop, z_vsop);
        let (l_j, b_j, r_j) = xyz_to_lbr(x_jpl, y_jpl, z_jpl);
        let dl_rad = l_v - l_j;
        let dl_rad_wrap = if dl_rad > std::f64::consts::PI {
            dl_rad - 2.0 * std::f64::consts::PI
        } else if dl_rad < -std::f64::consts::PI {
            dl_rad + 2.0 * std::f64::consts::PI
        } else {
            dl_rad
        };
        let (dl_arcsec, db_arcsec, dr_km_lbr) = (
            dl_rad_wrap * RAD_TO_ARCSEC,
            (b_v - b_j) * RAD_TO_ARCSEC,
            r_v - r_j,
        );
        println!(
            "           LBR: L={:.6},{:.6} rad  B={:.6},{:.6} rad  R={:.1},{:.1} km  => dL={:+.4}\" dB={:+.4}\" dR={:+.2} km",
            l_v, l_j, b_v, b_j, r_v, r_j, dl_arcsec, db_arcsec, dr_km_lbr
        );
        assert!(
            dr_km <= TOL_KM,
            "JD {} VSOP87+patch=({:.3},{:.3},{:.3}) DE406=({:.3},{:.3},{:.3}) |dr|={:.2} km tol={}",
            jd, x_vsop, y_vsop, z_vsop, x_jpl, y_jpl, z_jpl, dr_km, TOL_KM
        );
    }
}

/// Rust DE406（本仓库 BSP 读取）vs Python/jplephem DE406：同一 BSP、同一 JD，地心日/月 ICRS (km, km/s) 直接对比，不经过 CSV。
#[test]
fn de406_rust_vs_python_de406() {
    if std::env::var("PYO3_PYTHON").is_err() {
        println!("de406_rust_vs_python_de406: skipped (PYO3_PYTHON not set)");
        return;
    }
    set_python_home_if_requested();
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
        println!("de406_rust_vs_python_de406: skipped (no DE406 BSP file at DE406_BSP or {})", crate::repo::paths::DE406_BSP_CANDIDATES[1]);
        return;
    }
    let jds: Vec<f64> = vec![2444239.5, 2451545.0, 2451545.5, 2455000.0, 2457397.5];

    let (py_rows, _err): (Vec<Vec<f64>>, String) = match pyo3::Python::with_gil(|py| -> pyo3::PyResult<(Vec<Vec<f64>>, String)> {
        #[allow(deprecated)]
        let mod_ = pyo3::types::PyModule::from_code_bound(py, PY_CODE_DE406_GEOCENTRIC, "de406_geocentric.py", "de406_geocentric")?;
        let func = mod_.getattr("run_geocentric")?;
        let tuple = func.call1((bsp_path.as_str(), jds.clone()))?;
        tuple.extract()
    }) {
        Ok((rows, _)) if !rows.is_empty() => (rows, String::new()),
        Ok((_, msg)) => {
            println!("de406_rust_vs_python_de406: skipped ({})", if msg.is_empty() { "no rows" } else { msg.trim() });
            return;
        }
        Err(e) => {
            println!("de406_rust_vs_python_de406: skipped (Python error: {})", e);
            return;
        }
    };

    let kernel = match De406Kernel::open(&bsp_path) {
        Ok(k) => k,
        Err(e) => {
            println!("de406_rust_vs_python_de406: skipped (Rust open BSP failed: {})", e);
            return;
        }
    };

    const TOL_POS_KM: f64 = 1e-6;  // 1 mm
    const TOL_VEL_KM_S: f64 = 1e-6;
    println!("JD(TDB)     Rust DE406 vs Python/jplephem: 太阳/月球 ICRS (km, km/s) 残差");
    for row in &py_rows {
        if row.len() < 13 {
            continue;
        }
        let jd = row[0];
        let (ps_km, vs_km_s) = match kernel.geocentric_sun(jd) {
            Ok(pv) => (pv.0, pv.1),
            Err(e) => {
                panic!("Rust geocentric_sun({}) failed: {}", jd, e);
            }
        };
        let (pm_km, vm_km_s) = match kernel.geocentric_moon(jd) {
            Ok(pv) => (pv.0, pv.1),
            Err(e) => {
                panic!("Rust geocentric_moon({}) failed: {}", jd, e);
            }
        };
        let rust_sun = [ps_km[0] / 1000.0, ps_km[1] / 1000.0, ps_km[2] / 1000.0];
        let rust_sun_v = [vs_km_s[0] / 1000.0, vs_km_s[1] / 1000.0, vs_km_s[2] / 1000.0];
        let rust_moon = [pm_km[0] / 1000.0, pm_km[1] / 1000.0, pm_km[2] / 1000.0];
        let rust_moon_v = [vm_km_s[0] / 1000.0, vm_km_s[1] / 1000.0, vm_km_s[2] / 1000.0];
        let py_sun = [row[1], row[2], row[3]];
        let py_sun_v = [row[4], row[5], row[6]];
        let py_moon = [row[7], row[8], row[9]];
        let py_moon_v = [row[10], row[11], row[12]];
        for i in 0..3 {
            let d = rust_sun[i] - py_sun[i];
            assert!(d.abs() <= TOL_POS_KM, "JD {} Sun pos[{}] Rust={} Py={} diff={}", jd, i, rust_sun[i], py_sun[i], d);
        }
        for i in 0..3 {
            let d = rust_sun_v[i] - py_sun_v[i];
            assert!(d.abs() <= TOL_VEL_KM_S, "JD {} Sun vel[{}] Rust={} Py={} diff={}", jd, i, rust_sun_v[i], py_sun_v[i], d);
        }
        for i in 0..3 {
            let d = rust_moon[i] - py_moon[i];
            assert!(d.abs() <= TOL_POS_KM, "JD {} Moon pos[{}] Rust={} Py={} diff={}", jd, i, rust_moon[i], py_moon[i], d);
        }
        for i in 0..3 {
            let d = rust_moon_v[i] - py_moon_v[i];
            assert!(d.abs() <= TOL_VEL_KM_S, "JD {} Moon vel[{}] Rust={} Py={} diff={}", jd, i, rust_moon_v[i], py_moon_v[i], d);
        }
        let (dxs, dys, dzs) = (rust_sun[0] - py_sun[0], rust_sun[1] - py_sun[1], rust_sun[2] - py_sun[2]);
        let (dxm, dym, dzm) = (rust_moon[0] - py_moon[0], rust_moon[1] - py_moon[1], rust_moon[2] - py_moon[2]);
        let dr_sun = (dxs * dxs + dys * dys + dzs * dzs).sqrt();
        let dr_moon = (dxm * dxm + dym * dym + dzm * dzm).sqrt();
        println!("  {:.1}   Sun |dr|={:.2e} km   Moon |dr|={:.2e} km", jd, dr_sun, dr_moon);
    }
}
