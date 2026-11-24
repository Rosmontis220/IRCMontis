<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { app } from '$lib/stores/app.svelte';
  import { identiconUrl } from '$lib/identicon';

  let { session }: { session: string } = $props();

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
    width: 190px; flex: none; display: flex; flex-direction: column;
    border-left: 1px solid var(--border-light); background: var(--bg-panel);
  }
  .head { padding: 10px 12px; font-size: 12px; color: var(--text-secondary); border-bottom: 1px solid var(--border-light); }
  .list { overflow-y: auto; flex: 1; padding: 6px; }
  .group { font-size: 11px; color: var(--text-secondary); padding: 8px 6px 4px; }
  .item {
    display: flex; align-items: center; gap: 8px; width: 100%;
    border: none; background: transparent; border-radius: 6px;
    padding: 5px 6px; cursor: pointer; color: var(--text-primary); text-align: left;
  }
  .item:hover { background: var(--bg-session-active); }
  .item img { width: 24px; height: 24px; border-radius: 5px; }
  .nick { font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>