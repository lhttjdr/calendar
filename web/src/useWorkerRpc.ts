/**
 * 与计算 Worker 的请求/应答：每个请求带自增 id，Worker 回 {id, result} 或 {id, error}。
 * `create` 须在调用处写成 `() => new Worker(new URL('./xxx.ts', import.meta.url), { type: 'module' })`，Vite 才能打包该 Worker。
 */
import { useCallback, useEffect, useRef } from 'react'

/** 对联合类型逐支去掉 id（内置 Omit 会把联合压成公共字段）。 */
export type WithoutId<R> = R extends unknown ? Omit<R, 'id'> : never

export function useWorkerRpc<Req extends { id: number }>(create: () => Worker) {
  const workerRef = useRef<Worker | null>(null)
  const pending = useRef(new Map<number, { resolve: (v: unknown) => void; reject: (e: Error) => void }>())
  const nextId = useRef(1)
  useEffect(() => {
    const w = create()
    w.onmessage = (ev: MessageEvent<{ id: number; result?: unknown; error?: string }>) => {
      const p = pending.current.get(ev.data.id)
      if (!p) return
      pending.current.delete(ev.data.id)
      if (ev.data.error) p.reject(new Error(ev.data.error))
      else p.resolve(ev.data.result)
    }
    workerRef.current = w
    return () => w.terminate()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])
  return useCallback(<T,>(req: WithoutId<Req>): Promise<T> => {
    const id = nextId.current++
    return new Promise<T>((resolve, reject) => {
      pending.current.set(id, { resolve: resolve as (v: unknown) => void, reject })
      workerRef.current!.postMessage({ ...req, id })
    })
  }, [])
}
