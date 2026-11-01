/**
 * 民用历与儒略日互换（仅显示与输入用）：1582-10-15 起格里历，此前规则儒略历（与 Rust `calendar::gregorian::civil_to_julian_day`、
 * 实历朔日表同约定）；年份为天文纪年（0 = 前 1 年）。日历还可切换其他西历写法，见 CalendarMode。
 */

export type Civil = { y: number; m: number; d: number; h: number; mi: number; s: number }

/** JD → 民用历日期时刻（先取整到秒，避免 23:59:59.6 进位丢日）。 */
export function jdToCivil(jd: number): Civil {
  const jdRounded = Math.round(jd * 86400) / 86400
  const z = Math.floor(jdRounded + 0.5)
  const f = jdRounded + 0.5 - z
  let a = z
  if (z >= 2299161) {
    const alpha = Math.floor((z - 1867216.25) / 36524.25)
    a = z + 1 + alpha - Math.floor(alpha / 4)
  }
  const b = a + 1524
  const c = Math.floor((b - 122.1) / 365.25)
  const dd = Math.floor(365.25 * c)
  const e = Math.floor((b - dd) / 30.6001)
  const d = b - dd - Math.floor(30.6001 * e)
  const m = e < 14 ? e - 1 : e - 13
  const y = m > 2 ? c - 4716 : c - 4715
  const secs = Math.min(86399, Math.round(f * 86400))
  return { y, m, d, h: Math.floor(secs / 3600), mi: Math.floor((secs % 3600) / 60), s: secs % 60 }
}

/** 民用历日期 + 当日小时数 → JD。 */
export function civilToJd(y: number, m: number, d: number, hours = 0): number {
  let yy = y
  let mm = m
  if (mm <= 2) {
    yy -= 1
    mm += 12
  }
  const gregorian = y > 1582 || (y === 1582 && (m > 10 || (m === 10 && d >= 15)))
  const a = Math.floor(yy / 100)
  const b = gregorian ? 2 - a + Math.floor(a / 4) : 0
  return Math.floor(365.25 * (yy + 4716)) + Math.floor(30.6001 * (mm + 1)) + d + b - 1524.5 + hours / 24
}

export function yearLabel(y: number): string {
  return y <= 0 ? `前${1 - y}` : `${y}`
}

const pad = (n: number) => n.toString().padStart(2, '0')

export function fmtDate(jd: number): string {
  const c = jdToCivil(jd)
  return `${yearLabel(c.y)}-${pad(c.m)}-${pad(c.d)}`
}

export function fmtTime(jd: number, withSeconds = false): string {
  const c = jdToCivil(jd)
  return withSeconds ? `${pad(c.h)}:${pad(c.mi)}:${pad(c.s)}` : `${pad(c.h)}:${pad(c.mi)}`
}

/** 按规则的通用历（1582-10-15 起格里历，此前儒略历）是否已到格里历段。 */
const isGregorianByRule = (y: number, m: number, d: number) =>
  y > 1582 || (y === 1582 && (m > 10 || (m === 10 && d >= 15)))

/** 罗马实历的一种复原：列出公元前的闰年，end 为与规则相合的那一年（该年 3 月 1 日起相合）。 */
type RomanScheme = { label: string; leapBc: number[]; end: number }

const triennial = (first: number, last: number) => {
  const out: number[] = []
  for (let bc = first; bc >= last; bc -= 3) out.push(bc)
  return out
}

/** 各家方案及依据见 doc/calendar/calendar-compendium.md「西历的三种写法」。 */
export const ROMAN_SCHEMES = {
  bennett: { label: 'Bennett 2003', leapBc: triennial(44, 8), end: 0 },
  scaliger: { label: 'Scaliger 1583', leapBc: triennial(42, 9), end: 4 },
  ideler: { label: 'Ideler、Mommsen 1859', leapBc: triennial(45, 9), end: 4 },
  matzat: { label: 'Matzat 1883', leapBc: triennial(44, 11), end: 0 },
  soltau: { label: 'Soltau 1889', leapBc: [45, ...triennial(41, 11)], end: 4 },
  kepler: { label: 'Kepler 1614', leapBc: triennial(43, 10), end: 4 },
  radke: { label: 'Radke 1960', leapBc: triennial(45, 12), end: 0 },
  harriot: { label: 'Harriot', leapBc: triennial(43, 10), end: 0 },
  buenting: { label: 'Bünting 1590', leapBc: triennial(45, 12), end: 0 },
} satisfies Record<string, RomanScheme>

/**
 * 西历写法。civil：天文与史学通用，1582-10-15 前为规则儒略历，不依赖古代实历（默认，与 Rust、实历朔日表同约定）；
 * iso：ISO 8601 的前推格里历；其余：在 civil 之上还原公元前 45 年起罗马实际闰年。
 */
