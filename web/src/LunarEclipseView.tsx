/**
 * 月食图：月食时刻全球同一，地图只回答何处月亮在地平上。画各食相的月出月没界线、月下点，把「食始时月在地平下」「食终时月在地平下」
 * 两块区域着色（即月出带食、月没带食，两者重叠处全程不可见，均未着色处全程可见）；另画月亮穿过地影图。点击地图或都城看该地食况。
 *
 * 计算全在 Rust（eclipseWorker → EclipseEngine）。界线为站心几何高度 0（含月亮视差，不计折射）；某地「可见」按标准地平高度 −0°50′。
 * ΔT 只影响经度：滑块平移全图，某地食况按当前 ΔT 重算。
 */
import { useEffect, useMemo, useRef, useState } from 'react'
import { Card, Checkbox, Descriptions, Flex, InputNumber, Radio, Slider, Space, Spin, Typography } from 'antd'
import { geoContains } from 'd3-geo'
import type { Feature, MultiLineString, Polygon } from 'geojson'
import { fmtDate, fmtTime } from './civilTime'
import type { Capital } from './capitals'
import { GeoCanvas, type MapView } from './GeoCanvas'
import { DATA_BASE } from './ephemerisUrls'
import { EclipsePlaybackBar, useEclipsePlayback } from './Playback'
import type { EclipseRequest } from './eclipseWorker'
import type { WithoutId } from './useWorkerRpc'

const { Text } = Typography

type Pt = [number, number, number]
export type LunarListItem = {
  greatestTt: number
  gamma: number
  penumbralMagnitude: number
  umbralMagnitude: number
  kind: 'N' | 'P' | 'T'
  deltaT: number
  deltaTSigma: number
}
type Curve = { phase: string; jdTt: number; sublunar: [Pt]; horizon: Pt[]; east: number; north: number }
type LunarMap = {
  eclipse: Omit<LunarListItem, 'deltaT' | 'deltaTSigma'>
  deltaT: number
  curves: Curve[]
  umbraRadius: number
  penumbraRadius: number
  moonSemidiameter: number
  shadowPath: [number, number, number][]
  lonShiftPerSecond: number
}
/** 月食动画的一帧（Rust lunarEclipseFrame）；食分影外为负 */
type LunarFrame = { jdTt: number; sublunar: [Pt]; horizon: Pt[]; east: number; north: number; umbralMagnitude: number; penumbralMagnitude: number }
type Local = {
  phases: { phase: string; jdTt: number; alt: number }[]
  visibleFrom: number | null
  visibleTo: number | null
  maxVisibleUmbral: number | null
  maxVisiblePenumbral: number | null
}

const PHASE_LABEL: Record<string, string> = { P1: '半影食始', U1: '初亏', U2: '食既', G: '食甚', U3: '生光', U4: '复圆', P4: '半影食终' }
const PHASE_COLOR: Record<string, string> = { P1: '#7f8fa6', U1: '#2e6fb6', U2: '#8e44ad', G: '#c0392b', U3: '#8e44ad', U4: '#d9822b', P4: '#7f8fa6' }
const RISE_FILL = 'rgba(46,111,182,0.20)'
const SET_FILL = 'rgba(217,130,43,0.22)'

/** 月亮在地平下的区域：界线闭合多边形，按对跖点是否在内定走向（d3 以多边形走向定内外）。 */
function belowHorizon(c: Pick<Curve, 'horizon' | 'sublunar'>, dLon: number): Feature<Polygon> {
  const ring = c.horizon.map((p) => [p[0] + dLon, p[1]])
  const s = c.sublunar[0]
  const anti: [number, number] = [s[0] + dLon + 180, -s[1]]
  const make = (r: number[][]): Feature<Polygon> => ({ type: 'Feature', properties: {}, geometry: { type: 'Polygon', coordinates: [r] } })
  const f = make(ring)
  return geoContains(f, anti) ? f : make([...ring].reverse())
}

