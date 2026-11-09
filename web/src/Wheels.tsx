/**
 * 圆形方位盘（罗盘式）：奇门八宫、大六壬天地盘、太乙十六神与八宫。
 * 一律南上北下、东左西右；方位角按罗盘（自北顺时针），屏幕角 = 方位角 + 180°（北在下）。
 * 每字字头朝外，绕圆心统一旋转（与择日二十四山图同），便于罗盘模式整盘转动。
 */
import type { ReactNode } from 'react'

const SIZE = 440
const C = SIZE / 2

/** 罗盘方位角（度，北 0 东 90）→ 屏幕角（度，自上顺时针；南上北下） */
const scr = (az: number) => az + 180
const pt = (r: number, deg: number) => {
  const a = (deg * Math.PI) / 180
  return [C + r * Math.sin(a), C - r * Math.cos(a)] as const
}

/** 字头朝外的文字（tangential 书写），可多行沿半径向内排 */
function T(p: { r: number; deg: number; children: ReactNode; size?: number; fill?: string; weight?: number }) {
  const [x, y] = pt(p.r, p.deg)
  return (
    <text x={x} y={y} fontSize={p.size ?? 14} fill={p.fill ?? '#000'} fontWeight={p.weight ?? 400} textAnchor="middle" dominantBaseline="central" transform={`rotate(${p.deg} ${x} ${y})`}>
      {p.children}
    </text>
  )
}

/** 扇区（屏幕角 d0—d1、半径 r0—r1）路径 */
function sector(r0: number, r1: number, d0: number, d1: number) {
  const [a, b] = pt(r1, d0), [c, d] = pt(r1, d1), [e, f] = pt(r0, d1), [g, h] = pt(r0, d0)
  const large = d1 - d0 > 180 ? 1 : 0
  return `M ${a} ${b} A ${r1} ${r1} 0 ${large} 1 ${c} ${d} L ${e} ${f} A ${r0} ${r0} 0 ${large} 0 ${g} ${h} Z`
}

function Frame({ children, label }: { children: ReactNode; label: string }) {
  return (
    <svg viewBox={`0 0 ${SIZE} ${SIZE}`} style={{ width: '100%', maxWidth: SIZE, height: 'auto' }} role="img" aria-label={label}>
      {children}
      {[['南', 180], ['西', 270], ['北', 0], ['東', 90]].map(([t, az]) => (
        <T key={t as string} r={C - 7} deg={scr(az as number)} size={12} fill={t === '北' ? '#a8071a' : '#999'}>{t}</T>
      ))}
    </svg>
  )
}

/** 洛书宫 → 罗盘方位角（坎北 0、艮东北 45、震东 90、巽东南 135、离南 180、坤西南 225、兑西 270、乾西北 315） */
const GONG_AZ: Record<number, number> = { 1: 0, 8: 45, 3: 90, 4: 135, 9: 180, 2: 225, 7: 270, 6: 315 }
const GUA: Record<number, string> = { 1: '坎', 8: '艮', 3: '震', 4: '巽', 9: '離', 2: '坤', 7: '兌', 6: '乾', 5: '中' }

export type QimenWheelData = {
  zhiFu: string; zhiShi: string; zhiFuGong: number; zhiShiGong: number
  gong: { di: string[]; tian: string[]; xing: string[]; men: string | null; shen: string | null }[]
}

