//! 世界磁场模型 WMM2025（NOAA/NCEI 与英国地质调查局；美国政府作品，公有领域）：任一地点、日期的地磁要素。
//!
//! 用途：网页罗盘把手机指南针所给的磁北方位换算为真北（真方位 = 磁方位 + 磁偏角 D，东偏为正）。
//! 算法依 *The US/UK World Magnetic Model for 2025–2030: Technical Report*（NOAA, 2024；下称技术报告）第 1.2 节：
//! 系数按长期变化外推到所求时刻；大地坐标 → 地心球坐标（WGS84）；施密特半归一化缔合勒让德函数球谐展开（n ≤ 12）
//! 得地心球坐标下的 X′ Y′ Z′；转到大地（椭球）坐标得 X Y Z；再求 H F I D。
//! 系数：`vendor/geomag/wmm2025/WMM.COF`（原样）；[`Wmm::embedded`] 编译期嵌入，供 wasm 直接用。
//! 有效期 2025.0—2030.0（技术报告：模型在此期间之外不保证），期外 [`Wmm::field`] 返回 `None`。

use crate::math::real::{real, real_const, Real, RealOps};

/// 球谐展开最高阶
pub const N_MAX: usize = 12;
/// 参考球半径（km），技术报告表 1
const A_REF_KM: f64 = 6371.2;
/// WGS84 长半轴（km）与扁率倒数
const WGS84_A_KM: f64 = 6378.137;
const WGS84_INV_F: f64 = 298.257223563;

/// 嵌入的系数文件（WMM2025，2024-11-13 版）
pub const WMM2025_COF: &str = include_str!("../../../../vendor/geomag/wmm2025/WMM.COF");

/// 一套高斯系数：g、h 与其年变率（nT、nT/年），下标 [n][m]
#[derive(Clone, Debug)]
pub struct Wmm {
    /// 模型历元（年，如 2025.0）
    pub epoch: f64,
    /// 模型名（如 WMM-2025）
    pub name: String,
    g: [[f64; N_MAX + 1]; N_MAX + 1],
    h: [[f64; N_MAX + 1]; N_MAX + 1],
    gd: [[f64; N_MAX + 1]; N_MAX + 1],
    hd: [[f64; N_MAX + 1]; N_MAX + 1],
}

/// 地磁要素（大地坐标：X 北、Y 东、Z 向下）
#[derive(Clone, Copy, Debug)]
pub struct GeomagField {
    /// 北向、东向、垂直向下分量（nT）
    pub x: Real,
    pub y: Real,
    pub z: Real,
    /// 水平强度、总强度（nT）
    pub h: Real,
    pub f: Real,
    /// 磁倾角（度，向下为正）
    pub incl_deg: Real,
    /// 磁偏角（度，东偏为正）
    pub decl_deg: Real,
}