/** 半径 a、b 两圆心距 d 时的重叠面积。 */
function overlapArea(a: number, b: number, d: number): number {
  if (d >= a + b) return 0
  if (d <= Math.abs(a - b)) return Math.PI * Math.min(a, b) ** 2
  const ca = Math.acos(Math.max(-1, Math.min(1, (d * d + a * a - b * b) / (2 * d * a))))
  const cb = Math.acos(Math.max(-1, Math.min(1, (d * d + b * b - a * a) / (2 * d * b))))
  return a * a * (ca - Math.sin(2 * ca) / 2) + b * b * (cb - Math.sin(2 * cb) / 2)
}

/** 某地某刻的月面几何（Rust moonDiskView，度）。 */
type MoonGeom = { jdTt: number; alt: number; appAlt: number; az: number; zenithPa: number; libLon: number; libLat: number; axisPa: number; vScale: number }

/** LRO 月面图（等距圆柱，中央经线为月面经度 0°，上为北），解码为线性值。近侧半球平均线性亮度见 scripts/phenomena/moon_texture.py。 */
type MoonTexture = { w: number; h: number; lin: Float32Array }
const MOON_TEXTURE_MEAN = 0.2242
let moonTexturePromise: Promise<MoonTexture> | null = null
function loadMoonTexture(): Promise<MoonTexture> {
  moonTexturePromise ??= new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => {
      const c = document.createElement('canvas')
      c.width = img.width
      c.height = img.height
      const ctx = c.getContext('2d')!
      ctx.drawImage(img, 0, 0)
      const d = ctx.getImageData(0, 0, c.width, c.height).data
      const lut = new Float32Array(256).map((_, i) => { const x = i / 255; return x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4 })
      const lin = new Float32Array(c.width * c.height * 3)
      for (let k = 0, o = 0; k < d.length; k += 4, o += 3) {
        lin[o] = lut[d[k]]
        lin[o + 1] = lut[d[k + 1]]
        lin[o + 2] = lut[d[k + 2]]
      }
      resolve({ w: c.width, h: c.height, lin })
    }
    img.onerror = () => reject(new Error('月面贴图加载失败'))
    img.src = `${DATA_BASE}/sky/moon_lroc_color_2k.jpg`
  })
  return moonTexturePromise
}

const MAX_GAIN = 40
/** 大气消光系数（星等/大气质量），R、G、B 三通道：瑞利散射 ∝ λ⁻⁴ 加气溶胶，取晴朗低海拔的典型值 */
const EXTINCTION_K = [0.12, 0.2, 0.32]
/** 满月近侧平均（线性）显示亮度 */
const FULL_MOON_LEVEL = 0.4

/** Kasten–Young 大气质量（视高度，度）。 */
function airmass(hDeg: number): number {
  const h = Math.max(hDeg, -0.5)
  return 1 / (Math.sin((h * Math.PI) / 180) + 0.50572 * (h + 6.07995) ** -1.6364)
}

/**
 * 月面实况：逐像素求该点在月面上的经纬度（取 LRO 贴图）与受照比例。
 * - 受照：地影半径 Rp = π + s′、Ru = π − s′，从月面一点看，日面半径 s′ = (Rp − Ru)/2、地球半径 ρ = (Rp + Ru)/2，两圆心距即该点到地影中心的
 *   角距 r；受照比例 = 1 − 重叠面积/日面积（均匀日面）。本影内亮度与色调由地球大气决定（丹戎等级），几何算不出，只画示意的暗红、向影边渐亮。
 * - 朝向：给定观测地时天顶朝上（按天顶方向的位置角旋转），否则北上东左；月面经纬度由站心天平动 (l, b) 与月轴位置角 P 定。
 * - 观测地的地平：视高度 < 0 处画地面；折射使月面竖直方向缩放 vScale；按大气质量逐通道消光（低空发黄发红）。
 * - 线性亮度按月面最亮处自动曝光（同眼睛、相机的适应），增益至多 MAX_GAIN，再转 sRGB。
 */
