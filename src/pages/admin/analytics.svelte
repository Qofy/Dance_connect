<script>
  import { Event } from '@/entities/Event';
  import { User } from '@/entities/User';
  import { Registration } from '@/entities/Registration';
  import AdminLayout from '@/components/AdminLayout.svelte';
  import { TrendingUp, Users, Calendar, MapPin, Music, DollarSign } from 'lucide-svelte';

  let loading = $state(true);

  // Computed metrics
  let totalUsers = $state(0);
  let totalEvents = $state(0);
  let totalRegistrations = $state(0);
  let totalRevenue = $state(0);
  let avgTicketPrice = $state(0);
  let avgEventsPerUser = $state(0);

  let styleBreakdown = $state([]);   // [{ style, count, pct }]
  let cityBreakdown = $state([]);    // [{ city, count, pct }]
  let roleBreakdown = $state([]);    // [{ label, count, pct }]
  let topEvents = $state([]);        // top events by attendees

  $effect(async () => {
    try {
      const [allUsers, allEvents, allRegs] = await Promise.all([
        User.list(),
        Event.list(),
        Registration.list(),
      ]);

      const users = allUsers || [];
      const events = allEvents || [];
      const regs = allRegs || [];

      totalUsers = users.length;
      totalEvents = events.length;
      totalRegistrations = regs.length;

      // Revenue from ticketed events (current_attendees × ticket_price)
      const rev = events.reduce((sum, e) =>
        sum + (e.ticket_price || 0) * (e.current_attendees || 0), 0);
      totalRevenue = rev;
      const ticketed = events.filter(e => e.ticket_price > 0);
      avgTicketPrice = ticketed.length
        ? ticketed.reduce((s, e) => s + e.ticket_price, 0) / ticketed.length
        : 0;

      // Avg events per user
      avgEventsPerUser = users.length > 0
        ? +(regs.length / users.length).toFixed(1)
        : 0;

      // Style breakdown
      const styleCounts = {};
      events.forEach(e =>
        (e.dance_styles || []).forEach(s => {
          styleCounts[s] = (styleCounts[s] || 0) + 1;
        })
      );
      const totalStyleUses = Object.values(styleCounts).reduce((a, b) => a + b, 0) || 1;
      styleBreakdown = Object.entries(styleCounts)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 8)
        .map(([style, count]) => ({
          style: style.toUpperCase(),
          count,
          pct: Math.round((count / totalStyleUses) * 100),
        }));

      // City breakdown
      const cityCounts = {};
      events.forEach(e => {
        if (e.city) cityCounts[e.city] = (cityCounts[e.city] || 0) + 1;
      });
      const totalCityEvents = Object.values(cityCounts).reduce((a, b) => a + b, 0) || 1;
      cityBreakdown = Object.entries(cityCounts)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 6)
        .map(([city, count]) => ({
          city,
          count,
          pct: Math.round((count / totalCityEvents) * 100),
        }));

      // User role breakdown
      const roleCounts = { dancer: 0, creator: 0, both: 0, admin: 0 };
      users.forEach(u => {
        if (u.role === 'admin') roleCounts.admin++;
        else if (u.user_type === 'both') roleCounts.both++;
        else if (u.user_type === 'creator') roleCounts.creator++;
        else roleCounts.dancer++;
      });
      roleBreakdown = [
        { label: 'DANCER',  count: roleCounts.dancer,  bg: 'bg-blue-600' },
        { label: 'CREATOR', count: roleCounts.creator, bg: 'bg-purple-600' },
        { label: 'BOTH',    count: roleCounts.both,    bg: 'bg-pink-600' },
        { label: 'ADMIN',   count: roleCounts.admin,   bg: 'bg-red-600' },
      ].filter(r => r.count > 0);

      // Top events by attendees
      topEvents = [...events]
        .sort((a, b) => (b.current_attendees || 0) - (a.current_attendees || 0))
        .slice(0, 5);

    } catch (e) {
      console.error('Analytics load error', e);
    }
    loading = false;
  });
</script>

