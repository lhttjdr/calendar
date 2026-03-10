import { useState, useEffect, useMemo, useRef, createContext, useContext } from 'react'
import { Button, Card, Select, Radio, Space, Flex, Typography, Tag, Spin, Alert, Modal, Tabs } from 'antd'
import { LeftOutlined, RightOutlined, CalendarOutlined, SettingOutlined, ApartmentOutlined } from '@ant-design/icons'
import { TransformGraphPage } from './TransformGraphPage'
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
  GANZHI_PRESET_OPTIONS,
  GANZHI_PRESET_NAMES,
  getPresetIndexFromOptions,
  GANZHI_YEAR_BOUNDARY_OPTIONS,
  GANZHI_MONTH_BOUNDARY_OPTIONS,
  GANZHI_LEAP_MONTH_OPTIONS,
  GANZHI_DAY_BOUNDARY_OPTIONS,
  LUNAR_MONTH_NAMES,
  lunarDayStr,
  EPHEMERIS_SOURCE_LABELS,
  type YearData,
  type GanzhiResult,
  type GanzhiOptionsTuple,
  type RealBackendVariant,
  type EphemerisDataSource,
} from './useLunarWasm'

const REAL_BACKEND_STORAGE_KEY = 'lunar-real-backend'

function getStoredRealBackend(): RealBackendVariant {
  try {
    const v = localStorage.getItem(REAL_BACKEND_STORAGE_KEY)
    if (v === 'f64' || v === 'twofloat') return v
  } catch {}
  return 'f64'
}

const { Text } = Typography

/** 实时时间 Context：仅在 enabled 时每秒更新，供 LiveClock 与时宪历联动 */
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

/** 将 Date 格式化为指定时区的「年月日 时:分:秒」 */
function formatDateInTimeZone(d: Date, timeZone: string): { year: number; month: number; day: number; hour: number; minute: number; second: number } {
  const fmt = new Intl.DateTimeFormat('zh-CN', {
    timeZone,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })
  const parts = fmt.formatToParts(d)
  const get = (type: string) => parseInt(parts.find((p) => p.type === type)?.value ?? '0', 10)
  return { year: get('year'), month: get('month'), day: get('day'), hour: get('hour'), minute: get('minute'), second: get('second') }
}

type DateParts = { year: number; month: number; day: number; hour: number; minute: number; second: number }
function datePartsEqual(a: DateParts, b: DateParts): boolean {
  return a.year === b.year && a.month === b.month && a.day === b.day && a.hour === b.hour && a.minute === b.minute && a.second === b.second
}
function datePartsLessThan(a: DateParts, b: DateParts): boolean {
  if (a.year !== b.year) return a.year < b.year
  if (a.month !== b.month) return a.month < b.month
  if (a.day !== b.day) return a.day < b.day
  if (a.hour !== b.hour) return a.hour < b.hour
  if (a.minute !== b.minute) return a.minute < b.minute
  return a.second < b.second
}

/** 将「某时区下的本地时间」转为 UTC Date，用于切换时区时保持同一时刻 */
function utcFromLocalInZone(y: number, m: number, d: number, h: number, min: number, s: number, timeZone: string): Date {
  const target: DateParts = { year: y, month: m, day: d, hour: h, minute: min, second: s }
  let lo = Date.UTC(y, m - 1, d, 0, 0, 0) - 86400000 * 2
  let hi = Date.UTC(y, m - 1, d, 23, 59, 59) + 86400000 * 2
  for (let i = 0; i < 40; i++) {
    const mid = new Date((lo + hi) / 2)
    const p = formatDateInTimeZone(mid, timeZone)
    if (datePartsEqual(p, target)) return mid
    const midVal = mid.getTime()
    if (datePartsLessThan(p, target)) {
      lo = midVal
    } else {
      hi = midVal
    }
  }
  return new Date((lo + hi) / 2)
}

