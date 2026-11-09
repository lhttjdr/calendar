/**
 * 圆形方位盘（罗盘式）：奇门八宫、大六壬天地盘、太乙十六神与八宫。
 * 一律南上北下、东左西右；方位角按罗盘（自北顺时针），屏幕角 = 方位角 + 180°（北在下）。
 * 每字字头朝外，绕圆心统一旋转（与择日二十四山图同），便于罗盘模式整盘转动。
 * 多层罗盘式：每类一环，不挤一层；方位盘外加二十四山与周天度数两环。
 */
import { useEffect, useRef, type ReactNode } from 'react'
import { nowJdUt, sunBoardAz, sunLonRa } from './timeClock'

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

/**
 * 天盘转角（度，屏幕顺时针）：目标角变化时取最短路径累加，免于跨 0° 时倒转一周；配合 CSS transition 呈现天盘相对地盘之转动。
 */
function useSpin(target: number) {
  const ref = useRef<number | null>(null)
  if (ref.current == null) ref.current = target
  else {
    const d = ((((target - ref.current) % 360) + 540) % 360) - 180
    ref.current += d
  }
  return ref.current
}
const spinG = (a: number, ms = 900) => ({ transform: `rotate(${a}deg)`, transformOrigin: `${C}px ${C}px`, transition: `transform ${ms}ms ease-in-out` })

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

export type QimenGongMarks = {
  anGan?: string | null; kong?: boolean; ma?: boolean; menGong?: string | null; jiXing?: string[] | string | null
  ganMu?: string[] | string | null; xingMu?: string[] | string | null; menMu?: boolean
  keYing?: { t: string; d: string; n: string; text: string }[]; ge?: string[]
}
/** 奇门方位格（演义卷二月将加时诸格），系于地盘支 */
export type QimenFangGe = { n: string; zhi: string; gong: number; ji: boolean; basis: string; note: string | null }
export type QimenWheelData = {
  zhiFu: string; zhiShi: string; zhiFuGong: number; zhiShiGong: number
  gong: ({ di: string[]; tian: string[]; xing: string[]; men: string | null; shen: string | null } & QimenGongMarks)[]
  fangGe?: QimenFangGe[]
}
/** 方位格盘上简称：地私門作私門、貴人登天門作登天門 */
const FANG_SHORT: Record<string, string> = { 地私門: '私門', 貴人登天門: '登天門' }

const nonEmpty = (v: string[] | string | null | undefined) => (Array.isArray(v) ? v.length > 0 : !!v)
/** 奇门逐宫标记简称：空（旬空）、馬（驿马）、迫（门迫）、刑（六仪击刑）、干墓、星墓、門墓 */
export function qimenMarks(c: QimenGongMarks): string[] {
  const m: string[] = []
  if (c.kong) m.push('空')
  if (c.ma) m.push('馬')
  if (c.menGong === '迫') m.push('迫')
  if (nonEmpty(c.jiXing)) m.push('刑')
  if (nonEmpty(c.ganMu)) m.push('干墓')
  if (nonEmpty(c.xingMu)) m.push('星墓')
  if (c.menMu) m.push('門墓')
  return m
}

/**
 * 奇门八宫盘（多层罗盘式），自内而外：中五；宫卦与洛书数；地盘干；暗干；八门；九星；天盘干；八神；
 * 逐宫标记（空馬迫刑墓）；方位格（有则另起一环，按地盘十二支标天門地戸等，其山底色标出）；二十四山；周天度数。值符、值使之宫底色标出。
 */
/**
 * `next`：时辰内插值（示意，非原典）——下一时辰之盘与本时辰已过之比例 f（0—1）。转盘三组各由本时辰之宫数 k 向下一时辰之 k' 取最短路径按 f 转，
 * 到时辰交界恰与下一局重合；局变（节气、三元交界）时地盘亦变，仍有一跳。
 */
