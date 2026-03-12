//! 解析 IERS 表 5.3a/5.3b 章动系数。
//! IERS 原表：5.3a 仅经度（µas），5.3b 仅倾角（µas）；代码内合并为四元组供 Iau2000a 使用。

/// 单行解析结果：(14 个整数系数, (第一振幅, 第二振幅))，振幅为表值未换算
pub type ParsedTerm = (Vec<i32>, (f64, f64));

/// IERS 表 5.3a/5.3b 月日项键：(l, l′, F, D, Ω)
pub type IersLuniSolarKey = (i32, i32, i32, i32, i32);

/// µas → 弧秒
const MICROARCSEC_TO_ARCSEC: f64 = 1e-6;

// ---------- IERS 原表 5.3a/5.3b 解析（单位 µas，列序见 IERS Conventions Ch.5）----------

fn parse_f64(s: &str) -> f64 {
    s.replace('D', "E").replace('d', "e").parse().unwrap_or(0.0)
}

/// 解析 IERS 5.3a 数据行：i A_i A"_i l l' F D Om (9 行星)。仅保留月日项（行星列全 0）。返回 (key, (A_i, A"_i)) µas。
pub fn parse_iers_53a_j0_row(line: &str) -> Option<(IersLuniSolarKey, (f64, f64))> {
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() < 17 {
        return None;
    }
    let _i: i32 = parts[0].parse().ok()?;
    let a_i = parse_f64(parts[1]);
    let a_pp = parse_f64(parts[2]);
    let l: i32 = parts[3].parse().ok()?;
    let lp: i32 = parts[4].parse().ok()?;
    let f: i32 = parts[5].parse().ok()?;
    let d: i32 = parts[6].parse().ok()?;
    let om: i32 = parts[7].parse().ok()?;
    for p in parts.iter().take(17).skip(8) {
        if p.parse::<i32>().ok()? != 0 {
            return None;
        }
    }
    Some(((l, lp, f, d, om), (a_i, a_pp)))
}

/// 解析 IERS 5.3a j=1 行：i A'_i A"'_i l l' F D Om (9 行星)。
pub fn parse_iers_53a_j1_row(line: &str) -> Option<(IersLuniSolarKey, (f64, f64))> {
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() < 17 {
        return None;
    }
    let ap = parse_f64(parts[1]);
    let appp = parse_f64(parts[2]);
    let l: i32 = parts[3].parse().ok()?;
    let lp: i32 = parts[4].parse().ok()?;
    let f: i32 = parts[5].parse().ok()?;
    let d: i32 = parts[6].parse().ok()?;
    let om: i32 = parts[7].parse().ok()?;
    for p in parts.iter().take(17).skip(8) {
        if p.parse::<i32>().ok()? != 0 {
            return None;
        }
    }
    Some(((l, lp, f, d, om), (ap, appp)))
}

/// 解析 IERS 5.3b j=0 行：i B"_i B_i l l' F D Om。Δε = B·cos + B''·sin。
pub fn parse_iers_53b_j0_row(line: &str) -> Option<(IersLuniSolarKey, (f64, f64))> {
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() < 17 {
        return None;
    }
    let b_pp = parse_f64(parts[1]);
    let b_i = parse_f64(parts[2]);
    let l: i32 = parts[3].parse().ok()?;
    let lp: i32 = parts[4].parse().ok()?;
    let f: i32 = parts[5].parse().ok()?;
    let d: i32 = parts[6].parse().ok()?;
    let om: i32 = parts[7].parse().ok()?;
    for p in parts.iter().take(17).skip(8) {
        if p.parse::<i32>().ok()? != 0 {
            return None;
        }
    }
    Some(((l, lp, f, d, om), (b_i, b_pp)))
}