function MoonView({ map, east, north, geom, local, onGain }: {
  map: LunarMap; east: number; north: number; geom: MoonGeom | null; local: boolean; onGain: (g: number) => void
}) {
  const ref = useRef<HTMLCanvasElement | null>(null)
  const [tex, setTex] = useState<MoonTexture | null>(null)
  const css = 240
  useEffect(() => {
    let alive = true
    loadMoonTexture().then((t) => alive && setTex(t)).catch(() => {})
    return () => { alive = false }
  }, [])
  useEffect(() => {
    const cv = ref.current
    if (!cv) return
    const dpr = Math.min(2, window.devicePixelRatio || 1)
    const n = Math.round(css * dpr)
    cv.width = n
    cv.height = n
    const ctx = cv.getContext('2d')
    if (!ctx) return
    const img = ctx.createImageData(n, n)
    const rad = Math.PI / 180
    const sm = map.moonSemidiameter
    const scale = (sm * 1.4) / (n / 2) // 每像素多少度（月面直径约占画面七成）
    const sSun = (map.penumbraRadius - map.umbraRadius) / 2
    const rho = (map.penumbraRadius + map.umbraRadius) / 2
    const sunArea = Math.PI * sSun * sSun
    const enc = (x: number) => Math.round(255 * (x <= 0.0031308 ? 12.92 * x : 1.055 * x ** (1 / 2.4) - 0.055))
    // 屏幕「上」的位置角：有观测地时为天顶方向，否则天北
    const pz = local && geom ? geom.zenithPa * rad : 0
    const vs = local && geom ? geom.vScale : 1
    const [spz, cpz] = [Math.sin(pz), Math.cos(pz)]
    // 月面朝向
    const P = (geom?.axisPa ?? 0) * rad
    const [sP, cP] = [Math.sin(P), Math.cos(P)]
    const l0 = (geom?.libLon ?? 0) * rad
    const b0 = (geom?.libLat ?? 0) * rad
    const o = [Math.cos(b0) * Math.cos(l0), Math.cos(b0) * Math.sin(l0), Math.sin(b0)]
    const eL = [-Math.sin(l0), Math.cos(l0), 0]
    const nL = [-Math.sin(b0) * Math.cos(l0), -Math.sin(b0) * Math.sin(l0), Math.cos(b0)]
    const texScale = FULL_MOON_LEVEL / MOON_TEXTURE_MEAN
    const lin = new Float32Array(n * n * 4)
    const ground = new Uint8Array(n * n)
    let peak = 0
    for (let j = 0; j < n; j++) {
      for (let i = 0; i < n; i++) {
        const xs = (i + 0.5 - n / 2) * scale // 屏幕向右
        const ysApp = -(j + 0.5 - n / 2) * scale // 屏幕向上（视）
        const q = j * n + i
        if (local && geom && geom.appAlt + ysApp < 0) {
          ground[q] = 1
          continue
        }
        const ys = ysApp / vs // 折射前
        // 屏幕 → 天球（向东、向北）
        const de = ys * spz - xs * cpz
        const dn = ys * cpz + xs * spz
        const rr = Math.hypot(de, dn)
        if (rr > sm) continue
        // 受照
        const r = Math.hypot(east + de, north + dn)
        const lit = 1 - overlapArea(sSun, rho, r) / sunArea
        let illum: number[]
        if (lit > 0) {
          illum = [lit, lit, lit]
        } else {
          // 本影示意：中心约为满月的 1/2000，影边约 1/100
          const t = Math.max(0, Math.min(1, r / map.umbraRadius))
          const lv = (0.0005 + 0.01 * t ** 4) / FULL_MOON_LEVEL
          illum = [lv, 0.42 * lv, 0.22 * lv]
        }
        // 月面经纬度 → 贴图
        let alb = [FULL_MOON_LEVEL, FULL_MOON_LEVEL * 0.97, FULL_MOON_LEVEL * 0.9]
        if (tex) {
          const a = de / sm
          const c = dn / sm
          const w = Math.sqrt(Math.max(0, 1 - a * a - c * c))
          const yl = a * sP + c * cP // 沿月轴北
          const xw = -(a * cP - c * sP) // 向天西 = 月面经度增加方向
          const X = xw * eL[0] + yl * nL[0] + w * o[0]
          const Y = xw * eL[1] + yl * nL[1] + w * o[1]
          const Z = xw * eL[2] + yl * nL[2] + w * o[2]
          const lon = Math.atan2(Y, X)
          const lat = Math.asin(Math.max(-1, Math.min(1, Z)))
          const u = Math.min(tex.w - 1, Math.floor(((lon / rad + 180) / 360) * tex.w))
          const v = Math.min(tex.h - 1, Math.floor(((90 - lat / rad) / 180) * tex.h))
          const k = (v * tex.w + u) * 3
          alb = [tex.lin[k] * texScale, tex.lin[k + 1] * texScale, tex.lin[k + 2] * texScale]
        }
        // 消光
        let ext = [1, 1, 1]
        if (local && geom) {
          const X = airmass(geom.appAlt + ysApp)
          ext = EXTINCTION_K.map((k) => 10 ** (-0.4 * k * X))
        }
        const p4 = q * 4
        for (let ch = 0; ch < 3; ch++) lin[p4 + ch] = alb[ch] * illum[ch] * ext[ch]
        lin[p4 + 3] = Math.max(0, Math.min(1, (sm - rr) / scale)) // 月边抗锯齿
        peak = Math.max(peak, illum[0] * ext[1])
      }
    }
    const g = peak > 0 ? Math.min(MAX_GAIN, Math.max(1, 1 / peak)) : 1
    onGain(g)
    const sky = [11, 13, 20]
    const earth = [28, 30, 24]
    for (let q = 0; q < n * n; q++) {
      const p4 = q * 4
      const bg = ground[q] ? earth : sky
      const al = ground[q] ? 0 : lin[p4 + 3]
      for (let ch = 0; ch < 3; ch++) img.data[p4 + ch] = Math.round(enc(Math.min(1, lin[p4 + ch] * g)) * al + bg[ch] * (1 - al))
      img.data[p4 + 3] = 255
    }
    ctx.putImageData(img, 0, 0)
  }, [map, east, north, geom, local, tex, onGain])
  const up = local ? '天顶' : '北'
  return (
    <div style={{ background: '#0b0d14', borderRadius: 6, overflow: 'hidden', position: 'relative', width: css }}>
      <canvas ref={ref} style={{ width: css, height: css, display: 'block' }} />
      <span style={{ position: 'absolute', top: 3, left: 0, right: 0, textAlign: 'center', fontSize: 11, color: '#8a93a5' }}>{up}</span>
      {local && geom ? (
        <>
          <span style={{ position: 'absolute', top: 3, left: 6, fontSize: 11, color: '#8a93a5' }}>方位 {geom.az.toFixed(1)}°</span>
          <span style={{ position: 'absolute', bottom: 3, left: 6, fontSize: 11, color: '#8a93a5' }}>视高度 {geom.appAlt.toFixed(1)}°</span>
          {/* 天北方向：位置角自「上」逆时针增加，天北在 −天顶位置角处，即顺时针转天顶位置角 */}
          <span style={{ position: 'absolute', bottom: 3, right: 6, fontSize: 11, color: '#8a93a5' }}>
            北 <span style={{ display: 'inline-block', transform: `rotate(${geom.zenithPa}deg)` }}>↑</span>
          </span>
        </>
      ) : (
        <span style={{ position: 'absolute', top: '50%', left: 5, fontSize: 11, color: '#8a93a5', transform: 'translateY(-50%)' }}>东</span>
      )}
    </div>
  )
}

