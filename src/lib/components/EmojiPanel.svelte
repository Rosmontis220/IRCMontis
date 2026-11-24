<script lang="ts">
  let { onpick }: { onpick: (emoji: string) => void } = $props();

  const EMOJI =
    '😀 😃 😄 😁 😆 😅 😂 🤣 😊 😇 🙂 😉 😌 😍 🥰 😘 😋 😛 😜 🤪 🤔 🤨 😐 😑 😶 🙄 😏 😣 😥 😮 🤐 😯 😪 😫 🥱 😴 😒 😓 😔 😕 🙃 🤑 😲 🙁 😖 😞 😟 😤 😢 😭 😦 😧 😨 😩 🤯 😬 😰 😱 🥵 🥶 😳 😵 🥴 😠 😡 🤬 😷 🤒 🤕 🤢 🤮 🥳 🥺 🤠 😎 🤓 🧐 😈 👿 👻 👍 👎 👏 🙌 👌 🤝 🙏 💪 ❤️ 💔 💯 💢 💥 💫 💦 🔥 ⭐ 🌟 ✨ 🎉 🎊 🎁 🌹 🌸 🍀 🍎 🍺 ☕ 🍰 🐱'.split(' ');
</script>

<div class="panel">
  {#each EMOJI as e, i (i)}
    <button onclick={() => onpick(e)}>{e}</button>
  {/each}
</div>

<style>
  .panel {
    display: grid; grid-template-columns: repeat(10, 1fr); gap: 2px;
    padding: 8px; max-height: 180px; overflow-y: auto;
    background: var(--bg-panel); border: 1px solid var(--border-light);
    border-radius: 10px; box-shadow: var(--shadow-card);
  }
  .panel button {
    border: none; background: transparent; border-radius: 6px;
    font-size: 18px; line-height: 1; padding: 5px 0; cursor: pointer;
  }
  .panel button:hover { background: var(--bg-session-active); }
</style>