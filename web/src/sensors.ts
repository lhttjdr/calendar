/**
 * 设备传感器：GPS 定位（Geolocation API）与罗盘朝向（DeviceOrientation）。
 *
 * 罗盘朝向：
 * - iOS Safari：`webkitCompassHeading`（已是自磁北顺时针的度数）；需在用户手势中调用
 *   `DeviceOrientationEvent.requestPermission()` 授权。
 * - Android Chrome：`deviceorientationabsolute` 事件的 `alpha`（地球坐标系，逆时针），朝向 = 360 − alpha。
 * - 屏幕横竖旋转时加 `screen.orientation.angle`。
 * 罗盘须手机平放：此时顶端所指之方位即读数；立起时 alpha 所表不再是平面朝向，故另报倾角（beta 前后、gamma 左右），
 * 倾角过大时界面提示放平。iOS 的 webkitCompassHeading 已作倾斜补偿，仍以平放为准。
 * 所得为磁北方位；真北 = 磁北方位 + 磁偏角（东偏为正），磁偏角由 WMM 按所在经纬度求（见 doc/astronomy/geomag.md）。
 * 浏览器只在安全上下文（https 或 localhost）提供这些接口。
 */
import { useCallback, useEffect, useRef, useState } from 'react'

export type GeoFix = { latitude: number; longitude: number; altitude: number | null; accuracy: number; time: number }

/** 取当前位置（一次）。失败时 reject 中文原因 */
export function getCurrentPosition(): Promise<GeoFix> {
  return new Promise((resolve, reject) => {
    if (typeof navigator === 'undefined' || !navigator.geolocation) {
      reject(new Error('此浏览器不支持定位'))
      return
    }
    navigator.geolocation.getCurrentPosition(
      (p) =>
        resolve({
          latitude: p.coords.latitude,
          longitude: p.coords.longitude,
          altitude: p.coords.altitude,
          accuracy: p.coords.accuracy,
          time: p.timestamp,
        }),
      (e) =>
        reject(
          new Error(
            e.code === e.PERMISSION_DENIED
              ? '定位权限被拒绝'
              : e.code === e.POSITION_UNAVAILABLE
                ? '无法取得位置'
                : e.code === e.TIMEOUT
                  ? '定位超时'
                  : e.message,
          ),
        ),
      { enableHighAccuracy: true, timeout: 15000, maximumAge: 60000 },
    )
  })
}

type OrientationEventWithCompass = DeviceOrientationEvent & { webkitCompassHeading?: number }
type DOEStatic = typeof DeviceOrientationEvent & { requestPermission?: () => Promise<'granted' | 'denied'> }

function screenAngle(): number {
  const a = typeof screen !== 'undefined' && screen.orientation ? screen.orientation.angle : (window as unknown as { orientation?: number }).orientation
  return typeof a === 'number' ? a : 0
}

/**
 * 罗盘朝向（磁北，度，0—360，设备顶端所指）。`start()` 须在点击等用户手势中调用（iOS 授权）。
 * 读数以角度圆周平滑（指数平均，免得抖动）。
 */
export function useCompassHeading() {
  const [heading, setHeading] = useState<number | null>(null)
  const [active, setActive] = useState(false)
  const [error, setError] = useState<string | null>(null)
  /** 倾角（度）：前后 beta、左右 gamma 中较大者的绝对值；平放约 0 */
  const [tilt, setTilt] = useState<number | null>(null)
  /** iOS 报告的读数精度（度，越小越好；负为未校准） */
  const [accuracy, setAccuracy] = useState<number | null>(null)
  const smooth = useRef<{ x: number; y: number } | null>(null)
  const cleanup = useRef<(() => void) | null>(null)

  const onReading = useCallback((h: number) => {
    const r = (h * Math.PI) / 180
    const k = 0.25
    const s = smooth.current
    smooth.current = s ? { x: s.x + k * (Math.cos(r) - s.x), y: s.y + k * (Math.sin(r) - s.y) } : { x: Math.cos(r), y: Math.sin(r) }
    const v = (Math.atan2(smooth.current.y, smooth.current.x) * 180) / Math.PI
    setHeading((v + 360) % 360)
  }, [])

  const stop = useCallback(() => {
    cleanup.current?.()
    cleanup.current = null
    smooth.current = null
    setActive(false)
    setHeading(null)
    setTilt(null)
    setAccuracy(null)
  }, [])

  const start = useCallback(async () => {
    setError(null)
    if (typeof window === 'undefined' || typeof DeviceOrientationEvent === 'undefined') {
      setError('此设备不支持方向传感器')
      return
    }
    const DOE = DeviceOrientationEvent as DOEStatic
    if (typeof DOE.requestPermission === 'function') {
      try {
        if ((await DOE.requestPermission()) !== 'granted') {
          setError('方向传感器权限被拒绝')
          return
        }
      } catch {
        setError('方向传感器权限请求失败（须在点击后请求）')
        return
      }
    }
    const handler = (e: Event) => {
      const ev = e as OrientationEventWithCompass & { webkitCompassAccuracy?: number }
      if (ev.beta != null && ev.gamma != null) setTilt(Math.max(Math.abs(ev.beta), Math.abs(ev.gamma)))
      if (typeof ev.webkitCompassAccuracy === 'number') setAccuracy(ev.webkitCompassAccuracy)
      if (typeof ev.webkitCompassHeading === 'number' && !Number.isNaN(ev.webkitCompassHeading)) {
        onReading(ev.webkitCompassHeading + screenAngle())
      } else if (ev.absolute && ev.alpha != null) {
        onReading(360 - ev.alpha + screenAngle())
      }
    }
    const absName = 'ondeviceorientationabsolute' in window ? 'deviceorientationabsolute' : 'deviceorientation'
    window.addEventListener(absName, handler, true)
    cleanup.current = () => window.removeEventListener(absName, handler, true)
    setActive(true)
    // 数秒无读数则提示（桌面浏览器常有事件而无数据）
    window.setTimeout(() => {
      if (smooth.current == null && cleanup.current) setError('未收到罗盘读数（设备可能没有磁力计，或浏览器未提供绝对方向）')
    }, 3000)
  }, [onReading])

  useEffect(() => () => cleanup.current?.(), [])
  return { heading, active, error, tilt, accuracy, start, stop }
}
