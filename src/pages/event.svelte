<script>
  import { page } from '@roxi/routify';
  import { Event } from "@/entities/Event";
  import { MapPin, User as UserIcon, Calendar } from "lucide-svelte";
  import { Button } from "@/components/ui/Button.svelte";
  import Layout from "@/components/Layout.svelte";

  let event = $state(null);
  let loading = $state(true);

  $effect(async () => {
    const eventId = $page.query.id;
    if (!eventId) {
      loading = false;
      return;
    }

    try {
      const eventData = await Event.get(eventId);
      event = eventData;
    } catch (error) {
      console.error('Failed to fetch event:', error);
    }
    loading = false;
  });
</script>

<Layout>
  {#if loading}
    <div class="p-6">
      <div class="max-w-4xl mx-auto">
        <div class="h-64 bg-gray-200 neo-border neo-shadow animate-pulse" />
      </div>
    </div>
  {:else if !event}
    <div class="p-6">
      <div class="max-w-4xl mx-auto text-center">
        <h1 class="text-3xl font-black mb-4">EVENT NOT FOUND</h1>
        <p class="font-bold">The event you're looking for doesn't exist.</p>
      </div>
    </div>
  {:else}
    <div class="p-6 bg-white">
      <div class="max-w-4xl mx-auto">
        <div class="bg-gradient-to-br from-blue-600 to-pink-600 text-white p-8 neo-border neo-shadow mb-8">
          <h1 class="text-4xl font-black mb-4">{event.title}</h1>
          <div class="flex flex-wrap gap-4 text-lg font-bold">
            <div class="flex items-center gap-2">
              <MapPin class="w-5 h-5" />
              {event.city}, {event.state}
            </div>
            <div class="flex items-center gap-2">
              <UserIcon class="w-5 h-5" />
              {event.current_attendees} dancers
            </div>
            <div class="flex items-center gap-2">
              <Calendar class="w-5 h-5" />
              {new Date(event.start_date).toLocaleDateString()}
            </div>
          </div>
        </div>

        <div class="grid md:grid-cols-2 gap-8">
          <div class="bg-white neo-border neo-shadow p-6">
            <h2 class="text-2xl font-black mb-4">EVENT DETAILS</h2>
            <div class="space-y-3">
              <p><strong>Venue:</strong> {event.venue_name || 'TBA'}</p>
              <p><strong>Address:</strong> {event.address || 'TBA'}</p>
              <p><strong>Price:</strong> ${event.ticket_price || 'Free'}</p>
              <p><strong>Max Attendees:</strong> {event.max_attendees || 'Unlimited'}</p>
              <p><strong>Type:</strong> {event.event_type || 'Event'}</p>
            </div>
          </div>

          <div class="bg-white neo-border neo-shadow p-6">
            <h2 class="text-2xl font-black mb-4">DESCRIPTION</h2>
            <p class="font-bold">{event.description || 'No description available.'}</p>

            {#if event.dance_styles && event.dance_styles.length > 0}
              <div class="mt-4">
                <h3 class="font-black mb-2">DANCE STYLES:</h3>
                <div class="flex flex-wrap gap-2">
                  {#each event.dance_styles as style (style)}
                    <span class="bg-yellow-400 text-black px-3 py-1 neo-border font-bold text-sm">
                      {style.toUpperCase()}
                    </span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>

        <div class="mt-8 text-center">
          <Button class="bg-blue-600 text-white neo-border neo-shadow neo-hover font-black px-8 py-3">
            REGISTER FOR EVENT
          </Button>
        </div>
      </div>
    </div>
  {/if}
</Layout>
