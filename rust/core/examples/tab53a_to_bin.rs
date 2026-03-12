//! 将 IERS 5.3a+5.3b 文本加载并导出为 .bin（共通读写在 repo）。
//! 运行：cargo run -p lunar-core --example tab53a_to_bin --no-default-features --features twofloat
//! 可选参数 [项目根] 未传时使用 REPO_ROOT 或 CARGO_MANIFEST_DIR 上两级。

fn main() {
    let path_a = lunar_core::repo::paths::IAU2000_TAB53A;
    let path_b = lunar_core::repo::paths::IAU2000_TAB53B;
    let lines_a = lunar_core::repo::read_lines(path_a).expect("读取 5.3a 失败");
    let lines_b = lunar_core::repo::read_lines(path_b).expect("读取 5.3b 失败");
    let iau = lunar_core::astronomy::frame::nutation::parse_iau2000a_from_iers_lines(&lines_a, &lines_b)
        .expect("解析 IERS 5.3a+5.3b 失败");
    let bin = iau.to_binary();
    lunar_core::repo::write_bytes(lunar_core::repo::paths::IAU2000_TAB53A_BIN, &bin).expect("写入 .bin 失败");
    println!(
        "{} + {} ({} 项) -> {} ({} bytes)",
        path_a,
        path_b,
        iau.term_count(),
        lunar_core::repo::paths::IAU2000_TAB53A_BIN,
        bin.len()
    );
}
