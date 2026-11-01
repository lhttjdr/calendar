/**
 * 自动播放：每画完一帧、再等一个间隔才推进下一帧，所以计算跟不上时只是变慢，不会积压请求。
 * useFrameTimer 是星图与日月食共用的节拍；useEclipsePlayback 是日月食的播放状态（在初亏到复圆之间走时）。
 */
import { useEffect, useRef, useState } from 'react'
import { Button, Flex, Select, Slider, Typography } from 'antd'

const { Text } = Typography

export const SPEEDS = [
  { label: '慢', value: 800 },
  { label: '中', value: 300 },
  { label: '快', value: 60 },
]

/** 播放中，frame 每次更新（即新一帧已到手）后过 intervalMs 调一次 advance。 */
export function useFrameTimer(playing: boolean, frame: unknown, intervalMs: number, advance: () => void) {
  const latest = useRef(advance)
  latest.current = advance
  useEffect(() => {
    if (!playing || frame == null) return
    const t = setTimeout(() => latest.current(), intervalMs)
    return () => clearTimeout(t)
  }, [playing, frame, intervalMs])
}

export function PlayButton(props: { playing: boolean; onClick: () => void }) {
  return (
    <Button size="small" type="primary" onClick={props.onClick}>
      {props.playing ? '❚❚ 暂停' : '▶ 播放'}
    </Button>
  )
}

const ECLIPSE_STEPS = [
  { label: '1 分', value: 1 },
  { label: '2 分', value: 2 },
  { label: '5 分', value: 5 },
  { label: '10 分', value: 10 },
  { label: '30 分', value: 30 },
]

/** 在 range（TT 儒略日）内走时；range 变了就回到未播放。fetchFrame 由 Rust 算一帧。 */
export function useEclipsePlayback<F>(range: [number, number] | null, fetchFrame: (t: number) => Promise<F>) {
  const [t, setT] = useState<number | null>(null)
  const [frame, setFrame] = useState<F | null>(null)
  const [playing, setPlaying] = useState(false)
  const [stepMin, setStepMin] = useState(5)
  const [interval, setIntervalMs] = useState(SPEEDS[1].value)
  const fetcher = useRef(fetchFrame)
  fetcher.current = fetchFrame
  const [a, b] = range ?? [0, 0]

  useEffect(() => {
    setT(null)
    setFrame(null)
    setPlaying(false)
  }, [a, b])

  useEffect(() => {
    if (t == null) return
    let cancelled = false
    fetcher.current(t)
      .then((f) => !cancelled && setFrame(f))
      .catch(() => !cancelled && setPlaying(false))
    return () => {
      cancelled = true
    }
  }, [t])

  useFrameTimer(playing, frame, interval, () => {
    if (t == null || t >= b) setPlaying(false)
    else setT(Math.min(b, t + stepMin / 1440))
  })

  const toggle = () => {
    if (!playing && (t == null || t >= b)) setT(a)
    setPlaying(!playing)
  }
  const reset = () => {
    setPlaying(false)
    setT(null)
    setFrame(null)
  }
  return { t, frame, playing, stepMin, setStepMin, interval, setIntervalMs, toggle, reset, setT, setPlaying, a, b }
}

export function EclipsePlaybackBar(props: { pb: Omit<ReturnType<typeof useEclipsePlayback<unknown>>, 'frame'>; label: string }) {
  const { pb, label } = props
  return (
    <Flex gap={10} align="center" wrap="wrap">
      <PlayButton playing={pb.playing} onClick={pb.toggle} />
      <Text>步长</Text>
      <Select size="small" style={{ width: 80 }} value={pb.stepMin} onChange={pb.setStepMin} options={ECLIPSE_STEPS} />
      <Text>速度</Text>
      <Select size="small" style={{ width: 66 }} value={pb.interval} onChange={pb.setIntervalMs} options={SPEEDS} />
      <div style={{ flex: '1 1 200px', minWidth: 160 }}>
        <Slider min={pb.a} max={pb.b} step={1 / 8640} value={pb.t ?? pb.a} tooltip={{ open: false }}
          onChange={(v) => { pb.setPlaying(false); pb.setT(v) }} />
      </div>
      <Text style={{ fontVariantNumeric: 'tabular-nums' }}>{label}</Text>
      {pb.t != null && <Button size="small" onClick={pb.reset}>退出演示</Button>}
    </Flex>
  )
}
