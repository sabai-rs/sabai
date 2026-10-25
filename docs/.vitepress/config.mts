import { defineConfig } from 'vitepress'
import container from 'markdown-it-container'
import llms from 'vitepress-plugin-llms'

export default defineConfig({
  title: 'Sabai',
  description: 'A Rust web framework for people who already know Laravel.',
  lang: 'en',
  base: '/sabai/',
  cleanUrls: true,
  lastUpdated: true,
  srcExclude: ['README.md'],

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/sabai/logo.svg' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    [
      'link',
      {
        rel: 'stylesheet',
        href: 'https://fonts.googleapis.com/css2?family=Anuphan:wght@500;600;700&family=IBM+Plex+Sans+Thai:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap',
      },
    ],
  ],

  markdown: {
    config(md) {
      md.use(container, 'laravel', {
        render(tokens, index) {
          return tokens[index].nesting === 1
            ? '<div class="laravel-note"><p class="laravel-note-title">Coming from Laravel</p>\n'
            : '</div>\n'
        },
      })
    },
  },

  vite: {
    plugins: [llms()],
  },

  themeConfig: {
    logo: { light: '/logo.svg', dark: '/logo-dark.svg', alt: '' },
    siteTitle: 'sabai',

    nav: [
      { text: 'Docs', link: '/introduction', activeMatch: '^/(?!$)' },
      { text: 'Roadmap', link: 'https://github.com/sabai-rs/sabai/blob/main/ROADMAP.md' },
    ],

    sidebar: [
      {
        text: 'Getting started',
        items: [
          { text: 'Introduction', link: '/introduction' },
          { text: 'Installation', link: '/installation' },
          { text: 'Configuration', link: '/configuration' },
          { text: 'Directory structure', link: '/structure' },
        ],
      },
      {
        text: 'The basics',
        items: [
          { text: 'Routing', link: '/routing' },
          { text: 'Middleware', link: '/middleware' },
          { text: 'Controllers', link: '/controllers' },
          { text: 'Requests', link: '/requests' },
          { text: 'Validation', link: '/validation' },
          { text: 'Responses', link: '/responses' },
          { text: 'Errors', link: '/errors' },
        ],
      },
      {
        text: 'Database',
        items: [
          { text: 'Getting started', link: '/database' },
          { text: 'Migrations', link: '/migrations' },
          { text: 'Eloquent', link: '/eloquent' },
        ],
      },
      {
        text: 'Security',
        items: [
          { text: 'Authentication', link: '/authentication' },
          { text: 'Hashing', link: '/hashing' },
        ],
      },
      {
        text: 'Testing',
        items: [{ text: 'HTTP tests', link: '/http-tests' }],
      },
      {
        text: 'Architecture',
        items: [
          { text: 'Service container', link: '/container' },
          { text: 'Service providers', link: '/providers' },
          { text: 'Console', link: '/console' },
          { text: 'Dates', link: '/dates' },
        ],
      },
    ],

    socialLinks: [{ icon: 'github', link: 'https://github.com/sabai-rs/sabai' }],
    search: { provider: 'local' },
    outline: { level: [2, 3], label: 'On this page' },
    editLink: {
      pattern: 'https://github.com/sabai-rs/sabai/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },
    docFooter: { prev: 'Previous page', next: 'Next page' },
    footer: { message: 'sabai (สบาย) is Thai for at ease. Made in Thailand.' },
  },
})
