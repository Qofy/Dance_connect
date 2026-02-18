<script>
  import { goto } from '@roxi/routify';

  // Redirect to login on load
  $effect(() => {
    goto('/login');
  });
</script>

<!-- Index page redirects to login -->
