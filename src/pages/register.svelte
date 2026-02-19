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
  // user_type tracks DANCER / CREATOR / BOTH for UX routing
  let userType = $state('dancer');
  let loading = $state(false);
  let errorMsg = $state('');
  let isDuplicateEmail = $state(false);

  const roleOptions = [
    { value: 'dancer',  label: 'DANCER',  description: 'I want to join events' },
    { value: 'creator', label: 'CREATOR', description: 'I organize events' },
    { value: 'both',    label: 'BOTH',    description: 'I dance and organize' },
    { value: 'admin',   label: 'ADMIN',   description: 'Platform administrator' }
  ];

  function selectRole(value) {
    userType = value;
    formData.role = value;
  }

  async function handleSubmit(e) {
    e.preventDefault();
    loading = true;
    errorMsg = '';
    isDuplicateEmail = false;

    try {
      const response = await apiClient.post('/register', formData);
      const user = response.user;

      // Enrich user_type locally so profile.svelte checks work
      if (userType === 'both') {
        user.user_type = 'both';
      }

      localStorage.setItem('token', response.token);
      localStorage.setItem('user', JSON.stringify(user));

      // Route based on role
      if (userType === 'admin') {
        _goto('/admin');
      } else if (userType === 'dancer' || userType === 'both') {
        _goto('/onboarding/dance-styles');
      } else {
        _goto('/dashboard');
      }
    } catch (error) {
      if (error.message?.includes('409') || error.message?.toLowerCase().includes('exists')) {
        errorMsg = 'Email already registered.';
        isDuplicateEmail = true;
      } else if (error.message?.toLowerCase().includes('password')) {
        errorMsg = 'Must be 8+ chars, include a number & symbol.';
      } else {
        errorMsg = 'Registration failed. Please try again.';
      }
    }
    loading = false;
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100 p-4">
  <div class="max-w-md w-full bg-white neo-border neo-shadow p-8">
    <!-- Header -->
    <div class="text-center mb-8">
      <h1 class="text-4xl font-black transform -rotate-1">CREATE ACCOUNT</h1>
      <p class="font-bold text-gray-600 mt-2">JOIN THE DANCE REVOLUTION</p>
    </div>

    <form onsubmit={handleSubmit} class="space-y-6">
      <!-- Name -->
      <div>
        <label class="block font-bold mb-2">NAME</label>
        <Input required bind:value={formData.name} class="w-full" placeholder="Your full name" />
      </div>

      <!-- Email -->
      <div>
        <label class="block font-bold mb-2">EMAIL</label>
        <Input type="email" required bind:value={formData.email} class="w-full" placeholder="you@example.com" />
      </div>

      <!-- Password -->
      <div>
        <label class="block font-bold mb-2">PASSWORD</label>
        <Input type="password" required minlength="8" bind:value={formData.password} class="w-full" placeholder="Min 8 chars" />
      </div>

      <!-- Role Selection (radio buttons) -->
      <div>
        <label class="block font-bold mb-3">ROLE <span class="text-red-600">*</span></label>
        <div class="space-y-3">
          {#each roleOptions as opt (opt.value)}
            <button
              type="button"
              onclick={() => selectRole(opt.value)}
              class={`w-full p-4 neo-border text-left transition-all font-bold flex items-center gap-4 ${
                userType === opt.value
                  ? 'bg-blue-600 text-white neo-shadow'
                  : 'bg-white text-black hover:bg-gray-100'
              }`}
            >
              <!-- Radio dot -->
              <span class={`w-5 h-5 rounded-full border-2 flex-shrink-0 flex items-center justify-center ${
                userType === opt.value ? 'border-white' : 'border-black'
              }`}>
                {#if userType === opt.value}
                  <span class="w-2.5 h-2.5 rounded-full bg-white"></span>
                {/if}
              </span>
              <span>
                <span class="block text-base">{opt.label}</span>
                <span class={`block text-sm font-normal ${userType === opt.value ? 'text-blue-100' : 'text-gray-500'}`}>
                  {opt.description}
                </span>
              </span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Error Message -->
      {#if errorMsg}
        <div class="p-3 bg-red-100 neo-border border-red-600 text-red-700 font-bold text-sm">
          {errorMsg}
          {#if isDuplicateEmail}
            <a href="/login" class="underline ml-1">Login here →</a>
          {/if}
        </div>
      {/if}

      <Button
        type="submit"
        disabled={loading}
        class="w-full bg-green-600 text-white font-black text-lg py-4"
      >
        {loading ? 'CREATING ACCOUNT...' : 'NEXT →'}
      </Button>
    </form>

    <div class="text-center mt-6">
      <p class="font-bold">
        Already have an account?
        <a href="/login" class="text-blue-600 underline ml-1">Login here</a>
      </p>
    </div>
  </div>
</div>
