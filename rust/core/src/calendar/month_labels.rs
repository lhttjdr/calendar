//! 历法域第 3 层（派生层）：历史月序改制——主表月份 ↔ 当时政权实际使用的月名。
//!
//! 第 2 层主表（[crate::calendar::historical::HistoricalCalendar]，`实历朔日表`）的岁与月一律按建寅（夏正）编号；
//! 而新莽（建丑）、魏景初（建丑，景初元年无三月、景初三年有「后十二月」）、武周（建子：正月、腊月、一月……）、
//! 唐肃宗末年（建子，以「建子月」「建丑月」……斗建为月名）当时用的月名与岁首不同。本模块只做**命名**：
//! 朔日、月长仍取主表，不改第 2 层；与 [crate::calendar::era_names] 同层（依附于历法的纪年、月名派生）。
//!
//! 数据：`data/calendar/month_label_reforms.csv`，由 `scripts/calendar_data/build_month_label_reforms.py`
//! 按史志原文规则生成，只列与建寅标准月名不同的月份。按政权区分：同一年可有并存政权用不同月序
//! （如地皇四年新莽建丑、更始建寅；景初年间吴、蜀仍建寅）。史料出处见 `doc/calendar/historical-calendar.md`「月序改制」。

use std::collections::HashMap;

/// 建寅标准月名（1 → 正月 … 12 → 十二月）。
pub const STANDARD_MONTH_NAMES: [&str; 12] = [
    "正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月",
];

/// 建寅标准月名；`month` 须在 1..=12。
pub fn standard_month_name(month: u8) -> &'static str {
    STANDARD_MONTH_NAMES[(month - 1) as usize]
}

/// 主表中的一个月：岁（建寅）、月序（1..=12）、是否闰月。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TableMonth {
    pub lunar_year: i32,
    pub month: u8,
    pub is_leap: bool,
}

/// 当时实际使用的月名：名义岁（天文纪年，按该政权岁首起算）、月名（「正月」「腊月」「一月」「后十二月」「建子月」等）、闰否。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MonthLabel {
    pub year: i32,
    pub name: String,
    pub is_leap: bool,
}

/// 一条改制月份记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthLabelReform {
    pub regime: String,
    /// 时期名，如「魏景初建丑」。
    pub period: String,
    pub table: TableMonth,
    pub label: MonthLabel,
}

/// 月序改制表：按政权双向查询。
#[derive(Clone, Debug, Default)]
pub struct MonthLabelReforms {
    rows: Vec<MonthLabelReform>,
    by_table: HashMap<(String, TableMonth), usize>,
    by_label: HashMap<(String, MonthLabel), usize>,
}

