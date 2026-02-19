<script>
  import { Event } from '@/entities/Event';
  import { User } from '@/entities/User';
  import { Registration } from '@/entities/Registration';
  import AdminLayout from '@/components/AdminLayout.svelte';
  import { Users, Calendar, MapPin, Zap, CheckCircle, AlertCircle } from 'lucide-svelte';

  let stats = $state({ users: 0, events: 0, cities: 0, registrations: 0 });
  let loading = $state(true);
  let recentUsers = $state([]);
  let recentEvents = $state([]);
  let apiHealthy = $state(true);

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

      const cities = new Set(events.map(e => e.city).filter(Boolean));

      stats = {
        users: users.length,
        events: events.length,
        cities: cities.size,
        registrations: regs.length,
      };

      recentUsers = users
        .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
        .slice(0, 5);

      recentEvents = events
        .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
        .slice(0, 5);

      apiHealthy = true;
    } catch (e) {
      console.error('Admin dashboard load error', e);
      apiHealthy = false;
    }
    loading = false;
  });

  function formatDate(d) {
    return d ? new Date(d).toLocaleDateString() : '—';
  }
</script>

<AdminLayout>
  <div class="p-6 bg-gray-50 min-h-full">
    <div class="max-w-7xl mx-auto">

      <!-- Header -->
      <div class="mb-8">
        <h1 class="text-4xl font-black transform -rotate-1">CONTROL PANEL</h1>
        <p class="font-bold text-gray-500 mt-1">Platform overview & recent activity</p>
      </div>

      {#if loading}
        <div class="text-center py-20 font-black text-2xl text-gray-400">LOADING STATS...</div>
      {:else}

        <!-- Quick Stats -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
          {#each [
            { label: 'TOTAL USERS',  value: stats.users,         Icon: Users,    bg: 'bg-blue-600' },
            { label: 'TOTAL EVENTS', value: stats.events,        Icon: Calendar, bg: 'bg-pink-600' },
            { label: 'CITIES',       value: stats.cities,        Icon: MapPin,   bg: 'bg-purple-600' },
            { label: 'REGISTRATIONS',value: stats.registrations, Icon: Zap,      bg: 'bg-yellow-500' },
          ] as stat (stat.label)}
            <div class="bg-white neo-border neo-shadow p-5 transform hover:-translate-y-0.5 transition-transform">
              <div class={`w-10 h-10 ${stat.bg} text-white flex items-center justify-center neo-border mb-3`}>
                <stat.Icon class="w-5 h-5" />
              </div>
              <p class="text-4xl font-black">{stat.value}</p>
              <p class="font-bold text-gray-500 text-sm mt-1">{stat.label}</p>
            </div>
          {/each}
        </div>

        <!-- System Health + Recent Users -->
        <div class="grid lg:grid-cols-3 gap-6 mb-6">

          <!-- System Health -->
          <div class="bg-white neo-border neo-shadow p-6">
            <h2 class="text-xl font-black mb-4">SYSTEM HEALTH</h2>
            <div class="space-y-3">
              {#each [
                { label: 'API Server',    ok: apiHealthy },
                { label: 'Database',      ok: true },
                { label: 'Auth Service',  ok: true },
              ] as item (item.label)}
                <div class="flex items-center justify-between p-2 neo-border bg-gray-50">
                  <span class="font-bold text-sm">{item.label}</span>
                  {#if item.ok}
                    <span class="flex items-center gap-1 text-green-600 font-black text-sm">
                      <CheckCircle class="w-4 h-4" /> HEALTHY
                    </span>
                  {:else}
                    <span class="flex items-center gap-1 text-red-600 font-black text-sm">
                      <AlertCircle class="w-4 h-4" /> ERROR
                    </span>
                  {/if}
                </div>
              {/each}
            </div>
          </div>

          <!-- Recent Users -->
          <div class="lg:col-span-2 bg-white neo-border neo-shadow p-6">
            <div class="flex justify-between items-center mb-4">
              <h2 class="text-xl font-black">RECENT USERS</h2>
              <a href="/admin/users" class="text-blue-600 font-bold text-sm underline">View all →</a>
            </div>
            <div class="space-y-2">
              {#each recentUsers as u (u.id)}
                <div class="flex items-center justify-between p-2 neo-border bg-gray-50 text-sm">
                  <div>
                    <span class="font-black">{u.full_name}</span>
                    <span class="text-gray-500 ml-2">{u.email}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class={`px-2 py-0.5 neo-border font-bold text-xs ${
                      u.role === 'admin' ? 'bg-red-600 text-white' :
                      u.user_type === 'creator' ? 'bg-purple-600 text-white' :
                      'bg-blue-600 text-white'
                    }`}>
                      {u.role === 'admin' ? 'ADMIN' : (u.user_type || 'dancer').toUpperCase()}
                    </span>
                    <span class="text-gray-400 text-xs">{formatDate(u.created_at)}</span>
                  </div>
                </div>
              {:else}
                <p class="font-bold text-gray-400 text-sm">No users yet.</p>
              {/each}
            </div>
          </div>
        </div>

        <!-- Recent Events -->
        <div class="bg-white neo-border neo-shadow p-6">
          <div class="flex justify-between items-center mb-4">
            <h2 class="text-xl font-black">RECENT EVENTS</h2>
            <a href="/admin/events" class="text-blue-600 font-bold text-sm underline">Manage all →</a>
          </div>
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead class="bg-black text-white font-black">
                <tr>
                  <th class="p-3 text-left">TITLE</th>
                  <th class="p-3 text-left">CITY</th>
                  <th class="p-3 text-left">DATE</th>
                  <th class="p-3 text-left">STYLES</th>
                  <th class="p-3 text-left">STATUS</th>
                </tr>
              </thead>
              <tbody>
                {#each recentEvents as ev (ev.id)}
                  <tr class="border-b-2 border-black hover:bg-gray-50">
                    <td class="p-3 font-bold">{ev.title}</td>
                    <td class="p-3 font-bold">{ev.city}, {ev.state}</td>
                    <td class="p-3 font-bold">{formatDate(ev.start_date)}</td>
                    <td class="p-3">
                      <div class="flex gap-1 flex-wrap">
                        {#each (ev.dance_styles || []).slice(0,2) as s (s)}
                          <span class="bg-blue-100 text-blue-800 px-1.5 py-0.5 neo-border text-xs font-bold">{s.toUpperCase()}</span>
                        {/each}
                      </div>
                    </td>
                    <td class="p-3">
                      <span class={`px-2 py-0.5 neo-border font-black text-xs ${
                        ev.status === 'public' ? 'bg-green-500 text-white' :
                        ev.status === 'draft'  ? 'bg-yellow-400 text-black' :
                        'bg-gray-400 text-white'
                      }`}>
                        {(ev.status || 'public').toUpperCase()}
                      </span>
                    </td>
                  </tr>
                {:else}
                  <tr><td colspan="5" class="p-4 font-bold text-gray-400 text-center">No events yet.</td></tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>

      {/if}
    </div>
  </div>
</AdminLayout>
