<script setup lang="ts">
import { ref } from 'vue'
import { withBase } from 'vitepress'

const examples = {
  'Routes': {
    php: ['routes/api.php', `Route::apiResource('posts', PostController::class)
    ->only(['index', 'store', 'show']);`],
    rust: ['routes/api.rs', `route.api_resource("posts", |posts| {
    posts.index(PostController::index);
    posts.store(PostController::store);
    posts.show(PostController::show);
});`],
  },
  'Form request': {
    php: ['StorePostRequest.php', `public function rules(): array
{
    return [
        'title' => ['required', 'string', 'max:255'],
        'body'  => ['required', 'string'],
    ];
}`],
    rust: ['store_post_request.rs', `impl FormRequest for StorePostRequest {
    fn rules() -> Rules {
        Rules::new()
            .field("title", [required(), string(), max(255)])
            .field("body", [required(), string()])
    }
}`],
  },
  'Model': {
    php: ['Post.php', `class Post extends Model {}

Post::all();
Post::findOrFail($id);`],
    rust: ['post.rs', `#[derive(Model)]
pub struct Post { pub id: u64, pub title: String }

Post::all().await?;
Post::find_or_fail(id).await?;`],
  },
  'Migration': {
    php: ['create_posts_table.php', `Schema::create('posts', function (Blueprint $table) {
    $table->id();
    $table->string('title');
    $table->timestamps();
});`],
    rust: ['create_posts_table.rs', `schema.create("posts", |table| {
    table.id();
    table.string("title");
    table.timestamps();
});`],
  },
  'Feature test': {
    php: ['StoreTest.php', `$response = $this->postJson(route(self::ROUTE), []);

$response->assertStatus(422)
    ->assertJsonFragment(['title' => [...]]);`],
    rust: ['store.rs', `let response = app().post_json(route(ROUTE), json!({}));

response
    .assert_status(422)
    .assert_json_fragment(json!({"title": [...]}));`],
  },
} as const

type Example = keyof typeof examples
const names = Object.keys(examples) as Example[]
const current = ref<Example>('Routes')
</script>

