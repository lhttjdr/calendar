//! 中国星官体系的适用年代（仲裁层：只按年份「选」体系，不做计算）。
//!
//! 数据：`data/sky_culture/system_periods.csv`（手工整理）。规则：各体系自其观测或成书历元起适用，至下一体系历元前一年止；
//! 历元以前不用后出的认星结果，最早的陈卓体系兼作此前各代的兜底。星图「按时期」据此决定星官连线、星名与二十八宿距星。

/// 一个体系的适用年代。
#[derive(Clone, Debug)]
pub struct SystemPeriod {
    /// 体系 id（同距星表 `chinese_lunar_mansion_determinatives.csv` 的 `system`）
    pub system: String,
    /// 起止年（天文纪年，含端点）；None 为不限
    pub from_year: Option<i32>,
    pub to_year: Option<i32>,
    /// 历元说明
    pub epoch: String,
    /// 依据
    pub basis: String,
}

impl SystemPeriod {
    pub fn contains(&self, year: i32) -> bool {
        self.from_year.is_none_or(|a| year >= a) && self.to_year.is_none_or(|b| year <= b)
    }
}

/// 解析 `system_periods.csv`（`#` 开头为注释，首个非注释行为表头）。
pub fn parse_system_periods(lines: &[String]) -> Result<Vec<SystemPeriod>, String> {
    let mut out = vec![];
    for line in lines.iter().filter(|l| !l.starts_with('#')).skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.splitn(5, ',').collect();
        if f.len() != 5 {
            return Err(format!("列数不为 5：{line}"));
        }
        let year = |s: &str| -> Result<Option<i32>, String> {
            if s.is_empty() { Ok(None) } else { s.parse().map(Some).map_err(|e| format!("{e}：{line}")) }
        };
        out.push(SystemPeriod { system: f[0].into(), from_year: year(f[1])?, to_year: year(f[2])?, epoch: f[3].into(), basis: f[4].into() });
    }
    Ok(out)
}

/// 读取仓库内体系年代表（[`crate::repo::paths::CHINESE_SYSTEM_PERIODS`]）。
pub fn load_system_periods_from_repo() -> Result<Vec<SystemPeriod>, String> {
    let lines = crate::repo::read_lines(crate::repo::paths::CHINESE_SYSTEM_PERIODS).map_err(|e| format!("{e:?}"))?;
    parse_system_periods(&lines)
}

/// 某年适用的体系。
pub fn system_for_year(periods: &[SystemPeriod], year: i32) -> Option<&SystemPeriod> {
    periods.iter().find(|p| p.contains(year))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::sky_culture::chinese::load_lunar_mansion_systems_from_repo;

    /// 年代表首尾不限、前后相接无缺无叠，体系 id 都在距星表中；抽查若干年份。
    #[test]
    fn periods_cover_all_years() {
        let p = load_system_periods_from_repo().unwrap();
        assert!(p.first().unwrap().from_year.is_none() && p.last().unwrap().to_year.is_none());
        for w in p.windows(2) {
            assert_eq!(w[0].to_year.unwrap() + 1, w[1].from_year.unwrap(), "{} → {}", w[0].system, w[1].system);
        }
        let ids: Vec<String> = load_lunar_mansion_systems_from_repo().unwrap().into_iter().map(|s| s.id).collect();
        for x in &p {
            assert!(ids.contains(&x.system), "距星表无体系 {}", x.system);
        }
        for (year, want) in [(-180, "chenzhuo"), (1051, "chenzhuo"), (1052, "song_huangyou"), (1363, "yuan"), (1700, "qing_lingtai"), (2026, "qing")] {
            assert_eq!(system_for_year(&p, year).unwrap().system, want, "{year}");
        }
    }
}
