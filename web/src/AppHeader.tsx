/**
 * 常驻顶栏：页面导航 + 时空条（时刻步进、实时、地点）。设计见 doc/engineering/ui-interaction.md。
 * 宽屏：一行导航 + 一行时空条；窄屏（< 768）：顶部只留时空条，导航改为底部标签栏。
 */
import { useEffect, useState, type ReactNode } from 'react'
import { Button, Dropdown, Flex, InputNumber, Popover, Select, Switch, Typography } from 'antd'
import {
  AimOutlined,
  ApartmentOutlined,
  AppstoreOutlined,
  CalendarOutlined,
  CompassOutlined,
  DoubleLeftOutlined,
  DoubleRightOutlined,
  EllipsisOutlined,
  EnvironmentOutlined,
  GlobalOutlined,
  LeftOutlined,
  ReadOutlined,
  RightOutlined,
  StarOutlined,
} from '@ant-design/icons'

const { Text } = Typography

export type ViewKey = 'calendar' | 'graph' | 'eclipse' | 'stars' | 'zeri' | 'paipan'

/** 主导航（按用户路径排：先看日子与天象，再择日、排盘）；变换图属工程工具，放「更多」 */
const PRIMARY: { key: ViewKey; label: string; icon: ReactNode }[] = [
  { key: 'calendar', label: '日历', icon: <CalendarOutlined /> },
  { key: 'stars', label: '星图', icon: <StarOutlined /> },
  { key: 'eclipse', label: '日月食', icon: <GlobalOutlined /> },
  { key: 'zeri', label: '择日', icon: <CompassOutlined /> },
  { key: 'paipan', label: '排盘', icon: <AppstoreOutlined /> },
]

export function useIsNarrow(px = 768): boolean {
  const q = `(max-width: ${px - 1}px)`
  const [narrow, setNarrow] = useState(() => typeof window !== 'undefined' && window.matchMedia(q).matches)
  useEffect(() => {
    const m = window.matchMedia(q)
    const on = () => setNarrow(m.matches)
    m.addEventListener('change', on)
    return () => m.removeEventListener('change', on)
  }, [q])
  return narrow
}

/** 一组步进键：如「日」「时辰」 */
export type Step = { label: string; prev: () => void; next: () => void }

export type DateTimeValue = { year: number; month: number; day: number; hour: number; minute: number }

export type HeaderProps = {
  view: ViewKey
  onView: (v: ViewKey) => void
  /** 本页是否用全局时空（星图、日月食、变换图暂不用） */
  showTime: boolean
  /** 主标签：公历日期时刻 */
  title: string
  /** 窄屏用的短标签 */
  titleShort?: string
  /** 副标签：农历、干支 */
  subtitle?: string
  steps: Step[]
  isLive: boolean
  onLive: (on: boolean) => void
  /** 当前所选（民用历）时刻，供选择器初值 */
  value: DateTimeValue
  onJump: (v: DateTimeValue) => void
  latitude: number
  longitude: number
  declination: number | null
  onPlace: (lat: number, lon: number) => void
  onLocate: () => Promise<string>
}

function docsHref() {
  return `${import.meta.env.BASE_URL}docs/`
}

function MoreMenu(props: { onView: (v: ViewKey) => void; children: ReactNode }) {
  return (
    <Dropdown
      trigger={['click']}
      menu={{
        items: [
          { key: 'docs', icon: <ReadOutlined />, label: <a href={docsHref()}>文档</a> },
          { key: 'graph', icon: <ApartmentOutlined />, label: '视位置变换图（开发）', onClick: () => props.onView('graph') },
        ],
      }}
    >
      {props.children}
    </Dropdown>
  )
}

