/**
 * 日月食图共用的地图底图：地球（正射投影，可拖动旋转）或平面（等距圆柱，可平移），滚轮缩放，海岸线、经纬网、都城、选点。
 * 专题内容由 overlay 画在陆地之上、都城之下。
 */
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { geoArea, geoEquirectangular, geoGraticule10, geoOrthographic, geoPath, type GeoPath, type GeoProjection } from 'd3-geo'
import { feature } from 'topojson-client'
import type { Topology } from 'topojson-specification'
import type { Feature, FeatureCollection, Geometry, Point, Polygon } from 'geojson'
import land50m from 'world-atlas/land-50m.json'
import type { Capital } from './capitals'

export type MapView = 'globe' | 'flat'
export type OverlayContext = { path: GeoPath; pt: (lon: number, lat: number) => [number, number]; zoom: number }

const LAND = feature(land50m as unknown as Topology, (land50m as unknown as Topology).objects.land) as unknown as FeatureCollection
const GRATICULE = geoGraticule10()
const SPHERE = { type: 'Sphere' } as const

/** 闭合折线 → 面（d3 按走向定内外，取面积较小的一侧）；未闭合返回 null。 */
export function closedRing(line: [number, number, number][], dLon: number): Feature<Polygon> | null {
  const [f, l] = [line[0], line[line.length - 1]]
  if (line.length < 4 || f[0] !== l[0] || f[1] !== l[1]) return null
  const ring = line.map((p) => [p[0] + dLon, p[1]])
  const make = (r: number[][]): Feature<Polygon> => ({ type: 'Feature', properties: {}, geometry: { type: 'Polygon', coordinates: [r] } })
  const a = make(ring)
  return geoArea(a) > 2 * Math.PI ? make([...ring].reverse()) : a
}

/**
 * 影子落在地面的填色区（只用于绘图）。影锥部分越出地球时轮廓是一段开口的弧，两端都在昼夜界上：沿昼夜界（日下点 90° 的圆）
 * 走较短的一边接回去。影锥比地球小，落在昼夜界上的那段总短于半圈。分成多段的少见情形不填色，只留轮廓线。
 */
