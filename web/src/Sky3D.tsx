/**
 * 3D 星图：站在当地仰望，拖动转向、滚轮或双指缩放。
 *
 * 天体位置仍由 Rust 算（同盖图），这里只做显示：星和线存赤道单位向量，转视角只改一个矩阵，不重算也不重建几何。
 * 投影是顶点着色器里的球极投影（公式见 skyProjection.ts）。文字和选中标记画在上层的 2D 画布，用同一公式算坐标。
 * 看到的是视地平坐标：日月五星取站心位置，折射查 Rust 给的对照表（refractionAt，着色器里同一算法），地面按视高度遮挡。
 * 天色：亮度来自 Rust 的全天网格（Schaefer 模型，日食时按未遮比例减小太阳贡献），星按所在方向的极限星等淡出；亮度到颜色的映射只是显示用。
 * 月亮按月→日方向逐像素分明暗；放大后暗面不透光，日食时能遮住太阳。
 * 银河：NASA Deep Star Maps（银道坐标贴图），按天光淡出。全食时加日冕（Baumbach 1937 亮度，与天光同单位相加）。依据见 doc/astronomy/sky-view-3d.md。
 * 月亮眩光（Schaefer 1991 的眼内散射，同偕日）压低月旁的极限星等。
 * 日月：真实圆面大于符号时画圆面（逐像素把视高度反算回真高度再判断，近地平的压扁由此而来），否则画符号；上缘的视高度高于地平线就显示，地面逐像素遮挡。
 */
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import * as THREE from 'three'
import { BODY_COLOR, BODY_LABEL, COLORS, MANSIONS, SCHOOL_COLOR, HORIZON_SAMPLES, LIMIT_CURVE_SAMPLES, LIMIT_CURVE_STEP, horizonAt, MOON_GLARE_MIN_DEG, SKY_GRID_ALT, SKY_GRID_AZ, SYSTEMS, REFRACTION_SAMPLES, limitCurveAt, greatCircle, refractionAt, starColor, starRadius, type Culture, type Picked, type SnapshotJson, type Star } from './skyCommon'
import { DATA_BASE } from './ephemerisUrls'
import { apply, equatorialToLocal, localToView, projectionK, unitVector, viewToPixel, type Mat3 } from './skyProjection'

const MIN_FOV = 5
const MAX_FOV = 180
const GROUND = '#1a2118'
const GROUND_FAR = '#0c110b'
const CARDINALS: [string, number][] = [['北', 0], ['东北', 45], ['东', 90], ['东南', 135], ['南', 180], ['西南', 225], ['西', 270], ['西北', 315]]

// 地平线：每个方位一个视高度（地形、海平线俯角、天文地平都是它的特例）
const HORIZON_GLSL = /* glsl */ `
  uniform highp sampler2D uHorizonTex;
  float horizonSinAt(vec3 l) {
    float az = atan(l.x, l.y);
    if (az < 0.0) az += 6.283185307;
    float x = az / 6.283185307 * float(${HORIZON_SAMPLES});
    int i = int(floor(x));
    float a = texelFetch(uHorizonTex, ivec2(i % ${HORIZON_SAMPLES}, 0), 0).x;
    float b = texelFetch(uHorizonTex, ivec2((i + 1) % ${HORIZON_SAMPLES}, 0), 0).x;
    return mix(a, b, x - float(i));
  }
`

const COMMON_GLSL = /* glsl */ `
  uniform mat3 uEqToLocal;
  uniform mat3 uLocalToView;
  uniform float uRefr[${REFRACTION_SAMPLES}];
  uniform float uK;
  uniform vec2 uHalf;
  // 地方坐标（东、北、天顶）的真方向 → 视方向：方位不变，高度加折射。表的取样见 skyCommon.ts
  float refraction(float alt) {
    float x = sqrt(max(alt + 5.0, 0.0)) / 0.1;
    int i = int(min(floor(x), float(${REFRACTION_SAMPLES - 2})));
    return mix(uRefr[i], uRefr[i + 1], x - float(i));
  }
  vec3 observed(vec3 l) {
    float alt = degrees(asin(clamp(l.z, -1.0, 1.0)));
    float a = radians(alt + refraction(alt));
    float len = length(l.xy);
    if (len < 1e-6) return l;
    return vec3(l.xy / len * cos(a), sin(a));
  }
  vec2 toPixel(vec3 v) { return v.xy * uK * uHalf.y / max(1.0 + v.z, 0.02); }
`

