import { defineConfig } from 'vitepress'
import cjkFriendly from 'markdown-it-cjk-friendly'

// GitHub Pages 项目站为 /<repo>/docs/，通过环境变量 DOCS_BASE 传入（如 /calendar/docs/）
const base = process.env.DOCS_BASE ?? '/'
// 应用站（文档站上一级）须给完整 URL：站内绝对路径会被 VitePress 再加一次 base；本地缺省指向 web 的 vite dev
const appUrl = process.env.APP_URL ?? 'http://localhost:5173/'

export default defineConfig({
  base,
  lang: 'zh-Hans',
  title: '农历天文历法文档',
  description: '月相与二十四节气计算、农历编算、历史历法复原技术文档',
  srcExclude: ['references/**'],
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
    editLink: {
      pattern: 'https://github.com/lhttjdr/calendar/edit/master/doc/:path',
      text: '在 GitHub 上编辑此页',
    },
    nav: [
      { text: '首页', link: '/' },
      { text: '总体架构', link: '/architecture/three-domains' },
      { text: '天文基础', link: '/astronomy/time-and-timescales' },
      { text: '农历编算', link: '/calendar/lunar-calendar' },
      { text: '工程实现', link: '/engineering/project-and-implementation' },
      { text: '在线日历', link: appUrl, target: '_self' },
      { text: 'GitHub', link: 'https://github.com/lhttjdr/calendar' },
    ],
    sidebar: [
      {
        text: '〇、总体架构',
        collapsed: false,
        items: [
          { text: '三大域与域内四段结构', link: '/architecture/three-domains' },
        ],
      },
      {
        text: '一、天文基础（按《月相和二十四节气的计算》章节）',
        collapsed: false,
        items: [
          { text: '时间与儒略日、时标', link: '/astronomy/time-and-timescales' },
          { text: '参考系与坐标', link: '/astronomy/reference-frames' },
          { text: '岁差', link: '/astronomy/precession' },
          { text: '章动', link: '/astronomy/nutation' },
          { text: '历表与 DE 对齐', link: '/astronomy/ephemerides-and-de-align' },
          { text: '光行时与光行差', link: '/astronomy/light-time-and-aberration' },
          { text: '视位置计算', link: '/astronomy/apparent-longitude-and-syzygy' },
          { text: '月相与合朔', link: '/astronomy/lunar-phases-and-syzygy' },
          { text: '二十四节气与定气', link: '/astronomy/solar-terms' },
          { text: '天象（日月食、行星、恒星与星官）', link: '/astronomy/celestial-phenomena' },
          { text: '3D 星空（当地所见）', link: '/astronomy/sky-view-3d' },
          { text: '天象逐条校验结果', link: '/astronomy/phenomena-checks' },
        ],
      },
      {
        text: '二、农历编算',
        collapsed: false,
        items: [
          { text: '农历编算', link: '/calendar/lunar-calendar' },
          { text: '历法域内部结构（四层）', link: '/calendar/calendar-layers' },
          { text: '实历模块', link: '/calendar/historical-calendar' },
          { text: '以本纪核验主表：逐条结果', link: '/calendar/main-table-checks' },
          { text: '古历复原模块', link: '/calendar/ancient-calendar-restore' },
          { text: '历代历法总录', link: '/calendar/calendar-compendium' },
          { text: '历法原文摘录', link: '/calendar/calendar-source-texts' },
          { text: '各朝代历法推步算法', link: '/calendar/push-algorithms' },
          { text: '秦与汉初历法复原接入评估', link: '/calendar/qin-han-calendar-evaluation' },
        ],
      },
      {
        text: '三、工程实现',
        collapsed: false,
        items: [
          { text: '项目与实现', link: '/engineering/project-and-implementation' },
          { text: 'Rust + WASM + React 性能调优', link: '/engineering/wasm-react-performance' },
          { text: '历表二进制化与零解析方案', link: '/engineering/ephemeris-binary-format' },
          { text: '历表与数据路径总结', link: '/engineering/data-paths-summary' },
          { text: '管线图：节点与边的分类', link: '/engineering/pipeline-graph-taxonomy' },
        ],
      },
      {
        text: '四、案例分析',
        collapsed: false,
        items: [
          { text: 'DE406 定气失败分析', link: '/case-studies/de406-solar-term-failure-analysis' },
        ],
      },
    ],
  },
})
