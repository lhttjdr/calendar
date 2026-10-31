/**
 * Web Worker：日食计算。首次请求时 fetch 全二进制历表并构造常驻的 EclipseEngine（wasm-lib-f64），之后的
 * 年表、地图、某地食况都在 Worker 内算，避免阻塞 UI。数据源取舍沿用 ephemerisSourceRegistry.ts（只走全二进制一级）。
 */

import { fetchFullBinaryEphemerisSet, fetchNutationTab53aBin, applyEarthPatchIfAvailable } from './ephemerisSourceRegistry'

type Engine = {
  solarEclipsesInYear(year: number): string
  solarEclipseMap(jdTtGuess: number, deltaTS: number, stepS: number, outlineIntervalS: number): string
  solarEclipseLocal(jdTtGuess: number, deltaTS: number, lonDeg: number, latDeg: number): string
}

type WasmModule = {
  default?: () => Promise<unknown>
  init_vsop87_earth_patch_from_binary?: (bytes: Uint8Array) => boolean
  deltaTEstimate(jdTt: number): Float64Array
  EclipseEngine: new (
    vsop: Uint8Array,
    e1: Uint8Array,
    e2: Uint8Array,
    e3: Uint8Array,
    p1: Uint8Array,
    p2: Uint8Array,
    p3: Uint8Array,
    nutation: Uint8Array
  ) => Engine
}

export type EclipseRequest =
  | { id: number; type: 'list'; year: number }
  | { id: number; type: 'map'; jd: number; deltaT: number; stepS: number; outlineS: number }
  | { id: number; type: 'local'; jd: number; deltaT: number; lon: number; lat: number }
  | { id: number; type: 'deltaT'; jd: number }

let enginePromise: Promise<{ wasm: WasmModule; engine: Engine }> | null = null

function getEngine() {
  if (!enginePromise) {
    enginePromise = (async () => {
      const wasm = (await import('lunar-wasm-f64')) as unknown as WasmModule
      if (typeof wasm.default === 'function') await wasm.default()
      const set = await fetchFullBinaryEphemerisSet()
      if (!set) throw new Error('未取到历表二进制（请先在 web/ 下执行 npm run copy-data）')
      await applyEarthPatchIfAvailable(wasm)
      const nutation = await fetchNutationTab53aBin()
      const [e1, e2, e3, p1, p2, p3] = set.elpBins
      const engine = new wasm.EclipseEngine(set.vsop87Bin, e1, e2, e3, p1, p2, p3, nutation)
      return { wasm, engine }
    })()
    enginePromise.catch(() => {
      enginePromise = null
    })
  }
  return enginePromise
}

self.onmessage = async (ev: MessageEvent<EclipseRequest>) => {
  const req = ev.data
  try {
    const { wasm, engine } = await getEngine()
    let result: unknown
    switch (req.type) {
      case 'list':
        result = JSON.parse(engine.solarEclipsesInYear(req.year))
        break
      case 'map':
        result = JSON.parse(engine.solarEclipseMap(req.jd, req.deltaT, req.stepS, req.outlineS))
        break
      case 'local':
        result = JSON.parse(engine.solarEclipseLocal(req.jd, req.deltaT, req.lon, req.lat))
        break
      case 'deltaT':
        result = Array.from(wasm.deltaTEstimate(req.jd))
        break
    }
    self.postMessage({ id: req.id, result })
  } catch (e) {
    self.postMessage({ id: req.id, error: e instanceof Error ? e.message : String(e) })
  }
}
