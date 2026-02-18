<script>
  import { Event } from '@/entities/Event';
  import Button from '@/components/ui/Button.svelte';
  import Input from '@/components/ui/Input.svelte';
  import { Calendar as CalendarIcon, ChevronLeft, ChevronRight } from 'lucide-svelte';
  import {
    format,
    startOfMonth,
    endOfMonth,
    eachDayOfInterval,
    startOfWeek,
    endOfWeek,
    addMonths,
    subMonths,
    isEqual,
    isToday
  } from 'date-fns';
  import { createPageUrl } from '@/utils';
  import Layout from '@/components/Layout.svelte';

  let currentMonth = $state(new Date());
  let events = $state([]);
  let loading = $state(true);
  let filters = $state({ city: '', state: '', zip: '' });

  $effect(async () => {
    loading = true;
    const allEvents = await Event.list();
    let filteredEvents = allEvents;
    if (filters.city)
      filteredEvents = filteredEvents.filter(e =>
        e.city.toLowerCase().includes(filters.city.toLowerCase())
      );
    if (filters.state)
      filteredEvents = filteredEvents.filter(e =>
        e.state.toLowerCase().includes(filters.state.toLowerCase())
      );
    if (filters.zip)
      filteredEvents = filteredEvents.filter(e => e.zip_code.includes(filters.zip));
    events = filteredEvents;
    loading = false;
  });

  const dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

  function getDaysInCalendar() {
    const monthStart = startOfMonth(currentMonth);
    const monthEnd = endOfMonth(currentMonth);
    const startDate = startOfWeek(monthStart);
    const endDate = endOfWeek(monthEnd);
    return eachDayOfInterval({ start: startDate, end: endDate });
  }

  function getEventsByDate() {
    return events.reduce((acc, event) => {
      const dateKey = format(new Date(event.start_date), 'yyyy-MM-dd');
      if (!acc[dateKey]) acc[dateKey] = [];
      acc[dateKey].push(event);
      return acc;
    }, {});
  }

  let days = $derived(getDaysInCalendar());
  let eventsByDate = $derived(getEventsByDate());
</script>

<Layout>
  <div class="p-6 bg-white">
    <div class="max-w-7xl mx-auto">
      <div class="text-center mb-8">
        <h1 class="text-5xl font-black mb-4 transform -rotate-1 flex items-center justify-center gap-4">
          <CalendarIcon class="w-12 h-12 text-blue-600" />
          BRUTAL EVENT CALENDAR
        </h1>
      </div>

      <div class="grid md:grid-cols-3 gap-4 mb-8">
        <Input
          placeholder="Filter by City..."
          bind:value={filters.city}
          class="neo-border font-bold h-12"
        />
        <Input
          placeholder="Filter by State..."
          bind:value={filters.state}
          class="neo-border font-bold h-12"
        />
        <Input
          placeholder="Filter by Zip Code..."
          bind:value={filters.zip}
          class="neo-border font-bold h-12"
        />
      </div>

      <div class="flex justify-between items-center mb-6">
        <Button
          onclick={() => (currentMonth = subMonths(currentMonth, 1))}
          class="neo-border neo-shadow neo-hover bg-white text-black"
        >
          <ChevronLeft />
        </Button>
        <h2 class="text-3xl font-black">{format(currentMonth, 'MMMM yyyy')}</h2>
        <Button
          onclick={() => (currentMonth = addMonths(currentMonth, 1))}
          class="neo-border neo-shadow neo-hover bg-white text-black"
        >
          <ChevronRight />
        </Button>
      </div>

      {#if loading}
        <div class="h-[600px] bg-gray-200 neo-border animate-pulse flex items-center justify-center font-black text-2xl">
          LOADING CALENDAR...
        </div>
      {:else}
        <div class="grid grid-cols-7 gap-px bg-black neo-border">
          {#each dayNames as day (day)}
            <div class="bg-gray-200 p-2 text-center font-black">{day}</div>
          {/each}
          {#each days as day (day.toString())}
            {@const dateKey = format(day, 'yyyy-MM-dd')}
            {@const dayEvents = eventsByDate[dateKey] || []}
            {@const monthStart = startOfMonth(currentMonth)}
            <div
              class={`bg-white p-2 min-h-[120px] ${!isEqual(startOfMonth(day), monthStart) ? 'bg-gray-100' : ''}`}
            >
              <div
                class={`font-black ${isToday(day) ? 'bg-yellow-400 rounded-full w-8 h-8 flex items-center justify-center neo-border' : ''}`}
              >
                {format(day, 'd')}
              </div>
              <div class="mt-1 space-y-1">
                {#each dayEvents.slice(0, 2) as event (event.id)}
                  <a href={`/event?id=${event.id}`}>
                    <div class="text-xs font-bold bg-blue-600 text-white p-1 truncate neo-border cursor-pointer neo-hover">
                      {event.title}
                    </div>
                  </a>
                {/each}
                {#if dayEvents.length > 2}
                  <div class="text-xs font-bold text-center bg-gray-300 p-1 neo-border">
                    +{dayEvents.length - 2} more
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</Layout>
