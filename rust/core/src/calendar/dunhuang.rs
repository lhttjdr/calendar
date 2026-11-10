//! 历法域第 2 层（聚合/实算层，查表）：敦煌政权朔日表。
//!
//! 吐蕃占领（786—847）与归义军（848—1036）时期，沙州与中原隔绝或往来稀少，当地自造具注历日，朔闰与中原历
//! 常差一二日、闰位亦有不同。本模块解析 `data/calendar/敦煌朔日表.csv`——邓文宽《敦煌天文历法文献辑校》（1996）附录一
//! 比较表敦煌曆一栏有朔之月（由 `scripts/calendar_data/build_dunhuang_shuori.py` 生成），只提供**有据之月**：
//! 朔日、月序、月日数与依据（原件 / 原书据残历推算）。出土历日是实物，属资料可靠性第 1 级。
//!
//! 无据之月怎样补（退回中原历，或只认有据之月）是第 4 层的选择，见 [crate::calendar::lifa::resolve_dunhuang_day]，
//! 不在本模块内。敦煌纪年（吐蕃时期地支纪年、归义军奉中原年号）在第 3 层 [crate::calendar::era_names::dunhuang_year_name]。
//! 说明与逐条检验见 `doc/calendar/dunhuang-calendar.md`。

use super::gregorian::civil_to_jdn;

/// 政权名（与 [crate::calendar::push_calendar::RegimeLunarEntry::region] 同一命名空间）。
pub const DUNHUANG_REGIME: &str = "敦煌";
/// 吐蕃时期起年（沙州陷蕃，通说贞元二年 786；一说建中二年 781）。只决定无据之月是否给出敦煌一行。
pub const DUNHUANG_TUBO_FIRST_YEAR: i32 = 786;
/// 归义军时期起年（大中二年 848 张议潮逐吐蕃）。
pub const DUNHUANG_GUIYIJUN_FIRST_YEAR: i32 = 848;
/// 归义军时期止年（景祐三年 1036 西夏取沙州，通说）。
pub const DUNHUANG_LAST_YEAR: i32 = 1036;

/// 敦煌某农历年所属时期的历名：吐蕃时期敦煌历 / 归义军时期敦煌历；不在敦煌政权年限内为 None。
pub fn dunhuang_calendar_name(lunar_year: i32) -> Option<&'static str> {
    if (DUNHUANG_TUBO_FIRST_YEAR..DUNHUANG_GUIYIJUN_FIRST_YEAR).contains(&lunar_year) {
        Some("吐蕃时期敦煌历")
    } else if (DUNHUANG_GUIYIJUN_FIRST_YEAR..=DUNHUANG_LAST_YEAR).contains(&lunar_year) {
        Some("归义军时期敦煌历")
    } else {
        None
    }
}

/// 朔日依据。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShuoBasis {
    /// 出土历日原件可读。
    Original,
    /// 原书括号格：据残历本身条件推算（同件内推演，仍是该历）。
    Inferred,
}

/// 敦煌有据之月。
#[derive(Clone, Debug)]
pub struct DunhuangMonth {
    /// 朔日 JDN（民用历日期换算）。
    pub jdn: i64,
    /// 敦煌历自身的岁（天文纪年）与月序（建寅）、闰否。
    pub lunar_year: i32,
    pub month: u8,
    pub is_leap: bool,
    /// 吐蕃时期敦煌历 / 归义军时期敦煌历。
    pub calendar: String,
    pub shuo_basis: ShuoBasis,
    /// 月日数（大 30、小 29）；原书「?」为 None。
    pub days: Option<u8>,
    /// 所据文书：(编号, 原书定名, 该件历日本岁)。
    pub documents: Vec<(String, String, i32)>,
}

/// 敦煌政权朔日表（只含有据之月，按朔日排序）。
#[derive(Clone, Debug, Default)]
pub struct DunhuangCalendar {
    months: Vec<DunhuangMonth>,
}

impl DunhuangCalendar {
    /// 仓库内嵌的 `data/calendar/敦煌朔日表.csv`（254 月，数 KB，供 wasm 直接用）。
    pub fn embedded() -> Self {
        Self::from_csv(include_str!("../../../../data/calendar/敦煌朔日表.csv")).expect("敦煌朔日表.csv 解析失败")
    }

