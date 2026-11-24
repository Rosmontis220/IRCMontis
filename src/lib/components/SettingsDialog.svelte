<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { get } from 'svelte/store';
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { app } from '$lib/stores/app.svelte';
  import { api } from '$lib/irc/api';
  import { setLocale } from '$lib/i18n';

  let tab = $state<'account' | 'appearance' | 'notifications' | 'general'>('appearance');

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
            <input aria-label={$_('connect.nick')} value={app.settings.nick} oninput={(e) => (app.settings.nick = e.currentTarget.value)} onchange={() => app.persist()} />
          </div>
          <div class="field">
            {$_('connect.saslAccount')}
            <input aria-label={$_('connect.saslAccount')} value={app.settings.saslAccount} oninput={(e) => (app.settings.saslAccount = e.currentTarget.value)} onchange={() => app.persist()} />
          </div>
          <div class="field">
            {$_('connect.saslPassword')}
            <input aria-label={$_('connect.saslPassword')} type="password" value={app.settings.saslPassword} oninput={(e) => (app.settings.saslPassword = e.currentTarget.value)} onchange={() => app.persist()} />
          </div>
          {#if app.connected}
            <button class="btn" onclick={() => app.disconnect()}>{$_('settings.disconnect')}</button>
          {:else}
            <button class="btn primary" onclick={() => app.connect()}>{$_('settings.connect')}</button>
          {/if}
        {:else if tab === 'appearance'}
          <div class="field">
            {$_('settings.theme')}
            <select aria-label={$_('settings.theme')} value={app.settings.theme} onchange={(e) => setTheme(e.currentTarget.value)}>
              <option value="light">{$_('settings.themeLight')}</option>
              <option value="dark">{$_('settings.themeDark')}</option>
              <option value="system">{$_('settings.themeSystem')}</option>
            </select>
          </div>
          <div class="field">
            {$_('settings.background')}
            <div class="bgrow">
              {#if app.settings.backgroundPath}
                <img class="thumb" src={convertFileSrc(app.settings.backgroundPath)} alt="" />
              {/if}
              <button class="btn" onclick={pickBackground}>{$_('settings.backgroundPick')}</button>
              {#if app.settings.backgroundPath}
                <button class="btn" onclick={() => { app.settings.backgroundPath = null; app.persist(); }}>{$_('settings.backgroundClear')}</button>
              {/if}
            </div>
          </div>
          {#if app.settings.backgroundPath}
            <div class="field">
              {$_('settings.backgroundOpacity')} · {app.settings.backgroundOpacity}%
              <input
                type="range"
                aria-label={$_('settings.backgroundOpacity')}
                min="5"
                max="100"
                step="5"
                value={app.settings.backgroundOpacity}
                oninput={(e) => { app.settings.backgroundOpacity = Number(e.currentTarget.value); app.persist(); }}
              />
            </div>
          {/if}
        {:else if tab === 'notifications'}
          <div class="switchrow">
            <button
              class="switch"
              class:on={app.settings.notifications}
              role="switch"
              aria-checked={app.settings.notifications}
              aria-label={$_('settings.enableNotifications')}
              onclick={() => { app.settings.notifications = !app.settings.notifications; app.persist(); }}
            ><span class="knob"></span></button>
            {$_('settings.enableNotifications')}
          </div>
          <p class="hint">{$_('sessions.muted')}：{$_('sessionMenu.mute')}</p>
        {:else}
          <div class="field">
            {$_('settings.language')}
            <select aria-label={$_('settings.language')} value={app.settings.locale} onchange={(e) => setLanguage(e.currentTarget.value)}>
              <option value="system">{$_('settings.langSystem')}</option>
              <option value="zh">中文</option>
              <option value="en">English</option>
            </select>
          </div>
          <div class="switchrow">
            <button
              class="switch"
              class:on={app.settings.closeToTray}
              role="switch"
              aria-checked={app.settings.closeToTray}
              aria-label={$_('settings.closeToTray')}
              onclick={() => { app.settings.closeToTray = !app.settings.closeToTray; app.persist(); }}
            ><span class="knob"></span></button>
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
    color: var(--text-primary); cursor: pointer; font-family: inherit;
  }
  nav button:hover { background: var(--bg-hover); }
  nav button.on { background: var(--bg-session-active); color: var(--accent); font-weight: 600; }
  .content { flex: 1; display: flex; flex-direction: column; gap: 14px; min-width: 0; }
  .bgrow { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .thumb { width: 72px; height: 44px; object-fit: cover; border-radius: 6px; border: 1px solid var(--border-light); }
  .switchrow { display: flex; align-items: center; gap: 10px; font-size: 13px; }
</style>