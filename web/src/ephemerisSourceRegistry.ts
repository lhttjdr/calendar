/**
 * 历表/章动/拟合表数据源仲裁层（显式建模）。
 *
 * 天文域的「选哪个数据源」决策目前留在前端（浏览器环境本就是决定权所在——能不能用二进制、
 * 网络能不能拿到某个文件，都是运行环境的事，不是天文计算本身的事），但决策逻辑必须集中在
 * 这一个模块里显式命名，不能散落进各调用方（主线程 useLunarWasm.ts、Worker yearDataWorker.ts）
 * 各写一份、逐渐漂移。
 *
 * 两类数据源，互相独立：
 *
 * 1. 岁数据历表（VSOP87 + ELPMPP02）：三级优先级，从高到低——
 *    full_binary（全二进制）> vsop_binary_elp_text（VSOP87 二进制 + ELP 文本）> full_text（全文本）
 *    上一级不可用（fetch 失败 / 魔数校验不过 / wasm 未导出对应函数）才降级到下一级，不做部分降级；
 *    调用哪一级对应哪个 wasm 导出函数（compute_year_data_full_binary / _from_binary / _wasm）由
 *    调用方决定，本模块只负责「某一级的数据能不能凑齐」。
 *
 * 2. 旁路可选数据（章动完整表 tab5.3a.bin、VSOP87 地球改正）：与上面三级历法无关，尽力 fetch，
 *    取不到就传空 / 不传，由 Rust 侧内部决定回落（章动回落 77 项、地球改正不启用），不算错误。
 */

import { EARTH_PATCH_TARGET, EPHEMERIS_BINARY_URL_LIST, EPHEMERIS_BINARY_URLS } from './ephemerisUrls'
import { fetchBinaryMaybeBrotli, isVsop87Binary, isElpBinary, isNutationTab53aBinary } from './fetchEphemerisBinary'

/** 岁数据实际用上的历表数据源，用于状态栏展示；三个取值即三级优先级，从高到低。 */
export type EphemerisDataSource = 'full_binary' | 'vsop_binary_elp_text' | 'full_text'

/** 数据源对应的状态栏文案 */
export const EPHEMERIS_SOURCE_LABELS: Record<EphemerisDataSource, string> = {
  full_binary: '历表：全二进制 (.bin/.br)',
  vsop_binary_elp_text: '历表：VSOP87 二进制 + ELP 文本',
  full_text: '历表：全文本 (.ear / ELP)',
}

/** 数据源对应的格式简述（是否二进制），用于状态栏 */
export const EPHEMERIS_FORMAT_LABELS: Record<EphemerisDataSource, string> = {
  full_binary: '二进制',
  vsop_binary_elp_text: '混合',
  full_text: '文本',
}

const EMPTY_BIN = new Uint8Array(0)

/** 6 个 ELP 二进制，顺序固定为 MAIN.S1/S2/S3、PERT.S1/S2/S3（与 EPHEMERIS_BINARY_URL_LIST[1..6] 一致）。 */
export type ElpBinSet = [Uint8Array, Uint8Array, Uint8Array, Uint8Array, Uint8Array, Uint8Array]

/**
 * 一级：尝试取全二进制历表集（VSOP87 + 6×ELP），逐个校验 magic。
 * 任一缺失或校验不过，视为「一级不可用」，返回 null——调用方应降级到二级（vsop_binary_elp_text）。
 */
export async function fetchFullBinaryEphemerisSet(): Promise<{ vsop87Bin: Uint8Array; elpBins: ElpBinSet } | null> {
  const settled = await Promise.allSettled(EPHEMERIS_BINARY_URL_LIST.map((url) => fetchBinaryMaybeBrotli(url)))
  const vsop87Bin = settled[0].status === 'fulfilled' ? settled[0].value : null
  const elpBins = settled
    .slice(1, 7)
    .map((r) => (r.status === 'fulfilled' ? r.value : null))
    .filter((u): u is Uint8Array => u != null && u.length >= 4)
  const ok = Boolean(vsop87Bin && elpBins.length === 6 && isVsop87Binary(vsop87Bin) && elpBins.every(isElpBinary))
  if (import.meta.env.DEV && !ok) {
    const vsopOk = vsop87Bin != null && isVsop87Binary(vsop87Bin)
    const elpOk = elpBins.length === 6 && elpBins.every(isElpBinary)
    console.log('[历表仲裁] full_binary 不可用，降级:', {
      vsop: vsop87Bin ? `${vsop87Bin.length}B, magic=${vsopOk}` : 'rejected',
      elpCount: elpBins.length,
      elpMagicOk: elpOk,
      settled: settled.map((s) => (s.status === 'fulfilled' ? 'ok' : 'rejected')),
    })
  }
  if (!ok || !vsop87Bin) return null
  return { vsop87Bin, elpBins: elpBins as ElpBinSet }
}

/**
 * 旁路可选数据：章动完整表二进制（tab5.3a.bin）。与上面三级历法无关，任何一级都可能用得上。
 * 取不到 / magic 校验不过时返回空数组——传给 wasm 后 Rust 侧据此回落 77 项，不算错误，调用方不必特殊处理。
 */
export async function fetchNutationTab53aBin(): Promise<Uint8Array> {
  try {
    const buf = await fetchBinaryMaybeBrotli(EPHEMERIS_BINARY_URLS.nutation_tab53a_bin)
    return isNutationTab53aBinary(buf) ? buf : EMPTY_BIN
  } catch {
    return EMPTY_BIN
  }
}

/**
 * 旁路可选数据：VSOP87 地球改正二进制（拟合到 DE406 或 DE441，见 EARTH_PATCH_TARGET），并在 wasm 后端上应用
 * （若后端支持该导出函数）。取不到时静默跳过——Rust 侧对应函数未被调用，地球改正就是「未加载」状态，不算错误。
 */
export async function applyEarthPatchIfAvailable(w: { init_vsop87_earth_patch_from_binary?: (bytes: Uint8Array) => boolean }): Promise<void> {
  if (typeof w.init_vsop87_earth_patch_from_binary !== 'function') return
  try {
    const url = EARTH_PATCH_TARGET === 'de441' ? EPHEMERIS_BINARY_URLS.earth_patch_de441_bin : EPHEMERIS_BINARY_URLS.earth_patch_de406_bin
    const buf = await fetchBinaryMaybeBrotli(url)
    w.init_vsop87_earth_patch_from_binary(buf)
  } catch {
    // 地球改正可选，404/网络失败则跳过
  }
}