<template>
  <div class="home">
    <div class="inner hero">
      <div>
        <p class="entry"><b>sabai</b>สบาย /sà.baːj/, at ease</p>
        <h1>Rust, but <span>sabai.</span></h1>
        <p class="lead">
          A Rust web framework for people who already know Laravel. Routes, controllers, form
          requests and models keep the names you know, and the app checks itself before it serves a
          request.
        </p>
        <div class="actions">
          <a class="btn primary" :href="withBase('/at-a-glance')">Get started</a>
          <a class="btn plain" :href="withBase('/introduction#coming-from-laravel')">Coming from Laravel</a>
        </div>
        <code class="pill">$ <b>cargo run -p blog</b></code>
      </div>

      <svg class="scene" viewBox="0 0 520 330" fill="none" role="img" aria-label="Ferris the Rust crab resting in a hammock under the sun">
        <circle class="sun" cx="430" cy="64" r="34" />
        <path class="ground" d="M20 300h480" stroke-width="2" stroke-dasharray="2 8" stroke-linecap="round" />
        <path class="post" d="M70 92v208M450 92v208" stroke-width="9" stroke-linecap="round" />
        <g class="sway">
          <path class="netfill" d="M78 100 C150 250 370 250 442 100 C370 214 150 214 78 100Z" />
          <path class="net" d="M78 100 C150 250 370 250 442 100" stroke-width="5" stroke-linecap="round" />
          <path class="net" d="M78 100 C150 214 370 214 442 100" stroke-width="3" stroke-linecap="round" opacity=".7" />
          <path class="net" d="M140 152l30 40M200 172l20 30M260 178v30M320 172l-20 30M380 152l-30 40" stroke-width="2" opacity=".5" />
          <g transform="translate(260 168)">
            <path class="crab" d="M-58 -6 C-58 -40 58 -40 58 -6 C58 10 -58 10 -58 -6Z" />
            <path class="crab" d="M-58 -14 C-80 -22 -86 -40 -74 -50 C-70 -38 -62 -32 -52 -30Z" />
            <path class="crab" d="M58 -14 C80 -22 86 -40 74 -50 C70 -38 62 -32 52 -30Z" />
            <path class="legs" d="M-40 6l-14 16M-22 9l-8 18M22 9l8 18M40 6l14 16" stroke-width="6" stroke-linecap="round" />
            <path class="face" d="M-20 -26 q6 -5 12 0M8 -26 q6 -5 12 0" stroke-width="3.2" stroke-linecap="round" />
            <path class="face" d="M-8 -12 q8 6 16 0" stroke-width="2.6" stroke-linecap="round" />
          </g>
        </g>
        <text class="z" x="330" y="96" font-size="22">z</text>
        <text class="z" x="350" y="74" font-size="17">z</text>
      </svg>
    </div>

    <section class="band">
      <div class="inner">
        <h2>You already know where everything goes</h2>
        <p class="sub">Pick something you write every day. On the left is Laravel; on the right, the same thing in Sabai.</p>
        <div class="tabs" role="tablist" aria-label="Examples">
          <button
            v-for="name in names"
            :key="name"
            role="tab"
            :aria-selected="name === current"
            @click="current = name"
          >{{ name }}</button>
        </div>
        <div class="pair">
          <div class="file">
            <div class="tab"><span>Laravel</span><span>{{ examples[current].php[0] }}</span></div>
            <pre>{{ examples[current].php[1] }}</pre>
          </div>
          <div class="file sabai">
            <div class="tab"><span>Sabai</span><span>{{ examples[current].rust[0] }}</span></div>
            <pre>{{ examples[current].rust[1] }}</pre>
          </div>
        </div>
      </div>
    </section>

    <section>
      <div class="inner split">
        <div>
          <h2>Forget a binding, and it tells you before anyone else does</h2>
          <p class="sub">Laravel finds a missing service on the first request that needs it. Sabai checks every route when it starts, and refuses to listen.</p>
          <div class="callouts">
            <div><b>It names the route</b><span>So you know which page would have broken.</span></div>
            <div><b>It says what to write</b><span>The line to add to your provider, ready to paste.</span></div>
          </div>
        </div>
        <pre class="term"><span class="dim">$ cargo run -p blog</span>
<span class="err">Error:</span> route `GET /api/health` cannot start:
no service is bound to `dyn Clock`;
bind it in a provider's `register`:
`app.bind::&lt;dyn Clock&gt;(Arc::new(...))`</pre>
      </div>
    </section>

    <section class="band">
      <div class="inner">
        <h2>Measured on every change</h2>
        <p class="sub">Compile speed is the second rule of the project, after the developer experience. These come from the example blog.</p>
        <div class="numbers">
          <div><strong>0.5 s</strong><b>to rebuild after editing a route</b><span>Incremental build of the blog on an M-series Mac.</span></div>
          <div><strong>19 ns</strong><b>for a model to find its database</b><span>So <code>Post::all()</code> needs no <code>db</code> argument, at no cost you can measure.</span></div>
          <div><strong>0</strong><b>lines of sqlx compiled into your app</b><span>The ORM boxes its futures, so the pool's code compiles once, in the framework.</span></div>
        </div>
      </div>
    </section>

    <section>
      <div class="inner">
        <h2>Where it is today</h2>
        <p class="sub">Sabai is young. The first goal is one path through everything: create an app, migrate, serve, and read posts from MySQL.</p>
        <div class="steps">
          <div><code>M1</code><span>Routing, controllers, form requests, validation</span><em class="done">Done</em></div>
          <div><code>M2</code><span>Migrations, models, MySQL and SQLite, several connections</span><em class="done">Done</em></div>
          <div><code>M5</code><span><code>sabai new</code>, <code>sabai serve</code>, <code>sabai migrate</code></span><em class="next">Next</em></div>
          <div><code>M1, M3</code><span>Middleware, then auth</span><em>Later</em></div>
        </div>
      </div>
    </section>

    <section class="band">
      <div class="inner closing">
        <h2 class="big">Lie back. It compiles.</h2>
        <p class="sub">Start with the example blog, then read the routing guide.</p>
        <div class="actions">
          <a class="btn primary" :href="withBase('/at-a-glance')">Get started</a>
          <a class="btn plain" href="https://github.com/sabai-rs/sabai/blob/main/docs/ROADMAP.md">Read the roadmap</a>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.home { font-size: 15.5px; line-height: 1.65; color: var(--vp-c-text-1); }