/** 奇门八宫盘：每宫一扇区，由内向外：宫名、地盘干、八门、九星、天盘干、八神；中五居圆心 */
export function QimenWheel({ d }: { d: QimenWheelData }) {
  const R = [62, 92, 122, 152, 182, 205]
  return (
    <Frame label="奇门遁甲八宫盘">
      <circle cx={C} cy={C} r={R[5] + 8} fill="#fffdf7" stroke="#999" />
      {[1, 8, 3, 4, 9, 2, 7, 6].map((g) => {
        const deg = scr(GONG_AZ[g]!)
        const c = d.gong[g - 1]!
        const hot = g === d.zhiFuGong || g === d.zhiShiGong
        return (
          <g key={g}>
            <path d={sector(R[0] - 8, R[5] + 8, deg - 22.5, deg + 22.5)} fill={hot ? '#fff7e6' : 'none'} stroke="#ccc" />
            <T r={R[0] + 2} deg={deg} size={11} fill="#888">{`${GUA[g]}${g}`}</T>
            <T r={R[1]} deg={deg} size={16}>{c.di.join('')}</T>
            {c.men && <T r={R[2]} deg={deg} size={14} fill="#0958d9" weight={c.men === d.zhiShi ? 700 : 400}>{c.men}</T>}
            <T r={R[3]} deg={deg} size={14} weight={c.xing.includes(d.zhiFu) ? 700 : 400}>{c.xing.join(' ')}</T>
            <T r={R[4]} deg={deg} size={16} fill="#a8071a">{c.tian.join('')}</T>
            {c.shen && <T r={R[5]} deg={deg} size={13} fill="#531dab">{c.shen}</T>}
          </g>
        )
      })}
      <circle cx={C} cy={C} r={R[0] - 8} fill={d.zhiFuGong === 5 || d.zhiShiGong === 5 ? '#fff7e6' : '#fff'} stroke="#999" />
      <text x={C} y={C - 10} fontSize={11} fill="#888" textAnchor="middle">中5</text>
      <text x={C} y={C + 10} fontSize={16} textAnchor="middle">{d.gong[4]!.di.join('')}</text>
    </Frame>
  )
}

const ZHI = '子丑寅卯辰巳午未申酉戌亥'

/** 大六壬天地盘：内圈地盘十二支（不动），外圈天盘十二神（月将加时而转）；三传标红，占时之位底色标出 */
export function LiurenWheel({ pan, shi, sanChuan, label, tianJiang }: { pan: string[]; shi: string; sanChuan: string[]; label: string; tianJiang?: string[] }) {
  return (
    <Frame label="大六壬天地盘">
      <circle cx={C} cy={C} r={205} fill="#fffdf7" stroke="#999" />
      <circle cx={C} cy={C} r={130} fill="#fff" stroke="#999" />
      <circle cx={C} cy={C} r={70} fill="#fff" stroke="#bbb" />
      {[...ZHI].map((z, i) => {
        const deg = scr(30 * i)
        const tian = pan[i]!
        const hot = sanChuan.includes(tian)
        return (
          <g key={z}>
            <path d={sector(70, 205, deg - 15, deg + 15)} fill={z === shi ? '#fff7e6' : 'none'} stroke="#ddd" />
            <T r={100} deg={deg} size={18} fill="#666">{z}</T>
            <T r={tianJiang ? 155 : 168} deg={deg} size={24} fill={hot ? '#a8071a' : '#000'} weight={hot ? 700 : 500}>{tian}</T>
            {tianJiang && <T r={190} deg={deg} size={13} fill={tianJiang[i] === '貴人' ? '#ad4e00' : '#531dab'} weight={tianJiang[i] === '貴人' ? 700 : 400}>{tianJiang[i]}</T>}
          </g>
        )
      })}
      <text x={C} y={C - 6} fontSize={14} textAnchor="middle">{label}</text>
      <text x={C} y={C + 14} fontSize={11} textAnchor="middle" fill="#888">{tianJiang ? '外天将 · 中天盘 · 内地盘' : '外天盘 · 内地盘'}</text>
    </Frame>
  )
}

/** 太乙十六神位（自子起顺行），方位角：支每 30°，维在 45° 之位 */
const POS16_AZ: [string, number][] = [
  ['子', 0], ['丑', 30], ['艮', 45], ['寅', 60], ['卯', 90], ['辰', 120], ['巽', 135], ['巳', 150],
  ['午', 180], ['未', 210], ['坤', 225], ['申', 240], ['酉', 270], ['戌', 300], ['乾', 315], ['亥', 330],
]

export type TaiyiWheelData = {
  gong: number; tianMuPos: string; jiShen: string; shiJiPos: string
  zhuDa: number | null; zhuCan: number | null; keDa: number | null; keCan: number | null
}

