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
    elp_pert_s3: string
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
    elp_pert_s3: string
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
    elp_pert_s3_bin: Uint8Array
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array }
  /** 视位置变换图可视化数据（节点=参考架，边=6×6 状态转移，含步骤标签）。可选，由 wasm-pack 构建产物提供。 */
  transformGraphVisualizationData?(): {
    readonly nodeIds: string[]
    readonly edges: readonly { readonly from_id: string; readonly to_id: string; readonly cost: number; readonly label?: string | null }[]
  }
}
