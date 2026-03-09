/**
 * 加载农历后端（Rust WASM）。数据从 /data/ 下 fetch 后由 compute_year_data_wasm 现算。
 * variant: 'f64' = Real=f64（默认，体积小、速度快），'twofloat' = Real 双字浮点（高精度）。
 */
import type { LunarBackend } from './lunar-backend-types'

export type RealBackendVariant = 'twofloat' | 'f64'

export async function loadLunarBackend(variant: RealBackendVariant = 'f64'): Promise<LunarBackend> {
  const mod = variant === 'f64'
    ? await import('lunar-wasm-f64')
    : await import('lunar-wasm')
  if (typeof mod.default === 'function') await mod.default()
  return mod as unknown as LunarBackend
}
