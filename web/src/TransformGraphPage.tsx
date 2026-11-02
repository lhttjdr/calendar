/**
 * 视位置变换图展示页：由 WASM 取得 TransformGraph 的节点与边，用 React Flow 绘制有向图。
 * 图是网络：互逆的两条边合并成一条双向箭头画，布局允许有环（graphLayout.ts）。
 */
import { useMemo, useEffect, useRef } from 'react'
import { ReactFlow, Background, Controls, Handle, Position, MarkerType, useNodesState, type Node, type Edge, type NodeProps } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { stressLayout } from './graphLayout'
import { Card, Typography } from 'antd'
import type { LunarBackend } from './lunar-backend-types'

const { Text } = Typography

/** WASM 返回的变换图可视化数据。边可能为 getter 方法（from_id()/to_id()/label()）或属性。 */
type GraphEdgeList = readonly (
  | { from_id: string; to_id: string; cost: number; label?: string | null }
  | { from_id: () => string; to_id: () => string; cost: number; label?: () => string | null }
)[]
interface TransformGraphViz {
  readonly nodeIds: string[] | (() => string[])
  readonly edges: GraphEdgeList | (() => GraphEdgeList)
}

function edgeFromId(e: GraphEdgeList[number] | null | undefined): string {
  if (e == null) return ''
  return typeof (e as { from_id: string | (() => string) }).from_id === 'function'
    ? (e as { from_id: () => string }).from_id()
    : (e as { from_id: string }).from_id
}
function edgeToId(e: GraphEdgeList[number] | null | undefined): string {
  if (e == null) return ''
  return typeof (e as { to_id: string | (() => string) }).to_id === 'function'
    ? (e as { to_id: () => string }).to_id()
    : (e as { to_id: string }).to_id
}
function edgeCost(e: GraphEdgeList[number] | null | undefined): number {
  if (e == null) return 0
  return (e as { cost: number }).cost
}
function edgeLabel(e: GraphEdgeList[number] | null | undefined): string | null {
  if (e == null) return null
  try {
    const l = (e as { label?: string | null | (() => string | null) }).label
    if (l == null) return null
    // wasm 的 label 是方法，必须带 this 调用
    const v = typeof l === 'function' ? l.call(e) ?? null : l ?? null
    return v != null && String(v).trim() !== '' ? String(v) : null
  } catch {
    return null
  }
}

/** 节点宽高；布局间距单独设大一些，保证连线足够长、线上文字可显示且框不压线 */
const NODE_WIDTH = 210
const NODE_HEIGHT = 60
const PAD = 32
/** 一跳的屏幕距离要够长，线上文字才放得下；框与框的最小间距 */
const LAYOUT_UNIT = 200
const LAYOUT_GAP = 30
/** 箭头放大到缩放后仍看得出方向；双向边两端都画。 */
const EDGE_COLOR = '#8c8c8c'
const ARROW = { type: MarkerType.ArrowClosed, width: 20, height: 20, color: EDGE_COLOR }

function getGraphData(wasm: LunarBackend & { transformGraphVisualizationData?: () => TransformGraphViz }): TransformGraphViz | null {
  if (typeof (wasm as { transformGraphVisualizationData?: () => TransformGraphViz }).transformGraphVisualizationData !== 'function') {
    return null
  }
  return (wasm as { transformGraphVisualizationData: () => TransformGraphViz }).transformGraphVisualizationData()
}

const HELIOCENTRIC_MEAN_ECLIPTIC = '日心_MeanEcliptic(J2000)'

/** 框内：标架 + 坐标系 + 历元 */
const NODE_BOX_LABELS: Record<string, string> = {
  VSOP87: 'VSOP87（含 DE406/DE441 拟合改正）· J2000 动力学平黄道',
  [HELIOCENTRIC_MEAN_ECLIPTIC]: '日心 · J2000 平黄道',
  ELPMPP02: 'ELPMPP02 · J2000 月心平架',
  ELPMPP02_MEAN_LUNAR: 'ELP 平均根数 · J2000 平黄道',
  'DE406/DE441': 'DE406 / DE441 · JPL 历表',
  Hipparcos: 'Hipparcos · 恒星星表',
  'MeanEcliptic(J2000)': 'MeanEcliptic · J2000 平黄道',
  'MeanEcliptic(epoch)': 'MeanEcliptic · 当日平黄道',
  'TrueEcliptic(epoch)': 'TrueEcliptic · 当日真黄道',
  FK5: 'FK5 · J2000 平赤道',
  ICRS: 'ICRS · 国际天球参考系',
  IcrsAberrated: 'IcrsAberrated · ICRS（含周年光行差）',
  'MeanEquator(epoch)': 'MeanEquator · 历元平赤道',
  'TrueEquator(epoch)': 'TrueEquator · 历元真赤道',
  'ApparentEquator(epoch)': 'ApparentEquator · 视赤道（显式光行差）',
  'ApparentEcliptic(epoch)': 'ApparentEcliptic · 视黄道（光行差已折叠）',
  'TopocentricEquator(epoch)': 'TopocentricEquator · 站心真赤道',
  'TopocentricAberratedEquator(epoch)': 'TopocentricAberratedEquator · 站心真赤道（含周日光行差）',
  'HorizontalGeometric(epoch)': 'HorizontalGeometric · 几何地平（东、北、天顶）',
  'HorizontalApparent(epoch)': 'HorizontalApparent · 视地平（东、北、天顶）',
}

