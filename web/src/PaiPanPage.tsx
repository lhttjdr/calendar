/**
 * 排盘页：奇门遁甲、大六壬、太乙、六爻·周易、玄空飞星、七政四余（经星图 Worker）、紫微斗数。
 * 与日历共用所选时刻；日柱、时支取日历所示四柱。图依古式南上北下、东左西右。
 * 规则与出处见知识库 doc/folk/liuren.md、taiyi.md、qimen.md。
 */
import { Button, Flex, InputNumber, Radio, Switch, Tabs, Typography } from 'antd'
import { LeftOutlined, RightOutlined } from '@ant-design/icons'
import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import type { LunarBackend } from './lunar-backend-types'
import { CompassSpin, useCompassBar, type CompassSpinSource, type LocationProps } from './CompassBar'
import { LiurenWheel, QimenWheel, type QimenGongMarks, type QimenFangGe, QizhengWheel, TaiyiWheel, XuankongGrid, XuankongWheel, YAO_SHORT, type XuankongWheelData, type QizhengWheelData } from './Wheels'
import { useWorkerRpc } from './useWorkerRpc'
import type { SkyRequest } from './skyWorker'
import { DayanDemo } from './DayanDemo'

const { Text } = Typography

const ZHI = '子丑寅卯辰巳午未申酉戌亥'

/** 罗盘模式：方位盘（九宫、天地盘、十六神盘、宅盘、七政十二宫）整盘旋转至实地方位；紫微命盘不对应实地方位，不转 */

type LiurenData = { jiang: string; shi: string; pan: string[]; siKe: { xia: string; shang: string }[]; sanChuan: string[]; fa: string; tianJiang?: string[]; gui?: string; shun?: boolean; zhou?: boolean
  dunGan?: (string | null)[]; sanChuanDun?: (string | null)[]; xunShou?: string; xunKong?: string[]
  yiMa?: string; dingMa?: string | null; tianMa?: string; riDe?: string; riLu?: string; yangRen?: string; riGui?: string[]; yueJian?: string
  zongMen?: string; keMing?: string | null; keGe?: { n: string; note: string }[]
  benMing?: string | null; xingNian?: string | null; xingNianZhi?: string | null; xuSui?: number | null
  /** 日干之墓、日支之墓（卷一「墓」，五行墓／十干墓两说）；太岁；贵人所临地盘；是否因出狱移贵 */
  ganMu?: string; zhiMu?: string; taiSui?: string; guiLin?: string; chuYu?: boolean
  /** 今日为四立、分至；是否土王用事；月宿（起月宿法）与其宫；配偶行年 */
  siLi?: boolean; fenZhi?: boolean; tuWang?: boolean; yueSu?: string | null; yueSuZhi?: string | null; peiXingNian?: string | null
}

/** 恒星过宫之月将（卷十一宿度宫界）：要星表与距星，经星图 Worker 求，只在选此说时挂载 */
function LiurenXiuJiang(props: { jdUt: number; onJiang: (j: number | null, err?: string) => void }) {
  const call = useWorkerRpc<SkyRequest>(() => new Worker(new URL('./skyWorker.ts', import.meta.url), { type: 'module' }))
  const { onJiang } = props
  useEffect(() => {
    let live = true
    call<number>({ type: 'liurenJiangXiu', jdUt: props.jdUt })
      .then((j) => { if (live) onJiang(j >= 0 ? j : null) })
      .catch((e: Error) => { if (live) onJiang(null, e.message) })
    return () => { live = false }
  }, [call, props.jdUt, onJiang])
  return null
}
/** 六壬盘上神煞简称：旬空、驿马、丁马、天马、日德、日禄、羊刃、日鬼（《六壬大全》卷一神煞） */
function liurenExtra(d: LiurenData) {
  const t: Record<string, string[]> = {}
  const add = (z: string | null | undefined, m: string) => { if (z) (t[z] ??= []).push(m) }
  d.xunKong?.forEach((z) => add(z, '空'))
  add(d.yiMa, '馬'); add(d.dingMa, '丁'); add(d.tianMa, '天馬'); add(d.riDe, '德'); add(d.riLu, '祿'); add(d.yangRen, '刃')
  d.riGui?.forEach((z) => add(z, '鬼'))
  add(d.ganMu, '墓'); if (d.zhiMu && d.zhiMu !== d.ganMu) add(d.zhiMu, '辰墓')
  const di: Record<string, string[]> = {}
  if (d.benMing) (di[d.benMing] ??= []).push('命')
  if (d.xingNianZhi) (di[d.xingNianZhi] ??= []).push('年')
  return { dunGan: d.dunGan, tianMarks: t, diMarks: di, center2: [d.zongMen, d.keMing].filter(Boolean).join('·') }
}
type TaiyiData = {
  kind: string; r: number; dun: string; yuan: string; ju: number; ji: number; gong: number; ruGong: number
  tianMu: string; tianMuPos: string; jiShen: string; shiJi: string; shiJiPos: string
  zhuSuan: number; keSuan: number; zhuDa: number | null; zhuCan: number | null; keDa: number | null; keCan: number | null
  suiNian?: string; dingJi?: string; dingJiPos?: string | null; dingJiSuan?: number; dingJiDa?: number | null; dingJiCan?: number | null
  siShen?: TyG; tianYi?: TyG; diYi?: TyG; zhiFu?: TyG; wuFu?: TyG; daYou?: TyG; xiaoYou?: number | null
  junJi?: { zhi: string; ruNian?: number } | null; chenJi?: { zhi: string; ruNian?: number } | null; minJi?: { zhi: string; ruNian?: number } | null
  daYouTianMu?: string | null; daYouTianMuPos?: string | null
  tianMuZhuang?: string; shiJiZhuang?: string; zhuDaZhuang?: string; zhuCanZhuang?: string; keDaZhuang?: string; keCanZhuang?: string
  geJu?: string[]; zhuJiangFa?: boolean; keJiangFa?: boolean
  /** 八门、门具、算和、三才、十精、阳九百六（doc/folk/taiyi.md 第十一至十四节） */
  zhiShiMen?: string; zhiShiRu?: number; men?: (string | null)[]; menJu?: string; zhuMenJu?: boolean; keMenJu?: boolean
  zhuSuanHe?: TySuan; keSuanHe?: TySuan
  shiJing?: { n: string; pos?: string; gong?: number; shu?: number; label: string }[]; shiJingHe?: string[]
  yangJiu?: TyELi; baiLiu?: TyELi
  /** 金鏡章岁法所求干支年之闰月（11、12 为前一年冬月腊月），无闰 null */
  zhangSuiRun?: number | null
}
type TyG = { gong: number; name?: string; zhi?: string; ruNian?: number } | null
type TySuan = { he: boolean; ming: string[]; sanCai: string }
type TyELi = { di: number; ruNian: number; bang: string }

type QimenData = {
  geju?: { n: string; gong: number | null; ji: boolean; basis: string; note: string | null }[]
  ju: string; xunShou: string; xunYi: string; zhiFu: string; zhiShi: string; zhiFuGong: number; zhiShiGong: number
  /** 家名与局（时家、日家、月家、年家）；顺飞诸法之文字（遁年奇、遁月奇、统宗日家、演义休门） */
  jia?: string; shunFei?: string[]
  gong: ({ di: string[]; tian: string[]; xing: string[]; men: string | null; shen: string | null } & QimenGongMarks)[]
  xunKong?: string[]; xu?: string[]; kongGong?: number[]; riXunKong?: string[]; ma?: string; maGong?: number | null; jieLuKong?: boolean
  /** 方位格（演义卷二）：月将、天門地戸私門天馬亭亭貴人登天門（系于地盘支）、三勝五不擊、寄宫避五（卷三） */
  yueJiang?: string
  fangGe?: QimenFangGe[]
  sanSheng?: { n: string; who: string | null; gong: number | null; basis: string }[]
  biWu?: { n: string; gong: number | null; basis: string; note: string | null }[]
}


