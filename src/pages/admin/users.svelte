<script>
  import { User } from '@/entities/User';
  import { Registration } from '@/entities/Registration';
  import AdminLayout from '@/components/AdminLayout.svelte';
  import Button from '@/components/ui/Button.svelte';
  import Input from '@/components/ui/Input.svelte';
  import apiClient from '@/integrations/Core';
  import { Search, X, UserCircle, Filter } from 'lucide-svelte';

  let users = $state([]);
  let allRegistrations = $state([]);
  let loading = $state(true);

  // Search + filter
  let searchTerm = $state('');
  let filterRole = $state('all');
  let filterType = $state('all');

  // User detail modal
  let selectedUser = $state(null);
  let editData = $state({});
  let saving = $state(false);
  let saveMsg = $state('');

  let filteredUsers = $derived(
    users.filter(u => {
      const q = searchTerm.toLowerCase();
      const matchSearch = !q ||
        u.full_name?.toLowerCase().includes(q) ||
        u.email?.toLowerCase().includes(q);
      const matchRole = filterRole === 'all' || u.role === filterRole;
      const matchType = filterType === 'all' || u.user_type === filterType;
      return matchSearch && matchRole && matchType;
    })
  );

  // How many events has this user registered for?
  function regCount(userId) {
    const u = users.find(u => u.id === userId);
    if (!u) return 0;
    return allRegistrations.filter(r => r.user_email === u.email).length;
  }

  $effect(async () => {
    await loadAll();
  });

  async function loadAll() {
    loading = true;
    try {
      const [us, regs] = await Promise.all([User.list(), Registration.list()]);
      users = us || [];
      allRegistrations = regs || [];
    } catch (e) {
      console.error('Failed to load users', e);
    }
    loading = false;
  }

  function openUser(u) {
    selectedUser = u;
    editData = {
      role: u.role,
      user_type: u.user_type || 'dancer',
      status: u.status || 'active',
    };
    saveMsg = '';
  }

  function closeUser() {
    selectedUser = null;
    saveMsg = '';
  }

  async function saveUser() {
    if (!selectedUser) return;
    saving = true;
    saveMsg = '';
    try {
      await apiClient.put(`/users/${selectedUser.id}`, editData);
      await loadAll();
      // Refresh selectedUser with updated data
      const updated = users.find(u => u.id === selectedUser.id);
      if (updated) selectedUser = updated;
      saveMsg = 'Saved!';
    } catch (e) {
      saveMsg = 'Save failed.';
    }
    saving = false;
  }

  async function deleteUser(id) {
    if (!confirm('Permanently delete this user? This cannot be undone.')) return;
    try {
      await apiClient.delete(`/users/${id}`);
      closeUser();
      await loadAll();
    } catch (e) {
      alert('Delete failed (backend may not support delete yet).');
    }
  }

  function formatDate(d) {
    return d ? new Date(d).toLocaleDateString() : '—';
  }

  function roleChip(role, type) {
    if (role === 'admin')   return { label: 'ADMIN',   bg: 'bg-red-600 text-white' };
    if (type === 'creator') return { label: 'CREATOR', bg: 'bg-purple-600 text-white' };
    if (type === 'both')    return { label: 'BOTH',    bg: 'bg-pink-600 text-white' };
    return                         { label: 'DANCER',  bg: 'bg-blue-600 text-white' };
  }

  function typeInitial(type, role) {
    if (role === 'admin')   return '—';
    if (type === 'creator') return 'C';
    if (type === 'both')    return 'B';
    return 'D';
  }
</script>

