/**
 * 视位置变换图展示页：由 WASM 取得 TransformGraph 的节点与边，用 React Flow 绘制有向图。
 * 使用 dagre 做从左到右的 DAG 自动布局。
 */
import { useMemo, useEffect, useRef } from 'react'
import { ReactFlow, Background, Controls, Handle, Position, MarkerType, useNodesState, type Node, type Edge, type NodeProps } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import dagre from 'dagre'
import { Card, Typography } from 'antd'
import type { LunarBackend } from './lunar-backend-types'

const { Text } = Typography

/** WASM 返回的变换图可视化数据。边可能为 getter 方法（from_id()/to_id()/label()）或属性。 */
interface TransformGraphViz {
  readonly nodeIds: string[]
  readonly edges: readonly ((
    | { from_id: string; to_id: string; cost: number; label?: string | null }
    | { from_id: () => string; to_id: () => string; cost: number; label?: () => string | null }
  ))[]
}

function edgeFromId(e: TransformGraphViz['edges'][number] | null | undefined): string {
  if (e == null) return ''
  return typeof (e as { from_id: string | (() => string) }).from_id === 'function'
    ? (e as { from_id: () => string }).from_id()
    : (e as { from_id: string }).from_id
}
function edgeToId(e: TransformGraphViz['edges'][number] | null | undefined): string {
  if (e == null) return ''
  return typeof (e as { to_id: string | (() => string) }).to_id === 'function'
    ? (e as { to_id: () => string }).to_id()
    : (e as { to_id: string }).to_id
}
function edgeCost(e: TransformGraphViz['edges'][number] | null | undefined): number {
  if (e == null) return 0
  return (e as { cost: number }).cost
}
function edgeLabel(e: TransformGraphViz['edges'][number] | null | undefined): string | null {
  if (e == null) return null
  try {
    const l = (e as { label?: string | null | (() => string | null) }).label
    if (l == null) return null
    const v = typeof l === 'function' ? l() ?? null : l ?? null
    return v != null && String(v).trim() !== '' ? String(v) : null
  } catch {
    return null
  }
}

/** 节点宽高；布局间距单独设大一些，保证连线足够长、线上文字可显示且框不压线 */
const NODE_WIDTH = 160
const NODE_HEIGHT = 44
const PAD = 32
/** dagre 同层节点水平间距、层与层垂直间距：加大以拉长连线、留出边标签空间 */
const LAYOUT_NODESEP = 140
const LAYOUT_RANKSEP = 120

function getGraphData(wasm: LunarBackend & { transformGraphVisualizationData?: () => TransformGraphViz }): TransformGraphViz | null {
  if (typeof (wasm as { transformGraphVisualizationData?: () => TransformGraphViz }).transformGraphVisualizationData !== 'function') {
    return null
  }
  return (wasm as { transformGraphVisualizationData: () => TransformGraphViz }).transformGraphVisualizationData()
}

const HELIOCENTRIC_MEAN_ECLIPTIC = '日心_MeanEcliptic(epoch)'
const ECLIPTIC_PATCH_ID = 'Vsop87De406EclipticPatch'

const MEAN_ECLIPTIC_EQUATOR = 'MeanEcliptic(epoch)_赤道'
const MEAN_ECLIPTIC_ECLIPTIC = 'MeanEcliptic(epoch)_黄道'
const FK5_UNCORRECTED = 'FK5_未修正'
const FK5_CORRECTED = 'FK5_已修正'

