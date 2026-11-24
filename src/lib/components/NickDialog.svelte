<script lang="ts">
  import '@fluentui/web-components/button.js';
  import '@fluentui/web-components/text-input.js';
  import { _ } from 'svelte-i18n';
  import { app } from '$lib/stores/app.svelte';
  import { api } from '$lib/irc/api';

  let nick = $state(`${app.nickInUse ?? ''}_`);

  const val = (e: Event) => (e.currentTarget as unknown as { value: string }).value;

  function submit() {
    const next = nick.trim();
    if (!next) return;
    app.nickInUse = null;
    app.connecting = true;
    void api.setNick(next);
  }
</script>

<div class="overlay">
  <div class="modal">
    <h2>{$_('nick.title')}</h2>
    <p class="hint">{$_('nick.body', { values: { nick: app.nickInUse ?? '' } })}</p>
    <div class="field">
      <span>{$_('connect.nick')}</span>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <fluent-text-input
        aria-label={$_('connect.nick')}
        value={nick}
        oninput={(e: Event) => (nick = val(e))}
        onkeydown={(e: Event) => (e as KeyboardEvent).key === 'Enter' && submit()}
      ></fluent-text-input>
    </div>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <fluent-button appearance="primary" onclick={submit}>{$_('nick.submit')}</fluent-button>
  </div>
</div>