// 天光网格：x = lg 亮度，y = 极限星等。地方坐标方向取样，手写双线性（浮点纹理的线性过滤不是所有设备都有）
const SKY_GLSL = /* glsl */ `
  uniform highp sampler2D uSkyGrid;
  vec2 skyAt(vec3 l) {
    float az = atan(l.x, l.y);
    if (az < 0.0) az += 6.283185307;
    float u = az / 6.283185307 * float(${SKY_GRID_AZ - 1});
    float v = sqrt(clamp(asin(clamp(l.z, 0.0, 1.0)) / 1.570796327, 0.0, 1.0)) * float(${SKY_GRID_ALT - 1});
    int i = int(min(floor(u), float(${SKY_GRID_AZ - 2})));
    int j = int(min(floor(v), float(${SKY_GRID_ALT - 2})));
    vec2 f = vec2(u - float(i), v - float(j));
    vec2 a = mix(texelFetch(uSkyGrid, ivec2(i, j), 0).xy, texelFetch(uSkyGrid, ivec2(i + 1, j), 0).xy, f.x);
    vec2 b = mix(texelFetch(uSkyGrid, ivec2(i, j + 1), 0).xy, texelFetch(uSkyGrid, ivec2(i + 1, j + 1), 0).xy, f.x);
    return mix(a, b, f.y);
  }
  // 此方向的极限星等：网格值 + 月亮眩光使它变亮的量（眩光 ∝ 1/θ²，变化太陡，网格分辨不了，单独算）
  uniform vec3 uMoonDir;
  uniform float uMoonGlare;
  uniform float uLimitCurve[${LIMIT_CURVE_SAMPLES}];
  float limitCurve(float lg) {
    float x = clamp(lg / ${LIMIT_CURVE_STEP.toFixed(2)}, 0.0, float(${LIMIT_CURVE_SAMPLES}) - 1.001);
    int i = int(floor(x));
    return mix(uLimitCurve[i], uLimitCurve[i + 1], x - float(i));
  }
  float limitAt(vec3 l) {
    vec2 g = skyAt(l);
    if (uMoonGlare <= 0.0) return g.y;
    float theta = max(degrees(length(l - uMoonDir)), ${MOON_GLARE_MIN_DEG.toFixed(2)});
    float b = pow(10.0, g.x);
    float withGlare = log(b + uMoonGlare / (theta * theta)) / log(10.0);
    return g.y + limitCurve(withGlare) - limitCurve(g.x);
  }
  // 亮度 lg(nL) → 显示色。只是把模型亮度映射到屏幕，不是色度计算
  vec3 skyColor(float lg) {
    vec3 c = mix(vec3(0.020, 0.032, 0.090), vec3(0.055, 0.085, 0.210), smoothstep(2.3, 4.0, lg));
    c = mix(c, vec3(0.140, 0.240, 0.460), smoothstep(4.0, 6.0, lg));
    c = mix(c, vec3(0.300, 0.500, 0.780), smoothstep(6.0, 7.8, lg));
    return mix(c, vec3(0.520, 0.720, 0.950), smoothstep(7.8, 9.3, lg));
  }
`