const QimenPanel = memo(function QimenPanel(props: { wasm: LunarBackend; ear: string; jdUt: number; hourGz: number; dayGz: number; spin: CompassSpinSource }) {
  const [method, setMethod] = useState(0)
  const [jiGong, setJiGong] = useState(0)
  const [shenMing, setShenMing] = useState(0)
  const [dingJu, setDingJu] = useState(0)
  const [source, setSource] = useState(0)
  /** 逐宫标记诸说（doc/folk/qimen.md「逐宫标记」）：暗干、干墓、击刑、旬空落宫 */
  const [anGan, setAnGan] = useState(0)
  const [ganMu, setGanMu] = useState(0)
  const [jiXing, setJiXing] = useState(0)
  const [kong, setKong] = useState(0)
  /** 家：0 时家、1 日家、2 月家、3 年家；月家元局 0 演义、1 统宗 */
  const [jia, setJia] = useState(0)
  const [jiaRule, setJiaRule] = useState(0)
  const d = useMemo((): QimenData | null => {
    if (typeof props.wasm.qimen_json !== 'function') return null
    try { return JSON.parse(props.wasm.qimen_json(props.jdUt, props.ear, props.hourGz, method, jiGong, shenMing, dingJu, source, props.dayGz, anGan, ganMu, jiXing, kong, jia, jiaRule)) as QimenData } catch { return null }
  }, [props.wasm, props.ear, props.jdUt, props.hourGz, props.dayGz, method, jiGong, shenMing, dingJu, source, anGan, ganMu, jiXing, kong, jia, jiaRule])
  /** 时辰内插值（示意）：下一时辰之盘与本时辰已过比例（按所选时刻之民用钟点，时辰起于奇数时） */
  const [interp, setInterp] = useState(false)
  const next = useMemo(() => {
    if (!interp || method !== 0 || jia !== 0 || typeof props.wasm.qimen_json !== 'function') return undefined
    const t = new Date((props.jdUt - 2440587.5) * 86400000)
    const h = t.getHours() + t.getMinutes() / 60 + t.getSeconds() / 3600
    const into = (((h + 1) % 2) + 2) % 2 // 本时辰已过之时数（0—2）
    const jdNext = props.jdUt + (2 - into) / 24 + 1 / 1440
    const hourNext = (props.hourGz + 1) % 60
    const dayNext = hourNext % 12 === 0 ? (props.dayGz + 1) % 60 : props.dayGz // 入子时换日（与四柱缺省日界同）
    try {
      const nd = JSON.parse(props.wasm.qimen_json(jdNext, props.ear, hourNext, method, jiGong, shenMing, dingJu, source, dayNext, anGan, ganMu, jiXing, kong, 0, jiaRule)) as QimenData
      return { d: nd, f: into / 2 }
    } catch { return undefined }
  }, [interp, props.wasm, props.ear, props.jdUt, props.hourGz, props.dayGz, method, jiGong, shenMing, dingJu, source, anGan, ganMu, jiXing, kong, jia, jiaRule])
  return (
    <Flex gap={16} wrap align="flex-start">
      <CompassSpin spin={props.spin} style={{ flex: '1 1 320px', maxWidth: 440 }}>{d ? <QimenWheel d={d} turn={method === 0} next={next} /> : <Text type="secondary">无法布盘</Text>}</CompassSpin>
      <div style={{ flex: '1 1 280px' }}>
        {d && (
          <div style={{ marginBottom: 10, lineHeight: 1.8 }}>
            <div style={{ fontSize: 16 }}>{d.jia ?? d.ju}</div>
            <div>{`旬首${d.xunShou}（遁${d.xunYi}）· 直符${d.zhiFu}加${d.zhiFuGong}宫 · 直使${d.zhiShi}加${d.zhiShiGong}宫`}</div>
            <div style={{ fontSize: 13 }}>{`时旬空${(d.xunKong ?? []).join('')}（落${(d.kongGong ?? []).join('、')}宫）· 驿马${d.ma ?? ''}（${d.maGong ?? ''}宫）${d.jieLuKong ? ' · 截路空亡' : ''}`}</div>
            {d.gong.some((c) => (c.keYing?.length ?? 0) > 0) && (
              <div style={{ fontSize: 12, marginTop: 4 }}>
                <Text strong style={{ fontSize: 12 }}>十干克应（寶鑑）：</Text>
                {d.gong.flatMap((c, i) => (c.keYing ?? []).map((k) => (
                  <span key={`${i}${k.t}${k.d}`} title={k.text} style={{ marginRight: 8 }}>{`${i + 1}宫${k.t}加${k.d}「${k.n}」`}</span>
                )))}
              </div>
            )}
            {d.geju && d.geju.length > 0 && (
              <div style={{ fontSize: 12, marginTop: 4 }}>
                <Text strong style={{ fontSize: 12 }}>格局（演义卷二）：</Text>
                {d.geju.map((g, i) => (
                  <span key={i} title={`${g.basis}${g.note ? '；' + g.note : ''}`} style={{ color: g.ji ? '#237804' : '#a8071a', marginRight: 8 }}>
                    {`${g.n}${g.gong ? `（${g.gong}宫）` : ''}`}
                  </span>
                ))}
              </div>
            )}
            {d.biWu && d.biWu.length > 0 && (
              <div style={{ fontSize: 12, marginTop: 4 }}>
                <Text strong style={{ fontSize: 12 }}>寄宫（演义卷三）：</Text>
                {d.biWu.map((g, i) => (
                  <span key={i} title={`${g.basis}${g.note ? '；' + g.note : ''}`} style={{ color: '#a8071a', marginRight: 8 }}>
                    {`${g.n}·${(g.note ?? '').slice(0, 4)}（寄${g.gong ?? ''}宫）`}
                  </span>
                ))}
              </div>
            )}
            {d.fangGe && d.fangGe.length > 0 && (
              <div style={{ fontSize: 12, marginTop: 4 }}>
                <Text strong style={{ fontSize: 12 }}>{`方位格（演义卷二，月将${d.yueJiang ?? ''}加时）：`}</Text>
                {Array.from(new Set(d.fangGe.map((g) => g.n))).map((n) => {
                  const gs = d.fangGe!.filter((g) => g.n === n)
                  return (
                    <span key={n} title={`${gs[0]!.basis}${gs[0]!.note ? '；' + gs[0]!.note : ''}`} style={{ color: '#237804', marginRight: 8 }}>
                      {`${n}${gs.map((g) => `${g.zhi}（${g.gong}宫）`).join('')}`}
                    </span>
                  )
                })}
              </div>
            )}
            {d.sanSheng && d.sanSheng.length > 0 && (
              <div style={{ fontSize: 12, marginTop: 4 }}>
                <Text strong style={{ fontSize: 12 }}>三勝五不擊（演义卷二引湯謂）：</Text>
                {['三勝', '五不擊'].map((n) => {
                  const gs = d.sanSheng!.filter((g) => g.n === n)
                  return gs.length > 0 && (
                    <span key={n} title={gs[0]!.basis} style={{ marginRight: 8 }}>
                      {`${n}：${gs.map((g) => `${g.who ?? ''}${g.gong ?? ''}`).join('、')}`}
                    </span>
                  )
                })}
              </div>
            )}
          </div>
        )}
        <Flex vertical gap={6}>
          <Radio.Group size="small" value={jia} onChange={(e) => setJia(e.target.value)} optionType="button"
            options={[{ label: '时家', value: 0 }, { label: '日家（统宗遁日奇）', value: 1 }, { label: '月家', value: 2 }, { label: '年家（演义）', value: 3 }]} />
          {jia === 2 && (
            <Radio.Group size="small" value={jiaRule} onChange={(e) => setJiaRule(e.target.value)} optionType="button"
              options={[{ label: '月家元局·演义（孟上仲中季下）', value: 0 }, { label: '统宗（仲上孟中季下）', value: 1 }]} />
          )}
          {d?.shunFei && d.shunFei.length > 0 && (
            <div style={{ fontSize: 12 }}>
              <Text strong style={{ fontSize: 12 }}>年月日家顺飞诸法：</Text>
              {d.shunFei.map((l, i) => <div key={i}>{l}</div>)}
              <Text type="secondary" style={{ fontSize: 11 }}>统宗自谓「诸说纷纭，全无定见……姑备录之」；阴遁诸式原文无例，依「阳顺阴逆」推，未核</Text>
            </div>
          )}
          <Radio.Group size="small" value={method} onChange={(e) => setMethod(e.target.value)} optionType="button"
            options={[{ label: '转盘', value: 0 }, { label: '飞盘（未核）', value: 1 }]} />
          {method === 0 && (
            <label style={{ fontSize: 12 }}>
              <input type="checkbox" checked={interp} onChange={(e) => setInterp(e.target.checked)} /> 时辰内插值（示意，非原典：九星、八门、八神按时辰已过之比例向下一时辰之位转去；局变时仍跳）
            </label>
          )}
          <Radio.Group size="small" value={jiGong} onChange={(e) => setJiGong(e.target.value)} optionType="button"
            options={[{ label: '中五寄坤', value: 0 }, { label: '阳坤阴艮', value: 1 }, { label: '阳巽阴震', value: 2 }]} />
          <Radio.Group size="small" value={shenMing} onChange={(e) => setShenMing(e.target.value)} optionType="button"
            options={[{ label: '白虎玄武（演义）', value: 0 }, { label: '勾陈朱雀（集成）', value: 1 }]} />
          <Radio.Group size="small" value={dingJu} onChange={(e) => setDingJu(e.target.value)} optionType="button"
            options={[{ label: '置闰（演义）', value: 0 }, { label: '置闰（统宗）', value: 1 }, { label: '拆补', value: 2 }]} />
          <Radio.Group size="small" value={source} onChange={(e) => setSource(e.target.value)} optionType="button"
            options={[{ label: '节气从行用历', value: 0 }, { label: '定气', value: 1 }]} />
          <Radio.Group size="small" value={anGan} onChange={(e) => setAnGan(e.target.value)} optionType="button"
            options={[{ label: '暗干·值使宫起（中五例外）', value: 0 }, { label: '暗干·一律值使宫起', value: 1 }, { label: '不布暗干', value: 2 }]} />
          <Radio.Group size="small" value={ganMu} onChange={(e) => setGanMu(e.target.value)} optionType="button"
            options={[{ label: '干墓阳顺阴逆', value: 0 }, { label: '五行墓·土戌', value: 1 }, { label: '五行墓·土辰', value: 2 }]} />
          <Radio.Group size="small" value={jiXing} onChange={(e) => setJiXing(e.target.value)} optionType="button"
            options={[{ label: '击刑只论直符（演义）', value: 0 }, { label: '天盘诸仪（寶鑑）', value: 1 }, { label: '诸仪兼一本', value: 2 }]} />
          <Radio.Group size="small" value={kong} onChange={(e) => setKong(e.target.value)} optionType="button"
            options={[{ label: '旬空以支论宫', value: 0 }, { label: '以仪论宫', value: 1 }, { label: '两者并取', value: 2 }]} />
        </Flex>
        <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
          八宫按八卦方位，南上北下；每宫由内向外为地盘干、暗干、八门（蓝）、九星、天盘干（红）、八神、标记（空旬空、馬驿马、迫门迫、刑击刑、干墓星墓門墓）；二十四山内一环为方位格，标于其支（演义卷二月将加时：天門、地戸、私門、天馬、亭亭、登天門为贵人登天门）；暗干原典未见，今人之法；直符、直使所加之宫底色标出。定局据《遁甲演義》卷一、卷三；转盘布法据演义、《太白陰經》卷九、《古今圖書集成》卷七百八；寄宫三说以集成十八局表衡量，寄坤最合；时干在中五诸书原例皆直符加寄宫，无直符不动之说。飞盘原典未见，出处未核。
        </Text>
      </div>
    </Flex>
  )
})

