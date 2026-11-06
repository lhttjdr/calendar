//! ELP/MPP02 常数改正集：论文 Table 3 的 13 个常数改正 + Table 6 式长期项加性改正。
//!
//! 一组改正决定一套历表常数（见 [ParseConstants::from_fit](super::parse_constants::ParseConstants::from_fit)，
//! 照 Fortran `INITIAL` 导出幅角多项式与振幅改正）。内置两组论文原值：
//! - [Elpmpp02Fit::llr]：Table 3 第 3 列（拟合 1970—2001 LLR），无长期项改正（Fortran icor=0）；
//! - [Elpmpp02Fit::de405]：Table 3 第 2 列（拟合 DE405，1950—2060）+ Table 6（使 6000 年内贴近 DE406）（Fortran icor=1）。
//!
//! 本项目按同一方法对 JPL 历表的拟合结果（`data/fit/elpmpp02-<目标>.txt`，由 `examples/elpmpp02_de_fit.rs` 生成，
//! 格式见 [Elpmpp02Fit::parse]）编译时嵌入：[Elpmpp02Fit::de431]（缺省，MS2016/2021 ΔT 所配历表）、[Elpmpp02Fit::de441]。

use crate::platform::LoadError;

/// 一组 ELP/MPP02 常数改正。单位：角秒；带 `_1` 的为角秒/儒略世纪，带 `_2` 的为角秒/世纪²；
/// `secular[i][k]` 为 W_{i+1} 的 T^k 系数的加性改正（角秒/世纪^k，只用 k ≥ 2）。
#[derive(Clone, Debug, PartialEq)]
pub struct Elpmpp02Fit {
    /// ΔW1⁽⁰⁾：月亮平黄经 J2000 值
    pub dw1_0: f64,
    /// ΔW2⁽⁰⁾：近地点平黄经 J2000 值
    pub dw2_0: f64,
    /// ΔW3⁽⁰⁾：升交点平黄经 J2000 值
    pub dw3_0: f64,
    /// ΔW1⁽¹⁾ = Δν：月亮恒星平运动
    pub dw1_1: f64,
    /// ΔW2⁽¹⁾：近地点平运动（另加由 Δν、ΔΓ、ΔE、Δe′、Δn′ 推得的 δW2⁽¹⁾，论文 §4.3.2）
    pub dw2_1: f64,
    /// ΔW3⁽¹⁾：升交点平运动（另加推得的 δW3⁽¹⁾）
    pub dw3_1: f64,
    /// ΔW1⁽²⁾：月亮平黄经二次项（含潮汐加速度之半）
    pub dw1_2: f64,
    /// ΔΓ：纬度中 sin F 系数之半
    pub dgam: f64,
    /// ΔE：经度中 sin l 系数之半
    pub de: f64,
    /// ΔT⁽⁰⁾：地月质心平黄经 J2000 值
    pub deart_0: f64,
    /// ΔT⁽¹⁾ = Δn′：地月质心平运动
    pub deart_1: f64,
    /// Δϖ′⁽⁰⁾：地月质心近日点平黄经 J2000 值
    pub dperi: f64,
    /// Δe′：地月质心日心轨道偏心率
    pub dep: f64,
    /// 长期项加性改正（论文 Table 6 的形式）：`secular[i][k]` 加到 W_{i+1} 的 T^k 系数。
    pub secular: [[f64; 5]; 3],
}

/// 文本格式中的参数名（顺序即输出顺序）。
const SCALAR_KEYS: [&str; 13] = [
    "dw1_0", "dw2_0", "dw3_0", "dw1_1", "dw2_1", "dw3_1", "dw1_2", "dgam", "de", "deart_0", "deart_1", "dperi", "dep",
];

impl Elpmpp02Fit {
    /// 论文 Table 3 第 3 列：拟合 LLR（Fortran icor=0），无长期项改正。
    pub fn llr() -> Self {
        Self {
            dw1_0: -0.10525,
            dw2_0: 0.16826,
            dw3_0: -0.10760,
            dw1_1: -0.32311,
            dw2_1: 0.08017,
            dw3_1: -0.04317,
            dw1_2: -0.03794,
            dgam: 0.00069,
            de: 0.00005,
            deart_0: -0.04012,
            deart_1: 0.01442,
            dperi: -0.04854,
            dep: 0.00226,
            secular: [[0.0; 5]; 3],
        }
    }

    /// 论文 Table 3 第 2 列（拟合 DE405，1950—2060）+ Table 6 长期项改正（Fortran icor=1）。
    pub fn de405() -> Self {
        let mut secular = [[0.0; 5]; 3];
        secular[0][3] = -0.00018865;
        secular[0][4] = -0.00001024;
        secular[1][2] = 0.00470602;
        secular[1][3] = -0.00025213;
        secular[2][2] = -0.00261070;
        secular[2][3] = -0.00010712;
        Self {
            dw1_0: -0.07008,
            dw2_0: 0.20794,
            dw3_0: -0.07215,
            dw1_1: -0.35106,
            dw2_1: 0.08017,
            dw3_1: -0.04317,
            dw1_2: -0.03743,
            dgam: 0.00085,
            de: -0.00006,
            deart_0: -0.00033,
            deart_1: 0.00732,
            dperi: -0.00749,
            dep: 0.00224,
            secular,
        }
    }

