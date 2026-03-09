/**
 * 加载农历后端（lunar-wasm）并封装岁数据现算与公历→农历。
 * 数据从 /data/ 下 fetch 7 个文件后调用 compute_year_data_wasm 现算。
 *
 * 历法依赖关系（简要）：
 * - 格里高利历：不依赖其它历，只依赖儒略历（JD）；与儒略历可互相换算。
 * - 农历：依赖 气（中气）、朔、儒略历；通过儒略历与格里高利历换算。
 * - 干支历：年/月柱的换界有两种约定——
 *   · 节气派（年界=立春、月界=节气）：只依赖 气（节气）、儒略历；不依赖农历，不依赖朔。
 *   · 农历派（年界=春节、月界=朔日）：换年=正月初一、换月=初一，即依赖 朔（及气，用于定正月等）；
 *     也可说依赖农历，因春节、初一即农历的岁、月边界，而农历本身依赖气、朔。
 *
 * 重复计算与可统一性：
 * - 当前实现：农历岁数据只算 14 朔 + 12 中气；节气派干支历单独用 VSOP87 算 24 节气（取十二节）。
 *   故「定气」（太阳视黄经→节气时刻）会算两套：农历用中气，节气派用节，底层都是同一套太阳历算。
 * - 若统一为「一次算出 朔 + 24 节气」的岁/年数据，农历用 朔+中气，节气派用 十二节，则可复用，
 *   避免重复计算；需要扩展岁数据或增加「公历年的 24 节气」缓存并在两处共用。
 */

import { DATA_BASE, ELP_BASE, EPHEMERIS_BINARY_URL_LIST, EPHEMERIS_BINARY_URLS } from './ephemerisUrls'

const FILES = {
  vsop87_ear: `${DATA_BASE}/vsop87/VSOP87B.ear`,
  vsop87_ear_bin: EPHEMERIS_BINARY_URLS.vsop87_ear_bin,
  elp_main_s1: `${ELP_BASE}/ELP_MAIN.S1`,
  elp_main_s2: `${ELP_BASE}/ELP_MAIN.S2`,
  elp_main_s3: `${ELP_BASE}/ELP_MAIN.S3`,
  elp_pert_s1: `${ELP_BASE}/ELP_PERT.S1`,
  elp_pert_s2: `${ELP_BASE}/ELP_PERT.S2`,
  elp_pert_s3: `${ELP_BASE}/ELP_PERT.S3`,
  elp_main_s1_bin: EPHEMERIS_BINARY_URLS.elp_main_s1_bin,
  elp_main_s2_bin: EPHEMERIS_BINARY_URLS.elp_main_s2_bin,
  elp_main_s3_bin: EPHEMERIS_BINARY_URLS.elp_main_s3_bin,
  elp_pert_s1_bin: EPHEMERIS_BINARY_URLS.elp_pert_s1_bin,
  elp_pert_s2_bin: EPHEMERIS_BINARY_URLS.elp_pert_s2_bin,
  elp_pert_s3_bin: EPHEMERIS_BINARY_URLS.elp_pert_s3_bin,
} as const

/** 设为 true 可打印干支历计算与缓存相关日志，排查问题时使用 */
const GANZHI_DEBUG = false

export type YearData = {
  lunarYear: number
  newMoonJds: Float64Array
  zhongQiJds: Float64Array
}

/** 岁数据使用的历表数据源，用于状态栏展示 */
export type EphemerisDataSource = 'full_binary' | 'vsop_binary_elp_text' | 'full_text'

/** 数据源对应的状态栏文案 */
export const EPHEMERIS_SOURCE_LABELS: Record<EphemerisDataSource, string> = {
  full_binary: '历表：全二进制 (.bin/.br)',
  vsop_binary_elp_text: '历表：VSOP87 二进制 + ELP 文本',
  full_text: '历表：全文本 (.ear / ELP)',
}

import type { LunarBackend } from './lunar-backend-types'
import { loadLunarBackend, type RealBackendVariant } from './lunar-backend-loader'
import { fetchBinaryMaybeBrotli, isVsop87Binary, isElpBinary } from './fetchEphemerisBinary'

export type { RealBackendVariant } from './lunar-backend-loader'

