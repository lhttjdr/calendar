/**
 * 择日页：年神方位二十四山图（《协纪辨方书》卷三）、当日神煞宜忌（月表 + 卷十铺注 + 时宪书体例）、日游神鹤神。
 * 与日历共用所选日期；图依古图南上北下、东左西右。
 */
import { Button, Card, Checkbox, Flex, InputNumber, Typography } from 'antd'
import { LeftOutlined, RightOutlined } from '@ant-design/icons'
import { useMemo, useState } from 'react'
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
  /** 当月月厌、厌对所在之山；厌前（阳）、厌后（阴）诸山；当日干支与阴阳将 */
  yueYan?: string
  yanDui?: string
  yanQian?: string[]
  yanHou?: string[]
  dayJiang?: string
  dayGz?: string
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
      {data.yueYan && data.shan.map((s, i) => {
        // 外缘色带：厌前（阳）浅红、厌后（阴）浅蓝；月厌、厌对标字（类编以月厌为界分二十四山）
        const a0 = ang(i) - (7.5 * Math.PI) / 180, a1 = ang(i) + (7.5 * Math.PI) / 180
        const rb0 = r2 - size * 0.028, rb1 = r2
        const [x0, y0] = pt(rb1, a0), [x1, y1] = pt(rb1, a1), [x2, y2] = pt(rb0, a1), [x3, y3] = pt(rb0, a0)
        const isYan = s.p === data.yueYan, isDui = s.p === data.yanDui
        const fill = isYan || isDui ? '#ffd666' : data.yanQian?.includes(s.p) ? '#ffccc7' : data.yanHou?.includes(s.p) ? '#bae0ff' : 'none'
        const [lx, ly] = pt((rb0 + rb1) / 2, ang(i))
        return (
          <g key={'yan' + s.p}>
            <path d={`M ${x0} ${y0} A ${rb1} ${rb1} 0 0 1 ${x1} ${y1} L ${x2} ${y2} A ${rb0} ${rb0} 0 0 0 ${x3} ${y3} Z`} fill={fill} opacity={0.8} />
            {(isYan || isDui) && <text x={lx} y={ly} fontSize={size * 0.017} textAnchor="middle" dominantBaseline="central" fill="#ad4e00" transform={`rotate(${(ang(i) * 180) / Math.PI} ${lx} ${ly})`}>{isYan ? '月厭' : '厭對'}</text>}
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

type MonthLunar = { jd0: number; months: number[]; days: number[]; leaps: number[]; dims: number[]; phases: number[] }
type JiaquDay = {
  jdn: number; ok: boolean; verdict?: string; jie?: [string, string][]; yi: boolean; ji: boolean; jieHunYin: boolean; naCai: boolean; nvMingYue: string
  zhouTang: string | null; baiHu: string | null; naXu: string | null; buJiang: boolean; jiang: string
  leibian: [string, string][]; reasons: string[]; notes: string[]
}
const GZ_GAN = '甲乙丙丁戊己庚辛壬癸', GZ_ZHI = '子丑寅卯辰巳午未申酉戌亥'

/** 婚嫁择吉：所显示月份逐日判定（协纪为纲、类编历法通书大全卷七并列；见知识库 jiaqu） */
function JiaquPanel(props: { wasm: LunarBackend; ear: string; monthLunar: MonthLunar; monthLabel: string }) {
  const [nvYear, setNvYear] = useState(2000)
  const [wengGu, setWengGu] = useState(true)
  const [fuMu, setFuMu] = useState(true)
  const [naXu, setNaXu] = useState(false)
  const [useYue, setUseYue] = useState(false)
  const [ztLeibian, setZtLeibian] = useState(false)
  const [useXiong, setUseXiong] = useState(false)
  const [onlyOk, setOnlyOk] = useState(false)
  const [zhiHua, setZhiHua] = useState(false)
  const [aoTou, setAoTou] = useState(false)
  const nvGz = (((nvYear - 4) % 60) + 60) % 60
  const m = props.monthLunar
  const list = useMemo((): JiaquDay[] | null => {
    if (typeof props.wasm.jiaqu_days_json !== 'function') return null
    try {
      return JSON.parse(props.wasm.jiaqu_days_json(m.jd0, props.ear, Uint8Array.from(m.months), Uint8Array.from(m.days), Uint8Array.from(m.leaps), Uint8Array.from(m.dims), Int8Array.from(m.phases), nvGz, wengGu, fuMu, naXu, (useYue ? 1 : 0) | (ztLeibian ? 2 : 0) | (useXiong ? 4 : 0) | (zhiHua ? 8 : 0) | (aoTou ? 16 : 0))) as JiaquDay[]
    } catch { return null }
  }, [props.wasm, props.ear, m, nvGz, wengGu, fuMu, naXu, useYue, ztLeibian, useXiong, zhiHua, aoTou])
  return (
    <Card size="small" title={`婚嫁择吉（${props.monthLabel}）`} style={{ marginTop: 16 }}>
      <Flex gap={12} wrap align="center" style={{ marginBottom: 8 }}>
        <span>女命生年 <InputNumber size="small" value={nvYear} min={1} max={2200} onChange={(v) => v != null && setNvYear(v)} />{` ${GZ_GAN[nvGz % 10]}${GZ_ZHI[nvGz % 12]}（立春换年）`}</span>
        <Checkbox checked={wengGu} onChange={(e) => setWengGu(e.target.checked)}>翁姑在</Checkbox>
        <Checkbox checked={fuMu} onChange={(e) => setFuMu(e.target.checked)}>女父母在</Checkbox>
        <Checkbox checked={naXu} onChange={(e) => setNaXu(e.target.checked)}>纳婿</Checkbox>
      </Flex>
      <Flex gap={12} wrap align="center" style={{ marginBottom: 8 }}>
        <Checkbox checked={useYue} onChange={(e) => setUseYue(e.target.checked)}>女命行嫁月否决（类编；协纪奏准删除）</Checkbox>
        <Checkbox checked={ztLeibian} onChange={(e) => setZtLeibian(e.target.checked)}>周堂从类编</Checkbox>
        <Checkbox checked={useXiong} onChange={(e) => setUseXiong(e.target.checked)}>类编嫁娶凶神否决</Checkbox>
        <Checkbox checked={aoTou} onChange={(e) => setAoTou(e.target.checked)}>白虎从鳌头（值门床堂路）</Checkbox>
        <Checkbox checked={zhiHua} onChange={(e) => setZhiHua(e.target.checked)}>用制化（周堂回避、白虎禳法；协纪不录）</Checkbox>
        <Checkbox checked={onlyOk} onChange={(e) => setOnlyOk(e.target.checked)}>只看可用之日</Checkbox>
      </Flex>
      {!list ? <Text type="secondary">无法判定</Text> : (
        <div style={{ fontSize: 12, lineHeight: 1.7 }}>
          {list.map((d, i) => (onlyOk && !d.ok ? null : (
            <div key={d.jdn} style={{ borderBottom: '1px solid #f0f0f0', padding: '2px 0' }}>
              <Text strong type={d.verdict === '經制化可用' ? 'warning' : d.ok ? 'success' : 'secondary'}>{`${i + 1}日 ${d.verdict ?? (d.ok ? '可用' : '不可用')}`}</Text>
              {` · 农历${m.leaps[i] ? '閏' : ''}${m.months[i]}月${m.days[i]}日`}
              {d.yi ? ' · 宜嫁娶' : d.ji ? ' · 忌嫁娶' : ''}
              {d.buJiang ? ' · 不將（协纪歷例表）' : ''}{d.jiang ? ` · 类编法${d.jiang}` : ''}
              {d.zhouTang ? ` · 周堂值${d.zhouTang}` : ''}
              {` · 女命${d.nvMingYue}`}
              {d.reasons.length > 0 && <Text type="danger">{` · ${d.reasons.join('；')}`}</Text>}
              {d.jie && d.jie.length > 0 && <Text type="warning">{` · 制化：${d.jie.map(([a, b]) => `${a}（${b}）`).join('；')}`}</Text>}
              {d.leibian.length > 0 && <Text type="secondary">{` · 类编凶神：${d.leibian.map(([a]) => a).join(' ')}`}</Text>}
            </div>
          )))}
        </div>
      )}
      <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
        以协纪为纲（月表加铺注之宜忌嫁娶、嫁娶周堂、阴阳不将；协纪于大利月诸说「奏准刪除」，缺省只列不否决），并列《类编历法通书大全》卷七之女命行嫁月、周堂、嫁娶凶神表为选项。只算到规则给出的结果，不论吉凶应验。
      </Text>
    </Card>
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
  monthLunar?: MonthLunar | null
  monthLabel?: string
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
            {data.yueYan && (
              <div style={{ marginBottom: 8 }}>
                {`月厭在${data.yueYan}、厭對在${data.yanDui}；${data.dayGz}日${data.dayJiang}`}
                <Text type="secondary" style={{ fontSize: 11, display: 'block' }}>
                  图外缘色带：浅红为厌前（阳）、浅蓝为厌后（阴）；厌前之干配厌后之支为阴阳不将（《类编历法通书大全》卷七「論隂陽不將日」，戊己寄厌前；协纪戊寄艮己寄坤之说见婚嫁择吉）
                </Text>
              </div>
            )}
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
      {wasm && ear != null && props.monthLunar && props.monthLabel && (
        <JiaquPanel wasm={wasm} ear={ear} monthLunar={props.monthLunar} monthLabel={props.monthLabel} />
      )}
    </div>
  )
}