/** 实时时钟：与 LiveTimeProvider 联动，按所选时区显示当前时刻 */
function LiveClock({ timeZone }: { timeZone: string }) {
  const t = useLiveTime()
  if (t == null) return null
  const p = formatDateInTimeZone(t, timeZone)
  return (
    <span style={{ fontVariantNumeric: 'tabular-nums', fontWeight: 600 }}>
      {p.year}年{p.month}月{p.day}日{' '}
      {p.hour.toString().padStart(2, '0')}:{p.minute.toString().padStart(2, '0')}:{p.second.toString().padStart(2, '0')}
      {' '}<Text type="secondary" style={{ fontSize: 12 }}>{timeZone}</Text>
    </span>
  )
}

/** 时宪历：与 LiveClock 共用同一实时时间 */
function ShixianFromLiveTime() {
  const t = useLiveTime()
  if (t == null) return null
  return <div style={{ marginTop: 2 }}>{formatShixianKe(t.getHours(), t.getMinutes(), t.getSeconds())}</div>
}

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
  const ganIndex = (dayGanIndex * 2 + zhiIndex) % 10
  return GAN[ganIndex]! + ZHI[zhiIndex]!
}

/** 年份转汉字，数字0用〇（如 2026→二〇二六） */
function yearToChinese(n: number): string {
  const digits = '〇一二三四五六七八九'
  return String(n).replace(/[0-9]/g, (d) => digits[parseInt(d, 10)]!)
}

/** 0~99 转汉字（时宪历分、秒用）：0→零，1→一，10→十，83→八十三 */
function numToChineseSmall(n: number): string {
  if (n < 0 || n > 99) return String(n)
  const digits = '零一二三四五六七八九'
  if (n === 0) return '零'
  if (n < 10) return digits[n]!
  const tens = Math.floor(n / 10)
  const ones = n % 10
  const tenStr = tens === 1 ? '十' : digits[tens]! + '十'
  return ones === 0 ? tenStr : tenStr + digits[ones]
}

/** 时宪历（日96刻，每时辰8刻；《御制数理精蕴》每刻15分、每分60秒）将时分秒转为「刻+零+分秒」全汉字（不重复显示地支，四柱已有）。 */
function formatShixianKe(hour: number, minute: number, second: number): string {
  const fromMidnight = hour * 3600 + minute * 60 + second
  const fromZichu = (fromMidnight + 3600) % 86400 // 子初=23:00 为 0，与干支历一致
  const keIndex = Math.min(95, Math.floor(fromZichu / 900)) // 0~95，每刻900秒=15分
  const keInZhi = keIndex % 8 // 时辰内刻 0~7
  const keNames = ['初初刻', '初一刻', '初二刻', '初三刻', '正初刻', '正一刻', '正二刻', '正三刻']
  const withinKeSeconds = fromZichu % 900
  const fen = Math.floor(withinKeSeconds / 60) // 0~14
  const miao = Math.floor(withinKeSeconds % 60) // 0~59
  const keName = keNames[keInZhi]!
  if (fen === 0 && miao === 0) return keName
  const fenStr = numToChineseSmall(fen)
  const miaoStr = numToChineseSmall(miao)
  const fenMiao = miao > 0 ? `${fenStr}分${miaoStr}秒` : `${fenStr}分`
  return `${keName}零${fenMiao}`
}

function getWeekday(y: number, m: number, d: number): number {
  const a = Math.floor((14 - m) / 12)
  const y2 = y + 4800 - a
  const m2 = m + 12 * a - 3
  const jd = d + Math.floor((153 * m2 + 2) / 5) + 365 * y2 + Math.floor(y2 / 4) - Math.floor(y2 / 100) + Math.floor(y2 / 400) - 32045
  return (jd + 1) % 7
}

