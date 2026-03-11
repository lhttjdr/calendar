//! 将 tab5.3a 文本转为 .bin（与 vsop87_to_bin、elpmpp02_to_bin 一致）。.br 由前端或脚本压缩。
//! 运行：cargo run -p lunar-core --example tab53a_to_bin --no-default-features --features twofloat -- [项目根目录]
//! 默认项目根为 ../..（从 rust 目录跑时），读取 data/IAU2000/tab5.3a.txt，输出 data/IAU2000/tab5.3a.bin。

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base = args.get(1).map(String::as_str).unwrap_or("../..");
    let loader = lunar_core::platform::DataLoaderNative::new(base);
    let path = lunar_core::astronomy::frame::nutation::DEFAULT_TAB53A_PATH;
    let iau = lunar_core::astronomy::frame::nutation::load_iau2000a(&loader, path)
        .expect("加载 tab5.3a 失败（检查路径与 DataLoader）");
    let bin = iau.to_binary();
    let out_path = std::path::Path::new(base).join("data/IAU2000/tab5.3a.bin");
    if let Some(p) = out_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    std::fs::write(&out_path, &bin).expect("写入 .bin 失败");
    println!("{} ({} 行) -> {} ({} bytes)", path, iau.term_count(), out_path.display(), bin.len());
}