export type CalendarMode = 'civil' | 'iso' | keyof typeof ROMAN_SCHEMES

export const isCalendarMode = (v: unknown): v is CalendarMode =>
  v === 'civil' || v === 'iso' || (typeof v === 'string' && Object.prototype.hasOwnProperty.call(ROMAN_SCHEMES, v))

let mode: CalendarMode = 'civil'

/** 切换日历所用的西历写法。以下 display 系列函数都按它算；史料对照与 Rust 侧不受影响。 */
export function setCalendarMode(next: CalendarMode) {
  mode = next
}

const ROMAN_FIRST_YEAR = -44

function currentScheme(): { leap: Set<number>; end: number } | null {
  if (mode === 'civil' || mode === 'iso') return null
  const s = ROMAN_SCHEMES[mode]
  return { leap: new Set(s.leapBc.map((bc) => 1 - bc)), end: s.end }
}

/** 该年 1 月 1 日的 JD（罗马实历）。从相合点倒推，故首日由闰年序列决定。 */
function romanYearStartJd(y: number, scheme: { leap: Set<number>; end: number }): number {
  let jd = civilToJd(scheme.end, 3, 1) - 59
  for (let k = scheme.end - 1; k >= y; k--) jd -= scheme.leap.has(k) ? 366 : 365
  return jd
}

/** 前推格里历日期 → JD。 */
function prolepticToJd(y: number, m: number, d: number): number {
  const yy = m <= 2 ? y - 1 : y
  const mm = m <= 2 ? m + 12 : m
  const a = Math.floor(yy / 100)
  return Math.floor(365.25 * (yy + 4716)) + Math.floor(30.6001 * (mm + 1)) + d + 2 - a + Math.floor(a / 4) - 1524.5
}

function prolepticFromJd(jd: number): { year: number; month: number; day: number } {
  const z = Math.floor(jd + 0.5)
  const alpha = Math.floor((z - 1867216.25) / 36524.25)
  const a = z + 1 + alpha - Math.floor(alpha / 4)
  const b = a + 1524
  const c = Math.floor((b - 122.1) / 365.25)
  const dd = Math.floor(365.25 * c)
  const e = Math.floor((b - dd) / 30.6001)
  const day = b - dd - Math.floor(30.6001 * e)
  const month = e < 14 ? e - 1 : e - 13
  return { year: month > 2 ? c - 4716 : c - 4715, month, day }
}

/** 日历上的日期是否已用格里历写法。 */
export function isGregorianCivil(y: number, m: number, d: number): boolean {
  return mode === 'iso' || isGregorianByRule(y, m, d)
}

/** 该月是否全部用格里历。此时显示日期与计算用的前推格里历逐日相同，无须换算。 */
export function isFullyGregorianMonth(y: number, m: number): boolean {
  return mode === 'iso' || y > 1582 || (y === 1582 && m > 10)
}

/** 日历显示的日期 → JD，按当前西历写法。史料与 Rust 侧的日期用 civilToJd，不走这里。 */
export function displayCivilToJd(y: number, m: number, d: number): number {
  if (mode === 'iso') return prolepticToJd(y, m, d)
  const scheme = currentScheme()
  if (!scheme || y < ROMAN_FIRST_YEAR || y > scheme.end) return civilToJd(y, m, d)
  let jd = romanYearStartJd(y, scheme)
  for (let k = 1; k < m; k++) jd += civilMonthDays(y, k).length
  return jd + d - 1
}

/** JD → 日历显示的日期，切换写法时用来留在同一天。 */
export function jdToDisplayDate(jd: number): { year: number; month: number; day: number } {
  if (mode === 'iso') return prolepticFromJd(jd)
  const rule = jdToCivil(jd)
  const scheme = currentScheme()
  if (scheme && rule.y >= ROMAN_FIRST_YEAR - 1 && rule.y <= scheme.end) {
    if (jd < romanYearStartJd(ROMAN_FIRST_YEAR, scheme)) {
      // 首日晚于规则 1 月 1 日的方案，规则前 45 年 1 月 1 日在实历里没有写法，落到实历首日
      return rule.y === ROMAN_FIRST_YEAR ? { year: ROMAN_FIRST_YEAR, month: 1, day: 1 } : { year: rule.y, month: rule.m, day: rule.d }
    }
    for (let y = ROMAN_FIRST_YEAR; y <= scheme.end; y++) {
      if (jd >= romanYearStartJd(y + 1, scheme) && y < scheme.end) continue
      if (y === scheme.end && jd >= civilToJd(y, 3, 1)) break
      let rest = Math.floor(jd - romanYearStartJd(y, scheme))
      for (let m = 1; m <= 12; m++) {
        const n = civilMonthDays(y, m).length
        if (rest < n) return { year: y, month: m, day: rest + 1 }
        rest -= n
      }
    }
  }
  return { year: rule.y, month: rule.m, day: rule.d }
}

