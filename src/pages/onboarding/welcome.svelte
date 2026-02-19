<script>
  import { goto } from '@roxi/routify';
  import Button from '@/components/ui/Button.svelte';
  import apiClient from '@/integrations/Core';
  import { MapPin, User as UserIcon } from 'lucide-svelte';

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  let user = $state(null);
  let styles = $state([]);
  let location = $state({});
  let saving = $state(false);
  let saveError = $state('');

  $effect(() => {
    user = JSON.parse(localStorage.getItem('user') || 'null');
    styles = JSON.parse(localStorage.getItem('onboarding_styles') || '[]');
    location = JSON.parse(localStorage.getItem('onboarding_location') || '{}');
  });

  async function goToDashboard() {
    saving = true;
    saveError = '';

    try {
      if (user?.id) {
        const updates = {};

        if (styles.length > 0) {
          updates.dance_styles = styles;
        }
        if (location.city) {
          updates.city = location.city;
          updates.state = location.state || '';
          updates.zip_code = location.zip_code || '';
          if (location.latitude) updates.latitude = location.latitude;
          if (location.longitude) updates.longitude = location.longitude;
        }

        if (Object.keys(updates).length > 0) {
          const updatedUser = await apiClient.put(`/users/${user.id}`, updates);
          // Merge with stored user (keep token-related fields)
          const enriched = { ...user, ...updatedUser };
          localStorage.setItem('user', JSON.stringify(enriched));
        }
      }
    } catch (err) {
      // Non-fatal: profile can be updated later in edit-profile
      console.warn('Could not save onboarding data to server:', err);
    }

    // Mark onboarding complete so dashboard can show first-time welcome
    localStorage.setItem('is_new_dancer', 'true');

    // Clean up temporary onboarding keys
    localStorage.removeItem('onboarding_styles');
    localStorage.removeItem('onboarding_location');

    saving = false;
    _goto('/dashboard');
  }

  function getUserType() {
    if (!user) return '';
    if (user.user_type === 'both') return 'DANCER & CREATOR';
    if (user.user_type === 'event_creator' || user.role === 'creator') return 'CREATOR';
    return 'DANCER';
  }
</script>

<div class="min-h-screen bg-gray-100 flex items-center justify-center p-4">
  <div class="max-w-md w-full">
    <!-- Progress Bar -->
    <div class="mb-6">
      <div class="flex justify-between font-bold text-sm mb-2">
        <span>STEP 4 OF 4</span>
        <span class="text-green-600">COMPLETE!</span>
      </div>
      <div class="w-full bg-gray-300 neo-border h-3">
        <div class="bg-green-500 h-full transition-all" style="width: 100%"></div>
      </div>
    </div>

    <div class="bg-white neo-border neo-shadow p-8 text-center">
      <!-- Celebration -->
      <div class="text-6xl mb-4">🎉</div>
      <h1 class="text-4xl font-black mb-2 transform -rotate-1">WELCOME TO DANCECONNECT!</h1>
      <p class="font-bold text-gray-600 mb-8">Your profile is ready. Time to dance!</p>

      <!-- Profile Summary -->
      {#if user}
        <div class="bg-yellow-400 neo-border neo-shadow p-6 mb-8 text-left transform rotate-1">
          <h2 class="font-black text-lg mb-4 text-center">YOUR PROFILE</h2>
          <div class="space-y-3">
            <div class="flex items-center gap-3">
              <UserIcon class="w-5 h-5 flex-shrink-0" />
              <div>
                <span class="font-bold text-xs text-gray-600 block">NAME</span>
                <span class="font-black">{user.full_name || user.name || '—'}</span>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <span class="w-5 h-5 flex-shrink-0 font-black text-lg leading-none">★</span>
              <div>
                <span class="font-bold text-xs text-gray-600 block">ROLE</span>
                <span class="font-black">{getUserType()}</span>
              </div>
            </div>

            {#if styles.length > 0}
              <div class="flex items-start gap-3">
                <span class="w-5 h-5 flex-shrink-0 font-black text-lg leading-none">♪</span>
                <div>
                  <span class="font-bold text-xs text-gray-600 block">DANCE STYLES</span>
                  <div class="flex flex-wrap gap-1 mt-1">
                    {#each styles as style (style)}
                      <span class="bg-blue-600 text-white text-xs font-bold px-2 py-0.5 neo-border">
                        {style.toUpperCase()}
                      </span>
                    {/each}
                  </div>
                </div>
              </div>
            {/if}

            {#if location?.city}
              <div class="flex items-center gap-3">
                <MapPin class="w-5 h-5 flex-shrink-0" />
                <div>
                  <span class="font-bold text-xs text-gray-600 block">LOCATION</span>
                  <span class="font-black">
                    {location.city}{location.state ? `, ${location.state}` : ''}
                  </span>
                </div>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      {#if saveError}
        <p class="text-red-600 font-bold text-sm mb-4">{saveError}</p>
      {/if}

      <Button
        type="button"
        onclick={goToDashboard}
        disabled={saving}
        class="w-full bg-green-600 text-white font-black text-lg py-4 neo-border neo-shadow neo-hover"
      >
        {saving ? 'SAVING...' : 'GO TO DASHBOARD →'}
      </Button>
    </div>
  </div>
</div>
