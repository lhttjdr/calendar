/**
 * 星图：某地某时的天空，两种画法。
 * - 盖图：以北天极为中心的等距方位投影（半径正比于去极度），画内规、赤道、外规、黄道、二十八宿宿界与宿度，并按当地当时
 *   画地平线（其下加暗）。方向按「面北仰视」：本地恒星时所在子午圈朝上，赤经向东（顺时针）增大，可改为固定方位。
 * - 地平图：以天顶为中心的球极投影，北上东左（仰视）。
 *
 * 天体位置（当日真赤道与真春分点的视位置）、星等、宿度、恒星时都由 Rust 算（skyWorker → SkyEngine）；这里只做投影、
 * 赤道→地平的坐标旋转、入宿度这类由所给位置直接相减的量，以及绘图。星官连线与星名取 Stellarium 中国星空文化
 * （CC BY-SA 4.0）。
 */
import { useEffect, useMemo, useRef, useState } from 'react'
import { Alert, Button, Card, Checkbox, Descriptions, Flex, InputNumber, Radio, Select, Slider, Space, Spin, Typography } from 'antd'
import { civilToJd, fmtDate, fmtTime, jdToCivil, yearLabel } from './civilTime'
import { useWorkerRpc } from './useWorkerRpc'
import { loadCapitals, type Capital } from './capitals'
import type { SkyRequest } from './skyWorker'

const { Text } = Typography

// ---------------------------------------------------------------------------------------------------------------------
// 数据类型（与 rust/wasm-lib/src/lib_impl.rs 星图部分的 JSON 对应）

type CultureAsterism = { system: string; id: string; name: string; det: number | null; lines: number[][] }
type Culture = {
  asterisms: CultureAsterism[]
  names: [string, number, string][]
  mansions: { system: string; determinatives: number[] }[]
  /** 陈卓体系星官的三家分属（据《开元占经》卷六十至七十） */
  schools: { id: string; school: string | null; basis: string; source: string }[]
}
type BodyJson = { name: string; ra: number; dec: number; mag: number | null; phase: number | null }
type SnapshotJson = {
  jdTt: number
  gastDeg: number
  eclipticPole: [number, number]
  stars: (number | null)[]
  bodies: BodyJson[]
  mansionWidths: Record<string, number[]>
}
type Star = { hip: number; ra: number; dec: number; v: number; bv: number | null }

/** 星官体系：距星表 id → 所用星官连线体系（《灵台仪象志》只改距星，星官仍用《仪象考成》）。 */
const SYSTEMS: { id: string; label: string; asterisms: string }[] = [
  { id: 'chenzhuo', label: '陈卓（三国—晋，汇汉三家）', asterisms: 'chenzhuo' },
  { id: 'song_huangyou', label: '宋皇祐（1052）', asterisms: 'song_huangyou' },
  { id: 'yuan', label: '元《天文汇钞》', asterisms: 'yuan' },
  { id: 'ming_xianglin', label: '明《象林》（1634）', asterisms: 'ming_xianglin' },
  { id: 'qing_lingtai', label: '清《灵台仪象志》（1674）距星', asterisms: 'qing' },
  { id: 'qing', label: '清《仪象考成》（1744）', asterisms: 'qing' },
]

const MANSIONS = '角亢氐房心尾箕斗牛女虚危室壁奎娄胃昴毕觜参井鬼柳星张翼轸'.split('')

const BODY_LABEL: Record<string, string> = {
  sun: '日',
  moon: '月',
  mercury: '辰星',
  venus: '太白',
  mars: '荧惑',
  jupiter: '岁星',
  saturn: '填星',
}
const BODY_COLOR: Record<string, string> = {
  sun: '#ffd34d',
  moon: '#e8e8e0',
  mercury: '#c9b8a0',
  venus: '#fff4c8',
  mars: '#ff7a4d',
  jupiter: '#f2d6a8',
  saturn: '#e6cf8f',
}

/** 三家传统配色：石氏赤、甘氏黑、巫咸黄（敦煌星图等）；深色底上甘氏改用白。 */
const SCHOOL_COLOR: Record<string, string> = { 石氏: '#ff6b5a', 甘氏: '#e6e6e6', 巫咸: '#ffd23f' }

const COLORS = {
  sky: '#0b1530',
  skyEdge: '#33456e',
  below: 'rgba(0,0,0,0.45)',
  asterism: 'rgba(120,160,220,0.75)',
  asterismName: '#9fb7e0',
  mansion: 'rgba(214,170,80,0.55)',
  mansionText: '#e0bd6a',
  equator: 'rgba(110,190,140,0.7)',
  ecliptic: 'rgba(240,180,70,0.8)',
  circle: 'rgba(170,180,210,0.45)',
  horizon: 'rgba(120,220,200,0.9)',
  text: '#cdd6e8',
  pick: '#ff6b6b',
}

