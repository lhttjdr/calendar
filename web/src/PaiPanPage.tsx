/**
 * 排盘页：大六壬天地盘与四课三传、太乙盘（十六神与九宫）、奇门遁甲（定局；布盘待接入）。
 * 与日历共用所选时刻；日柱、时支取日历所示四柱。图依古式南上北下、东左西右。
 * 规则与出处见知识库 doc/folk/liuren.md、taiyi.md、qimen.md。
 */
import { Button, Flex, Radio, Switch, Tabs, Typography } from 'antd'
import { LeftOutlined, RightOutlined } from '@ant-design/icons'
import { useMemo, useState } from 'react'
import type { LunarBackend } from './lunar-backend-types'
import { useCompassBar, type LocationProps } from './CompassBar'

const { Text } = Typography

const ZHI = '子丑寅卯辰巳午未申酉戌亥'

/** 罗盘模式：方位盘（九宫、天地盘、十六神盘）整盘旋转至实地方位；紫微命盘不对应实地方位，不转 */
const spinStyle = (deg: number) => (deg === 0 ? {} : { transform: `rotate(${deg}deg)`, transition: 'transform 120ms linear', margin: '10% 0' })

type LiurenData = { jiang: string; shi: string; pan: string[]; siKe: { xia: string; shang: string }[]; sanChuan: string[]; fa: string }
type TaiyiData = {
  kind: string; r: number; dun: string; yuan: string; ju: number; ji: number; gong: number; ruGong: number
  tianMu: string; tianMuPos: string; jiShen: string; shiJi: string; shiJiPos: string
  zhuSuan: number; keSuan: number; zhuDa: number | null; zhuCan: number | null; keDa: number | null; keCan: number | null
}

type QimenData = {
  ju: string; xunShou: string; xunYi: string; zhiFu: string; zhiShi: string; zhiFuGong: number; zhiShiGong: number
  gong: { di: string[]; tian: string[]; xing: string[]; men: string | null; shen: string | null }[]
}
/** 洛书九宫，南上：巽四 离九 坤二 / 震三 中五 兑七 / 艮八 坎一 乾六 */
const LUOSHU: [number, string][][] = [[[4, '巽'], [9, '離'], [2, '坤']], [[3, '震'], [5, '中'], [7, '兌']], [[8, '艮'], [1, '坎'], [6, '乾']]]

function QimenBoard({ d }: { d: QimenData }) {
  const s = 120
  return (
    <svg viewBox={`0 0 ${s * 3} ${s * 3}`} style={{ width: '100%', maxWidth: 420 }} role="img" aria-label="奇门遁甲九宫盘">
      {LUOSHU.flatMap((row, ry) => row.map(([g, gua], rx) => {
        const c = d.gong[g - 1]!
        const x = rx * s, y = ry * s
        const hot = g === d.zhiFuGong || g === d.zhiShiGong
        return (
          <g key={g}>
            <rect x={x + 1} y={y + 1} width={s - 2} height={s - 2} fill={hot ? '#fff7e6' : '#fffdf7'} stroke="#999" />
            <text x={x + 6} y={y + 16} fontSize={12} fill="#888">{`${gua}${g}`}</text>
            {c.shen && <text x={x + s / 2} y={y + 18} fontSize={14} textAnchor="middle" fill="#531dab">{c.shen}</text>}
            <text x={x + s - 8} y={y + 18} fontSize={18} textAnchor="end" fill="#a8071a">{c.tian.join('')}</text>
            <text x={x + s / 2} y={y + s / 2 + 6} fontSize={17} textAnchor="middle" fontWeight={c.xing.includes(d.zhiFu) ? 700 : 400}>{c.xing.join(' ')}</text>
            {c.men && <text x={x + 8} y={y + s - 10} fontSize={16} fill="#0958d9" fontWeight={c.men === d.zhiShi ? 700 : 400}>{c.men}</text>}
            <text x={x + s - 8} y={y + s - 10} fontSize={18} textAnchor="end">{c.di.join('')}</text>
          </g>
        )
      }))}
    </svg>
  )
}

