/**
 * 3D 星图的视向与投影。这里的公式与 Sky3D.tsx 着色器里的一致，供文字标注与点选在 CPU 上用同一套算。
 *
 * 坐标：赤道单位向量 → 地方坐标（东、北、天顶）→ 视线坐标（右、上、前）。方位自北经东量，与 toHorizon 同。
 * 投影：球极（立体）投影，视场大时星官变形小。前方夹角 θ 处的屏幕半径 ∝ tan(θ/2)。
 */

export type Mat3 = [number, number, number, number, number, number, number, number, number] // 按行

const d2r = Math.PI / 180

export function unitVector(raDeg: number, decDeg: number): [number, number, number] {
  const r = raDeg * d2r
  const d = decDeg * d2r
  return [Math.cos(d) * Math.cos(r), Math.cos(d) * Math.sin(r), Math.sin(d)]
}

/** 赤道 → 地方坐标（东、北、天顶）。 */
export function equatorialToLocal(lstDeg: number, latDeg: number): Mat3 {
  const cl = Math.cos(lstDeg * d2r)
  const sl = Math.sin(lstDeg * d2r)
  const cp = Math.cos(latDeg * d2r)
  const sp = Math.sin(latDeg * d2r)
  return [
    -sl, cl, 0,
    -sp * cl, -sp * sl, cp,
    cp * cl, cp * sl, sp,
  ]
}

/** 地方坐标 → 视线坐标（右、上、前）。 */
export function localToView(azDeg: number, altDeg: number): Mat3 {
  const ca = Math.cos(azDeg * d2r)
  const sa = Math.sin(azDeg * d2r)
  const ch = Math.cos(altDeg * d2r)
  const sh = Math.sin(altDeg * d2r)
  return [
    ca, -sa, 0,
    -sa * sh, -ca * sh, ch,
    sa * ch, ca * ch, sh,
  ]
}

export function apply(m: Mat3, v: readonly number[]): [number, number, number] {
  return [m[0] * v[0] + m[1] * v[1] + m[2] * v[2], m[3] * v[0] + m[4] * v[1] + m[5] * v[2], m[6] * v[0] + m[7] * v[1] + m[8] * v[2]]
}

/** 竖直视场为 fov 时的投影系数：视场上缘（θ = fov/2）落在归一化坐标 1 处。 */
export const projectionK = (fovDeg: number) => 1 / Math.tan((fovDeg / 4) * d2r)

/** 视线坐标 → 屏幕像素（原点在画面中心，y 向上）。身后太远的点返回 null。 */
export function viewToPixel(v: readonly number[], k: number, halfHeight: number): [number, number] | null {
  if (v[2] < -0.9) return null
  const s = (k * halfHeight) / (1 + v[2])
  return [v[0] * s, v[1] * s]
}
