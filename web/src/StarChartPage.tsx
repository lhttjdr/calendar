/**
 * 星图：某地某时的天空，两种画法。
 * - 盖图：以北天极为中心的等距方位投影（半径正比于去极度），画内规、赤道、外规、黄道、二十八宿宿界与宿度，并按当地当时
 *   画地平线（其下加暗）。方向按「面北仰视」：本地恒星时所在子午圈朝上，赤经向东（顺时针）增大，可改为固定方位。
 * - 地平图：以天顶为中心的球极投影，北上东左（仰视）。日月五星取站心位置，高度计折射（视高度），可关。盖图仍是赤道坐标图，用地心位置、不计折射。
 * - 3D：站在当地仰望，拖动转向、滚轮缩放（Sky3D.tsx，three.js）。
 *
 * 天体位置（当日真赤道与真春分点的视位置）、星等、宿度、恒星时都由 Rust 算（skyWorker → SkyEngine）；这里只做投影、
 * 赤道→地平的坐标旋转、入宿度这类由所给位置直接相减的量，以及绘图。星官连线与星名取 Stellarium 中国星空文化
 * （CC BY-SA 4.0）。
 */
import { lazy, Suspense, useEffect, useMemo, useRef, useState } from 'react'
import { Alert, Button, Card, Checkbox, Descriptions, Flex, InputNumber, Radio, Select, Slider, Space, Spin, Typography } from 'antd'
import { clampCivilDay, civilToJd, fmtDate, fmtTime, jdToCivil, yearLabel } from './civilTime'
import { useWorkerRpc } from './useWorkerRpc'
import { PlayButton, SPEEDS, useFrameTimer } from './Playback'
import { loadCapitals, type Capital } from './capitals'
import type { Site, SkyRequest } from './skyWorker'
import {
  BODY_COLOR, BODY_LABEL, COLORS, MANSIONS, SCHOOL_COLOR, SYSTEMS, d2r, fromHorizon, greatCircle, HORIZON_SAMPLES, mod360, r2d, refractionAt, refractionTableAltitudes, starColor, starRadius, toHorizon,
  type BodyJson, type Culture, type Picked, type SnapshotJson, type Star,
} from './skyCommon'

// three.js 体积大，进入 3D 视图时才加载
const Sky3D = lazy(() => import('./Sky3D'))

const { Text } = Typography

type View = 'gai' | 'horizon' | 'sky3d'
type Projector = (ra: number, dec: number, margin?: number) => [number, number] | null

// ---------------------------------------------------------------------------------------------------------------------

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
  refraction: number[] | null
}) {
  const { refraction, view, stars, starMap, snap, culture, systemId, lstDeg, lat, magLimit, layers, fixedOrientation, zhouTian, picked, onPick } = props
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
    return (ra, dec, margin = 0) => {
      const { alt, az } = toHorizon(ra, dec, lstDeg, lat)
      const seen = alt + refractionAt(refraction, alt)
      // margin 是视半径：看上缘的视高度
      if ((margin > 0 ? alt + margin + refractionAt(refraction, alt + margin) : seen) < 0) return null
      const r = R * Math.tan(((90 - seen) / 2) * d2r)
      return [cx - r * Math.sin(az * d2r), cy - r * Math.cos(az * d2r)]
    }
  }, [view, rimDec, topRa, lstDeg, lat, R, cx, cy, refraction])

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

  // 地平图是当地所见，用站心位置；盖图是赤道坐标图，用地心位置
  const bodyProject = (b: BodyJson) => (view === 'horizon' ? project(b.topoRa, b.topoDec, b.semidiameter ?? 0) : project(b.ra, b.dec))
  const bodies = snap.bodies.map((b) => ({ b, xy: bodyProject(b) }))

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
    return b ? bodyProject(b) : null
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

