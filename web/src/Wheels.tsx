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