export function shadowRegion(lines: [number, number, number][][], sun: [number, number], dLon: number): Feature<Polygon> | null {
  if (lines.length !== 1) return null
  const line = lines[0]
  const closed = closedRing(line, dLon)
  if (closed) return closed
  if (line.length < 2) return null
  const rad = Math.PI / 180
  const vec = (lon: number, lat: number) => [Math.cos(lat * rad) * Math.cos(lon * rad), Math.cos(lat * rad) * Math.sin(lon * rad), Math.sin(lat * rad)]
  const dot = (a: number[], b: number[]) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
  const k = vec(sun[0], sun[1])
  // 昼夜界所在平面的一组正交基
  const u0 = Math.abs(k[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0]
  const d0 = dot(u0, k)
  const uRaw = u0.map((x, i) => x - d0 * k[i])
  const un = Math.hypot(uRaw[0], uRaw[1], uRaw[2])
  const u = uRaw.map((x) => x / un)
  const v = [k[1] * u[2] - k[2] * u[1], k[2] * u[0] - k[0] * u[2], k[0] * u[1] - k[1] * u[0]]
  const az = (p: [number, number, number]) => {
    const w = vec(p[0], p[1])
    return Math.atan2(dot(w, v), dot(w, u))
  }
  const a0 = az(line[line.length - 1])
  let da = az(line[0]) - a0
  if (da > Math.PI) da -= 2 * Math.PI
  if (da < -Math.PI) da += 2 * Math.PI
  const n = Math.max(2, Math.ceil(Math.abs(da) / (3 * rad)))
  const limb: [number, number, number][] = []
  for (let i = 1; i < n; i++) {
    const a = a0 + (da * i) / n
    const w = u.map((x, j) => x * Math.cos(a) + v[j] * Math.sin(a))
    limb.push([Math.atan2(w[1], w[0]) / rad, Math.asin(Math.max(-1, Math.min(1, w[2]))) / rad, 0])
  }
  return closedRing([...line, ...limb, line[0]], dLon)
}

export function GeoCanvas(props: {
  view: MapView
  /** 变化时复位视角 */
  resetKey: unknown
  globeCenter: [number, number]
  flatLon: number
  capitals: Capital[]
  picked: [number, number] | null
  onPick: (lon: number, lat: number) => void
  overlay: (ctx: OverlayContext) => ReactNode
}) {
  const { view, resetKey, globeCenter, flatLon, capitals, picked, onPick, overlay } = props
  const width = 760
  const height = view === 'globe' ? 640 : 400
  const [rotate, setRotate] = useState<[number, number]>([-globeCenter[0], -globeCenter[1]])
  const [zoom, setZoom] = useState(1)
  const [pan, setPan] = useState<[number, number]>([0, 0])
  const svgRef = useRef<SVGSVGElement | null>(null)

  useEffect(() => {
    setRotate(view === 'globe' ? [-globeCenter[0], -globeCenter[1]] : [-flatLon, 0])
    setZoom(1)
    setPan([0, 0])
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [resetKey, view])

  const projection: GeoProjection = useMemo(() => {
    if (view === 'globe') {
      return geoOrthographic()
        .rotate(rotate)
        .scale((Math.min(width, height) / 2 - 8) * zoom)
        .translate([width / 2 + pan[0], height / 2 + pan[1]])
        .clipAngle(90)
    }
    return geoEquirectangular()
      .rotate([rotate[0], 0])
      .scale((width / (2 * Math.PI)) * zoom)
      .translate([width / 2 + pan[0], height / 2 + pan[1]])
  }, [view, rotate, zoom, pan, height])
  const path = useMemo(() => geoPath(projection), [projection])
  // 海岸线点多，播放时每帧重绘，只在视角变化时重算
  const base = useMemo(() => ({ sphere: path(SPHERE) ?? '', graticule: path(GRATICULE) ?? '', land: path(LAND) ?? '' }), [path])

  // 拖动：地球转动、平面平移；位移很小视为点击
  const drag = useRef<{ x: number; y: number; rot: [number, number]; pan: [number, number]; moved: boolean } | null>(null)
  const onPointerDown = (e: React.PointerEvent) => {
    ;(e.target as Element).setPointerCapture?.(e.pointerId)
    drag.current = { x: e.clientX, y: e.clientY, rot: rotate, pan, moved: false }
  }
  const onPointerMove = (e: React.PointerEvent) => {
    const d = drag.current
    if (!d) return
    const dx = e.clientX - d.x
    const dy = e.clientY - d.y
    if (Math.abs(dx) + Math.abs(dy) > 3) d.moved = true
    if (!d.moved) return
    if (view === 'globe') {
      const k = 90 / projection.scale()
      setRotate([d.rot[0] + dx * k, Math.max(-90, Math.min(90, d.rot[1] - dy * k))])
    } else {
      setPan([d.pan[0] + dx, d.pan[1] + dy])
    }
  }
  const onPointerUp = (e: React.PointerEvent) => {
    const d = drag.current
    drag.current = null
    if (!d || d.moved || !svgRef.current) return
    const rect = svgRef.current.getBoundingClientRect()
    const x = ((e.clientX - rect.left) / rect.width) * width
    const y = ((e.clientY - rect.top) / rect.height) * height
    const ll = projection.invert?.([x, y])
    if (!ll || !Number.isFinite(ll[0])) return
    // 正射投影 invert 对背面点也有解，用前向投影回验
    const back = projection(ll)
    if (!back || Math.hypot(back[0] - x, back[1] - y) > 1) return
    onPick(((ll[0] + 540) % 360) - 180, ll[1])
  }
  // 滚轮缩放：平面图以鼠标处为中心（该点位置不动），地球以球心为中心。用原生非 passive 监听以阻止页面跟着滚动
  const latest = useRef({ zoom, pan, view })
  latest.current = { zoom, pan, view }
  useEffect(() => {
    const el = svgRef.current
    if (!el) return
    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      const { zoom: z, pan: p, view: v } = latest.current
      const z2 = Math.max(0.6, Math.min(40, z * (e.deltaY < 0 ? 1.15 : 1 / 1.15)))
      if (v === 'flat') {
        const rect = el.getBoundingClientRect()
        const cx = ((e.clientX - rect.left) / rect.width) * width
        const cy = ((e.clientY - rect.top) / rect.height) * height
        const k = z2 / z
        setPan([cx - k * (cx - (width / 2 + p[0])) - width / 2, cy - k * (cy - (height / 2 + p[1])) - height / 2])
      }
      setZoom(z2)
    }
    el.addEventListener('wheel', onWheel, { passive: false })
    return () => el.removeEventListener('wheel', onWheel)
  }, [height])

  const pt = (lon: number, lat: number) => {
    const g: Point = { type: 'Point', coordinates: [lon, lat] }
    return path.centroid(g as unknown as Feature<Geometry>) as [number, number] // 被裁掉时为 NaN
  }

  return (
    <svg
      ref={svgRef}
      viewBox={`0 0 ${width} ${height}`}
      style={{ width: '100%', maxWidth: width, touchAction: 'none', cursor: 'grab', userSelect: 'none', display: 'block' }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
    >
      <defs>
        <clipPath id="eclipse-map-clip">
          <rect x={0} y={0} width={width} height={height} />
        </clipPath>
      </defs>
      <g clipPath="url(#eclipse-map-clip)">
        <path d={base.sphere} fill="#dfeaf3" stroke="#8aa" strokeWidth={0.8} />
        <path d={base.graticule} fill="none" stroke="rgba(80,100,120,0.18)" strokeWidth={0.6} />
        <path d={base.land} fill="#f4f1e8" stroke="#a59f8e" strokeWidth={0.5} />
        {overlay({ path, pt, zoom })}
        {capitals.map((c) => {
          const [x, y] = pt(c.lon, c.lat)
          return Number.isFinite(x) ? (
            <g key={c.name} style={{ cursor: 'pointer' }} onPointerUp={(e) => { e.stopPropagation(); drag.current = null; onPick(c.lon, c.lat) }}>
              <rect x={x - 2.5} y={y - 2.5} width={5} height={5} fill="#222" />
              {zoom >= 2.5 && <text x={x + 5} y={y + 4} fontSize={11} fill="#222">{c.name}</text>}
              <title>{`${c.name}（${c.regimes.join('、')}）`}</title>
            </g>
          ) : null
        })}
        {picked && (() => {
          const [x, y] = pt(picked[0], picked[1])
          return Number.isFinite(x) ? <circle cx={x} cy={y} r={4} fill="#d35400" stroke="#fff" strokeWidth={1.2} /> : null
        })()}
      </g>
    </svg>
  )
}