/** `hl`：高亮之宫（与文字联动）；`onGong`：点宫回调（1—9） */
export function QimenWheel({ d, turn, next, hl, onGong }: { d: QimenWheelData; turn?: boolean; next?: { d: QimenWheelData; f: number }; hl?: number[]; onGong?: (g: number) => void }) {
  const fang = (d.fangGe?.length ?? 0) > 0
  const R = fang ? [38, 58, 78, 98, 120, 142, 162, 180, 196, 212, 226, 236] : [40, 62, 84, 104, 128, 152, 174, 194, 212, 226, 236]
  const kShan = fang ? 9 : 8
  // 方位格按支归并：支 → 简称数组
  const fangAt: Record<string, string[]> = {}
  d.fangGe?.forEach((g) => { (fangAt[g.zhi] ??= []).push(FANG_SHORT[g.n] ?? g.n) })
  const fangHot = Object.fromEntries(Object.keys(fangAt).map((z) => [z, '#d9f7be']))
  const mid = (k: number) => (R[k]! + R[k + 1]!) / 2
  // 转盘：九星（连天盘干）、八门、八神三组各绕八宫转。各组取其转过之宫数 k（天蓬离坎、休門离坎、直符离坎），
  // 组内按「本位」画（第 i 位画的是 k 宫之后那一宫的内容），整组转 45k°，时辰交界即见其转动；飞盘不转（k=0，原位重画）
  const RING8 = [1, 8, 3, 4, 9, 2, 7, 6]
  const stepOf = (pred: (c: QimenWheelData['gong'][number]) => boolean) => {
    if (!turn) return 0
    const g = RING8.findIndex((x) => pred(d.gong[x - 1]!))
    return g < 0 ? 0 : g
  }
  const kx = stepOf((c) => c.xing.includes('天蓬')), km = stepOf((c) => c.men === '休門'), ks = stepOf((c) => c.shen === '直符' || c.shen === '值符')
  // 插值：下一时辰同组之宫数，取最短步数（−3…+4），按 f 转过
  const stepNext = (pred: (c: QimenWheelData['gong'][number]) => boolean, k: number) => {
    if (!turn || !next) return 0
    const g = RING8.findIndex((x) => pred(next.d.gong[x - 1]!))
    if (g < 0) return 0
    const dk = (((g - k) % 8) + 8) % 8
    return dk > 4 ? dk - 8 : dk // −3…+4
  }
  const f = next ? Math.min(1, Math.max(0, next.f)) : 0
  const ix = stepNext((c) => c.xing.includes('天蓬'), kx) * f
  const im = stepNext((c) => c.men === '休門', km) * f
  const is = stepNext((c) => c.shen === '直符' || c.shen === '值符', ks) * f
  const ax = useSpin(45 * (kx + ix)), am = useSpin(45 * (km + im)), as_ = useSpin(45 * (ks + is))
  const cellAt = (i: number, k: number) => d.gong[RING8[(i + k) % 8]! - 1]!
  return (
    <Frame label="奇门遁甲八宫盘">
      <DegreeRing r0={R[kShan + 1]!} r1={R[kShan + 2]!} />
      <Shan24Ring r0={R[kShan]!} r1={R[kShan + 1]!} hot={fang ? fangHot : undefined} />
      {fang && (
        <g>
          <Ring r0={R[8]!} r1={R[9]!} n={12} offset={0} fill="#f6ffed" />
          {Object.entries(fangAt).map(([z, ns]) => {
            const txt = ns.join(' ')
            return <T key={z} r={mid(8)} deg={scr(30 * ZHI.indexOf(z))} size={txt.length > 9 ? 8 : 9} fill="#237804" weight={600}>{txt}</T>
          })}
        </g>
      )}
      {[7, 6, 5, 4, 3, 2, 1, 0].map((k) => <Ring key={k} r0={R[k]!} r1={R[k + 1]!} n={8} offset={0} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} />)}
      {RING8.map((g) => {
        const deg = scr(GONG_AZ[g]!)
        const c = d.gong[g - 1]!
        const hot = g === d.zhiFuGong || g === d.zhiShiGong
        const mk = qimenMarks(c)
        return (
          <g key={g}>
            {hot && <path d={sector(R[0]!, R[7]!, deg - 22.5, deg + 22.5)} fill="#ffe7ba" opacity={0.45} />}
            {hl?.includes(g) && <path d={sector(R[0]!, R[7]!, deg - 22.5, deg + 22.5)} fill="#bae0ff" opacity={0.6} stroke="#1677ff" strokeWidth={1.5} />}
            <T r={mid(0)} deg={deg} size={11} fill="#888">{`${GUA[g]}${g}`}</T>
            <T r={mid(1)} deg={deg} size={16}>{c.di.join('')}</T>
            {c.anGan && <T r={mid(2)} deg={deg} size={12} fill="#8c8c8c">{`暗${c.anGan}`}</T>}
            {mk.length > 0 && <T r={mid(7)} deg={deg} size={mk.length > 3 ? 9 : 11} fill="#a8071a" weight={600}>{mk.join(' ')}</T>}
          </g>
        )
      })}
      {turn && (
        <>
      {/* 八门 */}
      <g style={spinG(am)}>
        {RING8.map((g, i) => {
          const c = cellAt(i, km), deg = scr(GONG_AZ[g]!)
          return c.men ? <T key={g} r={mid(3)} deg={deg} size={15} fill="#0958d9" weight={c.men === d.zhiShi ? 700 : 400}>{c.men}</T> : null
        })}
      </g>
      {/* 九星与天盘干 */}
      <g style={spinG(ax)}>
        {RING8.map((g, i) => {
          const c = cellAt(i, kx), deg = scr(GONG_AZ[g]!)
          return (
            <g key={g}>
              <T r={mid(4)} deg={deg} size={c.xing.length > 1 ? 12 : 15} weight={c.xing.includes(d.zhiFu) ? 700 : 400}>{c.xing.join(' ')}</T>
              <T r={mid(5)} deg={deg} size={16} fill="#a8071a">{c.tian.join('')}</T>
            </g>
          )
        })}
      </g>
      {/* 八神 */}
      <g style={spinG(as_)}>
        {RING8.map((g, i) => {
          const c = cellAt(i, ks), deg = scr(GONG_AZ[g]!)
          return c.shen ? <T key={g} r={mid(6)} deg={deg} size={14} fill="#531dab">{c.shen}</T> : null
        })}
      </g>
        </>
      )}
      <circle cx={C} cy={C} r={R[0]} fill={hl?.includes(5) ? '#bae0ff' : d.zhiFuGong === 5 || d.zhiShiGong === 5 ? '#fff7e6' : '#fff'} stroke={hl?.includes(5) ? '#1677ff' : '#999'} />
      <text x={C} y={C - 14} fontSize={10} fill="#888" textAnchor="middle">中5</text>
      <text x={C} y={C + 4} fontSize={15} textAnchor="middle">{d.gong[4]!.di.join('')}</text>
      <text x={C} y={C + 20} fontSize={9} textAnchor="middle" fill="#a8071a">{[d.gong[4]!.anGan ? `暗${d.gong[4]!.anGan}` : '', ...qimenMarks(d.gong[4]!)].filter(Boolean).join(' ')}</text>
      {/* 飞盘：星、门、神、天盘干按名追踪所在宫，换时沿洛书路径飞行（经五穿中心） */}
      {!turn && (() => {
        const at = (key: (c: QimenWheelData['gong'][number]) => string[]) =>
          [1, 2, 3, 4, 5, 6, 7, 8, 9].flatMap((g) => key(d.gong[g - 1]!).map((n) => [n, g] as const))
        const P = (k: number, cy: number) => (g: number): [number, number, number] => (g === 5 ? [C, C + cy, 0] : polar(mid(k), scr(GONG_AZ[g]!)))
        return (
          <g>
            {at((c) => (c.men ? [c.men] : [])).map(([n, g]) => <Fly key={n} g={g} pos={P(3, -26)} size={g === 5 ? 9 : 15} fill="#0958d9">{n}</Fly>)}
            {at((c) => c.xing).map(([n, g]) => <Fly key={n} g={g} pos={P(4, -34)} size={g === 5 ? 9 : 14} fill="#000">{n}</Fly>)}
            {(() => { const seen: Record<string, number> = {}; return at((c) => c.tian).map(([n, g]) => { const k = `${n}#${(seen[n] = (seen[n] ?? 0) + 1)}`; return <Fly key={k} g={g} pos={P(5, 30)} size={g === 5 ? 10 : 16} fill="#a8071a">{n}</Fly> }) })()}
            {at((c) => (c.shen ? [c.shen] : [])).map(([n, g]) => <Fly key={n} g={g} pos={P(6, 40)} size={g === 5 ? 9 : 14} fill="#531dab">{n}</Fly>)}
          </g>
        )
      })()}
      {/* 点宫：透明点击层（在最上） */}
      {onGong && (
        <g style={{ cursor: 'pointer' }}>
          {RING8.map((g) => {
            const deg = scr(GONG_AZ[g]!)
            return <path key={g} d={sector(R[0]!, R[7]!, deg - 22.5, deg + 22.5)} fill="transparent" pointerEvents="all" onClick={() => onGong(g)}><title>{`${GUA[g]}${g}宫`}</title></path>
          })}
          <circle cx={C} cy={C} r={R[0]} fill="transparent" pointerEvents="all" onClick={() => onGong(5)}><title>中5宫</title></circle>
        </g>
      )}
    </Frame>
  )
}

const ZHI = '子丑寅卯辰巳午未申酉戌亥'
/** 十二月将（天盘十二神）名 */
const JIANG_NAME: Record<string, string> = { 子: '神后', 丑: '大吉', 寅: '功曹', 卯: '太衝', 辰: '天罡', 巳: '太乙', 午: '勝光', 未: '小吉', 申: '傳送', 酉: '從魁', 戌: '河魁', 亥: '登明' }

/** 大六壬天地盘（多层），自内而外：地盘十二支（不动）；天盘支（月将加时而转）；天盘神名；十二天将；三传；二十四山；周天度数 */
export type LiurenExtra = {
  /** 遁干：按地盘子至亥，其上天盘神之遁干 */
  dunGan?: (string | null)[]
  /** 天盘神上之标记（支 → 简称数组）：三传、旬空、驿马、丁马、天马、日德、日禄、羊刃、日鬼 */
  tianMarks?: Record<string, string[]>
  /** 地盘位上之标记（本命、行年） */
  diMarks?: Record<string, string[]>
  /** 中宫第二行（课体） */
  center2?: string
}

