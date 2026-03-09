//! 将 VSOP87 文本（如 VSOP87B.ear）转为二进制 .bin，供前端 fetch 零解析加载。
//! 运行：cargo run -p lunar-core --example vsop87_to_bin -- data/vsop87/VSOP87B.ear
//! 输出：同目录下 VSOP87B.ear.bin（或通过第二参数指定输出路径）。

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = args.get(1).expect("用法: vsop87_to_bin <input.ear> [output.bin]");
    let content = std::fs::read_to_string(input).expect("读取输入文件失败");
    let lines: Vec<String> = content.lines().map(String::from).collect();
    let vsop = lunar_core::astronomy::ephemeris::vsop87::Vsop87Parse::parse_from_lines(&lines)
        .expect("解析 VSOP87 文本失败");
    let bin = vsop.to_binary();
    let output = args.get(2).cloned().unwrap_or_else(|| format!("{}.bin", input));
    std::fs::write(&output, &bin).expect("写入输出文件失败");
    println!("{} -> {} ({} bytes)", input, output, bin.len());
}
