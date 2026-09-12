<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import {
    getPreference,
    getRunningEntry,
    listProjects,
    listTasks,
    startEntry,
    stopEntry,
    type Entry,
    type Project,
    type Task,
  } from "./db";
  import { formatDuration, formatTime } from "./format";
  import Select from "./Select.svelte";

  let {
    onChanged,
    reloadKey = 0,
  }: { onChanged?: () => void; reloadKey?: number } = $props();

  let projects: Project[] = $state([]);
  let tasks: Task[] = $state([]);
  let selectedProjectId: number | null = $state(null);
  let selectedTaskId: number | null = $state(null);
  let running = $state<Entry | null>(null);
  let now = $state(Date.now());
  let tick: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    projects = await listProjects();
    running = await getRunningEntry();
    if (running) {
      selectedProjectId = running.project_id;
      selectedTaskId = running.task_id;
    } else {
      const defProj = await getPreference("default_project_id");
      const defTask = await getPreference("default_task_id");
      selectedProjectId = defProj ? Number(defProj) : (projects[0]?.id ?? null);
      selectedTaskId = defTask ? Number(defTask) : null;
    }
    tasks = await listTasks(selectedProjectId);
  }

  async function onProjectChange() {
    tasks = await listTasks(selectedProjectId);
    if (!tasks.some((t) => t.id === selectedTaskId)) {
      selectedTaskId = tasks[0]?.id ?? null;
    }
  }

  async function toggle() {
    if (running) {
      await stopEntry(running.id);
    } else {
      await startEntry(selectedProjectId, selectedTaskId);
    }
    await refresh();
    onChanged?.();
  }

  onMount(() => {
    refresh();
    tick = setInterval(() => (now = Date.now()), 1000);
  });

  $effect(() => {
    reloadKey;
    refresh();
  });

  onDestroy(() => {
    if (tick) clearInterval(tick);
  });

  const elapsedMs = $derived.by(() => {
    const r = running;
    return r ? now - new Date(r.start_time).getTime() : 0;
  });

  const currentProjectName = $derived(
    projects.find((p) => p.id === selectedProjectId)?.name ?? "—",
  );
  const currentTaskName = $derived(
    tasks.find((t) => t.id === selectedTaskId)?.name ?? null,
  );
</script>

<div class="tracker">
  <section class="hero" class:running={!!running}>
    <div class="clock" aria-live="polite">
      {formatDuration(elapsedMs)}
    </div>
    <div class="status">
      {#if running}
        <span class="dot"></span>
        <span
          >Since {formatTime(running.start_time)} on {currentProjectName}{currentTaskName
            ? ` / ${currentTaskName}`
            : ""}</span
        >
      {:else}
        <span>Idle</span>
      {/if}
    </div>
  </section>

  <div class="fields">
    <div class="field">
      <span class="label">Project</span>
      <Select
        bind:value={selectedProjectId}
        options={projects.map((p) => ({ value: p.id, label: p.name }))}
        onchange={onProjectChange}
        disabled={!!running}
        placeholder="No project"
      />
    </div>

    <div class="field">
      <span class="label">Task</span>
      <Select
        bind:value={selectedTaskId}
        options={[
          { value: null, label: "No task" },
          ...tasks.map((t) => ({ value: t.id, label: t.name })),
        ]}
        disabled={!!running}
        placeholder="No task"
      />
    </div>
  </div>

  <button class="cta" class:stop={!!running} onclick={toggle}>
    {running ? "Stop" : "Start"}
  </button>
</div>

<style>
  .tracker {
    display: flex;
    flex-direction: column;
    gap: var(--s-5);
    padding-top: var(--s-2);
  }

  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: var(--s-5) 0 var(--s-4);
  }

  .clock {
    font-family: var(--font-mono);
    font-size: 46px;
    font-weight: 400;
    letter-spacing: -0.02em;
    color: var(--ink-subtle);
    font-variant-numeric: tabular-nums;
    line-height: 1;
    transition: color 0.2s ease;
  }

  .hero.running .clock {
    color: var(--accent);
  }

  .status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--ink-muted);
    letter-spacing: -0.005em;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--r-pill);
    background: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
    animation: pulse 1.6s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  @media (prefers-reduced-motion: reduce) {
    .dot { animation: none; }
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label {
    font-size: 12px;
    color: var(--ink-muted);
    font-weight: 500;
    letter-spacing: -0.005em;
    padding-left: 2px;
  }

  .cta {
    background: var(--accent);
    color: var(--accent-ink);
    border: none;
    border-radius: var(--r-pill);
    padding: 14px;
    font-size: 15px;
    font-weight: 600;
    letter-spacing: -0.01em;
    cursor: pointer;
    margin-top: var(--s-2);
    transition:
      background 0.14s ease,
      transform 0.06s ease;
  }

  .cta:hover {
    background: var(--accent-hover);
  }

  .cta:active {
    transform: scale(0.985);
  }

  .cta.stop {
    background: var(--danger);
  }

  .cta.stop:hover {
    background: #b91c1c;
  }
</style>