function ShadowDiagram({ map, frame, ut, geom, picked, onPick }: {
  map: LunarMap; frame: LunarFrame | null; ut: (jd: number) => number
  geom: MoonGeom | null; picked: [number, number] | null; onPick: (p: [number, number] | null) => void
}) {
  const size = 360
  const c = size / 2
  const extent = Math.max(map.penumbraRadius + map.moonSemidiameter * 2, ...map.curves.map((x) => Math.hypot(x.east, x.north) + map.moonSemidiameter)) * 1.04
  const k = (c - 6) / extent
  // 天空视角：北上东左
  const X = (east: number) => c - east * k
  const Y = (north: number) => c - north * k
  const [gain, setGain] = useState(1)
  const greatest = map.curves.find((x) => x.phase === 'G') ?? map.curves[Math.floor(map.curves.length / 2)]
  const d = map.shadowPath.map((p, i) => `${i ? 'L' : 'M'}${X(p[0]).toFixed(1)},${Y(p[1]).toFixed(1)}`).join('')
  return (
    <Flex gap={20} wrap="wrap" align="center">
    <svg viewBox={`0 0 ${size} ${size}`} style={{ width: '100%', maxWidth: size, display: 'block' }}>
      <circle cx={c} cy={c} r={map.penumbraRadius * k} fill="#d7dbe3" stroke="#8a93a5" />
      <circle cx={c} cy={c} r={map.umbraRadius * k} fill="#6f6a78" stroke="#444" />
      <path d={d} fill="none" stroke="#333" strokeDasharray="4 3" />
      <text x={c} y={13} fontSize={11} textAnchor="middle" fill="#555">北</text>
      <text x={8} y={c + 4} fontSize={11} fill="#555">东</text>
      {map.curves.map((x, i) => (
        <g key={x.phase}>
          <circle cx={X(x.east)} cy={Y(x.north)} r={map.moonSemidiameter * k} fill="rgba(255,250,225,0.45)" stroke={PHASE_COLOR[x.phase]} strokeWidth={1.2} />
          <text x={X(x.east)} y={Y(x.north) + 4} fontSize={11} fontWeight={600} textAnchor="middle" fill={PHASE_COLOR[x.phase]}
            style={{ paintOrder: 'stroke', stroke: '#fff', strokeWidth: 2.5 }}>{i + 1}</text>
        </g>
      ))}
      {frame && (
        <circle cx={X(frame.east)} cy={Y(frame.north)} r={map.moonSemidiameter * k} stroke="#222" strokeWidth={1.4}
          fill={frame.umbralMagnitude >= 1 ? '#a8492f' : frame.umbralMagnitude > 0 ? '#c9a66b' : '#fff6cf'} />
      )}
    </svg>
      <Flex vertical gap={4}>
        {map.curves.map((x, i) => (
          <Text key={x.phase} style={{ color: PHASE_COLOR[x.phase] }}>{i + 1}. {PHASE_LABEL[x.phase]} {fmtTime(ut(x.jdTt), true)} UT</Text>
        ))}
      </Flex>
      <Flex vertical gap={4} style={{ maxWidth: 260 }}>
        <MoonView map={map} east={(frame ?? greatest).east} north={(frame ?? greatest).north} geom={geom} local={!!picked} onGain={setGain} />
        <Text style={{ fontSize: 12 }}>
          月面实况（{frame ? `${fmtTime(ut(frame.jdTt), true)} UT` : '食甚'}，{picked ? '天顶朝上' : '北上东左'}{gain > 1.05 ? `，曝光 ×${gain.toFixed(gain < 10 ? 1 : 0)}` : ''}）
          {picked && geom && geom.appAlt < -map.moonSemidiameter ? '：月在地平下，看不到' : ''}
        </Text>
        <Space size={4} wrap>
          <Text style={{ fontSize: 12 }}>观测地</Text>
          <InputNumber size="small" style={{ width: 92 }} min={-180} max={180} step={0.1} placeholder="东经" value={picked?.[0]}
            onChange={(v) => v != null && onPick([v, picked?.[1] ?? 0])} suffix="E" />
          <InputNumber size="small" style={{ width: 88 }} min={-90} max={90} step={0.1} placeholder="北纬" value={picked?.[1]}
            onChange={(v) => v != null && onPick([picked?.[0] ?? 0, v])} suffix="N" />
          {picked && <a style={{ fontSize: 12 }} onClick={() => onPick(null)}>清除</a>}
        </Space>
        <Text type="secondary" style={{ fontSize: 12 }}>
          {picked ? '按该地站心方向画：天顶朝上，地平以下画地面，低空按大气质量消光、按折射压扁。' : '可在此填经纬度，或点地图、都城，看该地实际所见。'}
          月面取 NASA LRO 彩色月面图（NASA's Scientific Visualization Studio），按天平动与月轴方向贴上。半影处按日面被地球遮住的比例着色（均匀日面）；
          本影内的亮度与红色取决于地球大气，此处为示意。按月面最亮处自动曝光（同眼睛适应，至多 ×{MAX_GAIN}）。
        </Text>
      </Flex>
    </Flex>
  )
}

