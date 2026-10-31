/**
 * Web Worker：星图计算。首次请求时 fetch 历表（日月五星）、星表与星官数据，构造常驻的 SkyEngine（wasm-lib-f64），
 * 之后的快照在 Worker 内算，避免阻塞 UI。数据由 `npm run copy-data` 拷到 /data/。
 */

import { fetchFullBinaryEphemerisSet, fetchNutationTab53aBin, applyEarthPatchIfAvailable } from './ephemerisSourceRegistry'
import { DATA_BASE } from './ephemerisUrls'

type Engine = {
  skyCulture(): string
  snapshot(jdTt: number, deltaTS: number): string
  systemForYear(year: number): string
}

type WasmModule = {
  default?: () => Promise<unknown>
  init_vsop87_earth_patch_from_binary?: (bytes: Uint8Array) => boolean
  deltaTEstimate(jdTt: number): Float64Array
  SkyEngine: new (
    vsop: Uint8Array,
    e1: Uint8Array,
    e2: Uint8Array,
    e3: Uint8Array,
    p1: Uint8Array,
    p2: Uint8Array,
    p3: Uint8Array,
    nutation: Uint8Array,
    planetTexts: string,
    planetPatchText: string,
    starsCsv: string,
    asterismsCsv: string,
    starNamesCsv: string,
    mansionsCsv: string,
    periodsCsv: string,
    schoolsCsv: string
  ) => Engine
}

export type SkyRequest =
  | { id: number; type: 'culture' }
  | { id: number; type: 'snapshot'; jdTt: number; deltaT: number }
  | { id: number; type: 'deltaT'; jd: number }
  | { id: number; type: 'systemForYear'; year: number }

async function text(path: string): Promise<string> {
  const r = await fetch(`${DATA_BASE}/${path}`)
  if (!r.ok) throw new Error(`取 /data/${path} 失败（${r.status}），请先在 web/ 下执行 npm run copy-data`)
  return r.text()
}

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
      const planetTexts = await Promise.all(['mer', 'ven', 'mar', 'jup', 'sat'].map((b) => text(`vsop87/VSOP87B.${b}`)))
      const patch = await text('fit/vsop87-de406-planets.txt').catch(() => '')
      const [stars, asterisms, names, mansions, periods, schools] = await Promise.all([
        text('stars/hipparcos_bright.csv'),
        text('sky_culture/chinese_asterisms.csv'),
        text('sky_culture/chinese_star_names.csv'),
        text('sky_culture/chinese_lunar_mansion_determinatives.csv'),
        text('sky_culture/system_periods.csv'),
        text('sky_culture/sanjia_schools.csv'),
      ])
      const [e1, e2, e3, p1, p2, p3] = set.elpBins
      const engine = new wasm.SkyEngine(
        set.vsop87Bin, e1, e2, e3, p1, p2, p3, nutation,
        planetTexts.join('\u001e'), patch, stars, asterisms, names, mansions, periods, schools
      )
      return { wasm, engine }
    })()
    enginePromise.catch(() => {
      enginePromise = null
    })
  }
  return enginePromise
}

self.onmessage = async (ev: MessageEvent<SkyRequest>) => {
  const req = ev.data
  try {
    const { wasm, engine } = await getEngine()
    let result: unknown
    switch (req.type) {
      case 'culture':
        result = JSON.parse(engine.skyCulture())
        break
      case 'snapshot':
        result = JSON.parse(engine.snapshot(req.jdTt, req.deltaT))
        break
      case 'deltaT':
        result = Array.from(wasm.deltaTEstimate(req.jd))
        break
      case 'systemForYear':
        result = JSON.parse(engine.systemForYear(req.year))
        break
    }
    self.postMessage({ id: req.id, result })
  } catch (e) {
    self.postMessage({ id: req.id, error: e instanceof Error ? e.message : String(e) })
  }
}
