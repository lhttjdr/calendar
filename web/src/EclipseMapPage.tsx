/**
 * 日食图（食带图）：按年列出日食，选中一次后画中心线、本影/半影南北界、带食出没界线、影子轮廓与昼夜界线。
 * 可切换「地球」（正射投影，可拖动旋转，缺省从太阳方向看）与「平面」（等距圆柱）；点击地图或都城给出该地食况。
 *
 * 计算全在 Rust（eclipseWorker → EclipseEngine），这里只画。ΔT 只经地球自转进入，换 ΔT 时全图严格沿经度平移
 * （lonShiftPerSecond），所以 ΔT 滑块不重算，只平移；某地食况按当前 ΔT 重算。
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Alert, Button, Card, Checkbox, Descriptions, Flex, InputNumber, Radio, Slider, Space, Spin, Table, Tag, Typography } from 'antd'
import { geoCircle, geoEquirectangular, geoGraticule10, geoOrthographic, geoPath, type GeoProjection } from 'd3-geo'
import { feature } from 'topojson-client'
import type { Topology } from 'topojson-specification'
import type { Feature, FeatureCollection, Geometry, MultiLineString, Point } from 'geojson'
import land50m from 'world-atlas/land-50m.json'
import type { EclipseRequest } from './eclipseWorker'
import { fmtDate, fmtTime, yearLabel } from './civilTime'
import { useWorkerRpc } from './useWorkerRpc'
import { loadCapitals, type Capital } from './capitals'

const { Text } = Typography

// ---------------------------------------------------------------------------------------------------------------------
// 数据类型（与 rust/wasm-lib/src/lib_impl.rs 日食部分的 JSON 对应）

type Kind = 'P' | 'A' | 'T' | 'H'
type Global = { greatestTt: number; gamma: number; magnitude: number; kind: Kind; central: boolean }
type ListItem = Global & { deltaT: number; deltaTSigma: number }
/** [经度°, 纬度°, jd_tt] */
type Pt = [number, number, number]
type Line = Pt[]
type EclipseMap = {
  global: Global
  deltaT: number
  p1: number
  p4: number
  centralContacts: [number, number] | null
  greatestPoint: [Pt]
  subsolar: [Pt]
  centralLine: Line[]
  umbraNorth: Line[]
  umbraSouth: Line[]
  penumbraLimits: Line[]
  riseSet: Line[]
  magnitudeContours: { level: number; lines: Line[] }[]
  outlines: { jdTt: number; penumbra: Line[]; umbra: Line[] }[]
  lonShiftPerSecond: number
}
type Local = {
  first: number
  second: number | null
  greatest: number
  third: number | null
  last: number
  magnitude: number
  altFirst: number
  altGreatest: number
  altLast: number
  visibleFrom: number | null
  visibleTo: number | null
  maxVisibleMagnitude: number | null
}

const KIND_LABEL: Record<Kind, string> = { T: '全食', A: '环食', H: '全环食', P: '偏食' }
const KIND_COLOR: Record<Kind, string> = { T: 'red', A: 'gold', H: 'purple', P: 'default' }

// ---------------------------------------------------------------------------------------------------------------------
// 绘图

const LAND = feature(land50m as unknown as Topology, (land50m as unknown as Topology).objects.land) as unknown as FeatureCollection
const GRATICULE = geoGraticule10()
const SPHERE = { type: 'Sphere' } as const

const COLORS = {
  ocean: '#dfeaf3',
  land: '#f4f1e8',
  landStroke: '#a59f8e',
  graticule: 'rgba(80,100,120,0.18)',
  night: 'rgba(20,30,60,0.22)',
  penumbraLimit: '#3b6fb6',
  umbraLimit: '#c0392b',
  central: '#8e1b10',
  riseSet: '#2e8b57',
  outlinePen: 'rgba(59,111,182,0.55)',
  outlineUmbra: 'rgba(192,57,43,0.75)',
  sigma: 'rgba(142,27,16,0.45)',
  capital: '#222',
  pick: '#d35400',
  magnitude: '#7a5c99',
}

