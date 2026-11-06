/** 历表数据 URL，供主线程与 Worker 共用。使用 Vite BASE_URL，部署到子路径时 /data 仍正确。 */
const BASE = (typeof import.meta !== 'undefined' && (import.meta as { env?: { BASE_URL?: string } }).env?.BASE_URL)
  ? (import.meta as { env: { BASE_URL: string } }).env.BASE_URL.replace(/\/$/, '')
  : ''
export const DATA_BASE = `${BASE}/data`
export const ELP_BASE = `${DATA_BASE}/elpmpp02`

/** 二进制 URL 带缓存破坏，避免浏览器沿用旧 404 缓存导致一直走全文本 */
const BIN_V = '?v=3'
const FIT_BASE = `${DATA_BASE}/fit`

export const EPHEMERIS_BINARY_URLS = {
  vsop87_ear_bin: `${DATA_BASE}/vsop87/VSOP87B.ear.bin${BIN_V}`,
  elp_main_s1_bin: `${ELP_BASE}/ELP_MAIN.S1.bin${BIN_V}`,
  elp_main_s2_bin: `${ELP_BASE}/ELP_MAIN.S2.bin${BIN_V}`,
  elp_main_s3_bin: `${ELP_BASE}/ELP_MAIN.S3.bin${BIN_V}`,
  elp_pert_s1_bin: `${ELP_BASE}/ELP_PERT.S1.bin${BIN_V}`,
  elp_pert_s2_bin: `${ELP_BASE}/ELP_PERT.S2.bin${BIN_V}`,
  elp_pert_s3_bin: `${ELP_BASE}/ELP_PERT.S3.bin${BIN_V}`,
  /** VSOP87 地球改正（拟合到 DE406，Chapront 2000 方法），可选；有则状态栏显示「地球改正: 已加载」 */
  earth_patch_de406_bin: `${FIT_BASE}/vsop87-de406-earth.bin${BIN_V}`,
  /** 同上，拟合到 DE431（缺省，与 MS2021 ΔT 所配历表一致） */
  earth_patch_de431_bin: `${FIT_BASE}/vsop87-de431-earth.bin${BIN_V}`,
  /** 同上，拟合到 DE441 */
  earth_patch_de441_bin: `${FIT_BASE}/vsop87-de441-earth.bin${BIN_V}`,
  /** 章动完整表 IAU2000A（tab5.3a.bin），与历表二进制同一链路；有则状态栏显示「完整 IAU2000A」，否则回落 77 项 */
  nutation_tab53a_bin: `${DATA_BASE}/IAU2000/tab5.3a.bin${BIN_V}`,
} as const

export const EPHEMERIS_BINARY_URL_LIST = [
  EPHEMERIS_BINARY_URLS.vsop87_ear_bin,
  EPHEMERIS_BINARY_URLS.elp_main_s1_bin,
  EPHEMERIS_BINARY_URLS.elp_main_s2_bin,
  EPHEMERIS_BINARY_URLS.elp_main_s3_bin,
  EPHEMERIS_BINARY_URLS.elp_pert_s1_bin,
  EPHEMERIS_BINARY_URLS.elp_pert_s2_bin,
  EPHEMERIS_BINARY_URLS.elp_pert_s3_bin,
] as const

/** 日月历表对齐的目标历表（与 Rust 侧缺省 `PatchTarget::DEFAULT`、`Elpmpp02Correction::DEFAULT` 即 DE431 一致）；改为 'de441' 即
 * VSOP87 地球用 DE441 版改正、ELPMPP02 用拟合 DE441 的常数集（'de406' 则为论文常数集），ΔT 配套随之换。 */
export const EARTH_PATCH_TARGET: 'de406' | 'de431' | 'de441' = 'de431'