const backendCache: Partial<Record<RealBackendVariant, LunarBackend>> = {}
const yearDataCache = new Map<number, YearData>()
let cachedVsop87Ear: string | null = null

/** Worker 计算岁数据，避免阻塞主线程；仅在全二进制路径可用时生效 */
let yearDataWorker: Worker | null = null
const yearDataWorkerPending = new Map<
  number,
  {
    resolve: (d: YearData) => void
    reject: (e: unknown) => void
    lunarYear: number
    onSourceUsed?: (source: EphemerisDataSource) => void
  }
>()
let yearDataWorkerId = 0
/** Worker 无响应时超时（毫秒），超时后回退主线程计算，避免界面卡死 */
const YEAR_DATA_WORKER_TIMEOUT_MS = 45_000

function getYearDataWorker(): Worker | null {
  if (yearDataWorker != null) return yearDataWorker
  try {
    const w = new Worker(new URL('./yearDataWorker.ts', import.meta.url), { type: 'module' })
    w.onmessage = (ev: MessageEvent<{ type?: string; id?: number; yearData?: YearData; fallback?: boolean; lunarYear?: number }>) => {
      const { id, yearData: data, fallback } = ev.data
      if (typeof id === 'number') {
        const pending = yearDataWorkerPending.get(id)
        yearDataWorkerPending.delete(id)
        if (pending) {
          if (data) {
            yearDataCache.set(data.lunarYear, data)
            pending.onSourceUsed?.('full_binary')
            pending.resolve(data)
          } else if (fallback && typeof pending.lunarYear === 'number') {
            getYearDataMainThreadImpl(pending.lunarYear, undefined, pending.onSourceUsed).then(pending.resolve).catch(pending.reject)
          } else {
            pending.reject(new Error('Worker year data failed'))
          }
        }
      }
    }
    w.onerror = () => {
      yearDataWorker = null
      for (const [, p] of yearDataWorkerPending) p.reject(new Error('Worker error'))
      yearDataWorkerPending.clear()
    }
    yearDataWorker = w
  } catch {
    yearDataWorker = null
  }
  return yearDataWorker
}

/**
 * 加载农历后端（WASM），按 Real 变体分别缓存。
 * @param variant 'f64' = 默认体积/速度优先，'twofloat' = 高精度
 */
export async function loadWasm(variant: RealBackendVariant = 'f64'): Promise<LunarBackend> {
  const cached = backendCache[variant]
  if (cached) return cached
  const backend = await loadLunarBackend(variant)
  backendCache[variant] = backend
  return backend
}

const ELP_KEYS = [
  'elp_main_s1',
  'elp_main_s2',
  'elp_main_s3',
  'elp_pert_s1',
  'elp_pert_s2',
  'elp_pert_s3',
] as const

async function fetchElpTexts(): Promise<Record<(typeof ELP_KEYS)[number], string>> {
  const entries = await Promise.all(
    ELP_KEYS.map(async (k) => {
      const url = FILES[k]
      const r = await fetch(url)
      if (!r.ok) throw new Error(`fetch ${url}: ${r.status}`)
      return [k, await r.text()] as const
    })
  )
  return Object.fromEntries(entries) as Record<(typeof ELP_KEYS)[number], string>
}

async function fetchAll(): Promise<Record<keyof typeof FILES, string>> {
  const textKeys = (Object.keys(FILES) as (keyof typeof FILES)[]).filter(
    (k) => k !== 'vsop87_ear_bin' && !ELP_BIN_KEYS.includes(k as (typeof ELP_BIN_KEYS)[number])
  )
  const entries = await Promise.all(
    textKeys.map(async (k) => {
      const url = FILES[k]
      const r = await fetch(url)
      if (!r.ok) throw new Error(`fetch ${url}: ${r.status}`)
      return [k, await r.text()] as const
    })
  )
  const out = Object.fromEntries(entries) as Record<string, string>
  if (out.vsop87_ear) {
    cachedVsop87Ear = out.vsop87_ear
    if (GANZHI_DEBUG) console.log('[干支历] fetchAll 已设置 cachedVsop87Ear 长度', cachedVsop87Ear.length)
  }
  return out
}

