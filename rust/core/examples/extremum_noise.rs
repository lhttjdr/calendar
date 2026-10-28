//! 极值处目标函数的数值噪声：在金星大距（2020-03-24 22:15 TT 附近）前后 ±5 min 每 0.5 s 求金星–太阳角距，拟合二次曲线，
//! 看残差（噪声）多大、由哪一环节来。比较 DE406 与 VSOP87 + ELPMPP02 两种历表提供者。
//! 用法（rust/ 目录）：cargo run --release -p lunar-core --example extremum_noise [--no-default-features --features real-f64]
use lunar_core::astronomy::apparent::ApparentPipelineOptions;
use lunar_core::astronomy::ephemeris::vsop87_patch::PatchTarget;
use lunar_core::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo, De406Kernel};
use lunar_core::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use lunar_core::astronomy::phenomena::angular_separation_deg;
use lunar_core::astronomy::pipeline::{Body, EphemerisProvider, Planet, Vsop87Elp, Vsop87Planets};
use lunar_core::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use lunar_core::math::real::{real, RealOps};

fn noise<P: EphemerisProvider>(name: &str, eph: &P, opts: &ApparentPipelineOptions) {
    let t0 = 2458933.427; // 2020-03-24 22:15 TT
    let n = 1201;
    let h = 0.5 / 86400.0;
    let xs: Vec<f64> = (0..n).map(|i| (i as f64 - 600.0) * h).collect();
    // 角距（rad），减去中点值以免抵消
    let ys_r: Vec<_> = xs.iter().map(|x| angular_separation_deg(eph, Body::Planet(Planet::Venus), Body::Sun, real(t0) + real(*x), opts)).collect();
    let y0 = ys_r[600];
    let ys: Vec<f64> = ys_r.iter().map(|y| ((*y - y0).as_f64()).to_radians()).collect();
    // 最小二乘二次拟合 y = a + b x + c x²（x 以日计）
    let (mut s, mut m) = ([0.0f64; 5], [0.0f64; 3]);
    for (x, y) in xs.iter().zip(&ys) {
        let mut p = 1.0;
        for k in 0..5 { s[k] += p; if k < 3 { m[k] += p * y; } p *= x; }
    }
    let a = [[s[0], s[1], s[2]], [s[1], s[2], s[3]], [s[2], s[3], s[4]]];
    let det = |a: [[f64; 3]; 3]| a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1]) - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0]) + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    let d = det(a);
    let sol: Vec<f64> = (0..3).map(|k| { let mut b = a; for r in 0..3 { b[r][k] = m[r]; } det(b) / d }).collect();
    let res: Vec<f64> = xs.iter().zip(&ys).map(|(x, y)| y - (sol[0] + sol[1] * x + sol[2] * x * x)).collect();
    let rms = (res.iter().map(|r| r * r).sum::<f64>() / n as f64).sqrt();
    let max = res.iter().fold(0.0f64, |a, r| a.max(r.abs()));
    let t_max = -sol[1] / (2.0 * sol[2]) * 86400.0;
    // 噪声 rms 对应的极值时刻不确定度：Δt = sqrt(2·rms / |c|)
    let dt = (2.0 * rms / sol[2].abs()).sqrt() * 86400.0;
    println!("{name}：二次项 {:.3e} rad/d²，拟合极大在中点 {t_max:+.2} s；残差 rms {rms:.2e} rad，最大 {max:.2e} rad → 极值时刻不确定度约 {dt:.1} s", sol[2]);
    // 相邻差分的符号：真曲线在极大两侧应单调，噪声会使符号乱跳
    let flips = ys.windows(3).filter(|w| (w[1] - w[0]).signum() != (w[2] - w[1]).signum()).count();
    println!("  相邻差分符号翻转 {flips} 次（无噪声应为 1 次）");
}

