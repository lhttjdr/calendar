/**
 * 择日页：年神方位二十四山图（《协纪辨方书》卷三）、当日神煞宜忌（月表 + 卷十铺注 + 时宪书体例；可切《董公選擇日要覽》为另一说）、日游神鹤神。
 * 与日历共用所选日期；图依古图南上北下、东左西右。
 */
import { Button, Card, Checkbox, Flex, InputNumber, Radio, Tag, Typography } from 'antd'
import { memo, useMemo, useRef, useState } from 'react'
import { Notes, OptionsFold, useOpt, useOptGroup } from './fold'
import type { LunarBackend } from './lunar-backend-types'
import { useCompassBar, useSpinDriver, type CompassSpinSource } from './CompassBar'
import { ensureLiShu, LISHU_FAMILY } from './Wheels'
import { JIAQU_DEFAULTS as JD, JIAQU_IN_NS, JIAQU_RULE_NS, hunShenList, leibianShenList, jiaquMonth, nvGzOf, type JiaquDay, type MonthLunar } from './jiaquShared'

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

/** `spin`：罗盘转盘驱动源（开罗盘时整盘随设备朝向转，由 rAF 直接写 transform）；文字随盘转 */
export const ShanWheel = memo(function ShanWheel({ data, size = 560, spin }: { data: NianShenData; size?: number; spin?: CompassSpinSource }) {
  ensureLiShu()
  const c = size / 2
  const gRef = useRef<SVGGElement | null>(null)
  useSpinDriver(gRef, spin, c)
  // 内圈年干支，中圈二十四山，外圈诸神（沿半径向外书写，按弧长等距排开）
  const r0 = size * 0.17
  const r1 = size * 0.25
  const r2 = size * 0.46
  // 二十四山自子起顺行，子在下（南上北下），东在左
  const ang = (i: number) => ((180 + 15 * i) * Math.PI) / 180
  const pt = (r: number, a: number) => [c + r * Math.sin(a), c - r * Math.cos(a)] as const
  return (
    <svg viewBox={`0 0 ${size} ${size}`} style={{ width: '100%', maxWidth: size, height: 'auto', fontFamily: LISHU_FAMILY }} role="img" aria-label="年神方位二十四山图">
      <g ref={gRef}>
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
      {spin?.active && <path d={`M ${c} ${size * 0.005} l -8 14 l 16 0 z`} fill="#a8071a" />}
    </svg>
  )
})

const GZ_GAN = '甲乙丙丁戊己庚辛壬癸', GZ_ZHI = '子丑寅卯辰巳午未申酉戌亥'

const WEEK = '日一二三四五六'
const verdictOf = (d: JiaquDay) => d.verdict ?? (d.ok ? '可用' : '不可用')
const verdictKind = (d: JiaquDay): 'ok' | 'zhi' | 'no' => (d.verdict === '經制化可用' ? 'zhi' : d.ok ? 'ok' : 'no')

/** 婚嫁月格：可用日着色，点日即把全站所选日期移到该日（当日卡、顶栏随之），下方列出该日逐条依据 */
function JiaquGrid(props: { list: JiaquDay[]; m: MonthLunar; selJdn: number; onPick: (jdn: number) => void; onlyOk: boolean }) {
  const first = props.list[0]
  if (!first) return null
  // 与日历同以周一为首（JDN+1 模 7：0 为周日）
  const lead = ((first.jdn + 1) % 7 + 6) % 7
  const cells: (JiaquDay | null)[] = [...Array(lead).fill(null), ...props.list]
  return (
    <div className="jq-grid" role="grid" aria-label="婚嫁择吉月格">
      {[...WEEK.slice(1), WEEK[0]].map((w) => <div key={w} className="jq-wk">{w}</div>)}
      {cells.map((d, k) => {
        if (!d) return <div key={`p${k}`} />
        const i = k - lead
        const kind = verdictKind(d)
        const dim = props.onlyOk && kind === 'no'
        return (
          <button key={d.jdn} type="button" role="gridcell" aria-selected={d.jdn === props.selJdn}
            className={`jq-cell jq-${kind}${d.jdn === props.selJdn ? ' jq-sel' : ''}${dim ? ' jq-dim' : ''}`}
            title={`${i + 1}日 ${verdictOf(d)}${d.reasons.length ? `：${d.reasons.join('；')}` : ''}`}
            onClick={() => props.onPick(d.jdn)}>
            <span className="jq-d">{i + 1}</span>
            <span className="jq-l">{`${props.m.days[i] === 1 ? `${props.m.leaps[i] ? '閏' : ''}${props.m.months[i]}月` : props.m.days[i]}`}</span>
            <span className="jq-v">{kind === 'ok' ? '可用' : kind === 'zhi' ? '制化' : d.yi ? '宜·否' : ''}</span>
            {(d.buJiang || (d.jiri?.length ?? 0) > 0) && <span className="jq-dot" title={[d.buJiang && '不將', d.jiri?.length && '类编吉日表'].filter(Boolean).join('、')} />}
          </button>
        )
      })}
    </div>
  )
}

