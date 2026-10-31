/** 各政权都城（data/phenomena/capitals.csv，经 npm run copy-data 拷到 /data/phenomena/），按都城名合并所属政权。 */
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
    const name = f[col('name')]
    const lon = Number(f[col('lon')])
    const lat = Number(f[col('lat')])
    if (!name || !Number.isFinite(lon)) continue
    const c = byName.get(name) ?? { name, lon, lat, regimes: [] }
    const regime = f[col('regime')]
    if (!c.regimes.includes(regime)) c.regimes.push(regime)
    byName.set(name, c)
  }
  return [...byName.values()]
}