function QimenPanel(props: { wasm: LunarBackend; ear: string; jdUt: number; hourGz: number; rotate: number }) {
  const [method, setMethod] = useState(0)
  const [jiGong, setJiGong] = useState(0)
  const [shenMing, setShenMing] = useState(0)
  const [dingJu, setDingJu] = useState(0)
  const [source, setSource] = useState(0)
  const d = useMemo((): QimenData | null => {
    if (typeof props.wasm.qimen_json !== 'function') return null
    try { return JSON.parse(props.wasm.qimen_json(props.jdUt, props.ear, props.hourGz, method, jiGong, shenMing, dingJu, source)) as QimenData } catch { return null }
  }, [props.wasm, props.ear, props.jdUt, props.hourGz, method, jiGong, shenMing, dingJu, source])
  return (
    <Flex gap={16} wrap align="flex-start">
      <div style={{ flex: '1 1 320px', maxWidth: 440, ...spinStyle(props.rotate) }}>{d ? <QimenBoard d={d} /> : <Text type="secondary">无法布盘</Text>}</div>
      <div style={{ flex: '1 1 280px' }}>
        {d && (
          <div style={{ marginBottom: 10, lineHeight: 1.8 }}>
            <div style={{ fontSize: 16 }}>{d.ju}</div>
            <div>{`旬首${d.xunShou}（遁${d.xunYi}）· 直符${d.zhiFu}加${d.zhiFuGong}宫 · 直使${d.zhiShi}加${d.zhiShiGong}宫`}</div>
          </div>
        )}
        <Flex vertical gap={6}>
          <Radio.Group size="small" value={method} onChange={(e) => setMethod(e.target.value)} optionType="button"
            options={[{ label: '转盘', value: 0 }, { label: '飞盘（未核）', value: 1 }]} />
          <Radio.Group size="small" value={jiGong} onChange={(e) => setJiGong(e.target.value)} optionType="button"
            options={[{ label: '中五寄坤', value: 0 }, { label: '阳坤阴艮', value: 1 }, { label: '阳巽阴震', value: 2 }]} />
          <Radio.Group size="small" value={shenMing} onChange={(e) => setShenMing(e.target.value)} optionType="button"
            options={[{ label: '白虎玄武（演义）', value: 0 }, { label: '勾陈朱雀（集成）', value: 1 }]} />
          <Radio.Group size="small" value={dingJu} onChange={(e) => setDingJu(e.target.value)} optionType="button"
            options={[{ label: '置闰（演义）', value: 0 }, { label: '置闰（统宗）', value: 1 }, { label: '拆补', value: 2 }]} />
          <Radio.Group size="small" value={source} onChange={(e) => setSource(e.target.value)} optionType="button"
            options={[{ label: '节气从行用历', value: 0 }, { label: '定气', value: 1 }]} />
        </Flex>
        <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
          每宫：上中八神、右上天盘干（红）、中九星、左下八门（蓝）、右下地盘干；直符、直使所加之宫底色标出。定局据《遁甲演義》卷一、卷三；转盘布法据演义、《太白陰經》卷九、《古今圖書集成》卷七百八；寄宫三说以集成十八局表衡量，寄坤最合。飞盘原典未见，出处未核。
        </Text>
      </div>
    </Flex>
  )
}

type ZiweiData = {
  palaces: string[]; gan: string[]; stars: [string, number][]; siHua: [string, string][]
  ming: number; shen: number; ju: number; juName: string; mingZhu: string; shenZhu: string
  daXian: [number, number, number][]; changSheng: string[]; boShi: string[]
}
const MAIN14 = new Set(['紫微', '天機', '太陽', '武曲', '天同', '廉貞', '天府', '太陰', '貪狼', '巨門', '天相', '天梁', '七殺', '破軍'])

