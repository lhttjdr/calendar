/**
 * 圆形方位盘（罗盘式）：奇门八宫、大六壬天地盘、太乙十六神与八宫。
 * 一律南上北下、东左西右；方位角按罗盘（自北顺时针），屏幕角 = 方位角 + 180°（北在下）。
 * 每字字头朝外，绕圆心统一旋转（与择日二十四山图同），便于罗盘模式整盘转动。
 * 多层罗盘式：每类一环，不挤一层；方位盘外加二十四山与周天度数两环。
 */
import type { ReactNode } from 'react'

/**
 * 盘面字体：王漢宗中隸書繁（GPL v2），按盘面用字子集化为 public/fonts/lishu.woff2（scripts/dev/subset_lishu.sh）。
 * 首次用盘时以 FontFace 加载；字库缺字（简体、卦符等）由浏览器回退缺省字体。
 */
export const LISHU_FAMILY = "'LunarLiShu', 'LiSu', 'STLiti', serif"
let lishuLoading = false
export function ensureLiShu() {
  if (lishuLoading || typeof FontFace === 'undefined' || typeof document === 'undefined') return
  lishuLoading = true
  new FontFace('LunarLiShu', `url(${import.meta.env.BASE_URL}fonts/lishu.woff2) format('woff2')`)
    .load()
    .then((f) => document.fonts.add(f))
    .catch(() => { lishuLoading = false })
}

const SIZE = 500
const C = SIZE / 2

/** 罗盘方位角（度，北 0 东 90）→ 屏幕角（度，自上顺时针；南上北下） */
const scr = (az: number) => az + 180
const pt = (r: number, deg: number) => {
  const a = (deg * Math.PI) / 180
  return [C + r * Math.sin(a), C - r * Math.cos(a)] as const
}

/**
 * 字头朝外的文字，沿圆弧排（罗盘刻法）：多字各占弧上一位、各依其位旋转，顺时针读；
 * 字距按字号与半径折成角度，空格计半字。非字符串内容（如数字表达式）照单一位置排。
 */
function T(p: { r: number; deg: number; children: ReactNode; size?: number; fill?: string; weight?: number }) {
  const size = p.size ?? 14
  const common = { fontSize: size, fill: p.fill ?? '#000', fontWeight: p.weight ?? 400, textAnchor: 'middle' as const, dominantBaseline: 'central' as const }
  const text = typeof p.children === 'string' ? p.children : typeof p.children === 'number' ? String(p.children) : null
  const chars = text != null ? Array.from(text) : null
  if (!chars || chars.length <= 1) {
    const [x, y] = pt(p.r, p.deg)
    return <text x={x} y={y} {...common} transform={`rotate(${p.deg} ${x} ${y})`}>{p.children}</text>
  }
  // 每字所占弧长：汉字一字宽，空格半字，ASCII（数字、°）约六成
  const w = chars.map((c) => (c === ' ' ? 0.5 : c.charCodeAt(0) < 0x2e80 ? 0.6 : 1) * size * 1.02)
  const total = w.reduce((a, b) => a + b, 0)
  const toDeg = 180 / Math.PI / p.r
  let acc = -total / 2
  return (
    <g>
      {chars.map((c, k) => {
        const d = p.deg + (acc + w[k]! / 2) * toDeg
        acc += w[k]!
        if (c === ' ') return null
        const [x, y] = pt(p.r, d)
        return <text key={k} x={x} y={y} {...common} transform={`rotate(${d} ${x} ${y})`}>{c}</text>
      })}
    </g>
  )
}

/** 扇区（屏幕角 d0—d1、半径 r0—r1）路径 */
function sector(r0: number, r1: number, d0: number, d1: number) {
  const [a, b] = pt(r1, d0), [c, d] = pt(r1, d1), [e, f] = pt(r0, d1), [g, h] = pt(r0, d0)
  const large = d1 - d0 > 180 ? 1 : 0
  return `M ${a} ${b} A ${r1} ${r1} 0 ${large} 1 ${c} ${d} L ${e} ${f} A ${r0} ${r0} 0 ${large} 0 ${g} ${h} Z`
}

