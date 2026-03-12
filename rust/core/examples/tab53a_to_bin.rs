//! 将 IERS 5.3a+5.3b 文本加载并导出为 .bin（共通读写在 repo）。
//! 支持 IERS 原表格式与 VLBI/脚本合并格式（单文件 5.3a），通过 load_iau2000a_from_repo 统一解析。
//! 运行：cargo run -p lunar-core --example tab53a_to_bin --no-default-features --features twofloat
//! 可选参数 [项目根] 未传时使用 REPO_ROOT 或 CARGO_MANIFEST_DIR 上两级。

fn main() {
    let iau = lunar_core::astronomy::frame::nutation::load_iau2000a_from_repo()
        .expect("从 repo 加载章动表失败（支持 IERS 5.3a/5.3b 或 VLBI 合并格式）");
    let bin = iau.to_binary();
    lunar_core::repo::write_bytes(lunar_core::repo::paths::IAU2000_TAB53A_BIN, &bin).expect("写入 .bin 失败");
    println!(
        "章动表 ({} 项) -> {} ({} bytes)",
        iau.term_count(),
        lunar_core::repo::paths::IAU2000_TAB53A_BIN,
        bin.len()
    );
}