function ZiweiBoard({ d }: { d: ZiweiData }) {
  const s = 120
  const hua = new Map(d.siHua.map(([h, st]) => [st, h.replace('化', '')]))
  return (
    <svg viewBox={`0 0 ${s * 4} ${s * 4}`} style={{ width: '100%', maxWidth: 520 }} role="img" aria-label="紫微斗数命盘">
      {LIUREN_CELLS.map(([z, cx, cy]) => {
        const zi = ZHI.indexOf(z)
        const stars = d.stars.filter(([, k]) => k === zi).map(([n]) => n)
        const main = stars.filter((n) => MAIN14.has(n)), minor = stars.filter((n) => !MAIN14.has(n))
        const dx = d.daXian.find(([, , k]) => k === zi)
        const x = cx * s, y = cy * s
        const isMing = zi === d.ming, isShen = zi === d.shen
        return (
          <g key={z}>
            <rect x={x + 1} y={y + 1} width={s - 2} height={s - 2} fill={isMing ? '#fff1b8' : '#fffdf7'} stroke="#999" />
            {main.map((n, i) => (
              <text key={n} x={x + 8} y={y + 18 + i * 17} fontSize={15} fontWeight={600} fill="#a8071a">
                {n}{hua.get(n) ? <tspan fontSize={11} fill="#0958d9">{` ${hua.get(n)}`}</tspan> : null}
              </text>
            ))}
            {minor.map((n, i) => (
              <text key={n} x={x + s - 6} y={y + 15 + i * 13} fontSize={11} textAnchor="end" fill="#333">
                {n}{hua.get(n) ? <tspan fill="#0958d9">{hua.get(n)}</tspan> : null}
              </text>
            ))}
            <text x={x + 6} y={y + s - 22} fontSize={10} fill="#888">{`${d.changSheng[zi]} ${d.boShi[zi]}`}</text>
            {dx && <text x={x + 6} y={y + s - 8} fontSize={10} fill="#888">{`${dx[0]}—${dx[1]}`}</text>}
            <text x={x + s - 6} y={y + s - 8} fontSize={13} textAnchor="end" fontWeight={isMing ? 700 : 400}>
              {`${d.palaces[zi]}${isShen ? '·身' : ''} ${d.gan[zi]}${z}`}
            </text>
          </g>
        )
      })}
      <text x={s * 2} y={s * 1.6} fontSize={16} textAnchor="middle" fontWeight={600}>{d.juName}</text>
      <text x={s * 2} y={s * 1.95} fontSize={13} textAnchor="middle">{`命主${d.mingZhu} · 身主${d.shenZhu}`}</text>
      <text x={s * 2} y={s * 2.3} fontSize={12} textAnchor="middle" fill="#0958d9">{d.siHua.map(([h, st]) => `${st}${h}`).join(' ')}</text>
    </svg>
  )
}

