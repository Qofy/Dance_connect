<script>
  import { User } from '@/entities/User';
  import Select from '@/components/ui/Select.svelte';
  import Layout from '@/components/Layout.svelte';

  let users = $state([]);
  let loading = $state(true);

  async function fetchUsers() {
    loading = true;
    try {
      const allUsers = await User.list();
      users = allUsers || [];
    } catch (e) {
      console.error("Failed to fetch users", e);
      alert("You do not have permission to view this page.");
    }
    loading = false;
  }

  async function updateUserRole(userId, field, value) {
    try {
      await User.update(userId, { [field]: value });
      alert("User updated successfully.");
      fetchUsers();
    } catch (e) {
      alert("Failed to update user.");
    }
  }

  $effect(() => {
    fetchUsers();
  });
</script>

<Layout>
  {#if loading}
    <div class="p-6 font-black text-2xl text-center">LOADING USERS...</div>
  {:else}
    <div class="p-6 bg-white">
      <div class="max-w-7xl mx-auto">
        <h1 class="text-4xl font-black mb-8">ADMIN USER MANAGER</h1>

        <div class="overflow-x-auto neo-border bg-white">
          <table class="w-full">
            <thead class="bg-black text-white font-black">
              <tr>
                <th class="p-3 text-left">Name</th>
                <th class="p-3 text-left">Email</th>
                <th class="p-3 text-left">User Type</th>
                <th class="p-3 text-left">Role</th>
              </tr>
            </thead>
            <tbody>
              {#each users as user (user.id)}
                <tr class="border-b-2 border-black">
                  <td class="p-3 font-bold">{user.full_name}</td>
                  <td class="p-3 font-bold">{user.email}</td>
                  <td class="p-3">
                    <Select
                      bind:value={user.user_type}
                      onchange={(e) => updateUserRole(user.id, 'user_type', e.target.value)}
                      class="neo-border font-bold w-48"
                    >
                      <option value="dancer">Dancer</option>
                      <option value="event_creator">Creator</option>
                      <option value="both">Both</option>
                    </Select>
                  </td>
                  <td class="p-3">
                    <Select
                      bind:value={user.role}
                      onchange={(e) => updateUserRole(user.id, 'role', e.target.value)}
                      class="neo-border font-bold w-48"
                    >
                      <option value="user">User</option>
                      <option value="admin">Admin</option>
                    </Select>
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
