<script>
  import { goto } from '@roxi/routify';
  import Button from '@/components/ui/Button.svelte';
  import DanceStylePicker from '@/components/forms/DanceStylePicker.svelte';

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  let selectedStyles = $state([]);

  function toggleStyle(style) {
    if (selectedStyles.includes(style)) {
      selectedStyles = selectedStyles.filter(s => s !== style);
    } else if (selectedStyles.length < 8) {
      selectedStyles = [...selectedStyles, style];
    }
  }

  function proceed() {
    // Save selections to localStorage for the welcome/save step
    localStorage.setItem('onboarding_styles', JSON.stringify(selectedStyles));
    _goto('/onboarding/location');
  }

  function skip() {
    localStorage.setItem('onboarding_styles', JSON.stringify([]));
    _goto('/onboarding/location');
  }
</script>

<div class="min-h-screen bg-gray-100 flex items-center justify-center p-4">
  <div class="max-w-2xl w-full">
    <!-- Progress Bar -->
    <div class="mb-6">
      <div class="flex justify-between font-bold text-sm mb-2">
        <span>STEP 2 OF 4</span>
        <span class="text-gray-500">DANCE STYLES</span>
      </div>
      <div class="w-full bg-gray-300 neo-border h-3">
        <div class="bg-blue-600 h-full transition-all" style="width: 50%"></div>
      </div>
    </div>

    <div class="bg-white neo-border neo-shadow p-8">
      <!-- Header -->
      <div class="text-center mb-8">
        <h1 class="text-3xl font-black transform rotate-1">SELECT YOUR DANCE STYLES</h1>
        <p class="font-bold text-gray-600 mt-2">
          Pick at least 1 <span class="text-gray-400">(optional, max 8)</span>
        </p>
      </div>

      <!-- Style count badge -->
      {#if selectedStyles.length > 0}
        <div class="mb-4 text-center">
          <span class="bg-blue-600 text-white font-black px-4 py-1 neo-border neo-shadow">
            {selectedStyles.length} / 5 SELECTED
          </span>
        </div>
      {/if}

      <!-- Picker -->
      <DanceStylePicker {selectedStyles} onToggleStyle={toggleStyle} />

      <!-- Selected styles preview -->
      {#if selectedStyles.length > 0}
        <div class="mt-6 p-4 bg-blue-50 neo-border">
          <p class="font-bold text-sm mb-2">YOUR STYLES:</p>
          <div class="flex flex-wrap gap-2">
            {#each selectedStyles as style (style)}
              <span class="bg-blue-600 text-white font-bold px-3 py-1 neo-border text-sm">
                {style.toUpperCase()}
              </span>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Navigation -->
      <div class="flex gap-4 mt-8">
        <Button
          type="button"
          onclick={skip}
          class="flex-1 bg-gray-200 text-black font-bold"
        >
          SKIP
        </Button>
        <Button
          type="button"
          onclick={proceed}
          class="flex-2 bg-blue-600 text-white font-black px-8"
        >
          NEXT →
        </Button>
      </div>
    </div>
  </div>
</div>