function Frame({ children, label }: { children: ReactNode; label: string }) {
  ensureLiShu()
  return (
    <svg viewBox={`0 0 ${SIZE} ${SIZE}`} style={{ width: '100%', maxWidth: SIZE, height: 'auto', fontFamily: LISHU_FAMILY }} role="img" aria-label={label}>
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

/** 周天刻度环：每 1° 短刻、5° 中刻、15° 长刻 */
function DegreeRing({ r0, r1 }: { r0: number; r1: number }) {
  return (
    <g>
      <circle cx={C} cy={C} r={r1} fill="#f5ecd7" stroke="#999" />
      <circle cx={C} cy={C} r={r0} fill="none" stroke="#999" />
      {Array.from({ length: 360 }, (_, a) => {
        const len = a % 15 === 0 ? r1 - r0 : a % 5 === 0 ? (r1 - r0) * 0.6 : (r1 - r0) * 0.3
        const [x0, y0] = pt(r1, scr(a)), [x1, y1] = pt(r1 - len, scr(a))
        return <line key={a} x1={x0} y1={y0} x2={x1} y2={y1} stroke="#8c6d3f" strokeWidth={a % 15 === 0 ? 1 : 0.5} />
      })}
    </g>
  )
}

/** 二十四山环（地盘正针，子居正北，每山 15°；阳红阴黑） */
function Shan24Ring({ r0, r1, hot }: { r0: number; r1: number; hot?: Record<string, string> }) {
  return (
    <g>
      <circle cx={C} cy={C} r={r1} fill="#fffdf7" stroke="#999" />
      {SHAN24.map((s, i) => {
        const deg = scr(15 * i)
        return (
          <g key={s}>
            <path d={sector(r0, r1, deg - 7.5, deg + 7.5)} fill={hot?.[s] ?? 'none'} stroke="#ddd" />
            <T r={(r0 + r1) / 2} deg={deg} size={12} fill={SHAN_YANG.has(s) ? '#a8071a' : '#333'}>{s}</T>
          </g>
        )
      })}
    </g>
  )
}

/** 一环：底圆＋按方位分格 */
function Ring({ r0, r1, n, offset = 0, fill = '#fffdf7', dividers }: { r0: number; r1: number; n: number; offset?: number; fill?: string; dividers?: number[] }) {
  const azs = dividers ?? Array.from({ length: n }, (_, k) => offset + (360 / n) * k + 180 / n)
  return (
    <g>
      <circle cx={C} cy={C} r={r1} fill={fill} stroke="#999" />
      {azs.map((az, k) => {
        const a = scr(az)
        const [x0, y0] = pt(r0, a), [x1, y1] = pt(r1, a)
        return <line key={k} x1={x0} y1={y0} x2={x1} y2={y1} stroke="#ddd" />
      })}
    </g>
  )
}

export type QimenWheelData = {
  zhiFu: string; zhiShi: string; zhiFuGong: number; zhiShiGong: number
  gong: { di: string[]; tian: string[]; xing: string[]; men: string | null; shen: string | null }[]
}

/** 奇门八宫盘（多层罗盘式），自内而外：中五；宫卦与洛书数；地盘干；八门；九星；天盘干；八神；二十四山；周天度数。值符、值使之宫底色标出 */
export function QimenWheel({ d }: { d: QimenWheelData }) {
  const R = [40, 66, 94, 122, 150, 178, 202, 222, 234]
  const mid = (k: number) => (R[k]! + R[k + 1]!) / 2
  return (
    <Frame label="奇门遁甲八宫盘">
      <DegreeRing r0={R[7]!} r1={R[8]!} />
      <Shan24Ring r0={R[6]!} r1={R[7]!} />
      {[5, 4, 3, 2, 1, 0].map((k) => <Ring key={k} r0={R[k]!} r1={R[k + 1]!} n={8} offset={0} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} />)}
      {[1, 8, 3, 4, 9, 2, 7, 6].map((g) => {
        const deg = scr(GONG_AZ[g]!)
        const c = d.gong[g - 1]!
        const hot = g === d.zhiFuGong || g === d.zhiShiGong
        return (
          <g key={g}>
            {hot && <path d={sector(R[0]!, R[6]!, deg - 22.5, deg + 22.5)} fill="#ffe7ba" opacity={0.45} />}
            <T r={mid(0)} deg={deg} size={11} fill="#888">{`${GUA[g]}${g}`}</T>
            <T r={mid(1)} deg={deg} size={16}>{c.di.join('')}</T>
            {c.men && <T r={mid(2)} deg={deg} size={15} fill="#0958d9" weight={c.men === d.zhiShi ? 700 : 400}>{c.men}</T>}
            <T r={mid(3)} deg={deg} size={c.xing.length > 1 ? 12 : 15} weight={c.xing.includes(d.zhiFu) ? 700 : 400}>{c.xing.join(' ')}</T>
            <T r={mid(4)} deg={deg} size={16} fill="#a8071a">{c.tian.join('')}</T>
            {c.shen && <T r={mid(5)} deg={deg} size={14} fill="#531dab">{c.shen}</T>}
          </g>
        )
      })}
      <circle cx={C} cy={C} r={R[0]} fill={d.zhiFuGong === 5 || d.zhiShiGong === 5 ? '#fff7e6' : '#fff'} stroke="#999" />
      <text x={C} y={C - 8} fontSize={11} fill="#888" textAnchor="middle">中5</text>
      <text x={C} y={C + 12} fontSize={16} textAnchor="middle">{d.gong[4]!.di.join('')}</text>
    </Frame>
  )
}

const ZHI = '子丑寅卯辰巳午未申酉戌亥'
/** 十二月将（天盘十二神）名 */
const JIANG_NAME: Record<string, string> = { 子: '神后', 丑: '大吉', 寅: '功曹', 卯: '太衝', 辰: '天罡', 巳: '太乙', 午: '勝光', 未: '小吉', 申: '傳送', 酉: '從魁', 戌: '河魁', 亥: '登明' }

/** 大六壬天地盘（多层），自内而外：地盘十二支（不动）；天盘支（月将加时而转）；天盘神名；十二天将；三传；二十四山；周天度数 */
export function LiurenWheel({ pan, shi, sanChuan, label, tianJiang }: { pan: string[]; shi: string; sanChuan: string[]; label: string; tianJiang?: string[] }) {
  const R = [56, 94, 132, 156, 184, 202, 222, 234]
  const mid = (k: number) => (R[k]! + R[k + 1]!) / 2
  return (
    <Frame label="大六壬天地盘">
      <DegreeRing r0={R[6]!} r1={R[7]!} />
      <Shan24Ring r0={R[5]!} r1={R[6]!} />
      {[4, 3, 2, 1, 0].map((k) => <Ring key={k} r0={R[k]!} r1={R[k + 1]!} n={12} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} />)}
      {[...ZHI].map((z, i) => {
        const deg = scr(30 * i)
        const tian = pan[i]!
        const ci = sanChuan.indexOf(tian)
        return (
          <g key={z}>
            {z === shi && <path d={sector(R[0]!, R[1]!, deg - 15, deg + 15)} fill="#ffe7ba" />}
            <T r={mid(0)} deg={deg} size={18} fill="#666">{z}</T>
            <T r={mid(1)} deg={deg} size={24} fill={ci >= 0 ? '#a8071a' : '#000'} weight={ci >= 0 ? 700 : 500}>{tian}</T>
            <T r={mid(2)} deg={deg} size={11} fill="#8c6d3f">{JIANG_NAME[tian] ?? ''}</T>
            {tianJiang && <T r={mid(3)} deg={deg} size={13} fill={tianJiang[i] === '貴人' ? '#ad4e00' : '#531dab'} weight={tianJiang[i] === '貴人' ? 700 : 400}>{tianJiang[i]}</T>}
            {ci >= 0 && <T r={mid(4)} deg={deg} size={11} fill="#a8071a" weight={700}>{['初傳', '中傳', '末傳'][ci]}</T>}
          </g>
        )
      })}
      <circle cx={C} cy={C} r={R[0]} fill="#fff" stroke="#999" />
      <text x={C} y={C - 4} fontSize={13} textAnchor="middle">{label}</text>
      <text x={C} y={C + 14} fontSize={10} textAnchor="middle" fill="#888">底色為占時</text>
    </Frame>
  )
}

