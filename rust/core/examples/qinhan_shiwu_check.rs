//! 秦汉初出土历谱、质日、纪年文书逐月检验（`data/calendar/qinhan_shiwu_months.csv`）：
//! 1. 每条实物要求的合朔改正 δ 可行区间（1/940 日，加在颛顼历平朔上），各段之交——即 `qinhan_shiyong::SEGMENTS` 的依据；
//! 2. 逐月对照：实物 / 本项目实用历推步 / 颛顼历推步 / 李忠林复原历；
//! 3. 前 221—太初元年四月各月的「依据」分类：实物年、实物定参数而唯一、δ 区间内未定、段界未定；
//! 4. 实用历推步与李忠林复原历不同之月。
//! 输出 doc/calendar/qinhan-shiwu-checks.md（覆盖写）。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example qinhan_shiwu_check
use lunar_core::calendar::artifacts::zhongyuan_artifact_year;
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::qinhan_shiyong::{self as sy, SEGMENTS};
use lunar_core::calendar::{qinhan_restore, zhuanxu};

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

fn mname(m: u8, leap: bool) -> String {
    let n = [
        "", "正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二",
    ][m as usize];
    if leap {
        format!("後{n}月")
    } else {
        format!("{n}月")
    }
}

fn yname(n: i32) -> String {
    format!("前 {}", 1 - n)
}

/// 建亥年 n 之月 (m, leap) 在改正 δ 下的朔日（不含史载例外）。
fn day(n: i32, m: u8, leap: bool, delta: i64) -> Option<i64> {
    sy::jianhai_year_months_with(n, delta)
        .into_iter()
        .find(|x| (x.1, x.2) == (m, leap))
        .map(|x| x.0)
}