/** 这一天的西历名称。 */
export function displayCalendarName(y: number, m: number, d: number): string {
  if (isGregorianByRule(y, m, d)) return '格里高利历'
  if (mode === 'iso') return '前推格里高利历'
  // 儒略历行用于公元前 45 年（天文纪年 -44）起，再往前是前推，历法本身尚不存在
  if (y < ROMAN_FIRST_YEAR) return '前推儒略历'
  if (mode === 'civil') return '儒略历'
  const s = ROMAN_SCHEMES[mode]
  return y < s.end || (y === s.end && m <= 2) ? `罗马实历（${s.label}）` : '儒略历'
}

/** 日历上某月实有的日子。通用写法 1582 年 10 月没有 5—14 日。 */
export function civilMonthDays(y: number, m: number): number[] {
  const scheme = currentScheme()
  const leap = isFullyGregorianMonth(y, m)
    ? (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0
    : scheme && y >= ROMAN_FIRST_YEAR && y <= scheme.end ? scheme.leap.has(y) : y % 4 === 0
  const last = m === 2 ? (leap ? 29 : 28) : [4, 6, 9, 11].includes(m) ? 30 : 31
  const all = Array.from({ length: last }, (_, i) => i + 1)
  return mode !== 'iso' && y === 1582 && m === 10 ? all.filter((d) => d < 5 || d > 14) : all
}

/** 换月换年后把日子落到该月实有的日子上。 */
export function clampCivilDay(y: number, m: number, d: number): number {
  const days = civilMonthDays(y, m)
  return days.find((x) => x >= d) ?? days[days.length - 1]!
}

/** 日历显示的日期 → 前推格里历日期。内部计算（wasm 的 gregorian_* 接口）都用后者。 */
export function civilToProleptic(y: number, m: number, d: number): { year: number; month: number; day: number } {
  return prolepticFromJd(displayCivilToJd(y, m, d))
}

/** 本地时间的 Date。不用 new Date(y, ...)，它会把 0—99 年当成 19xx 年。 */
export function localDate(y: number, m: number, d: number, h = 0, mi = 0, s = 0): Date {
  const t = new Date(2000, 0, 1, h, mi, s)
  t.setFullYear(y, m - 1, d)
  return t
}

export type DateParts = { year: number; month: number; day: number; hour: number; minute: number; second: number }

/**
 * Date 在指定时区的年月日时分秒。日期是前推格里历，年份是天文纪年（Intl 本身不带负年，要从纪元换算）。
 * 1900 年前后以前，Intl 按该时区的地方平时计算（如上海 +8:05:43）。
 */
export function formatDateInTimeZone(d: Date, timeZone: string): DateParts {
  const parts = new Intl.DateTimeFormat('en-US', {
    timeZone,
    calendar: 'gregory',
    era: 'short',
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
    second: 'numeric',
    hourCycle: 'h23',
  }).formatToParts(d)
  const get = (type: string) => parseInt(parts.find((p) => p.type === type)?.value ?? '0', 10)
  const bc = parts.find((p) => p.type === 'era')?.value.startsWith('B') ?? false
  return { year: bc ? 1 - get('year') : get('year'), month: get('month'), day: get('day'), hour: get('hour'), minute: get('minute'), second: get('second') }
}

/** 前推格里历的年月日时分秒按 UTC 读成毫秒。不用 Date.UTC(y, ...)，它会把 0—99 年当成 19xx 年。 */
function prolepticUtcMs(p: DateParts): number {
  const t = new Date(0)
  t.setUTCFullYear(p.year, p.month - 1, p.day)
  t.setUTCHours(p.hour, p.minute, p.second, 0)
  return t.getTime()
}

/** 某时区的本地时间（前推格里历）→ 时刻。 */
export function utcFromLocalInZone(local: DateParts, timeZone: string): Date {
  const wall = prolepticUtcMs(local)
  const offsetAt = (ms: number) => prolepticUtcMs(formatDateInTimeZone(new Date(ms), timeZone)) - ms
  // 第二次按算出的时刻重取偏移，跨夏令时切换时才准
  return new Date(wall - offsetAt(wall - offsetAt(wall)))
}

/** 日历上的日期时刻从一个时区换到另一个时区，保持同一时刻；进出都按当前西历写法。 */
export function convertDisplayDateTime(y: number, m: number, d: number, h: number, mi: number, s: number, from: string, to: string): DateParts {
  const p = formatDateInTimeZone(utcFromLocalInZone({ ...civilToProleptic(y, m, d), hour: h, minute: mi, second: s }, from), to)
  return { ...p, ...jdToDisplayDate(prolepticToJd(p.year, p.month, p.day)) }
}
