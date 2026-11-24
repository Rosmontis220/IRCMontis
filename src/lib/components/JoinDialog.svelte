<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { app } from '$lib/stores/app.svelte';
  import { api } from '$lib/irc/api';

  let channel = $state('');

  const val = (e: Event) => (e.currentTarget as unknown as { value: string }).value;

  function submit() {
    let name = channel.trim();
    if (!name) return;
    if (!name.startsWith('#') && !name.startsWith('&')) name = `#${name}`;
    name = name.toLowerCase();
    app.showJoin = false;
    void api.join(name);
    app.openSession(name, 'channel');
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="overlay" onclick={(e) => e.target === e.currentTarget && (app.showJoin = false)}>
  <div class="modal">
    <h2>{$_('join.title')}</h2>
    <div class="field">
      <input
        aria-label={$_('join.title')}
        placeholder={$_('join.placeholder')}
        value={channel}
        oninput={(e: Event) => (channel = val(e))}
        onkeydown={(e: Event) => (e as KeyboardEvent).key === 'Enter' && submit()}
      />
    </div>
    <button class="btn primary" onclick={submit}>{$_('join.submit')}</button>
  </div>
</div>