/** 仅拉取 VSOP87，用于节气历年/月干支兜底。若已缓存则直接 resolve。 */
export async function ensureVsop87Cached(): Promise<void> {
  if (cachedVsop87Ear != null) {
    if (GANZHI_DEBUG) console.log('[干支历] ensureVsop87Cached 已缓存，跳过')
    return
  }
  if (GANZHI_DEBUG) console.log('[干支历] ensureVsop87Cached 开始拉取', FILES.vsop87_ear)
  const r = await fetch(FILES.vsop87_ear)
  if (!r.ok) throw new Error(`fetch ${FILES.vsop87_ear}: ${r.status}`)
  cachedVsop87Ear = await r.text()
  if (GANZHI_DEBUG) console.log('[干支历] ensureVsop87Cached 已设置 长度', cachedVsop87Ear.length)
}

/** 是否已缓存 VSOP87（节气历年/月干支兜底用）。用于避免重复触发加载。 */
export function isVsop87Cached(): boolean {
  return cachedVsop87Ear != null
}

/**
 * 获取指定农历年的岁数据（14 朔 + 12 中气），优先走缓存。
 * 朔（newMoonJds）即初一时刻，干支历「月界=朔日(初一)」时，月柱按初一换，依赖本接口计算的朔日。
 * @param backend 指定后端时用其计算，否则用默认 loadWasm()
 * @param onSourceUsed 得到岁数据后回调当前历表数据源（用于状态栏展示）
 */
const ELP_BIN_KEYS = [
  'elp_main_s1_bin',
  'elp_main_s2_bin',
  'elp_main_s3_bin',
  'elp_pert_s1_bin',
  'elp_pert_s2_bin',
  'elp_pert_s3_bin',
] as const

export async function getYearData(
  lunarYear: number,
  backend?: LunarBackend,
  onSourceUsed?: (source: EphemerisDataSource) => void
): Promise<YearData> {
  const cached = yearDataCache.get(lunarYear)
  if (cached) return cached

  const worker = !backend ? getYearDataWorker() : null
  if (worker) {
    const id = ++yearDataWorkerId
    const workerPromise = new Promise<YearData>((resolve, reject) => {
      yearDataWorkerPending.set(id, { resolve, reject, lunarYear, onSourceUsed })
      worker.postMessage({ type: 'getYearData', id, lunarYear })
    })
    const timeoutPromise = new Promise<YearData>((_, reject) => {
      setTimeout(() => {
        if (yearDataWorkerPending.has(id)) {
          yearDataWorkerPending.delete(id)
          reject(new Error('Worker timeout'))
        }
      }, YEAR_DATA_WORKER_TIMEOUT_MS)
    })
    return Promise.race([workerPromise, timeoutPromise]).catch(() =>
      getYearDataMainThreadImpl(lunarYear, backend, onSourceUsed)
    )
  }

  return getYearDataMainThreadImpl(lunarYear, backend, onSourceUsed)
}

