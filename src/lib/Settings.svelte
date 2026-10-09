<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import {
    getPreference,
    listProjects,
    listTasks,
    setPreference,
    type Project,
    type Task,
  } from "./db";
  import { setTimeFormat, timeFormat } from "./timeFormat.svelte";
  import Select from "./Select.svelte";

  let projects: Project[] = $state([]);
  let defaultProjectId: number | null = $state(null);
  let defaultTaskId: number | null = $state(null);
  let defaultProjectTasks: Task[] = $state([]);
  let useLastUsedDefault = $state(false);

  let dbPath = $state("");
  let dbIsDefault = $state(true);
  let pendingRestart = $state(false);
  let dbError = $state<string | null>(null);

  async function load() {
    await loadDbLocation();
    projects = await listProjects();
    const dp = await getPreference("default_project_id");
    const dt = await getPreference("default_task_id");
    defaultProjectId = dp ? Number(dp) : null;
    defaultTaskId = dt ? Number(dt) : null;
    defaultProjectTasks = defaultProjectId
      ? await listTasks(defaultProjectId)
      : [];
    useLastUsedDefault = (await getPreference("use_last_used_default")) === "1";
  }

  async function loadDbLocation() {
    const loc = await invoke<{ path: string; is_default: boolean }>(
      "get_db_location",
    );
    dbPath = loc.path;
    dbIsDefault = loc.is_default;
  }

  async function openExistingDb() {
    dbError = null;
    const picked = await open({
      multiple: false,
      directory: false,
      title: "Open existing jellytrack database",
      filters: [
        { name: "SQLite database", extensions: ["db", "sqlite", "sqlite3"] },
      ],
    });
    if (typeof picked !== "string") return;
    try {
      await invoke("set_db_location", { path: picked });
      await loadDbLocation();
      pendingRestart = true;
    } catch (e) {
      dbError = String(e);
    }
  }

  async function createNewDb() {
    dbError = null;
    const picked = await save({
      title: "Create new jellytrack database",
      defaultPath: "jellytrack.db",
      filters: [
        { name: "SQLite database", extensions: ["db", "sqlite", "sqlite3"] },
      ],
    });
    if (typeof picked !== "string") return;
    try {
      await invoke("create_new_db", { path: picked });
      await loadDbLocation();
      pendingRestart = true;
    } catch (e) {
      dbError = String(e);
    }
  }

  async function resetDbFile() {
    dbError = null;
    try {
      await invoke("reset_db_location");
      await loadDbLocation();
      pendingRestart = true;
    } catch (e) {
      dbError = String(e);
    }
  }

  async function relaunchApp() {
    await invoke("relaunch_app");
  }

  async function onUseLastUsedChange() {
    await setPreference("use_last_used_default", useLastUsedDefault ? "1" : "0");
  }

  async function onDefaultProjectChange() {
    await setPreference("default_project_id", String(defaultProjectId ?? ""));
    defaultProjectTasks = defaultProjectId
      ? await listTasks(defaultProjectId)
      : [];
    if (!defaultProjectTasks.some((t) => t.id === defaultTaskId)) {
      defaultTaskId = null;
      await setPreference("default_task_id", "");
    }
  }

  async function onDefaultTaskChange() {
    await setPreference("default_task_id", String(defaultTaskId ?? ""));
  }

  async function on24hChange(e: Event) {
    const next = (e.currentTarget as HTMLInputElement).checked;
    await setTimeFormat(next);
  }

  onMount(load);
</script>

