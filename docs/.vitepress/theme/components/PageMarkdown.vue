<script setup lang="ts">
import { computed, ref } from 'vue'
import { useData, withBase } from 'vitepress'

const { page } = useData()
const markdownUrl = computed(() => withBase('/' + page.value.relativePath))
const copied = ref(false)

async function copy() {
  try {
    const text = await (await fetch(markdownUrl.value)).text()
    await navigator.clipboard.writeText(text)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch {
    window.open(markdownUrl.value, '_blank')
  }
}
</script>

<template>
  <div class="page-markdown">
    <button type="button" @click="copy">{{ copied ? 'Copied' : 'Copy as Markdown' }}</button>
    <a :href="markdownUrl" target="_blank" rel="noreferrer">View as Markdown</a>
  </div>
</template>

<style scoped>
.page-markdown { display: flex; flex-wrap: wrap; gap: 8px 20px; margin-top: 32px; font-size: 14px; }
.page-markdown button { color: var(--sabai-teal); font-weight: 500; }
.page-markdown a { color: var(--vp-c-text-2); }
.page-markdown button:focus-visible { outline: 2px solid var(--sabai-teal); outline-offset: 2px; }
</style>
