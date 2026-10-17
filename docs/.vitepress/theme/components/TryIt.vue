<script setup lang="ts">
import { computed, ref } from 'vue'

export interface TryRequest {
  label: string
  status: string
  ok: boolean
  body: string
  why: string
}

// Each page passes the blog's real answers, checked with curl.
const props = defineProps<{ requests: TryRequest[] }>()

const index = ref(0)
const current = computed(() => props.requests[index.value])
</script>

<template>
  <div class="try-it">
    <div class="bar" role="tablist" aria-label="Requests">
      <button
        v-for="(request, i) in props.requests"
        :key="request.label"
        role="tab"
        :aria-selected="i === index"
        @click="index = i"
      >{{ request.label }}</button>
    </div>
    <div class="out">
      <span class="status" :class="{ ok: current.ok }">{{ current.status }}</span>
      <pre>{{ current.body }}</pre>
      <p>{{ current.why }}</p>
    </div>
  </div>
</template>

<style scoped>
.try-it { margin: 16px 0; border: 1px solid var(--vp-c-divider); border-radius: 8px; overflow: hidden; background: var(--vp-c-bg-elv); }
.bar { display: flex; flex-wrap: wrap; gap: 6px; padding: 8px; border-bottom: 1px solid var(--vp-c-divider); background: var(--sabai-band); }
.bar button { font-family: var(--vp-font-family-mono); font-size: 12px; color: var(--vp-c-text-2); background: var(--vp-c-bg-elv); border: 1px solid var(--vp-c-divider); border-radius: 6px; padding: 4px 10px; }
.bar button[aria-selected='true'] { color: var(--vp-c-bg); background: var(--vp-c-text-1); border-color: var(--vp-c-text-1); }
.bar button:focus-visible { outline: 2px solid var(--sabai-teal); outline-offset: 2px; }
.out { padding: 12px 14px; display: grid; gap: 8px; }
.status { justify-self: start; font-family: var(--vp-font-family-mono); font-size: 12.5px; font-weight: 500; padding: 2px 9px; border-radius: 999px; background: var(--sabai-crab-soft); color: var(--sabai-crab); }
.status.ok { background: var(--sabai-teal-soft); color: var(--sabai-teal); }
pre { margin: 0; font-family: var(--vp-font-family-mono); font-size: 12.5px; white-space: pre-wrap; overflow-wrap: anywhere; font-variant-ligatures: none; }
.out p { margin: 0; color: var(--vp-c-text-2); font-size: 14px; }
</style>