/** 连线仅写动作，不重复框内标架/坐标系；与 Rust edge_label 一致，WASM 为空时兜底 */
function getEdgeLabelFallback(fromId: string, toId: string): string {
  const key = `${fromId}\0${toId}`
  const map: Record<string, string> = {
    ['VSOP87\0' + HELIOCENTRIC_MEAN_ECLIPTIC]: '光行时→tr；历表求值',
    [HELIOCENTRIC_MEAN_ECLIPTIC + '\0' + MEAN_ECLIPTIC_EQUATOR]: '日心→地心',
    ['VSOP87\0' + MEAN_ECLIPTIC_EQUATOR]: '光行时→tr；历表输出',
    ['VSOP87\0' + ECLIPTIC_PATCH_ID]: '光行时→tr；黄道拟合：L,B 拟合修正',
    [ECLIPTIC_PATCH_ID + '\0' + MEAN_ECLIPTIC_ECLIPTIC]: 'R_x(ε₀)+Frame bias',
    ['ELPMPP02\0ELPMPP02_MEAN_LUNAR']: '历表求值（含 DE405/Table6 修正）',
    ['ELPMPP02_MEAN_LUNAR\0' + MEAN_ECLIPTIC_ECLIPTIC]: 'Laskar P,Q 旋转',
    [MEAN_ECLIPTIC_EQUATOR + '\0' + FK5_UNCORRECTED]: '黄赤交角 R_x(ε₀)',
    [MEAN_ECLIPTIC_ECLIPTIC + '\0' + FK5_CORRECTED]: '黄赤交角 R_x(ε₀)',
    [FK5_UNCORRECTED + '\0Fk5ToIcrsBias+Vsop87FitDe406Equatorial']: 'Frame bias B + DE406 拟合修正',
    ['Fk5ToIcrsBias+Vsop87FitDe406Equatorial\0ICRS']: '恒等',
    ['ICRS\0' + FK5_CORRECTED]: 'B^T（Frame bias 逆）',
    [FK5_CORRECTED + '\0MeanEquator(epoch)']: '岁差（P03）',
    ['MeanEquator(epoch)\0TrueEquator(epoch)']: '章动',
    ['TrueEquator(epoch)\0ApparentEcliptic(epoch)']: 'R_x(ε) 真黄赤交角',
  }
  return map[key] ?? '几何变换'
}

/** 框内：标架 + 坐标系 + 历元。赤道/黄道拆两节点；FK5 拆为未修正→patch→已修正 */
const NODE_BOX_LABELS: Record<string, string> = {
  VSOP87: 'VSOP87 · J2000 动力学平黄道',
  [HELIOCENTRIC_MEAN_ECLIPTIC]: '日心 · 历元平黄道',
  Vsop87De406EclipticPatch: 'Vsop87+黄道补丁 · J2000 平黄道',
  ELPMPP02: 'ELPMPP02 · J2000 月心平架',
  ELPMPP02_MEAN_LUNAR: 'ELP 平均根数 · J2000 平黄道',
  'MeanEcliptic(epoch)_赤道': 'MeanEcliptic · 历元平黄道（赤道路径）',
  'MeanEcliptic(epoch)_黄道': 'MeanEcliptic · 历元平黄道（黄道路径）',
  'FK5_未修正': 'FK5 · J2000 平赤道（未修正）',
  'FK5_已修正': 'FK5 · J2000 平赤道（已修正）',
    'Fk5ToIcrsBias+Vsop87FitDe406Equatorial': 'Vsop→DE406 拟合 · ICRS',
  ICRS: 'ICRS · 国际天球参考系',
  'MeanEquator(epoch)': 'MeanEquator · 历元平赤道',
  'TrueEquator(epoch)': 'TrueEquator · 历元真赤道',
  'ApparentEcliptic(epoch)': 'ApparentEcliptic · 历元视黄道',
}

