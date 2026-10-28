//! 测试专用：测试用数据文件路径与解析辅助。
//!
//! 本模块仅随 `cargo test` 编译，**不会编入库或 wasm 等最终产物**。
//! 测试中需要仓库根、BSP、参考表等路径时，可统一通过此处获取，避免在各测试里重复拼路径。

#![cfg(test)]
#[allow(dead_code)] // 供各测试按需选用，未全部迁移前会有未使用

use std::path::Path;

/// 测试用仓库根目录（与 [crate::repo::repo_root] 一致，仅限 test 下使用）。
#[cfg(not(target_arch = "wasm32"))]
pub fn base() -> std::path::PathBuf {
    crate::repo::repo_root()
}

/// 解析 DE406 BSP 路径：环境变量 `DE406_BSP`（若为文件）→ 候选路径 [0] → [1] → 否则 `vendor/jpl` 目录路径（用于 skip 提示）。
/// 返回的路径不保证存在，调用方需自行 `Path::is_file()` 判断。
#[cfg(not(target_arch = "wasm32"))]
pub fn de406_bsp_path() -> String {
    let base = base();
    std::env::var("DE406_BSP")
        .ok()
        .filter(|p| Path::new(p).is_file())
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
        .unwrap_or_else(|| base.join(crate::repo::paths::JPL_DATA_DIR).to_string_lossy().into_owned())
}

/// DE406 数据目录或 BSP 所在目录的路径（用于 Python/jplephem 等传目录的场景）。
#[cfg(not(target_arch = "wasm32"))]
pub fn de406_ephem_path() -> String {
    std::env::var("DE406_PATH")
        .unwrap_or_else(|_| base().join(crate::repo::paths::JPL_DATA_DIR).to_string_lossy().into_owned())
}

/// 节气朔望参考表完整路径（《月相和二十四节气的计算》§7.4 TDBtimes.txt）。
#[cfg(not(target_arch = "wasm32"))]
pub fn solar_terms_reference_path() -> std::path::PathBuf {
    base().join(crate::repo::paths::SOLAR_TERMS_REFERENCE)
}

/// ELPMPP02 vs JPL DE406 样本 CSV 完整路径。
#[cfg(not(target_arch = "wasm32"))]
pub fn jpl_elp_vs_jpl_samples_csv_path() -> std::path::PathBuf {
    base().join(crate::repo::paths::JPL_ELP_VS_JPL_SAMPLES_CSV)
}

/// 若 BSP 路径指向的文件不存在，返回 `None`；否则返回 `Some(path)`。
#[cfg(not(target_arch = "wasm32"))]
pub fn de406_bsp_path_if_exists() -> Option<String> {
    let p = de406_bsp_path();
    Path::new(&p).is_file().then_some(p)
}

/// 本次构建的 `Real` 后端，写入校验表说明行。
#[cfg(feature = "real-f64")]
pub const NUMERIC_BACKEND: &str = "f64（real-f64 构建）";
#[cfg(not(feature = "real-f64"))]
pub const NUMERIC_BACKEND: &str = "double-double（twofloat，库缺省构建）";

/// 逐条校验结果表：设置环境变量 `PHENOMENA_CHECKS=1` 时写入 `data/phenomena/checks/<name>.csv`（或 `PHENOMENA_CHECKS_DIR`），否则不写（平时跑测试不改仓库文件）。
/// 由 `scripts/dev/phenomena_checks.sh` 统一重算，`scripts/phenomena/checks_to_md.py` 据此生成 doc/astronomy/phenomena-checks.md。
/// `meta` 为若干行说明（写作 `# ` 注释），`header` 为列名，`rows` 每行各列已格式化（不得含逗号）。
#[cfg(not(target_arch = "wasm32"))]
pub struct CheckTable {
    name: &'static str,
    meta: Vec<String>,
    header: Vec<&'static str>,
    rows: Vec<Vec<String>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CheckTable {
    pub fn new(name: &'static str, header: &[&'static str]) -> Self {
        Self { name, meta: vec![], header: header.to_vec(), rows: vec![] }
    }

    pub fn meta(&mut self, line: impl Into<String>) -> &mut Self {
        self.meta.push(line.into());
        self
    }

    pub fn row(&mut self, cells: Vec<String>) {
        assert_eq!(cells.len(), self.header.len(), "{}: 列数不符", self.name);
        self.rows.push(cells.into_iter().map(|c| c.replace(',', "，").replace('\n', " ")).collect());
    }

    pub fn write(&self) {
        if std::env::var("PHENOMENA_CHECKS").as_deref() != Ok("1") {
            return;
        }
        // 输出目录可由 PHENOMENA_CHECKS_DIR 指定（两种数值后端对比时各写一处）
        let dir = std::env::var("PHENOMENA_CHECKS_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| base().join("data/phenomena/checks"));
        std::fs::create_dir_all(&dir).unwrap();
        let mut s = String::new();
        for m in &self.meta {
            s += &format!("# {m}\n");
        }
        s += &format!("# 数值后端：{}\n", NUMERIC_BACKEND);
        s += &self.header.join(",");
        s += "\n";
        for r in &self.rows {
            s += &r.join(",");
            s += "\n";
        }
        std::fs::write(dir.join(format!("{}.csv", self.name)), s).unwrap();
    }
}

/// 儒略日 → 「年-月-日 时:分」（1582-10-15 前儒略历，之后格里高利历；天文纪年）。仅用于校验表显示。
pub fn jd_to_calendar(jd: f64) -> String {
    let z = (jd + 0.5).floor();
    let f = jd + 0.5 - z;
    let a = if z < 2299161.0 {
        z
    } else {
        let al = ((z - 1867216.25) / 36524.25).floor();
        z + 1.0 + al - (al / 4.0).floor()
    };
    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();
    let day = b - d - (30.6001 * e).floor();
    let m = if e < 14.0 { e - 1.0 } else { e - 13.0 };
    let y = if m > 2.0 { c - 4716.0 } else { c - 4715.0 };
    let mins = (f * 1440.0).round() as i64;
    let (mut day, mut mins) = (day as i64, mins);
    if mins >= 1440 {
        mins -= 1440;
        day += 1; // 显示用，月末进位不处理（极少见）
    }
    format!("{}-{:02}-{:02} {:02}:{:02}", y as i64, m as i64, day, mins / 60, mins % 60)
}

/// 并行映射（保序）：按 `std::thread::available_parallelism` 开线程，逐项领取（各项耗时差异大时自动均衡），结果按原序返回。
/// 校验类测试的逐条推算彼此独立，用它并行；结果与串行逐位相同（各项计算互不共享可变状态）。
/// 线程数可由环境变量 `PHENOMENA_THREADS` 指定（1 即串行）。
pub fn par_map<T: Sync, R: Send>(items: &[T], f: impl Fn(usize, &T) -> R + Sync) -> Vec<R> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let threads = std::env::var("PHENOMENA_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1))
        .clamp(1, items.len().max(1));
    let next = AtomicUsize::new(0);
    let mut parts: Vec<Vec<(usize, R)>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                s.spawn(|| {
                    let mut out = vec![];
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        if i >= items.len() {
                            break out;
                        }
                        out.push((i, f(i, &items[i])));
                    }
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("par_map 工作线程 panic")).collect()
    });
    let mut all: Vec<(usize, R)> = parts.drain(..).flatten().collect();
    all.sort_by_key(|(i, _)| *i);
    all.into_iter().map(|(_, r)| r).collect()
}
