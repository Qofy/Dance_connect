# Svelte + Vite

This template should help get you started developing with Svelte in Vite.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode).

## Need an official Svelte framework?

Check out [SvelteKit](https://github.com/sveltejs/kit#readme), which is also powered by Vite. Deploy anywhere with its serverless-first approach and adapt to various platforms, with out of the box support for TypeScript, SCSS, and Less, and easily-added support for mdsvex, GraphQL, PostCSS, Tailwind CSS, and more.

## Technical considerations

**Why use this over SvelteKit?**

- It brings its own routing solution which might not be preferable for some users.
- It is first and foremost a framework that just happens to use Vite under the hood, not a Vite app.

This template contains as little as possible to get started with Vite + Svelte, while taking into account the developer experience with regards to HMR and intellisense. It demonstrates capabilities on par with the other `create-vite` templates and is a good starting point for beginners dipping their toes into a Vite + Svelte project.

Should you later need the extended capabilities and extensibility provided by SvelteKit, the template has been structured similarly to SvelteKit so that it is easy to migrate.

**Why include `.vscode/extensions.json`?**

Other templates indirectly recommend extensions via the README, but this file allows VS Code to prompt the user to install the recommended extension upon opening the project.

**Why enable `checkJs` in the JS template?**

It is likely that most cases of changing variable types in runtime are likely to be accidental, rather than deliberate. This provides advanced typechecking out of the box. Should you like to take advantage of the dynamically-typed nature of JavaScript, it is trivial to change the configuration.

**Why is HMR not preserving my local component state?**

HMR state preservation comes with a number of gotchas! It has been disabled by default in both `svelte-hmr` and `@sveltejs/vite-plugin-svelte` due to its often surprising behavior. You can read the details [here](https://github.com/sveltejs/svelte-hmr/tree/master/packages/svelte-hmr#preservation-of-local-state).

If you have state that's important to retain within a component, consider creating an external store which would not be replaced by HMR.

```js
// store.js
// An extremely simple external store
import { writable } from 'svelte/store'
export default writable(0)
```
Context

 The user has a fully-featured React 18 DanceConnect frontend app and wants it converted to Svelte 5 inside an existing Svelte template. The React source
 has 12 pages, 14 components, an API layer, and Tailwind CSS with neobrutalist design. The Svelte target already has @roxi/routify installed but is still
 an empty starter.

 Source (React): /Users/kofisafoagyekum/Desktop/internship/20260217_kof_kofi_task_097/20260217_task/202502_danceconnect_svelte_v061/src/
 Target (Svelte): /Users/kofisafoagyekum/Desktop/internship/20260218_kofi_task_098/20260218_kofi_task/Dance_connect/

 ---
 Step 1 — Update package.json + install deps

 Add to target's package.json:
 - devDependencies: tailwindcss@^3, autoprefixer, postcss
 - dependencies: lucide-svelte, date-fns

 Then run: cd Dance_connect && bun install

 ---
 Step 2 — Config files

 Modify vite.config.js — add @ path alias:
 import path from "path";
 import { fileURLToPath } from "url";
 const __dirname = path.dirname(fileURLToPath(import.meta.url));
 // add to defineConfig:
 resolve: { alias: { "@": path.resolve(__dirname, "src") } }

 Create tailwind.config.js:
 export default {
   content: ["./index.html", "./src/**/*.{svelte,js,ts}"],
   theme: { extend: {} },
   plugins: [],
 }

 Create postcss.config.js:
 export default { plugins: { tailwindcss: {}, autoprefixer: {} } }

 ---
 Step 3 — CSS setup

 Replace src/app.css with Tailwind directives + all CSS from two sources:
 1. Tailwind directives: @tailwind base; @tailwind components; @tailwind utilities;
 2. Neumorphic tokens from src/index.css (lines 6–70) — --base, .neumorphic-* classes
 3. Neobrutalist tokens from src/Layout.jsx inline style (lines 61–101) — --neo-primary, .neo-shadow, .neo-border, .neo-hover, .neo-active, .leaflet-*
 4. Base body/html rules: html,body,#app{height:100%} (swap #root → #app)

 Copy src/logo_dance.svg from source to target.

 ---
 Step 4 — Copy non-React files as-is

 These have zero React imports — direct copy:
 - src/integrations/Core.js
 - src/entities/Event.js
 - src/entities/Registration.js
 - src/entities/User.js
 - src/utils/index.js

 ---
 Step 5 — UI Primitives (src/components/ui/)

 Create 4 Svelte components. Key Svelte 5 patterns:
 - Props via let { class: className = '', children, ...rest } = $props()
 - class is a reserved word → use class: className in destructure
 - Children via {@render children?.()}
 - Two-way binding: $bindable() + bind:value

 ┌─────────────────┬──────────────────────────────────────────────────────────────────────────────────────────────┐
 │      File       │                                            Source                                            │
 ├─────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ Button.svelte   │ button.jsx                                                                                   │
 ├─────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ Input.svelte    │ input.jsx                                                                                    │
 ├─────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ Textarea.svelte │ textarea.jsx                                                                                 │
 ├─────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────┤
 │ Select.svelte   │ select.jsx — collapse SelectTrigger/Content/Item into single <select> with <option> children │
 └─────────────────┴──────────────────────────────────────────────────────────────────────────────────────────────┘

 ---
 Step 6 — Modal + Shared components

 ┌───────────────────────────────────────────┬──────────────────────┬────────────────────────────────────────────────┐
 │                   File                    │        Source        │                   Key change                   │
 ├───────────────────────────────────────────┼──────────────────────┼────────────────────────────────────────────────┤
 │ components/modals/BugReportModal.svelte   │ BugReportModal.jsx   │ if(!isOpen) return null → {#if isOpen} wrapper │
 ├───────────────────────────────────────────┼──────────────────────┼────────────────────────────────────────────────┤
 │ components/modals/WishRequestModal.svelte │ WishRequestModal.jsx │ Same pattern                                   │
 ├───────────────────────────────────────────┼──────────────────────┼────────────────────────────────────────────────┤
 │ components/shared/PersonaSwitcher.svelte  │ PersonaSwitcher.jsx  │ useState → $state; native bind:value           │
 └───────────────────────────────────────────┴──────────────────────┴────────────────────────────────────────────────┘

 ---
 Step 7 — Form + Dashboard components

 File: components/forms/DanceStylePicker.svelte
 Source: DanceStylePicker.jsx
 Key change: arr.map(...) → {#each}, onClick → onclick
 ────────────────────────────────────────
 File: components/dashboard/LocationDetector.svelte
 Source: LocationDetector.jsx
 Key change: useCallback removed; useEffect(fn,[]) → $effect(()=>{})
 ────────────────────────────────────────
 File: components/dashboard/QuickActions.svelte
 Source: QuickActions.jsx
 Key change: <item.icon /> → <svelte:component this={item.icon}>; <Link> → <a>
 ────────────────────────────────────────
 File: components/dashboard/StatsPanel.svelte
 Source: StatsPanel.jsx
 Key change: Same icon pattern; $derived for stats array
 ────────────────────────────────────────
 File: components/events/EventCard.svelte
 Source: EventCard.jsx
 Key change: Replace Math.random() with deterministic hash of event.id; <Link> → <a href="/event?id={event.id}">

 ---
 Step 8 — Layout component

 src/components/Layout.svelte (from src/Layout.jsx)

 Key conversions:
 - useLocation() → import { page } from '@roxi/routify' then $page.path for active route detection
 - {children} prop → let { children } = $props() + {@render children()}
 - <Link to={url}> → <a href={url}>
 - <item.icon className="w-5 h-5" /> → <svelte:component this={item.icon} class="w-5 h-5" />
 - React.useState → $state for modal states
 - onClick → onclick
 - className → class
 - Inline <style> JSX tag → already moved to app.css in Step 3

 ---
 Step 9 — Update src/App.svelte

 Replace the Vite starter content with Routify Router:

 <script>
   import { Router } from '@roxi/routify';
   import { routes } from '../.routify/routes.js';
 </script>

 <Router {routes} />

 The routify() Vite plugin auto-generates .routify/routes.js by scanning src/pages/ on dev/build.

 ---
 Step 10 — Pages (src/pages/)

 Create src/pages/ directory. Routes mirror the React App.jsx routing:

 ┌─────────────────────┬───────────────────────┬───────────────┬─────────────────────────────────────┐
 │     Svelte file     │     React source      │      URL      │         Wrapped in Layout?          │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ index.svelte        │ (new)                 │ /             │ No — just goto('/login')            │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ login.svelte        │ Login.jsx             │ /login        │ No                                  │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ register.svelte     │ Register.jsx          │ /register     │ No                                  │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ dashboard.svelte    │ Dashboard.jsx         │ /dashboard    │ Yes — import Layout                 │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ today.svelte        │ Today.jsx             │ /today        │ Yes                                 │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ calendar.svelte     │ Calendar.jsx          │ /calendar     │ Yes                                 │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ map.svelte          │ MapView.jsx           │ /map          │ Yes                                 │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ profile.svelte      │ Profile.jsx           │ /profile      │ Yes                                 │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ create-event.svelte │ CreateEvent.jsx       │ /create-event │ Yes                                 │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ event.svelte        │ EventProfile.jsx      │ /event        │ Yes — reads ?id= via $page.query.id │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ edit-profile.svelte │ EditProfile.jsx       │ /edit-profile │ Yes                                 │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ admin/events.svelte │ AdminEventManager.jsx │ /admin/events │ Yes                                 │
 ├─────────────────────┼───────────────────────┼───────────────┼─────────────────────────────────────┤
 │ admin/users.svelte  │ UserManager.jsx       │ /admin/users  │ Yes                                 │
 └─────────────────────┴───────────────────────┴───────────────┴─────────────────────────────────────┘

 Layout strategy: Each authenticated page imports and wraps with <Layout> directly (matching React's pattern). Login/register/index have no Layout wrapper.

 Svelte 5 conversion rules per page:
 - useState(val) → let x = $state(val)
 - useEffect(fn, []) → $effect(() => { fn() }) — runs once after mount
 - useEffect(fn, [dep]) → $effect(() => { /* reads dep inside */ }) — auto-tracks
 - useNavigate() → import { goto } from '@roxi/routify'; call goto('/path')
 - useSearchParams() for ?id= → import { page } from '@roxi/routify'; use $page.query.id
 - filteredEvents = events.filter(...) computed inline → let filtered = $derived(events.filter(...))
 - className= → class=; onChange= → oninput= or bind:value; onClick= → onclick=
 - arr.map(x => <X key={x.id} />) → {#each arr as x (x.id)}<X />{/each}
 - {condition && <X />} → {#if condition}<X />{/if}
 - <Link to="/path"> → <a href="/path">

 ---
 Files to Create/Modify (ordered)

 MODIFY  package.json
 RUN     bun install
 MODIFY  vite.config.js
 CREATE  tailwind.config.js
 CREATE  postcss.config.js
 REPLACE src/app.css
 COPY    src/logo_dance.svg
 COPY    src/integrations/Core.js
 COPY    src/entities/Event.js
 COPY    src/entities/Registration.js
 COPY    src/entities/User.js
 COPY    src/utils/index.js
 CREATE  src/components/ui/Button.svelte
 CREATE  src/components/ui/Input.svelte
 CREATE  src/components/ui/Textarea.svelte
 CREATE  src/components/ui/Select.svelte
 CREATE  src/components/modals/BugReportModal.svelte
 CREATE  src/components/modals/WishRequestModal.svelte
 CREATE  src/components/shared/PersonaSwitcher.svelte
 CREATE  src/components/forms/DanceStylePicker.svelte
 CREATE  src/components/dashboard/LocationDetector.svelte
 CREATE  src/components/dashboard/QuickActions.svelte
 CREATE  src/components/dashboard/StatsPanel.svelte
 CREATE  src/components/events/EventCard.svelte
 CREATE  src/components/Layout.svelte
 MODIFY  src/App.svelte
 CREATE  src/pages/index.svelte
 CREATE  src/pages/login.svelte
 CREATE  src/pages/register.svelte
 CREATE  src/pages/dashboard.svelte
 CREATE  src/pages/today.svelte
 CREATE  src/pages/calendar.svelte
 CREATE  src/pages/map.svelte
 CREATE  src/pages/profile.svelte
 CREATE  src/pages/create-event.svelte
 CREATE  src/pages/event.svelte
 CREATE  src/pages/edit-profile.svelte
 CREATE  src/pages/admin/events.svelte
 CREATE  src/pages/admin/users.svelte
 DELETE  src/lib/Counter.svelte  (old starter file, no longer needed)

 ---
 Verification

 1. Run bun run dev — dev server starts without errors
 2. Visit http://localhost:5173/ → redirects to /login
 3. Visit all 13 routes — each renders with correct content
 4. Check neo-border / neo-shadow CSS classes apply visually
 5. Verify active nav item highlights on each page
 6. Run bun run build — no TypeScript/compile errors
╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