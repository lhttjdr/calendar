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
  phaseJds?: Float64Array
  phaseKinds?: Uint8Array
  /** 前後各一公历年的二十四节气（每年 24 项），主线程据以灌节气缓存 */
  termYears?: Int32Array
  termJds?: Float64Array
}

type WorkerWasm = {
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
  ) => { lunar_year: number; new_moon_jds: number[]; zhong_qi_jds: number[]; phase_jds?: number[]; phase_kinds?: number[]; solar_term_years?: number[]; solar_term_jds?: number[] }
}

type Variant = 'f64' | 'twofloat'
/** 两种 Real 变体各自加载、各自缓存：与主线程所选后端一致 */
const wasmByVariant: Partial<Record<Variant, WorkerWasm>> = {}

async function loadWasm(variant: Variant): Promise<WorkerWasm> {
  const cached = wasmByVariant[variant]
  if (cached) return cached
  const mod = variant === 'twofloat' ? await import('lunar-wasm') : await import('lunar-wasm-f64')
  if (typeof mod.default === 'function') await mod.default()
  const w = mod as unknown as WorkerWasm
  wasmByVariant[variant] = w
  return w
}

async function computeInWorker(lunarYear: number, variant: Variant): Promise<YearDataMessage | null> {
  const w = await loadWasm(variant)
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
  const phaseJds = result.phase_jds ? new Float64Array(result.phase_jds) : undefined
  const phaseKinds = result.phase_kinds ? new Uint8Array(result.phase_kinds) : undefined
  const termYears = result.solar_term_years ? new Int32Array(result.solar_term_years) : undefined
  const termJds = result.solar_term_jds ? new Float64Array(result.solar_term_jds) : undefined
  return { lunarYear: result.lunar_year, newMoonJds, zhongQiJds, phaseJds, phaseKinds, termYears, termJds }
}

self.onmessage = async (ev: MessageEvent<{ type: string; id: number; lunarYear?: number; variant?: Variant }>) => {
  const { type, id, lunarYear, variant = 'f64' } = ev.data
  if (type !== 'getYearData' || typeof lunarYear !== 'number' || typeof id !== 'number') return
  try {
    const data = await computeInWorker(lunarYear, variant)
    if (data) {
      // 用 options 形式传 transfer：tsconfig 只含 DOM 类型（self 按 Window 检查），此形式在 Window 与 Worker 下签名一致
      const transfer: ArrayBuffer[] = [data.newMoonJds.buffer as ArrayBuffer, data.zhongQiJds.buffer as ArrayBuffer]
      if (data.phaseJds) transfer.push(data.phaseJds.buffer as ArrayBuffer)
      if (data.phaseKinds) transfer.push(data.phaseKinds.buffer as ArrayBuffer)
      if (data.termYears) transfer.push(data.termYears.buffer as ArrayBuffer)
      if (data.termJds) transfer.push(data.termJds.buffer as ArrayBuffer)
      self.postMessage({ id, yearData: data }, { transfer })
    } else {
      self.postMessage({ id, fallback: true, lunarYear })
    }
  } catch {
    self.postMessage({ id, fallback: true, lunarYear })
  }
}
