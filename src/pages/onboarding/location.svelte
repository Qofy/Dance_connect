<script>
  import { goto } from '@roxi/routify';
  import Button from '@/components/ui/Button.svelte';
  import Input from '@/components/ui/Input.svelte';
  import { MapPin, Loader2 } from 'lucide-svelte';

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  let detecting = $state(false);
  let detectError = $state('');
  let detectedCity = $state('');
  let detectedState = $state('');
  let detectedLat = $state(null);
  let detectedLng = $state(null);
  let useDetected = $state(false);

  // Manual entry
  let city = $state('');
  let state = $state('');
  let zip = $state('');

  async function detectLocation() {
    detecting = true;
    detectError = '';
    useDetected = false;

    if (!navigator.geolocation) {
      detectError = 'Geolocation not supported by your browser.';
      detecting = false;
      return;
    }

    navigator.geolocation.getCurrentPosition(
      async (position) => {
        try {
          const res = await fetch(
            `https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${position.coords.latitude}&longitude=${position.coords.longitude}&localityLanguage=en`
          );
          const data = await res.json();
          detectedCity = data.city || data.locality || '';
          detectedState = data.principalSubdivision || '';
          detectedLat = position.coords.latitude;
          detectedLng = position.coords.longitude;
          useDetected = true;
          detecting = false;
        } catch (err) {
          detectError = 'Could not resolve your location. Enter manually below.';
          detecting = false;
        }
      },
      () => {
        detectError = 'Location access denied. Enter manually below.';
        detecting = false;
      },
      { enableHighAccuracy: true, timeout: 10000, maximumAge: 600000 }
    );
  }

  function confirmDetected() {
    city = detectedCity;
    state = detectedState;
    useDetected = false; // dismiss the banner, keep values in manual fields
  }

  function proceed() {
    const locationData = {
      city: city.trim(),
      state: state.trim(),
      zip_code: zip.trim(),
      latitude: detectedLat,
      longitude: detectedLng
    };
    localStorage.setItem('onboarding_location', JSON.stringify(locationData));
    _goto('/onboarding/welcome');
  }

  function skip() {
    localStorage.setItem('onboarding_location', JSON.stringify({}));
    _goto('/onboarding/welcome');
  }
</script>

<div class="min-h-screen bg-gray-100 flex items-center justify-center p-4">
  <div class="max-w-lg w-full">
    <!-- Progress Bar -->
    <div class="mb-6">
      <div class="flex justify-between font-bold text-sm mb-2">
        <span>STEP 3 OF 4</span>
        <span class="text-gray-500">LOCATION</span>
      </div>
      <div class="w-full bg-gray-300 neo-border h-3">
        <div class="bg-blue-600 h-full transition-all" style="width: 75%"></div>
      </div>
    </div>

    <div class="bg-white neo-border neo-shadow p-8">
      <!-- Header -->
      <div class="text-center mb-8">
        <h1 class="text-3xl font-black transform -rotate-1">WHERE DO YOU DANCE?</h1>
        <p class="font-bold text-gray-600 mt-2">We'll show you events near you</p>
      </div>

      <!-- Detect Button -->
      <div class="text-center mb-6">
        <Button
          type="button"
          onclick={detectLocation}
          disabled={detecting}
          class="bg-pink-600 text-white font-black neo-border neo-shadow neo-hover px-8 py-3"
        >
          {#if detecting}
            <Loader2 class="w-5 h-5 inline mr-2 animate-spin" />
            DETECTING...
          {:else}
            <MapPin class="w-5 h-5 inline mr-2" />
            DETECT MY LOCATION
          {/if}
        </Button>

        {#if detectError}
          <p class="text-red-600 font-bold text-sm mt-3">{detectError}</p>
        {/if}
      </div>

      <!-- Detected Location Banner -->
      {#if useDetected && detectedCity}
        <div class="mb-6 p-4 bg-green-100 neo-border border-green-600 flex items-center justify-between gap-4">
          <div>
            <p class="font-black text-green-800">
              <MapPin class="w-4 h-4 inline mr-1" />
              {detectedCity}, {detectedState}
            </p>
            <p class="font-bold text-green-700 text-sm">Location detected!</p>
          </div>
          <Button
            type="button"
            onclick={confirmDetected}
            class="bg-green-600 text-white font-bold text-sm"
          >
            ✓ USE THIS
          </Button>
        </div>
      {/if}

      <!-- Divider -->
      <div class="flex items-center gap-4 mb-6">
        <div class="flex-1 h-px bg-gray-300"></div>
        <span class="font-bold text-gray-500 text-sm">OR ENTER MANUALLY</span>
        <div class="flex-1 h-px bg-gray-300"></div>
      </div>

      <!-- Manual Entry -->
      <div class="space-y-4">
        <div>
          <label class="block font-bold mb-2">CITY</label>
          <Input bind:value={city} class="w-full" placeholder="San Francisco" />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block font-bold mb-2">STATE</label>
            <Input bind:value={state} class="w-full" placeholder="CA" />
          </div>
          <div>
            <label class="block font-bold mb-2">ZIP CODE</label>
            <Input bind:value={zip} class="w-full" placeholder="94105" />
          </div>
        </div>
      </div>

      <!-- Navigation -->
      <div class="flex gap-4 mt-8">
        <Button
          type="button"
          onclick={skip}
          class="flex-1 bg-gray-200 text-black font-bold"
        >
          SKIP
        </Button>
        <Button
          type="button"
          onclick={proceed}
          disabled={!city.trim() && !useDetected}
          class="flex-2 bg-blue-600 text-white font-black px-8 disabled:opacity-50"
        >
          COMPLETE →
        </Button>
      </div>
      <p class="text-center text-gray-500 font-bold text-xs mt-3">
        Skipping shows national events instead
      </p>
    </div>
  </div>
</div>
