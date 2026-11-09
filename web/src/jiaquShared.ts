/**
 * 婚嫁择吉的输入与取舍规则：择日页设、日历「择吉」图层读，同取本机所存（localStorage，键与 fold.tsx 的 useOpt 同式）。
 * 两页因此按同一套条件判定（doc/engineering/ui-interaction.md 第三期）。
 */
import type { LunarBackend } from './lunar-backend-types'

export type MonthLunar = { jd0: number; months: number[]; days: number[]; leaps: number[]; dims: number[]; phases: number[] }
export type JiaquDay = {
  jdn: number; ok: boolean; verdict?: string; jie?: [string, string][]; yi: boolean; ji: boolean; jieHunYin: boolean; naCai: boolean; nvMingYue: string
  zhouTang: string | null; baiHu: string | null; naXu: string | null; buJiang: boolean; jiang: string
  leibian: [string, string][]; liuRen?: string; wuRi?: string | null; jiri?: [string, string][]; reasons: string[]; notes: string[]
  /** 协纪月表嫁娶宜忌之依据：月表（铺注前）与铺注后之注、所值卷十一嫁娶宜神忌神、卷十等第、改注之铺注 */
  xj?: { yuebiao: string; final: string; yiShen: string[]; jiShen: string[]; dengDi: string; puzhu: string[] }
}

/** 输入（女命生年、翁姑等）与规则；缺省与择日页同 */
export const JIAQU_DEFAULTS = {
  nvYear: 2000, wengGu: true, fuMu: true, naXu: false,
  useYue: false, xieji: 0, zt: 0, useXiong: false, zhiHua: false, aoTou: false, chiKou: 0, hengTian: false,
  /** 协纪推法中不看之神，以「、」连名（名见 hun_shen_list_json）；空即全依协纪月表 */
  ignore: '',
  /** 类编凶神逐神否决（位掩码，位义见 leibian_shen_list_json）；0 则依 useXiong（缺省所列或全不用） */
  leibianMask: 0,
  /** 俗说诸忌（协纪卷三十六斥之）：楊公忌、天狗（頭尾口腹）、章光否决；紅沙 0 不用 1 轉神厯 2 又一本 */
  yangGong: false, tianGou: false, zhangGuang: false, hongSha: 0,
}
export type JiaquParams = typeof JIAQU_DEFAULTS
/** 输入之组名（不入起例摘要）与规则之组名（起例区「取舍规则」） */
export const JIAQU_IN_NS = 'jiaquIn'
export const JIAQU_RULE_NS = 'jiaqu'
const IN_KEYS = new Set(['nvYear', 'wengGu', 'fuMu', 'naXu'])

export function readJiaquParams(): JiaquParams {
  const p = { ...JIAQU_DEFAULTS } as unknown as Record<string, number | boolean | string>
  for (const k of Object.keys(JIAQU_DEFAULTS)) {
    try {
      const s = localStorage.getItem(`opt:${IN_KEYS.has(k) ? JIAQU_IN_NS : JIAQU_RULE_NS}/${k}`)
      if (s == null) continue
      const v = JSON.parse(s) as unknown
      if (typeof v === typeof p[k]) p[k] = v as number | boolean | string | string
    } catch { /* 读不到即用缺省 */ }
  }
  return p as unknown as JiaquParams
}

export function jiaquFlags(p: JiaquParams): number {
  return (p.useYue ? 1 : 0) | (p.zt === 1 ? 2 : p.zt === 2 ? 128 : 0) | (p.xieji === 1 ? 32 : p.xieji === 2 ? 64 : 0) | (p.useXiong ? 4 : 0)
    | (p.zhiHua ? 8 : 0) | (p.aoTou ? 16 : 0) | (p.chiKou === 1 ? 256 : p.chiKou === 2 ? 512 : p.chiKou === 3 ? 2048 : 0) | (p.hengTian ? 1024 : 0)
    | (p.yangGong ? 4096 : 0) | (p.hongSha === 1 ? 8192 : p.hongSha === 2 ? 16384 : 0) | (p.tianGou ? 32768 : 0) | (p.zhangGuang ? 65536 : 0)
}

export const nvGzOf = (year: number) => (((year - 4) % 60) + 60) % 60

/** 婚事宜神、忌神（与「特定日」）之表：[名, 是否宜神] */
export function hunShenList(wasm: LunarBackend): [string, boolean][] {
  if (typeof wasm.hun_shen_list_json !== 'function') return []
  try { return JSON.parse(wasm.hun_shen_list_json()) as [string, boolean][] } catch { return [] }
}

/** 类编嫁娶凶神：[名, 缺省是否否决] */
export function leibianShenList(wasm: LunarBackend): [string, boolean][] {
  if (typeof wasm.leibian_shen_list_json !== 'function') return []
  try { return JSON.parse(wasm.leibian_shen_list_json()) as [string, boolean][] } catch { return [] }
}

/** 不看之神 → 位掩码（低、高 32 位） */
export function ignoreMask(wasm: LunarBackend, ignore: string): [number, number] {
  const set = new Set(ignore.split('、').filter(Boolean))
  let lo = 0, hi = 0
  hunShenList(wasm).forEach(([n], i) => {
    if (!set.has(n)) return
    if (i < 32) lo = (lo | (1 << i)) >>> 0
    else hi = (hi | (1 << (i - 32))) >>> 0
  })
  return [lo, hi]
}

/** 一月逐日判定；wasm 无此函数返回 null，出错抛出 */
export function jiaquMonth(wasm: LunarBackend, ear: string, m: MonthLunar, p: JiaquParams): JiaquDay[] | null {
  if (typeof wasm.jiaqu_days_json !== 'function') return null
  return JSON.parse(wasm.jiaqu_days_json(m.jd0, ear, Uint8Array.from(m.months), Uint8Array.from(m.days), Uint8Array.from(m.leaps), Uint8Array.from(m.dims), Int8Array.from(m.phases),
    nvGzOf(p.nvYear), p.wengGu, p.fuMu, p.naXu, jiaquFlags(p), ...ignoreMask(wasm, p.ignore), p.leibianMask >>> 0)) as JiaquDay[]
}
