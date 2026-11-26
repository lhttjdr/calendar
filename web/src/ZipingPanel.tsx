/**
 * 排盘页子平八字：四柱、十神、藏干、十二长生、纳音、空亡神煞、月令取格、起运大运、小运流年、胎元命宫身宫；
 * 另附合婚面板（通书说，协纪删除、官历不用，缺省收起）。
 * 只排盘、不断吉凶；诸家之说并列为起例选项（核心 folk::ziping::ZipingOptions、folk::hehun::HeHunOptions）。
 * 出处与逐条检验见知识库 doc/folk/ziping.md、doc/folk/jiaqu.md「合婚」。
 */
import { Button, Flex, InputNumber, Radio, Table, Typography } from 'antd'
import { memo, useMemo, useState, type ReactNode } from 'react'
import { Notes, OptionsFold, useOpt, useOptGroup } from './fold'
import type { LunarBackend } from './lunar-backend-types'
import { civilToJd, jdToCivil, yearLabel } from './civilTime'

const { Text } = Typography

const G = '甲乙丙丁戊己庚辛壬癸'
const Z = '子丑寅卯辰巳午未申酉戌亥'
const PILLAR = ['年', '月', '日', '時']
const gzName = (gz: number) => G[gz % 10]! + Z[gz % 12]!

type ZipingData = {
  sizhu: number[]; names: string[]; nayin: string[]
  shiShen: (string | null)[]; cangGan: [string, string][][]; changSheng: string[]; ziZuo: string[]
  xunKong: number[]; kongAt: number[]; shensha: { n: string; base: number; at: number }[]
  geJu: { name: string; gan: string; ss: string; you: string; jianTou: string[] }
  prevJie: { name: string; jd: number }; nextJie: { name: string; jd: number }; daysSinceJie: number; siLing: string
  qiYun: { forward: boolean; jie: string; jieJd: number; days: number; age: number; years: number; months: number; dayPart: number; startJd: number }
  daYun: { gz: number; age: number; startJd: number; shiShen: string; cs: string }[]
  xiaoYun: [number, number, number][]
  taiYuan: number; mingGong: number; shenGong: number; taiXi: number; bian: number
  ganzhiYear: number; monthOk: boolean; hourOk: boolean
}

/** 起例诸项（下标即 ziping_json 之 opts 位，0 为缺省；出处见 doc/folk/ziping.md 各节） */
const ZP_OPTS: { key: string; label: string; opts: string[] }[] = [
  { key: 'dayB', label: '日界', opts: ['子正换日（三命）', '子初换日（今人多从）'] },
  { key: 'term', label: '节气', opts: ['定气', '平气（当时行用历）'] },
  { key: 'yeZi', label: '夜子时时干', opts: ['依次日（命理探源）', '依当日'] },
  { key: 'qiYun', label: '起运折算', opts: ['实历折除（三命）', '整岁（淵海零借）'] },
  { key: 'qiYunP', label: '起运计法', opts: ['时刻精确', '以时辰计'] },
  { key: 'xiaoYun', label: '小运', opts: ['男丙寅女壬申（淵海、三命）', '自时柱起', '按生年之旬'] },
  { key: 'taiYuan', label: '胎元', opts: ['月前四位（淵海）', '生前三百日（三命）'] },
  { key: 'mingM', label: '命宫生月', opts: ['依节', '依中气（今人）'] },
  { key: 'cangGan', label: '藏干表', opts: ['通行表（真詮、神峰）', '淵海歌诀', '三命人元司事'] },
  { key: 'siLing', label: '司令日数', opts: ['三命通會', '命理約言舊例'] },
  { key: 'cs', label: '十二长生', opts: ['阳顺阴逆', '阴阳同生'] },
  { key: 'tuCs', label: '土之长生', opts: ['随火（戊寅）', '随水（戊申）'] },
  { key: 'shaBase', label: '神煞所依', opts: ['年、日各起', '只依年', '只依日'] },
  { key: 'tianYi', label: '天乙贵人', opts: ['甲戊庚牛羊、六辛马虎', '甲戊牛羊、庚辛马虎'] },
  { key: 'yangRen', label: '羊刃', opts: ['惟阳干（三命卷五）', '禄前一辰（三命卷三）'] },
  { key: 'kong', label: '空亡', opts: ['依日柱', '依年柱'] },
  { key: 'jiaoYun', label: '交运时刻', opts: ['按历月（淵海、三命：月作三十日）', '岁数乘回归年'] },
  { key: 'jiaoJie', label: '交节当日生', opts: ['照常折算', '作一岁运（淵海又說）'] },
]

