//! ELPMPP02/VSOP87 vs JPL DE406：Rust 测试内实时调用 Python/jplephem，内存传递，无需 CSV。
//! 运行：PYO3_PYTHON=.venv/bin/python cargo test elpmpp02_vs_jpl_de406_python
//!      PYO3_PYTHON=... cargo test vsop87_vs_jpl_de406_python

#![cfg(all(test, not(target_arch = "wasm32"), feature = "python-test"))]

use super::*;
use crate::astronomy::apparent::sun_position_icrs;
use crate::astronomy::ephemeris::{load_earth_vsop87, Vsop87};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};
use crate::platform::DataLoaderNative;
use pyo3::types::PyAnyMethods;

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

/// DE406 地心太阳在 J2000 平黄道 (x,y,z) km；与 PY_CODE 同路径与旋转。
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
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ephem_path: String = std::env::var("DE406_PATH")
        .unwrap_or_else(|_| base.join("data/jpl").to_string_lossy().into_owned());
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

    let loader = DataLoaderNative::new(&base);
    let data = match load_all(&loader, "data/elpmpp02", Elpmpp02Correction::DE406) {
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
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ephem_path: String = std::env::var("DE406_PATH")
        .unwrap_or_else(|_| base.join("data/jpl").to_string_lossy().into_owned());
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

    let loader = DataLoaderNative::new(&base);
    let vsop: Vsop87 = match load_earth_vsop87(&loader, "data/vsop87/VSOP87B.ear") {
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
