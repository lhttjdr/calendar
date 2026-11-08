/**
 * 择日页：年神方位二十四山图（《协纪辨方书》卷三）、当日神煞宜忌（月表 + 卷十铺注 + 时宪书体例）、日游神鹤神。
 * 与日历共用所选日期；图依古图南上北下、东左西右。
 */
import { Button, Card, Flex, Typography } from 'antd'
import { LeftOutlined, RightOutlined } from '@ant-design/icons'
import { useMemo } from 'react'
import type { LunarBackend } from './lunar-backend-types'
import { useCompassBar } from './CompassBar'

const { Text } = Typography

type NianShenData = {
  year: string
  suiDe: string
  suiDeHe: string
  shan: { p: string; gods: { n: string; ji: boolean }[] }[]
  riYou: string
  heShen: string
  taiSuiYou: string
}

/** 太岁、岁破等最重之神 */
const MAJOR = new Set(['太歲', '歲破', '大將軍', '劫煞', '災煞', '歲煞'])
const WEI = new Set(['乾', '坤', '艮', '巽'])
const GAN = new Set(['甲', '乙', '丙', '丁', '庚', '辛', '壬', '癸'])

/** `rotate`：整盘顺时针旋转的角度（罗盘模式由设备朝向给出）；文字随盘转 */
export function ShanWheel({ data, size = 560, rotate = 0 }: { data: NianShenData; size?: number; rotate?: number }) {
  const c = size / 2
  // 内圈年干支，中圈二十四山，外圈诸神（沿半径向外书写，按弧长等距排开）
  const r0 = size * 0.17
  const r1 = size * 0.25
  const r2 = size * 0.46
  // 二十四山自子起顺行，子在下（南上北下），东在左
  const ang = (i: number) => ((180 + 15 * i) * Math.PI) / 180
  const pt = (r: number, a: number) => [c + r * Math.sin(a), c - r * Math.cos(a)] as const
  return (
    <svg viewBox={`0 0 ${size} ${size}`} style={{ width: '100%', maxWidth: size, height: 'auto' }} role="img" aria-label="年神方位二十四山图">
      <g transform={`rotate(${rotate} ${c} ${c})`} style={{ transition: 'transform 120ms linear' }}>
      <circle cx={c} cy={c} r={r2} fill="var(--zr-bg, #fffdf7)" stroke="#999" />
      <circle cx={c} cy={c} r={r1} fill="none" stroke="#999" />
      <circle cx={c} cy={c} r={r0} fill="none" stroke="#bbb" />
      {data.shan.map((s, i) => {
        const a0 = ang(i) - (7.5 * Math.PI) / 180
        const [x1, y1] = pt(r0, a0)
        const [x2, y2] = pt(r2, a0)
        const a = ang(i)
        const [lx, ly] = pt((r0 + r1) / 2, a)
        const kind = WEI.has(s.p) ? '#7a4f00' : GAN.has(s.p) ? '#555' : '#000'
        const deg = (a * 180) / Math.PI
        // 罗盘式：每字字头朝外（绕圆心一周统一旋转，不翻转）
        return (
          <g key={s.p}>
            <line x1={x1} y1={y1} x2={x2} y2={y2} stroke="#ddd" />
            <text x={lx} y={ly} fontSize={size * 0.032} fontWeight={WEI.has(s.p) || !GAN.has(s.p) ? 600 : 400} fill={kind} textAnchor="middle" dominantBaseline="central" transform={`rotate(${deg} ${lx} ${ly})`}>
              {s.p}
            </text>
            {s.gods.length === 0 ? (
              (() => {
                const [tx, ty] = pt(r1 + 14, a)
                return <text x={tx} y={ty} fontSize={size * 0.02} fill="#bbb" textAnchor="middle" dominantBaseline="central" transform={`rotate(${deg} ${tx} ${ty})`}>空</text>
              })()
            ) : (
              (() => {
                // 诸神竖排：每位一竖行沿半径，字头朝外，自外端起读；一山之内诸神由内向外依次排，放不下时分列
                const n = s.gods.length
                const fs = size * 0.021
                const step = fs * 1.05
                const gap = fs * 0.6
                const avail = r2 - r1 - 8
                const len = (g: { n: string }) => g.n.length * step + gap
                let cols = 1
                while (cols < 3) {
                  const rows = Math.ceil(n / cols)
                  const need = Math.max(...Array.from({ length: cols }, (_, c) => s.gods.slice(c * rows, (c + 1) * rows).reduce((t, g) => t + len(g), 0)))
                  if (need <= avail) break
                  cols++
                }
                const rows = Math.ceil(n / cols)
                const rm = (r1 + r2) / 2
                return s.gods.flatMap((g, k) => {
                  const col = Math.floor(k / rows)
                  const inner = r1 + 6 + s.gods.slice(col * rows, k).reduce((t, x) => t + len(x), 0)
                  const off = ((col - (cols - 1) / 2) * fs * 1.25) / rm
                  const chars = [...g.n]
                  const fill = g.ji ? '#237804' : MAJOR.has(g.n) ? '#a8071a' : '#333'
                  return chars.map((ch, ci) => {
                    // 首字在最外
                    const r = inner + (chars.length - 1 - ci) * step + step / 2
                    const [tx, ty] = pt(r, a + off)
                    return (
                      <text key={`${g.n}-${ci}`} x={tx} y={ty} fontSize={fs} fill={fill} fontWeight={MAJOR.has(g.n) ? 600 : 400}
                        textAnchor="middle" dominantBaseline="central" transform={`rotate(${deg} ${tx} ${ty})`}>
                        {ch}
                      </text>
                    )
                  })
                })
              })()
            )}
          </g>
        )
      })}
      {[['南', 0], ['西', 90], ['北', 180], ['東', 270]].map(([t, d]) => {
        const [x, y] = pt(r2 + size * 0.015, ((d as number) * Math.PI) / 180)
        return <text key={t as string} x={x} y={y} fontSize={size * 0.022} fill={t === '北' ? '#a8071a' : '#888'} textAnchor="middle" dominantBaseline="central" transform={`rotate(${d} ${x} ${y})`}>{t}</text>
      })}
      </g>
      <text x={c} y={c - 8} fontSize={size * 0.03} textAnchor="middle" fontWeight={600}>{data.year}</text>
      <text x={c} y={c + 14} fontSize={size * 0.022} textAnchor="middle" fill="#237804">{`歲德${data.suiDe} 合${data.suiDeHe}`}</text>
      {rotate !== 0 && <path d={`M ${c} ${size * 0.005} l -8 14 l 16 0 z`} fill="#a8071a" />}
    </svg>
  )
}

