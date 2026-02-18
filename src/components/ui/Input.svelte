<script>
  let { class: className = '', ...rest } = $props();
</script>

<input class="px-4 py-2 neo-border neo-shadow bg-white font-bold {className}" {...rest} />
