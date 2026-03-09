//! 将 ELP-MPP02 六个文本文件转为二进制 .bin，供前端零解析加载。
//! 运行（在 rust 目录）：cargo run -p lunar-core --example elpmpp02_to_bin --no-default-features --features twofloat -- ../data/elpmpp02
//! 会在同目录下生成 ELP_MAIN.S1.bin, ELP_MAIN.S2.bin, ELP_MAIN.S3.bin, ELP_PERT.S1.bin, ELP_PERT.S2.bin, ELP_PERT.S3.bin。

use lunar_core::astronomy::ephemeris::elpmpp02::{load_all, parse::terms_to_binary};
use lunar_core::astronomy::ephemeris::Elpmpp02Correction;
use lunar_core::platform::DataLoaderNative;
use std::path::Path;

fn main() {
    let base = std::env::args()
        .nth(1)
        .expect("用法: elpmpp02_to_bin <data/elpmpp02 目录路径>");
    let base_path = Path::new(&base);
    let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
    let base_name = base_path.file_name().and_then(|p| p.to_str()).unwrap_or("elpmpp02");
    let loader = DataLoaderNative::new(parent);
    let data = load_all(&loader, base_name, Elpmpp02Correction::DE406).expect("加载 ELP-MPP02 文本失败");

    let names = [
        ("ELP_MAIN.S1", data.period_v.as_slice()),
        ("ELP_MAIN.S2", data.period_u.as_slice()),
        ("ELP_MAIN.S3", data.period_r.as_slice()),
        ("ELP_PERT.S1", data.poisson_v.as_slice()),
        ("ELP_PERT.S2", data.poisson_u.as_slice()),
        ("ELP_PERT.S3", data.poisson_r.as_slice()),
    ];
    for (name, terms) in names {
        let bin = terms_to_binary(terms);
        let out_path = base_path.join(format!("{}.bin", name));
        std::fs::write(&out_path, &bin).expect("写入失败");
        println!("{} -> {} ({} bytes)", name, out_path.display(), bin.len());
    }
}