impl MonthLabelReforms {
    /// 解析 `data/calendar/month_label_reforms.csv`：
    /// regime, period, table_lunar_year, table_lunar_month, table_is_leap, label_year, label_month, label_is_leap。
    pub fn from_csv(csv: &str) -> Result<Self, String> {
        let mut out = Self::default();
        for (i, line) in csv.lines().enumerate().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            if f.len() != 8 {
                return Err(format!("month_label_reforms 第 {} 行：应为 8 列", i + 1));
            }
            let int = |s: &str| s.parse::<i32>().map_err(|e| format!("month_label_reforms 第 {} 行：{e}", i + 1));
            let month = int(f[3])?;
            if !(1..=12).contains(&month) {
                return Err(format!("month_label_reforms 第 {} 行：月序越界", i + 1));
            }
            let row = MonthLabelReform {
                regime: f[0].to_string(),
                period: f[1].to_string(),
                table: TableMonth { lunar_year: int(f[2])?, month: month as u8, is_leap: f[4] == "1" },
                label: MonthLabel { year: int(f[5])?, name: f[6].to_string(), is_leap: f[7] == "1" },
            };
            let idx = out.rows.len();
            if out.by_table.insert((row.regime.clone(), row.table), idx).is_some()
                || out.by_label.insert((row.regime.clone(), row.label.clone()), idx).is_some()
            {
                return Err(format!("month_label_reforms 第 {} 行：重复", i + 1));
            }
            out.rows.push(row);
        }
        Ok(out)
    }

    pub fn rows(&self) -> &[MonthLabelReform] {
        &self.rows
    }

    /// 该政权该月是否改过名（在改制期内且月名与建寅不同）。
    pub fn is_relabeled(&self, regime: &str, table: TableMonth) -> bool {
        self.by_table.contains_key(&(regime.to_string(), table))
    }

    /// 主表月 → 该政权当时的月名；不在改制期内则为建寅标准月名。
    pub fn label_of(&self, regime: &str, table: TableMonth) -> MonthLabel {
        match self.by_table.get(&(regime.to_string(), table)) {
            Some(&i) => self.rows[i].label.clone(),
            None => MonthLabel {
                year: table.lunar_year,
                name: standard_month_name(table.month).to_string(),
                is_leap: table.is_leap,
            },
        }
    }

    /// 当时月名 → 主表月。先查改制表；查不到时按建寅标准月名回退，但若回退到的主表月在该政权已被改名
    /// （如景初元年「三月」、武周「十一月」，当时并无此月名），返回 None。
    /// 不检查主表是否真有该月（如某年无闰），由调用方查主表时判定。
    pub fn table_of(&self, regime: &str, label: &MonthLabel) -> Option<TableMonth> {
        if let Some(&i) = self.by_label.get(&(regime.to_string(), label.clone())) {
            return Some(self.rows[i].table);
        }
        let month = STANDARD_MONTH_NAMES.iter().position(|n| *n == label.name)? as u8 + 1;
        let table = TableMonth { lunar_year: label.year, month, is_leap: label.is_leap };
        (!self.is_relabeled(regime, table)).then_some(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gan_zhi::jd_to_gan_zhi_day;
    use crate::calendar::gregorian::civil_to_julian_day;
    use crate::calendar::historical::HistoricalCalendar;

    fn load() -> (MonthLabelReforms, HistoricalCalendar) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let r = MonthLabelReforms::from_csv(
            &std::fs::read_to_string(root.join("data/calendar/month_label_reforms.csv")).unwrap(),
        )
        .unwrap();
        let h = HistoricalCalendar::from_csv(
            &std::fs::read_to_string(root.join("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap(),
        )
        .unwrap();
        (r, h)
    }

    fn label(year: i32, name: &str, is_leap: bool) -> MonthLabel {
        MonthLabel { year, name: name.to_string(), is_leap }
    }

    /// 主表该月朔日的日干支。
    fn new_moon_ganzhi(h: &HistoricalCalendar, t: TableMonth) -> String {
        let (y, m, d) = h.new_moon_date(t.lunar_year, t.month, t.is_leap).expect("主表无此月");
        jd_to_gan_zhi_day(civil_to_julian_day(y, m, d)).to_string()
    }

    /// 史志所记朔日干支：按当时月名查主表，朔日干支须一致。
    #[test]
    fn historical_new_moon_records() {
        let (r, h) = load();
        let cases = [
            // 《汉书·王莽传上》：以十二月朔癸酉为建国元年正月之朔
            ("新", label(9, "正月", false), "癸酉"),
            // 《汉书·王莽传下》：地皇四年十月戊申朔，兵从宣平城门入
            ("新", label(23, "十月", false), "戊申"),
            // 《旧唐书·肃宗纪》：建辰月庚辰朔；建巳月庚戌朔
            ("唐", label(762, "建辰月", false), "庚辰"),
            ("唐", label(762, "建巳月", false), "庚戌"),
        ];
        for (regime, l, gz) in cases {
            let t = r.table_of(regime, &l).unwrap_or_else(|| panic!("{regime} {l:?} 无对应"));
            assert_eq!(new_moon_ganzhi(&h, t), gz, "{regime} {l:?} → {t:?}");
        }
    }

    #[test]
    fn jingchu_and_wuzhou_labels() {
        let (r, _) = load();
        let t = |y, m, leap| TableMonth { lunar_year: y, month: m, is_leap: leap };
        // 景初：青龙五年三月改为景初元年四月，景初元年无三月；景初三年末为「后十二月」；吴、蜀仍建寅
        assert_eq!(r.label_of("魏", t(237, 3, false)), label(237, "四月", false));
        assert_eq!(r.table_of("魏", &label(237, "三月", false)), None);
        assert_eq!(r.table_of("魏", &label(237, "二月", false)), Some(t(237, 2, false)));
        assert_eq!(r.table_of("魏", &label(238, "正月", false)), Some(t(237, 12, false)));
        assert_eq!(r.table_of("魏", &label(239, "后十二月", false)), Some(t(239, 12, false)));
        assert_eq!(r.table_of("吴", &label(238, "正月", false)), Some(t(238, 1, false)));
        // 武周：载初元年正月 = 永昌元年十一月；腊月、一月；久视元年十月以后复旧
        assert_eq!(r.label_of("武周", t(689, 11, false)), label(690, "正月", false));
        assert_eq!(r.label_of("武周", t(689, 12, false)), label(690, "腊月", false));
        assert_eq!(r.label_of("武周", t(690, 1, false)), label(690, "一月", false));
        assert_eq!(r.label_of("武周", t(695, 2, true)), label(695, "二月", true));
        assert_eq!(r.table_of("武周", &label(691, "十一月", false)), None);
        assert_eq!(r.label_of("武周", t(700, 11, false)), label(700, "十一月", false));
        // 往返一致
        for row in r.rows() {
            assert_eq!(r.table_of(&row.regime, &row.label), Some(row.table));
            assert_eq!(r.label_of(&row.regime, row.table), row.label);
        }
    }
}