async function getYearDataMainThreadImpl(
  lunarYear: number,
  backend?: LunarBackend,
  onSourceUsed?: (source: EphemerisDataSource) => void
): Promise<YearData> {
  const w = backend ?? await loadWasm()

  if (typeof w.compute_year_data_full_binary === 'function') {
    const settled = await Promise.allSettled(EPHEMERIS_BINARY_URL_LIST.map((url) => fetchBinaryMaybeBrotli(url)))
    const vsop87Bin = settled[0].status === 'fulfilled' ? settled[0].value : null
    const elpBins = settled
      .slice(1, 7)
      .map((r) => (r.status === 'fulfilled' ? r.value : null))
      .filter((u): u is Uint8Array => u != null && u.length >= 4)
    const hasAll = Boolean(
      vsop87Bin &&
        elpBins.length === 6 &&
        isVsop87Binary(vsop87Bin) &&
        elpBins.every(isElpBinary)
    )
    if (import.meta.env.DEV && !hasAll) {
      const vsopOk = vsop87Bin != null && isVsop87Binary(vsop87Bin)
      const elpOk = elpBins.length === 6 && elpBins.every(isElpBinary)
      console.log('[历表] 未走全二进制:', {
        vsop: vsop87Bin ? `${vsop87Bin.length}B, magic=${vsopOk}` : 'rejected',
        elpCount: elpBins.length,
        elpMagicOk: elpOk,
        settled: settled.map((s) => (s.status === 'fulfilled' ? 'ok' : 'rejected')),
      })
    }
    if (hasAll) {
      const result = w.compute_year_data_full_binary(
        lunarYear,
        vsop87Bin,
        elpBins[0]!,
        elpBins[1]!,
        elpBins[2]!,
        elpBins[3]!,
        elpBins[4]!,
        elpBins[5]!
      )
      const yearData: YearData = {
        lunarYear: result.lunar_year,
        newMoonJds: new Float64Array(result.new_moon_jds),
        zhongQiJds: new Float64Array(result.zhong_qi_jds),
      }
      yearDataCache.set(lunarYear, yearData)
      if (import.meta.env.DEV) console.log('[历表] 使用全二进制')
      onSourceUsed?.('full_binary')
      return yearData
    }
  }

  let vsop87BinFallback: Uint8Array | null = null
  try {
    vsop87BinFallback = await fetchBinaryMaybeBrotli(FILES.vsop87_ear_bin)
  } catch {
    // ignore
  }
  if (vsop87BinFallback && isVsop87Binary(vsop87BinFallback) && typeof w.compute_year_data_from_binary === 'function') {
    const vsop87Bin = vsop87BinFallback
    const elp = await fetchElpTexts()
    const result = w.compute_year_data_from_binary(
      lunarYear,
      vsop87Bin,
      elp.elp_main_s1,
      elp.elp_main_s2,
      elp.elp_main_s3,
      elp.elp_pert_s1,
      elp.elp_pert_s2,
      elp.elp_pert_s3
    )
    const yearData: YearData = {
      lunarYear: result.lunar_year,
      newMoonJds: new Float64Array(result.new_moon_jds),
      zhongQiJds: new Float64Array(result.zhong_qi_jds),
    }
    yearDataCache.set(lunarYear, yearData)
    onSourceUsed?.('vsop_binary_elp_text')
    return yearData
  }

  const texts = await fetchAll()
  const result = w.compute_year_data_wasm(
    lunarYear,
    texts.vsop87_ear,
    texts.elp_main_s1,
    texts.elp_main_s2,
    texts.elp_main_s3,
    texts.elp_pert_s1,
    texts.elp_pert_s2,
    texts.elp_pert_s3
  )
  const yearData: YearData = {
    lunarYear: result.lunar_year,
    newMoonJds: new Float64Array(result.new_moon_jds),
    zhongQiJds: new Float64Array(result.zhong_qi_jds),
  }
  yearDataCache.set(lunarYear, yearData)
  onSourceUsed?.('full_text')
  return yearData
}

/**
 * 根据当前显示的公历 (年, 月) 决定需要哪一「岁」的气朔。
 * 流程：先定格里高利历范围 → 再取该范围对应的主岁（及必要时相邻岁）。
 * - 1 月：可能跨春节，取 [year-1, year]，主岁 year-1
 * - 2 月：春节多在 2 月，月初可能仍属上岁，取 [year, year-1]（主岁 year，优先试当年岁）
 * - 12 月：可能跨冬至入下一岁，取 [year, year+1]，主岁 year
 * - 3～11 月：仅本公历年的岁 [year]
 */
export function getLunarYearsForDisplay(year: number, month: number): number[] {
  if (month === 1) return [year - 1, year]
  if (month === 2) return [year, year - 1]
  if (month === 12) return [year, year + 1]
  return [year]
}

/**
 * 按「排格里高利历 → 取得该年气朔」顺序：为当前显示月取岁数据。
 * 返回 [主岁, ...次岁]，主岁优先用于排农历与干支。
 * @param backend 指定后端时用其计算
 * @param onSourceUsed 得到岁数据后回调当前历表数据源（用于状态栏展示）
 */
export async function getYearDataForDisplay(
  displayYear: number,
  displayMonth: number,
  backend?: LunarBackend,
  onSourceUsed?: (source: EphemerisDataSource) => void
): Promise<YearData[]> {
  const lunarYears = getLunarYearsForDisplay(displayYear, displayMonth)
  const uniq = [...new Set(lunarYears)]
  const list = await Promise.all(uniq.map((ly) => getYearData(ly, backend, onSourceUsed)))
  // 保证顺序与 getLunarYearsForDisplay 一致，主岁在前
  const order = lunarYears
  return order.map((ly) => list.find((d) => d.lunarYear === ly)!).filter(Boolean)
}

