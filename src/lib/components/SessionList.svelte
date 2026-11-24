<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { app, type Session } from '$lib/stores/app.svelte';
  import { plainText } from '$lib/format';
  import { channelAvatarUrl, identiconUrl } from '$lib/identicon';

  let menu = $state<{ name: string; x: number; y: number } | null>(null);
  const menuSession = $derived(menu ? app.session(menu.name) : undefined);

  let collapsed = $state<{ channel: boolean; pm: boolean }>({ channel: false, pm: false });
  const groups = $derived(
    [
      { kind: 'channel' as const, label: $_('sessions.channels'), items: app.sessions.filter((s) => s.kind === 'channel') },
      { kind: 'pm' as const, label: $_('sessions.direct'), items: app.sessions.filter((s) => s.kind === 'pm') }
    ].filter((g) => g.items.length > 0)
  );

  function preview(s: Session): string {
    const arr = app.messages[s.name];
    const last = arr?.[arr.length - 1];
    if (!last) return s.kind === 'channel' ? s.topic : '';
    if (last.kind === 'system') return last.text;
    return `${last.sender}: ${plainText(last.text)}`;
  }

  function timeOf(ts: number): string {
    if (!ts) return '';
    const d = new Date(ts);
    const now = new Date();
    if (d.toDateString() === now.toDateString()) {
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }
    return d.toLocaleDateString([], { month: '2-digit', day: '2-digit' });
  }

  function openMenu(e: MouseEvent, s: Session) {
    e.preventDefault();
    menu = { name: s.name, x: e.clientX, y: e.clientY };
  }

  function closeMenu() {
    menu = null;
  }
</script>

<svelte:window onclick={closeMenu} />

<div class="sessions">
  <div class="head">
    <span>{$_('sessions.title')}</span>
    <button class="join" title={$_('sessions.join')} onclick={() => (app.showJoin = true)}>
      <svg width="14" height="14" viewBox="0 0 14 14"><path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.6"/></svg>
    </button>
  </div>
  <div class="list">
    {#each groups as g (g.kind)}
      <button class="group" class:collapsed={collapsed[g.kind]} onclick={() => (collapsed[g.kind] = !collapsed[g.kind])}>
        <span class="chev">▾</span>
        <span>{g.label}</span>
        <span class="gcount">{g.items.length}</span>
      </button>
      {#if !collapsed[g.kind]}
        {#each g.items as s (s.name)}
          <button
            class="item"
            class:active={app.active === s.name}
            onclick={() => app.openSession(s.name, s.kind)}
            oncontextmenu={(e) => openMenu(e, s)}
          >
            <img class="avatar" src={s.kind === 'channel' ? channelAvatarUrl(s.name) : identiconUrl(s.name)} alt="" draggable="false" />
            <div class="mid">
              <div class="row1">
                <span class="name">{s.name}</span>
                <span class="time">{timeOf(s.lastTs)}</span>
              </div>
              <div class="row2">
                <span class="preview">{preview(s)}</span>
                {#if s.muted}
                  <span class="mute" title={$_('sessions.muted')}>
                    <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1 5h3l3-3v8l-3-3H1z" fill="currentColor"/><path d="M8.5 4l3 4M11.5 4l-3 4" stroke="currentColor" stroke-width="1"/></svg>
                  </span>
                {/if}
                {#if s.unread > 0}
                  <span class="badge" class:mention={s.mention}>{s.mention ? '@' : ''}{s.unread > 99 ? '99+' : s.unread}</span>
                {/if}
              </div>
            </div>
          </button>
        {/each}
      {/if}
    {:else}
      <div class="empty">{$_('sessions.empty')}</div>
    {/each}
  </div>
</div>

{#if menu}
  <div class="menu" style="left:{menu.x}px; top:{menu.y}px" role="menu">
    {#if menuSession}
      <button onclick={() => { app.setMuted(menuSession.name, !menuSession.muted); closeMenu(); }}>
        {menuSession.muted ? $_('sessionMenu.unmute') : $_('sessionMenu.mute')}
      </button>
      {#if menuSession.kind === 'channel'}
        <button onclick={() => { app.leaveChannel(menuSession.name); closeMenu(); }}>{$_('sessionMenu.leave')}</button>
      {/if}
      <button class="danger" onclick={() => { app.deleteSession(menuSession.name); closeMenu(); }}>{$_('sessionMenu.delete')}</button>
    {/if}
  </div>
{/if}

<style>
  .sessions {
    width: 260px;
    flex: none;
    display: flex;
    flex-direction: column;
    background: var(--bg-session);
    border-right: 1px solid var(--border-light);
  }
  .head {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 12px; font-size: 13px; font-weight: 600;
  }
  .join {
    width: 26px; height: 26px; display: grid; place-items: center;
    border: none; border-radius: 6px; background: transparent;
    color: var(--text-secondary); cursor: pointer;
  }
  .join:hover { background: var(--bg-hover); color: var(--accent); }
  .list { overflow-y: auto; flex: 1; padding: 0 6px 6px; }
  .group {
    display: flex; align-items: center; gap: 6px; width: 100%;
    border: none; background: transparent; cursor: pointer;
    font-size: 11px; color: var(--text-3); padding: 10px 6px 4px;
    text-align: left; font-family: inherit;
  }
  .group:hover { color: var(--text-secondary); }
  .group .chev { display: inline-block; font-size: 9px; transition: transform 0.15s; }
  .group.collapsed .chev { transform: rotate(-90deg); }
  .group .gcount { margin-left: auto; color: var(--text-3); }
  .item {
    display: flex; gap: 10px; width: 100%;
    padding: 8px; border: none; border-radius: var(--r);
    background: transparent; cursor: pointer; text-align: left;
    color: var(--text-primary);
  }
  .item:hover { background: var(--bg-hover); }
  .item.active { background: var(--bg-session-active); }
  .avatar { width: 34px; height: 34px; border-radius: var(--r); flex: none; }
  .mid { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
  .row1 { display: flex; align-items: baseline; gap: 6px; }
  .name { font-size: 13px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .time { margin-left: auto; font-size: 11px; color: var(--text-3); flex: none; }
  .row2 { display: flex; align-items: center; gap: 4px; }
  .preview {
    flex: 1; min-width: 0; font-size: 12px; color: var(--text-secondary);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .mute { color: var(--text-3); flex: none; display: inline-flex; }
  .badge {
    flex: none; min-width: 18px; height: 18px; padding: 0 5px;
    border-radius: 9px; background: var(--accent); color: #fff;
    font-size: 11px; display: grid; place-items: center;
  }
  .badge.mention { background: var(--accent-strong); }
  .empty { padding: 24px 12px; font-size: 12px; color: var(--text-secondary); text-align: center; }
  .menu {
    position: fixed; z-index: 60; min-width: 150px;
    background: var(--bg-panel); border: 1px solid var(--border-light);
    border-radius: var(--r); box-shadow: var(--shadow-card); padding: 4px;
    display: flex; flex-direction: column;
  }
  .menu button {
    border: none; background: transparent; text-align: left;
    padding: 7px 10px; border-radius: 6px; font-size: 12px;
    color: var(--text-primary); cursor: pointer;
  }
  .menu button:hover { background: var(--bg-hover); }
  .menu button.danger { color: var(--danger); }
</style>