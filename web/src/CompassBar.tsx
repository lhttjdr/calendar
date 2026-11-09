/**
 * 罗盘控件（择日、排盘页共用；定位已移常驻顶栏）：罗盘开时给出整盘应转的角度，
 * 使南上北下的盘图与实地方位相合（磁北经 WMM 磁偏角改为真北；平放检测、精度提示）。见 doc/engineering/web-folk-pages.md。
 */
import { Flex, Switch, Typography } from 'antd'
import { CompassOutlined } from '@ant-design/icons'
import { useEffect, useMemo, useRef, type CSSProperties, type ReactNode, type RefObject } from 'react'
import { useCompassHeading } from './sensors'

const { Text } = Typography

export type LocationProps = {
  latitude: number
  longitude: number
  /** 定位：取 GPS 经纬度写回观测地，返回提示 */
  onLocate: () => Promise<string>
  /** 所在地磁偏角（度，东偏为正）；未知为 null */
  declination: number | null
}

/** 罗盘转盘所需：最新读数之 ref、磁偏角、开否（对象在三者不变时保持同一引用，供 React.memo） */
export type CompassSpinSource = { headingRef: RefObject<number | null>; declination: number; active: boolean }

/**
 * 罗盘转盘驱动：每帧读最新朝向，目标角 = −真北方位 − 180°（南上北下之图），以时间常数约 90 ms 的指数趋近、取最短路径（角度连续累加，
 * 跨 0°/360° 不倒转一周），直接写元素 transform（不经 React 重绘、不用 CSS transition，免颤动与重影）；变化小于 0.02° 不写。
 * `svgCenter` 给定时写 SVG `transform` 属性（绕该点转），否则写 CSS `transform`。
 */
export function useSpinDriver(ref: RefObject<HTMLElement | SVGGElement | null>, spin: CompassSpinSource | undefined, svgCenter?: number) {
  useEffect(() => {
    const el = ref.current
    if (!el) return
    const apply = (a: number | null) => {
      if (svgCenter != null) {
        if (a == null) el.removeAttribute('transform')
        else el.setAttribute('transform', `rotate(${a.toFixed(2)} ${svgCenter} ${svgCenter})`)
      } else (el as HTMLElement).style.transform = a == null ? '' : `rotate(${a.toFixed(2)}deg)`
    }
    if (!spin?.active) {
      apply(null)
      return
    }
    let cur: number | null = null
    let written = Number.NaN
    let last = performance.now()
    let raf = 0
    const tick = (now: number) => {
      const h = spin.headingRef.current
      if (h != null) {
        const target = -(h + spin.declination) - 180
        if (cur == null) cur = target
        else {
          const d = ((((target - cur) % 360) + 540) % 360) - 180
          cur += d * (1 - Math.exp(-(now - last) / 90))
        }
        if (!(Math.abs(cur - written) < 0.02)) {
          apply(cur)
          written = cur
        }
      }
      last = now
      raf = requestAnimationFrame(tick)
    }
    raf = requestAnimationFrame(tick)
    return () => {
      cancelAnimationFrame(raf)
      apply(null)
    }
  }, [ref, spin, svgCenter])
}

/** 随罗盘整盘转的容器（排盘诸盘）：开罗盘时成独立合成层（will-change），由 rAF 直接转 */
export function CompassSpin({ spin, style, children }: { spin?: CompassSpinSource; style?: CSSProperties; children: ReactNode }) {
  const ref = useRef<HTMLDivElement | null>(null)
  useSpinDriver(ref, spin)
  return <div ref={ref} style={{ ...style, willChange: spin?.active ? 'transform' : undefined }}>{children}</div>
}

/** 返回整盘旋转角（度；南上北下之图转 −真北方位 − 180°，罗盘关时为 0；节流后的值，只供文字与测向）、控件条与转盘驱动源 `spin` */
export function useCompassBar(p: LocationProps): { rotate: number; bar: ReactNode; trueHeading: number | null; spin: CompassSpinSource } {
  const compass = useCompassHeading()
  const decl = p.declination ?? 0
  const spin = useMemo(() => ({ headingRef: compass.headingRef, declination: decl, active: compass.active }), [compass.headingRef, decl, compass.active])
  const trueHeading = compass.heading == null ? null : compass.heading + (p.declination ?? 0)
  const rotate = trueHeading == null ? 0 : -trueHeading - 180
  const bar = (
    <Flex align="center" gap={8} wrap style={{ marginBottom: 12 }}>
      <Switch size="small" checked={compass.active} onChange={(on) => (on ? compass.start() : compass.stop())} />
      <span><CompassOutlined /> 罗盘（手机：盘随设备转至实地方位）</span>
      {compass.active && p.declination != null && (
        <Text type="secondary" style={{ fontSize: 12 }}>{`磁偏角 ${p.declination >= 0 ? '东' : '西'}${Math.abs(p.declination).toFixed(1)}°`}</Text>
      )}
      {compass.active && trueHeading != null && (
        <Text style={{ fontSize: 12 }}>{`设备朝向 真北 ${((trueHeading % 360) + 360) % 360 | 0}°（磁北 ${compass.heading! | 0}°）`}</Text>
      )}
      {compass.active && compass.tilt != null && (
        compass.tilt > 15
          ? <Text type="warning" style={{ fontSize: 12 }}>{`请将手机放平（倾斜 ${compass.tilt | 0}°）`}</Text>
          : <Text type="success" style={{ fontSize: 12 }}>已水平</Text>
      )}
      {compass.active && compass.source === 'ios' && compass.accuracy != null && (
        compass.accuracy < 0
          ? <Text type="warning" style={{ fontSize: 12 }}>罗盘未校准，请远离金属并将手机画「8」字校准</Text>
          : compass.accuracy > 15
            ? <Text type="warning" style={{ fontSize: 12 }}>{`罗盘精度低（±${Math.round(compass.accuracy)}°），请远离金属并将手机画「8」字校准`}</Text>
            : <Text type="secondary" style={{ fontSize: 12 }}>{`罗盘精度 ±${Math.round(compass.accuracy)}°`}</Text>
      )}
      {compass.active && compass.source === 'absolute' && (
        <Text type="secondary" style={{ fontSize: 12 }}>此浏览器不报罗盘精度；读数不稳或与实地不符时，请远离金属并将手机画「8」字校准</Text>
      )}
      {compass.error && <Text type="danger" style={{ fontSize: 12 }}>{compass.error}</Text>}
    </Flex>
  )
  return { rotate, bar, trueHeading: trueHeading == null ? null : ((trueHeading % 360) + 360) % 360, spin }
}
