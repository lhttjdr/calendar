/**
 * 3D 星图：站在当地仰望，拖动转向、滚轮或双指缩放。
 *
 * 天体位置仍由 Rust 算（同盖图），这里只做显示：星和线存赤道单位向量，转视角只改一个矩阵，不重算也不重建几何。
 * 投影是顶点着色器里的球极投影（公式见 skyProjection.ts）。文字和选中标记画在上层的 2D 画布，用同一公式算坐标。
 */
import { useEffect, useMemo, useRef, useState } from 'react'
import * as THREE from 'three'
import { BODY_COLOR, BODY_LABEL, COLORS, MANSIONS, SCHOOL_COLOR, SYSTEMS, greatCircle, starColor, starRadius, type Culture, type Picked, type SnapshotJson, type Star } from './skyCommon'
import { apply, equatorialToLocal, localToView, multiply, projectionK, unitVector, viewToPixel, type Mat3 } from './skyProjection'

const MIN_FOV = 5
const MAX_FOV = 180
const GROUND = '#1a2118'
const GROUND_FAR = '#0c110b'
const CARDINALS: [string, number][] = [['北', 0], ['东北', 45], ['东', 90], ['东南', 135], ['南', 180], ['西南', 225], ['西', 270], ['西北', 315]]

const COMMON_GLSL = /* glsl */ `
  uniform mat3 uEqToView;
  uniform vec3 uUpRow;
  uniform float uK;
  uniform vec2 uHalf;
  vec2 toPixel(vec3 v) { return v.xy * uK * uHalf.y / max(1.0 + v.z, 0.02); }
`

const POINT_VERT = /* glsl */ `
  ${COMMON_GLSL}
  uniform float uGround;
  uniform float uScale;
  attribute vec3 aColor;
  attribute float aSize;
  varying vec3 vColor;
  void main() {
    vec3 v = uEqToView * position;
    vColor = aColor;
    if (v.z < -0.9 || (uGround > 0.5 && dot(uUpRow, position) < 0.0)) {
      gl_Position = vec4(2.0, 2.0, 0.0, 1.0);
      gl_PointSize = 0.0;
      return;
    }
    gl_Position = vec4(toPixel(v) / uHalf, 0.0, 1.0);
    gl_PointSize = aSize * uScale;
  }
`
const STAR_FRAG = /* glsl */ `
  precision highp float;
  varying vec3 vColor;
  void main() {
    float r = length(gl_PointCoord - 0.5) * 2.0;
    if (r > 1.0) discard;
    float core = 1.0 - smoothstep(0.25, 0.55, r);
    float glow = pow(1.0 - r, 2.0) * 0.45;
    gl_FragColor = vec4(vColor, clamp(core + glow, 0.0, 1.0));
  }
`
const BODY_FRAG = /* glsl */ `
  precision highp float;
  varying vec3 vColor;
  void main() {
    float r = length(gl_PointCoord - 0.5) * 2.0;
    if (r > 1.0) discard;
    gl_FragColor = vec4(vColor, 1.0 - smoothstep(0.86, 1.0, r));
  }
`

// 线段不用 gl.LINES（线宽基本只有 1 像素）：每段实例化成一个四边形，在屏幕上展开成定宽
const LINE_VERT = /* glsl */ `
  ${COMMON_GLSL}
  uniform float uWidth;
  attribute vec3 iStart;
  attribute vec3 iEnd;
  attribute vec4 iColor;
  varying vec4 vColor;
  varying float vAcross;
  varying float vAlt;
  void main() {
    vec3 a = uEqToView * iStart;
    vec3 b = uEqToView * iEnd;
    vColor = iColor;
    vAcross = position.y;
    vAlt = dot(uUpRow, mix(iStart, iEnd, position.x));
    if (a.z < -0.9 || b.z < -0.9) { gl_Position = vec4(2.0, 2.0, 0.0, 1.0); return; }
    vec2 pa = toPixel(a);
    vec2 pb = toPixel(b);
    vec2 d = pb - pa;
    float n = length(d);
    vec2 nor = n > 1e-4 ? vec2(-d.y, d.x) / n : vec2(0.0);
    vec2 p = mix(pa, pb, position.x) + nor * position.y * uWidth * 0.5;
    gl_Position = vec4(p / uHalf, 0.0, 1.0);
  }
`
const LINE_FRAG = /* glsl */ `
  precision highp float;
  uniform float uGround;
  varying vec4 vColor;
  varying float vAcross;
  varying float vAlt;
  void main() {
    if (uGround > 0.5 && vAlt < 0.0) discard;
    gl_FragColor = vec4(vColor.rgb, vColor.a * (1.0 - smoothstep(0.5, 1.0, abs(vAcross))));
  }
`

