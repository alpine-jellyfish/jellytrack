<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import {
    getLastUsedEntry,
    getPreference,
    getRunningEntry,
    listProjects,
    listTasks,
    startEntry,
    stopEntry,
    updateEntry,
    type Entry,
    type Project,
    type Task,
  } from "./db";
  import { formatDuration, formatTime } from "./format";
  import { timeFormat } from "./timeFormat.svelte";
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
  let noteDraft = $state("");
  let now = $state(Date.now());
  let tick: ReturnType<typeof setInterval> | null = null;
  let version = $state("");

  async function refresh() {
    projects = await listProjects();
    running = await getRunningEntry();
    if (running) {
      selectedProjectId = running.project_id;
      selectedTaskId = running.task_id;
      noteDraft = running.note ?? "";
    } else {
      const useLast = (await getPreference("use_last_used_default")) === "1";
      let projId: number | null = null;
      let taskId: number | null = null;

      const isActiveProject = (id: number | null): boolean =>
        id != null && projects.some((p) => p.id === id && !p.archived_at);

      async function validTaskFor(
        proj: number | null,
        task: number | null,
      ): Promise<number | null> {
        if (proj == null || task == null) return null;
        const activeTasks = await listTasks(proj);
        return activeTasks.some((t) => t.id === task) ? task : null;
      }

      if (useLast) {
        const last = await getLastUsedEntry();
        if (last && isActiveProject(last.project_id)) {
          projId = last.project_id;
          taskId = await validTaskFor(projId, last.task_id);
        }
      }
      if (projId == null) {
        const defProj = await getPreference("default_project_id");
        const defTask = await getPreference("default_task_id");
        const defProjId = defProj ? Number(defProj) : null;
        const defTaskId = defTask ? Number(defTask) : null;
        if (isActiveProject(defProjId)) {
          projId = defProjId;
          taskId = await validTaskFor(projId, defTaskId);
        }
      }
      selectedProjectId = projId ?? (projects[0]?.id ?? null);
      selectedTaskId = taskId;
      noteDraft = "";
    }
    tasks = await listTasks(selectedProjectId);
  }

  async function saveNote() {
    if (!running) return;
    const next = noteDraft.trim() ? noteDraft : null;
    if ((running.note ?? null) === next) return;
    await updateEntry(running.id, { note: next });
    running = { ...running, note: next };
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
    getVersion().then((v) => (version = v));
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
          >Since {formatTime(running.start_time, timeFormat.use24h)} on {currentProjectName}{currentTaskName
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

  {#if running}
    <div class="field">
      <span class="label">Notes</span>
      <textarea
        class="notes"
        rows="3"
        placeholder="What are you working on?"
        bind:value={noteDraft}
        onblur={saveNote}
      ></textarea>
    </div>
  {/if}

  <button class="cta" class:stop={!!running} onclick={toggle}>
    {running ? "Stop" : "Start"}
  </button>

  {#if version}
    <div class="version">jellytrack v{version}</div>
  {/if}
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

  .notes {
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 10px 12px;
    font-size: 13px;
    font-family: inherit;
    resize: vertical;
    min-height: 72px;
    transition: border-color 0.14s ease;
  }
  .notes:hover {
    border-color: var(--line-strong);
  }

  .version {
    text-align: center;
    font-size: 10px;
    color: var(--ink-muted);
    opacity: 0.55;
    letter-spacing: 0.02em;
  }
</style>
