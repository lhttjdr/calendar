//! 将 VSOP87–DE406 赤道拟合表文本转为 .bin（共通读写在 repo）。
//! 运行：cargo run -p lunar-core --example patch_to_bin --no-default-features --features twofloat
//! 读取 data/fit/vsop87-de406-icrs.txt，写出 data/fit/vsop87-de406-icrs.bin。

fn main() {
    use lunar_core::astronomy::frame::vsop87_de406_icrs_patch;
    use lunar_core::repo;
    use lunar_core::repo::paths;

    let lines = repo::read_lines(paths::FIT_VSOP87_DE406_ICRS).expect("从 repo 读取拟合表文本失败");
    let data = vsop87_de406_icrs_patch::parse_patch_lines(&lines).expect("解析拟合表失败");
    let bin = vsop87_de406_icrs_patch::to_binary(&data);
    repo::write_bytes(paths::FIT_VSOP87_DE406_ICRS_BIN, &bin).expect("写入 .bin 失败");
    println!(
        "拟合表 (RA {} 项, Dec {} 项, R {} 项) -> {} ({} bytes)",
        data.ra_terms.len(),
        data.dec_terms.len(),
        data.r_terms.len(),
        paths::FIT_VSOP87_DE406_ICRS_BIN,
        bin.len()
    );
}
