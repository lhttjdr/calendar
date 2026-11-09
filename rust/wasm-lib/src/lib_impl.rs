use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use lunar_core::calendar::era_gaiyuan::GaiyuanDays;
use lunar_core::calendar::era_names::EraEntry;
use lunar_core::math::real::{real, RealOps};
use lunar_core::platform::{DataLoader, LoadError};
use lunar_core::astronomy::ephemeris::{load_all_from_binary_with_fit, load_earth_vsop87_from_repo, Elpmpp02Correction, Elpmpp02Data, Elpmpp02Fit, Elpmpp02Raw, Vsop87};
use lunar_core::astronomy::frame::nutation;

/// Wasm 侧 DataLoader 实现（与 core 的 DataLoaderNative 对称）。path → 文件内容，文本用 files，二进制用 binary（如 fetch 的 .bin）。
struct DataLoaderWasm {
    pub files: HashMap<String, String>,
    pub binary: HashMap<String, Vec<u8>>,
}

impl DataLoaderWasm {
    /// 用文本文件表构造；binary 表为空，可按需再插入。
    fn new(files: HashMap<String, String>) -> Self {
        Self { files, binary: HashMap::new() }
    }
}

impl DataLoader for DataLoaderWasm {
    fn read_lines(&self, path: &str) -> Result<Vec<String>, LoadError> {
        self.files
            .get(path)
            .map(|s| s.lines().map(String::from).collect())
            .ok_or_else(|| LoadError::NotFound(path.to_string()))
    }

    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, LoadError> {
        self.binary
            .get(path)
            .cloned()
            .ok_or_else(|| LoadError::NotFound(path.to_string()))
    }
}

const VSOP87_EAR_PATH: &str = "vendor/vsop87/VSOP87B.ear";
const ELP_BASE: &str = "vendor/elpmpp02";

/// 缓存已解析的 VSOP87，避免日历每格调用时重复解析 300KB+ 文本（节气派干支历）。
static VSOP87_CACHE: Lazy<Mutex<Option<Arc<Vsop87>>>> = Lazy::new(|| Mutex::new(None));

/// 上次岁数据计算时的 repo 辅助数据加载状态（章动表、地球改正），供状态栏展示。
static REPO_AUX_STATUS: Lazy<Mutex<(bool, bool)>> = Lazy::new(|| Mutex::new((false, false)));

/// VSOP87 地球的 DE406/DE441 拟合改正（Chapront 2000 方法，`data/fit/vsop87-<目标>-earth.bin`），由
/// [`init_vsop87_earth_patch_from_binary`] 设置；之后本模块构造的地球 VSOP87 都附上它。
static EARTH_PATCH: Lazy<Mutex<Option<lunar_core::astronomy::ephemeris::vsop87_patch::LbrPatch>>> =
    Lazy::new(|| Mutex::new(None));

/// 给地球 VSOP87 附上已加载的改正（若有）。
fn with_earth_patch(vsop: Vsop87) -> Vsop87 {
    match EARTH_PATCH.lock().unwrap().as_ref() {
        Some(p) => vsop.with_patch(p.clone()),
        None => vsop,
    }
}

fn earth_patch_loaded() -> bool {
    EARTH_PATCH.lock().unwrap().is_some()
}

/// ELPMPP02 常数集：缺省 [`Elpmpp02Correction::DEFAULT`]（拟合 DE431，与 MS2021 ΔT 同一月亮）；前端经
/// [`set_elpmpp02_correction`] 换（如选 DE441 时与 VSOP87 地球的 DE441 改正一起换）。
static ELP_FIT: Lazy<Mutex<(Elpmpp02Fit, Elpmpp02Correction)>> = Lazy::new(|| {
    let c = Elpmpp02Correction::DEFAULT;
    Mutex::new((c.builtin_fit().unwrap(), c))
});

fn elp_fit() -> (Elpmpp02Fit, Elpmpp02Correction) {
    ELP_FIT.lock().unwrap().clone()
}

/// 按当前常数集从文本加载 ELPMPP02（经宿主注入的 loader）。
fn load_elp_text() -> Result<Elpmpp02Data, JsValue> {
    let (fit, corr) = elp_fit();
    let raw = Elpmpp02Raw::load(lunar_core::repo::get_loader().unwrap(), ELP_BASE).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(raw.build(&fit, corr))
}

/// 按当前常数集从 6 个二进制 buffer 加载 ELPMPP02。
fn load_elp_binary(bufs: [&[u8]; 6]) -> Result<Elpmpp02Data, lunar_core::platform::LoadError> {
    let (fit, corr) = elp_fit();
    load_all_from_binary_with_fit(bufs, &fit, corr)
}

/// 设置 ELPMPP02 常数集（"DE431" 缺省、"DE406"、"DE441"，不分大小写）；时间尺度换算（TT↔UT1）的 ΔT 配套随之换。
/// 名称无效或该常数集无实测黄经差（LLR）时返回 false，不变。计算前调用；已建的引擎不受影响。
#[wasm_bindgen]
pub fn set_elpmpp02_correction(name: &str) -> bool {
    let Some(c) = Elpmpp02Correction::from_name(name) else { return false };
    let Some(offset) = c.lunar_offset_vs_de431() else { return false };
    *ELP_FIT.lock().unwrap() = (c.builtin_fit().unwrap(), c);
    lunar_core::astronomy::time::delta_t::set_time_scale_lunar_offset(offset);
    *VSOP87_CACHE.lock().unwrap() = None;
    true
}

/// 当前 ELPMPP02 常数集名（DE406 或 DE441），供状态栏展示。
#[wasm_bindgen]
pub fn get_elpmpp02_correction() -> String {
    format!("{:?}", elp_fit().1)
}

#[wasm_bindgen]
pub fn gregorian_to_jd(year: i32, month: i32, day: i32) -> f64 {
    lunar_core::calendar::gregorian::Gregorian::to_julian_day(year, month, day).as_f64()
}

#[wasm_bindgen]
pub fn jd_to_gregorian(jd: f64) -> JdToGregorianResult {
    let (y, m, d) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real(jd));
    JdToGregorianResult { year: y, month: m, day: d }
}

#[wasm_bindgen]
pub struct JdToGregorianResult {
    pub year: i32,
    pub month: i32,
    pub day: i32,
}

#[wasm_bindgen]
pub fn j2000() -> f64 {
    lunar_core::astronomy::constant::J2000.as_f64()
}

#[wasm_bindgen]
pub fn approximate_new_moon_jd(n: i32) -> f64 {
    lunar_core::astronomy::aspects::approximate_new_moon_jd(n).as_f64()
}

#[wasm_bindgen]
pub struct ChineseLunarResult {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub is_leap_month: bool,
    pub days_in_month: i32,
}

/// 公历 → 农历；岁数据由调用方传入（14 朔 + 12 中气）
#[wasm_bindgen]
pub fn gregorian_to_chinese_lunar(
    year: i32,
    month: i32,
    day: i32,
    lunar_year: i32,
    new_moon_jds: Vec<f64>,
    zhong_qi_jds: Vec<f64>,
) -> Option<ChineseLunarResult> {
    let nm: Vec<_> = new_moon_jds.into_iter().map(real).collect();
    let zq: Vec<_> = zhong_qi_jds.into_iter().map(real).collect();
    let year_data =
        lunar_core::calendar::chinese_lunar::ChineseLunarYearData::new(lunar_year, nm, zq);
    lunar_core::calendar::convert::gregorian_to_chinese_lunar(year, month, day, &year_data).map(|d| {
        ChineseLunarResult {
            year: d.year,
            month: d.month as i32,
            day: d.day as i32,
            is_leap_month: d.is_leap_month,
            days_in_month: d.days_in_month as i32,
        }
    })
}

/// 公历整月逐日农历（结构化）。索引 0 = 1 号。显示/繁简/多语言由前端负责。
#[wasm_bindgen]
pub fn gregorian_month_to_lunar(
    year: i32,
    month: i32,
    lunar_year: i32,
    new_moon_jds: Vec<f64>,
    zhong_qi_jds: Vec<f64>,
) -> MonthLunarResult {
    let nm: Vec<_> = new_moon_jds.into_iter().map(real).collect();
    let zq: Vec<_> = zhong_qi_jds.into_iter().map(real).collect();
    let year_data =
        lunar_core::calendar::chinese_lunar::ChineseLunarYearData::new(lunar_year, nm, zq);
    let arr = lunar_core::calendar::convert::gregorian_month_to_lunar(year, month, &year_data);
    let mut years = Vec::with_capacity(arr.len());
    let mut months = Vec::with_capacity(arr.len());
    let mut days = Vec::with_capacity(arr.len());
    let mut is_leap = Vec::with_capacity(arr.len());
    let mut days_in_lunar_month = Vec::with_capacity(arr.len());
    for o in arr {
        if let Some(d) = o {
            years.push(d.year);
            months.push(d.month as i32);
            days.push(d.day as i32);
            is_leap.push(if d.is_leap_month { 1 } else { 0 });
            days_in_lunar_month.push(d.days_in_month as i32);
        } else {
            years.push(0);
            months.push(0);
            days.push(0);
            is_leap.push(0);
            days_in_lunar_month.push(0);
        }
    }
    MonthLunarResult {
        lunar_years: years,
        lunar_months: months,
        day_of_months: days,
        is_leap_months: is_leap,
        days_in_lunar_months: days_in_lunar_month,
    }
}

#[wasm_bindgen]
pub struct MonthLunarResult {
    lunar_years: Vec<i32>,
    lunar_months: Vec<i32>,
    day_of_months: Vec<i32>,
    is_leap_months: Vec<i32>,
    days_in_lunar_months: Vec<i32>,
}

#[wasm_bindgen]
impl MonthLunarResult {
    #[wasm_bindgen(getter, js_name = lunarYears)]
    pub fn lunar_years(&self) -> Vec<i32> {
        self.lunar_years.clone()
    }
    #[wasm_bindgen(getter, js_name = lunarMonths)]
    pub fn lunar_months(&self) -> Vec<i32> {
        self.lunar_months.clone()
    }
    #[wasm_bindgen(getter, js_name = dayOfMonths)]
    pub fn day_of_months(&self) -> Vec<i32> {
        self.day_of_months.clone()
    }
    #[wasm_bindgen(getter, js_name = isLeapMonths)]
    pub fn is_leap_months(&self) -> Vec<i32> {
        self.is_leap_months.clone()
    }
    #[wasm_bindgen(getter, js_name = daysInLunarMonths)]
    pub fn days_in_lunar_months(&self) -> Vec<i32> {
        self.days_in_lunar_months.clone()
    }
}

/// 调试：返回公历→农历的中间量字符串，便于排查日界问题。
#[wasm_bindgen]
pub fn gregorian_to_chinese_lunar_debug(
    year: i32,
    month: i32,
    day: i32,
    lunar_year: i32,
    new_moon_jds: Vec<f64>,
    zhong_qi_jds: Vec<f64>,
) -> String {
    let nm: Vec<_> = new_moon_jds.into_iter().map(real).collect();
    let zq: Vec<_> = zhong_qi_jds.into_iter().map(real).collect();
    let year_data =
        lunar_core::calendar::chinese_lunar::ChineseLunarYearData::new(lunar_year, nm, zq);
    lunar_core::calendar::chinese_lunar::gregorian_to_chinese_lunar_debug(year, month, day, &year_data)
}

/// 农历 → 公历；岁数据由调用方传入
#[wasm_bindgen]
pub fn chinese_lunar_to_gregorian(
    lunar_year: i32,
    month: i32,
    day: i32,
    is_leap_month: bool,
    data_lunar_year: i32,
    new_moon_jds: Vec<f64>,
    zhong_qi_jds: Vec<f64>,
) -> Option<JdToGregorianResult> {
    let nm: Vec<_> = new_moon_jds.into_iter().map(real).collect();
    let zq: Vec<_> = zhong_qi_jds.into_iter().map(real).collect();
    let year_data =
        lunar_core::calendar::chinese_lunar::ChineseLunarYearData::new(data_lunar_year, nm, zq);
    let date = lunar_core::calendar::chinese_lunar::ChineseLunarDate {
        year: lunar_year,
        month: month as u8,
        day: day as u8,
        is_leap_month,
        days_in_month: 30,
    };
    lunar_core::calendar::convert::chinese_lunar_to_gregorian(date, &year_data).map(|(y, m, d)| {
        JdToGregorianResult {
            year: y,
            month: m,
            day: d,
        }
    })
}

#[wasm_bindgen]
pub fn delta_t_seconds(jd_tt: f64) -> f64 {
    lunar_core::astronomy::time::delta_t(real(jd_tt)).seconds().as_f64()
}

// ---------- 年号：跨来源汇总表 era_names_merged.csv（主线 + 并存政权，见 data/calendar/汇总/README.md）----------
use lunar_core::calendar::era_names::{year_to_cn, EraNames};

static GAIYUAN_DAYS: Lazy<GaiyuanDays> = Lazy::new(|| {
    GaiyuanDays::from_csv(include_str!("../../../data/calendar/era_gaiyuan_day.csv")).unwrap_or_default()
});

static ERA_NAMES: Lazy<EraNames> = Lazy::new(|| {
    EraNames::from_csv(include_str!("../../../data/calendar/汇总/era_names_merged.csv")).unwrap_or_default()
});

/// 主线该农历年行用之历的名称（如「麟德历」），日历显示用，代替笼统的「农历」。
#[wasm_bindgen(js_name = get_calendar_name)]
pub fn get_calendar_name(lunar_year: i32) -> String {
    use lunar_core::calendar::lifa::MainCalendarPeriods;
    thread_local! {
        static PERIODS: Option<MainCalendarPeriods> =
            MainCalendarPeriods::from_csv(include_str!("../../../data/calendar/汇总/main_calendar_periods.csv")).ok();
    }
    // 表解析失败时不能落到「现代推算」，那会给每一年标上错误的历名
    PERIODS.with(|p| p.as_ref().map_or("农历", |p| p.name_for_lunar_year(lunar_year)).to_string())
}

/// 该农历年通行的计时写法（辰刻、时宪刻分秒、24 小时制等）；时制不明的年代返回空串。
#[wasm_bindgen(js_name = format_time_for_lunar_year)]
pub fn format_time_for_lunar_year(lunar_year: i32, hour: u32, minute: u32, second: u32) -> String {
    lunar_core::calendar::timekeeping::format_time_for_lunar_year(lunar_year, hour, minute, second)
}

/// 按农历年查"正统主线"年号（对应旧单一年号表的等价行为）。同一年若有并存政权，见 [get_era_name_parts_all]。
#[wasm_bindgen(js_name = get_era_name)]
pub fn get_era_name(lunar_year: i32, _simp: bool) -> String {
    ERA_NAMES.get_primary(lunar_year).map(|e| e.era_simp()).unwrap_or_default()
}

fn json_era_parts_ext(
    e: &EraEntry,
    gaiyuan_resolved: bool,
    gaiyuan_on_this_day: bool,
    gaiyuan_precision: &str,
) -> String {
    let era_year = e.era_year.map(year_to_cn).unwrap_or_default();
    let gaiyuan_prev_year = e.gaiyuan_prev_era_year.map(year_to_cn).unwrap_or_default();
    let era = format!("{}{}", e.era_name_simp, era_year);
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        r#"{{"regime":"{}","emperor":"{}","era_name":"{}","era_year":"{}","era":"{}","gaiyuan_prev_era_name":"{}","gaiyuan_prev_era_year":"{}","has_gaiyuan":{},"gaiyuan_resolved":{},"gaiyuan_on_this_day":{},"gaiyuan_precision":"{}","is_primary":{}}}"#,
        esc(&e.regime_simp),
        esc(&e.emperor_display()),
        esc(&e.era_name_simp),
        esc(&era_year),
        esc(&era),
        esc(&e.gaiyuan_prev_era_name_simp),
        esc(&gaiyuan_prev_year),
        e.has_gaiyuan,
        gaiyuan_resolved,
        gaiyuan_on_this_day,
        esc(gaiyuan_precision),
        e.is_primary,
    )
}

fn json_era_parts_empty() -> String {
    json_era_parts_ext(&EraEntry::default(), true, false, "unknown")
}

/// 按农历年查"正统主线"年号并拆分为 政权、帝号、年号名、年数；改元时含 gaiyuan_prev_era_name/year。返回 JSON。
#[wasm_bindgen(js_name = get_era_name_parts)]
pub fn get_era_name_parts(lunar_year: i32, _simp: bool) -> String {
    match ERA_NAMES.get_primary(lunar_year) {
        Some(e) => json_era_parts_ext(e, !e.has_gaiyuan, false, "unknown"),
        None => json_era_parts_empty(),
    }
}

/// 按农历年查**全部**并存政权（三国魏蜀吴、五代十国等同一年可能有多条），返回 JSON 数组，
/// 每条形状与 [get_era_name_parts] 相同（含 `is_primary` 标出哪条是"正统主线"）。
#[wasm_bindgen(js_name = get_era_name_parts_all)]
pub fn get_era_name_parts_all(lunar_year: i32) -> String {
    let all = ERA_NAMES.get_all(lunar_year);
    let parts: Vec<String> = all
        .iter()
        .map(|e| json_era_parts_ext(e, !e.has_gaiyuan, false, "unknown"))
        .collect();
    format!("[{}]", parts.join(","))
}

/// 按农历年 + 公历日 + 农历月日解析"正统主线"现行年号。改元日已补则按日切换；未补则前年号/新年号并列。
#[wasm_bindgen(js_name = get_era_name_parts_for_day)]
pub fn get_era_name_parts_for_day(
    lunar_year: i32,
    g_year: i32,
    g_month: i32,
    g_day: i32,
    lunar_month: i32,
    lunar_is_leap: bool,
    lunar_day: i32,
) -> String {
    let Some(entry) = ERA_NAMES.get_primary(lunar_year) else {
        return json_era_parts_empty();
    };
    let lunar_md = u8::try_from(lunar_month).ok().zip(u8::try_from(lunar_day).ok()).map(|(m, d)| (m, lunar_is_leap, d));
    // 前端日期为前推格里高利历；改元日表按民用历（1582-10-15 前儒略历）记，先换成民用历再比较。
    let civil = lunar_core::calendar::gregorian::civil_from_julian_day(
        lunar_core::calendar::gregorian::Gregorian::to_julian_day(g_year, g_month, g_day),
    );
    let on = GAIYUAN_DAYS.resolve(
        entry,
        lunar_year,
        Some(civil),
        lunar_md,
    );
    // GaiyuanDays::resolve 返回的 EraOnDay 用的是中文数字年数文本（跟 era_gaiyuan_day.csv 的列一致），
    // 这里桥接回 json_era_parts_ext 需要的整数年数字段——用 parse_cn_era_year 转一次。
    use lunar_core::calendar::era_names::{gregorian_civil_year_era_year, parse_cn_era_year};
    // 近现代政权（如中华民国）按公历年纪年，CSV 里存的 era_year 是按农历正月初一取的近似值；
    // 这里既然已经有具体公历年份，就用精确公式覆盖，不依赖 era_gaiyuan_day.csv 机制
    // （那套是给"某具体历史事件哪天改元"用的，公历跨年这种规律性换算不需要逐年録入事件表）。
    let era_year = gregorian_civil_year_era_year(&on.regime_simp, g_year)
        .or_else(|| parse_cn_era_year(&on.era_year_simp));
    // 改元日解析可能换了帝号（如新君当年），本名只在帝号未变时沿用
    let emperor_name = if on.emperor_simp == entry.emperor_simp { entry.emperor_name.clone() } else { String::new() };
    let resolved_entry = EraEntry {
        emperor_name,
        is_primary: entry.is_primary,
        has_gaiyuan: on.has_gaiyuan,
        regime_simp: on.regime_simp,
        emperor_simp: on.emperor_simp,
        era_name_simp: on.era_name_simp,
        era_year,
        gaiyuan_prev_era_name_simp: on.gaiyuan_prev_era_name_simp,
        gaiyuan_prev_era_year: parse_cn_era_year(&on.gaiyuan_prev_era_year_simp),
        gaiyuan_month: entry.gaiyuan_month,
    };
    json_era_parts_ext(&resolved_entry, on.gaiyuan_resolved, on.gaiyuan_on_this_day, on.precision.as_str())
}


/// 公历 (y,m,d) 下各推步政权（蜀、吴等）的农历月日，返回 JSON 数组 [{ region, calendar_name, lunar_year, month, is_leap, day }]。主表由前端提供。
#[wasm_bindgen(js_name = get_regimes_lunar_for_day)]
pub fn get_regimes_lunar_for_day(g_year: i32, g_month: i32, g_day: i32) -> String {
    let entries = lunar_core::calendar::push_calendar::gregorian_to_all_regimes_lunar(
        g_year, g_month, g_day,
    );
    let parts: Vec<String> = entries
        .iter()
        .map(|e| {
            let region = e.region.replace('\\', "\\\\").replace('"', "\\\"");
            let cal = e.calendar_name.replace('\\', "\\\\").replace('"', "\\\"");
            format!(
                r#"{{"region":"{}","calendar_name":"{}","lunar_year":{},"month":{},"is_leap":{},"day":{}}}"#,
                region, cal, e.lunar_year, e.month, e.is_leap, e.day
            )
        })
        .collect();
    format!("[{}]", parts.join(","))
}

