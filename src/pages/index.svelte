<script>
  import { goto } from '@roxi/routify';

  // Redirect to login on load
  let _goto;
  goto.subscribe((fn) => (_goto = fn));
  _goto('/login');
</script>

<!-- Index page redirects to login -->