<AdminLayout>
  <div class="p-6 bg-gray-50 min-h-full">
    <div class="max-w-7xl mx-auto">

      <div class="mb-8">
        <h1 class="text-4xl font-black transform rotate-1">PLATFORM ANALYTICS</h1>
        <p class="font-bold text-gray-500 mt-1">Live metrics computed from platform data</p>
      </div>

      {#if loading}
        <div class="text-center py-20 font-black text-2xl text-gray-400">COMPUTING STATS...</div>
      {:else}

        <!-- KPI Row -->
        <div class="grid grid-cols-2 lg:grid-cols-3 gap-4 mb-8">
          {#each [
            { label: 'TOTAL USERS',       value: totalUsers,                   Icon: Users,       bg: 'bg-blue-600' },
            { label: 'TOTAL EVENTS',      value: totalEvents,                  Icon: Calendar,    bg: 'bg-pink-600' },
            { label: 'REGISTRATIONS',     value: totalRegistrations,           Icon: TrendingUp,  bg: 'bg-green-600' },
            { label: 'AVG EVENTS / USER', value: avgEventsPerUser,             Icon: Users,       bg: 'bg-purple-600' },
            { label: 'TOTAL REVENUE',     value: `$${totalRevenue.toFixed(0)}`,Icon: DollarSign,  bg: 'bg-yellow-500' },
            { label: 'AVG TICKET PRICE',  value: `$${avgTicketPrice.toFixed(2)}`,Icon: DollarSign,bg: 'bg-orange-500' },
          ] as kpi (kpi.label)}
            <div class="bg-white neo-border neo-shadow p-5">
              <div class={`w-9 h-9 ${kpi.bg} text-white flex items-center justify-center neo-border mb-3`}>
                <kpi.Icon class="w-4 h-4" />
              </div>
              <p class="text-3xl font-black">{kpi.value}</p>
              <p class="font-bold text-gray-500 text-xs mt-1">{kpi.label}</p>
            </div>
          {/each}
        </div>

        <div class="grid lg:grid-cols-2 gap-6 mb-6">

          <!-- Dance Style Breakdown -->
          <div class="bg-white neo-border neo-shadow p-6">
            <h2 class="text-xl font-black mb-4 flex items-center gap-2">
              <Music class="w-5 h-5" /> POPULAR DANCE STYLES
            </h2>
            {#if styleBreakdown.length}
              <div class="space-y-3">
                {#each styleBreakdown as { style, count, pct } (style)}
                  <div>
                    <div class="flex justify-between font-bold text-sm mb-1">
                      <span>{style}</span>
                      <span>{count} events ({pct}%)</span>
                    </div>
                    <div class="w-full bg-gray-200 neo-border h-4">
                      <div
                        class="bg-blue-600 h-full transition-all"
                        style="width: {pct}%"
                      ></div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="font-bold text-gray-400">No event data yet.</p>
            {/if}
          </div>

          <!-- City Breakdown -->
          <div class="bg-white neo-border neo-shadow p-6">
            <h2 class="text-xl font-black mb-4 flex items-center gap-2">
              <MapPin class="w-5 h-5" /> MOST ACTIVE CITIES
            </h2>
            {#if cityBreakdown.length}
              <div class="space-y-3">
                {#each cityBreakdown as { city, count, pct } (city)}
                  <div>
                    <div class="flex justify-between font-bold text-sm mb-1">
                      <span>{city}</span>
                      <span>{count} events ({pct}%)</span>
                    </div>
                    <div class="w-full bg-gray-200 neo-border h-4">
                      <div
                        class="bg-pink-600 h-full transition-all"
                        style="width: {pct}%"
                      ></div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="font-bold text-gray-400">No city data yet.</p>
            {/if}
          </div>
        </div>

        <div class="grid lg:grid-cols-2 gap-6">

          <!-- User Type Breakdown -->
          <div class="bg-white neo-border neo-shadow p-6">
            <h2 class="text-xl font-black mb-4 flex items-center gap-2">
              <Users class="w-5 h-5" /> USER TYPE BREAKDOWN
            </h2>
            <div class="space-y-3">
              {#each roleBreakdown as r (r.label)}
                <div class="flex items-center justify-between p-3 neo-border bg-gray-50">
                  <div class="flex items-center gap-3">
                    <span class={`w-3 h-3 neo-border ${r.bg}`}></span>
                    <span class="font-black">{r.label}</span>
                  </div>
                  <div class="text-right">
                    <span class="font-black text-lg">{r.count}</span>
                    <span class="font-bold text-gray-500 text-sm ml-1">
                      ({totalUsers > 0 ? Math.round((r.count / totalUsers) * 100) : 0}%)
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          </div>

          <!-- Top Events by Attendance -->
          <div class="bg-white neo-border neo-shadow p-6">
            <h2 class="text-xl font-black mb-4 flex items-center gap-2">
              <TrendingUp class="w-5 h-5" /> TOP EVENTS BY ATTENDANCE
            </h2>
            {#if topEvents.length}
              <div class="space-y-3">
                {#each topEvents as ev, i (ev.id)}
                  <div class="flex items-center gap-3 p-3 neo-border bg-gray-50">
                    <span class={`w-7 h-7 flex items-center justify-center font-black text-sm flex-shrink-0 neo-border ${
                      i === 0 ? 'bg-yellow-400' : i === 1 ? 'bg-gray-300' : i === 2 ? 'bg-orange-300' : 'bg-gray-100'
                    }`}>
                      #{i + 1}
                    </span>
                    <div class="flex-1 min-w-0">
                      <p class="font-black text-sm truncate">{ev.title}</p>
                      <p class="text-gray-500 text-xs font-bold">{ev.city}</p>
                    </div>
                    <span class="font-black">{ev.current_attendees || 0}</span>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="font-bold text-gray-400">No event data yet.</p>
            {/if}
          </div>
        </div>

      {/if}
    </div>
  </div>
</AdminLayout>