/** 太乙十六神位（自子起顺行），方位角：支每 30°，维在 45° 之位 */
const POS16_AZ: [string, number][] = [
  ['子', 0], ['丑', 30], ['艮', 45], ['寅', 60], ['卯', 90], ['辰', 120], ['巽', 135], ['巳', 150],
  ['午', 180], ['未', 210], ['坤', 225], ['申', 240], ['酉', 270], ['戌', 300], ['乾', 315], ['亥', 330],
]

/** 十六神位不等距（支 30°、维居 45°），分格线取相邻两位之中 */
const POS16_DIV = POS16_AZ.map(([, az], k) => {
  const nx = POS16_AZ[(k + 1) % 16]![1] + (k === 15 ? 360 : 0)
  return (az + nx) / 2
})

export type TaiyiWheelData = {
  gong: number; tianMuPos: string; jiShen: string; shiJiPos: string
  zhuDa: number | null; zhuCan: number | null; keDa: number | null; keCan: number | null
}

/** 太乙盘（多层），自内而外：中五；八宫（太乙所在底色）；主将（主大、主参）；客将（客大、客参）；十六神位；文昌、计神、始击各占一环；周天度数 */
export function TaiyiWheel({ d }: { d: TaiyiWheelData }) {
  const R = [40, 82, 108, 134, 162, 184, 204, 222, 234]
  const mid = (k: number) => (R[k]! + R[k + 1]!) / 2
  const at = (k: number, g: number | null, t: string, fill: string) => g != null && g !== 5 && GONG_AZ[g] != null ? <T r={mid(k)} deg={scr(GONG_AZ[g]!)} size={12} fill={fill}>{t}</T> : null
  const at16 = (k: number, p: string, t: string) => {
    const e = POS16_AZ.find(([n]) => n === p)
    return e ? <T r={mid(k)} deg={scr(e[1])} size={12} fill="#a8071a" weight={600}>{t}</T> : null
  }
  return (
    <Frame label="太乙盘">
      <DegreeRing r0={R[7]!} r1={R[8]!} />
      {[6, 5, 4, 3].map((k) => <Ring key={k} r0={R[k]!} r1={R[k + 1]!} n={16} dividers={POS16_DIV} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} />)}
      {[2, 1, 0].map((k) => <Ring key={k} r0={R[k]!} r1={R[k + 1]!} n={8} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} />)}
      {[1, 8, 3, 4, 9, 2, 7, 6].map((g) => {
        const deg = scr(GONG_AZ[g]!)
        return (
          <g key={g}>
            {d.gong === g && <path d={sector(R[0]!, R[1]!, deg - 22.5, deg + 22.5)} fill="#fff1b8" />}
            <T r={mid(0) - 8} deg={deg} size={11} fill="#888">{`${GUA[g]}${g}`}</T>
            {d.gong === g && <T r={mid(0) + 9} deg={deg} size={13} fill="#ad4e00" weight={700}>太乙</T>}
          </g>
        )
      })}
      {at(1, d.zhuDa, '主大', '#0958d9')}
      {d.zhuCan === d.zhuDa ? null : at(1, d.zhuCan, '主參', '#0958d9')}
      {at(2, d.keDa, '客大', '#531dab')}
      {d.keCan === d.keDa ? null : at(2, d.keCan, '客參', '#531dab')}
      {POS16_AZ.map(([p, az]) => <T key={p} r={mid(3)} deg={scr(az)} size={14} fill="#555">{p}</T>)}
      {at16(4, d.tianMuPos, '文昌')}
      {at16(5, d.jiShen, '計神')}
      {at16(6, d.shiJiPos, '始擊')}
      <circle cx={C} cy={C} r={R[0]} fill="#fff" stroke="#999" />
      <text x={C} y={C + 4} fontSize={12} textAnchor="middle" fill="#888">中五</text>
    </Frame>
  )
}