.inner { max-width: 1152px; margin: 0 auto; padding: 0 24px; }
section { padding: 64px 0; }
.band { background: var(--sabai-band); }
h2 { font-family: var(--sabai-display); font-weight: 600; font-size: clamp(1.6rem, 3vw, 2.2rem); letter-spacing: -0.02em; line-height: 1.15; margin: 0 0 10px; text-wrap: balance; }
.sub { margin: 0 0 28px; color: var(--vp-c-text-2); max-width: 60ch; font-size: 16.5px; }

.hero { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 40px; align-items: center; padding-top: 48px; padding-bottom: 64px; }
.entry { font-size: 14px; color: var(--vp-c-text-2); margin: 0 0 14px; }
.entry b { font-family: var(--sabai-display); color: var(--vp-c-text-1); font-weight: 600; font-size: 16px; margin-right: 6px; }
h1 { font-family: var(--sabai-display); font-weight: 700; font-size: clamp(2.8rem, 6.4vw, 4.6rem); line-height: 1; letter-spacing: -0.035em; margin: 0 0 16px; }
h1 span { color: var(--sabai-teal); }
.lead { margin: 0 0 24px; color: var(--vp-c-text-2); max-width: 44ch; font-size: 17px; }
.actions { display: flex; flex-wrap: wrap; gap: 10px; }
.btn { font-weight: 600; font-size: 14.5px; padding: 10px 18px; border-radius: 999px; border: 1px solid transparent; text-decoration: none; }
.btn.primary { background: var(--sabai-teal); color: var(--vp-c-bg); }
.btn.plain { border-color: var(--vp-c-divider); color: var(--vp-c-text-1); background: var(--vp-c-bg-elv); }
.pill { display: inline-block; margin-top: 18px; font-size: 13px; background: var(--vp-c-bg-elv); border: 1px solid var(--vp-c-divider); border-radius: 999px; padding: 6px 14px; color: var(--vp-c-text-2); }
.pill b { color: var(--vp-c-text-1); font-weight: 500; }

.scene { width: 100%; max-width: 520px; justify-self: center; }
.sun { fill: var(--sabai-marigold-bright); }
.post { stroke: var(--vp-c-text-1); }
.ground { stroke: var(--vp-c-divider); }
.net { stroke: var(--sabai-teal); }
.netfill { fill: var(--sabai-teal-soft); }
.crab { fill: var(--sabai-crab); }
.legs { stroke: var(--sabai-crab); }
.face { stroke: var(--vp-c-text-1); }
.z { fill: var(--vp-c-text-2); font-family: var(--sabai-display); font-weight: 600; }
.sway { transform-origin: 260px 92px; animation: sway 5.5s ease-in-out infinite; }
@keyframes sway { 0%, 100% { transform: rotate(-2.2deg); } 50% { transform: rotate(2.2deg); } }
@media (prefers-reduced-motion: reduce) { .sway { animation: none; } }

