/**
 * 月食图：月食时刻全球同一，地图只回答何处月亮在地平上。画各食相的月出月没界线、月下点，把「食始时月在地平下」「食终时月在地平下」
 * 两块区域着色（即月出带食、月没带食，两者重叠处全程不可见，均未着色处全程可见）；另画月亮穿过地影图。点击地图或都城看该地食况。
 *
 * 计算全在 Rust（eclipseWorker → EclipseEngine）。界线为站心几何高度 0（含月亮视差，不计折射）；某地「可见」按标准地平高度 −0°50′。
 * ΔT 只影响经度：滑块平移全图，某地食况按当前 ΔT 重算。
 */
import { useEffect, useMemo, useState } from 'react'
import { Card, Checkbox, Descriptions, Flex, Radio, Slider, Space, Spin, Typography } from 'antd'
import { geoContains } from 'd3-geo'
import type { Feature, MultiLineString, Polygon } from 'geojson'
import { fmtDate, fmtTime } from './civilTime'
import type { Capital } from './capitals'
import { GeoCanvas, type MapView } from './GeoCanvas'
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
function belowHorizon(c: Curve, dLon: number): Feature<Polygon> {
  const ring = c.horizon.map((p) => [p[0] + dLon, p[1]])
  const s = c.sublunar[0]
  const anti: [number, number] = [s[0] + dLon + 180, -s[1]]
  const make = (r: number[][]): Feature<Polygon> => ({ type: 'Feature', properties: {}, geometry: { type: 'Polygon', coordinates: [r] } })
  const f = make(ring)
  return geoContains(f, anti) ? f : make([...ring].reverse())
}

function ShadowDiagram({ map, ut }: { map: LunarMap; ut: (jd: number) => number }) {
  const size = 360
  const c = size / 2
  const extent = Math.max(map.penumbraRadius + map.moonSemidiameter * 2, ...map.curves.map((x) => Math.hypot(x.east, x.north) + map.moonSemidiameter)) * 1.04
  const k = (c - 6) / extent
  // 天空视角：北上东左
  const X = (east: number) => c - east * k
  const Y = (north: number) => c - north * k
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
    </svg>
      <Flex vertical gap={4}>
        {map.curves.map((x, i) => (
          <Text key={x.phase} style={{ color: PHASE_COLOR[x.phase] }}>{i + 1}. {PHASE_LABEL[x.phase]} {fmtTime(ut(x.jdTt), true)} UT</Text>
        ))}
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
            食甚 {fmtDate(ut(map.eclipse.greatestTt))} {fmtTime(ut(map.eclipse.greatestTt), true)} UT，γ = {map.eclipse.gamma.toFixed(4)}，本影食分 {map.eclipse.umbralMagnitude.toFixed(4)}，半影食分 {map.eclipse.penumbralMagnitude.toFixed(4)}。
          </Text>
        </Flex>
      </Card>
      <Card size="small" style={{ marginTop: 12 }} title="月亮穿过地影（北上东左；地影按食甚时大小）">
        <ShadowDiagram map={map} ut={ut} />
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
                  : `${fmtTime(lmt(local.visibleFrom))}—${fmtTime(lmt(local.visibleTo!))} 地方平时，可见最大本影食分 ${local.maxVisibleUmbral?.toFixed(3)}、半影食分 ${local.maxVisiblePenumbral?.toFixed(3)}`}
              </Descriptions.Item>
            </Descriptions>
          )}
        </Card>
      )}
    </>
  )
}
