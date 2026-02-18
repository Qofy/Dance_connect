<script>
  import { goto } from '@roxi/routify';
  import { Event } from "@/entities/Event";
  import Button from "@/components/ui/Button.svelte";
  import Input from "@/components/ui/Input.svelte";
  import Textarea from "@/components/ui/Textarea.svelte";
  import Select from "@/components/ui/Select.svelte";
  import { PlusCircle } from "lucide-svelte";
  import DanceStylePicker from "@/components/forms/DanceStylePicker.svelte";
  import Layout from "@/components/Layout.svelte";

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  let formData = $state({
    title: "",
    description: "",
    event_type: "social",
    dance_styles: [],
    start_date: "",
    end_date: "",
    city: "",
    state: "",
    address: "",
    ticket_price: 0
  });
  let submitting = $state(false);

  function handleDanceStyleToggle(style) {
    if (formData.dance_styles.includes(style)) {
      formData.dance_styles = formData.dance_styles.filter(s => s !== style);
    } else {
      formData.dance_styles = [...formData.dance_styles, style];
    }
  }

  async function handleSubmit(e) {
    e.preventDefault();
    submitting = true;
    try {
      const eventData = {
        ...formData,
        start_date: formData.start_date,
        end_date: formData.end_date || null,
        ticket_price: formData.ticket_price || null,
      };

      await Event.create(eventData);
      alert("Event created successfully!");
      _goto('/dashboard');
    } catch (error) {
      console.error("Failed to create event:", error);
      alert("Failed to create event. Please try again.");
    }
    submitting = false;
  }
</script>

<Layout>
  <div class="p-6 bg-white">
    <div class="max-w-3xl mx-auto">
      <div class="text-center mb-8">
        <h1 class="text-5xl font-black mb-4 transform -rotate-1 flex items-center justify-center gap-4">
          <PlusCircle class="w-12 h-12 text-pink-600" />
          CREATE A BRUTAL EVENT
        </h1>
      </div>

      <form onsubmit={handleSubmit} class="space-y-6 neo-border neo-shadow p-8 bg-gray-50">
        <div>
          <label class="block font-bold mb-2">EVENT TITLE *</label>
          <Input
            required
            bind:value={formData.title}
            class="neo-border font-bold text-lg h-12"
          />
        </div>

        <div>
          <label class="block font-bold mb-2">DESCRIPTION</label>
          <Textarea
            bind:value={formData.description}
            rows={5}
            class="neo-border font-bold"
          />
        </div>

        <DanceStylePicker selectedStyles={formData.dance_styles} onToggleStyle={handleDanceStyleToggle} />

        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <label class="block font-bold mb-2">EVENT TYPE *</label>
            <Select bind:value={formData.event_type} class="neo-border font-bold h-12">
              <option value="social">Social</option>
              <option value="workshop">Workshop</option>
              <option value="competition">Competition</option>
              <option value="festival">Festival</option>
              <option value="showcase">Showcase</option>
            </Select>
          </div>
          <div>
            <label class="block font-bold mb-2">CITY *</label>
            <Input
              required
              bind:value={formData.city}
              class="neo-border font-bold text-lg h-12"
            />
          </div>
        </div>

        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <label class="block font-bold mb-2">STATE *</label>
            <Input
              required
              bind:value={formData.state}
              class="neo-border font-bold text-lg h-12"
            />
          </div>
          <div>
            <label class="block font-bold mb-2">ADDRESS *</label>
            <Input
              required
              bind:value={formData.address}
              class="neo-border font-bold text-lg h-12"
            />
          </div>
        </div>

        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <label class="block font-bold mb-2">START DATE *</label>
            <Input
              required
              type="datetime-local"
              bind:value={formData.start_date}
              class="neo-border font-bold text-lg h-12"
            />
          </div>
          <div>
            <label class="block font-bold mb-2">END DATE</label>
            <Input
              type="datetime-local"
              bind:value={formData.end_date}
              class="neo-border font-bold text-lg h-12"
            />
          </div>
        </div>

        <div>
          <label class="block font-bold mb-2">TICKET PRICE ($)</label>
          <Input
            type="number"
            step="0.01"
            bind:value={formData.ticket_price}
            class="neo-border font-bold text-lg h-12"
          />
        </div>

        <div class="flex gap-4 pt-4">
          <Button
            type="submit"
            disabled={submitting}
            class="flex-1 bg-pink-600 text-white neo-border neo-shadow neo-hover font-black py-3"
          >
            {submitting ? 'CREATING...' : 'CREATE EVENT'}
          </Button>
          <Button
            type="button"
            onclick={() => _goto('/dashboard')}
            class="flex-1 bg-gray-600 text-white neo-border neo-shadow font-bold"
          >
            CANCEL
          </Button>
        </div>
      </form>
    </div>
  </div>
</Layout>
