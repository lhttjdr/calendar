import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import Anim from './Anim.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('Anim', Anim)
  },
} satisfies Theme
