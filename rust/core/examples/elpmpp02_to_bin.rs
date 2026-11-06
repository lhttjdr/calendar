//! 将 ELP-MPP02 六个文本文件的原始系数转为二进制 .bin（格式 v2，与常数集无关），供前端零解析加载。
//! 运行（在 rust 目录）：cargo run -p lunar-core --example elpmpp02_to_bin -- ../vendor/elpmpp02
//! 会在同目录下生成 ELP_MAIN.S1.bin … ELP_PERT.S3.bin。

use lunar_core::astronomy::ephemeris::elpmpp02::parse::{main_to_binary, pert_to_binary, Elpmpp02Raw};
use lunar_core::platform::DataLoaderNative;
use std::path::Path;

fn main() {
    let base = std::env::args()
        .nth(1)
        .expect("用法: elpmpp02_to_bin <vendor/elpmpp02 目录路径>");
    let base_path = Path::new(&base);
    let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
    let base_name = base_path.file_name().and_then(|p| p.to_str()).unwrap_or("elpmpp02");
    let loader = DataLoaderNative::new(parent);
    let raw = Elpmpp02Raw::load(&loader, base_name).expect("加载 ELP-MPP02 文本失败");

    let mut out: Vec<(String, Vec<u8>)> = Vec::new();
    for (i, terms) in raw.main.iter().enumerate() {
        out.push((format!("ELP_MAIN.S{}", i + 1), main_to_binary(terms)));
    }
    for (i, terms) in raw.pert.iter().enumerate() {
        out.push((format!("ELP_PERT.S{}", i + 1), pert_to_binary(terms)));
    }
    for (name, bin) in out {
        let out_path = base_path.join(format!("{name}.bin"));
        std::fs::write(&out_path, &bin).expect("写入失败");
        println!("{} -> {} ({} bytes)", name, out_path.display(), bin.len());
    }
}
