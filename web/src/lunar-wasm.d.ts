declare module 'lunar-wasm' {
  export function gregorian_to_jd(year: number, month: number, day: number): number
  export function jd_to_gregorian(jd: number): { year: number; month: number; day: number }
  export function gregorian_to_chinese_lunar(
    year: number,
    month: number,
    day: number,
    lunar_year: number,
    new_moon_jds: Float64Array | number[],
    zhong_qi_jds: Float64Array | number[]
  ): { year: number; month: number; day: number; is_leap_month: boolean; days_in_month: number } | null
  export function gregorian_to_chinese_lunar_debug(
    year: number,
    month: number,
    day: number,
    lunar_year: number,
    new_moon_jds: Float64Array | number[],
    zhong_qi_jds: Float64Array | number[]
  ): string
  export function gregorian_month_to_lunar(
    year: number,
    month: number,
    lunar_year: number,
    new_moon_jds: Float64Array | number[],
    zhong_qi_jds: Float64Array | number[]
  ): MonthLunarResult
  export interface MonthLunarResult {
    readonly lunarYears: Int32Array | number[]
    readonly lunarMonths: Int32Array | number[]
    readonly dayOfMonths: Int32Array | number[]
    readonly isLeapMonths: Int32Array | number[]
    readonly daysInLunarMonths: Int32Array | number[]
  }
  export function gregorian_to_gan_zhi_day(year: number, month: number, day: number): string
  export function gregorian_to_gan_zhi_day_with_options(
    year: number,
    month: number,
    day: number,
    day_boundary: number
  ): string
  export function ganzhi_preset_name(preset_index: number): string
  export function ganzhi_from_jd_solar_wasm(
    jd: number,
    vsop87_ear: string,
    day_boundary: number
  ): { year_name: string; month_name: string; day_name: string }
  export function ganzhi_from_jd_lunar_wasm(
    jd: number,
    lunar_year: number,
    new_moon_jds: Float64Array | number[],
    zhong_qi_jds: Float64Array | number[],
    year_boundary: number,
    month_boundary: number,
    leap_month_handling: number,
    day_boundary: number
  ): { year_name: string; month_name: string; day_name: string } | null
  export function compute_year_data_wasm(
    lunar_year: number,
    vsop87_ear: string,
    elp_main_s1: string,
    elp_main_s2: string,
    elp_main_s3: string,
    elp_pert_s1: string,
    elp_pert_s2: string,
    elp_pert_s3: string
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array }

  /** 视位置变换图可视化：节点=参考架，边=6×6 状态转移。供前端画架变换图（如 D3 / Mermaid）。 */
  export function transformGraphVisualizationData(): TransformGraphViz

  export interface TransformGraphViz {
    readonly nodeIds: string[]
    readonly edges: GraphEdgeViz[]
  }

  export interface GraphEdgeViz {
    readonly from_id: string
    readonly to_id: string
    readonly cost: number
    readonly label: string | null
  }

  export default function init?(): Promise<void>
}
