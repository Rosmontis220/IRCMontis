<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { app } from '$lib/stores/app.svelte';
  import { identiconUrl } from '$lib/identicon';
  import type { StoredMessage } from '$lib/irc/types';

  let {
    session,
    query,
    results,
    searching,
    muted,
    onSearch,
    onJump,
    onMute
  }: {
    session: string;
    query: string;
    results: StoredMessage[];
    searching: boolean;
    muted: boolean;
    onSearch: (v: string) => void;
    onJump: (m: StoredMessage) => void;
    onMute: () => void;
  } = $props();

  const rank = (p: string) => (p === '~' || p === '&' ? 0 : p === '@' ? 1 : p === '%' ? 2 : p === '+' ? 3 : 4);

  const groups = $derived.by(() => {
    const members = [...(app.session(session)?.members ?? [])].sort(
      (a, b) => rank(a.prefix) - rank(b.prefix) || a.nick.localeCompare(b.nick)
    );
    return [
      { label: $_('members.owners'), items: members.filter((m) => rank(m.prefix) === 0) },
      { label: $_('members.admins'), items: members.filter((m) => rank(m.prefix) === 1) },
      { label: $_('members.halfops'), items: members.filter((m) => rank(m.prefix) === 2) },
      { label: $_('members.voices'), items: members.filter((m) => rank(m.prefix) === 3) },
      { label: $_('members.members'), items: members.filter((m) => rank(m.prefix) === 4) }
    ].filter((g) => g.items.length > 0);
  });

  const total = $derived(app.session(session)?.members.length ?? 0);
</script>

<div class="members">
  <div class="tools">
    <input
      class="search"
      placeholder={$_('chat.searchPlaceholder')}
      value={query}
      oninput={(e) => onSearch(e.currentTarget.value)}
    />
    <div class="mute-row">
      <span class="mute-label">{$_('sessions.muted')}</span>
      <button
        class="switch"
        class:on={muted}
        type="button"
        role="switch"
        aria-checked={muted}
        title={muted ? $_('sessionMenu.unmute') : $_('sessionMenu.mute')}
        onclick={onMute}
      ><span class="knob"></span></button>
    </div>
    {#if query.trim()}
      <div class="meta">{searching ? '…' : results.length}</div>
      <div class="results">
        {#each results as m (m.id)}
          <button onclick={() => onJump(m)}>
            <span class="rtime">{new Date(m.ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
            <span class="rtext">{m.sender}: {m.text}</span>
          </button>
        {:else}
          <div class="noresult">{$_('chat.searchEmpty')}</div>
        {/each}
      </div>
    {/if}
  </div>
  <div class="head">{$_('members.count', { values: { n: total } })}</div>
  <div class="list">
    {#each groups as g (g.label)}
      <div class="group">{g.label} · {g.items.length}</div>
      {#each g.items as m (m.nick)}
        <button class="item" onclick={() => app.openSession(m.nick, 'pm')}>
          <img src={identiconUrl(m.nick)} alt="" draggable="false" />
          <span class="nick">{m.prefix}{m.nick}</span>
        </button>
      {/each}
    {/each}
  </div>
</div>

<style>
  .members {
    width: 210px; flex: none; display: flex; flex-direction: column;
    border-left: 1px solid var(--border-light); background: var(--bg-panel);
    min-height: 0;
  }
  .tools {
    flex: none; display: flex; flex-direction: column; gap: 8px;
    padding: 10px 12px; border-bottom: 1px solid var(--border-light);
  }
  .search {
    width: 100%; background: var(--bg-input); border: 1px solid var(--border-light);
    border-radius: 5px; color: var(--text-primary); font-size: 12px;
    padding: 7px 10px; outline: none; font-family: inherit;
  }
  .search:focus { border-color: var(--accent); }
  .search::placeholder { color: var(--text-3); }
  .mute-row { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .mute-label { font-size: 12px; color: var(--text-secondary); }
  .meta { font-size: 10px; color: var(--text-3); }
  .results { max-height: 180px; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; }
  .results button {
    display: flex; gap: 6px; width: 100%; text-align: left;
    border: none; background: transparent; border-radius: 5px;
    padding: 5px 6px; cursor: pointer; color: var(--text-primary); font-size: 11px;
  }
  .results button:hover { background: var(--bg-hover); }
  .rtime { color: var(--text-3); flex: none; }
  .rtext { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .noresult { padding: 8px; text-align: center; font-size: 11px; color: var(--text-3); }
  .head { padding: 10px 12px; font-size: 12px; color: var(--text-secondary); border-bottom: 1px solid var(--border-2); }
  .list { overflow-y: auto; flex: 1; padding: 6px; min-height: 0; }
  .group { font-size: 11px; color: var(--text-3); padding: 8px 6px 4px; }
  .item {
    display: flex; align-items: center; gap: 8px; width: 100%;
    border: none; background: transparent; border-radius: 6px;
    padding: 5px 6px; cursor: pointer; color: var(--text-primary); text-align: left;
  }
  .item:hover { background: var(--bg-hover); }
  .item img { width: 24px; height: 24px; border-radius: 5px; }
  .nick { font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>