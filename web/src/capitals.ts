/** 各政权都城（data/phenomena/capitals.csv，经 npm run copy-data 拷到 /data/phenomena/），按都城名与坐标合并所属政权。
 * 同名异址（长安、洛阳各两城）分为两点，name 取 city 列（如「汉长安城」「隋唐洛阳城」）以相区别。 */
import { DATA_BASE } from './ephemerisUrls'

export type Capital = { name: string; lon: number; lat: number; regimes: string[] }

export async function loadCapitals(): Promise<Capital[]> {
  const r = await fetch(`${DATA_BASE}/phenomena/capitals.csv`)
  if (!r.ok) return []
  const rows = (await r.text()).split('\n').filter((l) => l && !l.startsWith('#'))
  const header = rows.shift()!.split(',')
  const col = (n: string) => header.indexOf(n)
  const byName = new Map<string, Capital>()
  for (const row of rows) {
    const f = row.split(',')
    const city = col('city') >= 0 ? (f[col('city')] ?? '').trim() : ''
    const name = city || f[col('name')]
    const lon = Number(f[col('lon')])
    const lat = Number(f[col('lat')])
    if (!name || !Number.isFinite(lon)) continue
    const key = `${name},${lon},${lat}`
    const c = byName.get(key) ?? { name, lon, lat, regimes: [] }
    const regime = f[col('regime')]
    if (!c.regimes.includes(regime)) c.regimes.push(regime)
    byName.set(key, c)
  }
  return [...byName.values()]
}
