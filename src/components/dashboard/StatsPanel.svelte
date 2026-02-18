<script>
  import { Calendar, MapPin, Users, Zap } from "lucide-svelte";

  let { events = [] } = $props();

  let stats = $derived([
    {
      title: "TOTAL EVENTS",
      value: events.length,
      icon: Calendar,
      color: "bg-blue-600"
    },
    {
      title: "CITIES",
      value: new Set(events.map(e => e.city)).size,
      icon: MapPin,
      color: "bg-green-500"
    },
    {
      title: "TOTAL DANCERS",
      value: events.reduce((sum, e) => sum + (e.current_attendees || 0), 0),
      icon: Users,
      color: "bg-purple-600"
    },
    {
      title: "ENERGY LEVEL",
      value: "MAXIMUM",
      icon: Zap,
      color: "bg-yellow-500"
    }
  ]);
</script>

<div class="my-8">
  <h2 class="text-2xl font-black mb-6">PLATFORM STATS</h2>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
    {#each stats as stat, index (stat.title)}
      <div
        class="{stat.color} text-white p-4 neo-border neo-shadow transform {index % 2 === 0 ? 'rotate-1' : '-rotate-1'}"
      >
        <div class="flex items-center gap-2 mb-2">
          <svelte:component this={stat.icon} class="w-5 h-5" />
          <span class="font-bold text-sm">{stat.title}</span>
        </div>
        <div class="text-2xl font-black">{stat.value}</div>
      </div>
    {/each}
  </div>
</div>
