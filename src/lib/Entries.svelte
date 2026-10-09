<script lang="ts">
  import { onMount } from "svelte";
  import {
    createEntry,
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
  import { timeFormat } from "./timeFormat.svelte";
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

  async function saveNote(e: Entry, next: string) {
    const value = next.trim() ? next : null;
    if ((e.note ?? null) === value) return;
    await updateEntry(e.id, { note: value });
    e.note = value;
  }

  let showAdd = $state(false);
  let addDate = $state(todayLocal());
  let addStart = $state("09:00");
  let addEnd = $state("10:00");
  let addProjectId: number | null = $state(null);
  let addTaskId: number | null = $state(null);
  let addTasks: Task[] = $state([]);
  let addNote = $state("");
  let addError = $state<string | null>(null);

  let editingId = $state<number | null>(null);
  let editDate = $state("");
  let editStart = $state("");
  let editEnd = $state("");
  let editProjectId: number | null = $state(null);
  let editTaskId: number | null = $state(null);
  let editTasks: Task[] = $state([]);
  let editNote = $state("");
  let editError = $state<string | null>(null);

  function localDateYMD(iso: string): string {
    const d = new Date(iso);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  }

  function localTimeHHMM(iso: string): string {
    const d = new Date(iso);
    return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  }

  function openEdit(e: Entry) {
    if (!e.end_time) return;
    editingId = e.id;
    editDate = localDateYMD(e.start_time);
    editStart = localTimeHHMM(e.start_time);
    editEnd = localTimeHHMM(e.end_time);
    editProjectId = e.project_id;
    editTaskId = e.task_id;
    editTasks = e.project_id != null ? (tasksByProject[e.project_id] ?? []) : [];
    editNote = e.note ?? "";
    editError = null;
    cancelDelete();
  }

  function cancelEdit() {
    editingId = null;
    editError = null;
  }

  async function onEditProjectChange(v: number | null) {
    editProjectId = v;
    editTasks = v != null ? (tasksByProject[v] ?? []) : [];
    if (!editTasks.some((t) => t.id === editTaskId)) editTaskId = null;
  }

  async function submitEdit(id: number) {
    editError = null;
    if (!editDate || !editStart || !editEnd) {
      editError = "Date, start and end are required.";
      return;
    }
    const startISO = localToISO(editDate, editStart);
    const endISO = localToISO(editDate, editEnd);
    if (new Date(endISO).getTime() <= new Date(startISO).getTime()) {
      editError = "End must be after start.";
      return;
    }
    await updateEntry(id, {
      start_time: startISO,
      end_time: endISO,
      project_id: editProjectId,
      task_id: editTaskId,
      note: editNote.trim() || null,
    });
    editingId = null;
    await load();
  }

  function todayLocal(): string {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  }

  function localToISO(dateStr: string, timeStr: string): string {
    const [y, m, d] = dateStr.split("-").map(Number);
    const [hh, mm] = timeStr.split(":").map(Number);
    return new Date(y, m - 1, d, hh, mm, 0, 0).toISOString();
  }

  function openAdd() {
    showAdd = true;
    addDate = todayLocal();
    addStart = "09:00";
    addEnd = "10:00";
    addProjectId = projects.filter((p) => !p.archived_at)[0]?.id ?? null;
    addTaskId = null;
    addNote = "";
    addError = null;
    void refreshAddTasks();
  }

  function cancelAdd() {
    showAdd = false;
    addError = null;
  }

  async function refreshAddTasks() {
    addTasks = addProjectId ? (tasksByProject[addProjectId] ?? []) : [];
    if (!addTasks.some((t) => t.id === addTaskId)) addTaskId = null;
  }

  async function onAddProjectChange(v: number | null) {
    addProjectId = v;
    await refreshAddTasks();
  }

  async function submitAdd() {
    addError = null;
    if (!addDate || !addStart || !addEnd) {
      addError = "Date, start and end are required.";
      return;
    }
    const startISO = localToISO(addDate, addStart);
    const endISO = localToISO(addDate, addEnd);
    if (new Date(endISO).getTime() <= new Date(startISO).getTime()) {
      addError = "End must be after start.";
      return;
    }
    await createEntry({
      start_time: startISO,
      end_time: endISO,
      project_id: addProjectId,
      task_id: addTaskId,
      note: addNote.trim() || null,
    });
    showAdd = false;
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
  <div class="toolbar">
    {#if !showAdd}
      <button class="add-entry" onclick={openAdd}>+ Add entry</button>
    {/if}
  </div>

  {#if showAdd}
    <section class="add-form">
      <div class="add-grid">
        <label class="add-field">
          <span class="lbl">Date</span>
          <input type="date" bind:value={addDate} />
        </label>
        <label class="add-field">
          <span class="lbl">Start</span>
          <input type="time" bind:value={addStart} />
        </label>
        <label class="add-field">
          <span class="lbl">End</span>
          <input type="time" bind:value={addEnd} />
        </label>
      </div>
      <div class="add-grid">
        <div class="add-field">
          <span class="lbl">Project</span>
          <Select
            size="sm"
            value={addProjectId}
            options={[
              { value: null, label: "No project" },
              ...projects
                .filter((p) => !p.archived_at)
                .map((p) => ({ value: p.id, label: p.name })),
            ]}
            onchange={(v) => onAddProjectChange(v)}
            placeholder="No project"
          />
        </div>
        <div class="add-field">
          <span class="lbl">Task</span>
          <Select
            size="sm"
            bind:value={addTaskId}
            options={[
              { value: null, label: "No task" },
              ...addTasks.map((t) => ({ value: t.id, label: t.name })),
            ]}
            disabled={addProjectId == null}
            placeholder="No task"
          />
        </div>
      </div>
      <label class="add-field">
        <span class="lbl">Notes</span>
        <textarea class="notes" rows="3" placeholder="What was this session about?" bind:value={addNote}></textarea>
      </label>
      {#if addError}
        <p class="add-error">{addError}</p>
      {/if}
      <div class="add-actions">
        <button class="add-cancel" onclick={cancelAdd}>Cancel</button>
        <button class="add-save" onclick={submitAdd}>Save entry</button>
      </div>
    </section>
  {/if}

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
            {#if editingId === e.id}
              <div class="edit-form">
                <div class="add-grid">
                  <label class="add-field">
                    <span class="lbl">Date</span>
                    <input type="date" bind:value={editDate} />
                  </label>
                  <label class="add-field">
                    <span class="lbl">Start</span>
                    <input type="time" bind:value={editStart} />
                  </label>
                  <label class="add-field">
                    <span class="lbl">End</span>
                    <input type="time" bind:value={editEnd} />
                  </label>
                </div>
                <div class="add-grid two-col">
                  <div class="add-field">
                    <span class="lbl">Project</span>
                    <Select
                      size="sm"
                      value={editProjectId}
                      options={[
                        { value: null, label: "No project" },
                        ...projects
                          .filter((p) => !p.archived_at)
                          .map((p) => ({ value: p.id, label: p.name })),
                      ]}
                      onchange={(v) => onEditProjectChange(v)}
                      placeholder="No project"
                    />
                  </div>
                  <div class="add-field">
                    <span class="lbl">Task</span>
                    <Select
                      size="sm"
                      bind:value={editTaskId}
                      options={[
                        { value: null, label: "No task" },
                        ...editTasks.map((t) => ({ value: t.id, label: t.name })),
                      ]}
                      disabled={editProjectId == null}
                      placeholder="No task"
                    />
                  </div>
                </div>
                <label class="add-field">
                  <span class="lbl">Notes</span>
                  <textarea
                    class="notes"
                    rows="3"
                    placeholder="What was this session about?"
                    bind:value={editNote}
                  ></textarea>
                </label>
                {#if editError}
                  <p class="add-error">{editError}</p>
                {/if}
                <div class="add-actions">
                  <button class="add-cancel" onclick={cancelEdit}>Cancel</button>
                  <button class="add-save" onclick={() => submitEdit(e.id)}>Save</button>
                </div>
              </div>
            {:else}
              <div class="row-top">
                <span class="time">
                  {formatTime(e.start_time, timeFormat.use24h)}<span class="sep">–</span>{e.end_time
                    ? formatTime(e.end_time, timeFormat.use24h)
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
                <div class="row-actions">
                  {#if e.end_time}
                    <button
                      class="edit"
                      onclick={() => openEdit(e)}
                      aria-label="Edit entry"
                      title="Edit entry"
                    >
                      <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
                        <path
                          d="M2 10l6-6 2 2-6 6H2v-2z M8 4l2-2 2 2-2 2"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="1.2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        />
                      </svg>
                    </button>
                  {/if}
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
                </div>
              </div>
              <textarea
                class="row-note"
                rows="2"
                placeholder="Add a note…"
                value={e.note ?? ""}
                onblur={(ev) =>
                  saveNote(e, (ev.currentTarget as HTMLTextAreaElement).value)}
              ></textarea>
              {#if pendingDeleteId === e.id}
                <div class="delete-overlay">
                  <span class="delete-prompt">Delete this entry?</span>
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
                </div>
              {/if}
            {/if}
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
    position: relative;
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

  .row-actions {
    display: inline-flex;
    gap: 2px;
    align-items: center;
  }

  .edit {
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
  .edit:hover {
    color: var(--ink);
    background: var(--sunk);
  }

  .edit-form {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }
  .edit-form .add-grid.two-col {
    grid-template-columns: 1fr 1fr;
  }
  .edit-form input[type="date"],
  .edit-form input[type="time"] {
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 8px 10px;
    font-size: 13px;
    font-family: inherit;
  }
  .edit-form input:hover {
    border-color: var(--line-strong);
  }
  .edit-form .notes {
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 10px 12px;
    font-size: 13px;
    font-family: inherit;
    resize: vertical;
    min-height: 72px;
  }
  .edit-form .notes:hover {
    border-color: var(--line-strong);
  }

  .delete-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    padding: 10px 14px;
    background: var(--danger-soft);
    border: 1px solid var(--danger);
    border-radius: var(--r-md);
    z-index: 2;
  }
  .delete-prompt {
    font-size: 13px;
    font-weight: 500;
    color: var(--ink);
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

  .toolbar {
    display: flex;
    justify-content: flex-end;
  }

  .add-entry {
    background: var(--ink);
    color: var(--paper);
    border: none;
    border-radius: var(--r-md);
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.14s ease;
  }
  .add-entry:hover {
    background: #000;
  }

  .add-form {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
    padding: var(--s-3);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
  }

  .add-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: var(--s-2);
  }
  .add-grid:has(> .add-field:nth-child(2):last-child) {
    grid-template-columns: 1fr 1fr;
  }

  .add-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .lbl {
    font-size: 11px;
    color: var(--ink-muted);
    font-weight: 500;
    padding-left: 2px;
  }

  .add-form input[type="date"],
  .add-form input[type="time"] {
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 8px 10px;
    font-size: 13px;
    font-family: inherit;
  }
  .add-form input:hover {
    border-color: var(--line-strong);
  }

  .add-form .notes {
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 10px 12px;
    font-size: 13px;
    font-family: inherit;
    resize: vertical;
    min-height: 72px;
  }
  .add-form .notes:hover {
    border-color: var(--line-strong);
  }

  .add-error {
    color: var(--danger);
    font-size: 12px;
    margin: 0;
  }

  .add-actions {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }

  .add-cancel,
  .add-save {
    border: none;
    border-radius: var(--r-md);
    padding: 8px 14px;
    font-size: 13px;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.14s ease;
  }
  .add-cancel {
    background: transparent;
    color: var(--ink-muted);
  }
  .add-cancel:hover {
    background: var(--sunk);
    color: var(--ink);
  }
  .add-save {
    background: var(--accent);
    color: var(--accent-ink);
  }
  .add-save:hover {
    background: var(--accent-hover);
  }

  .row-note {
    background: var(--sunk);
    color: var(--ink);
    border: 1px solid transparent;
    border-radius: var(--r-md);
    padding: 8px 10px;
    font-size: 12px;
    font-family: inherit;
    resize: vertical;
    min-height: 48px;
    width: 100%;
    transition:
      border-color 0.14s ease,
      background 0.14s ease;
  }
  .row-note:hover {
    border-color: var(--line);
    background: var(--surface);
  }
  .row-note:focus-visible {
    background: var(--surface);
    outline: none;
    box-shadow: var(--focus-ring);
  }
</style>
