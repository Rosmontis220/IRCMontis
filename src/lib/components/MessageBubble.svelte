<script lang="ts">
  import type { ChatMessage } from '$lib/irc/types';
  import type { Segment as FmtSegment } from '$lib/format';
  import { parseFormatting, splitLinks } from '$lib/format';
  import { identiconUrl } from '$lib/identicon';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let { msg, self = false, mentioned = false }: { msg: ChatMessage; self?: boolean; mentioned?: boolean } = $props();

  function timeOf(ts: number): string {
    return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  function link(e: MouseEvent, url: string | null) {
    e.preventDefault();
    if (!url) return;
    void openUrl(url).catch(() => window.open(url, '_blank'));
  }

  function segStyle(seg: FmtSegment): string {
    const s: string[] = [];
    if (seg.bold) s.push('font-weight:700');
    if (seg.italic) s.push('font-style:italic');
    const deco: string[] = [];
    if (seg.underline) deco.push('underline');
    if (seg.strike) deco.push('line-through');
    if (deco.length) s.push(`text-decoration:${deco.join(' ')}`);
    if (seg.mono) s.push('font-family:var(--font-mono)');
    if (seg.fg) s.push(`color:${seg.fg}`);
    if (seg.bg) s.push(`background:${seg.bg}`);
    return s.join(';');
  }
</script>

{#if msg.kind === 'system' || msg.kind === 'notice'}
  <div class="systemline" class:notice={msg.kind === 'notice'}>{msg.text}</div>
{:else if msg.kind === 'action'}
  <div class="actionline">
    <span class="who">{msg.sender}</span>
    {#each parseFormatting(msg.text) as seg}
      <span style={segStyle(seg)}>{seg.text}</span>
    {/each}
  </div>
{:else}
  <div class="row" class:self>
    <img class="avatar" src={identiconUrl(msg.sender)} alt="" draggable="false" />
    <div class="col">
      {#if !self}<span class="nick">{msg.sender}</span>{/if}
      <div class="bubble" class:mentioned class:failed={msg.failed} title={timeOf(msg.ts)}>
        {#each parseFormatting(msg.text) as seg}
          {#each splitLinks(seg.text) as part}
            {#if part.href}
              <a href={part.href} onclick={(e) => link(e, part.href)} style={segStyle(seg)}>{part.text}</a>
            {:else}
              <span style={segStyle(seg)}>{part.text}</span>
            {/if}
          {/each}
        {/each}
        {#if msg.failed}<span class="failmark">!</span>{/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .systemline {
    text-align: center; font-size: 11px; color: var(--text-secondary);
    padding: 4px 16px; user-select: text;
  }
  .systemline.notice { color: var(--qq-blue); }
  .actionline {
    text-align: center; font-size: 12px; color: var(--text-secondary);
    font-style: italic; padding: 4px 16px; user-select: text;
  }
  .actionline .who { font-weight: 600; margin-right: 4px; }
  .row { display: flex; gap: 8px; padding: 4px 16px; align-items: flex-start; }
  .row.self { flex-direction: row-reverse; }
  .avatar { width: 34px; height: 34px; border-radius: 6px; flex: none; margin-top: 2px; }
  .col { display: flex; flex-direction: column; gap: 3px; max-width: 72%; min-width: 0; }
  .row.self .col { align-items: flex-end; }
  .nick { font-size: 11px; color: var(--text-secondary); padding: 0 2px; }
  .bubble {
    position: relative;
    background: var(--bg-bubble-other);
    border-radius: 10px; padding: 7px 11px;
    font-size: 13px; line-height: 1.55;
    color: var(--text-primary);
    word-break: break-word; white-space: pre-wrap;
    user-select: text;
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.06);
  }
  .row.self .bubble { background: var(--bg-bubble-self); }
  .bubble.mentioned { background: var(--mention-bg); box-shadow: inset 0 0 0 1px var(--qq-blue); }
  .bubble a { color: var(--qq-blue); text-decoration: underline; }
  .bubble.failed { opacity: 0.65; }
  .failmark {
    position: absolute; right: -18px; top: 50%; translate: 0 -50%;
    width: 14px; height: 14px; border-radius: 50%;
    background: var(--danger); color: #fff; font-size: 10px;
    display: grid; place-items: center;
  }
  .row.self .failmark { right: auto; left: -18px; }
</style>