/** 使用 dagre 做 TB 的 DAG 自动布局，减少交叉与长边。 */
function getLayoutedNodes(
  nodes: Node[],
  edges: { source: string; target: string }[]
): Node[] {
  const g = new dagre.graphlib.Graph()
  g.setGraph({ rankdir: 'TB', nodesep: LAYOUT_NODESEP, ranksep: LAYOUT_RANKSEP })
  g.setDefaultEdgeLabel(() => ({}))
  nodes.forEach((node) => {
    const w = (node.width as number) ?? NODE_WIDTH
    const h = (node.height as number) ?? NODE_HEIGHT
    g.setNode(node.id, { width: w, height: h })
  })
  edges.forEach((e) => { g.setEdge(e.source, e.target) })
  dagre.layout(g)
  return nodes.map((node) => {
    const n = g.node(node.id)
    if (!n) return { ...node, sourcePosition: Position.Bottom, targetPosition: Position.Top }
    const w = (node.width as number) ?? NODE_WIDTH
    const h = (node.height as number) ?? NODE_HEIGHT
    return {
      ...node,
      position: { x: n.x - w / 2, y: n.y - h / 2 },
      sourcePosition: Position.Bottom,
      targetPosition: Position.Top,
    }
  })
}

/** 四向 Handle：上下左右均可连出/连入，连线不限于上下或左右，可任意方向（左连上、右连上、下连右等） */
const SIDE_IDS = ['top', 'bottom', 'left', 'right'] as const
type SideId = (typeof SIDE_IDS)[number]
function sourceHandleId(side: SideId): string {
  return `${side}-source`
}
function targetHandleId(side: SideId): string {
  return `${side}-target`
}

/** 按两节点中心相对位置选锚点（就近原则），得到任意方向的连线组合 */
function getBestHandlePair(
  sourceNode: { position: { x: number; y: number }; width?: number; height?: number },
  targetNode: { position: { x: number; y: number }; width?: number; height?: number }
): { sourceHandle: string; targetHandle: string } {
  const sw = (sourceNode.width as number) ?? NODE_WIDTH
  const sh = (sourceNode.height as number) ?? NODE_HEIGHT
  const tw = (targetNode.width as number) ?? NODE_WIDTH
  const th = (targetNode.height as number) ?? NODE_HEIGHT
  const scx = sourceNode.position.x + sw / 2
  const scy = sourceNode.position.y + sh / 2
  const tcx = targetNode.position.x + tw / 2
  const tcy = targetNode.position.y + th / 2
  const dx = tcx - scx
  const dy = tcy - scy
  const sourceSide: SideId =
    Math.abs(dx) >= Math.abs(dy) ? (dx >= 0 ? 'right' : 'left') : (dy >= 0 ? 'bottom' : 'top')
  const targetSide: SideId =
    Math.abs(dx) >= Math.abs(dy) ? (dx >= 0 ? 'left' : 'right') : (dy >= 0 ? 'top' : 'bottom')
  return { sourceHandle: sourceHandleId(sourceSide), targetHandle: targetHandleId(targetSide) }
}

/** 节点 data：框 = 静态概念（标架·坐标系·历元），连线 = 动作 */
type BoxNodeData = { label?: string }

/** 自定义节点：框内仅静态概念标题，四向锚点 */
function BoxNode({ data }: NodeProps<BoxNodeData>) {
  const label = data?.label ?? ''
  const parts = label.split(' · ')
  const positions: { side: SideId; position: Position }[] = [
    { side: 'top', position: Position.Top },
    { side: 'bottom', position: Position.Bottom },
    { side: 'left', position: Position.Left },
    { side: 'right', position: Position.Right },
  ]
  return (
    <div
      style={{
        padding: '10px 14px',
        minWidth: NODE_WIDTH,
        minHeight: NODE_HEIGHT,
        border: '1.5px solid #1677ff',
        borderRadius: 8,
        background: '#fff',
        textAlign: 'center',
        fontSize: 11,
        color: '#262626',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'stretch',
        position: 'relative',
        gap: 6,
      }}
    >
      {positions.map(({ side, position }) => (
        <Handle key={`t-${side}`} type="target" id={targetHandleId(side)} position={position} />
      ))}
      {positions.map(({ side, position }) => (
        <Handle key={`s-${side}`} type="source" id={sourceHandleId(side)} position={position} />
      ))}
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 2 }}>
        {parts.length <= 1 ? (
          <span>{label}</span>
        ) : (
          parts.map((p, i) => <span key={i}>{p}</span>)
        )}
      </div>
    </div>
  )
}

