<script>
  const DANCE_STYLES = [
    // Latin
    "Salsa", "Bachata", "Merengue", "Cha-Cha", "Rumba", "Samba", "Paso Doble", "Cumbia",
    // Swing
    "Lindy Hop", "Swing", "West Coast Swing", "East Coast Swing", "Jive", "Balboa", "Shag", "Charleston",
    // Ballroom
    "Waltz", "Tango", "Foxtrot", "Quickstep", "Viennese Waltz",
    // Street & Club
    "Hip-Hop", "Breakdance", "Popping", "Locking", "House", "Voguing", "Waacking", "Krump", "Dancehall",
    // Folk & Traditional
    "Folk", "Square Dance", "Contra Dance", "Irish Dance", "Scottish Dance",
    // Expressive/Modern
    "Contemporary", "Modern", "Jazz", "Lyrical", "Acro", "Ballet",
    // Blues & Fusion
    "Blues", "Fusion", "Zouk", "Kizomba"
  ];

  let { selectedStyles = [], onToggleStyle } = $props();

  function toggleStyle(style) {
    onToggleStyle(style.toLowerCase());
  }
</script>

<div>
  <label class="block font-bold mb-4">DANCE STYLES *</label>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
    {#each DANCE_STYLES as style (style)}
      <div
        onclick={() => toggleStyle(style)}
        class={`p-3 neo-border cursor-pointer text-center font-bold text-sm transition-all ${
          selectedStyles.includes(style.toLowerCase())
            ? "bg-green-500 text-white"
            : "bg-white text-black hover:bg-gray-100"
        }`}
      >
        {style.toUpperCase()}
      </div>
    {/each}
  </div>
</div>