/// 公历 (年, 月, 日) → 干支日名称（如「庚辰」）
#[wasm_bindgen]
pub fn gregorian_to_gan_zhi_day(year: i32, month: i32, day: i32) -> String {
    lunar_core::calendar::gan_zhi::gregorian_to_gan_zhi_day(year, month, day).to_string()
}

// ---------- 干支历选项：换年/换月/闰月/换日，多套标准 ----------
// JS 传 u8：YearBoundary 0=LiChun 1=LunarNewYear 2=WinterSolstice；MonthBoundary 0=SolarTerm 1=LunarFirstDay；
// LeapMonthHandling 0=Ignore 1=InheritPrevious 2=SplitMidway 3=ShiftToNext；DayBoundary 0=Hour23 1=Hour0

fn day_boundary_from_u8(n: u8) -> lunar_core::calendar::gan_zhi_options::DayBoundary {
    if n == 0 {
        lunar_core::calendar::gan_zhi_options::DayBoundary::Hour23
    } else {
        lunar_core::calendar::gan_zhi_options::DayBoundary::Hour0
    }
}

/// 节气模式：0=定气，1=平气（按行用历）
fn solar_term_mode_from_u8(v: u8) -> lunar_core::calendar::gan_zhi_options::SolarTermMode {
    if v == 1 {
        lunar_core::calendar::gan_zhi_options::SolarTermMode::Pingqi
    } else {
        lunar_core::calendar::gan_zhi_options::SolarTermMode::Dingqi
    }
}

fn options_from_u8s(
    year_b: u8,
    month_b: u8,
    leap_b: u8,
    day_b: u8,
    term_b: u8,
) -> lunar_core::calendar::gan_zhi_options::GanzhiOptions {
    use lunar_core::calendar::gan_zhi_options::{LeapMonthHandling, MonthBoundary, YearBoundary};
    lunar_core::calendar::gan_zhi_options::GanzhiOptions {
        year_boundary: match year_b {
            0 => YearBoundary::LiChun,
            2 => YearBoundary::WinterSolstice,
            _ => YearBoundary::LunarNewYear,
        },
        month_boundary: if month_b == 0 {
            MonthBoundary::SolarTerm
        } else {
            MonthBoundary::LunarFirstDay
        },
        leap_month_handling: match leap_b {
            0 => LeapMonthHandling::Ignore,
            2 => LeapMonthHandling::SplitMidway,
            3 => LeapMonthHandling::ShiftToNext,
            _ => LeapMonthHandling::InheritPrevious,
        },
        day_boundary: day_boundary_from_u8(day_b),
        solar_term_mode: solar_term_mode_from_u8(term_b),
    }
}

/// 按选项的日界：公历 → 干支日（day_boundary: 0=子初23:00, 1=子正00:00）
#[wasm_bindgen]
pub fn gregorian_to_gan_zhi_day_with_options(
    year: i32,
    month: i32,
    day: i32,
    day_boundary: u8,
) -> String {
    let jd = lunar_core::calendar::gregorian::Gregorian::to_julian_day(year, month, day);
    lunar_core::calendar::gan_zhi_options::jd_to_gan_zhi_day_with_options(jd, day_boundary_from_u8(day_boundary))
        .to_string()
}

/// 干支年月日结果（String 非 Copy，用 getter_with_clone 暴露给 JS）
#[wasm_bindgen]
pub struct GanzhiResult {
    year_name: String,
    month_name: String,
    day_name: String,
}

#[wasm_bindgen]
impl GanzhiResult {
    #[wasm_bindgen(getter_with_clone)]
    pub fn year_name(&self) -> String {
        self.year_name.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn month_name(&self) -> String {
        self.month_name.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn day_name(&self) -> String {
        self.day_name.clone()
    }
}

/// 节气派（立春换年、十二节换月）：需 VSOP87。返回 (年干支, 月干支, 日干支) 名称。
/// VSOP87 解析结果在 WASM 内缓存，同一会话中多次调用只解析一次，避免日历每格重复解析 300KB+ 文本。
#[wasm_bindgen]
pub fn ganzhi_from_jd_solar_wasm(
    jd: f64,
    vsop87_ear: &str,
    day_boundary: u8,
    solar_term_mode: u8,
) -> Result<GanzhiResult, JsValue> {
    let vsop = {
        let mut guard = VSOP87_CACHE.lock().unwrap();
        if let Some(cached) = guard.as_ref() {
            Arc::clone(cached)
        } else {
            let mut files = HashMap::new();
            files.insert(VSOP87_EAR_PATH.to_string(), vsop87_ear.to_string());
            let loader = DataLoaderWasm::new(files);
            lunar_core::repo::set_loader(Box::new(loader));
            let parsed = with_earth_patch(load_earth_vsop87_from_repo().map_err(|e| JsValue::from_str(&e.to_string()))?);
            let arc = Arc::new(parsed);
            *guard = Some(Arc::clone(&arc));
            arc
        }
    };
    let opts = lunar_core::calendar::gan_zhi_options::preset_zi_ping_ba_zi();
    let opts = lunar_core::calendar::gan_zhi_options::GanzhiOptions {
        day_boundary: day_boundary_from_u8(day_boundary),
        solar_term_mode: solar_term_mode_from_u8(solar_term_mode),
        ..opts
    };
    let (yi, mi, di) = lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_solar(vsop.as_ref(), jd, &opts);
    Ok(GanzhiResult {
        year_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(yi).to_string(),
        month_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(mi).to_string(),
        day_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(di).to_string(),
    })
}

/// 灌入他处（网页 Worker）算好的节气：`years` 每年对应 `jds` 中 24 项（0 春分 … 23 驚蟄）。返回灌入的年数。
/// 经 JS 传递为 f64（JD 约 40 μs 分辨率），对逐日判定足够；已有缓存的年份不覆盖。
#[wasm_bindgen]
pub fn seed_solar_terms_wasm(years: Vec<i32>, jds: Vec<f64>) -> u32 {
    use lunar_core::astronomy::aspects::seed_solar_terms_cache;
    if jds.len() != years.len() * 24 {
        return 0;
    }
    years
        .iter()
        .zip(jds.chunks(24))
        .filter(|(y, c)| seed_solar_terms_cache(**y, c.iter().map(|&x| real(x)).collect()))
        .count() as u32
}

/// 以二进制（`VSOP87B.ear.bin`，经地球改正）初始化本实例的地球 VSOP87 缓存。其后各函数的 `vsop87_ear` 参数可传空串
/// （主线程不再下载 343 KB 的文本版）。已缓存则不覆盖。返回是否已就绪。
#[wasm_bindgen]
pub fn init_vsop87_earth_from_binary(bytes: &[u8]) -> bool {
    let mut guard = VSOP87_CACHE.lock().unwrap();
    if guard.is_some() {
        return true;
    }
    match Vsop87::from_binary(bytes) {
        Ok(v) => {
            *guard = Some(Arc::new(with_earth_patch(v)));
            true
        }
        Err(_) => false,
    }
}

/// 取缓存的地球 VSOP87；未缓存时用传入文本解析一次。
fn vsop87_cached(vsop87_ear: &str) -> Result<Arc<Vsop87>, JsValue> {
    let mut guard = VSOP87_CACHE.lock().unwrap();
    if let Some(cached) = guard.as_ref() {
        return Ok(Arc::clone(cached));
    }
    let mut files = HashMap::new();
    files.insert(VSOP87_EAR_PATH.to_string(), vsop87_ear.to_string());
    lunar_core::repo::set_loader(Box::new(DataLoaderWasm::new(files)));
    let parsed = with_earth_patch(load_earth_vsop87_from_repo().map_err(|e| JsValue::from_str(&e.to_string()))?);
    let arc = Arc::new(parsed);
    *guard = Some(Arc::clone(&arc));
    Ok(arc)
}

/// 当地太阳时（小时，[0, 24)），供时柱按当地时间划分时辰。
/// jd_ut：UT 儒略日；longitude_deg：东经；mode：1=平太阳时，2=视太阳时（真太阳时）。
#[wasm_bindgen]
pub fn local_solar_hours_wasm(jd_ut: f64, longitude_deg: f64, mode: u8, vsop87_ear: &str) -> Result<f64, JsValue> {
    use lunar_core::astronomy::apparent::{local_apparent_solar_hours, local_mean_solar_hours};
    use lunar_core::quantity::angle::PlaneAngle;
    let lon = PlaneAngle::from_deg(real(longitude_deg));
    let h = if mode == 2 {
        let vsop = vsop87_cached(vsop87_ear)?;
        local_apparent_solar_hours(vsop.as_ref(), real(jd_ut), lon)
    } else {
        local_mean_solar_hours(real(jd_ut), lon)
    };
    Ok(h.as_f64())
}

/// 与定气同名的平气 JD(TT)，按当时行用历推算；qi_from_winter_solstice：冬至 0、小寒 1 …；表外返回 NaN。
#[wasm_bindgen]
pub fn pingqi_tt_wasm(dingqi_tt: f64, qi_from_winter_solstice: i32) -> f64 {
    lunar_core::calendar::pingqi::pingqi_tt(real(dingqi_tt), qi_from_winter_solstice)
        .map(|(_, jd)| jd.as_f64())
        .unwrap_or(f64::NAN)
}

/// 公元 year 年行用的平气历名；表外返回空串。
#[wasm_bindgen]
pub fn pingqi_li_name_wasm(year: i32) -> String {
    lunar_core::calendar::pingqi::pingqi_li_for_year(year)
        .map(|li| li.name.clone())
        .unwrap_or_default()
}

/// 干支名（如「甲子」）的纳音名目；不是合法干支返回空串。
#[wasm_bindgen]
pub fn nayin_of_ganzhi_name(name: &str) -> String {
    lunar_core::calendar::gan_zhi::GAN_ZHI_60
        .iter()
        .position(|g| *g == name)
        .map(|i| lunar_core::folk::nayin::nayin_name(i as u8).to_string())
        .unwrap_or_default()
}

/// 排盘摘要：奇门时家局、大六壬起课、太乙时计、三元九运。`jd_ut` 为所选时刻（UT）；`day_gz`、`hour_zhi` 取页面所示日柱、时支，
/// 与四柱一致（日界、时辰口径由四柱预设定）。各法取缺省：奇门置闰法（演义之界）、节气按当时行用历；六壬按中气换将；
/// 太乙时计冬至后阳遁、夏至后阴遁（以交至时刻为界）；九运每运二十年。出处见 doc/folk/qimen.md、liuren.md、taiyi.md。
#[wasm_bindgen]
pub fn paipan_wasm(jd_ut: f64, vsop87_ear: &str, day_gz: u8, hour_zhi: u8) -> Result<String, JsValue> {
    use lunar_core::astronomy::aspects::solar_term_jds_for_year_cached;
    use lunar_core::calendar::gan_zhi_options::{ganzhi_from_jd_solar, preset_zi_ping_ba_zi};
    use lunar_core::folk::{liuren, qimen, taiyi};
    const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    let vsop = vsop87_cached(vsop87_ear)?;
    let jd_tt = jd_ut + lunar_core::astronomy::time::delta_t(real(jd_ut)).seconds().as_f64() / 86400.0;
    let (day_gz, hour_zhi) = (day_gz % 60, hour_zhi % 12);
    let mut out = Vec::new();
    // 奇门
    let sj = qimen::shi_jia_ju(vsop.as_ref(), real(jd_ut), qimen::DEFAULT_DING_JU, qimen::JieQiSource::XingYong);
    out.push(format!("奇門：{}（置閏法，節氣從行用曆）", sj.describe()));
    // 六壬
    let k = liuren::qi_ke_at(vsop.as_ref(), real(jd_tt), day_gz, hour_zhi, liuren::JiangRule::ZhongQi);
    let ke: Vec<String> = k.si_ke.iter().map(|x| ZHI[x.shang as usize].to_string()).collect();
    out.push(format!(
        "六壬：{}將加{}時；四課上神 {}；三傳 {}{}{}（{}）",
        ZHI[k.pan.jiang as usize],
        ZHI[hour_zhi as usize],
        ke.join(" "),
        ZHI[k.san_chuan[0] as usize],
        ZHI[k.san_chuan[1] as usize],
        ZHI[k.san_chuan[2] as usize],
        k.fa.name()
    ));
    // 太乙时计：冬至后阳遁、夏至后阴遁（取最近已交之至）
    let (gy, _, _) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real(jd_ut));
    let mut zhi_list: Vec<(f64, bool)> = Vec::new();
    for y in [gy - 1, gy] {
        let t = solar_term_jds_for_year_cached(vsop.as_ref(), y);
        zhi_list.push((t[18].as_f64(), true)); // 冬至
        zhi_list.push((t[6].as_f64(), false)); // 夏至
    }
    let yang = zhi_list.iter().filter(|(t, _)| *t <= jd_tt).max_by(|a, b| a.0.partial_cmp(&b.0).unwrap()).map(|x| x.1).unwrap_or(true);
    let dun = if yang { taiyi::Dun::Yang } else { taiyi::Dun::Yin };
    let r = taiyi::shi_ji(day_gz, hour_zhi);
    let j = taiyi::ju(r, dun);
    out.push(format!(
        "太乙：時計第{}局 {}遁，太乙在{}宮，文昌{}，計神{}，始擊{}",
        j.r,
        if yang { "陽" } else { "陰" },
        j.gong,
        taiyi::SHEN16[j.tian_mu],
        taiyi::ji_shen_zhi(&j),
        taiyi::SHEN16[j.shi_ji]
    ));
    // 三元九运：干支年（立春分）
    let (yi, _, _) = ganzhi_from_jd_solar(vsop.as_ref(), jd_ut, &preset_zi_ping_ba_zi());
    let year = if (gy - 4).rem_euclid(60) as u8 == yi { gy } else { gy - 1 };
    let (yuan, yun) = taiyi::san_yuan_jiu_yun(year, taiyi::JiuYun::Even20);
    let yuan = match yuan {
        taiyi::SanYuan::Shang => "上元",
        taiyi::SanYuan::Zhong => "中元",
        taiyi::SanYuan::Xia => "下元",
    };
    out.push(format!("三元九運：{yuan}{}運", ["一", "二", "三", "四", "五", "六", "七", "八", "九"][yun as usize - 1]));
    Ok(out.join("\n"))
}

fn json_arr<I: IntoIterator<Item = String>>(it: I) -> String {
    format!("[{}]", it.into_iter().collect::<Vec<_>>().join(","))
}

/// 立春分年的年干支序（与年九星同）
fn folk_year_gz(vsop: &Vsop87, jd: f64) -> u8 {
    use lunar_core::calendar::gan_zhi_options::{ganzhi_from_jd_solar, preset_zi_ping_ba_zi};
    ganzhi_from_jd_solar(vsop, jd, &preset_zi_ping_ba_zi()).0
}

/// 择日页「年神方位」二十四山图：{"year":"乙酉","suiDe":"庚","suiDeHe":"乙","shan":[{"p":"子","gods":[{"n":"太歲","ji":false}]}…]}，
/// 二十四山自子起顺行；年以立春分；另附当日日游神、鹤神、太岁出游（`jd` 所在民用日）。
#[wasm_bindgen]
pub fn nian_shen_json(jd: f64, vsop87_ear: &str) -> Result<String, JsValue> {
    use lunar_core::folk::nianshen::{by_position, he_shen, nian_shen, ri_you_shen, tai_sui_chu_you};
    let vsop = vsop87_cached(vsop87_ear)?;
    let y = folk_year_gz(vsop.as_ref(), jd);
    let all = nian_shen(y);
    let p = |n: &str| all.iter().find(|s| s.name == n).map(|s| s.positions.join("")).unwrap_or_default();
    let ji = |n: &str| all.iter().find(|s| s.name == n).is_some_and(|s| s.ji);
    let shan = by_position(y).into_iter().map(|(q, v)| {
        format!(
            "{{\"p\":{},\"gods\":{}}}",
            json_str(q),
            json_arr(v.into_iter().map(|n| format!("{{\"n\":{},\"ji\":{}}}", json_str(n), ji(n))))
        )
    });
    let dgz = ((jd + 0.5).floor() as i64 + 49).rem_euclid(60) as u8;
    // 月厌、厌对与厌前厌后（类编「論隂陽不將日」以月厌为界分二十四山；月以节换，正月厌戌逆行）
    let (_, mi, _) = lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_solar(vsop.as_ref(), jd, &lunar_core::calendar::gan_zhi_options::preset_zi_ping_ba_zi());
    let month_ord = ((mi % 12) as i32 - 2).rem_euclid(12) as u8;
    let yan = (10 - month_ord as i32).rem_euclid(12) as usize;
    const SHAN: [&str; 24] = ["子", "癸", "丑", "艮", "寅", "甲", "卯", "乙", "辰", "巽", "巳", "丙", "午", "丁", "未", "坤", "申", "庚", "酉", "辛", "戌", "乾", "亥", "壬"];
    let yp = (yan * 2) as i32;
    let qian: Vec<String> = (1..12).map(|k| json_str(SHAN[(yp - k).rem_euclid(24) as usize])).collect();
    let hou: Vec<String> = (1..12).map(|k| json_str(SHAN[(yp + k).rem_euclid(24) as usize])).collect();
    let day_jiang = match lunar_core::folk::jiaqu::jiang(month_ord, dgz, lunar_core::folk::jiaqu::WuJiJi::LeiBian) {
        lunar_core::folk::jiaqu::Jiang::BuJiang => "陰陽不將",
        lunar_core::folk::jiaqu::Jiang::YangJiang => "陽將",
        lunar_core::folk::jiaqu::Jiang::YinJiang => "陰將",
        lunar_core::folk::jiaqu::Jiang::JuJiang => "陰陽俱將",
        lunar_core::folk::jiaqu::Jiang::Neither => "（日支值厭或厭對）",
    };
    Ok(format!(
        "{{\"year\":{},\"suiDe\":{},\"suiDeHe\":{},\"shan\":{},\"riYou\":{},\"heShen\":{},\"taiSuiYou\":{},\"yueYan\":{},\"yanDui\":{},\"yanQian\":{},\"yanHou\":{},\"dayJiang\":{},\"dayGz\":{}}}",
        json_str(lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(y)),
        json_str(&p("歲德")),
        json_str(&p("歲德合")),
        json_arr(shan),
        json_str(ri_you_shen(dgz)),
        json_str(he_shen(dgz)),
        json_str(tai_sui_chu_you(dgz).unwrap_or("")),
        json_str(SHAN[yp as usize]),
        json_str(SHAN[((yp + 12) % 24) as usize]),
        json_arr(qian),
        json_arr(hou),
        json_str(day_jiang),
        json_str(lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(dgz))
    ))
}

/// 排盘页奇门遁甲：时家定局并布盘。`hour_gz` 时干支序（页面所示时柱）；
/// `method` 0 转盘（缺省）1 飞盘（出处未核）；`ji_gong` 0 寄坤（缺省）1 阳坤阴艮 2 阳巽阴震；`shen_ming` 0 白虎玄武（演义）1 勾陈朱雀（集成）；
/// `ding_ju` 0 置闰·演义之界（缺省）1 置闰·统宗之界 2 拆补；`source` 0 当时行用历（缺省）1 定气。
/// 返回 {"ju":"立冬中元 隂遁9局","xunShou":"甲子","xunYi":"戊","zhiFu":"天蓬","zhiShi":"休門","zhiFuGong":1,"zhiShiGong":1,
/// "gong":[{"di":["戊"],"tian":["…"],"xing":["…"],"men":"休門"|null,"shen":"直符"|null, …逐宫标记}…洛书1—9宫]}。
/// 逐宫标记（`folk::qimen::biao_ji`，doc/folk/qimen.md「逐宫标记」）：`an_gan` 0 值使宫起时干·中五例外（缺省，今人法原典未见）
/// 1 值使宫起不论中五 2 不布；`gan_mu` 0 阳顺阴逆·土墓戌（缺省）1 五行墓·土墓戌 2 五行墓·土墓辰；`ji_xing` 0 只论直符（演义，缺省）
/// 1 天盘诸仪（寶鑑）2 诸仪兼一本；`kong` 0 以支论宫（缺省）1 以仪论宫 2 并取（元靈經）。
/// 另出 "xunKong"/"xu"（时旬孤虚二支）、"kongGong"、"riXunKong"、"ma"/"maGong"（时支驿马）、"jieLuKong"；每宫加 "anGan"、"kong"、"ma"、
/// "menGong"（迫制和義比|null）、"jiXing"、"ganMu"、"xingMu"、"menMu"、"keYing":[{"t","d","n","text"}]、"ge"（本宫格名）
#[wasm_bindgen]
pub fn qimen_json(jd_ut: f64, vsop87_ear: &str, hour_gz: u8, method: u8, ji_gong: u8, shen_ming: u8, ding_ju: u8, source: u8, day_gz: u8, an_gan: u8, gan_mu: u8, ji_xing: u8, kong: u8) -> Result<String, JsValue> {
    use lunar_core::folk::qimen;
    let vsop = vsop87_cached(vsop87_ear)?;
    let dj = match ding_ju {
        1 => qimen::DingJu::ZhiRun(qimen::RunRule::TongZong),
        2 => qimen::DingJu::ChaiBu,
        _ => qimen::DEFAULT_DING_JU,
    };
    let src = if source == 1 { qimen::JieQiSource::DingQi } else { qimen::JieQiSource::XingYong };
    let ju = qimen::shi_jia_ju(vsop.as_ref(), real(jd_ut), dj, src);
    let opts = qimen::PanOptions {
        method: if method == 1 { qimen::PanMethod::FeiPan } else { qimen::PanMethod::ZhuanPan },
        ji_gong: match ji_gong {
            1 => qimen::JiGong::KunGen,
            2 => qimen::JiGong::XunZhen,
            _ => qimen::JiGong::Kun,
        },
        shen_ming: if shen_ming == 1 { qimen::ShenMing::GouChenZhuQue } else { qimen::ShenMing::BaiHuXuanWu },
    };
    let pan = qimen::pai_pan(&ju, hour_gz % 60, opts);
    // 格局（演义卷二）：日干支取页面日柱；年干支按立春；月朔干不给（月格不判）
    let year_gz = folk_year_gz(vsop.as_ref(), jd_ut);
    let gj = qimen::ge_ju(&pan, &qimen::GeJuContext { day_gz: day_gz % 60, year_gz: Some(year_gz), month_shuo_gan: None });
    let geju = json_arr(gj.iter().map(|g| {
        format!(
            "{{\"n\":{},\"gong\":{},\"ji\":{},\"basis\":{},\"note\":{}}}",
            json_str(g.name),
            g.gong.map(|x| x.to_string()).unwrap_or_else(|| "null".into()),
            g.ji,
            json_str(g.basis),
            g.note.map(json_str).unwrap_or_else(|| "null".into())
        )
    }));
    let list = |v: &Vec<&'static str>| json_arr(v.iter().map(|x| json_str(x)));
    let opt = |v: Option<&'static str>| v.map(json_str).unwrap_or_else(|| "null".into());
    // 逐宫标记
    let bo = qimen::BiaoJiOptions {
        kong: match kong {
            1 => qimen::KongRule::YiGong,
            2 => qimen::KongRule::Both,
            _ => qimen::KongRule::ZhiGong,
        },
        an_gan: match an_gan {
            1 => qimen::AnGanRule::ZhiShi,
            2 => qimen::AnGanRule::None,
            _ => qimen::AnGanRule::ZhiShiZhongWu,
        },
        ji_xing: match ji_xing {
            1 => qimen::JiXingRule::LiuYi,
            2 => qimen::JiXingRule::LiuYiYiBen,
            _ => qimen::JiXingRule::ZhiFu,
        },
        gan_mu: if gan_mu == 0 { qimen::GanMuRule::YangShunYinNi } else { qimen::GanMuRule::WuXing },
        tu_mu: if gan_mu == 2 { qimen::TuMu::Chen } else { qimen::TuMu::Xu },
    };
    let bj = qimen::biao_ji(&pan, &qimen::GeJuContext { day_gz: day_gz % 60, year_gz: Some(year_gz), month_shuo_gan: None }, bo);
    let zhi = |z: u8| json_str(qimen::ZHI[(z % 12) as usize]);
    let gong = json_arr(pan.gong.iter().zip(bj.gong.iter()).map(|(c, b)| {
        let ky = json_arr(b.ke_ying.iter().map(|k| format!("{{\"t\":{},\"d\":{},\"n\":{},\"text\":{}}}", json_str(k.tian), json_str(k.di), json_str(k.name), json_str(k.text))));
        format!(
            "{{\"di\":{},\"tian\":{},\"xing\":{},\"men\":{},\"shen\":{},\"anGan\":{},\"kong\":{},\"ma\":{},\"menGong\":{},\"jiXing\":{},\"ganMu\":{},\"xingMu\":{},\"menMu\":{},\"keYing\":{},\"ge\":{}}}",
            list(&c.di_pan),
            list(&c.tian_pan),
            list(&c.xing),
            opt(c.men),
            opt(c.shen),
            if b.an_gan.is_empty() { "null".into() } else { json_str(b.an_gan) },
            b.kong,
            b.ma,
            opt(b.men_gong),
            list(&b.ji_xing),
            list(&b.gan_mu),
            list(&b.xing_mu),
            b.men_mu,
            ky,
            list(&b.ge)
        )
    }));
    Ok(format!(
        "{{\"ju\":{},\"xunShou\":{},\"xunYi\":{},\"zhiFu\":{},\"zhiShi\":{},\"zhiFuGong\":{},\"zhiShiGong\":{},\"gong\":{},\"geju\":{},\"xunKong\":{},\"xu\":{},\"kongGong\":{},\"riXunKong\":{},\"ma\":{},\"maGong\":{},\"jieLuKong\":{}}}",
        json_str(&pan.ju.describe()),
        json_str(pan.xun_shou),
        json_str(pan.xun_yi),
        json_str(pan.zhi_fu),
        json_str(pan.zhi_shi),
        pan.zhi_fu_gong,
        pan.zhi_shi_gong,
        gong,
        geju,
        json_arr(bj.xun_kong.iter().map(|&z| zhi(z))),
        json_arr(bj.gu_xu.iter().map(|&z| zhi(z))),
        json_arr(bj.kong_gong.iter().map(|g| g.to_string())),
        json_arr(bj.ri_xun_kong.iter().map(|&z| zhi(z))),
        zhi(bj.ma_zhi),
        bj.ma_gong,
        bj.jie_lu_kong
    ))
}

/// 排盘页紫微斗数（十四主星派，《紫微斗數全書》）。输入生年干支序（农历正月初一换年）、农历月日（含闰）、时辰支序、性别；
/// 选项：`leap` 0 闰月作下月（缺省）1 作本月 2 十五日为界；`xin` 0 辛年魁午钺寅（缺省）1 魁寅钺午；`bing_ding` 0 丙丁钺酉（缺省）1 钺戌；
/// `si_hua` 0 全书表（缺省）1 近人表；`chang_sheng` 0 男顺女逆（缺省）1 阳男阴女顺。
/// 返回 {"palaces":[地支子…亥之宫名],"gan":[宫干],"stars":[[星,支]…],"siHua":[[化,星]…],"ming":支,"shen":支,"ju":数,"juName":"水二局",
/// "mingZhu":"…","shenZhu":"…","daXian":[[起岁,止岁,支]…],"changSheng":[…],"boShi":[…]}
#[wasm_bindgen]
pub fn ziwei_json(
    year_gz: u8,
    lunar_month: u8,
    is_leap: bool,
    lunar_day: u8,
    hour_zhi: u8,
    male: bool,
    opt_leap: u8,
    opt_xin: u8,
    opt_bing_ding: u8,
    opt_si_hua: u8,
    opt_chang_sheng: u8,
) -> String {
    use lunar_core::folk::ziwei::{chart, BingDingYue, ChangShengDir, LeapRule, SiHuaTable, XinKuiYue, ZiweiInput, ZiweiOptions};
    const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
    let o = ZiweiOptions {
        leap: match opt_leap {
            1 => LeapRule::SameMonth,
            2 => LeapRule::SplitAt15,
            _ => LeapRule::NextMonth,
        },
        xin_kui_yue: if opt_xin == 1 { XinKuiYue::KuiYinYueWu } else { XinKuiYue::KuiWuYueYin },
        bing_ding_yue: if opt_bing_ding == 1 { BingDingYue::Xu } else { BingDingYue::You },
        si_hua: if opt_si_hua == 1 { SiHuaTable::Modern } else { SiHuaTable::QuanShu },
        chang_sheng: if opt_chang_sheng == 1 { ChangShengDir::YangMaleYinFemaleForward } else { ChangShengDir::MaleForward },
    };
    let inp = ZiweiInput { year_gz: year_gz % 60, lunar_month, is_leap_month: is_leap, lunar_day, hour_zhi: hour_zhi % 12, male };
    let c = chart(&inp, &o);
    let strs = |v: &[&'static str]| json_arr(v.iter().map(|x| json_str(x)));
    format!(
        "{{\"palaces\":{},\"gan\":{},\"stars\":{},\"siHua\":{},\"ming\":{},\"shen\":{},\"ju\":{},\"juName\":{},\"mingZhu\":{},\"shenZhu\":{},\"daXian\":{},\"changSheng\":{},\"boShi\":{}}}",
        strs(&c.palaces),
        json_arr(c.palace_gan.iter().map(|g| json_str(GAN[*g as usize % 10]))),
        json_arr(c.stars.iter().map(|st| format!(
            "[{},{},{}]",
            json_str(st.name),
            st.zhi,
            lunar_core::folk::ziwei::miao_wang(st.name, st.zhi).map(|m| json_str(m.name())).unwrap_or_else(|| "null".into())
        ))),
        json_arr(c.si_hua.iter().map(|(h, st)| format!("[{},{}]", json_str(h), json_str(st)))),
        c.ming,
        c.shen,
        c.ju,
        json_str(c.ju_name),
        json_str(c.ming_zhu),
        json_str(c.shen_zhu),
        json_arr(c.da_xian.iter().map(|(a, b, z)| format!("[{a},{b},{z}]"))),
        strs(&c.chang_sheng),
        strs(&c.bo_shi)
    )
}

/// 排盘页紫微斗数十八飞星派（续道藏本，明万历刻本校）：缺省照原书。返回 {"palaces":[…],"stars":[[星,支]…],"ming":支,"shen":支,
/// "daXian":[[起岁,止岁,支]…],"liuNian":[…]}
#[wasm_bindgen]
pub fn ziwei18_json(year_gz: u8, lunar_month: u8, is_leap: bool, lunar_day: u8, hour_zhi: u8, male: bool) -> String {
    use lunar_core::folk::ziwei18::{chart, Ziwei18Input, Ziwei18Options};
    let c = chart(
        &Ziwei18Input { year_gz: year_gz % 60, lunar_month, is_leap_month: is_leap, lunar_day, hour_zhi: hour_zhi % 12, male },
        &Ziwei18Options::default(),
    );
    format!(
        "{{\"palaces\":{},\"stars\":{},\"ming\":{},\"shen\":{},\"daXian\":{},\"liuNian\":{}}}",
        json_arr(c.palaces.iter().map(|x| json_str(x))),
        json_arr(c.stars.iter().map(|st| format!("[{},{}]", json_str(st.name), st.zhi))),
        c.ming,
        c.shen,
        json_arr(c.da_xian.iter().map(|(a, b, z)| format!("[{a},{b},{z}]"))),
        json_arr(c.liu_nian.iter().map(|x| json_str(x)))
    )
}

/// 择日页婚嫁择吉：自 `jd0` 所在民用日起连续 `months.len()` 日逐日判定（协纪为纲、类编卷七并列，见 doc/folk/jiaqu.md）。
/// 每日农历（月、日、闰、月大小）与朔弦望（0 朔 1 上弦 2 望 3 下弦，−1 无）由调用方按页面所示农历给出（与日历同源）。
/// 女命生年干支序按立春分；`opts` 位：1 用女命行嫁月否决、2 周堂用类编、4 用类编凶神否决、8 用制化、16 白虎从鳌头（值门床堂路皆禳）、
/// 32 协纪月表只避忌嫁娶、64 不用协纪月表、128 不用嫁娶周堂。可用与否全依所选规则。
/// 返回逐日 JSON 数组；`verdict` 为「可用」「經制化可用」「不可用」，`jie` 为经制化之忌与制法出处。
#[wasm_bindgen]
pub fn jiaqu_days_json(
    jd0: f64,
    vsop87_ear: &str,
    months: Vec<u8>,
    days: Vec<u8>,
    leaps: Vec<u8>,
    dims: Vec<u8>,
    phases: Vec<i8>,
    nv_year_gz: u8,
    has_weng_gu: bool,
    has_nv_fu_mu: bool,
    na_xu: bool,
    opts: u8,
) -> Result<String, JsValue> {
    use lunar_core::calendar::chinese_lunar::ChineseLunarDate;
    use lunar_core::folk::jiaqu::{judge_day, Jiang, JiaquInput, JiaquOptions, ZhouTangRule};
    use lunar_core::folk::puzhu::{QiWangWangCount, TermTable};
    let vsop = vsop87_cached(vsop87_ear)?;
    let j0 = (jd0 + 0.5).floor() as i64;
    let n = months.len().min(days.len()).min(leaps.len()).min(dims.len()).min(phases.len());
    let (gy0, _, _) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real(j0 as f64));
    let (gy1, _, _) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real((j0 + n as i64) as f64));
    let table = TermTable::new(vsop.as_ref(), gy0, gy1);
    let inp = JiaquInput { nv_year_gz: nv_year_gz % 60, has_weng_gu, has_nv_fu_mu, na_xu };
    let o = JiaquOptions {
        xieji_yiji: if opts & 64 != 0 {
            lunar_core::folk::jiaqu::XieJiYiJi::BuYong
        } else if opts & 32 != 0 {
            lunar_core::folk::jiaqu::XieJiYiJi::BiJi
        } else {
            lunar_core::folk::jiaqu::XieJiYiJi::XuYi
        },
        nv_ming_yue: opts & 1 != 0,
        zhou_tang: if opts & 128 != 0 {
            ZhouTangRule::BuYong
        } else if opts & 2 != 0 {
            ZhouTangRule::LeiBian
        } else {
            ZhouTangRule::XieJi
        },
        leibian_xiong: opts & 4 != 0,
        zhi_hua: opts & 8 != 0,
        bai_hu: if opts & 16 != 0 { lunar_core::folk::jiaqu::BaiHuRule::AoTou } else { lunar_core::folk::jiaqu::BaiHuRule::LeiBian },
        ..JiaquOptions::default()
    };
    let strs = |v: &[String]| json_arr(v.iter().map(|x| json_str(x)));
    let opt = |v: Option<&'static str>| v.map(json_str).unwrap_or_else(|| "null".into());
    let out = (0..n).map(|i| {
        let jdn = j0 + i as i64;
        let (gy, _, _) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real(jdn as f64));
        let lunar = ChineseLunarDate { year: gy, month: months[i], day: days[i], is_leap_month: leaps[i] != 0, days_in_month: dims[i] };
        let ph = match phases[i] {
            0 => Some("朔"),
            1 => Some("上弦"),
            2 => Some("望"),
            3 => Some("下弦"),
            _ => None,
        };
        let d = table.day(jdn, lunar, ph, QiWangWangCount::Inclusive);
        let r = judge_day(&d, &inp, &o);
        let jiang = match r.jiang {
            Jiang::BuJiang => "陰陽不將",
            Jiang::YangJiang => "陽將",
            Jiang::YinJiang => "陰將",
            Jiang::JuJiang => "陰陽俱將",
            Jiang::Neither => "",
        };
        format!(
            "{{\"jdn\":{},\"ok\":{},\"verdict\":{},\"jie\":{},\"yi\":{},\"ji\":{},\"jieHunYin\":{},\"naCai\":{},\"nvMingYue\":{},\"zhouTang\":{},\"baiHu\":{},\"naXu\":{},\"buJiang\":{},\"jiang\":{},\"leibian\":{},\"reasons\":{},\"notes\":{}}}",
            jdn,
            r.usable(),
            json_str(match r.verdict {
                lunar_core::folk::jiaqu::Verdict::KeYong => "可用",
                lunar_core::folk::jiaqu::Verdict::ZhiHuaKeYong => "經制化可用",
                lunar_core::folk::jiaqu::Verdict::BuKeYong => "不可用",
            }),
            json_arr(r.jie.iter().map(|(a, b)| format!("[{},{}]", json_str(a), json_str(b)))),
            r.xieji_yi,
            r.xieji_ji,
            r.yi_jie_hun_yin,
            r.yi_na_cai,
            json_str(r.nv_ming_yue.name()),
            opt(r.zhou_tang),
            opt(r.xing_jia_bai_hu),
            opt(r.na_xu_zhou_tang),
            r.bu_jiang_xieji,
            json_str(jiang),
            json_arr(r.leibian_xiong.iter().map(|(a, b)| format!("[{},{}]", json_str(a), json_str(b)))),
            strs(&r.reasons),
            strs(&r.notes)
        )
    });
    Ok(json_arr(out))
}

