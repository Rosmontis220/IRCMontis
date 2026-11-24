<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { app, type Session } from '$lib/stores/app.svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { api } from '$lib/irc/api';
  import type { StoredMessage } from '$lib/irc/types';
  import MessageBubble from './MessageBubble.svelte';
  import MemberList from './MemberList.svelte';
  import InputBar from './InputBar.svelte';

  let { session }: { session: Session } = $props();

  let scroller: HTMLDivElement | undefined = $state();
  let showMembers = $state(false);
  let showSearch = $state(false);
  let query = $state('');
  let results = $state<StoredMessage[]>([]);
  let searching = $state(false);
  let highlightId = $state<number | null>(null);
  let nearBottom = $state(true);

  const msgs = $derived(app.messages[session.name] ?? []);

  $effect(() => {
    session.name;
    showSearch = false;
    query = '';
    results = [];
    highlightId = null;
    nearBottom = true;
    queueMicrotask(() => scroller?.scrollTo({ top: scroller.scrollHeight }));
  });

  $effect(() => {
    msgs.length;
    if (nearBottom) queueMicrotask(() => scroller?.scrollTo({ top: scroller.scrollHeight }));
  });

  function onScroll() {
    if (!scroller) return;
    nearBottom = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 60;
    if (scroller.scrollTop < 40 && app.loaded[session.name] && !showSearch) {
      const before = scroller.scrollHeight;
      void app.loadOlder(session.name).then(() => {
        if (scroller) scroller.scrollTop += scroller.scrollHeight - before;
      });
    }
  }

  async function runSearch() {
    if (!query.trim()) {
      results = [];
      return;
    }
    searching = true;
    try {
      results = await api.search(session.name, query.trim(), 100);
    } finally {
      searching = false;
    }
  }

  async function jump(m: StoredMessage) {
    const rows = await app.jumpTo(session.name, m.ts);
    highlightId = m.id;
    showSearch = false;
    queueMicrotask(() => {
      const el = scroller?.querySelector(`[data-mid="${m.id}"]`);
      el?.scrollIntoView({ block: 'center' });
    });
    void rows;
  }

  function dayLabel(ts: number): string {
    return new Date(ts).toLocaleDateString([], { year: 'numeric', month: 'long', day: 'numeric' });
  }

  function mentioned(text: string): boolean {
    return !!app.nick && text.toLowerCase().includes(app.nick.toLowerCase());
  }
</script>

