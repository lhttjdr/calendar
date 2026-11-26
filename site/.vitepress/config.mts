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
      // 宽表横向滚动、不把短列挤成一字一行。列数不少于 7 的宽表：单元格（去掉 Markdown 记号后）24 字以内加 nw 类不折行，
      // 更长者加 wide 类、内容包 div.cell-wide 保底宽度，合宽超出正文时表格横向滚动（custom.css）；小表只让 8 字以内者不折行，
      // 其余照常折行、排在正文宽度内
      md.core.ruler.push('table_cell_width', (state) => {
        const toks = state.tokens
        let wideTable = false
        for (let i = 0; i < toks.length - 1; i++) {
          const t = toks[i]
          if (t.type === 'table_open') {
            let cols = 0
            for (let k = i + 1; k < toks.length && toks[k].type !== 'tr_close'; k++) if (toks[k].type === 'th_open') cols++
            wideTable = cols >= 7
          }
          if (t.type !== 'td_open' && t.type !== 'th_open') continue
          const inline = toks[i + 1]
          const text = inline.type === 'inline' ? inline.content.replace(/[`*_\[\]]|\(.*?\)/g, '') : ''
          const n = [...text].length
          if (n <= (wideTable ? 24 : 8)) t.attrJoin('class', 'nw')
          else if (wideTable) t.attrJoin('class', 'wide')
        }
      })
      // 单元格上的 min-width 在自动表格布局里不生效：长文本单元格内容包一层 div.cell-wide，保底宽度设在 div 上
      for (const tag of ['td', 'th']) {
        const open = md.renderer.rules[`${tag}_open`] ?? ((t, i, o, e, self) => self.renderToken(t, i, o))
        const close = md.renderer.rules[`${tag}_close`] ?? ((t, i, o, e, self) => self.renderToken(t, i, o))
        md.renderer.rules[`${tag}_open`] = (t, i, o, e, self) =>
          open(t, i, o, e, self) + ((t[i].attrGet('class') ?? '').includes('wide') ? '<div class="cell-wide">' : '')
        md.renderer.rules[`${tag}_close`] = (t, i, o, e, self) => {
          let j = i - 1
          while (j >= 0 && t[j].type !== `${tag}_open`) j--
          return ((j >= 0 && (t[j].attrGet('class') ?? '').includes('wide')) ? '</div>' : '') + close(t, i, o, e, self)
        }
      }
    },
  },
  themeConfig: {
    outline: { level: [2, 3], label: '本页目录' },
    docFooter: { prev: '上一章', next: '下一章' },
    search: { provider: 'local' },
    nav: [
      { text: '数学篇', link: '/math/vectors' },
      { text: '天文篇', link: '/astronomy/sphere' },
      { text: '历法篇', link: '/calendar/dates' },
      { text: '民俗篇', link: '/folk/yuanliu' },
      { text: '附录', link: '/appendix/phenomena-checks' },
      { text: '在线日历', link: appUrl, target: '_self' },
      { text: 'GitHub', link: 'https://github.com/lhttjdr/calendar' },
    ],
    // 篇按三大域依赖方向排（数学 → 天文 → 历法 → 民俗），篇内按域的层次分组；章名一个词、最多三个词，不带副标题
    sidebar: [
      {
        text: '数学篇',
        items: [
          {
            text: '基础',
            items: [
              { text: '向量与空间', link: '/math/vectors' },
              { text: '转动', link: '/math/rotations' },
              { text: '数值精度', link: '/math/precision' },
            ],
          },
          {
            text: '数值方法',
            items: [
              { text: '级数求值', link: '/math/series' },
              { text: '求根与极值', link: '/math/roots' },
              { text: '最小二乘拟合', link: '/math/fitting' },
            ],
          },
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
              { text: '行用历日', link: '/calendar/historical' },
            ],
          },
          {
            text: '古历',
            items: [
              { text: '古历复原', link: '/calendar/ancient' },
              { text: '秦与汉初历法', link: '/calendar/qin-han' },
              { text: '各朝推步', link: '/calendar/push-algorithms' },
              { text: '历代历法总录', link: '/calendar/compendium' },
            ],
          },
          { text: '纪年', items: [{ text: '年号纪年', link: '/calendar/eras' }] },
        ],
      },
      {
        text: '民俗篇',
        items: [
          { text: '总论', items: [{ text: '民俗源流', link: '/folk/yuanliu' }] },
          {
            text: '纪时',
            items: [
              { text: '干支历四柱', link: '/folk/ganzhi' },
              { text: '纳音与值日', link: '/folk/nayin' },
              { text: '更点时刻', link: '/folk/gengdian' },
            ],
          },
          { text: '节令', items: [{ text: '杂节气', link: '/folk/zajie' }] },
          {
            text: '择日',
            items: [
              { text: '建除与黄道', link: '/folk/jianchu' },
              { text: '神煞宜忌', link: '/folk/shensha' },
              { text: '通书异同', link: '/folk/tongshu' },
              { text: '婚嫁择吉', link: '/folk/jiaqu' },
            ],
          },
          {
            text: '式占',
            items: [
              { text: '奇门遁甲', link: '/folk/qimen' },
              { text: '大六壬', link: '/folk/liuren' },
              { text: '太乙', link: '/folk/taiyi' },
            ],
          },
          {
            text: '筮法',
            items: [
              { text: '周易筮法', link: '/folk/zhouyi' },
              { text: '六爻纳甲', link: '/folk/liuyao' },
            ],
          },
          {
            text: '命理',
            items: [
              { text: '子平八字', link: '/folk/ziping' },
              { text: '七政四余', link: '/folk/qizheng' },
              { text: '紫微斗数', link: '/folk/ziwei' },
            ],
          },
          { text: '风水', items: [{ text: '三元玄空', link: '/folk/xuankong' }] },
        ],
      },
      {
        text: '附录',
        collapsed: true,
        items: [
          { text: '天象逐条校验', link: '/appendix/phenomena-checks', collapsed: true, items: checksSidebar },
          { text: '本纪核验历日', link: '/appendix/benji-checks' },
          { text: '本纪朔日误书', link: '/appendix/benji-qita' },
          { text: '历法原文摘录', link: '/appendix/source-texts' },
        ],
      },
    ],
  },
})