/// 排盘页筮法：六爻纳甲装卦（京房八宫）与《周易》考变占取辞（朱熹《易學啟蒙》）。`values` 初至上六爻之数（6/7/8/9）；
/// 月建支序（以节换月）、日干支序取页面四柱；`gua_shen` 0 火珠林卦身（缺省）1 不用；`san_bian` 0 初爻在变主贞（缺省，推定未核）1 贞悔并列。
#[wasm_bindgen]
pub fn liuyao_json(values: Vec<u8>, yue_zhi: u8, day_gz: u8, gua_shen: u8, san_bian: u8) -> String {
    use lunar_core::folk::{liuyao as ly, zhouyi as zy};
    let mut v = [7u8; 6];
    for (i, x) in values.iter().take(6).enumerate() {
        v[i] = *x;
    }
    let p = ly::pai_gua(
        v,
        Some(yue_zhi % 12),
        Some(day_gz % 60),
        ly::LiuyaoOptions { gua_shen: if gua_shen == 1 { ly::GuaShenRule::None } else { ly::GuaShenRule::HuoZhuLin }, ..Default::default() },
    );
    let yao = json_arr(p.yao.iter().map(|y| {
        format!(
            "{{\"yang\":{},\"dong\":{},\"gz\":{},\"wx\":{},\"qin\":{},\"shen\":{},\"fu\":{},\"bian\":{}}}",
            y.yang,
            y.dong,
            json_str(&format!("{}{}", ly::GAN[y.gan as usize], ly::ZHI[y.zhi as usize])),
            json_str(ly::WU_XING[y.wx as usize]),
            json_str(ly::LIU_QIN[y.qin as usize]),
            y.shen.map(|k| json_str(ly::LIU_SHEN[k as usize])).unwrap_or_else(|| "null".into()),
            y.fu.map(|(q, g, z)| json_str(&format!("{}{}{}", ly::LIU_QIN[q as usize], ly::GAN[g as usize], ly::ZHI[z as usize]))).unwrap_or_else(|| "null".into()),
            y.bian.map(|(g, z, w, q)| json_str(&format!("{}{}{} {}", ly::GAN[g as usize], ly::ZHI[z as usize], ly::WU_XING[w as usize], ly::LIU_QIN[q as usize]))).unwrap_or_else(|| "null".into())
        )
    }));
    let zc = zy::kao_bian_zhan(v, zy::ZhanOptions { san_bian: if san_bian == 1 { zy::SanBianZhu::Both } else { zy::SanBianZhu::ChuYaoZaiBian } });
    let readings = json_arr(zc.readings.iter().map(|r| {
        let ci = match r.ci {
            zy::Ci::GuaCi => "卦辭".to_string(),
            zy::Ci::Yao(k) => format!("第{k}爻辭"),
            zy::Ci::Yong => "用辭".to_string(),
        };
        let role = match r.role {
            zy::Role::Zhu => "主",
            zy::Role::Bing => "並",
            zy::Role::Zhen => "貞",
            zy::Role::Hui => "悔",
        };
        format!("{{\"gua\":{},\"ci\":{},\"role\":{},\"text\":{}}}", json_str(zy::name(r.gua)), json_str(&ci), json_str(role), json_str(r.text))
    }));
    format!(
        "{{\"ben\":{},\"zhi\":{},\"gong\":{},\"gongWx\":{},\"shi\":{},\"ying\":{},\"xunKong\":{},\"guaShen\":{},\"yao\":{},\"rule\":{},\"readings\":{}}}",
        json_str(p.ben_name),
        json_str(p.zhi_name),
        json_str(p.gong_name),
        json_str(ly::WU_XING[p.gong_wx as usize]),
        p.wei.shi,
        p.wei.ying,
        p.xun_kong.map(|k| json_str(&format!("{}{}", ly::ZHI[k[0] as usize], ly::ZHI[k[1] as usize]))).unwrap_or_else(|| "null".into()),
        p.gua_shen.map(|z| json_str(ly::ZHI[z as usize])).unwrap_or_else(|| "null".into()),
        yao,
        json_str(zc.rule),
        readings
    )
}

/// 起卦：梅花易数年月日时起卦（年支、农历月日、时支），返回初至上六爻之数
#[wasm_bindgen]
pub fn meihua_time_values(year_zhi: u8, lunar_month: u8, lunar_day: u8, hour_zhi: u8) -> Vec<u8> {
    use lunar_core::folk::liuyao as ly;
    let (u, l, d) = ly::meihua_time(year_zhi % 12, lunar_month, lunar_day, hour_zhi % 12);
    ly::meihua_values(u, l, d).to_vec()
}

/// 起卦：大衍揲蓍一爻（三变各取一随机数，由调用方给出），返回 6/7/8/9
#[wasm_bindgen]
pub fn dayan_yao_wasm(r1: u32, r2: u32, r3: u32) -> u8 {
    lunar_core::folk::zhouyi::dayan_yao_from_random([r1, r2, r3])
}