impl Wmm {
    /// 解析 WMM.COF：首行「历元 模型名 日期」，其后每行「n m g h ġ ḣ」，以 9999… 行结束
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text.lines().filter(|l| !l.trim().is_empty());
        let head = lines.next().ok_or("WMM.COF 为空")?;
        let mut hw = head.split_whitespace();
        let epoch: f64 = hw.next().ok_or("缺历元")?.parse().map_err(|e| format!("历元：{e}"))?;
        let name = hw.next().unwrap_or("").to_string();
        let z = [[0.0; N_MAX + 1]; N_MAX + 1];
        let mut w = Wmm { epoch, name, g: z, h: z, gd: z, hd: z };
        for l in lines {
            if l.trim_start().starts_with("9999") {
                break;
            }
            let v: Vec<&str> = l.split_whitespace().collect();
            if v.len() < 6 {
                return Err(format!("WMM.COF 行格式不符：{l}"));
            }
            let n: usize = v[0].parse().map_err(|e| format!("{e}：{l}"))?;
            let m: usize = v[1].parse().map_err(|e| format!("{e}：{l}"))?;
            if n > N_MAX || m > n {
                return Err(format!("阶次越界：{l}"));
            }
            let p = |s: &str| s.parse::<f64>().map_err(|e| format!("{e}：{l}"));
            w.g[n][m] = p(v[2])?;
            w.h[n][m] = p(v[3])?;
            w.gd[n][m] = p(v[4])?;
            w.hd[n][m] = p(v[5])?;
        }
        Ok(w)
    }

    /// 编译期嵌入的 WMM2025
    pub fn embedded() -> Self {
        Self::parse(WMM2025_COF).expect("嵌入的 WMM.COF 应可解析")
    }

    /// 从仓库读 `vendor/geomag/wmm2025/WMM.COF`
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_repo() -> Result<Self, String> {
        let lines = crate::repo::read_lines(crate::repo::paths::WMM2025_COF).map_err(|e| format!("{e:?}"))?;
        Self::parse(&lines.join("\n"))
    }

    /// 有效期（年）：历元起五年
    pub fn valid_range(&self) -> (f64, f64) {
        (self.epoch, self.epoch + 5.0)
    }

    /// 大地纬度、经度（度）、椭球高（km）、十进制年处的地磁要素；有效期外返回 `None`。
    pub fn field(&self, lat_deg: f64, lon_deg: f64, alt_km: f64, decimal_year: f64) -> Option<GeomagField> {
        let (t0, t1) = self.valid_range();
        if !(decimal_year >= t0 && decimal_year <= t1) || !(-90.0..=90.0).contains(&lat_deg) {
            return None;
        }
        Some(self.field_unchecked(lat_deg, lon_deg, alt_km, decimal_year))
    }

    /// 同 [`Wmm::field`]，不检查有效期（用于研究外推误差；结果不受模型保证）
    pub fn field_unchecked(&self, lat_deg: f64, lon_deg: f64, alt_km: f64, decimal_year: f64) -> GeomagField {
        let deg = Real::pi() / real_const(180.0);
        let dt = real(decimal_year - self.epoch);
        let phi = real(lat_deg) * deg;
        let lam = real(lon_deg) * deg;
        let hgt = real(alt_km);
        // 大地 → 地心球坐标（技术报告式 7、8）
        let a = real(WGS84_A_KM);
        let f = real_const(1.0) / real(WGS84_INV_F);
        let e2 = f * (real_const(2.0) - f);
        let (sp, cp) = (phi.sin(), phi.cos());
        let rc = a / (real_const(1.0) - e2 * sp * sp).sqrt();
        let p = (rc + hgt) * cp;
        let zz = (rc * (real_const(1.0) - e2) + hgt) * sp;
        let r = (p * p + zz * zz).sqrt();
        let phi_c = (zz / r).asin();
        // 施密特半归一化缔合勒让德函数 P_n^m(sin φ′) 及其对余纬 θ 的导数（φ′ = 90° − θ）
        let x = phi_c.sin(); // cos θ
        let s = phi_c.cos(); // sin θ
        let zero = Real::zero();
        let mut pnm = [[zero; N_MAX + 1]; N_MAX + 1];
        let mut dp = [[zero; N_MAX + 1]; N_MAX + 1];
        pnm[0][0] = Real::one();
        pnm[1][0] = x;
        dp[1][0] = -s;
        pnm[1][1] = s;
        dp[1][1] = x;
        for n in 2..=N_MAX {
            let k = real((2 * n - 1) as f64 / (2 * n) as f64).sqrt();
            pnm[n][n] = k * s * pnm[n - 1][n - 1];
            dp[n][n] = k * (x * pnm[n - 1][n - 1] + s * dp[n - 1][n - 1]);
            for m in 0..n {
                let c1 = real((2 * n - 1) as f64);
                let den = real((n * n - m * m) as f64).sqrt();
                let (p2, d2) = if n >= 2 && m <= n - 2 {
                    let c2 = real((((n - 1) * (n - 1)) - m * m) as f64).sqrt();
                    (c2 * pnm[n - 2][m], c2 * dp[n - 2][m])
                } else {
                    (zero, zero)
                };
                pnm[n][m] = (c1 * x * pnm[n - 1][m] - p2) / den;
                dp[n][m] = (c1 * (x * dp[n - 1][m] - s * pnm[n - 1][m]) - d2) / den;
            }
        }
        // 球谐求和（技术报告式 10—12）
        let ar = real(A_REF_KM) / r;
        let (mut xs, mut ys, mut zs) = (zero, zero, zero);
        let mut arn = ar * ar; // (a/r)^(n+2)，n 从 1 起
        for n in 1..=N_MAX {
            arn = arn * ar;
            let (mut sx, mut sy, mut sz) = (zero, zero, zero);
            for m in 0..=n {
                let g = real(self.g[n][m]) + dt * real(self.gd[n][m]);
                let hh = real(self.h[n][m]) + dt * real(self.hd[n][m]);
                let ml = real(m as f64) * lam;
                let (cm, sm) = (ml.cos(), ml.sin());
                let gh = g * cm + hh * sm;
                // X′ = −Σ (a/r)^(n+2) Σ (g cos + h sin) dP/dφ′ ；dP/dφ′ = −dP/dθ
                sx = sx + gh * dp[n][m];
                sy = sy + real(m as f64) * (g * sm - hh * cm) * pnm[n][m];
                sz = sz + gh * pnm[n][m];
            }
            xs = xs + arn * sx;
            ys = ys + arn * sy;
            zs = zs - real((n + 1) as f64) * arn * sz;
        }
        ys = ys / s;
        // 地心球坐标 → 大地坐标（技术报告式 17）
        let d = phi_c - phi;
        let (cd, sd) = (d.cos(), d.sin());
        let bx = xs * cd - zs * sd;
        let by = ys;
        let bz = xs * sd + zs * cd;
        let h = (bx * bx + by * by).sqrt();
        let ff = (h * h + bz * bz).sqrt();
        GeomagField {
            x: bx,
            y: by,
            z: bz,
            h,
            f: ff,
            incl_deg: bz.atan2(h) / deg,
            decl_deg: by.atan2(bx) / deg,
        }
    }
}

