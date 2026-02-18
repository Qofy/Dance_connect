<script>
  import { url } from '@roxi/routify';
  import { Calendar, Map, User, Plus, Home, Bug, Hammer, Sun } from "lucide-svelte";
  import { createPageUrl } from "@/utils";
  import { UploadFile } from "@/integrations/Core";
  import BugReportModal from "./modals/BugReportModal.svelte";
  import WishRequestModal from "./modals/WishRequestModal.svelte";
  import PersonaSwitcher from "./shared/PersonaSwitcher.svelte";

  let { children } = $props();

  let bugModalState = $state({ isOpen: false, screenshotUrl: null });
  let wishModalState = $state({ isOpen: false, screenshotUrl: null });

  const navigationItems = [
    { title: "Dashboard", url: createPageUrl("Dashboard"), icon: Home },
    { title: "Today", url: createPageUrl("Today"), icon: Sun },
    { title: "Calendar", url: createPageUrl("Calendar"), icon: Calendar },
    { title: "Map", url: createPageUrl("MapView"), icon: Map },
    { title: "Profile", url: createPageUrl("Profile"), icon: User },
    { title: "Create Event", url: createPageUrl("CreateEvent"), icon: Plus, creatorOnly: true },
  ];

  let currentUrl = $derived($url);

  async function handleReportClick(type) {
    const canvas = document.createElement('canvas');
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#f0f0f0';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#000';
    ctx.font = '20px Arial';
    ctx.fillText(`Screenshot for ${type} report at ${new Date().toLocaleTimeString()}`, 50, 100);

    canvas.toBlob(async (blob) => {
      try {
        const file = new File([blob], `${type}-screenshot-${Date.now()}.png`, { type: 'image/png' });
        const { file_url } = await UploadFile({ file });

        if (type === 'bug') {
          bugModalState = { isOpen: true, screenshotUrl: file_url };
        } else {
          wishModalState = { isOpen: true, screenshotUrl: file_url };
        }
      } catch (error) {
        console.error('Screenshot upload failed:', error);
        if (type === 'bug') bugModalState = { isOpen: true, screenshotUrl: null };
        else wishModalState = { isOpen: true, screenshotUrl: null };
      }
    });
  }
</script>

<div class="min-h-screen bg-white">
  <!-- Header -->
  <header class="neo-border border-b-3 bg-white p-4 sticky top-0 z-40">
    <div class="max-w-7xl mx-auto flex items-center justify-between">
      <a href={createPageUrl("Dashboard")} class="flex items-center gap-3">
        <div class="w-12 h-12 bg-gradient-to-br from-blue-600 to-pink-600 neo-border neo-shadow flex items-center justify-center transform -rotate-2">
          <span class="text-white font-black text-xl">DC</span>
        </div>
        <div>
          <h1 class="text-2xl font-black text-black transform rotate-1">
            DANCECONNECT
          </h1>
          <p class="text-sm font-bold text-gray-600 transform -rotate-1">
            BRUTAL DANCE EVENTS
          </p>
        </div>
      </a>

      <div class="flex items-center gap-4">
        <PersonaSwitcher />
        <!-- Desktop Navigation -->
        <nav class="hidden md:flex gap-2">
          {#each navigationItems as item (item.title)}
            <a
              href={item.url}
              class={`px-4 py-2 font-bold neo-border transition-all neo-hover ${
                currentUrl === item.url
                  ? "bg-blue-600 text-white"
                  : "bg-white text-black hover:bg-blue-600 hover:text-white"
              }`}
            >
              <div class="flex items-center gap-2">
                <svelte:component this={item.icon} class="w-5 h-5" />
                {item.title}
              </div>
            </a>
          {/each}
        </nav>

        <!-- Mobile Menu Button -->
        <button class="md:hidden neo-border p-2 bg-white neo-shadow neo-hover">
          <User class="w-6 h-6" />
        </button>
      </div>
    </div>
  </header>

  <!-- Main Content -->
  <main class="flex-1">
    {@render children()}
  </main>

  <!-- Floating Action Buttons -->
  <div class="fixed bottom-24 md:bottom-6 right-6 flex flex-col gap-4 z-50">
    <!-- Bug Report Button -->
    <button
      onclick={() => handleReportClick('bug')}
      class="w-16 h-16 bg-red-500 text-white neo-border neo-shadow neo-hover font-black text-xl transform rotate-3"
      title="Report Bug"
    >
      <Bug class="w-6 h-6 mx-auto" />
    </button>

    <!-- Wish Request Button -->
    <button
      onclick={() => handleReportClick('wish')}
      class="w-16 h-16 bg-yellow-400 text-black neo-border neo-shadow neo-hover font-black text-xl transform -rotate-3"
      title="Make a Wish"
    >
      <Hammer class="w-6 h-6 mx-auto" />
    </button>
  </div>

  <!-- Mobile Navigation -->
  <nav class="md:hidden fixed bottom-0 left-0 right-0 bg-white neo-border border-t-3 p-2 z-50">
    <div class="flex justify-around">
      {#each navigationItems.slice(0, 4) as item (item.title)}
        <a
          href={item.url}
          class={`p-3 neo-border flex flex-col items-center gap-1 text-xs font-bold transition-all ${
            currentUrl === item.url
              ? "bg-blue-600 text-white"
              : "bg-white text-black"
          }`}
        >
          <svelte:component this={item.icon} class="w-5 h-5" />
          {item.title}
        </a>
      {/each}
    </div>
  </nav>

  <!-- Modals -->
  <BugReportModal
    isOpen={bugModalState.isOpen}
    screenshotUrl={bugModalState.screenshotUrl}
    onClose={() => (bugModalState = { isOpen: false, screenshotUrl: null })}
  />
  <WishRequestModal
    isOpen={wishModalState.isOpen}
    screenshotUrl={wishModalState.screenshotUrl}
    onClose={() => (wishModalState = { isOpen: false, screenshotUrl: null })}
  />
</div>
