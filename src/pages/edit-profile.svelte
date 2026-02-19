<script>
  import { goto } from '@roxi/routify';
  import Button from '@/components/ui/Button.svelte';
  import Input from '@/components/ui/Input.svelte';
  import DanceStylePicker from '@/components/forms/DanceStylePicker.svelte';
  import apiClient from '@/integrations/Core';
  import Layout from '@/components/Layout.svelte';

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  let formData = $state({
    name: '',
    email: '',
    role: 'dancer',
    city: '',
    state: '',
    zip_code: '',
    dance_styles: []
  });
  let loading = $state(false);

  $effect(() => {
    const user = JSON.parse(localStorage.getItem('user') || '{}');
    formData = {
      name: user.full_name || user.name || '',
      email: user.email || '',
      role: user.role || 'dancer',
      city: user.city || '',
      state: user.state || '',
      zip_code: user.zip_code || '',
      dance_styles: Array.isArray(user.dance_styles) ? user.dance_styles : []
    };
  });

  function toggleStyle(style) {
    if (formData.dance_styles.includes(style)) {
      formData.dance_styles = formData.dance_styles.filter(s => s !== style);
    } else if (formData.dance_styles.length < 5) {
      formData.dance_styles = [...formData.dance_styles, style];
    }
  }

  async function handleSubmit(e) {
    e.preventDefault();
    loading = true;

    try {
      const user = JSON.parse(localStorage.getItem('user') || '{}');
      const response = await apiClient.put(`/users/${user.id}`, formData);
      localStorage.setItem('user', JSON.stringify({ ...user, ...response }));
      alert('Profile updated successfully!');
      _goto('/profile');
    } catch (error) {
      alert('Failed to update profile. Please try again.');
    }
    loading = false;
  }
</script>

<Layout>
  <div class="p-6 bg-white">
    <div class="max-w-2xl mx-auto">
      <h1 class="text-3xl font-black text-center mb-8">EDIT PROFILE</h1>

      <form onsubmit={handleSubmit} class="space-y-8">

        <!-- Basic Info -->
        <section class="neo-border neo-shadow p-6 bg-gray-50">
          <h2 class="text-xl font-black mb-4">BASIC INFO</h2>
          <div class="space-y-4">
            <div>
              <label class="block font-bold mb-2">NAME</label>
              <Input required bind:value={formData.name} class="w-full" />
            </div>

            <div>
              <label class="block font-bold mb-2">EMAIL</label>
              <Input type="email" required bind:value={formData.email} class="w-full" />
            </div>

            <div>
              <label class="block font-bold mb-2">ROLE</label>
              <select
                bind:value={formData.role}
                class="w-full p-3 neo-border font-bold bg-white"
              >
                <option value="dancer">DANCER</option>
                <option value="creator">CREATOR</option>
                <option value="admin">ADMIN</option>
              </select>
            </div>
          </div>
        </section>

        <!-- Dance Styles (shown for dancers) -->
        {#if formData.role === 'dancer' || formData.role === 'creator'}
          <section class="neo-border neo-shadow p-6 bg-gray-50">
            <h2 class="text-xl font-black mb-1">DANCE STYLES</h2>
            <p class="font-bold text-gray-500 text-sm mb-4">Select up to 5 styles</p>

            {#if formData.dance_styles.length > 0}
              <div class="mb-4 flex flex-wrap gap-2">
                {#each formData.dance_styles as style (style)}
                  <span class="bg-blue-600 text-white font-bold px-3 py-1 neo-border text-sm">
                    {style.toUpperCase()}
                  </span>
                {/each}
                <span class="font-bold text-gray-500 text-sm self-center">
                  ({formData.dance_styles.length}/5)
                </span>
              </div>
            {/if}

            <DanceStylePicker
              selectedStyles={formData.dance_styles}
              onToggleStyle={toggleStyle}
            />
          </section>
        {/if}

        <!-- Location -->
        <section class="neo-border neo-shadow p-6 bg-gray-50">
          <h2 class="text-xl font-black mb-1">LOCATION</h2>
          <p class="font-bold text-gray-500 text-sm mb-4">Used to show nearby events</p>
          <div class="space-y-4">
            <div>
              <label class="block font-bold mb-2">CITY</label>
              <Input bind:value={formData.city} class="w-full" placeholder="San Francisco" />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block font-bold mb-2">STATE</label>
                <Input bind:value={formData.state} class="w-full" placeholder="CA" />
              </div>
              <div>
                <label class="block font-bold mb-2">ZIP CODE</label>
                <Input bind:value={formData.zip_code} class="w-full" placeholder="94105" />
              </div>
            </div>
          </div>
        </section>

        <!-- Actions -->
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
            onclick={() => _goto('/profile')}
            class="flex-1 bg-gray-600 text-white font-bold"
          >
            CANCEL
          </Button>
        </div>
      </form>
    </div>
  </div>
</Layout>
