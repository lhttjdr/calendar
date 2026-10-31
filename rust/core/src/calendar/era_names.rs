//! 历法域第 3 层（派生层）：政权·帝王·年号（纪年历）。
//!
//! 依附于第 4 层（仲裁/路由层）路由到的历法（见 [crate::calendar::lifa]），按政权划分；主表对应 id [lifa::LIFA_ID_ZHONGYUAN_SHILI]（各朝据历还原推步的连续表）。
//! 基于跨来源汇总的年号表（`data/calendar/汇总/era_names_merged.csv`）按农历年查询年号与改元标记；**同一年
//! 可能有多个并存政权**（如三国期间魏/蜀/吴），因此按年份索引的是一组 [EraEntry] 而非单条记录。
//! CSV 前 12 列（另有 gaiyuan_earlier_eras、emperor_name 两列在末尾）：lunar_year, regime, emperor, era_name, era_year, is_primary, has_gaiyuan,
//! gaiyuan_prev_era_name, gaiyuan_prev_era_year, gaiyuan_month, source, note。
//! 表结构、汇总方法、已知局限见 `data/calendar/汇总/README.md`。

use std::collections::BTreeMap;

/// 单年号记录：政权、帝号、年号名、年数；改元时含改元前年号名/年数，及可能的改元月份。
///
/// `era_year`/`gaiyuan_prev_era_year` 是**整数**，不是"五年"这样的中文数字文本——中文数字→整数的换算已经
/// 在生成 `era_names_merged.csv` 的 Python 脚本里做过一次，Rust 侧直接拿到干净的数字，不需要也不应该
/// 再存一份文本反复解析；展示成中文用 [EraEntry::era_simp]。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EraEntry {
    /// 是否为该年的"正统主线"记录（对应原单一年号表的等价行为）；同一年可能有多条非主线的并存政权记录。
    pub is_primary: bool,
    pub has_gaiyuan: bool,
    pub regime_simp: String,
    pub emperor_simp: String,
    /// 君主本名（emperor_simp 已带括号本名的行为空）。
    pub emperor_name: String,
    pub era_name_simp: String,
    /// 年号第几年，如「元封五年」的 5。
    pub era_year: Option<u32>,
    pub gaiyuan_prev_era_name_simp: String,
    /// 改元前年号的年数，如「始元七年」的 7。
    pub gaiyuan_prev_era_year: Option<u32>,
    /// 改元/建号所在的农历月份（1-12），仅在该年是某年号第一年时可能有值；来源数据本身不含日期精度。
    pub gaiyuan_month: Option<u8>,
}

impl EraEntry {
    /// 拼接为年号全称（简体），如 "(汉武帝元封五年)"；改元年为 "(汉昭帝始元七年/元凤元年)"。
    /// 展示用君主称呼：帝号 本名，如「玄宗 李隆基」；表中「赧王(姬延)」也写成「赧王 姬延」。
    pub fn emperor_display(&self) -> String {
        let bracketed = self.emperor_simp.strip_suffix(')').and_then(|s| s.split_once('('));
        match (bracketed, self.emperor_simp.is_empty(), self.emperor_name.is_empty()) {
            (Some((title, name)), _, _) => format!("{title} {name}"),
            (None, true, _) => self.emperor_name.clone(),
            (None, false, true) => self.emperor_simp.clone(),
            (None, false, false) => format!("{} {}", self.emperor_simp, self.emperor_name),
        }
    }

    pub fn era_simp(&self) -> String {
        let era_year_cn = self.era_year.map(year_to_cn).unwrap_or_default();
        let main = format!(
            "{}{}{}{}",
            self.regime_simp, self.emperor_simp, self.era_name_simp, era_year_cn
        );
        let s = if self.has_gaiyuan && !self.gaiyuan_prev_era_name_simp.is_empty() {
            match self.gaiyuan_prev_era_year {
                Some(prev_year) => format!(
                    "{}{}{}{}/{}{}",
                    self.regime_simp,
                    self.emperor_simp,
                    self.gaiyuan_prev_era_name_simp,
                    year_to_cn(prev_year),
                    self.era_name_simp,
                    era_year_cn
                ),
                None => main.clone(),
            }
        } else {
            main
        };
        if s.is_empty() {
            s
        } else {
            format!("({})", s)
        }
    }
}

