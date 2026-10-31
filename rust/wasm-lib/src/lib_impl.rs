use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use lunar_core::calendar::era_gaiyuan::GaiyuanDays;
use lunar_core::calendar::era_names::EraEntry;
use lunar_core::math::real::{real, RealOps};
use lunar_core::platform::{DataLoader, LoadError};
use lunar_core::astronomy::ephemeris::{load_all, load_all_from_binary, load_earth_vsop87_from_repo, Elpmpp02Correction, Vsop87};
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
        static PERIODS: MainCalendarPeriods =
            MainCalendarPeriods::from_csv(include_str!("../../../data/calendar/汇总/main_calendar_periods.csv")).unwrap_or_default();
    }
    PERIODS.with(|p| p.name_for_lunar_year(lunar_year).to_string())
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
        esc(&e.emperor_simp),
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
    let resolved_entry = EraEntry {
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

fn options_from_u8s(
    year_b: u8,
    month_b: u8,
    leap_b: u8,
    day_b: u8,
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
        ..opts
    };
    let (yi, mi, di) = lunar_core::calendar::gan_zhi_options::ganzhi_from_jd_solar(vsop.as_ref(), jd, &opts);
    Ok(GanzhiResult {
        year_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(yi).to_string(),
        month_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(mi).to_string(),
        day_name: lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name(di).to_string(),
    })
}

