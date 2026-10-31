import { defineConfig } from 'vitepress'

// GitHub Pages 项目站为 /<repo>/docs/，通过环境变量 DOCS_BASE 传入（如 /calendar/docs/）
const base = process.env.DOCS_BASE ?? '/'

export default defineConfig({
  base,
  lang: 'zh-Hans',
  title: '农历天文历法文档',
  description: '月相与二十四节气计算、农历编算、历史历法复原技术文档',
  srcExclude: ['references/**'],
  cleanUrls: true,
  markdown: {
    math: true,
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
      { text: '天文基础', link: '/astronomy/time-and-timescales' },
      { text: '农历编算', link: '/calendar/lunar-calendar' },
      { text: '工程实现', link: '/engineering/project-and-implementation' },
      { text: 'GitHub', link: 'https://github.com/lhttjdr/calendar' },
    ],
    sidebar: [
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
          { text: '天象逐条校验结果', link: '/astronomy/phenomena-checks' },
        ],
      },
      {
        text: '二、农历编算',
        collapsed: false,
        items: [
          { text: '农历编算', link: '/calendar/lunar-calendar' },
          { text: '实历模块', link: '/calendar/historical-calendar' },
          { text: '古历复原模块', link: '/calendar/ancient-calendar-restore' },
          { text: '历代历法总录', link: '/calendar/calendar-compendium' },
          { text: '历法原文摘录', link: '/calendar/calendar-source-texts' },
          { text: '各朝代历法推步算法', link: '/calendar/push-algorithms' },
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
