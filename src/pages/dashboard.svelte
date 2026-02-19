<script>
  import { Event } from "@/entities/Event";
  import { User } from "@/entities/User";
  import Button from "@/components/ui/Button.svelte";
  import Input from "@/components/ui/Input.svelte";
  import { MapPin, Filter, Search, Zap, X } from "lucide-svelte";
  import { createPageUrl } from "@/utils";
  import EventCard from "@/components/events/EventCard.svelte";
  import LocationDetector from "@/components/dashboard/LocationDetector.svelte";
  import QuickActions from "@/components/dashboard/QuickActions.svelte";
  import StatsPanel from "@/components/dashboard/StatsPanel.svelte";
  import Layout from "@/components/Layout.svelte";

  let events = $state([]);
  let user = $state(null);
  let loading = $state(true);
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

  // Dancer's saved styles (for filter pre-population)
  let dancerStyles = $derived(
    Array.isArray(user?.dance_styles) && user.dance_styles.length > 0
      ? user.dance_styles
      : []
  );

  let filteredEvents = $derived(
    Array.isArray(events)
      ? events.filter(event => {
          const matchesSearch =
            event.title?.toLowerCase().includes(searchTerm.toLowerCase()) ||
            event.description?.toLowerCase().includes(searchTerm.toLowerCase());
          const matchesStyle =
            selectedStyle === "all" ||
            event.dance_styles?.includes(selectedStyle);
          return matchesSearch && matchesStyle;
        })
      : []
  );

  $effect(async () => {
    // Check first-time dancer flag
    if (localStorage.getItem('is_new_dancer') === 'true') {
      isNewDancer = true;
      showWelcomeBanner = true;
    }

    try {
      const currentUser = await User.me();
      user = currentUser;

      // Auto-select first saved dance style if dancer has preferences
      if (
        Array.isArray(currentUser.dance_styles) &&
        currentUser.dance_styles.length > 0 &&
        selectedStyle === "all"
      ) {
        selectedStyle = currentUser.dance_styles[0];
      }

      // Use saved location if available and no location detected yet
      if (!location && currentUser.city) {
        location = {
          city: currentUser.city,
          state: currentUser.state || '',
          latitude: currentUser.latitude || null,
          longitude: currentUser.longitude || null
        };
      }
    } catch (error) {
      console.log("User not authenticated");
    }

    const popularEvents = await Event.list("-current_attendees", 20);
    events = Array.isArray(popularEvents) ? popularEvents : [];
    loading = false;
  });

  $effect(async () => {
    if (!location) return;

    loading = true;
    try {
      const nearbyEvents = await Event.filter(
        { city: location.city, state: location.state },
        "-start_date",
        50
      );

      if (!nearbyEvents || nearbyEvents.length === 0) {
        const popularEvents = await Event.list("-current_attendees", 20);
        events = Array.isArray(popularEvents) ? popularEvents : [];
      } else {
        events = Array.isArray(nearbyEvents) ? nearbyEvents : [];
      }
    } finally {
      loading = false;
    }
  });

  function dismissWelcomeBanner() {
    showWelcomeBanner = false;
    localStorage.removeItem('is_new_dancer');
  }

  function isDancer() {
    if (!user) return false;
    return user.role === 'dancer' || user.user_type === 'dancer' || user.user_type === 'both';
  }

  function getHeroTitle() {
    if (user && isNewDancer) return `WELCOME, ${(user.full_name || user.name || 'DANCER').toUpperCase().split(' ')[0]}!`;
    if (user && isDancer()) return 'DANCE THE WORLD';
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

    <!-- First-Time Dancer Welcome Banner -->
    {#if showWelcomeBanner}
      <div class="bg-green-500 text-white p-4 relative neo-border-b">
        <div class="max-w-7xl mx-auto flex items-center justify-between gap-4">
          <p class="font-black text-lg">
            🎉 WELCOME TO THE DANCE SCENE! Your profile is live — start exploring events below.
          </p>
          <button
            onclick={dismissWelcomeBanner}
            class="flex-shrink-0 p-1 hover:bg-green-600 rounded"
            aria-label="Dismiss"
          >
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

        <!-- Dancer's saved styles chips -->
        {#if isDancer() && dancerStyles.length > 0}
          <div class="text-center mb-4">
            <p class="font-bold text-white/80 text-sm mb-2">YOUR STYLES</p>
            <div class="flex flex-wrap justify-center gap-2">
              {#each dancerStyles as style (style)}
                <button
                  onclick={() => selectedStyle = style}
                  class={`px-3 py-1 neo-border font-bold text-sm transition-all ${
                    selectedStyle === style
                      ? 'bg-white text-black neo-shadow'
                      : 'bg-white/20 text-white hover:bg-white/40'
                  }`}
                >
                  {style.toUpperCase()}
                </button>
              {/each}
              <button
                onclick={() => selectedStyle = 'all'}
                class={`px-3 py-1 neo-border font-bold text-sm transition-all ${
                  selectedStyle === 'all'
                    ? 'bg-white text-black neo-shadow'
                    : 'bg-white/20 text-white hover:bg-white/40'
                }`}
              >
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
      <!-- Quick Actions -->
      <QuickActions {user} />

      <!-- Stats Panel -->
      <StatsPanel {events} />

      <!-- Search and Filter -->
      <div class="mb-8">
        <div class="grid md:grid-cols-3 gap-4">
          <div class="md:col-span-2">
            <div class="relative">
              <Search class="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-600" />
              <Input
                placeholder="SEARCH BRUTAL DANCE EVENTS..."
                bind:value={searchTerm}
                class="pl-12 neo-border neo-shadow font-bold text-lg h-14 bg-white"
              />
            </div>
          </div>

          <div class="relative">
            <Filter class="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-600 z-10" />
            <select
              bind:value={selectedStyle}
              class="w-full pl-12 pr-4 py-4 neo-border neo-shadow font-bold text-lg bg-white appearance-none"
            >
              {#each danceStyles as style (style)}
                <option value={style}>
                  {style.toUpperCase()}
                  {dancerStyles.includes(style) ? ' ★' : ''}
                </option>
              {/each}
            </select>
          </div>
        </div>

        <!-- Active filter chip -->
        {#if selectedStyle !== 'all'}
          <div class="mt-3 flex items-center gap-2">
            <span class="font-bold text-sm text-gray-600">FILTERING BY:</span>
            <span class="bg-blue-600 text-white font-bold px-3 py-1 neo-border text-sm flex items-center gap-2">
              {selectedStyle.toUpperCase()}
              <button onclick={() => selectedStyle = 'all'} class="hover:text-blue-200">
                <X class="w-3 h-3" />
              </button>
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
                  <button onclick={() => { location = null; }} class="text-blue-600 underline ml-1">
                    Browse national events
                  </button>
                </p>
              {:else}
                <p class="font-bold text-gray-600 mb-4">Time to create some epic dance events!</p>
              {/if}
              <a href={createPageUrl("CreateEvent")}>
                <Button class="bg-pink-600 text-white neo-border neo-shadow neo-hover font-black">
                  CREATE FIRST EVENT
                </Button>
              </a>
            </div>
          </div>
        {/if}
      </div>

      <!-- CTA for logged-out visitors -->
      {#if !user}
        <div class="text-center py-12">
          <div class="bg-yellow-400 text-black p-8 neo-border neo-shadow transform -rotate-1 inline-block">
            <h3 class="text-3xl font-black mb-4">JOIN THE DANCE REVOLUTION</h3>
            <p class="font-bold mb-6">Register as dancer or event creator</p>
            <div class="flex gap-4 justify-center">
              <a href={createPageUrl("Register")}>
                <Button class="bg-blue-600 text-white neo-border neo-shadow neo-hover font-black px-8 py-3">
                  GET STARTED
                </Button>
              </a>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</Layout>