const d2r = Math.PI / 180
const r2d = 180 / Math.PI
const mod360 = (x: number) => ((x % 360) + 360) % 360

/** B−V → 星色（Ballesteros 近似的分段线性版）。 */
function starColor(bv: number | null): string {
  if (bv == null) return '#f4f4ff'
  const stops: [number, [number, number, number]][] = [
    [-0.4, [155, 176, 255]],
    [0.0, [202, 215, 255]],
    [0.4, [248, 247, 255]],
    [0.8, [255, 236, 200]],
    [1.2, [255, 210, 161]],
    [1.8, [255, 180, 107]],
  ]
  const x = Math.max(-0.4, Math.min(1.8, bv))
  for (let i = 0; i < stops.length - 1; i++) {
    const [a, ca] = stops[i]
    const [b, cb] = stops[i + 1]
    if (x <= b) {
      const t = (x - a) / (b - a)
      const c = ca.map((v, k) => Math.round(v + (cb[k] - v) * t))
      return `rgb(${c[0]},${c[1]},${c[2]})`
    }
  }
  return 'rgb(255,180,107)'
}

const starRadius = (v: number) => Math.max(0.55, 3.3 - 0.45 * v)

/** 赤道 → 地平（几何，不计折射）：方位自北经东量。 */
function toHorizon(raDeg: number, decDeg: number, lstDeg: number, latDeg: number): { alt: number; az: number } {
  const h = (lstDeg - raDeg) * d2r
  const d = decDeg * d2r
  const p = latDeg * d2r
  const alt = Math.asin(Math.sin(p) * Math.sin(d) + Math.cos(p) * Math.cos(d) * Math.cos(h))
  const az = Math.atan2(-Math.cos(d) * Math.sin(h), Math.sin(d) * Math.cos(p) - Math.cos(d) * Math.cos(h) * Math.sin(p))
  return { alt: alt * r2d, az: mod360(az * r2d) }
}

/** 地平 → 赤道。 */
function fromHorizon(altDeg: number, azDeg: number, lstDeg: number, latDeg: number): { ra: number; dec: number } {
  const a = altDeg * d2r
  const z = azDeg * d2r
  const p = latDeg * d2r
  const dec = Math.asin(Math.sin(p) * Math.sin(a) + Math.cos(p) * Math.cos(a) * Math.cos(z))
  const h = Math.atan2(-Math.sin(z) * Math.cos(a), Math.cos(p) * Math.sin(a) - Math.sin(p) * Math.cos(a) * Math.cos(z))
  return { ra: mod360(lstDeg - h * r2d), dec: dec * r2d }
}