/// 排盘页玄空飞星：`year` 干支年（定运，每运二十年）；`zuo` 坐山序（0 子……23 壬），或 `zuo_az` ≥ 0 时以坐山方位角（真北起）定山与替卦、空亡。
/// 返回 {"yun":8,"zuo":"子","xiang":"午","ti":false,"kongWang":null,"yunPan":[洛书 1—9 宫],"shanPan":[…],"xiangPan":[…],"geju":[…],"chengMen":[[宫,正否,可用否]…],"yearPan":[…],
/// "pian":偏离坐山中线度数|null,"zhengShen":[宫…],"lingShen":[宫…],"gongMarks":[九宫各 [{n,note}…]],
/// "jieXian":{"zuo":{界},"xiang":{界}}（界 = {"shan","center","shanJie":[lo,hi],"guaJie":[lo,hi],"duan":[{"from","to","kind","jian"}…]}，方位角度数，可为负或逾 360），
/// "daJie":{"gong":[三宫],"li":离宫否,"zhen":真否}|null,"ruQiu":{"shan":运|null,"xiang":运|null},"monthPans":[寅月至丑月十二盘]}
#[wasm_bindgen]
pub fn xuankong_json(year: i32, zuo: u8, ti: bool, zuo_az: f64) -> String {
    use lunar_core::folk::{taiyi::JiuYun, xuankong as xk};
    let o = xk::DEFAULT_OPTIONS;
    let yun = xk::yun_of_year(year, JiuYun::Even20);
    let p = if zuo_az >= 0.0 { xk::zhai_pan_azimuth(yun, real(zuo_az), &o) } else { xk::zhai_pan(yun, zuo as usize % 24, ti, &o) };
    let qian = xk::qian_shi_nian(year);
    let nums = |a: &[u8; 9]| json_arr(a.iter().map(|x| x.to_string()));
    let gjs = |v: &[xk::GeJu]| json_arr(v.iter().map(|g| format!("{{\"n\":{},\"note\":{}}}", json_str(g.name), json_str(g.note))));
    let opt = |x: Option<u8>| x.map(|n| n.to_string()).unwrap_or_else(|| "null".into());
    let jie = |shan: usize| {
        let j = xk::shan_jie(shan, &o);
        format!(
            "{{\"shan\":{},\"center\":{},\"shanJie\":[{},{}],\"guaJie\":[{},{}],\"duan\":{}}}",
            json_str(xk::SHAN24[j.shan]),
            j.center.as_f64(),
            j.shan_jie[0].as_f64(),
            j.shan_jie[1].as_f64(),
            j.gua_jie[0].as_f64(),
            j.gua_jie[1].as_f64(),
            json_arr(j.duan.iter().map(|d| format!(
                "{{\"from\":{},\"to\":{},\"kind\":{},\"jian\":{}}}",
                d.from.as_f64(),
                d.to.as_f64(),
                json_str(d.kind.name()),
                d.jian.map(|s| json_str(xk::SHAN24[s])).unwrap_or_else(|| "null".into())
            )))
        )
    };
    let (zs, ls) = xk::ling_zheng(p.yun, qian, &o);
    let (qs, qx) = xk::ru_qiu(&p);
    format!(
        "{{\"yun\":{},\"zuo\":{},\"xiang\":{},\"ti\":{},\"kongWang\":{},\"yunPan\":{},\"shanPan\":{},\"xiangPan\":{},\"geju\":{},\"chengMen\":{},\"yearPan\":{},\
\"pian\":{},\"zhengShen\":{},\"lingShen\":{},\"gongMarks\":{},\"jieXian\":{{\"zuo\":{},\"xiang\":{}}},\"daJie\":{},\"ruQiu\":{{\"shan\":{},\"xiang\":{}}},\"monthPans\":{}}}",
        p.yun,
        json_str(xk::SHAN24[p.zuo]),
        json_str(xk::SHAN24[p.xiang]),
        p.ti,
        match p.kong_wang {
            Some(xk::KongWang::Da) => json_str("大空亡"),
            Some(xk::KongWang::Xiao) => json_str("小空亡"),
            None => "null".into(),
        },
        nums(&p.yun_pan),
        nums(&p.shan_pan),
        nums(&p.xiang_pan),
        gjs(&p.ge_ju),
        json_arr(xk::cheng_men(&p, &o).iter().map(|(g, z, k)| format!("[{g},{z},{k}]"))),
        nums(&xk::year_pan(year)),
        p.pian_deg.map(|x| x.as_f64().to_string()).unwrap_or_else(|| "null".into()),
        json_arr(zs.iter().map(|g| g.to_string())),
        json_arr(ls.iter().map(|g| g.to_string())),
        json_arr(xk::gong_biao(&p, qian, &o).iter().map(|v| gjs(v))),
        jie(p.zuo),
        jie(p.xiang),
        match xk::da_jie(&p, &o) {
            Some(d) => format!("{{\"gong\":{},\"li\":{},\"zhen\":{}}}", json_arr(d.gong.iter().map(|g| g.to_string())), d.li, d.zhen),
            None => "null".into(),
        },
        opt(qs),
        opt(qx),
        json_arr(xk::month_pans(year).iter().map(|a| nums(a)))
    )
}

/// 排盘页大六壬：{"jiang":"卯","shi":"申","pan":[地盘子…亥上的天盘神],"siKe":[{"xia":"甲","shang":"子"}…],"sanChuan":[…],"fa":"知一"}。
/// `rule`：0 按中气换将（缺省，《六壬大全》卷二）、1 按节换将（未核）。
/// 盘面标注（doc/folk/liuren.md「盘面标注」「课体」）：`dun_rule` 0 旬遁（缺省）1 日干五子遁；`ben_ming_gz` 本命年干支序（−1 不给，则不出年命）；
/// `male` 1 男 0 女（行年男丙寅顺、女壬申逆）。占年、月建取子平四柱（立春换年、交节换月）。
/// 新增字段：dunGan[地盘子…亥上天盘神之遁干，旬空为 null]、sanChuanDun、xunShou、xunKong、yiMa、dingMa、tianMa、riDe、riLu、yangRen、riGui、
/// yueJian、zongMen、keMing、keGe[{n,note}]、benMing、xingNian、xingNianZhi、xuSui
#[wasm_bindgen]
pub fn liuren_json(jd_ut: f64, vsop87_ear: &str, day_gz: u8, hour_zhi: u8, rule: u8, gui_rule: u8, zhou: i8, dun_rule: u8, ben_ming_gz: i16, male: u8) -> Result<String, JsValue> {
    use lunar_core::folk::liuren;
    use lunar_core::folk::liuren::{ke_ti, shen_sha};
    const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
    let vsop = vsop87_cached(vsop87_ear)?;
    let jd_tt = jd_ut + lunar_core::astronomy::time::delta_t(real(jd_ut)).seconds().as_f64() / 86400.0;
    let r = if rule == 1 { liuren::JiangRule::Jie } else { liuren::JiangRule::ZhongQi };
    let k = liuren::qi_ke_at(vsop.as_ref(), real(jd_tt), day_gz % 60, hour_zhi % 12, r);
    let pan = json_arr((0..12u8).map(|d| json_str(ZHI[k.pan.above(d) as usize])));
    let gz = day_gz % 60;
    // 天将：贵人起例 0 俗例（缺省，《六壬大全》提要「尚沿俗例」）1 考原定法；昼夜 zhou −1 以占时卯至申为昼（缺省）、0 夜、1 昼
    let is_zhou = if zhou < 0 { liuren::is_zhou_mao_you(hour_zhi % 12) } else { zhou == 1 };
    let gr = if gui_rule == 1 { liuren::GuiRule::KaoYuan } else { liuren::GuiRule::SuLi };
    let tj = liuren::tian_jiang(&k.pan, gz % 10, is_zhou, gr);
    // 一课之下为日干，二课之下为一课上神，三课之下为日支，四课之下为三课上神
    let xia = [GAN[(gz % 10) as usize], ZHI[k.si_ke[0].shang as usize], ZHI[(gz % 12) as usize], ZHI[k.si_ke[2].shang as usize]];
    let si_ke = json_arr((0..4).map(|i| format!("{{\"xia\":{},\"shang\":{}}}", json_str(xia[i]), json_str(ZHI[k.si_ke[i].shang as usize]))));
    // 盘面标注：占年、月建依子平四柱（立春、交节）
    let (nian_gz, yue_gz, _) = lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_solar(vsop.as_ref(), jd_ut, &lunar_core::calendar::gan_zhi_options::preset_zi_ping_ba_zi());
    let yue_jian = yue_gz % 12;
    let dr = if dun_rule == 1 { shen_sha::DunRule::WuZiDun } else { shen_sha::DunRule::XunDun };
    let pb = shen_sha::pan_biao(&k.pan, gz, dr, Some(yue_jian));
    let opt_gan = |g: Option<u8>| g.map_or("null".to_string(), |g| json_str(GAN[g as usize]));
    let zs = |z: u8| json_str(ZHI[z as usize]);
    let gz_str = |x: u8| format!("{}{}", GAN[(x % 10) as usize], ZHI[(x % 12) as usize]);
    let ke_ge = ke_ti::ke_ge(&k, gz, Some(&tj), Some(yue_jian));
    let nian_ming = if (0..60).contains(&ben_ming_gz) {
        let bm = ben_ming_gz as u8;
        let sui = shen_sha::xu_sui(bm, nian_gz);
        let xn = shen_sha::xing_nian(sui, male == 1);
        format!("\"benMing\":{},\"xingNian\":{},\"xingNianZhi\":{},\"xuSui\":{}", zs(bm % 12), json_str(&gz_str(xn)), zs(xn % 12), sui)
    } else {
        "\"benMing\":null,\"xingNian\":null,\"xingNianZhi\":null,\"xuSui\":null".to_string()
    };
    let biao = format!(
        "\"dunGan\":{},\"sanChuanDun\":{},\"xunShou\":{},\"xunKong\":{},\"yiMa\":{},\"dingMa\":{},\"tianMa\":{},\"riDe\":{},\"riLu\":{},\"yangRen\":{},\"riGui\":{},\"yueJian\":{},\"zongMen\":{},\"keMing\":{},\"keGe\":{},{}",
        json_arr(pb.dun.iter().map(|g| opt_gan(*g))),
        json_arr(k.san_chuan.iter().map(|z| opt_gan(shen_sha::dun_gan(gz, *z, dr)))),
        zs(pb.xun_shou),
        json_arr(pb.xun_kong.iter().map(|z| zs(*z))),
        zs(pb.yi_ma),
        zs(pb.ding_ma),
        pb.tian_ma.map_or("null".to_string(), zs),
        zs(pb.ri_de),
        zs(pb.ri_lu),
        zs(pb.yang_ren),
        json_arr(pb.ri_gui.iter().map(|z| zs(*z))),
        zs(yue_jian),
        json_str(k.fa.zong_men().name()),
        json_str(ke_ti::ke_ming(&k, gz)),
        json_arr(ke_ge.iter().map(|g| format!("{{\"n\":{},\"note\":{}}}", json_str(g.name), json_str(g.note)))),
        nian_ming
    );
    Ok(format!(
        "{{\"jiang\":{},\"shi\":{},\"pan\":{},\"siKe\":{},\"sanChuan\":{},\"fa\":{},\"tianJiang\":{},\"gui\":{},\"shun\":{},\"zhou\":{},{}}}",
        json_str(ZHI[k.pan.jiang as usize]),
        json_str(ZHI[(hour_zhi % 12) as usize]),
        pan,
        si_ke,
        json_arr(k.san_chuan.iter().map(|z| json_str(ZHI[*z as usize]))),
        json_str(k.fa.name()),
        json_arr((0..12u8).map(|d| json_str(tj.name_of(k.pan.above(d))))),
        json_str(ZHI[tj.gui as usize]),
        tj.shun,
        is_zhou,
        biao
    ))
}

/// 排盘页太乙：`kind` 0 岁计、1 月计、2 日计、3 时计；`school` 0 金镜（缺省）、1 淘金歌（只影响岁计）。
/// 岁计、月计按岁计之例取阳遁（`dun` 0 阳 1 阴，调用方可切换）；时计冬至后阳遁、夏至后阴遁自动定（dun 参数忽略）。
/// 返回 {"kind":"時計","r":273,"dun":"陰","yuan":"…","ju":..,"ji":..,"gong":7,"ruGong":2,"tianMu":"太陽","tianMuPos":"巳",
/// "jiShen":"子","shiJi":"大神","shiJiPos":"未","zhuSuan":..,"keSuan":..,"zhuDa":4|null,"zhuCan":..,"keDa":..,"keCan":..,
/// 诸神与格局（皆缺省之法）："suiNian":干支年,"dingJi":"呂申","dingJiPos":"寅","dingJiSuan":16,"dingJiDa":6|null,"dingJiCan":..,
/// "siShen"/"tianYi"/"diYi"/"zhiFu":{"gong":1—12,"name":"絳宮","zhi":"亥","ruNian":1—3},
/// "junJi"/"chenJi"/"minJi":{"zhi":"午","ruNian":..},"wuFu":{"gong":1|3|9|7|5,"name":"黃始宮","ruNian":..},
/// "daYou":{"gong":1—9,"ruNian":..},"daYouTianMu":"天道","daYouTianMuPos":"未","xiaoYou":1—9,
/// "tianMuZhuang"/"shiJiZhuang"/"zhuDaZhuang"/"zhuCanZhuang"/"keDaZhuang"/"keCanZhuang":"囚"|"外辰迫"|…|"",
/// "geJu":["主大將客大將關",…],"zhuJiangFa":bool,"keJiangFa":bool}
#[wasm_bindgen]
pub fn taiyi_json(jd_ut: f64, vsop87_ear: &str, day_gz: u8, hour_zhi: u8, kind: u8, school: u8, dun: u8) -> Result<String, JsValue> {
    use lunar_core::astronomy::aspects::solar_term_jds_for_year_cached;
    use lunar_core::calendar::gan_zhi_options::{ganzhi_from_jd_solar, preset_zi_ping_ba_zi};
    use lunar_core::folk::taiyi;
    let vsop = vsop87_cached(vsop87_ear)?;
    let jd_tt = jd_ut + lunar_core::astronomy::time::delta_t(real(jd_ut)).seconds().as_f64() / 86400.0;
    let (gy, _, _) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real(jd_ut));
    let (yi, mi, _) = ganzhi_from_jd_solar(vsop.as_ref(), jd_ut, &preset_zi_ping_ba_zi());
    let year = if (gy - 4).rem_euclid(60) as u8 == yi { gy } else { gy - 1 };
    let sch = if school == 1 { taiyi::JiNian::TaoJinGe } else { taiyi::JiNian::JinJing };
    let mut d = if dun == 1 { taiyi::Dun::Yin } else { taiyi::Dun::Yang };
    let (name, r) = match kind {
        0 => ("歲計", taiyi::sui_ji(year, sch)),
        1 => ("月計", taiyi::yue_ji(year, ((mi % 12) as i32 - 2).rem_euclid(12) as u8, sch)),
        2 => ("日計", taiyi::ri_ji((jd_ut + 8.0 / 24.0 + 0.5).floor() as i64)),
        _ => {
            let mut zl: Vec<(f64, bool)> = Vec::new();
            for y in [gy - 1, gy] {
                let t = solar_term_jds_for_year_cached(vsop.as_ref(), y);
                zl.push((t[18].as_f64(), true));
                zl.push((t[6].as_f64(), false));
            }
            let yang = zl.iter().filter(|(t, _)| *t <= jd_tt).max_by(|a, b| a.0.partial_cmp(&b.0).unwrap()).map(|x| x.1).unwrap_or(true);
            d = if yang { taiyi::Dun::Yang } else { taiyi::Dun::Yin };
            ("時計", taiyi::shi_ji(day_gz % 60, hour_zhi % 12))
        }
    };
    let j = taiyi::ju(r, d);
    let opt = |g: Option<u8>| g.map(|v| v.to_string()).unwrap_or_else(|| "null".into());
    Ok(format!(
        "{{\"kind\":{},\"r\":{},\"dun\":{},\"yuan\":{},\"ju\":{},\"ji\":{},\"gong\":{},\"ruGong\":{},\"tianMu\":{},\"tianMuPos\":{},\"jiShen\":{},\"shiJi\":{},\"shiJiPos\":{},\"zhuSuan\":{},\"keSuan\":{},\"zhuDa\":{},\"zhuCan\":{},\"keDa\":{},\"keCan\":{}}}",
        json_str(name),
        j.r,
        json_str(if j.dun == taiyi::Dun::Yang { "陽" } else { "陰" }),
        json_str(taiyi::YUAN[j.yuan as usize]),
        j.ju,
        j.ji,
        j.gong,
        j.ru_gong,
        json_str(taiyi::SHEN16[j.tian_mu]),
        json_str(taiyi::POS16[j.tian_mu]),
        json_str(taiyi::ji_shen_zhi(&j)),
        json_str(taiyi::SHEN16[j.shi_ji]),
        json_str(taiyi::POS16[j.shi_ji]),
        j.zhu_suan,
        j.ke_suan,
        opt(j.zhu_da),
        opt(j.zhu_can),
        opt(j.ke_da),
        opt(j.ke_can)
    ))
}

/// 年神方位文字：岁德、岁德合一句，其后按二十四山列有神之位
fn nian_shen_text(year_gz: u8) -> String {
    use lunar_core::folk::nianshen::{by_position, nian_shen};
    let all = nian_shen(year_gz);
    let p = |n: &str| all.iter().find(|s| s.name == n).map(|s| s.positions.join("")).unwrap_or_default();
    let mut parts = vec![format!("歲德在{}合在{}", p("歲德"), p("歲德合"))];
    parts.extend(by_position(year_gz).into_iter().filter(|(_, v)| !v.is_empty()).map(|(q, v)| format!("{q}：{}", v.join(" "))));
    parts.join("；")
}

/// 某民用日（JD 取该日任一时刻）的值日宿与七曜，以及该时刻的年、月紫白九星。
/// 九星按立春换年、按节换月（《協紀辨方書》卷八、卷三十四），不随四柱预设变化。
#[wasm_bindgen]
pub fn folk_day_wasm(jd: f64, vsop87_ear: &str) -> Result<FolkDayResult, JsValue> {
    use lunar_core::calendar::gan_zhi_options::{ganzhi_from_jd_solar, preset_zi_ping_ba_zi};
    use lunar_core::folk::{jiuxing, zhiri};
    let jdn = (jd + 0.5).floor() as i64;
    let xiu = zhiri::xiu_index(jdn);
    let vsop = vsop87_cached(vsop87_ear)?;
    let (yi, mi, _) = ganzhi_from_jd_solar(vsop.as_ref(), jd, &preset_zi_ping_ba_zi());
    // 干支年：公元 Y 年对应干支序 (Y − 4) mod 60；JD 所在公历年或其前一年
    let (gy, _, _) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real(jd));
    let ganzhi_year = if (gy - 4).rem_euclid(60) as u8 == yi { gy } else { gy - 1 };
    let month_ord = ((mi % 12) as i32 - 2).rem_euclid(12) as u8;
    Ok(FolkDayResult {
        xiu: zhiri::XIU_28[xiu as usize].to_string(),
        qi_yao: zhiri::QI_YAO[zhiri::xiu_qi_yao(xiu) as usize].to_string(),
        year_star: jiuxing::JIU_XING[jiuxing::year_star(ganzhi_year) as usize - 1].to_string(),
        month_star: jiuxing::JIU_XING[jiuxing::month_star(yi % 12, month_ord) as usize - 1].to_string(),
        nian_shen: nian_shen_text(yi),
        ri_shen: {
            use lunar_core::folk::nianshen::{he_shen, ri_you_shen, tai_sui_chu_you};
            let dgz = (jdn + 49).rem_euclid(60) as u8;
            let mut t = format!("日遊神{} · 鶴神{}", ri_you_shen(dgz), he_shen(dgz));
            if let Some(f) = tai_sui_chu_you(dgz) {
                t.push_str(&format!(" · 太歲{}遊", if f == "中宮" { "中宮" } else { f }));
            }
            t
        },
    })
}

