use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use lunar_core::math::real::{real, RealOps};
use lunar_core::platform::{DataLoader, LoadError};
use lunar_core::astronomy::ephemeris::{load_all, load_all_from_binary, load_earth_vsop87_from_repo, Elpmpp02Correction, Vsop87};
use lunar_core::astronomy::frame::nutation;
use lunar_core::astronomy::frame::vsop87_de406_icrs_patch;

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

const VSOP87_EAR_PATH: &str = "data/vsop87/VSOP87B.ear";
const ELP_BASE: &str = "data/elpmpp02";

/// 缓存已解析的 VSOP87，避免日历每格调用时重复解析 300KB+ 文本（节气派干支历）。
static VSOP87_CACHE: Lazy<Mutex<Option<Arc<Vsop87>>>> = Lazy::new(|| Mutex::new(None));

/// 上次岁数据计算时的 repo 辅助数据加载状态（章动表、拟合表），供状态栏展示。
static REPO_AUX_STATUS: Lazy<Mutex<(bool, bool)>> = Lazy::new(|| Mutex::new((false, false)));

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
            let parsed = load_earth_vsop87_from_repo().map_err(|e| JsValue::from_str(&e.to_string()))?;
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
            match load_earth_vsop87_from_repo() {
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
    let nutation_ok = nutation::try_init_full_nutation_from_repo();
    let patch_ok = vsop87_de406_icrs_patch::try_init_de406_patch_from_repo();
    *REPO_AUX_STATUS.lock().unwrap() = (nutation_ok, patch_ok);
    let vsop = load_earth_vsop87_from_repo().map_err(|e| JsValue::from_str(&e.to_string()))?;
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
) -> Result<YearDataResult, JsValue> {
    let vsop = Vsop87::from_binary(vsop87_ear_bin).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut files = HashMap::new();
    files.insert(format!("{}/ELP_MAIN.S1", ELP_BASE), elp_main_s1.to_string());
    files.insert(format!("{}/ELP_MAIN.S2", ELP_BASE), elp_main_s2.to_string());
    files.insert(format!("{}/ELP_MAIN.S3", ELP_BASE), elp_main_s3.to_string());
    files.insert(format!("{}/ELP_PERT.S1", ELP_BASE), elp_pert_s1.to_string());
    files.insert(format!("{}/ELP_PERT.S2", ELP_BASE), elp_pert_s2.to_string());
    files.insert(format!("{}/ELP_PERT.S3", ELP_BASE), elp_pert_s3.to_string());
    let loader = DataLoaderWasm::new(files);
    lunar_core::repo::set_loader(Box::new(loader));
    let nutation_ok = nutation::try_init_full_nutation_from_repo();
    let patch_ok = vsop87_de406_icrs_patch::try_init_de406_patch_from_repo();
    *REPO_AUX_STATUS.lock().unwrap() = (nutation_ok, patch_ok);
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
) -> Result<YearDataResult, JsValue> {
    let vsop = Vsop87::from_binary(vsop87_ear_bin).map_err(|e| JsValue::from_str(&e.to_string()))?;
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
    *REPO_AUX_STATUS.lock().unwrap() = (false, false);
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

/// 上次岁数据计算时的 repo 辅助数据加载状态，供状态栏展示。全二进制路径下章动/拟合未从 repo 加载，为 false。
#[wasm_bindgen]
pub fn get_repo_aux_nutation_full() -> bool {
    REPO_AUX_STATUS.lock().unwrap().0
}

#[wasm_bindgen]
pub fn get_repo_aux_patch_icrs() -> bool {
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