<AdminLayout>
  <div class="p-6 bg-gray-50 min-h-full">
    <div class="max-w-7xl mx-auto">

      <!-- Header -->
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2 mb-6">
        <div>
          <h1 class="text-4xl font-black">MANAGE USERS</h1>
          <p class="font-bold text-gray-500 text-sm mt-1">{users.length} total registered</p>
        </div>
      </div>

      <!-- Filters -->
      <div class="bg-white neo-border neo-shadow p-4 mb-6 grid sm:grid-cols-3 gap-3">
        <div class="relative sm:col-span-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
          <Input bind:value={searchTerm} placeholder="Name, email, or ID..." class="pl-9 w-full" />
        </div>

        <div class="relative">
          <Filter class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 z-10" />
          <select bind:value={filterRole} class="w-full pl-9 pr-3 py-2 neo-border font-bold bg-white appearance-none">
            <option value="all">ALL ROLES</option>
            <option value="admin">ADMIN</option>
            <option value="creator">CREATOR</option>
            <option value="dancer">DANCER</option>
          </select>
        </div>

        <select bind:value={filterType} class="w-full px-3 py-2 neo-border font-bold bg-white">
          <option value="all">ALL USER TYPES</option>
          <option value="dancer">DANCER (D)</option>
          <option value="creator">CREATOR (C)</option>
          <option value="both">BOTH (B)</option>
        </select>
      </div>

      <!-- Count -->
      <p class="font-bold text-gray-500 text-sm mb-3">
        Showing {filteredUsers.length} of {users.length}
      </p>

      <!-- Legend -->
      <div class="flex gap-3 mb-4 flex-wrap text-xs font-bold text-gray-500">
        <span>D = Dancer</span>
        <span>C = Creator</span>
        <span>B = Both</span>
        <span>— = Admin</span>
      </div>

      <!-- Table -->
      {#if loading}
        <div class="text-center py-16 font-black text-2xl text-gray-400">LOADING USERS...</div>
      {:else}
        <div class="bg-white neo-border neo-shadow overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-black text-white font-black">
              <tr>
                <th class="p-3 text-left">NAME</th>
                <th class="p-3 text-left">EMAIL</th>
                <th class="p-3 text-left">ROLE</th>
                <th class="p-3 text-left">TYPE</th>
                <th class="p-3 text-left">JOINED</th>
                <th class="p-3 text-left">REGS</th>
                <th class="p-3 text-left">ACTION</th>
              </tr>
            </thead>
            <tbody>
              {#each filteredUsers as user (user.id)}
                {@const chip = roleChip(user.role, user.user_type)}
                <tr class="border-b-2 border-black hover:bg-gray-50 cursor-pointer" onclick={() => openUser(user)}>
                  <td class="p-3 font-black">{user.full_name}</td>
                  <td class="p-3 text-gray-600">{user.email}</td>
                  <td class="p-3">
                    <span class={`px-2 py-0.5 neo-border font-black text-xs ${chip.bg}`}>
                      {chip.label}
                    </span>
                  </td>
                  <td class="p-3 font-black text-center">{typeInitial(user.user_type, user.role)}</td>
                  <td class="p-3 text-gray-600">{formatDate(user.created_at)}</td>
                  <td class="p-3 font-bold">{regCount(user.id)}</td>
                  <td class="p-3">
                    <button
                      onclick={(e) => { e.stopPropagation(); openUser(user); }}
                      class="px-3 py-1 bg-blue-600 text-white neo-border font-bold text-xs hover:bg-blue-700"
                    >
                      EDIT
                    </button>
                  </td>
                </tr>
              {:else}
                <tr>
                  <td colspan="7" class="p-8 text-center font-black text-gray-400">NO USERS FOUND</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </div>
</AdminLayout>

<!-- ───────── USER DETAIL MODAL ───────── -->
{#if selectedUser}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60">
    <div class="bg-white neo-border neo-shadow w-full max-w-lg max-h-[90vh] overflow-y-auto">

      <!-- Modal Header -->
      <div class="flex justify-between items-center p-4 bg-black text-white sticky top-0">
        <div class="flex items-center gap-3">
          <UserCircle class="w-6 h-6" />
          <div>
            <h2 class="font-black">{selectedUser.full_name}</h2>
            <p class="text-gray-400 text-xs">{selectedUser.email}</p>
          </div>
        </div>
        <button onclick={closeUser} class="hover:text-gray-300">
          <X class="w-5 h-5" />
        </button>
      </div>

      <div class="p-6 space-y-5">

        <!-- Read-only info -->
        <div class="grid grid-cols-2 gap-3">
          <div class="p-3 neo-border bg-gray-50">
            <p class="text-xs font-bold text-gray-500">JOINED</p>
            <p class="font-black">{formatDate(selectedUser.created_at)}</p>
          </div>
          <div class="p-3 neo-border bg-gray-50">
            <p class="text-xs font-bold text-gray-500">EVENT REGISTRATIONS</p>
            <p class="font-black">{regCount(selectedUser.id)}</p>
          </div>
          {#if selectedUser.city}
            <div class="p-3 neo-border bg-gray-50 col-span-2">
              <p class="text-xs font-bold text-gray-500">LOCATION</p>
              <p class="font-black">{selectedUser.city}, {selectedUser.state}</p>
            </div>
          {/if}
          {#if selectedUser.dance_styles?.length}
            <div class="p-3 neo-border bg-gray-50 col-span-2">
              <p class="text-xs font-bold text-gray-500 mb-1">DANCE STYLES</p>
              <div class="flex flex-wrap gap-1">
                {#each selectedUser.dance_styles as s (s)}
                  <span class="bg-blue-100 text-blue-800 px-2 py-0.5 neo-border text-xs font-bold">
                    {s.toUpperCase()}
                  </span>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <!-- Editable fields -->
        <div class="space-y-3 pt-2 border-t-2 border-black">
          <h3 class="font-black text-sm">PERMISSIONS</h3>

          <div>
            <label class="block font-bold text-sm mb-1">ROLE</label>
            <select bind:value={editData.role} class="w-full p-3 neo-border font-bold bg-white">
              <option value="dancer">USER (Dancer)</option>
              <option value="creator">USER (Creator)</option>
              <option value="admin">ADMIN</option>
            </select>
          </div>

          <div>
            <label class="block font-bold text-sm mb-1">USER TYPE</label>
            <select bind:value={editData.user_type} class="w-full p-3 neo-border font-bold bg-white">
              <option value="dancer">DANCER — attends events</option>
              <option value="creator">CREATOR — organizes events</option>
              <option value="both">BOTH — dances & organizes</option>
            </select>
          </div>
        </div>

        <!-- Save message -->
        {#if saveMsg}
          <p class={`font-bold text-sm text-center ${saveMsg === 'Saved!' ? 'text-green-600' : 'text-red-600'}`}>
            {saveMsg}
          </p>
        {/if}

        <!-- Actions -->
        <div class="flex gap-3 pt-2">
          <Button
            onclick={saveUser}
            disabled={saving}
            class="flex-1 bg-green-600 text-white font-black"
          >
            {saving ? 'SAVING...' : 'SAVE'}
          </Button>
          <Button onclick={closeUser} class="flex-1 bg-gray-600 text-white font-bold">
            CANCEL
          </Button>
        </div>

        <div class="pt-2 border-t-2 border-red-200">
          <Button
            onclick={() => deleteUser(selectedUser.id)}
            class="w-full bg-red-600 text-white font-bold border-red-700"
          >
            DELETE USER
          </Button>
          <p class="text-center text-xs text-gray-400 font-bold mt-2">
            This permanently removes the account.
          </p>
        </div>

      </div>
    </div>
  </div>
{/if}
