<script>
  import { Event } from "@/entities/Event";
  import { User } from "@/entities/User";
  import { Registration } from "@/entities/Registration";
  import Button from "@/components/ui/Button.svelte";
  import Input from "@/components/ui/Input.svelte";
  import { MapPin, Filter, Search, Zap, X, Plus, Edit2, Trash2, Users, BarChart2 } from "lucide-svelte";
  import { createPageUrl } from "@/utils";
  import EventCard from "@/components/events/EventCard.svelte";
  import LocationDetector from "@/components/dashboard/LocationDetector.svelte";
  import QuickActions from "@/components/dashboard/QuickActions.svelte";
  import StatsPanel from "@/components/dashboard/StatsPanel.svelte";
  import Layout from "@/components/Layout.svelte";

  // ── shared ──
  let user = $state(null);
  let loading = $state(true);

  // ── dancer state ──
  let events = $state([]);
  let location = $state(null);
  let searchTerm = $state("");
  let selectedStyle = $state("all");
  let isNewDancer = $state(false);
  let showWelcomeBanner = $state(false);

  const danceStyles = [
    "all", "salsa", "bachata", "kizomba", "swing", "tango",
    "hip-hop", "contemporary", "ballroom", "latin", "jazz", "house",
    "lindy hop", "west coast swing", "zouk", "blues", "fusion"
  ];

  let dancerStyles = $derived(
    Array.isArray(user?.dance_styles) && user.dance_styles.length > 0
      ? user.dance_styles : []
  );

  let filteredEvents = $derived(
    Array.isArray(events)
      ? events.filter(ev => {
          const matchesSearch =
            ev.title?.toLowerCase().includes(searchTerm.toLowerCase()) ||
            ev.description?.toLowerCase().includes(searchTerm.toLowerCase());
          const matchesStyle =
            selectedStyle === "all" || ev.dance_styles?.includes(selectedStyle);
          return matchesSearch && matchesStyle;
        })
      : []
  );

  // ── creator state ──
  let creatorEvents = $state([]);
  let regsModal = $state({
    open: false, eventId: null, eventTitle: '', regs: [], search: '', loading: false
  });

  let creatorStats = $derived({
    total:    creatorEvents.length,
    upcoming: creatorEvents.filter(e => new Date(e.start_date) >= new Date()).length,
    totalRegs: creatorEvents.reduce((s, e) => s + (e.current_attendees || 0), 0),
    capacity:  creatorEvents.reduce((s, e) => s + (e.max_attendees || 0), 0),
  });

  let filteredRegs = $derived(
    regsModal.regs.filter(r => {
      const q = regsModal.search.toLowerCase();
      return !q ||
        r.dancer_name?.toLowerCase().includes(q) ||
        r.user_email?.toLowerCase().includes(q);
    })
  );

  // ── helpers ──
  function isCreator() {
    if (!user) return false;
    return (
      user.role === 'creator' ||
      user.role === 'admin' ||
      user.user_type === 'creator' ||
      user.user_type === 'both'
    );
  }

  function isDancer() {
    if (!user) return false;
    return user.role === 'dancer' || user.user_type === 'dancer' || user.user_type === 'both';
  }

  function statusColor(status) {
    const map = { public: 'bg-green-500', draft: 'bg-yellow-400 text-black', cancelled: 'bg-red-500', completed: 'bg-gray-500' };
    return map[status] || 'bg-gray-400';
  }

  function formatDate(d) {
    if (!d) return '—';
    return new Date(d).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }

  function exportRegsCSV() {
    const rows = [['Name', 'Email', 'Status', 'Check-in', 'Registered']];
    filteredRegs.forEach(r => rows.push([
      r.dancer_name || '—', r.user_email, r.status, r.check_in_status,
      new Date(r.created_at).toLocaleDateString()
    ]));
    const csv = rows.map(r => r.join(',')).join('\n');
    const a = Object.assign(document.createElement('a'), {
      href: URL.createObjectURL(new Blob([csv], { type: 'text/csv' })),
      download: `registrations-${regsModal.eventId}.csv`
    });
    a.click();
  }

  async function openRegistrations(ev) {
    regsModal = { open: true, eventId: ev.id, eventTitle: ev.title, regs: [], search: '', loading: true };
    try {
      const data = await Registration.getByEvent(ev.id);
      regsModal = { ...regsModal, regs: Array.isArray(data) ? data : [], loading: false };
    } catch {
      regsModal = { ...regsModal, regs: [], loading: false };
    }
  }

  async function deleteCreatorEvent(id) {
    if (!confirm('Delete this event? This cannot be undone.')) return;
    try {
      await Event.delete(id);
      creatorEvents = creatorEvents.filter(e => e.id !== id);
    } catch {
      alert('Failed to delete event.');
    }
  }

  // ── data loading ──
  $effect(async () => {
    if (localStorage.getItem('is_new_dancer') === 'true') {
      isNewDancer = true;
      showWelcomeBanner = true;
    }

    try {
      const currentUser = await User.me();
      user = currentUser;

      if (Array.isArray(currentUser.dance_styles) && currentUser.dance_styles.length > 0 && selectedStyle === "all") {
        selectedStyle = currentUser.dance_styles[0];
      }
      if (!location && currentUser.city) {
        location = { city: currentUser.city, state: currentUser.state || '', latitude: currentUser.latitude || null, longitude: currentUser.longitude || null };
      }
    } catch {
      console.log("User not authenticated");
    }

    const popularEvents = await Event.list("-current_attendees", 20);
    events = Array.isArray(popularEvents) ? popularEvents : [];
    loading = false;
  });

  // Load creator's own events
  $effect(async () => {
    if (!user || !isCreator()) return;
    try {
      const myEvents = await Event.getByOrganizer(user.id);
      creatorEvents = Array.isArray(myEvents) ? myEvents : [];
    } catch {
      creatorEvents = [];
    }
  });

  $effect(async () => {
    if (!location || isCreator()) return;
    loading = true;
    try {
      const nearbyEvents = await Event.filter({ city: location.city, state: location.state }, "-start_date", 50);
      events = (nearbyEvents?.length ? nearbyEvents : await Event.list("-current_attendees", 20)) ?? [];
    } finally {
      loading = false;
    }
  });

  function dismissWelcomeBanner() {
    showWelcomeBanner = false;
    localStorage.removeItem('is_new_dancer');
  }

  function getHeroTitle() {
    if (user && isNewDancer) return `WELCOME, ${(user.full_name || 'DANCER').toUpperCase().split(' ')[0]}!`;
    return 'DANCE THE WORLD';
  }

  function getEventSectionTitle() {
    if (selectedStyle !== 'all') return `${selectedStyle.toUpperCase()} EVENTS`;
    if (location) return 'LOCAL DANCE EVENTS';
    return 'POPULAR DANCE EVENTS';
  }
