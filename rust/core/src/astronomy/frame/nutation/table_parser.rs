//! 解析 IERS 表 5.3a/5.3b 章动系数。
//! 通用格式：每行 14 个整数 (C1..C14) + 2 个实数（振幅，单位 0.0001 mas）。

/// 单行解析结果：(14 个整数系数, (第一振幅, 第二振幅))，振幅为表值未换算
pub type ParsedTerm = (Vec<i32>, (f64, f64));

/// 跳过空行和 # 注释；否则按空白分割，至少 16 列：14 个整数 + 2 个实数。
/// 支持 D→E 科学计数法。
pub fn parse_line(line: &str) -> Option<ParsedTerm> {
    let s = line.trim();
    if s.is_empty() || s.starts_with('#') {
        return None;
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 16 {
        return None;
    }
    let mut ints = Vec::with_capacity(14);
    for p in parts.iter().take(14) {
        let s = p.replace('D', "E").replace('d', "e");
        let v: i32 = s.parse().ok()?;
        ints.push(v);
    }
    let a1 = parts[14].replace('D', "E").replace('d', "e").parse::<f64>().ok()?;
    let a2 = parts[15].replace('D', "E").replace('d', "e").parse::<f64>().ok()?;
    Some((ints, (a1, a2)))
}

/// 解析多行，返回所有有效行对应的项列表
pub fn parse_file(lines: &[String]) -> Vec<ParsedTerm> {
    lines.iter().filter_map(|s| parse_line(s)).collect()
}

/// data/IAU2000/tab5.3a 格式：每行 5 个乘数 (L Lm F D Om) + Period(days) + 8 列 (mas)。
/// 返回 (ψ项, ψ率项, ε项, ε率项)；振幅 mas→弧秒×0.001。
pub fn parse_tab53a_line(line: &str) -> Option<[ParsedTerm; 4]> {
    let s = line.trim();
    if s.is_empty() || s.starts_with('*') {
        return None;
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 5 + 1 + 8 {
        return None;
    }
    let parse_f64 = |i: usize| -> f64 {
        parts[i].replace('D', "E").replace('d', "e").parse().unwrap_or(0.0)
    };
    let mut c5 = Vec::with_capacity(5);
    for i in 0..5 {
        c5.push(parse_f64(i).round() as i32);
    }
    let mut c14 = c5;
    c14.extend(std::iter::repeat(0).take(9));
    let scale = 0.001_f64;
    let psi_in = parse_f64(6) * scale;
    let d_psi_in = parse_f64(7) * scale;
    let eps_in = parse_f64(8) * scale;
    let d_eps_in = parse_f64(9) * scale;
    let psi_out = parse_f64(10) * scale;
    let d_psi_out = parse_f64(11) * scale;
    let eps_out = parse_f64(12) * scale;
    let d_eps_out = parse_f64(13) * scale;
    Some([
        (c14.clone(), (psi_in, psi_out)),
        (c14.clone(), (d_psi_in, d_psi_out)),
        (c14.clone(), (eps_in, eps_out)),
        (c14.clone(), (d_eps_in, d_eps_out)),
    ])
}

/// 从 DataLoader 加载并解析 tab5.3a 文件，返回所有行的四元组 (ψ, ψ率, ε, ε率) 列表。
pub fn load_tab53a<L: crate::platform::DataLoader>(
    loader: &L,
    path: &str,
) -> Result<Vec<[ParsedTerm; 4]>, crate::platform::LoadError> {
    let lines = loader.read_lines(path)?;
    let quads: Vec<[ParsedTerm; 4]> = lines
        .iter()
        .filter_map(|s| parse_tab53a_line(s))
        .collect();
    Ok(quads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn parse_line_skip_empty_and_comment() {
        assert!(parse_line("").is_none());
        assert!(parse_line("  ").is_none());
        assert!(parse_line("# comment").is_none());
    }

    #[test]
    fn parse_line_14_ints_2_reals() {
        // IERS-style: Omega-only term for Δψ, approximate first term from PDF (26)
        let line = "0 0 0 0 1 0 0 0 0 0 0 0 0 0 -172064161 33386";
        let opt = parse_line(line);
        assert!(opt.is_some());
        let (c, (a1, a2)) = opt.unwrap();
        assert_eq!(c.len(), 14);
        assert_eq!(c[4], 1);
        assert_eq!(c[0], 0);
        assert!(real(a1).is_near(real(-172064161.0), 1.0));
        assert!(real(a2).is_near(real(33386.0), 1.0));
    }

    #[test]
    fn parse_file_multiple_lines() {
        let lines = [
            "0 0 0 0 1 0 0 0 0 0 0 0 0 0 -172064161 33386".to_string(),
            "0 0 2 -2 2 0 0 0 0 0 0 0 0 0 -13170906 -13696".to_string(),
        ];
        let terms = parse_file(&lines);
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].0[4], 1);
        assert_eq!(terms[1].0[2], 2);
    }

    #[test]
    fn parse_tab53a_first_data_line() {
        let line = "   0  0  0  0  1    -6798.383 -17206.4161 -17.4666  9205.2331  0.9086  3.3386  0.0029  1.5377  0.0002";
        let quad = parse_tab53a_line(line);
        assert!(quad.is_some());
        let q = quad.unwrap();
        assert_eq!(q[0].0[4], 1);
        assert!(real(q[0].1.0).is_near(real(-17.2064161), 0.001));
    }

    /// 当存在 data/IAU2000/tab5.3a.txt 时加载并解析（外部数据迁移）
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn load_tab53a_from_data_dir() {
        use crate::platform::DataLoaderNative;
        let loader = DataLoaderNative::new(".");
        let r = load_tab53a(&loader, "data/IAU2000/tab5.3a.txt");
        if let Ok(quads) = r {
            assert!(!quads.is_empty(), "tab5.3a should have rows");
            assert_eq!(quads[0][0].0[4], 1);
        }
    }
}
