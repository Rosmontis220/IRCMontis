<script lang="ts">
  import '@fluentui/web-components/button.js';
  import '@fluentui/web-components/text-input.js';
  import '@fluentui/web-components/switch.js';
  import '@fluentui/web-components/slider.js';
  import '@fluentui/web-components/dropdown.js';
  import '@fluentui/web-components/listbox.js';
  import '@fluentui/web-components/option.js';
  import { _ } from 'svelte-i18n';
  import { get } from 'svelte/store';
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { app } from '$lib/stores/app.svelte';
  import { api } from '$lib/irc/api';
  import { setLocale } from '$lib/i18n';

  let tab = $state<'account' | 'appearance' | 'notifications' | 'general'>('appearance');
  let themeDd: HTMLElement | undefined = $state();
  let langDd: HTMLElement | undefined = $state();

  // fluent-dropdown's value setter needs its slotted listbox; assigning during
  // initial render throws, so set it once the slot assignment has run
  $effect(() => {
    tab;
    const theme = app.settings.theme;
    const locale = app.settings.locale;
    setTimeout(() => {
      if (themeDd) (themeDd as unknown as { value: string }).value = theme;
      if (langDd) (langDd as unknown as { value: string }).value = locale;
    }, 0);
  });

  const val = (e: Event) => (e.currentTarget as unknown as { value: string }).value;
  const chk = (e: Event) => (e.currentTarget as unknown as { checked: boolean }).checked;

  function setTheme(v: string) {
    app.settings.theme = v as typeof app.settings.theme;
    app.applyAppearance();
    app.persist();
  }

  function setLanguage(v: string) {
    app.settings.locale = v as typeof app.settings.locale;
    setLocale(app.settings.locale);
    app.persist();
    void api.trayLabels(get(_)('tray.show'), get(_)('tray.quit'));
  }

  async function pickBackground() {
    const picked = await open({
      multiple: false,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] }]
    });
    if (typeof picked === 'string') {
      app.settings.backgroundPath = await api.saveBackground(picked);
      app.persist();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="overlay" onclick={(e) => e.target === e.currentTarget && (app.showSettings = false)}>
  <div class="modal wide">
    <div class="top">
      <h2>{$_('settings.title')}</h2>
      <button class="x" onclick={() => (app.showSettings = false)}>✕</button>
    </div>
    <div class="grid">
      <nav>
        <button class:on={tab === 'account'} onclick={() => (tab = 'account')}>{$_('settings.account')}</button>
        <button class:on={tab === 'appearance'} onclick={() => (tab = 'appearance')}>{$_('settings.appearance')}</button>
        <button class:on={tab === 'notifications'} onclick={() => (tab = 'notifications')}>{$_('settings.notifications')}</button>
        <button class:on={tab === 'general'} onclick={() => (tab = 'general')}>{$_('settings.general')}</button>
      </nav>
      <div class="content">
        {#if tab === 'account'}
          <div class="field">
            {$_('connect.nick')}
            <fluent-text-input aria-label={$_('connect.nick')} value={app.settings.nick} oninput={(e: Event) => (app.settings.nick = val(e))} onchange={() => app.persist()}></fluent-text-input>
          </div>
          <div class="field">
            {$_('connect.saslAccount')}
            <fluent-text-input aria-label={$_('connect.saslAccount')} value={app.settings.saslAccount} oninput={(e: Event) => (app.settings.saslAccount = val(e))} onchange={() => app.persist()}></fluent-text-input>
          </div>
          <div class="field">
            {$_('connect.saslPassword')}
            <fluent-text-input aria-label={$_('connect.saslPassword')} type="password" value={app.settings.saslPassword} oninput={(e: Event) => (app.settings.saslPassword = val(e))} onchange={() => app.persist()}></fluent-text-input>
          </div>
          {#if app.connected}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
            <fluent-button onclick={() => app.disconnect()}>{$_('settings.disconnect')}</fluent-button>
          {:else}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
            <fluent-button appearance="primary" onclick={() => app.connect()}>{$_('settings.connect')}</fluent-button>
          {/if}
        {:else if tab === 'appearance'}
          <div class="field">
            {$_('settings.theme')}
            <fluent-dropdown bind:this={themeDd} aria-label={$_('settings.theme')} onchange={(e: Event) => setTheme(val(e))}>
              <fluent-listbox>
                <fluent-option value="light">{$_('settings.themeLight')}</fluent-option>
                <fluent-option value="dark">{$_('settings.themeDark')}</fluent-option>
                <fluent-option value="system">{$_('settings.themeSystem')}</fluent-option>
              </fluent-listbox>
            </fluent-dropdown>
          </div>
          <div class="field">
            {$_('settings.background')}
            <div class="bgrow">
              {#if app.settings.backgroundPath}
                <img class="thumb" src={convertFileSrc(app.settings.backgroundPath)} alt="" />
              {/if}
              <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
              <fluent-button onclick={pickBackground}>{$_('settings.backgroundPick')}</fluent-button>
              {#if app.settings.backgroundPath}
                <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
                <fluent-button onclick={() => { app.settings.backgroundPath = null; app.persist(); }}>{$_('settings.backgroundClear')}</fluent-button>
              {/if}
            </div>
          </div>
          {#if app.settings.backgroundPath}
            <div class="field">
              {$_('settings.backgroundOpacity')} · {app.settings.backgroundOpacity}%
              <fluent-slider
                aria-label={$_('settings.backgroundOpacity')}
                min="5"
                max="100"
                step="5"
                value={String(app.settings.backgroundOpacity)}
                onchange={(e: Event) => { app.settings.backgroundOpacity = Number(val(e)); app.persist(); }}
              ></fluent-slider>
            </div>
          {/if}
        {:else if tab === 'notifications'}
          <div class="switchrow">
            <fluent-switch aria-label={$_('settings.enableNotifications')} checked={app.settings.notifications} onchange={(e: Event) => { app.settings.notifications = chk(e); app.persist(); }}></fluent-switch>
            {$_('settings.enableNotifications')}
          </div>
          <p class="hint">{$_('sessions.muted')}：{$_('sessionMenu.mute')}</p>
        {:else}
          <div class="field">
            {$_('settings.language')}
            <fluent-dropdown bind:this={langDd} aria-label={$_('settings.language')} onchange={(e: Event) => setLanguage(val(e))}>
              <fluent-listbox>
                <fluent-option value="system">{$_('settings.langSystem')}</fluent-option>
                <fluent-option value="zh">中文</fluent-option>
                <fluent-option value="en">English</fluent-option>
              </fluent-listbox>
            </fluent-dropdown>
          </div>
          <div class="switchrow">
            <fluent-switch aria-label={$_('settings.closeToTray')} checked={app.settings.closeToTray} onchange={(e: Event) => { app.settings.closeToTray = chk(e); app.persist(); }}></fluent-switch>
            {$_('settings.closeToTray')}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .wide { width: min(560px, calc(100vw - 48px)); }
  .top { display: flex; align-items: center; justify-content: space-between; }
  .x { border: none; background: transparent; color: var(--text-secondary); cursor: pointer; font-size: 14px; }
  .x:hover { color: var(--danger); }
  .grid { display: flex; gap: 16px; min-height: 260px; }
  nav { display: flex; flex-direction: column; gap: 2px; width: 96px; flex: none; }
  nav button {
    border: none; background: transparent; text-align: left;
    padding: 8px 10px; border-radius: 6px; font-size: 13px;
    color: var(--text-primary); cursor: pointer;
  }
  nav button:hover { background: var(--bg-session-active); }
  nav button.on { background: var(--bg-session-active); color: var(--qq-blue); font-weight: 600; }
  .content { flex: 1; display: flex; flex-direction: column; gap: 14px; min-width: 0; }
  .bgrow { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .thumb { width: 72px; height: 44px; object-fit: cover; border-radius: 6px; border: 1px solid var(--border-light); }
  .switchrow { display: flex; align-items: center; gap: 10px; font-size: 13px; }
</style>