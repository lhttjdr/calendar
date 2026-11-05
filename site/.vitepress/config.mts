import { defineConfig } from 'vitepress'
import cjkFriendly from 'markdown-it-cjk-friendly'

// 本站是面向读者的文档（site/），与仓库 doc/ 下供开发与 AI 使用的知识库分开，doc/ 不发布。
// GitHub Pages 项目站为 /<repo>/docs/，通过环境变量 DOCS_BASE 传入（如 /calendar/docs/）
const base = process.env.DOCS_BASE ?? '/'
// 应用站（文档站上一级）须给完整 URL：站内绝对路径会被 VitePress 再加一次 base；本地缺省指向 web 的 vite dev
const appUrl = process.env.APP_URL ?? 'http://localhost:5173/'

export default defineConfig({
  base,
  lang: 'zh-Hans',
  title: '农历的天文计算',
  description: '从日月运行算出合朔与节气，排出农历；按历代术文复原古代历日，并以史书检验',
  cleanUrls: true,
  markdown: {
    math: true,
    // 中文标点紧贴 ** 时（如「**世界时（UT）**逐渐」）CommonMark 不认粗体，此插件按 CJK 放宽定界规则
    config: (md) => {
      md.use(cjkFriendly)
    },
  },
  themeConfig: {
    outline: { level: [2, 3], label: '本页目录' },
    docFooter: { prev: '上一章', next: '下一章' },
    search: { provider: 'local' },
    nav: [
      { text: '天文篇', link: '/astronomy/sphere' },
      { text: '历法篇', link: '/calendar/rules' },
      { text: '在线日历', link: appUrl, target: '_self' },
      { text: 'GitHub', link: 'https://github.com/lhttjdr/calendar' },
    ],
    sidebar: [
      {
        text: '天文篇',
        items: [
          { text: '天球与坐标', link: '/astronomy/sphere' },
          { text: '时间：儒略日与各种时标', link: '/astronomy/time' },
          { text: '岁差', link: '/astronomy/precession' },
          { text: '章动', link: '/astronomy/nutation' },
          { text: '历元、分点与参考架', link: '/astronomy/frames' },
          { text: '转动的表示', link: '/astronomy/rotations' },
          { text: '历表：太阳和月亮在哪里', link: '/astronomy/ephemerides' },
          { text: '视位置', link: '/astronomy/apparent' },
          { text: '合朔与月相', link: '/astronomy/syzygy' },
          { text: '二十四节气', link: '/astronomy/solar-terms' },
          { text: '日月食与其他天象', link: '/astronomy/phenomena' },
        ],
      },
      {
        text: '历法篇',
        items: [
          { text: '农历怎样编排', link: '/calendar/rules' },
          { text: '历史上实际用过的历日', link: '/calendar/historical' },
          { text: '古历的推步与复原', link: '/calendar/ancient' },
        ],
      },
    ],
  },
})