// 全屏底图：逐像素反算视线方向，高度角小于 0 处画地面，并描地平线
const BACKDROP_VERT = /* glsl */ `
  varying vec2 vNdc;
  void main() {
    vNdc = position.xy;
    gl_Position = vec4(position.xy, 0.0, 1.0);
  }
`
const BACKDROP_FRAG = /* glsl */ `
  precision highp float;
  uniform float uK;
  uniform vec2 uHalf;
  uniform vec3 uUpView;
  uniform float uGround;
  uniform vec3 uSky;
  uniform vec3 uGroundNear;
  uniform vec3 uGroundFar;
  uniform vec3 uHorizon;
  varying vec2 vNdc;
  void main() {
    vec2 s = vNdc * vec2(uHalf.x / uHalf.y, 1.0) / uK;
    float rho2 = dot(s, s);
    vec3 v = vec3(2.0 * s, 1.0 - rho2) / (1.0 + rho2);
    float up = dot(uUpView, v);
    vec3 c = uSky;
    if (uGround > 0.5) {
      if (up < 0.0) c = mix(uGroundNear, uGroundFar, clamp(-up * 1.6, 0.0, 1.0));
      float w = fwidth(up);
      c = mix(c, uHorizon, 1.0 - smoothstep(w * 0.6, w * 1.6, abs(up)));
    }
    gl_FragColor = vec4(c, 1.0);
  }
`

type Rgba = [number, number, number, number]

function parseColor(c: string): Rgba {
  const m = c.match(/rgba?\(([^)]+)\)/)
  if (m) {
    const p = m[1].split(',').map(Number)
    return [p[0] / 255, p[1] / 255, p[2] / 255, p[3] ?? 1]
  }
  // 不用 THREE.Color：它会把十六进制色转成线性空间，而这里输出的就是显示空间的值
  const n = parseInt(c.slice(1), 16)
  return [(n >> 16) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255, 1]
}

/** 线段收集器：两点间按大圆细分，球极投影下长线才是弯的。 */
class Segments {
  start: number[] = []
  end: number[] = []
  color: number[] = []
  add(a: number[], b: number[], c: Rgba) {
    const dot = Math.max(-1, Math.min(1, a[0] * b[0] + a[1] * b[1] + a[2] * b[2]))
    const angle = Math.acos(dot)
    const n = Math.max(1, Math.ceil(angle / (3 * Math.PI / 180)))
    let p = a
    for (let i = 1; i <= n; i++) {
      const t = i / n
      const q = [0, 1, 2].map((k) => a[k] + (b[k] - a[k]) * t)
      const len = Math.hypot(q[0], q[1], q[2])
      const u = q.map((x) => x / len)
      this.start.push(p[0], p[1], p[2])
      this.end.push(u[0], u[1], u[2])
      this.color.push(c[0], c[1], c[2], c[3])
      p = u
    }
  }
  /** 依次连接；dashed 时隔段跳过。 */
  polyline(pts: { ra: number; dec: number }[], c: Rgba, dashed = false) {
    for (let i = 0; i + 1 < pts.length; i++) {
      if (dashed && i % 2) continue
      this.add(unitVector(pts[i].ra, pts[i].dec), unitVector(pts[i + 1].ra, pts[i + 1].dec), c)
    }
  }
}

type Props = {
  stars: Star[]
  starMap: Map<number, Star>
  snap: SnapshotJson
  culture: Culture
  systemId: string
  lstDeg: number
  lat: number
  magLimit: number
  layers: Set<string>
  zhouTian: number
  picked: Picked | null
  onPick: (p: Picked | null) => void
  playing: boolean
}