/// 中文数字年份转整数，如 "五年" -> 5，"元年" -> 1，"十一年" -> 11，"二十三年" -> 23。是 [year_to_cn] 的逆运算。
/// `pub`（不只 `pub(crate)`）：`wasm-lib` 是独立 crate，自己解析 vendor 原始的
/// `vendor/calendar/chinese_calendar/年号表.csv`（中文数字文本列），桥接到 [EraEntry] 时需要这个转换。
pub fn parse_cn_era_year(s: &str) -> Option<u32> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s == "元年" || s == "元载" {
        return Some(1);
    }
    let s = s.strip_suffix('年').or_else(|| s.strip_suffix('载')).unwrap_or(s);
    if s.is_empty() {
        return None;
    }
    const DIGITS: &str = "零一二三四五六七八九";
    let digit_val = |c: char| DIGITS.chars().position(|d| d == c).map(|i| i as u32);
    if s == "十" {
        return Some(10);
    }
    if let Some(rest) = s.strip_prefix('十') {
        // "十X" -> 10+X（如 "十一" -> 11）；rest 为空即 "十" 本身已在上面处理。
        let mut chars = rest.chars();
        let ones = chars.next().and_then(digit_val).unwrap_or(0);
        return Some(10 + ones);
    }
    if let Some(idx) = s.find('十') {
        // "X十[Y]" -> X*10+Y（如 "二十三" -> 23，"二十" -> 20）。
        let tens_char = s[..idx].chars().next()?;
        let tens = digit_val(tens_char)?;
        let rest = &s[idx + '十'.len_utf8()..];
        let ones = rest.chars().next().and_then(digit_val).unwrap_or(0);
        return Some(tens * 10 + ones);
    }
    if s.chars().count() == 1 {
        return digit_val(s.chars().next()?);
    }
    s.parse().ok()
}

/// 部分近现代政权以**公历年**为界纪年（如中华民国：N 年 = 公历 1911+N 年，公历 1 月 1 日换年），
/// 与本模块默认的"按农历年分一行、年份随该行走完整个农历年"的编排规则不同——`era_names_merged.csv`
/// 里存的 `era_year` 是按农历年正月初一取值的近似（该农历年多数时间是对的，公历 1 月 1 日到下个农历正月初一
/// 那一小段其实该年份该加一，CSV 行本身不区分）。给定政权名与**具体公历年份**（如果调用方有），
/// 本函数返回该政权在这个公历年应显示的精确纪年数；不属于这类政权则返回 `None`，调用方应回落到
/// CSV 里存的近似值（一般够用，只在农历新年前那一小段公历日期上会偏小 1）。
/// 只在能拿到具体公历日期时才有意义调用，因此调用方是 wasm 的 `get_era_name_parts_for_day`
/// （唯一同时知道农历年与具体公历年月日的查询路径），不影响只按农历年查询的 `get_era_name_parts` 等。
pub fn gregorian_civil_year_era_year(regime_simp: &str, gregorian_year: i32) -> Option<u32> {
    match regime_simp {
        "中华民国" => u32::try_from(gregorian_year - 1911).ok(),
        _ => None,
    }
}

/// 整数年数转中文数字年份，如 5 -> "五年"，11 -> "十一年"，20 -> "二十年"，23 -> "二十三年"。
/// 仅覆盖历史年号实际会用到的范围（1-99，帝王在位一般不超此数）；超出范围原样转阿拉伯数字+"年"兜底。
/// `pub`（不只 `pub(crate)`）：[crate::calendar::era_gaiyuan] 及 `wasm-lib`（独立 crate）都需要把
/// [EraEntry] 的整数年数转回中文文本展示。
pub fn year_to_cn(n: u32) -> String {
    const DIGITS: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    if n == 1 {
        return "元年".to_string();
    }
    if n == 0 || n > 99 {
        return format!("{n}年");
    }
    if n < 10 {
        return format!("{}年", DIGITS[n as usize]);
    }
    if n == 10 {
        return "十年".to_string();
    }
    let tens = n / 10;
    let ones = n % 10;
    let tens_part = if tens == 1 { "十".to_string() } else { format!("{}十", DIGITS[tens as usize]) };
    if ones == 0 {
        format!("{tens_part}年")
    } else {
        format!("{tens_part}{}年", DIGITS[ones as usize])
    }
}

