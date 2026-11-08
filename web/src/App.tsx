import { useState, useEffect, useMemo, useRef, createContext, useContext } from 'react'
import { Button, Card, Select, Radio, Checkbox, Space, Flex, Typography, Tag, Spin, Alert, Modal, Tabs, InputNumber } from 'antd'
import { LeftOutlined, RightOutlined, CalendarOutlined, SettingOutlined, ApartmentOutlined, EyeOutlined, GlobalOutlined, StarOutlined, ReadOutlined } from '@ant-design/icons'
import { TransformGraphPage } from './TransformGraphPage'
import { EclipseMapPage } from './EclipseMapPage'
import { StarChartPage } from './StarChartPage'
import type { LunarBackend } from './lunar-backend-types'
import {
  getYearDataForDisplay,
  formatLunarWithFallback,
  applyLunarDisplayLayers,
  getGanzhiForDay,
  getGanzhiForMonth,
  getLunarSlotForDay,
  loadWasm,
  ensureVsop87Cached,
  isVsop87Cached,
  getCachedVsop87Ear,
  GANZHI_PRESET_OPTIONS,
  GANZHI_PRESET_NAMES,
  getPresetIndexFromOptions,
  getLunarYearsForDisplay,
  GANZHI_YEAR_BOUNDARY_OPTIONS,
  GANZHI_MONTH_BOUNDARY_OPTIONS,
  GANZHI_LEAP_MONTH_OPTIONS,
  GANZHI_DAY_BOUNDARY_OPTIONS,
  GANZHI_SOLAR_TERM_MODE_OPTIONS,
  HOUR_CLOCK_OPTIONS,
  getLocalSolarHours,
  LUNAR_MONTH_NAMES,
  lunarDayStr,
  type YearData,
  type GanzhiResult,
  type GanzhiOptionsTuple,
  type RealBackendVariant,
} from './useLunarWasm'
import { utcFromLocalInZone, civilMonthDays, convertDisplayDateTime, displayCivilToJd, formatDateInTimeZone, civilToProleptic, clampCivilDay, displayCalendarName, isCalendarMode, isFullyGregorianMonth, jdToDisplayDate, localDate, ROMAN_SCHEMES, setCalendarMode, type CalendarMode } from './civilTime'
import { EPHEMERIS_SOURCE_LABELS, EPHEMERIS_FORMAT_LABELS, type EphemerisDataSource } from './ephemerisSourceRegistry'
import { EARTH_PATCH_TARGET } from './ephemerisUrls'

const REAL_BACKEND_STORAGE_KEY = 'lunar-real-backend'
const CALENDAR_MODE_STORAGE_KEY = 'lunar-calendar-mode'

function getStoredCalendarMode(): CalendarMode {
  try {
    const v = localStorage.getItem(CALENDAR_MODE_STORAGE_KEY)
    if (isCalendarMode(v)) return v
  } catch {}
  return 'civil'
}

const CALENDAR_MODE_OPTIONS = [
  { label: '通用（1582 年前儒略历，默认）', value: 'civil' },
  { label: 'ISO 8601（前推格里高利历）', value: 'iso' },
  {
    label: '罗马实历（公元前 45 年起的实际闰年）',
    options: Object.entries(ROMAN_SCHEMES).map(([value, s]) => ({ label: s.label, value })),
  },
]

function getStoredRealBackend(): RealBackendVariant {
  try {
    const v = localStorage.getItem(REAL_BACKEND_STORAGE_KEY)
    if (v === 'f64' || v === 'twofloat') return v
  } catch {}
  return 'f64'
}

/**
 * 单日信息卡片按「三大域四段结构」（见 doc/architecture/three-domains.md）分组的可选显示项。
 * 农历年月日本身是骨架信息，不做可关闭项；这里只管天文/历法的事件层、派生层输出，
 * 以及工程性质的仲裁层状态（Repo 历表加载情况），彼此独立开关、各自持久化到 localStorage。
 */
type DayInfoLayerKey =
  | 'astro_synodic' // 天文·事件层：合朔时刻
  | 'astro_solarTerm' // 天文·事件层：节气交节（含平气对比）
  | 'calendar_eraNames' // 历法·派生层：政权·帝王·年号（含并存政权）
  | 'calendar_regimeCalendars' // 历法·独立基础层：其它政权推步历（蜀汉四分、东吴乾象等）
  | 'calendar_ganzhi' // 历法：干支历四柱
  | 'folk_jishi' // 民俗·独立基础层：纳音、值日宿与七曜、年月紫白九星
  | 'repoStatus' // 工程：天文仲裁层状态（历表/章动表/地球改正加载情况）

const DAY_INFO_LAYERS_STORAGE_KEY = 'lunar-day-info-layers'

const DAY_INFO_LAYER_GROUPS: { title: string; items: { key: DayInfoLayerKey; label: string }[] }[] = [
  {
    title: '天文（事件层）',
    items: [
      { key: 'astro_synodic', label: '合朔时刻' },
      { key: 'astro_solarTerm', label: '节气交节' },
    ],
  },
  {
    title: '历法',
    items: [
      { key: 'calendar_eraNames', label: '政权·年号（含并存政权）' },
      { key: 'calendar_regimeCalendars', label: '其它政权推步历（蜀汉/东吴等）' },
      { key: 'calendar_ganzhi', label: '干支历四柱' },
    ],
  },
  {
    title: '民俗',
    items: [{ key: 'folk_jishi', label: '纳音·值日宿·九星·建除黄道·神煞·杂节气·更点' }],
  },
  {
    title: '工程（仲裁层状态）',
    items: [{ key: 'repoStatus', label: 'Repo/历表加载状态' }],
  },
]

const DEFAULT_DAY_INFO_LAYERS: Record<DayInfoLayerKey, boolean> = {
  astro_synodic: true,
  astro_solarTerm: true,
  calendar_eraNames: true,
  calendar_regimeCalendars: true,
  calendar_ganzhi: true,
  folk_jishi: true,
  repoStatus: true,
}

function getStoredDayInfoLayers(): Record<DayInfoLayerKey, boolean> {
  try {
    const raw = localStorage.getItem(DAY_INFO_LAYERS_STORAGE_KEY)
    if (raw) return { ...DEFAULT_DAY_INFO_LAYERS, ...(JSON.parse(raw) as Partial<Record<DayInfoLayerKey, boolean>>) }
  } catch {}
  return DEFAULT_DAY_INFO_LAYERS
}

const { Text } = Typography

/** 实时时间 Context：仅在 enabled 时每秒更新，供各日期行的时刻联动 */
const LiveTimeContext = createContext<Date | null>(null)
function LiveTimeProvider({ enabled, children }: { enabled: boolean; children: React.ReactNode }) {
  const [t, setT] = useState(() => new Date())
  useEffect(() => {
    if (!enabled) return
    setT(new Date())
    const id = setInterval(() => setT(new Date()), 1000)
    return () => clearInterval(id)
  }, [enabled])
  return <LiveTimeContext.Provider value={enabled ? t : null}>{children}</LiveTimeContext.Provider>
}
function useLiveTime(): Date | null {
  return useContext(LiveTimeContext)
}

/** 日期行末尾的时刻：实时模式下随 LiveTimeProvider 每秒更新，写法由 format 决定 */
function LineTime({ format, live, timeZone, hour, minute, second }: {
  format: (hour: number, minute: number, second: number) => string
  live: boolean
  timeZone: string
  hour: number
  minute: number
  second: number
}) {
  const t = useLiveTime()
  const p = live && t != null ? formatDateInTimeZone(t, timeZone) : { hour, minute, second }
  const s = format(p.hour, p.minute, p.second)
  return s ? <span style={{ fontVariantNumeric: 'tabular-nums' }}> {s}</span> : null
}

/** 岁数据中 12 中气顺序：zhongQiJds[0]=冬至,…,zhongQiJds[11]=小雪 */
const ZHONG_QI_NAMES = ['冬至', '大寒', '雨水', '春分', '谷雨', '小满', '夏至', '大暑', '处暑', '秋分', '霜降', '小雪']

const JD_UNIX_EPOCH = 2440587.5
function jdUtcToDate(jdUtc: number): Date {
  return new Date((jdUtc - JD_UNIX_EPOCH) * 86400 * 1000)
}
function beijingDayFromJdUtc(jdUtc: number): { y: number; m: number; d: number } {
  const p = formatDateInTimeZone(jdUtcToDate(jdUtc), 'Asia/Shanghai')
  return { y: p.year, m: p.month, d: p.day }
}
function formatJdUtcInZone(jdUtc: number, timeZone: string): string {
  const d = jdUtcToDate(jdUtc)
  const p = formatDateInTimeZone(d, timeZone)
  return `${p.hour.toString().padStart(2, '0')}:${p.minute.toString().padStart(2, '0')}`
}
/** 定气 JD(TT) → JD(UTC)；需 wasm.delta_t_seconds。 */
function jdTtToUtc(wasm: LunarBackend, jdTt: number): number {
  const dt = wasm.delta_t_seconds?.(jdTt) ?? 0
  return jdTt - dt / 86400
}
const PING_QI_DIFF_THRESHOLD_DAYS = 0.01

