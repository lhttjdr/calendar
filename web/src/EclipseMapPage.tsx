/**
 * 日月食图：按年列出日食、月食；月食的画法在 LunarEclipseView。日食（食带图）选中一次后画中心线、本影/半影南北界、带食出没界线、影子轮廓与昼夜界线。
 * 可切换「地球」（正射投影，可拖动旋转，缺省从太阳方向看）与「平面」（等距圆柱）；点击地图或都城给出该地食况。
 *
 * 计算全在 Rust（eclipseWorker → EclipseEngine），这里只画。ΔT 只经地球自转进入，换 ΔT 时全图严格沿经度平移
 * （lonShiftPerSecond），所以 ΔT 滑块不重算，只平移；某地食况按当前 ΔT 重算。
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Alert, Button, Card, Checkbox, Descriptions, Flex, InputNumber, Radio, Slider, Space, Spin, Table, Tag, Typography } from 'antd'
import { geoCircle } from 'd3-geo'
import type { MultiLineString } from 'geojson'
import { GeoCanvas, type MapView as View } from './GeoCanvas'
import { LunarEclipseView, type LunarListItem } from './LunarEclipseView'
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
const LUNAR_KIND_LABEL: Record<string, string> = { T: '月全食', P: '月偏食', N: '半影月食' }
const LUNAR_KIND_COLOR: Record<string, string> = { T: 'magenta', P: 'blue', N: 'default' }
type AnyItem = (ListItem & { body: 'solar' }) | (LunarListItem & { body: 'lunar' })

// ---------------------------------------------------------------------------------------------------------------------
// 绘图

const COLORS = {
  night: 'rgba(20,30,60,0.22)',
  penumbraLimit: '#3b6fb6',
  umbraLimit: '#c0392b',
  central: '#8e1b10',
  riseSet: '#2e8b57',
  outlinePen: 'rgba(59,111,182,0.55)',
  outlineUmbra: 'rgba(192,57,43,0.75)',
  sigma: 'rgba(142,27,16,0.45)',
  magnitude: '#7a5c99',
}

function shiftLines(lines: Line[], dLon: number): MultiLineString {
  return { type: 'MultiLineString', coordinates: lines.map((l) => l.map((p) => [p[0] + dLon, p[1]])) }
}

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
  const sub = map.subsolar[0]
  const gp = map.greatestPoint[0]
  // 半径取 89.99° 而非 90°：从太阳方向看时夜半球边界恰在正射投影裁剪圈上，裁剪退化会填满整个圆盘
  const night = useMemo(() => geoCircle().center([sub[0] + dLon + 180, -sub[1]]).radius(89.99)(), [sub, dLon])

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

  return (
    <GeoCanvas
      view={view}
      resetKey={map}
      globeCenter={[sub[0] + dLon, sub[1]]}
      flatLon={gp[0] + dLon}
      capitals={layers.has('capitals') ? capitals : []}
      picked={picked}
      onPick={onPick}
      overlay={({ path, pt }) => {
        const line = (lines: Line[], stroke: string, w: number, dash?: string) => (
          <path d={path(shiftLines(lines, dLon)) ?? ''} fill="none" stroke={stroke} strokeWidth={w} strokeDasharray={dash} />
        )
        return (
          <>
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
                    style={{ paintOrder: 'stroke', stroke: '#f4f1e8', strokeWidth: 2.5 }}>{c.level.toFixed(1)}</text>
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
          </>
        )
      }}
    />
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
  const [list, setList] = useState<AnyItem[] | null>(null)
  const [selected, setSelected] = useState<AnyItem | null>(null)
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
      setBusy('计算日月食表')
      setError(null)
      try {
        const [sol, lun] = await Promise.all([call<ListItem[]>({ type: 'list', year: y }), call<LunarListItem[]>({ type: 'listLunar', year: y })])
        const r: AnyItem[] = [
          ...sol.map((x) => ({ ...x, body: 'solar' as const })),
          ...lun.map((x) => ({ ...x, body: 'lunar' as const })),
        ].sort((a, b) => a.greatestTt - b.greatestTt)
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
    if (!selected || selected.body === 'lunar') {
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
    if (!picked || !selected || selected.body === 'lunar') return
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
          <Table<AnyItem>
            size="small"
            pagination={false}
            rowKey={(r) => String(r.greatestTt)}
            dataSource={list}
            locale={{ emptyText: `${yearLabel(year)} 年无日月食` }}
            rowSelection={{
              type: 'radio',
              selectedRowKeys: selected ? [String(selected.greatestTt)] : [],
              onChange: (_, rows) => setSelected(rows[0] ?? null),
            }}
            onRow={(r) => ({ onClick: () => setSelected(r) })}
            columns={[
              { title: '日期', render: (_, r) => fmtDate(r.greatestTt - r.deltaT / 86400) },
              { title: '食甚 UT', render: (_, r) => fmtTime(r.greatestTt - r.deltaT / 86400) },
              { title: '类型', render: (_, r) => r.body === 'lunar'
                ? <Tag color={LUNAR_KIND_COLOR[r.kind]}>{LUNAR_KIND_LABEL[r.kind]}</Tag>
                : <Tag color={KIND_COLOR[r.kind]}>日{KIND_LABEL[r.kind]}{r.central ? '' : '（非中心）'}</Tag> },
              { title: 'γ', render: (_, r) => r.gamma.toFixed(4) },
              { title: '食分', render: (_, r) => r.body === 'lunar' ? `本影 ${r.umbralMagnitude.toFixed(4)} · 半影 ${r.penumbralMagnitude.toFixed(4)}` : r.magnitude.toFixed(4) },
              { title: 'ΔT（s）', render: (_, r) => `${r.deltaT.toFixed(0)} ± ${r.deltaTSigma.toFixed(0)}` },
            ]}
          />
        )}
        {selected?.body === 'lunar' && (
          <LunarEclipseView key={selected.greatestTt} item={selected} call={call} capitals={capitals} onBusy={setBusy} onError={setError} />
        )}
        {map && selected?.body === 'solar' && (
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
                    marks={sigma >= 5 ? { [-2 * sigma]: '−2σ', [-sigma]: '−σ', 0: '0', [sigma]: '+σ', [2 * sigma]: '+2σ' } : undefined}
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
        {picked && map && selected?.body === 'solar' && (
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
