<script>
  let { class: className = '', children, ...rest } = $props();
</script>

<button class="px-4 py-2 font-bold neo-border neo-shadow neo-hover transition-all {className}" {...rest}>
  {@render children?.()}
</button>