/// 预设名称，供 UI 显示：0=子平八字 1=紫微斗数 2=民俗黄历 3=协纪辨方书
#[wasm_bindgen]
pub fn ganzhi_preset_name(preset_index: u8) -> String {
    match preset_index {
        0 => "子平八字",
        1 => "紫微斗数",
        2 => "民俗黄历",
        3 => "协纪辨方书",
        _ => "民俗黄历",
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
    let opts = options_from_u8s(year_boundary, month_boundary, leap_month_handling, day_boundary);
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
) -> MonthGanzhiResult {
    use lunar_core::calendar::gan_zhi_options::gan_zhi_index_to_name;
    let days = lunar_core::calendar::gregorian::Gregorian::days_in_month(year, month) as i32;
    let opts = options_from_u8s(year_boundary, month_boundary, leap_month_handling, day_boundary);
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
}

#[wasm_bindgen]
impl YearDataResult {
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
    let elp = load_all(lunar_core::repo::get_loader().unwrap(), ELP_BASE, Elpmpp02Correction::DE406)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let year_data = lunar_core::calendar::chinese_lunar::compute_year_data(
        &vsop,
        &elp,
        lunar_year,
        lunar_core::quantity::angle::PlaneAngle::from_rad(real(1e-8)),
        30,
    )
    .map_err(|e| JsValue::from_str(&e))?;
    Ok(YearDataResult {
        lunar_year: year_data.lunar_year,
        new_moon_jds: year_data.new_moon_jds.iter().map(|r| r.as_f64()).collect(),
        zhong_qi_jds: year_data.zhong_qi_jds.iter().map(|r| r.as_f64()).collect(),
    })
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
    let elp = load_all(lunar_core::repo::get_loader().unwrap(), ELP_BASE, Elpmpp02Correction::DE406)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let year_data = lunar_core::calendar::chinese_lunar::compute_year_data(
        &vsop,
        &elp,
        lunar_year,
        lunar_core::quantity::angle::PlaneAngle::from_rad(real(1e-8)),
        30,
    )
    .map_err(|e| JsValue::from_str(&e))?;
    Ok(YearDataResult {
        lunar_year: year_data.lunar_year,
        new_moon_jds: year_data.new_moon_jds.iter().map(|r| r.as_f64()).collect(),
        zhong_qi_jds: year_data.zhong_qi_jds.iter().map(|r| r.as_f64()).collect(),
    })
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
    let elp = load_all_from_binary(
        elp_main_s1_bin,
        elp_main_s2_bin,
        elp_main_s3_bin,
        elp_pert_s1_bin,
        elp_pert_s2_bin,
        elp_pert_s3_bin,
        Elpmpp02Correction::DE406,
    )
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
    Ok(YearDataResult {
        lunar_year: year_data.lunar_year,
        new_moon_jds: year_data.new_moon_jds.iter().map(|r| r.as_f64()).collect(),
        zhong_qi_jds: year_data.zhong_qi_jds.iter().map(|r| r.as_f64()).collect(),
    })
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

use lunar_core::astronomy::ephemeris::Elpmpp02Data;
use lunar_core::astronomy::phenomena::{
    eclipse_window, solar_eclipse_global_in, solar_eclipse_local_in, solar_eclipse_map as core_solar_eclipse_map, EclipseEphemeris,
    MapPoint, SolarEclipseGlobal, SolarEclipseKind, SolarEclipseMapOptions,
};
use lunar_core::quantity::duration::Duration as QDuration;

/// 日月历表常驻对象：构造时加载一次 VSOP87（地球，含已注入的 DE 改正）、ELPMPP02 与章动表，之后反复求日食。
#[wasm_bindgen]
pub struct EclipseEngine {
    vsop: Vsop87,
    elp: Elpmpp02Data,
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

/// ΔT 估计（Morrison–Stephenson 2021，按 ELPMPP02 − DE431 实测月亮黄经差配套修正，与 examples/historical_eclipses 同）。
fn ms2021_delta_t(jd_tt: f64) -> (f64, f64) {
    use lunar_core::astronomy::time::delta_t::{delta_t_for_lunar_offset, lunar_longitude_offset, MorrisonStephenson2021};
    let e = delta_t_for_lunar_offset(&MorrisonStephenson2021, real(jd_tt), &lunar_longitude_offset::ELPMPP02_DE406_MINUS_DE431);
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

/// ΔT 估计 [中值, 1σ]（秒）：Morrison–Stephenson 2021，已按本仓库月历表配套修正。
#[wasm_bindgen(js_name = deltaTEstimate)]
pub fn delta_t_estimate(jd_tt: f64) -> Vec<f64> {
    let (v, s) = ms2021_delta_t(jd_tt);
    vec![v, s]
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
        let elp = load_all_from_binary(
            elp_main_s1_bin,
            elp_main_s2_bin,
            elp_main_s3_bin,
            elp_pert_s1_bin,
            elp_pert_s2_bin,
            elp_pert_s3_bin,
            Elpmpp02Correction::DE406,
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
        if !nutation_tab53a_bin.is_empty() {
            nutation::try_init_full_nutation_from_binary(nutation_tab53a_bin);
        }
        Ok(EclipseEngine { vsop, elp })
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
        let elp = load_all_from_binary(
            elp_main_s1_bin,
            elp_main_s2_bin,
            elp_main_s3_bin,
            elp_pert_s1_bin,
            elp_pert_s2_bin,
            elp_pert_s3_bin,
            Elpmpp02Correction::DE406,
        )
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
    /// `delta_t_s` 只用于恒星时（GAST，地平星图用）。
    #[wasm_bindgen]
    pub fn snapshot(&self, jd_tt: f64, delta_t_s: f64) -> String {
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
        let bodies: Vec<String> = snap
            .bodies
            .iter()
            .map(|b| {
                format!(
                    "{{\"name\":\"{}\",\"ra\":{:.5},\"dec\":{:.5},\"mag\":{},\"phase\":{}}}",
                    body_name(b.body),
                    deg(b.ra).rem_euclid(360.0),
                    deg(b.dec),
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
        let jd_ut1 = jd_tt - delta_t_s / 86400.0;
        let gast = lunar_core::astronomy::time::earth_rotation::gast(real(jd_ut1), real(jd_tt));
        format!(
            "{{\"jdTt\":{},\"gastDeg\":{},\"eclipticPole\":[{},{}],\"stars\":[{}],\"bodies\":[{}],\"mansionWidths\":{{{}}}}}",
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