/** 太乙盘：外圈十六神位（标文昌、计神、始击），内圈八宫（标太乙、主客大将参将；不入中五） */
export function TaiyiWheel({ d }: { d: TaiyiWheelData }) {
  return (
    <Frame label="太乙盘">
      <circle cx={C} cy={C} r={205} fill="#fffdf7" stroke="#999" />
      <circle cx={C} cy={C} r={135} fill="#fff" stroke="#999" />
      <circle cx={C} cy={C} r={45} fill="#fff" stroke="#bbb" />
      {POS16_AZ.map(([p, az]) => {
        const deg = scr(az)
        const m: string[] = []
        if (d.tianMuPos === p) m.push('文昌')
        if (d.jiShen === p) m.push('計神')
        if (d.shiJiPos === p) m.push('始擊')
        return (
          <g key={p}>
            <T r={150} deg={deg} size={14} fill="#666">{p}</T>
            {m.map((t, i) => <T key={t} r={172 + i * 15} deg={deg} size={12} fill="#a8071a">{t}</T>)}
          </g>
        )
      })}
      {[1, 8, 3, 4, 9, 2, 7, 6].map((g) => {
        const deg = scr(GONG_AZ[g]!)
        const m: string[] = []
        if (d.gong === g) m.push('太乙')
        if (d.zhuDa === g) m.push('主大')
        if (d.zhuCan === g) m.push('主參')
        if (d.keDa === g) m.push('客大')
        if (d.keCan === g) m.push('客參')
        return (
          <g key={g}>
            <path d={sector(45, 135, deg - 22.5, deg + 22.5)} fill={d.gong === g ? '#fff1b8' : 'none'} stroke="#ddd" />
            <T r={54} deg={deg} size={11} fill="#888">{`${GUA[g]}${g}`}</T>
            {m.map((t, i) => <T key={t} r={124 - i * 16} deg={deg} size={12} fill={t === '太乙' ? '#ad4e00' : '#333'} weight={t === '太乙' ? 700 : 400}>{t}</T>)}
          </g>
        )
      })}
      <text x={C} y={C + 4} fontSize={12} textAnchor="middle" fill="#888">中五</text>
    </Frame>
  )
}

export type QizhengWheelData = {
  yao: { n: string; gong: number; gongDeg: number; xiu: string | null; xiuDeg: number | null; retro: boolean }[]
  ming: number
  palaces: string[]
  /** 星命（命主、身主、宫主、庙旺、强弱、限） */
  xingMing?: XingMingData
}

export type XingMingData = {
  mingLon: number; mingXiu: string | null; mingXiuDeg: number | null
  mingGongZhu: string; mingDuZhu: string | null; mingZhu: string
  shenGong: number; shenGongZhu: string; shenDuZhu: string | null; shenZhu: string
  gongZhu: string[]; dignity: string[][]; strong: boolean[]; relation: string[]
  tongXian: number; daXian: { gong: number; palace: string; from: number; to: number }[]
}

/** 十二次名，下标为地支序 */
const GONG_CI = ['玄枵', '星紀', '析木', '大火', '壽星', '鶉尾', '鶉火', '鶉首', '實沈', '大梁', '降婁', '娵訾']
/** 盘上简称（时宪书七政躔度亦作日月水金火木土羅計孛炁） */
export const YAO_SHORT: Record<string, string> = { 太陽: '日', 太陰: '月', 水星: '水', 金星: '金', 火星: '火', 木星: '木', 土星: '土', 羅睺: '羅', 計都: '計', 月孛: '孛', 紫炁: '炁' }
/** 七政用阴阳家五色：日月朱，五星依五行，四余紫 */
const YAO_FILL: Record<string, string> = { 太陽: '#cf1322', 太陰: '#cf1322', 水星: '#1d39c4', 金星: '#8c8c8c', 火星: '#d4380d', 木星: '#389e0d', 土星: '#ad6800' }