/// 使该月朔日为 d 的 δ 区间 [lo, hi)。
fn interval(n: i32, m: u8, leap: bool, d: i64) -> Option<(i64, i64)> {
    let base = day(n, m, leap, 0)?;
    let lo = (-1200..2400).find(|&x| day(n, m, leap, x).unwrap() >= d)?;
    let hi = (lo..2400).find(|&x| day(n, m, leap, x).unwrap() > d)?;
    let _ = base;
    (day(n, m, leap, lo) == Some(d)).then_some((lo, hi))
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let csv = std::fs::read_to_string(root.join("data/calendar/qinhan_shiwu_months.csv")).unwrap();
    let mut lines = csv
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    let head: Vec<&str> = lines.next().unwrap().split(',').collect();
    let col = |n: &str| head.iter().position(|c| *c == n).unwrap();
    let (cs, cj, cy, cm, cl, cg, cb, cn) = (
        col("来源"),
        col("纪年"),
        col("年"),
        col("月"),
        col("闰"),
        col("朔干支"),
        col("依据"),
        col("简号"),
    );
    let recs: Vec<Vec<&str>> = lines.map(|l| l.split(',').collect()).collect();
    let is_exception = |n: i32, m: u8, leap: bool| {
        sy::EXCEPTIONS
            .iter()
            .any(|e| (e.0, e.1, e.2) == (n, m, leap))
    };

    let mut md = vec![
        "# 秦汉初出土历日逐月检验（自动生成）".to_string(),
        String::new(),
        "> 由 `rust/core/examples/qinhan_shiwu_check.rs` 生成，勿手改。转录见 `data/calendar/qinhan_shiwu_months.csv`，方法见 [qin-han-calendar-evaluation.md](./qin-han-calendar-evaluation.md) 第七节。".into(),
        String::new(),
        "δ：加在颛顼历平朔上的合朔改正，以 1/940 日计；「区间」为使该月朔日与实物相同的 δ 范围 [下, 上)。依据：书 = 简文直书朔干支，推 = 由简文月大小或日序推出，补 = 整理者补释。".into(),
        String::new(),
        "## 一、各段 δ 可行区间".into(),
        String::new(),
    ];
    // 1. 各段
    for (k, s) in SEGMENTS.iter().enumerate() {
        let to = SEGMENTS.get(k + 1).map_or(-103, |t| t.from - 1);
        md.push(format!(
            "### {}—{}（建亥年 {}—{}）：取 δ = {}，可行区间 [{}, {})",
            yname(s.from.max(-220)),
            yname(to),
            s.from.max(-220),
            to,
            s.delta(),
            s.lo,
            s.hi
        ));
        md.push(String::new());
        md.push(format!("依据：{}。", s.basis));
        md.push(String::new());
        md.push("| 来源 | 纪年 | 月 | 朔 | 依据 | 简号 | 区间 | 定下界/上界 |".into());
        md.push("|---|---|---|---|---|---|---|---|".into());
        let (mut lo, mut hi) = (i64::MIN, i64::MAX);
        let mut rows = Vec::new();
        for f in recs.iter() {
            let n: i32 = f[cy].parse().unwrap();
            if !(s.from..=to).contains(&n) {
                continue;
            }
            let (m, leap) = (f[cm].parse::<u8>().unwrap(), f[cl] == "1");
            let base = day(n, m, leap, 500).unwrap();
            let gi = GAN_ZHI_60.iter().position(|g| *g == f[cg]).unwrap() as i64;
            let d = (base - 30..=base + 29)
                .find(|&j| (j + 49).rem_euclid(60) == gi)
                .unwrap();
            let iv = interval(n, m, leap, d);
            let skip =
                f[cb] == "补" || is_exception(n, m, leap) || (f[cs] == "周家台" && n == -212);
            if let (Some((a, b)), false) = (iv, skip) {
                lo = lo.max(a);
                hi = hi.min(b);
            }
            rows.push((f, iv, skip));
        }
        for (f, iv, skip) in rows {
            let ivs = iv.map_or("—".into(), |(a, b)| format!("[{a}, {b})"));
            let mark = if skip {
                "不计".to_string()
            } else {
                match iv {
                    Some((a, b)) if a == lo && b == hi => "下界、上界".into(),
                    Some((a, _)) if a == lo => "下界".into(),
                    Some((_, b)) if b == hi => "上界".into(),
                    _ => String::new(),
                }
            };
            let (m, leap) = (f[cm].parse::<u8>().unwrap(), f[cl] == "1");
            md.push(format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                f[cs],
                f[cj],
                mname(m, leap),
                f[cg],
                f[cb],
                f[cn],
                ivs,
                mark
            ));
        }
        md.push(String::new());
        md.push(format!("各条之交：[{lo}, {hi})。「不计」：整理者补字、史载改朔（高祖九年七月）与秦始皇三十四年周家台本（与同年岳麓本三月、五月、七月相差一日，见下）。"));
        md.push(String::new());
    }

    // 2. 逐月对照
    md.push("## 二、逐月对照".into());
    md.push(String::new());
    md.push("实用 = 本项目实用历推步（`qinhan_shiyong`）；颛顼 = 颛顼历按术推步；李 = 李忠林复原历。✓ 合、✗ 不合。".into());
    md.push(String::new());
    md.push("| 来源 | 纪年 | 月 | 实物朔 | 依据 | 实用 | 颛顼 | 李 |".into());
    md.push("|---|---|---|---|---|---|---|---|".into());
    let (mut tot, mut hit) = (0, [0usize; 3]);
    for f in recs.iter() {
        let n: i32 = f[cy].parse().unwrap();
        let (m, leap) = (f[cm].parse::<u8>().unwrap(), f[cl] == "1");
        let find =
            |v: Vec<(i64, u8, bool)>| v.into_iter().find(|x| (x.1, x.2) == (m, leap)).map(|x| x.0);
        let a = find(sy::jianhai_year_months(n));
        let b = find(zhuanxu::jianhai_year_months(n));
        let c = find(qinhan_restore::jianhai_year_months(n));
        let cell = |x: Option<i64>, k: usize, hit: &mut [usize; 3]| match x {
            Some(j) if gz(j) == f[cg] => {
                hit[k] += 1;
                "✓".to_string()
            }
            Some(j) => format!("✗ {}", gz(j)),
            None => "—".into(),
        };
        tot += 1;
        let (x, y, z) = (
            cell(a, 0, &mut hit),
            cell(b, 1, &mut hit),
            cell(c, 2, &mut hit),
        );
        md.push(format!(
            "| {} | {} | {} | {} | {} | {x} | {y} | {z} |",
            f[cs],
            f[cj],
            mname(m, leap),
            f[cg],
            f[cb]
        ));
    }
    md.push(String::new());
    md.push(format!(
        "合计 {tot} 条：实用 {}、颛顼 {}、李 {}。",
        hit[0], hit[1], hit[2]
    ));
    md.push(String::new());
    println!("逐月 {tot}：实用 {} 颛顼 {} 李 {}", hit[0], hit[1], hit[2]);

    // 3. 逐月依据分类
    // 段界未定之窗：前一段最后一个实物年之后、后一段第一个实物年之前
    let attested: Vec<i32> = recs.iter().map(|f| f[cy].parse::<i32>().unwrap()).collect();
    let seg_of = |n: i32| SEGMENTS.iter().rposition(|s| n >= s.from).unwrap_or(0);
    let windows: Vec<(i32, i32, usize, usize)> = (1..SEGMENTS.len())
        .filter_map(|k| {
            let last_prev = attested
                .iter()
                .copied()
                .filter(|&n| seg_of(n) == k - 1)
                .max()?;
            let first_next = attested.iter().copied().filter(|&n| seg_of(n) == k).min()?;
            (first_next - last_prev > 1).then_some((last_prev + 1, first_next - 1, k - 1, k))
        })
        .collect();
    md.push("## 三、前 221—太初元年四月各月的依据".into());
    md.push(String::new());
    md.push(format!(
        "段界未定之窗（前段最后一个实物年与后段第一个实物年之间）：{}。窗内各月以两段 δ 推得不同日者记「段界未定」；本段 δ 可行区间两端推得不同日者记「区间未定」；其余由实物参数唯一确定，记「唯一」。",
        windows.iter().map(|w| format!("{}—{}（{} 段 / {} 段）", yname(w.0), yname(w.1), w.2 + 1, w.3 + 1)).collect::<Vec<_>>().join("、")
    ));
    md.push(String::new());
    md.push("| 夏正年 | 月数 | 实物年 | 唯一 | 区间未定 | 段界未定 | 未定之月 |".into());
    md.push("|---|---|---|---|---|---|---|".into());
    let (mut t_all, mut t_art, mut t_pin, mut t_iv, mut t_win) = (0, 0, 0, 0, 0);
    for y in -220..=-103 {
        let ms = sy::year_months(y).unwrap();
        let ms: Vec<_> = if y == -103 {
            ms.into_iter().filter(|x| x.1 <= 4).collect()
        } else {
            ms
        };
        let art = zhongyuan_artifact_year(y).is_some();
        let (mut pin, mut iv, mut win, mut list) = (0, 0, 0, Vec::new());
        for &(d, m, leap) in &ms {
            let n = if m >= 10 { y + 1 } else { y };
            if art {
                continue;
            }
            let s = &SEGMENTS[seg_of(n)];
            let alt = windows
                .iter()
                .find(|w| (w.0..=w.1).contains(&n))
                .map(|w| SEGMENTS[if seg_of(n) == w.2 { w.3 } else { w.2 }].delta());
            if is_exception(n, m, leap) {
                pin += 1;
            } else if alt.is_some_and(|a| day(n, m, leap, a) != Some(d)) {
                win += 1;
                list.push(format!(
                    "{}（{}/{}）",
                    mname(m, leap),
                    gz(d),
                    gz(day(n, m, leap, alt.unwrap()).unwrap())
                ));
            } else if day(n, m, leap, s.lo) != day(n, m, leap, s.hi - 1) {
                iv += 1;
                list.push(format!(
                    "{}（{}/{}）",
                    mname(m, leap),
                    gz(day(n, m, leap, s.lo).unwrap()),
                    gz(day(n, m, leap, s.hi - 1).unwrap())
                ));
            } else {
                pin += 1;
            }
        }
        t_all += ms.len();
        t_art += if art { ms.len() } else { 0 };
        t_pin += pin;
        t_iv += iv;
        t_win += win;
        md.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            yname(y),
            ms.len(),
            if art { "✓" } else { "" },
            pin,
            iv,
            win,
            list.join("、")
        ));
    }
    md.push(String::new());
    md.push(format!("合计 {t_all} 月：实物年 {t_art}，唯一 {t_pin}，区间未定 {t_iv}，段界未定 {t_win}。未定之月括注「本项目取 / 另一可能」。"));
    md.push(String::new());
    println!("各月：{t_all} 实物 {t_art} 唯一 {t_pin} 区间未定 {t_iv} 段界未定 {t_win}");

    // 4. 与李忠林复原历之差
    md.push("## 四、实用历推步与李忠林复原历不同之月".into());
    md.push(String::new());
    md.push("| 夏正年 | 月 | 实用 | 李 |".into());
    md.push("|---|---|---|---|".into());
    let mut nd = 0;
    for y in -220..=-104 {
        let a = sy::year_months(y).unwrap();
        let b = qinhan_restore::year_months(y).unwrap();
        for (x, z) in a.iter().zip(&b) {
            assert_eq!((x.1, x.2), (z.1, z.2));
            if x.0 != z.0 {
                nd += 1;
                md.push(format!(
                    "| {} | {} | {} | {} |",
                    yname(y),
                    mname(x.1, x.2),
                    gz(x.0),
                    gz(z.0)
                ));
            }
        }
    }
    md.push(String::new());
    md.push(format!("共 {nd} 月（置闰逐年相同）。"));
    println!("与李忠林不同 {nd} 月");
    std::fs::write(
        root.join("doc/calendar/qinhan-shiwu-checks.md"),
        md.join("\n") + "\n",
    )
    .unwrap();
}