<div class="settings-tab">
  <section>
    <h3>Defaults</h3>
    <p class="hint">Used when Start is pressed without picking a project or task.</p>
    <label class="toggle">
      <input
        type="checkbox"
        bind:checked={useLastUsedDefault}
        onchange={onUseLastUsedChange}
      />
      <span>Use last used project as default</span>
    </label>
    {#if !useLastUsedDefault}
      <div class="grid">
        <div class="field">
          <span class="lbl">Project</span>
          <Select
            bind:value={defaultProjectId}
            options={[
              { value: null, label: "None" },
              ...projects.map((p) => ({ value: p.id, label: p.name })),
            ]}
            onchange={onDefaultProjectChange}
            placeholder="None"
          />
        </div>
        <div class="field">
          <span class="lbl">Task</span>
          <Select
            bind:value={defaultTaskId}
            options={[
              { value: null, label: "None" },
              ...defaultProjectTasks.map((t) => ({ value: t.id, label: t.name })),
            ]}
            onchange={onDefaultTaskChange}
            disabled={!defaultProjectId}
            placeholder="None"
          />
        </div>
      </div>
    {/if}
  </section>

  <section>
    <h3>Appearance</h3>
    <label class="toggle">
      <input
        type="checkbox"
        checked={timeFormat.use24h}
        onchange={on24hChange}
      />
      <span>Use 24-hour time format</span>
    </label>
  </section>

  <section>
    <h3>Database</h3>
    <p class="hint">
      SQLite file, inspect with any SQL tool.
      {#if !dbIsDefault}<span class="tag">custom</span>{/if}
    </p>
    <code class="path">{dbPath}</code>
    <p class="warn">
      Changing the database file requires a restart of jellytrack.
    </p>
    <div class="db-actions">
      <button class="add-btn secondary" onclick={createNewDb}>Create new…</button>
      <button class="add-btn secondary" onclick={openExistingDb}>Open existing…</button>
      {#if !dbIsDefault}
        <button class="add-btn secondary" onclick={resetDbFile}>Reset to default</button>
      {/if}
    </div>
    {#if pendingRestart}
      <div class="restart-strip">
        <span>Database location changed — relaunch to apply.</span>
        <button class="add-btn" onclick={relaunchApp}>Relaunch</button>
      </div>
    {/if}
    {#if dbError}
      <p class="err">{dbError}</p>
    {/if}
  </section>
</div>

<style>
  .settings-tab {
    display: flex;
    flex-direction: column;
    gap: var(--s-5);
    overflow-y: auto;
    padding-right: 2px;
    margin-right: -2px;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  h3 {
    font-size: 13px;
    color: var(--ink);
    font-weight: 600;
    letter-spacing: -0.01em;
    margin: 0;
  }

  .hint {
    font-size: 12px;
    color: var(--ink-muted);
    margin: 0 0 4px;
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--s-2);
  }

  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--ink);
    cursor: pointer;
    user-select: none;
    padding: 2px 0;
  }

  .toggle input {
    margin: 0;
    cursor: pointer;
    accent-color: var(--accent);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .lbl {
    font-size: 12px;
    color: var(--ink-muted);
    padding-left: 2px;
    font-weight: 500;
  }

  .add-btn {
    background: var(--ink);
    color: var(--paper);
    border: none;
    border-radius: var(--r-md);
    padding: 0 14px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.14s ease;
  }
  .add-btn:hover {
    background: #000;
  }

  .path {
    display: block;
    background: var(--sunk);
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    padding: 8px 10px;
    font-family: var(--font-mono);
    font-size: 11px;
    word-break: break-all;
    color: var(--ink-muted);
    line-height: 1.5;
  }

  .warn {
    font-size: 11px;
    color: var(--ink-muted);
    background: var(--sunk);
    border: 1px dashed var(--line-strong);
    border-radius: var(--r-sm);
    padding: 6px 8px;
    margin: 0;
  }

  .db-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .add-btn.secondary {
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    padding: 6px 10px;
    font-size: 12px;
  }
  .add-btn.secondary:hover {
    background: var(--sunk);
    border-color: var(--line-strong);
  }

  .restart-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px;
    border: 1px solid var(--accent);
    background: var(--accent-soft);
    border-radius: var(--r-md);
    font-size: 12px;
    color: var(--ink);
  }

  .tag {
    display: inline-block;
    margin-left: 6px;
    padding: 1px 6px;
    font-size: 10px;
    font-weight: 600;
    color: var(--ink);
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    border-radius: var(--r-pill);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .err {
    font-size: 11px;
    color: var(--danger);
    margin: 0;
  }
</style>