    /// 解析 `敦煌朔日表.csv`：前 8 列同 `推步政权朔日表.csv`（regime, calendar, 民用历年月日, lunar_year, lunar_month,
    /// is_leap），其后 朔干支、朔依据、月日数、大小依据、文书、文书纪年、文书岁、书页。`#` 起首为注释。
    pub fn from_csv(csv: &str) -> Result<Self, String> {
        let mut months = Vec::new();
        for (i, line) in csv.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("regime,") {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            let err = || format!("敦煌朔日表 第 {} 行格式不对", i + 1);
            if f.len() < 15 {
                return Err(err());
            }
            let n = |k: usize| f[k].trim().parse::<i32>().map_err(|_| err());
            let jdn = civil_to_jdn(n(2)?, n(3)?, n(4)?);
            let shuo_basis = match f[9] {
                "原件" => ShuoBasis::Original,
                "推算" => ShuoBasis::Inferred,
                _ => return Err(err()),
            };
            let days = match f[10] {
                "" => None,
                s => Some(s.parse::<u8>().map_err(|_| err())?),
            };
            if crate::calendar::gan_zhi::GAN_ZHI_60[(jdn + 49).rem_euclid(60) as usize] != f[8] {
                return Err(format!("敦煌朔日表 第 {} 行：日期与朔干支 {} 不合", i + 1, f[8]));
            }
            let split = |s: &str| s.split('；').map(str::to_string).collect::<Vec<_>>();
            let (ids, titles, years) = (split(f[12]), split(f[13]), split(f[14]));
            let documents = ids
                .into_iter()
                .zip(titles)
                .zip(years)
                .map(|((a, b), c)| c.parse::<i32>().map(|y| (a, b, y)).map_err(|_| err()))
                .collect::<Result<Vec<_>, _>>()?;
            months.push(DunhuangMonth {
                jdn,
                lunar_year: n(5)?,
                month: u8::try_from(n(6)?).map_err(|_| err())?,
                is_leap: f[7] == "1",
                calendar: f[1].to_string(),
                shuo_basis,
                days,
                documents,
            });
        }
        months.sort_by_key(|m| m.jdn);
        Ok(Self { months })
    }

    /// 全部有据之月（按朔日先后）。
    pub fn months(&self) -> &[DunhuangMonth] {
        &self.months
    }

    /// 第 `i` 个有据之月的已知止日（不含）：月日数已知取 朔 + 日数；否则次一有据之月若紧接（相距 29 或 30 日）取其朔；
    /// 都不知为 None（该月只有前 29 日确定）。
    pub fn known_end(&self, i: usize) -> Option<i64> {
        let m = self.months.get(i)?;
        if let Some(d) = m.days {
            return Some(m.jdn + d as i64);
        }
        self.months.get(i + 1).map(|n| n.jdn).filter(|&j| (29..=30).contains(&(j - m.jdn)))
    }

    /// 某敦煌月（岁、月、闰）的有据之月下标与内容。
    pub fn find_month(&self, lunar_year: i32, month: u8, is_leap: bool) -> Option<(usize, &DunhuangMonth)> {
        self.months
            .iter()
            .enumerate()
            .find(|(_, m)| m.lunar_year == lunar_year && m.month == month && m.is_leap == is_leap)
    }

    /// `jdn` 若落在有据之月的确定日内（已知止日之前，止日不知则前 29 日），返回 (下标, 该月, 日序 1..=30)。
    pub fn evidenced_day(&self, jdn: i64) -> Option<(usize, &DunhuangMonth, u8)> {
        let i = self.months.partition_point(|m| m.jdn <= jdn).checked_sub(1)?;
        let m = &self.months[i];
        let end = self.known_end(i).unwrap_or(m.jdn + 29);
        (jdn < end).then(|| (i, m, (jdn - m.jdn + 1) as u8))
    }