function ZiweiPanel(props: { wasm: LunarBackend; lunar: { yearGz: number; month: number; leap: boolean; day: number }; hourZhi: number }) {
  const [male, setMale] = useState(true)
  const [leap, setLeap] = useState(0)
  const [xin, setXin] = useState(0)
  const [bingDing, setBingDing] = useState(0)
  const [siHua, setSiHua] = useState(0)
  const [cs, setCs] = useState(0)
  const { lunar } = props
  const d = useMemo((): ZiweiData | null => {
    if (typeof props.wasm.ziwei_json !== 'function') return null
    try {
      return JSON.parse(props.wasm.ziwei_json(lunar.yearGz, lunar.month, lunar.leap, lunar.day, props.hourZhi, male, leap, xin, bingDing, siHua, cs)) as ZiweiData
    } catch { return null }
  }, [props.wasm, lunar.yearGz, lunar.month, lunar.leap, lunar.day, props.hourZhi, male, leap, xin, bingDing, siHua, cs])
  const G = '甲乙丙丁戊己庚辛壬癸', Z = '子丑寅卯辰巳午未申酉戌亥'
  return (
    <Flex gap={16} wrap align="flex-start">
      <div style={{ flex: '1 1 360px', maxWidth: 540 }}>{d ? <ZiweiBoard d={d} /> : <Text type="secondary">无法起盘</Text>}</div>
      <div style={{ flex: '1 1 260px' }}>
        <div style={{ marginBottom: 8 }}>
          {`生于${G[lunar.yearGz % 10]}${Z[lunar.yearGz % 12]}年 ${lunar.leap ? '閏' : ''}${lunar.month}月${lunar.day}日 ${Z[props.hourZhi]}时`}
          <Text type="secondary" style={{ fontSize: 11, display: 'block' }}>以日历所选时刻为生时；年以农历正月初一换</Text>
        </div>
        <Flex vertical gap={6}>
          <Radio.Group size="small" value={male} onChange={(e) => setMale(e.target.value)} optionType="button" options={[{ label: '男命', value: true }, { label: '女命', value: false }]} />
          <Radio.Group size="small" value={leap} onChange={(e) => setLeap(e.target.value)} optionType="button"
            options={[{ label: '闰月作下月（全书）', value: 0 }, { label: '作本月', value: 1 }, { label: '十五日为界', value: 2 }]} />
          <Radio.Group size="small" value={siHua} onChange={(e) => setSiHua(e.target.value)} optionType="button"
            options={[{ label: '四化：全书', value: 0 }, { label: '近人表（未核）', value: 1 }]} />
          <Radio.Group size="small" value={bingDing} onChange={(e) => setBingDing(e.target.value)} optionType="button"
            options={[{ label: '丙丁钺酉', value: 0 }, { label: '钺戌（录文）', value: 1 }]} />
          <Radio.Group size="small" value={xin} onChange={(e) => setXin(e.target.value)} optionType="button"
            options={[{ label: '辛魁午钺寅', value: 0 }, { label: '魁寅钺午', value: 1 }]} />
          <Radio.Group size="small" value={cs} onChange={(e) => setCs(e.target.value)} optionType="button"
            options={[{ label: '长生男顺女逆（全书）', value: 0 }, { label: '阳男阴女顺（未核）', value: 1 }]} />
        </Flex>
        <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
          十四主星派，据《紫微斗數全書》卷二（安身命、五局安紫微、诸星诀、四化），韩国国立中央图书馆藏写本《紫微斗數方書》旁证。红字主星，蓝字四化；宫内下方为长生、博士十二神与大限岁数。各家异说并列，缺省取全书。
        </Text>
      </div>
    </Flex>
  )
}

/** 十二支方框（地盘）：巳午未申在上，亥子丑寅在下（南上北下） */
const LIUREN_CELLS: [string, number, number][] = [
  ['巳', 0, 0], ['午', 1, 0], ['未', 2, 0], ['申', 3, 0],
  ['辰', 0, 1], ['酉', 3, 1],
  ['卯', 0, 2], ['戌', 3, 2],
  ['寅', 0, 3], ['丑', 1, 3], ['子', 2, 3], ['亥', 3, 3],
]

function LiurenBoard({ d }: { d: LiurenData }) {
  const s = 84
  return (
    <svg viewBox={`0 0 ${s * 4} ${s * 4}`} style={{ width: '100%', maxWidth: 360 }} role="img" aria-label="大六壬天地盘">
      {LIUREN_CELLS.map(([z, cx, cy]) => {
        const tian = d.pan[ZHI.indexOf(z)]!
        const hot = d.sanChuan.includes(tian)
        return (
          <g key={z}>
            <rect x={cx * s + 1} y={cy * s + 1} width={s - 2} height={s - 2} fill={z === d.shi ? '#fff7e6' : '#fffdf7'} stroke="#999" />
            <text x={cx * s + s / 2} y={cy * s + s * 0.45} fontSize={s * 0.36} textAnchor="middle" dominantBaseline="central" fill={hot ? '#a8071a' : '#000'} fontWeight={hot ? 700 : 500}>{tian}</text>
            <text x={cx * s + s / 2} y={cy * s + s * 0.82} fontSize={s * 0.18} textAnchor="middle" fill="#888">{z}</text>
          </g>
        )
      })}
      <text x={s * 2} y={s * 1.75} fontSize={s * 0.2} textAnchor="middle">{`${d.jiang}將加${d.shi}時`}</text>
      <text x={s * 2} y={s * 2.15} fontSize={s * 0.17} textAnchor="middle" fill="#888">大字天盘 · 小字地盘</text>
    </svg>
  )
}

