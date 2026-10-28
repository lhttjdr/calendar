/**
 * Web Worker：在 Worker 内 fetch 历表二进制并调用 WASM 计算岁数据，结果用 Transferable 回传主线程，避免阻塞 UI。
 * 仅处理一级「全二进制」路径（fetchFullBinaryEphemerisSet 不可用时直接回退主线程，不在 Worker 内实现二/三级）；
 * 数据源仲裁逻辑（哪一级、旁路数据取舍）统一在 ephemerisSourceRegistry.ts，不在这里重复。
 */

import { fetchFullBinaryEphemerisSet, fetchNutationTab53aBin, applyEarthPatchIfAvailable } from './ephemerisSourceRegistry'

export type YearDataMessage = {
  lunarYear: number
  newMoonJds: Float64Array
  zhongQiJds: Float64Array
}

let wasm: {
  init_vsop87_earth_patch_from_binary?: (bytes: Uint8Array) => boolean
  compute_year_data_full_binary: (
    lunarYear: number,
    vsop: Uint8Array,
    e1: Uint8Array,
    e2: Uint8Array,
    e3: Uint8Array,
    p1: Uint8Array,
    p2: Uint8Array,
    p3: Uint8Array,
    nutationTab53aBin: Uint8Array
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
  const fullSet = await fetchFullBinaryEphemerisSet()
  if (!fullSet) return null
  await applyEarthPatchIfAvailable(w)
  const nutationBin = await fetchNutationTab53aBin()
  const result = w.compute_year_data_full_binary(
    lunarYear,
    fullSet.vsop87Bin,
    fullSet.elpBins[0],
    fullSet.elpBins[1],
    fullSet.elpBins[2],
    fullSet.elpBins[3],
    fullSet.elpBins[4],
    fullSet.elpBins[5],
    nutationBin
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
