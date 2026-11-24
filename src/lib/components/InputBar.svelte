<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { app } from '$lib/stores/app.svelte';
  import EmojiPanel from './EmojiPanel.svelte';

  let { session }: { session: string } = $props();

  let value = $state('');
  let area: HTMLTextAreaElement | undefined = $state();
  let showEmoji = $state(false);
  let completion = $state<{ start: number; at: boolean; base: string; candidates: string[]; index: number } | null>(null);

  function insertAtCaret(text: string) {
    if (!area) return;
    const start = area.selectionStart ?? value.length;
    const end = area.selectionEnd ?? start;
    value = value.slice(0, start) + text + value.slice(end);
    queueMicrotask(() => {
      area?.focus();
      const pos = start + text.length;
      area?.setSelectionRange(pos, pos);
    });
  }

  function send() {
    const text = value;
    if (!text.trim()) return;
    app.sendInput(text);
    value = '';
    completion = null;
    showEmoji = false;
  }

  function complete() {
    if (!area) return;
    const caret = area.selectionStart ?? 0;
    if (caret !== (area.selectionEnd ?? 0)) return;
    const before = value.slice(0, caret);
    const m = /(@?)([A-Za-z0-9_[\]\\^{}|`-]*)$/.exec(before);
    if (!m) return;
    const at = m[1] === '@';
    const word = m[2];
    const start = caret - m[0].length;
    if (completion && completion.start === start && completion.base === word) {
      completion.index = (completion.index + 1) % completion.candidates.length;
    } else {
      if (!word) return;
      const nicks = (app.session(session)?.members ?? []).map((x) => x.nick);
      const candidates = nicks.filter((n) => n.toLowerCase().startsWith(word.toLowerCase()));
      if (!candidates.length) return;
      completion = { start, at, base: word, candidates, index: 0 };
    }
    const nick = completion.candidates[completion.index];
    const insert = at ? `@${nick} ` : start === 0 ? `${nick}: ` : `${nick} `;
    value = value.slice(0, start) + insert + value.slice(caret);
    queueMicrotask(() => {
      const pos = start + insert.length;
      area?.setSelectionRange(pos, pos);
    });
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) {
      e.preventDefault();
      send();
      return;
    }
    if (e.key === 'Tab') {
      e.preventDefault();
      complete();
      return;
    }
    if (e.key === 'Escape') showEmoji = false;
    if (completion && e.key !== 'Tab') completion = null;
  }
</script>

<div class="inputbar">
  {#if showEmoji}
    <div class="emojiwrap">
      <EmojiPanel onpick={(e) => insertAtCaret(e)} />
    </div>
  {/if}
  <div class="row">
    <button class="emoji" title={$_('input.emoji')} onclick={() => (showEmoji = !showEmoji)}>
      <svg width="18" height="18" viewBox="0 0 18 18"><circle cx="9" cy="9" r="7.4" fill="none" stroke="currentColor" stroke-width="1.4"/><circle cx="6.4" cy="7.2" r="1" fill="currentColor"/><circle cx="11.6" cy="7.2" r="1" fill="currentColor"/><path d="M5.6 11c.9 1.3 2.1 2 3.4 2s2.5-.7 3.4-2" fill="none" stroke="currentColor" stroke-width="1.3"/></svg>
    </button>
    <textarea
      bind:this={area}
      bind:value
      rows="2"
      placeholder={$_('input.placeholder')}
      onkeydown={onKey}
      oninput={() => (completion = null)}
    ></textarea>
    <button class="btn primary" disabled={!app.connected || !value.trim()} onclick={send}>
      {$_('input.send')}
    </button>
  </div>
</div>

<style>
  .inputbar {
    flex: none; position: relative;
    border-top: 1px solid var(--border-light);
    background: var(--bg-panel); padding: 8px 12px 10px;
  }
  .emojiwrap { position: absolute; bottom: 100%; left: 12px; margin-bottom: 6px; z-index: 30; }
  .row { display: flex; gap: 8px; align-items: flex-end; }
  .emoji {
    width: 32px; height: 32px; flex: none; display: grid; place-items: center;
    border: none; border-radius: 6px; background: transparent;
    color: var(--text-secondary); cursor: pointer;
  }
  .emoji:hover { background: var(--bg-hover); color: var(--accent); }
  textarea {
    flex: 1; resize: none; max-height: 132px;
    border: 1px solid var(--border-light); border-radius: 5px;
    background: var(--bg-input); color: var(--text-primary);
    padding: 7px 10px; font: inherit; font-size: 13px; line-height: 1.5;
    outline: none;
  }
  textarea:focus { border-color: var(--accent); }
  .btn { flex: none; }
</style>