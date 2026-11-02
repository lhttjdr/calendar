/**
 * 允许有环的网络布局：应力布局（SMACOF）。把图当无向图，让节点的屏幕距离逼近图上跳数，环自然展开成环。
 * 初值由节点序决定、没有随机数，同一张图每次布局相同。
 */

export interface LayoutEdge {
  source: string
  target: string
}

/** 无向图上两两跳数；不连通的按最大跳数 + 1。 */
function hopDistances(ids: string[], edges: LayoutEdge[]): number[][] {
  const index = new Map(ids.map((id, i) => [id, i]))
  const adj: number[][] = ids.map(() => [])
  for (const e of edges) {
    const [a, b] = [index.get(e.source), index.get(e.target)]
    if (a == null || b == null) continue
    adj[a].push(b)
    adj[b].push(a)
  }
  const dist = ids.map((_, start) => {
    const d = ids.map(() => Infinity)
    d[start] = 0
    const queue = [start]
    for (let head = 0; head < queue.length; head++) {
      for (const v of adj[queue[head]]) {
        if (d[v] === Infinity) {
          d[v] = d[queue[head]] + 1
          queue.push(v)
        }
      }
    }
    return d
  })
  const finite = dist.flat().filter((x) => x !== Infinity)
  const far = Math.max(1, ...finite) + 1
  return dist.map((row) => row.map((x) => (x === Infinity ? far : x)))
}

/** 把相互重叠的矩形沿重叠较小的轴推开。 */
function separateBoxes(pos: { x: number; y: number }[], width: number, height: number, gap: number) {
  for (let round = 0; round < 200; round++) {
    let moved = false
    for (let i = 0; i < pos.length; i++) {
      for (let j = i + 1; j < pos.length; j++) {
        const [dx, dy] = [pos[j].x - pos[i].x, pos[j].y - pos[i].y]
        const [ox, oy] = [width + gap - Math.abs(dx), height + gap - Math.abs(dy)]
        if (ox <= 0 || oy <= 0) continue
        moved = true
        if (ox < oy) {
          const s = (dx >= 0 ? ox : -ox) / 2
          pos[i].x -= s
          pos[j].x += s
        } else {
          const s = (dy >= 0 ? oy : -oy) / 2
          pos[i].y -= s
          pos[j].y += s
        }
      }
    }
    if (!moved) return
  }
}

/** 应力与朝向无关：把主轴转到水平，长链横放才贴合宽画布，缩放不至于太小。 */
function alignPrincipalAxis(pos: { x: number; y: number }[]) {
  const n = pos.length
  const [cx, cy] = [pos.reduce((s, p) => s + p.x, 0) / n, pos.reduce((s, p) => s + p.y, 0) / n]
  let [sxx, syy, sxy] = [0, 0, 0]
  for (const p of pos) {
    const [dx, dy] = [p.x - cx, p.y - cy]
    sxx += dx * dx
    syy += dy * dy
    sxy += dx * dy
  }
  const theta = 0.5 * Math.atan2(2 * sxy, sxx - syy)
  const [c, s] = [Math.cos(-theta), Math.sin(-theta)]
  for (const p of pos) {
    const [dx, dy] = [p.x - cx, p.y - cy]
    p.x = c * dx - s * dy
    p.y = s * dx + c * dy
  }
}

/** 返回各节点框左上角坐标。`unit` 为一跳的屏幕距离。 */
export function stressLayout(
  ids: string[],
  edges: LayoutEdge[],
  options: { unit: number; width: number; height: number; gap: number; iterations?: number },
): Map<string, { x: number; y: number }> {
  const n = ids.length
  const d = hopDistances(ids, edges)
  // 初值：节点序排成螺旋，避免全共线时迭代卡住
  const pos = ids.map((_, i) => {
    const angle = i * 2.399963
    const r = options.unit * (1 + i * 0.3)
    return { x: r * Math.cos(angle), y: r * Math.sin(angle) }
  })
  // SMACOF，权重 1/d²：近邻的距离优先准确。每轮应力单调不增
  for (let iter = 0; iter < (options.iterations ?? 300); iter++) {
    for (let i = 0; i < n; i++) {
      let [sw, sx, sy] = [0, 0, 0]
      for (let j = 0; j < n; j++) {
        if (i === j) continue
        const target = d[i][j] * options.unit
        const w = 1 / (d[i][j] * d[i][j])
        const [dx, dy] = [pos[i].x - pos[j].x, pos[i].y - pos[j].y]
        const len = Math.hypot(dx, dy) || 1e-6
        sw += w
        sx += w * (pos[j].x + (target * dx) / len)
        sy += w * (pos[j].y + (target * dy) / len)
      }
      pos[i] = { x: sx / sw, y: sy / sw }
    }
  }
  alignPrincipalAxis(pos)
  separateBoxes(pos, options.width, options.height, options.gap)
  const [minX, minY] = [Math.min(...pos.map((p) => p.x)), Math.min(...pos.map((p) => p.y))]
  return new Map(ids.map((id, i) => [id, { x: pos[i].x - minX, y: pos[i].y - minY }]))
}