/// 解析 IERS 5.3b j=1 行：i B"'_i B'_i l l' F D Om。
pub fn parse_iers_53b_j1_row(line: &str) -> Option<(IersLuniSolarKey, (f64, f64))> {
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() < 17 {
        return None;
    }
    let bppp = parse_f64(parts[1]);
    let bp = parse_f64(parts[2]);
    let l: i32 = parts[3].parse().ok()?;
    let lp: i32 = parts[4].parse().ok()?;
    let f: i32 = parts[5].parse().ok()?;
    let d: i32 = parts[6].parse().ok()?;
    let om: i32 = parts[7].parse().ok()?;
    for p in parts.iter().take(17).skip(8) {
        if p.parse::<i32>().ok()? != 0 {
            return None;
        }
    }
    Some(((l, lp, f, d, om), (bp, bppp)))
}

/// 从 IERS 5.3a 文件内容抽取 j=0 月日项，保持表内顺序。
pub fn load_iers_53a_j0_keys_and_coeffs(lines: &[String]) -> Vec<(IersLuniSolarKey, (f64, f64))> {
    let mut in_j0 = false;
    let mut out = Vec::new();
    for line in lines {
        let t = line.trim();
        if t.contains("j = 0") && t.contains("Number of terms") {
            in_j0 = true;
            continue;
        }
        if in_j0 && (t.contains("j = 1") || t.starts_with("--")) {
            if t.contains("j = 1") {
                break;
            }
            continue;
        }
        if !in_j0 {
            continue;
        }
        if t.is_empty() {
            continue;
        }
        if let Some(pair) = parse_iers_53a_j0_row(t) {
            out.push(pair);
        }
    }
    out
}

/// 从 IERS 5.3a 文件内容抽取 j=1 月日项。
pub fn load_iers_53a_j1_map(lines: &[String]) -> std::collections::HashMap<IersLuniSolarKey, (f64, f64)> {
    let mut in_j1 = false;
    let mut out = std::collections::HashMap::new();
    for line in lines {
        let t = line.trim();
        if t.contains("j = 1") && t.contains("Number of terms") {
            in_j1 = true;
            continue;
        }
        if in_j1 && t.starts_with("--") && t.len() > 10 {
            continue;
        }
        if !in_j1 {
            continue;
        }
        if t.is_empty() {
            continue;
        }
        if let Some((k, v)) = parse_iers_53a_j1_row(t) {
            out.insert(k, v);
        }
    }
    out
}

/// 从 IERS 5.3b 文件抽取 j=0 月日项。
pub fn load_iers_53b_j0_map(lines: &[String]) -> std::collections::HashMap<IersLuniSolarKey, (f64, f64)> {
    let mut in_j0 = false;
    let mut out = std::collections::HashMap::new();
    for line in lines {
        let t = line.trim();
        if t.contains("j = 0") && t.contains("Number of terms") {
            in_j0 = true;
            continue;
        }
        if in_j0 && (t.contains("j = 1") || (t.starts_with("--") && t.len() > 10)) {
            if t.contains("j = 1") {
                break;
            }
            continue;
        }
        if !in_j0 {
            continue;
        }
        if t.is_empty() {
            continue;
        }
        if let Some((k, v)) = parse_iers_53b_j0_row(t) {
            out.insert(k, v);
        }
    }
    out
}

/// 从 IERS 5.3b 文件抽取 j=1 月日项。
pub fn load_iers_53b_j1_map(lines: &[String]) -> std::collections::HashMap<IersLuniSolarKey, (f64, f64)> {
    let mut in_j1 = false;
    let mut out = std::collections::HashMap::new();
    for line in lines {
        let t = line.trim();
        if t.contains("j = 1") && t.contains("Number of terms") {
            in_j1 = true;
            continue;
        }
        if in_j1 && t.starts_with("--") && t.len() > 10 {
            continue;
        }
        if !in_j1 {
            continue;
        }
        if t.is_empty() {
            continue;
        }
        if let Some((k, v)) = parse_iers_53b_j1_row(t) {
            out.insert(k, v);
        }
    }
    out
}