type ZiweiData = {
  palaces: string[]; gan: string[]; stars: [string, number, string | null][]; siHua: [string, string][]
  ming: number; shen: number; ju: number; juName: string; mingZhu: string; shenZhu: string
  daXian: [number, number, number][]; changSheng: string[]; boShi: string[]
  /** 十八飞星派：辅星亦标庙乐旺 */
  minorMw?: boolean
  /** 十八飞星派「術天機」图之本命星（生年支所配，原书未言其用） */
  benMing?: string
  /** 流年（ziwei_json opt_extra 位 8 起为流年干支序 + 1） */
  liu?: { gz: number; age: number; xiaoXian: number; daXian: number | null; stars: [string, number][]; ge: string[] } | null
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
        const mw = new Map(d.stars.filter(([, k]) => k === zi).map(([n, , b]) => [n, b ? b[0] : '']))
        const main = stars.filter((n) => MAIN14.has(n)), minor = stars.filter((n) => !MAIN14.has(n))
        const dx = d.daXian.find(([, , k]) => k === zi)
        const x = cx * s, y = cy * s
        const isMing = zi === d.ming, isShen = zi === d.shen
        return (
          <g key={z}>
            <rect x={x + 1} y={y + 1} width={s - 2} height={s - 2} fill={isMing ? '#fff1b8' : '#fffdf7'} stroke="#999" />
            {main.map((n, i) => (
              <text key={n} x={x + 8} y={y + 18 + i * 17} fontSize={15} fontWeight={600} fill="#a8071a">
                {n}{mw.get(n) ? <tspan fontSize={10} fill="#888">{mw.get(n)}</tspan> : null}{hua.get(n) ? <tspan fontSize={11} fill="#0958d9">{` ${hua.get(n)}`}</tspan> : null}
              </text>
            ))}
            {minor.map((n, i) => (
              <text key={n} x={x + s - 6} y={y + 15 + i * 13} fontSize={11} textAnchor="end" fill="#333">
                {n}{d.minorMw && mw.get(n) ? <tspan fontSize={9} fill="#888">{mw.get(n)}</tspan> : null}{hua.get(n) ? <tspan fill="#0958d9">{hua.get(n)}</tspan> : null}
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

const ZiweiPanel = memo(function ZiweiPanel(props: { wasm: LunarBackend; lunar: { yearGz: number; month: number; leap: boolean; day: number }; hourZhi: number }) {
  const [male, setMale] = useState(true)
  const [school, setSchool] = useState(14)
  const [leap, setLeap] = useState(0)
  const [xin, setXin] = useState(0)
  const [bingDing, setBingDing] = useState(0)
  const [siHua, setSiHua] = useState(0)
  const [cs, setCs] = useState(0)
  /** 十八飞星派选项（ziwei18_json 之 opts 位）：1 庙乐旺用卷三星论、2 五行用術天機图 */
  const [o18, setO18] = useState(0)
  /** 十四主星派补充：写本诸星、流年（公历年，按干支年） */
  const [xieBen, setXieBen] = useState(false)
  const [liuYear, setLiuYear] = useState<number | null>(null)
  const { lunar } = props
  const d = useMemo((): ZiweiData | null => {
    if (typeof props.wasm.ziwei_json !== 'function') return null
    try {
      if (school === 18) {
        if (typeof props.wasm.ziwei18_json !== 'function') return null
        // 十八飞星派：映射到同一命盘图（宫干、四化、局等此派无，留空；宫下小字为流年十二神）
        const e = JSON.parse(props.wasm.ziwei18_json(lunar.yearGz, lunar.month, lunar.leap, lunar.day, props.hourZhi, male, o18)) as { palaces: string[]; stars: [string, number, string | null, string | null][]; ming: number; shen: number; daXian: [number, number, number][]; liuNian: string[]; benMing?: string }
        return { palaces: e.palaces, gan: Array(12).fill(''), stars: e.stars.map(([n, z, g]) => [n, z, g]), siHua: [], ming: e.ming, shen: e.shen, ju: 0, juName: '十八飛星', mingZhu: '—', shenZhu: '—', benMing: e.benMing, daXian: e.daXian, changSheng: e.liuNian, boShi: Array(12).fill(''), minorMw: true }
      }
      const extra = (xieBen ? 1 : 0) | (liuYear == null ? 0 : (((liuYear - 4) % 60 + 60) % 60 + 1) << 8)
      return JSON.parse(props.wasm.ziwei_json(lunar.yearGz, lunar.month, lunar.leap, lunar.day, props.hourZhi, male, leap, xin, bingDing, siHua, cs, extra)) as ZiweiData
    } catch { return null }
  }, [props.wasm, lunar.yearGz, lunar.month, lunar.leap, lunar.day, props.hourZhi, male, leap, xin, bingDing, siHua, cs, school, o18, xieBen, liuYear])
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
          <Radio.Group size="small" value={school} onChange={(e) => setSchool(e.target.value)} optionType="button"
            options={[{ label: '十四主星派（全书）', value: 14 }, { label: '十八飞星派（续道藏）', value: 18 }]} />
          <Radio.Group size="small" value={male} onChange={(e) => setMale(e.target.value)} optionType="button" options={[{ label: '男命', value: true }, { label: '女命', value: false }]} />
          {school === 14 && (<>
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
          <label style={{ fontSize: 12 }}><input type="checkbox" checked={xieBen} onChange={(e) => setXieBen(e.target.checked)} /> 加排写本诸星（天官、天福、华盖、孤寡、破碎、大耗、劫杀、桃花、天厨、太极、科名、文星、山河、福星；全书无）</label>
          <div style={{ fontSize: 12 }}>流年 <InputNumber size="small" placeholder="公历年" min={-2000} max={3000} value={liuYear} onChange={(v) => setLiuYear(v == null ? null : Number(v))} style={{ width: 96 }} /></div>
          {d?.liu && (
            <div style={{ fontSize: 12 }}>
              {`${G[d.liu.gz % 10]}${Z[d.liu.gz % 12]}年 虚岁${d.liu.age}：小限${Z[d.liu.xiaoXian]}${d.liu.daXian != null ? `、大限${Z[d.liu.daXian]}` : ''}；`}
              {d.liu.stars.filter(([n]) => ['流祿', '流羊', '流陀', '流昌', '流曲'].includes(n)).map(([n, z]) => `${n}${Z[z]}`).join(' ')}
              {d.liu.ge.length > 0 && <div><b>{d.liu.ge.join('、')}</b>（全书卷三、写本第 11—12 页；判法未核）</div>}
            </div>
          )}
          </>)}
          {school === 18 && (<>
          <Radio.Group size="small" value={o18 & 1} onChange={(e) => setO18((o18 & ~1) | e.target.value)} optionType="button"
            options={[{ label: '庙乐旺：卷一图', value: 0 }, { label: '卷三星论', value: 1 }]} />
          </>)}
        </Flex>
        <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
          {school === 14
            ? '十四主星派，据《紫微斗數全書》卷二（安身命、五局安紫微、诸星诀、四化、庙旺表），韩国国立中央图书馆藏写本《紫微斗數方書》旁证。红字主星，旁注小字为庙旺，蓝字四化；宫内下方为长生、博士十二神与大限岁数。各家异说并列，缺省取全书。'
            : `十八飞星派，据《續道藏》本《紫微斗數》（题陈抟），以国家图书馆藏明万历三十五年内府刻本校。原书「甲丙戊庚壬属阴」与通例相反，照原书；宫下小字为流年十二神与大限岁数；星旁小字为卷一「廟樂旺」图之等（卷三星论为选项，二者有异）。${d?.benMing ? `「術天機」图本命星：${d.benMing}（原书未言其用）。` : ''}`}
        </Text>
      </div>
    </Flex>
  )
})

type LiuyaoData = {
  ben: string; zhi: string; gong: string; gongWx: string; shi: number; ying: number; xunKong: string | null; guaShen: string | null
  yao: { yang: boolean; dong: boolean; gz: string; wx: string; qin: string; shen: string | null; fu: string | null; bian: string | null }[]
  rule: string; readings: { gua: string; ci: string; role: string; text: string }[]
}

/** 六爻纳甲与周易筮法：掷钱、揲蓍、梅花时间起卦，或逐爻手选；只排卦与给出应读之辞，不断吉凶 */
/** 爻之四象：六爻家记作单拆重交（⚊⚋○✕），即《周易》大衍筮法之七八九六（少阳、少阴、老阳、老阴；九六为变） */
const YAO_NAME: Record<number, string> = { 7: '⚊ 单（少阳）', 8: '⚋ 拆（少阴）', 9: '○ 重（老阳）', 6: '✕ 交（老阴）' }

type LiuyaoMarks = {
  jing: { fei: string; fu: string; ming: string; outer: boolean; shi: number; feiGz: string; fuGz: string }
  wang: (string | null)[]; marks: string[][]; gua: string[]
}

const LiuyaoPanel = memo(function LiuyaoPanel(props: { wasm: LunarBackend; lunar: { yearGz: number; month: number; leap: boolean; day: number } | null; hourZhi: number; dayGz: number; monthZhi: number | null }) {
  const timeValues = (): number[] => {
    if (!props.lunar || typeof props.wasm.meihua_time_values !== 'function') return [7, 7, 7, 7, 7, 7]
    return Array.from(props.wasm.meihua_time_values(props.lunar.yearGz % 12, props.lunar.month, props.lunar.day, props.hourZhi))
  }
  const [values, setValues] = useState<number[]>(timeValues)
  const [method, setMethod] = useState('梅花时间起卦')
  const [guaShen, setGuaShen] = useState(0)
  const [sanBian, setSanBian] = useState(0)
  const [fuShen, setFuShen] = useState(0)
  const [demo, setDemo] = useState(false)
  const rnd = () => Math.floor(Math.random() * 1e9)
  // 三钱一掷，舊法以无字之背为阳：一背单（少阳 7）、二背拆（少阴 8）、三背重（老阳 9）、三字交（老阴 6）。
  // 項安世《項氏家說》卷二、《古今圖書集成·藝術典》卷五百四十四「以錢代蓍畫法」；朱子以有字为阳，反之（儲泳《祛疑說》）。见 doc/folk/liuyao.md
  const coin = () => { const bei = [0, 0, 0].reduce((t) => t + (Math.random() < 0.5 ? 1 : 0), 0); return [6, 7, 8, 9][bei]! }
  const d = useMemo((): LiuyaoData | null => {
    if (typeof props.wasm.liuyao_json !== 'function') return null
    try { return JSON.parse(props.wasm.liuyao_json(Uint8Array.from(values), props.monthZhi ?? 0, props.dayGz, guaShen, sanBian)) as LiuyaoData } catch { return null }
  }, [props.wasm, values, props.monthZhi, props.dayGz, guaShen, sanBian])
  // 京房飞伏与《黄金策》诸象（月破、暗动、进退神、回头生克、反吟伏吟等），见 doc/folk/liuyao.md
  const mk = useMemo((): LiuyaoMarks | null => {
    if (typeof props.wasm.liuyao_marks_json !== 'function') return null
    try { return JSON.parse(props.wasm.liuyao_marks_json(Uint8Array.from(values), props.monthZhi ?? 0, props.dayGz)) as LiuyaoMarks } catch { return null }
  }, [props.wasm, values, props.monthZhi, props.dayGz])
  return (
    <Flex gap={16} wrap align="flex-start">
      <div style={{ flex: '1 1 380px', maxWidth: 560 }}>
        <Flex gap={6} wrap style={{ marginBottom: 8 }}>
          <Button size="small" onClick={() => { setValues(timeValues()); setMethod('梅花时间起卦') }}>梅花时间起卦</Button>
          <Button size="small" onClick={() => { setValues(Array.from({ length: 6 }, coin)); setMethod('掷钱') }}>掷钱（三钱六掷）</Button>
          <Button size="small" onClick={() => { setValues(Array.from({ length: 6 }, () => props.wasm.dayan_yao_wasm ? props.wasm.dayan_yao_wasm(rnd(), rnd(), rnd()) : 7)); setMethod('大衍揲蓍') }}>大衍揲蓍</Button>
          <Button size="small" type={demo ? 'primary' : 'default'} onClick={() => setDemo(!demo)}>逐变演示</Button>
        </Flex>
        {demo && <DayanDemo wasm={props.wasm} onUse={(v) => { setValues(v); setMethod('大衍揲蓍（逐变）') }} />}
        <Text type="secondary" style={{ fontSize: 12 }}>{`起卦：${method}；可逐爻改（单、拆、重、交；即大衍之七、八、九、六）`}</Text>
        {d && (
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13, marginTop: 6 }}>
            <tbody>
              {[5, 4, 3, 2, 1, 0].map((k) => {
                const y = d.yao[k]!
                return (
                  <tr key={k} style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ color: '#531dab', width: 40 }}>{y.shen ?? ''}</td>
                    <td style={{ color: '#888', fontSize: 11, width: 80 }}>{fuShen === 1 ? (mk && mk.jing.shi === k + 1 ? `伏${mk.jing.fuGz}` : '') : y.fu ? `伏${y.fu}` : ''}</td>
                    <td style={{ width: 90 }}>{`${y.qin} ${y.gz}${y.wx}`}</td>
                    <td style={{ fontFamily: 'monospace', fontSize: 16, width: 70 }}>{y.yang ? '━━━' : '━ ━'}{y.dong ? (y.yang ? ' ○' : ' ×') : ''}</td>
                    <td style={{ width: 24, color: '#a8071a' }}>{d.shi === k + 1 ? '世' : d.ying === k + 1 ? '應' : ''}</td>
                    <td style={{ color: '#0958d9', fontSize: 12 }}>{y.bian ? `→ ${y.bian}` : ''}{mk && mk.marks[k]!.length > 0 && <div style={{ color: '#ad6800', fontSize: 11 }}>{`${mk.wang[k] ?? ''} ${mk.marks[k]!.join(' ')}`}</div>}</td>
                    <td>
                      <select value={values[k]} onChange={(e) => { const v = [...values]; v[k] = Number(e.target.value); setValues(v); setMethod('手选') }}>
                        {[7, 8, 9, 6].map((n) => <option key={n} value={n}>{YAO_NAME[n]}</option>)}
                      </select>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        )}
      </div>
      <div style={{ flex: '1 1 300px' }}>
        {d && (
          <div style={{ lineHeight: 1.8 }}>
            <div style={{ fontSize: 16 }}>{d.ben === d.zhi ? d.ben : `${d.ben} 之 ${d.zhi}`}</div>
            <div>{`${d.gong}宫（${d.gongWx}）· 旬空${d.xunKong ?? '—'}${d.guaShen ? ` · 卦身${d.guaShen}` : ''}`}</div>
            {mk && <div style={{ fontSize: 13 }}>{`京房飞伏：${mk.jing.fei}飞${mk.jing.fu}伏（书「與${mk.jing.ming}為飛伏」），世${mk.jing.shi}爻 ${mk.jing.feiGz}，伏 ${mk.jing.fuGz}${mk.gua.length ? ` · ${mk.gua.join('、')}` : ''}`}</div>}
            <div style={{ marginTop: 8 }}><Text strong>考变占应读之辞</Text>（{d.rule}）</div>
            {d.readings.map((r, i) => <div key={i} style={{ fontSize: 13 }}>{`【${r.role}】${r.gua}${r.ci}：${r.text}`}</div>)}
          </div>
        )}
        <Flex vertical gap={6} style={{ marginTop: 8 }}>
          <Radio.Group size="small" value={guaShen} onChange={(e) => setGuaShen(e.target.value)} optionType="button" options={[{ label: '卦身（火珠林）', value: 0 }, { label: '不用卦身（增删卜易）', value: 1 }]} />
          <Radio.Group size="small" value={sanBian} onChange={(e) => setSanBian(e.target.value)} optionType="button" options={[{ label: '三爻变：初爻在变主贞（推定）', value: 0 }, { label: '贞悔并列', value: 1 }]} />
          <Radio.Group size="small" value={fuShen} onChange={(e) => setFuShen(e.target.value)} optionType="button" options={[{ label: '伏神：本宫首卦（火珠林、增删卜易）', value: 0 }, { label: '京房飞伏（世爻）', value: 1 }]} />
        </Flex>
        <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
          装卦据京房八宫纳甲、《火珠林》《增删卜易》（六神以日干起、月建日辰从日历四柱）；取辞据朱熹《易学启蒙·考变占》，经文据《周易》。大衍揲蓍各爻概率 6、7、8、9 为 1/16、5/16、7/16、3/16，掷钱为 1/8、3/8、3/8、1/8。爻旁小字为《黄金策》所引诸象（月建定旺相休囚，月破、暗动、日破、冲空、合绊、进退神、回头生克等），只标象。只排卦与应读之辞，不断吉凶。
        </Text>
      </div>
    </Flex>
  )
})

type XkData = {
  yun: number; zuo: string; xiang: string; ti: boolean; kongWang: string | null
  yunPan: number[]; shanPan: number[]; xiangPan: number[]; geju: { n: string; note: string }[]; chengMen: [number, boolean, boolean][]; yearPan: number[]
  pian?: number | null; zhengShen?: number[]; lingShen?: number[]; gongMarks?: { n: string; note: string }[][]
  jieXian?: XuankongWheelData['jieXian']; daJie?: { gong: number[]; li: boolean; zhen: boolean } | null
  ruQiu?: { shan: number | null; xiang: number | null }; monthPans?: number[][]
  sanZhen?: { zhong: XkZhen; feng: XkZhen } | null
  daGua?: { zuo: XkGua[]; xiang: XkGua[] }
  riShi?: { jdn: number; dayGz: number; yang: boolean; qiJiaZi: number; riXing: number; riPan: number[]; shiZhi: number; shiXing: number; shiPan: number[]; shiXings: number[] } | null
}
type XkZhen = { zuo: string; xiang: string; pian: number }
type XkGua = { gua: string; from: number; to: number; yun: number; shu: number }
const XK_GZ = (g: number) => '甲乙丙丁戊己庚辛壬癸'[g % 10]! + '子丑寅卯辰巳午未申酉戌亥'[g % 12]!
/** 儒略日数 → 公历月日（1582 后） */
const XK_MD = (jdn: number) => { const t = new Date((jdn - 2440588) * 86400000); return `${t.getUTCFullYear()}-${t.getUTCMonth() + 1}-${t.getUTCDate()}` }
const XK_SHAN = ['子', '癸', '丑', '艮', '寅', '甲', '卯', '乙', '辰', '巽', '巳', '丙', '午', '丁', '未', '坤', '申', '庚', '酉', '辛', '戌', '乾', '亥', '壬']
const CN_NUM = '〇一二三四五六七八九'

/** 玄空飞星：运盘、山盘、向盘（洛书九宫，南上）；坐向可选，或以罗盘测（手机顶端朝向首，坐即其背） */
const XuankongPanel = memo(function XuankongPanel(props: { wasm: LunarBackend; ganzhiYear: number; heading: number | null; spin: CompassSpinSource; ear?: string; jdUt?: number }) {
  /** 盘式：圆盘（罗盘式，缺省）或九宫格（书式） */
  const [layout, setLayout] = useState<'round' | 'grid'>('round')
  const [zuo, setZuo] = useState(0)
  const [ti, setTi] = useState(false)
  const [useCompass, setUseCompass] = useState(false)
  /** 锁定的向首方位（量一次后定下，人走动不再变） */
  const [locked, setLocked] = useState<number | null>(null)
  const xiangAz = locked ?? (useCompass ? props.heading : null)
  const az = xiangAz != null ? (xiangAz + 180) % 360 : -1
  const d = useMemo((): XkData | null => {
    if (typeof props.wasm.xuankong_json !== 'function') return null
    try { return JSON.parse(props.wasm.xuankong_json(props.ganzhiYear, zuo, ti, az, props.jdUt ?? null, props.ear ?? null)) as XkData } catch { return null }
  }, [props.wasm, props.ganzhiYear, zuo, ti, az, props.jdUt, props.ear])
  return (
    <Flex gap={16} wrap align="flex-start">
      <div style={{ flex: '1 1 320px', maxWidth: 440 }}>
        <Radio.Group size="small" value={layout} onChange={(e) => setLayout(e.target.value)} optionType="button" style={{ marginBottom: 6 }}
          options={[{ label: '圆盘（罗盘式）', value: 'round' }, { label: '九宫格（书式）', value: 'grid' }]} />
        {d && <CompassSpin spin={props.spin}>{layout === 'grid' ? <XuankongGrid d={d} /> : <XuankongWheel d={d} />}</CompassSpin>}
        <Text type="secondary" style={{ fontSize: 11 }}>罗盘式，自内而外：中宫；八宫飞星（蓝山星居左、红向星居右，其外运星、流年紫白；绿底正神、红底零神）；逐宫标记；后天八卦与洛书数；三元龙（天人地）；二十四山（红阳黑阴，蓝底坐、红底向）；坐向两山旁色带为兼向分段（绿下卦、橙替卦、红空亡）；人盘中针（各山逆时针偏半山，格龙）；天盘缝针（顺时针偏半山，消砂纳水；协纪卷二、卷三十三）；周天度数。虚线为坐向（罗盘测向时为实测方位）。南上北下，开罗盘则随设备转至实地方位</Text>
      </div>
      <div style={{ flex: '1 1 280px' }}>
        <Flex vertical gap={6}>
          <span>坐山{' '}
            <select value={zuo} disabled={useCompass} onChange={(e) => setZuo(Number(e.target.value))}>
              {XK_SHAN.map((n, i) => <option key={n} value={i}>{`${n}山${XK_SHAN[(i + 12) % 24]}向`}</option>)}
            </select>{' '}
            <label><input type="checkbox" checked={ti} disabled={useCompass} onChange={(e) => setTi(e.target.checked)} /> 替卦（兼向）</label>
          </span>
          <label><input type="checkbox" checked={useCompass} onChange={(e) => { setUseCompass(e.target.checked); if (!e.target.checked) setLocked(null) }} /> 用罗盘测向（先开上方罗盘；站于门内、手机顶端朝门外向首，平放）</label>
          {useCompass && (
            <span>
              <Button size="small" disabled={props.heading == null && locked == null} onClick={() => setLocked(locked == null ? props.heading : null)}>
                {locked == null ? '锁定坐向' : '解除锁定'}
              </Button>
              {locked != null && <Text style={{ fontSize: 12 }}>{` 已锁定：向首 ${locked | 0}°`}</Text>}
            </span>
          )}
        </Flex>
        {d && (
          <div style={{ marginTop: 8, lineHeight: 1.8 }}>
            <div style={{ fontSize: 16 }}>{`${CN_NUM[d.yun]}运 ${d.zuo}山${d.xiang}向${d.ti ? '（替卦）' : '（下卦）'}${d.kongWang ? ` · ${d.kongWang}` : ''}`}</div>
            {xiangAz != null && <div style={{ fontSize: 12 }}>{`${locked != null ? '锁定' : '实测'}：向首真北 ${xiangAz | 0}°，坐 ${(xiangAz + 180) % 360 | 0}°`}</div>}
            <div style={{ fontSize: 13 }}>{d.geju.map((g) => g.n).join('、') || '—'}</div>
            {d.zhengShen && <div style={{ fontSize: 13 }}>{`正神${d.zhengShen.join('、')}宫（宜山），零神${(d.lingShen ?? []).join('、')}宫（宜水）`}</div>}
            {d.daJie && <div style={{ fontSize: 13 }}>{`七星打劫：${d.daJie.li ? '离宫' : '坎宫'}打劫（${d.daJie.gong.join('、')}宫）${d.daJie.zhen ? '真' : '假'}`}</div>}
            {d.ruQiu && (d.ruQiu.shan != null || d.ruQiu.xiang != null) && <div style={{ fontSize: 13 }}>{`入囚：山星${d.ruQiu.shan != null ? `${CN_NUM[d.ruQiu.shan]}运` : '—'}，向星${d.ruQiu.xiang != null ? `${CN_NUM[d.ruQiu.xiang]}运` : '—'}`}</div>}
            {d.jieXian && <div style={{ fontSize: 12 }}>{`兼向：${d.jieXian.zuo.duan.map((x) => `${(((x.from % 360) + 360) % 360).toFixed(1)}°起${x.kind}${x.jian ? `兼${x.jian}` : ''}`).join('；')}`}</div>}
            {d.gongMarks && (
              <div style={{ fontSize: 12, marginTop: 4 }}>
                {d.gongMarks.map((ms, i) => ms.length > 0 && (
                  <div key={i}><b>{`${['坎', '坤', '震', '巽', '中', '乾', '兌', '艮', '離'][i]}${CN_NUM[i + 1]}`}</b>{' '}{ms.map((m) => <span key={m.n} title={m.note} style={{ marginRight: 6 }}>{m.n}</span>)}</div>
                ))}
              </div>
            )}
            {d.riShi && (
              <div style={{ fontSize: 12 }}>
                {`日紫白：${XK_GZ(d.riShi.dayGz)}日${CN_NUM[d.riShi.riXing]}入中（${d.riShi.yang ? '冬至' : '夏至'}后甲子 ${XK_MD(d.riShi.qiJiaZi)} 起${d.riShi.yang ? '，阳顺' : '，阴逆'}）；时紫白：${'子丑寅卯辰巳午未申酉戌亥'[d.riShi.shiZhi]}时${CN_NUM[d.riShi.shiXing]}入中`}
                <div>{`日盘：${[1, 8, 3, 4, 9, 2, 7, 6].map((g) => `${['', '坎', '坤', '震', '巽', '中', '乾', '兌', '艮', '離'][g]}${CN_NUM[d.riShi!.riPan[g - 1]!]}`).join(' ')}`}</div>
                <div>{`各时入中：${d.riShi.shiXings.map((x, h) => `${'子丑寅卯辰巳午未申酉戌亥'[h]}${CN_NUM[x]}`).join(' ')}`}</div>
              </div>
            )}
            {d.sanZhen && <div style={{ fontSize: 12 }}>{`三针：正针${d.zuo}山${d.xiang}向；中针（人盘，格龙）${d.sanZhen.zhong.zuo}山${d.sanZhen.zhong.xiang}向；缝针（天盘，消砂纳水）${d.sanZhen.feng.zuo}山${d.sanZhen.feng.xiang}向`}</div>}
            {d.daGua && <div style={{ fontSize: 12 }}>{`大卦（今人六十四卦，未核）：坐 ${d.daGua.zuo.map((g) => `${g.gua}（运${CN_NUM[g.yun]}数${CN_NUM[g.shu]}）`).join(' ')}；向 ${d.daGua.xiang.map((g) => `${g.gua}（运${CN_NUM[g.yun]}数${CN_NUM[g.shu]}）`).join(' ')}`}</div>}
            {d.monthPans && <div style={{ fontSize: 12 }}>{`流月入中：${d.monthPans.map((p, i) => `${'寅卯辰巳午未申酉戌亥子丑'[i]}${CN_NUM[p[4]!]}`).join(' ')}`}</div>}
            <div style={{ fontSize: 13 }}>{`城门：${d.chengMen.map(([g, z, k]) => `${['', '坎', '坤', '震', '巽', '中', '乾', '兌', '艮', '離'][g]}${z ? '（正）' : '（副）'}${k ? '可用' : '不可用'}`).join('；')}`}</div>
          </div>
        )}
        <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
          三元九运以康熙二十三年甲子为上元（协纪卷八），每运二十年。山向飞星依二十四山阴阳顺逆，替卦依《青囊奥语》挨星诀；排法成文规则出自沈氏，原刊未见，出处未核。兼向替卦限 3°、空亡限 1.5°（未核）。日紫白据《奇門遁甲統宗》「日冬至後用陽局一七四，夏至後用陰局九三六」，冬至（夏至）当日或其后第一个甲子起；时紫白依协纪「時家亦有三竒紫白倣月例推」连续推得（未核）。
        </Text>
      </div>
    </Flex>
  )
})

/** 十二支方框（地盘）：巳午未申在上，亥子丑寅在下（南上北下） */
const LIUREN_CELLS: [string, number, number][] = [
  ['巳', 0, 0], ['午', 1, 0], ['未', 2, 0], ['申', 3, 0],
  ['辰', 0, 1], ['酉', 3, 1],
  ['卯', 0, 2], ['戌', 3, 2],
  ['寅', 0, 3], ['丑', 1, 3], ['子', 2, 3], ['亥', 3, 3],
]


const LiurenPanel = memo(function LiurenPanel(props: { wasm: LunarBackend; ear: string; jdUt: number; dayGz: number; hourZhi: number; spin: CompassSpinSource; latitude: number; longitude: number; live?: boolean; lunar: { month: number; day: number } | null }) {
  /** 天盘连续转（月将加时之连续形，示意）：缺省按时辰跳转 */
  const [contTurn, setContTurn] = useState(false)
  const [rule, setRule] = useState(0)
  /** 贵人：2 书中实用（缺省：卷五卷六课例 49 则与卷六、卷九、卷十明言昼夜诸例全合）、0 俗例（曹震圭）、1 考原 */
  const [guiRule, setGuiRule] = useState(2)
  /** 昼夜：−2 卯至酉（缺省，配书中实用表课例全合）、−1 卯至申、−3／−4 当地日出入（日心地平／−50′）、1 昼、0 夜 */
  const [zhou, setZhou] = useState(-2)
  /** 贵人临辰戌：0 不出狱（课例）、1 太乙起子逆後尋、2 以冲為神 */
  const [chuYu, setChuYu] = useState(0)
  /** 日墓：0 五行墓（书中诸例）、1 十干墓（卷一墓条文） */
  const [muRule, setMuRule] = useState(0)
  /** 遥克俱比俱不比：0 论渉害（卷五訂訛）、1 先舉（觀月經）；渉害：0 訂訛、1 尉山人、2 比用格 */
  const [yaoRule, setYaoRule] = useState(0)
  const [sheHai, setSheHai] = useState(0)
  /** 遁干：0 旬遁（缺省，卷九財乗丁馬格）、1 日干五子遁 */
  const [dunRule, setDunRule] = useState(0)
  /** 本命出生年（公历；空则不排行年）与性别 */
  const [birthYear, setBirthYear] = useState<number | null>(null)
  const [male, setMale] = useState(1)
  const benMingGz = birthYear == null ? -1 : (((birthYear - 4) % 60) + 60) % 60
  /** 配偶出生年（繁昌课用，性别与本人相反） */
  const [peiYear, setPeiYear] = useState<number | null>(null)
  const peiGz = peiYear == null ? -1 : (((peiYear - 4) % 60) + 60) % 60
  /** 旺相：0 季月土旺（缺省）、1 土旺四季各十八日（卷八） */
  const [wangRule, setWangRule] = useState(0)
  /** 恒星过宫之月将（rule 2 时由 Worker 求） */
  const [xiuJiang, setXiuJiang] = useState<{ j: number | null; err?: string }>({ j: null })
  const onXiuJiang = useCallback((j: number | null, err?: string) => setXiuJiang({ j, err }), [])
  const jiangOverride = rule === 2 ? (xiuJiang.j ?? -1) : -1
  const lm = props.lunar?.month ?? 0, ld = props.lunar?.day ?? 0
  const d = useMemo((): LiurenData | null => {
    if (typeof props.wasm.liuren_json !== 'function') return null
    try { return JSON.parse(props.wasm.liuren_json(props.jdUt, props.ear, props.dayGz, props.hourZhi, rule === 1 ? 1 : 0, guiRule, zhou, dunRule, benMingGz, male, chuYu, muRule, yaoRule, sheHai, props.latitude, props.longitude, wangRule, lm, ld, jiangOverride, peiGz)) as LiurenData } catch { return null }
  }, [props.wasm, props.ear, props.jdUt, props.dayGz, props.hourZhi, rule, guiRule, zhou, dunRule, benMingGz, male, chuYu, muRule, yaoRule, sheHai, props.latitude, props.longitude, wangRule, lm, ld, jiangOverride, peiGz])
  if (!d) return <Text type="secondary">无法起课</Text>
  return (
    <Flex gap={16} wrap align="flex-start">
      <CompassSpin spin={props.spin} style={{ flex: '1 1 300px', maxWidth: 380 }}><LiurenWheel pan={d.pan} shi={d.shi} sanChuan={d.sanChuan} tianJiang={d.tianJiang} label={`${d.jiang}將加${d.shi}時`} extra={liurenExtra(d)} cont={contTurn ? { jdUt: props.jdUt, lonDeg: props.longitude, live: !!props.live } : undefined} /></CompassSpin>
      <div style={{ flex: '1 1 260px' }}>
        <label style={{ fontSize: 12, display: 'block', marginBottom: 8 }}>
          <input type="checkbox" checked={contTurn} onChange={(e) => setContTurn(e.target.checked)} /> 天盘连续转（月将加时之连续形：月将中太阳之位对准真太阳时角；时辰正中与按时辰之盘重合；实时则随时刻缓转）
        </label>
        <div style={{ marginBottom: 8 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>换将</Text>{' '}
          <Radio.Group size="small" value={rule} onChange={(e) => setRule(e.target.value)} optionType="button"
            options={[{ label: '中气（六壬大全卷二）', value: 0 }, { label: '恒星过宫（卷十一宿度）', value: 2 }, { label: '节（未核）', value: 1 }]} />
          {rule === 2 && <LiurenXiuJiang jdUt={props.jdUt} onJiang={onXiuJiang} />}
          {rule === 2 && <div style={{ fontSize: 11, color: '#888' }}>{xiuJiang.err ? `恒星过宫求将失败（${xiuJiang.err}），暂按中气` : xiuJiang.j == null ? '加载星表求恒星过宫之将……（暂按中气）' : '太阳按卷十一「太陽過宫」宿度宫界（授时距星、古法黄道宿度）所在之宫为将'}</div>}
        </div>
        <div style={{ marginBottom: 8 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>贵人</Text>{' '}
          <Radio.Group size="small" value={guiRule} onChange={(e) => setGuiRule(e.target.value)} optionType="button"
            options={[{ label: '书中实用', value: 2 }, { label: '俗例（曹震圭）', value: 0 }, { label: '考原定法', value: 1 }]} />
          <div style={{ marginTop: 4 }}>
            <Text type="secondary" style={{ fontSize: 12 }}>昼夜</Text>{' '}
            <Radio.Group size="small" value={zhou} onChange={(e) => setZhou(e.target.value)} optionType="button"
              options={[{ label: '卯至酉', value: -2 }, { label: '卯至申', value: -1 }, { label: '日出入（日心）', value: -3 }, { label: '日出入（−50′）', value: -4 }, { label: '昼', value: 1 }, { label: '夜', value: 0 }]} />
          </div>
          <div style={{ marginTop: 4 }}>
            <Text type="secondary" style={{ fontSize: 12 }}>临辰戌</Text>{' '}
            <Radio.Group size="small" value={chuYu} onChange={(e) => setChuYu(e.target.value)} optionType="button"
              options={[{ label: '不出狱（课例）', value: 0 }, { label: '逆後尋（未核）', value: 1 }, { label: '以冲為神', value: 2 }]} />
          </div>
          {d.gui && <div style={{ fontSize: 12 }}>{`${d.zhou ? '昼' : '夜'}贵人${d.gui}临${d.guiLin ?? ''}${d.chuYu ? '（出狱移贵）' : ''}，${d.shun ? '顺' : '逆'}布十二天将`}{(zhou === -3 || zhou === -4) && `；以北纬${props.latitude.toFixed(2)}°、东经${props.longitude.toFixed(2)}°太阳高度定昼夜`}</div>}
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
          {d.sanChuanDun && <div style={{ fontSize: 12, color: '#0958d9' }}>{`遁干：${d.sanChuanDun.map((g) => g ?? '空').join(' ')}`}</div>}
          <Text type="secondary" style={{ fontSize: 11 }}>初传、中传、末传。据《六壬大全》卷一九法、卷五订讹</Text>
        </div>
        <div style={{ marginTop: 8 }}>
          <Text strong>课体</Text> {[d.zongMen, d.keMing].filter(Boolean).join('·')}
          <div style={{ fontSize: 11, color: '#888' }}>{[d.siLi && '今日四立（天禍）', d.fenZhi && '今日分至（天寇）', d.tuWang && '土王用事', d.yueSu && `月宿${d.yueSu}（${d.yueSuZhi}，起月宿法）`].filter(Boolean).join('；')}</div>
          {d.keGe && d.keGe.length > 0 && <div style={{ fontSize: 12 }}>{d.keGe.map((g, i) => <div key={i}><b>{g.n}</b> <Text type="secondary" style={{ fontSize: 11 }}>{g.note}</Text></div>)}</div>}
        </div>
        <div style={{ marginTop: 8, fontSize: 12 }}>
          {`旬首${d.xunShou ?? ''}，旬空${(d.xunKong ?? []).join('')}；驿马${d.yiMa ?? ''}、丁马${d.dingMa ?? '—'}、天马${d.tianMa ?? ''}；日德${d.riDe ?? ''}、日禄${d.riLu ?? ''}、羊刃${d.yangRen ?? ''}、日鬼${(d.riGui ?? []).join('')}`}
          <div>{`日墓${d.ganMu ?? ''}、辰墓${d.zhiMu ?? ''}；太岁${d.taiSui ?? ''}`}</div>
          {d.xingNian && <div>{`本命${d.benMing}，行年${d.xingNian}（虚岁 ${d.xuSui}）`}</div>}
        </div>
        <div style={{ marginTop: 8 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>遁干</Text>{' '}
          <Radio.Group size="small" value={dunRule} onChange={(e) => setDunRule(e.target.value)} optionType="button"
            options={[{ label: '旬遁（大全卷九）', value: 0 }, { label: '日干五子遁', value: 1 }]} />
        </div>
        <div style={{ marginTop: 8 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>日墓</Text>{' '}
          <Radio.Group size="small" value={muRule} onChange={(e) => setMuRule(e.target.value)} optionType="button"
            options={[{ label: '五行墓（书中诸例）', value: 0 }, { label: '十干墓（卷一条文）', value: 1 }]} />
        </div>
        <div style={{ marginTop: 8 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>遥克俱比</Text>{' '}
          <Radio.Group size="small" value={yaoRule} onChange={(e) => setYaoRule(e.target.value)} optionType="button"
            options={[{ label: '论渉害（訂訛）', value: 0 }, { label: '先舉（觀月經）', value: 1 }]} />{' '}
          <Text type="secondary" style={{ fontSize: 12 }}>渉害</Text>{' '}
          <Radio.Group size="small" value={sheHai} onChange={(e) => setSheHai(e.target.value)} optionType="button"
            options={[{ label: '訂訛', value: 0 }, { label: '尉山人', value: 1 }, { label: '比用格', value: 2 }]} />
        </div>
        <div style={{ marginTop: 8 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>本命</Text>{' '}
          <InputNumber size="small" placeholder="出生年" min={-2000} max={3000} value={birthYear} onChange={(v) => setBirthYear(v == null ? null : Number(v))} style={{ width: 96 }} />{' '}
          <Radio.Group size="small" value={male} onChange={(e) => setMale(e.target.value)} optionType="button" options={[{ label: '男', value: 1 }, { label: '女', value: 0 }]} />
          <Text type="secondary" style={{ fontSize: 11 }}>{' '}按干支年，立春前生者填前一年；行年：男一岁丙寅顺行、女一岁壬申逆行（卷六）</Text>
          <div style={{ marginTop: 4 }}>
            <Text type="secondary" style={{ fontSize: 12 }}>配偶本命</Text>{' '}
            <InputNumber size="small" placeholder="出生年" min={-2000} max={3000} value={peiYear} onChange={(v) => setPeiYear(v == null ? null : Number(v))} style={{ width: 96 }} />
            <Text type="secondary" style={{ fontSize: 11 }}>{` 繁昌课论夫妻行年${d.peiXingNian ? `；配偶行年${d.peiXingNian}` : ''}`}</Text>
          </div>
        </div>
        <div style={{ marginTop: 8 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>旺相</Text>{' '}
          <Radio.Group size="small" value={wangRule} onChange={(e) => setWangRule(e.target.value)} optionType="button"
            options={[{ label: '季月土旺（觀月經注）', value: 0 }, { label: '土旺四季各十八日（卷八）', value: 1 }]} />
        </div>
      </div>
    </Flex>
  )
})



const TaiyiPanel = memo(function TaiyiPanel(props: { wasm: LunarBackend; ear: string; jdUt: number; dayGz: number; hourZhi: number; spin: CompassSpinSource }) {
  const [kind, setKind] = useState(3)
  const [school, setSchool] = useState(0)
  const [dun, setDun] = useState(0)
  /** 诸说选项（taiyi_json 之 opts 位）：布门、门具、算和与算名、三才、阳九百六积年、月计 */
  const [buMen, setBuMen] = useState(0)
  const [menJu, setMenJu] = useState(0)
  const [heFa, setHeFa] = useState(0)
  const [sanCai, setSanCai] = useState(0)
  const [eLi, setELi] = useState(0)
  const [yueFa, setYueFa] = useState(0)
  /** 时计八门：张良（缺省）／王希明二至加时起门 */
  const [shiMen, setShiMen] = useState(0)
  const opts = buMen | (menJu << 2) | (heFa << 4) | (sanCai << 6) | (eLi << 8) | (yueFa << 9) | (shiMen << 10)
  const d = useMemo((): TaiyiData | null => {
    if (typeof props.wasm.taiyi_json !== 'function') return null
    try { return JSON.parse(props.wasm.taiyi_json(props.jdUt, props.ear, props.dayGz, props.hourZhi, kind, school, dun, opts)) as TaiyiData } catch { return null }
  }, [props.wasm, props.ear, props.jdUt, props.dayGz, props.hourZhi, kind, school, dun, opts])
  /** 时计插值（示意）：下一时辰之局与本时辰已过比例；只文昌、计神沿环滑行 */
  const [interp, setInterp] = useState(false)
  const next = useMemo(() => {
    if (!interp || kind !== 3 || typeof props.wasm.taiyi_json !== 'function') return undefined
    const t = new Date((props.jdUt - 2440587.5) * 86400000)
    const h = t.getHours() + t.getMinutes() / 60
    const into = (((h + 1) % 2) + 2) % 2
    const hz = (props.hourZhi + 1) % 12
    const dg = hz === 0 ? (props.dayGz + 1) % 60 : props.dayGz
    try {
      const nd = JSON.parse(props.wasm.taiyi_json(props.jdUt + (2 - into) / 24 + 1 / 1440, props.ear, dg, hz, kind, school, dun, opts)) as TaiyiData
      return { d: nd, f: into / 2 }
    } catch { return undefined }
  }, [interp, kind, props.wasm, props.ear, props.jdUt, props.dayGz, props.hourZhi, school, dun, opts])
  const three = (v: number, set: (v: number) => void, label: string, a: string[]) => (
    <div style={{ fontSize: 12 }}>{label}：<Radio.Group size="small" value={v} onChange={(e) => set(e.target.value)} options={a.map((x, i) => ({ label: x, value: i }))} /></div>
  )
  const suanText = (x?: TySuan) => x ? `${x.he ? '和' : '不和'}${x.ming.length ? `（${x.ming.join('、')}）` : ''}${x.sanCai ? `、${x.sanCai}` : ''}` : ''
  return (
    <Flex gap={16} wrap align="flex-start">
      <CompassSpin spin={props.spin} style={{ flex: '1 1 300px', maxWidth: 460 }}>{d ? <TaiyiWheel d={d} next={next} /> : <Text type="secondary">无法起局</Text>}</CompassSpin>
      <div style={{ flex: '1 1 260px' }}>
        <Flex vertical gap={6}>
          <Radio.Group size="small" value={kind} onChange={(e) => setKind(e.target.value)} optionType="button"
            options={[{ label: '岁计', value: 0 }, { label: '月计', value: 1 }, { label: '日计', value: 2 }, { label: '时计', value: 3 }]} />
          {kind === 3 && (
            <label style={{ fontSize: 12 }}>
              <input type="checkbox" checked={interp} onChange={(e) => setInterp(e.target.checked)} /> 时辰内插值（示意，非原典：只文昌、计神沿环向下一局之位滑去；太乙按洛书移宫飞行，始击、主客将仍逐局跳）
            </label>
          )}
          {kind !== 3 && (
            <Radio.Group size="small" value={dun} onChange={(e) => setDun(e.target.value)} optionType="button"
              options={[{ label: '阳遁', value: 0 }, { label: '阴遁', value: 1 }]} />
          )}
          {kind === 0 && (
            <Radio.Group size="small" value={school} onChange={(e) => setSchool(e.target.value)} optionType="button"
              options={[{ label: '金镜积年', value: 0 }, { label: '淘金歌积年', value: 1 }]} />
          )}
          {kind === 1 && three(yueFa, setYueFa, '月计', ['逐月节气', '金镜章岁法（含闰）'])}
          {kind === 3 && three(shiMen, setShiMen, '时计八门', ['张良（甲丙己辛日起）', '王希明（二至加时起，未核）'])}
          {three(buMen, setBuMen, '布门', ['直使加太乙', '开（杜）门加太乙', '八门定向'])}
          {three(menJu, setMenJu, '门具', ['金镜', '淘金歌', '秘书'])}
          {three(heFa, setHeFa, '算和', ['金镜', '淘金歌', '秘书'])}
          {three(sanCai, setSanCai, '三才', ['金镜', '淘金歌', '秘书'])}
          {three(eLi, setELi, '阳九百六', ['金镜甲寅积年', '黄帝元年起'])}
        </Flex>
        {d && (
          <div style={{ marginTop: 10, lineHeight: 1.8 }}>
            <div>{`${d.kind}第${d.r}局 ${d.dun}遁 · ${d.yuan}元第${d.ju}局 · 第${d.ji}纪`}</div>
            <div>{`太乙在${d.gong}宫（入宫第${d.ruGong}）· 文昌${d.tianMu}（${d.tianMuPos}）· 計神${d.jiShen} · 始擊${d.shiJi}（${d.shiJiPos}）`}</div>
            <div>{`主算${d.zhuSuan}（大將${d.zhuDa ?? '杜'}、參將${d.zhuCan ?? '杜'}）· 客算${d.keSuan}（大將${d.keDa ?? '杜'}、參將${d.keCan ?? '杜'}）`}</div>
            {d.dingJi && <div>{`定计目${d.dingJi}（${d.dingJiPos}）· 定算${d.dingJiSuan}（大將${d.dingJiDa ?? '杜'}、參將${d.dingJiCan ?? '杜'}）`}</div>}
            {d.siShen && <div style={{ fontSize: 13 }}>{`${d.suiNian ?? ''}岁：四神${d.siShen.name}、天乙${d.tianYi?.name ?? ''}、地乙${d.diYi?.name ?? ''}、直符${d.zhiFu?.name ?? ''}；五福${d.wuFu?.name ?? ''}；大游${d.daYou?.gong ?? ''}宫、小游${d.xiaoYou ?? ''}宫；君基${d.junJi?.zhi ?? ''}、臣基${d.chenJi?.zhi ?? ''}、民基${d.minJi?.zhi ?? ''}`}</div>}
            {(() => {
              const z = [['文昌', d.tianMuZhuang], ['始擊', d.shiJiZhuang], ['主大', d.zhuDaZhuang], ['主參', d.zhuCanZhuang], ['客大', d.keDaZhuang], ['客參', d.keCanZhuang]].filter(([, v]) => v)
              return z.length > 0 && <div style={{ fontSize: 13 }}>{`对太乙：${z.map(([k, v]) => `${k}${v}`).join('、')}`}</div>
            })()}
            {d.geJu && d.geJu.length > 0 && <div style={{ fontSize: 13 }}><b>格局</b> {d.geJu.join('、')}</div>}
            {(d.zhuJiangFa || d.keJiangFa) && <div style={{ fontSize: 13 }}>{[d.zhuJiangFa && '主將發', d.keJiangFa && '客將發'].filter(Boolean).join('、')}</div>}
            {d.zhiShiMen && <div style={{ fontSize: 13 }}><b>八门</b> {`${d.zhiShiMen}直使（入第${d.zhiShiRu}）· ${d.menJu ?? ''}`}</div>}
            {d.zhuSuanHe && <div style={{ fontSize: 13 }}>{`主算${suanText(d.zhuSuanHe)} · 客算${suanText(d.keSuanHe)}`}</div>}
            {d.shiJing && <div style={{ fontSize: 13 }}><b>十精</b> {d.shiJing.map((x) => `${x.n}${x.label}`).join('、')}{d.shiJingHe?.length ? `；${d.shiJingHe.join('、')}` : ''}</div>}
            {kind === 1 && yueFa === 1 && d.zhangSuiRun !== undefined && <div style={{ fontSize: 13 }}>{`章岁法本年${d.zhangSuiRun == null ? '无闰' : `闰${['', '正', '二', '三', '四', '五', '六', '七', '八', '九', '十', '十一', '十二'][d.zhangSuiRun]}月${d.zhangSuiRun >= 11 ? '（前一年）' : ''}`}（閏差二十秒二十七）`}</div>}
            {d.yangJiu && <div style={{ fontSize: 13 }}>{`${d.suiNian ?? ''}岁：第${d.yangJiu.di}陽九第${d.yangJiu.ruNian}年（${d.yangJiu.bang}邦）· 第${d.baiLiu?.di}百六第${d.baiLiu?.ruNian}年（${d.baiLiu?.bang}邦）`}</div>}
          </div>
        )}
        <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 8 }}>
          据《太乙金鏡式經》卷一、《古今圖書集成·藝術典》卷六八八、《太乙秘書》、淘金歌；时计冬至后阳遁、夏至后阴遁。内圈八宫（太乙不入中五），其外主客定计将、宫中诸神；八门（绿为开休生）；外圈十六神位与神名、目击诸神、三基与天皇帝符天时。诸岁神依金镜卷五按年推（月日时计亦用该岁）；十精依金镜卷七以本计积数推，阴遁取阳遁所冲；阳九百六依金镜卷七按年推。
        </Text>
      </div>
    </Flex>
  )
})

function BitRadio(p: { label: string; o: { value: boolean; onChange: (v: boolean) => void }; a: string; b: string }) {
  return <div>{p.label}：<Radio.Group size="small" value={p.o.value} onChange={(e) => p.o.onChange(e.target.value)} options={[{ label: p.a, value: false }, { label: p.b, value: true }]} /></div>
}

/** 七政四余：日月五星视黄经要五大行星历表，经星图 Worker（SkyEngine）按需算，切到本页签才加载 */
const QizhengPanel = memo(function QizhengPanel(props: { jdUt: number; hourZhi: number; latitude: number; longitude: number; spin: CompassSpinSource; live?: boolean }) {
  const call = useWorkerRpc<SkyRequest>(() => new Worker(new URL('./skyWorker.ts', import.meta.url), { type: 'module' }))
  const [luoAsc, setLuoAsc] = useState(false)
  const [meanNode, setMeanNode] = useState(false)
  const [ascendant, setAscendant] = useState(false)
  /** 星命诸说（位义见 SkyEngine::qizheng）：分宫、命主、身主、庙旺、强弱、大限年 */
  /** 天盘随时转（太阳宫加时之连续形）；缺省宫居本位（古盘静式） */
  const [turn, setTurn] = useState(false)
  const [gongRule, setGongRule] = useState(0)
  const [flagBits, setFlagBits] = useState(0)
  const flags = gongRule | flagBits
  const bit = (b: number) => ({ value: (flagBits & b) !== 0, onChange: (v: boolean) => setFlagBits(v ? flagBits | b : flagBits & ~b) })
  const [d, setD] = useState<QizhengWheelData | null>(null)
  const [err, setErr] = useState<string | null>(null)
  useEffect(() => {
    let live = true
    call<QizhengWheelData>({ type: 'qizheng', jdUt: props.jdUt, lon: props.longitude, lat: props.latitude, hourZhi: props.hourZhi, luoAsc, meanNode, ascendant, flags })
      .then((r) => { if (live) { setD(r); setErr(null) } })
      .catch((e: Error) => { if (live) setErr(e.message) })
    return () => { live = false }
  }, [call, props.jdUt, props.longitude, props.latitude, props.hourZhi, luoAsc, meanNode, ascendant, flags])
  return (
    <Flex gap={24} wrap align="flex-start">
      <CompassSpin spin={props.spin} style={{ flex: '1 1 320px', maxWidth: 440 }}>
        {d ? <QizhengWheel d={d} turn={turn ? { jdUt: props.jdUt, lonDeg: props.longitude, live: props.live } : undefined} /> : <Text type="secondary">{err ?? '加载五星历表与距星……'}</Text>}
      </CompassSpin>
      <div style={{ flex: '1 1 300px', minWidth: 0 }}>
        <Flex vertical gap={8}>
          <div>罗计：<Radio.Group size="small" value={luoAsc} onChange={(e) => setLuoAsc(e.target.value)} options={[{ label: '计都升交（回回历、清中期后时宪书）', value: false }, { label: '罗睺升交（崇祯新法、清初）', value: true }]} /></div>
          <div>交点：<Radio.Group size="small" value={meanNode} onChange={(e) => setMeanNode(e.target.value)} options={[{ label: '真交点', value: false }, { label: '平交点', value: true }]} /></div>
          <div>命宫：<Radio.Group size="small" value={ascendant} onChange={(e) => setAscendant(e.target.value)} options={[{ label: '太阳宫加时、卯上起命', value: false }, { label: '东方上升点', value: true }]} /></div>
          <BitRadio label="盘式" o={{ value: turn, onChange: setTurn }} a="宫居本位（静盘）" b="天盘随时转（地盘不动，卯上为命）" />
          <div>分宫：<Radio.Group size="small" value={gongRule} onChange={(e) => setGongRule(e.target.value)} options={[{ label: '中气', value: 0 }, { label: '宿度（星宗）', value: 1 }, { label: '宿度（大成）', value: 2 }, { label: '宿度（授时）', value: 3 }]} /></div>
          <BitRadio label="命主" o={bit(4)} a="命度主（星命溯源）" b="命宫主" />
          <BitRadio label="身主" o={bit(8)} a="身度主" b="身宫主（大成）" />
          <BitRadio label="庙旺" o={bit(16)} a="星论（三辰通载）" b="廟樂旺节（五星）" />
          <BitRadio label="强弱" o={bit(32)} a="大成" b="星宗" />
          <BitRadio label="大限" o={bit(64)} a="星宗定行限度" b="大成卷三例" />
          {d?.xingMing && (() => {
            const x = d.xingMing
            return (
              <Text style={{ fontSize: 13 }}>
                命度 {x.mingXiu != null && x.mingXiuDeg != null ? `${x.mingXiu}${x.mingXiuDeg.toFixed(1)}°` : `${x.mingLon.toFixed(1)}°`}；
                命宫主 {x.mingGongZhu}，命度主 {x.mingDuZhu ?? '—'}，<b>命主 {x.mingZhu}</b>；
                身宫 {ZHI[x.shenGong]}，身宫主 {x.shenGongZhu}，身度主 {x.shenDuZhu ?? '—'}，<b>身主 {x.shenZhu}</b>；
                童限至 {x.tongXian.toFixed(0)} 岁
              </Text>
            )
          })()}
          {d && (
            <table style={{ borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr>{['曜', '宫', '入宫度', '宿度', '', '庙旺', '对命主'].map((h) => <th key={h} style={{ textAlign: 'left', padding: '2px 8px', borderBottom: '1px solid #ddd' }}>{h}</th>)}</tr></thead>
              <tbody>
                {d.yao.map((y, k) => (
                  <tr key={y.n}>
                    <td style={{ padding: '2px 8px' }}>{y.n}</td>
                    <td style={{ padding: '2px 8px' }}>{ZHI[y.gong]}</td>
                    <td style={{ padding: '2px 8px' }}>{y.gongDeg.toFixed(2)}°</td>
                    <td style={{ padding: '2px 8px' }}>{y.xiu != null && y.xiuDeg != null ? `${y.xiu}${y.xiuDeg.toFixed(2)}°` : '—'}</td>
                    <td style={{ padding: '2px 8px', color: '#a8071a' }}>{y.retro ? '逆' : ''}</td>
                    <td style={{ padding: '2px 8px' }}>{d.xingMing?.dignity[k]?.join('') ?? ''}</td>
                    <td style={{ padding: '2px 8px' }}>{d.xingMing?.relation[k] ?? ''}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {d?.xingMing && (
            <table style={{ borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr>{['宫', '十二宫', '宫主', '强弱', '大限'].map((h) => <th key={h} style={{ textAlign: 'left', padding: '2px 8px', borderBottom: '1px solid #ddd' }}>{h}</th>)}</tr></thead>
              <tbody>
                {[...Array(12).keys()].map((i) => (d.ming - i + 12) % 12).map((z) => {
                  const dx = d.xingMing!.daXian.find((v) => v.gong === z)
                  return (
                    <tr key={z} style={z === d.ming ? { background: '#fff7e6' } : undefined}>
                      <td style={{ padding: '2px 8px' }}>{ZHI[z]}</td>
                      <td style={{ padding: '2px 8px' }}>{d.palaces[z]}{d.xingMing!.shenGong === z ? '·身' : ''}</td>
                      <td style={{ padding: '2px 8px' }}>{YAO_SHORT[d.xingMing!.gongZhu[z]!] ?? d.xingMing!.gongZhu[z]}</td>
                      <td style={{ padding: '2px 8px', color: d.xingMing!.strong[z] ? '#389e0d' : '#999' }}>{d.xingMing!.strong[z] ? '强' : '弱'}</td>
                      <td style={{ padding: '2px 8px' }}>{dx ? `${dx.from.toFixed(0)}—${dx.to.toFixed(0)} 岁` : ''}</td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          )}
          {d?.shenSha && d.shenSha.length > 0 && (() => {
            const hua = d.shenSha.filter((x) => x.yao != null)
            const gong = d.shenSha.filter((x) => x.yao == null && x.gong != null)
            const byYao = new Map<string, string[]>()
            hua.forEach((x) => byYao.set(x.yao!, [...(byYao.get(x.yao!) ?? []), x.n]))
            return (
              <div style={{ fontSize: 12, lineHeight: 1.7 }}>
                <div><b>化曜</b>（{d.nianGz != null ? `${'甲乙丙丁戊己庚辛壬癸'[d.nianGz % 10]}${ZHI[d.nianGz % 12]}年生，` : ''}{d.zhou ? '晝' : '夜'}生）：{[...byYao.entries()].map(([y, ns]) => `${YAO_SHORT[y] ?? y}為${ns.join('、')}`).join('；')}</div>
                <div><b>宫位神煞</b>：{[...Array(12).keys()].map((z) => {
                  const ns = gong.filter((x) => x.gong === z).map((x) => x.n)
                  return ns.length ? `${ZHI[z]}：${ns.join('、')}` : ''
                }).filter(Boolean).join('；')}</div>
              </div>
            )
          })()}
          {d?.zhuLuo && d.zhuLuo.length > 0 && <div style={{ fontSize: 12 }}>{`竹羅三限：${d.zhuLuo.map((z) => `${YAO_SHORT[z.yao] ?? z.yao}限 ${z.from}—${z.to} 岁（起${ZHI[z.start]}）`).join('；')}`}</div>}
          {(() => {
            /* 《星學大成》卷七急脚限、黄泉限（急脚之逆）、生年納音四煞宫、倒限之候（火羅孛土皆照限宫） */
            const ex = d as (QizhengWheelData & { jiJiao?: number[]; huangQuan?: number[]; siSha?: number[]; daoXian?: { sui: number; xian: string }[] }) | null
            if (!ex?.jiJiao?.length) return null
            const decades = (v: number[]) => [0, 10, 20, 30, 40, 50, 60, 70, 80, 90].map((i) => `${i + 1}${ZHI[v[i]!]}`).join(' ')
            const dao = new Map<number, string[]>()
            ex.daoXian?.forEach((x) => dao.set(x.sui, [...(dao.get(x.sui) ?? []), x.xian]))
            return (
              <div style={{ fontSize: 12, lineHeight: 1.7 }}>
                <div><b>急脚限</b>（每十年逆二宫，年内逐岁顺行）：{decades(ex.jiJiao)}</div>
                {ex.huangQuan?.length ? <div><b>黄泉限</b>（急脚之逆，读法未核）：{decades(ex.huangQuan)}</div> : null}
                {ex.siSha?.length === 4 ? <div><b>四煞宫</b>（納音沐浴、死、墓、絶）：{ex.siSha.map((z) => ZHI[z]).join('、')}；子午黄泉路、卯酉二八門</div> : null}
                <div><b>倒限之候</b>（火羅孛土皆照限宫）：{dao.size ? [...dao.entries()].map(([s, xs]) => `${s}岁${xs.join('、')}`).join('；') : '无'}</div>
              </div>
            )
          })()}
          <Text type="secondary" style={{ fontSize: 12 }}>
            视黄经（当日真春分点）；宫依中气分宫，黄经 0°—30° 为戌宫；宿度依清代二十八宿距星；紫炁依时宪书平行；命宫用所在地经纬度（上升点法）。以所选时刻为生时；命度依量天尺：命宫中与太阳同位之度（集成本《張果星宗》安命度法）。
          </Text>
        </Flex>
      </div>
    </Flex>
  )
})

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
  /** 月柱之支（月建，以节换月）；干支年（立春分，玄空定运用） */
  monthZhi?: number | null
  ganzhiYear?: number
  dateLabel: string
  onShiftHour: (n: number) => void
  onShiftDay: (n: number) => void
}) {
  const { wasm, ear, dayGz, hourZhi } = props
  const { bar: compassBar, trueHeading, spin } = useCompassBar(props)
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
                  ? <QimenPanel wasm={wasm!} ear={ear!} jdUt={jdUt} hourGz={props.hourGz} dayGz={dayGz!} spin={spin} />
                  : <Text type="secondary">{qimenLine}</Text>
              ),
            },
            { key: 'liuren', label: '大六壬', children: <LiurenPanel wasm={wasm!} ear={ear!} jdUt={jdUt} dayGz={dayGz!} hourZhi={hourZhi!} spin={spin} latitude={props.latitude} longitude={props.longitude} live={props.isLive} lunar={props.lunar} /> },
            { key: 'taiyi', label: '太乙', children: <TaiyiPanel wasm={wasm!} ear={ear!} jdUt={jdUt} dayGz={dayGz!} hourZhi={hourZhi!} spin={spin} /> },
            {
              key: 'liuyao', label: '六爻·周易',
              children: <LiuyaoPanel wasm={wasm!} lunar={props.lunar} hourZhi={hourZhi!} dayGz={dayGz!} monthZhi={props.monthZhi ?? null} />,
            },
            {
              key: 'xuankong', label: '玄空飞星',
              children: <XuankongPanel wasm={wasm!} ganzhiYear={props.ganzhiYear ?? new Date().getFullYear()} heading={trueHeading} spin={spin} ear={ear!} jdUt={jdUt} />,
            },
            {
              key: 'qizheng', label: '七政四余',
              children: <QizhengPanel jdUt={jdUt} hourZhi={hourZhi!} latitude={props.latitude} longitude={props.longitude} spin={spin} live={props.isLive} />,
            },
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