/// 某民用日（东八区）的杂节气标签：三伏、社日（两家）、寒食、腊日、入梅出梅（两家）、土王用事、数九、七十二候。
/// 节气取定气、东八区日。返回以「；」分隔的标签串（无则空串）。
#[wasm_bindgen]
pub fn zajie_for_day_wasm(jd: f64, vsop87_ear: &str) -> Result<String, JsValue> {
    use lunar_core::astronomy::aspects::solar_term_jds_for_year_cached;
    use lunar_core::calendar::chinese_lunar::tt_to_day_number_utc8;
    use lunar_core::folk::zajie as z;
    let vsop = vsop87_cached(vsop87_ear)?;
    let jdn = (jd + 0.5).floor() as i64;
    let (y, _, _) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real(jd));
    let tt = |yy: i32| solar_term_jds_for_year_cached(vsop.as_ref(), yy);
    let (tp, tc, tn) = (tt(y - 1), tt(y), tt(y + 1));
    let d = |v: &Vec<lunar_core::math::real::Real>, k: usize| tt_to_day_number_utc8(v[k]);
    // 索引：0 春分 3 立夏 5 芒种 6 夏至 7 小暑 9 立秋 12 秋分 15 立冬 18 冬至 21 立春
    let mut out: Vec<String> = Vec::new();
    let mut hit = |day: i64, s: &str| {
        if day == jdn {
            out.push(s.to_string());
        }
    };
    let (f1, f2, f3) = z::san_fu(d(&tc, 6), d(&tc, 9));
    hit(f1, "初伏");
    hit(f2, "中伏");
    hit(f3, "末伏");
    hit(f3 + 10, "出伏");
    hit(z::she_wu_wu(d(&tc, 21)), "春社（立春后五戊）");
    hit(z::she_wu_wu(d(&tc, 9)), "秋社（立秋后五戊）");
    hit(z::she_jin_wu(d(&tc, 0)), "春社（春分近戊）");
    hit(z::she_jin_wu(d(&tc, 12)), "秋社（秋分近戊）");
    hit(z::han_shi(d(&tp, 18), 105), "寒食（冬至后一百五日）");
    hit(d(&tc, 1) - 2, "寒食（清明前二日，《協紀》按平气所言）");
    for dz in [d(&tp, 18), d(&tc, 18)] {
        hit(z::la_ri(dz, 10), "腊日（冬至后三戌）");
    }
    let (r1, c1) = z::mei_shen_shu(d(&tc, 5), d(&tc, 7));
    hit(r1, "入梅（神樞經）");
    hit(c1, "出梅（神樞經）");
    let (r2, c2) = z::mei_sui_jin(d(&tc, 5), d(&tc, 6));
    hit(r2, "入梅（碎金）");
    hit(c2, "出梅（碎金）");
    for (k, name) in [(21usize, "立春"), (3, "立夏"), (9, "立秋"), (15, "立冬")] {
        // 立春土王用事在上年冬，取本年立春之前十八日
        hit(z::tu_wang(d(&tc, k)), &format!("土王用事（{name}前十八日）"));
    }
    for dz in [d(&tp, 18), d(&tc, 18)] {
        if let Some((n, k)) = z::shu_jiu(dz, jdn) {
            const NUM: [&str; 9] = ["一", "二", "三", "四", "五", "六", "七", "八", "九"];
            out.push(format!("{}九第{}日", NUM[n as usize - 1], NUM[k as usize - 1]));
        }
    }
    // 七十二候：本气至下气三等分；QI_HOU 从立春起，立春 = 索引 21
    let all: Vec<lunar_core::math::real::Real> = tp.iter().chain(tc.iter()).chain(tn.iter()).cloned().collect();
    let mut seq: Vec<(lunar_core::math::real::Real, usize)> = Vec::new();
    for (i, t) in all.iter().enumerate() {
        seq.push((*t, i % 24));
    }
    seq.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for w in seq.windows(2) {
        let (t0, k) = w[0];
        let t1 = w[1].0;
        let qi = (k + 3) % 24; // solar_term 索引 21（立春）→ QI_HOU 0
        for h in 0..3 {
            let start = tt_to_day_number_utc8(t0 + (t1 - t0) * real(h as f64 / 3.0));
            let end = tt_to_day_number_utc8(t0 + (t1 - t0) * real((h + 1) as f64 / 3.0));
            if jdn >= start && jdn < end {
                let (qn, hs) = z::QI_HOU[qi];
                out.push(format!("{qn}{}候 {}", ["初", "次", "末"][h], hs[h]));
            }
        }
    }
    Ok(out.join("；"))
}

/// 更点：某民用日、某纬度的昏、各更、攒点、旦（当地视太阳时，清制时刻名）。
/// method：0 = 清台官法，1 = 元授时法。太阳赤纬取当地正午（以 jd 所在日 + 经度换算）时的视赤纬。
#[wasm_bindgen]
pub fn gengdian_wasm(jd: f64, latitude_deg: f64, longitude_deg: f64, method: u8, vsop87_ear: &str) -> Result<String, JsValue> {
    use lunar_core::astronomy::apparent::{right_ascension_declination, sun_apparent_equatorial, ApparentPipelineOptions};
    use lunar_core::astronomy::time::{ut1_to_tt_jd, TimePoint, TimeScale};
    use lunar_core::folk::gengdian::{qing, qing_shike, shou_shi};
    let vsop = vsop87_cached(vsop87_ear)?;
    // 当地正午的 UT：民用日中午（东八区 JD 取整）减经度换算
    let jdn = (jd + 0.5).floor();
    let noon_ut = jdn - longitude_deg / 360.0;
    let tt = ut1_to_tt_jd(real(noon_ut));
    let sun = sun_apparent_equatorial(vsop.as_ref(), TimePoint::new(TimeScale::TT, tt), &ApparentPipelineOptions::pipeline_default());
    let (_, dec) = right_ascension_declination(&sun);
    let dec_deg = dec.arcsec() / real(3600.0);
    let g = if method == 1 { shou_shi(real(latitude_deg), dec_deg) } else { qing(real(latitude_deg), dec_deg) };
    const N: [&str; 5] = ["一", "二", "三", "四", "五"];
    let mut parts = vec![format!("日入{}", qing_shike(g.ri_ru)), format!("昏{}", qing_shike(g.hun))];
    for (k, t) in g.geng.iter().enumerate() {
        parts.push(format!("{}更{}", N[k], qing_shike(*t)));
    }
    parts.push(format!("攢點{}", qing_shike(g.zan_dian)));
    parts.push(format!("旦{}", qing_shike(g.dan)));
    parts.push(format!("日出{}", qing_shike(g.ri_chu)));
    Ok(parts.join(" "))
}

/// 某民用日的建除、日黄黑道、百忌（月建按节；交节日即属新月）。返回如「定日 · 天德（黄道） · 乙不栽植 未不服藥」。
#[wasm_bindgen]
pub fn jianchu_day_wasm(jd: f64, vsop87_ear: &str) -> Result<String, JsValue> {
    use lunar_core::calendar::gan_zhi_options::{ganzhi_from_jd_solar, preset_zi_ping_ba_zi};
    use lunar_core::folk::jianchu as j;
    let vsop = vsop87_cached(vsop87_ear)?;
    let jdn = (jd + 0.5).floor();
    // 取该日（东八区）将尽之时判断月建，使交节当日全日属新月：北京该日 24 时 = JDN + 0.5 − 8/24（UT，TT 差一分余可略）
    let (_, mi, _) = ganzhi_from_jd_solar(vsop.as_ref(), jdn + 0.5 - 8.0 / 24.0 - 1e-6, &preset_zi_ping_ba_zi());
    let gz = (jdn as i64 + 49).rem_euclid(60) as u8;
    let (dg, dz, mz) = (gz % 10, gz % 12, mi % 12);
    let shen = j::huang_hei(dz, mz);
    Ok(format!(
        "{}日 · {}（{}） · {} {}",
        j::JIAN_CHU[j::jian_chu(dz, mz) as usize],
        j::HUANG_HEI_SHEN[shen as usize],
        if j::is_huang_dao(shen) { "黄道" } else { "黑道" },
        j::BAI_JI_GAN[dg as usize],
        j::BAI_JI_ZHI[dz as usize]
    ))
}

/// 某民用日的神煞宜忌：《協紀》月表（卷二十至三十一）加卷十铺注（年份、节气、朔望诸日）与时宪书体例（嫁娶周堂、交节日）。
/// 农历日期由前端传入（与所显示者一致）：lunar_month 1—12（闰月取所闰之月）、lunar_day、days_in_month（29/30）；
/// phase：该日所值 0 朔、1 上弦、2 望、3 下弦，无则 -1。节气用地球历表现算。
/// 返回五行：「吉神：…；凶神：…」「宜：…」「忌：…」「時憲書宜：…」（民用三十七事）「時憲書朱字：…」（底栏朱字应值之神）。
#[wasm_bindgen]
pub fn shensha_day_wasm(
    jd: f64,
    vsop87_ear: &str,
    lunar_month: u8,
    lunar_day: u8,
    is_leap: bool,
    days_in_month: u8,
    phase: i32,
) -> Result<String, JsValue> {
    use lunar_core::calendar::chinese_lunar::ChineseLunarDate;
    use lunar_core::folk::puzhu::{minyong, yiji_for_day, zhu_zi, QiWangWangCount, TermTable};
    let vsop = vsop87_cached(vsop87_ear)?;
    let jdn = (jd + 0.5).floor() as i64;
    let (gy, _, _) = lunar_core::calendar::gregorian::Gregorian::from_julian_day(real(jd));
    let table = TermTable::new(vsop.as_ref(), gy, gy);
    let lunar = ChineseLunarDate { year: gy, month: lunar_month, day: lunar_day, is_leap_month: is_leap, days_in_month };
    let ph = match phase {
        0 => Some("朔"),
        1 => Some("上弦"),
        2 => Some("望"),
        3 => Some("下弦"),
        _ => None,
    };
    let day = table.day(jdn, lunar, ph, QiWangWangCount::Inclusive);
    let d = yiji_for_day(&day);
    let join = |v: &[String]| if v.is_empty() { "—".to_string() } else { v.join(" ") };
    let yi = match &d.yi { Some(v) => join(v), None => "諸事不宜".to_string() };
    let (my, _) = minyong(&d);
    let my = match my { Some(v) if v.is_empty() => "—".to_string(), Some(v) => v.join(" "), None => "諸事不宜".to_string() };
    let zz = zhu_zi(&day);
    Ok(format!(
        "吉神：{}；凶神：{}\n宜：{}\n忌：{}\n時憲書宜：{}\n時憲書朱字：{}",
        join(&d.ji_shen),
        join(&d.xiong_shen),
        yi,
        join(&d.ji),
        my,
        if zz.is_empty() { "—".to_string() } else { zz.join(" ") }
    ))
}

/// 择日页宜忌另一说：《董公選擇日要覽》。`jd` 所在民用日；月建按节（与协纪同），用地球历表现算。
/// 返回 {"monthZhi":"寅","dayGz":"丁酉","jianChu":"危","noText":false,"text":"宜忌之文","fullText":"原文","entryNote":"条目校记",
/// "gods":["天德",…],"rules":[{"event":"安葬","yi":true,"grade":"","basis":"所据原句","cond":"","note":"校记","xieji":["安葬"]}…]}；
/// noText 为真时 rules 为空（「董公无宜忌之文」），不回退协纪。见 doc/folk/tongshu_yiji.md。
#[wasm_bindgen]
pub fn dong_gong_day_json(jd: f64, vsop87_ear: &str) -> Result<String, JsValue> {
    use lunar_core::folk::dong_gong::{dong_gong_day, xieji_events};
    let vsop = vsop87_cached(vsop87_ear)?;
    let (_, mi, _) = lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_solar(vsop.as_ref(), jd, &lunar_core::calendar::gan_zhi_options::preset_zi_ping_ba_zi());
    let month_zhi = (mi % 12) as u8;
    let dgz = ((jd + 0.5).floor() as i64 + 49).rem_euclid(60) as u8;
    let d = dong_gong_day(month_zhi, dgz);
    const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    let rules = d.rules.iter().map(|r| {
        format!(
            "{{\"event\":{},\"yi\":{},\"grade\":{},\"basis\":{},\"cond\":{},\"note\":{},\"xieji\":{}}}",
            json_str(r.event),
            r.yi,
            json_str(r.grade),
            json_str(r.basis),
            json_str(r.condition),
            json_str(r.note),
            json_arr(xieji_events(r.event).iter().map(|e| json_str(e)))
        )
    });
    Ok(format!(
        "{{\"monthZhi\":{},\"dayGz\":{},\"jianChu\":{},\"noText\":{},\"text\":{},\"fullText\":{},\"entryNote\":{},\"gods\":{},\"rules\":{}}}",
        json_str(ZHI[month_zhi as usize]),
        json_str(lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(dgz)),
        json_str(d.jian_chu),
        d.no_text(),
        json_str(d.text),
        json_str(d.full_text),
        json_str(d.entry_note),
        json_arr(d.gods.iter().map(|g| json_str(g))),
        json_arr(rules)
    ))
}

/// 时黄黑道：按日支起时。day_name、hour_name 为干支名（如「乙未」「丙子」）；不合法返回空串。
#[wasm_bindgen]
pub fn huanghei_hour_wasm(day_name: &str, hour_name: &str) -> String {
    use lunar_core::folk::jianchu as j;
    const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";
    let zhi_of = |n: &str| n.chars().nth(1).and_then(|c| ZHI.chars().position(|z| z == c)).map(|i| i as u8);
    match (zhi_of(day_name), zhi_of(hour_name)) {
        (Some(d), Some(h)) => {
            let shen = j::huang_hei(h, d);
            format!("{}（{}）", j::HUANG_HEI_SHEN[shen as usize], if j::is_huang_dao(shen) { "黄道" } else { "黑道" })
        }
        _ => String::new(),
    }
}

/// 值日宿、七曜、年月九星
#[wasm_bindgen]
pub struct FolkDayResult {
    xiu: String,
    qi_yao: String,
    year_star: String,
    month_star: String,
    /// 年神方位（协纪卷三；年以立春分）：「歲德在某合在某；酉：太歲 歲刑 大煞；……」，按二十四山自子起，空位不列
    nian_shen: String,
    /// 当日日游神、鹤神方位与太岁出游（协纪卷三）
    ri_shen: String,
}