/** 常用时区；若环境支持则用 Intl.supportedValuesOf('timeZone') 提供完整列表 */
const COMMON_TIMEZONES = [
  'Asia/Shanghai',
  'Asia/Hong_Kong',
  'Asia/Taipei',
  'Asia/Tokyo',
  'Asia/Seoul',
  'Europe/London',
  'Europe/Paris',
  'America/New_York',
  'America/Los_Angeles',
  'UTC',
]
function getTimeZoneOptions(): { label: string; value: string }[] {
  try {
    if (typeof Intl.supportedValuesOf === 'function') {
      const all = Intl.supportedValuesOf('timeZone') as string[]
      const set = new Set([...COMMON_TIMEZONES, ...all])
      return Array.from(set).sort().map((z) => ({ label: z, value: z }))
    }
  } catch (_) {}
  return COMMON_TIMEZONES.map((z) => ({ label: z, value: z }))
}

const WEEKDAY_NAMES = ['日', '一', '二', '三', '四', '五', '六']
const GAN = '甲乙丙丁戊己庚辛壬癸'
const ZHI = '子丑寅卯辰巳午未申酉戌亥'
const ZODIAC_BY_BRANCH: Record<string, string> = {
  '子': '鼠', '丑': '牛', '寅': '虎', '卯': '兔', '辰': '龙', '巳': '蛇',
  '午': '马', '未': '羊', '申': '猴', '酉': '鸡', '戌': '狗', '亥': '猪',
}

/** 根据日干支与当前小时计算时干支（日上起时） */
function getHourGanzhi(dayGanzhi: string, hour: number): string {
  if (dayGanzhi.length < 2) return ''
  const dayGanIndex = GAN.indexOf(dayGanzhi[0]!)
  const dayZhiIndex = ZHI.indexOf(dayGanzhi[1]!)
  if (dayGanIndex < 0 || dayZhiIndex < 0) return ''
  const zhiIndex = hour === 23 ? 0 : Math.floor((hour + 1) / 2)
  // 23 时已入次日子时，无论日柱是否换日，时干都按次日起（甲日 23 时为丙子）
  const ganIndex = ((dayGanIndex + (hour === 23 ? 1 : 0)) * 2 + zhiIndex) % 10
  return GAN[ganIndex]! + ZHI[zhiIndex]!
}

/** 年份转汉字，数字0用〇（如 2026→二〇二六） */
function yearToChinese(n: number): string {
  const digits = '〇一二三四五六七八九'
  return String(n).replace(/[0-9]/g, (d) => digits[parseInt(d, 10)]!)
}

function getWeekday(y: number, m: number, d: number): number {
  const a = Math.floor((14 - m) / 12)
  const y2 = y + 4800 - a
  const m2 = m + 12 * a - 3
  const jd = d + Math.floor((153 * m2 + 2) / 5) + 365 * y2 + Math.floor(y2 / 4) - Math.floor(y2 / 100) + Math.floor(y2 / 400) - 32045
  return (jd + 1) % 7
}

/** 民用历一个月可能跨前推格里历的两个月，两边的岁数据都要。 */
async function getYearDataForCivilMonth(
  year: number,
  month: number,
  backend: LunarBackend,
  onSourceUsed: (source: EphemerisDataSource) => void,
): Promise<YearData[]> {
  const days = civilMonthDays(year, month)
  const first = civilToProleptic(year, month, days[0]!)
  const last = civilToProleptic(year, month, days[days.length - 1]!)
  const lists = await Promise.all([
    getYearDataForDisplay(first.year, first.month, backend, onSourceUsed),
    getYearDataForDisplay(last.year, last.month, backend, onSourceUsed),
  ])
  return [...new Map(lists.flat().map((d) => [d.lunarYear, d])).values()]
}

/** 按该前推格里历月份的主次顺序排岁数据，主岁在前。 */
function orderedYearData(list: YearData[], year: number, month: number): YearData[] {
  return getLunarYearsForDisplay(year, month)
    .map((ly) => list.find((d) => d.lunarYear === ly))
    .filter((d): d is YearData => d != null)
}

function getDayOfYear(y: number, m: number, d: number): number {
  return Math.round(displayCivilToJd(y, m, d) - displayCivilToJd(y, 1, 1)) + 1
}

function getWeekOfYear(y: number, m: number, d: number): number {
  const dow = (Math.floor(displayCivilToJd(y, 1, 1) + 0.5) + 1) % 7
  const mon1 = (dow === 0 ? -5 : 2 - dow)
  const doy = getDayOfYear(y, m, d)
  return Math.floor((doy - mon1) / 7) + 1
}

