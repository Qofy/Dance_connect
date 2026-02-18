<script>
  import { Event } from '@/entities/Event';
  import EventCard from '@/components/events/EventCard.svelte';
  import Layout from '@/components/Layout.svelte';

  let todayEvents = $state([]);
  let loading = $state(true);

  $effect(async () => {
    try {
      const allEvents = await Event.list();
      const today = new Date().toDateString();
      const eventsToday = allEvents.filter(event =>
        new Date(event.start_date).toDateString() === today
      );
      todayEvents = eventsToday;
    } catch (error) {
      console.error('Failed to fetch events:', error);
      todayEvents = [];
    }
    loading = false;
  });
</script>

<Layout>
  <div class="p-6">
    <h1 class="text-3xl font-black mb-6">TODAY'S EVENTS</h1>
    {#if loading}
      <div class="bg-gray-200 neo-border neo-shadow p-8 animate-pulse">
        <p class="text-xl font-bold">Loading today's events...</p>
      </div>
    {:else if todayEvents.length > 0}
      <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
        {#each todayEvents as event (event.id)}
          <EventCard {event} />
        {/each}
      </div>
    {:else}
      <div class="bg-white neo-border neo-shadow p-8 text-center">
        <p class="text-xl font-bold">No events scheduled for today</p>
      </div>
    {/if}
  </div>
</Layout>
