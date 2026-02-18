<script>
  import { Plus, User, Calendar, Map } from "lucide-svelte";
  import { createPageUrl } from "@/utils";

  let { user } = $props();

  const actions = [
    {
      title: user ? "CREATE EVENT" : "REGISTER",
      icon: user ? Plus : User,
      url: user ? createPageUrl("CreateEvent") : createPageUrl("Register"),
      color: "bg-pink-600",
      rotation: "rotate-2"
    },
    {
      title: "CALENDAR VIEW",
      icon: Calendar,
      url: createPageUrl("Calendar"),
      color: "bg-green-500",
      rotation: "-rotate-1"
    },
    {
      title: "MAP VIEW",
      icon: Map,
      url: createPageUrl("MapView"),
      color: "bg-purple-600",
      rotation: "rotate-1"
    }
  ];
</script>

<div class="my-8">
  <h2 class="text-2xl font-black mb-6">QUICK ACTIONS</h2>
  <div class="grid md:grid-cols-3 gap-4">
    {#each actions as action (action.title)}
      <a href={action.url}>
        <div class="{action.color} text-white p-6 neo-border neo-shadow neo-hover transform {action.rotation} cursor-pointer">
          <div class="flex items-center gap-3">
            <svelte:component this={action.icon} class="w-6 h-6" />
            <span class="font-black text-lg">{action.title}</span>
          </div>
        </div>
      </a>
    {/each}
  </div>
</div>
