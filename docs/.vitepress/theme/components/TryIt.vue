<script setup lang="ts">
import { computed, ref } from 'vue'

// The blog's real answers to these requests, checked with curl.
const requests = [
  {
    label: 'GET /api/posts/1',
    status: '200 OK',
    ok: true,
    body: '{"id":1,"title":"Sabai","body":"Rust, but sabai"}',
    why: 'Post::find_or_fail(1) found the row.',
  },
  {
    label: 'GET /api/posts/99',
    status: '404 Not Found',
    ok: false,
    body: '{"message":"Not Found"}',
    why: "find_or_fail answers 404 when no row has that id, like Laravel's findOrFail.",
  },
  {
    label: 'GET /api/posts/abc',
    status: '404 Not Found',
    ok: false,
    body: '{"message":"Not Found"}',
    why: 'Param<u64> rejects abc before the handler runs.',
  },
  {
    label: 'POST /api/posts {}',
    status: '422 Unprocessable Entity',
    ok: false,
    body: '{"message":"The title field is required. (and 1 more error)","errors":{"title":["The title field is required."],"body":["The body field is required."]}}',
    why: "StorePostRequest's rules ran first, in Laravel's words and shape.",
  },
]

const index = ref(0)
const current = computed(() => requests[index.value])
</script>

<template>
  <div class="try-it">
    <div class="bar" role="tablist" aria-label="Requests">
      <button
        v-for="(request, i) in requests"
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
.status { justify-self: start; font-family: var(--vp-font-family-mono); font-size: 12.5px; font-weight: 500; padding: 2px 9px; border-radius: 999px; background: var(--sabai-marigold-soft); color: var(--sabai-marigold); }
.status.ok { background: var(--sabai-teal-soft); color: var(--sabai-teal); }
pre { margin: 0; font-family: var(--vp-font-family-mono); font-size: 12.5px; white-space: pre-wrap; word-break: break-word; }
.out p { margin: 0; color: var(--vp-c-text-2); font-size: 14px; }
</style>