/// 历表提供者的几何地心位置（不经视位置管线）各分量在 ±5 min 内的二次拟合残差（米）。
fn raw_noise<P: EphemerisProvider>(name: &str, eph: &P, body: Body) {
    use lunar_core::astronomy::time::{TimePoint, TimeScale};
    let t0 = 2458933.427;
    let h = 0.5 / 86400.0;
    let xs: Vec<f64> = (0..1201).map(|i| (i as f64 - 600.0) * h).collect();
    let st: Vec<[f64; 3]> = xs.iter().map(|x| {
        let p = eph.compute_state(body, TimePoint::new(TimeScale::TT, real(t0) + real(*x))).position.to_meters();
        [p[0].as_f64(), p[1].as_f64(), p[2].as_f64()]
    }).collect();
    let mut worst = 0.0f64;
    for c in 0..3 {
        let y0 = st[600][c];
        let ys: Vec<f64> = st.iter().map(|p| p[c] - y0).collect();
        // 三点二阶差分的离散度即噪声（真曲线二阶差分在 0.5 s 步长下近乎常数）
        let d2: Vec<f64> = ys.windows(3).map(|w| w[0] - 2.0 * w[1] + w[2]).collect();
        let mean = d2.iter().sum::<f64>() / d2.len() as f64;
        let sd = (d2.iter().map(|d| (d - mean) * (d - mean)).sum::<f64>() / d2.len() as f64).sqrt();
        worst = worst.max(sd / 6f64.sqrt());
    }
    println!("{name} 几何地心 {body:?} 位置噪声（二阶差分估计）约 {worst:.2e} m");
}

/// 分段找噪声来源：(a) 几何地心方向；(b) 光行时修正后（未转架）。各取金星–太阳角距的二阶差分离散度（rad）。
fn stage_noise<P: EphemerisProvider>(name: &str, eph: &P) {
    use lunar_core::astronomy::pipeline::{Fk5ToIcrsBias, LightTimeCorrector};
    use lunar_core::astronomy::time::{TimePoint, TimeScale};
    let t0 = 2458933.427;
    let h = 0.5 / 86400.0;
    let ang = |u: [f64; 3], v: [f64; 3]| {
        let cr = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt().atan2(u[0] * v[0] + u[1] * v[1] + u[2] * v[2])
    };
    let f = |p: [lunar_core::math::real::Real; 3]| [p[0].as_f64(), p[1].as_f64(), p[2].as_f64()];
    let spread = |ys: Vec<f64>| {
        let d2: Vec<f64> = ys.windows(3).map(|w| w[0] - 2.0 * w[1] + w[2]).collect();
        let mean = d2.iter().sum::<f64>() / d2.len() as f64;
        (d2.iter().map(|d| (d - mean) * (d - mean)).sum::<f64>() / d2.len() as f64).sqrt() / 6f64.sqrt()
    };
    let corr: LightTimeCorrector<'_, P, Fk5ToIcrsBias> = LightTimeCorrector { ephemeris: eph, mapper: None, max_iter: 3 };
    let (mut ga, mut lt) = (vec![], vec![]);
    for i in 0..601 {
        let t = TimePoint::new(TimeScale::TT, real(t0) + real((i as f64 - 300.0) * h));
        let v = eph.compute_state(Body::Planet(Planet::Venus), t.to_scale(eph.evaluation_time_scale())).position.to_meters();
        let sn = eph.compute_state(Body::Sun, t.to_scale(eph.evaluation_time_scale())).position.to_meters();
        ga.push(ang(f(v), f(sn)));
        let (_, sv) = corr.retarded_state(t, Body::Planet(Planet::Venus));
        let (_, ss) = corr.retarded_state(t, Body::Sun);
        lt.push(ang(f(sv.position.to_meters()), f(ss.position.to_meters())));
    }
    println!("{name} 分段噪声：几何 {:.2e} rad，光行时后 {:.2e} rad", spread(ga), spread(lt));
    // τ(t) 与推迟时刻处的位置
    let (mut tau, mut px) = (vec![], vec![]);
    for i in 0..601 {
        let t = TimePoint::new(TimeScale::TT, real(t0) + real((i as f64 - 300.0) * h));
        let (tr, sv) = corr.retarded_state(t, Body::Planet(Planet::Venus));
        tau.push(((t.jd - tr.to_scale(TimeScale::TT).jd) * real(86400.0)).as_f64());
        px.push(sv.position.to_meters()[0].as_f64());
        if i == 300 { println!("  tr 时标 {:?}，τ = {:.6} s", tr.scale, tau[300]); }
    }
    println!("  τ 噪声 {:.2e} s；推迟时刻金星 x 噪声 {:.2e} m", spread(tau), spread(px));
    let (mut a, mut b) = (vec![], vec![]);
    for i in 0..601 {
        let t = TimePoint::new(TimeScale::TT, real(t0) + real((i as f64 - 300.0) * h) - real(353.64 / 86400.0));
        a.push(eph.compute_state(Body::Planet(Planet::Venus), t).position.to_meters()[0].as_f64());
        b.push(eph.compute_state(Body::Planet(Planet::Venus), t.to_scale(TimeScale::TDB)).position.to_meters()[0].as_f64());
    }
    println!("  常数偏移：传 TT {:.2e} m，先转 TDB 再传 {:.2e} m", spread(a), spread(b));
    let mut maxd: f64 = 0.0;
    let mut c = vec![];
    for i in 0..601 {
        let t = TimePoint::new(TimeScale::TT, real(t0) + real((i as f64 - 300.0) * h));
        let (tr, sv) = corr.retarded_state(t, Body::Planet(Planet::Venus));
        let direct = eph.compute_state(Body::Planet(Planet::Venus), tr.to_scale(eph.evaluation_time_scale())).position.to_meters()[0];
        maxd = maxd.max((sv.position.to_meters()[0] - direct).as_f64().abs());
        c.push(direct.as_f64());
    }
    println!("  同一推迟时刻：光行时状态与直接求值之差最大 {maxd:.2e} m；直接求值序列噪声 {:.2e} m", spread(c.clone()));
    let d2: Vec<f64> = c.windows(3).map(|w| w[0] - 2.0 * w[1] + w[2]).collect();
    let mean = d2.iter().sum::<f64>() / d2.len() as f64;
    let big: Vec<(usize, f64)> = d2.iter().enumerate().map(|(i, d)| (i, d - mean)).filter(|(_, d)| d.abs() > 1e-3).collect();
    for i in 0..4 {
        let t = TimePoint::new(TimeScale::TT, real(t0) + real((i as f64 - 300.0) * h));
        let (tr, _) = corr.retarded_state(t, Body::Planet(Planet::Venus));
        let hi = tr.jd.as_f64();
        let te = tr.to_scale(eph.evaluation_time_scale());
        let full = eph.compute_state(Body::Planet(Planet::Venus), te).position.to_meters()[0];
        let rounded = eph.compute_state(Body::Planet(Planet::Venus), TimePoint::new(te.scale, real(te.jd.as_f64()))).position.to_meters()[0];
        println!("  x(全精度) − x(取整到 f64) = {:.3e} m", (full - rounded).as_f64());
        println!("  t.jd lo {:.3e}  tr.jd hi {hi:.10} lo {:.3e}", (t.jd - real(t.jd.as_f64())).as_f64(), (tr.jd - real(hi)).as_f64());
    }
    println!("  二阶差分偏离均值 >1 mm 的点 {} 个，前几个 {:?}", big.len(), &big[..big.len().min(6)]);
}

