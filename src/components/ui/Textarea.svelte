<script>
  let { class: className = '', ...rest } = $props();
</script>

<textarea class="px-4 py-2 neo-border neo-shadow bg-white font-bold resize-none {className}" {...rest} />
