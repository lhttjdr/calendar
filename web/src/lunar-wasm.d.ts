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
  export function nayin_of_ganzhi_name(name: string): string
  export function nian_shen_json(jd: number, vsop87_ear: string): string
  export function qimen_json(jd_ut: number, vsop87_ear: string, hour_gz: number, method: number, ji_gong: number, shen_ming: number, ding_ju: number, source: number, day_gz: number, an_gan: number, gan_mu: number, ji_xing: number, kong: number): string
  export function ziwei_json(year_gz: number, lunar_month: number, is_leap: boolean, lunar_day: number, hour_zhi: number, male: boolean, leap: number, xin: number, bing_ding: number, si_hua: number, chang_sheng: number): string
  export function liuyao_json(values: Uint8Array, yue_zhi: number, day_gz: number, gua_shen: number, san_bian: number): string
  export function meihua_time_values(year_zhi: number, lunar_month: number, lunar_day: number, hour_zhi: number): Uint8Array
  export function dayan_yao_wasm(r1: number, r2: number, r3: number): number
  export function xuankong_json(year: number, zuo: number, ti: boolean, zuo_az: number): string
  export function liuren_json(jd_ut: number, vsop87_ear: string, day_gz: number, hour_zhi: number, rule: number, gui_rule: number, zhou: number, dun_rule: number, ben_ming_gz: number, male: number): string
  export function ziwei18_json(year_gz: number, lunar_month: number, is_leap: boolean, lunar_day: number, hour_zhi: number, male: boolean): string
  export function jiaqu_days_json(jd0: number, vsop87_ear: string, months: Uint8Array, days: Uint8Array, leaps: Uint8Array, dims: Uint8Array, phases: Int8Array, nv_year_gz: number, has_weng_gu: boolean, has_nv_fu_mu: boolean, na_xu: boolean, opts: number): string
  export function taiyi_json(jd_ut: number, vsop87_ear: string, day_gz: number, hour_zhi: number, kind: number, school: number, dun: number): string
  export function paipan_wasm(jd_ut: number, vsop87_ear: string, day_gz: number, hour_zhi: number): string
  export function seed_solar_terms_wasm(years: Int32Array, jds: Float64Array): number
  export function shensha_day_wasm(jd: number, vsop87_ear: string, lunar_month: number, lunar_day: number, is_leap: boolean, days_in_month: number, phase: number): string
  export function dong_gong_day_json(jd: number, vsop87_ear: string): string
  export function qili_shen_day_json(jd: number, vsop87_ear: string, tian_de: number, tian_huo: number, ming_fei: number, ming_fei_dui: number, fu_ri: number, huang_wu: number, can_ming: number): string
  export function jianchu_day_wasm(jd: number, vsop87_ear: string): string
  export function huanghei_hour_wasm(day_name: string, hour_name: string): string
  export function gengdian_wasm(jd: number, latitude_deg: number, longitude_deg: number, method: number, vsop87_ear: string): string
  export function zajie_for_day_wasm(jd: number, vsop87_ear: string): string
  export function folk_day_wasm(jd: number, vsop87_ear: string): { xiu: string; qi_yao: string; year_star: string; month_star: string; nian_shen?: string; ri_shen?: string }
  export function pingqi_tt_wasm(dingqi_tt: number, qi_from_winter_solstice: number): number
  export function pingqi_li_name_wasm(year: number): string
  export function local_solar_hours_wasm(jd_ut: number, longitude_deg: number, mode: number, vsop87_ear: string): number
  export function ganzhi_from_jd_solar_wasm(
    jd: number,
    vsop87_ear: string,
    day_boundary: number,
    solar_term_mode: number
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
    elp_pert_s3: string,
    nutation_tab53a_bin: Uint8Array
  ): { lunar_year: number; new_moon_jds: Float64Array; zhong_qi_jds: Float64Array; phase_jds?: Float64Array; phase_kinds?: Uint8Array; solar_term_years?: Int32Array; solar_term_jds?: Float64Array }

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

  export default function init(): Promise<void>
}