function App() {
  const [timeZone, setTimeZone] = useState(() => Intl.DateTimeFormat().resolvedOptions().timeZone)
  const today = useMemo(() => {
    const t = formatDateInTimeZone(new Date(), timeZone)
    return { year: t.year, month: t.month, day: t.day }
  }, [timeZone])

  const [calendarMode, setCalendarModeState] = useState<CalendarMode>(() => {
    const m = getStoredCalendarMode()
    setCalendarMode(m)
    return m
  })
  const [year, setYear] = useState(today.year)
  const [month, setMonth] = useState(today.month)
  const [selectedDate, setSelectedDate] = useState<{ year: number; month: number; day: number }>(today)
  const [selectedTime, setSelectedTime] = useState(() => {
    const n = new Date()
    return { hour: n.getHours(), minute: n.getMinutes(), second: n.getSeconds() }
  })
  const [isLiveMode, setIsLiveMode] = useState(true)
  const [yearDataList, setYearDataList] = useState<YearData[]>([])
  /** 左侧面板「显示日期」所在月的岁数据；当显示日期与日历视图同月时用 yearDataList，否则单独拉取 */
  const [displayDateYearDataList, setDisplayDateYearDataList] = useState<YearData[]>([])
  const [wasm, setWasm] = useState<LunarBackend | null>(null)
  const [loading, setLoading] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [ganzhiOptions, setGanzhiOptions] = useState<GanzhiOptionsTuple>(() => GANZHI_PRESET_OPTIONS[3]!)
  const [optionsPanelOpen, setOptionsPanelOpen] = useState(false)
  /** 时柱所用钟点：0=时区钟点 1=当地平太阳时 2=当地真太阳时；经度为东经度数 */
  const [hourClockMode, setHourClockMode] = useState(0)
  const [observerLongitude, setObserverLongitude] = useState(116.4)
  /** 更点：观测地纬度（北纬为正）与方法 0=清台官法 1=元授时法 */
  const [observerLatitude, setObserverLatitude] = useState(39.92)
  const [gengdianMethod, setGengdianMethod] = useState(0)
  const [displayLayersPanelOpen, setDisplayLayersPanelOpen] = useState(false)
  /** 仅小时变化时更新，用于年月日时与农历/八字；不每秒更新避免整页重算 */
  const [nowForLunar, setNowForLunar] = useState(() => new Date())
  const lastMinuteKeyRef = useRef<string>('')
  const [vsop87Tick, setVsop87Tick] = useState(0)
  /** 一周起始日：0=周日 1=周一 … 6=周六 */
  const [weekStart, setWeekStart] = useState(1)
  /** 当前视图：日历 | 视位置变换图 */
  const [view, setView] = useState<'calendar' | 'graph' | 'eclipse' | 'stars'>('calendar')
  /** Real 后端：twofloat=高精度，f64=体积/速度；切换后重新加载 WASM */
  const [realBackendVariant, setRealBackendVariant] = useState<RealBackendVariant>(getStoredRealBackend)
  const [ephemerisSource, setEphemerisSource] = useState<EphemerisDataSource | null>(null)
  /** 章动表/地球改正加载状态（由 WASM get_repo_aux_* 提供，仅文本/混合路径有值；全二进制为 77 项+未加载） */
  const [repoAux, setRepoAux] = useState<{ nutationFull: boolean; earthPatch: boolean; elpCorrection: string } | null>(null)
  /** 单日信息卡片按域分组的可选显示项（天文事件层/历法派生层/工程仲裁层状态），见 DAY_INFO_LAYER_GROUPS */
  const [dayInfoLayers, setDayInfoLayers] = useState<Record<DayInfoLayerKey, boolean>>(getStoredDayInfoLayers)
  const toggleDayInfoLayer = (key: DayInfoLayerKey) => {
    setDayInfoLayers((prev) => {
      const next = { ...prev, [key]: !prev[key] }
      try { localStorage.setItem(DAY_INFO_LAYERS_STORAGE_KEY, JSON.stringify(next)) } catch {}
      return next
    })
  }
  const prevTimeZoneRef = useRef<string | null>(null)

  useEffect(() => {
    const tick = () => {
      const d = new Date()
      const key = String(Math.floor(d.getTime() / 60000))
      if (lastMinuteKeyRef.current !== key) {
        lastMinuteKeyRef.current = key
        setNowForLunar(d)
      }
    }
    tick() // 首次立即同步
    const t = setInterval(tick, 1000)
    return () => clearInterval(t)
  }, [])

  useEffect(() => {
    if (!wasm || yearDataList.length === 0) return
    const p = civilToProleptic(selectedDate.year, selectedDate.month, selectedDate.day)
    const g = getGanzhiForDay(wasm, p.year, p.month, p.day, yearDataList, ganzhiOptions)
    if (g.yearName && g.monthName) return
    if (isVsop87Cached()) return
    ensureVsop87Cached().then(() => setVsop87Tick((n) => n + 1)).catch(() => {})
  }, [wasm, yearDataList.length, selectedDate.year, selectedDate.month, selectedDate.day, ganzhiOptions, vsop87Tick, calendarMode])

  useEffect(() => {
    if (!wasm || ephemerisSource == null) {
      setRepoAux(null)
      return
    }
    const nutationFull = typeof wasm.get_repo_aux_nutation_full === 'function' ? wasm.get_repo_aux_nutation_full() : false
    const earthPatch = typeof wasm.get_repo_aux_earth_patch === 'function' ? wasm.get_repo_aux_earth_patch() : false
    const elpCorrection = typeof wasm.get_elpmpp02_correction === 'function' ? wasm.get_elpmpp02_correction() : 'DE431'
    setRepoAux({ nutationFull, earthPatch, elpCorrection })
  }, [wasm, ephemerisSource])

  const presetSelectValue = getPresetIndexFromOptions(ganzhiOptions)
  const dropdownValue = presetSelectValue >= 0 ? presetSelectValue : 4

  const weekdays = Array.from({ length: 7 }, (_, i) => WEEKDAY_NAMES[(weekStart + i) % 7])
  // year、month、selectedDate 都是民用历（1582-10-15 前为儒略历）；调 wasm 前换成前推格里历
  const monthDays = civilMonthDays(year, month)
  const days = monthDays[monthDays.length - 1]!
  const start = ((Math.floor(displayCivilToJd(year, month, 1) + 0.5) + 1) % 7 - weekStart + 7) % 7
  const cells: (number | null)[] = Array.from({ length: 42 }, (_, i) => monthDays[i - start] ?? null)

  // 整月农历一次 wasm 调用，返回结构化数据，由显示层格式化
  const lunarByDay = useMemo(() => {
    const out: (string | null)[] = []
    if (!wasm || yearDataList.length === 0) return out
    if (!isFullyGregorianMonth(year, month)) {
      for (const day of monthDays) {
        const p = civilToProleptic(year, month, day)
        out[day] = formatLunarWithFallback(wasm, p.year, p.month, p.day, orderedYearData(yearDataList, p.year, p.month), p.year, p.month) ?? null
      }
      return out
    }
    const primary = yearDataList[0]
    if (!primary || !wasm.gregorian_month_to_lunar) {
      for (let day = 1; day <= days; day++) {
        out[day] = formatLunarWithFallback(wasm, year, month, day, yearDataList, year, month) ?? null
      }
      if (year === 2026 && month === 2) console.log('[整月农历] 使用 fallback 路径')
      return out
    }
    const res = wasm.gregorian_month_to_lunar(
      year,
      month,
      primary.lunarYear,
      new Float64Array(primary.newMoonJds),
      new Float64Array(primary.zhongQiJds)
    )
    const ys = res.lunarYears
    const ms = res.lunarMonths
    const ds = res.dayOfMonths
    const leaps = res.isLeapMonths
    const daysInLunar = res.daysInLunarMonths ?? []
    const n = Math.min(ys.length, days)
    for (let i = 0; i < n; i++) {
      const slot = {
        lunarYear: ys[i],
        lunarMonth: ms[i],
        dayOfMonth: ds[i],
        isLeapMonth: leaps[i],
        daysInMonth: (daysInLunar[i] ?? 30) as number,
      }
      out[i + 1] = applyLunarDisplayLayers(slot, 2) ?? null
    }
    return out
  }, [wasm, year, month, days, yearDataList, calendarMode])

  // 整月干支历：优先批量 WASM 一次调用（ganzhiForGregorianMonthWasm），否则逐日兜底
  const ganzhiByDay = useMemo((): Record<number, GanzhiResult | null> => {
    if (!wasm || yearDataList.length === 0) return {}
    if (!isFullyGregorianMonth(year, month)) {
      const out: Record<number, GanzhiResult | null> = {}
      for (const day of civilMonthDays(year, month)) {
        const p = civilToProleptic(year, month, day)
        out[day] = getGanzhiForDay(wasm, p.year, p.month, p.day, orderedYearData(yearDataList, p.year, p.month), ganzhiOptions)
      }
      return out
    }
    return getGanzhiForMonth(wasm, year, month, yearDataList, ganzhiOptions)
  }, [wasm, year, month, yearDataList, ganzhiOptions, calendarMode])

  // 按流程：按所选 Real 后端加载 WASM → 取得该月对应气朔；同时预拉 VSOP87 供节气历年/月兜底
  useEffect(() => {
    let cancelled = false
    setError(null)
    setLoading('WASM')
    loadWasm(realBackendVariant)
      .then((w) => {
        if (cancelled) return
        setWasm(w)
        setLoading('数据')
        return Promise.all([
          getYearDataForCivilMonth(year, month, w, setEphemerisSource),
          ensureVsop87Cached().catch(() => {}),
        ]).then((res): YearData[] => res[0] ?? [])
      })
      .then((list) => {
        if (cancelled || !list) return
        setYearDataList(list)
        setLoading(null)
      })
      .catch((e) => {
        if (!cancelled) {
          setError(String(e?.message ?? e))
          setLoading(null)
        }
      })
    return () => { cancelled = true }
  }, [year, month, realBackendVariant, calendarMode])

  // 当左侧显示日期与日历视图不同月时，单独拉取显示日期所在月的岁数据，否则农历年月日会因无对应气朔而显示为 —
  useEffect(() => {
    const selected = civilToProleptic(selectedDate.year, selectedDate.month, selectedDate.day)
    const now = formatDateInTimeZone(nowForLunar, timeZone)
    const displayY = isLiveMode ? now.year : selected.year
    const displayM = isLiveMode ? now.month : selected.month
    // 日历的年月是显示写法，要和选中日的显示写法比，不能和前推格里历比
    if (isLiveMode ? displayY === year && displayM === month : selectedDate.year === year && selectedDate.month === month) {
      setDisplayDateYearDataList([])
      return
    }
    if (!wasm) return
    let cancelled = false
    getYearDataForDisplay(displayY, displayM, wasm, setEphemerisSource).then((list) => {
      if (!cancelled) setDisplayDateYearDataList(list)
    })
    return () => { cancelled = true }
  }, [wasm, year, month, isLiveMode, nowForLunar, selectedDate.year, selectedDate.month, selectedDate.day, calendarMode, timeZone])

  // 切换时区时，将当前选中的日期时间从旧时区换算到新时区（同一时刻）
  useEffect(() => {
    const prevTz = prevTimeZoneRef.current
    prevTimeZoneRef.current = timeZone
    if (prevTz == null) return
    if (prevTz === timeZone) return
    const { year: y, month: m, day: d } = selectedDate
    const { hour: h, minute: min, second: s } = selectedTime
    const p = convertDisplayDateTime(y, m, d, h, min, s, prevTz, timeZone)
    setYear(p.year)
    setMonth(p.month)
    setSelectedDate({ year: p.year, month: p.month, day: p.day })
    setSelectedTime({ hour: p.hour, minute: p.minute, second: p.second })
  }, [timeZone])

  /** 换西历写法时留在同一天，日期的写法跟着变。 */
  const handleCalendarModeChange = (next: CalendarMode) => {
    const jd = displayCivilToJd(selectedDate.year, selectedDate.month, selectedDate.day)
    setCalendarMode(next)
    const d = jdToDisplayDate(jd)
    setCalendarModeState(next)
    setYear(d.year)
    setMonth(d.month)
    setSelectedDate(d)
    try { localStorage.setItem(CALENDAR_MODE_STORAGE_KEY, next) } catch {}
  }

  const handleYearChange = (y: number) => {
    setYear(y)
    setSelectedDate((prev) => ({
      year: y,
      month: prev.month,
      day: clampCivilDay(y, prev.month, prev.day),
    }))
    setIsLiveMode(false)
  }
  const prev = () => {
    const [newYear, newMonth] = month === 1 ? [year - 1, 12] : [year, month - 1]
    setYear(newYear)
    setMonth(newMonth)
    setSelectedDate((prev) => ({
      year: newYear,
      month: newMonth,
      day: clampCivilDay(newYear, newMonth, prev.day),
    }))
    setIsLiveMode(false)
  }
  const next = () => {
    const [newYear, newMonth] = month === 12 ? [year + 1, 1] : [year, month + 1]
    setYear(newYear)
    setMonth(newMonth)
    setSelectedDate((prev) => ({
      year: newYear,
      month: newMonth,
      day: clampCivilDay(newYear, newMonth, prev.day),
    }))
    setIsLiveMode(false)
  }
  const goToday = () => {
    setYear(today.year)
    setMonth(today.month)
    setSelectedDate(today)
    const n = formatDateInTimeZone(new Date(), timeZone)
    setSelectedTime({ hour: n.hour, minute: n.minute, second: n.second })
    setIsLiveMode(true)
  }

  const sel = selectedDate
  const selProleptic = civilToProleptic(sel.year, sel.month, sel.day)
  const nowInZone = formatDateInTimeZone(nowForLunar, timeZone)
  const displayDate = isLiveMode ? localDate(nowInZone.year, nowInZone.month, nowInZone.day, nowInZone.hour, nowInZone.minute, nowInZone.second) : localDate(selProleptic.year, selProleptic.month, selProleptic.day, selectedTime.hour, selectedTime.minute, selectedTime.second)
  // 显示用民用历；实时模式必在 1582 年之后，两者相同
  const civilY = isLiveMode ? nowInZone.year : sel.year
  const civilM = isLiveMode ? nowInZone.month : sel.month
  const civilD = isLiveMode ? nowInZone.day : sel.day
  const displayY = displayDate.getFullYear()
  const displayM = displayDate.getMonth() + 1
  const displayD = displayDate.getDate()
  const displayHour = displayDate.getHours()
  const displayMinute = displayDate.getMinutes()
  const displaySecond = displayDate.getSeconds()
  const isSelectedToday = displayY === today.year && displayM === today.month && displayD === today.day
  const weekdayIndex = getWeekday(displayY, displayM, displayD)
  const dayOfYear = getDayOfYear(civilY, civilM, civilD)
  const weekOfYear = getWeekOfYear(civilY, civilM, civilD)

  /** 左侧面板用：选中日在日历当前月时用当前月岁数据和月历格子（按显示写法的日号），否则用单独拉取的岁数据 */
  const inShownMonth = civilY === year && civilM === month
  const panelYearDataList = inShownMonth ? yearDataList : displayDateYearDataList

  const selectedLunar: string | null =
    inShownMonth
      ? (lunarByDay[civilD] ?? null)
      : wasm && panelYearDataList.length > 0
        ? formatLunarWithFallback(wasm, displayY, displayM, displayD, panelYearDataList, displayY, displayM)
        : null
  const selectedLunarSlot = wasm && panelYearDataList.length > 0
    ? getLunarSlotForDay(wasm, displayY, displayM, displayD, panelYearDataList)
    : null
  const selectedGanzhi =
    wasm
      ? inShownMonth
        ? (ganzhiByDay[civilD] ?? getGanzhiForDay(wasm, displayY, displayM, displayD, panelYearDataList, ganzhiOptions))
        : getGanzhiForDay(wasm, displayY, displayM, displayD, panelYearDataList, ganzhiOptions)
      : null
  const zodiac =
    selectedGanzhi?.yearName?.length === 2
      ? ZODIAC_BY_BRANCH[selectedGanzhi.yearName[1]!]
      : null

  const selectedYearData = selectedLunarSlot != null ? panelYearDataList.find((d) => d.lunarYear === selectedLunarSlot.lunarYear) : null
  const newMoonJdUtcOnDay: number | null =
    wasm && selectedLunarSlot?.dayOfMonth === 1 && selectedYearData && typeof wasm.delta_t_seconds === 'function'
      ? (() => {
          const target = { y: displayY, m: displayM, d: displayD }
          for (let i = 0; i < selectedYearData.newMoonJds.length; i++) {
            const jdUtc = jdTtToUtc(wasm, selectedYearData.newMoonJds[i]!)
            const day = beijingDayFromJdUtc(jdUtc)
            if (day.y === target.y && day.m === target.m && day.d === target.d) return jdUtc
          }
          return null
        })()
      : null
  // 平气按当时行用历推算（data/calendar/pingqi_lifa.csv，只收已据律历志核定的历）；表外不显示
  const zhongQiOnDay: { name: string; jdUtc: number; pingQiJdUtc: number | null; pingQiLi: string }[] =
    wasm && selectedYearData && typeof wasm.delta_t_seconds === 'function'
      ? (() => {
          const target = { y: displayY, m: displayM, d: displayD }
          const out: { name: string; jdUtc: number; pingQiJdUtc: number | null; pingQiLi: string }[] = []
          for (let k = 0; k < 12; k++) {
            const jdTt = selectedYearData.zhongQiJds[k]!
            const jdUtc = jdTtToUtc(wasm, jdTt)
            const day = beijingDayFromJdUtc(jdUtc)
            if (day.y !== target.y || day.m !== target.m || day.d !== target.d) continue
            // zhongQiJds 第 k 个中气 = 冬至起第 2k 气
            const pingTt = typeof wasm.pingqi_tt_wasm === 'function' ? wasm.pingqi_tt_wasm(jdTt, 2 * k) : NaN
            const pingUtc = Number.isFinite(pingTt) ? jdTtToUtc(wasm, pingTt) : NaN
            const show = Number.isFinite(pingUtc) && Math.abs(pingUtc - jdUtc) > PING_QI_DIFF_THRESHOLD_DAYS
            out.push({
              name: ZHONG_QI_NAMES[k]!,
              jdUtc,
              pingQiJdUtc: show ? pingUtc : null,
              pingQiLi: show ? wasm.pingqi_li_name_wasm?.(day.y) ?? '' : '',
            })
          }
          return out
        })()
      : []

  // 时柱钟点：按所选时区钟点，或换成当地平太阳时、真太阳时（《協紀辨方書》卷三十四取太阳到方时刻）。
  // 当地太阳时与钟点之差可跨日（如西部地区），日柱随之进退一日。
  const wallHours = displayHour + displayMinute / 60 + displaySecond / 3600
  const localSolarHours = hourClockMode === 0 || !wasm
    ? null
    : (() => {
        const utc = utcFromLocalInZone({ year: displayY, month: displayM, day: displayD, hour: displayHour, minute: displayMinute, second: displaySecond }, timeZone)
        const jdUt = utc.getTime() / 86400000 + 2440587.5
        return getLocalSolarHours(wasm, jdUt, observerLongitude, hourClockMode as 1 | 2)
      })()
  let pillarDayShift = 0
  let pillarHour = displayHour
  if (localSolarHours != null) {
    let diff = localSolarHours - wallHours
    diff -= 24 * Math.round(diff / 24)
    const t = wallHours + diff
    pillarDayShift = Math.floor(t / 24)
    pillarHour = Math.floor(t - 24 * pillarDayShift)
  }
  const shiftGanzhi = (name: string, n: number) =>
    GAN[(((GAN.indexOf(name[0]!) + n) % 10) + 10) % 10]! + ZHI[(((ZHI.indexOf(name[1]!) + n) % 12) + 12) % 12]!
  const baseDayName = selectedGanzhi?.dayName && /^[\u4e00-\u9fa5]{2}$/.test(selectedGanzhi.dayName)
    ? shiftGanzhi(selectedGanzhi.dayName, pillarDayShift)
    : selectedGanzhi?.dayName
  // 子初换日（日界选项 0）：23 时起日柱已是次日；格子和 selectedGanzhi 按日期，这里再按时刻进一日
  const dayPillar = baseDayName && /^[\u4e00-\u9fa5]{2}$/.test(baseDayName) && ganzhiOptions[3] === 0 && pillarHour === 23
    ? shiftGanzhi(baseDayName, 1)
    : baseDayName
  const hourGanzhi = baseDayName && /^[\u4e00-\u9fa5]{2}$/.test(baseDayName)
    ? getHourGanzhi(baseDayName, pillarHour)
    : ''

  // 民俗：四柱纳音、值日宿与七曜、年月九星（规则见 rust/core/src/folk，出处见 doc/folk/nayin-zhiri.md）
  const folkDay = (() => {
    if (!wasm || !dayInfoLayers.folk_jishi || typeof wasm.folk_day_wasm !== 'function') return null
    const ear = getCachedVsop87Ear()
    if (!ear) return null
    try {
      const jd = wasm.gregorian_to_jd(displayY, displayM, displayD) + (displayHour + displayMinute / 60) / 24
      const r = wasm.folk_day_wasm(jd, ear)
      const get = (k: 'xiu' | 'qi_yao' | 'year_star' | 'month_star'): string => {
        const v = (r as unknown as Record<string, unknown>)[k]
        return typeof v === 'function' ? String((v as (this: unknown) => string).call(r)) : String(v ?? '')
      }
      const zajie = typeof wasm.zajie_for_day_wasm === 'function' ? wasm.zajie_for_day_wasm(jd, ear) : ''
      const geng = typeof wasm.gengdian_wasm === 'function'
        ? wasm.gengdian_wasm(wasm.gregorian_to_jd(displayY, displayM, displayD), observerLatitude, observerLongitude, gengdianMethod, ear)
        : ''
      const jianchu = typeof wasm.jianchu_day_wasm === 'function' ? wasm.jianchu_day_wasm(wasm.gregorian_to_jd(displayY, displayM, displayD), ear) : ''
      const shensha = typeof wasm.shensha_day_wasm === 'function' ? wasm.shensha_day_wasm(wasm.gregorian_to_jd(displayY, displayM, displayD), ear) : ''
      return { xiu: get('xiu'), qiYao: get('qi_yao'), yearStar: get('year_star'), monthStar: get('month_star'), zajie, geng, jianchu, shensha }
    } catch {
      return null
    }
  })()
  const nayinOf = (name: string | null | undefined) =>
    wasm && name && typeof wasm.nayin_of_ganzhi_name === 'function' ? wasm.nayin_of_ganzhi_name(name) : ''

  const yearOptions = useMemo(() => {
    const start = -2000
    const end = Math.max(2100, today.year + 50)
    return Array.from({ length: end - start + 1 }, (_, i) => {
      const y = start + i
      // y 是天文年号：0 为公元前 1 年，-1 为公元前 2 年
      const label = y <= 0 ? `公元前${1 - y}年` : `${y}年`
      return { label, value: y }
    })
  }, [today.year])
  const timeZoneOptions = useMemo(() => getTimeZoneOptions(), [])
  const weekStartOptions = WEEKDAY_NAMES.map((name, idx) => ({ label: `周${name}`, value: idx }))
  const ganzhiPresetOptions = [
    ...GANZHI_PRESET_NAMES.map((name, idx) => ({ label: name, value: idx })),
    { label: '自定义', value: 4 },
  ]

  const viewNav = (
    <Flex align="center" gap={12} style={{ marginBottom: 16 }}>
      <Button type={view === 'calendar' ? 'primary' : 'link'} icon={<CalendarOutlined />} onClick={() => setView('calendar')}>
        日历
      </Button>
      <Button type={view === 'graph' ? 'primary' : 'link'} icon={<ApartmentOutlined />} onClick={() => setView('graph')}>
        视位置变换图
      </Button>
      <Button type={view === 'eclipse' ? 'primary' : 'link'} icon={<GlobalOutlined />} onClick={() => setView('eclipse')}>
        日月食图
      </Button>
      <Button type={view === 'stars' ? 'primary' : 'link'} icon={<StarOutlined />} onClick={() => setView('stars')}>
        星图
      </Button>
      {/* 文档站（VitePress）由 pages.yml 构建后放在应用站 docs/ 子路径；本地 dev 需另起 doc/ 的 npm run dev */}
      <Button type="link" icon={<ReadOutlined />} href={`${import.meta.env.BASE_URL}docs/`}>
        文档
      </Button>
    </Flex>
  )

  if (view === 'graph') {
    return (
      <div style={{ maxWidth: 1200, margin: '0 auto', padding: 24, overflow: 'visible' }}>
        {viewNav}
        <TransformGraphPage wasm={wasm} />
      </div>
    )
  }

  if (view === 'stars') {
    return (
      <div style={{ maxWidth: 1200, margin: '0 auto', padding: 24, overflow: 'visible' }}>
        {viewNav}
        <StarChartPage />
      </div>
    )
  }

  if (view === 'eclipse') {
    return (
      <div style={{ maxWidth: 1200, margin: '0 auto', padding: 24, overflow: 'visible' }}>
        {viewNav}
        <EclipseMapPage />
      </div>
    )
  }

  return (
    <div style={{ maxWidth: 1200, margin: '0 auto', padding: 24, overflow: 'visible' }}>
      {viewNav}
      <Spin spinning={!!loading} tip={loading ? `加载${loading}…` : undefined}>
        {error && <Alert type="error" message={error} showIcon style={{ marginBottom: 16 }} />}

        <Flex gap={24} align="flex-start" wrap="wrap">
          {/* 左侧：单日信息（合并）。LiveTimeProvider 使实时模式下各行时刻共用同一时间源 */}
          <LiveTimeProvider enabled={isLiveMode}>
          <Flex vertical gap={12} style={{ width: 260, flexShrink: 0, overflow: 'visible' }}>
            <Card size="small" style={{ width: '100%', overflow: 'visible' }} styles={{ body: { padding: 0, overflow: 'visible' } }}>
              {/* 顶部：只保留公历年月 */}
              <div style={{ padding: '10px 12px', borderBottom: '1px solid rgba(0,0,0,0.06)', textAlign: 'center' }}>
                <Text type="secondary" style={{ fontSize: 11 }}>{year > 0 ? year : `公元前${1 - year}`}年{month}月</Text>
              </div>
              {/* 日（大号） */}
              <div style={{ padding: '16px 12px 8px', textAlign: 'center' }}>
                <div style={{ fontSize: 48, fontWeight: 700, lineHeight: 1, color: 'rgba(0,0,0,0.88)', fontVariantNumeric: 'tabular-nums' }}>
                  {civilD}
                </div>
                <Text type="secondary" style={{ fontSize: 12, marginTop: 4 }}>{timeZone}</Text>
              </div>
              {/* 各行统一为：历法 纪元 月 日 时刻 */}
              <div style={{ padding: '0 12px 12px', borderBottom: '1px solid rgba(0,0,0,0.06)' }}>
                {(() => {
                  const timeProps = { live: isLiveMode, timeZone, hour: displayHour, minute: displayMinute, second: displaySecond }
                  const clock = (h: number, m: number, s: number) =>
                    [h, m, s].map((n) => n.toString().padStart(2, '0')).join(':')
                  const timeOf = selectedLunarSlot != null && wasm?.format_time_for_lunar_year != null
                    ? (h: number, m: number, s: number) => wasm.format_time_for_lunar_year!(selectedLunarSlot.lunarYear, h, m, s)
                    : clock
                  const westernName = displayCalendarName(civilY, civilM, civilD)
                  const gregorianMonthDay = `${civilM}月${civilD}日`
                  const yearName = (y: number) => (y > 0 ? `${yearToChinese(y)}年` : `西元前${yearToChinese(1 - y)}年`)

                  type Era = { regime: string; emperor: string; era_name: string; era_year: string; era: string; is_primary?: boolean }
                  const eraMid = (p: Era) => [p.emperor, [p.era_name, p.era_year].filter(Boolean).join('') || p.era].filter(Boolean).join(' ')

                  const lines: React.ReactNode[] = []
                  lines.push(
                    <div key="gregorian" style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>
                      {westernName} 西元 {civilY > 0 ? `${civilY}年` : `前${1 - civilY}年`}{gregorianMonthDay}<LineTime format={clock} {...timeProps} />
                    </div>,
                  )

                  // 主线
                  if (selectedLunarSlot != null) {
                    const monthDay = `${selectedLunarSlot.isLeapMonth ? '闰' : ''}${LUNAR_MONTH_NAMES[selectedLunarSlot.lunarMonth - 1]}月 ${lunarDayStr(selectedLunarSlot.dayOfMonth)}日`
                    let era = yearName(selectedLunarSlot.lunarYear)
                    let hint = ''
                    if (dayInfoLayers.calendar_eraNames && (wasm?.get_era_name_parts_for_day != null || wasm?.get_era_name_parts != null)) {
                      try {
                        const partsJson = wasm.get_era_name_parts_for_day != null
                          ? wasm.get_era_name_parts_for_day(
                              selectedLunarSlot.lunarYear,
                              displayY,
                              displayM,
                              displayD,
                              selectedLunarSlot.lunarMonth,
                              !!selectedLunarSlot.isLeapMonth,
                              selectedLunarSlot.dayOfMonth,
                            )
                          : wasm.get_era_name_parts!(selectedLunarSlot.lunarYear, true)
                        const parts = JSON.parse(partsJson) as Era & {
                          gaiyuan_prev_era_name?: string
                          gaiyuan_prev_era_year?: string
                          has_gaiyuan?: boolean
                          gaiyuan_resolved?: boolean
                          gaiyuan_on_this_day?: boolean
                        }
                        const eraStr = [parts.era_name, parts.era_year].filter(Boolean).join('') || parts.era
                        const prevEraStr = [parts.gaiyuan_prev_era_name, parts.gaiyuan_prev_era_year].filter(Boolean).join('')
                        const shown = parts.gaiyuan_on_this_day && prevEraStr
                          ? `${prevEraStr}→${eraStr}`
                          : parts.has_gaiyuan && parts.gaiyuan_resolved === false && prevEraStr
                            ? `${prevEraStr}/${eraStr}`
                            : eraStr
                        // 没有年号也没有年数（如中华人民共和国）时写年份，否则这一行缺「年」
                        const mid = [parts.emperor, shown || yearName(selectedLunarSlot.lunarYear)].filter(Boolean).join(' ')
                        era = parts.regime ? `${parts.regime} · ${mid}` : mid
                        hint = parts.gaiyuan_on_this_day ? ' 改元' : parts.has_gaiyuan && parts.gaiyuan_resolved === false ? ' 改元日未补' : ''
                      } catch { /* 年号解析失败时退回只写年份 */ }
                    }
                    lines.push(
                      <div key="main" style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>
                        {wasm?.get_calendar_name?.(selectedLunarSlot.lunarYear) ?? '农历'} {era} {monthDay}<LineTime format={timeOf} {...timeProps} />{hint}
                      </div>,
                    )
                  } else if (selectedLunar != null) {
                    lines.push(<div key="main" style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>{selectedLunar}</div>)
                  } else {
                    lines.push(<div key="main" style={{ marginBottom: 4 }}><Text type="secondary">历日 —</Text></div>)
                  }

                  // 并存政权与分政权推步
                  type RegimeDay = { region: string; calendar_name: string; lunar_year: number; month: number; is_leap: boolean; day: number }
                  let regimeDays: RegimeDay[] = []
                  if (wasm?.get_regimes_lunar_for_day != null) {
                    try {
                      const arr = JSON.parse(wasm.get_regimes_lunar_for_day(displayY, displayM, displayD))
                      if (Array.isArray(arr)) regimeDays = arr as RegimeDay[]
                    } catch { /* 推步是补充信息，解析失败不影响主线 */ }
                  }
                  const regimeMonthDay = (e: RegimeDay) => `${e.is_leap ? '闰' : ''}${LUNAR_MONTH_NAMES[e.month - 1]}月 ${lunarDayStr(e.day)}日`
                  const usedRegions = new Set<string>()
                  const secondary: React.ReactNode[] = []
                  if (dayInfoLayers.calendar_eraNames && selectedLunarSlot != null && wasm?.get_era_name_parts_all != null) {
                    try {
                      const all = JSON.parse(wasm.get_era_name_parts_all(selectedLunarSlot.lunarYear)) as Era[]
                      for (const p of Array.isArray(all) ? all.filter((x) => !x.is_primary && x.regime) : []) {
                        const own = regimeDays.find((e) => e.region.split('/').includes(p.regime))
                        if (own != null) usedRegions.add(own.region)
                        const era = `${p.regime} · ${eraMid(p) || '—'}`
                        // 民国纪年配格里高利历；其余政权没有推步数据时不套用主线月日，如实写历未详
                        secondary.push(
                          <div key={`concurrent-${p.regime}`}>
                            {p.regime === '中华民国'
                              ? <>格里高利历 {era} {gregorianMonthDay}<LineTime format={clock} {...timeProps} /></>
                              : own != null
                                ? <>{own.calendar_name} {era} {regimeMonthDay(own)}<LineTime format={timeOf} {...timeProps} /></>
                                : <>历未详 {era}</>}
                          </div>,
                        )
                      }
                    } catch { /* 并存政权是补充信息，解析失败不影响主线 */ }
                  }
                  if (dayInfoLayers.calendar_regimeCalendars) {
                    for (const e of regimeDays.filter((x) => !usedRegions.has(x.region))) {
                      secondary.push(
                        <div key={`${e.region}-${e.calendar_name}`}>
                          {e.calendar_name} {e.region} {regimeMonthDay(e)}<LineTime format={timeOf} {...timeProps} />
                        </div>,
                      )
                    }
                  }
                  return (
                    <>
                      {lines}
                      <div style={{ fontSize: 13, fontWeight: 600 }}>星期{WEEKDAY_NAMES[weekdayIndex]}</div>
                      <Text type="secondary" style={{ fontSize: 11 }}>第{dayOfYear}天，第{weekOfYear}周</Text>
                      {!isLiveMode && isSelectedToday && <Tag color="blue" style={{ marginTop: 6 }}>今天</Tag>}
                      {secondary.length > 0 && <div style={{ fontSize: 12, marginTop: 6, color: 'rgba(0,0,0,0.65)' }}>{secondary}</div>}
                    </>
                  )
                })()}
                {/* 朔日：合朔时刻（天文，非实历） */}
                {dayInfoLayers.astro_synodic && selectedLunarSlot?.dayOfMonth === 1 && newMoonJdUtcOnDay != null && (
                  <div style={{ fontSize: 11, marginTop: 6, color: 'rgba(0,0,0,0.6)' }}>
                    合朔 {formatJdUtcInZone(newMoonJdUtcOnDay, timeZone)}（天文，古代可能算不准）
                  </div>
                )}
                {/* 节气：交节时刻（定气）；若与平气不一致则显示平气 */}
                {dayInfoLayers.astro_solarTerm && zhongQiOnDay.length > 0 && (
                  <div style={{ fontSize: 11, marginTop: 6, color: 'rgba(0,0,0,0.6)' }}>
                    {zhongQiOnDay.map((q) => (
                      <div key={q.name}>
                        {q.name} 交节 {formatJdUtcInZone(q.jdUtc, timeZone)}（定气）
                        {q.pingQiJdUtc != null && (
                          <span>；当时{q.pingQiLi}平气 {formatJdUtcInZone(q.pingQiJdUtc, timeZone)}</span>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
              {/* 底部：干支历 */}
              {dayInfoLayers.calendar_ganzhi && (
              <div style={{ padding: '10px 12px 12px', display: 'grid', gap: 8 }}>
                {dayInfoLayers.calendar_ganzhi && (
                <div>
                  <Text type="secondary" style={{ fontSize: 10, display: 'block', marginBottom: 2 }}>干支历</Text>
                  <div style={{ fontSize: 12, lineHeight: 1.5, fontVariantNumeric: 'tabular-nums' }}>
                    {[
                      selectedGanzhi?.yearName && /^[\u4e00-\u9fa5]{2}$/.test(selectedGanzhi.yearName) ? `${selectedGanzhi.yearName}${zodiac != null ? `[${zodiac}]` : ''}年` : '—',
                      selectedGanzhi?.monthName && /^[\u4e00-\u9fa5]{2}$/.test(selectedGanzhi.monthName) ? `${selectedGanzhi.monthName}月` : '—',
                      dayPillar != null && /^[\u4e00-\u9fa5]{2}$/.test(dayPillar) ? `${dayPillar}日` : '—',
                      hourGanzhi ? `${hourGanzhi}时` : '—',
                    ].join(' ')}
                  </div>
                </div>
                )}
                {dayInfoLayers.folk_jishi && (
                <div>
                  <Text type="secondary" style={{ fontSize: 10, display: 'block', marginBottom: 2 }}>纳音 · 值日 · 九星</Text>
                  <div style={{ fontSize: 12, lineHeight: 1.5 }}>
                    {[selectedGanzhi?.yearName, selectedGanzhi?.monthName, dayPillar, hourGanzhi].map(nayinOf).map((n) => n || '—').join(' ')}
                  </div>
                  {folkDay && (
                    <div style={{ fontSize: 12, lineHeight: 1.5 }}>
                      {`${folkDay.xiu}宿 · ${folkDay.qiYao}曜 · 年${folkDay.yearStar} · 月${folkDay.monthStar}`}
                    </div>
                  )}
                  {folkDay?.jianchu && (
                    <div style={{ fontSize: 12, lineHeight: 1.5 }}>
                      {folkDay.jianchu}
                      {dayPillar && hourGanzhi && typeof wasm?.huanghei_hour_wasm === 'function'
                        ? ` · ${hourGanzhi}时${wasm.huanghei_hour_wasm(dayPillar, hourGanzhi)}`
                        : ''}
                    </div>
                  )}
                  {folkDay?.shensha && (
                    <div style={{ fontSize: 11, lineHeight: 1.5, color: 'rgba(0,0,0,0.65)', whiteSpace: 'pre-line' }}>
                      {folkDay.shensha}
                      <div style={{ color: 'rgba(0,0,0,0.45)' }}>据《协纪辨方书》月表，未计年份、节气相关铺注（無祿、四離四絕、氣往亡等）</div>
                    </div>
                  )}
                  {folkDay?.zajie && (
                    <div style={{ fontSize: 12, lineHeight: 1.5 }}>{folkDay.zajie}</div>
                  )}
                  {folkDay?.geng && (
                    <div style={{ fontSize: 11, lineHeight: 1.5, color: 'rgba(0,0,0,0.6)' }}>
                      {`更点（${gengdianMethod === 1 ? '元授时法' : '清台官法'}，当地真太阳时）：${folkDay.geng}`}
                    </div>
                  )}
                </div>
                )}
              </div>
              )}
            </Card>
          </Flex>
          </LiveTimeProvider>

          {/* 中间：日历 */}
          <div style={{ flex: 1, minWidth: 320 }}>
          <Card styles={{ body: { padding: 0 } }}>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(7, 1fr)',
                gridAutoRows: '72px',
                gap: 1,
                padding: 12,
              }}
            >
              {weekdays.map((w) => (
                <div
                  key={w}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    minHeight: 72,
                    minWidth: 0,
                    fontWeight: 600,
                    fontSize: 13,
                    color: w === '六' || w === '日' ? '#ff4d4f' : undefined,
                  }}
                >
                  {w}
                </div>
              ))}
              {cells.map((cellDay, i) => {
                const col = i % 7
                const dow = (weekStart + col) % 7
                const isWeekend = dow === 0 || dow === 6
                const isCurMonth = cellDay != null
                const isToday = isCurMonth && year === today.year && month === today.month && cellDay === today.day
                const isSelected = isCurMonth && year === sel.year && month === sel.month && cellDay === sel.day
                const lunarStr = isCurMonth && cellDay != null ? (lunarByDay[cellDay] ?? null) : null
                const ganzhi = isCurMonth && cellDay != null ? (ganzhiByDay[cellDay] ?? null) : null
                return (
                  <Button
                    key={cellDay != null ? `d-${year}-${month}-${cellDay}-${lunarStr ?? ''}` : `pad-${i}`}
                    type="text"
                    block
                    onClick={() => {
                      if (!isCurMonth || cellDay == null) return
                      setSelectedDate({ year, month, day: cellDay })
                      setIsLiveMode(false)
                    }}
                    disabled={!isCurMonth}
                    style={{
                      height: '100%',
                      minHeight: 72,
                      minWidth: 0,
                      padding: 4,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      textAlign: 'center',
                      border: '1px solid #f0f0f0',
                      borderRadius: 0,
                      boxSizing: 'border-box',
                      outline: isSelected ? '2px solid #1677ff' : 'none',
                      outlineOffset: -1,
                      background: isToday ? '#e6f4ff' : isSelected ? '#f0f7ff' : undefined,
                      color: isCurMonth ? (isWeekend ? '#ff4d4f' : undefined) : '#bfbfbf',
                      overflow: 'hidden',
                      position: 'relative',
                    }}
                  >
                    {isToday && (
                      <Tag color="blue" style={{ position: 'absolute', top: 4, right: 4, margin: 0 }}>
                        今
                      </Tag>
                    )}
                    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 0, width: '100%' }}>
                      <div style={{ fontWeight: 600, fontSize: isCurMonth ? 16 : 14 }}>{cellDay ?? ''}</div>
                      {lunarStr != null && <Text type="secondary" style={{ fontSize: 11 }}>{lunarStr}</Text>}
                      {ganzhi?.dayName != null && /^[\u4e00-\u9fa5]{2}$/.test(ganzhi.dayName) && (
                        <Text type="secondary" style={{ fontSize: 10 }}>{ganzhi.dayName}</Text>
                      )}
                    </div>
                  </Button>
                )
              })}
            </div>
          </Card>
          </div>

          {/* 右侧：日期时间 / 时区与语言 选项卡 + 选项 */}
          <Flex vertical gap={16} style={{ width: 260, flexShrink: 0 }}>
            <Card size="small" style={{ width: '100%' }}>
              <Tabs
                size="small"
                items={[
                  {
                    key: 'datetime',
                    label: '日期与时间',
                    children: (
                      <Space direction="vertical" style={{ width: '100%' }}>
                        <div>
                          <Text type="secondary" style={{ fontSize: 12 }}>年</Text>
                          <Select value={year} onChange={handleYearChange} options={yearOptions} showSearch optionFilterProp="label" filterOption={(input, opt) => (opt?.label ?? '').toString().includes(input)} style={{ width: '100%', marginTop: 4 }} />
                        </div>
                        <div>
                          <Text type="secondary" style={{ fontSize: 12 }}>月</Text>
                          <Space.Compact style={{ width: '100%', marginTop: 4 }}>
                            <Button icon={<LeftOutlined />} onClick={prev} />
                            <Button style={{ flex: 1 }}>{month}月</Button>
                            <Button icon={<RightOutlined />} onClick={next} />
                          </Space.Compact>
                        </div>
                        <div>
                          <Text type="secondary" style={{ fontSize: 12 }}>日</Text>
                          <Select
                            value={selectedDate.day}
                            onChange={(d) => {
                              setSelectedDate((prev) => ({ ...prev, day: d }))
                              setIsLiveMode(false)
                            }}
                            options={monthDays.map((d) => ({ label: `${d}日`, value: d }))}
                            style={{ width: '100%', marginTop: 4 }}
                          />
                        </div>
                        <div>
                          <Text type="secondary" style={{ fontSize: 12 }}>时分秒</Text>
                          <Flex gap={6} style={{ marginTop: 4 }}>
                            <Select
                              size="small"
                              value={selectedTime.hour}
                              onChange={(h) => {
                                setSelectedTime((t) => ({ ...t, hour: h }))
                                setIsLiveMode(false)
                              }}
                              options={Array.from({ length: 24 }, (_, i) => ({ label: i.toString().padStart(2, '0'), value: i }))}
                              style={{ flex: 1, minWidth: 0, fontSize: 12 }}
                              getPopupContainer={(n) => n.parentElement ?? document.body}
                            />
                            <Select
                              size="small"
                              value={selectedTime.minute}
                              onChange={(m) => {
                                setSelectedTime((t) => ({ ...t, minute: m }))
                                setIsLiveMode(false)
                              }}
                              options={Array.from({ length: 60 }, (_, i) => ({ label: i.toString().padStart(2, '0'), value: i }))}
                              style={{ flex: 1, minWidth: 0, fontSize: 12 }}
                              getPopupContainer={(n) => n.parentElement ?? document.body}
                            />
                            <Select
                              size="small"
                              value={selectedTime.second}
                              onChange={(s) => {
                                setSelectedTime((t) => ({ ...t, second: s }))
                                setIsLiveMode(false)
                              }}
                              options={Array.from({ length: 60 }, (_, i) => ({ label: i.toString().padStart(2, '0'), value: i }))}
                              style={{ flex: 1, minWidth: 0, fontSize: 12 }}
                              getPopupContainer={(n) => n.parentElement ?? document.body}
                            />
                          </Flex>
                        </div>
                        {isLiveMode ? null : (
                          <Button type="primary" icon={<CalendarOutlined />} onClick={() => { setIsLiveMode(true); goToday() }} block>返回实时</Button>
                        )}
                      </Space>
                    ),
                  },
                  {
                    key: 'locale',
                    label: '时区与语言',
                    children: (
                      <Space direction="vertical" style={{ width: '100%' }}>
                        <div>
                          <Text type="secondary" style={{ fontSize: 12 }}>时区</Text>
                          <Select
                            value={timeZone}
                            onChange={setTimeZone}
                            options={timeZoneOptions}
                            style={{ width: '100%', marginTop: 4 }}
                            showSearch
                            optionFilterProp="label"
                            placeholder="选择时区"
                          />
                        </div>
                        <div>
                          <Text type="secondary" style={{ fontSize: 12 }}>语言</Text>
                          <Select
                            value="zh-CN"
                            options={[{ label: '简体中文', value: 'zh-CN' }]}
                            style={{ width: '100%', marginTop: 4 }}
                            disabled
                          />
                        </div>
                      </Space>
                    ),
                  },
                ]}
              />
            </Card>
            <Button
              type="text"
              icon={<SettingOutlined />}
              onClick={() => setOptionsPanelOpen(true)}
              title="选项"
              style={{ width: '100%' }}
            >
              选项
            </Button>
            <Modal
              title="选项"
              open={optionsPanelOpen}
              onCancel={() => setOptionsPanelOpen(false)}
              footer={
                <Button type="primary" onClick={() => setOptionsPanelOpen(false)}>
                  确定
                </Button>
              }
              width={360}
            >
              <Flex vertical gap={20}>
                <div>
                  <Text type="secondary" style={{ fontSize: 12 }}>Real 后端</Text>
                  <Select
                    value={realBackendVariant}
                    onChange={(v) => {
                      if (v !== 'twofloat' && v !== 'f64') return
                      setRealBackendVariant(v)
                      try { localStorage.setItem(REAL_BACKEND_STORAGE_KEY, v) } catch {}
                    }}
                    options={[
                      { label: 'f64（默认，体积小/速度快）', value: 'f64' },
                      { label: 'TwoFloat（高精度）', value: 'twofloat' },
                    ]}
                    style={{ width: '100%', marginTop: 4 }}
                  />
                </div>
                <div>
                  <Text type="secondary" style={{ fontSize: 12 }}>西历写法</Text>
                  <Select
                    value={calendarMode}
                    onChange={handleCalendarModeChange}
                    options={CALENDAR_MODE_OPTIONS}
                    style={{ width: '100%', marginTop: 4 }}
                  />
                </div>
                <div>
                  <Text type="secondary" style={{ fontSize: 12 }}>起始日</Text>
                  <Select value={weekStart} onChange={setWeekStart} options={weekStartOptions} style={{ width: '100%', marginTop: 4 }} />
                </div>
                <div>
                  <Text type="secondary" style={{ fontSize: 12 }}>干支历</Text>
                  <Select
                    value={dropdownValue}
                    onChange={(v) => v >= 0 && v <= 3 && setGanzhiOptions(GANZHI_PRESET_OPTIONS[v]!)}
                    options={ganzhiPresetOptions}
                    style={{ width: '100%', marginTop: 4 }}
                  />
                </div>
                <div>
                  <Text type="secondary" style={{ fontSize: 12 }}>干支历详细</Text>
                  <div style={{ marginTop: 8 }}>
                    <div style={{ marginBottom: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>年界</Text>
                      <Radio.Group
                        value={ganzhiOptions[0]}
                        onChange={(e) => setGanzhiOptions((o) => [e.target.value, o[1], o[2], o[3], o[4]])}
                        options={GANZHI_YEAR_BOUNDARY_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div style={{ marginBottom: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>月界</Text>
                      <Radio.Group
                        value={ganzhiOptions[1]}
                        onChange={(e) => setGanzhiOptions((o) => [o[0], e.target.value, o[2], o[3], o[4]])}
                        options={GANZHI_MONTH_BOUNDARY_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div style={{ marginBottom: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>闰月</Text>
                      <Radio.Group
                        value={ganzhiOptions[2]}
                        onChange={(e) => setGanzhiOptions((o) => [o[0], o[1], e.target.value, o[3], o[4]])}
                        options={GANZHI_LEAP_MONTH_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div>
                      <Text type="secondary" style={{ fontSize: 11 }}>日界</Text>
                      <Radio.Group
                        value={ganzhiOptions[3]}
                        onChange={(e) => setGanzhiOptions((o) => [o[0], o[1], o[2], e.target.value, o[4]])}
                        options={GANZHI_DAY_BOUNDARY_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div style={{ marginTop: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>节气</Text>
                      <Radio.Group
                        value={ganzhiOptions[4]}
                        onChange={(e) => setGanzhiOptions((o) => [o[0], o[1], o[2], o[3], e.target.value])}
                        options={GANZHI_SOLAR_TERM_MODE_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div style={{ marginTop: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>时柱钟点</Text>
                      <Radio.Group
                        value={hourClockMode}
                        onChange={(e) => setHourClockMode(e.target.value)}
                        options={HOUR_CLOCK_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div style={{ marginTop: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>观测地（时柱当地时、更点用）</Text>
                      <div style={{ marginTop: 6, display: 'flex', gap: 8, flexWrap: 'wrap' }}>
                        <InputNumber size="small" min={-180} max={180} step={0.1} value={observerLongitude}
                          onChange={(v) => v != null && setObserverLongitude(v)} addonBefore="东经" addonAfter="°" />
                        <InputNumber size="small" min={-66} max={66} step={0.1} value={observerLatitude}
                          onChange={(v) => v != null && setObserverLatitude(v)} addonBefore="北纬" addonAfter="°" />
                      </div>
                    </div>
                    <div style={{ marginTop: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>更点</Text>
                      <Radio.Group
                        value={gengdianMethod}
                        onChange={(e) => setGengdianMethod(e.target.value)}
                        options={[{ value: 0, label: '清台官法（日入后八刻起更）' }, { value: 1, label: '元授时法（昏明各二刻半）' }]}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                  </div>
                </div>
              </Flex>
            </Modal>
            <Button
              type="text"
              icon={<EyeOutlined />}
              onClick={() => setDisplayLayersPanelOpen(true)}
              title="显示图层"
              style={{ width: '100%' }}
            >
              显示图层
            </Button>
            <Modal
              title="显示图层"
              open={displayLayersPanelOpen}
              onCancel={() => setDisplayLayersPanelOpen(false)}
              footer={
                <Button type="primary" onClick={() => setDisplayLayersPanelOpen(false)}>
                  确定
                </Button>
              }
              width={360}
            >
              <Text type="secondary" style={{ fontSize: 11, display: 'block' }}>
                按天文/历法/工程分组（见 doc/architecture/three-domains.md），农历本身是骨架信息，不在此列
              </Text>
              {DAY_INFO_LAYER_GROUPS.map((group) => (
                <div key={group.title} style={{ marginTop: 14 }}>
                  <Text type="secondary" style={{ fontSize: 11, fontWeight: 600 }}>{group.title}</Text>
                  <div style={{ marginTop: 4, display: 'flex', flexDirection: 'column', gap: 4 }}>
                    {group.items.map((item) => (
                      <Checkbox
                        key={item.key}
                        checked={dayInfoLayers[item.key]}
                        onChange={() => toggleDayInfoLayer(item.key)}
                      >
                        {item.label}
                      </Checkbox>
                    ))}
                  </div>
                </div>
              ))}
            </Modal>
          </Flex>
        </Flex>
        {dayInfoLayers.repoStatus && (
        <div style={{ marginTop: 12, paddingTop: 8, borderTop: '1px solid #f0f0f0' }}>
          <Text type="secondary" style={{ fontSize: 12 }}>
            {loading === 'WASM' && 'Repo：加载 WASM…'}
            {loading === '数据' && 'Repo：加载历表…'}
            {loading == null && error != null && 'Repo：加载失败'}
            {loading == null && error == null && ephemerisSource != null &&
              `Repo：已加载 · 格式：${EPHEMERIS_FORMAT_LABELS[ephemerisSource]} · ${EPHEMERIS_SOURCE_LABELS[ephemerisSource]} · 计算后端：${realBackendVariant === 'f64' ? 'f64（体积/速度优先）' : 'TwoFloat（高精度）'}`}
            {loading == null && error == null && ephemerisSource == null && wasm != null && 'Repo：就绪'}
            {loading == null && error == null && wasm == null && 'Repo：—'}
          </Text>
          {ephemerisSource != null && (
            <div
              style={{
                marginTop: 6,
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
                rowGap: 2,
                columnGap: 16,
              }}
            >
              <Text type="secondary" style={{ fontSize: 11 }}>
                VSOP87（日月地心历）：{ephemerisSource === 'full_text' ? '文本' : '二进制 (.bin/.br)'}
              </Text>
              <Text type="secondary" style={{ fontSize: 11 }}>
                ELPMPP02（月球运动）：{ephemerisSource === 'full_binary' ? '二进制 (.bin/.br)' : '文本'}
              </Text>
              {repoAux != null && (
                <>
                  <Text type="secondary" style={{ fontSize: 11 }}>
                    IAU2000 章动表：{repoAux.nutationFull ? '完整 IAU2000A（二进制）' : 'IAU2000B 简版（77 项，回落）'}
                  </Text>
                  <Text type="secondary" style={{ fontSize: 11 }}>
                    VSOP87 地球改正（拟合 {EARTH_PATCH_TARGET.toUpperCase()}）：{repoAux.earthPatch ? '已加载（二进制）' : '未加载'}
                  </Text>
                  <Text type="secondary" style={{ fontSize: 11 }}>
                    ELPMPP02 常数集：{repoAux.elpCorrection === 'DE441' ? '拟合 DE441' : repoAux.elpCorrection === 'DE406' ? '论文 DE405 + Table 6（贴近 DE406）' : '拟合 DE431（与 MS2021 ΔT 配套）'}
                  </Text>
                </>
              )}
            </div>
          )}
        </div>
        )}
      </Spin>
    </div>
  )
}

export default App
