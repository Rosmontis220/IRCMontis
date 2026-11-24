<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { app } from '$lib/stores/app.svelte';

  let secs = $state(0);
  $effect(() => {
    if (app.reconnectIn != null) {
      secs = app.reconnectIn;
      const id = setInterval(() => (secs = Math.max(0, secs - 1)), 1000);
      return () => clearInterval(id);
    }
  });

  function win() {
    try {
      return getCurrentWindow();
    } catch {
      return null;
    }
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <img class="logo" src="/icon.svg" alt="" draggable="false" />
  <span class="name" data-tauri-drag-region>IRCMontis</span>
  <span class="status" data-tauri-drag-region>
    {#if app.connecting}
      {$_('connect.connecting')}
    {:else if app.connected}
      {$_('status.connected')} · {app.nick}
    {:else if app.reconnectIn != null}
      {$_('status.reconnecting', { values: { secs } })}
    {:else}
      {$_('status.offline')}
    {/if}
  </span>
  <div class="buttons">
    <button title={$_('settings.title')} onclick={() => (app.showSettings = true)}>
      <svg width="14" height="14" viewBox="0 0 14 14"><circle cx="7" cy="7" r="2.2" fill="none" stroke="currentColor" stroke-width="1.3"/><path d="M7 1.2v1.6M7 11.2v1.6M1.2 7h1.6M11.2 7h1.6M2.9 2.9l1.1 1.1M10 10l1.1 1.1M11.1 2.9L10 4M4 10l-1.1 1.1" stroke="currentColor" stroke-width="1.3"/></svg>
    </button>
    <button title={$_('titlebar.minimize')} onclick={() => win()?.minimize()}>
      <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1 6h10" stroke="currentColor" stroke-width="1.2"/></svg>
    </button>
    <button title={$_('titlebar.maximize')} onclick={() => win()?.toggleMaximize()}>
      <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
    </button>
    <button class="close" title={$_('titlebar.close')} onclick={() => win()?.close()}>
      <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
    </button>
  </div>
</div>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 8px 0 12px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-light);
    flex: none;
  }
  .logo { width: 20px; height: 20px; }
  .name { font-weight: 600; font-size: 13px; }
  .status { font-size: 12px; color: var(--text-secondary); margin-left: 8px; }
  .buttons { margin-left: auto; display: flex; gap: 2px; }
  .buttons button {
    width: 36px; height: 30px;
    display: grid; place-items: center;
    background: transparent; border: none; border-radius: 6px;
    color: var(--text-primary); cursor: pointer;
  }
  .buttons button:hover { background: var(--bg-session-active); }
  .buttons button.close:hover { background: var(--danger); color: #fff; }
</style>