    /// 本岁历日文书的原书定名（同一岁可有两件，如 897 年羅四、伯3248 同为一件；924 年伯3555B-14 列次年正月不计）。
    pub fn document_titles(&self, lunar_year: i32) -> Vec<&str> {
        let mut v: Vec<&str> = self
            .months
            .iter()
            .flat_map(|m| m.documents.iter())
            .filter(|d| d.2 == lunar_year)
            .map(|d| d.1.as_str())
            .collect();
        v.dedup();
        v.sort();
        v.dedup();
        v
    }
}

/// 比较表（`data/calendar/敦煌历日_比较表.csv`，邓文宽 1996 附录一转录）敦煌曆一栏有朔之格，供逐条检验：
/// 换算须重现每一格的朔干支与月大小。只收 800 年以后（吐蕃、归义军）；450、451 年为北魏历。
#[derive(Clone, Debug)]
pub struct ComparisonRow {
    pub document: String,
    pub title: String,
    /// 中原曆月序原文（如「閏三月」「上年閏十二月」）。
    pub central_label: String,
    /// 敦煌历自身的岁、月、闰。
    pub lunar_year: i32,
    pub month: u8,
    pub is_leap: bool,
    pub shuo_ganzhi: String,
    pub shuo_basis: ShuoBasis,
    pub days: Option<u8>,
    pub page: String,
}

fn parse_month_label(s: &str) -> Option<(bool, u8, bool)> {
    const N: [&str; 12] = ["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"];
    let (prev, s) = s.strip_prefix("上年").map_or((false, s), |r| (true, r));
    let (leap, s) = s.strip_prefix('閏').map_or((false, s), |r| (true, r));
    let s = s.strip_suffix('月')?;
    let m = N.iter().position(|&x| x == s)? as u8 + 1;
    Some((prev, m, leap))
}

/// 解析比较表（逐格转录原样，括号为原书推算格）。
pub fn parse_comparison_table(csv: &str) -> Result<Vec<ComparisonRow>, String> {
    let mut out = Vec::new();
    for (i, line) in csv.lines().enumerate() {
        if line.starts_with('#') || line.starts_with("文书,") || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        let err = || format!("比较表第 {} 行格式不对", i + 1);
        if f.len() < 12 {
            return Err(err());
        }
        let year: i32 = f[2].parse().map_err(|_| err())?;
        let cell = f[8].trim();
        let gz = cell.trim_start_matches('(').trim_end_matches(')');
        if year < 800 || !crate::calendar::gan_zhi::GAN_ZHI_60.contains(&gz) {
            continue;
        }
        let (prev, m, leap) = parse_month_label(f[3]).ok_or_else(err)?;
        let (month, is_leap) = if f[6].is_empty() {
            if prev {
                return Err(err());
            }
            (m, leap)
        } else {
            let (_, m, l) = parse_month_label(f[6]).ok_or_else(err)?;
            (m, l)
        };
        let days = match f[7].trim_start_matches('(').trim_end_matches(')') {
            "大" => Some(30),
            "小" => Some(29),
            _ => None,
        };
        out.push(ComparisonRow {
            document: f[0].to_string(),
            title: f[1].to_string(),
            central_label: f[3].to_string(),
            lunar_year: year,
            month,
            is_leap,
            shuo_ganzhi: gz.to_string(),
            shuo_basis: if cell.starts_with('(') { ShuoBasis::Inferred } else { ShuoBasis::Original },
            days,
            page: f[11].to_string(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_table_is_consistent() {
        let dh = DunhuangCalendar::embedded();
        let ms = dh.months();
        assert_eq!(ms.len(), 254);
        assert_eq!(ms.iter().filter(|m| m.shuo_basis == ShuoBasis::Original).count(), 144);
        // 月日数与紧接下一有据之月之朔相合（转录自校在表内重做一次）
        for (i, w) in ms.windows(2).enumerate() {
            if let Some(d) = w[0].days {
                let gap = w[1].jdn - w[0].jdn;
                if gap < 40 {
                    assert_eq!(gap, d as i64, "第 {i} 月 {}年{}月 大小与次月朔不合", w[0].lunar_year, w[0].month);
                }
            }
        }
        assert_eq!(dh.document_titles(926), vec!["後唐同光四年丙戌歲"]);
        assert_eq!(dh.document_titles(924), vec!["後唐同光二年甲申歲"]);
    }
}
