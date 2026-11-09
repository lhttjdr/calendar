import React from 'react'
import ReactDOM from 'react-dom/client'
import { ConfigProvider, theme } from 'antd'
import zhCN from 'antd/locale/zh_CN'
import App from './App'
import { useTheme } from './theme'
import './app.css'

/** 明暗主题：antd 算法随 <html data-theme>（theme.ts） */
function Root() {
  const { dark } = useTheme()
  return (
    <ConfigProvider locale={zhCN} getPopupContainer={() => document.body} theme={{ algorithm: dark ? theme.darkAlgorithm : theme.defaultAlgorithm }}>
      <App />
    </ConfigProvider>
  )
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
)