/** 卷十宜忌等第表所论之主凶神 */
const MAJOR_XIONG = ['平日', '收日', '閉日', '劫煞', '災煞', '月煞', '月刑', '月害', '月厭', '大時', '天吏', '月破', '四廢']
/** 协纪月表嫁娶之注从何而来：卷十一宜神、忌神 → 卷十相併相抵与等第 → 铺注 */
function XieJiBasisView({ xj, ignored }: { xj: NonNullable<JiaquDay['xj']>; ignored?: string }) {
  const mode = xj.dengDi.split('→')[1] ?? ''
  const why = (() => {
    if (xj.yuebiao === '諸事不宜') return '月表諸事不宜'
    if (xj.yiShen.length && !xj.jiShen.length) return '有宜神、无忌神，注宜'
    if (!xj.yiShen.length && xj.jiShen.length) return '有忌神、无宜神，注忌'
    if (!xj.yiShen.length && !xj.jiShen.length) return '宜神忌神皆不值，两不注'
    // 宜忌并值：主凶神按等第，次凶神按卷十「與德等併則不忌」
    if (mode) {
      const left = xj.dengDi.split('→')[0] ?? ''
      const major = xj.jiShen.filter((n) => MAJOR_XIONG.includes(n))
      const grade = left.replace('遇德', '')
      const eff = { 從宜: '注宜', 相抵: '宜忌相抵，两不注', 從忌: '注忌', 皆忌: '宜事皆忌' }[mode] ?? mode
      return `宜忌并值：主凶神${major.length ? `（${major.join('、')}）` : ''}等第「${grade}」${left.includes('遇德') ? '而遇德' : '、不遇德'}，${mode}——${eff}（卷十宜忌等第表）`
    }
    return xj.yuebiao === '宜' ? '宜忌并值：所值忌神为德、赦、願、六合等所解（卷十各条「併則不忌」）' : '宜忌并值：忌神不为所值之吉神所解，从忌'
  })()
  return (
    <div style={{ fontSize: 12, margin: '4px 0', padding: '4px 8px', background: 'var(--bg-2)', borderRadius: 6 }}>
      <div>
        <Text strong style={{ fontSize: 12 }}>{ignored ? `协纪推法（不看${ignored}）` : '协纪月表'} </Text>
        {`嫁娶${xj.final === '宜' ? '宜' : xj.final === '忌' ? '忌' : xj.final === '諸事不宜' ? '（諸事不宜）' : '两不注'}`}
        {xj.final !== xj.yuebiao && <Text type="warning" style={{ fontSize: 12 }}>{`（月表原${xj.yuebiao}，铺注${xj.puzhu.length ? `值${xj.puzhu.join('、')}` : ''}而改）`}</Text>}
      </div>
      <div><Text type="success" style={{ fontSize: 12 }}>宜神 </Text>{xj.yiShen.join(' ') || '—'}<Text type="danger" style={{ fontSize: 12, marginLeft: 8 }}>忌神 </Text>{xj.jiShen.join(' ') || '—'}</div>
      <div style={{ color: 'var(--text-3)' }}>{`${ignored ? '推' : '月表'}：${why}`}</div>
    </div>
  )
}

/** 一日之依据：否决、制化、标注分组 */
function JiaquDetail(props: { d: JiaquDay; i: number; m: MonthLunar; ignored?: string }) {
  const { d, i, m } = props
  const kind = verdictKind(d)
  const marks = [
    d.yi ? '月表宜嫁娶' : d.ji ? '月表忌嫁娶' : '',
    d.buJiang ? '不將（协纪歷例表）' : '',
    d.jiang ? `类编法${d.jiang}` : '',
    d.zhouTang ? `周堂值${d.zhouTang}` : '',
    `女命${d.nvMingYue}`,
    d.liuRen ? `小六壬${d.liuRen}` : '',
    d.wuRi ?? '',
  ].filter(Boolean)
  return (
    <div className="jq-detail">
      <div style={{ fontSize: 15, marginBottom: 4 }}>
        <Text strong type={kind === 'ok' ? 'success' : kind === 'zhi' ? 'warning' : 'secondary'}>{`${i + 1}日 ${verdictOf(d)}`}</Text>
        <Text type="secondary" style={{ fontSize: 12 }}>{` · 农历${m.leaps[i] ? '閏' : ''}${m.months[i]}月${m.days[i]}日`}</Text>
      </div>
      {d.xj && <XieJiBasisView xj={d.xj} ignored={props.ignored} />}
      {d.reasons.length > 0 && <div><Text type="danger" strong style={{ fontSize: 12 }}>否决 </Text>{d.reasons.map((r, k) => <div key={k} className="jq-r">{r}</div>)}</div>}
      {d.jie && d.jie.length > 0 && <div><Text type="warning" strong style={{ fontSize: 12 }}>制化 </Text>{d.jie.map(([a, b], k) => <div key={k} className="jq-r">{`${a}（${b}）`}</div>)}</div>}
      <div style={{ fontSize: 12, marginTop: 4 }}><Text type="secondary">标注 </Text>{marks.join(' · ')}</div>
      {d.leibian.length > 0 && <div style={{ fontSize: 12 }}><Text type="secondary">类编凶神 </Text>{d.leibian.map(([a]) => a).join(' ')}</div>}
      {(() => {
        // 核心附注中标注栏未含者（俗说诸忌与吉位、天狗、五合五離之名、受死等）
        const skip = /^(嫁娶周堂值|女命|类编所列|类编吉日表|不將|赤口|上兀|下兀)/
        const ns = d.notes.filter((x) => !skip.test(x))
        return ns.length > 0 && <div style={{ fontSize: 12 }}><Text type="secondary">附注 </Text>{ns.join('；')}</div>
      })()}
      {d.jiri && d.jiri.length > 0 && <div style={{ fontSize: 12 }}><Text type="success">类编吉日表 </Text>{d.jiri.map(([a, b]) => (b ? `${a}（${b}）` : a)).join('、')}</div>}
    </div>
  )
}

