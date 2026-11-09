/**
 * 取舍规则面板（目录驱动，统一样式）：每一项都是同一种「芯片」——亮为参与判定、暗为不用；
 * 暗为不用（协纪诸神缺省亮，通书诸说缺省暗）；与缺省不同者外加黄框。多选一者为同高的分段钮。点芯片右侧的 ⓘ 在右侧抽屉看该项各家之说（书、卷篇、立否、起例、所主、原文），各家并列，不以一家为裁断。
 * 设计见 doc/engineering/ui-interaction.md「取舍规则面板」。
 */
import { useState, type ReactNode } from 'react'
import { Button, Descriptions, Drawer, Flex, Typography } from 'antd'
import { InfoCircleOutlined } from '@ant-design/icons'

const { Text } = Typography

/** 一家之说：书、卷篇、立否（立、不载、斥、删、改、异说）、此书之起例、所主、原文 */
export type RuleView = { book: string; cite?: string; stance?: string; rule?: string; effect?: string; quote?: string }
/** 一项之说明：各家并列（核心目录 RULE_CATALOG / hun_shen_info_json 供），不以一家为裁断 */
export type RuleInfo = { name: string; group?: string; views: RuleView[]; extra?: ReactNode }

/** 开关项：on 为参与判定；tone 定色（宜神绿、忌神红、余为中性） */
export type ToggleItem = { kind: 'toggle'; key: string; name: string; on: boolean; onChange: (on: boolean) => void; tone?: 'yi' | 'ji' | 'plain'; info?: RuleInfo; changed?: boolean }
/** 多选一项 */
export type ChoiceItem = { kind: 'choice'; key: string; name: string; value: number; options: { label: string; value: number }[]; onChange: (v: number) => void; info?: RuleInfo; changed?: boolean }
export type RuleItem = ToggleItem | ChoiceItem

export type RuleGroup = {
  key: string
  title: string
  /** 组下一句说明（取舍之义） */
  note?: string
  items: RuleItem[]
  /** 组级动作（全用、全不用、缺省） */
  actions?: { label: string; onClick: () => void }[]
}

function Chip({ it, onInfo }: { it: ToggleItem; onInfo: () => void }) {
  return (
    <span className={`rule-chip rule-${it.tone ?? 'plain'}${it.on ? ' on' : ''}${it.changed ? ' changed' : ''}`}>
      <button type="button" className="rule-chip-main" aria-pressed={it.on} onClick={() => it.onChange(!it.on)}
        title={it.on ? '参与判定（点之不用）' : '不用（点之参与判定）'}>
        {it.name}
      </button>
      {it.info && (
        <button type="button" className="rule-chip-info" aria-label={`${it.name}之说明`} onClick={onInfo}>
          <InfoCircleOutlined />
        </button>
      )}
    </span>
  )
}

function Choice({ it, onInfo }: { it: ChoiceItem; onInfo: () => void }) {
  return (
    <span className={`rule-choice${it.changed ? ' changed' : ''}`}>
      <span className="rule-choice-name">
        {it.name}
        {it.info && (
          <button type="button" className="rule-chip-info" aria-label={`${it.name}之说明`} onClick={onInfo}>
            <InfoCircleOutlined />
          </button>
        )}
      </span>
      <span className="rule-seg" role="radiogroup" aria-label={it.name}>
        {it.options.map((o) => (
          <button key={o.value} type="button" role="radio" aria-checked={it.value === o.value}
            className={it.value === o.value ? 'on' : ''} onClick={() => it.onChange(o.value)}>
            {o.label}
          </button>
        ))}
      </span>
    </span>
  )
}

export function RulePanel(props: { groups: RuleGroup[] }) {
  const [info, setInfo] = useState<RuleInfo | null>(null)
  return (
    <div className="rule-panel">
      {props.groups.filter((g) => g.items.length > 0).map((g) => (
        <section key={g.key} className="rule-group">
          <Flex align="baseline" gap={8} wrap>
            <Text strong style={{ fontSize: 13 }}>{g.title}</Text>
            {g.note && <Text type="secondary" style={{ fontSize: 12 }}>{g.note}</Text>}
            {g.actions?.map((a) => (
              <Button key={a.label} size="small" type="link" style={{ padding: 0, height: 'auto', fontSize: 12 }} onClick={a.onClick}>{a.label}</Button>
            ))}
          </Flex>
          <div className="rule-items">
            {g.items.map((it) => (it.kind === 'toggle'
              ? <Chip key={it.key} it={it} onInfo={() => setInfo(it.info ?? null)} />
              : <Choice key={it.key} it={it} onInfo={() => setInfo(it.info ?? null)} />))}
          </div>
        </section>
      ))}
      <Drawer open={info != null} onClose={() => setInfo(null)} title={info?.name} placement="right" width={420}>
        {info && (
          <Flex vertical gap={12}>
            {info.group && <Text type="secondary" style={{ fontSize: 12 }}>{info.group}</Text>}
            {info.views.length === 0 && <Text type="secondary">暂无说明</Text>}
            {info.views.map((v, i) => (
              <Descriptions key={i} column={1} size="small" bordered
                title={<span>{v.book}{v.stance ? <Text type="secondary" style={{ fontSize: 12, fontWeight: 400 }}>{`　${v.stance}`}</Text> : null}</span>}>
                {v.cite ? <Descriptions.Item label="出处">{v.cite}</Descriptions.Item> : null}
                {v.rule ? <Descriptions.Item label="起例">{v.rule}</Descriptions.Item> : null}
                {v.effect ? <Descriptions.Item label="所主">{v.effect}</Descriptions.Item> : null}
                {v.quote ? <Descriptions.Item label="原文">{v.quote}</Descriptions.Item> : null}
              </Descriptions>
            ))}
          </Flex>
        )}
        {info?.extra && <div style={{ marginTop: 12 }}>{info.extra}</div>}
      </Drawer>
    </div>
  )
}