function getDayOfYear(y: number, m: number, d: number): number {
  const start = new Date(y, 0, 1).getTime()
  const curr = new Date(y, m - 1, d).getTime()
  return Math.floor((curr - start) / 86400000) + 1
}

function getWeekOfYear(y: number, m: number, d: number): number {
  const jan1 = new Date(y, 0, 1)
  const dow = jan1.getDay()
  const mon1 = (dow === 0 ? -5 : 2 - dow)
  const doy = getDayOfYear(y, m, d)
  return Math.floor((doy - mon1) / 7) + 1
}

function App() {
  const today = useMemo(() => {
    const t = new Date()
    return { year: t.getFullYear(), month: t.getMonth() + 1, day: t.getDate() }
  }, [])

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
  const [ganzhiOptions, setGanzhiOptions] = useState<GanzhiOptionsTuple>(() => GANZHI_PRESET_OPTIONS[2]!)
  const [optionsPanelOpen, setOptionsPanelOpen] = useState(false)
  /** 仅小时变化时更新，用于年月日时与农历/八字；不每秒更新避免整页重算 */
  const [nowForLunar, setNowForLunar] = useState(() => new Date())
  const lastHourKeyRef = useRef<string>('')
  const [vsop87Tick, setVsop87Tick] = useState(0)
  /** 一周起始日：0=周日 1=周一 … 6=周六 */
  const [weekStart, setWeekStart] = useState(1)
  /** 当前视图：日历 | 视位置变换图 */
  const [view, setView] = useState<'calendar' | 'graph'>('calendar')
  /** Real 后端：twofloat=高精度，f64=体积/速度；切换后重新加载 WASM */
  const [realBackendVariant, setRealBackendVariant] = useState<RealBackendVariant>(getStoredRealBackend)
  const [ephemerisSource, setEphemerisSource] = useState<EphemerisDataSource | null>(null)
  const [timeZone, setTimeZone] = useState(() => Intl.DateTimeFormat().resolvedOptions().timeZone)
  const prevTimeZoneRef = useRef<string | null>(null)

  useEffect(() => {
    const tick = () => {
      const d = new Date()
      const key = `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}-${d.getHours()}`
      if (lastHourKeyRef.current !== key) {
        lastHourKeyRef.current = key
        setNowForLunar(d)
      }
    }
    tick() // 首次立即同步
    const t = setInterval(tick, 1000)
    return () => clearInterval(t)
  }, [])

  useEffect(() => {
    if (!wasm || yearDataList.length === 0) return
    const g = getGanzhiForDay(wasm, selectedDate.year, selectedDate.month, selectedDate.day, yearDataList, ganzhiOptions)
    if (g.yearName && g.monthName) return
    if (isVsop87Cached()) return
    ensureVsop87Cached().then(() => setVsop87Tick((n) => n + 1)).catch(() => {})
  }, [wasm, yearDataList.length, selectedDate.year, selectedDate.month, selectedDate.day, ganzhiOptions, vsop87Tick])

  const presetSelectValue = getPresetIndexFromOptions(ganzhiOptions)
  const dropdownValue = presetSelectValue >= 0 ? presetSelectValue : 4

  const weekdays = Array.from({ length: 7 }, (_, i) => WEEKDAY_NAMES[(weekStart + i) % 7])
  const daysInMonth = (y: number, m: number) => {
    const leap = (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0
    if (m === 2) return leap ? 29 : 28
    if ([4, 6, 9, 11].includes(m)) return 30
    return 31
  }
  const firstDow = (y: number, m: number) => {
    const a = Math.floor((14 - m) / 12)
    const y2 = y + 4800 - a
    const m2 = m + 12 * a - 3
    const jd = 1 + Math.floor((153 * m2 + 2) / 5) + 365 * y2 + Math.floor(y2 / 4) - Math.floor(y2 / 100) + Math.floor(y2 / 400) - 32045
    return (Math.floor(jd + 0.5) + 1) % 7
  }
  const days = daysInMonth(year, month)
  const start = (firstDow(year, month) - weekStart + 7) % 7
  const cells: (number | null)[] = []
  let d = 1
  for (let i = 0; i < 42; i++) {
    if (i < start || d > days) cells.push(null)
    else { cells.push(d); d++ }
  }

  // 整月农历一次 wasm 调用，返回结构化数据，由显示层格式化
  const lunarByDay = useMemo(() => {
    const out: (string | null)[] = []
    if (!wasm || yearDataList.length === 0) return out
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
  }, [wasm, year, month, days, yearDataList])

  // 整月干支历：优先批量 WASM 一次调用（ganzhiForGregorianMonthWasm），否则逐日兜底
  const ganzhiByDay = useMemo((): Record<number, GanzhiResult | null> => {
    if (!wasm || yearDataList.length === 0) return {}
    return getGanzhiForMonth(wasm, year, month, yearDataList, ganzhiOptions)
  }, [wasm, year, month, yearDataList, ganzhiOptions])

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
          getYearDataForDisplay(year, month, w, setEphemerisSource),
          ensureVsop87Cached().catch(() => {}),
        ]).then((res): YearData[] => res[0] ?? [])
      })
      .then((list) => {
        if (cancelled) return
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
  }, [year, month, realBackendVariant])

  // 当左侧显示日期与日历视图不同月时，单独拉取显示日期所在月的岁数据，否则农历年月日会因无对应气朔而显示为 —
  useEffect(() => {
    const displayY = (isLiveMode ? nowForLunar : new Date(selectedDate.year, selectedDate.month - 1, selectedDate.day)).getFullYear()
    const displayM = (isLiveMode ? nowForLunar : new Date(selectedDate.year, selectedDate.month - 1, selectedDate.day)).getMonth() + 1
    if (displayY === year && displayM === month) {
      setDisplayDateYearDataList([])
      return
    }
    if (!wasm) return
    let cancelled = false
    getYearDataForDisplay(displayY, displayM, wasm, setEphemerisSource).then((list) => {
      if (!cancelled) setDisplayDateYearDataList(list)
    })
    return () => { cancelled = true }
  }, [wasm, year, month, isLiveMode, nowForLunar, selectedDate.year, selectedDate.month, selectedDate.day])

  // 切换时区时，将当前选中的日期时间从旧时区换算到新时区（同一时刻）
  useEffect(() => {
    const prevTz = prevTimeZoneRef.current
    prevTimeZoneRef.current = timeZone
    if (prevTz == null) return
    if (prevTz === timeZone) return
    const { year: y, month: m, day: d } = selectedDate
    const { hour: h, minute: min, second: s } = selectedTime
    const utc = utcFromLocalInZone(y, m, d, h, min, s, prevTz)
    const p = formatDateInTimeZone(utc, timeZone)
    setYear(p.year)
    setMonth(p.month)
    setSelectedDate({ year: p.year, month: p.month, day: p.day })
    setSelectedTime({ hour: p.hour, minute: p.minute, second: p.second })
  }, [timeZone])

  const handleYearChange = (y: number) => {
    setYear(y)
    setSelectedDate((prev) => ({
      year: y,
      month: prev.month,
      day: Math.min(prev.day, daysInMonth(y, prev.month)),
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
      day: Math.min(prev.day, daysInMonth(newYear, newMonth)),
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
      day: Math.min(prev.day, daysInMonth(newYear, newMonth)),
    }))
    setIsLiveMode(false)
  }
  const goToday = () => {
    setYear(today.year)
    setMonth(today.month)
    setSelectedDate(today)
    const n = new Date()
    setSelectedTime({ hour: n.getHours(), minute: n.getMinutes(), second: n.getSeconds() })
    setIsLiveMode(true)
  }

  const sel = selectedDate
  const displayDate = isLiveMode ? nowForLunar : new Date(sel.year, sel.month - 1, sel.day, selectedTime.hour, selectedTime.minute, selectedTime.second)
  const displayY = displayDate.getFullYear()
  const displayM = displayDate.getMonth() + 1
  const displayD = displayDate.getDate()
  const displayHour = displayDate.getHours()
  const displayMinute = displayDate.getMinutes()
  const displaySecond = displayDate.getSeconds()
  const isSelectedToday = displayY === today.year && displayM === today.month && displayD === today.day
  const weekdayIndex = getWeekday(displayY, displayM, displayD)
  const dayOfYear = getDayOfYear(displayY, displayM, displayD)
  const weekOfYear = getWeekOfYear(displayY, displayM, displayD)

  /** 左侧面板用：显示日期与日历同月时用当前月岁数据，否则用单独拉取的 displayDateYearDataList */
  const panelYearDataList =
    displayY === year && displayM === month ? yearDataList : displayDateYearDataList

  const selectedLunar: string | null =
    displayY === year && displayM === month
      ? (lunarByDay[displayD] ?? null)
      : wasm && panelYearDataList.length > 0
        ? formatLunarWithFallback(wasm, displayY, displayM, displayD, panelYearDataList, displayY, displayM)
        : null
  const selectedLunarSlot = wasm && panelYearDataList.length > 0
    ? getLunarSlotForDay(wasm, displayY, displayM, displayD, panelYearDataList)
    : null
  const selectedGanzhi =
    wasm
      ? displayY === year && displayM === month
        ? (ganzhiByDay[displayD] ?? getGanzhiForDay(wasm, displayY, displayM, displayD, panelYearDataList, ganzhiOptions))
        : getGanzhiForDay(wasm, displayY, displayM, displayD, panelYearDataList, ganzhiOptions)
      : null
  const zodiac =
    selectedGanzhi?.yearName?.length === 2
      ? ZODIAC_BY_BRANCH[selectedGanzhi.yearName[1]!]
      : null

  const hourGanzhi = selectedGanzhi?.dayName && /^[\u4e00-\u9fa5]{2}$/.test(selectedGanzhi.dayName)
    ? getHourGanzhi(selectedGanzhi.dayName, displayHour)
    : ''

  const clockStr = isLiveMode
    ? '' /* 实时模式用 LiveClock 组件显示 */
    : `${displayY}年${displayM}月${displayD}日 ${selectedTime.hour.toString().padStart(2, '0')}:${selectedTime.minute.toString().padStart(2, '0')}:${selectedTime.second.toString().padStart(2, '0')} ${timeZone}`

  const yearOptions = Array.from({ length: 21 }, (_, i) => ({ label: `${today.year - 10 + i}年`, value: today.year - 10 + i }))
  const timeZoneOptions = useMemo(() => getTimeZoneOptions(), [])
  const weekStartOptions = WEEKDAY_NAMES.map((name, idx) => ({ label: `周${name}`, value: idx }))
  const ganzhiPresetOptions = [
    ...GANZHI_PRESET_NAMES.map((name, idx) => ({ label: name, value: idx })),
    { label: '自定义', value: 4 },
  ]

  if (view === 'graph') {
    return (
      <div style={{ maxWidth: 1200, margin: '0 auto', padding: 24, overflow: 'visible' }}>
        <Flex align="center" gap={12} style={{ marginBottom: 16 }}>
          <Button type={view === 'calendar' ? 'link' : 'primary'} icon={<CalendarOutlined />} onClick={() => setView('calendar')}>
            日历
          </Button>
          <Button type={view === 'graph' ? 'primary' : 'link'} icon={<ApartmentOutlined />} onClick={() => setView('graph')}>
            视位置变换图
          </Button>
        </Flex>
        <TransformGraphPage wasm={wasm} />
      </div>
    )
  }

  return (
    <div style={{ maxWidth: 1200, margin: '0 auto', padding: 24, overflow: 'visible' }}>
      <Flex align="center" gap={12} style={{ marginBottom: 16 }}>
        <Button type="primary" icon={<CalendarOutlined />} onClick={() => setView('calendar')}>
          日历
        </Button>
        <Button type="link" icon={<ApartmentOutlined />} onClick={() => setView('graph')}>
          视位置变换图
        </Button>
      </Flex>
      <Spin spinning={!!loading} tip={loading ? `加载${loading}…` : undefined}>
        {error && <Alert type="error" message={error} showIcon style={{ marginBottom: 16 }} />}

        <Flex gap={24} align="flex-start" wrap="wrap">
          {/* 左侧：单日信息（合并）。LiveTimeProvider 使实时模式下时钟与时宪历共用同一时间源 */}
          <LiveTimeProvider enabled={isLiveMode}>
          <Flex vertical gap={12} style={{ width: 260, flexShrink: 0, overflow: 'visible' }}>
            <Card size="small" style={{ width: '100%', overflow: 'visible' }} styles={{ body: { padding: 0, overflow: 'visible' } }}>
              {/* 顶部：年月条（撕页日历式） */}
              <div style={{ padding: '10px 12px', borderBottom: '1px solid rgba(0,0,0,0.06)', textAlign: 'center' }}>
                <Text type="secondary" style={{ fontSize: 11 }}>{year}年{month}月</Text>
                {selectedLunarSlot != null && (
                  <Text type="secondary" style={{ fontSize: 11, marginLeft: 8 }}>
                    {selectedLunarSlot.isLeapMonth ? '闰' : ''}{LUNAR_MONTH_NAMES[selectedLunarSlot.lunarMonth - 1]}{selectedLunarSlot.daysInMonth === 30 ? '大' : '小'}
                  </Text>
                )}
              </div>
              {/* 中央：大号日期（视觉焦点） */}
              <div style={{ padding: '16px 12px 8px', textAlign: 'center' }}>
                <div style={{ fontSize: 48, fontWeight: 700, lineHeight: 1, color: 'rgba(0,0,0,0.88)', fontVariantNumeric: 'tabular-nums' }}>
                  {sel.day}
                </div>
                {isLiveMode && (
                  <div style={{ fontSize: 13, marginTop: 6, fontVariantNumeric: 'tabular-nums' }}>
                    <LiveClock timeZone={timeZone} />
                  </div>
                )}
                {!isLiveMode && (
                  <Text type="secondary" style={{ fontSize: 12, marginTop: 4 }}>{clockStr}</Text>
                )}
              </div>
              {/* 农历 + 星期（主日期下方，并列突出） */}
              <div style={{ padding: '0 12px 12px', borderBottom: '1px solid rgba(0,0,0,0.06)' }}>
                <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>
                  {selectedLunarSlot != null ? (
                    <>农历{yearToChinese(selectedLunarSlot.lunarYear)}年{selectedLunarSlot.isLeapMonth ? '闰' : ''}{LUNAR_MONTH_NAMES[selectedLunarSlot.lunarMonth - 1]}月{selectedLunarSlot.daysInMonth === 30 ? '大' : '小'} {lunarDayStr(selectedLunarSlot.dayOfMonth)}日</>
                  ) : selectedLunar != null ? (
                    <>农历 {selectedLunar}</>
                  ) : (
                    <Text type="secondary">农历 —</Text>
                  )}
                </div>
                <div style={{ fontSize: 13, fontWeight: 600 }}>
                  星期{WEEKDAY_NAMES[weekdayIndex]}
                </div>
                <Text type="secondary" style={{ fontSize: 11 }}>第{dayOfYear}天，第{weekOfYear}周</Text>
                {isSelectedToday && isLiveMode && <Tag color="blue" style={{ marginTop: 6 }}>今天</Tag>}
              </div>
              {/* 底部：干支历、时宪历（紧凑网格块） */}
              <div style={{ padding: '10px 12px 12px', display: 'grid', gap: '8px 12px', gridTemplateColumns: '1fr 1fr', alignItems: 'start' }}>
                <div>
                  <Text type="secondary" style={{ fontSize: 10, display: 'block', marginBottom: 2 }}>干支历</Text>
                  <div style={{ fontSize: 12, lineHeight: 1.5, fontVariantNumeric: 'tabular-nums' }}>
                    {[
                      selectedGanzhi?.yearName && /^[\u4e00-\u9fa5]{2}$/.test(selectedGanzhi.yearName) ? `${selectedGanzhi.yearName}${zodiac != null ? `[${zodiac}]` : ''}年` : '—',
                      selectedGanzhi?.monthName && /^[\u4e00-\u9fa5]{2}$/.test(selectedGanzhi.monthName) ? `${selectedGanzhi.monthName}月` : '—',
                      selectedGanzhi?.dayName != null && /^[\u4e00-\u9fa5]{2}$/.test(selectedGanzhi?.dayName ?? '') ? `${selectedGanzhi.dayName}日` : '—',
                      hourGanzhi ? `${hourGanzhi}时` : '—',
                    ].join(' ')}
                  </div>
                </div>
                <div>
                  <Text type="secondary" style={{ fontSize: 10, display: 'block', marginBottom: 2 }}>时宪历</Text>
                  <div style={{ fontSize: 12, lineHeight: 1.5, fontVariantNumeric: 'tabular-nums' }}>
                    {isLiveMode ? <ShixianFromLiveTime /> : formatShixianKe(displayHour, displayMinute, displaySecond)}
                  </div>
                </div>
              </div>
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
                          <Select value={year} onChange={handleYearChange} options={yearOptions} style={{ width: '100%', marginTop: 4 }} />
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
                            options={Array.from({ length: daysInMonth(year, month) }, (_, i) => ({
                              label: `${i + 1}日`,
                              value: i + 1,
                            }))}
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
                        <Button type="primary" icon={<CalendarOutlined />} onClick={goToday} block>今天</Button>
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
                        onChange={(e) => setGanzhiOptions((o) => [e.target.value, o[1], o[2], o[3]])}
                        options={GANZHI_YEAR_BOUNDARY_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div style={{ marginBottom: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>月界</Text>
                      <Radio.Group
                        value={ganzhiOptions[1]}
                        onChange={(e) => setGanzhiOptions((o) => [o[0], e.target.value, o[2], o[3]])}
                        options={GANZHI_MONTH_BOUNDARY_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div style={{ marginBottom: 12 }}>
                      <Text type="secondary" style={{ fontSize: 11 }}>闰月</Text>
                      <Radio.Group
                        value={ganzhiOptions[2]}
                        onChange={(e) => setGanzhiOptions((o) => [o[0], o[1], e.target.value, o[3]])}
                        options={GANZHI_LEAP_MONTH_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                    <div>
                      <Text type="secondary" style={{ fontSize: 11 }}>日界</Text>
                      <Radio.Group
                        value={ganzhiOptions[3]}
                        onChange={(e) => setGanzhiOptions((o) => [o[0], o[1], o[2], e.target.value])}
                        options={GANZHI_DAY_BOUNDARY_OPTIONS.map(({ value, label }) => ({ value, label }))}
                        style={{ marginTop: 4, display: 'block' }}
                      />
                    </div>
                  </div>
                </div>
              </Flex>
            </Modal>
          </Flex>
        </Flex>
        {ephemerisSource != null && (
          <div style={{ marginTop: 12, paddingTop: 8, borderTop: '1px solid #f0f0f0' }}>
            <Text type="secondary" style={{ fontSize: 12 }}>
              {EPHEMERIS_SOURCE_LABELS[ephemerisSource]}
            </Text>
          </div>
        )}
      </Spin>
    </div>
  )
}

export default App
