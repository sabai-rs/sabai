import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import PageMarkdown from './components/PageMarkdown.vue'
import SabaiHome from './components/SabaiHome.vue'
import TryIt from './components/TryIt.vue'
import './style.css'

export default {
  extends: DefaultTheme,
  Layout: () => h(DefaultTheme.Layout, null, { 'doc-footer-before': () => h(PageMarkdown) }),
  enhanceApp({ app }) {
    app.component('SabaiHome', SabaiHome)
    app.component('TryIt', TryIt)
  },
} satisfies Theme
