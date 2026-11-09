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
  /** 所值协纪改删旧说与类编、鳌头、董公诸神（名见 tongshu_shen_list_json）；董公婚姻宜忌；鳌头婚姻四表所列 */
  tongshu?: string[]; dongGong?: string[]; aotou?: string[]
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
  /** 协纪改删旧说与类编、鳌头、董公诸神逐神否决（位掩码，位义见 tongshu_shen_list_json）；0 只作附注 */
  tongshuMask: 0,
  /** 协纪铺注婚事诸日逐项不看（位掩码，位义见 puzhu_hun_list_json）；0 即时宪书体例 */
  puzhuSkip: 0,
  /** 董公逐日婚姻宜忌：0 只作附注 1 忌婚姻否决 2 须宜婚姻 */
  dongGong: 0,
  /** 须见鳌头嫁娶或備急表 */
  aotouJiri: false,
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
    | (p.dongGong === 1 ? 131072 : p.dongGong === 2 ? 262144 : 0) | (p.aotouJiri ? 524288 : 0)
}

/** 协纪改删旧说与类编、鳌头、董公诸神：[名, 组, 所主与协纪之断] */
export function tongshuShenList(wasm: LunarBackend): [string, string, string][] {
  if (typeof wasm.tongshu_shen_list_json !== 'function') return []
  try { return JSON.parse(wasm.tongshu_shen_list_json()) as [string, string, string][] } catch { return [] }
}

/** 铺注婚事诸日之名（下标即 puzhuSkip 位） */
export function puzhuHunList(wasm: LunarBackend): string[] {
  if (typeof wasm.puzhu_hun_list_json !== 'function') return []
  try { return JSON.parse(wasm.puzhu_hun_list_json()) as string[] } catch { return [] }
}

/** 一书之说：书、卷篇、立/不载/斥/删/改/异说、起例、所主所忌、原文摘录 */
export type RuleView = { book: string; cite: string; stance: string; rule: string; effect: string; quote: string }
/** 可取舍诸项之说明（核心 RULE_CATALOG）：key 与本文件参数名一致，类编各神 `lb:名`、诸神 `ts:名`、铺注 `pz:名`；协纪婚事诸神 `xj:名` */
export type RuleInfo = { key: string; name: string; group: string; views: RuleView[] }
export function jiaquRuleCatalog(wasm: LunarBackend): RuleInfo[] {
  if (typeof wasm.jiaqu_rule_catalog_json !== 'function') return []
  try { return JSON.parse(wasm.jiaqu_rule_catalog_json()) as RuleInfo[] } catch { return [] }
}

/** 协纪婚事诸神（HUN_SHEN 之序）之说明：group 为「宜」「忌」 */
export function hunShenInfo(wasm: LunarBackend): RuleInfo[] {
  if (typeof wasm.hun_shen_info_json !== 'function') return []
  try { return JSON.parse(wasm.hun_shen_info_json()) as RuleInfo[] } catch { return [] }
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
    nvGzOf(p.nvYear), p.wengGu, p.fuMu, p.naXu, jiaquFlags(p), ...ignoreMask(wasm, p.ignore), p.leibianMask >>> 0,
    p.tongshuMask >>> 0, p.puzhuSkip >>> 0)) as JiaquDay[]
}