<div class="chat">
  <div class="head">
    <div class="titles">
      <span class="name">{session.name}</span>
      {#if session.topic}<span class="topic" title={session.topic}>{session.topic}</span>{/if}
    </div>
    <div class="actions">
      <button class:on={showSearch} title={$_('chat.search')} onclick={() => (showSearch = !showSearch)}>
        <svg width="15" height="15" viewBox="0 0 15 15"><circle cx="6.2" cy="6.2" r="4.4" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M9.6 9.6L13.5 13.5" stroke="currentColor" stroke-width="1.5"/></svg>
      </button>
      {#if session.kind === 'channel'}
        <button class:on={showMembers} title={$_('chat.members')} onclick={() => (showMembers = !showMembers)}>
          <svg width="15" height="15" viewBox="0 0 15 15"><circle cx="5.4" cy="4.6" r="2.5" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M1.2 12.6c0-2.4 1.9-4 4.2-4s4.2 1.6 4.2 4" fill="none" stroke="currentColor" stroke-width="1.4"/><circle cx="10.8" cy="5" r="2" fill="none" stroke="currentColor" stroke-width="1.2"/><path d="M10.6 8.8c1.9.2 3.2 1.6 3.2 3.6" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
        </button>
      {/if}
      <button class:on={session.muted} title={session.muted ? $_('sessionMenu.unmute') : $_('sessionMenu.mute')} onclick={() => app.setMuted(session.name, !session.muted)}>
        <svg width="15" height="15" viewBox="0 0 15 15"><path d="M1.5 5.5h3l3.5-3v10l-3.5-3h-3z" fill="currentColor"/>{#if session.muted}<path d="M9.5 5l4 5M13.5 5l-4 5" stroke="currentColor" stroke-width="1.4"/>{:else}<path d="M10 5.2c1.4 1.3 1.4 3.3 0 4.6M11.8 3.6c2.3 2.2 2.3 5.6 0 7.8" fill="none" stroke="currentColor" stroke-width="1.3"/>{/if}</svg>
      </button>
    </div>
  </div>

  {#if showSearch}
    <div class="searchbar">
      <input
        placeholder={$_('chat.searchPlaceholder')}
        bind:value={query}
        oninput={runSearch}
        onkeydown={(e) => e.key === 'Escape' && (showSearch = false)}
      />
      <span class="count">{searching ? '…' : results.length}</span>
      <button title={$_('chat.close')} onclick={() => (showSearch = false)}>✕</button>
    </div>
    <div class="results">
      {#each results as m (m.id)}
        <button onclick={() => jump(m)}>
          <span class="rtime">{new Date(m.ts).toLocaleString()}</span>
          <span class="rsender">{m.sender}</span>
          <span class="rtext">{m.text}</span>
        </button>
      {:else}
        <div class="noresult">{$_('chat.noResult')}</div>
      {/each}
    </div>
  {/if}

  <div class="body">
    {#if app.settings.backgroundPath}
      <div
        class="bg"
        style="background-image:url('{convertFileSrc(app.settings.backgroundPath)}'); opacity:{app.settings.backgroundOpacity / 100}"
      ></div>
    {/if}
    <div class="scroller" bind:this={scroller} onscroll={onScroll}>
      {#if !app.loaded[session.name]}
        <div class="loading">{$_('chat.loading')}</div>
      {/if}
      {#each msgs as m, i (m.id ?? `${m.ts}-${i}`)}
        {#if i === 0 || dayLabel(m.ts) !== dayLabel(msgs[i - 1].ts)}
          <div class="day">{dayLabel(m.ts)}</div>
        {/if}
        <div data-mid={m.id} class:highlight={highlightId === m.id}>
          <MessageBubble msg={m} self={m.sender === app.nick && m.kind !== 'system'} mentioned={mentioned(m.text)} />
        </div>
      {/each}
    </div>
    {#if showMembers && session.kind === 'channel'}
      <MemberList session={session.name} />
    {/if}
  </div>

  <InputBar session={session.name} />
</div>

<style>
  .chat { flex: 1; min-width: 0; display: flex; flex-direction: column; background: var(--bg-chat); }
  .head {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 16px; border-bottom: 1px solid var(--border-light);
    background: var(--bg-panel); flex: none;
  }
  .titles { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .name { font-size: 14px; font-weight: 600; }
  .topic {
    font-size: 11px; color: var(--text-secondary);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 560px;
  }
  .actions { margin-left: auto; display: flex; gap: 4px; }
  .actions button {
    width: 30px; height: 30px; display: grid; place-items: center;
    border: none; border-radius: 6px; background: transparent;
    color: var(--text-secondary); cursor: pointer;
  }
  .actions button:hover { background: var(--bg-session-active); color: var(--qq-blue); }
  .actions button.on { color: var(--qq-blue); background: var(--bg-session-active); }
  .searchbar {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 16px; background: var(--bg-panel);
    border-bottom: 1px solid var(--border-light); flex: none;
  }
  .searchbar input {
    flex: 1; border: 1px solid var(--border-light); border-radius: 6px;
    background: var(--bg-chat); color: var(--text-primary);
    padding: 6px 10px; font-size: 12px; outline: none;
  }
  .searchbar input:focus { border-color: var(--qq-blue); }
  .count { font-size: 11px; color: var(--text-secondary); }
  .searchbar button { border: none; background: transparent; color: var(--text-secondary); cursor: pointer; }
  .results {
    max-height: 220px; overflow-y: auto; flex: none;
    background: var(--bg-panel); border-bottom: 1px solid var(--border-light);
    padding: 4px 8px;
  }
  .results button {
    display: flex; gap: 8px; width: 100%; text-align: left;
    border: none; background: transparent; border-radius: 6px;
    padding: 6px 8px; cursor: pointer; color: var(--text-primary); font-size: 12px;
  }
  .results button:hover { background: var(--bg-session-active); }
  .rtime { color: var(--text-secondary); flex: none; font-size: 11px; }
  .rsender { color: var(--qq-blue); flex: none; max-width: 120px; overflow: hidden; text-overflow: ellipsis; }
  .rtext { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .noresult { padding: 12px; text-align: center; font-size: 12px; color: var(--text-secondary); }
  .body { flex: 1; min-height: 0; display: flex; position: relative; }
  .bg {
    position: absolute; inset: 0;
    background-size: cover; background-position: center;
    pointer-events: none;
  }
  .scroller { flex: 1; overflow-y: auto; padding: 8px 0 12px; position: relative; }
  .loading { text-align: center; font-size: 12px; color: var(--text-secondary); padding: 12px; }
  .day {
    text-align: center; font-size: 11px; color: var(--text-secondary);
    margin: 10px 0 4px;
  }
  .highlight > :global(*) { animation: flash 1.6s ease-out; }
  @keyframes flash {
    0% { background: var(--mention-bg); }
    100% { background: transparent; }
  }
</style>