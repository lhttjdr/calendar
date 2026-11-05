import { defineConfig } from 'vitepress'
import cjkFriendly from 'markdown-it-cjk-friendly'
// 天象逐条校验附录按节、按朝代拆页，条目由 scripts/figures/build_site_pages.py 生成
import checksSidebar from './appendix-sidebar.json' with { type: 'json' }

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
      { text: '附录', link: '/appendix/phenomena-checks' },
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
          { text: '日月食', link: '/astronomy/eclipses' },
          { text: '行星：位置、星等、会合与伏见', link: '/astronomy/planets' },
          { text: '恒星与二十八宿', link: '/astronomy/stars' },
          { text: '当地所见的星空', link: '/astronomy/sky-view' },
          { text: '史书天象记录检验', link: '/astronomy/records' },
        ],
      },
      {
        text: '历法篇',
        items: [
          { text: '农历怎样编排', link: '/calendar/rules' },
          { text: '实历与主表', link: '/calendar/historical' },
          { text: '古历的推步与复原', link: '/calendar/ancient' },
          { text: '历代历法总录', link: '/calendar/compendium' },
          { text: '各朝代、政权的推步算法', link: '/calendar/push-algorithms' },
          { text: '秦与汉初历法复原', link: '/calendar/qin-han' },
        ],
      },
      {
        text: '附录',
        collapsed: true,
        items: [
          { text: '天象逐条校验', link: '/appendix/phenomena-checks', collapsed: true, items: checksSidebar },
          { text: '以本纪核验主表', link: '/appendix/main-table-checks' },
          { text: '历法原文摘录', link: '/appendix/source-texts' },
        ],
      },
    ],
  },
})
