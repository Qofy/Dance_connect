<script>
  import { User } from "@/entities/User";
  import { Event } from "@/entities/Event";
  import { Registration } from "@/entities/Registration";
  import { Mail, Phone, MapPin, Edit, User as UserIcon, Shield } from "lucide-svelte";
  import { goto } from '@roxi/routify';
  import EventCard from "@/components/events/EventCard.svelte";
  import { Button } from "@/components/ui/Button.svelte";
  import { createPageUrl } from "@/utils";
  import Layout from "@/components/Layout.svelte";

  let user = $state(null);
  let createdEvents = $state([]);
  let registeredEvents = $state([]);
  let loading = $state(true);

  $effect(async () => {
    try {
      const currentUser = await User.me();
      user = currentUser;

      if (
        currentUser.user_type === 'event_creator' ||
        currentUser.user_type === 'both' ||
        currentUser.role === 'admin'
      ) {
        const events = await Event.filter({ created_by: currentUser.email });
        createdEvents = events || [];
      }

      const regs = await Registration.list();
      if (regs && regs.length > 0) {
        const eventIds = regs.map(r => r.event_id);
        const allEvents = await Event.list();
        registeredEvents = allEvents.filter(e => eventIds.includes(e.id)) || [];
      }
    } catch (e) {
      console.error("Failed to load profile", e);
    } finally {
      loading = false;
    }
  });

  async function handleLogout() {
    try {
      const token = localStorage.getItem('token');
      await fetch('/api/logout', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token })
      });
    } catch (error) {
      console.error('Logout error:', error);
    }
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    goto('/login');
  }
</script>

<Layout>
  <div class="p-6 bg-white">
    <div class="max-w-5xl mx-auto">
      {#if loading}
        <div class="p-6 font-black text-2xl text-center">LOADING PROFILE...</div>
      {:else if !user}
        <div class="p-6 font-black text-2xl text-center">PLEASE LOG IN TO VIEW YOUR PROFILE.</div>
      {:else}
        <!-- Profile Header -->
        <div class="mb-8 p-6 neo-border neo-shadow bg-yellow-400 transform -rotate-1">
          <div class="flex flex-col md:flex-row items-center gap-6">
            <div class="w-24 h-24 neo-border bg-blue-600 text-white flex items-center justify-center">
              <UserIcon class="w-16 h-16" />
            </div>
            <div class="flex-1 text-center md:text-left">
              <h1 class="text-4xl font-black">{user.full_name}</h1>
              <div class="flex flex-wrap justify-center md:justify-start gap-2 mt-2">
                {#if user.user_type}
                  <span class="px-3 py-1 bg-pink-600 text-white neo-border font-bold text-sm">
                    {user.user_type.toUpperCase()}
                  </span>
                {/if}
                {#if user.role === 'admin'}
                  <span class="px-3 py-1 bg-red-600 text-white neo-border font-bold text-sm flex items-center gap-1">
                    <Shield class="w-4 h-4" /> SUPERADMIN
                  </span>
                {/if}
              </div>
            </div>
            <Button
              onclick={() => goto('/edit-profile')}
              class="bg-white text-black neo-border neo-shadow neo-hover font-bold"
            >
              <Edit class="w-4 h-4 mr-2" /> EDIT PROFILE
            </Button>
          </div>
        </div>

        <!-- User Details -->
        <div class="grid md:grid-cols-3 gap-4 mb-12">
          <div class="p-4 neo-border bg-gray-100 flex items-center gap-3">
            <Mail class="w-5 h-5" />
            <span class="font-bold">{user.email}</span>
          </div>
          <div class="p-4 neo-border bg-gray-100 flex items-center gap-3">
            <Phone class="w-5 h-5" />
            <span class="font-bold">{user.phone || "N/A"}</span>
          </div>
          <div class="p-4 neo-border bg-gray-100 flex items-center gap-3">
            <MapPin class="w-5 h-5" />
            <span class="font-bold">{user.city || "N/A"}, {user.state || ""}</span>
          </div>
        </div>

        <!-- ADMIN PANEL -->
        {#if user.role === 'admin'}
          <div class="mb-12">
            <h2 class="text-3xl font-black mb-6">ADMIN PANEL</h2>
            <div class="grid md:grid-cols-2 gap-4">
              <a href={createPageUrl('AdminEventManager')}>
                <div class="p-6 neo-border neo-shadow neo-hover bg-red-600 text-white font-black text-xl cursor-pointer">
                  MANAGE ALL EVENTS
                </div>
              </a>
              <a href={createPageUrl('UserManager')}>
                <div class="p-6 neo-border neo-shadow neo-hover bg-red-600 text-white font-black text-xl cursor-pointer">
                  MANAGE USERS
                </div>
              </a>
            </div>
          </div>
        {/if}

        <!-- Created Events -->
        {#if user.user_type === 'event_creator' || user.user_type === 'both' || user.role === 'admin'}
          <div class="mb-12">
            <h2 class="text-3xl font-black mb-6">CREATED EVENTS</h2>
            {#if createdEvents.length > 0}
              <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
                {#each createdEvents as event (event.id)}
                  <EventCard {event} />
                {/each}
              </div>
            {:else}
              <p class="font-bold neo-border p-4 bg-gray-100">You haven't created any events yet.</p>
            {/if}
          </div>
        {/if}

        <!-- Registered Events -->
        <div class="mb-12">
          <h2 class="text-3xl font-black mb-6">REGISTERED EVENTS</h2>
          {#if registeredEvents.length > 0}
            <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {#each registeredEvents as event (event.id)}
                <EventCard {event} />
              {/each}
            </div>
          {:else}
            <p class="font-bold neo-border p-4 bg-gray-100">You are not registered for any events.</p>
          {/if}
        </div>

        <!-- Logout Button -->
        <div class="text-center mt-8">
          <Button
            onclick={handleLogout}
            class="bg-red-600 text-white neo-border neo-shadow neo-hover font-black px-8 py-3"
          >
            LOGOUT
          </Button>
        </div>
      {/if}
    </div>
  </div>
</Layout>