/// 公历日期 → 十进制年（WMM 惯例：年 + (年内日序 − 1 + 日内小数) / 年长日数）
pub fn decimal_year(year: i32, month: u32, day: u32, hour_frac: f64) -> f64 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let cum = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut doy = cum[(month as usize).clamp(1, 12) - 1] + day as i32;
    if leap && month > 2 {
        doy += 1;
    }
    let len = if leap { 366.0 } else { 365.0 };
    year as f64 + ((doy - 1) as f64 + hour_frac / 24.0) / len
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(path: &str) -> Vec<Vec<f64>> {
        let text = std::fs::read_to_string(crate::repo::repo_root().join(path)).unwrap();
        text.lines()
            .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
            .map(|l| l.split_whitespace().map(|x| x.parse::<f64>().unwrap_or(f64::NAN)).collect())
            .collect()
    }

    #[test]
    fn embedded_matches_repo_file() {
        let a = Wmm::embedded();
        let b = Wmm::from_repo().unwrap();
        assert_eq!(a.epoch, 2025.0);
        assert_eq!(a.name, "WMM-2025");
        assert_eq!(a.g, b.g);
        assert_eq!(a.hd, b.hd);
        assert_eq!(a.g[1][0], -29351.8);
        assert_eq!(a.h[1][1], 4545.4);
    }

    /// 官方 100 点全精度测试值（WMM2025_TestValues.txt，随系数包发布）：年、高(km)、纬、经、D、I、H、X、Y、Z、F
    #[test]
    fn official_test_values_full_precision() {
        let w = Wmm::embedded();
        let t = rows("vendor/geomag/wmm2025/WMM2025_TestValues.txt");
        assert_eq!(t.len(), 100);
        let (mut md, mut mi, mut mf) = (0f64, 0f64, 0f64);
        for r in &t {
            let f = w.field(r[2], r[3], r[1], r[0]).unwrap();
            let dd = (f.decl_deg.as_f64() - r[4]).abs();
            let di = (f.incl_deg.as_f64() - r[5]).abs();
            // D、I 在文件中印到 0.01°
            assert!(dd <= 0.006, "D {r:?} 得 {}", f.decl_deg.as_f64());
            assert!(di <= 0.006, "I {r:?} 得 {}", f.incl_deg.as_f64());
            for (got, want) in [(f.h, r[6]), (f.x, r[7]), (f.y, r[8]), (f.z, r[9]), (f.f, r[10])] {
                assert!((got.as_f64() - want).abs() < 1e-3, "{r:?}：{} 对 {want}", got.as_f64());
                mf = mf.max((got.as_f64() - want).abs());
            }
            md = md.max(dd);
            mi = mi.max(di);
        }
        println!("WMM2025 官方 100 点：|ΔD|≤{md:.4}°，|ΔI|≤{mi:.4}°，分量最大差 {mf:.2e} nT");
    }

    /// 技术报告所载 12 点测试值（WMM2025_TEST_VALUES.txt，印到 0.1 nT、0.01°）
    #[test]
    fn official_test_values_report() {
        let w = Wmm::embedded();
        let t = rows("vendor/geomag/wmm2025/WMM2025_TEST_VALUES.txt");
        assert_eq!(t.len(), 12);
        for r in &t {
            // 年、高、纬、经、X、Y、Z、H、F、I、D
            let f = w.field(r[2], r[3], r[1], r[0]).unwrap();
            for (got, want) in [(f.x, r[4]), (f.y, r[5]), (f.z, r[6]), (f.h, r[7]), (f.f, r[8])] {
                assert!((got.as_f64() - want).abs() <= 0.051, "{r:?}：{} 对 {want}", got.as_f64());
            }
            assert!((f.incl_deg.as_f64() - r[9]).abs() <= 0.0051, "I {r:?}");
            assert!((f.decl_deg.as_f64() - r[10]).abs() <= 0.0051, "D {r:?}");
        }
    }

    #[test]
    fn validity_window() {
        let w = Wmm::embedded();
        assert!(w.field(39.9, 116.4, 0.0, 2024.99).is_none());
        assert!(w.field(39.9, 116.4, 0.0, 2030.01).is_none());
        assert!(w.field(39.9, 116.4, 0.0, 2026.85).is_some());
        // 北京 2026 年末磁偏角约西偏 7°—8°（量级检查）
        let d = w.field(39.9, 116.4, 0.05, decimal_year(2026, 11, 8, 12.0)).unwrap().decl_deg.as_f64();
        assert!((-9.0..-6.0).contains(&d), "北京 D = {d}");
        assert!((decimal_year(2025, 1, 1, 0.0) - 2025.0).abs() < 1e-12);
        assert!((decimal_year(2025, 7, 2, 12.0) - 2025.5).abs() < 1e-9);
    }
}
