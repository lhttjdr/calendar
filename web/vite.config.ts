import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// GitHub Pages 项目站为 /<repo>/，通过环境变量 VITE_BASE 传入（如 /calendar/）
const base = process.env.VITE_BASE ?? '/'

export default defineConfig({
  base,
  plugins: [react()],
  worker: {
    format: 'es',
  },
  server: {
    fs: {
      allow: ['..'],
    },
  },
  optimizeDeps: {
    exclude: ['lunar-wasm'],
  },
})