/** 网络布局，允许有环，见 graphLayout.ts */
function getLayoutedNodes(nodes: Node[], edges: { source: string; target: string }[]): Node[] {
  const positions = stressLayout(nodes.map((n) => n.id), edges, { unit: LAYOUT_UNIT, width: NODE_WIDTH, height: NODE_HEIGHT, gap: LAYOUT_GAP })
  return nodes.map((n) => {
    const p = positions.get(n.id) ?? { x: 0, y: 0 }
    return { ...n, position: { x: p.x + PAD, y: p.y + PAD } }
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
type BoxNodeData = Node<{ label?: string }, 'box'>

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
        fontSize: 16,
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
    const rawNodeIds = typeof graphData.nodeIds === 'function'
      ? graphData.nodeIds()
      : Array.isArray(graphData.nodeIds)
        ? graphData.nodeIds
        : []
    nodeIds = Array.isArray(rawNodeIds)
      ? rawNodeIds.map((id) => (id != null && typeof id === 'string' ? id.trim() : String(id).trim()))
      : []
    const rawEdges: GraphEdgeList = typeof graphData.edges === 'function'
      ? graphData.edges()
      : Array.isArray(graphData.edges)
        ? graphData.edges
        : []
    const rawList = Array.isArray(rawEdges) ? rawEdges.filter((e): e is NonNullable<typeof e> => e != null) : []
    edgesData = rawList.map((e) => {
      const fromId = edgeFromId(e).trim()
      const toId = edgeToId(e).trim()
      const wasmLabel = edgeLabel(e)
      const label = wasmLabel != null && String(wasmLabel).trim() !== ''
        ? String(wasmLabel).trim()
        : `${fromId} → ${toId}`
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
  /** 正向在前：遇到已画过的边的反向就并进去，不再单画 */
  const displayEdges = useMemo(() => {
    const forward = new Set<string>()
    const pairs = new Set(edgesData.map((e) => `${e.from_id}\0${e.to_id}`))
    return edgesData.flatMap((e) => {
      if (forward.has(`${e.to_id}\0${e.from_id}`)) return []
      forward.add(`${e.from_id}\0${e.to_id}`)
      return [{ ...e, bidirectional: pairs.has(`${e.to_id}\0${e.from_id}`) }]
    })
  }, [edgesData])
  const edgeListForLayout = useMemo(
    () =>
      displayEdges
        .filter((e) => nodeIdSet.has(e.from_id) && nodeIdSet.has(e.to_id))
        .map((e) => ({ source: e.from_id, target: e.to_id })),
    [displayEdges, nodeIdSet]
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
    return displayEdges
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
          labelStyle: { fill: '#1677ff', fontSize: 14 },
          labelBgStyle: { fill: '#fff', fillOpacity: 1 },
          labelBgPadding: [6, 4] as [number, number],
          labelBgBorderRadius: 4,
          type: 'default' as const,
          style: { stroke: EDGE_COLOR, strokeWidth: 1.5 },
          markerEnd: ARROW,
          ...(e.bidirectional ? { markerStart: { ...ARROW, orient: 'auto-start-reverse' } } : {}),
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
    <div style={{ maxWidth: 1500, margin: '0 auto', padding: 24 }}>
      <Card
        title="视位置变换图"
        extra={
          <Text type="secondary" style={{ fontSize: 12 }}>
            框 = 静态概念（标架·坐标系·历元）；连线 = 动作（变换步骤）。cost 仅见下表。
          </Text>
        }
      >
        <div style={{ height: 760, marginBottom: 16, background: '#fafafa', borderRadius: 8 }}>
          <ReactFlow
            nodes={nodes}
            edges={flowEdges}
            onNodesChange={onNodesChange}
            nodeTypes={nodeTypes}
            fitView
            fitViewOptions={{ padding: 0.08 }}
            minZoom={0.2}
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
