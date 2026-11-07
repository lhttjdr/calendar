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
      { text: '数学篇', link: '/math/rotations' },
      { text: '天文篇', link: '/astronomy/sphere' },
      { text: '历法篇', link: '/calendar/dates' },
      { text: '民俗篇', link: '/folk/ganzhi' },
      { text: '附录', link: '/appendix/phenomena-checks' },
      { text: '在线日历', link: appUrl, target: '_self' },
      { text: 'GitHub', link: 'https://github.com/lhttjdr/calendar' },
    ],
    // 篇按三大域依赖方向排（数学 → 天文 → 历法 → 民俗），篇内按域的层次分组；章名一个词、最多三个词，不带副标题
    sidebar: [
      {
        text: '数学篇',
        items: [
          { text: '转动', link: '/math/rotations' },
          { text: '级数求值', link: '/math/series' },
          { text: '求根与极值', link: '/math/roots' },
          { text: '最小二乘拟合', link: '/math/fitting' },
          { text: '数值精度', link: '/math/precision' },
        ],
      },
      {
        text: '天文篇',
        items: [
          {
            text: '基础',
            items: [
              { text: '天球坐标', link: '/astronomy/sphere' },
              { text: '时间与时标', link: '/astronomy/time' },
              { text: '参考架', link: '/astronomy/frames' },
              { text: '岁差', link: '/astronomy/precession' },
              { text: '章动', link: '/astronomy/nutation' },
              { text: '历表', link: '/astronomy/ephemerides' },
              { text: '模型配套', link: '/astronomy/pairing' },
            ],
          },
          {
            text: '位置',
            items: [{ text: '视位置', link: '/astronomy/apparent' }],
          },
          {
            text: '天象',
            items: [
              { text: '合朔与月相', link: '/astronomy/syzygy' },
              { text: '二十四节气', link: '/astronomy/solar-terms' },
              { text: '日月食', link: '/astronomy/eclipses' },
              { text: '行星', link: '/astronomy/planets' },
              { text: '恒星与二十八宿', link: '/astronomy/stars' },
              { text: '当地星空', link: '/astronomy/sky-view' },
              { text: '史书天象检验', link: '/astronomy/records' },
            ],
          },
        ],
      },
      {
        text: '历法篇',
        items: [
          { text: '基础', items: [{ text: '历日与纪日', link: '/calendar/dates' }] },
          {
            text: '农历',
            items: [
              { text: '农历编排', link: '/calendar/rules' },
              { text: '实历与主表', link: '/calendar/historical' },
            ],
          },
          {
            text: '古历',
            items: [
              { text: '古历复原', link: '/calendar/ancient' },
              { text: '各朝推步', link: '/calendar/push-algorithms' },
              { text: '秦与汉初历法', link: '/calendar/qin-han' },
              { text: '历代历法总录', link: '/calendar/compendium' },
            ],
          },
          { text: '纪年', items: [{ text: '年号纪年', link: '/calendar/eras' }] },
        ],
      },
      {
        text: '民俗篇',
        items: [
          { text: '干支历四柱', link: '/folk/ganzhi' },
          { text: '纳音与值日', link: '/folk/nayin' },
          { text: '杂节气', link: '/folk/zajie' },
          { text: '更点时刻', link: '/folk/gengdian' },
          { text: '民俗规划', link: '/folk/plan' },
        ],
      },
      {
        text: '附录',
        collapsed: true,
        items: [
          { text: '天象逐条校验', link: '/appendix/phenomena-checks', collapsed: true, items: checksSidebar },
          { text: '本纪核验主表', link: '/appendix/main-table-checks' },
          { text: '历法原文摘录', link: '/appendix/source-texts' },
        ],
      },
    ],
  },
})