/** 时刻选择：年（天文纪年，可负）、月、日、时、分；民用历（1582-10-15 前儒略历） */
function DateTimePicker(props: { value: DateTimeValue; onJump: (v: DateTimeValue) => void; onToday: () => void; close: () => void }) {
  const [v, setV] = useState(props.value)
  useEffect(() => setV(props.value), [props.value])
  const set = (k: keyof DateTimeValue) => (x: number | null) => x != null && setV((p) => ({ ...p, [k]: x }))
  return (
    <Flex vertical gap={8} style={{ width: 260 }}>
      <Flex gap={6} align="center" wrap>
        <InputNumber size="small" value={v.year} onChange={set('year')} style={{ width: 84 }} />
        <span>年</span>
        <Select size="small" value={v.month} onChange={(m) => setV((p) => ({ ...p, month: m }))} style={{ width: 64 }}
          options={Array.from({ length: 12 }, (_, i) => ({ value: i + 1, label: `${i + 1}` }))} />
        <span>月</span>
        <InputNumber size="small" min={1} max={31} value={v.day} onChange={set('day')} style={{ width: 56 }} />
        <span>日</span>
      </Flex>
      <Flex gap={6} align="center">
        <InputNumber size="small" min={0} max={23} value={v.hour} onChange={set('hour')} style={{ width: 56 }} />
        <span>:</span>
        <InputNumber size="small" min={0} max={59} value={v.minute} onChange={set('minute')} style={{ width: 56 }} />
      </Flex>
      <Text type="secondary" style={{ fontSize: 11 }}>天文纪年（0 为公元前 1 年）；1582-10-15 前为儒略历</Text>
      <Flex gap={8} justify="flex-end">
        <Button size="small" onClick={() => { props.onToday(); props.close() }}>回到此刻</Button>
        <Button size="small" type="primary" onClick={() => { props.onJump(v); props.close() }}>跳转</Button>
      </Flex>
    </Flex>
  )
}

function PlacePicker(props: Pick<HeaderProps, 'latitude' | 'longitude' | 'declination' | 'onPlace' | 'onLocate'> & { close: () => void }) {
  const [lat, setLat] = useState(props.latitude)
  const [lon, setLon] = useState(props.longitude)
  const [msg, setMsg] = useState('')
  useEffect(() => { setLat(props.latitude); setLon(props.longitude) }, [props.latitude, props.longitude])
  return (
    <Flex vertical gap={8} style={{ width: 250 }}>
      <Button size="small" icon={<AimOutlined />} onClick={() => { setMsg('定位中…'); props.onLocate().then(setMsg, (e) => setMsg(String(e?.message ?? e))) }}>
        用设备定位
      </Button>
      {msg && <Text type="secondary" style={{ fontSize: 12 }}>{msg}</Text>}
      <Flex gap={6} align="center">
        <span>纬</span>
        <InputNumber size="small" min={-90} max={90} step={0.01} value={lat} onChange={(x) => x != null && setLat(x)} style={{ width: 90 }} />
        <span>经</span>
        <InputNumber size="small" min={-180} max={180} step={0.01} value={lon} onChange={(x) => x != null && setLon(x)} style={{ width: 90 }} />
      </Flex>
      <Text type="secondary" style={{ fontSize: 11 }}>北纬、东经为正。时柱当地时、更点、日出入、罗盘磁偏角、星图共用此地</Text>
      {props.declination != null && <Text type="secondary" style={{ fontSize: 11 }}>{`磁偏角 ${props.declination >= 0 ? '东' : '西'}${Math.abs(props.declination).toFixed(1)}°（WMM）`}</Text>}
      <Flex justify="flex-end">
        <Button size="small" type="primary" onClick={() => { props.onPlace(lat, lon); props.close() }}>设为此地</Button>
      </Flex>
    </Flex>
  )
}

export function placeLabel(lat: number, lon: number): string {
  return `${lat >= 0 ? '北' : '南'}${Math.abs(lat).toFixed(2)}° ${lon >= 0 ? '东' : '西'}${Math.abs(lon).toFixed(2)}°`
}

