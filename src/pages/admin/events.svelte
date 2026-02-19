<script>
  import { Event } from '@/entities/Event';
  import { Registration } from '@/entities/Registration';
  import { User } from '@/entities/User';
  import AdminLayout from '@/components/AdminLayout.svelte';
  import Button from '@/components/ui/Button.svelte';
  import Input from '@/components/ui/Input.svelte';
  import { createPageUrl } from '@/utils';
  import { Plus, Edit, Trash2, Users, Search, Filter, X, ChevronDown } from 'lucide-svelte';

  let events = $state([]);
  let allRegistrations = $state([]);
  let allUsers = $state([]);
  let loading = $state(true);

  // Search + filter
  let searchTerm = $state('');
  let filterStatus = $state('all');
  let filterCity = $state('all');

  // Edit modal
  let editingEvent = $state(null);
  let editLoading = $state(false);

  // Registrations modal
  let viewingRegsFor = $state(null);
  let regSearch = $state('');

  let cities = $derived([...new Set(events.map(e => e.city).filter(Boolean))].sort());

  let filteredEvents = $derived(
    events.filter(e => {
      const matchSearch = !searchTerm ||
        e.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
        e.city?.toLowerCase().includes(searchTerm.toLowerCase());
      const matchStatus = filterStatus === 'all' || e.status === filterStatus;
      const matchCity = filterCity === 'all' || e.city === filterCity;
      return matchSearch && matchStatus && matchCity;
    })
  );

  // Registrations for the event being viewed
  let eventRegs = $derived(
    viewingRegsFor
      ? allRegistrations.filter(r => r.event_id === viewingRegsFor.id)
      : []
  );

  let filteredEventRegs = $derived(
    eventRegs.filter(r =>
      !regSearch || r.user_email?.toLowerCase().includes(regSearch.toLowerCase())
    )
  );

  $effect(async () => {
    await loadAll();
  });

  async function loadAll() {
    loading = true;
    try {
      const [evs, regs, users] = await Promise.all([
        Event.list(),
        Registration.list(),
        User.list(),
      ]);
      events = evs || [];
      allRegistrations = regs || [];
      allUsers = users || [];
    } catch (e) {
      console.error('Failed to load admin events', e);
    }
    loading = false;
  }

  function openEdit(event) {
    editingEvent = { ...event };
  }

  function closeEdit() {
    editingEvent = null;
  }

  async function saveEdit() {
    if (!editingEvent) return;
    editLoading = true;
    try {
      await Event.update(editingEvent.id, editingEvent);
      await loadAll();
      editingEvent = null;
    } catch (e) {
      alert('Failed to save event.');
    }
    editLoading = false;
  }

  async function deleteEvent(id) {
    if (!confirm('Permanently delete this event? This cannot be undone.')) return;
    try {
      await Event.delete(id);
      await loadAll();
    } catch (e) {
      alert('Failed to delete event.');
    }
  }

  async function changeStatus(event, status) {
    try {
      await Event.update(event.id, { ...event, status });
      await loadAll();
    } catch (e) {
      alert('Failed to update status.');
    }
  }

  function openRegs(event) {
    viewingRegsFor = event;
    regSearch = '';
  }

  function getUserName(email) {
    const u = allUsers.find(u => u.email === email);
    return u ? u.full_name : email;
  }

  function formatDate(d) {
    return d ? new Date(d).toLocaleDateString() : '—';
  }

  function statusClass(s) {
    if (s === 'public')    return 'bg-green-500 text-white';
    if (s === 'draft')     return 'bg-yellow-400 text-black';
    if (s === 'cancelled') return 'bg-red-600 text-white';
    return 'bg-gray-400 text-white';
  }

  function exportCSV() {
    const rows = filteredEventRegs.map(r => `${getUserName(r.user_email)},${r.user_email},${formatDate(r.created_at)}`);
    const csv = ['Name,Email,Registered At', ...rows].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${viewingRegsFor?.title?.replace(/\s+/g, '_')}_registrations.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<AdminLayout>
  <div class="p-6 bg-gray-50 min-h-full">
    <div class="max-w-7xl mx-auto">

      <!-- Header -->
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6">
        <h1 class="text-4xl font-black">MANAGE EVENTS</h1>
        <a href={createPageUrl('CreateEvent')}>
          <Button class="bg-green-600 text-white font-black neo-border neo-shadow neo-hover">
            <Plus class="w-4 h-4 mr-2" /> NEW EVENT
          </Button>
        </a>
      </div>

      <!-- Filters -->
      <div class="bg-white neo-border neo-shadow p-4 mb-6 grid sm:grid-cols-3 gap-3">
        <div class="relative sm:col-span-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
          <Input bind:value={searchTerm} placeholder="Search events..." class="pl-9 w-full" />
        </div>

        <div class="relative">
          <Filter class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 z-10" />
          <select bind:value={filterStatus} class="w-full pl-9 pr-3 py-2 neo-border font-bold bg-white appearance-none">
            <option value="all">ALL STATUS</option>
            <option value="public">LIVE</option>
            <option value="draft">DRAFT</option>
            <option value="cancelled">CANCELLED</option>
          </select>
        </div>

        <div class="relative">
          <ChevronDown class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 z-10 pointer-events-none" />
          <select bind:value={filterCity} class="w-full px-3 py-2 neo-border font-bold bg-white appearance-none">
            <option value="all">ALL CITIES</option>
            {#each cities as city (city)}
              <option value={city}>{city}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Count -->
      <p class="font-bold text-gray-600 mb-3 text-sm">
        {filteredEvents.length} of {events.length} events
      </p>

      <!-- Events Table -->
      {#if loading}
        <div class="text-center py-16 font-black text-2xl text-gray-400">LOADING EVENTS...</div>
      {:else}
        <div class="bg-white neo-border neo-shadow overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-black text-white font-black">
              <tr>
                <th class="p-3 text-left">EVENT TITLE</th>
                <th class="p-3 text-left">CITY</th>
                <th class="p-3 text-left">DATE</th>
                <th class="p-3 text-left">STATUS</th>
                <th class="p-3 text-left">ATTENDEES</th>
                <th class="p-3 text-left">ACTIONS</th>
              </tr>
            </thead>
            <tbody>
              {#each filteredEvents as event (event.id)}
                <tr class="border-b-2 border-black hover:bg-gray-50">
                  <td class="p-3 font-bold max-w-xs truncate">{event.title}</td>
                  <td class="p-3 font-bold">{event.city}</td>
                  <td class="p-3 font-bold">{formatDate(event.start_date)}</td>
                  <td class="p-3">
                    <span class={`px-2 py-0.5 neo-border font-black text-xs ${statusClass(event.status)}`}>
                      {(event.status || 'public').toUpperCase()}
                    </span>
                  </td>
                  <td class="p-3 font-bold">{event.current_attendees || 0}</td>
                  <td class="p-3">
                    <div class="flex gap-1 flex-wrap">
                      <!-- Edit -->
                      <button
                        onclick={() => openEdit(event)}
                        class="p-1.5 bg-blue-600 text-white neo-border neo-shadow hover:bg-blue-700 transition-colors"
                        title="Edit"
                      >
                        <Edit class="w-3.5 h-3.5" />
                      </button>
                      <!-- Registrations -->
                      <button
                        onclick={() => openRegs(event)}
                        class="p-1.5 bg-purple-600 text-white neo-border neo-shadow hover:bg-purple-700 transition-colors"
                        title="View registrations"
                      >
                        <Users class="w-3.5 h-3.5" />
                      </button>
                      <!-- Status toggle: draft ↔ public -->
                      {#if event.status !== 'public'}
                        <button
                          onclick={() => changeStatus(event, 'public')}
                          class="px-2 py-1 bg-green-600 text-white neo-border font-bold text-xs hover:bg-green-700"
                          title="Publish"
                        >LIVE</button>
                      {:else}
                        <button
                          onclick={() => changeStatus(event, 'draft')}
                          class="px-2 py-1 bg-yellow-400 text-black neo-border font-bold text-xs hover:bg-yellow-500"
                          title="Set to draft"
                        >DRAFT</button>
                      {/if}
                      <!-- Suspend -->
                      <button
                        onclick={() => changeStatus(event, 'cancelled')}
                        class="px-2 py-1 bg-orange-600 text-white neo-border font-bold text-xs hover:bg-orange-700"
                        title="Suspend"
                      >SUSPEND</button>
                      <!-- Delete -->
                      <button
                        onclick={() => deleteEvent(event.id)}
                        class="p-1.5 bg-red-600 text-white neo-border neo-shadow hover:bg-red-700 transition-colors"
                        title="Delete"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </td>
                </tr>
              {:else}
                <tr>
                  <td colspan="6" class="p-8 text-center font-black text-gray-400">
                    NO EVENTS FOUND
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </div>
</AdminLayout>

<!-- ───────── EDIT EVENT MODAL ───────── -->
{#if editingEvent}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60">
    <div class="bg-white neo-border neo-shadow w-full max-w-xl max-h-[90vh] overflow-y-auto">
      <div class="flex justify-between items-center p-4 bg-black text-white sticky top-0">
        <h2 class="font-black text-lg">EDIT EVENT</h2>
        <button onclick={closeEdit} class="hover:text-gray-300">
          <X class="w-5 h-5" />
        </button>
      </div>

      <div class="p-6 space-y-4">
        <div>
          <label class="block font-bold mb-1 text-sm">TITLE</label>
          <Input bind:value={editingEvent.title} class="w-full" />
        </div>

        <div>
          <label class="block font-bold mb-1 text-sm">DESCRIPTION</label>
          <textarea
            bind:value={editingEvent.description}
            rows="3"
            class="w-full p-3 neo-border font-bold bg-white resize-none"
          ></textarea>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block font-bold mb-1 text-sm">CITY</label>
            <Input bind:value={editingEvent.city} class="w-full" />
          </div>
          <div>
            <label class="block font-bold mb-1 text-sm">STATE</label>
            <Input bind:value={editingEvent.state} class="w-full" />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block font-bold mb-1 text-sm">START DATE</label>
            <Input type="date" bind:value={editingEvent.start_date} class="w-full" />
          </div>
          <div>
            <label class="block font-bold mb-1 text-sm">MAX ATTENDEES</label>
            <Input type="number" bind:value={editingEvent.max_attendees} class="w-full" />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block font-bold mb-1 text-sm">TICKET PRICE ($)</label>
            <Input type="number" step="0.01" bind:value={editingEvent.ticket_price} class="w-full" />
          </div>
          <div>
            <label class="block font-bold mb-1 text-sm">STATUS</label>
            <select bind:value={editingEvent.status} class="w-full p-3 neo-border font-bold bg-white">
              <option value="public">LIVE</option>
              <option value="draft">DRAFT</option>
              <option value="cancelled">SUSPENDED</option>
            </select>
          </div>
        </div>

        <div class="flex gap-3 pt-2">
          <Button
            onclick={saveEdit}
            disabled={editLoading}
            class="flex-1 bg-green-600 text-white font-black"
          >
            {editLoading ? 'SAVING...' : 'SAVE'}
          </Button>
          <Button onclick={closeEdit} class="flex-1 bg-gray-600 text-white font-bold">
            CANCEL
          </Button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- ───────── REGISTRATIONS MODAL ───────── -->
{#if viewingRegsFor}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60">
    <div class="bg-white neo-border neo-shadow w-full max-w-2xl max-h-[85vh] overflow-y-auto">
      <div class="flex justify-between items-center p-4 bg-black text-white sticky top-0">
        <div>
          <h2 class="font-black text-lg">EVENT REGISTRATIONS</h2>
          <p class="text-gray-400 text-sm font-bold">{viewingRegsFor.title} — {eventRegs.length} registered</p>
        </div>
        <button onclick={() => viewingRegsFor = null} class="hover:text-gray-300">
          <X class="w-5 h-5" />
        </button>
      </div>

      <div class="p-4">
        <div class="flex gap-3 mb-4">
          <div class="flex-1 relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
            <Input bind:value={regSearch} placeholder="Search by email..." class="pl-9 w-full" />
          </div>
          <Button
            onclick={exportCSV}
            class="bg-blue-600 text-white font-bold neo-border neo-shadow"
          >
            EXPORT CSV
          </Button>
        </div>

        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-black text-white font-black">
              <tr>
                <th class="p-3 text-left">NAME</th>
                <th class="p-3 text-left">EMAIL</th>
                <th class="p-3 text-left">REGISTERED</th>
              </tr>
            </thead>
            <tbody>
              {#each filteredEventRegs as reg (reg.id)}
                <tr class="border-b border-gray-200 hover:bg-gray-50">
                  <td class="p-3 font-bold">{getUserName(reg.user_email)}</td>
                  <td class="p-3 text-gray-600">{reg.user_email}</td>
                  <td class="p-3 text-gray-600">{formatDate(reg.created_at)}</td>
                </tr>
              {:else}
                <tr>
                  <td colspan="3" class="p-6 text-center font-black text-gray-400">
                    NO REGISTRATIONS YET
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
{/if}