function shiftLines(lines: Line[], dLon: number): MultiLineString {
  return { type: 'MultiLineString', coordinates: lines.map((l) => l.map((p) => [p[0] + dLon, p[1]])) }
}

type View = 'globe' | 'flat'

function MapCanvas(props: {
  map: EclipseMap
  dLon: number
  sigmaLon: number
  view: View
  layers: Set<string>
  capitals: Capital[]
  picked: [number, number] | null
  onPick: (lon: number, lat: number) => void
}) {
  const { map, dLon, sigmaLon, view, layers, capitals, picked, onPick } = props
  const width = 760
  const height = view === 'globe' ? 640 : 400
  const sub = map.subsolar[0]
  const gp = map.greatestPoint[0]
  const [rotate, setRotate] = useState<[number, number]>([-sub[0], -sub[1]])
  const [zoom, setZoom] = useState(1)
  const [pan, setPan] = useState<[number, number]>([0, 0])
  const svgRef = useRef<SVGSVGElement | null>(null)

  // 换日食或视图时复位：地球从太阳方向看；平面以极大食点经度居中
  useEffect(() => {
    setRotate(view === 'globe' ? [-(sub[0] + dLon), -sub[1]] : [-(gp[0] + dLon), 0])
    setZoom(1)
    setPan([0, 0])
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [map, view])

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

  // 半径取 89.99° 而非 90°：从太阳方向看时夜半球边界恰在正射投影裁剪圈上，裁剪退化会填满整个圆盘
  const night = useMemo(
    () => geoCircle().center([sub[0] + dLon + 180, -sub[1]]).radius(89.99)(),
    [sub, dLon]
  )

  // 中心线上按影子轮廓时刻标 UT
  const centralTicks = useMemo(() => {
    const pts = map.centralLine.flat()
    if (!pts.length) return []
    return map.outlines.map((o) => {
      let best = pts[0]
      for (const p of pts) if (Math.abs(p[2] - o.jdTt) < Math.abs(best[2] - o.jdTt)) best = p
      return Math.abs(best[2] - o.jdTt) < 2 / 1440 ? { p: best, label: fmtTime(o.jdTt - map.deltaT / 86400) } : null
    }).filter((x): x is { p: Pt; label: string } => !!x)
  }, [map])

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
      const k = 90 / (projection.scale() * 1.5)
      setRotate([d.rot[0] + dx * k * 1.5, Math.max(-90, Math.min(90, d.rot[1] - dy * k * 1.5))])
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
    const lon = ((ll[0] + 540) % 360) - 180
    onPick(lon, ll[1])
  }
  // 滚轮缩放：平面图以鼠标处为中心（该点位置不动），地球以球心为中心。用原生非 passive 监听以阻止页面跟着滚动
  const view_ = useRef({ zoom, pan, view })
  view_.current = { zoom, pan, view }
  useEffect(() => {
    const el = svgRef.current
    if (!el) return
    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      const { zoom: z, pan: p, view: v } = view_.current
      const z2 = Math.max(0.6, Math.min(40, z * (e.deltaY < 0 ? 1.15 : 1 / 1.15)))
      if (v === 'flat') {
        const rect = el.getBoundingClientRect()
        const cx = ((e.clientX - rect.left) / rect.width) * width
        const cy = ((e.clientY - rect.top) / rect.height) * height
        const k = z2 / z
        const tx = width / 2 + p[0]
        const ty = height / 2 + p[1]
        setPan([cx - k * (cx - tx) - width / 2, cy - k * (cy - ty) - height / 2])
      }
      setZoom(z2)
    }
    el.addEventListener('wheel', onWheel, { passive: false })
    return () => el.removeEventListener('wheel', onWheel)
  }, [height])

  const pt = (lon: number, lat: number) => {
    const g: Point = { type: 'Point', coordinates: [lon, lat] }
    return path.centroid(g as unknown as Feature<Geometry>) // 被裁掉时为 NaN
  }

  const line = (lines: Line[], stroke: string, w: number, dash?: string, key?: string) => (
    <path key={key} d={path(shiftLines(lines, dLon)) ?? ''} fill="none" stroke={stroke} strokeWidth={w} strokeDasharray={dash} />
  )

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
        <path d={path(SPHERE) ?? ''} fill={COLORS.ocean} stroke="#8aa" strokeWidth={0.8} />
        <path d={path(GRATICULE) ?? ''} fill="none" stroke={COLORS.graticule} strokeWidth={0.6} />
        <path d={path(LAND) ?? ''} fill={COLORS.land} stroke={COLORS.landStroke} strokeWidth={0.5} />
        {layers.has('night') && <path d={path(night) ?? ''} fill={COLORS.night} stroke="none" />}
        {layers.has('outlines') &&
          map.outlines.map((o, i) => (
            <g key={`o${i}`}>
              {line(o.penumbra, COLORS.outlinePen, 0.7, '2 3')}
              {line(o.umbra, COLORS.outlineUmbra, 1)}
            </g>
          ))}
        {layers.has('magnitude') &&
          map.magnitudeContours.map((c) => (
            <g key={`m${c.level}`}>
              {line(c.lines, COLORS.magnitude, 0.8)}
              {c.lines.map((l, i) => {
                const p = l[Math.floor(l.length / 2)]
                const [x, y] = pt(p[0] + dLon, p[1])
                return Number.isFinite(x) ? (
                  <text key={i} x={x} y={y} fontSize={9} fill={COLORS.magnitude} textAnchor="middle" dy={-2}
                    style={{ paintOrder: 'stroke', stroke: COLORS.land, strokeWidth: 2.5 }}>{c.level.toFixed(1)}</text>
                ) : null
              })}
            </g>
          ))}
        {line(map.penumbraLimits, COLORS.penumbraLimit, 1.3)}
        {layers.has('riseSet') && line(map.riseSet, COLORS.riseSet, 1.2, '6 3')}
        {layers.has('sigma') && sigmaLon > 0 && (
          <>
            {line(map.umbraNorth.concat(map.umbraSouth).map((l) => l.map((p) => [p[0] - sigmaLon, p[1], p[2]] as Pt)), COLORS.sigma, 0.8, '3 3')}
            {line(map.umbraNorth.concat(map.umbraSouth).map((l) => l.map((p) => [p[0] + sigmaLon, p[1], p[2]] as Pt)), COLORS.sigma, 0.8, '3 3')}
          </>
        )}
        {line(map.umbraNorth, COLORS.umbraLimit, 1.4)}
        {line(map.umbraSouth, COLORS.umbraLimit, 1.4)}
        {line(map.centralLine, COLORS.central, 1.6)}
        {centralTicks.map((t, i) => {
          const [x, y] = pt(t.p[0] + dLon, t.p[1])
          return Number.isFinite(x) ? (
            <g key={`t${i}`}>
              <circle cx={x} cy={y} r={2.2} fill={COLORS.central} />
              <text x={x + 4} y={y - 4} fontSize={10} fill={COLORS.central}>{t.label}</text>
            </g>
          ) : null
        })}
        {(() => {
          const [x, y] = pt(gp[0] + dLon, gp[1])
          return Number.isFinite(x) ? (
            <g>
              <circle cx={x} cy={y} r={5} fill="none" stroke={COLORS.central} strokeWidth={1.6} />
              <line x1={x - 7} x2={x + 7} y1={y} y2={y} stroke={COLORS.central} />
              <line x1={x} x2={x} y1={y - 7} y2={y + 7} stroke={COLORS.central} />
            </g>
          ) : null
        })()}
        {layers.has('capitals') &&
          capitals.map((c) => {
            const [x, y] = pt(c.lon, c.lat)
            return Number.isFinite(x) ? (
              <g key={c.name} style={{ cursor: 'pointer' }} onPointerUp={(e) => { e.stopPropagation(); drag.current = null; onPick(c.lon, c.lat) }}>
                <rect x={x - 2.5} y={y - 2.5} width={5} height={5} fill={COLORS.capital} />
                {zoom >= 2.5 && <text x={x + 5} y={y + 4} fontSize={11} fill={COLORS.capital}>{c.name}</text>}
                <title>{`${c.name}（${c.regimes.join('、')}）`}</title>
              </g>
            ) : null
          })}
        {picked && (() => {
          const [x, y] = pt(picked[0], picked[1])
          return Number.isFinite(x) ? <circle cx={x} cy={y} r={4} fill={COLORS.pick} stroke="#fff" strokeWidth={1.2} /> : null
        })()}
      </g>
    </svg>
  )
}

