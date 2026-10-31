//! 恒星星表（天文域第 1 层独立基础层）。
//!
//! 数据：`data/stars/hipparcos_bright.csv`（V ≤ 6.5，8870 颗），由 `scripts/phenomena/hipparcos_bright.py` 从 vendor 原始星表生成：
//! 位置（ICRS，历元 J1991.25）、视差、自行取 Hipparcos 新归算 I/311（van Leeuwen 2007, A&A 474, 653）；
//! Johnson V、B−V 取原版主表 I/239（ESA 1997）。
//!
//! 空间运动：Hipparcos 无视向速度，按视向速度为 0 的直线运动推算（位置矢量 r₀ + vΔt 再归一），
//! 与 SOFA `iauPmsafe` 在 rv = 0 时同义；数千年外推时近距高速星（如天狼星）的视向速度项可达角秒至角分量级，
//! 用于肉眼可见性足够，精密星位另行处理。

use crate::math::real::{real, Real};

/// Hipparcos 星表历元 J1991.25（TT 儒略日）。
pub const HIPPARCOS_EPOCH_JD: f64 = 2448349.0625;

/// 星表中的一颗恒星。
#[derive(Clone, Copy, Debug)]
pub struct CatalogStar {
    pub hip: u32,
    /// ICRS 赤经、赤纬（度），历元 [`HIPPARCOS_EPOCH_JD`]
    pub ra_deg: Real,
    pub dec_deg: Real,
    pub parallax_mas: Real,
    /// μα* = μα cos δ（mas/年）
    pub pm_ra_mas_yr: Real,
    pub pm_dec_mas_yr: Real,
    pub vmag: Real,
    pub bv: Option<Real>,
}

impl CatalogStar {
    /// 历元 `jd_tt` 的 ICRS 质心方向（单位向量）。
    pub fn icrs_direction(&self, jd_tt: Real) -> [Real; 3] {
        let d2r = real(core::f64::consts::PI / 180.0);
        let (a, d) = (self.ra_deg * d2r, self.dec_deg * d2r);
        let (sa, ca, sd, cd) = (a.sin(), a.cos(), d.sin(), d.cos());
        let p = [cd * ca, cd * sa, sd];
        let e_a = [-sa, ca, real(0.0)];
        let e_d = [-sd * ca, -sd * sa, cd];
        let mas = real(core::f64::consts::PI / (180.0 * 3600.0 * 1000.0));
        let years = (jd_tt - real(HIPPARCOS_EPOCH_JD)) / real(365.25);
        let (ua, ud) = (self.pm_ra_mas_yr * mas * years, self.pm_dec_mas_yr * mas * years);
        let r = [p[0] + ua * e_a[0] + ud * e_d[0], p[1] + ua * e_a[1] + ud * e_d[1], p[2] + ua * e_a[2] + ud * e_d[2]];
        let n = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        [r[0] / n, r[1] / n, r[2] / n]
    }
}

/// 星表。
#[derive(Clone, Debug, Default)]
pub struct StarCatalog {
    pub stars: Vec<CatalogStar>,
}

impl StarCatalog {
    /// 解析 `hipparcos_bright.csv`（`#` 开头为注释，首个非注释行为表头）。
    pub fn parse_hipparcos_bright(lines: &[String]) -> Result<Self, String> {
        let mut stars = vec![];
        for line in lines.iter().filter(|l| !l.starts_with('#')).skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            if f.len() < 8 {
                return Err(format!("列数不足 8：{line}"));
            }
            let num = |s: &str| s.trim().parse::<f64>().map(real).map_err(|e| format!("{e}：{line}"));
            stars.push(CatalogStar {
                hip: f[0].trim().parse().map_err(|e| format!("{e}：{line}"))?,
                ra_deg: num(f[1])?,
                dec_deg: num(f[2])?,
                parallax_mas: num(f[3])?,
                pm_ra_mas_yr: num(f[4])?,
                pm_dec_mas_yr: num(f[5])?,
                vmag: num(f[6])?,
                bv: if f[7].trim().is_empty() { None } else { Some(num(f[7])?) },
            });
        }
        Ok(Self { stars })
    }

    pub fn by_hip(&self, hip: u32) -> Option<&CatalogStar> {
        self.stars.binary_search_by_key(&hip, |s| s.hip).ok().map(|i| &self.stars[i])
    }
}

/// 读取仓库内亮星表（[`crate::repo::paths::HIPPARCOS_BRIGHT`]）。
pub fn load_hipparcos_bright_from_repo() -> Result<StarCatalog, String> {
    let lines = crate::repo::read_lines(crate::repo::paths::HIPPARCOS_BRIGHT).map_err(|e| format!("{e:?}"))?;
    StarCatalog::parse_hipparcos_bright(&lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::RealOps;

    #[test]
    fn loads_catalog_and_sirius() {
        let cat = load_hipparcos_bright_from_repo().unwrap();
        assert!(cat.stars.len() >= 8870, "{}", cat.stars.len());
        let s = cat.by_hip(32349).unwrap();
        assert!((s.vmag.as_f64() + 1.44).abs() < 1e-9);
        // 天狼星自行约 1.34″/年：J1991.25 → J2000 方向变化约 11.7″
        let a = s.icrs_direction(real(HIPPARCOS_EPOCH_JD));
        let b = s.icrs_direction(real(2451545.0));
        let ang = ((a[1] * b[2] - a[2] * b[1]).powi(2) + (a[2] * b[0] - a[0] * b[2]).powi(2) + (a[0] * b[1] - a[1] * b[0]).powi(2)).sqrt();
        let arcsec = ang.as_f64() * 206264.806;
        assert!((arcsec - 1.3394 * 8.75).abs() < 0.01, "{arcsec}");
    }
}