/** 以 (raP, decP) 为极的大圆上 n 个点（赤经、赤纬，度）。 */
function greatCircle(raP: number, decP: number, n = 360): { ra: number; dec: number }[] {
  const p = [Math.cos(decP * d2r) * Math.cos(raP * d2r), Math.cos(decP * d2r) * Math.sin(raP * d2r), Math.sin(decP * d2r)]
  // 与极正交的两单位向量
  const ref = Math.abs(p[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0]
  const a = [p[1] * ref[2] - p[2] * ref[1], p[2] * ref[0] - p[0] * ref[2], p[0] * ref[1] - p[1] * ref[0]]
  const na = Math.hypot(a[0], a[1], a[2])
  const A = a.map((x) => x / na)
  const B = [p[1] * A[2] - p[2] * A[1], p[2] * A[0] - p[0] * A[2], p[0] * A[1] - p[1] * A[0]]
  return Array.from({ length: n + 1 }, (_, k) => {
    const t = (2 * Math.PI * k) / n
    const v = [0, 1, 2].map((i) => Math.cos(t) * A[i] + Math.sin(t) * B[i])
    return { ra: mod360(Math.atan2(v[1], v[0]) * r2d), dec: Math.asin(Math.max(-1, Math.min(1, v[2]))) * r2d }
  })
}

type View = 'gai' | 'horizon'
type Projector = (ra: number, dec: number) => [number, number] | null

// ---------------------------------------------------------------------------------------------------------------------

type Picked = { kind: 'star'; hip: number } | { kind: 'body'; name: string }

function SkyCanvas(props: {
  view: View
  stars: Star[]
  starMap: Map<number, Star>
  snap: SnapshotJson
  culture: Culture
  systemId: string
  lstDeg: number
  lat: number
  magLimit: number
  layers: Set<string>
  fixedOrientation: boolean
  zhouTian: number
  picked: Picked | null
  onPick: (p: Picked | null) => void
}) {
  const { view, stars, starMap, snap, culture, systemId, lstDeg, lat, magLimit, layers, fixedOrientation, zhouTian, picked, onPick } = props
  const size = 760
  const cx = size / 2
  const cy = size / 2
  const R = size / 2 - 34
  const system = SYSTEMS.find((s) => s.id === systemId)!
  // 盖图外缘：外规（恒隐圈，赤纬 φ − 90°）再放宽 5°
  const rimDec = Math.max(-89, lat - 90 - 5)
  const topRa = fixedOrientation ? 0 : lstDeg

  const project: Projector = useMemo(() => {
    if (view === 'gai') {
      return (ra, dec) => {
        if (dec < rimDec) return null
        const rr = ((90 - dec) / (90 - rimDec)) * R
        const t = (ra - topRa) * d2r
        return [cx + rr * Math.sin(t), cy - rr * Math.cos(t)]
      }
    }
    return (ra, dec) => {
      const { alt, az } = toHorizon(ra, dec, lstDeg, lat)
      if (alt < 0) return null
      const r = R * Math.tan(((90 - alt) / 2) * d2r)
      return [cx - r * Math.sin(az * d2r), cy - r * Math.cos(az * d2r)]
    }
  }, [view, rimDec, topRa, lstDeg, lat, R, cx, cy])

  const polyline = (pts: { ra: number; dec: number }[]) => {
    let d = ''
    let pen = false
    for (const p of pts) {
      const xy = project(p.ra, p.dec)
      if (!xy) {
        pen = false
        continue
      }
      d += `${pen ? 'L' : 'M'}${xy[0].toFixed(1)},${xy[1].toFixed(1)}`
      pen = true
    }
    return d
  }
  const decCircle = (dec: number) => polyline(Array.from({ length: 361 }, (_, k) => ({ ra: k, dec })))

  // 当前体系的距星赤经（宿界）
  const mansionSystem = culture.mansions.find((m) => m.system === systemId)
  const detStars = mansionSystem?.determinatives.map((h) => starMap.get(h)) ?? []
  const widths = snap.mansionWidths[systemId]

  const asterisms = useMemo(() => culture.asterisms.filter((a) => a.system === system.asterisms), [culture, system.asterisms])
  const schoolOf = useMemo(() => new Map(culture.schools.map((x) => [x.id, x])), [culture])
  // 三家分属只对陈卓体系（汇三家而成）有意义
  const schoolColoring = layers.has('schools') && system.asterisms === 'chenzhuo'
  const asterismHips = useMemo(() => new Set(asterisms.flatMap((a) => a.lines.flat())), [asterisms])

  // 画哪些星：亮于极限星等者，以及当前体系星官的成员（连线端点）
  const shown = useMemo(() => {
    const out: { s: Star; xy: [number, number] }[] = []
    for (const s of stars) {
      if (s.v > magLimit && !(layers.has('asterisms') && asterismHips.has(s.hip))) continue
      const xy = project(s.ra, s.dec)
      if (xy) out.push({ s, xy })
    }
    return out
  }, [stars, magLimit, layers, asterismHips, project])

  const bodies = snap.bodies.map((b) => ({ b, xy: project(b.ra, b.dec) }))

  const svgRef = useRef<SVGSVGElement | null>(null)
  const onClick = (e: React.MouseEvent) => {
    const rect = svgRef.current!.getBoundingClientRect()
    const x = ((e.clientX - rect.left) / rect.width) * size
    const y = ((e.clientY - rect.top) / rect.height) * size
    let best: Picked | null = null
    let bestD = 12
    for (const { b, xy } of bodies) {
      if (!xy) continue
      const d = Math.hypot(xy[0] - x, xy[1] - y)
      if (d < bestD) {
        bestD = d
        best = { kind: 'body', name: b.name }
      }
    }
    for (const { s, xy } of shown) {
      const d = Math.hypot(xy[0] - x, xy[1] - y) - starRadius(s.v) * 0.5
      if (d < bestD) {
        bestD = d
        best = { kind: 'star', hip: s.hip }
      }
    }
    onPick(best)
  }

  const pickedXY = (() => {
    if (!picked) return null
    if (picked.kind === 'star') {
      const s = starMap.get(picked.hip)
      return s ? project(s.ra, s.dec) : null
    }
    const b = snap.bodies.find((x) => x.name === picked.name)
    return b ? project(b.ra, b.dec) : null
  })()

  // 地平线（盖图中）与其下的暗区：外缘圆与地平线闭曲线以 evenodd 填充
  const horizonPts = Array.from({ length: 361 }, (_, k) => fromHorizon(0, k, lstDeg, lat))
  const rimPath = `M${cx - R},${cy}a${R},${R} 0 1,0 ${2 * R},0a${R},${R} 0 1,0 ${-2 * R},0`

  const labelFor = (deg: number, r: number): [number, number] => {
    const t = (deg - topRa) * d2r
    return [cx + r * Math.sin(t), cy - r * Math.cos(t)]
  }

  return (
    <svg ref={svgRef} viewBox={`0 0 ${size} ${size}`} style={{ width: '100%', maxWidth: size, display: 'block', cursor: 'crosshair' }} onClick={onClick}>
      <defs>
        <clipPath id="sky-clip">
          <circle cx={cx} cy={cy} r={R} />
        </clipPath>
      </defs>
      <rect x={0} y={0} width={size} height={size} fill="transparent" />
      <circle cx={cx} cy={cy} r={R} fill={COLORS.sky} stroke={COLORS.skyEdge} strokeWidth={1.5} />
      <g clipPath="url(#sky-clip)">
        {view === 'gai' && layers.has('circles') && (
          <>
            <path d={decCircle(90 - lat)} fill="none" stroke={COLORS.circle} strokeWidth={0.8} strokeDasharray="4 3" />
            <path d={decCircle(lat - 90)} fill="none" stroke={COLORS.circle} strokeWidth={0.8} strokeDasharray="4 3" />
          </>
        )}
        {view === 'horizon' && layers.has('circles') &&
          [30, 60].map((alt) => {
            const r = R * Math.tan(((90 - alt) / 2) * d2r)
            return <circle key={alt} cx={cx} cy={cy} r={r} fill="none" stroke={COLORS.circle} strokeWidth={0.5} strokeDasharray="2 4" />
          })}
        {layers.has('equator') && <path d={decCircle(0)} fill="none" stroke={COLORS.equator} strokeWidth={1} />}
        {layers.has('ecliptic') && (
          <path d={polyline(greatCircle(snap.eclipticPole[0], snap.eclipticPole[1]))} fill="none" stroke={COLORS.ecliptic} strokeWidth={1} />
        )}
        {layers.has('mansions') && detStars.length === 28 && (
          <g>
            {detStars.map((s, i) => {
              if (!s) return null
              const inner = view === 'gai' ? 90 - lat : 89.5
              const pts = Array.from({ length: 60 }, (_, k) => ({ ra: s.ra, dec: inner - ((inner - (view === 'gai' ? rimDec : -89)) * k) / 59 }))
              return <path key={i} d={polyline(pts)} fill="none" stroke={COLORS.mansion} strokeWidth={0.8} />
            })}
          </g>
        )}
        {layers.has('asterisms') &&
          asterisms.map((a) =>
            a.lines.map((line, k) => {
              const pts = line.map((h) => starMap.get(h)).filter((s): s is Star => !!s).map((s) => project(s.ra, s.dec))
              let d = ''
              let pen = false
              for (const xy of pts) {
                if (!xy) {
                  pen = false
                  continue
                }
                d += `${pen ? 'L' : 'M'}${xy[0].toFixed(1)},${xy[1].toFixed(1)}`
                pen = true
              }
              const school = schoolColoring ? schoolOf.get(a.id)?.school : null
              return (
                <path key={`${a.id}-${k}`} d={d} fill="none" stroke={school ? SCHOOL_COLOR[school] : COLORS.asterism} strokeWidth={school ? 0.9 : 0.7} />
              )
            })
          )}
        {shown.map(({ s, xy }) => (
          <circle key={s.hip} cx={xy[0]} cy={xy[1]} r={starRadius(s.v)} fill={starColor(s.bv)} />
        ))}
        {layers.has('mansions') &&
          detStars.map((s, i) => {
            const xy = s && project(s.ra, s.dec)
            return xy ? <circle key={`d${i}`} cx={xy[0]} cy={xy[1]} r={starRadius(s.v) + 2.5} fill="none" stroke={COLORS.mansionText} strokeWidth={0.8} /> : null
          })}
        {layers.has('names') &&
          asterisms.map((a) => {
            // 只标最亮成员不暗于 max(3.5, 极限星等 − 2) 的星官，免得满图是字
            const members = a.lines.flat().map((h) => starMap.get(h)).filter((s): s is Star => !!s)
            if (!members.length || Math.min(...members.map((s) => s.v)) > Math.max(3.5, magLimit - 2)) return null
            const first = a.lines[0]?.map((h) => starMap.get(h)).find((s): s is Star => !!s)
            const xy = first && project(first.ra, first.dec)
            return xy ? (
              <text key={`n${a.id}`} x={xy[0] + 4} y={xy[1] - 3} fontSize={9.5} fill={COLORS.asterismName}>
                {a.name}
              </text>
            ) : null
          })}
        {view === 'gai' && layers.has('horizon') && (
          <>
            <path d={`${rimPath}${polyline(horizonPts)}Z`} fill={COLORS.below} fillRule="evenodd" stroke="none" />
            <path d={polyline(horizonPts)} fill="none" stroke={COLORS.horizon} strokeWidth={1.2} />
          </>
        )}
        {layers.has('bodies') &&
          bodies.map(({ b, xy }) =>
            xy ? (
              <g key={b.name}>
                <circle cx={xy[0]} cy={xy[1]} r={b.name === 'sun' ? 7 : b.name === 'moon' ? 6 : 3.6} fill={BODY_COLOR[b.name]} stroke="#000" strokeWidth={0.6} />
                <text x={xy[0] + 8} y={xy[1] + 4} fontSize={12} fill={BODY_COLOR[b.name]} style={{ fontWeight: 600 }}>
                  {BODY_LABEL[b.name]}
                </text>
              </g>
            ) : null
          )}
        {pickedXY && <circle cx={pickedXY[0]} cy={pickedXY[1]} r={9} fill="none" stroke={COLORS.pick} strokeWidth={1.5} />}
      </g>
      {/* 外圈注记 */}
      {view === 'gai' && layers.has('mansions') && widths && detStars.length === 28 &&
        detStars.map((s, i) => {
          if (!s) return null
          const mid = s.ra + widths[i] / 2
          const [x, y] = labelFor(mid, R + 13)
          const [x2, y2] = labelFor(mid, R + 26)
          return (
            <g key={`m${i}`}>
              <text x={x} y={y + 4} fontSize={12} fill={COLORS.mansionText} textAnchor="middle">{MANSIONS[i]}</text>
              <text x={x2} y={y2 + 3} fontSize={8.5} fill={COLORS.mansionText} textAnchor="middle" opacity={0.85}>
                {((widths[i] * zhouTian) / 360).toFixed(2)}
              </text>
            </g>
          )
        })}
      {view === 'horizon' &&
        [
          ['北', 0],
          ['东', 90],
          ['南', 180],
          ['西', 270],
        ].map(([t, az]) => {
          const a = (az as number) * d2r
          return (
            <text key={t as string} x={cx - (R + 16) * Math.sin(a)} y={cy - (R + 16) * Math.cos(a) + 5} fontSize={14} fill="#556" textAnchor="middle">
              {t}
            </text>
          )
        })}
    </svg>
  )
}

function Legend({ schools }: { schools: boolean }) {
  const item = (color: string, label: string, dash?: string) => (
    <Space size={4} key={label}>
      <svg width={26} height={8}>
        <line x1={0} x2={26} y1={4} y2={4} stroke={color} strokeWidth={2} strokeDasharray={dash} />
      </svg>
      <Text style={{ fontSize: 12 }}>{label}</Text>
    </Space>
  )
  return (
    <Flex gap={14} wrap="wrap">
      {schools ? (
        <>
          {item(SCHOOL_COLOR['石氏'], '石氏星官')}
          {item(SCHOOL_COLOR['甘氏'], '甘氏星官（传统作黑）')}
          {item(SCHOOL_COLOR['巫咸'], '巫咸星官')}
          {item(COLORS.asterism, '三家名录无者')}
        </>
      ) : (
        item(COLORS.asterism, '星官连线')
      )}
      {item(COLORS.mansion, '二十八宿宿界（经距星）')}
      {item(COLORS.equator, '赤道')}
      {item(COLORS.ecliptic, '黄道')}
      {item(COLORS.circle, '内规、外规', '4 3')}
      {item(COLORS.horizon, '地平')}
    </Flex>
  )
}

// ---------------------------------------------------------------------------------------------------------------------

export function StarChartPage() {
  const call = useWorkerRpc<SkyRequest>(() => new Worker(new URL('./skyWorker.ts', import.meta.url), { type: 'module' }))
  const now = jdToCivil(Date.now() / 86400000 + 2440587.5)
  const [date, setDate] = useState({ y: now.y, m: now.m, d: now.d, h: 20, mi: 0 })
  const [capitals, setCapitals] = useState<Capital[]>([])
  const [place, setPlace] = useState<{ name: string; lon: number; lat: number }>({ name: '长安', lon: 108.94, lat: 34.27 })
  /** 'auto' = 按图上日期取适用体系（Rust：sky_culture::periods），否则为手选的体系 id */
  const [systemChoice, setSystemChoice] = useState<string>('auto')
  const [period, setPeriod] = useState<{ system: string; epoch: string; basis: string } | null>(null)
  const [view, setView] = useState<View>('gai')
  const [magLimit, setMagLimit] = useState(5.5)
  const [zhouTian, setZhouTian] = useState(365.25)
  const [fixedOrientation, setFixedOrientation] = useState(false)
  const [layers, setLayers] = useState<string[]>(['asterisms', 'schools', 'mansions', 'equator', 'ecliptic', 'circles', 'horizon', 'bodies'])
  const [culture, setCulture] = useState<Culture | null>(null)
  const [snap, setSnap] = useState<SnapshotJson | null>(null)
  const [deltaT, setDeltaT] = useState<[number, number] | null>(null)
  const [busy, setBusy] = useState<string | null>('加载星表与历表')
  const [error, setError] = useState<string | null>(null)
  const [picked, setPicked] = useState<Picked | null>(null)

  useEffect(() => {
    loadCapitals().then(setCapitals).catch(() => setCapitals([]))
    call<Culture>({ type: 'culture' })
      .then(setCulture)
      .catch((e) => setError(String(e instanceof Error ? e.message : e)))
  }, [call])

  // 地方平时 → UT：UT = 地方平时 − 经度/15 时
  const jdUt = civilToJd(date.y, date.m, date.d, date.h + date.mi / 60 - place.lon / 15)

  useEffect(() => {
    let cancelled = false
    const t = setTimeout(async () => {
      try {
        setBusy('计算天象')
        // ΔT 以 TT 为自变量：迭代两次
        let dt = (await call<number[]>({ type: 'deltaT', jd: jdUt }))[0]
        const est = await call<number[]>({ type: 'deltaT', jd: jdUt + dt / 86400 })
        dt = est[0]
        const s = await call<SnapshotJson>({ type: 'snapshot', jdTt: jdUt + dt / 86400, deltaT: dt })
        if (cancelled) return
        setDeltaT([est[0], est[1]])
        setSnap(s)
        setError(null)
      } catch (e) {
        if (!cancelled) setError(String(e instanceof Error ? e.message : e))
      } finally {
        if (!cancelled) setBusy(null)
      }
    }, 150)
    return () => {
      cancelled = true
      clearTimeout(t)
    }
  }, [jdUt, call])

  const stars = useMemo(() => {
    if (!snap) return []
    const out: Star[] = []
    const a = snap.stars
    for (let i = 0; i + 4 < a.length; i += 5) {
      out.push({ hip: a[i] as number, ra: a[i + 1] as number, dec: a[i + 2] as number, v: a[i + 3] as number, bv: a[i + 4] })
    }
    return out
  }, [snap])
  const starMap = useMemo(() => new Map(stars.map((s) => [s.hip, s])), [stars])
  const names = useMemo(() => {
    const m = new Map<string, string>()
    culture?.names.forEach(([sys, hip, name]) => m.set(`${sys}:${hip}`, name))
    return m
  }, [culture])

  useEffect(() => {
    let cancelled = false
    call<{ system: string; epoch: string; basis: string } | null>({ type: 'systemForYear', year: date.y })
      .then((p) => !cancelled && setPeriod(p))
      .catch(() => !cancelled && setPeriod(null))
    return () => {
      cancelled = true
    }
  }, [date.y, call])
  const systemId = systemChoice === 'auto' ? period?.system ?? 'chenzhuo' : systemChoice

  const lstDeg = snap ? mod360(snap.gastDeg + place.lon) : 0
  const layerSet = useMemo(() => new Set(layers), [layers])
  const system = SYSTEMS.find((s) => s.id === systemId)!

  const shift = (hours: number) => {
    const c = jdToCivil(civilToJd(date.y, date.m, date.d, date.h + date.mi / 60) + hours / 24)
    setDate({ y: c.y, m: c.m, d: c.d, h: c.h, mi: c.mi })
  }

  // 所选天体信息
  const info = (() => {
    if (!picked || !snap || !culture) return null
    const target =
      picked.kind === 'star'
        ? starMap.get(picked.hip)
        : snap.bodies.find((b) => b.name === picked.name)
    if (!target) return null
    const mansionSys = culture.mansions.find((m) => m.system === systemId)
    // 入宿度：西侧最近距星以东的赤经差（同一历元视位置直接相减，与 Rust MansionFrame::ru_xiu 同义）
    let ruXiu: { i: number; d: number } | null = null
    if (mansionSys) {
      for (let i = 0; i < 28; i++) {
        const det = starMap.get(mansionSys.determinatives[i])
        if (!det) continue
        const dd = mod360(target.ra - det.ra)
        if (!ruXiu || dd < ruXiu.d) ruXiu = { i, d: dd }
      }
    }
    const memberOf = (hip: number) => culture.asterisms.filter((a) => a.system === system.asterisms && a.lines.some((l) => l.includes(hip)))
    const hz = toHorizon(target.ra, target.dec, lstDeg, place.lat)
    const toGu = (deg: number) => ((deg * zhouTian) / 360).toFixed(2)
    const title =
      picked.kind === 'star'
        ? (() => {
            const own = names.get(`${system.asterisms}:${picked.hip}`)
            const member = memberOf(picked.hip).map((a) => a.name)
            return `${own ?? (member.length ? `${member.join('、')}（星官成员）` : '（本体系无名）')} · HIP ${picked.hip}`
          })()
        : `${BODY_LABEL[picked.name]}`
    const mag = 'v' in target ? target.v : target.mag
    const schoolRows =
      picked.kind === 'star' && system.asterisms === 'chenzhuo'
        ? memberOf(picked.hip).map((a) => {
            const x = culture.schools.find((s) => s.id === a.id)
            return `${a.name}：${x?.school ?? '三家名录无'}${x?.school ? `（${x.basis}；《开元占经》${x.source}）` : ''}`
          })
        : []
    return (
      <Card size="small" title={title}>
        <Descriptions size="small" column={{ xs: 1, sm: 2, md: 3 }} bordered>
          <Descriptions.Item label="视星等">{mag != null ? mag.toFixed(2) : '—'}{'bv' in target && target.bv != null ? `（B−V ${target.bv.toFixed(2)}）` : ''}</Descriptions.Item>
          <Descriptions.Item label="赤经 / 赤纬">{target.ra.toFixed(3)}° / {target.dec.toFixed(3)}°</Descriptions.Item>
          <Descriptions.Item label="入宿度">{ruXiu ? `入${MANSIONS[ruXiu.i]}宿 ${toGu(ruXiu.d)} 度` : '—'}</Descriptions.Item>
          <Descriptions.Item label="去极度">{toGu(90 - target.dec)} 度</Descriptions.Item>
          <Descriptions.Item label="地平高度 / 方位">{hz.alt.toFixed(2)}° / {hz.az.toFixed(2)}°（自北经东）</Descriptions.Item>
          {'phase' in target && target.phase != null && <Descriptions.Item label="相角">{target.phase.toFixed(1)}°</Descriptions.Item>}
          {schoolRows.length > 0 && <Descriptions.Item label="三家" span={3}>{schoolRows.join('；')}</Descriptions.Item>}
        </Descriptions>
      </Card>
    )
  })()

  return (
    <Flex vertical gap={12}>
      <Card size="small">
        <Flex vertical gap={10}>
          <Flex gap={8} align="center" wrap="wrap">
            <Text>日期</Text>
            <InputNumber value={date.y} min={-2000} max={3000} style={{ width: 90 }} onChange={(v) => v != null && setDate({ ...date, y: v })} />
            <Text type="secondary">年</Text>
            <InputNumber value={date.m} min={1} max={12} style={{ width: 60 }} onChange={(v) => v != null && setDate({ ...date, m: v })} />
            <Text type="secondary">月</Text>
            <InputNumber value={date.d} min={1} max={31} style={{ width: 60 }} onChange={(v) => v != null && setDate({ ...date, d: v })} />
            <Text type="secondary">日</Text>
            <InputNumber value={date.h} min={0} max={23} style={{ width: 60 }} onChange={(v) => v != null && setDate({ ...date, h: v })} />
            <Text type="secondary">:</Text>
            <InputNumber value={date.mi} min={0} max={59} style={{ width: 60 }} onChange={(v) => v != null && setDate({ ...date, mi: v })} />
            <Text type="secondary">地方平时（{yearLabel(date.y)} 年；天文纪年，1582-10-15 前儒略历）</Text>
          </Flex>
          <Flex gap={8} align="center" wrap="wrap">
            <Button size="small" onClick={() => shift(-24)}>−1 日</Button>
            <Button size="small" onClick={() => shift(-1)}>−1 时</Button>
            <Button size="small" onClick={() => shift(1)}>+1 时</Button>
            <Button size="small" onClick={() => shift(24)}>+1 日</Button>
            <Text style={{ marginLeft: 12 }}>地点</Text>
            <Select
              style={{ width: 150 }}
              value={capitals.some((c) => c.name === place.name) ? place.name : undefined}
              placeholder="自定"
              options={capitals.map((c) => ({ label: c.name, value: c.name }))}
              onChange={(n) => {
                const c = capitals.find((x) => x.name === n)
                if (c) setPlace({ name: c.name, lon: c.lon, lat: c.lat })
              }}
            />
            <InputNumber value={place.lon} step={0.01} style={{ width: 100 }} suffix="E" onChange={(v) => v != null && setPlace({ name: '', lon: v, lat: place.lat })} />
            <InputNumber value={place.lat} step={0.01} style={{ width: 100 }} suffix="N" onChange={(v) => v != null && setPlace({ name: '', lon: place.lon, lat: v })} />
          </Flex>
          <Flex gap={8} align="center" wrap="wrap">
            <Text>星官体系</Text>
            <Select
              style={{ width: 260 }}
              value={systemChoice}
              onChange={setSystemChoice}
              options={[{ label: '按时期（自动）', value: 'auto' }, ...SYSTEMS.map((s) => ({ label: s.label, value: s.id }))]}
            />
            {systemChoice === 'auto' && period && (
              <Text type="secondary" style={{ fontSize: 12 }}>
                {yearLabel(date.y)} 年适用：{SYSTEMS.find((s) => s.id === period.system)?.label}（历元 {period.epoch}；{period.basis}）
              </Text>
            )}
            <Text style={{ marginLeft: 12 }}>周天</Text>
            <Radio.Group size="small" optionType="button" value={zhouTian} onChange={(e) => setZhouTian(e.target.value)}
              options={[{ label: '365¼ 古度', value: 365.25 }, { label: '360°', value: 360 }]} />
          </Flex>
        </Flex>
      </Card>
      {error && <Alert type="error" message={error} showIcon closable onClose={() => setError(null)} />}
      <Card size="small">
        <Flex vertical gap={10}>
          <Flex gap={16} align="center" wrap="wrap">
            <Radio.Group value={view} onChange={(e) => setView(e.target.value)} optionType="button" size="small"
              options={[{ label: '盖图', value: 'gai' }, { label: '地平', value: 'horizon' }]} />
            {view === 'gai' && <Checkbox checked={fixedOrientation} onChange={(e) => setFixedOrientation(e.target.checked)}>固定方位（春分点朝上）</Checkbox>}
            <Checkbox.Group value={layers} onChange={(v) => setLayers(v as string[])}
              options={[
                { label: '星官', value: 'asterisms' },
                { label: '星官名', value: 'names' },
                ...(system.asterisms === 'chenzhuo' ? [{ label: '三家着色', value: 'schools' }] : []),
                { label: '二十八宿', value: 'mansions' },
                { label: '赤道', value: 'equator' },
                { label: '黄道', value: 'ecliptic' },
                { label: view === 'gai' ? '内外规' : '高度圈', value: 'circles' },
                ...(view === 'gai' ? [{ label: '地平', value: 'horizon' }] : []),
                { label: '日月五星', value: 'bodies' },
              ]} />
          </Flex>
          <Flex gap={12} align="center" wrap="wrap">
            <Text>极限星等 {magLimit.toFixed(1)}</Text>
            <div style={{ width: 220 }}>
              <Slider min={2} max={6.5} step={0.1} value={magLimit} onChange={setMagLimit} />
            </div>
            {snap && deltaT && (
              <Text type="secondary" style={{ fontSize: 12 }}>
                UT {fmtDate(jdUt)} {fmtTime(jdUt)}；ΔT {deltaT[0].toFixed(0)} ± {deltaT[1].toFixed(0)} s；地方恒星时 {(lstDeg / 15).toFixed(3)} h
              </Text>
            )}
          </Flex>
          <Spin spinning={!!busy} tip={busy ?? undefined}>
            {snap && culture ? (
              <SkyCanvas view={view} stars={stars} starMap={starMap} snap={snap} culture={culture} systemId={systemId} lstDeg={lstDeg} lat={place.lat}
                magLimit={magLimit} layers={layerSet} fixedOrientation={fixedOrientation} zhouTian={zhouTian} picked={picked} onPick={setPicked} />
            ) : (
              <div style={{ height: 400 }} />
            )}
          </Spin>
          <Legend schools={layerSet.has('schools') && system.asterisms === 'chenzhuo'} />
          <Text type="secondary" style={{ fontSize: 12 }}>
            点击星或日月五星看名称、入宿度、去极度与地平坐标。位置为当日真赤道视位置；盖图外圈注宿名与宿度（距星赤经差，按所选周天）。
            星官连线与星名据 Stellarium 中国星空文化（CC BY-SA 4.0）；陈卓体系可按石氏、甘氏、巫咸三家着色（据《开元占经》卷六十至七十，data/sky_culture/sanjia_schools.csv）；「按时期」时各体系自其历元起适用至下一体系历元（data/sky_culture/system_periods.csv），历元以前不用后出的认星；《灵台仪象志》只改距星，星官同《仪象考成》。
          </Text>
        </Flex>
      </Card>
      {info}
    </Flex>
  )
}
