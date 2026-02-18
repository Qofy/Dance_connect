<script>
  import { Event } from "@/entities/Event";
  import Button from "@/components/ui/Button.svelte";
  import { MapPin, Calendar, Users } from "lucide-svelte";
  import Layout from "@/components/Layout.svelte";

  let events = $state([]);
  let loading = $state(true);

  $effect(async () => {
    try {
      const allEvents = await Event.list();
      events = Array.isArray(allEvents) ? allEvents : [];
    } catch (error) {
      console.error('Failed to fetch events:', error);
      events = [];
    }
    loading = false;
  });
</script>

<Layout>
  <div class="p-6 bg-white">
    <div class="max-w-7xl mx-auto">
      <div class="text-center mb-8">
        <h1 class="text-5xl font-black mb-4 transform -rotate-1">
          DANCE EVENTS MAP
        </h1>
        <p class="bg-yellow-400 text-black px-6 py-2 neo-border neo-shadow inline-block font-bold transform rotate-1">
          EVENTS BY LOCATION
        </p>
      </div>

      {#if loading}
        <div class="h-[600px] w-full neo-border neo-shadow bg-gray-200 animate-pulse flex items-center justify-center">
          <p class="font-black text-2xl">LOADING EVENTS...</p>
        </div>
      {:else}
        <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {#each events as event (event.id)}
            <div class="bg-white neo-border neo-shadow p-6 transform hover:scale-105 transition-transform">
              <h3 class="text-xl font-black mb-3">{event.title}</h3>
              <div class="space-y-2 mb-4">
                <div class="flex items-center gap-2">
                  <MapPin class="w-4 h-4" />
                  <span class="font-bold">{event.city}, {event.state}</span>
                </div>
                <div class="flex items-center gap-2">
                  <Calendar class="w-4 h-4" />
                  <span>{new Date(event.start_date).toLocaleDateString()}</span>
                </div>
                <div class="flex items-center gap-2">
                  <Users class="w-4 h-4" />
                  <span>{event.current_attendees} dancers</span>
                </div>
              </div>
              <a href={`/event?id=${event.id}`}>
                <Button class="w-full bg-blue-600 text-white neo-border neo-shadow neo-hover font-bold">
                  VIEW EVENT
                </Button>
              </a>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</Layout>