/// VLBI/合并格式：单文件每行 "L Lm F D Om  Period  Psi dPsi Eps dEps  Psi_out dPsi_out Eps_out dEps_out"（mas），
/// 如 scripts/iers_nutation_to_tab53a.py 输出或 IERS "NUTATION SERIES FROM VLBI DATA"。返回 quads，系数已转弧秒。
pub fn parse_vlbi_merged_to_quads(lines: &[String]) -> Vec<[ParsedTerm; 4]> {
    const MAS_TO_ARCSEC: f64 = 1e-3;
    let mut quads = Vec::new();
    for line in lines {
        let t = line.trim();
        if t.is_empty() || t.starts_with('*') {
            continue;
        }
        let parts: Vec<&str> = t.split_whitespace().collect();
        if parts.len() < 14 {
            continue;
        }
        let l: i32 = match parts[0].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let lp: i32 = match parts[1].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let f: i32 = match parts[2].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let d: i32 = match parts[3].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let om: i32 = match parts[4].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let psi_in = parse_f64(parts[6]) * MAS_TO_ARCSEC;
        let dpsi_in = parse_f64(parts[7]) * MAS_TO_ARCSEC;
        let eps_in = parse_f64(parts[8]) * MAS_TO_ARCSEC;
        let deps_in = parse_f64(parts[9]) * MAS_TO_ARCSEC;
        let psi_out = parse_f64(parts[10]) * MAS_TO_ARCSEC;
        let dpsi_out = parse_f64(parts[11]) * MAS_TO_ARCSEC;
        let eps_out = parse_f64(parts[12]) * MAS_TO_ARCSEC;
        let deps_out = parse_f64(parts[13]) * MAS_TO_ARCSEC;
        let mut c14 = vec![l, lp, f, d, om];
        c14.extend(std::iter::repeat(0).take(9));
        quads.push([
            (c14.clone(), (psi_in, psi_out)),
            (c14.clone(), (dpsi_in, dpsi_out)),
            (c14.clone(), (eps_in, eps_out)),
            (c14.clone(), (deps_in, deps_out)),
        ]);
    }
    quads
}

/// 将 IERS 5.3a+5.3b 合并为 Iau2000a 使用的四元组列表。顺序按 5.3a j=0 月日项顺序；系数 µas→弧秒。
pub fn merge_iers_53a_53b_to_quads(
    a0: Vec<(IersLuniSolarKey, (f64, f64))>,
    a1: &std::collections::HashMap<IersLuniSolarKey, (f64, f64)>,
    b0: &std::collections::HashMap<IersLuniSolarKey, (f64, f64)>,
    b1: &std::collections::HashMap<IersLuniSolarKey, (f64, f64)>,
) -> Vec<[ParsedTerm; 4]> {
    let scale = MICROARCSEC_TO_ARCSEC;
    let mut quads = Vec::with_capacity(a0.len());
    for (key, (a_i, a_pp)) in a0 {
        let (l, lp, f, d, om) = key;
        let (ap, appp) = a1.get(&key).copied().unwrap_or((0.0, 0.0));
        let (b_i, b_pp) = b0.get(&key).copied().unwrap_or((0.0, 0.0));
        let (bp, bppp) = b1.get(&key).copied().unwrap_or((0.0, 0.0));
        let mut c14 = vec![l, lp, f, d, om];
        c14.extend(std::iter::repeat(0).take(9));
        quads.push([
            (c14.clone(), (a_i * scale, a_pp * scale)),
            (c14.clone(), (ap * scale, appp * scale)),
            (c14.clone(), (b_i * scale, b_pp * scale)),
            (c14.clone(), (bp * scale, bppp * scale)),
        ]);
    }
    quads
}

#[cfg(test)]
mod tests {
    /// 当存在 data/IAU2000/tab5.3a.txt + tab5.3b.txt 时从 repo 读并合并
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn load_iers_53a_53b_from_data_dir() {
        use super::super::load;
        if let Ok(iau) = load::load_iau2000a_from_repo() {
            assert!(iau.term_count() > 0, "IERS 5.3a+5.3b 合并后应有项");
        }
    }
}
