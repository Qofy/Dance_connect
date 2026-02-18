<script>
  import { goto } from '@roxi/routify';
  import Button from '@/components/ui/Button.svelte';
  import Input from '@/components/ui/Input.svelte';
  import apiClient from '@/integrations/Core';
  import Layout from '@/components/Layout.svelte';

  let formData = $state({
    name: '',
    email: '',
    role: 'dancer'
  });
  let loading = $state(false);

  $effect(() => {
    const user = JSON.parse(localStorage.getItem('user') || '{}');
    formData = {
      name: user.name || '',
      email: user.email || '',
      role: user.role || 'dancer'
    };
  });

  async function handleSubmit(e) {
    e.preventDefault();
    loading = true;

    try {
      const user = JSON.parse(localStorage.getItem('user') || '{}');
      const response = await apiClient.put(`/users/${user.id}`, formData);
      localStorage.setItem('user', JSON.stringify(response));
      alert('Profile updated successfully!');
      goto('/profile');
    } catch (error) {
      alert('Failed to update profile. Please try again.');
    }
    loading = false;
  }
</script>

<Layout>
  <div class="p-6 bg-white">
    <div class="max-w-md mx-auto">
      <h1 class="text-3xl font-black text-center mb-8">EDIT PROFILE</h1>

      <form onsubmit={handleSubmit} class="space-y-6 neo-border neo-shadow p-8 bg-gray-50">
        <div>
          <label class="block font-bold mb-2">NAME</label>
          <Input
            required
            bind:value={formData.name}
            class="w-full"
          />
        </div>

        <div>
          <label class="block font-bold mb-2">EMAIL</label>
          <Input
            type="email"
            required
            bind:value={formData.email}
            class="w-full"
          />
        </div>

        <div>
          <label class="block font-bold mb-2">ROLE</label>
          <select
            bind:value={formData.role}
            class="w-full p-3 border rounded font-bold"
          >
            <option value="dancer">DANCER</option>
            <option value="creator">CREATOR</option>
            <option value="admin">ADMIN</option>
          </select>
        </div>

        <div class="flex gap-4">
          <Button
            type="submit"
            disabled={loading}
            class="flex-1 bg-green-600 text-white font-bold"
          >
            {loading ? 'UPDATING...' : 'UPDATE PROFILE'}
          </Button>

          <Button
            type="button"
            onclick={() => goto('/profile')}
            class="flex-1 bg-gray-600 text-white font-bold"
          >
            CANCEL
          </Button>
        </div>
      </form>
    </div>
  </div>
</Layout>