/** 婚嫁择吉：所显示月份逐日判定（协纪为纲、类编历法通书大全卷七并列；见知识库 jiaqu） */
function JiaquPanel(props: { wasm: LunarBackend; ear: string; monthLunar: MonthLunar; monthLabel: string; selJdn: number; onPick: (jdn: number) => void }) {
  const [view, setView] = useState<'grid' | 'list'>('grid')
  // 输入与规则皆记于本机，日历「择吉」图层同读（jiaquShared.ts）
  const gi = useOptGroup(JIAQU_IN_NS)
  const [nvYear, setNvYear] = useOpt(gi, 'nvYear', JD.nvYear)
  const [wengGu, setWengGu] = useOpt(gi, 'wengGu', JD.wengGu)
  const [fuMu, setFuMu] = useOpt(gi, 'fuMu', JD.fuMu)
  const [naXu, setNaXu] = useOpt(gi, 'naXu', JD.naXu)
  const g = useOptGroup(JIAQU_RULE_NS)
  const [useYue, setUseYue] = useOpt(g, 'useYue', JD.useYue, '女命行嫁月')
  const [xieji, setXieji] = useOpt(g, 'xieji', JD.xieji, '协纪月表')
  const [zt, setZt] = useOpt(g, 'zt', JD.zt, '嫁娶周堂')
  const [useXiong, setUseXiong] = useOpt(g, 'useXiong', JD.useXiong, '类编凶神')
  const [onlyOk, setOnlyOk] = useState(false)
  const [zhiHua, setZhiHua] = useOpt(g, 'zhiHua', JD.zhiHua, '制化')
  const [aoTou, setAoTou] = useOpt(g, 'aoTou', JD.aoTou, '鳌头白虎')
  const [chiKou, setChiKou] = useOpt(g, 'chiKou', JD.chiKou, '赤口')
  const [hengTian, setHengTian] = useOpt(g, 'hengTian', JD.hengTian, '橫天朱雀')
  /** 类编凶神逐神否决（位掩码）；俗说诸忌 */
  const [leibianMask, setLeibianMask] = useOpt(g, 'leibianMask', JD.leibianMask, '类编凶神逐神')
  const [yangGong, setYangGong] = useOpt(g, 'yangGong', JD.yangGong, '楊公忌')
  const [tianGou, setTianGou] = useOpt(g, 'tianGou', JD.tianGou, '天狗')
  const [zhangGuang, setZhangGuang] = useOpt(g, 'zhangGuang', JD.zhangGuang, '章光')
  const [hongSha, setHongSha] = useOpt(g, 'hongSha', JD.hongSha, '紅沙')
  const lbList = useMemo(() => leibianShenList(props.wasm), [props.wasm])
  /** 类编实际否决之集：有掩码按掩码，否则「用类编凶神」取缺省所列 */
  const lbOn = (i: number) => (leibianMask ? (leibianMask >>> i) & 1 : useXiong ? Number(lbList[i]?.[1]) : 0) === 1
  const lbToggle = (i: number) => {
    let m = 0
    lbList.forEach((_, j) => { if (j === i ? !lbOn(j) : lbOn(j)) m |= 1 << j })
    setLeibianMask(m >>> 0)
    if (m === 0) setUseXiong(false)
  }
  const lbPreset = (def: boolean) => { setLeibianMask(0); setUseXiong(def) }
  /** 协纪推法中不看之神（「、」连名） */
  const [ignore, setIgnore] = useOpt(g, 'ignore', JD.ignore, '不看之神')
  const shenList = useMemo(() => hunShenList(props.wasm), [props.wasm])
  const ignoreSet = new Set(ignore.split('、').filter(Boolean))
  const toggleShen = (n: string) => {
    const s2 = new Set(ignoreSet)
    if (s2.has(n)) s2.delete(n); else s2.add(n)
    setIgnore(shenList.map(([x]) => x).filter((x) => s2.has(x)).join('、'))
  }
  const nvGz = nvGzOf(nvYear)
  const m = props.monthLunar
  const [err, setErr] = useState<string | null>(null)
  const list = useMemo((): JiaquDay[] | null => {
    try {
      setErr(null)
      return jiaquMonth(props.wasm, props.ear, m, { nvYear, wengGu, fuMu, naXu, useYue, xieji, zt, useXiong, zhiHua, aoTou, chiKou, hengTian, ignore, leibianMask, yangGong, tianGou, zhangGuang, hongSha })
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      console.error('jiaqu_days_json', e)
      setErr(msg)
      return null
    }
  }, [props.wasm, props.ear, m, nvYear, wengGu, fuMu, naXu, useYue, zt, xieji, useXiong, zhiHua, aoTou, chiKou, hengTian, ignore, leibianMask, yangGong, tianGou, zhangGuang, hongSha])
  return (
    <Card size="small" title={`婚嫁择吉（${props.monthLabel}）`} style={{ marginTop: 16 }}>
      <Flex gap={12} wrap align="center" style={{ marginBottom: 8 }}>
        <span>女命生年 <InputNumber size="small" value={nvYear} min={1} max={2200} onChange={(v) => v != null && setNvYear(v)} />{` ${GZ_GAN[nvGz % 10]}${GZ_ZHI[nvGz % 12]}（立春换年）`}</span>
        <Checkbox checked={wengGu} onChange={(e) => setWengGu(e.target.checked)}>翁姑在</Checkbox>
        <Checkbox checked={fuMu} onChange={(e) => setFuMu(e.target.checked)}>女父母在</Checkbox>
        <Checkbox checked={naXu} onChange={(e) => setNaXu(e.target.checked)}>纳婿</Checkbox>
        <Checkbox checked={onlyOk} onChange={(e) => setOnlyOk(e.target.checked)}>只看可用之日</Checkbox>
      </Flex>
      <OptionsFold g={g} title="取舍规则" defaultNote="依协纪：月表须宜嫁娶、协纪周堂">
      <Flex gap={12} wrap align="center">
        <span>协纪月表 <Radio.Group size="small" value={xieji} onChange={(e) => setXieji(e.target.value)} optionType="button"
          options={[{ label: '须宜嫁娶', value: 0 }, { label: '只避忌嫁娶', value: 1 }, { label: '不用', value: 2 }]} /></span>
        {xieji !== 2 && shenList.length > 0 && (
          <div style={{ flexBasis: '100%', fontSize: 12, lineHeight: 2 }}>
            <Text type="secondary" style={{ fontSize: 12 }}>协纪所看之神（点去即「不看」，按协纪推法重推婚事宜忌；全看即月表）：</Text>
            {([true, false] as const).map((isYi) => (
              <div key={String(isYi)}>
                <Text style={{ fontSize: 12, color: isYi ? 'var(--ok-text)' : '#cf1322' }}>{isYi ? '宜神 ' : '忌神 '}</Text>
                {shenList.filter(([, y]) => y === isYi).map(([n]) => (
                  <Tag.CheckableTag key={n} checked={!ignoreSet.has(n)} onChange={() => toggleShen(n)}
                    style={ignoreSet.has(n) ? { textDecoration: 'line-through', opacity: 0.6 } : undefined}>
                    {n}
                  </Tag.CheckableTag>
                ))}
              </div>
            ))}
            {ignoreSet.size > 0 && <Text type="warning" style={{ fontSize: 12 }}>{`已不看：${[...ignoreSet].join('、')}——此时所注已非协纪月表，乃按其推法略去诸神所得`}</Text>}
          </div>
        )}
        <details className="day-ji" style={{ flexBasis: '100%', fontSize: 12 }}>
          <summary>协纪月表怎样注嫁娶</summary>
          <div style={{ lineHeight: 1.7, color: 'var(--text-2)' }}>
            <div>1. <b>宜神</b>（卷十一「嫁娶」）：天德、月德、天德合、月德合、天赦、天願、三合、天喜、六合、不將。</div>
            <div>2. <b>忌神</b>（同上）：月破、平日、收日、閉日、劫煞、災煞、月煞、月刑、月害、月厭、厭對、大時、天吏、四廢、四忌、四窮、五墓、往亡、八專、亥日（百忌日「亥不嫁娶」）。</div>
            <div>3. <b>宜忌并值</b>：主凶神（平收閉日、劫災月煞、月刑、月害、月厭、大時、天吏）按卷十宜忌等第表，以所值最重之等第与遇德否定「從宜／相抵／從忌／皆忌」；厭對、八專等次凶神按卷十各条「與德等併則不忌」。月表 720 日皆据此推出并与影印本逐页核过。</div>
            <div>4. <b>铺注</b>（卷十鋪註條例）：周堂值夫、值婦不注宜嫁娶；上朔、四離、四絕、晦日、無祿、月忌等止注数事，余事不注宜或注忌。</div>
            <div>「须宜嫁娶」：注宜方可用（时宪书注宜之日）；「只避忌嫁娶」：两不注之日亦可；「不用」：不看协纪月表。每日所值之神与推得之由见月格点日之依据。</div>
          </div>
        </details>
        <span>嫁娶周堂 <Radio.Group size="small" value={zt} onChange={(e) => setZt(e.target.value)} optionType="button"
          options={[{ label: '协纪', value: 0 }, { label: '类编', value: 1 }, { label: '不用', value: 2 }]} /></span>
        <Checkbox checked={useYue} onChange={(e) => setUseYue(e.target.checked)}>女命行嫁月否决（类编；协纪奏准删除）</Checkbox>
        <div style={{ flexBasis: '100%', fontSize: 12, lineHeight: 2 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>类编卷七嫁娶凶神（点选即据以否决，未选者只列）：</Text>
          <Button size="small" type="link" onClick={() => lbPreset(true)}>按「論嫁娶凶日」等缺省所列</Button>
          <Button size="small" type="link" onClick={() => lbPreset(false)}>全不用</Button>
          <div>
            {lbList.map(([n], i) => (
              <Tag.CheckableTag key={n} checked={lbOn(i)} onChange={() => lbToggle(i)}>{n}</Tag.CheckableTag>
            ))}
          </div>
        </div>
        <div style={{ flexBasis: '100%', fontSize: 12 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>通书俗说（协纪卷三十六辨讹斥之；不选则只列于标注）：</Text>
          <Flex gap={12} wrap align="center">
            <Checkbox checked={yangGong} onChange={(e) => setYangGong(e.target.checked)}>楊公忌否决</Checkbox>
            <Checkbox checked={tianGou} onChange={(e) => setTianGou(e.target.checked)}>天狗頭尾口腹否决</Checkbox>
            <Checkbox checked={zhangGuang} onChange={(e) => setZhangGuang(e.target.checked)}>章光否决</Checkbox>
            <span>紅沙 <Radio.Group size="small" value={hongSha} onChange={(e) => setHongSha(e.target.value)} optionType="button"
              options={[{ label: '不用（协纪）', value: 0 }, { label: '轉神厯（孟酉仲巳季丑）', value: 1 }, { label: '又一本（孟巳仲酉季丑）', value: 2 }]} /></span>
          </Flex>
        </div>
        <Checkbox checked={aoTou} onChange={(e) => setAoTou(e.target.checked)}>白虎从鳌头（值门床堂路）</Checkbox>
        <span>赤口 <Radio.Group size="small" value={chiKou} onChange={(e) => setChiKou(e.target.value)} optionType="button"
          options={[{ label: '不用（协纪）', value: 0 }, { label: '类编（凡赤口）', value: 1 }, { label: '祛疑说（值寅巳酉戌）', value: 2 }, { label: '兀日起法（协纪所述：阳年起小吉、阴年起留連）', value: 3 }]} /></span>
        <Checkbox checked={hengTian} onChange={(e) => setHengTian(e.target.checked)}>橫天朱雀否决（鳌头：初一）</Checkbox>
        <Checkbox checked={zhiHua} onChange={(e) => setZhiHua(e.target.checked)}>用制化（周堂回避、白虎禳法；协纪不录）</Checkbox>
      </Flex>
      </OptionsFold>
      <div style={{ height: 8 }} />
      <Radio.Group size="small" value={view} onChange={(e) => setView(e.target.value)} optionType="button" style={{ marginBottom: 8 }}
        options={[{ label: '月格', value: 'grid' }, { label: '逐日列表', value: 'list' }]} />
      {!list ? <Text type="secondary">{`无法判定${err ? `：${err}` : '（wasm 无此函数）'}`}</Text> : view === 'grid' ? (
        <Flex gap={16} wrap align="flex-start">
          <div style={{ flex: '1 1 320px', maxWidth: 520 }}>
            <JiaquGrid list={list} m={m} selJdn={props.selJdn} onPick={props.onPick} onlyOk={onlyOk} />
            <Text type="secondary" style={{ fontSize: 11 }}>{`本月可用 ${list.filter((d) => d.ok).length} 日；绿为可用、黄为经制化可用、灰为不可用（「宜·否」：月表宜嫁娶而为他规则否决）；角上小点为不將或类编吉日表所列。点日看依据，全站所选日期随之移到该日。`}</Text>
          </div>
          <div style={{ flex: '1 1 280px' }}>
            {(() => {
              const i = list.findIndex((d) => d.jdn === props.selJdn)
              return i >= 0 ? <JiaquDetail d={list[i]!} i={i} m={m} ignored={ignore} /> : <Text type="secondary">点月格中某日看其依据</Text>
            })()}
          </div>
        </Flex>
      ) : (
        <div style={{ fontSize: 12, lineHeight: 1.7 }}>
          {list.map((d, i) => (onlyOk && !d.ok ? null : (
            <div key={d.jdn} style={{ borderBottom: '1px solid var(--border-1)', padding: '2px 0' }}>
              <Text strong type={d.verdict === '經制化可用' ? 'warning' : d.ok ? 'success' : 'secondary'}>{`${i + 1}日 ${d.verdict ?? (d.ok ? '可用' : '不可用')}`}</Text>
              {` · 农历${m.leaps[i] ? '閏' : ''}${m.months[i]}月${m.days[i]}日`}
              {d.yi ? ' · 宜嫁娶' : d.ji ? ' · 忌嫁娶' : ''}
              {d.buJiang ? ' · 不將（协纪歷例表）' : ''}{d.jiang ? ` · 类编法${d.jiang}` : ''}
              {d.zhouTang ? ` · 周堂值${d.zhouTang}` : ''}
              {` · 女命${d.nvMingYue}`}
              {d.reasons.length > 0 && <Text type="danger">{` · ${d.reasons.join('；')}`}</Text>}
              {d.jie && d.jie.length > 0 && <Text type="warning">{` · 制化：${d.jie.map(([a, b]) => `${a}（${b}）`).join('；')}`}</Text>}
              {d.leibian.length > 0 && <Text type="secondary">{` · 类编凶神：${d.leibian.map(([a]) => a).join(' ')}`}</Text>}
              {d.liuRen ? <Text type="secondary">{` · 小六壬${d.liuRen}`}</Text> : null}
              {d.wuRi ? <Text type="secondary">{` · ${d.wuRi}`}</Text> : null}
              {d.jiri && d.jiri.length > 0 && <Text type="success">{` · 类编吉日表：${d.jiri.map(([a, b]) => (b ? `${a}（${b}）` : a)).join('、')}`}</Text>}
            </div>
          )))}
        </div>
      )}
      <Notes>
        可用与否全依上面所选的规则：每条规则可单独取舍，每日列出否决它的规则与出处。缺省选协纪诸项（月表加铺注须宜嫁娶、协纪嫁娶周堂），因协纪为清代官定；协纪于女命大利月诸说「奏准刪除」，故缺省只列不否决。《类编历法通书大全》卷七之女命行嫁月、周堂、嫁娶凶神表，鳌头之白虎与橫天朱雀，赤口（类编卷十五小六壬，协纪不立；或依协纪卷三十六所述兀日起法，即类编卷二六輪兀，值下兀为赤口；闰月皆从本月起），以及周堂回避、白虎禳法等制化，皆为可选。阴阳不将只作标注（协纪歷例表与类编法两家）；类编卷七逐月求婚、下定、嫁娶吉日表所列之日亦只作标注。只算到规则给出的结果，不论吉凶应验。
      </Notes>
    </Card>
  )
}

type DongGongRule = { event: string; yi: boolean; grade: string; basis: string; cond: string; note: string; xieji: string[] }
type DongGongData = {
  monthZhi: string
  dayGz: string
  jianChu: string
  noText: boolean
  text: string
  fullText: string
  entryNote: string
  gods: string[]
  rules: DongGongRule[]
}

/** 宜忌另一说：《董公選擇日要覽》。无宜忌之文的日子只说明，不以协纪填补 */
function DongGongYiJi({ wasm, ear, jd }: { wasm: LunarBackend; ear: string; jd: number }) {
  const d = useMemo((): DongGongData | null => {
    if (typeof wasm.dong_gong_day_json !== 'function') return null
    try {
      return JSON.parse(wasm.dong_gong_day_json(jd, ear)) as DongGongData
    } catch {
      return null
    }
  }, [wasm, ear, jd])
  if (!d) return <Text type="secondary">董公宜忌需新版 wasm</Text>
  const label = (r: DongGongRule) => `${r.event}${r.grade ? `（${r.grade}）` : ''}${r.cond ? `〔${r.cond}〕` : ''}`
  const yi = d.rules.filter((r) => r.yi).map(label)
  const ji = d.rules.filter((r) => !r.yi).map(label)
  // 缘由：按所据原句归并
  const basis: { b: string; yi: string[]; ji: string[]; notes: string[] }[] = []
  for (const r of d.rules) {
    let g = basis.find((x) => x.b === r.basis)
    if (!g) basis.push((g = { b: r.basis, yi: [], ji: [], notes: [] }))
    ;(r.yi ? g.yi : g.ji).push(label(r))
    if (r.note && !g.notes.includes(r.note)) g.notes.push(r.note)
  }
  return (
    <div style={{ fontSize: 13, lineHeight: 1.7 }}>
      <div>{`董公：${d.monthZhi}月${d.jianChu}${d.dayGz.slice(1)}日 · ${d.dayGz}`}</div>
      {d.noText ? (
        <div><Text strong>董公无宜忌之文</Text></div>
      ) : (
        <>
          <div>{`宜：${yi.length ? yi.join(' ') : '—'}`}</div>
          <div>{`忌：${ji.length ? ji.join(' ') : '—'}`}</div>
        </>
      )}
      {d.gods.length > 0 && <div>{`董公所举神煞：${d.gods.join(' ')}`}</div>}
      {basis.length > 0 && (
        <div style={{ marginTop: 4 }}>
          {basis.map((g) => (
            <div key={g.b} style={{ fontSize: 12 }}>
              {`「${g.b}」→ ${g.yi.length ? `宜 ${g.yi.join(' ')}` : ''}${g.yi.length && g.ji.length ? '；' : ''}${g.ji.length ? `忌 ${g.ji.join(' ')}` : ''}`}
              {g.notes.length > 0 && <Text type="secondary" style={{ fontSize: 11 }}>{`（校：${g.notes.join('；')}）`}</Text>}
            </div>
          ))}
        </div>
      )}
      <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 4 }}>
        {`原文（${d.monthZhi}月${d.jianChu}${d.dayGz.slice(1)}日条）：${d.fullText}`}
        {d.entryNote ? `　校：${d.entryNote}` : ''}
      </Text>
    </div>
  )
}

/** 神煞起例诸说（核心 shensha::ShenshaOptions、nianshen::NianShenOptions；出处见知识库 tongshu.md 第五节） */
const QILI_ITEMS: { key: string; label: string; opts: string[] }[] = [
  { key: 'tianDe', label: '天德·天德合四仲', opts: ['协纪（四仲无）', '类编（申亥寅巳）'] },
  { key: 'tianHuo', label: '天火', opts: ['协纪（逆行）', '类编（顺行四仲）'] },
  { key: 'mingFei', label: '鳴吠', opts: ['协纪', '类编十四日', '类编安葬十二日'] },
  { key: 'mingFeiDui', label: '鳴吠對', opts: ['协纪', '类编十一日', '类编十日'] },
  { key: 'fuRi', label: '復日', opts: ['协纪', '集正', '百忌', '阴阳备用', '百忌歌两干'] },
  { key: 'huangWu', label: '荒蕪', opts: ['不列（协纪：即五虚）', '按季三支', '类编表序', '类编新增起例'] },
  { key: 'tianZei', label: '天賊', opts: ['协纪（正月丑逆行）', '类编（正月辰进五辰）'] },
  { key: 'canMing', label: '蠶命（年神）', opts: ['协纪', '类编'] },
]
const QILI_LEIBIAN: Record<string, number> = { tianDe: 1, tianHuo: 1, mingFei: 1, mingFeiDui: 1, fuRi: 1, huangWu: 2, tianZei: 1, canMing: 1 }

/** 神煞起例：part=controls 为起例区内的选择，part=diff 为当日结果区「较协纪多／少」 */
function QiLiPanel(props: { wasm: LunarBackend; ear: string; jd: number; o: Record<string, number>; setO: (o: Record<string, number>) => void; part: 'controls' | 'diff' }) {
  const { o, setO } = props
  const [open, setOpen] = useState(false)
  const v = (k: string) => o[k] ?? 0
  const r = useMemo(() => {
    if (props.part !== 'diff' || typeof props.wasm.qili_shen_day_json !== 'function') return null
    try {
      return JSON.parse(props.wasm.qili_shen_day_json(props.jd, props.ear, v('tianDe'), v('tianHuo'), v('mingFei'), v('mingFeiDui'), v('fuRi'), v('huangWu'), v('canMing'), v('tianZei'))) as {
        add: [string, boolean][]; remove: [string, boolean][]; canMing: { xieji: string; sel: string }
      }
    } catch { return null }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.wasm, props.ear, props.jd, o, props.part])
  const all = Object.values(o).every((x) => x === 0) ? 'xieji' : QILI_ITEMS.every((it) => v(it.key) === QILI_LEIBIAN[it.key]) ? 'leibian' : 'mix'
  if (props.part === 'diff') return r && all !== 'xieji' ? (
    <div style={{ marginTop: 6, fontSize: 13 }}>
      {r.add.length + r.remove.length === 0 ? <Text type="secondary">所选起例下，本日所值之神与协纪同</Text> : (
        <>
          {r.add.length > 0 && <div>{'较协纪多：'}{r.add.map(([n, j]) => <span key={n} style={{ color: j ? '#237804' : '#a8071a', marginRight: 6 }}>{n}</span>)}</div>}
          {r.remove.length > 0 && <div>{'较协纪少：'}{r.remove.map(([n, j]) => <span key={n} style={{ color: j ? '#237804' : '#a8071a', marginRight: 6, textDecoration: 'line-through' }}>{n}</span>)}</div>}
        </>
      )}
      {r.canMing.sel !== r.canMing.xieji && <div>{`蠶命：协纪在${r.canMing.xieji}，所选在${r.canMing.sel}`}</div>}
    </div>
  ) : null
  return (
    <div style={{ fontSize: 13 }}>
      <Flex align="center" gap={8} wrap>
        <Text>神煞起例依：</Text>
        <Radio.Group size="small" value={all} onChange={(e) => { if (e.target.value === 'xieji') setO({}); else if (e.target.value === 'leibian') setO({ ...QILI_LEIBIAN }); else setOpen(true) }}>
          <Radio.Button value="xieji">协纪（缺省）</Radio.Button>
          <Radio.Button value="leibian">类编</Radio.Button>
          <Radio.Button value="mix">逐神选</Radio.Button>
        </Radio.Group>
        <Button size="small" type="link" onClick={() => setOpen(!open)}>{open ? '收起' : '逐神'}</Button>
      </Flex>
      {open && (
        <div style={{ marginTop: 6 }}>
          {QILI_ITEMS.map((it) => (
            <div key={it.key} style={{ marginBottom: 4 }}>
              <Text style={{ fontSize: 12, display: 'inline-block', width: 110 }}>{it.label}</Text>
              <Radio.Group size="small" value={v(it.key)} onChange={(e) => setO({ ...o, [it.key]: e.target.value })}
                options={it.opts.map((label, i) => ({ label, value: i }))} />
            </div>
          ))}
        </div>
      )}
      <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 4 }}>
        只改所列当日之神，宜忌仍依协纪月表。类编为元末《類編曆法通書大全》，协纪按语所斥「通書」诸说多见于此（鸣吠、复日、天火、蚕命等；天贼两说无一日同）；乾隆三十年时宪书蚕命从协纪。
      </Text>
    </div>
  )
}

