<script>
  import { goto } from '@roxi/routify';
  import { Button } from '@/components/ui/Button.svelte';
  import { Input } from '@/components/ui/Input.svelte';
  import apiClient from '@/integrations/Core';

  let formData = $state({
    email: '',
    password: ''
  });
  let loading = $state(false);

  async function handleSubmit(e) {
    e.preventDefault();
    loading = true;

    try {
      const response = await apiClient.post('/login', formData);
      localStorage.setItem('token', response.token);
      localStorage.setItem('user', JSON.stringify(response.user));
      goto('/dashboard');
    } catch (error) {
      alert('Login failed. Please check your credentials.');
    }
    loading = false;
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100">
  <div class="max-w-md w-full bg-white neo-border neo-shadow p-8">
    <h1 class="text-3xl font-black text-center mb-8">LOGIN</h1>

    <form onsubmit={handleSubmit} class="space-y-6">
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
        <label class="block font-bold mb-2">PASSWORD</label>
        <Input
          type="password"
          required
          bind:value={formData.password}
          class="w-full"
        />
      </div>

      <Button
        type="submit"
        disabled={loading}
        class="w-full bg-blue-600 text-white font-bold"
      >
        {loading ? 'LOGGING IN...' : 'LOGIN'}
      </Button>
    </form>

    <div class="text-center mt-6">
      <p class="font-bold">
        Don't have an account?{' '}
        <a href="/register" class="text-blue-600 underline">
          Register here
        </a>
      </p>
    </div>
  </div>
</div>
