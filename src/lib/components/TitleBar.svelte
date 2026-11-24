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
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.313-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z"/><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/></svg>
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