export function ZeRiPage(props: {
  wasm: LunarBackend | null
  ear: string | null
  /** 所选民用日（任一时刻）的 JD */
  jd: number
  dateLabel: string
  /** shensha_day_wasm 的输出（吉凶神、宜、忌、時憲書宜、時憲書朱字），无则空 */
  shensha: string
  jianchu: string
  onShiftDay: (n: number) => void
  latitude: number
  longitude: number
  /** 定位：取 GPS 经纬度写回观测者位置 */
  onLocate: () => Promise<string>
  /** 所在地磁偏角（度，东偏为正）；未知为 null */
  declination: number | null
}) {
  const { wasm, ear, jd } = props
  const { rotate, bar: compassBar } = useCompassBar(props)
  const data = useMemo((): NianShenData | null => {
    if (!wasm || ear == null || typeof wasm.nian_shen_json !== 'function') return null
    try {
      return JSON.parse(wasm.nian_shen_json(jd, ear)) as NianShenData
    } catch {
      return null
    }
  }, [wasm, ear, jd])
  return (
    <div>
      <Flex align="center" gap={8} wrap style={{ marginBottom: 12 }}>
        <Button size="small" icon={<LeftOutlined />} onClick={() => props.onShiftDay(-1)}>前一日</Button>
        <Text strong>{props.dateLabel}</Text>
        <Button size="small" onClick={() => props.onShiftDay(1)}>后一日 <RightOutlined /></Button>
        <Text type="secondary" style={{ fontSize: 12 }}>与日历所选日期同步</Text>
      </Flex>
      {compassBar}
      {!data ? (
        <Text type="secondary">加载历表后显示（需 VSOP87 历表）</Text>
      ) : (
        <Flex gap={16} wrap align="flex-start">
          <Card size="small" title={`年神方位（${data.year}年，立春换年）`} style={{ flex: '1 1 420px', maxWidth: 600 }}>
            <ShanWheel data={data} rotate={rotate} />
            <Text type="secondary" style={{ fontSize: 11, display: 'block' }}>
              据《协纪辨方书》卷三；南上北下，东左西右。打开罗盘（手机），盘随设备转动，图上方位与实地相合（磁北经磁偏角改为真北；磁偏角未知时按磁北）。绿为吉神（岁德、奏书、博士），红为太岁、岁破、大将军、三煞。时宪书附言：「惟奏書博士宜向之，餘各有所忌」。
            </Text>
          </Card>
          <Card size="small" title="当日" style={{ flex: '1 1 320px' }}>
            {props.jianchu && <div style={{ marginBottom: 8 }}>{props.jianchu}</div>}
            <div style={{ marginBottom: 8 }}>
              {`日遊神${data.riYou} · 鶴神${data.heShen}`}
              {data.taiSuiYou ? ` · 太歲${data.taiSuiYou}遊` : ''}
            </div>
            {props.shensha ? (
              <div style={{ whiteSpace: 'pre-line', fontSize: 13, lineHeight: 1.7 }}>{props.shensha}</div>
            ) : (
              <Text type="secondary">神煞宜忌需农历与岁数据，加载后显示</Text>
            )}
            <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
              据《协纪辨方书》月表加卷十铺注（反支、上朔、四離四絕、晦日、二至二分、土王用事、伏社、朔弦望、月忌、人神、長短星、氣往亡、無祿），并依时宪书体例计嫁娶周堂、交节日；「時憲書宜」为民用三十七事；「時憲書朱字」为底栏朱字应值之神（含钦天监沿用的通书之神荒蕪、枯焦、四不祥、天獄）。日遊神、鶴神、太歲出遊见卷三。
            </Text>
          </Card>
        </Flex>
      )}
    </div>
  )
}