const nodeTypes = { box: BoxNode }

export function TransformGraphPage({ wasm }: { wasm: LunarBackend | null }) {
  const graphData = useMemo(() => (wasm ? getGraphData(wasm as LunarBackend & { transformGraphVisualizationData?: () => TransformGraphViz }) : null), [wasm])

  if (!wasm) {
    return (
      <Card>
        <Text type="secondary">请先加载日历页以初始化 WASM。</Text>
      </Card>
    )
  }

  if (!graphData) {
    return (
      <Card>
        <Text type="secondary">当前 WASM 构建未包含 transformGraphVisualizationData，请使用最新 wasm-pack 构建。</Text>
      </Card>
    )
  }

  /** 解析后立即物化为普通对象，避免 WASM 引用在后续渲染中失效 */
  type EdgeRow = { from_id: string; to_id: string; cost: number; label: string | null }
  let nodeIds: string[] = []
  let edgesData: EdgeRow[] = []
  try {
    const rawNodeIds = typeof (graphData as { nodeIds: string[] | (() => string[]) }).nodeIds === 'function'
      ? (graphData as { nodeIds: () => string[] }).nodeIds()
      : Array.isArray((graphData as { nodeIds: string[] }).nodeIds)
        ? (graphData as { nodeIds: string[] }).nodeIds
        : []
    nodeIds = Array.isArray(rawNodeIds)
      ? rawNodeIds.map((id) => (id != null && typeof id === 'string' ? id.trim() : String(id).trim()))
      : []
    const rawEdges = typeof (graphData as { edges: unknown[] | (() => unknown[]) }).edges === 'function'
      ? (graphData as { edges: () => TransformGraphViz['edges'] }).edges()
      : Array.isArray((graphData as { edges: unknown[] }).edges)
        ? (graphData as { edges: TransformGraphViz['edges'] }).edges
        : []
    const rawList = Array.isArray(rawEdges) ? rawEdges.filter((e): e is NonNullable<typeof e> => e != null) : []
    edgesData = rawList.map((e) => {
      const fromId = edgeFromId(e).trim()
      const toId = edgeToId(e).trim()
      const wasmLabel = edgeLabel(e)
      const label = wasmLabel != null && String(wasmLabel).trim() !== ''
        ? String(wasmLabel).trim()
        : getEdgeLabelFallback(fromId, toId)
      return { from_id: fromId, to_id: toId, cost: edgeCost(e), label }
    })
  } catch (e) {
    return (
      <Card>
        <Text type="secondary">读取图数据时出错，请刷新或更新 WASM 构建。</Text>
        {typeof (e as Error)?.message === 'string' && (
          <pre style={{ marginTop: 8, fontSize: 12, color: '#8c8c8c' }}>{(e as Error).message}</pre>
        )}
      </Card>
    )
  }

  const nodeIdSet = useMemo(() => new Set(nodeIds), [nodeIds])
  const edgeListForLayout = useMemo(
    () =>
      edgesData
        .filter((e) => nodeIdSet.has(e.from_id) && nodeIdSet.has(e.to_id))
        .map((e) => ({ source: e.from_id, target: e.to_id })),
    [edgesData, nodeIdSet]
  )
  const flowNodes: Node[] = useMemo(() => {
    const rawNodes: Node[] = nodeIds.map((id) => ({
      id,
      type: 'box' as const,
      position: { x: 0, y: 0 },
      data: { label: NODE_BOX_LABELS[id] ?? id },
      width: NODE_WIDTH,
      height: NODE_HEIGHT,
    }))
    return getLayoutedNodes(rawNodes, edgeListForLayout)
  }, [nodeIds, edgeListForLayout])
  const flowEdges: Edge[] = useMemo(() => {
    const nodeMap = new Map(flowNodes.map((n) => [n.id, n]))
    return edgesData
      .filter((e) => nodeIdSet.has(e.from_id) && nodeIdSet.has(e.to_id))
      .map((e, i) => {
        const sourceNode = nodeMap.get(e.from_id)
        const targetNode = nodeMap.get(e.to_id)
        const handles =
          sourceNode && targetNode && sourceNode.position != null && targetNode.position != null
            ? getBestHandlePair(
                { position: sourceNode.position, width: sourceNode.width, height: sourceNode.height },
                { position: targetNode.position, width: targetNode.width, height: targetNode.height }
              )
            : null
        return {
          id: `e-${i}-${e.from_id}-${e.to_id}`,
          source: e.from_id,
          target: e.to_id,
          ...(handles ? { sourceHandle: handles.sourceHandle, targetHandle: handles.targetHandle } : {}),
          label: e.label,
          labelStyle: { fill: '#1677ff', fontSize: 10 },
          labelBgStyle: { fill: '#fff', fillOpacity: 1 },
          labelBgPadding: [6, 4] as [number, number],
          labelBgBorderRadius: 4,
          type: 'default' as const,
          markerEnd: { type: MarkerType.ArrowClosed },
        }
      })
  }, [edgesData, nodeIdSet, flowNodes])

  const [nodes, setNodes, onNodesChange] = useNodesState(flowNodes)
  const layoutKeyRef = useRef<string>('')
  const layoutKey = nodeIds.join(',') + edgeListForLayout.length
  useEffect(() => {
    if (layoutKeyRef.current !== layoutKey) {
      layoutKeyRef.current = layoutKey
      setNodes(flowNodes)
    }
  }, [layoutKey, flowNodes, setNodes])

  return (
    <div style={{ maxWidth: 1000, margin: '0 auto', padding: 24 }}>
      <Card
        title="视位置变换图"
        extra={
          <Text type="secondary" style={{ fontSize: 12 }}>
            框 = 静态概念（标架·坐标系·历元）；连线 = 动作（变换步骤）。cost 仅见下表。
          </Text>
        }
      >
        <div style={{ height: 680, marginBottom: 16, background: '#fafafa', borderRadius: 8 }}>
          <ReactFlow
            nodes={nodes}
            edges={flowEdges}
            onNodesChange={onNodesChange}
            nodeTypes={nodeTypes}
            fitView
            fitViewOptions={{ padding: 0.25 }}
            nodesDraggable
            nodesConnectable={false}
            elementsSelectable
            panOnDrag
            zoomOnScroll
            zoomOnPinch
            defaultEdgeOptions={{
              type: 'default',
              animated: false,
              markerEnd: { type: MarkerType.ArrowClosed },
            }}
          >
            <Background color="#e8e8e8" gap={12} />
            <Controls showInteractive={false} />
          </ReactFlow>
        </div>
        <div>
          <Text type="secondary" style={{ fontSize: 12 }}>边列表：步骤=线上变换内容，代价=路径搜索用。部分步骤在管线中合并为一步（如光行时与赤道拟合、岁差/章动等），详见文档。</Text>
          <table style={{ width: '100%', marginTop: 8, borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #f0f0f0' }}>
                <th style={{ textAlign: 'left', padding: '8px 12px', fontWeight: 600 }}>从</th>
                <th style={{ textAlign: 'left', padding: '8px 12px', fontWeight: 600 }}>到</th>
                <th style={{ textAlign: 'left', padding: '8px 12px', fontWeight: 600 }}>步骤</th>
                <th style={{ textAlign: 'right', padding: '8px 12px', fontWeight: 600 }}>代价</th>
              </tr>
            </thead>
            <tbody>
              {edgesData.map((e, i) => (
                <tr key={i} style={{ borderBottom: '1px solid #fafafa' }}>
                  <td style={{ padding: '8px 12px' }}>{e.from_id}</td>
                  <td style={{ padding: '8px 12px' }}>{e.to_id}</td>
                  <td style={{ padding: '8px 12px', color: '#1677ff' }}>{e.label}</td>
                  <td style={{ padding: '8px 12px', textAlign: 'right' }}>{e.cost}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  )
}
