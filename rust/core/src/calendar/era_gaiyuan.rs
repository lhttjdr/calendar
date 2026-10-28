//! 改元事件：按日落年号。数据 `data/calendar/era_gaiyuan_day.csv` 由 `scripts/calendar_data/fill_era_gaiyuan_day.py`
//! 据逐条查证的 `data/calendar/era_gaiyuan_sources.csv`（正史本纪出处 + 原文）填入。
//!
//! 年表 [crate::calendar::era_names::EraNames] 只知道「这一农历年有过改元」。
//! 本表一行一次改元；日期列可空。查某日年号时：有日则按日切换，无日则仍返回前号/后号并列。

use super::era_names::{split_csv_line, unquote, EraEntry};
use std::collections::BTreeMap;

/// 改元日期精度。未补日期时为 [Unknown]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GaiyuanPrecision {
    #[default]
    Unknown,
    Month,
    Day,
}

impl GaiyuanPrecision {
    pub fn as_str(self) -> &'static str {
        match self {
            GaiyuanPrecision::Unknown => "unknown",
            GaiyuanPrecision::Month => "month",
            GaiyuanPrecision::Day => "day",
        }
    }

    fn parse(s: &str) -> Self {
        match s.trim() {
            "day" => GaiyuanPrecision::Day,
            "month" => GaiyuanPrecision::Month,
            _ => GaiyuanPrecision::Unknown,
        }
    }
}

/// 一次改元（同一农历年可多行）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GaiyuanEvent {
    pub lunar_year: i32,
    pub regime_simp: String,
    pub emperor_simp: String,
    pub from_era_name_simp: String,
    pub from_era_year_simp: String,
    pub to_era_name_simp: String,
    pub to_era_year_simp: String,
    pub lunar_month: Option<u8>,
    pub lunar_is_leap: bool,
    pub lunar_day: Option<u8>,
    /// 1582-10-15 前为儒略历，之后为格里高利历（与朔日表相同）。
    pub gregorian: Option<(i32, u8, u8)>,
    pub precision: GaiyuanPrecision,
    pub source: String,
    pub note: String,
}

impl GaiyuanEvent {
    fn has_gregorian(&self) -> bool {
        self.gregorian.is_some()
            && matches!(
                self.precision,
                GaiyuanPrecision::Day | GaiyuanPrecision::Month
            )
    }

    fn has_lunar_day(&self) -> bool {
        self.lunar_month.is_some()
            && self.lunar_day.is_some()
            && self.precision == GaiyuanPrecision::Day
    }

    fn cmp_gregorian(&self, y: i32, m: i32, d: i32) -> Option<std::cmp::Ordering> {
        let (ey, em, ed) = self.gregorian?;
        if !self.has_gregorian() {
            return None;
        }
        Some((ey, em as i32, ed as i32).cmp(&(y, m, d)))
    }

    fn cmp_lunar(&self, month: u8, is_leap: bool, day: u8) -> Option<std::cmp::Ordering> {
        let em = self.lunar_month?;
        let ed = self.lunar_day?;
        if !self.has_lunar_day() {
            return None;
        }
        Some((em, self.lunar_is_leap, ed).cmp(&(month, is_leap, day)))
    }
}

/// 改元日表：按农历年索引。
#[derive(Clone, Debug, Default)]
pub struct GaiyuanDays {
    by_year: BTreeMap<i32, Vec<GaiyuanEvent>>,
}

/// 某日解析出的年号。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EraOnDay {
    pub regime_simp: String,
    pub emperor_simp: String,
    pub era_name_simp: String,
    pub era_year_simp: String,
    pub gaiyuan_prev_era_name_simp: String,
    pub gaiyuan_prev_era_year_simp: String,
    pub has_gaiyuan: bool,
    /// 已按改元日切到单一现行年号。
    pub gaiyuan_resolved: bool,
    /// 查询日即为改元日（新号生效日，含当日）。
    pub gaiyuan_on_this_day: bool,
    pub precision: GaiyuanPrecision,
}

