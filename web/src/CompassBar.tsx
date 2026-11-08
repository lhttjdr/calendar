/**
 * 定位与罗盘控件（择日、排盘页共用）：GPS 定位写回观测地；罗盘开时给出整盘应转的角度，
 * 使南上北下的盘图与实地方位相合（磁北经 WMM 磁偏角改为真北；平放检测、精度提示）。见 doc/engineering/web-folk-pages.md。
 */
import { Button, Flex, Switch, Typography } from 'antd'
import { AimOutlined, CompassOutlined } from '@ant-design/icons'
import { useState, type ReactNode } from 'react'
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

/** 返回整盘旋转角（度；南上北下之图转 −真北方位 − 180°，罗盘关时为 0）与控件条 */
export function useCompassBar(p: LocationProps): { rotate: number; bar: ReactNode } {
  const compass = useCompassHeading()
  const [locMsg, setLocMsg] = useState('')
  const trueHeading = compass.heading == null ? null : compass.heading + (p.declination ?? 0)
  const rotate = trueHeading == null ? 0 : -trueHeading - 180
  const bar = (
    <Flex align="center" gap={8} wrap style={{ marginBottom: 12 }}>
      <Button size="small" icon={<AimOutlined />} onClick={() => { setLocMsg('定位中…'); p.onLocate().then(setLocMsg, (e) => setLocMsg(String(e?.message ?? e))) }}>
        定位
      </Button>
      <Text style={{ fontSize: 12 }}>
        {`北纬 ${p.latitude.toFixed(4)}° 东经 ${p.longitude.toFixed(4)}°`}
        {p.declination != null ? ` · 磁偏角 ${p.declination >= 0 ? '东' : '西'}${Math.abs(p.declination).toFixed(1)}°` : ''}
      </Text>
      {locMsg && <Text type="secondary" style={{ fontSize: 12 }}>{locMsg}</Text>}
      <span style={{ marginLeft: 8 }}><CompassOutlined /> 罗盘</span>
      <Switch size="small" checked={compass.active} onChange={(on) => (on ? compass.start() : compass.stop())} />
      {compass.active && trueHeading != null && (
        <Text style={{ fontSize: 12 }}>{`设备朝向 真北 ${((trueHeading % 360) + 360) % 360 | 0}°（磁北 ${compass.heading! | 0}°）`}</Text>
      )}
      {compass.active && compass.tilt != null && (
        compass.tilt > 15
          ? <Text type="warning" style={{ fontSize: 12 }}>{`请将手机放平（倾斜 ${compass.tilt | 0}°）`}</Text>
          : <Text type="success" style={{ fontSize: 12 }}>已水平</Text>
      )}
      {compass.active && compass.accuracy != null && (compass.accuracy < 0 || compass.accuracy > 15) && (
        <Text type="warning" style={{ fontSize: 12 }}>罗盘精度低，请远离金属并将手机画「8」字校准</Text>
      )}
      {compass.error && <Text type="danger" style={{ fontSize: 12 }}>{compass.error}</Text>}
    </Flex>
  )
  return { rotate, bar }
}
