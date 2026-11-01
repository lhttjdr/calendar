//! 由高程数据算当地地平线：每个方位上，地形最高遮挡的视高度。依据、数据来源与检验见 doc/astronomy/sky-view-3d.md。
//!
//! 沿大圆向外取样，距离按比例增大（近处密，远处疏）。第 d 处地面的视高度：
//!   tan h = (H(d) − H₀)/d − (1 − k)·d/(2R)
//! 第二项是地球曲率减去大气折射的抬升，k 是地面光线折射系数（测量常用约 0.13），R 取地球平均半径。
//! 这里得到的是**视**高度，直接与天体的视高度比较。

use crate::math::real::{real, Real, RealOps};

/// 地面光线折射系数（大地测量常用值，随气象变化，贴地逆温时可大得多）。
pub const TERRESTRIAL_REFRACTION_K: f64 = 0.13;
const EARTH_RADIUS_M: f64 = 6_371_000.0;
/// 取样从此距离开始。高程数据测的是地表，含现代建筑与堤坝（北京 300 m 处可达 1.4°），近处不可用于古代。
const MIN_DISTANCE_M: f64 = 2000.0;
const DISTANCE_RATIO: f64 = 1.02;

/// Web 墨卡托瓦片拼成的高程网格（z 级，左上角瓦片 (x0, y0)，按行存，单位米）。
pub struct DemMosaic<'a> {
    pub z: u32,
    pub x0: f64,
    pub y0: f64,
    /// 每行、每列的像素数
    pub width: usize,
    pub height: usize,
    /// 每瓦片的边长（像素）
    pub tile_size: f64,
    pub data: &'a [f32],
}

impl DemMosaic<'_> {
    /// 双线性取样；出网格为 None。
    pub fn sample(&self, lon_deg: Real, lat_deg: Real) -> Option<Real> {
        let n = real(2f64.powi(self.z as i32));
        let lat = lat_deg * Real::pi() / real(180.0);
        let tx = (lon_deg + real(180.0)) / real(360.0) * n;
        let ty = (real(1.0) - (lat.tan() + real(1.0) / lat.cos()).ln() / Real::pi()) / real(2.0) * n;
        // 像素中心在 +0.5 处
        let px = ((tx - real(self.x0)) * real(self.tile_size) - real(0.5)).as_f64();
        let py = ((ty - real(self.y0)) * real(self.tile_size) - real(0.5)).as_f64();
        if px < 0.0 || py < 0.0 || px >= (self.width - 1) as f64 || py >= (self.height - 1) as f64 {
            return None;
        }
        let (i, j) = (px.floor() as usize, py.floor() as usize);
        let (fx, fy) = (px - i as f64, py - j as f64);
        let at = |a: usize, b: usize| self.data[b * self.width + a] as f64;
        let top = at(i, j) * (1.0 - fx) + at(i + 1, j) * fx;
        let bottom = at(i, j + 1) * (1.0 - fx) + at(i + 1, j + 1) * fx;
        Some(real(top * (1.0 - fy) + bottom * fy))
    }
}

/// 地平线计算结果。
#[derive(Clone, Debug)]
pub struct TerrainHorizon {
    /// 观测点的地面海拔 [m]
    pub ground_m: Real,
    /// 方位 360°·i/n（自北经东）处地平线的视高度 [°]
    pub altitude_deg: Vec<Real>,
}