impl EraOnDay {
    fn from_year_entry(e: &EraEntry) -> Self {
        // EraEntry 的年数是整数（[super::era_names::year_to_cn] 换算过一次），本模块内部仍用中文数字文本
        // 与 GaiyuanEvent 的 from_era_year_simp/to_era_year_simp（era_gaiyuan_day.csv 里的原始文本列）混用，
        // 这里转回文本以保持本模块内部统一。
        use super::era_names::year_to_cn;
        Self {
            regime_simp: e.regime_simp.clone(),
            emperor_simp: e.emperor_simp.clone(),
            era_name_simp: e.era_name_simp.clone(),
            era_year_simp: e.era_year.map(year_to_cn).unwrap_or_default(),
            gaiyuan_prev_era_name_simp: e.gaiyuan_prev_era_name_simp.clone(),
            gaiyuan_prev_era_year_simp: e.gaiyuan_prev_era_year.map(year_to_cn).unwrap_or_default(),
            has_gaiyuan: e.has_gaiyuan,
            gaiyuan_resolved: !e.has_gaiyuan,
            gaiyuan_on_this_day: false,
            precision: GaiyuanPrecision::Unknown,
        }
    }
}

impl GaiyuanDays {
    /// 解析 `era_gaiyuan_day.csv`。日期列可空。
    pub fn from_csv(csv: &str) -> Result<Self, String> {
        let mut by_year: BTreeMap<i32, Vec<GaiyuanEvent>> = BTreeMap::new();
        for (line_no, line) in csv.lines().enumerate() {
            let line_no = line_no + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with("lunar_year") {
                continue;
            }
            let event = parse_event_row(line)
                .ok_or_else(|| format!("era_gaiyuan_day line {}: invalid row", line_no))?;
            by_year.entry(event.lunar_year).or_default().push(event);
        }
        Ok(GaiyuanDays { by_year })
    }

