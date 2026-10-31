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
  ganzhi_from_jd_solar_wasm(jd: number, vsop87_ear: string, day_boundary: number): { year_name: string; month_name: string; day_name: string }
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
    vsop87_ear: string
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
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array }
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
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array }
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
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array }
  /** 视位置变换图可视化数据（节点=参考架，边=6×6 状态转移，含步骤标签）。可选，由 wasm-pack 构建产物提供。 */
  transformGraphVisualizationData?(): {
    readonly nodeIds: string[]
    readonly edges: readonly { readonly from_id: string; readonly to_id: string; readonly cost: number; readonly label?: string | null }[]
  }
  /** 加载 VSOP87 地球改正（拟合到 DE406/DE441，.bin 或解压后的 .br）。可选，有则状态栏显示「地球改正: 已加载」。 */
  init_vsop87_earth_patch_from_binary?(bytes: Uint8Array): boolean
  /** 上次岁数据计算时章动表是否已加载为完整 IAU2000A（否则为 77 项）。可选，供状态栏展示。 */
  get_repo_aux_nutation_full?(): boolean
  /** 上次岁数据计算时 VSOP87 地球改正是否已加载。可选，供状态栏展示。 */
  get_repo_aux_earth_patch?(): boolean
  /** ΔT 秒数（TT − UT），用于将岁数据中的 JD(TT) 转为 JD(UT) 以显示合朔/交节时刻。可选。 */
  delta_t_seconds?(jd_tt: number): number
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