function Legend() {
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
      {item(COLORS.central, '中心线')}
      {item(COLORS.umbraLimit, '本影（全/环食）南北界')}
      {item(COLORS.penumbraLimit, '半影（偏食）南北界')}
      {item(COLORS.magnitude, '可见极大食分等值线')}
      {item(COLORS.riseSet, '带食出没界线', '6 3')}
      {item(COLORS.outlineUmbra, '影子轮廓（整点/半点 UT）')}
      {item(COLORS.sigma, '本影界 ΔT ± σ', '3 3')}
    </Flex>
  )
}

// ---------------------------------------------------------------------------------------------------------------------

export function EclipseMapPage() {
  const call = useWorkerRpc<EclipseRequest>(() => new Worker(new URL('./eclipseWorker.ts', import.meta.url), { type: 'module' }))
  const [year, setYear] = useState(-180)
  const [list, setList] = useState<ListItem[] | null>(null)
  const [selected, setSelected] = useState<ListItem | null>(null)
  const [map, setMap] = useState<EclipseMap | null>(null)
  const [busy, setBusy] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [view, setView] = useState<View>('globe')
  const [layers, setLayers] = useState<string[]>(['magnitude', 'outlines', 'riseSet', 'night', 'capitals', 'sigma'])
  /** 相对缺省 ΔT（Morrison–Stephenson 2021）的偏移，秒 */
  const [dtOffset, setDtOffset] = useState(0)
  const [capitals, setCapitals] = useState<Capital[]>([])
  const [picked, setPicked] = useState<[number, number] | null>(null)
  const [local, setLocal] = useState<Local | null | undefined>(undefined)

  useEffect(() => {
    loadCapitals().then(setCapitals).catch(() => setCapitals([]))
  }, [])

  const query = useCallback(
    async (y: number) => {
      setBusy('计算日食表')
      setError(null)
      try {
        const r = await call<ListItem[]>({ type: 'list', year: y })
        setList(r)
        setSelected(r[0] ?? null)
      } catch (e) {
        setError(String(e instanceof Error ? e.message : e))
      } finally {
        setBusy(null)
      }
    },
    [call]
  )

  useEffect(() => {
    query(year)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    if (!selected) {
      setMap(null)
      return
    }
    let cancelled = false
    setBusy('计算食带')
    setDtOffset(0)
    setLocal(undefined)
    call<EclipseMap>({ type: 'map', jd: selected.greatestTt, deltaT: selected.deltaT, stepS: 60, outlineS: 1800 })
      .then((m) => !cancelled && setMap(m))
      .catch((e) => !cancelled && setError(String(e instanceof Error ? e.message : e)))
      .finally(() => !cancelled && setBusy(null))
    return () => {
      cancelled = true
    }
  }, [selected, call])

  const deltaT = (selected?.deltaT ?? 0) + dtOffset
  const sigma = selected?.deltaTSigma ?? 0

  useEffect(() => {
    if (!picked || !selected) return
    let cancelled = false
    call<Local | null>({ type: 'local', jd: selected.greatestTt, deltaT, lon: picked[0], lat: picked[1] })
      .then((r) => !cancelled && setLocal(r))
      .catch(() => !cancelled && setLocal(null))
    return () => {
      cancelled = true
    }
  }, [picked, selected, deltaT, call])

  const dLon = map ? dtOffset * map.lonShiftPerSecond : 0
  const sigmaLon = map ? sigma * Math.abs(map.lonShiftPerSecond) : 0
  const layerSet = useMemo(() => new Set(layers), [layers])

  const ut = (jdTt: number) => jdTt - deltaT / 86400
  /** 地方平太阳时 = UT + 经度/15 */
  const lmt = (jdTt: number) => ut(jdTt) + (picked ? picked[0] / 360 : 0)
  const pickedCapital = picked && capitals.find((c) => Math.abs(c.lon - picked[0]) < 1e-6 && Math.abs(c.lat - picked[1]) < 1e-6)

  return (
    <Flex vertical gap={12}>
      <Card size="small">
        <Flex gap={12} align="center" wrap="wrap">
          <Text>年份（天文纪年，0 = 前 1 年）</Text>
          <InputNumber value={year} min={-1999} max={3000} onChange={(v) => v != null && setYear(v)} onPressEnter={() => query(year)} />
          <Button type="primary" onClick={() => query(year)}>查询</Button>
          <Button onClick={() => { const y = year - 1; setYear(y); query(y) }}>上一年</Button>
          <Button onClick={() => { const y = year + 1; setYear(y); query(y) }}>下一年</Button>
          <Text type="secondary">{yearLabel(year)} 年；1582-10-15 前用儒略历；时刻为 UT</Text>
        </Flex>
      </Card>
      {error && <Alert type="error" message={error} showIcon closable onClose={() => setError(null)} />}
      <Spin spinning={!!busy} tip={busy ?? undefined}>
        {list && (
          <Table<ListItem>
            size="small"
            pagination={false}
            rowKey={(r) => String(r.greatestTt)}
            dataSource={list}
            locale={{ emptyText: `${yearLabel(year)} 年无日食` }}
            rowSelection={{
              type: 'radio',
              selectedRowKeys: selected ? [String(selected.greatestTt)] : [],
              onChange: (_, rows) => setSelected(rows[0] ?? null),
            }}
            onRow={(r) => ({ onClick: () => setSelected(r) })}
            columns={[
              { title: '日期', render: (_, r) => fmtDate(r.greatestTt - r.deltaT / 86400) },
              { title: '食甚 UT', render: (_, r) => fmtTime(r.greatestTt - r.deltaT / 86400) },
              { title: '类型', render: (_, r) => <Tag color={KIND_COLOR[r.kind]}>{KIND_LABEL[r.kind]}{r.central ? '' : '（非中心）'}</Tag> },
              { title: 'γ', render: (_, r) => r.gamma.toFixed(4) },
              { title: '食分', render: (_, r) => r.magnitude.toFixed(4) },
              { title: 'ΔT（s）', render: (_, r) => `${r.deltaT.toFixed(0)} ± ${r.deltaTSigma.toFixed(0)}` },
            ]}
          />
        )}
        {map && selected && (
          <Card size="small" style={{ marginTop: 12 }}>
            <Flex vertical gap={10}>
              <Flex gap={16} align="center" wrap="wrap">
                <Radio.Group value={view} onChange={(e) => setView(e.target.value)} optionType="button" size="small"
                  options={[{ label: '地球', value: 'globe' }, { label: '平面', value: 'flat' }]} />
                <Checkbox.Group value={layers} onChange={(v) => setLayers(v as string[])}
                  options={[
                    { label: '食分等值线', value: 'magnitude' },
                    { label: '影子轮廓', value: 'outlines' },
                    { label: '带食出没', value: 'riseSet' },
                    { label: '夜半球', value: 'night' },
                    { label: '都城', value: 'capitals' },
                    { label: 'ΔT±σ', value: 'sigma' },
                  ]} />
              </Flex>
              <Flex gap={12} align="center" wrap="wrap">
                <Text>ΔT = {deltaT.toFixed(0)} s</Text>
                <Text type="secondary">（MS2021 {selected.deltaT.toFixed(0)} ± {sigma.toFixed(0)} s；偏移 {sigma > 0 ? (dtOffset / sigma).toFixed(2) : '0'}σ，全图东移 {dLon.toFixed(3)}°）</Text>
                <div style={{ flex: '1 1 240px', minWidth: 200 }}>
                  <Slider min={-3 * Math.max(sigma, 1)} max={3 * Math.max(sigma, 1)} step={1} value={dtOffset} onChange={setDtOffset}
                    marks={sigma > 0 ? { [-2 * sigma]: '−2σ', [-sigma]: '−σ', 0: '0', [sigma]: '+σ', [2 * sigma]: '+2σ' } : undefined}
                    tooltip={{ formatter: (v) => `${(v ?? 0) >= 0 ? '+' : ''}${v} s` }} />
                </div>
              </Flex>
              <MapCanvas map={map} dLon={dLon} sigmaLon={sigmaLon} view={view} layers={layerSet} capitals={capitals} picked={picked}
                onPick={(lon, lat) => setPicked([lon, lat])} />
              <Legend />
              <Text type="secondary" style={{ fontSize: 12 }}>
                拖动旋转（平面图为平移），滚轮缩放（放大后显示都城名），点击地图或都城看该地食况。
                极大食 {fmtDate(ut(map.global.greatestTt))} {fmtTime(ut(map.global.greatestTt), true)} UT，
                γ = {map.global.gamma.toFixed(4)}，食分 {map.global.magnitude.toFixed(4)}；
                半影触地 {fmtTime(ut(map.p1))}—{fmtTime(ut(map.p4))} UT
                {map.centralContacts && `；中心食 ${fmtTime(ut(map.centralContacts[0]))}—${fmtTime(ut(map.centralContacts[1]))} UT`}。
                ΔT 只影响经度：滑块平移全图而不重算。
              </Text>
            </Flex>
          </Card>
        )}
        {picked && map && (
          <Card size="small" style={{ marginTop: 12 }}
            title={`${pickedCapital ? `${pickedCapital.name}（${pickedCapital.regimes.join('、')}）` : '所选地点'} ${picked[0].toFixed(2)}°${picked[0] >= 0 ? 'E' : 'W'} ${Math.abs(picked[1]).toFixed(2)}°${picked[1] >= 0 ? 'N' : 'S'}，ΔT ${deltaT.toFixed(0)} s`}>
            {local === undefined ? <Spin size="small" /> : local === null ? <Text>该地无食。</Text> : (
              <Descriptions size="small" column={{ xs: 1, sm: 2, md: 3 }} bordered>
                <Descriptions.Item label="初亏">{fmtTime(ut(local.first), true)} UT · 地方平时 {fmtTime(lmt(local.first))} · 日高 {local.altFirst.toFixed(1)}°</Descriptions.Item>
                {local.second != null && <Descriptions.Item label="食既">{fmtTime(ut(local.second), true)} UT</Descriptions.Item>}
                <Descriptions.Item label="食甚">{fmtTime(ut(local.greatest), true)} UT · 地方平时 {fmtTime(lmt(local.greatest))} · 日高 {local.altGreatest.toFixed(1)}°</Descriptions.Item>
                {local.third != null && <Descriptions.Item label="生光">{fmtTime(ut(local.third), true)} UT</Descriptions.Item>}
                <Descriptions.Item label="复圆">{fmtTime(ut(local.last), true)} UT · 地方平时 {fmtTime(lmt(local.last))} · 日高 {local.altLast.toFixed(1)}°</Descriptions.Item>
                <Descriptions.Item label="食分">{local.magnitude.toFixed(3)}{local.second != null ? (local.magnitude >= 1 ? '（全食）' : '（环食）') : ''}</Descriptions.Item>
                <Descriptions.Item label="可见">
                  {local.visibleFrom == null ? '全程在地平下' : `${fmtTime(lmt(local.visibleFrom))}—${fmtTime(lmt(local.visibleTo!))} 地方平时，可见最大食分 ${local.maxVisibleMagnitude?.toFixed(3)}`}
                </Descriptions.Item>
              </Descriptions>
            )}
          </Card>
        )}
      </Spin>
    </Flex>
  )
}