/** 七政四余盘：十二宫依地支方位（同六壬地盘），内圈支与十二次，中圈人事十二宫（命宫起逆布），外圈本宫诸曜（名、入宫度，逆行注「逆」） */
export function QizhengWheel({ d }: { d: QizhengWheelData }) {
  return (
    <Frame label="七政四余盘">
      <circle cx={C} cy={C} r={205} fill="#fffdf7" stroke="#999" />
      <circle cx={C} cy={C} r={118} fill="#fff" stroke="#999" />
      <circle cx={C} cy={C} r={60} fill="#fff" stroke="#bbb" />
      {[...ZHI].map((z, i) => {
        const deg = scr(30 * i)
        const here = d.yao.filter((y) => y.gong === i).sort((a, b) => a.gongDeg - b.gongDeg)
        const ming = i === d.ming
        return (
          <g key={z}>
            <path d={sector(60, 205, deg - 15, deg + 15)} fill={ming ? '#fff7e6' : 'none'} stroke="#ddd" />
            <T r={74} deg={deg} size={16} fill="#666">{z}</T>
            <T r={92} deg={deg} size={11} fill="#999">{GONG_CI[i]}</T>
            <T r={108} deg={deg} size={12} fill={ming ? '#ad4e00' : '#531dab'} weight={ming ? 700 : 400}>{`${d.palaces[i]}${d.xingMing && d.xingMing.shenGong === i ? '·身' : ''}`}</T>
            {here.map((y, k) => {
              // 宫内多曜：沿半径排开，每曜一行（名＋度）；过多则缩字
              const step = here.length > 5 ? 15 : 17
              return (
                <T key={y.n} r={130 + k * step} deg={deg} size={here.length > 5 ? 10 : 12} fill={YAO_FILL[y.n] ?? '#531dab'}>
                  {`${YAO_SHORT[y.n] ?? y.n}${Math.floor(y.gongDeg)}°${y.retro ? '逆' : ''}`}
                </T>
              )
            })}
          </g>
        )
      })}
      <text x={C} y={C - 4} fontSize={13} textAnchor="middle">命宮在{ZHI[d.ming]}</text>
      <text x={C} y={C + 14} fontSize={10} textAnchor="middle" fill="#888">中气分宫</text>
    </Frame>
  )
}

const SHAN24 = ['子', '癸', '丑', '艮', '寅', '甲', '卯', '乙', '辰', '巽', '巳', '丙', '午', '丁', '未', '坤', '申', '庚', '酉', '辛', '戌', '乾', '亥', '壬']
const CN_NUM = '〇一二三四五六七八九'

export type XuankongWheelData = { zuo: string; xiang: string; yunPan: number[]; shanPan: number[]; xiangPan: number[]; yearPan: number[] }

/** 二十四山阴阳：甲庚丙壬、乾坤艮巽、寅申巳亥为阳，子午卯酉、乙辛丁癸、辰戌丑未为阴（与 Rust xuankong::is_yang 同） */
const SHAN_YANG = new Set(['甲', '庚', '丙', '壬', '乾', '坤', '艮', '巽', '寅', '申', '巳', '亥'])
/** 三元龙：每卦三山，左为地元、中为天元、右为人元（子午卯酉乾坤艮巽天元，壬丙甲庚辰戌丑未地元，癸丁乙辛寅申巳亥人元） */
const YUAN_LONG = (i: number) => ['天', '人', '地'][i % 3]!
const GUA_SYM: Record<number, string> = { 1: '☵', 8: '☶', 3: '☳', 4: '☴', 9: '☲', 2: '☷', 7: '☱', 6: '☰' }

/**
 * 玄空飞星宅盘（多层罗盘式），自内而外：
 * 中宫（天池）；八宫飞星（卦名、山星蓝居左、向星红居右、运星、流年紫白）；后天八卦与洛书数；三元龙；
 * 二十四山（阳红阴黑，坐蓝底、向红底）；周天 360° 刻度。坐向以一线贯天池（天心十道之意）。
 */
