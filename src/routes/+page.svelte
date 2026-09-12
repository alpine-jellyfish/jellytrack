<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import Tracker from "$lib/Tracker.svelte";
  import Entries from "$lib/Entries.svelte";
  import Projects from "$lib/Projects.svelte";
  import {
    getPreference,
    getRunningEntry,
    startEntry,
    stopEntry,
  } from "$lib/db";

  type Tab = "tracker" | "entries" | "projects";
  let tab: Tab = $state("tracker");
  let refreshKey = $state(0);

  let unlistenOpenTab: UnlistenFn | null = null;
  let unlistenToggle: UnlistenFn | null = null;

  async function syncTrayIcon() {
    const r = await getRunningEntry();
    await invoke("set_tray_running", { running: !!r });
  }

  async function handleTrayToggle() {
    const current = await getRunningEntry();
    if (current) {
      await stopEntry(current.id);
    } else {
      const defProj = await getPreference("default_project_id");
      const projId = defProj ? Number(defProj) : null;
      if (projId == null) {
        tab = "tracker";
        await invoke("show_main_window");
        return;
      }
      const defTask = await getPreference("default_task_id");
      const taskId = defTask ? Number(defTask) : null;
      await startEntry(projId, taskId);
    }
    await syncTrayIcon();
    refreshKey++;
  }

  onMount(async () => {
    await syncTrayIcon();
    unlistenOpenTab = await listen<string>("tray-open-tab", (event) => {
      const t = event.payload;
      if (t === "tracker" || t === "entries" || t === "projects") {
        tab = t;
      }
    });
    unlistenToggle = await listen("tray-toggle-running", () => {
      void handleTrayToggle();
    });
  });

  onDestroy(() => {
    unlistenOpenTab?.();
    unlistenToggle?.();
  });

  async function onTrackerChanged() {
    refreshKey++;
    await syncTrayIcon();
  }
</script>

<div class="app">
  <div class="tabs" role="tablist">
    <button
      role="tab"
      aria-selected={tab === "tracker"}
      class:active={tab === "tracker"}
      onclick={() => (tab = "tracker")}
    >
      Tracker
    </button>
    <button
      role="tab"
      aria-selected={tab === "entries"}
      class:active={tab === "entries"}
      onclick={() => (tab = "entries")}
    >
      Entries
    </button>
    <button
      role="tab"
      aria-selected={tab === "projects"}
      class:active={tab === "projects"}
      onclick={() => (tab = "projects")}
    >
      Projects
    </button>
  </div>

  <main>
    {#if tab === "tracker"}
      <Tracker onChanged={onTrackerChanged} reloadKey={refreshKey} />
    {:else if tab === "entries"}
      <Entries {refreshKey} />
    {:else}
      <Projects />
    {/if}
  </main>
</div>

<style>
  :global(:root) {
    --paper: #fbfaf7;
    --surface: #ffffff;
    --sunk: #f4f2ed;
    --line: #e7e3d8;
    --line-strong: #d6d1c4;
    --ink: #1b1b1f;
    --ink-muted: #6b6b75;
    --ink-subtle: #a8a69f;
    --accent: #7c5cff;
    --accent-hover: #6b4bff;
    --accent-soft: #f0ebff;
    --accent-ink: #ffffff;
    --danger: #dc2626;
    --danger-soft: #fdecec;
    --success: #16a34a;

    --font-ui: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui,
      "Segoe UI", sans-serif;
    --font-mono: ui-monospace, "SF Mono", "Menlo", "Cascadia Code", monospace;

    --s-1: 4px;
    --s-2: 8px;
    --s-3: 12px;
    --s-4: 16px;
    --s-5: 20px;
    --s-6: 24px;
    --s-8: 32px;

    --r-sm: 6px;
    --r-md: 10px;
    --r-lg: 14px;
    --r-pill: 999px;

    --focus-ring: 0 0 0 3px rgba(124, 92, 255, 0.28);

    font-family: var(--font-ui);
    font-size: 14px;
    line-height: 1.45;
    color: var(--ink);
    background: var(--paper);
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-rendering: optimizeLegibility;
  }

  :global(html),
  :global(body) {
    margin: 0;
    background: transparent;
    color: var(--ink);
    user-select: none;
    -webkit-user-select: none;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(input),
  :global(select),
  :global(textarea) {
    user-select: auto;
    -webkit-user-select: auto;
  }

  :global(button) {
    font-family: inherit;
  }

  :global(button:focus-visible),
  :global(select:focus-visible),
  :global(input:focus-visible) {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: var(--s-4);
    gap: var(--s-4);
    background: var(--paper);
    border-radius: 14px;
    overflow: hidden;
    border: 0.5px solid rgba(27, 27, 31, 0.08);
  }

  .tabs {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 2px;
    background: var(--sunk);
    padding: 3px;
    border-radius: var(--r-md);
  }

  .tabs button {
    background: transparent;
    color: var(--ink-muted);
    border: none;
    padding: 7px 10px;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: -0.01em;
    cursor: pointer;
    border-radius: 7px;
    transition:
      background 0.14s ease,
      color 0.14s ease;
  }

  .tabs button:hover {
    color: var(--ink);
  }

  .tabs button.active {
    background: var(--surface);
    color: var(--ink);
    box-shadow:
      0 1px 2px rgba(27, 27, 31, 0.06),
      0 0 0 0.5px rgba(27, 27, 31, 0.04);
  }

  main {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>