/**
 * 获取多个农历年的岁数据（用于跨农历年的月份），并行加载。
 * @deprecated 优先使用 getYearDataForDisplay(displayYear, displayMonth) 按流程取主岁。
 */
export async function getYearDataList(lunarYears: number[]): Promise<YearData[]> {
  const uniq = [...new Set(lunarYears)].sort((a, b) => a - b)
  return Promise.all(uniq.map((ly) => getYearData(ly)))
}

/** 显示层：农历月名（简中）。可替换为繁中/多语言。 */
export const LUNAR_MONTH_NAMES = '正二三四五六七八九十冬腊'.split('')

const D1 = '一二三四五六七八九'
/** 农历日数字 → 初一、廿六 等显示，供公历/农历年月日分开展示用 */
export function lunarDayStr(day: number): string {
  if (day === 1) return '初一'
  if (day <= 9) return '初' + D1[day - 1]  // 初二→D1[1]='二'，原 day-2 导致初二显示成初一
  if (day === 10) return '初十'
  if (day <= 19) return '十' + D1[day - 11]
  if (day === 20) return '二十'
  if (day <= 29) return '廿' + D1[day - 21]
  return '三十'
}

/**
 * 显示层：将结构化农历数据格式化为当前语言的日期串（此处为简中）。
 * 第一层：完整格式，如 正月初一、正月初二、二月十三。
 * 后续可接繁简、英文等 i18n。month=0 表示无数据，返回 null。
 */
export function formatLunarFromStruct(
  lunarYear: number,
  lunarMonth: number,
  dayOfMonth: number,
  isLeapMonth: number
): string | null {
  if (lunarMonth <= 0 || dayOfMonth <= 0) return null
  const monthStr = isLeapMonth ? '闰' + LUNAR_MONTH_NAMES[lunarMonth - 1] + '月' : LUNAR_MONTH_NAMES[lunarMonth - 1] + '月'
  return monthStr + lunarDayStr(dayOfMonth)
}

/** 单日农历数据槽，用于显示层管道 */
export type LunarSlot = {
  lunarYear: number
  lunarMonth: number
  dayOfMonth: number
  isLeapMonth: number
  daysInMonth: number
}

/** 显示层函数：(slot, 上一层输出) => 本层输出。第一层时 prev 为 undefined。 */
export type LunarDisplayLayer = (slot: LunarSlot, prev: string | null) => string | null

/** 第一层：完整格式 正月初一、二月十三 等 */
const layer1Full: LunarDisplayLayer = (slot, _prev) => {
  return formatLunarFromStruct(slot.lunarYear, slot.lunarMonth, slot.dayOfMonth, slot.isLeapMonth)
}

/** 第二层：初一 → 月名+大/小，其余 → 仅日（初二、廿三） */
const layer2Short: LunarDisplayLayer = (slot, _prev) => {
  if (slot.lunarMonth <= 0 || slot.dayOfMonth <= 0) return null
  if (slot.dayOfMonth === 1) {
    const monthStr = slot.isLeapMonth ? '闰' + LUNAR_MONTH_NAMES[slot.lunarMonth - 1] + '月' : LUNAR_MONTH_NAMES[slot.lunarMonth - 1] + '月'
    const size = slot.daysInMonth >= 30 ? '大' : '小'
    return monthStr + size
  }
  return lunarDayStr(slot.dayOfMonth)
}

/** 当前启用的显示层序列，后续可追加第三层、第四层… */
const LUNAR_DISPLAY_LAYERS: LunarDisplayLayer[] = [layer1Full, layer2Short]

/**
 * 按优先级应用显示层管道，得到最终展示串。
 * @param slot 单日农历数据（含 daysInMonth 用于初一显示大/小）
 * @param maxLayer 使用到第几层（1=仅第一层 正月初一，2=第二层 正月大/初二）
 */
export function applyLunarDisplayLayers(slot: LunarSlot, maxLayer: number): string | null {
  let out: string | null = null
  const n = Math.min(maxLayer, LUNAR_DISPLAY_LAYERS.length)
  for (let i = 0; i < n; i++) {
    out = LUNAR_DISPLAY_LAYERS[i](slot, out)
    if (out == null) return null
  }
  return out
}

/**
 * 公历 (y,m,d) → 农历显示字符串；需先有该日所在农历年的岁数据。
 */