type Engine = {
  renderer: THREE.WebGLRenderer
  scene: THREE.Scene
  camera: THREE.Camera
  shared: {
    uEqToView: { value: THREE.Matrix3 }
    uUpRow: { value: THREE.Vector3 }
    uK: { value: number }
    uHalf: { value: THREE.Vector2 }
    uGround: { value: number }
    uScale: { value: number }
  }
  upView: { value: THREE.Vector3 }
  lineMaterial: THREE.ShaderMaterial
  starMaterial: THREE.ShaderMaterial
  bodyMaterial: THREE.ShaderMaterial
  lines: THREE.Mesh | null
  starPoints: THREE.Points | null
  bodyPoints: THREE.Points | null
}

function pointGeometry(vecs: number[], colors: number[], sizes: number[]) {
  const g = new THREE.BufferGeometry()
  g.setAttribute('position', new THREE.Float32BufferAttribute(vecs, 3))
  g.setAttribute('aColor', new THREE.Float32BufferAttribute(colors, 3))
  g.setAttribute('aSize', new THREE.Float32BufferAttribute(sizes, 1))
  return g
}

function createEngine(canvas: HTMLCanvasElement): Engine {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false })
  renderer.outputColorSpace = THREE.LinearSRGBColorSpace // 配色都是显示空间的值，不再转换
  const scene = new THREE.Scene()
  const camera = new THREE.Camera() // 位置全由着色器算，相机不起作用
  const shared = {
    uEqToView: { value: new THREE.Matrix3() },
    uUpRow: { value: new THREE.Vector3(0, 0, 1) },
    uK: { value: 1 },
    uHalf: { value: new THREE.Vector2(1, 1) },
    uGround: { value: 1 },
    uScale: { value: 1 },
  }
  const upView = { value: new THREE.Vector3(0, 1, 0) }
  const rgb = (c: string) => {
    const [r, g, b] = parseColor(c)
    return { value: new THREE.Vector3(r, g, b) }
  }
  const backdrop = new THREE.Mesh(
    new THREE.BufferGeometry().setAttribute('position', new THREE.Float32BufferAttribute([-1, -1, 0, 3, -1, 0, -1, 3, 0], 3)),
    new THREE.ShaderMaterial({
      vertexShader: BACKDROP_VERT,
      fragmentShader: BACKDROP_FRAG,
      depthTest: false,
      depthWrite: false,
      uniforms: {
        uK: shared.uK,
        uHalf: shared.uHalf,
        uGround: shared.uGround,
        uUpView: upView,
        uSky: rgb(COLORS.sky),
        uGroundNear: rgb(GROUND),
        uGroundFar: rgb(GROUND_FAR),
        uHorizon: rgb(COLORS.horizon),
      },
    })
  )
  backdrop.frustumCulled = false
  backdrop.renderOrder = 0
  scene.add(backdrop)
  const blend = { transparent: true, depthTest: false, depthWrite: false }
  const lineMaterial = new THREE.ShaderMaterial({
    vertexShader: LINE_VERT,
    fragmentShader: LINE_FRAG,
    ...blend,
    uniforms: { uEqToView: shared.uEqToView, uUpRow: shared.uUpRow, uK: shared.uK, uHalf: shared.uHalf, uGround: shared.uGround, uWidth: { value: 1.5 } },
  })
  const pointUniforms = { uEqToView: shared.uEqToView, uUpRow: shared.uUpRow, uK: shared.uK, uHalf: shared.uHalf, uGround: shared.uGround, uScale: shared.uScale }
  const starMaterial = new THREE.ShaderMaterial({ vertexShader: POINT_VERT, fragmentShader: STAR_FRAG, ...blend, blending: THREE.AdditiveBlending, uniforms: pointUniforms })
  const bodyMaterial = new THREE.ShaderMaterial({ vertexShader: POINT_VERT, fragmentShader: BODY_FRAG, ...blend, uniforms: pointUniforms })
  return { renderer, scene, camera, shared, upView, lineMaterial, starMaterial, bodyMaterial, lines: null, starPoints: null, bodyPoints: null }
}