.tabs { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 14px; }
.tabs button { font-size: 14px; font-weight: 500; color: var(--vp-c-text-2); border: 1px solid var(--vp-c-divider); border-radius: 999px; padding: 5px 14px; }
.tabs button[aria-selected='true'] { background: var(--vp-c-text-1); color: var(--vp-c-bg); border-color: var(--vp-c-text-1); }
.tabs button:focus-visible { outline: 2px solid var(--sabai-teal); outline-offset: 2px; }
.pair { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
.file { background: var(--vp-c-bg-elv); border: 1px solid var(--vp-c-divider); border-radius: 8px; overflow: hidden; min-width: 0; }
.file .tab { display: flex; justify-content: space-between; gap: 8px; padding: 6px 12px; font-family: var(--vp-font-family-mono); font-size: 12px; color: var(--vp-c-text-2); border-bottom: 1px solid var(--vp-c-divider); }
.file.sabai { border-color: var(--sabai-teal); }
.file.sabai .tab { color: var(--sabai-teal); background: var(--sabai-teal-soft); }
.file pre { margin: 0; padding: 14px; font-family: var(--vp-font-family-mono); font-size: 13px; line-height: 1.6; overflow-x: auto; min-height: 150px; }

.split { display: grid; grid-template-columns: minmax(0, 5fr) minmax(0, 6fr); gap: 32px; align-items: center; }
.term { margin: 0; background: var(--sabai-term-bg); color: var(--sabai-term-fg); border-radius: 8px; padding: 16px 18px; font-family: var(--vp-font-family-mono); font-size: 13px; line-height: 1.7; overflow-x: auto; }
.term .dim { color: var(--sabai-term-dim); }
.term .err { color: var(--sabai-term-red); font-weight: 500; }
.callouts { display: grid; gap: 14px; }
.callouts div { border-left: 2px solid var(--sabai-teal); padding-left: 14px; display: grid; }
.callouts b { font-family: var(--sabai-display); font-weight: 600; }
.callouts span { color: var(--vp-c-text-2); font-size: 14.5px; }

.numbers { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); border-top: 1px solid var(--vp-c-divider); }
.numbers > div { padding: 22px 22px 6px 0; display: grid; align-content: start; gap: 4px; min-width: 0; }
.numbers > div + div { padding-left: 22px; border-left: 1px solid var(--vp-c-divider); }
.numbers strong { font-family: var(--sabai-display); font-size: clamp(2.4rem, 5vw, 3.4rem); font-weight: 700; letter-spacing: -0.03em; line-height: 1; color: var(--sabai-teal); font-variant-numeric: tabular-nums; }
.numbers b { margin-top: 6px; }
.numbers span { color: var(--vp-c-text-2); font-size: 14px; }

.steps { border-top: 1px solid var(--vp-c-divider); }
.steps > div { display: grid; grid-template-columns: 110px minmax(0, 1fr) auto; gap: 16px; align-items: baseline; padding: 12px 0; border-bottom: 1px solid var(--vp-c-divider); }
.steps code { font-size: 13px; color: var(--vp-c-text-2); background: none; padding: 0; }
.steps span code { color: var(--vp-c-text-1); }
.steps em { font-style: normal; font-size: 13px; font-weight: 600; padding: 2px 10px; border-radius: 999px; border: 1px solid var(--vp-c-divider); color: var(--vp-c-text-2); }
.steps em.done { background: var(--sabai-teal-soft); color: var(--sabai-teal); border-color: transparent; }
.steps em.next { background: var(--sabai-marigold-soft); color: var(--sabai-marigold); border-color: transparent; }

.closing { display: grid; gap: 4px; }
h2.big { font-weight: 700; font-size: clamp(2rem, 4.4vw, 3rem); letter-spacing: -0.03em; }

@media (max-width: 900px) {
  .hero, .split { grid-template-columns: minmax(0, 1fr); }
  .scene { max-width: 420px; }
  .pair { grid-template-columns: minmax(0, 1fr); }
}
@media (max-width: 640px) {
  .inner { padding: 0 16px; }
  section { padding: 44px 0; }
  .numbers { grid-template-columns: minmax(0, 1fr); }
  .numbers > div + div { padding-left: 0; border-left: 0; border-top: 1px solid var(--vp-c-divider); }
  .steps > div { grid-template-columns: minmax(0, 1fr) auto; }
  .steps > div > code { grid-column: 1 / -1; }
}
</style>