export function LunarEclipseView(props: {
  item: LunarListItem
  call: <T>(req: WithoutId<EclipseRequest>) => Promise<T>
  capitals: Capital[]
  onBusy: (s: string | null) => void
  onError: (s: string) => void
}) {
  const { item, call, capitals, onBusy, onError } = props
  const [map, setMap] = useState<LunarMap | null>(null)
  const [view, setView] = useState<MapView>('globe')
  const [layers, setLayers] = useState<string[]>(['regions', 'curves', 'capitals', 'sigma'])
  const [dtOffset, setDtOffset] = useState(0)
  const [picked, setPicked] = useState<[number, number] | null>(null)
  const [local, setLocal] = useState<Local | null | undefined>(undefined)

  useEffect(() => {
    let cancelled = false
    onBusy('计算月食图')
    call<LunarMap>({ type: 'lunarMap', jd: item.greatestTt, deltaT: item.deltaT })
      .then((m) => !cancelled && setMap(m))
      .catch((e) => !cancelled && onError(String(e instanceof Error ? e.message : e)))
      .finally(() => !cancelled && onBusy(null))
    return () => {
      cancelled = true
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [item, call])

  const playback = useEclipsePlayback<LunarFrame>(map ? [map.curves[0].jdTt, map.curves[map.curves.length - 1].jdTt] : null, (t) =>
    call<LunarFrame>({ type: 'lunarFrame', jd: item.greatestTt, t, deltaT: item.deltaT })
  )
  const frame = playback.frame
  const deltaT = item.deltaT + dtOffset
  const sigma = item.deltaTSigma

  useEffect(() => {
    if (!picked) return
    let cancelled = false
    setLocal(undefined)
    call<Local | null>({ type: 'lunarLocal', jd: item.greatestTt, deltaT, lon: picked[0], lat: picked[1] })
      .then((r) => !cancelled && setLocal(r))
      .catch(() => !cancelled && setLocal(null))
    return () => {
      cancelled = true
    }
  }, [picked, item, deltaT, call])

  // 月面实况的几何：有观测地按该地；否则取月下点（站心即地心方向，只用其天平动与月轴方向）
  const [moonGeom, setMoonGeom] = useState<MoonGeom | null>(null)
  const geomT = frame?.jdTt ?? map?.curves.find((x) => x.phase === 'G')?.jdTt ?? null
  const geomSub = frame?.sublunar[0] ?? map?.curves.find((x) => x.phase === 'G')?.sublunar[0] ?? null
  const geomLon = picked ? picked[0] : geomSub?.[0] ?? null
  const geomLat = picked ? picked[1] : geomSub?.[1] ?? null
  useEffect(() => {
    if (geomT == null || geomLon == null || geomLat == null) return
    let cancelled = false
    call<MoonGeom>({ type: 'moonView', jd: item.greatestTt, t: geomT, deltaT: picked ? deltaT : item.deltaT, lon: geomLon, lat: geomLat })
      .then((r) => !cancelled && setMoonGeom(r))
      .catch(() => !cancelled && setMoonGeom(null))
    return () => {
      cancelled = true
    }
  }, [geomT, geomLon, geomLat, picked, item, deltaT, call])

  const layerSet = useMemo(() => new Set(layers), [layers])
  const ut = (jdTt: number) => jdTt - deltaT / 86400
  const lmt = (jdTt: number) => ut(jdTt) + (picked ? picked[0] / 360 : 0)
  if (!map) return <div style={{ height: 120 }} />

  const dLon = dtOffset * map.lonShiftPerSecond
  const sigmaLon = sigma * Math.abs(map.lonShiftPerSecond)
  const g = map.curves.find((c) => c.phase === 'G')!
  const sub = g.sublunar[0]
  // 带食区域按本影食（初亏、复圆）；半影月食无本影，按半影食始终
  const first = map.curves.find((c) => c.phase === 'U1') ?? map.curves[0]
  const last = map.curves.find((c) => c.phase === 'U4') ?? map.curves[map.curves.length - 1]
  const pickedCapital = picked && capitals.find((c) => Math.abs(c.lon - picked[0]) < 1e-6 && Math.abs(c.lat - picked[1]) < 1e-6)
  const sigmaKeys = [first, last]

  return (
    <>
      <Card size="small" style={{ marginTop: 12 }}>
        <Flex vertical gap={10}>
          <Flex gap={16} align="center" wrap="wrap">
            <Radio.Group value={view} onChange={(e) => setView(e.target.value)} optionType="button" size="small"
              options={[{ label: '地球', value: 'globe' }, { label: '平面', value: 'flat' }]} />
            <Checkbox.Group value={layers} onChange={(v) => setLayers(v as string[])}
              options={[
                { label: '带食区域', value: 'regions' },
                { label: '各食相界线', value: 'curves' },
                { label: '都城', value: 'capitals' },
                { label: 'ΔT±σ', value: 'sigma' },
              ]} />
          </Flex>
          <Flex gap={12} align="center" wrap="wrap">
            <Text>ΔT = {deltaT.toFixed(0)} s</Text>
            <Text type="secondary">（MS2021 {item.deltaT.toFixed(0)} ± {sigma.toFixed(0)} s；偏移 {sigma > 0 ? (dtOffset / sigma).toFixed(2) : '0'}σ，全图东移 {dLon.toFixed(3)}°）</Text>
            <div style={{ flex: '1 1 240px', minWidth: 200 }}>
              <Slider min={-3 * Math.max(sigma, 1)} max={3 * Math.max(sigma, 1)} step={1} value={dtOffset} onChange={setDtOffset}
                marks={sigma >= 5 ? { [-2 * sigma]: '−2σ', [-sigma]: '−σ', 0: '0', [sigma]: '+σ', [2 * sigma]: '+2σ' } : undefined}
                tooltip={{ formatter: (v) => `${(v ?? 0) >= 0 ? '+' : ''}${v} s` }} />
            </div>
          </Flex>
          <EclipsePlaybackBar pb={playback}
            label={`${fmtTime(ut(playback.t ?? map.curves[0].jdTt), true)} UT${frame ? ` · ${frame.umbralMagnitude > 0 ? `本影食分 ${frame.umbralMagnitude.toFixed(3)}` : `半影食分 ${Math.max(0, frame.penumbralMagnitude).toFixed(3)}`}` : ''}`} />
          <GeoCanvas
            view={view}
            resetKey={map}
            globeCenter={[sub[0] + dLon, sub[1]]}
            flatLon={sub[0] + dLon}
            capitals={layerSet.has('capitals') ? capitals : []}
            picked={picked}
            onPick={(lon, lat) => setPicked([lon, lat])}
            overlay={({ path, pt }) => (
              <>
                {layerSet.has('regions') && (
                  <>
                    <path d={path(belowHorizon(first, dLon)) ?? ''} fill={RISE_FILL} stroke="none" />
                    <path d={path(belowHorizon(last, dLon)) ?? ''} fill={SET_FILL} stroke="none" />
                  </>
                )}
                {layerSet.has('sigma') && sigmaLon > 0 && sigmaKeys.flatMap((c) =>
                  [-sigmaLon, sigmaLon].map((s) => {
                    const l: MultiLineString = { type: 'MultiLineString', coordinates: [c.horizon.map((p) => [p[0] + dLon + s, p[1]])] }
                    return <path key={`${c.phase}${s}`} d={path(l) ?? ''} fill="none" stroke={PHASE_COLOR[c.phase]} strokeWidth={0.7} strokeDasharray="2 3" />
                  })
                )}
                {layerSet.has('curves') && map.curves.map((c) => {
                  const l: MultiLineString = { type: 'MultiLineString', coordinates: [c.horizon.map((p) => [p[0] + dLon, p[1]])] }
                  // 界线标注：取可见且最靠近赤道的点
                  const cand = c.horizon.filter((_, i) => i % 6 === 0).map((p) => ({ p, xy: pt(p[0] + dLon, p[1]) })).filter((x) => Number.isFinite(x.xy[0]))
                  const lab = cand.sort((a, b) => Math.abs(a.p[1]) - Math.abs(b.p[1]))[0]
                  return (
                    <g key={c.phase}>
                      <path d={path(l) ?? ''} fill="none" stroke={PHASE_COLOR[c.phase]} strokeWidth={c.phase === 'G' ? 1.6 : 1.1} />
                      {lab && <text x={lab.xy[0] + 3} y={lab.xy[1] - 3} fontSize={9.5} fill={PHASE_COLOR[c.phase]}>{PHASE_LABEL[c.phase]}</text>}
                    </g>
                  )
                })}
                {frame && (() => {
                  const [x, y] = pt(frame.sublunar[0][0] + dLon, frame.sublunar[0][1])
                  return (
                    <g>
                      <path d={path(belowHorizon(frame, dLon)) ?? ''} fill="rgba(20,25,50,0.28)" stroke="#27408b" strokeWidth={1.4} />
                      {Number.isFinite(x) && <circle cx={x} cy={y} r={4} fill="#fff8dc" stroke="#27408b" strokeWidth={1.4} />}
                    </g>
                  )
                })()}
                {(() => {
                  const [x, y] = pt(sub[0] + dLon, sub[1])
                  return Number.isFinite(x) ? (
                    <g>
                      <circle cx={x} cy={y} r={5} fill="#fff8dc" stroke="#c0392b" strokeWidth={1.4} />
                      <text x={x + 7} y={y + 4} fontSize={10} fill="#c0392b">食甚月下点</text>
                    </g>
                  ) : null
                })()}
              </>
            )}
          />
          <Flex gap={14} wrap="wrap">
            <Space size={4}><span style={{ display: 'inline-block', width: 18, height: 10, background: RISE_FILL }} /><Text style={{ fontSize: 12 }}>食始时月在地平下（月出带食）</Text></Space>
            <Space size={4}><span style={{ display: 'inline-block', width: 18, height: 10, background: SET_FILL }} /><Text style={{ fontSize: 12 }}>食终时月在地平下（月没带食）</Text></Space>
            <Text style={{ fontSize: 12 }} type="secondary">两色重叠处全程不可见；未着色处全程可见</Text>
          </Flex>
          <Text type="secondary" style={{ fontSize: 12 }}>
            区域按{first.phase === 'U1' ? '初亏、复圆' : '半影食始、半影食终'}。界线为月亮站心几何高度 0°（含视差，不计折射）。拖动旋转（平面图为平移），滚轮缩放，点击地图或都城看该地食况。
            食甚 {fmtDate(ut(map.eclipse.greatestTt))} {fmtTime(ut(map.eclipse.greatestTt), true)} UT，γ = {map.eclipse.gamma.toFixed(4)}，{map.eclipse.umbralMagnitude > 0 ? `本影食分 ${map.eclipse.umbralMagnitude.toFixed(4)}，` : ''}半影食分 {map.eclipse.penumbralMagnitude.toFixed(4)}。
          </Text>
        </Flex>
      </Card>
      <Card size="small" style={{ marginTop: 12 }} title="月亮穿过地影（北上东左；地影按食甚时大小）">
        <ShadowDiagram map={map} frame={frame} ut={ut} geom={moonGeom} picked={picked} onPick={setPicked} />
      </Card>
      {picked && (
        <Card size="small" style={{ marginTop: 12 }}
          title={`${pickedCapital ? `${pickedCapital.name}（${pickedCapital.regimes.join('、')}）` : '所选地点'} ${Math.abs(picked[0]).toFixed(2)}°${picked[0] >= 0 ? 'E' : 'W'} ${Math.abs(picked[1]).toFixed(2)}°${picked[1] >= 0 ? 'N' : 'S'}，ΔT ${deltaT.toFixed(0)} s`}>
          {local === undefined ? <Spin size="small" /> : local === null ? <Text>计算失败。</Text> : (
            <Descriptions size="small" column={{ xs: 1, sm: 2, md: 3 }} bordered>
              {local.phases.map((p) => (
                <Descriptions.Item key={p.phase} label={PHASE_LABEL[p.phase]}>
                  {fmtTime(ut(p.jdTt), true)} UT · 地方平时 {fmtTime(lmt(p.jdTt))} · 月高 {p.alt.toFixed(1)}°
                </Descriptions.Item>
              ))}
              <Descriptions.Item label="可见">
                {local.visibleFrom == null
                  ? '全程在地平下'
                  : `${fmtTime(lmt(local.visibleFrom))}—${fmtTime(lmt(local.visibleTo!))} 地方平时，可见最大${(local.maxVisibleUmbral ?? 0) > 0 ? `本影食分 ${local.maxVisibleUmbral?.toFixed(3)}、` : ''}半影食分 ${local.maxVisiblePenumbral?.toFixed(3)}`}
              </Descriptions.Item>
            </Descriptions>
          )}
        </Card>
      )}
    </>
  )
}