export function ZeRiPage(props: {
  wasm: LunarBackend | null
  ear: string | null
  /** 所选民用日（任一时刻）的 JD */
  jd: number
  /** shensha_day_wasm 的输出（吉凶神、宜、忌、時憲書宜、時憲書朱字），无则空 */
  shensha: string
  jianchu: string
  latitude: number
  longitude: number
  /** 定位：取 GPS 经纬度写回观测者位置 */
  onLocate: () => Promise<string>
  /** 所在地磁偏角（度，东偏为正）；未知为 null */
  declination: number | null
  monthLunar?: MonthLunar | null
  monthLabel?: string
  /** 把全站所选日期移到某民用日（JDN） */
  onPickJdn?: (jdn: number) => void
}) {
  const { wasm, ear, jd } = props
  const { bar: compassBar, spin } = useCompassBar(props)
  const g = useOptGroup('zeriDay')
  const [yiJiSrc, setYiJiSrc] = useOpt<'xieji' | 'dong'>(g, 'yiJiSrc', 'xieji', '宜忌依董公')
  const [qili, setQili] = useState<Record<string, number>>({})
  g.items.push({ label: '神煞起例', changed: Object.values(qili).some((x) => x !== 0), reset: () => setQili({}) })
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
      {compassBar}
      {!data ? (
        <Text type="secondary">加载历表后显示（需 VSOP87 历表）</Text>
      ) : (
        <Flex gap={16} wrap align="flex-start">
          <Card size="small" title={`年神方位（${data.year}年，立春换年）`} style={{ flex: '1 1 420px', maxWidth: 600 }}>
            <ShanWheel data={data} spin={spin} />
            <Notes>
              据《协纪辨方书》卷三；南上北下，东左西右。打开罗盘（手机），盘随设备转动，图上方位与实地相合（磁北经磁偏角改为真北；磁偏角未知时按磁北）。绿为吉神（岁德、奏书、博士），红为太岁、岁破、大将军、三煞。时宪书附言：「惟奏書博士宜向之，餘各有所忌」。
            </Notes>
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
            {wasm && ear != null && <QiLiPanel wasm={wasm} ear={ear} jd={jd} o={qili} setO={setQili} part="diff" />}
            {yiJiSrc === 'dong' && wasm && ear != null ? (
              <>
                <DongGongYiJi wasm={wasm} ear={ear} jd={jd} />
                <Notes>
                  据《董公選擇日要覽》（嘉庆二十二年蒋云抄传本，维基文库录文，未以书影校）：每月以建除分十二条，条内分举日干支；月建按节。与协纪月表相合约五成五，神煞（煞集中宮、紅沙、黃羅紫檀、到州星等协纪所不立）与吉凶相抵之法皆不同，作并列之说，不与协纪互补；董公无宜忌之文之日不以协纪填补。「諸事」为泛言（原文「百事不宜」「次吉」等不举事项者），〔〕内为原文所设条件。只录宜忌，不录「犯之……」应验之语。逐日判定经人工逐条校订，存疑者注「未核」。
                </Notes>
              </>
            ) : (
              <>
                {props.shensha ? (
                  <div style={{ whiteSpace: 'pre-line', fontSize: 13, lineHeight: 1.7 }}>{props.shensha}</div>
                ) : (
                  <Text type="secondary">神煞宜忌需农历与岁数据，加载后显示</Text>
                )}
                <Notes>
                  据《协纪辨方书》月表加卷十铺注（反支、上朔、四離四絕、晦日、二至二分、土王用事、伏社、朔弦望、月忌、人神、長短星、氣往亡、無祿），并依时宪书体例计嫁娶周堂、交节日；「時憲書宜」为民用三十七事；「時憲書朱字」为底栏朱字应值之神（含钦天监沿用的通书之神荒蕪、枯焦、四不祥、天獄）。日遊神、鶴神、太歲出遊见卷三。
                </Notes>
              </>
            )}
            <OptionsFold g={g} defaultNote="宜忌、神煞皆依协纪">
              <Flex align="center" gap={8} wrap>
                <Text style={{ fontSize: 13 }}>宜忌依：</Text>
                <Radio.Group size="small" value={yiJiSrc} onChange={(e) => setYiJiSrc(e.target.value)}>
                  <Radio.Button value="xieji">协纪（缺省）</Radio.Button>
                  <Radio.Button value="dong">董公</Radio.Button>
                </Radio.Group>
              </Flex>
              {wasm && ear != null && <QiLiPanel wasm={wasm} ear={ear} jd={jd} o={qili} setO={setQili} part="controls" />}
            </OptionsFold>
          </Card>
        </Flex>
      )}
      {wasm && ear != null && props.monthLunar && props.monthLabel && (
        <JiaquPanel wasm={wasm} ear={ear} monthLunar={props.monthLunar} monthLabel={props.monthLabel} selJdn={Math.floor(jd + 0.5)} onPick={(n) => props.onPickJdn?.(n)} />
      )}
    </div>
  )
}
