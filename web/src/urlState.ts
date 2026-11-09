/**
 * 网址即状态：#/<页>[/<页签>]?t=<年>-<月>-<日>T<时>:<分>&lat=<纬>&lon=<经>
 * 无 t 即实时；年为天文纪年（可负），日期为民用历。经纬度只取两位小数（约 1 公里），分享链接不暴露精确定位。见 doc/engineering/ui-interaction.md「网址即状态」。
 */
import type { ViewKey } from './AppHeader'

const VIEWS: ViewKey[] = ['calendar', 'graph', 'eclipse', 'stars', 'zeri', 'paipan']

export type UrlState = {
  view: ViewKey
  tab: string | null
  /** null 为实时 */
  t: { year: number; month: number; day: number; hour: number; minute: number } | null
  lat: number | null
  lon: number | null
}

const pad = (n: number) => String(n).padStart(2, '0')

export function parseHash(hash: string): UrlState {
  const h = hash.replace(/^#\/?/, '')
  const [path = '', query = ''] = h.split('?')
  const [v, tab] = path.split('/')
  const view = (VIEWS as string[]).includes(v ?? '') ? (v as ViewKey) : 'calendar'
  const q = new URLSearchParams(query)
  let t: UrlState['t'] = null
  const m = /^(-?\d+)-(\d{1,2})-(\d{1,2})T(\d{1,2}):(\d{1,2})$/.exec(q.get('t') ?? '')
  if (m) {
    const [year, month, day, hour, minute] = m.slice(1).map(Number) as [number, number, number, number, number]
    if (month >= 1 && month <= 12 && day >= 1 && day <= 31 && hour < 24 && minute < 60) t = { year, month, day, hour, minute }
  }
  const num = (k: string, lim: number) => {
    const s = q.get(k)
    if (s == null) return null
    const x = Number(s)
    return Number.isFinite(x) && Math.abs(x) <= lim ? x : null
  }
  return { view, tab: tab ? decodeURIComponent(tab) : null, t, lat: num('lat', 90), lon: num('lon', 180) }
}

export function buildHash(s: UrlState): string {
  const q = new URLSearchParams()
  const parts: string[] = []
  // t 不经 URLSearchParams，免把冒号编成 %3A
  if (s.t) parts.push(`t=${s.t.year}-${pad(s.t.month)}-${pad(s.t.day)}T${pad(s.t.hour)}:${pad(s.t.minute)}`)
  if (s.lat != null && s.lon != null) {
    q.set('lat', String(Math.round(s.lat * 100) / 100))
    q.set('lon', String(Math.round(s.lon * 100) / 100))
  }
  const qs = [...parts, q.toString()].filter(Boolean).join('&')
  return `#/${s.view}${s.tab ? `/${encodeURIComponent(s.tab)}` : ''}${qs ? `?${qs}` : ''}`
}
