<script setup lang="ts">
// 统计图：读取 public/charts/<src>.json（由 scripts/figures/ 生成），画折线 / 散点 / 分组柱状图。
// 配色取 dataviz 参考调色板（分类色固定顺序，深色模式单独取色）；悬停十字线 + 提示；可展开数据表。
import { computed, onMounted, ref } from 'vue'
import { withBase } from 'vitepress'

interface XY { name: string; data: [number, number][]; band?: [number, number, number][] }
interface BarSeries { name: string; values: number[] }
interface ChartSpec {
  type: 'line' | 'scatter' | 'bar'
  x: { label: string; unit?: string }
  y: { label: string; unit?: string; min?: number; max?: number }
  series: (XY | BarSeries)[]
  categories?: string[]
  caption?: string
  table?: { maxRows?: number }
}

const props = defineProps<{ src: string; height?: number }>()
const spec = ref<ChartSpec | null>(null)
const error = ref('')
onMounted(async () => {
  try {
    const r = await fetch(withBase(`/charts/${props.src}.json`))
    spec.value = await r.json()
  } catch (e) {
    error.value = String(e)
  }
})

const W = 720
const H = computed(() => props.height ?? 320)
const M = { l: 64, r: 16, t: 12, b: 44 }
const iw = W - M.l - M.r
const ih = computed(() => H.value - M.t - M.b)

function niceTicks(lo: number, hi: number, n = 5): number[] {
  if (!(hi > lo)) return [lo]
  const raw = (hi - lo) / n
  const p = Math.pow(10, Math.floor(Math.log10(raw)))
  const step = [1, 2, 2.5, 5, 10].map((k) => k * p).find((s) => s >= raw) ?? raw
  const out: number[] = []
  for (let v = Math.ceil(lo / step) * step; v <= hi + step * 1e-9; v += step) out.push(+v.toPrecision(12))
  return out
}
function fmt(v: number): string {
  const a = Math.abs(v)
  if (a !== 0 && (a < 1e-3 || a >= 1e6)) return v.toExponential(2)
  return (+v.toPrecision(4)).toString()
}

const isBar = computed(() => spec.value?.type === 'bar')
const xy = computed(() => (isBar.value ? [] : ((spec.value?.series ?? []) as XY[])))
const bars = computed(() => (isBar.value ? ((spec.value?.series ?? []) as BarSeries[]) : []))

const xDomain = computed<[number, number]>(() => {
  const xs = xy.value.flatMap((s) => s.data.map((d) => d[0]))
  return xs.length ? [Math.min(...xs), Math.max(...xs)] : [0, 1]
})
const yDomain = computed<[number, number]>(() => {
  const s = spec.value
  let ys: number[] = []
  if (isBar.value) ys = [0, ...bars.value.flatMap((b) => b.values)]
  else ys = xy.value.flatMap((se) => [...se.data.map((d) => d[1]), ...(se.band ?? []).flatMap((b) => [b[1], b[2]])])
  let lo = s?.y.min ?? Math.min(...ys)
  let hi = s?.y.max ?? Math.max(...ys)
  if (lo === hi) { lo -= 1; hi += 1 }
  const pad = (hi - lo) * 0.05
  return [s?.y.min ?? lo - (isBar.value ? 0 : pad), s?.y.max ?? hi + pad]
})
const xTicks = computed(() => niceTicks(...xDomain.value, 6))
const yTicks = computed(() => niceTicks(...yDomain.value, 5))
const sx = (x: number) => M.l + ((x - xDomain.value[0]) / (xDomain.value[1] - xDomain.value[0] || 1)) * iw
const sy = (y: number) => M.t + ih.value - ((y - yDomain.value[0]) / (yDomain.value[1] - yDomain.value[0] || 1)) * ih.value

function path(d: [number, number][]): string {
  return d.map((p, i) => `${i ? 'L' : 'M'}${sx(p[0]).toFixed(1)},${sy(p[1]).toFixed(1)}`).join('')
}
function bandPath(b: [number, number, number][]): string {
  const up = b.map((p, i) => `${i ? 'L' : 'M'}${sx(p[0]).toFixed(1)},${sy(p[2]).toFixed(1)}`).join('')
  const dn = [...b].reverse().map((p) => `L${sx(p[0]).toFixed(1)},${sy(p[1]).toFixed(1)}`).join('')
  return up + dn + 'Z'
}

// 柱状图几何
const cats = computed(() => spec.value?.categories ?? [])
const groupW = computed(() => iw / Math.max(cats.value.length, 1))
const barW = computed(() => Math.min(28, (groupW.value * 0.75) / Math.max(bars.value.length, 1) - 2))
function barX(ci: number, si: number): number {
  const total = bars.value.length * (barW.value + 2) - 2
  return M.l + ci * groupW.value + (groupW.value - total) / 2 + si * (barW.value + 2)
}
function barPath(x: number, y0: number, y1: number, w: number): string {
  const top = Math.min(y0, y1), h = Math.abs(y1 - y0), r = Math.min(4, h, w / 2)
  return `M${x},${top + h}V${top + r}Q${x},${top} ${x + r},${top}H${x + w - r}Q${x + w},${top} ${x + w},${top + r}V${top + h}Z`
}