function TimeBar(props: HeaderProps & { narrow: boolean }) {
  const [timeOpen, setTimeOpen] = useState(false)
  const [placeOpen, setPlaceOpen] = useState(false)
  const outer = props.steps[0]
  const inner = props.steps[1]
  // 窄屏不写字：外层（大步，如月、日）用双箭头，内层（小步，如日、时辰）用单箭头
  const stepBtn = (s: Step | undefined, dir: -1 | 1, big: boolean) =>
    s && (
      <Button size="small" type="text" onClick={dir < 0 ? s.prev : s.next} title={`${dir < 0 ? '前' : '后'}一${s.label}`} aria-label={`${dir < 0 ? '前' : '后'}一${s.label}`}
        style={props.narrow ? { paddingInline: 4 } : undefined}>
        {props.narrow
          ? (dir < 0 ? (big ? <DoubleLeftOutlined /> : <LeftOutlined />) : (big ? <DoubleRightOutlined /> : <RightOutlined />))
          : (dir < 0 ? <><LeftOutlined />{s.label}</> : <>{s.label}<RightOutlined /></>)}
      </Button>
    )
  const big = props.steps.length > 1
  return (
    <Flex align="center" gap={4} wrap={!props.narrow} style={{ minWidth: 0 }}>
      {stepBtn(outer, -1, big)}
      {stepBtn(inner, -1, false)}
      <Popover
        open={timeOpen}
        onOpenChange={setTimeOpen}
        trigger="click"
        content={<DateTimePicker value={props.value} onJump={props.onJump} onToday={() => props.onLive(true)} close={() => setTimeOpen(false)} />}
      >
        <button type="button" className="hdr-time" title="点选改日期时刻">
          <span className="hdr-time-main">{props.narrow && props.titleShort ? props.titleShort : props.title}</span>
          {props.subtitle && <span className="hdr-time-sub">{props.subtitle}</span>}
        </button>
      </Popover>
      {stepBtn(inner, 1, false)}
      {stepBtn(outer, 1, big)}
      {props.narrow
        ? <Switch size="small" checked={props.isLive} onChange={props.onLive} checkedChildren="实时" unCheckedChildren="实时" style={{ flexShrink: 0 }} />
        : (
          <Flex align="center" gap={4} style={{ marginLeft: 4 }}>
            <Switch size="small" checked={props.isLive} onChange={props.onLive} />
            <Text style={{ fontSize: 12 }}>实时</Text>
          </Flex>
        )}
      <Popover
        open={placeOpen}
        onOpenChange={setPlaceOpen}
        trigger="click"
        content={<PlacePicker {...props} close={() => setPlaceOpen(false)} />}
      >
        <Button size="small" type="text" icon={<EnvironmentOutlined />} style={{ marginLeft: 4 }}>
          {!props.narrow && placeLabel(props.latitude, props.longitude)}
        </Button>
      </Popover>
    </Flex>
  )
}

export function AppHeader(props: HeaderProps) {
  const narrow = useIsNarrow()
  if (narrow) {
    return (
      <>
        {props.showTime && (
          <div className="hdr hdr-narrow">
            <TimeBar {...props} narrow />
          </div>
        )}
        <nav className="bottom-nav">
          {PRIMARY.map((it) => (
            <button key={it.key} type="button" className={props.view === it.key ? 'active' : ''} onClick={() => props.onView(it.key)}>
              {it.icon}
              <span>{it.label}</span>
            </button>
          ))}
          <MoreMenu onView={props.onView}>
            <button type="button" className={props.view === 'graph' ? 'active' : ''}>
              <EllipsisOutlined />
              <span>更多</span>
            </button>
          </MoreMenu>
        </nav>
      </>
    )
  }
  return (
    <div className="hdr">
      <Flex align="center" gap={4} wrap>
        {PRIMARY.map((it) => (
          <Button key={it.key} type={props.view === it.key ? 'primary' : 'text'} icon={it.icon} onClick={() => props.onView(it.key)}>
            {it.label}
          </Button>
        ))}
        <Button type="text" icon={<ReadOutlined />} href={docsHref()}>文档</Button>
        <MoreMenu onView={props.onView}>
          <Button type={props.view === 'graph' ? 'primary' : 'text'} icon={<EllipsisOutlined />} aria-label="更多" />
        </MoreMenu>
      </Flex>
      {props.showTime && (
        <div style={{ marginTop: 8 }}>
          <TimeBar {...props} narrow={false} />
        </div>
      )}
    </div>
  )
}
