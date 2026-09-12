<script lang="ts">
  import { onMount } from "svelte";
  import {
    deleteEntry,
    listProjects,
    listRecentEntries,
    listTasks,
    updateEntry,
    type Entry,
    type Project,
    type Task,
  } from "./db";
  import { formatDuration, formatTime } from "./format";
  import Select from "./Select.svelte";

  let { refreshKey = 0 }: { refreshKey?: number } = $props();

  let entries: Entry[] = $state([]);
  let projects: Project[] = $state([]);
  let tasksByProject: Record<number, Task[]> = $state({});
  let pendingDeleteId = $state<number | null>(null);
  let pendingTimer: ReturnType<typeof setTimeout> | null = null;

  async function load() {
    entries = await listRecentEntries(100);
    projects = await listProjects(true);
    const map: Record<number, Task[]> = {};
    for (const p of projects) {
      map[p.id] = await listTasks(p.id, true);
    }
    tasksByProject = map;
  }

  onMount(load);
  $effect(() => {
    refreshKey;
    load();
  });

  async function onProjectChange(e: Entry, projectId: number | null) {
    await updateEntry(e.id, { project_id: projectId, task_id: null });
    await load();
  }

  async function onTaskChange(e: Entry, taskId: number | null) {
    await updateEntry(e.id, { task_id: taskId });
    await load();
  }

  function armDelete(id: number) {
    pendingDeleteId = id;
    if (pendingTimer) clearTimeout(pendingTimer);
    pendingTimer = setTimeout(() => {
      pendingDeleteId = null;
      pendingTimer = null;
    }, 3000);
  }

  function cancelDelete() {
    pendingDeleteId = null;
    if (pendingTimer) {
      clearTimeout(pendingTimer);
      pendingTimer = null;
    }
  }

  async function confirmDelete(id: number) {
    if (pendingTimer) {
      clearTimeout(pendingTimer);
      pendingTimer = null;
    }
    pendingDeleteId = null;
    await deleteEntry(id);
    await load();
  }

  function durationMs(e: Entry): number {
    const end = e.end_time ? new Date(e.end_time).getTime() : Date.now();
    return end - new Date(e.start_time).getTime();
  }

  function dayKey(iso: string): string {
    const d = new Date(iso);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  }

  function dayLabel(key: string): string {
    const [y, m, d] = key.split("-").map(Number);
    const date = new Date(y, m - 1, d);
    const today = new Date();
    const yesterday = new Date();
    yesterday.setDate(today.getDate() - 1);
    const sameDay = (a: Date, b: Date) =>
      a.getFullYear() === b.getFullYear() &&
      a.getMonth() === b.getMonth() &&
      a.getDate() === b.getDate();
    if (sameDay(date, today)) return "Today";
    if (sameDay(date, yesterday)) return "Yesterday";
    return date.toLocaleDateString([], {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  }

  function colorFor(projectId: number | null): string {
    if (projectId == null) return "#D6D1C4";
    const palette = [
      "#7C5CFF",
      "#16A34A",
      "#EA580C",
      "#0891B2",
      "#DB2777",
      "#CA8A04",
      "#059669",
      "#7C3AED",
    ];
    const p = projects.find((x) => x.id === projectId);
    if (p?.color) return p.color;
    return palette[projectId % palette.length];
  }

  type Group = { key: string; label: string; total: number; items: Entry[] };
  const groups: Group[] = $derived.by(() => {
    const map = new Map<string, Entry[]>();
    for (const e of entries) {
      const k = dayKey(e.start_time);
      if (!map.has(k)) map.set(k, []);
      map.get(k)!.push(e);
    }
    return [...map.entries()].map(([key, items]) => ({
      key,
      label: dayLabel(key),
      total: items.reduce((sum, e) => sum + durationMs(e), 0),
      items,
    }));
  });
</script>

<div class="entries">
  {#if entries.length === 0}
    <div class="empty">
      <p class="empty-title">No entries yet</p>
      <p class="empty-sub">Press Start on the Tracker tab to record your first entry.</p>
    </div>
  {/if}

  {#each groups as g (g.key)}
    <section class="day">
      <header class="day-head">
        <span class="day-label">{g.label}</span>
        <span class="day-total">{formatDuration(g.total)}</span>
      </header>
      <ul class="rows">
        {#each g.items as e (e.id)}
          <li class="row">
            <div class="row-top">
              <span class="time">
                {formatTime(e.start_time)}<span class="sep">–</span>{e.end_time
                  ? formatTime(e.end_time)
                  : "now"}
              </span>
              <span class="dur">{formatDuration(durationMs(e))}</span>
            </div>
            <div class="row-bot">
              <span class="swatch" style="background: {colorFor(e.project_id)}"></span>
              <div class="assign">
                <Select
                  size="sm"
                  value={e.project_id}
                  options={[
                    { value: null, label: "No project" },
                    ...projects.map((p) => ({ value: p.id, label: p.name })),
                  ]}
                  onchange={(v) => onProjectChange(e, v)}
                  placeholder="No project"
                />
                <Select
                  size="sm"
                  value={e.task_id}
                  options={[
                    { value: null, label: "No task" },
                    ...(e.project_id != null
                      ? (tasksByProject[e.project_id] ?? [])
                      : []
                    ).map((t) => ({ value: t.id, label: t.name })),
                  ]}
                  onchange={(v) => onTaskChange(e, v)}
                  disabled={e.project_id == null}
                  placeholder="No task"
                />
              </div>
              {#if pendingDeleteId === e.id}
                <div class="confirm">
                  <button
                    class="confirm-yes"
                    onclick={() => confirmDelete(e.id)}
                    aria-label="Confirm delete"
                  >
                    Delete
                  </button>
                  <button
                    class="confirm-no"
                    onclick={cancelDelete}
                    aria-label="Cancel delete"
                  >
                    Cancel
                  </button>
                </div>
              {:else}
                <button
                  class="del"
                  onclick={() => armDelete(e.id)}
                  aria-label="Delete entry"
                  title="Delete entry"
                >
                  <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
                    <path
                      d="M3 4h8m-6 0V2.5A.5.5 0 0 1 5.5 2h3a.5.5 0 0 1 .5.5V4M4 4v7a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1V4"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </button>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    </section>
  {/each}
</div>

<style>
  .entries {
    display: flex;
    flex-direction: column;
    gap: var(--s-5);
    overflow-y: auto;
    padding-right: 2px;
    margin-right: -2px;
  }

  .empty {
    padding: var(--s-8) var(--s-4);
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .empty-title {
    margin: 0;
    color: var(--ink);
    font-size: 14px;
    font-weight: 500;
  }
  .empty-sub {
    margin: 0;
    color: var(--ink-muted);
    font-size: 12px;
  }

  .day {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .day-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 0 2px 2px;
  }
  .day-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--ink);
    letter-spacing: -0.01em;
  }
  .day-total {
    font-size: 12px;
    color: var(--ink-muted);
    font-variant-numeric: tabular-nums;
    font-family: var(--font-mono);
  }

  .rows {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .row {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    transition: border-color 0.14s ease;
  }
  .row:hover {
    border-color: var(--line-strong);
  }

  .row-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .time {
    font-size: 12px;
    color: var(--ink-muted);
    font-variant-numeric: tabular-nums;
  }
  .sep {
    margin: 0 4px;
    color: var(--ink-subtle);
  }
  .dur {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--ink);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.01em;
  }

  .row-bot {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 8px;
    align-items: center;
  }

  .swatch {
    width: 10px;
    height: 10px;
    border-radius: var(--r-pill);
    flex-shrink: 0;
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
  }

  .assign {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
    min-width: 0;
  }

  .del {
    background: transparent;
    color: var(--ink-subtle);
    border: none;
    border-radius: 6px;
    padding: 4px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      color 0.14s ease,
      background 0.14s ease;
  }
  .del:hover {
    color: var(--danger);
    background: var(--danger-soft);
  }

  .confirm {
    display: inline-flex;
    gap: 2px;
    align-items: center;
  }
  .confirm-yes,
  .confirm-no {
    border: none;
    border-radius: 6px;
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.14s ease;
  }
  .confirm-yes {
    background: var(--danger);
    color: #fff;
  }
  .confirm-yes:hover {
    background: #b91c1c;
  }
  .confirm-no {
    background: transparent;
    color: var(--ink-muted);
  }
  .confirm-no:hover {
    background: var(--sunk);
    color: var(--ink);
  }
</style>