function fmtLocal(jdUt: number, tz: number) {
  const c = jdToCivil(jdUt + tz / 24)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${yearLabel(c.y)}-${p(c.m)}-${p(c.d)} ${p(c.h)}:${p(c.mi)}`
}

const GX_COLOR: Record<string, string> = { 木: '#237804', 火: '#a8071a', 土: '#7a4f00', 金: '#ad8b00', 水: '#0958d9' }
const ganWx = (g: string) => '木木火火土土金金水水'[G.indexOf(g)] ?? ''
const zhiWx = (z: string) => '水土木木土火火土金金土水'[Z.indexOf(z)] ?? ''
const Colored = ({ c, wx, size = 20 }: { c: string; wx: string; size?: number }) => <span style={{ color: GX_COLOR[wx], fontSize: size, fontWeight: 600 }}>{c}</span>

export const ZipingPanel = memo(function ZipingPanel(props: {
  wasm: LunarBackend; ear: string; jdUt: number
  /** 日柱时柱所依钟点与 UT 之差（时）：钟点模式为所选时区，当地平太阳时、真太阳时为其与 UT 之差 */
  clockOffset: number
  /** 所选时区与 UT 之差（时），显示时刻与合婚输入用 */
  tzOffset: number
}) {
  const g = useOptGroup('ziping')
  const [male, setMale] = useState(true)
  const opt = ZP_OPTS.map((it) => useOpt(g, it.key, 0, it.label)) // eslint-disable-line react-hooks/rules-of-hooks
  const optVals = opt.map(([v]) => v)
  const d = useMemo((): ZipingData | null => {
    if (typeof props.wasm.ziping_json !== 'function') return null
    try { return JSON.parse(props.wasm.ziping_json(props.jdUt, props.ear, props.clockOffset, male, 10, Uint8Array.from(optVals))) as ZipingData } catch { return null }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.wasm, props.ear, props.jdUt, props.clockOffset, male, optVals.join(',')])
  const tz = props.tzOffset
  const nayin = (gz: number) => (typeof props.wasm.nayin_of_ganzhi_name === 'function' ? props.wasm.nayin_of_ganzhi_name(gzName(gz)) : '')
  if (!d) return <Text type="secondary">子平排盘需新版 wasm</Text>
  const shaAt = (i: number) => d.shensha.filter((s) => s.at === i)
  const kongSet = new Set(d.kongAt)
  const rows: { key: string; label: string; cells: ReactNode[] }[] = [
    { key: 'ss', label: '十神', cells: [0, 1, 2, 3].map((i) => i === 2 ? <Text strong>{male ? '日元（乾）' : '日元（坤）'}</Text> : d.shiShen[i]) },
    { key: 'gan', label: '天干', cells: d.names.map((n) => <Colored c={n[0]!} wx={ganWx(n[0]!)} />) },
    { key: 'zhi', label: '地支', cells: d.names.map((n, i) => <>{<Colored c={n[1]!} wx={zhiWx(n[1]!)} />}{kongSet.has(i) && <sup style={{ color: '#8c8c8c', fontSize: 11 }}>空</sup>}</>) },
    { key: 'cg', label: '藏干', cells: d.cangGan.map((cg) => <div style={{ fontSize: 12, lineHeight: 1.4 }}>{cg.map(([gn, s]) => <div key={gn}><span style={{ color: GX_COLOR[ganWx(gn)] }}>{gn}</span> {s}</div>)}</div>) },
    { key: 'cs', label: '星运', cells: d.changSheng },
    { key: 'zz', label: '自坐', cells: d.ziZuo },
    { key: 'ny', label: '纳音', cells: d.nayin.map((n) => <span style={{ fontSize: 12 }}>{n}</span>) },
    { key: 'sha', label: '神煞', cells: [0, 1, 2, 3].map((i) => <div style={{ fontSize: 11, lineHeight: 1.4 }}>{shaAt(i).map((s, k) => <div key={k} title={`${PILLAR[s.base]}起`}>{s.n}<Text type="secondary" style={{ fontSize: 10 }}>{`(${PILLAR[s.base]})`}</Text></div>)}</div>) },
  ]
  const qy = d.qiYun
  return (
    <Flex gap={16} wrap align="flex-start">
      <div style={{ flex: '1 1 380px', maxWidth: 560 }}>
        <Table size="small" pagination={false} bordered rowKey="key"
          columns={[
            { title: '', dataIndex: 'label', width: 52, render: (t: string) => <Text type="secondary" style={{ fontSize: 12 }}>{t}</Text> },
            ...['年柱', '月柱', '日柱', '時柱'].map((t, i) => ({ title: t, key: t, align: 'center' as const, render: (_: unknown, r: (typeof rows)[number]) => r.cells[i] })),
          ]}
          dataSource={rows} />
        <div style={{ fontSize: 13, marginTop: 8, lineHeight: 1.8 }}>
          {`胎元${gzName(d.taiYuan)}（${nayin(d.taiYuan)}）· 命宫${gzName(d.mingGong)}（${nayin(d.mingGong)}）· 身宫${gzName(d.shenGong)} · 胎息${gzName(d.taiXi)} · 变${gzName(d.bian)}`}
          <div>{`旬空${Z[d.xunKong[0]!]}${Z[d.xunKong[1]!]}${d.kongAt.length ? `（见于${d.kongAt.map((i) => PILLAR[i]).join('、')}柱）` : ''}`}</div>
          {(!d.monthOk || !d.hourOk) && <Text type="warning" style={{ fontSize: 12 }}>{`${!d.monthOk ? '月干不合五虎遁 ' : ''}${!d.hourOk ? '时干不合五鼠遁' : ''}`}</Text>}
        </div>
        <div style={{ marginTop: 8 }}>
          <Text strong style={{ fontSize: 13 }}>大运</Text>
          <Text type="secondary" style={{ fontSize: 12 }}>{`（${qy.forward ? '阳男阴女顺行' : '阴男阳女逆行'}，数至${qy.forward ? '未来' : '过去'}之${qy.jie} ${fmtLocal(qy.jieJd, tz)}，${qy.days.toFixed(2)} 日；起运 ${qy.years} 岁 ${qy.months} 月 ${qy.dayPart} 日，交运 ${fmtLocal(qy.startJd, tz)}）`}</Text>
          <Table size="small" pagination={false} rowKey="gz" style={{ marginTop: 4 }}
            columns={[
              { title: '大运', dataIndex: 'gz', render: (z: number) => <span style={{ fontSize: 15 }}><Colored c={G[z % 10]!} wx={ganWx(G[z % 10]!)} size={15} /><Colored c={Z[z % 12]!} wx={zhiWx(Z[z % 12]!)} size={15} /></span> },
              { title: '十神', dataIndex: 'shiShen' },
              { title: '星运', dataIndex: 'cs' },
              { title: '起岁（实岁）', dataIndex: 'age', render: (a: number) => a.toFixed(1) },
              { title: '交运（约）', dataIndex: 'startJd', render: (j: number) => fmtLocal(j, tz).slice(0, 10) },
            ]}
            dataSource={d.daYun} />
        </div>
      </div>
      <div style={{ flex: '1 1 260px' }}>
        <div style={{ marginBottom: 8 }}>
          {`生于 ${fmtLocal(props.jdUt, tz)}（UTC${tz >= 0 ? '+' : ''}${tz}）`}
          <Text type="secondary" style={{ fontSize: 11, display: 'block' }}>以日历所选时刻为生时；年以立春、月以十二节换（比到交节时刻），日柱时柱依页面时柱钟点</Text>
        </div>
        <Radio.Group size="small" value={male} onChange={(e) => setMale(e.target.value)} optionType="button" options={[{ label: '乾造（男）', value: true }, { label: '坤造（女）', value: false }]} />
        <div style={{ fontSize: 13, marginTop: 8, lineHeight: 1.8 }}>
          <div>{`月令：交${d.prevJie.name}（${fmtLocal(d.prevJie.jd, tz)}）后 ${d.daysSinceJie.toFixed(2)} 日，人元司令${d.siLing}；下节${d.nextJie.name} ${fmtLocal(d.nextJie.jd, tz)}`}</div>
          <div>
            {'月令取格：'}<Text strong>{d.geJu.name}</Text>
            {`（${d.geJu.gan}${d.geJu.ss ? ` ${d.geJu.ss}` : ''}；${d.geJu.you}${d.geJu.jianTou.length ? `；兼透${d.geJu.jianTou.join('')}` : ''}）`}
          </div>
          <Text type="secondary" style={{ fontSize: 11, display: 'block' }}>取格只做《子平真詮》「以日干配月令所藏，看何者透出」一条；会局、清浊、成败救应与外格不机械判断</Text>
        </div>
        <div style={{ fontSize: 12, marginTop: 8 }}>
          <Text strong style={{ fontSize: 12 }}>小运·流年（虚岁）</Text>
          <div style={{ lineHeight: 1.7 }}>
            {d.xiaoYun.map(([s, x, l]) => <span key={s} style={{ marginRight: 10, whiteSpace: 'nowrap' }}>{`${s}岁 ${gzName(x)}`}<Text type="secondary" style={{ fontSize: 11 }}>{`/${gzName(l)}`}</Text></span>)}
          </div>
          <Text type="secondary" style={{ fontSize: 11 }}>斜线后为流年；童限未交大运用小运</Text>
        </div>
        <OptionsFold g={g} defaultNote="三命通會为主，藏干取通行表">
          {ZP_OPTS.map((it, k) => (
            <div key={it.key} style={{ marginBottom: 4 }}>
              <Text style={{ fontSize: 12, display: 'inline-block', width: 84 }}>{it.label}</Text>
              <Radio.Group size="small" value={opt[k]![0]} onChange={(e) => opt[k]![1](e.target.value)} options={it.opts.map((label, i) => ({ label, value: i }))} />
            </div>
          ))}
        </OptionsFold>
        <Notes>
          四柱：立春换年、十二节换月（定气，节气时刻由天文计算），子正换日；十神依《淵海子平》卷一「生我者為正印偏印……陽見陽為偏」；藏干缺省取今通行表（寅甲丙戊、辰戊乙癸合《子平真詮》），可改《淵海》歌诀、《三命通會》人元司事；星运为日干临四支之十二长生（阳顺阴逆，《三命通會》卷二、《子平真詮·論陰陽生死》），自坐为各柱天干临本支。大运依《淵海》「陽男陰女順……三日為一歲」，起运缺省按《三命通會》实历折除（一日四月、一时辰十日），交运时刻缺省按两书「行運之法」逐历月累加、小月进一日（即月作三十日，一岁三百六十日），此后「十週年方換一運」；可改以起运岁数乘回归年。交节当日生者，可依《淵海》「又說」作一岁运。小运缺省男一岁丙寅顺行、女一岁壬申逆行。胎元缺省月前四位，命宫依「子上起正月，逆數至生月……順數遇卯即安命」，身宫在命宫对冲。神煞起例依《三命通會》卷二、卷三、卷五，括注所依之柱；空亡缺省依日柱（今通行），可依年柱。只排盘，不断吉凶。
        </Notes>
        <HehunPanel wasm={props.wasm} ear={props.ear} jdUt={props.jdUt} tz={tz} />
      </div>
    </Flex>
  )
})

type HehunData = {
  shu: string
  nan: { sizhu: string[]; yuan: string; gong: number; gua: string }
  nv: { sizhu: string[]; yuan: string; gong: number; gua: string }
  gua: string; deng: string; nvJiaYue: string[]; panYu: string[]
}
type Birth = { y: number; m: number; d: number; h: number; mi: number }

const MONTHS = ['正', '二', '三', '四', '五', '六', '七', '八', '九', '十', '十一', '十二']

function BirthInputRow(props: { label: string; v: Birth; onChange: (b: Birth) => void; onTake: () => void }) {
  const f = (k: keyof Birth, min: number, max: number, w = 56) => (
    <InputNumber size="small" min={min} max={max} value={props.v[k]} style={{ width: w }} onChange={(x) => x != null && props.onChange({ ...props.v, [k]: Number(x) })} />
  )
  return (
    <Flex align="center" gap={4} wrap style={{ fontSize: 12 }}>
      <Text style={{ fontSize: 12, width: 24 }}>{props.label}</Text>
      {f('y', -2000, 3000, 72)}年{f('m', 1, 12)}月{f('d', 1, 31)}日{f('h', 0, 23)}时{f('mi', 0, 59)}分
      <Button size="small" type="link" onClick={props.onTake}>取所选时刻</Button>
    </Flex>
  )
}

/** 合婚：通书说（类编卷七、鳌头卷三），协纪卷首奏议「應請刪去」，官历不用；缺省收起 */
function HehunPanel(props: { wasm: LunarBackend; ear: string; jdUt: number; tz: number }) {
  const g = useOptGroup('hehun')
  const [open, setOpen] = useOpt(g, 'open', false, '启用合婚')
  const [shu, setShu] = useOpt(g, 'shu', 0, '依书')
  const [guLuan, setGuLuan] = useOpt(g, 'guLuan', 0, '孤鸞')
  const sel = (): Birth => { const c = jdToCivil(props.jdUt + props.tz / 24); return { y: c.y, m: c.m, d: c.d, h: c.h, mi: c.mi } }
  const [nan, setNan] = useState<Birth>(sel)
  const [nv, setNv] = useState<Birth>(() => ({ ...sel(), y: sel().y - 2 }))
  /** 民用历（1582-10-15 前儒略历）钟点 → UT */
  const toJd = (b: Birth) => civilToJd(b.y, b.m, b.d, b.h + b.mi / 60 - props.tz)
  const r = useMemo((): HehunData | string | null => {
    if (!open) return null
    if (typeof props.wasm.hehun_json !== 'function') return '合婚需新版 wasm'
    try { return JSON.parse(props.wasm.hehun_json(toJd(nan), toJd(nv), props.ear, props.tz, shu, guLuan)) as HehunData } catch (e) { return String(e) }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, props.wasm, props.ear, props.tz, shu, guLuan, nan, nv])
  return (
    <div style={{ marginTop: 12, borderTop: '1px dashed var(--hl-border, #ddd)', paddingTop: 8 }}>
      <Flex align="center" gap={8} wrap>
        <Text strong style={{ fontSize: 13 }}>合婚</Text>
        <Text type="secondary" style={{ fontSize: 12 }}>通书说，协纪删除、官历不用</Text>
        <Button size="small" type="link" onClick={() => setOpen(!open)}>{open ? '收起' : '展开'}</Button>
      </Flex>
      {open && (
        <div style={{ marginTop: 6 }}>
          <Flex vertical gap={4}>
            <BirthInputRow label="男" v={nan} onChange={setNan} onTake={() => setNan(sel())} />
            <BirthInputRow label="女" v={nv} onChange={setNv} onTake={() => setNv(sel())} />
            <Text type="secondary" style={{ fontSize: 11 }}>{`钟点按所选时区（UTC${props.tz >= 0 ? '+' : ''}${props.tz}）；四柱同子平（立春、节、子正）`}</Text>
            <div style={{ fontSize: 12 }}>依书：<Radio.Group size="small" value={shu} onChange={(e) => setShu(e.target.value)}
              options={[{ label: '類編卷七', value: 0 }, { label: '鰲頭卷三', value: 1 }, { label: '协纪所述之法', value: 2 }]} /></div>
            <div style={{ fontSize: 12 }}>孤鸞：<Radio.Group size="small" value={guLuan} onChange={(e) => setGuLuan(e.target.value)}
              options={[{ label: '八日（星學大成、三命卷三）', value: 0 }, { label: '四日（三命卷十二注）', value: 1 }]} /></div>
          </Flex>
          {typeof r === 'string' ? <Text type="secondary">{r}</Text> : r && (
            <div style={{ fontSize: 13, marginTop: 6, lineHeight: 1.7 }}>
              <div>{`男 ${r.nan.sizhu.join(' ')}（${r.nan.yuan}，命宫${r.nan.gong}·${r.nan.gua}）`}</div>
              <div>{`女 ${r.nv.sizhu.join(' ')}（${r.nv.yuan}，命宫${r.nv.gong}·${r.nv.gua}）`}</div>
              <div>{'命卦：'}<Text strong>{`${r.gua}（${r.deng}）`}</Text></div>
              <ul style={{ paddingLeft: 18, margin: '4px 0' }}>
                {r.panYu.map((s, i) => <li key={i} style={{ fontSize: 12 }}>{s}</li>)}
              </ul>
              <div style={{ fontSize: 12 }}>
                {'女命行嫁月：'}
                {r.nvJiaYue.map((x, k) => <span key={k} style={{ marginRight: 6, color: x === '大利月' ? '#237804' : undefined }}>{`${MONTHS[k]}月${x === '大利月' ? '大利' : x}`}</span>)}
              </div>
              <Text type="secondary" style={{ fontSize: 11, display: 'block' }}>
                {`依${r.shu}照录判语。协纪卷三十六「男女合婚大利月」述其法而逐条斥之，卷首奏议「男女合婚……應請刪去」；「协纪所述之法」为按协纪所述（命宫依《三元經》，五行月按长生十二位）推得，并非协纪所用。`}
              </Text>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
