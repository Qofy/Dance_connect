<script>
  let { isOpen, screenshotUrl, onClose } = $props();
  let bugDescription = $state('');
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white neo-border neo-shadow p-6 max-w-md w-full mx-4">
      <h2 class="text-2xl font-black mb-4">REPORT BUG</h2>
      <textarea
        bind:value={bugDescription}
        placeholder="Describe the bug..."
        class="w-full h-32 p-3 neo-border resize-none"
      />
      <div class="flex gap-3 mt-4">
        <button
          onclick={onClose}
          class="flex-1 bg-gray-500 text-white neo-border neo-shadow neo-hover font-bold py-2"
        >
          CANCEL
        </button>
        <button
          onclick={onClose}
          class="flex-1 bg-red-500 text-white neo-border neo-shadow neo-hover font-bold py-2"
        >
          SUBMIT
        </button>
      </div>
    </div>
  </div>
{/if}