export type QizhengWheelData = {
  yao: { n: string; lon: number; gong: number; gongDeg: number; xiu: string | null; xiuDeg: number | null; retro: boolean }[]
  ming: number
  palaces: string[]
  /** 星命（命主、身主、宫主、庙旺、强弱、限） */
  xingMing?: XingMingData
  /** 二十八宿距星视黄经（角起），十二宫宫首黄经（地支序） */
  xiuLon?: number[] | null
  gongStart?: number[]
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
const XIU28 = ['角', '亢', '氐', '房', '心', '尾', '箕', '斗', '牛', '女', '虛', '危', '室', '壁', '奎', '婁', '胃', '昴', '畢', '觜', '參', '井', '鬼', '柳', '星', '張', '翼', '軫']
/** 盘上简称（时宪书七政躔度亦作日月水金火木土羅計孛炁） */
export const YAO_SHORT: Record<string, string> = { 太陽: '日', 太陰: '月', 水星: '水', 金星: '金', 火星: '火', 木星: '木', 土星: '土', 羅睺: '羅', 計都: '計', 月孛: '孛', 紫炁: '炁' }
/** 七政用阴阳家五色：日月朱，五星依五行，四余紫 */
const YAO_FILL: Record<string, string> = { 太陽: '#cf1322', 太陰: '#cf1322', 水星: '#1d39c4', 金星: '#8c8c8c', 火星: '#d4380d', 木星: '#389e0d', 土星: '#ad6800' }

/** 黄经 → 罗盘方位角：十二宫依地支方位（子宫玄枵居正北），黄经顺行即地支逆行；中气分宫时子宫（300°—330°）正居子位 */
const lonAz = (lon: number) => (((315 - lon) % 360) + 360) % 360
const norm = (x: number) => ((x % 360) + 360) % 360

/**
 * 七政四余盘（多层），自内而外：十二宫（支与十二次，宫界依分宫法之实际黄经）；人事十二宫；宫主（绿强灰弱）；
 * 二十八宿（距星视黄经为界）；黄道（十一曜按视黄经落点，近者错层）；周天黄经刻度（命度以▲标）。
 */
export function QizhengWheel({ d }: { d: QizhengWheelData }) {
  const R = [44, 84, 110, 130, 154, 210, 222, 234]
  const mid = (k: number) => (R[k]! + R[k + 1]!) / 2
  const starts = d.gongStart ?? Array.from({ length: 12 }, (_, z) => 30 * (((10 - z) % 12 + 12) % 12))
  /** 宫 z 的黄经区间：宫首至逆支下一宫宫首 */
  const span = (z: number) => {
    const a = starts[z]!, b = starts[(z + 11) % 12]!
    return [a, norm(b - a) || 30] as const
  }
  /** 黄经区间 [a, a+w] 的扇区（屏幕角递减） */
  const lonSector = (r0: number, r1: number, a: number, w: number) => {
    const d1 = scr(lonAz(a))
    return sector(r0, r1, d1 - w, d1)
  }
  // 诸曜按黄经排序，近者（< 9°）错入下一层，免文字相叠
  const lanes = [168, 186, 202]
  const sorted = [...d.yao].sort((a, b) => a.lon - b.lon)
  const placed: { y: (typeof d.yao)[number]; r: number }[] = []
  sorted.forEach((y) => {
    let lane = 0
    while (lane < lanes.length - 1 && placed.some((p) => p.r === lanes[lane] && Math.abs(norm(p.y.lon - y.lon + 180) - 180) < 9)) lane++
    placed.push({ y, r: lanes[lane]! })
  })
  const xm = d.xingMing
  return (
    <Frame label="七政四余盘">
      <DegreeRing r0={R[5]!} r1={R[6]!} />
      <circle cx={C} cy={C} r={R[5]} fill="#fffdf7" stroke="#999" />
      {/* 二十八宿 */}
      <circle cx={C} cy={C} r={R[4]} fill="#fbf6ea" stroke="#999" />
      {d.xiuLon && d.xiuLon.map((a, k) => {
        const w = norm(d.xiuLon![(k + 1) % 28]! - a)
        return (
          <g key={k}>
            <path d={lonSector(R[3]!, R[4]!, a, w)} fill="none" stroke="#d9cfb8" />
            <T r={mid(3)} deg={scr(lonAz(a + w / 2))} size={w < 5 ? 9 : 12} fill="#5b4a2e">{XIU28[k]!}</T>
          </g>
        )
      })}
      {/* 十二宫三环 */}
      {[2, 1, 0].map((k) => <circle key={k} cx={C} cy={C} r={R[k + 1]} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} stroke="#999" />)}
      {[...ZHI].map((z, i) => {
        const [a, w] = span(i)
        const deg = scr(lonAz(a + w / 2))
        const ming = i === d.ming
        return (
          <g key={z}>
            <path d={lonSector(R[0]!, R[3]!, a, w)} fill={ming ? '#ffe7ba' : 'none'} opacity={ming ? 0.6 : 1} stroke="#ccc" />
            <T r={mid(0) - 7} deg={deg} size={15} fill="#555">{z}</T>
            <T r={mid(0) + 10} deg={deg} size={9} fill="#999">{GONG_CI[i]}</T>
            <T r={mid(1)} deg={deg} size={11} fill={ming ? '#ad4e00' : '#531dab'} weight={ming ? 700 : 400}>{`${d.palaces[i]}${xm && xm.shenGong === i ? '·身' : ''}`}</T>
            {xm && <T r={mid(2)} deg={deg} size={11} fill={xm.strong[i] ? '#389e0d' : '#999'}>{`${YAO_SHORT[xm.gongZhu[i]!] ?? ''}主`}</T>}
          </g>
        )
      })}
      {/* 黄道诸曜 */}
      {placed.map(({ y, r }) => {
        const deg = scr(lonAz(y.lon))
        const [x0, y0] = pt(R[5]! - 2, deg), [x1, y1] = pt(r + 8, deg)
        return (
          <g key={y.n}>
            <line x1={x0} y1={y0} x2={x1} y2={y1} stroke={YAO_FILL[y.n] ?? '#531dab'} strokeWidth={0.8} />
            <T r={r} deg={deg} size={13} fill={YAO_FILL[y.n] ?? '#531dab'} weight={600}>{`${YAO_SHORT[y.n] ?? y.n}${y.retro ? '逆' : ''}`}</T>
          </g>
        )
      })}
      {xm && <T r={R[6]! + 6} deg={scr(lonAz(xm.mingLon))} size={10} fill="#ad4e00">▲</T>}
      <circle cx={C} cy={C} r={R[0]} fill="#fff" stroke="#999" />
      <text x={C} y={C - 4} fontSize={12} textAnchor="middle">命宮在{ZHI[d.ming]}</text>
      <text x={C} y={C + 12} fontSize={9} textAnchor="middle" fill="#888">{xm ? `命主${YAO_SHORT[xm.mingZhu] ?? xm.mingZhu} 身主${YAO_SHORT[xm.shenZhu] ?? xm.shenZhu}` : ''}</text>
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