    pub fn events_in_year(&self, lunar_year: i32) -> &[GaiyuanEvent] {
        self.by_year
            .get(&lunar_year)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// 按公历日和/或农历月日解析该农历年的现行年号。`gregorian` 须为**民用历**日期（1582-10-15 前儒略历，与改元日表一致），
    /// 前推格里高利历日期先经 [crate::calendar::gregorian::civil_from_julian_day] 换算。
    ///
    /// 该年改元事件按表中先后（即时间先后，`fill_era_gaiyuan_day.py` 保证）依次推进：有日期且不晚于查询日的
    /// 生效（改元当日用新年号），晚于查询日即停；遇到只记到月（或未定）的事件时，按农历月比较：查询月更早则停，
    /// 更晚则该事件必已生效、继续推进，同月（或缺农历日期）无法确定——返回 `gaiyuan_resolved=false`，与该年完全无日期时相同，
    /// 仍带年末年号与年初年号供展示。不能把只记到月的事件当作不存在，否则其后的日子会被错判为旧号
    /// （如 402 年三月壬申复隆安、同月稍后改元大亨）。
    pub fn resolve(
        &self,
        year_entry: &EraEntry,
        lunar_year: i32,
        gregorian: Option<(i32, i32, i32)>,
        lunar_md: Option<(u8, bool, u8)>,
    ) -> EraOnDay {
        let mut out = EraOnDay::from_year_entry(year_entry);
        if !year_entry.has_gaiyuan {
            return out;
        }
        let events = self.events_in_year(lunar_year);
        if events.is_empty() {
            return out;
        }

        let first = &events[0];
        let mut era_name = if !year_entry.gaiyuan_prev_era_name_simp.is_empty() {
            year_entry.gaiyuan_prev_era_name_simp.clone()
        } else {
            first.from_era_name_simp.clone()
        };
        let mut era_year = if let Some(y) = year_entry.gaiyuan_prev_era_year {
            super::era_names::year_to_cn(y)
        } else {
            first.from_era_year_simp.clone()
        };
        let mut on_this_day: Option<&GaiyuanEvent> = None;
        let mut best_prec = GaiyuanPrecision::Unknown;

        for e in events {
            match cmp_query(e, gregorian, lunar_md) {
                Some(std::cmp::Ordering::Greater) => break,
                Some(ord) => {
                    best_prec = e.precision;
                    era_name = e.to_era_name_simp.clone();
                    era_year = e.to_era_year_simp.clone();
                    on_this_day = (ord == std::cmp::Ordering::Equal).then_some(e);
                }
                None => {
                    // 只记到月：查询的农历月早于该月则事件尚未发生，晚于该月则必已生效，同月（或缺农历日期/事件月份）不可定
                    match (e.lunar_month, lunar_md) {
                        (Some(em), Some((qm, ql, _))) if (qm, ql) < (em, e.lunar_is_leap) => break,
                        (Some(em), Some((qm, ql, _))) if (qm, ql) > (em, e.lunar_is_leap) => {
                            best_prec = e.precision;
                            era_name = e.to_era_name_simp.clone();
                            era_year = e.to_era_year_simp.clone();
                            on_this_day = None;
                        }
                        _ => return out,
                    }
                }
            }
        }

        out.gaiyuan_resolved = true;
        out.gaiyuan_on_this_day = on_this_day.is_some();
        out.precision = best_prec;
        out.era_name_simp = era_name;
        out.era_year_simp = era_year;
        match on_this_day {
            Some(e) => {
                out.gaiyuan_prev_era_name_simp = e.from_era_name_simp.clone();
                out.gaiyuan_prev_era_year_simp = e.from_era_year_simp.clone();
            }
            None => {
                out.gaiyuan_prev_era_name_simp.clear();
                out.gaiyuan_prev_era_year_simp.clear();
            }
        }
        out
    }
}

fn cmp_query(
    e: &GaiyuanEvent,
    gregorian: Option<(i32, i32, i32)>,
    lunar_md: Option<(u8, bool, u8)>,
) -> Option<std::cmp::Ordering> {
    if let Some((y, m, d)) = gregorian {
        if let Some(ord) = e.cmp_gregorian(y, m, d) {
            return Some(ord);
        }
    }
    if let Some((month, leap, day)) = lunar_md {
        if let Some(ord) = e.cmp_lunar(month, leap, day) {
            return Some(ord);
        }
    }
    None
}

fn parse_opt_i32(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

fn parse_event_row(line: &str) -> Option<GaiyuanEvent> {
    let mut fields = split_csv_line(line);
    let lunar_year: i32 = fields.next()?.trim().parse().ok()?;
    let regime = unquote(fields.next()?.trim()).to_string();
    let emperor = unquote(fields.next()?.trim()).to_string();
    let from_name = unquote(fields.next()?.trim()).to_string();
    let from_year = unquote(fields.next()?.trim()).to_string();
    let to_name = unquote(fields.next()?.trim()).to_string();
    let to_year = unquote(fields.next()?.trim()).to_string();
    let lunar_month = parse_opt_i32(unquote(fields.next()?.trim())).and_then(|n| u8::try_from(n).ok());
    let leap_raw = unquote(fields.next()?.trim());
    let lunar_is_leap = leap_raw == "1" || leap_raw.eq_ignore_ascii_case("true");
    let lunar_day = parse_opt_i32(unquote(fields.next()?.trim())).and_then(|n| u8::try_from(n).ok());
    let gy = parse_opt_i32(unquote(fields.next()?.trim()));
    let gm = parse_opt_i32(unquote(fields.next()?.trim())).and_then(|n| u8::try_from(n).ok());
    let gd = parse_opt_i32(unquote(fields.next()?.trim())).and_then(|n| u8::try_from(n).ok());
    let gregorian = match (gy, gm, gd) {
        (Some(y), Some(m), Some(d)) => Some((y, m, d)),
        _ => None,
    };
    let precision = GaiyuanPrecision::parse(unquote(fields.next()?.trim()));
    let source = fields
        .next()
        .map(|f| unquote(f.trim()).to_string())
        .unwrap_or_default();
    let note = fields
        .next()
        .map(|f| unquote(f.trim()).to_string())
        .unwrap_or_default();
    Some(GaiyuanEvent {
        lunar_year,
        regime_simp: regime,
        emperor_simp: emperor,
        from_era_name_simp: from_name,
        from_era_year_simp: from_year,
        to_era_name_simp: to_name,
        to_era_year_simp: to_year,
        lunar_month,
        lunar_is_leap,
        lunar_day,
        gregorian,
        precision,
        source,
        note,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn year_entry() -> EraEntry {
        EraEntry {
            is_primary: true,
            has_gaiyuan: true,
            regime_simp: "汉".into(),
            emperor_simp: "昭帝".into(),
            era_name_simp: "元凤".into(),
            era_year: Some(1),
            gaiyuan_prev_era_name_simp: "始元".into(),
            gaiyuan_prev_era_year: Some(7),
            gaiyuan_month: None,
        }
    }

    #[test]
    fn empty_dates_stay_unresolved() {
        let csv = r#"lunar_year,regime_simp,emperor_simp,from_era_name_simp,from_era_year_simp,to_era_name_simp,to_era_year_simp,lunar_month,lunar_is_leap,lunar_day,gregorian_year,gregorian_month,gregorian_day,precision,source,note
-79,汉,昭帝,始元,七年,元凤,元年,,,,,,,unknown,,
"#;
        let days = GaiyuanDays::from_csv(csv).unwrap();
        let r = days.resolve(&year_entry(), -79, Some((-79, 8, 1)), Some((8, false, 1)));
        assert!(!r.gaiyuan_resolved);
        assert_eq!(r.era_name_simp, "元凤");
        assert_eq!(r.gaiyuan_prev_era_name_simp, "始元");
    }

    #[test]
    fn splits_on_gregorian_day_inclusive() {
        let csv = r#"lunar_year,regime_simp,emperor_simp,from_era_name_simp,from_era_year_simp,to_era_name_simp,to_era_year_simp,lunar_month,lunar_is_leap,lunar_day,gregorian_year,gregorian_month,gregorian_day,precision,source,note
-79,汉,昭帝,始元,七年,元凤,元年,,,,-79,8,19,day,test,
"#;
        let days = GaiyuanDays::from_csv(csv).unwrap();
        let e = year_entry();
        let before = days.resolve(&e, -79, Some((-79, 8, 18)), None);
        assert!(before.gaiyuan_resolved);
        assert!(!before.gaiyuan_on_this_day);
        assert_eq!(before.era_name_simp, "始元");
        assert_eq!(before.era_year_simp, "七年");
        assert!(before.gaiyuan_prev_era_name_simp.is_empty());

        let on = days.resolve(&e, -79, Some((-79, 8, 19)), None);
        assert!(on.gaiyuan_on_this_day);
        assert_eq!(on.era_name_simp, "元凤");
        assert_eq!(on.gaiyuan_prev_era_name_simp, "始元");

        let after = days.resolve(&e, -79, Some((-79, 8, 20)), None);
        assert!(!after.gaiyuan_on_this_day);
        assert_eq!(after.era_name_simp, "元凤");
        assert!(after.gaiyuan_prev_era_name_simp.is_empty());
    }

    /// 只记到月的事件不能当作不存在：其所在月及以后无法判定（未定），所在月之前照常判定。
    #[test]
    fn month_precision_event_blocks_later_days() {
        let csv = r#"lunar_year,regime_simp,emperor_simp,from_era_name_simp,from_era_year_simp,to_era_name_simp,to_era_year_simp,lunar_month,lunar_is_leap,lunar_day,gregorian_year,gregorian_month,gregorian_day,precision,source,note
402,晋,安帝,元兴,元年,隆安,六年,3,0,4,402,4,21,day,t,
402,晋,安帝,隆安,六年,大亨,元年,3,0,,,,,month,t,
"#;
        let days = GaiyuanDays::from_csv(csv).unwrap();
        let e = EraEntry {
            is_primary: true,
            has_gaiyuan: true,
            regime_simp: "晋".into(),
            emperor_simp: "安帝".into(),
            era_name_simp: "大亨".into(),
            era_year: Some(1),
            gaiyuan_prev_era_name_simp: "元兴".into(),
            gaiyuan_prev_era_year: Some(1),
            gaiyuan_month: None,
        };
        let feb = days.resolve(&e, 402, Some((402, 3, 20)), Some((2, false, 20)));
        assert!(feb.gaiyuan_resolved);
        assert_eq!(feb.era_name_simp, "元兴");
        // 三月初四复隆安当日可定；其后同月（大亨月份内、日未知）不可定
        let on = days.resolve(&e, 402, Some((402, 4, 21)), Some((3, false, 4)));
        assert!(!on.gaiyuan_resolved, "同月稍后有只记到月的改元，三月内一律未定");
        let mar_greg_only = days.resolve(&e, 402, Some((402, 4, 25)), None);
        assert!(!mar_greg_only.gaiyuan_resolved);
        // 只给公历、且早于第一个有日事件：可定
        let early = days.resolve(&e, 402, Some((402, 4, 1)), None);
        assert!(early.gaiyuan_resolved);
        assert_eq!(early.era_name_simp, "元兴");
    }

    /// 真实数据：sources 逐条查证后填入的 era_gaiyuan_day.csv 能被解析，并与汇总表配合给出正确的按日年号。
    #[test]
    fn real_data_resolves_known_dates() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/calendar");
        let names = super::super::era_names::EraNames::from_csv(
            &std::fs::read_to_string(root.join("汇总/era_names_merged.csv")).unwrap(),
        )
        .unwrap();
        let days = GaiyuanDays::from_csv(&std::fs::read_to_string(root.join("era_gaiyuan_day.csv")).unwrap()).unwrap();
        let at = |y: i32, g: (i32, i32, i32)| days.resolve(names.get_primary(y).unwrap(), y, Some(g), None);

        // 690 年九月九日壬午（公历 690-10-16）改元天授
        let r = at(690, (690, 10, 15));
        assert_eq!((r.era_name_simp.as_str(), r.era_year_simp.as_str()), ("载初", "元年"));
        let r = at(690, (690, 10, 16));
        assert!(r.gaiyuan_on_this_day);
        assert_eq!(r.era_name_simp, "天授");

        // 304 年四次改元：太安 → 永安 → 建武 → 永安 → 永兴
        for (g, era) in [
            ((304, 2, 28), "太安"),
            ((304, 2, 29), "永安"),
            ((304, 9, 10), "建武"),
            ((304, 12, 25), "永安"),
            ((305, 2, 4), "永兴"),
        ] {
            assert_eq!(at(304, g).era_name_simp, era, "{g:?}");
        }

        // 697 年十一月（周正正月）改元圣历，落在夏正 697 年
        assert_eq!(at(697, (697, 12, 19)).era_name_simp, "神功");
        assert_eq!(at(697, (697, 12, 20)).era_name_simp, "圣历");

        // 年号表.csv 未标、据查证补标的年内改元：正月中旬改元（显庆正月初七）、年中即位改元（承圣十一月）、
        // 明代唯一的年内改元（天顺，夺门后正月丙戌）
        for (y, before, on, era_before, era_on) in [
            (656, (656, 2, 6), (656, 2, 7), "永徽", "显庆"),
            (552, (552, 12, 12), (552, 12, 13), "太清", "承圣"),
            (1457, (1457, 2, 14), (1457, 2, 15), "景泰", "天顺"),
        ] {
            assert_eq!(at(y, before).era_name_simp, era_before, "{y} {before:?}");
            assert_eq!(at(y, on).era_name_simp, era_on, "{y} {on:?}");
        }
        // 非元年的年号变动（年号表.csv 未载）：189 年光熹→昭宁→永汉→复中平六年；761 年十一月起去号只称「元年」
        for (g, era, year) in [
            ((189, 5, 14), "中平", "六年"),
            ((189, 5, 15), "光熹", "元年"),
            ((189, 9, 25), "昭宁", "元年"),
            ((189, 9, 28), "永汉", "元年"),
            ((190, 2, 19), "中平", "六年"),
        ] {
            let r = at(189, g);
            assert_eq!((r.era_name_simp.as_str(), r.era_year_simp.as_str()), (era, year), "{g:?}");
        }
        assert_eq!(at(761, (761, 12, 1)).era_name_simp, "上元");
        let r = at(761, (761, 12, 2));
        assert_eq!((r.era_name_simp.as_str(), r.era_year_simp.as_str()), ("", "元年"));
        let r = at(762, (762, 5, 13));
        assert_eq!((r.era_name_simp.as_str(), r.era_year_simp.as_str()), ("", "元年"));
        assert_eq!(at(762, (762, 5, 14)).era_name_simp, "宝应");

        // 前 5 年（天文 -4）六月甲子（初九）改元太初元将、八月废除复称建平（日未载）。实际调用同时给公历与农历：
        // 八月之前、八月之后可定，八月内不可定。
        let atl = |g: (i32, i32, i32), md: (u8, bool, u8)| days.resolve(names.get_primary(-4).unwrap(), -4, Some(g), Some(md));
        assert_eq!(atl((-4, 7, 12), (6, false, 8)).era_name_simp, "建平");
        assert_eq!(atl((-4, 7, 13), (6, false, 9)).era_name_simp, "太初元将");
        assert_eq!(atl((-4, 8, 10), (7, false, 7)).era_name_simp, "太初元将");
        assert!(!atl((-4, 10, 1), (8, false, 30)).gaiyuan_resolved);
        let r = atl((-4, 10, 2), (9, false, 1));
        assert!(r.gaiyuan_resolved);
        assert_eq!((r.era_name_simp.as_str(), r.era_year_simp.as_str()), ("建平", "二年"));

        // 每条记录都有出处；精确到日的都有公历日期
        for evs in (-200..=2000).map(|y| days.events_in_year(y)) {
            for e in evs {
                assert!(!e.source.is_empty(), "{} {} 缺出处", e.lunar_year, e.to_era_name_simp);
                if e.precision == GaiyuanPrecision::Day {
                    assert!(e.gregorian.is_some() && e.lunar_day.is_some());
                }
            }
        }
    }
}
