<script>
  let persona = $state('dancer');
</script>

<select
  bind:value={persona}
  class="neo-border neo-shadow bg-white font-bold px-3 py-2"
>
  <option value="dancer">DANCER</option>
  <option value="creator">CREATOR</option>
  <option value="admin">ADMIN</option>
</select>