/// 年号表：按农历年索引，一年可能对应多个并存政权的记录。
#[derive(Clone, Debug, Default)]
pub struct EraNames {
    by_year: BTreeMap<i32, Vec<EraEntry>>,
}

impl EraNames {
    /// 从 CSV 文本解析年号表（`data/calendar/汇总/era_names_merged.csv` 格式）。
    pub fn from_csv(csv: &str) -> Result<Self, String> {
        let mut by_year: BTreeMap<i32, Vec<EraEntry>> = BTreeMap::new();
        for (line_no, line) in csv.lines().enumerate() {
            let line_no = line_no + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with("lunar_year") {
                continue;
            }
            let (lunar_year, entry) = parse_era_row(line)
                .ok_or_else(|| format!("era_names line {}: invalid row", line_no))?;
            by_year.entry(lunar_year).or_default().push(entry);
        }
        Ok(EraNames { by_year })
    }

    /// 判断是否包含指定农历年（任意政权）。
    pub fn has_year(&self, lunar_year: i32) -> bool {
        self.by_year.contains_key(&lunar_year)
    }

    /// 获取指定农历年**全部**并存政权的记录；无记录时返回空切片。
    pub fn get_all(&self, lunar_year: i32) -> &[EraEntry] {
        self.by_year.get(&lunar_year).map(Vec::as_slice).unwrap_or(&[])
    }

    /// 获取指定农历年"正统主线"（`is_primary=true`）的记录；无记录时返回 None。
    /// 多数场景（展示单一年号、与旧版单一年号表行为对齐）用这个。
    pub fn get_primary(&self, lunar_year: i32) -> Option<&EraEntry> {
        self.by_year
            .get(&lunar_year)?
            .iter()
            .find(|e| e.is_primary)
    }

    /// 获取指定农历年"正统主线"记录；无记录时返回默认空值（不改元、政权/帝号/年号均为空）。
    pub fn get_primary_or_default(&self, lunar_year: i32) -> EraEntry {
        self.get_primary(lunar_year).cloned().unwrap_or_default()
    }

    /// 覆盖的农历年范围（若为空表则返回 None）。
    pub fn year_range(&self) -> Option<(i32, i32)> {
        let min = *self.by_year.keys().next()?;
        let max = *self.by_year.keys().next_back()?;
        Some((min, max))
    }
}

