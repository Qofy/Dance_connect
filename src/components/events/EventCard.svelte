<script>
  import { Calendar, MapPin, Users, DollarSign } from "lucide-svelte";
  import { format } from "date-fns";

  let { event } = $props();

  function formatDate(dateString) {
    try {
      return format(new Date(dateString), "MMM d, yyyy");
    } catch {
      return "TBA";
    }
  }

  // Deterministic hash to replace Math.random()
  function hashCode(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return hash;
  }

  const rotations = ["rotate-1", "-rotate-1", "rotate-2", "-rotate-2"];
  const colors = ["bg-blue-600", "bg-pink-600", "bg-green-500", "bg-purple-600"];
  const randomRotation = rotations[Math.abs(hashCode(event.id)) % rotations.length];
  const randomColor = colors[Math.abs(hashCode(event.id)) % colors.length];
</script>

<a href={`/event?id=${event.id}`}>
  <div class="bg-white neo-border neo-shadow neo-hover transform {randomRotation} cursor-pointer h-full">
    <!-- Event Image/Header -->
    <div class="{randomColor} text-white p-4 h-32 flex items-center justify-center">
      {#if event.image_url}
        <img
          src={event.image_url}
          alt={event.title}
          class="w-full h-full object-cover neo-border"
        />
      {:else}
        <div class="text-center">
          <h3 class="font-black text-xl break-words">{event.title?.toUpperCase()}</h3>
        </div>
      {/if}
    </div>

    <!-- Event Details -->
    <div class="p-4">
      {#if !event.image_url}
        <h3 class="font-black text-lg mb-3 text-black">
          {event.title?.toUpperCase()}
        </h3>
      {/if}

      <div class="space-y-2 text-sm font-bold">
        <div class="flex items-center gap-2">
          <Calendar class="w-4 h-4" />
          <span>{formatDate(event.start_date)}</span>
        </div>

        <div class="flex items-center gap-2">
          <MapPin class="w-4 h-4" />
          <span>{event.city}, {event.state}</span>
        </div>

        <div class="flex items-center gap-2">
          <Users class="w-4 h-4" />
          <span>{event.current_attendees || 0} DANCERS</span>
        </div>

        {#if event.ticket_price}
          <div class="flex items-center gap-2">
            <DollarSign class="w-4 h-4" />
            <span>${event.ticket_price}</span>
          </div>
        {/if}
      </div>

      <!-- Dance Styles -->
      {#if event.dance_styles && event.dance_styles.length > 0}
        <div class="mt-3">
          <div class="flex flex-wrap gap-1">
            {#each event.dance_styles.slice(0, 3) as style, index (style)}
              <span class="px-2 py-1 text-xs font-bold neo-border text-black bg-yellow-400">
                {style.toUpperCase()}
              </span>
            {/each}
            {#if event.dance_styles.length > 3}
              <span class="px-2 py-1 text-xs font-bold neo-border text-black bg-gray-200">
                +{event.dance_styles.length - 3} MORE
              </span>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>
</a>
