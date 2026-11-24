<script lang="ts">
  import '@fluentui/web-components/button.js';
  import '@fluentui/web-components/text-input.js';
  import { _ } from 'svelte-i18n';
  import { app } from '$lib/stores/app.svelte';

  let nick = $state(app.settings.nick);
  let account = $state(app.settings.saslAccount);
  let password = $state(app.settings.saslPassword);
  let error = $state('');

  const val = (e: Event) => (e.currentTarget as unknown as { value: string }).value;
  const enter = (e: Event) => (e as KeyboardEvent).key === 'Enter';

  function submit() {
    if (!nick.trim()) {
      error = $_('connect.nickRequired');
      return;
    }
    app.settings.nick = nick.trim();
    app.settings.saslAccount = account.trim();
    app.settings.saslPassword = password;
    app.persist();
    app.connect();
  }
</script>

<div class="overlay">
  <div class="modal">
    <div class="brand">
      <img src="/icon.svg" alt="" />
      <h2>{$_('connect.title')}</h2>
    </div>
    <div class="field">
      <span>{$_('connect.nick')}</span>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <fluent-text-input
        aria-label={$_('connect.nick')}
        placeholder={$_('connect.nickPlaceholder')}
        value={nick}
        oninput={(e: Event) => (nick = val(e))}
        onkeydown={(e: Event) => enter(e) && submit()}
      ></fluent-text-input>
    </div>
    <div class="field">
      <span>{$_('connect.saslAccount')}</span>
      <fluent-text-input aria-label={$_('connect.saslAccount')} value={account} oninput={(e: Event) => (account = val(e))}></fluent-text-input>
    </div>
    <div class="field">
      <span>{$_('connect.saslPassword')}</span>
      <fluent-text-input aria-label={$_('connect.saslPassword')} type="password" value={password} oninput={(e: Event) => (password = val(e))}></fluent-text-input>
    </div>
    {#if error}<div class="err">{error}</div>{/if}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <fluent-button appearance="primary" onclick={submit}>{$_('connect.submit')}</fluent-button>
    <p class="hint">{$_('connect.registerHint')}</p>
    <button class="skip" onclick={() => (app.showConnect = false)}>{$_('chat.close')}</button>
  </div>
</div>

<style>
  .brand { display: flex; align-items: center; gap: 10px; }
  .brand img { width: 34px; height: 34px; }
  .skip {
    align-self: center; border: none; background: transparent;
    color: var(--text-secondary); font-size: 12px; cursor: pointer;
  }
  .skip:hover { color: var(--qq-blue); }
</style>