#[wasm_bindgen]
impl FolkDayResult {
    #[wasm_bindgen(getter_with_clone)]
    pub fn xiu(&self) -> String {
        self.xiu.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn qi_yao(&self) -> String {
        self.qi_yao.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn year_star(&self) -> String {
        self.year_star.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn nian_shen(&self) -> String {
        self.nian_shen.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn ri_shen(&self) -> String {
        self.ri_shen.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn month_star(&self) -> String {
        self.month_star.clone()
    }
}

/// 预设名称，供 UI 显示：0=子平（子正） 1=子平（子初） 2=紫微斗数 3=农历月干支
#[wasm_bindgen]
pub fn ganzhi_preset_name(preset_index: u8) -> String {
    match preset_index {
        0 => "子平（子正）",
        1 => "子平（子初）",
        2 => "紫微斗数",
        _ => "农历月干支",
    }
    .to_string()
}

/// 农历派（正月初一换年、初一换月）：需岁数据。options: year_boundary, month_boundary, leap_month_handling, day_boundary 各 0..3。
#[wasm_bindgen]
pub fn ganzhi_from_jd_lunar_wasm(
    jd: f64,
    lunar_year: i32,
    new_moon_jds: Vec<f64>,
    zhong_qi_jds: Vec<f64>,
    year_boundary: u8,
    month_boundary: u8,
    leap_month_handling: u8,
    day_boundary: u8,
) -> Option<GanzhiResult> {
    let nm: Vec<_> = new_moon_jds.into_iter().map(real).collect();
    let zq: Vec<_> = zhong_qi_jds.into_iter().map(real).collect();
    let year_data =
        lunar_core::calendar::chinese_lunar::ChineseLunarYearData::new(lunar_year, nm, zq);
    let opts = options_from_u8s(year_boundary, month_boundary, leap_month_handling, day_boundary, 0);
    let (yi, mi, di) = lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_lunar(real(jd), &year_data, &opts)?;
    Some(GanzhiResult {
        year_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(yi).to_string(),
        month_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(mi).to_string(),
        day_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(di).to_string(),
    })
}

/// 整月干支结果：按公历日 1..=days 顺序，三组字符串。索引 0 = 1 号。
#[wasm_bindgen]
pub struct MonthGanzhiResult {
    year_names: Vec<String>,
    month_names: Vec<String>,
    day_names: Vec<String>,
}

#[wasm_bindgen]
impl MonthGanzhiResult {
    #[wasm_bindgen(getter, js_name = yearNames)]
    pub fn year_names(&self) -> Vec<String> {
        self.year_names.clone()
    }
    #[wasm_bindgen(getter, js_name = monthNames)]
    pub fn month_names(&self) -> Vec<String> {
        self.month_names.clone()
    }
    #[wasm_bindgen(getter, js_name = dayNames)]
    pub fn day_names(&self) -> Vec<String> {
        self.day_names.clone()
    }
}

/// 公历整月逐日干支（批量，一次跨边界）。主岁/次岁与「正月初一只认主岁」逻辑与单日 getGanzhiForDay 一致。
/// data2_lunar_year 为 0 表示无次岁；vsop87_ear 为空则不做节气历兜底。
#[wasm_bindgen(js_name = ganzhiForGregorianMonthWasm)]
pub fn ganzhi_for_gregorian_month_wasm(
    year: i32,
    month: i32,
    primary_lunar_year: i32,
    data1_lunar_year: i32,
    new_moon_jds_1: Vec<f64>,
    zhong_qi_jds_1: Vec<f64>,
    data2_lunar_year: i32,
    new_moon_jds_2: Vec<f64>,
    zhong_qi_jds_2: Vec<f64>,
    year_boundary: u8,
    month_boundary: u8,
    leap_month_handling: u8,
    day_boundary: u8,
    vsop87_ear: &str,
    solar_term_mode: u8,
) -> MonthGanzhiResult {
    use lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name;
    let days = lunar_core::calendar::gregorian::Gregorian::days_in_month(year, month) as i32;
    let opts = options_from_u8s(year_boundary, month_boundary, leap_month_handling, day_boundary, solar_term_mode);
    let db = day_boundary_from_u8(day_boundary);

    let year_data_1 = {
        let nm: Vec<_> = new_moon_jds_1.into_iter().map(real).collect();
        let zq: Vec<_> = zhong_qi_jds_1.into_iter().map(real).collect();
        lunar_core::calendar::chinese_lunar::ChineseLunarYearData::new(data1_lunar_year, nm, zq)
    };
    let year_data_2 = if data2_lunar_year != 0 {
        let nm: Vec<_> = new_moon_jds_2.into_iter().map(real).collect();
        let zq: Vec<_> = zhong_qi_jds_2.into_iter().map(real).collect();
        Some(lunar_core::calendar::chinese_lunar::ChineseLunarYearData::new(
            data2_lunar_year,
            nm,
            zq,
        ))
    } else {
        None
    };

    let vsop = if vsop87_ear.is_empty() {
        None
    } else {
        let mut guard = VSOP87_CACHE.lock().unwrap();
        if let Some(cached) = guard.as_ref() {
            Some(Arc::clone(cached))
        } else {
            let mut files = HashMap::new();
            files.insert(VSOP87_EAR_PATH.to_string(), vsop87_ear.to_string());
            let loader = DataLoaderWasm::new(files);
            lunar_core::repo::set_loader(Box::new(loader));
            match load_earth_vsop87_from_repo().map(with_earth_patch) {
                Ok(parsed) => {
                    let arc = Arc::new(parsed);
                    *guard = Some(Arc::clone(&arc));
                    Some(arc)
                }
                Err(_) => None,
            }
        }
    };

    let mut year_names = Vec::with_capacity(days as usize);
    let mut month_names = Vec::with_capacity(days as usize);
    let mut day_names = Vec::with_capacity(days as usize);

    for day in 1..=days {
        let jd = lunar_core::calendar::gregorian::Gregorian::to_julian_day(year, month, day);
        let day_name =
            lunar_core::calendar::gan_zhi_options::jd_to_gan_zhi_day_with_options(jd, db)
                .to_string();
        let jd_f64 = jd.as_f64();

        let (yn, mn) = if year_boundary == 1 && month_boundary == 1 {
            let try_lunar = |year_data: &lunar_core::calendar::chinese_lunar::ChineseLunarYearData,
                            data_ly: i32|
             -> Option<(String, String)> {
                let (yi, mi, _di) =
                    lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_lunar(
                        jd,
                        year_data,
                        &opts,
                    )?;
                let date = lunar_core::calendar::chinese_lunar::from_julian_day_in_year(
                    jd,
                    year_data,
                    None,
                )?;
                if date.month == 1 && date.day == 1 && primary_lunar_year != 0 && data_ly != primary_lunar_year {
                    return None;
                }
                Some((
                    gan_zhi_index_to_name(yi).to_string(),
                    gan_zhi_index_to_name(mi).to_string(),
                ))
            };
            try_lunar(&year_data_1, data1_lunar_year)
                .or_else(|| {
                    year_data_2.as_ref().and_then(|yd| try_lunar(yd, data2_lunar_year))
                })
                .unwrap_or_else(|| {
                    if let Some(ref vs) = vsop {
                        let (yi, mi, _) =
                            lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_solar(
                                vs.as_ref(),
                                jd_f64,
                                &opts,
                            );
                        (
                            gan_zhi_index_to_name(yi).to_string(),
                            gan_zhi_index_to_name(mi).to_string(),
                        )
                    } else {
                        (String::new(), String::new())
                    }
                })
        } else {
            if let Some(ref vs) = vsop {
                let (yi, mi, _) =
                    lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_solar(
                        vs.as_ref(),
                        jd_f64,
                        &opts,
                    );
                (
                    gan_zhi_index_to_name(yi).to_string(),
                    gan_zhi_index_to_name(mi).to_string(),
                )
            } else {
                (String::new(), String::new())
            }
        };

        year_names.push(yn);
        month_names.push(mn);
        day_names.push(day_name);
    }

    MonthGanzhiResult {
        year_names,
        month_names,
        day_names,
    }
}

/// 岁数据现算结果，供 JS 取 lunar_year / new_moon_jds / zhong_qi_jds。
#[wasm_bindgen]
pub struct YearDataResult {
    lunar_year: i32,
    new_moon_jds: Vec<f64>,
    zhong_qi_jds: Vec<f64>,
    /// 岁内朔、上弦、望、下弦时刻 JD(TT)（首朔至末朔），与 phase_kinds 一一对应
    phase_jds: Vec<f64>,
    /// 0 朔、1 上弦、2 望、3 下弦
    phase_kinds: Vec<u8>,
    /// 农历年前後各一公历年（lunar_year − 1 … lunar_year + 1）的二十四节气 JD(TT)，每年 24 项、0 春分 … 23 驚蟄。
    /// 网页 Worker 算出後交主线程以 [`seed_solar_terms_wasm`] 灌入节气缓存，逐日的干支、杂节、铺注便不在主线程求解
    solar_term_years: Vec<i32>,
    solar_term_jds: Vec<f64>,
}

/// 岁数据结果，附岁内朔弦望（铺注「朔弦望忌求醫療病」用）
fn year_data_result(
    year_data: &lunar_core::calendar::chinese_lunar::ChineseLunarYearData,
    vsop: &Vsop87,
    elp: &Elpmpp02Data,
) -> YearDataResult {
    use lunar_core::astronomy::aspects::LunarPhase;
    let nm = &year_data.new_moon_jds;
    let phases = lunar_core::calendar::chinese_lunar::year_phases(year_data, vsop, elp);
    let solar_term_years: Vec<i32> = ((year_data.lunar_year - 1)..=(year_data.lunar_year + 1)).collect();
    let solar_term_jds = solar_term_years
        .iter()
        .flat_map(|&y| lunar_core::astronomy::aspects::solar_term_jds_for_year_cached(vsop, y))
        .map(|r| r.as_f64())
        .collect();
    YearDataResult {
        solar_term_years,
        solar_term_jds,
        lunar_year: year_data.lunar_year,
        new_moon_jds: nm.iter().map(|r| r.as_f64()).collect(),
        zhong_qi_jds: year_data.zhong_qi_jds.iter().map(|r| r.as_f64()).collect(),
        phase_jds: phases.iter().map(|(t, _)| t.as_f64()).collect(),
        phase_kinds: phases
            .iter()
            .map(|(_, p)| match p {
                LunarPhase::Shuo => 0,
                LunarPhase::ShangXian => 1,
                LunarPhase::Wang => 2,
                LunarPhase::XiaXian => 3,
            })
            .collect(),
    }
}

#[wasm_bindgen]
impl YearDataResult {
    #[wasm_bindgen(getter)]
    pub fn solar_term_years(&self) -> Vec<i32> {
        self.solar_term_years.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn solar_term_jds(&self) -> Vec<f64> {
        self.solar_term_jds.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn lunar_year(&self) -> i32 {
        self.lunar_year
    }
    #[wasm_bindgen(getter)]
    pub fn new_moon_jds(&self) -> Vec<f64> {
        self.new_moon_jds.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn zhong_qi_jds(&self) -> Vec<f64> {
        self.zhong_qi_jds.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn phase_jds(&self) -> Vec<f64> {
        self.phase_jds.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn phase_kinds(&self) -> Vec<u8> {
        self.phase_kinds.clone()
    }
}

/// 在 wasm 内现算指定农历年的岁数据（14 朔 + 12 中气）。
#[wasm_bindgen]
pub fn compute_year_data_wasm(
    lunar_year: i32,
    vsop87_ear: &str,
    elp_main_s1: &str,
    elp_main_s2: &str,
    elp_main_s3: &str,
    elp_pert_s1: &str,
    elp_pert_s2: &str,
    elp_pert_s3: &str,
    nutation_tab53a_bin: &[u8],
) -> Result<YearDataResult, JsValue> {
    let mut files = HashMap::new();
    files.insert(VSOP87_EAR_PATH.to_string(), vsop87_ear.to_string());
    files.insert(format!("{}/ELP_MAIN.S1", ELP_BASE), elp_main_s1.to_string());
    files.insert(format!("{}/ELP_MAIN.S2", ELP_BASE), elp_main_s2.to_string());
    files.insert(format!("{}/ELP_MAIN.S3", ELP_BASE), elp_main_s3.to_string());
    files.insert(format!("{}/ELP_PERT.S1", ELP_BASE), elp_pert_s1.to_string());
    files.insert(format!("{}/ELP_PERT.S2", ELP_BASE), elp_pert_s2.to_string());
    files.insert(format!("{}/ELP_PERT.S3", ELP_BASE), elp_pert_s3.to_string());
    let loader = DataLoaderWasm::new(files);
    lunar_core::repo::set_loader(Box::new(loader));
    let nutation_ok = if !nutation_tab53a_bin.is_empty() {
        nutation::try_init_full_nutation_from_binary(nutation_tab53a_bin)
    } else {
        nutation::try_init_full_nutation_from_repo()
    };
    *REPO_AUX_STATUS.lock().unwrap() = (nutation_ok, earth_patch_loaded());
    let vsop = with_earth_patch(load_earth_vsop87_from_repo().map_err(|e| JsValue::from_str(&e.to_string()))?);
    let elp = load_elp_text()?;
    let year_data = lunar_core::calendar::chinese_lunar::compute_year_data(
        &vsop,
        &elp,
        lunar_year,
        lunar_core::quantity::angle::PlaneAngle::from_rad(real(1e-8)),
        30,
    )
    .map_err(|e| JsValue::from_str(&e))?;
    Ok(year_data_result(&year_data, &vsop, &elp))
}

/// 与 [compute_year_data_wasm] 相同，但 VSOP87 地心历表以二进制传入，零解析、省带宽与 CPU。
/// 前端优先 fetch `.ear.bin`，以 `Uint8Array` 传入本函数；ELP 仍为文本。
#[wasm_bindgen]
pub fn compute_year_data_from_binary(
    lunar_year: i32,
    vsop87_ear_bin: &[u8],
    elp_main_s1: &str,
    elp_main_s2: &str,
    elp_main_s3: &str,
    elp_pert_s1: &str,
    elp_pert_s2: &str,
    elp_pert_s3: &str,
    nutation_tab53a_bin: &[u8],
) -> Result<YearDataResult, JsValue> {
    let vsop = with_earth_patch(Vsop87::from_binary(vsop87_ear_bin).map_err(|e| JsValue::from_str(&e.to_string()))?);
    let mut files = HashMap::new();
    files.insert(format!("{}/ELP_MAIN.S1", ELP_BASE), elp_main_s1.to_string());
    files.insert(format!("{}/ELP_MAIN.S2", ELP_BASE), elp_main_s2.to_string());
    files.insert(format!("{}/ELP_MAIN.S3", ELP_BASE), elp_main_s3.to_string());
    files.insert(format!("{}/ELP_PERT.S1", ELP_BASE), elp_pert_s1.to_string());
    files.insert(format!("{}/ELP_PERT.S2", ELP_BASE), elp_pert_s2.to_string());
    files.insert(format!("{}/ELP_PERT.S3", ELP_BASE), elp_pert_s3.to_string());
    let loader = DataLoaderWasm::new(files);
    lunar_core::repo::set_loader(Box::new(loader));
    let nutation_ok = if !nutation_tab53a_bin.is_empty() {
        nutation::try_init_full_nutation_from_binary(nutation_tab53a_bin)
    } else {
        nutation::try_init_full_nutation_from_repo()
    };
    *REPO_AUX_STATUS.lock().unwrap() = (nutation_ok, earth_patch_loaded());
    let elp = load_elp_text()?;
    let year_data = lunar_core::calendar::chinese_lunar::compute_year_data(
        &vsop,
        &elp,
        lunar_year,
        lunar_core::quantity::angle::PlaneAngle::from_rad(real(1e-8)),
        30,
    )
    .map_err(|e| JsValue::from_str(&e))?;
    Ok(year_data_result(&year_data, &vsop, &elp))
}

/// 全二进制岁数据：VSOP87 + 6 个 ELP 均为二进制，零解析。前端 fetch 全部 .bin 时调用。
/// nutation_tab53a_bin：章动完整表二进制（tab5.3a.bin），传空切片则不启用完整章动（回落 77 项/零，见 REPO_AUX_STATUS）。
#[wasm_bindgen]
pub fn compute_year_data_full_binary(
    lunar_year: i32,
    vsop87_ear_bin: &[u8],
    elp_main_s1_bin: &[u8],
    elp_main_s2_bin: &[u8],
    elp_main_s3_bin: &[u8],
    elp_pert_s1_bin: &[u8],
    elp_pert_s2_bin: &[u8],
    elp_pert_s3_bin: &[u8],
    nutation_tab53a_bin: &[u8],
) -> Result<YearDataResult, JsValue> {
    let vsop = with_earth_patch(Vsop87::from_binary(vsop87_ear_bin).map_err(|e| JsValue::from_str(&e.to_string()))?);
    let elp = load_elp_binary([elp_main_s1_bin, elp_main_s2_bin, elp_main_s3_bin, elp_pert_s1_bin, elp_pert_s2_bin, elp_pert_s3_bin])
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let nutation_ok = if !nutation_tab53a_bin.is_empty() {
        nutation::try_init_full_nutation_from_binary(nutation_tab53a_bin)
    } else {
        false
    };
    *REPO_AUX_STATUS.lock().unwrap() = (nutation_ok, earth_patch_loaded());
    let year_data = lunar_core::calendar::chinese_lunar::compute_year_data(
        &vsop,
        &elp,
        lunar_year,
        lunar_core::quantity::angle::PlaneAngle::from_rad(real(1e-8)),
        30,
    )
    .map_err(|e| JsValue::from_str(&e))?;
    Ok(year_data_result(&year_data, &vsop, &elp))
}

/// 加载 VSOP87 地球的 DE406/DE441 拟合改正（`data/fit/vsop87-<目标>-earth.bin`，或解压后的 .br）。前端在计算前调用；
/// 之后构造的地球 VSOP87 都施加此改正，状态栏显示「地球改正: 已加载」。清空 VSOP87 缓存以免沿用未改正的对象。
#[wasm_bindgen]
pub fn init_vsop87_earth_patch_from_binary(bytes: &[u8]) -> bool {
    let Ok(patch) = lunar_core::astronomy::ephemeris::vsop87_patch::Vsop87DePatch::from_binary(bytes) else {
        return false;
    };
    let Some(earth) = patch.body("ear").cloned() else {
        return false;
    };
    *EARTH_PATCH.lock().unwrap() = Some(earth);
    *VSOP87_CACHE.lock().unwrap() = None;
    true
}

/// 上次岁数据计算时的 repo 辅助数据加载状态，供状态栏展示。全二进制路径下章动未从 repo 加载；地球改正若已通过
/// init_vsop87_earth_patch_from_binary 加载则为 true。
#[wasm_bindgen]
pub fn get_repo_aux_nutation_full() -> bool {
    REPO_AUX_STATUS.lock().unwrap().0
}

#[wasm_bindgen]
pub fn get_repo_aux_earth_patch() -> bool {
    REPO_AUX_STATUS.lock().unwrap().1
}

// ---------------------------------------------------------------------------
// 视位置管线：变换图可视化（供前端画架变换图）
// ---------------------------------------------------------------------------

/// 单条架变换边的可视化数据（含步骤标签与边分类）。
#[wasm_bindgen]
#[derive(Clone)]
pub struct GraphEdgeViz {
    from_id: String,
    to_id: String,
    cost: u32,
    label: Option<String>,
    /// 边的概念分类中文标签（标架旋转 / 标架平移 / 光行时 / 等）
    kind_cn: String,
    /// 边的执行形式中文标签（旋转 / 平移 / 映射 / 等）
    form_cn: String,
}

#[wasm_bindgen]
impl GraphEdgeViz {
    #[wasm_bindgen(getter_with_clone)]
    pub fn from_id(&self) -> String {
        self.from_id.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn to_id(&self) -> String {
        self.to_id.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn cost(&self) -> u32 {
        self.cost
    }
    /// 步骤标签（岁差、章动、拟合修正等）；始终返回字符串，避免 Option 过 WASM 边界时在前端变成 undefined。
    #[wasm_bindgen(getter_with_clone)]
    pub fn label(&self) -> String {
        self.label.clone().unwrap_or_else(|| "几何变换".into())
    }
    #[wasm_bindgen(getter_with_clone, js_name = kindCn)]
    pub fn kind_cn(&self) -> String {
        self.kind_cn.clone()
    }
    #[wasm_bindgen(getter_with_clone, js_name = formCn)]
    pub fn form_cn(&self) -> String {
        self.form_cn.clone()
    }
}

/// 节点原点角色（日心/地心/质心），供前端按角色着色或分组。
#[wasm_bindgen]
#[derive(Clone)]
pub struct NodeOriginViz {
    node_id: String,
    origin_cn: String,
}

#[wasm_bindgen]
impl NodeOriginViz {
    #[wasm_bindgen(getter_with_clone, js_name = nodeId)]
    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }
    #[wasm_bindgen(getter_with_clone, js_name = originCn)]
    pub fn origin_cn(&self) -> String {
        self.origin_cn.clone()
    }
}

/// 变换图的可视化数据：节点 id、节点原点角色、边列表，供前端绘图与分类展示。
#[wasm_bindgen]
pub struct TransformGraphViz {
    node_ids: Vec<String>,
    node_origins: Vec<NodeOriginViz>,
    edges: Vec<GraphEdgeViz>,
}

#[wasm_bindgen]
impl TransformGraphViz {
    #[wasm_bindgen(getter_with_clone, js_name = nodeIds)]
    pub fn node_ids(&self) -> Vec<String> {
        self.node_ids.clone()
    }
    #[wasm_bindgen(getter_with_clone, js_name = nodeOrigins)]
    pub fn node_origins(&self) -> Vec<NodeOriginViz> {
        self.node_origins.clone()
    }
    #[wasm_bindgen(getter_with_clone)]
    pub fn edges(&self) -> Vec<GraphEdgeViz> {
        self.edges.clone()
    }
}

/// 返回默认视位置变换图的可视化数据，供 WASM 前端画图（节点=参考架，边=6×6 状态转移）。
#[wasm_bindgen(js_name = transformGraphVisualizationData)]
pub fn transform_graph_visualization_data() -> TransformGraphViz {
    let graph = lunar_core::astronomy::pipeline::TransformGraph::default_graph();
    let data = graph.visualization_data();
    TransformGraphViz {
        node_ids: data.node_ids.clone(),
        node_origins: data
            .node_origins
            .into_iter()
            .map(|(id, r)| NodeOriginViz {
                node_id: id,
                origin_cn: r.label_cn().to_string(),
            })
            .collect(),
        edges: data
            .edges
            .into_iter()
            .map(|e| GraphEdgeViz {
                from_id: e.from_id,
                to_id: e.to_id,
                cost: e.cost,
                label: e.label,
                kind_cn: e.kind.label_cn().to_string(),
                form_cn: e.form.label_cn().to_string(),
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// 日食地图（食带图）
// ---------------------------------------------------------------------------

use lunar_core::astronomy::phenomena::{
    eclipse_window, solar_eclipse_global_in, solar_eclipse_local_in, solar_eclipse_map as core_solar_eclipse_map, EclipseEphemeris,
    InterpolatedSunMoon, MapPoint, SolarEclipseGlobal, SolarEclipseKind, SolarEclipseMapOptions,
};
use lunar_core::quantity::duration::Duration as QDuration;

/// 日月历表常驻对象：构造时加载一次 VSOP87（地球，含已注入的 DE 改正）、ELPMPP02 与章动表，之后反复求日食。
#[wasm_bindgen]
pub struct EclipseEngine {
    vsop: Vsop87,
    elp: Elpmpp02Data,
    /// 最近一次动画用的插值窗（按猜测时刻认），逐帧复用，不必每帧 12 次历表求值
    frame_window: std::cell::RefCell<Option<(f64, InterpolatedSunMoon)>>,
}

fn kind_code(k: SolarEclipseKind) -> &'static str {
    match k {
        SolarEclipseKind::Partial => "P",
        SolarEclipseKind::Annular => "A",
        SolarEclipseKind::Total => "T",
        SolarEclipseKind::Hybrid => "H",
    }
}

fn lunar_kind_code(k: lunar_core::astronomy::phenomena::LunarEclipseKind) -> &'static str {
    use lunar_core::astronomy::phenomena::LunarEclipseKind as K;
    match k {
        K::Penumbral => "N",
        K::Partial => "P",
        K::Total => "T",
    }
}

fn json_f(x: f64) -> String {
    if x.is_finite() {
        format!("{x}")
    } else {
        "null".into()
    }
}

/// 折线 → [[经度°, 纬度°, jd_tt], …]（经纬度取 4 位小数约 10 m）。
fn json_line(line: &[MapPoint]) -> String {
    let pts: Vec<String> = line
        .iter()
        .map(|p| {
            format!(
                "[{:.4},{:.4},{:.6}]",
                p.longitude.rad().as_f64().to_degrees(),
                p.latitude.rad().as_f64().to_degrees(),
                p.jd_tt.as_f64()
            )
        })
        .collect();
    format!("[{}]", pts.join(","))
}

fn json_lines(lines: &[Vec<MapPoint>]) -> String {
    format!("[{}]", lines.iter().map(|l| json_line(l)).collect::<Vec<_>>().join(","))
}

fn json_global(g: &SolarEclipseGlobal) -> String {
    format!(
        "{{\"greatestTt\":{},\"gamma\":{},\"magnitude\":{},\"kind\":\"{}\",\"central\":{}}}",
        json_f(g.greatest_tt.as_f64()),
        json_f(g.gamma.as_f64()),
        json_f(g.magnitude.as_f64()),
        kind_code(g.kind),
        g.central
    )
}

/// ΔT 估计（Morrison–Stephenson 2021，按当前 ELPMPP02 常数集 − DE431 的实测月亮黄经差配套修正，与 examples/historical_eclipses 同）。
fn ms2021_delta_t(jd_tt: f64) -> (f64, f64) {
    use lunar_core::astronomy::time::delta_t::{delta_t_for_lunar_offset, MorrisonStephenson2021};
    let offset = elp_fit().1.lunar_offset_vs_de431().expect("DE406/DE441 常数集均有实测黄经差");
    let e = delta_t_for_lunar_offset(&MorrisonStephenson2021, real(jd_tt), &offset);
    (e.value.seconds().as_f64(), e.sigma.map(|s| s.seconds().as_f64()).unwrap_or(0.0))
}

/// 民用年（UT，1582-10-15 前儒略历）对应的 TT 区间。年表按 UT 显示日期，年界须加 ΔT：古代 ΔT 约数小时，否则岁末的食会归到下一年。
fn civil_year_tt_range(year: i32) -> (lunar_core::math::real::Real, lunar_core::math::real::Real) {
    use lunar_core::calendar::gregorian::civil_to_julian_day;
    let tt = |ut: lunar_core::math::real::Real| {
        let u = ut.as_f64();
        let dt = ms2021_delta_t(u + ms2021_delta_t(u).0 / 86400.0).0;
        real(u + dt / 86400.0)
    };
    (tt(civil_to_julian_day(year, 1, 1)), tt(civil_to_julian_day(year + 1, 1, 1)))
}

/// 海拔 `height_m` 的标准大气；气温（℃）、气压（kPa）给 NaN 时取标准值。`inversion_depth_m` > 0 时加贴地逆温：此厚度内升温 `inversion_rise_k`。
fn site_atmosphere(height_m: f64, temperature_c: f64, pressure_kpa: f64, inversion_depth_m: f64, inversion_rise_k: f64) -> lunar_core::astronomy::correction::Atmosphere {
    use lunar_core::quantity::{length::Length, pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature, unit::LengthUnit};
    let mut a = lunar_core::astronomy::correction::Atmosphere::standard_at_height(Length::from_value(real(height_m), LengthUnit::Meter));
    if temperature_c.is_finite() {
        a.temperature = ThermodynamicTemperature::from_celsius(real(temperature_c));
    }
    if pressure_kpa.is_finite() {
        a.pressure = Pressure::from_kpa(real(pressure_kpa));
    }
    if inversion_depth_m > 0.0 {
        a = a.with_inversion(Length::from_value(real(inversion_depth_m), LengthUnit::Meter), real(inversion_rise_k / inversion_depth_m));
    }
    a
}

/// 地形地平线：near（高分辨率，只定观测点地面海拔）、far（约 250 m 分辨率，算沿途）是瓦片拼成的高程网格（米）。参数依次为 z、左上瓦片 x、y、宽、高（像素）。
/// 返回 [地面海拔（米）, 方位 0°、0.5°… 共 720 个视高度（度）]；观测点无数据时返回空。
#[wasm_bindgen(js_name = terrainHorizon)]
#[allow(clippy::too_many_arguments)]
pub fn terrain_horizon_js(lon_deg: f64, lat_deg: f64, eye_m: f64, near: &[f32], nz: u32, nx: f64, ny: f64, nw: usize, nh: usize, far: &[f32], fz: u32, fx: f64, fy: f64, fw: usize, fh: usize) -> Vec<f64> {
    use lunar_core::astronomy::terrain_horizon::{terrain_horizon, DemMosaic};
    let n = DemMosaic { z: nz, x0: nx, y0: ny, width: nw, height: nh, tile_size: 256.0, data: near };
    let f = DemMosaic { z: fz, x0: fx, y0: fy, width: fw, height: fh, tile_size: 256.0, data: far };
    // 地面海拔用高分辨率；沿途用低分辨率，滤掉现代建筑（见 terrain_horizon 模块）
    let Some(ground) = n.sample(real(lon_deg), real(lat_deg)) else {
        return vec![];
    };
    let h = terrain_horizon(real(lon_deg), real(lat_deg), ground, real(eye_m), 720, 200_000.0, |lon, lat| f.sample(lon, lat));
    std::iter::once(h.ground_m.as_f64()).chain(h.altitude_deg.iter().map(|a| a.as_f64())).collect()
}

/// 海地平的俯角（度，≥ 0）：观测者在海拔处看到的海平线低于高度 0° 多少，已含折射（光线追迹）。
#[wasm_bindgen(js_name = seaHorizonDip)]
pub fn sea_horizon_dip(height_m: f64, temperature_c: f64, pressure_kpa: f64, inversion_depth_m: f64, inversion_rise_k: f64) -> f64 {
    use lunar_core::astronomy::correction::{ray_trace_sea_horizon, RayTraceConstants};
    let atm = site_atmosphere(height_m, temperature_c, pressure_kpa, inversion_depth_m, inversion_rise_k);
    -ray_trace_sea_horizon(&atm, &RayTraceConstants::default()).0.as_f64().to_degrees()
}

/// 各真高度（度）处的折射量（度）：光线追迹（项目缺省模型），大气见 [`site_atmosphere`]。
#[wasm_bindgen(js_name = refractionTable)]
pub fn refraction_table_js(altitudes_deg: Vec<f64>, height_m: f64, temperature_c: f64, pressure_kpa: f64, inversion_depth_m: f64, inversion_rise_k: f64) -> Vec<f64> {
    use lunar_core::astronomy::correction::RefractionModel;
    let alts: Vec<_> = altitudes_deg.iter().map(|&h| real(h)).collect();
    let atm = site_atmosphere(height_m, temperature_c, pressure_kpa, inversion_depth_m, inversion_rise_k);
    lunar_core::astronomy::sky_map::refraction_table(&alts, RefractionModel::RayTrace, &atm).iter().map(|r| r.as_f64()).collect()
}

/// ΔT 估计 [中值, 1σ]（秒）：Morrison–Stephenson 2021，已按本仓库月历表配套修正。
#[wasm_bindgen(js_name = deltaTEstimate)]
pub fn delta_t_estimate(jd_tt: f64) -> Vec<f64> {
    let (v, s) = ms2021_delta_t(jd_tt);
    vec![v, s]
}

impl EclipseEngine {
    /// 取猜测时刻对应的插值窗，换一次日月食才重建。
    fn with_frame_window<R>(&self, jd_tt_guess: f64, f: impl FnOnce(&InterpolatedSunMoon) -> R) -> R {
        let mut slot = self.frame_window.borrow_mut();
        if slot.as_ref().map(|(g, _)| *g) != Some(jd_tt_guess) {
            let eph = EclipseEphemeris::new(&self.vsop, &self.elp);
            *slot = Some((jd_tt_guess, eclipse_window(&eph, real(jd_tt_guess))));
        }
        f(&slot.as_ref().unwrap().1)
    }
}

#[wasm_bindgen]
impl EclipseEngine {
    /// 全二进制加载（与 compute_year_data_full_binary 相同的 7 个历表文件 + 章动表；章动表可为空）。
    #[wasm_bindgen(constructor)]
    pub fn new(
        vsop87_ear_bin: &[u8],
        elp_main_s1_bin: &[u8],
        elp_main_s2_bin: &[u8],
        elp_main_s3_bin: &[u8],
        elp_pert_s1_bin: &[u8],
        elp_pert_s2_bin: &[u8],
        elp_pert_s3_bin: &[u8],
        nutation_tab53a_bin: &[u8],
    ) -> Result<EclipseEngine, JsValue> {
        let vsop = with_earth_patch(Vsop87::from_binary(vsop87_ear_bin).map_err(|e| JsValue::from_str(&e.to_string()))?);
        let elp = load_elp_binary([elp_main_s1_bin, elp_main_s2_bin, elp_main_s3_bin, elp_pert_s1_bin, elp_pert_s2_bin, elp_pert_s3_bin])
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        if !nutation_tab53a_bin.is_empty() {
            nutation::try_init_full_nutation_from_binary(nutation_tab53a_bin);
        }
        Ok(EclipseEngine { vsop, elp, frame_window: std::cell::RefCell::new(None) })
    }

    /// 某民用年（天文纪年；1582-10-15 前儒略历）内的日食列表，JSON 数组：
    /// [{greatestTt, gamma, magnitude, kind: P/A/T/H, central, deltaT, deltaTSigma}, …]。
    #[wasm_bindgen(js_name = solarEclipsesInYear)]
    pub fn solar_eclipses_in_year(&self, year: i32) -> String {
        let eph = EclipseEphemeris::new(&self.vsop, &self.elp);
        let (a, b) = civil_year_tt_range(year);
        let list = lunar_core::astronomy::phenomena::solar_eclipses_between(&eph, a, b);
        let items: Vec<String> = list
            .iter()
            .map(|e| {
                let (dt, sigma) = ms2021_delta_t(e.greatest_tt.as_f64());
                let mut s = json_global(e);
                s.pop();
                s.push_str(&format!(",\"deltaT\":{},\"deltaTSigma\":{}}}", json_f(dt), json_f(sigma)));
                s
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    /// 日食地图，JSON 对象：global、deltaT、p1/p4、centralContacts、greatestPoint、各条界线（折线为 [经°, 纬°, jd_tt] 数组）、
    /// outlines（[{jdTt, penumbra, umbra}]）、lonShiftPerSecond（ΔT 每增 1 s 全图东移的度数）。
    /// `step_s` 扫描步长（秒，缺省 60）；`outline_interval_s` 影子轮廓间隔（秒，≤0 不画）。
    #[wasm_bindgen(js_name = solarEclipseMap)]
    pub fn solar_eclipse_map(&self, jd_tt_guess: f64, delta_t_s: f64, step_s: f64, outline_interval_s: f64) -> Result<String, JsValue> {
        let eph = EclipseEphemeris::new(&self.vsop, &self.elp);
        let win = eclipse_window(&eph, real(jd_tt_guess));
        let opts = SolarEclipseMapOptions {
            step: QDuration::in_seconds(real(if step_s > 0.0 { step_s } else { 60.0 })),
            outline_interval: (outline_interval_s > 0.0).then(|| QDuration::in_seconds(real(outline_interval_s))),
            ..Default::default()
        };
        let m = core_solar_eclipse_map(&win, real(jd_tt_guess), QDuration::in_seconds(real(delta_t_s)), &opts)
            .ok_or_else(|| JsValue::from_str("该朔附近无日食"))?;
        let shift = lunar_core::astronomy::phenomena::longitude_shift_for_delta_t(QDuration::in_seconds(real(1.0)))
            .rad()
            .as_f64()
            .to_degrees();
        let central_contacts = match m.central_contacts {
            Some((a, b)) => format!("[{},{}]", json_f(a.as_f64()), json_f(b.as_f64())),
            None => "null".into(),
        };
        let outlines: Vec<String> = m
            .outlines
            .iter()
            .map(|o| format!("{{\"jdTt\":{},\"penumbra\":{},\"umbra\":{}}}", json_f(o.jd_tt.as_f64()), json_lines(&o.penumbra), json_lines(&o.umbra)))
            .collect();
        let contours: Vec<String> = m
            .magnitude_contours
            .iter()
            .map(|c| format!("{{\"level\":{},\"lines\":{}}}", json_f(c.level.as_f64()), json_lines(&c.lines)))
            .collect();
        Ok(format!(
            "{{\"global\":{},\"deltaT\":{},\"p1\":{},\"p4\":{},\"centralContacts\":{},\"greatestPoint\":{},\"subsolar\":{},\"centralLine\":{},\
\"umbraNorth\":{},\"umbraSouth\":{},\"penumbraLimits\":{},\"riseSet\":{},\"magnitudeContours\":[{}],\"outlines\":[{}],\"lonShiftPerSecond\":{}}}",
            json_global(&m.global),
            json_f(delta_t_s),
            json_f(m.penumbra_contacts.0.as_f64()),
            json_f(m.penumbra_contacts.1.as_f64()),
            central_contacts,
            json_line(&[m.greatest_point]),
            json_line(&[m.subsolar]),
            json_lines(&m.central_line),
            json_lines(&m.umbra_north),
            json_lines(&m.umbra_south),
            json_lines(&m.penumbra_limits),
            json_lines(&m.rise_set),
            contours.join(","),
            outlines.join(","),
            json_f(shift)
        ))
    }

    /// 日食动画的一帧，JSON：jdTt、penumbra、umbra（折线组）、center（[点] 或 []）、subsolar（[点]）。折线同日食图。
    #[wasm_bindgen(js_name = solarEclipseFrame)]
    pub fn solar_eclipse_frame(&self, jd_tt_guess: f64, jd_tt: f64, delta_t_s: f64) -> String {
        let f = self.with_frame_window(jd_tt_guess, |win| {
            lunar_core::astronomy::phenomena::solar_eclipse_frame(win, real(jd_tt), QDuration::in_seconds(real(delta_t_s)), 180)
        });
        let center: Vec<MapPoint> = f.center.into_iter().collect();
        format!(
            "{{\"jdTt\":{},\"penumbra\":{},\"umbra\":{},\"center\":{},\"subsolar\":{}}}",
            json_f(jd_tt), json_lines(&f.outline.penumbra), json_lines(&f.outline.umbra), json_line(&center), json_line(&[f.subsolar])
        )
    }

    /// 月食动画的一帧，JSON：jdTt、sublunar（[点]）、horizon（折线）、east、north（度）、umbralMagnitude、penumbralMagnitude（影外为负）。
    #[wasm_bindgen(js_name = lunarEclipseFrame)]
    pub fn lunar_eclipse_frame(&self, jd_tt_guess: f64, jd_tt: f64, delta_t_s: f64) -> String {
        let f = self.with_frame_window(jd_tt_guess, |win| {
            lunar_core::astronomy::phenomena::lunar_eclipse_frame(win, real(jd_tt), QDuration::in_seconds(real(delta_t_s)), 90)
        });
        let d = |x: lunar_core::math::real::Real| json_f(x.as_f64().to_degrees());
        format!(
            "{{\"jdTt\":{},\"sublunar\":{},\"horizon\":{},\"east\":{},\"north\":{},\"umbralMagnitude\":{},\"penumbralMagnitude\":{}}}",
            json_f(jd_tt), json_line(&[f.sublunar]), json_line(&f.horizon), d(f.offset.east), d(f.offset.north), json_f(f.umbral_magnitude.as_f64()), json_f(f.penumbral_magnitude.as_f64())
        )
    }

    /// 某地某刻看到的月面，JSON（角度为度）：jdTt、alt（几何）、appAlt（含折射）、az、zenithPa（天顶方向位置角）、
    /// libLon、libLat（站心天平动）、axisPa（月轴位置角）、vScale（折射竖直缩放）。
    #[wasm_bindgen(js_name = moonDiskView)]
    pub fn moon_disk_view(&self, jd_tt_guess: f64, jd_tt: f64, delta_t_s: f64, lon_deg: f64, lat_deg: f64) -> String {
        use lunar_core::astronomy::geodesy::Observer;
        use lunar_core::quantity::angle::PlaneAngle;
        use lunar_core::quantity::length::Length;
        use lunar_core::quantity::unit::LengthUnit;
        let obs = Observer::new(PlaneAngle::from_deg(real(lon_deg)), PlaneAngle::from_deg(real(lat_deg)), Length::from_value(real(0.0), LengthUnit::Meter));
        let v = self.with_frame_window(jd_tt_guess, |win| {
            lunar_core::astronomy::phenomena::moon_disk_view_in(win, &obs, real(jd_tt), QDuration::in_seconds(real(delta_t_s)))
        });
        let d = |a: PlaneAngle| json_f(a.rad().as_f64().to_degrees());
        format!(
            "{{\"jdTt\":{},\"alt\":{},\"appAlt\":{},\"az\":{},\"zenithPa\":{},\"libLon\":{},\"libLat\":{},\"axisPa\":{},\"vScale\":{}}}",
            json_f(jd_tt), d(v.altitude), d(v.apparent_altitude), d(v.azimuth), d(v.zenith_position_angle),
            d(v.libration_longitude), d(v.libration_latitude), d(v.axis_position_angle), json_f(v.vertical_scale.as_f64())
        )
    }

    /// 某民用年内的月食列表，JSON 数组：[{greatestTt, gamma, penumbralMagnitude, umbralMagnitude, kind: N/P/T, deltaT, deltaTSigma}, …]。
    #[wasm_bindgen(js_name = lunarEclipsesInYear)]
    pub fn lunar_eclipses_in_year(&self, year: i32) -> String {
        let eph = EclipseEphemeris::new(&self.vsop, &self.elp);
        let (a, b) = civil_year_tt_range(year);
        let list = lunar_core::astronomy::phenomena::lunar_eclipses_between(&eph, a, b);
        let items: Vec<String> = list
            .iter()
            .map(|e| {
                let t = e.greatest_tt.as_f64();
                let (dt, sigma) = ms2021_delta_t(t);
                format!(
                    "{{\"greatestTt\":{},\"gamma\":{},\"penumbralMagnitude\":{},\"umbralMagnitude\":{},\"kind\":\"{}\",\"deltaT\":{},\"deltaTSigma\":{}}}",
                    json_f(t), json_f(e.gamma.as_f64()), json_f(e.penumbral_magnitude.as_f64()), json_f(e.umbral_magnitude.as_f64()), lunar_kind_code(e.kind), json_f(dt), json_f(sigma)
                )
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    /// 月食图，JSON：eclipse、deltaT、curves（[{phase, jdTt, sublunar, horizon, east°, north°}]，折线同日食图）、
    /// umbraRadius、penumbraRadius、moonSemidiameter（度）、shadowPath（[[向东°, 向北°, jd_tt]…]）、lonShiftPerSecond。
    #[wasm_bindgen(js_name = lunarEclipseMap)]
    pub fn lunar_eclipse_map(&self, jd_tt_guess: f64, delta_t_s: f64) -> Result<String, JsValue> {
        let eph = EclipseEphemeris::new(&self.vsop, &self.elp);
        let win = eclipse_window(&eph, real(jd_tt_guess));
        let m = lunar_core::astronomy::phenomena::lunar_eclipse_map(&win, real(jd_tt_guess), QDuration::in_seconds(real(delta_t_s)), 180)
            .ok_or_else(|| JsValue::from_str("该望附近无月食"))?;
        let d = |x: lunar_core::math::real::Real| json_f(x.as_f64().to_degrees());
        let curves: Vec<String> = m
            .curves
            .iter()
            .map(|c| {
                format!(
                    "{{\"phase\":\"{}\",\"jdTt\":{},\"sublunar\":{},\"horizon\":{},\"east\":{},\"north\":{}}}",
                    c.phase.code(), json_f(c.jd_tt.as_f64()), json_line(&[c.sublunar]), json_line(&c.horizon), d(c.offset.east), d(c.offset.north)
                )
            })
            .collect();
        let path: Vec<String> = m.shadow_path.iter().map(|p| format!("[{},{},{}]", d(p.east), d(p.north), json_f(p.jd_tt.as_f64()))).collect();
        let shift = lunar_core::astronomy::phenomena::longitude_shift_for_delta_t(QDuration::in_seconds(real(1.0))).rad().as_f64().to_degrees();
        let e = &m.eclipse;
        Ok(format!(
            "{{\"eclipse\":{{\"greatestTt\":{},\"gamma\":{},\"penumbralMagnitude\":{},\"umbralMagnitude\":{},\"kind\":\"{}\"}},\"deltaT\":{},\"curves\":[{}],\"umbraRadius\":{},\"penumbraRadius\":{},\"moonSemidiameter\":{},\"shadowPath\":[{}],\"lonShiftPerSecond\":{}}}",
            json_f(e.greatest_tt.as_f64()), json_f(e.gamma.as_f64()), json_f(e.penumbral_magnitude.as_f64()), json_f(e.umbral_magnitude.as_f64()), lunar_kind_code(e.kind),
            json_f(delta_t_s), curves.join(","), d(m.umbra_radius), d(m.penumbra_radius), d(m.moon_semidiameter), path.join(","), json_f(shift)
        ))
    }

    /// 某地月食情况，JSON：phases（[{phase, jdTt, alt°}]）、visibleFrom、visibleTo、maxVisibleUmbral、maxVisiblePenumbral；无月食返回 "null"。
    #[wasm_bindgen(js_name = lunarEclipseLocal)]
    pub fn lunar_eclipse_local(&self, jd_tt_guess: f64, delta_t_s: f64, lon_deg: f64, lat_deg: f64) -> String {
        use lunar_core::astronomy::geodesy::Observer;
        use lunar_core::astronomy::phenomena::{lunar_eclipse_in, lunar_eclipse_local_in};
        use lunar_core::quantity::angle::PlaneAngle;
        use lunar_core::quantity::length::Length;
        use lunar_core::quantity::unit::LengthUnit;
        let eph = EclipseEphemeris::new(&self.vsop, &self.elp);
        let win = eclipse_window(&eph, real(jd_tt_guess));
        let Some(e) = lunar_eclipse_in(&win, real(jd_tt_guess)) else { return "null".into() };
        let obs = Observer::new(PlaneAngle::from_deg(real(lon_deg)), PlaneAngle::from_deg(real(lat_deg)), Length::from_value(real(0.0), LengthUnit::Meter));
        let l = lunar_eclipse_local_in(&win, &e, &obs, QDuration::in_seconds(real(delta_t_s)));
        let phases: Vec<String> = l
            .phases
            .iter()
            .map(|(p, t, a)| format!("{{\"phase\":\"{}\",\"jdTt\":{},\"alt\":{}}}", p.code(), json_f(t.as_f64()), json_f(a.rad().as_f64().to_degrees())))
            .collect();
        let opt = |x: Option<lunar_core::math::real::Real>| x.map(|v| json_f(v.as_f64())).unwrap_or_else(|| "null".into());
        format!(
            "{{\"phases\":[{}],\"visibleFrom\":{},\"visibleTo\":{},\"maxVisibleUmbral\":{},\"maxVisiblePenumbral\":{}}}",
            phases.join(","), opt(l.visible.map(|v| v.0)), opt(l.visible.map(|v| v.1)), opt(l.max_visible_umbral), opt(l.max_visible_penumbral)
        )
    }

    /// 某地日食局地情况，JSON；无食返回 "null"。时刻为 TT，太阳高度为站心几何高度（度）。
    #[wasm_bindgen(js_name = solarEclipseLocal)]
    pub fn solar_eclipse_local(&self, jd_tt_guess: f64, delta_t_s: f64, lon_deg: f64, lat_deg: f64) -> String {
        use lunar_core::astronomy::geodesy::Observer;
        use lunar_core::quantity::angle::PlaneAngle;
        use lunar_core::quantity::length::Length;
        use lunar_core::quantity::unit::LengthUnit;
        let eph = EclipseEphemeris::new(&self.vsop, &self.elp);
        let win = eclipse_window(&eph, real(jd_tt_guess));
        let obs = Observer::new(
            PlaneAngle::from_deg(real(lon_deg)),
            PlaneAngle::from_deg(real(lat_deg)),
            Length::from_value(real(0.0), LengthUnit::Meter),
        );
        let Some(e) = solar_eclipse_local_in(&win, &obs, real(jd_tt_guess), QDuration::in_seconds(real(delta_t_s))) else {
            return "null".into();
        };
        let opt = |x: Option<lunar_core::math::real::Real>| x.map(|v| json_f(v.as_f64())).unwrap_or_else(|| "null".into());
        let deg = |a: PlaneAngle| json_f(a.rad().as_f64().to_degrees());
        let (vis_a, vis_b) = match e.visible {
            Some((a, b)) => (json_f(a.as_f64()), json_f(b.as_f64())),
            None => ("null".into(), "null".into()),
        };
        format!(
            "{{\"first\":{},\"second\":{},\"greatest\":{},\"third\":{},\"last\":{},\"magnitude\":{},\"altFirst\":{},\"altGreatest\":{},\
\"altLast\":{},\"visibleFrom\":{},\"visibleTo\":{},\"maxVisibleMagnitude\":{}}}",
            json_f(e.first_contact.as_f64()),
            opt(e.second_contact),
            json_f(e.greatest.as_f64()),
            opt(e.third_contact),
            json_f(e.last_contact.as_f64()),
            json_f(e.magnitude.as_f64()),
            deg(e.sun_altitude_first),
            deg(e.sun_altitude_greatest),
            deg(e.sun_altitude_last),
            vis_a,
            vis_b,
            opt(e.max_visible_magnitude)
        )
    }
}

// ---------------------------------------------------------------------------
// 星图
// ---------------------------------------------------------------------------

use lunar_core::astronomy::pipeline::{Body, Planet, Vsop87Elp, Vsop87Planets};
use lunar_core::astronomy::sky_culture::chinese::{
    parse_asterisms, parse_lunar_mansion_systems, parse_star_names, Asterism, AsterismMember, LunarMansionSystem, StarName,
};
use lunar_core::astronomy::stars::StarCatalog;

const NAKED_EYE_PLANETS: [Planet; 5] = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn];

/// 星图常驻对象：日月五星历表、星表、各体系星官与星名、二十八宿距星。数据由宿主 fetch 后传入，直接用各自的解析函数
/// （即 `*_from_repo()` 内部所用者）解析，不经全局 repo loader（后者只能设置一次，与本页之外的用途冲突）。
#[wasm_bindgen]
pub struct SkyEngine {
    earth: Vsop87,
    planets: Vec<(Planet, Vsop87)>,
    elp: Elpmpp02Data,
    catalog: StarCatalog,
    asterisms: Vec<Asterism>,
    names: Vec<StarName>,
    mansions: Vec<LunarMansionSystem>,
    periods: Vec<lunar_core::astronomy::sky_culture::periods::SystemPeriod>,
    schools: Vec<lunar_core::astronomy::sky_culture::chinese::SchoolAttribution>,
}

fn text_lines(s: &str) -> Vec<String> {
    s.lines().map(String::from).collect()
}

fn body_name(b: Body) -> &'static str {
    match b {
        Body::Sun => "sun",
        Body::Moon => "moon",
        Body::Planet(Planet::Mercury) => "mercury",
        Body::Planet(Planet::Venus) => "venus",
        Body::Planet(Planet::Mars) => "mars",
        Body::Planet(Planet::Jupiter) => "jupiter",
        Body::Planet(Planet::Saturn) => "saturn",
        Body::Planet(Planet::Uranus) => "uranus",
        Body::Planet(Planet::Neptune) => "neptune",
    }
}

fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

#[wasm_bindgen]
impl SkyEngine {
    /// `planet_texts`：VSOP87B 的 mer、ven、mar、jup、sat 文本，以 `\u{1e}` 连接；`planet_patch_text`：
    /// `data/fit/vsop87-de406-planets.txt`（可为空，则行星不施改正）。其余为 data/ 下各 CSV 原文（末两项为体系年代表、三家分属表）。
    #[wasm_bindgen(constructor)]
    pub fn new(
        vsop87_ear_bin: &[u8],
        elp_main_s1_bin: &[u8],
        elp_main_s2_bin: &[u8],
        elp_main_s3_bin: &[u8],
        elp_pert_s1_bin: &[u8],
        elp_pert_s2_bin: &[u8],
        elp_pert_s3_bin: &[u8],
        nutation_tab53a_bin: &[u8],
        planet_texts: &str,
        planet_patch_text: &str,
        stars_csv: &str,
        asterisms_csv: &str,
        star_names_csv: &str,
        mansions_csv: &str,
        periods_csv: &str,
        schools_csv: &str,
    ) -> Result<SkyEngine, JsValue> {
        let err = |e: String| JsValue::from_str(&e);
        let earth = with_earth_patch(Vsop87::from_binary(vsop87_ear_bin).map_err(|e| err(e.to_string()))?);
        let elp = load_elp_binary([elp_main_s1_bin, elp_main_s2_bin, elp_main_s3_bin, elp_pert_s1_bin, elp_pert_s2_bin, elp_pert_s3_bin])
            .map_err(|e| err(e.to_string()))?;
        if !nutation_tab53a_bin.is_empty() {
            nutation::try_init_full_nutation_from_binary(nutation_tab53a_bin);
        }
        let patch = if planet_patch_text.is_empty() {
            None
        } else {
            Some(lunar_core::astronomy::ephemeris::vsop87_patch::Vsop87DePatch::parse(planet_patch_text).map_err(|e| err(e.to_string()))?)
        };
        let texts: Vec<&str> = planet_texts.split('\u{1e}').collect();
        if texts.len() != NAKED_EYE_PLANETS.len() {
            return Err(err(format!("行星历表应为 {} 份，得 {}", NAKED_EYE_PLANETS.len(), texts.len())));
        }
        let mut planets = Vec::new();
        for (p, t) in NAKED_EYE_PLANETS.iter().zip(texts) {
            let mut v = lunar_core::astronomy::ephemeris::Vsop87Parse::parse_from_lines(&text_lines(t)).map_err(|e| err(e.to_string()))?;
            if let Some(lbr) = patch.as_ref().and_then(|pa| pa.body(p.vsop87_suffix())) {
                v = v.with_patch(lbr.clone());
            }
            planets.push((*p, v));
        }
        Ok(SkyEngine {
            earth,
            planets,
            elp,
            catalog: StarCatalog::parse_hipparcos_bright(&text_lines(stars_csv)).map_err(err)?,
            asterisms: parse_asterisms(&text_lines(asterisms_csv)).map_err(err)?,
            names: parse_star_names(&text_lines(star_names_csv)).map_err(err)?,
            mansions: parse_lunar_mansion_systems(&text_lines(mansions_csv)).map_err(err)?,
            periods: lunar_core::astronomy::sky_culture::periods::parse_system_periods(&text_lines(periods_csv)).map_err(err)?,
            schools: lunar_core::astronomy::sky_culture::chinese::parse_school_attributions(&text_lines(schools_csv)).map_err(err)?,
        })
    }

    /// 星官、星名、距星（静态数据），JSON：{asterisms:[{system,id,name,det,lines:[[hip…]…]}], names:[[system,hip,name]…],
    /// mansions:[{system,determinatives:[28 个 hip]}], schools:[{id,school,basis,source}]（陈卓体系三家分属）}。连线只保留 Hipparcos 星（深空天体、盖亚星不在星表中）。
    #[wasm_bindgen(js_name = skyCulture)]
    pub fn sky_culture(&self) -> String {
        let asterisms: Vec<String> = self
            .asterisms
            .iter()
            .map(|a| {
                let lines: Vec<String> = a
                    .lines
                    .iter()
                    .map(|l| {
                        let hips: Vec<String> =
                            l.iter().filter_map(|m| if let AsterismMember::Hip(h) = m { Some(h.to_string()) } else { None }).collect();
                        format!("[{}]", hips.join(","))
                    })
                    .collect();
                format!(
                    "{{\"system\":{},\"id\":{},\"name\":{},\"det\":{},\"lines\":[{}]}}",
                    json_str(&a.system),
                    json_str(&a.id),
                    json_str(&a.name),
                    a.determinative.map(|d| d.to_string()).unwrap_or_else(|| "null".into()),
                    lines.join(",")
                )
            })
            .collect();
        let names: Vec<String> = self.names.iter().map(|n| format!("[{},{},{}]", json_str(&n.system), n.hip, json_str(&n.name))).collect();
        let mansions: Vec<String> = self
            .mansions
            .iter()
            .map(|m| {
                let d: Vec<String> = m.determinatives.iter().map(|h| h.to_string()).collect();
                format!("{{\"system\":{},\"determinatives\":[{}]}}", json_str(&m.id), d.join(","))
            })
            .collect();
        let schools: Vec<String> = self
            .schools
            .iter()
            .map(|a| {
                format!(
                    "{{\"id\":{},\"school\":{},\"basis\":{},\"source\":{}}}",
                    json_str(&a.id),
                    a.school.map(|s| json_str(s.name())).unwrap_or_else(|| "null".into()),
                    json_str(&a.basis),
                    json_str(&a.source)
                )
            })
            .collect();
        format!(
            "{{\"asterisms\":[{}],\"names\":[{}],\"mansions\":[{}],\"schools\":[{}]}}",
            asterisms.join(","),
            names.join(","),
            mansions.join(","),
            schools.join(",")
        )
    }

    /// 某年（天文纪年）适用的星官体系，JSON：{system, epoch, basis}；见 `data/sky_culture/system_periods.csv`。
    #[wasm_bindgen(js_name = systemForYear)]
    pub fn system_for_year(&self, year: i32) -> String {
        match lunar_core::astronomy::sky_culture::periods::system_for_year(&self.periods, year) {
            Some(p) => format!("{{\"system\":{},\"epoch\":{},\"basis\":{}}}", json_str(&p.system), json_str(&p.epoch), json_str(&p.basis)),
            None => "null".into(),
        }
    }

    /// 天空快照，JSON：{jdTt, gastDeg, eclipticPole:[赤经°, 赤纬°], stars:[hip, 赤经°, 赤纬°, V, B−V|null, …]（扁平），
    /// bodies:[{name, ra, dec, mag, phase}], mansionWidths:{体系: [28 个宿度（度，360° 制）]}}。坐标为当日真赤道与真春分点。
    /// bodies 另带 topoRa、topoDec、semidiameter（度，行星为 null）、topoDistM：所给地点的站心位置、视半径与距离，地平图和 3D 视图用。
    /// skyLight：{lgB, lim, sunVisible}，全天亮度与极限星等网格（先高度后方位）；trueToGalactic：当日赤道 → 银道矩阵（按行）；3D 视图用。
    /// 气温、气压给 NaN 时取标准值；`inversion_*` 见 [`site_atmosphere`]；`humidity` 为 0—1；`city_nl` 为城市光（nL）。
    /// `delta_t_s` 只用于恒星时（GAST）。
    #[wasm_bindgen]
    /// 七政四余（排盘页，按需经星图 Worker 调用）：十一曜视黄经、中气分宫、黄道宿度（清代距星）、命宫与十二宫。
    /// `luo_asc` 罗睺为升交点（清初之法；缺省 false 即计都升交）；`mean_node` 平交点（缺省真交点）；
    /// `ascendant` 命宫取上升点（缺省「卯上起命」）。`flags` 星命诸说（缺省 0）：位 0—1 分宫（0 中气、1 星宗宿度、2 大成宿度），
    /// 位 2 命主用宫主、位 3 身主用宫主、位 4 庙旺依「廟樂旺」节、位 5 强弱依星宗、位 6 大限年依大成卷三例。见 doc/folk/qizheng.md。
    pub fn qizheng(&self, jd_tt: f64, delta_t_s: f64, lon_deg: f64, lat_deg: f64, hour_zhi: u8, luo_asc: bool, mean_node: bool, ascendant: bool, flags: u32) -> String {
        use lunar_core::astronomy::apparent::ApparentPipelineOptions;
        use lunar_core::astronomy::sky_culture::chinese::MansionFrame;
        use lunar_core::astronomy::time::{TimePoint, TimeScale};
        use lunar_core::folk::qizheng as qz;
        let eph = Vsop87Elp {
            planets: Vsop87Planets { earth: &self.earth, planets: self.planets.iter().map(|(p, v)| (*p, v)).collect() },
            moon: &self.elp,
        };
        let opts = ApparentPipelineOptions::default();
        let t = TimePoint::new(TimeScale::TT, real(jd_tt));
        let frame = self.mansions.iter().find(|m| m.id == "qing").and_then(|sys| MansionFrame::new(&eph, &self.catalog, sys, t, &opts).ok());
        let o = qz::QizhengOptions {
            luo_ji: if luo_asc { qz::LuoJi::LuoAscending } else { qz::LuoJi::JiAscending },
            node: if mean_node { qz::NodeKind::Mean } else { qz::NodeKind::True },
            ming: if ascendant { qz::MingRule::Ascendant } else { qz::MingRule::SunOnHour },
            gong: match flags & 3 { 1 => qz::GongRule::XiuDuXingZong, 2 => qz::GongRule::XiuDuDaCheng, _ => qz::GongRule::ZhongQi },
            ming_zhu: if flags & 4 != 0 { qz::MingZhuRule::GongZhu } else { qz::MingZhuRule::DuZhu },
            shen_zhu: if flags & 8 != 0 { qz::ShenZhuRule::GongZhu } else { qz::ShenZhuRule::DuZhu },
            miao_wang: if flags & 16 != 0 { qz::MiaoWangRule::Jie } else { qz::MiaoWangRule::Lun },
            qiang_ruo: if flags & 32 != 0 { qz::QiangRuoRule::XingZong } else { qz::QiangRuoRule::DaCheng },
            da_xian: if flags & 64 != 0 { qz::DaXianYears::DaChengLi } else { qz::DaXianYears::XingZong },
            ..Default::default()
        };
        let jd_ut1 = jd_tt - delta_t_s / 86400.0;
        let gast = lunar_core::astronomy::time::earth_rotation::gast(real(jd_ut1), real(jd_tt));
        let lst = (gast.rad().as_f64().to_degrees() + lon_deg).rem_euclid(360.0);
        let c = qz::chart(&eph, t, frame.as_ref(), hour_zhi % 12, Some((real(lst), real(lat_deg))), &o, &opts);
        let yao: Vec<String> = c
            .yao
            .iter()
            .map(|y| {
                format!(
                    "{{\"n\":{},\"lon\":{:.4},\"gong\":{},\"gongDeg\":{:.4},\"xiu\":{},\"xiuDeg\":{},\"retro\":{}}}",
                    json_str(y.name),
                    y.lon.as_f64(),
                    y.gong,
                    y.gong_deg.as_f64(),
                    y.xiu.map(|(k, _)| json_str(qz::xiu_name(k))).unwrap_or_else(|| "null".into()),
                    y.xiu.map(|(_, d)| format!("{:.4}", d.as_f64())).unwrap_or_else(|| "null".into()),
                    y.retro
                )
            })
            .collect();
        let palaces: Vec<String> = c.palaces.iter().map(|x| json_str(x)).collect();
        let x = &c.xing_ming;
        let nm = |i: usize| json_str(qz::YAO.get(i).copied().unwrap_or("?"));
        let opt_nm = |o: Option<usize>| o.map(nm).unwrap_or_else(|| "null".into());
        let strs = |v: &[&str]| format!("[{}]", v.iter().map(|t| json_str(t)).collect::<Vec<_>>().join(","));
        let xm = format!(
            "{{\"mingLon\":{:.4},\"mingXiu\":{},\"mingXiuDeg\":{},\"mingGongZhu\":{},\"mingDuZhu\":{},\"mingZhu\":{},\"shenGong\":{},\"shenGongZhu\":{},\"shenDuZhu\":{},\"shenZhu\":{},\"gongZhu\":[{}],\"dignity\":[{}],\"strong\":[{}],\"relation\":{},\"tongXian\":{:.2},\"daXian\":[{}]}}",
            x.ming_lon.as_f64(),
            x.ming_xiu.map(|(k, _)| json_str(qz::xiu_name(k))).unwrap_or_else(|| "null".into()),
            x.ming_xiu.map(|(_, d)| format!("{:.4}", d.as_f64())).unwrap_or_else(|| "null".into()),
            nm(x.ming_gong_zhu),
            opt_nm(x.ming_du_zhu),
            nm(x.ming_zhu),
            x.shen_gong,
            nm(x.shen_gong_zhu),
            opt_nm(x.shen_du_zhu),
            nm(x.shen_zhu),
            x.gong_zhu.iter().map(|&i| nm(i)).collect::<Vec<_>>().join(","),
            x.dignity.iter().map(|d| strs(d)).collect::<Vec<_>>().join(","),
            x.strong.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","),
            strs(&x.relation),
            x.tong_xian.as_f64(),
            x.da_xian
                .iter()
                .map(|d| format!("{{\"gong\":{},\"palace\":{},\"from\":{:.1},\"to\":{:.1}}}", d.gong, json_str(d.palace), d.from.as_f64(), d.to.as_f64()))
                .collect::<Vec<_>>()
                .join(",")
        );
        // 画盘用：二十八宿距星视黄经、十二宫宫首黄经（与分宫法同）
        let mlon = frame.as_ref().map(|f| qz::mansion_longitudes(f, t));
        let jie = match (o.gong, mlon.is_some()) {
            (qz::GongRule::XiuDuXingZong, true) => Some(&qz::GONG_JIE_XINGZONG),
            (qz::GongRule::XiuDuDaCheng, true) => Some(&qz::GONG_JIE_DACHENG),
            _ => None,
        };
        let starts = qz::gong_starts(jie, mlon.as_ref());
        let fmt_arr = |v: &[lunar_core::math::real::Real]| v.iter().map(|x| format!("{:.4}", x.as_f64())).collect::<Vec<_>>().join(",");
        format!(
            "{{\"yao\":[{}],\"ming\":{},\"palaces\":[{}],\"xingMing\":{},\"xiuLon\":{},\"gongStart\":[{}]}}",
            yao.join(","),
            c.ming,
            palaces.join(","),
            xm,
            mlon.map(|m| format!("[{}]", fmt_arr(&m))).unwrap_or_else(|| "null".into()),
            fmt_arr(&starts)
        )
    }

    pub fn snapshot(&self, jd_tt: f64, delta_t_s: f64, lon_deg: f64, lat_deg: f64, height_m: f64, temperature_c: f64, pressure_kpa: f64, inversion_depth_m: f64, inversion_rise_k: f64, humidity: f64, city_nl: f64) -> String {
        use lunar_core::astronomy::apparent::ApparentPipelineOptions;
        use lunar_core::astronomy::sky_map::sky_snapshot;
        use lunar_core::astronomy::time::{TimePoint, TimeScale};
        let eph = Vsop87Elp {
            planets: Vsop87Planets { earth: &self.earth, planets: self.planets.iter().map(|(p, v)| (*p, v)).collect() },
            moon: &self.elp,
        };
        let opts = ApparentPipelineOptions::default();
        let t = TimePoint::new(TimeScale::TT, real(jd_tt));
        let snap = sky_snapshot(&eph, &self.catalog, t, &opts);
        let deg = |a: lunar_core::quantity::angle::PlaneAngle| a.rad().as_f64().to_degrees();
        let mut stars = String::with_capacity(snap.stars.len() * 40);
        for (k, s) in snap.stars.iter().enumerate() {
            if k > 0 {
                stars.push(',');
            }
            stars.push_str(&format!(
                "{},{:.5},{:.5},{:.2},{}",
                s.hip,
                deg(s.ra).rem_euclid(360.0),
                deg(s.dec),
                s.vmag.as_f64(),
                s.bv.map(|b| format!("{:.2}", b.as_f64())).unwrap_or_else(|| "null".into())
            ));
        }
        let jd_ut1 = jd_tt - delta_t_s / 86400.0;
        let gast = lunar_core::astronomy::time::earth_rotation::gast(real(jd_ut1), real(jd_tt));
        let observer = lunar_core::astronomy::geodesy::Observer::new(
            lunar_core::quantity::angle::PlaneAngle::from_deg(real(lon_deg)),
            lunar_core::quantity::angle::PlaneAngle::from_deg(real(lat_deg)),
            lunar_core::quantity::length::Length::from_value(real(height_m), lunar_core::quantity::unit::LengthUnit::Meter),
        );
        let topo = snap.topocentric_places(&observer, gast);
        let bodies: Vec<String> = snap
            .bodies
            .iter()
            .zip(&topo)
            .map(|(b, tp)| {
                format!(
                    "{{\"name\":\"{}\",\"ra\":{:.5},\"dec\":{:.5},\"topoRa\":{:.5},\"topoDec\":{:.5},\"semidiameter\":{},\"topoDistM\":{},\"mag\":{},\"phase\":{}}}",
                    body_name(b.body),
                    deg(b.ra).rem_euclid(360.0),
                    deg(b.dec),
                    deg(tp.ra).rem_euclid(360.0),
                    deg(tp.dec),
                    tp.semidiameter.map(|a| json_f(deg(a))).unwrap_or_else(|| "null".into()),
                    json_f(tp.distance_m.as_f64()),
                    b.magnitude.map(|m| json_f(m.as_f64())).unwrap_or_else(|| "null".into()),
                    b.phase_angle.map(|a| json_f(deg(a))).unwrap_or_else(|| "null".into())
                )
            })
            .collect();
        let widths: Vec<String> = self
            .mansions
            .iter()
            .filter_map(|m| {
                let w = snap.mansion_frame(m)?.widths(real(360.0));
                let w: Vec<String> = w.iter().map(|x| format!("{:.4}", x.as_f64())).collect();
                Some(format!("{}:[{}]", json_str(&m.id), w.join(",")))
            })
            .collect();
        let atm = site_atmosphere(height_m, temperature_c, pressure_kpa, inversion_depth_m, inversion_rise_k);
        let mut conditions = lunar_core::astronomy::phenomena::visibility::SkyConditions::schaefer1998(real(humidity), atm.temperature.celsius());
        conditions.city_nl = real(city_nl);
        let to_gal = lunar_core::astronomy::sky_map::true_equator_to_galactic(t);
        let to_gal: Vec<String> = to_gal.iter().flatten().map(|x| json_f(x.as_f64())).collect();
        let light = snap.sky_light(&topo, &observer, gast, &atm, &conditions, &lunar_core::astronomy::phenomena::visibility::Eye::default());
        let grid = |v: &[lunar_core::math::real::Real]| v.iter().map(|x| format!("{:.3}", x.as_f64())).collect::<Vec<_>>().join(",");
        let sky_light = format!(
            "{{\"lgB\":[{}],\"lim\":[{}],\"sunVisible\":{},\"moonGlare\":{},\"sunGlare\":{},\"limitCurve\":[{}],\"coronaUnit\":{},\"extinctionK\":{}}}",
            grid(&light.log10_brightness_nl),
            grid(&light.limiting_magnitude),
            json_f(light.sun_visible_fraction.as_f64()),
            json_f(light.moon_glare_nl_deg2.as_f64()),
            json_f(light.sun_glare_nl_deg2.as_f64()),
            grid(&light.limit_curve),
            json_f(light.corona_unit_nl.as_f64()),
            json_f(light.zenith_extinction.as_f64())
        );
        format!(
            "{{\"trueToGalactic\":[{}],\"skyLight\":{},\"jdTt\":{},\"gastDeg\":{},\"eclipticPole\":[{},{}],\"stars\":[{}],\"bodies\":[{}],\"mansionWidths\":{{{}}}}}",
            to_gal.join(","),
            sky_light,
            json_f(jd_tt),
            json_f(deg(gast)),
            json_f(deg(snap.ecliptic_pole.0)),
            json_f(deg(snap.ecliptic_pole.1)),
            stars,
            bodies.join(","),
            widths.join(",")
        )
    }
}

/// 磁偏角（度，东偏为正）：WMM2025（`astronomy::geomag`），大地纬度、经度（度）、椭球高（km）、十进制年。
/// 网页罗盘以此把磁方位换为真方位（真 = 磁 + D）。有效期 2025.0—2030.0 之外返回 NaN。
#[wasm_bindgen]
pub fn magnetic_declination_wasm(lat_deg: f64, lon_deg: f64, alt_km: f64, decimal_year: f64) -> f64 {
    use lunar_core::astronomy::geomag::Wmm;
    static WMM: once_cell::sync::Lazy<Wmm> = once_cell::sync::Lazy::new(Wmm::embedded);
    WMM.field(lat_deg, lon_deg, alt_km, decimal_year).map(|f| f.decl_deg.as_f64()).unwrap_or(f64::NAN)
}
