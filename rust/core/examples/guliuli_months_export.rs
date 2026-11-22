//! 导出战国前后（前 341—前 300）汉传六历与颛顼历各月，供出土楚简纪日检验（`scripts/calendar_data/baoshan_check.py`）。
//! 输出 data/calendar/guliuli_months_350_300.csv：历,闰法,朔日 JDN,下月朔 JDN,夏正年,夏正月,闰。
//! 月序一律换成夏正编号（子正历正月 = 夏正上一年十一月；颛顼历十月岁首照纪年年换算），便于与楚月名对应。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example guliuli_months_export
use lunar_core::calendar::guliuli::{year_months, Liu, Runfa};
use lunar_core::calendar::zhuanxu;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut out = String::from("历,闰法,朔日JDN,下月朔JDN,夏正年,夏正月,闰\n");
    let (y0, y1) = (-340, -299);
    let mut push = |name: &str, runfa: &str, ms: Vec<(i64, i32, u8, bool)>| {
        for w in ms.windows(2) {
            let (j, y, m, l) = w[0];
            out += &format!("{name},{runfa},{j},{},{y},{m},{}\n", w[1].0, l as u8);
        }
    };
    for li in Liu::ALL {
        let zheng_is_zi = !matches!(li, Liu::Xia | Liu::XiaYushui);
        for (rf, rn) in [(Runfa::Nianzhong, "年终"), (Runfa::Wuzhongqi, "无中气")] {
            let mut ms = Vec::new();
            for y in y0..=y1 {
                for (j, m, l) in year_months(li, y, rf) {
                    // 子正：正月 = 夏正上一年十一月
                    let (xy, xm) = if zheng_is_zi { if m <= 2 { (y - 1, m + 10) } else { (y, m - 2) } } else { (y, m) };
                    ms.push((j, xy, xm, l));
                }
            }
            push(li.name(), rn, ms);
        }
    }
    // 颛顼历：夏正年各月（十月岁首之年换回夏正年）
    let mut ms = Vec::new();
    for y in y0..=y1 {
        for (j, m, l) in zhuanxu::year_months(y) {
            ms.push((j, y, m, l));
        }
    }
    push("颛顼历", "年终", ms);
    std::fs::write(root.join("data/calendar/guliuli_months_350_300.csv"), out).unwrap();
    println!("ok");
}
