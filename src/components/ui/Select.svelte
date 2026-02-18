<script>
  let { class: className = '', children, ...rest } = $props();
</script>

<select class="px-4 py-2 neo-border neo-shadow bg-white font-bold {className}" {...rest}>
  {@render children?.()}
</select>