// 悬停
const hover = ref<{ x: number; y: number; lines: string[] } | null>(null)
function onMove(ev: MouseEvent) {
  const svg = ev.currentTarget as SVGSVGElement
  const pt = svg.getBoundingClientRect()
  const px = ((ev.clientX - pt.left) / pt.width) * W
  const py = ((ev.clientY - pt.top) / pt.height) * H.value
  const s = spec.value
  if (!s) return
  if (isBar.value) {
    const ci = Math.floor((px - M.l) / groupW.value)
    if (ci < 0 || ci >= cats.value.length) { hover.value = null; return }
    hover.value = { x: M.l + (ci + 0.5) * groupW.value, y: py, lines: [cats.value[ci], ...bars.value.map((b) => `${b.name}：${fmt(b.values[ci])}${s.y.unit ?? ''}`)] }
    return
  }
  const xv = xDomain.value[0] + ((px - M.l) / iw) * (xDomain.value[1] - xDomain.value[0])
  if (px < M.l || px > M.l + iw) { hover.value = null; return }
  const lines = [`${s.x.label} ${fmt(xv)}${s.x.unit ?? ''}`]
  for (const se of xy.value) {
    let best = se.data[0], bd = Infinity
    for (const d of se.data) { const dd = Math.abs(d[0] - xv); if (dd < bd) { bd = dd; best = d } }
    if (best) lines.push(`${se.name}：${fmt(best[1])}${s.y.unit ?? ''}`)
  }
  hover.value = { x: px, y: py, lines }
}

const tableRows = computed(() => {
  const s = spec.value
  if (!s) return []
  if (isBar.value) return cats.value.map((c, i) => [c, ...bars.value.map((b) => fmt(b.values[i]))])
  const max = s.table?.maxRows ?? 40
  const rows: string[][] = []
  for (const se of xy.value) {
    const step = Math.max(1, Math.ceil(se.data.length / max))
    for (let i = 0; i < se.data.length; i += step) rows.push([se.name, fmt(se.data[i][0]), fmt(se.data[i][1])])
  }
  return rows
})
</script>

<template>
  <figure class="viz">
    <div v-if="error" class="viz-err">图表加载失败：{{ error }}</div>
    <template v-else-if="spec">
      <div v-if="spec.series.length > 1" class="viz-legend">
        <span v-for="(se, i) in spec.series" :key="se.name"><i :class="`c${i + 1}`" />{{ se.name }}</span>
      </div>
      <div class="viz-plot">
        <svg :viewBox="`0 0 ${W} ${H}`" role="img" :aria-label="spec.caption ?? ''" @mousemove="onMove" @mouseleave="hover = null">
          <g class="grid">
            <line v-for="t in yTicks" :key="'gy' + t" :x1="M.l" :x2="M.l + iw" :y1="sy(t)" :y2="sy(t)" />
          </g>
          <g class="axis">
            <text v-for="t in yTicks" :key="'ty' + t" :x="M.l - 6" :y="sy(t) + 4" text-anchor="end">{{ fmt(t) }}</text>
            <template v-if="!isBar">
              <text v-for="t in xTicks" :key="'tx' + t" :x="sx(t)" :y="M.t + ih + 16" text-anchor="middle">{{ fmt(t) }}</text>
            </template>
            <template v-else>
              <text v-for="(c, i) in cats" :key="'cx' + c" :x="M.l + (i + 0.5) * groupW" :y="M.t + ih + 16" text-anchor="middle">{{ c }}</text>
            </template>
            <line class="base" :x1="M.l" :x2="M.l + iw" :y1="M.t + ih" :y2="M.t + ih" />
            <text :x="M.l + iw / 2" :y="H - 6" text-anchor="middle">{{ spec.x.label }}{{ spec.x.unit ? `（${spec.x.unit}）` : '' }}</text>
            <text :transform="`translate(14 ${M.t + ih / 2}) rotate(-90)`" text-anchor="middle">{{ spec.y.label }}{{ spec.y.unit ? `（${spec.y.unit}）` : '' }}</text>
          </g>
          <line v-if="!isBar && yDomain[0] < 0 && yDomain[1] > 0" class="zero" :x1="M.l" :x2="M.l + iw" :y1="sy(0)" :y2="sy(0)" />
          <template v-if="!isBar">
            <g v-for="(se, i) in xy" :key="se.name" :class="`s c${i + 1}`">
              <path v-if="se.band" class="band" :d="bandPath(se.band)" />
              <path v-if="spec.type === 'line'" class="line" :d="path(se.data)" />
              <template v-else>
                <circle v-for="(d, k) in se.data" :key="k" class="dot" :cx="sx(d[0])" :cy="sy(d[1])" r="2.5" />
              </template>
            </g>
          </template>
          <template v-else>
            <g v-for="(b, si) in bars" :key="b.name" :class="`s c${si + 1}`">
              <path v-for="(v, ci) in b.values" :key="ci" class="bar" :d="barPath(barX(ci, si), sy(0), sy(v), barW)" />
            </g>
          </template>
          <line v-if="hover && !isBar" class="cross" :x1="hover.x" :x2="hover.x" :y1="M.t" :y2="M.t + ih" />
        </svg>
        <div v-if="hover" class="viz-tip" :style="{ left: `${(hover.x / W) * 100}%`, top: `${(hover.y / H) * 100}%` }">
          <div v-for="(l, i) in hover.lines" :key="i" :class="{ head: i === 0 }">{{ l }}</div>
        </div>
      </div>
      <figcaption v-if="spec.caption">{{ spec.caption }}</figcaption>
      <details class="viz-table">
        <summary>数据表</summary>
        <table>
          <thead>
            <tr v-if="isBar"><th>{{ spec.x.label }}</th><th v-for="b in bars" :key="b.name">{{ b.name }}</th></tr>
            <tr v-else><th>系列</th><th>{{ spec.x.label }}</th><th>{{ spec.y.label }}</th></tr>
          </thead>
          <tbody><tr v-for="(r, i) in tableRows" :key="i"><td v-for="(c, j) in r" :key="j">{{ c }}</td></tr></tbody>
        </table>
      </details>
    </template>
  </figure>
