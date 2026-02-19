<script>
  import AdminLayout from '@/components/AdminLayout.svelte';
  import Button from '@/components/ui/Button.svelte';
  import Input from '@/components/ui/Input.svelte';

  // Platform config (frontend-only state; persisted to localStorage for demo)
  const SETTINGS_KEY = 'dc_admin_settings';

  function loadSettings() {
    try {
      return JSON.parse(localStorage.getItem(SETTINGS_KEY) || 'null') ?? defaultSettings();
    } catch {
      return defaultSettings();
    }
  }

  function defaultSettings() {
    return {
      siteName: 'DANCECONNECT',
      supportEmail: 'support@danceconnect.com',
      helpUrl: 'help.danceconnect.com',
      maintenanceMode: false,
      allowSignups: true,
      allowEventCreation: true,
      requireEmailVerification: false,
      limitEventsPerUser: false,
      notifyBugReports: true,
      notifyFeatureRequests: true,
      notifyFlaggedEvents: true,
      weeklyDigest: true,
    };
  }

  let settings = $state(loadSettings());
  let saved = $state(false);
  let resetConfirm = $state(false);

  function save() {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    saved = true;
    setTimeout(() => (saved = false), 2500);
  }

  function reset() {
    settings = defaultSettings();
    resetConfirm = false;
  }
</script>

<AdminLayout>
  <div class="p-6 bg-gray-50 min-h-full">
    <div class="max-w-2xl mx-auto">

      <div class="mb-8">
        <h1 class="text-4xl font-black transform -rotate-1">ADMIN SETTINGS</h1>
        <p class="font-bold text-gray-500 mt-1">Platform configuration & permissions</p>
      </div>

      <!-- Platform Configuration -->
      <section class="bg-white neo-border neo-shadow p-6 mb-6">
        <h2 class="text-xl font-black mb-4 pb-2 border-b-2 border-black">
          PLATFORM CONFIGURATION
        </h2>
        <div class="space-y-4">
          <div>
            <label class="block font-bold mb-1 text-sm">SITE NAME</label>
            <Input bind:value={settings.siteName} class="w-full" />
          </div>
          <div>
            <label class="block font-bold mb-1 text-sm">SUPPORT EMAIL</label>
            <Input type="email" bind:value={settings.supportEmail} class="w-full" />
          </div>
          <div>
            <label class="block font-bold mb-1 text-sm">HELP URL</label>
            <Input bind:value={settings.helpUrl} class="w-full" />
          </div>
          <div class="flex items-center justify-between p-4 neo-border {settings.maintenanceMode ? 'bg-red-50' : 'bg-gray-50'}">
            <div>
              <p class="font-black">MAINTENANCE MODE</p>
              <p class="font-bold text-gray-500 text-sm">Blocks all non-admin access to the platform</p>
            </div>
            <button
              onclick={() => settings.maintenanceMode = !settings.maintenanceMode}
              class={`w-14 h-7 neo-border font-black text-xs transition-colors ${
                settings.maintenanceMode ? 'bg-red-600 text-white' : 'bg-gray-300 text-black'
              }`}
            >
              {settings.maintenanceMode ? 'ON' : 'OFF'}
            </button>
          </div>
        </div>
      </section>

      <!-- Permissions & Access -->
      <section class="bg-white neo-border neo-shadow p-6 mb-6">
        <h2 class="text-xl font-black mb-4 pb-2 border-b-2 border-black">
          PERMISSIONS & ACCESS
        </h2>
        <div class="space-y-3">
          {#each [
            { key: 'allowSignups',            label: 'Allow user signups',              desc: 'New users can register' },
            { key: 'allowEventCreation',      label: 'Allow event creation',            desc: 'Creators can post new events' },
            { key: 'requireEmailVerification',label: 'Require email verification',      desc: 'Users must verify before access' },
            { key: 'limitEventsPerUser',      label: 'Limit events per user (5/month)', desc: 'Cap how many events one creator can post' },
          ] as opt (opt.key)}
            <label class="flex items-center justify-between p-3 neo-border bg-gray-50 cursor-pointer hover:bg-gray-100">
              <div>
                <p class="font-bold text-sm">{opt.label}</p>
                <p class="font-bold text-gray-500 text-xs">{opt.desc}</p>
              </div>
              <input
                type="checkbox"
                bind:checked={settings[opt.key]}
                class="w-5 h-5 neo-border cursor-pointer"
              />
            </label>
          {/each}
        </div>
      </section>

      <!-- Notifications -->
      <section class="bg-white neo-border neo-shadow p-6 mb-8">
        <h2 class="text-xl font-black mb-4 pb-2 border-b-2 border-black">
          NOTIFICATIONS
        </h2>
        <div class="space-y-3">
          {#each [
            { key: 'notifyBugReports',      label: 'Email me on new bug reports' },
            { key: 'notifyFeatureRequests', label: 'Email me on new feature requests' },
            { key: 'notifyFlaggedEvents',   label: 'Email me on flagged events' },
            { key: 'weeklyDigest',          label: 'Weekly stats digest' },
          ] as n (n.key)}
            <label class="flex items-center gap-3 p-3 neo-border bg-gray-50 cursor-pointer hover:bg-gray-100">
              <input
                type="checkbox"
                bind:checked={settings[n.key]}
                class="w-5 h-5 neo-border cursor-pointer"
              />
              <span class="font-bold text-sm">{n.label}</span>
            </label>
          {/each}
        </div>
      </section>

      <!-- Actions -->
      <div class="flex gap-4">
        <Button
          onclick={save}
          class="flex-1 bg-green-600 text-white font-black py-3 text-lg"
        >
          {saved ? '✓ SAVED!' : 'SAVE SETTINGS'}
        </Button>

        {#if !resetConfirm}
          <Button
            onclick={() => resetConfirm = true}
            class="bg-gray-200 text-black font-bold px-6"
          >
            RESET
          </Button>
        {:else}
          <Button
            onclick={reset}
            class="bg-red-600 text-white font-bold px-6"
          >
            CONFIRM RESET
          </Button>
        {/if}
      </div>

      {#if saved}
        <p class="text-center font-bold text-green-600 mt-3">
          Settings saved to your browser storage.
        </p>
      {/if}

    </div>
  </div>
</AdminLayout>