fn subquantum<P: EphemerisProvider>(name: &str, eph: &P) {
    use lunar_core::astronomy::time::{TimePoint, TimeScale};
    let base = real(2458933.4211765178);
    let xs: Vec<f64> = (0..8).map(|k| {
        let t = TimePoint::new(TimeScale::TDB, base + real(k as f64 * 1e-10));
        eph.compute_state(Body::Planet(Planet::Venus), t).position.to_meters()[0].as_f64()
    }).collect();
    let d: Vec<String> = xs.windows(2).map(|w| format!("{:.4}", w[1] - w[0])).collect();
    println!("{name} 时刻每步 1e-10 日（约 8.6 µs）x 的增量（m）：{}", d.join(" "));
}

fn main() {
    assert!(try_init_full_nutation_from_repo());
    let opts = ApparentPipelineOptions::pipeline_default();
    if let Some(bsp) = std::env::var("DE406_BSP").ok().or(Some("../vendor/jpl/de406/de406.bsp".into())).filter(|p| std::path::Path::new(p).exists()) {
        let k = De406Kernel::open(&bsp).unwrap();
        raw_noise("DE406", &k, Body::Planet(Planet::Venus));
        raw_noise("DE406", &k, Body::Sun);
        subquantum("DE406", &k);
        stage_noise("DE406", &k);
        noise("DE406", &k, &opts);
    }
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let loaded: Vec<_> = Planet::ALL.iter().map(|p| (*p, load_planet_vsop87_corrected_from_repo(*p, PatchTarget::De406).unwrap())).collect();
    let elp = load_all(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: loaded.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
    raw_noise("VSOP87 + ELPMPP02", &eph, Body::Planet(Planet::Venus));
    raw_noise("VSOP87 + ELPMPP02", &eph, Body::Sun);
    subquantum("VSOP87 + ELPMPP02", &eph);
    stage_noise("VSOP87 + ELPMPP02", &eph);
    noise("VSOP87 + ELPMPP02", &eph, &opts);
}