export function formatLunar(
  wasm: LunarBackend,
  year: number,
  month: number,
  day: number,
  data: YearData
): string | null {
  // 每次传入副本，避免 wasm 多次调用时共享同一 buffer 导致结果错乱
  const r = wasm.gregorian_to_chinese_lunar(
    year,
    month,
    day,
    data.lunarYear,
    new Float64Array(data.newMoonJds),
    new Float64Array(data.zhongQiJds)
  )
  if (r == null) return null
  const slot: LunarSlot = {
    lunarYear: r.year,
    lunarMonth: r.month,
    dayOfMonth: r.day,
    isLeapMonth: r.is_leap_month ? 1 : 0,
    daysInMonth: r.days_in_month ?? 30,
  }
  return applyLunarDisplayLayers(slot, 2)
}

/**
 * 用主岁（及必要时次岁）排农历。约定 yearDataList 已按 getYearDataForDisplay 顺序 [主岁, 次岁?]。
 * 先试主岁，主岁有结果即采用；正月初一只认主岁，避免两岁各算一次初一。
 */
export function formatLunarWithFallback(
  wasm: LunarBackend,
  year: number,
  month: number,
  day: number,
  yearDataList: YearData[],
  displayYear?: number,
  displayMonth?: number
): string | null {
  const primary = yearDataList[0]
  const primaryLunarYear = primary?.lunarYear
  for (const data of yearDataList) {
    const r = wasm.gregorian_to_chinese_lunar(
      year,
      month,
      day,
      data.lunarYear,
      new Float64Array(data.newMoonJds),
      new Float64Array(data.zhongQiJds)
    )
    if (r == null) continue
    if (r.month === 1 && r.day === 1 && primaryLunarYear != null && data.lunarYear !== primaryLunarYear) continue
    const slot: LunarSlot = {
      lunarYear: r.year,
      lunarMonth: r.month,
      dayOfMonth: r.day,
      isLeapMonth: r.is_leap_month ? 1 : 0,
      daysInMonth: r.days_in_month ?? 30,
    }
    return applyLunarDisplayLayers(slot, 2)
  }
  return null
}

/**
 * 公历 (y,m,d) → 农历结构化槽（年月日），用于分开展示 农历年/月/日。无数据时返回 null。
 */
export function getLunarSlotForDay(
  wasm: LunarBackend,
  year: number,
  month: number,
  day: number,
  yearDataList: YearData[]
): LunarSlot | null {
  const primary = yearDataList[0]
  const primaryLunarYear = primary?.lunarYear
  for (const data of yearDataList) {
    const r = wasm.gregorian_to_chinese_lunar(
      year,
      month,
      day,
      data.lunarYear,
      new Float64Array(data.newMoonJds),
      new Float64Array(data.zhongQiJds)
    )
    if (r == null) continue
    if (r.month === 1 && r.day === 1 && primaryLunarYear != null && data.lunarYear !== primaryLunarYear) continue
    return {
      lunarYear: r.year,
      lunarMonth: r.month,
      dayOfMonth: r.day,
      isLeapMonth: r.is_leap_month ? 1 : 0,
      daysInMonth: r.days_in_month ?? 30,
    }
  }
  return null
}

/** 干支历预设：0=子平八字 1=紫微斗数 2=民俗黄历 3=协纪辨方书。每项 [year_boundary, month_boundary, leap_handling, day_boundary] */
export const GANZHI_PRESET_OPTIONS: [number, number, number, number][] = [
  [0, 0, 0, 0],
  [1, 1, 2, 1],
  [1, 1, 1, 0],
  [0, 0, 1, 0],
]

/** 预设名称，与 GANZHI_PRESET_OPTIONS 下标对应 */
export const GANZHI_PRESET_NAMES = ['子平八字', '紫微斗数', '民俗黄历', '协纪辨方书'] as const

