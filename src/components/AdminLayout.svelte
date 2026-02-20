<script>
  import { goto } from '@roxi/routify';
  import { LayoutDashboard, Calendar, Users, BarChart2, Settings, LogOut, Shield, Menu, X } from 'lucide-svelte';

  let { children } = $props();

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  let sidebarOpen = $state(false);

  // Admin guard — runs on mount
  $effect(() => {
    const user = JSON.parse(localStorage.getItem('user') || 'null');
    if (!user) {
      _goto('/login');
    } else if (user.role !== 'admin') {
      _goto('/dashboard');
    }
  });

  function getUser() {
    return JSON.parse(localStorage.getItem('user') || 'null');
  }

  function logout() {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    _goto('/login');
  }

  const navItems = [
    { href: '/admin',          label: 'DASHBOARD',  Icon: LayoutDashboard },
    { href: '/admin/events',   label: 'EVENTS',     Icon: Calendar },
    { href: '/admin/users',    label: 'USERS',      Icon: Users },
    { href: '/admin/analytics',label: 'ANALYTICS',  Icon: BarChart2 },
    { href: '/admin/settings', label: 'SETTINGS',   Icon: Settings },
  ];

  function isActive(href) {
    if (typeof window === 'undefined') return false;
    const path = window.location.pathname.replace(/\/$/, '');
    const h = String(href).replace(/\/$/, '');

    // Exact match
    if (path === h) return true;

    // If href is not a prefix of the path, it's not active
    if (!path.startsWith(h)) return false;

    // Among all nav items that are prefixes of the path, only the longest (most specific)
    // should be considered active. This avoids multiple items getting the active class.
    return !navItems.some(item => {
      const ih = String(item.href).replace(/\/$/, '');
      return ih.length > h.length && path.startsWith(ih);
    });
  }
</script>

<div class="min-h-screen bg-gray-100 flex flex-col">
  <!-- Top Bar -->
  <header class="bg-black text-white h-14 flex items-center px-4 gap-4 flex-shrink-0 neo-border-b sticky top-0 z-50">
    <button
      onclick={() => sidebarOpen = !sidebarOpen}
      class="lg:hidden p-1 hover:bg-gray-800 rounded"
      aria-label="Toggle sidebar"
    >
      {#if sidebarOpen}
        <X class="w-5 h-5" />
      {:else}
        <Menu class="w-5 h-5" />
      {/if}
    </button>

    <div class="flex items-center gap-2 font-black text-lg">
      <Shield class="w-5 h-5 text-red-500" />
      <span>DANCECONNECT</span>
      <span class="bg-red-600 text-white text-xs font-black px-2 py-0.5 neo-border">ADMIN</span>
    </div>

    <div class="ml-auto flex items-center gap-3">
      <span class="font-bold text-sm text-gray-400 hidden sm:block">
        {getUser()?.full_name || 'Admin'}
      </span>
      <button
        onclick={logout}
        class="flex items-center gap-1 bg-red-600 text-white font-bold px-3 py-1 neo-border text-sm hover:bg-red-700 transition-colors"
      >
        <LogOut class="w-4 h-4" />
        <span class="hidden sm:inline">LOGOUT</span>
      </button>
    </div>
  </header>

  <div class="flex flex-1 overflow-hidden">
    <!-- Sidebar -->
    <aside class={`
      w-56 bg-black text-white flex-shrink-0 flex flex-col
      fixed lg:static inset-y-14 left-0 z-40
      transform transition-transform duration-200
      ${sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
    `}>
      <div class="p-4 border-b border-gray-800">
        <p class="font-black text-xs text-gray-500 tracking-widest">ADMIN PANEL</p>
      </div>

      <nav class="flex-1 p-2 space-y-1 overflow-y-auto">
        {#each navItems as { href, label, Icon } (href)}
          <a
            href={href}
            onclick={() => sidebarOpen = false}
            class={`flex items-center gap-3 px-3 py-2.5 font-bold text-sm transition-all ${
              isActive(href)
                ? 'bg-red-600 text-white neo-border'
                : 'text-gray-400 hover:bg-gray-800 hover:text-white'
            }`}
          >
            <Icon class="w-4 h-4 flex-shrink-0" />
            {label}
          </a>
        {/each}
      </nav>

      <div class="p-4 border-t border-gray-800">
        <a
          href="/dashboard"
          class="flex items-center gap-2 text-gray-500 hover:text-white font-bold text-xs transition-colors"
        >
          ← BACK TO SITE
        </a>
      </div>
    </aside>

    <!-- Overlay for mobile -->
    {#if sidebarOpen}
      <div
        class="fixed inset-0 z-30 bg-black/50 lg:hidden"
        onclick={() => sidebarOpen = false}
        role="presentation"
      ></div>
    {/if}

    <!-- Main Content -->
    <main class="flex-1 overflow-y-auto">
      {@render children()}
    </main>
  </div>
</div>

<!-- bg-red-600 text-white neo-border -->