/// 求地平线。`ground_m` 为观测点地面海拔，`elevation(经度°, 纬度°)` 给沿途海拔（米，宜用约 250 m 分辨率，滤掉现代地物），无数据时给 None，该点跳过。
/// `eye_m` 为眼高（地面以上），`azimuths` 为方位数，`max_distance_m` 为最远取样距离。
pub fn terrain_horizon(lon_deg: Real, lat_deg: Real, ground_m: Real, eye_m: Real, azimuths: usize, max_distance_m: f64, elevation: impl Fn(Real, Real) -> Option<Real>) -> TerrainHorizon {
    let ground = ground_m;
    let eye = ground + eye_m;
    let r = real(EARTH_RADIUS_M);
    let d2r = Real::pi() / real(180.0);
    let (p0, l0) = (lat_deg * d2r, lon_deg * d2r);
    let curve = (real(1.0) - real(TERRESTRIAL_REFRACTION_K)) / (real(2.0) * r);
    let mut distances = vec![];
    let mut d = MIN_DISTANCE_M;
    while d < max_distance_m {
        distances.push(d);
        d = (d * DISTANCE_RATIO).max(d + 30.0);
    }
    let altitude_deg = (0..azimuths)
        .map(|i| {
            let az = real(i as f64 / azimuths as f64 * 360.0) * d2r;
            // 地平线下限：比天文地平低多少由眼高决定，无地形遮挡时就是视线切地面处
            let mut best = (-(real(2.0) * eye_m.max(real(0.0)) * (real(1.0) - real(TERRESTRIAL_REFRACTION_K)) / r).sqrt()).atan2(real(1.0));
            for &dist in &distances {
                let a = real(dist) / r;
                let lat = (p0.sin() * a.cos() + p0.cos() * a.sin() * az.cos()).asin();
                let lon = l0 + (az.sin() * a.sin() * p0.cos()).atan2(a.cos() - p0.sin() * lat.sin());
                if let Some(h) = elevation(lon / d2r, lat / d2r) {
                    let alt = ((h - eye) / real(dist) - curve * real(dist)).atan2(real(1.0));
                    if alt > best {
                        best = alt;
                    }
                }
            }
            best / d2r
        })
        .collect();
    TerrainHorizon { ground_m: ground, altitude_deg }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(f: impl Fn(Real, Real) -> Option<Real>, eye: f64) -> TerrainHorizon {
        let ground = f(real(108.0), real(34.0)).unwrap();
        terrain_horizon(real(108.0), real(34.0), ground, real(eye), 72, 200_000.0, f)
    }

    /// 平地：地平线是视线切地面处，随眼高按 √(2h(1−k)/R) 下降。
    #[test]
    fn flat_ground_gives_tangent_dip() {
        let h = run(|_, _| Some(real(100.0)), 100.0);
        let want = -(2.0 * 100.0 * (1.0 - TERRESTRIAL_REFRACTION_K) / EARTH_RADIUS_M).sqrt().to_degrees();
        for a in &h.altitude_deg {
            assert!((a.as_f64() - want).abs() < 0.02, "{} vs {want}", a.as_f64());
        }
        assert_eq!(h.ground_m.as_f64(), 100.0);
    }

    /// 正南 50 km 处一座 2000 m 的山：高度角等于几何解，别的方位不受影响。
    #[test]
    fn single_mountain_south() {
        let km = 111.19_f64;
        let mountain = move |lon: Real, lat: Real| {
            let (x, y) = ((lon.as_f64() - 108.0) * km * 34f64.to_radians().cos(), (lat.as_f64() - 34.0) * km);
            let dist = ((x * x) + (y + 50.0) * (y + 50.0)).sqrt();
            Some(real(if dist < 3.0 { 2000.0 } else { 0.0 }))
        };
        let h = run(mountain, 0.0);
        let d = 47_000.0;
        let want = (2000.0 / d - (1.0 - TERRESTRIAL_REFRACTION_K) * d / (2.0 * EARTH_RADIUS_M)).atan().to_degrees();
        let south = h.altitude_deg[36].as_f64();
        assert!((south - want).abs() < 0.25, "南方 {south} vs {want}");
        assert!(h.altitude_deg[0].as_f64().abs() < 0.05, "北方 {}", h.altitude_deg[0].as_f64());
        assert!(h.altitude_deg[18].as_f64().abs() < 0.05, "东方 {}", h.altitude_deg[18].as_f64());
    }

    /// 地球曲率：150 km 外 3000 m 的山被曲率压低，1000 m 的已看不见；300 km 外在取样上限外不算。
    #[test]
    fn curvature_lowers_distant_terrain() {
        let km = 111.19_f64;
        let wall = move |dist_km: f64, h_m: f64| move |_: Real, lat: Real| {
            let y = (lat.as_f64() - 34.0) * km;
            Some(real(if y > dist_km - 2.0 && y < dist_km + 2.0 { h_m } else { 0.0 }))
        };
        let near = run(wall(150.0, 3000.0), 0.0).altitude_deg[0].as_f64();
        let want = (3000.0 / 148_000.0 - (1.0 - TERRESTRIAL_REFRACTION_K) * 148_000.0 / (2.0 * EARTH_RADIUS_M)).atan().to_degrees();
        assert!((near - want).abs() < 0.05, "{near} vs {want}");
        // 150 km 外 1000 m 的山已沉到地平线下，不起遮挡
        assert!(run(wall(150.0, 1000.0), 0.0).altitude_deg[0].as_f64().abs() < 1e-6);
        assert!(run(wall(300.0, 8000.0), 0.0).altitude_deg[0].as_f64().abs() < 1e-6, "取样上限外");
    }

    /// 瓦片取样：位置与值对得上（z=9 长安所在瓦片左上角）。
    #[test]
    fn mosaic_samples_tile_position() {
        let w = 256 * 2;
        let data: Vec<f32> = (0..w * w).map(|i| (i % w) as f32).collect();
        let m = DemMosaic { z: 9, x0: 410.0, y0: 203.0, width: w, height: w, tile_size: 256.0, data: &data };
        // 经度 108.94° 在 z=9 的 x = (108.94+180)/360*512 = 410.93 瓦片，像素 0.93*256 − 0.5
        let v = m.sample(real(108.94), real(34.27)).unwrap().as_f64();
        let want = (288.94 / 360.0 * 512.0 - 410.0) * 256.0 - 0.5;
        assert!((v - want).abs() < 1e-6, "{v} vs {want}");
        assert!(m.sample(real(0.0), real(34.0)).is_none());
    }
}