/** 详细选项：年界 0=立春 1=春节 2=冬至 */
export const GANZHI_YEAR_BOUNDARY_OPTIONS: { value: number; label: string }[] = [
  { value: 0, label: '立春' },
  { value: 1, label: '春节(正月初一)' },
  { value: 2, label: '冬至' },
]
/** 月界 0=节气 1=朔日(初一) */
export const GANZHI_MONTH_BOUNDARY_OPTIONS: { value: number; label: string }[] = [
  { value: 0, label: '节气' },
  { value: 1, label: '朔日(初一)' },
]
/** 闰月 0=忽略 1=随前月 2=分半月 3=顺延下一月 */
export const GANZHI_LEAP_MONTH_OPTIONS: { value: number; label: string }[] = [
  { value: 0, label: '忽略' },
  { value: 1, label: '随前月' },
  { value: 2, label: '分半月' },
  { value: 3, label: '顺延下一月' },
]
/** 日界 0=23时(子初) 1=0时(子正) */
export const GANZHI_DAY_BOUNDARY_OPTIONS: { value: number; label: string }[] = [
  { value: 0, label: '23时(子初)' },
  { value: 1, label: '0时(子正)' },
]

export type GanzhiOptionsTuple = [number, number, number, number]

/** 当前选项与某预设一致则返回其下标，否则返回 -1 */
export function getPresetIndexFromOptions(opts: GanzhiOptionsTuple): number {
  const [yb, mb, leap, db] = opts
  const idx = GANZHI_PRESET_OPTIONS.findIndex((p) => p[0] === yb && p[1] === mb && p[2] === leap && p[3] === db)
  return idx >= 0 ? idx : -1
}

export type GanzhiResult = { yearName: string; monthName: string; dayName: string }

/**
 * 公历整月干支（批量一次 WASM 调用）。若后端支持 ganzhiForGregorianMonthWasm 则走批量，否则逐日 getGanzhiForDay 兜底。
 * 返回 Record<day, GanzhiResult>，day 从 1 到该月天数。
 */
export function getGanzhiForMonth(
  wasm: LunarBackend,
  year: number,
  month: number,
  yearDataList: YearData[],
  opts: GanzhiOptionsTuple
): Record<number, GanzhiResult | null> {
  const days = new Date(year, month, 0).getDate()
  const [yb, mb, leap, db] = opts

  if (typeof wasm.ganzhiForGregorianMonthWasm === 'function' && yearDataList.length > 0) {
    const primary = yearDataList[0]!
    const secondary = yearDataList[1]
    const res = wasm.ganzhiForGregorianMonthWasm(
      year,
      month,
      primary.lunarYear,
      primary.lunarYear,
      new Float64Array(primary.newMoonJds),
      new Float64Array(primary.zhongQiJds),
      secondary?.lunarYear ?? 0,
      secondary ? new Float64Array(secondary.newMoonJds) : [],
      secondary ? new Float64Array(secondary.zhongQiJds) : [],
      yb,
      mb,
      leap,
      db,
      cachedVsop87Ear ?? ''
    )
    const yearNames = res.yearNames ?? []
    const monthNames = res.monthNames ?? []
    const dayNames = res.dayNames ?? []
    const out: Record<number, GanzhiResult | null> = {}
    for (let day = 1; day <= days; day++) {
      const i = day - 1
      out[day] = {
        yearName: yearNames[i] ?? '',
        monthName: monthNames[i] ?? '',
        dayName: dayNames[i] ?? '',
      }
    }
    return out
  }

  const out: Record<number, GanzhiResult | null> = {}
  for (let day = 1; day <= days; day++) {
    out[day] = getGanzhiForDay(wasm, year, month, day, yearDataList, opts)
  }
  return out
}

/**
 * 公历 (y,m,d) 的干支历年月日柱；按选项 [年界, 月界, 闰月, 日界] 计算。
 *
 * 干支历的年/月划分有两种方式：
 * - 年界=春节(1) 且 月界=朔日(1)：年、月按「农历岁」划分；月柱按初一换，需朔日（初一）时刻，
 *   由 getYearData 得到的 newMoonJds 经 ganzhi_from_jd_lunar_wasm 使用。
 * - 年界=立春(0) 或 月界=节气(0)：年、月按节气划分，用节气历 solar（ganzhi_from_jd_solar_wasm），
 *   如子平八字、协纪辨方书。
 *
 * 若为春节+朔日派但 wasm 对当前 JD 在已有岁数据中均返回 null（如该日不在已加载岁范围内），
 * 则用节气历结果兜底，避免年月为空。
 *
 * 性能：VSOP87 文本仅在前端拉取一次（cachedVsop87Ear）；WASM 内会缓存解析后的 Vsop87，
 * 同一会话中节气历计算不再重复解析。日历整月干支由 App 层 useMemo(ganzhiByDay) 按 (year, month, opts) 算一次供格子复用。
 */