/// 取一个可能为空的字段，空串返回 None，非空按 `T::from_str` 解析（解析失败也返回 None，不报错——
/// 数据本身的格式错误应该在生成 era_names_merged.csv 时就发现，这里容错即可）。
fn parse_opt_u32(field: Option<&str>) -> Option<u32> {
    let s = unquote(field?.trim());
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

fn parse_era_row(line: &str) -> Option<(i32, EraEntry)> {
    let mut fields = split_csv_line(line);
    let lunar_year: i32 = fields.next()?.trim().parse().ok()?;
    let regime = unquote(fields.next()?.trim()).to_string();
    let emperor = unquote(fields.next()?.trim()).to_string();
    let era_name = unquote(fields.next()?.trim()).to_string();
    let era_year = parse_opt_u32(fields.next());
    let is_primary = unquote(fields.next()?.trim()) == "1";
    let has_gaiyuan = fields.next().map(|f| unquote(f.trim()) == "1").unwrap_or(false);
    let gaiyuan_prev_name = fields.next().map(|f| unquote(f.trim()).to_string()).unwrap_or_default();
    let gaiyuan_prev_era_year = parse_opt_u32(fields.next());
    let gaiyuan_month = parse_opt_u32(fields.next()).and_then(|v| u8::try_from(v).ok());
    // source、note、gaiyuan_earlier_eras 不进结构体，跳过；末列为君主本名。
    let emperor_name = fields.nth(3).map(|f| unquote(f.trim()).to_string()).unwrap_or_default();
    Some((
        lunar_year,
        EraEntry {
            is_primary,
            has_gaiyuan,
            regime_simp: regime,
            emperor_simp: emperor,
            emperor_name,
            era_name_simp: era_name,
            era_year,
            gaiyuan_prev_era_name_simp: gaiyuan_prev_name,
            gaiyuan_prev_era_year,
            gaiyuan_month,
        },
    ))
}

pub(crate) fn unquote(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// 简单 CSV 行分割：逗号分隔，允许双引号内含逗号。
pub(crate) fn split_csv_line(line: &str) -> impl Iterator<Item = &str> {
    let mut in_quote = false;
    let mut start = 0;
    let mut v = Vec::new();
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'"' => in_quote = !in_quote,
            b',' if !in_quote => {
                v.push(&line[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    v.push(&line[start..]);
    v.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_csv() -> &'static str {
        "lunar_year,regime,emperor,era_name,era_year,is_primary,has_gaiyuan,gaiyuan_prev_era_name,gaiyuan_prev_era_year,gaiyuan_month,source,note\n\
         -105,\"汉\",\"武帝\",\"元封\",5,1,0,,,,\"era_names_csv\",\n\
         -79,\"汉\",\"昭帝\",\"元凤\",1,1,1,\"始元\",7,,\"era_names_csv\",\n\
         221,\"魏\",\"文帝\",\"黄初\",2,1,0,,,,\"era_names_csv\",\n\
         221,\"蜀\",\"\",\"章武\",1,0,,,,4,\"fang_shiming:05_三国纪年表\",\n"
    }

    #[test]
    fn gregorian_civil_year_era_year_roc_only() {
        assert_eq!(gregorian_civil_year_era_year("中华民国", 1912), Some(1));
        assert_eq!(gregorian_civil_year_era_year("中华民国", 1949), Some(38));
        assert_eq!(gregorian_civil_year_era_year("中华民国", 1950), Some(39));
        assert_eq!(gregorian_civil_year_era_year("中华民国", 2026), Some(115));
        assert_eq!(gregorian_civil_year_era_year("中华人民共和国", 2026), None);
        assert_eq!(gregorian_civil_year_era_year("汉", 2026), None);
    }

    #[test]
    fn parse_era_csv() {
        let names = EraNames::from_csv(sample_csv()).unwrap();
        assert!(names.has_year(-105));
        let e = names.get_primary(-105).unwrap();
        assert_eq!(e.regime_simp, "汉");
        assert!(e.is_primary);
        assert_eq!(e.era_year, Some(5));
        assert_eq!(e.era_simp(), "(汉武帝元封五年)");
        assert!(!e.has_gaiyuan);

        let g = names.get_primary(-79).unwrap();
        assert!(g.has_gaiyuan);
        assert_eq!(g.era_name_simp, "元凤");
        assert_eq!(g.era_year, Some(1));
        assert_eq!(g.gaiyuan_prev_era_name_simp, "始元");
        assert_eq!(g.gaiyuan_prev_era_year, Some(7));
        assert_eq!(g.era_simp(), "(汉昭帝始元七年/元凤元年)");

        assert!(names.get_primary(999).is_none());
        assert_eq!(names.get_primary_or_default(999).regime_simp, "");
    }

    #[test]
    fn cn_era_year_round_trip() {
        for n in 1..=99u32 {
            let cn = year_to_cn(n);
            assert_eq!(parse_cn_era_year(&cn), Some(n), "round-trip failed for {n} -> {cn:?}");
        }
        assert_eq!(parse_cn_era_year(""), None);
        assert_eq!(parse_cn_era_year("元年"), Some(1));
        assert_eq!(parse_cn_era_year("十年"), Some(10));
    }

    #[test]
    fn year_to_cn_formats() {
        assert_eq!(year_to_cn(1), "元年");
        assert_eq!(year_to_cn(5), "五年");
        assert_eq!(year_to_cn(10), "十年");
        assert_eq!(year_to_cn(11), "十一年");
        assert_eq!(year_to_cn(20), "二十年");
        assert_eq!(year_to_cn(23), "二十三年");
        assert_eq!(year_to_cn(61), "六十一年");
    }

    #[test]
    fn multiple_concurrent_regimes() {
        let names = EraNames::from_csv(sample_csv()).unwrap();
        let all = names.get_all(221);
        assert_eq!(all.len(), 2, "221 年应同时有魏（主线）与蜀（并存）两条记录");
        assert!(all.iter().any(|e| e.regime_simp == "魏" && e.is_primary));
        assert!(all.iter().any(|e| e.regime_simp == "蜀" && !e.is_primary));

        let primary = names.get_primary(221).unwrap();
        assert_eq!(primary.regime_simp, "魏");

        let shu = all.iter().find(|e| e.regime_simp == "蜀").unwrap();
        assert_eq!(shu.era_name_simp, "章武");
        assert_eq!(shu.gaiyuan_month, Some(4));
    }

    #[test]
    fn year_range() {
        let names = EraNames::from_csv(sample_csv()).unwrap();
        assert_eq!(names.year_range(), Some((-105, 221)));
    }

    #[test]
    fn empty_year_has_no_entries() {
        let names = EraNames::from_csv(sample_csv()).unwrap();
        assert!(!names.has_year(1));
        assert!(names.get_all(1).is_empty());
        assert!(names.get_primary(1).is_none());
    }

    /// 用 CARGO_MANIFEST_DIR 拼路径（而不是假设当前工作目录是仓库根），确保真实数据文件本身可解析、
    /// 不因为某次重新生成引入了格式错误。数据本身的正确性由 scripts/calendar_data/build_era_names_merged.py
    /// 与 data/calendar/汇总/校勘调研_年号表_vs_方诗铭.md 负责，这里只测"能不能正常解析、形状是否合理"。
    #[test]
    fn parses_real_merged_file() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/calendar/汇总/era_names_merged.csv");
        let csv = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("读取 {path:?} 失败：{e}；是否忘了先跑 build_era_names_merged.py？"));
        let names = EraNames::from_csv(&csv).unwrap();

        let (min, max) = names.year_range().expect("real data should not be empty");
        assert!(min <= -840, "应覆盖到方诗铭表起始的共和元年（前841年，天文纪年 -840）");
        // 前106年以前主线据方诗铭表，正统按《资治通鉴》体例：周 → 秦（前255年起）→ 汉（前206年起），逐年连续且唯一。
        for y in min..=-105 {
            assert_eq!(names.get_all(y).iter().filter(|e| e.is_primary).count(), 1, "天文纪年 {y} 年主线应恰有一条");
        }
        let p = |y: i32| names.get_primary(y).map(|e| (e.regime_simp.clone(), e.era_name_simp.clone(), e.era_year));
        assert_eq!(p(-840), Some(("周".into(), "共和".into(), Some(1))));
        assert_eq!(p(-255), Some(("周".into(), "".into(), Some(59))), "前256年周赧王五十九年");
        assert_eq!(p(-254), Some(("秦".into(), "".into(), Some(52))), "前255年秦昭襄王五十二年");
        assert_eq!(p(-205), Some(("汉".into(), "".into(), Some(1))), "前206年汉高祖元年");
        assert_eq!(p(-106), Some(("汉".into(), "元封".into(), Some(4))), "与年号表.csv 首年（元封五年）衔接");
        assert_eq!(p(-105), Some(("汉".into(), "元封".into(), Some(5))));
        assert!(max >= 1911, "应覆盖到清末");

        // 三国期间应能查到魏（主线）与蜀、吴（并存）三个政权。
        let sanguo = names.get_all(221);
        let regimes: std::collections::HashSet<&str> =
            sanguo.iter().map(|e| e.regime_simp.as_str()).collect();
        assert!(regimes.contains("魏"), "221 年应有魏");
        assert!(regimes.contains("蜀"), "221 年应有蜀");
        assert_eq!(names.get_primary(221).map(|e| e.regime_simp.as_str()), Some("魏"));

        // 145 年冲帝年号异说：主线应为"永嘉"且 era_year 已是整数而非中文数字文本。
        let e = names.get_primary(145).expect("145 年应有记录");
        assert_eq!(e.era_name_simp, "永嘉");
        assert_eq!(e.era_year, Some(1));

        // 697 年九月改元神功：年号表.csv 上游漏记，汇总表据方诗铭表订正（MANUAL_PRIMARY_FIXES）。
        let e = names.get_primary(697).expect("697 年应有记录");
        assert_eq!(e.era_name_simp, "神功");
        assert_eq!(e.era_year, Some(1));
        assert!(e.has_gaiyuan);
        assert_eq!(e.gaiyuan_prev_era_name_simp, "万岁通天");
        assert_eq!(e.gaiyuan_prev_era_year, Some(2));

        // 帝号首字与朝代简称同形（"元帝""明宗""后主"）时不应残留"帝/宗/后主"前缀。
        for (y, era) in [(260, "景元"), (926, "天成"), (583, "至德"), (1278, "祥兴")] {
            assert_eq!(names.get_primary(y).map(|e| e.era_name_simp.as_str()), Some(era), "{y} 年");
        }

        // 章节交界冲突已全部在 BOUNDARY_RESOLUTIONS 裁决：同一年同一政权只剩一行，且不留"需人工核对"。
        let jingnan: Vec<_> = names.get_all(960).iter().filter(|e| e.regime_simp == "荆南").collect();
        assert_eq!(jingnan.len(), 1, "960 年荆南应只剩年末状态一行");
        assert_eq!(jingnan[0].era_name_simp, "建隆");
        assert!(!csv.contains("需人工核对"), "汇总表有未裁决的章节交界冲突，需在 BOUNDARY_RESOLUTIONS 里核实补上");

        // 并存政权行君主/年号分列：奉正朔政权的括号年号进 era_name，君主进 emperor，「~」按同姓展开。
        let find = |y: i32, regime: &str| {
            names
                .get_all(y)
                .iter()
                .find(|e| e.regime_simp == regime)
                .unwrap_or_else(|| panic!("{y} 年应有 {regime}"))
                .clone()
        };
        let e = find(930, "荆南");
        assert_eq!((e.emperor_simp.as_str(), e.era_name_simp.as_str(), e.era_year), ("高从诲", "长兴", Some(1)));
        let e = find(906, "吴");
        assert_eq!((e.emperor_simp.as_str(), e.era_name_simp.as_str(), e.era_year), ("杨渥", "天祐", Some(3)));
        let e = find(1022, "大理");
        assert_eq!(e.era_name_simp, "明启");
        assert!(!e.emperor_simp.contains('〜') && e.emperor_simp.starts_with('段'), "{:?}", e.emperor_simp);
        // 渤海 774 年括注「宝历」是异说，年数仍续大兴；大彝震之后诸王只有在位纪年，不能沿用咸和。
        assert_eq!(find(774, "渤海").era_name_simp, "大兴");
        let e = find(907, "渤海");
        assert_eq!((e.emperor_simp.as_str(), e.era_name_simp.as_str()), ("大諲譔", ""));
        // 11 章吴越栏只印「(建隆) 1」，君主跨章接自 10 章（钱弘俶）。
        assert_eq!(find(961, "吴越").emperor_simp, "钱弘俶");
    }

    /// era_gaiyuan_day.csv 骨架由 sync_era_gaiyuan_day.py 从汇总表生成，必须与汇总表同源：
    /// [crate::calendar::era_gaiyuan::GaiyuanDays::resolve] 只在主线条目 has_gaiyuan 时查改元日表，
    /// 若某改元年在日表里没有对应行，将来补了日期也无处可填。
    #[test]
    fn gaiyuan_day_skeleton_covers_merged_gaiyuan_years() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/calendar");
        let names =
            EraNames::from_csv(&std::fs::read_to_string(root.join("汇总/era_names_merged.csv")).unwrap()).unwrap();
        let days = crate::calendar::era_gaiyuan::GaiyuanDays::from_csv(
            &std::fs::read_to_string(root.join("era_gaiyuan_day.csv")).unwrap(),
        )
        .unwrap();
        let (min, max) = names.year_range().unwrap();
        let missing: Vec<i32> = (min..=max)
            .filter(|&y| names.get_primary(y).is_some_and(|e| e.has_gaiyuan))
            .filter(|&y| days.events_in_year(y).is_empty())
            .collect();
        assert!(missing.is_empty(), "改元日表缺这些改元年的骨架行，需重跑 sync_era_gaiyuan_day.py：{missing:?}");
    }

    #[test]
    fn emperor_display_joins_title_and_personal_name() {
        let names = EraNames::from_csv(include_str!("../../../../data/calendar/汇总/era_names_merged.csv")).unwrap();
        let name = |y, r: &str| names.get_all(y).iter().find(|e| e.regime_simp == r).unwrap().emperor_display();
        assert_eq!(name(713, "唐"), "玄宗 李隆基");
        assert_eq!(name(-300, "周"), "赧王 姬延");
        assert_eq!(name(1800, "清"), "仁宗 爱新觉罗·颙琰");
        assert_eq!(name(1910, "清"), "爱新觉罗·溥仪");
        assert_eq!(name(2026, "中华人民共和国"), "习近平");
    }
}
