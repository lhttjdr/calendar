/**
 * 星图公用部分：与 Rust 对应的数据类型、配色、星色星径、赤道与地平互换。盖图、地平图（SVG）与 3D 视图（three.js）共用；
 * 单放一个文件，三维视图才能单独分包。
 */

// ---------------------------------------------------------------------------------------------------------------------
// 数据类型（与 rust/wasm-lib/src/lib_impl.rs 星图部分的 JSON 对应）

export type CultureAsterism = { system: string; id: string; name: string; det: number | null; lines: number[][] }
export type Culture = {
  asterisms: CultureAsterism[]
  names: [string, number, string][]
  mansions: { system: string; determinatives: number[] }[]
  /** 陈卓体系星官的三家分属（据《开元占经》卷六十至七十） */
  schools: { id: string; school: string | null; basis: string; source: string }[]
}
export type BodyJson = { name: string; ra: number; dec: number; topoRa: number; topoDec: number; semidiameter: number | null; topoDistM: number; mag: number | null; phase: number | null }
export type SnapshotJson = {
  jdTt: number
  gastDeg: number
  eclipticPole: [number, number]
  stars: (number | null)[]
  bodies: BodyJson[]
  /** 全天亮度 lg(nL) 与极限星等，先高度后方位（Rust：SkyLight）；sunVisible 为日面未遮比例 */
  skyLight: { lgB: number[]; lim: number[]; sunVisible: number; moonGlare: number; sunGlare: number; limitCurve: number[]; coronaUnit: number }
  /** 当日赤道 → 银道，按行 */
  trueToGalactic: number[]
  mansionWidths: Record<string, number[]>
}
export type Star = { hip: number; ra: number; dec: number; v: number; bv: number | null }

/** 星官体系：距星表 id → 所用星官连线体系（《灵台仪象志》只改距星，星官仍用《仪象考成》）。 */
export const SYSTEMS: { id: string; label: string; asterisms: string }[] = [
  { id: 'chenzhuo', label: '陈卓（三国—晋，汇汉三家）', asterisms: 'chenzhuo' },
  { id: 'song_huangyou', label: '宋皇祐（1052）', asterisms: 'song_huangyou' },
  { id: 'yuan', label: '元《天文汇钞》', asterisms: 'yuan' },
  { id: 'ming_xianglin', label: '明《象林》（1634）', asterisms: 'ming_xianglin' },
  { id: 'qing_lingtai', label: '清《灵台仪象志》（1674）距星', asterisms: 'qing' },
  { id: 'qing', label: '清《仪象考成》（1744）', asterisms: 'qing' },
]

export const MANSIONS = '角亢氐房心尾箕斗牛女虚危室壁奎娄胃昴毕觜参井鬼柳星张翼轸'.split('')

export const BODY_LABEL: Record<string, string> = {
  sun: '日',
  moon: '月',
  mercury: '辰星',
  venus: '太白',
  mars: '荧惑',
  jupiter: '岁星',
  saturn: '填星',
}
export const BODY_COLOR: Record<string, string> = {
  sun: '#ffd34d',
  moon: '#e8e8e0',
  mercury: '#c9b8a0',
  venus: '#fff4c8',
  mars: '#ff7a4d',
  jupiter: '#f2d6a8',
  saturn: '#e6cf8f',
}

/** 三家传统配色：石氏赤、甘氏黑、巫咸黄（敦煌星图等）；深色底上甘氏改用白。 */
export const SCHOOL_COLOR: Record<string, string> = { 石氏: '#ff6b5a', 甘氏: '#e6e6e6', 巫咸: '#ffd23f' }

export const COLORS = {
  sky: '#0b1530',
  skyEdge: '#33456e',
  below: 'rgba(0,0,0,0.45)',
  asterism: 'rgba(120,160,220,0.75)',
  asterismName: '#9fb7e0',
  mansion: 'rgba(214,170,80,0.55)',
  mansionText: '#e0bd6a',
  equator: 'rgba(110,190,140,0.7)',
  ecliptic: 'rgba(240,180,70,0.8)',
  circle: 'rgba(170,180,210,0.45)',
  horizon: 'rgba(120,220,200,0.9)',
  text: '#cdd6e8',
  pick: '#ff6b6b',
}

export const d2r = Math.PI / 180
export const r2d = 180 / Math.PI
export const mod360 = (x: number) => ((x % 360) + 360) % 360

/** B−V → 星色（Ballesteros 近似的分段线性版）。 */
export function starColor(bv: number | null): string {
  if (bv == null) return '#f4f4ff'
  const stops: [number, [number, number, number]][] = [
    [-0.4, [155, 176, 255]],
    [0.0, [202, 215, 255]],
    [0.4, [248, 247, 255]],
    [0.8, [255, 236, 200]],
    [1.2, [255, 210, 161]],
    [1.8, [255, 180, 107]],
  ]
  const x = Math.max(-0.4, Math.min(1.8, bv))
  for (let i = 0; i < stops.length - 1; i++) {
    const [a, ca] = stops[i]
    const [b, cb] = stops[i + 1]
    if (x <= b) {
      const t = (x - a) / (b - a)
      const c = ca.map((v, k) => Math.round(v + (cb[k] - v) * t))
      return `rgb(${c[0]},${c[1]},${c[2]})`
    }
  }
  return 'rgb(255,180,107)'
}