    /// 本项目按论文两步法拟合 DE431 的常数集（嵌入 `data/fit/elpmpp02-de431.txt`）。
    pub fn de431() -> Self {
        Self::parse(include_str!("../../../../../../data/fit/elpmpp02-de431.txt")).expect("data/fit/elpmpp02-de431.txt")
    }

    /// 本项目按论文两步法拟合 DE441 的常数集（嵌入 `data/fit/elpmpp02-de441.txt`）。
    pub fn de441() -> Self {
        Self::parse(include_str!("../../../../../../data/fit/elpmpp02-de441.txt")).expect("data/fit/elpmpp02-de441.txt")
    }

    fn scalar_mut(&mut self, key: &str) -> Option<&mut f64> {
        Some(match key {
            "dw1_0" => &mut self.dw1_0,
            "dw2_0" => &mut self.dw2_0,
            "dw3_0" => &mut self.dw3_0,
            "dw1_1" => &mut self.dw1_1,
            "dw2_1" => &mut self.dw2_1,
            "dw3_1" => &mut self.dw3_1,
            "dw1_2" => &mut self.dw1_2,
            "dgam" => &mut self.dgam,
            "de" => &mut self.de,
            "deart_0" => &mut self.deart_0,
            "deart_1" => &mut self.deart_1,
            "dperi" => &mut self.dperi,
            "dep" => &mut self.dep,
            _ => return None,
        })
    }

    /// 按名取参数的可变引用：Table 3 名（`dw1_0` 等）或长期项 `sw{i}_{k}`（i = 1..3，k = 2..4）。
    pub fn param_mut(&mut self, key: &str) -> Option<&mut f64> {
        if let Some(rest) = key.strip_prefix("sw") {
            let (i, k) = rest.split_once('_')?;
            let (i, k): (usize, usize) = (i.parse().ok()?, k.parse().ok()?);
            if !(1..=3).contains(&i) || !(2..=4).contains(&k) {
                return None;
            }
            return Some(&mut self.secular[i - 1][k]);
        }
        self.scalar_mut(key)
    }

    /// 按名取参数值，名称同 [Self::param_mut]。
    pub fn param(&self, key: &str) -> Option<f64> {
        self.clone().param_mut(key).map(|v| *v)
    }

    /// 全部参数名：13 个 Table 3 名，再接 9 个长期项名 `sw1_2`…`sw3_4`。
    pub fn param_names() -> Vec<String> {
        let mut v: Vec<String> = SCALAR_KEYS.iter().map(|s| s.to_string()).collect();
        for i in 1..=3 {
            for k in 2..=4 {
                v.push(format!("sw{i}_{k}"));
            }
        }
        v
    }

    /// 解析文本：每行 `名称 值`（空白分隔），`#` 起为注释；名称见 [Self::param_names]。
    /// 未列出的参数取 0（即不改正）；名称未知或重复时报错。
    pub fn parse(text: &str) -> Result<Self, LoadError> {
        let mut fit = Self { secular: [[0.0; 5]; 3], ..Self::zero() };
        let mut seen = std::collections::HashSet::new();
        for (ln, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut it = line.split_whitespace();
            let (Some(key), Some(val), None) = (it.next(), it.next(), it.next()) else {
                return Err(LoadError::Io(format!("ELPMPP02 改正集第 {} 行格式应为「名称 值」：{line}", ln + 1)));
            };
            let v: f64 = val
                .parse()
                .map_err(|_| LoadError::Io(format!("ELPMPP02 改正集第 {} 行数值无法解析：{val}", ln + 1)))?;
            if !seen.insert(key.to_string()) {
                return Err(LoadError::Io(format!("ELPMPP02 改正集参数重复：{key}")));
            }
            *fit
                .param_mut(key)
                .ok_or_else(|| LoadError::Io(format!("ELPMPP02 改正集未知参数：{key}")))? = v;
        }
        Ok(fit)
    }

    /// 写成 [Self::parse] 可读的文本（不含注释头；值为 0 的长期项不写）。
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        for name in Self::param_names() {
            let v = self.param(&name).unwrap();
            if name.starts_with("sw") && v == 0.0 {
                continue;
            }
            s.push_str(&format!("{name:<8} {v:+.10e}\n"));
        }
        s
    }

    /// 全零改正（即论文 Table 1 的名义值）。
    pub fn zero() -> Self {
        Self {
            dw1_0: 0.0,
            dw2_0: 0.0,
            dw3_0: 0.0,
            dw1_1: 0.0,
            dw2_1: 0.0,
            dw3_1: 0.0,
            dw1_2: 0.0,
            dgam: 0.0,
            de: 0.0,
            deart_0: 0.0,
            deart_1: 0.0,
            dperi: 0.0,
            dep: 0.0,
            secular: [[0.0; 5]; 3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trip() {
        let f = Elpmpp02Fit::de405();
        let g = Elpmpp02Fit::parse(&f.to_text()).unwrap();
        assert_eq!(f, g);
    }

    #[test]
    fn parse_rejects_unknown_and_duplicate() {
        assert!(Elpmpp02Fit::parse("dw9_0 1.0").is_err());
        assert!(Elpmpp02Fit::parse("sw1_1 1.0").is_err());
        assert!(Elpmpp02Fit::parse("dgam 1.0\ndgam 2.0").is_err());
        let f = Elpmpp02Fit::parse("# 注释\n dgam 0.5  # 行尾注释\nsw2_3 -1e-4\n").unwrap();
        assert_eq!(f.dgam, 0.5);
        assert_eq!(f.secular[1][3], -1e-4);
    }
}