/**
 * 大六壬天地盘（多层），自内而外：地盘十二支（不动；占时底色，本命行年标注）；天盘支（三传红）；遁干；天盘神名；十二天将；
 * 三传与神煞（空、馬、丁、德、祿、刃、鬼）；二十四山；周天度数。天盘诸环为一组，随月将加时相对地盘转。
 */
/**
 * `cont`：天盘连续转（月将加时之连续形）——月将（太阳所在之次）中太阳的实际位置对准太阳时角方位：
 * 天盘本位上太阳之方位 = 30°×月将支序 −（太阳黄经在本次中之偏，次中点为 0；黄经增向即支逆向），转角 = 太阳盘上方位 − 之。
 * 时辰正中、太阳居次中时与按时辰之「月将加时」重合；实时则每 250 ms 以当前时刻重算。月将以中气分次（缺省换将法）。
 */
/** `hl`：高亮之天盘支（与文字中三传、四课联动，随天盘转） */
export function LiurenWheel({ pan, shi, sanChuan, label, tianJiang, extra, cont, hl }: { pan: string[]; shi: string; sanChuan: string[]; label: string; tianJiang?: string[]; extra?: LiurenExtra; cont?: { jdUt: number; lonDeg: number; live: boolean }; hl?: string | null }) {
  const R = [50, 84, 118, 136, 156, 178, 200, 216, 228]
  const mid = (k: number) => (R[k]! + R[k + 1]!) / 2
  // 天盘 = 地盘转 k 位（地盘子位上所临之天盘支序为 k）；天盘诸环按本位画，整组转 −30k°（月将加时），时辰交界时可见其转动
  const k = Math.max(0, ZHI.indexOf(pan[0] ?? '子'))
  const spin = useSpin(-30 * k)
  const jiangIdx = ZHI.indexOf(pan[ZHI.indexOf(shi)] ?? '子') // 月将：天盘临占时之支
  const contAngle = (jd: number) => {
    const lam = sunLonRa(jd).lon
    const off = ((((lam % 30) + 30) % 30) - 15) // 太阳在本次中之偏（次界在中气，即黄经 30° 之倍）
    return sunBoardAz(jd, cont!.lonDeg) - (30 * jiangIdx - off)
  }
  const tianG = useRef<SVGGElement | null>(null)
  const contStatic = useSpin(cont && !cont.live ? contAngle(cont.jdUt) : -30 * k)
  useEffect(() => {
    if (!cont?.live || !tianG.current) return
    const el = tianG.current
    let last = Number.NaN
    const draw = () => {
      let a = contAngle(nowJdUt())
      if (!Number.isNaN(last)) a = last + (((((a - last) % 360) + 540) % 360) - 180) // 连续，不跨周倒转
      last = a
      el.style.transition = 'none'
      el.style.transform = `rotate(${a.toFixed(3)}deg)`
    }
    draw()
    const id = window.setInterval(draw, 250)
    return () => window.clearInterval(id)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cont?.live, cont?.lonDeg, jiangIdx])
  return (
    <Frame label="大六壬天地盘">
      <DegreeRing r0={R[7]!} r1={R[8]!} />
      <Shan24Ring r0={R[6]!} r1={R[7]!} />
      {[5, 4, 3, 2, 1, 0].map((n) => <Ring key={n} r0={R[n]!} r1={R[n + 1]!} n={12} fill={n % 2 ? '#fbf6ea' : '#fffdf7'} />)}
      {/* 地盘（不动） */}
      {[...ZHI].map((z, i) => {
        const deg = scr(30 * i)
        const dm = extra?.diMarks?.[z]
        return (
          <g key={z}>
            {z === shi && <path d={sector(R[0]!, R[1]!, deg - 15, deg + 15)} fill="#ffe7ba" />}
            <T r={mid(0) + (dm ? 5 : 0)} deg={deg} size={17} fill="#666">{z}</T>
            {dm && <T r={R[0]! + 7} deg={deg} size={9} fill="#ad4e00">{dm.join(' ')}</T>}
          </g>
        )
      })}
      {/* 天盘：天盘支、遁干、神名、天将、三传与神煞 */}
      <g ref={tianG} style={cont?.live ? { transformOrigin: `${C}px ${C}px` } : spinG(cont ? contStatic : spin)}>
        {[...ZHI].map((tian, j) => {
          const i = (j - k + 12) % 12 // 此天盘支所临之地盘位
          const deg = scr(30 * j)
          const ci = sanChuan.indexOf(tian)
          const tj = tianJiang?.[i]
          const dg = extra?.dunGan?.[i]
          const marks = [...(ci >= 0 ? [['初', '中', '末'][ci]!] : []), ...(extra?.tianMarks?.[tian] ?? [])]
          return (
            <g key={tian}>
              {tian === hl && <path d={sector(R[1]!, R[6]!, deg - 15, deg + 15)} fill="#bae0ff" opacity={0.7} stroke="#1677ff" strokeWidth={1.5} />}
              <T r={mid(1)} deg={deg} size={24} fill={ci >= 0 ? '#a8071a' : '#000'} weight={ci >= 0 ? 700 : 500}>{tian}</T>
              {dg && <T r={mid(2)} deg={deg} size={12} fill="#0958d9">{dg}</T>}
              <T r={mid(3)} deg={deg} size={11} fill="#8c6d3f">{JIANG_NAME[tian] ?? ''}</T>
              {tj && <T r={mid(4)} deg={deg} size={13} fill={tj === '貴人' ? '#ad4e00' : '#531dab'} weight={tj === '貴人' ? 700 : 400}>{tj}</T>}
              {marks.length > 0 && <T r={mid(5)} deg={deg} size={marks.length > 3 ? 9 : 11} fill="#a8071a" weight={600}>{marks.join('')}</T>}
            </g>
          )
        })}
      </g>
      <circle cx={C} cy={C} r={R[0]} fill="#fff" stroke="#999" />
      <text x={C} y={C - 6} fontSize={12} textAnchor="middle">{label}</text>
      <text x={C} y={C + 10} fontSize={10} textAnchor="middle" fill="#ad4e00">{extra?.center2 ?? ''}</text>
      <text x={C} y={C + 24} fontSize={8} textAnchor="middle" fill="#888">底色為占時</text>
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

type TyGong = { gong: number; name?: string; ruNian?: number } | null
type TyZhi = { zhi: string; ruNian?: number } | null
export type TaiyiWheelData = {
  gong: number; tianMuPos: string; jiShen: string; shiJiPos: string
  zhuDa: number | null; zhuCan: number | null; keDa: number | null; keCan: number | null
  zhuSuan?: number; keSuan?: number
  /** 诸神（taiyi::zhushen）：宫 1—9，四神诸神另有 10 絳宮、11 明堂、12 玉堂 */
  siShen?: TyGong; tianYi?: TyGong; diYi?: TyGong; zhiFu?: TyGong; wuFu?: TyGong; daYou?: TyGong; xiaoYou?: number | null
  junJi?: TyZhi; chenJi?: TyZhi; minJi?: TyZhi
  dingJiPos?: string | null; dingJiDa?: number | null; dingJiCan?: number | null; daYouTianMuPos?: string | null
  /** 八门：下标为宫数（0、5 为 null），值为门名（taiyi::bamen） */
  men?: (string | null)[]
  /** 十精（taiyi::shijing）：十六神类、十二辰类给 pos，宫类给 gong */
  shiJing?: { n: string; pos?: string; gong?: number; shu?: number; label: string }[]
}

/** 十六神名（与 POS16_AZ 同序，自子起；Rust taiyi::SHEN16） */
const SHEN16 = ['地主', '陽德', '和德', '呂申', '高叢', '太陽', '大炅', '大神', '大威', '天道', '大武', '武德', '太簇', '陰主', '陰德', '大義']

/**
 * 太乙盘（多层），自内而外：中五（主客筭及居中五、絳宮、明堂、玉堂之神）；八宫（太乙所在底色）；主客与定计大将参将；
 * 宫中诸神（四神、天乙、地乙、直符、五福、大游、小游）；八门；十六神位；十六神名；文昌、始击、计神、定计目、大游天目；
 * 君基、臣基、民基与十精之天皇、帝符、天时；周天度数。同格者沿弧并列。宫类十精（太尊、飞鸟、五行、三风等）列于面板。
 */
/**
 * `next`：时计插值（示意，非原典）——下一时辰之局与本时辰已过比例 f。只有文昌（十六神逐局进一位，遇重留则停）、计神（逆行一辰）
 * 两者逐局各移一位，可沿外环向下一局之位滑去；太乙按洛书数移宫（坎一→坤二→震三跨盘）、始击与主客将由算数得，皆不插值。
 * 太乙移宫时沿洛书路径飞行（经五穿中心）。
 */
/** `hl`、`onGong`：与文字联动之宫（1—9） */
export function TaiyiWheel({ d, next, hl, onGong }: { d: TaiyiWheelData; next?: { d: TaiyiWheelData; f: number }; hl?: number | null; onGong?: (g: number) => void }) {
  const R = [46, 76, 98, 120, 140, 160, 178, 198, 216, 226, 236]
  const mid = (k: number) => (R[k]! + R[k + 1]!) / 2
  const byGong = (pairs: [number | null | undefined, string][]) => {
    const m = new Map<number, string[]>()
    pairs.forEach(([g, t]) => { if (g != null) m.set(g, [...(m.get(g) ?? []), t]) })
    return m
  }
  const jiang = byGong([
    [d.zhuDa, '主大'], [d.zhuCan !== d.zhuDa ? d.zhuCan : null, '主參'], [d.keDa, '客大'], [d.keCan !== d.keDa ? d.keCan : null, '客參'],
    [d.dingJiDa, '定大'], [d.dingJiCan !== d.dingJiDa ? d.dingJiCan : null, '定參'],
  ])
  const shen = byGong([
    [d.siShen?.gong, '四神'], [d.tianYi?.gong, '天乙'], [d.diYi?.gong, '地乙'], [d.zhiFu?.gong, '直符'], [d.wuFu?.gong, '五福'], [d.daYou?.gong, '大游'], [d.xiaoYou, '小游'],
  ])
  const byPos = (pairs: [string | null | undefined, string][]) => {
    const m = new Map<string, string[]>()
    pairs.forEach(([p, t]) => { if (p) m.set(p, [...(m.get(p) ?? []), t]) })
    return m
  }
  // 插值时文昌、计神另画（沿环滑行），不入并列标注
  const mu = byPos([[next ? null : d.tianMuPos, '文昌'], [d.shiJiPos, '始擊'], [next ? null : d.jiShen, '計神'], [d.dingJiPos, '定目'], [d.daYouTianMuPos, '游目']])
  const az16 = (p: string | null | undefined) => POS16_AZ.find(([n]) => n === p)?.[1]
  const slide = (p: string | null | undefined, q: string | null | undefined) => {
    const a0 = az16(p)
    if (a0 == null) return null
    const a1 = az16(q) ?? a0
    const dd = ((((a1 - a0) % 360) + 540) % 360) - 180
    return a0 + dd * Math.min(1, Math.max(0, next!.f))
  }
  const ji = byPos([[d.junJi?.zhi, '君基'], [d.chenJi?.zhi, '臣基'], [d.minJi?.zhi, '民基'],
    ...(d.shiJing ?? []).filter((x) => x.pos).map((x): [string, string] => [x.pos!, x.n])])
  // 居中五及絳宮（10）、明堂（11）、玉堂（12）者，列于中宫
  const center = [...jiang.entries(), ...shen.entries()].filter(([g]) => g === 5 || g > 9)
    .flatMap(([g, ts]) => ts.map((t) => `${t}${g === 5 ? '' : ['絳宮', '明堂', '玉堂'][g - 10]}`))
  const ring = (k: number, n16: boolean) =>
    n16 ? <Ring key={k} r0={R[k]!} r1={R[k + 1]!} n={16} dividers={POS16_DIV} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} />
      : <Ring key={k} r0={R[k]!} r1={R[k + 1]!} n={8} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} />
  const lab = (ts: string[]) => ts.join(' ')
  const sz = (ts: string[], base: number) => (ts.length > 2 ? base - 3 : ts.length > 1 ? base - 1 : base)
  return (
    <Frame label="太乙盘">
      <DegreeRing r0={R[9]!} r1={R[10]!} />
      {[8, 7, 6, 5, 4].map((k) => ring(k, true))}
      {[3, 2, 1, 0].map((k) => ring(k, false))}
      {[1, 8, 3, 4, 9, 2, 7, 6].map((g) => {
        const deg = scr(GONG_AZ[g]!)
        const js = jiang.get(g), ss = shen.get(g)
        return (
          <g key={g}>
            {d.gong === g && <path d={sector(R[0]!, R[1]!, deg - 22.5, deg + 22.5)} fill="#fff1b8" />}
            {hl === g && <path d={sector(R[0]!, R[4]!, deg - 22.5, deg + 22.5)} fill="#bae0ff" opacity={0.55} stroke="#1677ff" strokeWidth={1.5} />}
            <T r={mid(0) - 7} deg={deg} size={11} fill="#888">{`${GUA[g]}${g}`}</T>
            {js && <T r={mid(1)} deg={deg} size={sz(js, 12)} fill="#0958d9">{lab(js)}</T>}
            {ss && <T r={mid(2)} deg={deg} size={sz(ss, 12)} fill="#531dab">{lab(ss)}</T>}
            {d.men?.[g] && <T r={mid(3)} deg={deg} size={12} fill={['開門', '休門', '生門'].includes(d.men[g]!) ? '#237804' : '#8c8c8c'}>{d.men[g]!}</T>}
          </g>
        )
      })}
      {POS16_AZ.map(([p, az], k) => {
        const m = mu.get(p), b = ji.get(p)
        return (
          <g key={p}>
            <T r={mid(4)} deg={scr(az)} size={14} fill="#555">{p}</T>
            <T r={mid(5)} deg={scr(az)} size={10} fill="#8c6d3f">{SHEN16[k]!}</T>
            {m && <T r={mid(6)} deg={scr(az)} size={sz(m, 11)} fill="#a8071a" weight={600}>{lab(m)}</T>}
            {b && <T r={mid(7)} deg={scr(az)} size={sz(b, 11)} fill="#389e0d">{lab(b)}</T>}
          </g>
        )
      })}
      {/* 太乙：移宫沿洛书路径飞行 */}
      <Fly g={d.gong} pos={(g) => (g === 5 ? [C, C, 0] : polar(mid(0) + 8, scr(GONG_AZ[g]!)))} size={13} fill="#ad4e00">太乙</Fly>
      {next && (() => {
        const w = slide(d.tianMuPos, next.d.tianMuPos), j = slide(d.jiShen, next.d.jiShen)
        return (
          <g>
            {w != null && <T r={mid(6) - 5} deg={scr(w)} size={11} fill="#a8071a" weight={700}>文昌</T>}
            {j != null && <T r={mid(6) + 6} deg={scr(j)} size={11} fill="#a8071a" weight={700}>計神</T>}
          </g>
        )
      })()}
      <circle cx={C} cy={C} r={R[0]} fill="#fff" stroke="#999" />
      <text x={C} y={C - 18} fontSize={10} textAnchor="middle" fill="#888">中五</text>
      {d.zhuSuan != null && <text x={C} y={C - 5} fontSize={10} textAnchor="middle" fill="#0958d9">{`主筭${d.zhuSuan} 客筭${d.keSuan}`}</text>}
      {center.slice(0, 3).map((t, i) => <text key={t} x={C} y={C + 9 + i * 11} fontSize={9} textAnchor="middle" fill="#531dab">{t}</text>)}
      {onGong && (
        <g style={{ cursor: 'pointer' }}>
          {[1, 8, 3, 4, 9, 2, 7, 6].map((g) => {
            const deg = scr(GONG_AZ[g]!)
            return <path key={g} d={sector(R[0]!, R[4]!, deg - 22.5, deg + 22.5)} fill="transparent" pointerEvents="all" onClick={() => onGong(g)}><title>{`${GUA[g]}${g}宫`}</title></path>
          })}
        </g>
      )}
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
  /** 神煞：宫位类 yao 为 null，化曜类 yao 为曜名；gong 地支序 */
  shenSha?: { n: string; by: string; yao: string | null; gong: number | null; src: string }[]
  zhuLuo?: { yao: string; from: number; to: number; start: number }[]
  nianGz?: number; zhou?: boolean
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
 * 七政四余盘（多层），自内而外：十二宫（支与十二次，宫界依分宫法之实际黄经）；人事十二宫；宫主（绿强灰弱）；宫位神煞（至多四名，全表见面板）；
 * 二十八宿（距星视黄经为界）；黄道（十一曜按视黄经落点，近者错层）；周天黄经刻度（命度以▲标）。
 */
export function QizhengWheel({ d, turn }: { d: QizhengWheelData; turn?: { jdUt: number; lonDeg: number; live?: boolean } }) {
  // 环：0 支与次、1 人事宫、2 宫主、3 神煞（宫位类）、4 二十八宿、5 黄道、6 刻度
  const R = [44, 80, 104, 122, 150, 172, 214, 226]
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
  const lanes = [184, 197, 208]
  const sorted = [...d.yao].sort((a, b) => a.lon - b.lon)
  const placed: { y: (typeof d.yao)[number]; r: number }[] = []
  sorted.forEach((y) => {
    let lane = 0
    while (lane < lanes.length - 1 && placed.some((p) => p.r === lanes[lane] && Math.abs(norm(p.y.lon - y.lon + 180) - 180) < 9)) lane++
    placed.push({ y, r: lanes[lane]! })
  })
  const xm = d.xingMing
  // 宫位类神煞按宫（地支序）归并；化曜类见面板
  const ss: string[][] = Array.from({ length: 12 }, () => [])
  d.shenSha?.forEach((x) => { if (x.yao == null && x.gong != null) ss[x.gong]!.push(x.n) })
  // 天盘随时转（选项）：太阳时角 H = 地方恒星时 − 太阳赤经；太阳在盘上方位 = 180° + H（午正居南、卯初居东），
  // 天盘整组转 Δ = 太阳方位 − 本位方位（315° − 太阳黄经），即「太阳宫加时」之连续形；地盘十二支不动，卯上之宫即命宫（卯上起命法）。
  let delta = 0
  const sun = d.yao.find((y) => y.n === '太陽')
  if (turn && sun) {
    const days = turn.jdUt - 2451545.0
    const gmst = 280.46061837 + 360.98564736629 * days
    const eps = ((23.4392911 - 0.0130042 * (days / 36525)) * Math.PI) / 180
    const lam = (sun.lon * Math.PI) / 180
    const ra = (Math.atan2(Math.sin(lam) * Math.cos(eps), Math.cos(lam)) * 180) / Math.PI
    const ha = gmst + turn.lonDeg - ra
    delta = norm(180 + ha - (315 - sun.lon))
  }
  const spin = useSpin(delta)
  // 实时：天盘随时刻连续转（每 250 ms 以当前时刻重算太阳时角，直接写组 transform；一日一周，与时针同步）
  const skyG = useRef<SVGGElement | null>(null)
  const sunLonData = sun?.lon
  const liveTurn = !!(turn && turn.live && sunLonData != null)
  useEffect(() => {
    if (!liveTurn || !skyG.current || sunLonData == null) return
    const el = skyG.current
    const draw = () => {
      const dlt = norm(sunBoardAz(nowJdUt(), turn!.lonDeg) - (315 - sunLonData))
      el.style.transition = 'none'
      el.style.transform = `rotate(${dlt.toFixed(3)}deg)`
    }
    draw()
    const id = window.setInterval(draw, 250)
    return () => window.clearInterval(id)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [liveTurn, sunLonData, turn?.lonDeg])
  return (
    <Frame label="七政四余盘">
      <DegreeRing r0={R[6]!} r1={R[7]!} />
      <circle cx={C} cy={C} r={R[6]} fill="#fffdf7" stroke="#999" />
      <g ref={skyG} style={liveTurn ? { transformOrigin: `${C}px ${C}px` } : spinG(spin, 600)}>
      {/* 二十八宿 */}
      <circle cx={C} cy={C} r={R[5]} fill="#fbf6ea" stroke="#999" />
      {d.xiuLon && d.xiuLon.map((a, k) => {
        const w = norm(d.xiuLon![(k + 1) % 28]! - a)
        return (
          <g key={k}>
            <path d={lonSector(R[4]!, R[5]!, a, w)} fill="none" stroke="#d9cfb8" />
            <T r={mid(4)} deg={scr(lonAz(a + w / 2))} size={w < 5 ? 9 : 12} fill="#5b4a2e">{XIU28[k]!}</T>
          </g>
        )
      })}
      {/* 十二宫三环 */}
      {[3, 2, 1, 0].map((k) => <circle key={k} cx={C} cy={C} r={R[k + 1]} fill={k % 2 ? '#fbf6ea' : '#fffdf7'} stroke="#999" />)}
      {[...ZHI].map((z, i) => {
        const [a, w] = span(i)
        const deg = scr(lonAz(a + w / 2))
        const ming = i === d.ming
        return (
          <g key={z}>
            <path d={lonSector(R[0]!, R[4]!, a, w)} fill={ming ? '#ffe7ba' : 'none'} opacity={ming ? 0.6 : 1} stroke="#ccc" />
            <T r={mid(0) - 7} deg={deg} size={15} fill="#555">{z}</T>
            <T r={mid(0) + 10} deg={deg} size={9} fill="#999">{GONG_CI[i]}</T>
            <T r={mid(1)} deg={deg} size={11} fill={ming ? '#ad4e00' : '#531dab'} weight={ming ? 700 : 400}>{`${d.palaces[i]}${xm && xm.shenGong === i ? '·身' : ''}`}</T>
            {xm && <T r={mid(2)} deg={deg} size={11} fill={xm.strong[i] ? '#389e0d' : '#999'}>{`${YAO_SHORT[xm.gongZhu[i]!] ?? ''}主`}</T>}
            {ss[i] && ss[i]!.length > 0 && <T r={mid(3)} deg={deg} size={ss[i]!.length > 3 ? 8 : 9} fill="#874d00">{ss[i]!.slice(0, 4).join(' ')}</T>}
          </g>
        )
      })}
      {/* 黄道诸曜 */}
      {placed.map(({ y, r }) => {
        const deg = scr(lonAz(y.lon))
        const [x0, y0] = pt(R[6]! - 2, deg), [x1, y1] = pt(r + 8, deg)
        return (
          <g key={y.n}>
            <line x1={x0} y1={y0} x2={x1} y2={y1} stroke={YAO_FILL[y.n] ?? '#531dab'} strokeWidth={0.8} />
            <T r={r} deg={deg} size={13} fill={YAO_FILL[y.n] ?? '#531dab'} weight={600}>{`${YAO_SHORT[y.n] ?? y.n}${y.retro ? '逆' : ''}`}</T>
          </g>
        )
      })}
      {xm && <T r={R[7]! + 6} deg={scr(lonAz(xm.mingLon))} size={10} fill="#ad4e00">▲</T>}
      </g>
      <circle cx={C} cy={C} r={R[0]} fill="#fff" stroke="#999" />
      {turn ? (
        <g>
          {/* 地盘十二支（方位，不动）；卯位底色：卯上起命 */}
          {[...ZHI].map((z, i) => (
            <g key={z}>
              {z === '卯' && <path d={sector(22, R[0]!, scr(30 * i) - 15, scr(30 * i) + 15)} fill="#ffe7ba" />}
              <T r={33} deg={scr(30 * i)} size={10} fill="#333">{z}</T>
            </g>
          ))}
          <circle cx={C} cy={C} r={22} fill="#fff" stroke="#bbb" />
          <text x={C} y={C + 4} fontSize={9} textAnchor="middle" fill="#888">地盤</text>
        </g>
      ) : (
        <>
          <text x={C} y={C - 4} fontSize={12} textAnchor="middle">命宮在{ZHI[d.ming]}</text>
          <text x={C} y={C + 12} fontSize={9} textAnchor="middle" fill="#888">{xm ? `命主${YAO_SHORT[xm.mingZhu] ?? xm.mingZhu} 身主${YAO_SHORT[xm.shenZhu] ?? xm.shenZhu}` : ''}</text>
        </>
      )}
    </Frame>
  )
}

const SHAN24 = ['子', '癸', '丑', '艮', '寅', '甲', '卯', '乙', '辰', '巽', '巳', '丙', '午', '丁', '未', '坤', '申', '庚', '酉', '辛', '戌', '乾', '亥', '壬']
const CN_NUM = '〇一二三四五六七八九'

type XkJie = { shan: string; center: number; shanJie: [number, number]; guaJie: [number, number]; duan: { from: number; to: number; kind: string; jian: string | null }[] }
export type XuankongWheelData = {
  zuo: string; xiang: string; yunPan: number[]; shanPan: number[]; xiangPan: number[]; yearPan: number[]
  pian?: number | null; zhengShen?: number[]; lingShen?: number[]; gongMarks?: { n: string; note: string }[][]
  jieXian?: { zuo: XkJie; xiang: XkJie }
}
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
/** 极坐标点（屏幕角）及该处字的旋转角 */
const polar = (r: number, deg: number): [number, number, number] => { const [x, y] = pt(r, deg); return [x, y, deg] }
/** 洛书飞行之序：一二三四五六七八九，九后复一（五居中） */
const LUO_NEXT = (g: number, dir: 1 | -1) => ((g - 1 + dir + 9) % 9) + 1

/**
 * 飞宫动画：一星从旧宫飞到新宫，沿洛书顺序逐宫经过（经五则穿过中心），取步数少的一向；
 * 每段约 220 ms，整段至多约 1.5 s。位置与字的旋转角逐帧插值，直接写 transform（不经 React 重绘）。
 */
function Fly({ g, pos, children, size, fill }: { g: number; pos: (g: number) => [number, number, number]; children: ReactNode; size: number; fill: string }) {
  const ref = useRef<SVGGElement | null>(null)
  const prev = useRef<number | null>(null)
  const posRef = useRef(pos)
  posRef.current = pos
  useEffect(() => {
    const el = ref.current
    if (!el) return
    const put = ([x, y, r]: [number, number, number]) => el.setAttribute('transform', `translate(${x.toFixed(2)} ${y.toFixed(2)}) rotate(${r.toFixed(2)})`)
    const from = prev.current
    prev.current = g
    if (from == null || from === g) { put(posRef.current(g)); return }
    const fwd = (((g - from) % 9) + 9) % 9
    const dir: 1 | -1 = fwd <= 4 ? 1 : -1
    const steps = dir === 1 ? fwd : 9 - fwd
    const way: number[] = [from]
    for (let k = 0, cur = from; k < steps; k++) { cur = LUO_NEXT(cur, dir); way.push(cur) }
    const pts = way.map((w) => posRef.current(w))
    const segMs = Math.min(220, 1500 / steps)
    const t0 = performance.now()
    let raf = 0
    const tick = (now: number) => {
      const u = Math.min(1, (now - t0) / (segMs * steps))
      const e = u < 0.5 ? 2 * u * u : 1 - 2 * (1 - u) * (1 - u) // 缓入缓出
      const x = e * steps, i = Math.min(steps - 1, Math.floor(x)), f = x - i
      const [ax, ay, ar] = pts[i]!, [bx, by, br] = pts[i + 1]!
      const dr = ((((br - ar) % 360) + 540) % 360) - 180
      put([ax + (bx - ax) * f, ay + (by - ay) * f, ar + dr * f])
      if (u < 1) raf = requestAnimationFrame(tick)
    }
    raf = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(raf)
  }, [g])
  return (
    <g ref={ref}>
      <text x={0} y={0} fontSize={size} fill={fill} textAnchor="middle" dominantBaseline="central">{children}</text>
    </g>
  )
}

/** 一盘九星（洛书九宫，下标 = 宫 − 1）：按星数追踪所在宫，各星独立飞行；中五字号可另设 */
function FlyStars({ pan, pos, size, centerSize, fill, prefix = '' }: { pan: number[]; pos: (g: number) => [number, number, number]; size: number; centerSize: number; fill: string; prefix?: string }) {
  return (
    <g>
      {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((n) => {
        const idx = pan.indexOf(n)
        if (idx < 0) return null
        const g = idx + 1
        return <Fly key={n} g={g} pos={pos} size={g === 5 ? centerSize : size} fill={fill}>{`${prefix}${CN_NUM[n]}`}</Fly>
      })}
    </g>
  )
}

/** 九宫格（南上）各宫之行列：巽四 離九 坤二／震三 中五 兌七／艮八 坎一 乾六 */
const GRID_RC: Record<number, [number, number]> = { 4: [0, 0], 9: [0, 1], 2: [0, 2], 3: [1, 0], 5: [1, 1], 7: [1, 2], 8: [2, 0], 1: [2, 1], 6: [2, 2] }

/**
 * 玄空宅盘九宫格式（书中方格之式，南上）：每宫左上卦名、山星（蓝）居左、向星（红）居右、其下运星、角注流年与逐宫标记；
 * 与圆盘同数据，诸星亦沿洛书路径飞行（方格上即九宫飞星图之路）。随罗盘整盘转时可覆于房屋平面。
 */
export function XuankongGrid({ d, sel, onSel }: { d: XuankongWheelData; sel?: number | null; onSel?: (g: number) => void }) {
  const S = 500, M = 30, W = (S - 2 * M) / 3
  const cx = (g: number) => M + W * GRID_RC[g]![1] + W / 2
  const cy = (g: number) => M + W * GRID_RC[g]![0] + W / 2
  const cnt = new Map<string, number>()
  d.gongMarks?.forEach((ms) => ms.forEach((m) => cnt.set(m.n, (cnt.get(m.n) ?? 0) + 1)))
  const marksOf = (g: number) => (d.gongMarks?.[g - 1] ?? []).filter((m) => (cnt.get(m.n) ?? 0) < 8).map((m) => XK_SHORT[m.n] ?? m.n)
  ensureLiShu()
  return (
    <svg viewBox={`0 0 ${S} ${S}`} style={{ width: '100%', maxWidth: S, height: 'auto', fontFamily: LISHU_FAMILY }} role="img" aria-label="玄空飞星宅盘（九宫格）">
      {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((g) => {
        const x = M + W * GRID_RC[g]![1], y = M + W * GRID_RC[g]![0]
        const zheng = d.zhengShen?.includes(g), ling = d.lingShen?.includes(g)
        const mk = marksOf(g)
        return (
          <g key={g} onClick={onSel ? () => onSel(g) : undefined} style={onSel ? { cursor: 'pointer' } : undefined}>
            <rect x={x + 1} y={y + 1} width={W - 2} height={W - 2} fill={g === sel ? '#bae0ff' : zheng ? '#f6ffed' : ling ? '#fff1f0' : '#fffdf7'} stroke={g === sel ? '#1677ff' : '#999'} strokeWidth={g === sel ? 2 : 1} />
            <text x={x + 8} y={y + 18} fontSize={13} fill="#5b4a2e">{`${GUA_SYM[g] ?? ''}${GUA[g]}${CN_NUM[g]}`}</text>
            {mk.length > 0 && <text x={x + W - 6} y={y + W - 8} fontSize={10} fill="#a8071a" textAnchor="end">{mk.join(' ')}</text>}
          </g>
        )
      })}
      {[['南', S / 2, 18], ['北', S / 2, S - 8], ['東', 12, S / 2], ['西', S - 12, S / 2]].map(([t, x, y]) => (
        <text key={t as string} x={x as number} y={y as number} fontSize={12} fill={t === '北' ? '#a8071a' : '#999'} textAnchor="middle" dominantBaseline="central">{t}</text>
      ))}
      <text x={S / 2} y={S - 20} fontSize={10} fill="#888" textAnchor="middle">{`${d.zuo}山${d.xiang}向`}</text>
      <FlyStars pan={d.shanPan} pos={(g) => [cx(g) - W * 0.2, cy(g) - W * 0.05, 0]} size={30} centerSize={30} fill="#0958d9" />
      <FlyStars pan={d.xiangPan} pos={(g) => [cx(g) + W * 0.2, cy(g) - W * 0.05, 0]} size={30} centerSize={30} fill="#a8071a" />
      <FlyStars pan={d.yunPan} pos={(g) => [cx(g), cy(g) + W * 0.27, 0]} size={20} centerSize={20} fill="#000" />
      <FlyStars pan={d.yearPan} pos={(g) => [cx(g) + W * 0.36, cy(g) - W * 0.38, 0]} size={11} centerSize={11} fill="#531dab" prefix="年" />
    </svg>
  )
}

/** 玄空逐宫标记简称（全名与注见面板） */
const XK_SHORT: Record<string, string> = { 正神: '正', 零神: '零', 正城門: '城', 副城門: '副城', 雙星到向: '雙向', 雙星到坐: '雙坐', 山星入囚: '山囚', 向星入囚: '向囚' }
/** 兼向分段之色：下卦绿、替卦橙、小空亡浅红、大空亡红 */
const DUAN_FILL: Record<string, string> = { 下卦: '#95de64', 替卦: '#ffc069', 小空亡: '#ffa39e', 大空亡: '#cf1322' }

/** `sel`：所选之宫（洛书数 1—9），`onSel` 点宫 */
export function XuankongWheel({ d, sel, onSel }: { d: XuankongWheelData; sel?: number | null; onSel?: (g: number) => void }) {
  const zuoI = SHAN24.indexOf(d.zuo), xiangI = SHAN24.indexOf(d.xiang)
  // 外加人盘中针（各山较正针逆时针半山）、天盘缝针（顺时针半山）两环（协纪卷二、卷三十三）
  const R = { tian: 36, gong: 112, mark: 132, gua: 148, yuan: 162, shan: 184, duan: 190, ren: 205, tianPan: 220, du: 232 }
  const ring = (r: number, fill = 'none') => <circle cx={C} cy={C} r={r} fill={fill} stroke="#999" />
  // 坐向线：按山排盘取山之中线；罗盘测向时取实测（中线 + 偏）
  const zuoAz = 15 * zuoI + (d.pian ?? 0)
  const [zx, zy] = pt(R.tianPan, scr(zuoAz)), [xx, xy] = pt(R.tianPan, scr(zuoAz + 180))
  // 坐向线在中针、缝针上所落之山（恰在山界上则不标）
  const onLine = (off: number) => {
    const m = (((zuoAz - off) % 15) + 15) % 15
    if (Math.abs(m - 7.5) < 1e-6) return [-1, -1]
    const i = ((Math.round((zuoAz - off) / 15) % 24) + 24) % 24
    return [i, (i + 12) % 24]
  }
  const zhongHit = onLine(-7.5), fengHit = onLine(7.5)
  // 遍及八宫以上之标记（全盘格）不逐宫标，列于面板
  const cnt = new Map<string, number>()
  d.gongMarks?.forEach((ms) => ms.forEach((m) => cnt.set(m.n, (cnt.get(m.n) ?? 0) + 1)))
  const marksOf = (g: number) => (d.gongMarks?.[g - 1] ?? []).filter((m) => (cnt.get(m.n) ?? 0) < 8).map((m) => XK_SHORT[m.n] ?? m.n)
  return (
    <Frame label="玄空飞星宅盘">
      {ring(R.du, '#f5ecd7')}
      {Array.from({ length: 360 }, (_, a) => {
        const len = a % 15 === 0 ? 9 : a % 5 === 0 ? 6 : 3
        const [x0, y0] = pt(R.du, scr(a)), [x1, y1] = pt(R.du - len, scr(a))
        return <line key={a} x1={x0} y1={y0} x2={x1} y2={y1} stroke="#8c6d3f" strokeWidth={a % 15 === 0 ? 1 : 0.5} />
      })}
      {ring(R.tianPan, '#f3f9f9')}
      {ring(R.ren, '#fbf6ea')}
      {/* 人盘中针（子在正针壬子之间，格龙）、天盘缝针（子在子癸之缝，消砂纳水） */}
      {SHAN24.map((s, i) => {
        const yang = SHAN_YANG.has(s)
        const zh = zhongHit.includes(i), fe = fengHit.includes(i)
        return (
          <g key={`zf-${s}`}>
            <path d={sector(R.duan, R.ren, scr(15 * i - 15), scr(15 * i))} fill={zh ? (i === zhongHit[0] ? '#d6e4ff' : '#ffd8d6') : 'none'} stroke="#e3d9c3">
              <title>{`人盘中针 ${s}：${(15 * i - 15 + 360) % 360}°—${15 * i}°`}</title>
            </path>
            <path d={sector(R.ren, R.tianPan, scr(15 * i), scr(15 * i + 15))} fill={fe ? (i === fengHit[0] ? '#d6e4ff' : '#ffd8d6') : 'none'} stroke="#cfe3e3">
              <title>{`天盘缝针 ${s}：${15 * i}°—${15 * i + 15}°`}</title>
            </path>
            <T r={(R.duan + R.ren) / 2} deg={scr(15 * i - 7.5)} size={10} fill={yang ? '#a8071a' : '#5b4a2e'} weight={zh ? 700 : 400}>{s}</T>
            <T r={(R.ren + R.tianPan) / 2} deg={scr(15 * i + 7.5)} size={10} fill={yang ? '#a8071a' : '#08595e'} weight={fe ? 700 : 400}>{s}</T>
          </g>
        )
      })}
      {ring(R.duan, '#fffdf7')}
      {/* 兼向分段（坐、向两山）：下卦、替卦、空亡 */}
      {d.jieXian && [d.jieXian.zuo, d.jieXian.xiang].flatMap((j, a) => j.duan.map((s, k) => (
        <path key={`${a}-${k}`} d={sector(R.shan, R.duan, scr(s.from), scr(s.to))} fill={DUAN_FILL[s.kind] ?? '#ddd'}>
          <title>{`${j.shan}山 ${s.from.toFixed(1)}°—${s.to.toFixed(1)}° ${s.kind}${s.jian ? `（兼${s.jian}）` : ''}`}</title>
        </path>
      )))}
      {ring(R.shan, '#fffdf7')}
      {ring(R.yuan, '#fbf6ea')}
      {ring(R.gua, '#fffdf7')}
      {ring(R.mark, '#fbf6ea')}
      {ring(R.gong, '#fffdf7')}
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
        const mk = marksOf(g)
        const zheng = d.zhengShen?.includes(g), ling = d.lingShen?.includes(g)
        return (
          <g key={g}>
            <path d={sector(R.tian, R.gong, deg - 22.5, deg + 22.5)} fill={zheng ? '#f6ffed' : ling ? '#fff1f0' : 'none'} stroke="#ccc" />
            <path d={sector(R.gong, R.mark, deg - 22.5, deg + 22.5)} fill="none" stroke="#ccc" />
            <path d={sector(R.mark, R.gua, deg - 22.5, deg + 22.5)} fill="none" stroke="#bbb" />
            <T r={(R.mark + R.gua) / 2} deg={deg} size={12} fill="#5b4a2e">{`${GUA_SYM[g]} ${GUA[g]}${CN_NUM[g]}`}</T>
            <T r={50} deg={deg} size={10} fill="#999">{GUA[g]!}</T>
            {mk.length > 0 && <T r={(R.gong + R.mark) / 2} deg={deg} size={mk.join('').length > 6 ? 8 : 10} fill="#a8071a" weight={600}>{mk.join(' ')}</T>}
          </g>
        )
      })}
      {/* 坐向线 */}
      <line x1={zx} y1={zy} x2={xx} y2={xy} stroke="#a8071a" strokeWidth={1} strokeDasharray="4 3" opacity={0.6} />
      {ring(R.tian, '#fff')}
      <text x={C} y={C + 30} fontSize={8} textAnchor="middle" fill="#a8071a">{marksOf(5).join(' ')}</text>
      {/* 飞星：按星数追踪所在宫，换运、换坐向、换年时各星沿洛书飞行路径飞至新宫 */}
      <FlyStars pan={d.shanPan} pos={(g) => (g === 5 ? [C - 11, C - 3, 0] : polar(72, scr(GONG_AZ[g]!) - 10))} size={20} centerSize={16} fill="#0958d9" />
      <FlyStars pan={d.xiangPan} pos={(g) => (g === 5 ? [C + 11, C - 3, 0] : polar(72, scr(GONG_AZ[g]!) + 10))} size={20} centerSize={16} fill="#a8071a" />
      <FlyStars pan={d.yunPan} pos={(g) => (g === 5 ? [C, C + 15, 0] : polar(92, scr(GONG_AZ[g]!)))} size={15} centerSize={13} fill="#000" />
      <FlyStars pan={d.yearPan} pos={(g) => (g === 5 ? [C, C + 24, 0] : polar(105, scr(GONG_AZ[g]!)))} size={9} centerSize={8} fill="#531dab" prefix="年" />
      {sel != null && sel !== 5 && <path d={sector(R.tian, R.gong, scr(GONG_AZ[sel]!) - 22.5, scr(GONG_AZ[sel]!) + 22.5)} fill="#bae0ff" opacity={0.45} stroke="#1677ff" strokeWidth={1.5} pointerEvents="none" />}
      {sel === 5 && <circle cx={C} cy={C} r={R.tian} fill="#bae0ff" opacity={0.45} stroke="#1677ff" strokeWidth={1.5} pointerEvents="none" />}
      {onSel && (
        <g style={{ cursor: 'pointer' }}>
          {[1, 2, 3, 4, 6, 7, 8, 9].map((g) => (
            <path key={g} d={sector(R.tian, R.gong, scr(GONG_AZ[g]!) - 22.5, scr(GONG_AZ[g]!) + 22.5)} fill="transparent" pointerEvents="all" onClick={() => onSel(g)}><title>{`${GUA[g]}${CN_NUM[g]}宫`}</title></path>
          ))}
          <circle cx={C} cy={C} r={R.tian} fill="transparent" pointerEvents="all" onClick={() => onSel(5)}><title>中五宫</title></circle>
        </g>
      )}
    </Frame>
  )
}