</script>

<Layout>
  <div class="min-h-screen bg-white">

    {#if isCreator()}
      <!-- ══════════════════════════════════════════
           CREATOR / ORGANIZER DASHBOARD
      ══════════════════════════════════════════ -->

      <!-- Hero -->
      <section class="bg-gradient-to-br from-gray-900 via-blue-900 to-purple-900 text-white p-8">
        <div class="max-w-7xl mx-auto">
          <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
            <div>
              <h1 class="text-4xl md:text-5xl font-black transform -rotate-1">
                WELCOME BACK, {(user?.full_name || 'ORGANIZER').toUpperCase().split(' ')[0]}!
              </h1>
              <p class="font-bold text-blue-300 mt-1">
                You have {creatorStats.upcoming} upcoming event{creatorStats.upcoming !== 1 ? 's' : ''}
              </p>
            </div>
            <a href="/create-event"
              class="flex items-center gap-2 px-6 py-4 bg-pink-600 text-white neo-border neo-shadow font-black text-lg hover:bg-pink-700 transition-all self-start">
              <Plus class="w-6 h-6" />
              CREATE EVENT
            </a>
          </div>

          <!-- Stats bar -->
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-8">
            {#each [
              { label: 'TOTAL EVENTS',       value: creatorStats.total },
              { label: 'UPCOMING',           value: creatorStats.upcoming },
              { label: 'TOTAL REGISTRATIONS', value: creatorStats.totalRegs },
              { label: 'CONVERSION',         value: creatorStats.capacity ? Math.round(creatorStats.totalRegs / creatorStats.capacity * 100) + '%' : '—' }
            ] as stat (stat.label)}
              <div class="bg-white/10 neo-border p-4 text-center">
                <p class="text-3xl font-black">{stat.value}</p>
                <p class="text-xs font-bold text-blue-300 mt-1">{stat.label}</p>
              </div>
            {/each}
          </div>
        </div>
      </section>

      <div class="max-w-7xl mx-auto p-6">

        <!-- Welcome banner (new organizer) -->
        {#if showWelcomeBanner}
          <div class="mb-6 bg-green-500 text-white p-4 neo-border flex items-center justify-between">
            <p class="font-black">🎉 Your organizer profile is live! Start by creating your first event.</p>
            <button onclick={dismissWelcomeBanner}><X class="w-5 h-5" /></button>
          </div>
        {/if}

        <!-- My Events -->
        <div class="mb-8">
          <h2 class="text-3xl font-black mb-6 flex items-center gap-3">
            <BarChart2 class="w-8 h-8 text-blue-600" />
            MY EVENTS
          </h2>

          {#if creatorEvents.length === 0}
            <div class="text-center py-16 neo-border neo-shadow bg-gray-50">
              <p class="text-2xl font-black mb-2">NO EVENTS YET</p>
              <p class="font-bold text-gray-500 mb-6">Create your first event to start accepting registrations.</p>
              <a href="/create-event"
                class="inline-flex items-center gap-2 px-6 py-3 bg-pink-600 text-white neo-border neo-shadow font-black hover:bg-pink-700 transition-all">
                <Plus class="w-5 h-5" /> CREATE FIRST EVENT
              </a>
            </div>
          {:else}
            <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {#each creatorEvents as ev (ev.id)}
                <div class="neo-border neo-shadow bg-white flex flex-col">
                  <!-- Color bar -->
                  <div class="h-2 {ev.status === 'public' ? 'bg-green-500' : ev.status === 'draft' ? 'bg-yellow-400' : 'bg-gray-400'}"></div>

                  <div class="p-5 flex-1 flex flex-col">
                    <!-- Status + date -->
                    <div class="flex items-center justify-between mb-3">
                      <span class="px-2 py-0.5 text-xs font-black text-white {statusColor(ev.status)} {ev.status === 'draft' ? '!text-black' : ''}">
                        {(ev.status || 'PUBLIC').toUpperCase()}
                      </span>
                      <span class="text-sm font-bold text-gray-500">{formatDate(ev.start_date)}</span>
                    </div>

                    <h3 class="text-xl font-black mb-1 line-clamp-2">{ev.title}</h3>
                    <p class="text-sm font-bold text-gray-500 mb-3">
                      <MapPin class="w-3 h-3 inline mr-1" />{ev.city}, {ev.state}
                    </p>

                    <!-- Attendees bar -->
                    {#if ev.max_attendees}
                      <div class="mb-4">
                        <div class="flex justify-between text-xs font-bold mb-1">
                          <span>{ev.current_attendees || 0}/{ev.max_attendees} dancers</span>
                          <span>{Math.round(((ev.current_attendees || 0) / ev.max_attendees) * 100)}%</span>
                        </div>
                        <div class="h-2 bg-gray-200 neo-border">
                          <div class="h-full bg-blue-600 transition-all"
                            style="width: {Math.min(100, Math.round(((ev.current_attendees || 0) / ev.max_attendees) * 100))}%">
                          </div>
                        </div>
                      </div>
                    {:else}
                      <p class="text-sm font-bold text-gray-400 mb-4">{ev.current_attendees || 0} registered</p>
                    {/if}

                    <!-- Actions -->
                    <div class="flex gap-2 mt-auto">
                      <a href="/create-event?id={ev.id}"
                        class="flex-1 flex items-center justify-center gap-1 py-2 neo-border font-bold text-sm bg-white hover:bg-blue-600 hover:text-white transition-all">
                        <Edit2 class="w-4 h-4" /> EDIT
                      </a>
                      <button
                        onclick={() => openRegistrations(ev)}
                        class="flex-1 flex items-center justify-center gap-1 py-2 neo-border font-bold text-sm bg-white hover:bg-green-600 hover:text-white transition-all">
                        <Users class="w-4 h-4" /> REGS
                      </button>
                      <button
                        onclick={() => deleteCreatorEvent(ev.id)}
                        class="py-2 px-3 neo-border bg-white hover:bg-red-600 hover:text-white transition-all">
                        <Trash2 class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Browse all public events link -->
        <div class="text-center py-6 border-t-2 border-black">
          <p class="font-bold text-gray-600 mb-3">Want to explore events as a dancer?</p>
          <a href="?browse=1" onclick={(e) => { e.preventDefault(); user = { ...user, user_type: 'dancer' }; }}
            class="px-6 py-2 neo-border font-bold bg-white hover:bg-blue-600 hover:text-white transition-all">
            BROWSE ALL EVENTS →
          </a>
        </div>
      </div>

      <!-- Registrations Modal -->
      {#if regsModal.open}
        <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
          <button class="absolute inset-0 bg-black/60" onclick={() => regsModal = { ...regsModal, open: false }}></button>
          <div class="relative bg-white neo-border neo-shadow w-full max-w-2xl max-h-[85vh] flex flex-col z-10">
            <!-- Modal header -->
            <div class="p-4 border-b-2 border-black flex items-center justify-between">
              <div>
                <h3 class="text-xl font-black">REGISTRATIONS</h3>
                <p class="text-sm font-bold text-gray-600 truncate max-w-xs">{regsModal.eventTitle}</p>
              </div>
              <div class="flex items-center gap-2">
                <button onclick={exportRegsCSV}
                  class="px-3 py-1 neo-border font-bold text-sm bg-green-600 text-white hover:bg-green-700 transition-all">
                  EXPORT CSV
                </button>
                <button onclick={() => regsModal = { ...regsModal, open: false }}
                  class="p-1 neo-border hover:bg-red-600 hover:text-white transition-all">
                  <X class="w-5 h-5" />
                </button>
              </div>
            </div>

            <!-- Search -->
            <div class="p-3 border-b border-gray-200">
              <input bind:value={regsModal.search} placeholder="Search by name or email..."
                class="w-full px-3 py-2 neo-border font-bold text-sm" />
            </div>

            <!-- List -->
            <div class="flex-1 overflow-y-auto">
              {#if regsModal.loading}
                <div class="p-8 text-center font-black text-gray-500">LOADING...</div>
              {:else if filteredRegs.length === 0}
                <div class="p-8 text-center font-black text-gray-500">
                  {regsModal.regs.length === 0 ? 'NO REGISTRATIONS YET' : 'NO MATCHES'}
                </div>
              {:else}
                <table class="w-full text-sm">
                  <thead class="bg-gray-100 border-b-2 border-black sticky top-0">
                    <tr>
                      <th class="text-left p-3 font-black">NAME</th>
                      <th class="text-left p-3 font-black">EMAIL</th>
                      <th class="text-left p-3 font-black">STATUS</th>
                      <th class="text-left p-3 font-black">DATE</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each filteredRegs as r (r.id)}
                      <tr class="border-b border-gray-100 hover:bg-gray-50">
                        <td class="p-3 font-bold">{r.dancer_name || '—'}</td>
                        <td class="p-3 text-gray-600">{r.user_email}</td>
                        <td class="p-3">
                          <span class="px-2 py-0.5 text-xs font-black {r.status === 'active' ? 'bg-green-500 text-white' : 'bg-red-500 text-white'}">
                            {r.status?.toUpperCase()}
                          </span>
                        </td>
                        <td class="p-3 text-gray-500 text-xs">{new Date(r.created_at).toLocaleDateString()}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              {/if}
            </div>

            <div class="p-3 border-t-2 border-black bg-gray-50 text-sm font-bold text-gray-600">
              {filteredRegs.length} of {regsModal.regs.length} registration{regsModal.regs.length !== 1 ? 's' : ''}
            </div>
          </div>
        </div>
      {/if}

    {:else}
      <!-- ══════════════════════════════════════════
           DANCER / PUBLIC DASHBOARD
      ══════════════════════════════════════════ -->

      <!-- First-Time Dancer Welcome Banner -->
      {#if showWelcomeBanner}
        <div class="bg-green-500 text-white p-4 relative neo-border-b">
          <div class="max-w-7xl mx-auto flex items-center justify-between gap-4">
            <p class="font-black text-lg">
              🎉 WELCOME TO THE DANCE SCENE! Your profile is live — start exploring events below.
            </p>
            <button onclick={dismissWelcomeBanner} class="flex-shrink-0 p-1 hover:bg-green-600 rounded" aria-label="Dismiss">
              <X class="w-5 h-5" />
            </button>
          </div>
        </div>
      {/if}

      <!-- Hero Section -->
      <section class="bg-gradient-to-br from-blue-600 via-pink-600 to-purple-600 text-white p-8">
        <div class="max-w-7xl mx-auto">
          <div class="text-center mb-8">
            <h1 class="text-5xl md:text-7xl font-black mb-4 transform -rotate-1 neo-shadow">
              {getHeroTitle()}
            </h1>
            <p class="text-xl font-bold transform rotate-1 bg-yellow-400 text-black px-6 py-2 neo-border neo-shadow inline-block">
              BRUTAL EVENTS • EPIC MOVES • PURE ENERGY
            </p>
          </div>

          {#if isDancer() && dancerStyles.length > 0}
            <div class="text-center mb-4">
              <p class="font-bold text-white/80 text-sm mb-2">YOUR STYLES</p>
              <div class="flex flex-wrap justify-center gap-2">
                {#each dancerStyles as style (style)}
                  <button onclick={() => selectedStyle = style}
                    class={`px-3 py-1 neo-border font-bold text-sm transition-all ${selectedStyle === style ? 'bg-white text-black neo-shadow' : 'bg-white/20 text-white hover:bg-white/40'}`}>
                    {style.toUpperCase()}
                  </button>
                {/each}
                <button onclick={() => selectedStyle = 'all'}
                  class={`px-3 py-1 neo-border font-bold text-sm transition-all ${selectedStyle === 'all' ? 'bg-white text-black neo-shadow' : 'bg-white/20 text-white hover:bg-white/40'}`}>
                  ALL
                </button>
              </div>
            </div>
          {/if}

          <LocationDetector onLocationFound={(loc) => (location = loc)} />

          {#if location}
            <div class="text-center mt-6">
              <div class="bg-white text-black px-6 py-3 neo-border neo-shadow inline-block font-bold">
                <MapPin class="w-5 h-5 inline mr-2" />
                SHOWING EVENTS IN {location.city?.toUpperCase()}, {location.state?.toUpperCase()}
              </div>
            </div>
          {/if}
        </div>
      </section>

      <div class="max-w-7xl mx-auto p-6">
        <QuickActions {user} />
        <StatsPanel {events} />

        <!-- Search and Filter -->
        <div class="mb-8">
          <div class="grid md:grid-cols-3 gap-4">
            <div class="md:col-span-2">
              <div class="relative">
                <Search class="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-600" />
                <Input placeholder="SEARCH DANCE EVENTS..." bind:value={searchTerm}
                  class="pl-12 neo-border neo-shadow font-bold text-lg h-14 bg-white" />
              </div>
            </div>
            <div class="relative">
              <Filter class="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-600 z-10" />
              <select bind:value={selectedStyle}
                class="w-full pl-12 pr-4 py-4 neo-border neo-shadow font-bold text-lg bg-white appearance-none">
                {#each danceStyles as style (style)}
                  <option value={style}>{style.toUpperCase()}{dancerStyles.includes(style) ? ' ★' : ''}</option>
                {/each}
              </select>
            </div>
          </div>
          {#if selectedStyle !== 'all'}
            <div class="mt-3 flex items-center gap-2">
              <span class="font-bold text-sm text-gray-600">FILTERING BY:</span>
              <span class="bg-blue-600 text-white font-bold px-3 py-1 neo-border text-sm flex items-center gap-2">
                {selectedStyle.toUpperCase()}
                <button onclick={() => selectedStyle = 'all'}><X class="w-3 h-3" /></button>
              </span>
            </div>
          {/if}
        </div>

        <!-- Events Grid -->
        <div class="mb-8">
          <h2 class="text-3xl font-black mb-6 flex items-center gap-3">
            <Zap class="w-8 h-8 text-yellow-500" />
            {getEventSectionTitle()}
          </h2>

          {#if loading}
            <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {#each Array(6) as _, i (i)}
                <div class="h-80 bg-gray-200 neo-border neo-shadow animate-pulse"></div>
              {/each}
            </div>
          {:else if filteredEvents.length > 0}
            <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {#each filteredEvents as event (event.id)}
                <EventCard {event} />
              {/each}
            </div>
          {:else}
            <div class="text-center py-16">
              <div class="bg-gray-100 neo-border neo-shadow p-8 inline-block transform rotate-1">
                <h3 class="text-2xl font-black mb-2">NO EVENTS FOUND</h3>
                {#if location}
                  <p class="font-bold text-gray-600 mb-4">
                    No events in {location.city} yet.
                    <button onclick={() => { location = null; }} class="text-blue-600 underline ml-1">Browse national events</button>
                  </p>
                {:else}
                  <p class="font-bold text-gray-600 mb-4">No events match your filters.</p>
                {/if}
              </div>
            </div>
          {/if}
        </div>

        {#if !user}
          <div class="text-center py-12">
            <div class="bg-yellow-400 text-black p-8 neo-border neo-shadow transform -rotate-1 inline-block">
              <h3 class="text-3xl font-black mb-4">JOIN THE DANCE REVOLUTION</h3>
              <p class="font-bold mb-6">Register as dancer or event creator</p>
              <a href={createPageUrl("Register")}>
                <Button class="bg-blue-600 text-white neo-border neo-shadow neo-hover font-black px-8 py-3">
                  GET STARTED
                </Button>
              </a>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</Layout>
