/**
 * 「结论→依据→设置」三层呈现的两件公用部件（doc/engineering/ui-interaction.md 第四节）：
 * - 起例区 OptionsFold：平时只显示一行摘要（缺省 / 已改几项 · 恢复缺省），点开才见逐项选项；
 *   选项用 useOpt 声明，记住各人所选（localStorage），并据缺省值数出「已改几项」。
 * - 说明区 Notes：长段说明与出处，缺省收起。
 */
import { useEffect, useState, type Dispatch, type ReactNode, type SetStateAction } from 'react'
import { Button, Typography } from 'antd'
import { DownOutlined, RightOutlined } from '@ant-design/icons'

const { Text } = Typography

type Item = { label: string; changed: boolean; reset: () => void }
/** 一组选项（一个面板的起例）。每次渲染新建，useOpt 在渲染时登记 */
export type OptGroup = { ns: string; items: Item[] }

export function useOptGroup(ns: string): OptGroup {
  return { ns, items: [] }
}

function read<T>(key: string, init: T): T {
  try {
    const s = localStorage.getItem(key)
    if (s == null) return init
    const v = JSON.parse(s) as unknown
    return typeof v === typeof init || (init === null && (typeof v === 'number' || v === null)) ? (v as T) : init
  } catch {
    return init
  }
}

/**
 * 声明一项起例选项：同 useState，另登记于组（数「已改」、可一键恢复缺省），并按 `组/name` 记住所选。
 * `label` 用于摘要提示（已改之项名）。
 */
export function useOpt<T>(g: OptGroup, name: string, init: T, label = name): [T, Dispatch<SetStateAction<T>>] {
  const key = `opt:${g.ns}/${name}`
  const [v, set] = useState<T>(() => read(key, init))
  useEffect(() => {
    try {
      if (v === init) localStorage.removeItem(key)
      else localStorage.setItem(key, JSON.stringify(v))
    } catch { /* 无痕模式等：只是不记住 */ }
  }, [key, v, init])
  g.items.push({ label, changed: v !== init, reset: () => set(init) })
  return [v, set]
}

function useStoredFlag(key: string, init: boolean): [boolean, (v: boolean) => void] {
  const [v, set] = useState(() => read(key, init))
  return [v, (x: boolean) => { set(x); try { localStorage.setItem(key, JSON.stringify(x)) } catch { /* 同上 */ } }]
}

/** 起例区：标题行「起例 · 缺省（…） / 已改 N 项：…  恢复缺省」，点开见选项 */
export function OptionsFold(props: { g: OptGroup; title?: string; defaultNote?: string; children: ReactNode }) {
  const [open, setOpen] = useStoredFlag(`fold:${props.g.ns}`, false)
  const changed = props.g.items.filter((i) => i.changed)
  return (
    <div className="opt-fold">
      <div className="opt-fold-head">
        <button type="button" className="opt-fold-toggle" onClick={() => setOpen(!open)} aria-expanded={open}>
          {open ? <DownOutlined /> : <RightOutlined />}
          <span className="opt-fold-title">{props.title ?? '起例'}</span>
          {changed.length === 0
            ? <Text type="secondary" style={{ fontSize: 12 }}>{`缺省${props.defaultNote ? `（${props.defaultNote}）` : ''}`}</Text>
            : <Text style={{ fontSize: 12, color: '#d46b08' }} title={changed.map((i) => i.label).join('、')}>{`已改 ${changed.length} 项：${changed.map((i) => i.label).join('、')}`}</Text>}
        </button>
        {changed.length > 0 && (
          <Button size="small" type="link" onClick={() => props.g.items.forEach((i) => i.reset())}>恢复缺省</Button>
        )}
      </div>
      {open && <div className="opt-fold-body">{props.children}</div>}
    </div>
  )
}

/** 说明与出处：缺省收起 */
export function Notes(props: { children: ReactNode; title?: string }) {
  const [open, setOpen] = useState(false)
  return (
    <div className="notes">
      <button type="button" className="notes-toggle" onClick={() => setOpen(!open)} aria-expanded={open}>
        {open ? <DownOutlined /> : <RightOutlined />} {props.title ?? '说明与出处'}
      </button>
      {open && <div className="notes-body">{props.children}</div>}
    </div>
  )
}
