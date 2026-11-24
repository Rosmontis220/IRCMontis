<script lang="ts">
  import '@fluentui/web-components/button.js';
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { app } from '$lib/stores/app.svelte';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import SessionList from '$lib/components/SessionList.svelte';
  import ChatView from '$lib/components/ChatView.svelte';
  import ConnectScreen from '$lib/components/ConnectScreen.svelte';
  import NickDialog from '$lib/components/NickDialog.svelte';
  import JoinDialog from '$lib/components/JoinDialog.svelte';
  import SettingsDialog from '$lib/components/SettingsDialog.svelte';

  onMount(() => {
    void app.init();
  });

  const activeSession = $derived(app.active ? app.session(app.active) : undefined);
</script>

{#if app.ready}
  <div class="shell">
    <TitleBar />
    <div class="body">
      <SessionList />
      {#if activeSession}
        <ChatView session={activeSession} />
      {:else}
        <div class="placeholder">
          <img src="/icon.svg" alt="" />
          <p>{$_('sessions.empty')}</p>
          {#if !app.connected && !app.connecting}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
            <fluent-button
              appearance="primary"
              onclick={() => (app.settings.nick ? app.connect() : (app.showConnect = true))}
            >
              {app.settings.nick ? $_('status.connectNow') : $_('connect.submit')}
            </fluent-button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{:else}
  <div class="splash">IRCMontis</div>
{/if}

{#if app.showConnect}<ConnectScreen />{/if}
{#if app.nickInUse}<NickDialog />{/if}
{#if app.showJoin}<JoinDialog />{/if}
{#if app.showSettings}<SettingsDialog />{/if}

<style>
  .shell { height: 100vh; display: flex; flex-direction: column; }
  .body { flex: 1; min-height: 0; display: flex; }
  .placeholder {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; justify-content: center; gap: 14px;
    background: var(--bg-chat);
  }
  .placeholder img { width: 84px; height: 84px; opacity: 0.85; }
  .placeholder p { margin: 0; font-size: 13px; color: var(--text-secondary); }
  .splash { height: 100vh; display: grid; place-items: center; color: var(--text-secondary); }
</style>