/** 播放步长：分钟按时间推进；月、年按民用历推进（同月同日同时刻，看岁差不被周日运动打乱）。 */
const PLAY_STEPS: { label: string; minutes?: number; months?: number }[] = [
  { label: '1 分', minutes: 1 },
  { label: '10 分', minutes: 10 },
  { label: '1 时', minutes: 60 },
  { label: '1 日', minutes: 1440 },
  { label: '1 月', months: 1 },
  { label: '1 年', months: 12 },
  { label: '10 年', months: 120 },
  { label: '100 年', months: 1200 },
]
const MIN_YEAR = -2000
const MAX_YEAR = 3000

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
  const [layers, setLayers] = useState<string[]>(['asterisms', 'schools', 'mansions', 'equator', 'ecliptic', 'circles', 'horizon', 'bodies', 'refraction', 'skylight', 'milkyway'])
  const [culture, setCulture] = useState<Culture | null>(null)
  const [snap, setSnap] = useState<SnapshotJson | null>(null)
  const [deltaT, setDeltaT] = useState<[number, number] | null>(null)
  const [busy, setBusy] = useState<string | null>('加载星表与历表')
  const [error, setError] = useState<string | null>(null)
  const [picked, setPicked] = useState<Picked | null>(null)
  const [refractionTable, setRefractionTable] = useState<number[] | null>(null)
  const [site, setSite] = useState<Site>({ heightM: 0, tempC: null, pressureKpa: null, invDepthM: 0, invRiseK: 0, humidity: 0.5, cityNl: 0 })
  const [seaDip, setSeaDip] = useState(0)
  const [terrain, setTerrain] = useState<{ key: string; groundM: number; altitudes: number[]; missing: number } | null>(null)
  const [terrainBusy, setTerrainBusy] = useState(false)
  const [eyeM, setEyeM] = useState(1.7)
  const [playing, setPlaying] = useState(false)
  const [playStep, setPlayStep] = useState(2)
  const [playInterval, setPlayInterval] = useState(SPEEDS[1].value)
  const playingRef = useRef(false)
  playingRef.current = playing

  useEffect(() => {
    loadCapitals().then(setCapitals).catch(() => setCapitals([]))
    call<Culture>({ type: 'culture' })
      .then(setCulture)
      .catch((e) => setError(String(e instanceof Error ? e.message : e)))
  }, [call])

  useEffect(() => {
    let cancelled = false
    call<number[]>({ type: 'refractionTable', altitudes: refractionTableAltitudes(), site })
      .then((t) => !cancelled && setRefractionTable(t))
      .catch((e) => !cancelled && setError(String(e instanceof Error ? e.message : e)))
    call<number>({ type: 'seaHorizonDip', site })
      .then((d) => !cancelled && setSeaDip(d))
      .catch((e) => !cancelled && setError(String(e instanceof Error ? e.message : e)))
    return () => {
      cancelled = true
    }
  }, [call, site])

  // 地方平时 → UT：UT = 地方平时 − 经度/15 时
  const jdUt = civilToJd(date.y, date.m, date.d, date.h + date.mi / 60 - place.lon / 15)

  useEffect(() => {
    let cancelled = false
    const t = setTimeout(async () => {
      try {
        if (!playingRef.current) setBusy('计算天象')
        // ΔT 以 TT 为自变量：迭代两次
        let dt = (await call<number[]>({ type: 'deltaT', jd: jdUt }))[0]
        const est = await call<number[]>({ type: 'deltaT', jd: jdUt + dt / 86400 })
        dt = est[0]
        const s = await call<SnapshotJson>({ type: 'snapshot', jdTt: jdUt + dt / 86400, deltaT: dt, lon: place.lon, lat: place.lat, site })
        if (cancelled) return
        setDeltaT([est[0], est[1]])
        setSnap(s)
        setError(null)
      } catch (e) {
        if (!cancelled) {
          setError(String(e instanceof Error ? e.message : e))
          setPlaying(false)
        }
      } finally {
        if (!cancelled) setBusy(null)
      }
    }, playingRef.current ? 0 : 150)
    return () => {
      cancelled = true
      clearTimeout(t)
    }
  }, [jdUt, place.lon, place.lat, site, call])

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

  // 地形只在勾选时才联网取高程瓦片；取到后用地面海拔作观测点海拔
  const wantTerrain = layers.includes('terrain') && view === 'sky3d'
  const terrainKey = `${place.lon},${place.lat},${eyeM}`
  useEffect(() => {
    if (!wantTerrain || terrain?.key === terrainKey) return
    let cancelled = false
    setTerrainBusy(true)
    call<{ groundM: number; altitudes: number[]; missing: number }>({ type: 'terrainHorizon', lon: place.lon, lat: place.lat, eyeM })
      .then((r) => {
        if (cancelled) return
        setTerrain({ key: terrainKey, ...r })
        setSite((s) => ({ ...s, heightM: Math.round(r.groundM + eyeM) }))
      })
      .catch((e) => {
        if (cancelled) return
        setError(String(e instanceof Error ? e.message : e))
        setLayers((l) => l.filter((x) => x !== 'terrain'))
      })
      .finally(() => !cancelled && setTerrainBusy(false))
    return () => {
      cancelled = true
    }
  }, [wantTerrain, terrainKey, terrain?.key, eyeM, place.lon, place.lat, call])
  const horizon = useMemo(() => {
    if (wantTerrain && terrain?.key === terrainKey) return terrain.altitudes
    if (layers.includes('seaHorizon') && seaDip > 0) return new Array<number>(HORIZON_SAMPLES).fill(-seaDip)
    return null
  }, [wantTerrain, terrain, terrainKey, layers, seaDip])

  const lstDeg = snap ? mod360(snap.gastDeg + place.lon) : 0
  const layerSet = useMemo(() => new Set(layers), [layers])
  const refraction = layerSet.has('refraction') ? refractionTable : null
  const system = SYSTEMS.find((s) => s.id === systemId)!

  const shift = (hours: number) => {
    const c = jdToCivil(civilToJd(date.y, date.m, date.d, date.h + date.mi / 60) + hours / 24)
    setDate({ y: c.y, m: c.m, d: c.d, h: c.h, mi: c.mi })
  }

  useFrameTimer(playing, snap, playInterval, () => {
    const st = PLAY_STEPS[playStep]
    let next = date
    if (st.minutes) {
      const c = jdToCivil(civilToJd(date.y, date.m, date.d, date.h + date.mi / 60) + st.minutes / 1440)
      next = { y: c.y, m: c.m, d: c.d, h: c.h, mi: c.mi }
    } else {
      const months = date.y * 12 + date.m - 1 + st.months!
      const y = Math.floor(months / 12)
      const m = months - y * 12 + 1
      next = { ...date, y, m, d: clampCivilDay(y, m, date.d) }
    }
    if (next.y < MIN_YEAR || next.y > MAX_YEAR) setPlaying(false)
    else setDate(next)
  })

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
    // 高度按当地所见：日月五星用站心位置
    const hz = 'topoRa' in target ? toHorizon(target.topoRa, target.topoDec, lstDeg, place.lat) : toHorizon(target.ra, target.dec, lstDeg, place.lat)
    const hzSeen = hz.alt + refractionAt(refractionTable, hz.alt)
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
          <Descriptions.Item label="视高度 / 真高度">{hzSeen.toFixed(2)}° / {hz.alt.toFixed(2)}°{'topoRa' in target ? '（站心）' : ''}</Descriptions.Item>
          <Descriptions.Item label="方位">{hz.az.toFixed(2)}°（自北经东）</Descriptions.Item>
          {'topoRa' in target && target.semidiameter != null && <Descriptions.Item label="视半径 / 上缘视高度">{(target.semidiameter * 60).toFixed(2)}′ / {(hz.alt + target.semidiameter + refractionAt(refractionTable, hz.alt + target.semidiameter)).toFixed(2)}°</Descriptions.Item>}
          {'topoRa' in target && <Descriptions.Item label="站心赤经 / 赤纬">{target.topoRa.toFixed(3)}° / {target.topoDec.toFixed(3)}°</Descriptions.Item>}
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
            <InputNumber value={date.y} min={MIN_YEAR} max={MAX_YEAR} style={{ width: 90 }} onChange={(v) => v != null && setDate({ ...date, y: v })} />
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
            <span style={{ marginLeft: 12 }}><PlayButton playing={playing} onClick={() => setPlaying(!playing)} /></span>
            <Text>步长</Text>
            <Select size="small" style={{ width: 84 }} value={playStep} onChange={setPlayStep} options={PLAY_STEPS.map((s, i) => ({ label: s.label, value: i }))} />
            <Text>速度</Text>
            <Select size="small" style={{ width: 66 }} value={playInterval} onChange={setPlayInterval} options={SPEEDS} />
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
              options={[{ label: '盖图', value: 'gai' }, { label: '地平', value: 'horizon' }, { label: '3D', value: 'sky3d' }]} />
            {view === 'gai' && <Checkbox checked={fixedOrientation} onChange={(e) => setFixedOrientation(e.target.checked)}>固定方位（春分点朝上）</Checkbox>}
            <Checkbox.Group value={layers} onChange={(v) => setLayers(v as string[])}
              options={[
                { label: '星官', value: 'asterisms' },
                { label: '星官名', value: 'names' },
                ...(system.asterisms === 'chenzhuo' ? [{ label: '三家着色', value: 'schools' }] : []),
                { label: '二十八宿', value: 'mansions' },
                { label: '赤道', value: 'equator' },
                { label: '黄道', value: 'ecliptic' },
                { label: view === 'horizon' ? '高度圈' : '内外规', value: 'circles' },
                ...(view === 'gai' ? [{ label: '地平', value: 'horizon' }] : view === 'sky3d' ? [{ label: '地面', value: 'horizon' }] : []),
                ...(view === 'gai' ? [] : [{ label: '大气折射', value: 'refraction' }]),
                ...(view === 'sky3d' ? [{ label: '大气散射', value: 'skylight' }, { label: '银河', value: 'milkyway' }, { label: '海平线俯角', value: 'seaHorizon' }, { label: '地形', value: 'terrain' }] : []),
                { label: '日月五星', value: 'bodies' },
              ]} />
          </Flex>
          <Flex gap={12} align="center" wrap="wrap">
            <Text>极限星等 {magLimit.toFixed(1)}</Text>
            <div style={{ width: 220 }}>
              <Slider min={2} max={6.5} step={0.1} value={magLimit} onChange={setMagLimit} />
            </div>
            {view !== 'gai' && (
              <>
                <Text>海拔</Text>
                <InputNumber size="small" style={{ width: 84 }} value={site.heightM} min={-400} max={9000} step={100} suffix="m" onChange={(v) => v != null && setSite({ ...site, heightM: v })} />
                <Text>气温</Text>
                <InputNumber size="small" style={{ width: 84 }} value={site.tempC} min={-60} max={60} placeholder="标准" suffix="℃" onChange={(v) => setSite({ ...site, tempC: v })} />
                <Text>气压</Text>
                <InputNumber size="small" style={{ width: 96 }} value={site.pressureKpa} min={30} max={110} step={0.5} placeholder="标准" suffix="kPa" onChange={(v) => setSite({ ...site, pressureKpa: v })} />
                <Text>贴地逆温</Text>
                <InputNumber size="small" style={{ width: 84 }} value={site.invDepthM} min={0} max={2000} step={50} suffix="m" onChange={(v) => setSite({ ...site, invDepthM: v ?? 0 })} />
                <Text type="secondary">内升</Text>
                <InputNumber size="small" style={{ width: 76 }} value={site.invRiseK} min={0} max={30} suffix="K" onChange={(v) => setSite({ ...site, invRiseK: v ?? 0 })} />
                {view === 'sky3d' && (
                  <>
                    <Text>湿度</Text>
                    <InputNumber size="small" style={{ width: 76 }} value={Math.round(site.humidity * 100)} min={0} max={100} step={10} suffix="%" onChange={(v) => setSite({ ...site, humidity: (v ?? 50) / 100 })} />
                    <Text>城市光</Text>
                    <InputNumber size="small" style={{ width: 96 }} value={site.cityNl} min={0} max={100000} step={500} suffix="nL" onChange={(v) => setSite({ ...site, cityNl: v ?? 0 })} />
                  </>
                )}
              </>
            )}
            {wantTerrain && (
              <>
                <Text>眼高</Text>
                <InputNumber size="small" style={{ width: 84 }} value={eyeM} min={0} max={500} step={1} suffix="m" onChange={(v) => setEyeM(v ?? 1.7)} />
                <Text type="secondary">{terrainBusy ? '取高程数据…' : terrain?.key === terrainKey ? `地面海拔 ${terrain.groundM.toFixed(0)} m` : ''}</Text>
              </>
            )}
            {snap && snap.skyLight.sunVisible < 1 && <Text type="warning">日面未遮 {(snap.skyLight.sunVisible * 100).toFixed(1)}%</Text>}
            {snap && deltaT && (
              <Text type="secondary" style={{ fontSize: 12 }}>
                UT {fmtDate(jdUt)} {fmtTime(jdUt)}；ΔT {deltaT[0].toFixed(0)} ± {deltaT[1].toFixed(0)} s；地方恒星时 {(lstDeg / 15).toFixed(3)} h
              </Text>
            )}
          </Flex>
          <Spin spinning={!!busy} tip={busy ?? undefined}>
            {snap && culture ? (
              view === 'sky3d' ? (
                <Suspense fallback={<div style={{ height: 400 }} />}>
                  <Sky3D stars={stars} starMap={starMap} snap={snap} culture={culture} systemId={systemId} lstDeg={lstDeg} lat={place.lat}
                    magLimit={magLimit} layers={layerSet} zhouTian={zhouTian} picked={picked} onPick={setPicked} playing={playing} refraction={refraction}
                    horizon={horizon}
                    overlay={
                      <Flex gap={8} align="center" wrap="wrap">
                        <Button size="small" onClick={() => shift(-1)}>−1 时</Button>
                        <PlayButton playing={playing} onClick={() => setPlaying(!playing)} />
                        <Button size="small" onClick={() => shift(1)}>+1 时</Button>
                        <Select size="small" style={{ width: 84 }} value={playStep} onChange={setPlayStep} getPopupContainer={(n) => n.parentElement!} options={PLAY_STEPS.map((st, i) => ({ label: st.label, value: i }))} />
                        <Select size="small" style={{ width: 66 }} value={playInterval} onChange={setPlayInterval} getPopupContainer={(n) => n.parentElement!} options={SPEEDS} />
                        <span style={{ fontSize: 13 }}>{date.y} 年 {date.m} 月 {date.d} 日 {String(date.h).padStart(2, '0')}:{String(date.mi).padStart(2, '0')} · {place.name}</span>
                      </Flex>
                    } />
                </Suspense>
              ) : (
                <SkyCanvas view={view} stars={stars} starMap={starMap} snap={snap} culture={culture} systemId={systemId} lstDeg={lstDeg} lat={place.lat}
                  magLimit={magLimit} layers={layerSet} fixedOrientation={fixedOrientation} zhouTian={zhouTian} picked={picked} onPick={setPicked} refraction={view === 'horizon' ? refraction : null} />
              )
            ) : (
              <div style={{ height: 400 }} />
            )}
          </Spin>
          <Legend schools={layerSet.has('schools') && system.asterisms === 'chenzhuo'} />
          <Text type="secondary" style={{ fontSize: 12 }}>
            点击星或日月五星看名称、入宿度、去极度与地平坐标。3D 视图拖动转向、滚轮或双指缩放，选中天体后可「跟踪选中」（放大播放日食时太阳不会漂出画面）；日月放大后显示真实圆面（站心视半径，近地平被折射压扁），视场大时圆面太小则画符号，符号大小不代表视径；上缘露出地平线即可见。地平图与 3D 是当地所见：日月五星取站心位置，计入大气折射（光线追迹，按所填海拔、气温、气压、逆温，留空取该海拔的标准大气）。地平线缺省取高度 0°；3D 可勾「海平线俯角」按海拔算海地平，或勾「地形」按现代高程数据算真实山形（联网取瓦片；山形千年尺度几乎不变；2 km 内不算，以滤掉现代建筑，城墙、林木也不在内；高程数据 Mapzen/Tilezen，含 USGS SRTM、GMTED2010 与 NOAA ETOPO1）。3D 的天色亮度与星的隐现据 Schaefer 模型，颜色只是显示；不计云。银河图：NASA/GSFC SVS Deep Star Maps，Gaia DR2: ESA/Gaia/DPAC。说明见文档「3D 星空」。关掉「大气散射」即无天光，白天也能看到星与日月同在（只影响显示，折射另有开关）。盖图是赤道坐标图，用地心位置、不计折射。位置为当日真赤道视位置；盖图外圈注宿名与宿度（距星赤经差，按所选周天）。
            星官连线与星名据 Stellarium 中国星空文化（CC BY-SA 4.0）；陈卓体系可按石氏、甘氏、巫咸三家着色（据《开元占经》卷六十至七十，data/sky_culture/sanjia_schools.csv）；「按时期」时各体系自其历元起适用至下一体系历元（data/sky_culture/system_periods.csv），历元以前不用后出的认星；《灵台仪象志》只改距星，星官同《仪象考成》。
          </Text>
        </Flex>
      </Card>
      {info}
    </Flex>
  )
}
