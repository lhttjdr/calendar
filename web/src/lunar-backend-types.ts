/**
 * 农历/干支计算后端的统一接口。
 * Rust WASM (lunar-wasm) 实现此接口，供 Web 使用。
 */
export interface MonthLunarResult {
  readonly lunarYears: Int32Array | number[]
  readonly lunarMonths: Int32Array | number[]
  readonly dayOfMonths: Int32Array | number[]
  readonly isLeapMonths: Int32Array | number[]
  readonly daysInLunarMonths: Int32Array | number[]
}

export interface LunarBackend {
  gregorian_to_jd(year: number, month: number, day: number): number
  jd_to_gregorian(jd: number): { year: number; month: number; day: number }
  gregorian_to_chinese_lunar(
    year: number,
    month: number,
    day: number,
    lunar_year: number,
    new_moon_jds: Float64Array | number[],
    zhong_qi_jds: Float64Array | number[]
  ): { year: number; month: number; day: number; is_leap_month: boolean; days_in_month: number } | null
  gregorian_to_chinese_lunar_debug(
    year: number,
    month: number,
    day: number,
    lunar_year: number,
    new_moon_jds: Float64Array | number[],
    zhong_qi_jds: Float64Array | number[]
  ): string
  gregorian_month_to_lunar(
    year: number,
    month: number,
    lunar_year: number,
    new_moon_jds: Float64Array | number[],
    zhong_qi_jds: Float64Array | number[]
  ): MonthLunarResult
  gregorian_to_gan_zhi_day(year: number, month: number, day: number): string
  gregorian_to_gan_zhi_day_with_options(year: number, month: number, day: number, day_boundary: number): string
  ganzhi_preset_name(preset_index: number): string
  ganzhi_from_jd_solar_wasm(jd: number, vsop87_ear: string, day_boundary: number, solar_term_mode: number): { year_name: string; month_name: string; day_name: string }
  ganzhi_from_jd_lunar_wasm(
    jd: number,
    lunar_year: number,
    new_moon_jds: Float64Array | number[],
    zhong_qi_jds: Float64Array | number[],
    year_boundary: number,
    month_boundary: number,
    leap_month_handling: number,
    day_boundary: number
  ): { year_name: string; month_name: string; day_name: string } | null
  /** 整月干支批量接口（一次跨边界）。可选，由新 wasm-pack 构建提供。 */
  ganzhiForGregorianMonthWasm?(
    year: number,
    month: number,
    primary_lunar_year: number,
    data1_lunar_year: number,
    new_moon_jds_1: Float64Array | number[],
    zhong_qi_jds_1: Float64Array | number[],
    data2_lunar_year: number,
    new_moon_jds_2: Float64Array | number[],
    zhong_qi_jds_2: Float64Array | number[],
    year_boundary: number,
    month_boundary: number,
    leap_month_handling: number,
    day_boundary: number,
    vsop87_ear: string,
    solar_term_mode: number
  ): { yearNames: string[]; monthNames: string[]; dayNames: string[] }
  compute_year_data_wasm(
    lunar_year: number,
    vsop87_ear: string,
    elp_main_s1: string,
    elp_main_s2: string,
    elp_main_s3: string,
    elp_pert_s1: string,
    elp_pert_s2: string,
    elp_pert_s3: string,
    nutation_tab53a_bin: Uint8Array
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array; phase_jds?: Float64Array; phase_kinds?: Uint8Array; solar_term_years?: Int32Array; solar_term_jds?: Float64Array }
  /** 岁数据：VSOP87 用二进制（零解析），ELP 仍为文本。可选，优先使用以省带宽与 CPU。 */
  compute_year_data_from_binary?(
    lunar_year: number,
    vsop87_ear_bin: Uint8Array,
    elp_main_s1: string,
    elp_main_s2: string,
    elp_main_s3: string,
    elp_pert_s1: string,
    elp_pert_s2: string,
    elp_pert_s3: string,
    nutation_tab53a_bin: Uint8Array
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array; phase_jds?: Float64Array; phase_kinds?: Uint8Array; solar_term_years?: Int32Array; solar_term_jds?: Float64Array }
  /** 岁数据：VSOP87 + 6 个 ELP 全二进制，零解析。可选，7 个 .bin 均可用时调用。 */
  compute_year_data_full_binary?(
    lunar_year: number,
    vsop87_ear_bin: Uint8Array,
    elp_main_s1_bin: Uint8Array,
    elp_main_s2_bin: Uint8Array,
    elp_main_s3_bin: Uint8Array,
    elp_pert_s1_bin: Uint8Array,
    elp_pert_s2_bin: Uint8Array,
    elp_pert_s3_bin: Uint8Array,
    /** 章动完整表二进制（tab5.3a.bin），未取到时传空 Uint8Array。 */
    nutation_tab53a_bin: Uint8Array
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array; phase_jds?: Float64Array; phase_kinds?: Uint8Array; solar_term_years?: Int32Array; solar_term_jds?: Float64Array }
  /** 视位置变换图可视化数据（节点=参考架，边=6×6 状态转移，含步骤标签）。可选，由 wasm-pack 构建产物提供。 */
  transformGraphVisualizationData?(): {
    readonly nodeIds: string[]
    readonly edges: readonly { readonly from_id: string; readonly to_id: string; readonly cost: number; readonly label?: string | null }[]
  }
  /** 加载 VSOP87 地球改正（拟合到 DE406/DE441，.bin 或解压后的 .br）。可选，有则状态栏显示「地球改正: 已加载」。 */
  init_vsop87_earth_patch_from_binary?(bytes: Uint8Array): boolean
  /** 设置 ELPMPP02 常数集（"DE431" 缺省、"DE406"、"DE441"），时间尺度换算的 ΔT 配套随之换。可选。 */
  set_elpmpp02_correction?(name: string): boolean
  /** 当前 ELPMPP02 常数集名（"DE431"、"DE406" 或 "DE441"）。可选，供状态栏展示。 */
  get_elpmpp02_correction?(): string
  /** 上次岁数据计算时章动表是否已加载为完整 IAU2000A（否则为 77 项）。可选，供状态栏展示。 */
  get_repo_aux_nutation_full?(): boolean
  /** 上次岁数据计算时 VSOP87 地球改正是否已加载。可选，供状态栏展示。 */
  get_repo_aux_earth_patch?(): boolean
  /** ΔT 秒数（TT − UT），用于将岁数据中的 JD(TT) 转为 JD(UT) 以显示合朔/交节时刻。可选。 */
  delta_t_seconds?(jd_tt: number): number
  /** 干支名的纳音名目 */
  nayin_of_ganzhi_name?(name: string): string
  /** 月家日神煞 */
  /** 以二进制初始化本实例的地球 VSOP87 缓存 */
  init_vsop87_earth_from_binary?(bytes: Uint8Array): boolean
  /** WMM 磁偏角（度，东偏为正；有效期外 NaN） */
  magnetic_declination_wasm?(lat_deg: number, lon_deg: number, alt_km: number, decimal_year: number): number
  /** 择日页年神二十四山（JSON） */
  nian_shen_json?(jd: number, vsop87_ear: string): string
  /** 排盘页奇门（JSON） */
  qimen_json?(jd_ut: number, vsop87_ear: string, hour_gz: number, method: number, ji_gong: number, shen_ming: number, ding_ju: number, source: number, day_gz: number): string
  /** 排盘页紫微斗数（JSON） */
  ziwei_json?(year_gz: number, lunar_month: number, is_leap: boolean, lunar_day: number, hour_zhi: number, male: boolean, leap: number, xin: number, bing_ding: number, si_hua: number, chang_sheng: number): string
  /** 六爻纳甲与周易考变占（JSON） */
  liuyao_json?(values: Uint8Array, yue_zhi: number, day_gz: number, gua_shen: number, san_bian: number): string
  meihua_time_values?(year_zhi: number, lunar_month: number, lunar_day: number, hour_zhi: number): Uint8Array
  dayan_yao_wasm?(r1: number, r2: number, r3: number): number
  /** 玄空飞星（JSON） */
  xuankong_json?(year: number, zuo: number, ti: boolean, zuo_az: number): string
  /** 排盘页大六壬（JSON） */
  liuren_json?(jd_ut: number, vsop87_ear: string, day_gz: number, hour_zhi: number, rule: number, gui_rule: number, zhou: number): string
  /** 紫微十八飞星派（JSON） */
  ziwei18_json?(year_gz: number, lunar_month: number, is_leap: boolean, lunar_day: number, hour_zhi: number, male: boolean): string
  /** 婚嫁择吉逐日（JSON） */
  jiaqu_days_json?(jd0: number, vsop87_ear: string, months: Uint8Array, days: Uint8Array, leaps: Uint8Array, dims: Uint8Array, phases: Int8Array, nv_year_gz: number, has_weng_gu: boolean, has_nv_fu_mu: boolean, na_xu: boolean, opts: number): string
  /** 排盘页太乙（JSON） */
  taiyi_json?(jd_ut: number, vsop87_ear: string, day_gz: number, hour_zhi: number, kind: number, school: number, dun: number): string
  /** 排盘摘要：奇门时家局、六壬起课、太乙时计、三元九运 */
  paipan_wasm?(jd_ut: number, vsop87_ear: string, day_gz: number, hour_zhi: number): string
  /** 灌入 Worker 解出的节气（每年 24 项），返回灌入年数 */
  seed_solar_terms_wasm?(years: Int32Array, jds: Float64Array): number
  shensha_day_wasm?(jd: number, vsop87_ear: string, lunar_month: number, lunar_day: number, is_leap: boolean, days_in_month: number, phase: number): string
  /** 建除、日黄黑道、百忌 */
  jianchu_day_wasm?(jd: number, vsop87_ear: string): string
  /** 时黄黑道 */
  huanghei_hour_wasm?(day_name: string, hour_name: string): string
  /** 更点（method 0 清台官法 1 元授时法） */
  gengdian_wasm?(jd: number, latitude_deg: number, longitude_deg: number, method: number, vsop87_ear: string): string
  /** 杂节气标签（「；」分隔） */
  zajie_for_day_wasm?(jd: number, vsop87_ear: string): string
  /** 值日宿、七曜、年月九星 */
  folk_day_wasm?(jd: number, vsop87_ear: string): { xiu: string; qi_yao: string; year_star: string; month_star: string; nian_shen?: string; ri_shen?: string }
  /** 与定气同名的平气 JD(TT)，按行用历；表外 NaN */
  pingqi_tt_wasm?(dingqi_tt: number, qi_from_winter_solstice: number): number
  /** 公元某年行用的平气历名；表外空串 */
  pingqi_li_name_wasm?(year: number): string
  /** 当地太阳时（小时）；mode 1=平太阳时 2=真太阳时 */
  local_solar_hours_wasm?(jd_ut: number, longitude_deg: number, mode: number, vsop87_ear: string): number
  /** 主线该农历年行用之历的名称（麟德历、时宪历……）。 */
  get_calendar_name?(lunar_year: number): string
  /** 该农历年通行的计时写法；时制不明的年代返回空串。 */
  format_time_for_lunar_year?(lunar_year: number, hour: number, minute: number, second: number): string
  /** 按农历年查主表年号（中原正朔），simp=true 简体。 */
  get_era_name?(lunar_year: number, simp: boolean): string
  /** 按农历年查主表年号拆分：JSON { regime, emperor, era_name, era_year, era }（汉/武帝/元封/五年），用于统一格式展示。 */
  get_era_name_parts?(lunar_year: number, simp: boolean): string
  /** 按日解析年号：JSON 另含 has_gaiyuan, gaiyuan_resolved, gaiyuan_on_this_day, gaiyuan_precision。改元日未补时 resolved=false，前年号/新年号并列。 */
  get_era_name_parts_for_day?(
    lunar_year: number,
    g_year: number,
    g_month: number,
    g_day: number,
    lunar_month: number,
    lunar_is_leap: boolean,
    lunar_day: number,
  ): string
  /** 按农历年查**全部**并存政权（如三国魏蜀吴、五代十国同年多国并存），JSON 数组，元素形状同 get_era_name_parts，
   *  含 is_primary 标出哪条是"正统主线"（对应 get_era_name_parts 的返回值）。 */
  get_era_name_parts_all?(lunar_year: number): string
  /** 公历日下各推步政权（蜀、吴等）的农历月日，JSON 数组 [{ region, calendar_name, lunar_year, month, is_leap, day }]。 */
  get_regimes_lunar_for_day?(g_year: number, g_month: number, g_day: number): string
}
