//! 导出大衍一系（大衍、五纪、正元等）逐月定朔明细，供进朔规则通盘检讨（scripts/calendar_data/dayan_jinshuo_rules.py）。
//! 列：历,冬至岁,月序,定朔所在日JDN,定小余时,进朔限时,有交,排月朔日JDN。
//!
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example dayan_months_dump > <输出>

use lunar_core::calendar::dingshuo::dingshuo_months;

fn main() {
    println!("历,冬至岁,月序,定朔所在日JDN,定小余时,进朔限时,有交,排月朔日JDN");
    for name in ["大衍历", "五纪历", "正元历"] {
        for m in dingshuo_months(name).unwrap_or_else(|| panic!("{name}")) {
            println!(
                "{name},{},{},{},{:.3},{},{},{}",
                m.dongzhi_year,
                m.index,
                m.ding_jdn,
                m.frac * 24.0,
                m.limit.map_or(String::new(), |l| format!("{:.3}", l * 24.0)),
                m.youjiao.map_or("", |b| if b { "1" } else { "0" }),
                m.day
            );
        }
    }
}