function replaceObject<T extends THREE.Mesh | THREE.Points>(e: Engine, old: T | null, next: T | null, order: number): T | null {
  if (old) {
    e.scene.remove(old)
    old.geometry.dispose()
  }
  if (next) {
    next.frustumCulled = false
    next.renderOrder = order
    e.scene.add(next)
  }
  return next
}

export default function Sky3D(props: Props) {
  const { stars, starMap, snap, culture, systemId, lstDeg, lat, magLimit, layers, zhouTian, picked, onPick, playing } = props
  const hostRef = useRef<HTMLDivElement>(null)
  const glRef = useRef<HTMLCanvasElement>(null)
  const textRef = useRef<HTMLCanvasElement>(null)
  const engineRef = useRef<Engine | null>(null)
  const [failed, setFailed] = useState<string | null>(null)
  const [fov, setFov] = useState(110)
  // 面南，抬头到能看见地平线与中天附近
  const look = useRef({ az: 180, alt: 30, fov: 110 })
  const dirty = useRef(true)
  const size = useRef({ w: 1, h: 1, dpr: 1 })
  const shownLst = useRef({ from: lstDeg, to: lstDeg, start: 0, duration: 0, lastChange: 0 })
  const propsRef = useRef(props)
  propsRef.current = props

  const system = SYSTEMS.find((s) => s.id === systemId)!
  const asterisms = useMemo(() => culture.asterisms.filter((a) => a.system === system.asterisms), [culture, system.asterisms])
  const asterismHips = useMemo(() => new Set(asterisms.flatMap((a) => a.lines.flat())), [asterisms])
  const shownStars = useMemo(
    () => stars.filter((s) => s.v <= magLimit || (layers.has('asterisms') && asterismHips.has(s.hip))),
    [stars, magLimit, layers, asterismHips]
  )
  const shownRef = useRef(shownStars)
  shownRef.current = shownStars

  // 当前视角下的矩阵。文字、点选与着色器都从这里取
  const matrices = (lst: number) => {
    const { az, alt } = look.current
    const toLocal = equatorialToLocal(lst, propsRef.current.lat)
    const toView: Mat3 = multiply(localToView(az, alt), toLocal)
    return { toLocal, toView }
  }

  const currentLst = (now: number) => {
    const t = shownLst.current
    if (t.duration <= 0 || now >= t.start + t.duration) return t.to
    return t.from + (t.to - t.from) * ((now - t.start) / t.duration)
  }

  const drawLabels = (toLocal: Mat3, toView: Mat3) => {
    const canvas = textRef.current
    const ctx = canvas?.getContext('2d')
    if (!canvas || !ctx) return
    const p = propsRef.current
    const { w, h, dpr } = size.current
    const k = projectionK(look.current.fov)
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    ctx.clearRect(0, 0, w, h)
    const ground = p.layers.has('horizon')
    /** 赤道单位向量 → 画布坐标；地面遮住或在画外时为 null。 */
    const place = (v: number[]): [number, number] | null => {
      if (ground && apply(toLocal, v)[2] < 0) return null
      const q = viewToPixel(apply(toView, v), k, h / 2)
      if (!q) return null
      const x = w / 2 + q[0]
      const y = h / 2 - q[1]
      return x < -40 || y < -40 || x > w + 40 || y > h + 40 ? null : [x, y]
    }
    const text = (s: string, x: number, y: number, font: string, color: string, align: CanvasTextAlign = 'left') => {
      ctx.font = font
      ctx.textAlign = align
      ctx.lineWidth = 3
      ctx.strokeStyle = 'rgba(5,10,25,0.75)'
      ctx.strokeText(s, x, y)
      ctx.fillStyle = color
      ctx.fillText(s, x, y)
    }

    if (p.layers.has('names')) {
      const list = p.culture.asterisms.filter((a) => a.system === SYSTEMS.find((s) => s.id === p.systemId)!.asterisms)
      for (const a of list) {
        const members = a.lines.flat().map((id) => p.starMap.get(id)).filter((s): s is Star => !!s)
        if (!members.length || Math.min(...members.map((s) => s.v)) > Math.max(3.5, p.magLimit - 2)) continue
        const first = a.lines[0]?.map((id) => p.starMap.get(id)).find((s): s is Star => !!s)
        const xy = first && place(unitVector(first.ra, first.dec))
        if (xy) text(a.name, xy[0] + 5, xy[1] - 4, '11px sans-serif', COLORS.asterismName)
      }
    }
    if (p.layers.has('mansions')) {
      const dets = p.culture.mansions.find((m) => m.system === p.systemId)?.determinatives.map((id) => p.starMap.get(id)) ?? []
      const widths = p.snap.mansionWidths[p.systemId]
      dets.forEach((s, i) => {
        if (!s) return
        const xy = place(unitVector(s.ra, s.dec))
        if (xy) {
          ctx.beginPath()
          ctx.arc(xy[0], xy[1], starRadius(s.v) + 3, 0, 2 * Math.PI)
          ctx.strokeStyle = COLORS.mansionText
          ctx.lineWidth = 1
          ctx.stroke()
        }
        if (widths && dets.length === 28) {
          const mid = place(unitVector(s.ra + widths[i] / 2, 0))
          if (mid) {
            text(MANSIONS[i], mid[0], mid[1] - 6, '14px sans-serif', COLORS.mansionText, 'center')
            text(((widths[i] * p.zhouTian) / 360).toFixed(2), mid[0], mid[1] + 12, '10px sans-serif', COLORS.mansionText, 'center')
          }
        }
      })
    }
    if (p.layers.has('bodies')) {
      for (const b of p.snap.bodies) {
        const xy = place(unitVector(b.ra, b.dec))
        if (xy) text(BODY_LABEL[b.name], xy[0] + 9, xy[1] + 4, '600 13px sans-serif', BODY_COLOR[b.name])
      }
    }
    if (p.picked) {
      const pk = p.picked
      const target = pk.kind === 'star' ? p.starMap.get(pk.hip) : p.snap.bodies.find((b) => b.name === pk.name)
      const xy = target && place(unitVector(target.ra, target.dec))
      if (xy) {
        ctx.beginPath()
        ctx.arc(xy[0], xy[1], 10, 0, 2 * Math.PI)
        ctx.strokeStyle = COLORS.pick
        ctx.lineWidth = 1.5
        ctx.stroke()
      }
    }
    // 方位：沿地平圈标注，始终显示
    const viewOnly = localToView(look.current.az, look.current.alt)
    for (const [name, az] of CARDINALS) {
      const a = (az * Math.PI) / 180
      const q = viewToPixel(apply(viewOnly, [Math.sin(a), Math.cos(a), 0]), k, h / 2)
      if (!q) continue
      const main = name.length === 1
      text(name, w / 2 + q[0], h / 2 - q[1] + (main ? 22 : 18), main ? '600 16px sans-serif' : '12px sans-serif', main ? '#ffb86b' : '#c9a57a', 'center')
    }
  }

  // 创建渲染器与绘制循环：只在有变化时重画
  useEffect(() => {
    const canvas = glRef.current!
    let engine: Engine
    try {
      engine = createEngine(canvas)
    } catch (e) {
      setFailed(String(e instanceof Error ? e.message : e))
      return
    }
    engineRef.current = engine
    const onLost = (ev: Event) => {
      ev.preventDefault()
      setFailed('WebGL 上下文丢失，等待恢复')
    }
    const onRestored = () => {
      setFailed(null)
      dirty.current = true
    }
    canvas.addEventListener('webglcontextlost', onLost)
    canvas.addEventListener('webglcontextrestored', onRestored)

    const host = hostRef.current!
    const resize = () => {
      const w = Math.max(1, host.clientWidth)
      const h = Math.max(1, host.clientHeight)
      const dpr = Math.min(window.devicePixelRatio || 1, 2)
      size.current = { w, h, dpr }
      engine.renderer.setPixelRatio(dpr)
      engine.renderer.setSize(w, h, false)
      const t = textRef.current!
      t.width = Math.round(w * dpr)
      t.height = Math.round(h * dpr)
      dirty.current = true
    }
    const ro = new ResizeObserver(resize)
    ro.observe(host)
    resize()

    let raf = 0
    const tick = (now: number) => {
      raf = requestAnimationFrame(tick)
      const t = shownLst.current
      const moving = t.duration > 0 && now < t.start + t.duration + 32
      if (!dirty.current && !moving) return
      dirty.current = false
      const { w, h, dpr } = size.current
      const { toLocal, toView } = matrices(currentLst(now))
      const s = engine.shared
      s.uEqToView.value.set(...toView)
      s.uUpRow.value.set(toLocal[6], toLocal[7], toLocal[8])
      s.uK.value = projectionK(look.current.fov)
      s.uHalf.value.set((w * dpr) / 2, (h * dpr) / 2)
      s.uGround.value = propsRef.current.layers.has('horizon') ? 1 : 0
      // 放大时星点略增大，暗星才分得开
      s.uScale.value = dpr * Math.pow(60 / look.current.fov, 0.3)
      const viewOnly = localToView(look.current.az, look.current.alt)
      engine.upView.value.set(viewOnly[2], viewOnly[5], viewOnly[8])
      engine.lineMaterial.uniforms.uWidth.value = 1.5 * dpr
      engine.renderer.render(engine.scene, engine.camera)
      drawLabels(toLocal, toView)
    }
    raf = requestAnimationFrame(tick)

    return () => {
      cancelAnimationFrame(raf)
      ro.disconnect()
      canvas.removeEventListener('webglcontextlost', onLost)
      canvas.removeEventListener('webglcontextrestored', onRestored)
      engine.scene.traverse((o) => {
        const m = o as THREE.Mesh
        m.geometry?.dispose?.()
        ;(m.material as THREE.Material | undefined)?.dispose?.()
      })
      engine.lineMaterial.dispose()
      engine.starMaterial.dispose()
      engine.bodyMaterial.dispose()
      engine.renderer.dispose()
      engineRef.current = null
    }
    // 绘制循环经 ref 取最新值，只建一次
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // 播放时把恒星时的跳变补成连续转动，帧间隔内走完（步长太大时直接跳，转得再快也看不清）
  useEffect(() => {
    const t = shownLst.current
    const now = performance.now()
    const gap = now - t.lastChange
    const delta = ((lstDeg - t.to + 540) % 360) - 180
    const from = currentLst(now)
    t.lastChange = now
    if (playing && Math.abs(delta) <= 30 && gap < 3000) {
      t.from = from
      t.to = t.to + delta
      t.start = now
      t.duration = Math.max(30, gap)
    } else {
      t.from = t.to = lstDeg
      t.duration = 0
    }
    dirty.current = true
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lstDeg, playing])

  // 星
  useEffect(() => {
    const e = engineRef.current
    if (!e) return
    const vec: number[] = []
    const col: number[] = []
    const siz: number[] = []
    for (const s of shownStars) {
      vec.push(...unitVector(s.ra, s.dec))
      const [r, g, b] = parseColor(starColor(s.bv))
      col.push(r, g, b)
      // 点精灵含外圈辉光：实心部分占半径的四成，直径取 5 倍星半径，实心部分才与 SVG 的星一样大
      siz.push(starRadius(s.v) * 5)
    }
    const pts = new THREE.Points(pointGeometry(vec, col, siz), e.starMaterial)
    e.starPoints = replaceObject(e, e.starPoints, pts, 2)
    dirty.current = true
  }, [shownStars])

  // 日月五星
  useEffect(() => {
    const e = engineRef.current
    if (!e) return
    if (!layers.has('bodies')) {
      e.bodyPoints = replaceObject(e, e.bodyPoints, null, 3)
      dirty.current = true
      return
    }
    const vec: number[] = []
    const col: number[] = []
    const siz: number[] = []
    for (const b of snap.bodies) {
      vec.push(...unitVector(b.ra, b.dec))
      const [r, g, bl] = parseColor(BODY_COLOR[b.name])
      col.push(r, g, bl)
      siz.push(b.name === 'sun' ? 16 : b.name === 'moon' ? 14 : 8)
    }
    e.bodyPoints = replaceObject(e, e.bodyPoints, new THREE.Points(pointGeometry(vec, col, siz), e.bodyMaterial), 3)
    dirty.current = true
  }, [snap, layers])

  // 线：星官、宿界、赤道、黄道、内外规
  useEffect(() => {
    const e = engineRef.current
    if (!e) return
    const seg = new Segments()
    const decCircle = (dec: number) => Array.from({ length: 121 }, (_, k) => ({ ra: k * 3, dec }))
    if (layers.has('equator')) seg.polyline(decCircle(0), parseColor(COLORS.equator))
    if (layers.has('ecliptic')) seg.polyline(greatCircle(snap.eclipticPole[0], snap.eclipticPole[1], 120), parseColor(COLORS.ecliptic))
    if (layers.has('circles')) {
      const c = parseColor(COLORS.circle)
      seg.polyline(decCircle(90 - Math.abs(lat)), c, true)
      seg.polyline(decCircle(Math.abs(lat) - 90), c, true)
    }
    if (layers.has('mansions')) {
      const c = parseColor(COLORS.mansion)
      for (const hip of culture.mansions.find((m) => m.system === systemId)?.determinatives ?? []) {
        const s = starMap.get(hip)
        if (!s) continue
        seg.polyline(Array.from({ length: 61 }, (_, k) => ({ ra: s.ra, dec: -90 + k * 3 })), c)
      }
    }
    if (layers.has('asterisms')) {
      const plain = parseColor(COLORS.asterism)
      const schoolOf = new Map(culture.schools.map((x) => [x.id, x.school]))
      const coloring = layers.has('schools') && system.asterisms === 'chenzhuo'
      for (const a of asterisms) {
        const school = coloring ? schoolOf.get(a.id) : null
        const c = school ? parseColor(SCHOOL_COLOR[school]) : plain
        for (const line of a.lines) {
          for (let i = 0; i + 1 < line.length; i++) {
            const p = starMap.get(line[i])
            const q = starMap.get(line[i + 1])
            if (p && q) seg.add(unitVector(p.ra, p.dec), unitVector(q.ra, q.dec), c)
          }
        }
      }
    }
    const g = new THREE.InstancedBufferGeometry()
    g.setAttribute('position', new THREE.Float32BufferAttribute([0, -1, 0, 1, -1, 0, 1, 1, 0, 0, -1, 0, 1, 1, 0, 0, 1, 0], 3))
    g.setAttribute('iStart', new THREE.InstancedBufferAttribute(new Float32Array(seg.start), 3))
    g.setAttribute('iEnd', new THREE.InstancedBufferAttribute(new Float32Array(seg.end), 3))
    g.setAttribute('iColor', new THREE.InstancedBufferAttribute(new Float32Array(seg.color), 4))
    g.instanceCount = seg.color.length / 4
    e.lines = replaceObject(e, e.lines, new THREE.Mesh(g, e.lineMaterial), 1)
    dirty.current = true
  }, [snap, culture, systemId, system.asterisms, asterisms, starMap, layers, lat])

  // 其余影响文字或地面的变化
  useEffect(() => {
    dirty.current = true
  }, [picked, zhouTian, lat, magLimit, layers])

  // 拖动转向、滚轮与双指缩放
  useEffect(() => {
    const host = hostRef.current!
    const pointers = new Map<number, { x: number; y: number }>()
    let moved = 0
    let pinch = 0
    const setFovClamped = (f: number) => {
      look.current.fov = Math.max(MIN_FOV, Math.min(MAX_FOV, f))
      setFov(look.current.fov)
      dirty.current = true
    }
    const down = (e: PointerEvent) => {
      host.setPointerCapture(e.pointerId)
      pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
      if (pointers.size === 1) moved = 0
      pinch = 0
    }
    const move = (e: PointerEvent) => {
      const last = pointers.get(e.pointerId)
      if (!last) return
      const dx = e.clientX - last.x
      const dy = e.clientY - last.y
      pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
      moved += Math.hypot(dx, dy)
      if (pointers.size === 2) {
        const [a, b] = [...pointers.values()]
        const d = Math.hypot(a.x - b.x, a.y - b.y)
        if (pinch > 0 && d > 0) setFovClamped((look.current.fov * pinch) / d)
        pinch = d
        return
      }
      // 抓着天空拖：画面中心处每像素对应的角度
      const k = projectionK(look.current.fov)
      const deg = (2 / (k * size.current.h)) * 2 * (180 / Math.PI)
      const l = look.current
      const c = Math.max(0.15, Math.cos((l.alt * Math.PI) / 180))
      l.az = (((l.az - (dx * deg) / c) % 360) + 360) % 360
      l.alt = Math.max(-89.5, Math.min(89.5, l.alt + dy * deg))
      dirty.current = true
    }
    const up = (e: PointerEvent) => {
      const wasOnly = pointers.size === 1
      pointers.delete(e.pointerId)
      if (!wasOnly || moved > 4 || e.type === 'pointercancel') return
      const rect = host.getBoundingClientRect()
      const px = e.clientX - rect.left
      const py = e.clientY - rect.top
      pick(px, py)
    }
    const pick = (px: number, py: number) => {
      const p = propsRef.current
      const { w, h } = size.current
      const { toLocal, toView } = matrices(currentLst(performance.now()))
      const k = projectionK(look.current.fov)
      const ground = p.layers.has('horizon')
      const dist = (ra: number, dec: number) => {
        const v = unitVector(ra, dec)
        if (ground && apply(toLocal, v)[2] < 0) return Infinity
        const q = viewToPixel(apply(toView, v), k, h / 2)
        return q ? Math.hypot(w / 2 + q[0] - px, h / 2 - q[1] - py) : Infinity
      }
      let best: Picked | null = null
      let bestD = 12
      if (p.layers.has('bodies')) {
        for (const b of p.snap.bodies) {
          const d = dist(b.ra, b.dec)
          if (d < bestD) {
            bestD = d
            best = { kind: 'body', name: b.name }
          }
        }
      }
      for (const s of shownRef.current) {
        const d = dist(s.ra, s.dec) - starRadius(s.v) * 0.5
        if (d < bestD) {
          bestD = d
          best = { kind: 'star', hip: s.hip }
        }
      }
      p.onPick(best)
    }
    const wheel = (e: WheelEvent) => {
      e.preventDefault()
      setFovClamped(look.current.fov * Math.exp(e.deltaY * 0.001))
    }
    const cancel = (e: PointerEvent) => {
      pointers.delete(e.pointerId)
    }
    host.addEventListener('pointerdown', down)
    host.addEventListener('pointermove', move)
    host.addEventListener('pointerup', up)
    host.addEventListener('pointercancel', cancel)
    host.addEventListener('wheel', wheel, { passive: false })
    return () => {
      host.removeEventListener('pointerdown', down)
      host.removeEventListener('pointermove', move)
      host.removeEventListener('pointerup', up)
      host.removeEventListener('pointercancel', cancel)
      host.removeEventListener('wheel', wheel)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const toggleFullscreen = () => {
    if (document.fullscreenElement) void document.exitFullscreen()
    else void hostRef.current?.requestFullscreen()
  }

  if (failed && !engineRef.current) {
    return <div style={{ padding: 24 }}>此浏览器无法使用 WebGL，3D 视图不可用（{failed}）。盖图和地平图不受影响。</div>
  }

  return (
    <div ref={hostRef} data-testid="sky3d" style={{ position: 'relative', width: '100%', height: 'min(78vh, 760px)', minHeight: 360, overflow: 'hidden', background: COLORS.sky, touchAction: 'none', cursor: 'grab', userSelect: 'none' }}>
      <canvas ref={glRef} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', display: 'block' }} />
      <canvas ref={textRef} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', pointerEvents: 'none' }} />
      <div style={{ position: 'absolute', right: 8, top: 8, display: 'flex', gap: 8, alignItems: 'center', color: COLORS.text, fontSize: 12 }}>
        <span>视场 {fov.toFixed(0)}°</span>
        <button type="button" onPointerDown={(e) => e.stopPropagation()} onPointerUp={(e) => e.stopPropagation()} onClick={toggleFullscreen} style={{ background: 'rgba(20,30,55,0.8)', color: COLORS.text, border: `1px solid ${COLORS.skyEdge}`, borderRadius: 4, padding: '2px 8px', cursor: 'pointer' }}>
          全屏
        </button>
      </div>
    </div>
  )
}

