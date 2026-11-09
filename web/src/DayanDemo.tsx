/**
 * 揲蓍逐变演示：《周易·繫辭上》「大衍之數五十，其用四十有九，分而為二以象兩，掛一以象三，揲之以四以象四時，歸奇於扐以象閏」。
 * 一变：分二、掛一、揲四、归奇；三变成一爻，十八变成卦。逐变计算取核心 `zhouyi::dayan_steps`（wasm `dayan_steps_json`），
 * 与排盘页「大衍揲蓍」（`dayan_yao_wasm`）同法：每变左手之分 left = 1 + r mod (所用策 − 2)。wasm 缺此函数时以下方同法之 JS 计算代之。
 */
import { Button, Flex, Typography } from 'antd'
import { useEffect, useMemo, useState } from 'react'
import type { LunarBackend } from './lunar-backend-types'

const { Text } = Typography

export type DayanStep = {
  n: number; yao: number; bian: number; total: number; left: number; right: number; rightAfterGua: number
  leftQi: number; rightQi: number; removed: number; remain: number; value: number | null
}

/** 与核心 zhouyi::dayan_steps 同法（备用）：余零作四；所去 = 掛一 + 左奇 + 右奇 */
function stepsJs(r: number[]): DayanStep[] {
  const out: DayanStep[] = []
  let total = 49
  const q = (n: number) => (n % 4 === 0 ? 4 : n % 4)
  for (let i = 0; i < 18; i++) {
    if (i % 3 === 0) total = 49
    const left = 1 + ((r[i] ?? 0) % (total - 2))
    const right = total - left
    const leftQi = q(left), rightQi = q(right - 1)
    const removed = 1 + leftQi + rightQi
    const remain = total - removed
    out.push({ n: i + 1, yao: Math.floor(i / 3) + 1, bian: (i % 3) + 1, total, left, right, rightAfterGua: right - 1, leftQi, rightQi, removed, remain, value: i % 3 === 2 ? remain / 4 : null })
    total = remain
  }
  return out
}

function randoms(): number[] {
  const a = new Uint32Array(18)
  try { crypto.getRandomValues(a) } catch { for (let i = 0; i < 18; i++) a[i] = Math.floor(Math.random() * 2 ** 32) }
  return Array.from(a)
}

const PHASE = ['合策', '分而為二', '掛一', '揲之以四', '歸奇於扐'] as const
const YAO_LABEL: Record<number, string> = { 9: '老陽（重 ○）', 8: '少陰（拆 ⚋）', 7: '少陽（單 ⚊）', 6: '老陰（交 ✕）' }
const POS = ['初', '二', '三', '四', '五', '上']

/** 策之位置：六策一行之「四策一束」网格 */
function grid(j: number, ox: number, oy: number): [number, number] {
  const b = Math.floor(j / 4), w = j % 4
  return [ox + (b % 6) * 24 + w * 4, oy + Math.floor(b / 6) * 36]
}

function Sticks({ s, phase }: { s: DayanStep; phase: number }) {
  const items = []
  let k = 0 // 归奇所去之序
  for (let i = 0; i < s.total; i++) {
    let x: number, y: number, color = '#8c6d1f', h = 28
    const isLeft = i < s.left
    const j = i - s.left
    const isGua = !isLeft && j === s.right - 1
    const isQi = isLeft ? i >= s.left - s.leftQi : !isGua && j >= s.rightAfterGua - s.rightQi
    if (phase === 0) [x, y] = grid(i, 185, 14)
    else if (isLeft) [x, y] = grid(i, 14, 14)
    else if (isGua && phase >= 2) { x = 258; y = 112; color = '#cf1322' }
    else [x, y] = grid(j, 362, 14)
    if (phase >= 3 && isQi) color = '#d46b08'
    if (phase >= 4 && (isQi || isGua)) { x = 150 + k * 9; y = 158; h = 22; k++ }
    if (phase >= 4 && !isQi && !isGua) color = '#389e0d'
    items.push(
      <line key={i} x1={0} y1={0} x2={0} y2={h} stroke={color} strokeWidth={2.2} strokeLinecap="round"
        style={{ transform: `translate(${x}px, ${y}px)`, transition: 'transform 0.6s ease, stroke 0.4s' }} />,
    )
  }
  return <g>{items}</g>
}

