//! 预算岁数据表（网页首屏用）：与网页 Worker 的 `compute_year_data_full_binary` 同一组二进制历表与配置
//! （VSOP87B 地球 + DE431 地球改正、ELPMPP02 二进制 + 缺省常数集 DE431、IAU2000 全章动表），
//! 逐农历岁算朔、中气（含下一冬至）、朔弦望，并逐公历年算二十四节气，写成二进制表。
//!
//! 用法（仓库根）：
//!   cargo run --release -p lunar-core --no-default-features --features real-f64 --example gen_year_table -- 1900 2100 web/public/data/precomputed/yeardata-de431-f64.bin
//!
//! 格式（小端）：b"LYD1"、u32 版本 1、u32 配置名长 + UTF-8（"de431-f64"）、i32 首农历岁、u32 岁数、
//! 每岁 { u8 朔数, u8 中气数, u16 月相数, f64 朔[], f64 中气[], f64 月相时刻[], u8 月相种类[]（0 朔 1 上弦 2 望 3 下弦） }、
//! i32 首公历年、u32 年数、每年 24 个 f64 节气（0 春分 … 23 驚蟄）。时刻皆 JD(TT)，f64 原值不截断。
//! 见 doc/engineering/wasm-react-performance.md §12.6.2。
use lunar_core::astronomy::aspects::{solar_term_jds_for_year_cached, LunarPhase};
use lunar_core::astronomy::ephemeris::vsop87_patch::Vsop87DePatch;
use lunar_core::astronomy::ephemeris::{load_all_from_binary_with_fit, Elpmpp02Correction, Vsop87};
use lunar_core::astronomy::frame::nutation;
use lunar_core::calendar::chinese_lunar::{compute_year_data, year_phases};
use lunar_core::math::real::{real, RealOps};
use lunar_core::quantity::angle::PlaneAngle;
use std::fs;

fn read(p: &str) -> Vec<u8> {
    fs::read(p).unwrap_or_else(|e| panic!("读 {p}: {e}"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let y0: i32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1900);
    let y1: i32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2100);
    let out = args.get(3).cloned().unwrap_or_else(|| "web/public/data/precomputed/yeardata-de431-f64.bin".into());

    // 与网页同一组文件（web/public/data 由 npm run copy-data 自这些位置复制）
    let patch = Vsop87DePatch::from_binary(&read("data/fit/vsop87-de431-earth.bin")).expect("地球改正");
    let vsop = Vsop87::from_binary(&read("vendor/vsop87/VSOP87B.ear.bin")).expect("VSOP87").with_patch(patch.body("ear").expect("ear").clone());
    let corr = Elpmpp02Correction::DEFAULT;
    let fit = corr.builtin_fit().expect("ELP 常数集");
    let names = ["ELP_MAIN.S1", "ELP_MAIN.S2", "ELP_MAIN.S3", "ELP_PERT.S1", "ELP_PERT.S2", "ELP_PERT.S3"];
    let bufs: Vec<Vec<u8>> = names.iter().map(|n| read(&format!("vendor/elpmpp02/{n}.bin"))).collect();
    let elp = load_all_from_binary_with_fit([&bufs[0], &bufs[1], &bufs[2], &bufs[3], &bufs[4], &bufs[5]], &fit, corr).expect("ELPMPP02");
    assert!(nutation::try_init_full_nutation_from_binary(&read("vendor/IAU2000/tab5.3a.bin")), "章动表");

    let tol = PlaneAngle::from_rad(real(1e-8));
    let years: Vec<i32> = (y0..=y1).collect();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let chunk = years.len().div_ceil(threads);
    let mut per_year: Vec<(i32, Vec<u8>)> = std::thread::scope(|s| {
        let hs: Vec<_> = years
            .chunks(chunk)
            .map(|ys| {
                let (vsop, elp) = (&vsop, &elp);
                s.spawn(move || {
                    ys.iter()
                        .map(|&ly| {
                            let yd = compute_year_data(vsop, elp, ly, tol, 30).unwrap_or_else(|e| panic!("{ly}: {e}"));
                            let ph = year_phases(&yd, vsop, elp);
                            let mut b = Vec::new();
                            b.push(yd.new_moon_jds.len() as u8);
                            b.push(yd.zhong_qi_jds.len() as u8);
                            b.extend((ph.len() as u16).to_le_bytes());
                            for t in yd.new_moon_jds.iter().chain(yd.zhong_qi_jds.iter()).chain(ph.iter().map(|(t, _)| t)) {
                                b.extend(t.as_f64().to_le_bytes());
                            }
                            b.extend(ph.iter().map(|(_, p)| match p {
                                LunarPhase::Shuo => 0u8,
                                LunarPhase::ShangXian => 1,
                                LunarPhase::Wang => 2,
                                LunarPhase::XiaXian => 3,
                            }));
                            (ly, b)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    per_year.sort_by_key(|(y, _)| *y);

    let cfg = b"de431-f64";
    let mut o = Vec::new();
    o.extend(b"LYD1");
    o.extend(1u32.to_le_bytes());
    o.extend((cfg.len() as u32).to_le_bytes());
    o.extend(cfg);
    o.extend(y0.to_le_bytes());
    o.extend((per_year.len() as u32).to_le_bytes());
    for (_, b) in &per_year {
        o.extend(b);
    }
    // 节气：覆盖各岁 lunar_year−1 … lunar_year+1（与 Worker 回传的 solar_term_years 同）
    let (g0, g1) = (y0 - 1, y1 + 1);
    o.extend(g0.to_le_bytes());
    o.extend(((g1 - g0 + 1) as u32).to_le_bytes());
    for g in g0..=g1 {
        for t in solar_term_jds_for_year_cached(&vsop, g) {
            o.extend(t.as_f64().to_le_bytes());
        }
    }
    if let Some(dir) = std::path::Path::new(&out).parent() {
        fs::create_dir_all(dir).unwrap();
    }
    fs::write(&out, &o).unwrap();
    eprintln!("{out}: {} 岁（{y0}—{y1}），{} 公历年节气，{} 字节", per_year.len(), g1 - g0 + 1, o.len());
}