export function XuankongWheel({ d }: { d: XuankongWheelData }) {
  const zuoI = SHAN24.indexOf(d.zuo), xiangI = SHAN24.indexOf(d.xiang)
  const R = { tian: 40, gong: 128, gua: 148, yuan: 166, shan: 190, du: 210 }
  const ring = (r: number, fill = 'none') => <circle cx={C} cy={C} r={r} fill={fill} stroke="#999" />
  const [zx, zy] = pt(R.shan, scr(15 * zuoI)), [xx, xy] = pt(R.shan, scr(15 * xiangI))
  return (
    <Frame label="玄空飞星宅盘">
      {ring(R.du, '#f5ecd7')}
      {ring(R.shan, '#fffdf7')}
      {ring(R.yuan, '#fbf6ea')}
      {ring(R.gua, '#fffdf7')}
      {ring(R.gong, '#fffdf7')}
      {/* 周天刻度：每 1° 短刻、5° 中刻、15° 长刻（与二十四山界对齐） */}
      {Array.from({ length: 360 }, (_, a) => {
        const len = a % 15 === 0 ? 9 : a % 5 === 0 ? 6 : 3
        const [x0, y0] = pt(R.du, scr(a)), [x1, y1] = pt(R.du - len, scr(a))
        return <line key={a} x1={x0} y1={y0} x2={x1} y2={y1} stroke="#8c6d3f" strokeWidth={a % 15 === 0 ? 1 : 0.5} />
      })}
      {SHAN24.map((s, i) => {
        const deg = scr(15 * i)
        const zuo = i === zuoI, xiang = i === xiangI
        const yang = SHAN_YANG.has(s)
        return (
          <g key={s}>
            <path d={sector(R.yuan, R.shan, deg - 7.5, deg + 7.5)} fill={zuo ? '#d6e4ff' : xiang ? '#ffd8d6' : 'none'} stroke="#ccc" />
            <path d={sector(R.gua, R.yuan, deg - 7.5, deg + 7.5)} fill="none" stroke="#e3d9c3" />
            <T r={(R.yuan + R.shan) / 2} deg={deg} size={15} fill={yang ? '#a8071a' : '#222'} weight={zuo || xiang ? 700 : 500}>{s}</T>
            <T r={(R.gua + R.yuan) / 2} deg={deg} size={9} fill="#8c6d3f">{YUAN_LONG(i)}</T>
          </g>
        )
      })}
      {[1, 8, 3, 4, 9, 2, 7, 6].map((g) => {
        const deg = scr(GONG_AZ[g]!)
        return (
          <g key={g}>
            <path d={sector(R.tian, R.gong, deg - 22.5, deg + 22.5)} fill="none" stroke="#ccc" />
            <path d={sector(R.gong, R.gua, deg - 22.5, deg + 22.5)} fill="none" stroke="#bbb" />
            <T r={(R.gong + R.gua) / 2} deg={deg} size={12} fill="#5b4a2e">{`${GUA_SYM[g]} ${GUA[g]}${CN_NUM[g]}`}</T>
            <T r={56} deg={deg} size={10} fill="#999">{GUA[g]!}</T>
            <T r={82} deg={deg - 10} size={22} fill="#0958d9">{CN_NUM[d.shanPan[g - 1]!]}</T>
            <T r={82} deg={deg + 10} size={22} fill="#a8071a">{CN_NUM[d.xiangPan[g - 1]!]}</T>
            <T r={104} deg={deg} size={16}>{CN_NUM[d.yunPan[g - 1]!]}</T>
            <T r={120} deg={deg} size={9} fill="#531dab">{`年${CN_NUM[d.yearPan[g - 1]!]}`}</T>
          </g>
        )
      })}
      {/* 坐向线 */}
      <line x1={zx} y1={zy} x2={xx} y2={xy} stroke="#a8071a" strokeWidth={1} strokeDasharray="4 3" opacity={0.6} />
      {ring(R.tian, '#fff')}
      <text x={C - 11} y={C - 3} fontSize={16} textAnchor="middle" fill="#0958d9">{CN_NUM[d.shanPan[4]!]}</text>
      <text x={C + 11} y={C - 3} fontSize={16} textAnchor="middle" fill="#a8071a">{CN_NUM[d.xiangPan[4]!]}</text>
      <text x={C} y={C + 15} fontSize={13} textAnchor="middle">{CN_NUM[d.yunPan[4]!]}</text>
      <text x={C} y={C + 28} fontSize={8} textAnchor="middle" fill="#531dab">{`年${CN_NUM[d.yearPan[4]!]}`}</text>
    </Frame>
  )
}
