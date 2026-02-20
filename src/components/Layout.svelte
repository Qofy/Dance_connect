<script>
  import { url, goto } from '@roxi/routify';
  import { Calendar, MapPin, User, LogOut, ChevronDown } from "lucide-svelte";
  import { createPageUrl } from "@/utils";

  let { children } = $props();

  let user = $state(null);
  let userMenuOpen = $state(false);

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  $effect(() => {
    const stored = localStorage.getItem('user');
    if (stored) {
      try { user = JSON.parse(stored); } catch { user = null; }
    }
  });

  const navItems = [
    { label: 'EVENTS',  href: '/dashboard',             icon: Calendar },
    { label: 'NEAR ME', href: createPageUrl("MapView"),  icon: MapPin },
  ];

  function logout() {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    userMenuOpen = false;
    _goto('/login');
  }

  let currentUrl = $derived($url);

  function initials(name) {
    if (!name) return '?';
    return name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
  }
</script>

<div class="min-h-screen bg-white">
  <!-- ── Public Header ── -->
  <header class="bg-white neo-border border-b-4 sticky top-0 z-40">
    <div class="max-w-7xl mx-auto px-4 py-3 flex items-center justify-between gap-4">

      <!-- Logo -->
      <a href="/dashboard" class="flex items-center gap-3 flex-shrink-0">
        <div class="w-11 h-11 bg-gradient-to-br from-blue-600 to-pink-600 neo-border neo-shadow flex items-center justify-center transform -rotate-2">
          <span class="text-white font-black text-lg">DC</span>
        </div>
        <div class="hidden sm:block leading-tight">
          <p class="text-xl font-black text-black">DANCECONNECT</p>
          <p class="text-xs font-bold text-gray-500 -mt-0.5">FIND YOUR RHYTHM</p>
        </div>
      </a>

      <!-- Desktop nav -->
      <nav class="hidden md:flex items-center gap-2">
        {#each navItems as item (item.label)}
          <a
            href={item.href}
            class={`flex items-center gap-2 px-4 py-2 neo-border font-bold transition-all ${
              currentUrl === item.href
                ? 'bg-blue-600 text-white neo-shadow'
                : 'bg-white text-black hover:bg-blue-600 hover:text-white'
            }`}
          >
            <svelte:component this={item.icon} class="w-4 h-4" />
            {item.label}
          </a>
        {/each}
      </nav>

      <!-- Right: user section -->
      <div class="flex items-center gap-2">
        {#if user}
          <!-- Avatar + dropdown -->
          <div class="relative">
            <button
              onclick={() => (userMenuOpen = !userMenuOpen)}
              class="flex items-center gap-2 px-3 py-2 neo-border bg-white font-bold hover:bg-gray-100 transition-all"
            >
              <span class="w-8 h-8 bg-blue-600 text-white font-black text-sm flex items-center justify-center flex-shrink-0">
                {initials(user.full_name || user.email)}
              </span>
              <span class="hidden sm:block max-w-32 truncate text-sm font-bold">
                {user.full_name || user.email}
              </span>
              <ChevronDown class="w-4 h-4 flex-shrink-0" />
            </button>

            {#if userMenuOpen}
              <button
                class="fixed inset-0 z-40"
                onclick={() => (userMenuOpen = false)}
                aria-label="Close menu"
              ></button>
              <div class="absolute right-0 top-full mt-1 w-48 bg-white neo-border neo-shadow z-50">
                <a
                  href="/profile"
                  onclick={() => (userMenuOpen = false)}
                  class="flex items-center gap-2 px-4 py-3 font-bold border-b-2 border-black hover:bg-blue-600 hover:text-white transition-all"
                >
                  <User class="w-4 h-4" />
                  MY PROFILE
                </a>
                <button
                  onclick={logout}
                  class="w-full flex items-center gap-2 px-4 py-3 font-bold text-left hover:bg-red-600 hover:text-white transition-all"
                >
                  <LogOut class="w-4 h-4" />
                  LOG OUT
                </button>
              </div>
            {/if}
          </div>
        {:else}
          <a href="/login"
            class="px-4 py-2 neo-border font-bold bg-white text-black hover:bg-gray-100 transition-all hidden sm:block">
            LOG IN
          </a>
          <a href="/register"
            class="px-4 py-2 neo-border neo-shadow font-bold bg-blue-600 text-white hover:bg-blue-700 transition-all">
            SIGN UP
          </a>
        {/if}
      </div>
    </div>
  </header>

  <!-- Main Content -->
  <main>
    {@render children()}
  </main>

  <!-- Mobile Bottom Nav -->
  <nav class="md:hidden fixed bottom-0 left-0 right-0 bg-white neo-border border-t-4 p-2 z-50">
    <div class="flex justify-around">
      <a
        href="/dashboard"
        class={`flex flex-col items-center gap-1 px-6 py-2 neo-border text-xs font-bold transition-all ${
          currentUrl === '/dashboard' ? 'bg-blue-600 text-white' : 'bg-white text-black'
        }`}
      >
        <Calendar class="w-5 h-5" />
        EVENTS
      </a>
      <a
        href={createPageUrl("MapView")}
        class={`flex flex-col items-center gap-1 px-6 py-2 neo-border text-xs font-bold transition-all ${
          currentUrl === createPageUrl("MapView") ? 'bg-blue-600 text-white' : 'bg-white text-black'
        }`}
      >
        <MapPin class="w-5 h-5" />
        NEAR ME
      </a>
      {#if user}
        <a
          href="/profile"
          class={`flex flex-col items-center gap-1 px-6 py-2 neo-border text-xs font-bold transition-all ${
            currentUrl === '/profile' ? 'bg-blue-600 text-white' : 'bg-white text-black'
          }`}
        >
          <User class="w-5 h-5" />
          PROFILE
        </a>
      {:else}
        <a
          href="/login"
          class="flex flex-col items-center gap-1 px-6 py-2 neo-border text-xs font-bold bg-white text-black"
        >
          <User class="w-5 h-5" />
          LOG IN
        </a>
      {/if}
    </div>
  </nav>
</div>