export const starRadius = (v: number) => Math.max(0.55, 3.3 - 0.45 * v)

/** 赤道 → 地平（几何，不计折射）：方位自北经东量。 */
export function toHorizon(raDeg: number, decDeg: number, lstDeg: number, latDeg: number): { alt: number; az: number } {
  const h = (lstDeg - raDeg) * d2r
  const d = decDeg * d2r
  const p = latDeg * d2r
  const alt = Math.asin(Math.sin(p) * Math.sin(d) + Math.cos(p) * Math.cos(d) * Math.cos(h))
  const az = Math.atan2(-Math.cos(d) * Math.sin(h), Math.sin(d) * Math.cos(p) - Math.cos(d) * Math.cos(h) * Math.sin(p))
  return { alt: alt * r2d, az: mod360(az * r2d) }
}

/** 地平 → 赤道。 */
export function fromHorizon(altDeg: number, azDeg: number, lstDeg: number, latDeg: number): { ra: number; dec: number } {
  const a = altDeg * d2r
  const z = azDeg * d2r
  const p = latDeg * d2r
  const dec = Math.asin(Math.sin(p) * Math.sin(a) + Math.cos(p) * Math.cos(a) * Math.cos(z))
  const h = Math.atan2(-Math.sin(z) * Math.cos(a), Math.cos(p) * Math.sin(a) - Math.sin(p) * Math.cos(a) * Math.cos(z))
  return { ra: mod360(lstDeg - h * r2d), dec: dec * r2d }
}

/** 以 (raP, decP) 为极的大圆上 n 个点（赤经、赤纬，度）。 */
export function greatCircle(raP: number, decP: number, n = 360): { ra: number; dec: number }[] {
  const p = [Math.cos(decP * d2r) * Math.cos(raP * d2r), Math.cos(decP * d2r) * Math.sin(raP * d2r), Math.sin(decP * d2r)]
  // 与极正交的两单位向量
  const ref = Math.abs(p[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0]
  const a = [p[1] * ref[2] - p[2] * ref[1], p[2] * ref[0] - p[0] * ref[2], p[0] * ref[1] - p[1] * ref[0]]
  const na = Math.hypot(a[0], a[1], a[2])
  const A = a.map((x) => x / na)
  const B = [p[1] * A[2] - p[2] * A[1], p[2] * A[0] - p[0] * A[2], p[0] * A[1] - p[1] * A[0]]
  return Array.from({ length: n + 1 }, (_, k) => {
    const t = (2 * Math.PI * k) / n
    const v = [0, 1, 2].map((i) => Math.cos(t) * A[i] + Math.sin(t) * B[i])
    return { ra: mod360(Math.atan2(v[1], v[0]) * r2d), dec: Math.asin(Math.max(-1, Math.min(1, v[2]))) * r2d }
  })
}

export type Picked = { kind: 'star'; hip: number } | { kind: 'body'; name: string }

// 折射对照表：x = √(真高度 + 5°) 等距取样，地平附近密。Rust 算（refractionTable），显示时线性插值；着色器里是同一算法。
export const REFRACTION_SAMPLES = 99
const REFRACTION_STEP = 0.1
export const refractionTableAltitudes = () =>
  Array.from({ length: REFRACTION_SAMPLES }, (_, i) => Math.min(90, (i * REFRACTION_STEP) ** 2 - 5))

/** 真高度 → 折射量（度）。table 为 null 时不计折射。 */
export function refractionAt(table: number[] | null, altDeg: number): number {
  if (!table) return 0
  const x = Math.sqrt(Math.max(altDeg + 5, 0)) / REFRACTION_STEP
  const i = Math.min(Math.floor(x), REFRACTION_SAMPLES - 2)
  return table[i] + (table[i + 1] - table[i]) * (x - i)
}

/** 天光网格尺寸，与 Rust SKY_GRID_AZ、SKY_GRID_ALT 相同。高度 = 90°·(i/(n−1))²。 */
export const SKY_GRID_AZ = 25
export const SKY_GRID_ALT = 13

/** 极限星等曲线的取样与月亮眩光最小角距，与 Rust LIMIT_CURVE_*、MOON_GLARE_MIN_DEG 相同。 */
export const LIMIT_CURVE_SAMPLES = 41
export const LIMIT_CURVE_STEP = 0.25
export const MOON_GLARE_MIN_DEG = 0.26

/** 极限星等曲线在 lg 亮度处的值（线性插值）。 */
export function limitCurveAt(curve: number[], lg: number): number {
  const x = Math.max(0, Math.min(LIMIT_CURVE_SAMPLES - 1.001, lg / LIMIT_CURVE_STEP))
  const i = Math.floor(x)
  return curve[i] + (curve[i + 1] - curve[i]) * (x - i)
}

/** 地平线的方位取样数（每 0.5°）。 */
export const HORIZON_SAMPLES = 720

/** 地平线在方位 az（度）处的视高度（度）。 */
export function horizonAt(profile: number[] | null, azDeg: number): number {
  if (!profile) return 0
  const x = (((azDeg % 360) + 360) % 360) / 360 * HORIZON_SAMPLES
  const i = Math.floor(x) % HORIZON_SAMPLES
  return profile[i] + (profile[(i + 1) % HORIZON_SAMPLES] - profile[i]) * (x - Math.floor(x))
}
