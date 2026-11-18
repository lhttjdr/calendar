//! 明郑颁历实物（`data/calendar/mingzheng_shiwu_months.csv`，1671、1676、1677 年）与明郑大统推步
//! （`dingshuo::reckon_year_any("大统历（明郑子初进朔）")`：定朔在夜子初以后者置次日）逐月比较。
//!
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example mingzheng_shiwu_check
use lunar_core::calendar::dingshuo::reckon_year_any;
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let src = std::fs::read_to_string(root.join("data/calendar/mingzheng_shiwu_months.csv")).unwrap();
    let (mut n, mut ok) = (0, 0);
    for l in src.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        let (Ok(y), Ok(m)) = (c[0].parse::<i32>(), c[1].parse::<u8>()) else { continue };
        let leap = c[2] == "1";
        let (dx, sg) = (c[3], c[4]);
        let a = reckon_year_any("大统历（明郑子初进朔）", y).unwrap();
        let b = reckon_year_any("大统历（明郑子初进朔）", y + 1).unwrap();
        let i = a.iter().position(|e| e.1 == m && e.2 == leap).unwrap();
        let j1 = a.get(i + 1).map(|e| e.0).unwrap_or(b[0].0);
        let (rg, rd) = (gz(a[i].0), if j1 - a[i].0 == 30 { "大" } else { "小" });
        n += 1;
        if rg == sg && rd == dx {
            ok += 1;
        } else {
            println!("  {y}-{m} 实物 {sg}{dx} 推步 {rg}{rd}");
        }
    }
    println!("明郑实物 {n} 月：合子初进朔推步 {ok}");
}