const POINT_VERT = /* glsl */ `
  ${COMMON_GLSL}
  ${SKY_GLSL}
  ${HORIZON_GLSL}
  uniform float uGround;
  uniform float uScale;
  attribute vec3 aColor;
  attribute float aSize;
  attribute float aMag;
  varying vec3 vColor;
  varying float vFade;
  void main() {
    vec3 l = observed(uEqToLocal * position);
    vec3 v = uLocalToView * l;
    vColor = aColor;
    // 暗于此方向极限星等的看不见，在极限附近 1 等内淡出
    vFade = clamp((limitAt(l) + 0.5 - aMag) / 1.0, 0.0, 1.0);
    if (vFade <= 0.0 || v.z < -0.9 || (uGround > 0.5 && l.z < horizonSinAt(l))) {
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
  varying float vFade;
  void main() {
    float r = length(gl_PointCoord - 0.5) * 2.0;
    if (r > 1.0) discard;
    float core = 1.0 - smoothstep(0.25, 0.55, r);
    float glow = pow(1.0 - r, 2.0) * 0.45;
    gl_FragColor = vec4(vColor, clamp(core + glow, 0.0, 1.0) * vFade);
  }
`
const BODY_VERT = /* glsl */ `
  ${COMMON_GLSL}
  ${SKY_GLSL}
  ${HORIZON_GLSL}
  uniform float uGround;
  uniform float uScale;
  attribute vec3 aColor;
  attribute float aSize;
  attribute float aRadius;
  attribute float aMag;
  attribute vec3 aToSun;
  varying vec3 vColor;
  varying vec3 vToSun;
  varying vec3 vTrueCenter;
  varying float vRadius;
  varying float vTrueDisc;
  varying float vPixelRadius;
  void main() {
    vec3 t = uEqToLocal * position;
    vec3 l = observed(t);
    vec3 v = uLocalToView * l;
    vColor = aColor;
    vTrueCenter = t;
    vRadius = aRadius;
    vToSun = uEqToLocal * aToSun;
    float limb = degrees(asin(clamp(t.z, -1.0, 1.0)) + aRadius);
    // 球极投影保角：圆面仍是圆，此处的比例尺 = 中心比例尺 ÷ (1 + cos θ)
    float rpx = aRadius * uK * uHalf.y / (1.0 + v.z);
    float sym = aSize * uScale * 0.5;
    vTrueDisc = rpx >= sym ? 1.0 : 0.0;
    vPixelRadius = rpx;
    // 上缘低于地平线才整个隐去；露出的部分由片元逐像素遮挡
    // 行星白天也按极限星等隐去；日月 aRadius > 0，不受此限
    bool faint = aRadius <= 0.0 && aMag > limitAt(l) + 0.5;
    if (faint || v.z < -0.9 || (uGround > 0.5 && limb + refraction(limb) < degrees(asin(horizonSinAt(l))))) {
      gl_Position = vec4(2.0, 2.0, 0.0, 1.0);
      gl_PointSize = 0.0;
      return;
    }
    gl_Position = vec4(toPixel(v) / uHalf, 0.0, 1.0);
    // 地平附近折射的变化率可使圆面在高度方向伸缩，精灵留足余量
    gl_PointSize = 2.0 * max(rpx * 1.6, sym) + 4.0;
  }
`
const BODY_FRAG = /* glsl */ `
  precision highp float;
  ${SKY_GLSL}
  ${HORIZON_GLSL}
  uniform mat3 uLocalToView;
  uniform float uRefr[${REFRACTION_SAMPLES}];
  uniform float uK;
  uniform vec2 uHalf;
  uniform float uGround;
  uniform float uTotality;
  varying vec3 vColor;
  varying vec3 vToSun;
  varying vec3 vTrueCenter;
  varying float vRadius;
  varying float vTrueDisc;
  varying float vPixelRadius;
  float refraction(float alt) {
    float x = sqrt(max(alt + 5.0, 0.0)) / 0.1;
    int i = int(min(floor(x), float(${REFRACTION_SAMPLES - 2})));
    return mix(uRefr[i], uRefr[i + 1], x - float(i));
  }
  void main() {
    // 此像素的视方向（地方坐标）：反算球极投影
    vec2 s = (gl_FragCoord.xy - uHalf) / (uK * uHalf.y);
    float rho2 = dot(s, s);
    vec3 d = vec3(2.0 * s, 1.0 - rho2) / (1.0 + rho2);
    vec3 l = transpose(uLocalToView) * d;
    if (uGround > 0.5 && l.z < horizonSinAt(l)) discard;
    float alpha;
    vec3 color = vColor;
    bool isMoon = dot(vToSun, vToSun) > 0.25;
    if (vTrueDisc > 0.5) {
      // 视高度 → 真高度（不动点迭代），在真方向上判断是否落在圆面内
      float seen = degrees(asin(clamp(l.z, -1.0, 1.0)));
      float t = seen;
      for (int k = 0; k < 4; k++) t = seen - refraction(t);
      float len = max(length(l.xy), 1e-6);
      vec3 truth = vec3(l.xy / len * cos(radians(t)), sin(radians(t)));
      vec3 offset = (truth - vTrueCenter) / vRadius;
      float r = length(offset);
      float edge = 1.0 / max(vPixelRadius, 1.0);
      alpha = 1.0 - smoothstep(1.0 - edge, 1.0 + edge, r);
      if (isMoon) {
        // 球面法向：切向偏移 + 指向观测者的分量；暗面取天色，不透光，所以日食时能遮日
        vec3 n = offset - vTrueCenter * sqrt(max(1.0 - r * r, 0.0));
        float lit = smoothstep(-0.03, 0.05, dot(n, vToSun));
        vec3 sky = skyColor(skyAt(l).x);
        // 全食时暗面压暗一些：与天空同色就看不出月轮了
        color = min(sky * (1.0 - 0.45 * uTotality) + vColor * lit, vec3(1.0));
      }
    } else {
      // 符号：精灵比符号大 4 像素，留给边缘抗锯齿
      vec2 p = (gl_PointCoord - 0.5) * vec2(2.0, -2.0);
      float r = length(p);
      alpha = 1.0 - smoothstep(0.78, 0.92, r);
      if (isMoon) {
        // 符号太小，暗面透明：新月附近不显示一个圆盖住太阳
        vec2 q = p / 0.85;
        vec3 view = vec3(q, -sqrt(max(1.0 - dot(q, q), 0.0)));
        alpha *= smoothstep(-0.03, 0.05, dot(view, uLocalToView * vToSun));
      }
    }
    if (alpha <= 0.0) discard;
    gl_FragColor = vec4(color, alpha);
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
  varying vec3 vLocal;
  void main() {
    vec3 la = observed(uEqToLocal * iStart);
    vec3 lb = observed(uEqToLocal * iEnd);
    vec3 a = uLocalToView * la;
    vec3 b = uLocalToView * lb;
    vColor = iColor;
    vAcross = position.y;
    vLocal = mix(la, lb, position.x);
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
  ${HORIZON_GLSL}
  uniform float uGround;
  varying vec4 vColor;
  varying float vAcross;
  varying vec3 vLocal;
  void main() {
    if (uGround > 0.5 && vLocal.z < horizonSinAt(vLocal)) discard;
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
  ${SKY_GLSL}
  ${HORIZON_GLSL}
  uniform float uCoronaUnit;
  uniform mat3 uLocalToView;
  uniform mat3 uEqToLocal;
  uniform mat3 uTrueToGal;
  uniform sampler2D uMilky;
  uniform float uMilkyOn;
  uniform vec3 uSunLocal;
  uniform float uSunRadius;
  uniform float uCorona;
  uniform float uTwilight;
  uniform float uK;
  uniform vec2 uHalf;
  uniform vec3 uUpView;
  uniform float uGround;
  uniform vec3 uGroundNear;
  uniform vec3 uGroundFar;
  uniform vec3 uHorizon;
  varying vec2 vNdc;
  void main() {
    vec2 s = vNdc * vec2(uHalf.x / uHalf.y, 1.0) / uK;
    float rho2 = dot(s, s);
    vec3 v = vec3(2.0 * s, 1.0 - rho2) / (1.0 + rho2);
    float up = dot(uUpView, v);
    vec3 l = transpose(uLocalToView) * v;
    float lg = skyAt(l).x;
    vec3 c = skyColor(lg);
    if (uCorona > 0.5) {
      // 日冕（Baumbach 1937，r 以太阳半径计）：以日面中心亮度的 1e-6 为单位，此单位由 Rust 按太阳星等推算。
      // 与天光同单位相加后走同一条亮度→颜色映射，所以日冕多亮、伸多远都由亮度决定
      float r = length(l - uSunLocal) / uSunRadius;
      if (r > 1.0) {
        float corona = uCoronaUnit * (0.0532 * pow(r, -2.5) + 1.425 * pow(r, -7.0) + 2.565 * pow(r, -17.0));
        float total = pow(10.0, lg) + corona;
        vec3 base = skyColor(log(total) / log(10.0));
        c = mix(base, vec3(0.93, 0.93, 0.96) * max(max(base.r, base.g), base.b) * 1.05, corona / total);
      }
    }
    if (uMilkyOn > 0.5) {
      // 贴图是银道坐标的等距圆柱投影：中心银心，银经向左增大（大麦哲伦云 l = 280° 在中心右侧 80°）
      vec3 g = uTrueToGal * (transpose(uEqToLocal) * l);
      vec2 uv = vec2(fract(0.5 - atan(g.y, g.x) / 6.283185307), 0.5 + asin(clamp(g.z, -1.0, 1.0)) / 3.141592654);
      c += texture2D(uMilky, uv).rgb * (1.0 - smoothstep(2.6, 4.4, lg));
    }
    // 晨昏时太阳方向近地平偏暖。纯显示效果，亮度仍是模型给的
    float near = pow(max(dot(l, uSunLocal), 0.0), 3.0) * exp(-max(l.z, 0.0) * 5.0);
    c = mix(c, vec3(1.0, 0.56, 0.26) * max(max(c.r, c.g), c.b) * 1.5, clamp(uTwilight * near, 0.0, 0.85));
    float horizon = horizonSinAt(l);
    if (uGround > 0.5) {
      if (l.z < horizon) {
        // 地面随天顶亮度明暗
        float day = smoothstep(2.5, 8.5, skyAt(vec3(0.0, 0.0, 1.0)).x);
        c = mix(uGroundNear, uGroundFar, clamp((horizon - l.z) * 1.6, 0.0, 1.0)) * mix(1.0, 4.0, day);
      }
      float w = fwidth(up) * 1.5;
      c = mix(c, uHorizon, 1.0 - smoothstep(w * 0.6, w * 1.6, abs(l.z - horizon)));
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
  /** 折射对照表（度），null 为不计 */
  refraction: number[] | null
  /** 地平线：方位 360°·i/720 处的视高度（度）；null 为高度 0° 的天文地平 */
  horizon: number[] | null
  /** 全屏时显示在画面下方的操作条（全屏后页面其余部分看不到） */
  overlay?: ReactNode
}

type Engine = {
  renderer: THREE.WebGLRenderer
  scene: THREE.Scene
  camera: THREE.Camera
  shared: {
    uEqToLocal: { value: THREE.Matrix3 }
    uLocalToView: { value: THREE.Matrix3 }
    uRefr: { value: Float32Array }
    uSkyGrid: { value: THREE.DataTexture }
    uSunLocal: { value: THREE.Vector3 }
    uTwilight: { value: number }
    uTotality: { value: number }
    uHorizonTex: { value: THREE.DataTexture }
    uCoronaUnit: { value: number }
    uMoonDir: { value: THREE.Vector3 }
    uMoonGlare: { value: number }
    uLimitCurve: { value: Float32Array }
    uTrueToGal: { value: THREE.Matrix3 }
    uMilky: { value: THREE.Texture | null }
    uMilkyOn: { value: number }
    uSunRadius: { value: number }
    uCorona: { value: number }
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

function pointGeometry(vecs: number[], colors: number[], sizes: number[], mags: number[], radii?: number[], toSun?: number[]) {
  const g = new THREE.BufferGeometry()
  g.setAttribute('position', new THREE.Float32BufferAttribute(vecs, 3))
  g.setAttribute('aColor', new THREE.Float32BufferAttribute(colors, 3))
  g.setAttribute('aSize', new THREE.Float32BufferAttribute(sizes, 1))
  g.setAttribute('aMag', new THREE.Float32BufferAttribute(mags, 1))
  if (radii) g.setAttribute('aRadius', new THREE.Float32BufferAttribute(radii, 1))
  if (toSun) g.setAttribute('aToSun', new THREE.Float32BufferAttribute(toSun, 3))
  return g
}

function createEngine(canvas: HTMLCanvasElement): Engine {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false })
  renderer.outputColorSpace = THREE.LinearSRGBColorSpace // 配色都是显示空间的值，不再转换
  const scene = new THREE.Scene()
  const camera = new THREE.Camera() // 位置全由着色器算，相机不起作用
  const skyGrid = new THREE.DataTexture(new Float32Array(SKY_GRID_AZ * SKY_GRID_ALT * 4), SKY_GRID_AZ, SKY_GRID_ALT, THREE.RGBAFormat, THREE.FloatType)
  skyGrid.minFilter = skyGrid.magFilter = THREE.NearestFilter
  const horizonTex = new THREE.DataTexture(new Float32Array(HORIZON_SAMPLES * 4), HORIZON_SAMPLES, 1, THREE.RGBAFormat, THREE.FloatType)
  horizonTex.minFilter = horizonTex.magFilter = THREE.NearestFilter
  horizonTex.needsUpdate = true
  const shared = {
    uEqToLocal: { value: new THREE.Matrix3() },
    uLocalToView: { value: new THREE.Matrix3() },
    uRefr: { value: new Float32Array(REFRACTION_SAMPLES) },
    uSkyGrid: { value: skyGrid },
    uSunLocal: { value: new THREE.Vector3(0, 0, -1) },
    uTwilight: { value: 0 },
    uTotality: { value: 0 },
    uHorizonTex: { value: horizonTex },
    uCoronaUnit: { value: 7e8 },
    uMoonDir: { value: new THREE.Vector3(0, 0, -1) },
    uMoonGlare: { value: 0 },
    uLimitCurve: { value: new Float32Array(LIMIT_CURVE_SAMPLES) },
    uTrueToGal: { value: new THREE.Matrix3() },
    uMilky: { value: null },
    uMilkyOn: { value: 0 },
    uSunRadius: { value: 0.0046 },
    uCorona: { value: 0 },
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
        uMoonDir: shared.uMoonDir, uMoonGlare: shared.uMoonGlare, uLimitCurve: shared.uLimitCurve,
        uSkyGrid: shared.uSkyGrid,
        uLocalToView: shared.uLocalToView,
        uEqToLocal: shared.uEqToLocal,
        uTrueToGal: shared.uTrueToGal,
        uMilky: shared.uMilky,
        uMilkyOn: shared.uMilkyOn,
        uSunRadius: shared.uSunRadius,
        uCorona: shared.uCorona,
        uHorizonTex: shared.uHorizonTex,
        uCoronaUnit: shared.uCoronaUnit,
        uSunLocal: shared.uSunLocal,
        uTwilight: shared.uTwilight,
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
    uniforms: { uEqToLocal: shared.uEqToLocal, uLocalToView: shared.uLocalToView, uRefr: shared.uRefr, uHorizonTex: shared.uHorizonTex, uK: shared.uK, uHalf: shared.uHalf, uGround: shared.uGround, uWidth: { value: 1.5 } },
  })
  const pointUniforms = { uMoonDir: shared.uMoonDir, uMoonGlare: shared.uMoonGlare, uLimitCurve: shared.uLimitCurve, uHorizonTex: shared.uHorizonTex, uTotality: shared.uTotality, uSkyGrid: shared.uSkyGrid, uEqToLocal: shared.uEqToLocal, uLocalToView: shared.uLocalToView, uRefr: shared.uRefr, uK: shared.uK, uHalf: shared.uHalf, uGround: shared.uGround, uScale: shared.uScale }
  const starMaterial = new THREE.ShaderMaterial({ vertexShader: POINT_VERT, fragmentShader: STAR_FRAG, ...blend, blending: THREE.AdditiveBlending, uniforms: pointUniforms })
  const bodyMaterial = new THREE.ShaderMaterial({ vertexShader: BODY_VERT, fragmentShader: BODY_FRAG, ...blend, uniforms: pointUniforms })
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
  const { stars, starMap, snap, culture, systemId, lstDeg, lat, magLimit, layers, zhouTian, picked, onPick, playing, refraction, horizon, overlay } = props
  const hostRef = useRef<HTMLDivElement>(null)
  const glRef = useRef<HTMLCanvasElement>(null)
  const textRef = useRef<HTMLCanvasElement>(null)
  const engineRef = useRef<Engine | null>(null)
  const [failed, setFailed] = useState<string | null>(null)
  const [fov, setFov] = useState(110)
  const [fullscreen, setFullscreen] = useState(false)
  const [tracking, setTracking] = useState(false)
  const trackingRef = useRef(false)
  trackingRef.current = tracking && !!picked
  // 面南，抬头到能看见地平线与中天附近
  const look = useRef({ az: 180, alt: 30, fov: 110 })
  const dirty = useRef(true)
  const size = useRef({ w: 1, h: 1, dpr: 1 })
  const shownLst = useRef({ from: lstDeg, to: lstDeg, start: 0, duration: 0, lastChange: 0 })
  const sunRef = useRef<number[]>([0, 0, -1])
  const moonRef = useRef<number[]>([0, 0, -1])
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
    return { toLocal: equatorialToLocal(lst, propsRef.current.lat), toView: localToView(az, alt) }
  }
  /** 地方坐标的真方向 → 视方向（与着色器 observed 同）。 */
  const observed = (l: number[]): number[] => {
    const alt = (Math.asin(Math.max(-1, Math.min(1, l[2]))) * 180) / Math.PI
    const a = ((alt + refractionAt(propsRef.current.refraction, alt)) * Math.PI) / 180
    const len = Math.hypot(l[0], l[1])
    return len < 1e-6 ? l : [(l[0] / len) * Math.cos(a), (l[1] / len) * Math.cos(a), Math.sin(a)]
  }

  /** 此方向看得见的最暗星等（极限星等 + 0.5，同着色器）；关掉散射时不限。方向取最近的网格点。 */
  const limitAt = (l: number[]) => {
    const p = propsRef.current
    if (!p.layers.has('skylight')) return Infinity
    const az = ((Math.atan2(l[0], l[1]) * 180) / Math.PI + 360) % 360
    const j = Math.round((az / 360) * (SKY_GRID_AZ - 1))
    const i = Math.round(Math.sqrt(Math.max(0, Math.asin(Math.max(0, l[2])) / (Math.PI / 2))) * (SKY_GRID_ALT - 1))
    const k = i * SKY_GRID_AZ + j
    let lim = p.snap.skyLight.lim[k]
    const moon = p.snap.bodies.find((b) => b.name === 'moon')!
    if (p.snap.skyLight.moonGlare > 0) {
      const m = observed(apply(matrices(currentLst(performance.now())).toLocal, unitVector(moon.topoRa, moon.topoDec)))
      const theta = Math.max((Math.hypot(l[0] - m[0], l[1] - m[1], l[2] - m[2]) * 180) / Math.PI, MOON_GLARE_MIN_DEG)
      const lg = p.snap.skyLight.lgB[k]
      lim += limitCurveAt(p.snap.skyLight.limitCurve, Math.log10(10 ** lg + p.snap.skyLight.moonGlare / (theta * theta))) - limitCurveAt(p.snap.skyLight.limitCurve, lg)
    }
    return lim + 0.5
  }

  /** 圆面上缘的视高度（度）：真高度加视半径后再计折射。 */
  const limbSeen = (t: number[], radiusDeg: number) => {
    const limb = (Math.asin(Math.max(-1, Math.min(1, t[2]))) * 180) / Math.PI + radiusDeg
    return limb + refractionAt(propsRef.current.refraction, limb)
  }

  /** 同着色器：点看视方向，圆面看上缘，都与地平（含俯角）比。 */
  const hiddenByGround = (t: number[], l: number[], radiusDeg: number) => {
    const h = horizonAt(propsRef.current.horizon, (Math.atan2(l[0], l[1]) * 180) / Math.PI)
    return radiusDeg > 0 ? limbSeen(t, radiusDeg) < h : l[2] < Math.sin((h * Math.PI) / 180)
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
    const place = (v: number[], radiusDeg = 0): [number, number] | null => {
      const t = apply(toLocal, v)
      const l = observed(t)
      if (ground && hiddenByGround(t, l, radiusDeg)) return null
      const q = viewToPixel(apply(toView, l), k, h / 2)
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
        // 行星看不见时不标名字
        if (b.semidiameter == null && (b.mag ?? 99) > limitAt(observed(apply(toLocal, unitVector(b.topoRa, b.topoDec))))) continue
        const xy = place(unitVector(b.topoRa, b.topoDec), b.semidiameter ?? 0)
        // 圆面放大后，字放到圆面外
        const offset = 9 + (((b.semidiameter ?? 0) * Math.PI) / 180) * k * (h / 2) * 0.5
        if (xy) text(BODY_LABEL[b.name], xy[0] + offset, xy[1] + 4, '600 13px sans-serif', BODY_COLOR[b.name])
      }
    }
    if (p.picked) {
      const pk = p.picked
      const body = pk.kind === 'body' ? p.snap.bodies.find((b) => b.name === pk.name) : null
      const star = pk.kind === 'star' ? p.starMap.get(pk.hip) : null
      const xy = body ? place(unitVector(body.topoRa, body.topoDec), body.semidiameter ?? 0) : star ? place(unitVector(star.ra, star.dec)) : null
      if (xy) {
        ctx.beginPath()
        const sr = ((body?.semidiameter ?? 0) * Math.PI) / 180
        ctx.arc(xy[0], xy[1], Math.max(10, (sr * k * (h / 2)) / 1.5 + 4), 0, 2 * Math.PI)
        ctx.strokeStyle = COLORS.pick
        ctx.lineWidth = 1.5
        ctx.stroke()
      }
    }
    // 方位：沿地平圈标注，始终显示
    for (const [name, az] of CARDINALS) {
      const a = (az * Math.PI) / 180
      const up = (Math.max(horizonAt(p.horizon, az), -5) * Math.PI) / 180
      const q = viewToPixel(apply(toView, [Math.sin(a) * Math.cos(up), Math.cos(a) * Math.cos(up), Math.sin(up)]), k, h / 2)
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
    new THREE.TextureLoader().load(`${DATA_BASE}/sky/milkyway_2020_4k_gal.jpg`, (t) => {
      // 不用 mipmap：贴图在 0°/360° 接缝处的 uv 跳变会选错层级，出一道缝
      t.generateMipmaps = false
      t.minFilter = t.magFilter = THREE.LinearFilter
      t.wrapS = THREE.RepeatWrapping
      engine.shared.uMilky.value = t
      dirty.current = true
    })
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
      if (trackingRef.current) {
        // 跟踪：视线对准所选天体的视方向
        const pr = propsRef.current
        const pk = pr.picked!
        const body = pk.kind === 'body' ? pr.snap.bodies.find((b) => b.name === pk.name) : null
        const star = pk.kind === 'star' ? pr.starMap.get(pk.hip) : null
        const dir = body ? unitVector(body.topoRa, body.topoDec) : star ? unitVector(star.ra, star.dec) : null
        if (dir) {
          const l = observed(apply(equatorialToLocal(currentLst(now), pr.lat), dir))
          look.current.az = (((Math.atan2(l[0], l[1]) * 180) / Math.PI) + 360) % 360
          look.current.alt = Math.max(-89.5, Math.min(89.5, (Math.asin(Math.max(-1, Math.min(1, l[2]))) * 180) / Math.PI))
        }
      }
      const { toLocal, toView } = matrices(currentLst(now))
      const s = engine.shared
      s.uEqToLocal.value.set(...toLocal)
      s.uLocalToView.value.set(...toView)
      s.uMilkyOn.value = propsRef.current.layers.has('milkyway') && s.uMilky.value ? 1 : 0
      const table = propsRef.current.refraction
      if (table) s.uRefr.value.set(table)
      else s.uRefr.value.fill(0)
      s.uK.value = projectionK(look.current.fov)
      s.uHalf.value.set((w * dpr) / 2, (h * dpr) / 2)
      s.uGround.value = propsRef.current.layers.has('horizon') ? 1 : 0
      // 放大时星点略增大，暗星才分得开
      s.uScale.value = dpr * Math.pow(60 / look.current.fov, 0.3)
      engine.upView.value.set(toView[2], toView[5], toView[8])
      const sunLocal = observed(apply(toLocal, sunRef.current))
      s.uSunLocal.value.set(sunLocal[0], sunLocal[1], sunLocal[2])
      const moonLocal = observed(apply(toLocal, moonRef.current))
      s.uMoonDir.value.set(moonLocal[0], moonLocal[1], moonLocal[2])
      const sunAlt = (Math.asin(sunLocal[2]) * 180) / Math.PI
      // 太阳在 −12°—+15° 间有晨昏暖色
      const ramp = (x: number, a: number, b: number) => Math.max(0, Math.min(1, (x - a) / (b - a)))
      s.uTwilight.value = propsRef.current.layers.has('skylight') ? ramp(sunAlt, -12, -3) * (1 - ramp(sunAlt, 3, 15)) : 0
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

  // 地平线
  useEffect(() => {
    const e = engineRef.current
    if (!e) return
    const tex = e.shared.uHorizonTex.value
    const data = tex.image.data as Float32Array
    for (let i = 0; i < HORIZON_SAMPLES; i++) data[i * 4] = Math.sin(((horizon?.[i] ?? 0) * Math.PI) / 180)
    tex.needsUpdate = true
    dirty.current = true
  }, [horizon])

  // 星
  useEffect(() => {
    const e = engineRef.current
    if (!e) return
    const vec: number[] = []
    const col: number[] = []
    const siz: number[] = []
    const mag: number[] = []
    for (const s of shownStars) {
      vec.push(...unitVector(s.ra, s.dec))
      const [r, g, b] = parseColor(starColor(s.bv))
      col.push(r, g, b)
      // 点精灵含外圈辉光：实心部分占半径的四成，直径取 5 倍星半径，实心部分才与 SVG 的星一样大
      siz.push(starRadius(s.v) * 5)
      mag.push(s.v)
    }
    const pts = new THREE.Points(pointGeometry(vec, col, siz, mag), e.starMaterial)
    e.starPoints = replaceObject(e, e.starPoints, pts, 2)
    dirty.current = true
  }, [shownStars])

  // 天光网格与太阳方向。关掉天色时给固定的夜空、不限星等
  useEffect(() => {
    const e = engineRef.current
    if (!e) return
    const tex = e.shared.uSkyGrid.value
    const data = tex.image.data as Float32Array
    const on = layers.has('skylight')
    const n = SKY_GRID_AZ * SKY_GRID_ALT
    for (let i = 0; i < n; i++) {
      data[i * 4] = on ? snap.skyLight.lgB[i] : 2.3
      data[i * 4 + 1] = on ? snap.skyLight.lim[i] : 99
    }
    tex.needsUpdate = true
    e.shared.uTotality.value = on && snap.skyLight.sunVisible < 0.001 ? 1 : 0
    const sun = snap.bodies.find((b) => b.name === 'sun')!
    sunRef.current = unitVector(sun.topoRa, sun.topoDec)
    const moonBody = snap.bodies.find((b) => b.name === 'moon')!
    moonRef.current = unitVector(moonBody.topoRa, moonBody.topoDec)
    e.shared.uMoonGlare.value = on ? snap.skyLight.moonGlare : 0
    e.shared.uLimitCurve.value.set(snap.skyLight.limitCurve)
    e.shared.uTrueToGal.value.set(...(snap.trueToGalactic as Mat3))
    e.shared.uSunRadius.value = ((sun.semidiameter ?? 0.27) * Math.PI) / 180
    e.shared.uCoronaUnit.value = snap.skyLight.coronaUnit
    e.shared.uCorona.value = on && snap.skyLight.sunVisible < 0.001 ? 1 : 0
    dirty.current = true
  }, [snap, layers])

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
    const rad: number[] = []
    const mag: number[] = []
    const toSun: number[] = []
    const sun = snap.bodies.find((b) => b.name === 'sun')!
    const moon = snap.bodies.find((b) => b.name === 'moon')!
    // 月→日方向：真向量相减，太阳也用站心方向（月日距比约 1/390，不可略到 0.15°）
    const moonToSun = (() => {
      const su = unitVector(sun.topoRa, sun.topoDec).map((x) => x * sun.topoDistM)
      const mo = unitVector(moon.topoRa, moon.topoDec).map((x) => x * moon.topoDistM)
      const d = [su[0] - mo[0], su[1] - mo[1], su[2] - mo[2]]
      const n = Math.hypot(d[0], d[1], d[2])
      return d.map((x) => x / n)
    })()
    for (const b of snap.bodies) {
      vec.push(...unitVector(b.topoRa, b.topoDec))
      const [r, g, bl] = parseColor(BODY_COLOR[b.name])
      col.push(r, g, bl)
      siz.push(b.name === 'sun' ? 16 : b.name === 'moon' ? 14 : 8)
      rad.push(((b.semidiameter ?? 0) * Math.PI) / 180)
      mag.push(b.mag ?? 99)
      if (b.name === 'moon') toSun.push(...moonToSun)
      else toSun.push(0, 0, 0)
    }
    e.bodyPoints = replaceObject(e, e.bodyPoints, new THREE.Points(pointGeometry(vec, col, siz, mag, rad, toSun), e.bodyMaterial), 3)
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
  }, [picked, zhouTian, lat, magLimit, layers, refraction, tracking])

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
      // 按钮上按下不算拖动：捕获指针后按钮就收不到点击了
      if ((e.target as Element).closest('button, [data-no-drag]')) return
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
      // 手动拖动就不再跟踪
      if (moved > 4) setTracking(false)
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
      const dist = (ra: number, dec: number, radiusDeg = 0) => {
        const v = unitVector(ra, dec)
        const t = apply(toLocal, v)
        const l = observed(t)
        const sr = (radiusDeg * Math.PI) / 180
        if (ground && hiddenByGround(t, l, radiusDeg)) return Infinity
        const vv = apply(toView, l)
        const q = viewToPixel(vv, k, h / 2)
        if (!q) return Infinity
        // 点在圆面内算距离 0
        return Math.max(0, Math.hypot(w / 2 + q[0] - px, h / 2 - q[1] - py) - (sr * k * (h / 2)) / (1 + vv[2]))
      }
      let best: Picked | null = null
      let bestD = 12
      if (p.layers.has('bodies')) {
        for (const b of p.snap.bodies) {
          const d = dist(b.topoRa, b.topoDec, b.semidiameter ?? 0)
          if (d < bestD) {
            bestD = d
            best = { kind: 'body', name: b.name }
          }
        }
      }
      // 看不见的星不可选：与着色器同一判据
      const limit = (ra: number, dec: number) => limitAt(observed(apply(toLocal, unitVector(ra, dec))))
      for (const s of shownRef.current) {
        if (s.v > limit(s.ra, s.dec)) continue
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

  useEffect(() => {
    const on = () => setFullscreen(document.fullscreenElement === hostRef.current)
    document.addEventListener('fullscreenchange', on)
    return () => document.removeEventListener('fullscreenchange', on)
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
        {picked && (
          <button type="button" onClick={() => setTracking(!tracking)} style={{ background: tracking ? 'rgba(40,110,200,0.9)' : 'rgba(20,30,55,0.8)', color: COLORS.text, border: `1px solid ${COLORS.skyEdge}`, borderRadius: 4, padding: '2px 8px', cursor: 'pointer' }}>
            {tracking ? '停止跟踪' : '跟踪选中'}
          </button>
        )}
        <button type="button" onClick={toggleFullscreen} style={{ background: 'rgba(20,30,55,0.8)', color: COLORS.text, border: `1px solid ${COLORS.skyEdge}`, borderRadius: 4, padding: '2px 8px', cursor: 'pointer' }}>
          {fullscreen ? '退出全屏' : '全屏'}
        </button>
      </div>
      {fullscreen && overlay && (
        <div data-no-drag style={{ position: 'absolute', left: '50%', bottom: 12, transform: 'translateX(-50%)', padding: '6px 12px', borderRadius: 6, background: 'rgba(12,20,40,0.85)', color: COLORS.text, cursor: 'default' }}>
          {overlay}
        </div>
      )}
    </div>
  )
}

