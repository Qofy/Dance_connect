<script>
  import { Event } from '@/entities/Event';
  import Button from '@/components/ui/Button.svelte';
  import { Trash2, Edit, Plus } from 'lucide-svelte';
  import { createPageUrl } from '@/utils';
  import Layout from '@/components/Layout.svelte';

  let events = $state([]);
  let loading = $state(true);

  async function fetchEvents() {
    loading = true;
    try {
      const allEvents = await Event.list();
      events = allEvents || [];
    } catch (e) {
      console.error("Failed to fetch events", e);
    }
    loading = false;
  }

  async function deleteEvent(id) {
    if (confirm("Are you sure you want to delete this event? This is irreversible.")) {
      try {
        await Event.delete(id);
        alert("Event deleted.");
        fetchEvents();
      } catch (e) {
        alert("Failed to delete event.");
      }
    }
  }

  $effect(() => {
    fetchEvents();
  });
</script>

<Layout>
  {#if loading}
    <div class="p-6 font-black text-2xl text-center">LOADING EVENTS...</div>
  {:else}
    <div class="p-6 bg-white">
      <div class="max-w-7xl mx-auto">
        <div class="flex justify-between items-center mb-8">
          <h1 class="text-4xl font-black">ADMIN EVENT MANAGER</h1>
          <a href={createPageUrl("CreateEvent")}>
            <Button class="bg-green-500 text-white neo-border neo-shadow neo-hover font-bold">
              <Plus class="mr-2" /> CREATE EVENT
            </Button>
          </a>
        </div>

        <div class="overflow-x-auto neo-border bg-white">
          <table class="w-full">
            <thead class="bg-black text-white font-black">
              <tr>
                <th class="p-3 text-left">Title</th>
                <th class="p-3 text-left">City</th>
                <th class="p-3 text-left">Date</th>
                <th class="p-3 text-left">Creator</th>
                <th class="p-3 text-left">Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each events as event (event.id)}
                <tr class="border-b-2 border-black">
                  <td class="p-3 font-bold">{event.title}</td>
                  <td class="p-3 font-bold">{event.city}</td>
                  <td class="p-3 font-bold">{new Date(event.start_date).toLocaleDateString()}</td>
                  <td class="p-3 font-bold text-sm">{event.created_by}</td>
                  <td class="p-3">
                    <div class="flex gap-2">
                      <Button size="icon" class="bg-blue-600 text-white neo-border neo-shadow neo-hover">
                        <Edit class="w-4 h-4" />
                      </Button>
                      <Button
                        onclick={() => deleteEvent(event.id)}
                        size="icon"
                        class="bg-red-600 text-white neo-border neo-shadow neo-hover"
                      >
                        <Trash2 class="w-4 h-4" />
                      </Button>
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  {/if}
</Layout>