</template>

<style scoped>
.viz {
  --surface: #fcfcfb; --ink: #0b0b0b; --ink2: #52514e; --muted: #898781;
  --grid: #e1e0d9; --base: #c3c2b7;
  --c1: #2a78d6; --c2: #eb6834; --c3: #1baf7a; --c4: #eda100;
  margin: 1.5rem 0; padding: 12px; border-radius: 8px; background: var(--surface);
  border: 1px solid rgba(11, 11, 11, 0.1);
}
:global(.dark) .viz {
  --surface: #1a1a19; --ink: #f0efec; --ink2: #c3c2b7; --muted: #898781;
  --grid: #2c2c2a; --base: #383835;
  --c1: #3987e5; --c2: #d95926; --c3: #199e70; --c4: #c98500;
  border-color: rgba(255, 255, 255, 0.1);
}
.viz-plot { position: relative; }
svg { width: 100%; height: auto; display: block; font: 12px system-ui, -apple-system, 'Segoe UI', sans-serif; font-variant-numeric: tabular-nums; }
.grid line { stroke: var(--grid); stroke-width: 1; }
.axis text { fill: var(--muted); }
.axis .base { stroke: var(--base); }
.zero { stroke: var(--base); stroke-dasharray: 3 3; }
.cross { stroke: var(--muted); stroke-width: 1; }
.s.c1 { --c: var(--c1); } .s.c2 { --c: var(--c2); } .s.c3 { --c: var(--c3); } .s.c4 { --c: var(--c4); }
.line { fill: none; stroke: var(--c); stroke-width: 2; stroke-linejoin: round; }
.band { fill: var(--c); opacity: 0.18; }
.dot { fill: var(--c); stroke: var(--surface); stroke-width: 1; }
.bar { fill: var(--c); }
.viz-legend { display: flex; flex-wrap: wrap; gap: 4px 16px; font-size: 13px; color: var(--ink2); margin-bottom: 4px; }
.viz-legend i { display: inline-block; width: 12px; height: 3px; border-radius: 2px; margin-right: 6px; vertical-align: middle; }
.viz-legend i.c1 { background: var(--c1); } .viz-legend i.c2 { background: var(--c2); }
.viz-legend i.c3 { background: var(--c3); } .viz-legend i.c4 { background: var(--c4); }
.viz-tip {
  position: absolute; transform: translate(12px, -50%); pointer-events: none; z-index: 2;
  background: var(--surface); color: var(--ink); border: 1px solid var(--grid); border-radius: 6px;
  padding: 6px 8px; font-size: 12px; white-space: nowrap; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}
.viz-tip .head { color: var(--ink2); margin-bottom: 2px; }
figcaption { margin-top: 6px; font-size: 0.9em; color: var(--ink2); text-align: center; }
.viz-table { margin-top: 6px; font-size: 13px; color: var(--ink2); }
.viz-table table { display: table; width: 100%; margin: 6px 0 0; }
.viz-err { color: var(--ink2); font-size: 13px; }
</style>