export function DayanDemo(props: { wasm: LunarBackend; onUse: (values: number[]) => void }) {
  const [r, setR] = useState<number[]>(randoms)
  const [idx, setIdx] = useState(0) // 第几变（0—17）
  const [phase, setPhase] = useState(0)
  const [auto, setAuto] = useState(false)
  const steps = useMemo((): DayanStep[] => {
    if (typeof props.wasm.dayan_steps_json === 'function') {
      try { return (JSON.parse(props.wasm.dayan_steps_json(Uint32Array.from(r))) as { steps: DayanStep[] }).steps } catch { /* 退回同法 JS */ }
    }
    return stepsJs(r)
  }, [props.wasm, r])
  const s = steps[idx]!
  const finished = idx === 17 && phase === 4
  const next = () => {
    if (phase < 4) setPhase(phase + 1)
    else if (idx < 17) { setIdx(idx + 1); setPhase(0) }
  }
  useEffect(() => {
    if (!auto) return
    if (finished) { setAuto(false); return }
    const t = setTimeout(next, phase === 4 ? 1100 : 750)
    return () => clearTimeout(t)
  })
  const done = steps.filter((x) => x.value !== null && (x.n - 1 < idx || (x.n - 1 === idx && phase === 4))).map((x) => x.value!)
  const restart = () => { setR(randoms()); setIdx(0); setPhase(0); setAuto(false) }
  const desc = [
    `用策 ${s.total}`,
    phase >= 1 ? `分左 ${s.left}、右 ${s.right}` : '',
    phase >= 2 ? `自右掛一，右余 ${s.rightAfterGua}` : '',
    phase >= 3 ? `左四四揲之余 ${s.leftQi}，右余 ${s.rightQi}（余零作四）` : '',
    phase >= 4 ? `歸奇：去 ${s.removed}（掛一 + ${s.leftQi} + ${s.rightQi}），余 ${s.remain}` : '',
  ].filter(Boolean).join('；')
  return (
    <div style={{ border: '1px solid #f0f0f0', borderRadius: 6, padding: 8, marginTop: 8 }}>
      <Flex gap={6} wrap align="center" style={{ marginBottom: 4 }}>
        <Text strong>揲蓍逐变</Text>
        <Button size="small" onClick={next} disabled={finished}>下一步</Button>
        <Button size="small" onClick={() => setAuto(!auto)} disabled={finished}>{auto ? '暂停' : '自动'}</Button>
        <Button size="small" onClick={() => { setIdx(17); setPhase(4); setAuto(false) }}>跳到成卦</Button>
        <Button size="small" onClick={restart}>重新揲蓍</Button>
        <Button size="small" type="primary" disabled={!finished} onClick={() => props.onUse(steps.filter((x) => x.value !== null).map((x) => x.value!))}>用此卦排盘</Button>
      </Flex>
      <Text style={{ fontSize: 12 }}>{`第 ${s.n} 变（${POS[s.yao - 1]}爻第 ${s.bian} 变）· ${PHASE[phase]}`}</Text>
      <svg viewBox="0 0 520 190" style={{ width: '100%', maxWidth: 520, display: 'block' }} role="img" aria-label="揲蓍逐变">
        <text x={14} y={10} fontSize={10} fill="#888">{phase >= 1 ? `左（象天）${s.left}` : ''}</text>
        <text x={362} y={10} fontSize={10} fill="#888">{phase >= 1 ? `右（象地）${phase >= 2 ? s.rightAfterGua : s.right}` : ''}</text>
        <text x={185} y={10} fontSize={10} fill="#888">{phase === 0 ? `所用之策 ${s.total}` : ''}</text>
        <text x={240} y={106} fontSize={10} fill="#cf1322">{phase >= 2 ? '掛一（象人）' : ''}</text>
        <line x1={10} y1={184} x2={510} y2={184} stroke="#eee" />
        <text x={14} y={176} fontSize={10} fill="#d46b08">{phase >= 4 ? `扐（象閏）去 ${s.removed}` : ''}</text>
        <Sticks key={s.n} s={s} phase={phase} />
      </svg>
      <div style={{ fontSize: 12 }}>{desc}</div>
      {phase === 4 && s.value !== null && (
        <div style={{ fontSize: 12, color: '#0958d9' }}>{`三变余 ${s.remain} 策，以四除之得 ${s.value}：${POS[s.yao - 1]}爻${YAO_LABEL[s.value]}`}</div>
      )}
      <div style={{ fontSize: 12, marginTop: 4 }}>
        {`已成：${done.length ? done.map((v, i) => `${POS[i]}${v}`).join(' ') : '—'}`}
        <Text type="secondary" style={{ fontSize: 11 }}>{'　第一变所去必五或九，二三变必四或八；三变之余 36、32、28、24 策即九、八、七、六'}</Text>
      </div>
    </div>
  )
}
