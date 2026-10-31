/**
 * 民用历与儒略日互换（仅显示与输入用）：1582-10-15 起格里历，此前儒略历（与 Rust `calendar::gregorian::civil_to_julian_day`、
 * 实历朔日表同约定）；年份为天文纪年（0 = 前 1 年）。
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

/** 民用历日期是否已是格里历（1582-10-15 起）。 */
export function isGregorianCivil(y: number, m: number, d: number): boolean {
  return y > 1582 || (y === 1582 && (m > 10 || (m === 10 && d >= 15)))
}

/** 该月是否全部用格里历。此时民用历与计算用的前推格里历逐日相同，无须换算。 */
export function isFullyGregorianMonth(y: number, m: number): boolean {
  return y > 1582 || (y === 1582 && m > 10)
}

/**
 * 罗马实际闰年（公元前 45 年—公元 4 年）。祭司误为三年一闰，奥古斯都停闰后才与规则相合。
 * 哪几年闰学者说法不一，这里取 Bennett（ZPE 142, 2003；147, 2004）：前 44、41……8 年闰，1 月 1 日（前 45）合规则的前 46 年 12 月 31 日。
 * 其余方案见 doc/calendar/calendar-compendium.md。键为天文纪年。
 */
const ROMAN_LEAP_YEARS = new Set([44, 41, 38, 35, 32, 29, 26, 23, 20, 17, 14, 11, 8].map((bc) => 1 - bc))
const ROMAN_FIRST_YEAR = -44
const ROMAN_LAST_YEAR = 0
const isRomanEra = (y: number) => y >= ROMAN_FIRST_YEAR && y <= ROMAN_LAST_YEAR

/** 该年 1 月 1 日的 JD（罗马实际历）。 */
function romanYearStartJd(y: number): number {
  let jd = civilToJd(ROMAN_FIRST_YEAR - 1, 12, 31)
  for (let k = ROMAN_FIRST_YEAR; k < y; k++) jd += ROMAN_LEAP_YEARS.has(k) ? 366 : 365
  return jd
}

/**
 * 日历显示用的民用历 → JD：在 civilToJd 的规则之外还原罗马实际闰年。
 * 史料与 Rust 侧的日期仍用 civilToJd（实历朔日表约定），不走这里。
 */
export function displayCivilToJd(y: number, m: number, d: number): number {
  if (!isRomanEra(y)) return civilToJd(y, m, d)
  let jd = romanYearStartJd(y)
  for (let k = 1; k < m; k++) jd += civilMonthDays(y, k).length
  return jd + d - 1
}

/** 民用历某月实有的日子。1582 年 10 月没有 5—14 日；罗马实际闰年见 ROMAN_LEAP_YEARS。 */
export function civilMonthDays(y: number, m: number): number[] {
  const gregorian = isFullyGregorianMonth(y, m)
  const leap = gregorian
    ? (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0
    : isRomanEra(y) ? ROMAN_LEAP_YEARS.has(y) : y % 4 === 0
  const last = m === 2 ? (leap ? 29 : 28) : [4, 6, 9, 11].includes(m) ? 30 : 31
  const all = Array.from({ length: last }, (_, i) => i + 1)
  return y === 1582 && m === 10 ? all.filter((d) => d < 5 || d > 14) : all
}

/** 换月换年后把日子落到该月实有的日子上。 */
export function clampCivilDay(y: number, m: number, d: number): number {
  const days = civilMonthDays(y, m)
  return days.find((x) => x >= d) ?? days[days.length - 1]!
}

/** 民用历日期 → 前推格里历日期。内部计算（wasm 的 gregorian_* 接口）都用后者。 */
export function civilToProleptic(y: number, m: number, d: number): { year: number; month: number; day: number } {
  const z = Math.floor(displayCivilToJd(y, m, d) + 0.5)
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

/** 本地时间的 Date。不用 new Date(y, ...)，它会把 0—99 年当成 19xx 年。 */
export function localDate(y: number, m: number, d: number, h = 0, mi = 0, s = 0): Date {
  const t = new Date(2000, 0, 1, h, mi, s)
  t.setFullYear(y, m - 1, d)
  return t
}
