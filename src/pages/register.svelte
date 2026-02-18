<script>
  import { goto } from '@roxi/routify';
  import Button from '@/components/ui/Button.svelte';
  import Input from '@/components/ui/Input.svelte';
  import apiClient from '@/integrations/Core';

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  let formData = $state({
    email: '',
    password: '',
    name: '',
    role: 'dancer'
  });
  let loading = $state(false);

  async function handleSubmit(e) {
    e.preventDefault();
    loading = true;

    try {
      const response = await apiClient.post('/register', formData);
      localStorage.setItem('token', response.token);
      localStorage.setItem('user', JSON.stringify(response.user));
      _goto('/dashboard');
    } catch (error) {
      alert('Registration failed. Please try again.');
    }
    loading = false;
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100">
  <div class="max-w-md w-full bg-white neo-border neo-shadow p-8">
    <h1 class="text-3xl font-black text-center mb-8">REGISTER</h1>

    <form onsubmit={handleSubmit} class="space-y-6">
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
        <label class="block font-bold mb-2">PASSWORD</label>
        <Input
          type="password"
          required
          bind:value={formData.password}
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

      <Button
        type="submit"
        disabled={loading}
        class="w-full bg-green-600 text-white font-bold"
      >
        {loading ? 'REGISTERING...' : 'REGISTER'}
      </Button>
    </form>

    <div class="text-center mt-6">
      <p class="font-bold">
        Already have an account?{' '}
        <a href="/login" class="text-blue-600 underline">
          Login here
        </a>
      </p>
    </div>
  </div>
</div>
