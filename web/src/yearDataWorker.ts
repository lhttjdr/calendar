/**
 * Web Worker：在 Worker 内 fetch 历表二进制并调用 WASM 计算岁数据，结果用 Transferable 回传主线程，避免阻塞 UI。
 * 仅处理「全二进制」路径；失败时通知主线程回退到主线程逻辑。
 */

import { fetchBinaryMaybeBrotli, isVsop87Binary, isElpBinary } from './fetchEphemerisBinary'
import { EPHEMERIS_BINARY_URLS, EPHEMERIS_BINARY_URL_LIST } from './ephemerisUrls'

export type YearDataMessage = {
  lunarYear: number
  newMoonJds: Float64Array
  zhongQiJds: Float64Array
}

let wasm: {
  compute_year_data_full_binary: (
    lunarYear: number,
    vsop: Uint8Array,
    e1: Uint8Array,
    e2: Uint8Array,
    e3: Uint8Array,
    p1: Uint8Array,
    p2: Uint8Array,
    p3: Uint8Array
  ) => { lunar_year: number; new_moon_jds: number[]; zhong_qi_jds: number[] }
} | null = null

async function loadWasm() {
  if (wasm) return wasm
  const mod = await import('lunar-wasm-f64')
  if (typeof mod.default === 'function') await mod.default()
  wasm = mod as unknown as typeof wasm
  return wasm!
}

async function computeInWorker(lunarYear: number): Promise<YearDataMessage | null> {
  const w = await loadWasm()
  if (typeof w.compute_year_data_full_binary !== 'function') return null
  const settled = await Promise.allSettled(
    EPHEMERIS_BINARY_URL_LIST.map((url) => fetchBinaryMaybeBrotli(url))
  )
  const vsop =
    settled[0].status === 'fulfilled' ? settled[0].value : null
  const elpBins = settled
    .slice(1, 7)
    .map((r) => (r.status === 'fulfilled' ? r.value : null))
    .filter((u): u is Uint8Array => u != null && u.length >= 4)
  if (!vsop || elpBins.length !== 6 || !isVsop87Binary(vsop) || elpBins.some((b) => !isElpBinary(b)))
    return null
  const result = w.compute_year_data_full_binary(
    lunarYear,
    vsop,
    elpBins[0]!,
    elpBins[1]!,
    elpBins[2]!,
    elpBins[3]!,
    elpBins[4]!,
    elpBins[5]!
  )
  const newMoonJds = new Float64Array(result.new_moon_jds)
  const zhongQiJds = new Float64Array(result.zhong_qi_jds)
  return { lunarYear: result.lunar_year, newMoonJds, zhongQiJds }
}

self.onmessage = async (ev: MessageEvent<{ type: string; id: number; lunarYear?: number }>) => {
  const { type, id, lunarYear } = ev.data
  if (type !== 'getYearData' || typeof lunarYear !== 'number' || typeof id !== 'number') return
  try {
    const data = await computeInWorker(lunarYear)
    if (data) {
      self.postMessage(
        { id, yearData: data },
        [data.newMoonJds.buffer, data.zhongQiJds.buffer]
      )
    } else {
      self.postMessage({ id, fallback: true, lunarYear })
    }
  } catch {
    self.postMessage({ id, fallback: true, lunarYear })
  }
}

