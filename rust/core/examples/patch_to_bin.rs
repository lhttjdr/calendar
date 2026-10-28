//! 将 VSOP87 → DE406/DE441 改正系数表转为 wasm 用的二进制（只含地球 `ear`，节气、定朔只需地球）。
//! 运行：cargo run -p lunar-core --example patch_to_bin --no-default-features --features twofloat
//! 读取 data/fit/vsop87-{de406,de441}-planets.txt，写出 data/fit/vsop87-{de406,de441}-earth.bin。

fn main() {
    use lunar_core::astronomy::ephemeris::vsop87_patch::{PatchTarget, Vsop87DePatch};
    use lunar_core::repo;
    use lunar_core::repo::paths;

    for (target, out) in [
        (PatchTarget::De406, paths::FIT_VSOP87_DE406_EARTH_BIN),
        (PatchTarget::De441, paths::FIT_VSOP87_DE441_EARTH_BIN),
    ] {
        let mut patch = Vsop87DePatch::from_repo(target).unwrap_or_else(|e| panic!("{}: {e}", target.repo_path()));
        patch.retain(&["ear"]);
        let bin = patch.to_binary();
        let back = Vsop87DePatch::from_binary(&bin).expect("二进制往返失败");
        assert!(back.body("ear").is_some(), "二进制中无地球段");
        repo::write_bytes(out, &bin).expect("写入 .bin 失败");
        println!("{} → {} ({} bytes)", target.repo_path(), out, bin.len());
    }
}
