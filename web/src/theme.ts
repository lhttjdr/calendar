/**
 * 明暗主题：跟随系统（缺省）、浅色、深色。所选记于本机；解析后的明暗写到 <html data-theme>，
 * 供 app.css 的颜色记号与 antd 算法共用（doc/engineering/ui-interaction.md 第五期）。
 * 盘面（罗盘、九宫、命盘）按纸色绘，深色下仍作纸面，不反色；星图本为夜空色。
 */
import { useEffect, useState } from 'react'

export type ThemeMode = 'auto' | 'light' | 'dark'
const KEY = 'theme-mode'
const EVT = 'theme-mode-change'

export function getThemeMode(): ThemeMode {
  try {
    const v = localStorage.getItem(KEY)
    return v === 'light' || v === 'dark' ? v : 'auto'
  } catch {
    return 'auto'
  }
}

export function setThemeMode(m: ThemeMode) {
  try {
    if (m === 'auto') localStorage.removeItem(KEY)
    else localStorage.setItem(KEY, m)
  } catch { /* 只是不记住 */ }
  window.dispatchEvent(new Event(EVT))
}

const systemDark = () => typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches

/** 当前模式与解析后是否深色；同步 <html data-theme> */
export function useTheme(): { mode: ThemeMode; dark: boolean } {
  const [mode, setMode] = useState<ThemeMode>(getThemeMode)
  const [sys, setSys] = useState(systemDark)
  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const onSys = () => setSys(mq.matches)
    const onMode = () => setMode(getThemeMode())
    mq.addEventListener('change', onSys)
    window.addEventListener(EVT, onMode)
    return () => { mq.removeEventListener('change', onSys); window.removeEventListener(EVT, onMode) }
  }, [])
  const dark = mode === 'dark' || (mode === 'auto' && sys)
  useEffect(() => {
    document.documentElement.dataset.theme = dark ? 'dark' : 'light'
    document.documentElement.style.colorScheme = dark ? 'dark' : 'light'
  }, [dark])
  return { mode, dark }
}
