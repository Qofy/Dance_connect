<script>
  import Button from "@/components/ui/Button.svelte";
  import { MapPin, Loader2 } from "lucide-svelte";

  let { onLocationFound } = $props();
  let loading = $state(false);
  let error = $state(null);

  function getCurrentLocation() {
    loading = true;
    error = null;

    if (!navigator.geolocation) {
      error = "Geolocation not supported";
      loading = false;
      return;
    }

    navigator.geolocation.getCurrentPosition(
      async (position) => {
        try {
          const response = await fetch(
            `https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${position.coords.latitude}&longitude=${position.coords.longitude}&localityLanguage=en`
          );
          const data = await response.json();

          const location = {
            city: data.city || data.locality || "Unknown",
            state: data.principalSubdivision || data.countryName || "Unknown",
            latitude: position.coords.latitude,
            longitude: position.coords.longitude
          };

          onLocationFound(location);
          loading = false;
        } catch (err) {
          error = "Failed to get location details";
          loading = false;
        }
      },
      (err) => {
        error = "Location access denied";
        loading = false;
      },
      {
        enableHighAccuracy: true,
        timeout: 10000,
        maximumAge: 600000
      }
    );
  }

  $effect(() => {
    getCurrentLocation();
  });
</script>

<div class="text-center">
  {#if loading}
    <div class="bg-white text-black px-6 py-3 neo-border neo-shadow inline-block font-bold">
      <Loader2 class="w-5 h-5 inline mr-2 animate-spin" />
      DETECTING LOCATION...
    </div>
  {:else if error}
    <div class="space-y-4">
      <div class="bg-red-500 text-white px-6 py-3 neo-border neo-shadow inline-block font-bold">
        LOCATION ERROR: {error.toUpperCase()}
      </div>
      <div>
        <Button
          onclick={getCurrentLocation}
          class="bg-white text-black neo-border neo-shadow neo-hover font-bold"
        >
          <MapPin class="w-5 h-5 mr-2" />
          TRY AGAIN
        </Button>
      </div>
    </div>
  {/if}
</div>
