/**
 * Web Worker：星图计算。首次请求时 fetch 历表（日月五星）、星表与星官数据，构造常驻的 SkyEngine（wasm-lib-f64），
 * 之后的快照在 Worker 内算，避免阻塞 UI。数据由 `npm run copy-data` 拷到 /data/。
 */

import { fetchFullBinaryEphemerisSet, fetchNutationTab53aBin, applyEarthPatchIfAvailable } from './ephemerisSourceRegistry'
import { DATA_BASE } from './ephemerisUrls'

type Engine = {
  skyCulture(): string
  qizheng(jdTt: number, deltaTS: number, lon: number, lat: number, hourZhi: number, luoAsc: boolean, meanNode: boolean, ascendant: boolean, flags: number): string
  snapshot(jdTt: number, deltaTS: number, lon: number, lat: number, heightM: number, tempC: number, pressureKpa: number, invDepthM: number, invRiseK: number, humidity: number, cityNl: number): string
  systemForYear(year: number): string
}

type WasmModule = {
  default?: () => Promise<unknown>
  init_vsop87_earth_patch_from_binary?: (bytes: Uint8Array) => boolean
  deltaTEstimate(jdTt: number): Float64Array
  terrainHorizon(lon: number, lat: number, eye: number, near: Float32Array, nz: number, nx: number, ny: number, nw: number, nh: number, far: Float32Array, fz: number, fx: number, fy: number, fw: number, fh: number): Float64Array
  refractionTable(altitudes: Float64Array, heightM: number, tempC: number, pressureKpa: number, invDepthM: number, invRiseK: number): Float64Array
  seaHorizonDip(heightM: number, tempC: number, pressureKpa: number, invDepthM: number, invRiseK: number): number
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

/** 观测点的海拔与气象。气温、气压为 null 时取该海拔的标准大气；逆温：invDepthM 厚度内升温 invRiseK（厚度 0 为无）；humidity 0—1；cityNl 城市光（nL）。 */
export type Site = { heightM: number; tempC: number | null; pressureKpa: number | null; invDepthM: number; invRiseK: number; humidity: number; cityNl: number }

const atmosphereArgs = (s: Site) => [s.heightM, s.tempC ?? NaN, s.pressureKpa ?? NaN, s.invDepthM, s.invRiseK] as const

export type SkyRequest =
  | { id: number; type: 'culture' }
  | { id: number; type: 'snapshot'; jdTt: number; deltaT: number; lon: number; lat: number; site: Site }
  | { id: number; type: 'refractionTable'; altitudes: number[]; site: Site }
  | { id: number; type: 'seaHorizonDip'; site: Site }
  | { id: number; type: 'terrainHorizon'; lon: number; lat: number; eyeM: number }
  | { id: number; type: 'deltaT'; jd: number }
  | { id: number; type: 'systemForYear'; year: number }
  /** 七政四余（排盘页）：jdUt 为 UT 儒略日，ΔT 由 Worker 估算 */
  | { id: number; type: 'qizheng'; jdUt: number; lon: number; lat: number; hourZhi: number; luoAsc: boolean; meanNode: boolean; ascendant: boolean; flags: number }

// Terrarium 高程瓦片（Mapzen/Tilezen，AWS Open Data）：像素值 → 米 = R·256 + G + B/256 − 32768。署名见 doc/astronomy/sky-view-3d.md
const TILE_URL = (z: number, x: number, y: number) => `https://s3.amazonaws.com/elevation-tiles-prod/terrarium/${z}/${x}/${y}.png`
type Mosaic = { z: number; x0: number; y0: number; w: number; h: number; data: Float32Array; missing: number; tiles: number }

async function fetchTile(z: number, x: number, y: number): Promise<Float32Array | null> {
  const r = await fetch(TILE_URL(z, x, y)).catch(() => null)
  if (!r || !r.ok) return null
  const bmp = await createImageBitmap(await r.blob(), { colorSpaceConversion: 'none', premultiplyAlpha: 'none' })
  const canvas = new OffscreenCanvas(256, 256)
  const ctx = canvas.getContext('2d', { willReadFrequently: true })!
  ctx.drawImage(bmp, 0, 0)
  const px = ctx.getImageData(0, 0, 256, 256).data
  const out = new Float32Array(256 * 256)
  for (let i = 0; i < out.length; i++) out[i] = px[i * 4] * 256 + px[i * 4 + 1] + px[i * 4 + 2] / 256 - 32768
  return out
}

/** 拼出以 (lon, lat) 为中心、半径 radiusM 的高程网格。 */
async function fetchMosaic(lon: number, lat: number, z: number, radiusM: number): Promise<Mosaic> {
  const n = 2 ** z
  const tx = ((lon + 180) / 360) * n
  const phi = (lat * Math.PI) / 180
  const ty = ((1 - Math.log(Math.tan(phi) + 1 / Math.cos(phi)) / Math.PI) / 2) * n
  const tileM = (40075016 * Math.cos(phi)) / n
  const span = Math.ceil(radiusM / tileM)
  const [x0, y0] = [Math.floor(tx) - span, Math.max(0, Math.floor(ty) - span)]
  const size = 2 * span + 1
  const data = new Float32Array(size * 256 * size * 256)
  const jobs: Promise<boolean>[] = []
  for (let j = 0; j < size; j++) {
    for (let i = 0; i < size; i++) {
      jobs.push(
        fetchTile(z, ((x0 + i) % n + n) % n, y0 + j).then((t) => {
          if (!t) return false
          for (let row = 0; row < 256; row++) data.set(t.subarray(row * 256, (row + 1) * 256), (j * 256 + row) * size * 256 + i * 256)
          return true
        })
      )
    }
  }
  const done = await Promise.all(jobs)
  return { z, x0, y0, w: size * 256, h: size * 256, data, tiles: done.length, missing: done.filter((x) => !x).length }
}

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
        result = JSON.parse(engine.snapshot(req.jdTt, req.deltaT, req.lon, req.lat, ...atmosphereArgs(req.site), req.site.humidity, req.site.cityNl))
        break
      case 'refractionTable':
        result = Array.from(wasm.refractionTable(new Float64Array(req.altitudes), ...atmosphereArgs(req.site)))
        break
      case 'terrainHorizon': {
        const near = await fetchMosaic(req.lon, req.lat, 12, 3000)
        const far = await fetchMosaic(req.lon, req.lat, 9, 200000)
        const missing = near.missing + far.missing
        if (missing > (near.tiles + far.tiles) / 4) throw new Error(`高程瓦片有 ${missing}/${near.tiles + far.tiles} 张取不到（需要联网）`)
        const r = wasm.terrainHorizon(req.lon, req.lat, req.eyeM, near.data, near.z, near.x0, near.y0, near.w, near.h, far.data, far.z, far.x0, far.y0, far.w, far.h)
        if (r.length === 0) throw new Error('此地没有高程数据')
        result = { groundM: r[0], altitudes: Array.from(r.subarray(1)), missing }
        break
      }
      case 'seaHorizonDip':
        result = wasm.seaHorizonDip(...atmosphereArgs(req.site))
        break
      case 'deltaT':
        result = Array.from(wasm.deltaTEstimate(req.jd))
        break
      case 'qizheng': {
        const dt = wasm.deltaTEstimate(req.jdUt)[0]!
        result = JSON.parse(engine.qizheng(req.jdUt + dt / 86400, dt, req.lon, req.lat, req.hourZhi, req.luoAsc, req.meanNode, req.ascendant, req.flags))
        break
      }
      case 'systemForYear':
        result = JSON.parse(engine.systemForYear(req.year))
        break
    }
    self.postMessage({ id: req.id, result })
  } catch (e) {
    self.postMessage({ id: req.id, error: e instanceof Error ? e.message : String(e) })
  }
}
