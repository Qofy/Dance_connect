<script>
  import { goto } from '@roxi/routify';
  import { Event } from "@/entities/Event";
  import Button from "@/components/ui/Button.svelte";
  import Input from "@/components/ui/Input.svelte";
  import Textarea from "@/components/ui/Textarea.svelte";
  import Select from "@/components/ui/Select.svelte";
  import { PlusCircle, Edit2 } from "lucide-svelte";
  import DanceStylePicker from "@/components/forms/DanceStylePicker.svelte";
  import Layout from "@/components/Layout.svelte";

  let _goto;
  goto.subscribe((fn) => (_goto = fn));

  let editEventId = $state(null);
  let isEditMode = $state(false);
  let submitting = $state(false);
  let successMsg = $state('');
  let errorMsg = $state('');

  let formData = $state({
    title: '',
    description: '',
    event_type: 'social',
    dance_styles: [],
    start_date: '',
    end_date: '',
    venue_name: '',
    address: '',
    city: '',
    state: '',
    ticket_price: '',
    max_attendees: ''
  });

  $effect(async () => {
    const params = new URLSearchParams(window.location.search);
    const id = params.get('id');
    if (!id) return;

    editEventId = id;
    isEditMode = true;
    try {
      const ev = await Event.get(id);
      formData = {
        title:        ev.title        || '',
        description:  ev.description  || '',
        event_type:   ev.event_type   || 'social',
        dance_styles: Array.isArray(ev.dance_styles) ? ev.dance_styles : [],
        start_date:   ev.start_date   || '',
        end_date:     ev.end_date     || '',
        venue_name:   ev.venue_name   || '',
        address:      ev.address      || '',
        city:         ev.city         || '',
        state:        ev.state        || '',
        ticket_price: ev.ticket_price != null ? String(ev.ticket_price) : '',
        max_attendees: ev.max_attendees != null ? String(ev.max_attendees) : ''
      };
    } catch {
      errorMsg = 'Failed to load event for editing.';
    }
  });

  function handleDanceStyleToggle(style) {
    if (formData.dance_styles.includes(style)) {
      formData.dance_styles = formData.dance_styles.filter(s => s !== style);
    } else {
      formData.dance_styles = [...formData.dance_styles, style];
    }
  }

  async function handleSubmit(status) {
    errorMsg = '';
    if (!formData.title.trim()) { errorMsg = 'Title is required.'; return; }
    if (!formData.city.trim() || !formData.state.trim()) { errorMsg = 'City and State are required.'; return; }
    if (!formData.start_date) { errorMsg = 'Start date is required.'; return; }
    if (formData.dance_styles.length === 0) { errorMsg = 'Select at least one dance style.'; return; }

    submitting = true;
    try {
      const payload = {
        ...formData,
        end_date:      formData.end_date     || null,
        ticket_price:  formData.ticket_price  ? parseFloat(formData.ticket_price)  : null,
        max_attendees: formData.max_attendees ? parseInt(formData.max_attendees)    : null,
        status,
      };

      if (isEditMode) {
        await Event.update(editEventId, payload);
        successMsg = status === 'draft' ? 'Draft saved!' : 'Event updated!';
      } else {
        await Event.create(payload);
        successMsg = status === 'draft' ? 'Draft saved!' : 'Event published!';
      }
      setTimeout(() => _goto('/dashboard'), 1200);
    } catch {
      errorMsg = 'Failed to save event. Please try again.';
    }
    submitting = false;
  }
</script>