function LiurenPanel(props: { wasm: LunarBackend; ear: string; jdUt: number; dayGz: number; hourZhi: number; rotate: number }) {
  const [rule, setRule] = useState(0)
  const d = useMemo((): LiurenData | null => {
    if (typeof props.wasm.liuren_json !== 'function') return null
    try { return JSON.parse(props.wasm.liuren_json(props.jdUt, props.ear, props.dayGz, props.hourZhi, rule)) as LiurenData } catch { return null }
  }, [props.wasm, props.ear, props.jdUt, props.dayGz, props.hourZhi, rule])
  if (!d) return <Text type="secondary">无法起课</Text>
  return (
    <Flex gap={16} wrap align="flex-start">
      <div style={{ flex: '1 1 300px', maxWidth: 380, ...spinStyle(props.rotate) }}><LiurenBoard d={d} /></div>
      <div style={{ flex: '1 1 260px' }}>
        <div style={{ marginBottom: 8 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>换将</Text>{' '}
          <Radio.Group size="small" value={rule} onChange={(e) => setRule(e.target.value)} optionType="button"
            options={[{ label: '中气（六壬大全）', value: 0 }, { label: '节（未核）', value: 1 }]} />
        </div>
        <div style={{ marginBottom: 8 }}>
          <Text strong>四课</Text>
          <Flex gap={4} style={{ marginTop: 4 }}>
            {[...d.siKe].reverse().map((k, i) => (
              <div key={i} style={{ border: '1px solid #ccc', padding: '2px 8px', textAlign: 'center', fontSize: 16 }}>
                <div>{k.shang}</div>
                <div style={{ color: '#888' }}>{k.xia}</div>
              </div>
            ))}
          </Flex>
          <Text type="secondary" style={{ fontSize: 11 }}>自右而左一课至四课；上为上神，下为日干、日支及其上神</Text>
        </div>
        <div>
          <Text strong>三传</Text>（{d.fa}）
          <div style={{ fontSize: 18, letterSpacing: 6 }}>{d.sanChuan.join(' ')}</div>
          <Text type="secondary" style={{ fontSize: 11 }}>初传、中传、末传。据《六壬大全》卷一九法、卷五订讹</Text>
        </div>
      </div>
    </Flex>
  )
}

/** 5×5：外圈十六神位（南上），内圈洛书九宫 */
const POS16_CELLS: [string, number, number][] = [
  ['巽', 0, 0], ['巳', 1, 0], ['午', 2, 0], ['未', 3, 0], ['坤', 4, 0],
  ['申', 4, 1], ['酉', 4, 2], ['戌', 4, 3],
  ['乾', 4, 4], ['亥', 3, 4], ['子', 2, 4], ['丑', 1, 4], ['艮', 0, 4],
  ['寅', 0, 3], ['卯', 0, 2], ['辰', 0, 1],
]
const GONG9: [number, number, number][] = [[4, 1, 1], [9, 2, 1], [2, 3, 1], [3, 1, 2], [5, 2, 2], [7, 3, 2], [8, 1, 3], [1, 2, 3], [6, 3, 3]]

function TaiyiBoard({ d }: { d: TaiyiData }) {
  const s = 66
  const marks = (g: number) => {
    const m: string[] = []
    if (d.gong === g) m.push('太乙')
    if (d.zhuDa === g) m.push('主大')
    if (d.zhuCan === g) m.push('主參')
    if (d.keDa === g) m.push('客大')
    if (d.keCan === g) m.push('客參')
    return m
  }
  return (
    <svg viewBox={`0 0 ${s * 5} ${s * 5}`} style={{ width: '100%', maxWidth: 380 }} role="img" aria-label="太乙盘">
      {POS16_CELLS.map(([p, cx, cy]) => {
        const m: string[] = []
        if (d.tianMuPos === p) m.push('文昌')
        if (d.jiShen === p) m.push('計神')
        if (d.shiJiPos === p) m.push('始擊')
        return (
          <g key={p}>
            <rect x={cx * s + 1} y={cy * s + 1} width={s - 2} height={s - 2} fill="#fffdf7" stroke="#bbb" />
            <text x={cx * s + s / 2} y={cy * s + 16} fontSize={13} textAnchor="middle" fill="#888">{p}</text>
            {m.map((t, i) => <text key={t} x={cx * s + s / 2} y={cy * s + 34 + i * 14} fontSize={12} textAnchor="middle" fill="#a8071a">{t}</text>)}
          </g>
        )
      })}
      {GONG9.map(([g, cx, cy]) => (
        <g key={g}>
          <rect x={cx * s + 1} y={cy * s + 1} width={s - 2} height={s - 2} fill={d.gong === g ? '#fff1b8' : '#fff'} stroke="#999" />
          <text x={cx * s + 8} y={cy * s + 16} fontSize={12} fill="#888">{g}</text>
          {marks(g).map((t, i) => <text key={t} x={cx * s + s / 2} y={cy * s + 28 + i * 13} fontSize={12} textAnchor="middle" fill={t === '太乙' ? '#ad4e00' : '#333'} fontWeight={t === '太乙' ? 700 : 400}>{t}</text>)}
        </g>
      ))}
    </svg>
  )
}

function TaiyiPanel(props: { wasm: LunarBackend; ear: string; jdUt: number; dayGz: number; hourZhi: number; rotate: number }) {
  const [kind, setKind] = useState(3)
  const [school, setSchool] = useState(0)
  const [dun, setDun] = useState(0)
  const d = useMemo((): TaiyiData | null => {
    if (typeof props.wasm.taiyi_json !== 'function') return null
    try { return JSON.parse(props.wasm.taiyi_json(props.jdUt, props.ear, props.dayGz, props.hourZhi, kind, school, dun)) as TaiyiData } catch { return null }
  }, [props.wasm, props.ear, props.jdUt, props.dayGz, props.hourZhi, kind, school, dun])
  return (
    <Flex gap={16} wrap align="flex-start">
      <div style={{ flex: '1 1 300px', maxWidth: 400, ...spinStyle(props.rotate) }}>{d ? <TaiyiBoard d={d} /> : <Text type="secondary">无法起局</Text>}</div>
      <div style={{ flex: '1 1 260px' }}>
        <Flex vertical gap={6}>
          <Radio.Group size="small" value={kind} onChange={(e) => setKind(e.target.value)} optionType="button"
            options={[{ label: '岁计', value: 0 }, { label: '月计', value: 1 }, { label: '日计', value: 2 }, { label: '时计', value: 3 }]} />
          {kind !== 3 && (
            <Radio.Group size="small" value={dun} onChange={(e) => setDun(e.target.value)} optionType="button"
              options={[{ label: '阳遁', value: 0 }, { label: '阴遁', value: 1 }]} />
          )}
          {kind === 0 && (
            <Radio.Group size="small" value={school} onChange={(e) => setSchool(e.target.value)} optionType="button"
              options={[{ label: '金镜积年', value: 0 }, { label: '淘金歌积年', value: 1 }]} />
          )}
        </Flex>
        {d && (
          <div style={{ marginTop: 10, lineHeight: 1.8 }}>
            <div>{`${d.kind}第${d.r}局 ${d.dun}遁 · ${d.yuan}元第${d.ju}局 · 第${d.ji}纪`}</div>
            <div>{`太乙在${d.gong}宫（入宫第${d.ruGong}）· 文昌${d.tianMu}（${d.tianMuPos}）· 計神${d.jiShen} · 始擊${d.shiJi}（${d.shiJiPos}）`}</div>
            <div>{`主算${d.zhuSuan}（大將${d.zhuDa ?? '杜'}、參將${d.zhuCan ?? '杜'}）· 客算${d.keSuan}（大將${d.keDa ?? '杜'}、參將${d.keCan ?? '杜'}）`}</div>
          </div>
        )}
        <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
          据《太乙金鏡式經》卷一、《古今圖書集成·藝術典》卷六八八、《太乙秘書》、淘金歌；时计冬至后阳遁、夏至后阴遁。外圈十六神位，内圈洛书九宫（不入中五）。
        </Text>
      </div>
    </Flex>
  )
}

export function PaiPanPage(props: LocationProps & {
  wasm: LunarBackend | null
  /** 所选日农历（紫微用）；无则 null */
  lunar: { yearGz: number; month: number; leap: boolean; day: number } | null
  /** 实时：盘随当前时刻（到下一时辰自动换局） */
  isLive: boolean
  onSetLive: (on: boolean) => void
  ear: string | null
  jdUt: number
  dayGz: number | null
  hourZhi: number | null
  /** 时柱干支序 */
  hourGz: number | null
  dateLabel: string
  onShiftHour: (n: number) => void
  onShiftDay: (n: number) => void
}) {
  const { wasm, ear, dayGz, hourZhi } = props
  const { rotate, bar: compassBar } = useCompassBar(props)
  // 按分钟取时刻：实时模式下时钟每秒走，盘只在分钟变时重算（局在时辰、节气交界才变）
  const jdUt = Math.floor(props.jdUt * 1440) / 1440
  const ready = wasm && ear != null && dayGz != null && hourZhi != null
  // 摘要（奇门定局、三元九运）只在本页按需计算
  const summary = useMemo(() => {
    if (!wasm || ear == null || dayGz == null || hourZhi == null || typeof wasm.paipan_wasm !== 'function') return ''
    try { return wasm.paipan_wasm(jdUt, ear, dayGz, hourZhi) } catch { return '' }
  }, [wasm, ear, jdUt, dayGz, hourZhi])
  const qimenLine = summary.split('\n').find((l) => l.startsWith('奇門')) ?? ''
  const sanyuan = summary.split('\n').find((l) => l.startsWith('三元')) ?? ''
  return (
    <div>
      <Flex align="center" gap={8} wrap style={{ marginBottom: 12 }}>
        <Button size="small" icon={<LeftOutlined />} onClick={() => props.onShiftDay(-1)}>前一日</Button>
        <Button size="small" onClick={() => props.onShiftHour(-2)}>前一时辰</Button>
        <Text strong>{props.dateLabel}</Text>
        <Button size="small" onClick={() => props.onShiftHour(2)}>后一时辰</Button>
        <Button size="small" onClick={() => props.onShiftDay(1)}>后一日 <RightOutlined /></Button>
        <span style={{ marginLeft: 8 }}>实时</span>
        <Switch size="small" checked={props.isLive} onChange={props.onSetLive} />
        <Text type="secondary" style={{ fontSize: 12 }}>与日历所选时刻同步；日柱、时辰从日历四柱；实时则到时辰交界自动换局</Text>
      </Flex>
      {compassBar}
      {!ready ? (
        <Text type="secondary">加载历表后显示（需 VSOP87 历表与四柱）</Text>
      ) : (
        <Tabs
          items={[
            {
              key: 'qimen', label: '奇门遁甲',
              children: (
                props.hourGz != null
                  ? <QimenPanel wasm={wasm!} ear={ear!} jdUt={jdUt} hourGz={props.hourGz} rotate={rotate} />
                  : <Text type="secondary">{qimenLine}</Text>
              ),
            },
            { key: 'liuren', label: '大六壬', children: <LiurenPanel wasm={wasm!} ear={ear!} jdUt={jdUt} dayGz={dayGz!} hourZhi={hourZhi!} rotate={rotate} /> },
            { key: 'taiyi', label: '太乙', children: <TaiyiPanel wasm={wasm!} ear={ear!} jdUt={jdUt} dayGz={dayGz!} hourZhi={hourZhi!} rotate={rotate} /> },
            {
              key: 'ziwei', label: '紫微斗数',
              children: props.lunar
                ? <ZiweiPanel wasm={wasm!} lunar={props.lunar} hourZhi={hourZhi!} />
                : <Text type="secondary">需所选日的农历（岁数据加载后显示）</Text>,
            },
          ]}
        />
      )}
      {sanyuan && <Text type="secondary" style={{ display: 'block', marginTop: 8 }}>{sanyuan}（三元一百八十年九运，以康熙二十三年甲子为上元，协纪卷八）</Text>}
    </div>
  )
}