export function getGanzhiForDay(
  wasm: LunarBackend,
  year: number,
  month: number,
  day: number,
  yearDataList: YearData[],
  opts: GanzhiOptionsTuple
): GanzhiResult {
  const [yb, mb, leap, db] = opts
  const jd = wasm.gregorian_to_jd(year, month, day)
  const dayName = wasm.gregorian_to_gan_zhi_day_with_options(year, month, day, db)

  if (GANZHI_DEBUG) {
    console.log('[干支历] getGanzhiForDay 入参', {
      date: `${year}-${month}-${day}`,
      jd,
      opts: { yb, mb, leap, db },
      yearDataListLen: yearDataList.length,
      hasCachedVsop87: cachedVsop87Ear != null,
      cachedVsop87Len: cachedVsop87Ear?.length ?? 0,
    })
  }

  /** wasm GanzhiResult 的 getter 是方法，必须用正确 this 调用，否则 __wbg_ptr 为 undefined */
  const getStr = (o: unknown, key: 'year_name' | 'month_name' | 'day_name'): string => {
    if (o == null) return ''
    const v = (o as Record<string, unknown>)[key]
    if (typeof v === 'string') return v
    if (typeof v === 'function') return String((v as (this: unknown) => string).call(o))
    return ''
  }

  let solarFallback: { yearName: string; monthName: string } | null = null
  if (cachedVsop87Ear != null) {
    try {
      const r = wasm.ganzhi_from_jd_solar_wasm(jd, cachedVsop87Ear, db)
      const yn = getStr(r, 'year_name')
      const mn = getStr(r, 'month_name')
      if (GANZHI_DEBUG) {
        console.log('[干支历] 节气历 solar 结果', {
          year_name: yn,
          month_name: mn,
          day_name: getStr(r, 'day_name'),
          rawKeys: r != null ? Object.keys(r) : [],
          year_nameType: r != null ? typeof (r as Record<string, unknown>).year_name : '-',
        })
      }
      if (yn && mn) solarFallback = { yearName: yn, monthName: mn }
    } catch (e) {
      console.error('[干支历] 节气历兜底失败:', e)
    }
  } else if (GANZHI_DEBUG) {
    console.log('[干支历] 未缓存 VSOP87，跳过节气历')
  }

  if (yb === 1 && mb === 1) {
    for (const data of yearDataList) {
      let r: { year_name?: string; month_name?: string } | null = null
      try {
        r = wasm.ganzhi_from_jd_lunar_wasm(
          jd,
          data.lunarYear,
          new Float64Array(data.newMoonJds),
          new Float64Array(data.zhongQiJds),
          yb,
          mb,
          leap,
          db
        )
      } catch (e) {
        if (GANZHI_DEBUG) console.warn('[干支历] 农历派 wasm 抛错', { lunarYear: data.lunarYear }, e)
      }
      if (GANZHI_DEBUG) {
        console.log('[干支历] 农历派 lunar', {
          lunarYear: data.lunarYear,
          rNull: r == null,
          year_name: r != null ? getStr(r, 'year_name') : '-',
          month_name: r != null ? getStr(r, 'month_name') : '-',
          newMoonLen: data.newMoonJds?.length,
          zhongQiLen: data.zhongQiJds?.length,
        })
      }
      if (r != null) {
        const yn = getStr(r, 'year_name')
        const mn = getStr(r, 'month_name')
        if (yn && mn) {
          if (GANZHI_DEBUG) console.log('[干支历] 使用农历派结果', { yearName: yn, monthName: mn })
          return { yearName: yn, monthName: mn, dayName }
        }
      }
    }
    if (GANZHI_DEBUG) console.log('[干支历] 农历派全部无匹配或空年/月')
  }

  if (solarFallback) {
    if (GANZHI_DEBUG) console.log('[干支历] 使用节气历兜底', solarFallback)
    return { ...solarFallback, dayName }
  }
  if (GANZHI_DEBUG) console.log('[干支历] 年/月为空，返回空')
  return { yearName: '', monthName: '', dayName }
}