<Layout>
  <div class="min-h-screen bg-white pb-16">
    <div class="max-w-3xl mx-auto p-6">

      <!-- Header -->
      <div class="text-center mb-8">
        <h1 class="text-4xl font-black mb-2 transform -rotate-1 flex items-center justify-center gap-3">
          {#if isEditMode}
            <Edit2 class="w-10 h-10 text-blue-600" />
            EDIT EVENT
          {:else}
            <PlusCircle class="w-10 h-10 text-pink-600" />
            CREATE EVENT
          {/if}
        </h1>
        {#if isEditMode}
          <p class="font-bold text-gray-500">Make changes and save or re-publish.</p>
        {/if}
      </div>

      {#if successMsg}
        <div class="mb-6 p-4 bg-green-100 neo-border border-green-600 text-green-800 font-black text-center">
          {successMsg}
        </div>
      {/if}
      {#if errorMsg}
        <div class="mb-6 p-4 bg-red-100 neo-border border-red-600 text-red-700 font-bold">
          {errorMsg}
        </div>
      {/if}

      <div class="space-y-6 neo-border neo-shadow p-8 bg-gray-50">

        <!-- EVENT DETAILS -->
        <h2 class="text-lg font-black border-b-2 border-black pb-2">EVENT DETAILS</h2>

        <div>
          <label class="block font-bold mb-2">TITLE <span class="text-red-600">*</span></label>
          <Input required bind:value={formData.title} maxlength="150"
            class="w-full neo-border font-bold text-lg h-12" placeholder="Your event title" />
        </div>

        <div>
          <label class="block font-bold mb-2">DESCRIPTION <span class="text-red-600">*</span></label>
          <Textarea bind:value={formData.description} rows={4} maxlength="1000"
            class="w-full neo-border font-bold" placeholder="What's happening at this event?" />
        </div>

        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <label class="block font-bold mb-2">EVENT TYPE <span class="text-red-600">*</span></label>
            <Select bind:value={formData.event_type} class="w-full neo-border font-bold h-12">
              <option value="social">Social</option>
              <option value="workshop">Workshop</option>
              <option value="competition">Competition</option>
              <option value="festival">Festival</option>
              <option value="showcase">Showcase</option>
            </Select>
          </div>
          <div>
            <label class="block font-bold mb-2">VENUE NAME</label>
            <Input bind:value={formData.venue_name}
              class="w-full neo-border font-bold h-12" placeholder="Studio / Club name" />
          </div>
        </div>

        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <label class="block font-bold mb-2">START DATE <span class="text-red-600">*</span></label>
            <Input required type="date" bind:value={formData.start_date}
              class="w-full neo-border font-bold h-12" />
          </div>
          <div>
            <label class="block font-bold mb-2">END DATE</label>
            <Input type="date" bind:value={formData.end_date}
              class="w-full neo-border font-bold h-12" />
          </div>
        </div>

        <!-- LOCATION -->
        <h2 class="text-lg font-black border-b-2 border-black pb-2 pt-2">LOCATION</h2>

        <div>
          <label class="block font-bold mb-2">ADDRESS</label>
          <Input bind:value={formData.address}
            class="w-full neo-border font-bold h-12" placeholder="123 Main St" />
        </div>

        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <label class="block font-bold mb-2">CITY <span class="text-red-600">*</span></label>
            <Input required bind:value={formData.city}
              class="w-full neo-border font-bold h-12" placeholder="New York" />
          </div>
          <div>
            <label class="block font-bold mb-2">STATE <span class="text-red-600">*</span></label>
            <Input required bind:value={formData.state}
              class="w-full neo-border font-bold h-12" placeholder="NY" />
          </div>
        </div>

        <!-- REGISTRATION SETTINGS -->
        <h2 class="text-lg font-black border-b-2 border-black pb-2 pt-2">REGISTRATION SETTINGS</h2>

        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <label class="block font-bold mb-2">MAX ATTENDEES</label>
            <Input type="number" min="1" bind:value={formData.max_attendees}
              class="w-full neo-border font-bold h-12" placeholder="100" />
          </div>
          <div>
            <label class="block font-bold mb-2">TICKET PRICE ($)</label>
            <Input type="number" min="0" step="0.01" bind:value={formData.ticket_price}
              class="w-full neo-border font-bold h-12" placeholder="0 = free" />
          </div>
        </div>

        <!-- DANCE STYLES -->
        <h2 class="text-lg font-black border-b-2 border-black pb-2 pt-2">DANCE STYLES <span class="text-red-600">*</span></h2>
        <DanceStylePicker selectedStyles={formData.dance_styles} onToggleStyle={handleDanceStyleToggle} />

        <!-- ACTIONS -->
        <div class="flex gap-4 pt-4">
          <Button
            type="button"
            onclick={() => handleSubmit('draft')}
            disabled={submitting}
            class="flex-1 bg-gray-700 text-white neo-border neo-shadow font-black py-3"
          >
            {submitting ? '...' : isEditMode ? 'SAVE DRAFT' : 'DRAFT'}
          </Button>
          <Button
            type="button"
            onclick={() => handleSubmit('public')}
            disabled={submitting}
            class="flex-1 bg-pink-600 text-white neo-border neo-shadow font-black py-3"
          >
            {submitting ? 'SAVING...' : isEditMode ? 'UPDATE & PUBLISH' : 'PUBLISH'}
          </Button>
          <Button
            type="button"
            onclick={() => _goto('/dashboard')}
            class="bg-white text-black neo-border font-bold px-6"
          >
            CANCEL
          </Button>
        </div>
      </div>
    </div>
  </div>
</Layout>
