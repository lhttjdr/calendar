import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import Anim from './Anim.vue'
import Chart from './Chart.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('Anim', Anim)
    app.component('Chart', Chart)
  },
} satisfies Theme
