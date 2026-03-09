/** 历表数据 URL，供主线程与 Worker 共用。使用 Vite BASE_URL，部署到子路径时 /data 仍正确。 */
const BASE = (typeof import.meta !== 'undefined' && (import.meta as { env?: { BASE_URL?: string } }).env?.BASE_URL)
  ? (import.meta as { env: { BASE_URL: string } }).env.BASE_URL.replace(/\/$/, '')
  : ''
export const DATA_BASE = `${BASE}/data`
export const ELP_BASE = `${DATA_BASE}/elpmpp02`

/** 二进制 URL 带缓存破坏，避免浏览器沿用旧 404 缓存导致一直走全文本 */
const BIN_V = '?v=2'
export const EPHEMERIS_BINARY_URLS = {
  vsop87_ear_bin: `${DATA_BASE}/vsop87/VSOP87B.ear.bin${BIN_V}`,
  elp_main_s1_bin: `${ELP_BASE}/ELP_MAIN.S1.bin${BIN_V}`,
  elp_main_s2_bin: `${ELP_BASE}/ELP_MAIN.S2.bin${BIN_V}`,
  elp_main_s3_bin: `${ELP_BASE}/ELP_MAIN.S3.bin${BIN_V}`,
  elp_pert_s1_bin: `${ELP_BASE}/ELP_PERT.S1.bin${BIN_V}`,
  elp_pert_s2_bin: `${ELP_BASE}/ELP_PERT.S2.bin${BIN_V}`,
  elp_pert_s3_bin: `${ELP_BASE}/ELP_PERT.S3.bin${BIN_V}`,
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
