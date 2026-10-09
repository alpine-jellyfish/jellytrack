<script lang="ts">
  import { onMount } from "svelte";
  import {
    archiveProject,
    archiveTask,
    createProject,
    createTask,
    listProjects,
    listTasks,
    type Project,
    type Task,
  } from "./db";

  let projects: Project[] = $state([]);
  let selectedProjectId: number | null = $state(null);
  let tasks: Task[] = $state([]);
  let newProjectName = $state("");
  let newTaskName = $state("");

  async function load() {
    projects = await listProjects();
    if (!selectedProjectId && projects.length > 0) {
      selectedProjectId = projects[0].id;
    }
    tasks = selectedProjectId ? await listTasks(selectedProjectId) : [];
  }

  onMount(load);

  async function onSelectedProjectChange() {
    tasks = selectedProjectId ? await listTasks(selectedProjectId) : [];
  }

  async function addProject() {
    const name = newProjectName.trim();
    if (!name) return;
    const id = await createProject(name);
    newProjectName = "";
    selectedProjectId = id;
    await load();
  }

  async function addTask() {
    const name = newTaskName.trim();
    if (!name || !selectedProjectId) return;
    await createTask(selectedProjectId, name);
    newTaskName = "";
    await load();
  }

  function colorFor(projectId: number): string {
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
    return p?.color ?? palette[projectId % palette.length];
  }
</script>

<div class="projects-tab">
  <section>
    <h3>Projects</h3>
    <div class="add">
      <input
        placeholder="New project"
        bind:value={newProjectName}
        onkeydown={(e) => e.key === "Enter" && addProject()}
      />
      <button class="add-btn" onclick={addProject}>Add</button>
    </div>
    <ul class="list">
      {#each projects as p (p.id)}
        <li class:selected={p.id === selectedProjectId}>
          <button
            class="row-btn"
            onclick={() => {
              selectedProjectId = p.id;
              onSelectedProjectChange();
            }}
          >
            <span class="swatch" style="background: {colorFor(p.id)}"></span>
            <span class="name">{p.name}</span>
          </button>
          <button
            class="action"
            onclick={async () => {
              await archiveProject(p.id);
              await load();
            }}
            title="Archive project"
          >
            Archive
          </button>
        </li>
      {/each}
    </ul>
  </section>

  {#if selectedProjectId}
    <section>
      <h3>
        Tasks in <span class="in-project"
          >{projects.find((p) => p.id === selectedProjectId)?.name}</span
        >
      </h3>
      <div class="add">
        <input
          placeholder="New task"
          bind:value={newTaskName}
          onkeydown={(e) => e.key === "Enter" && addTask()}
        />
        <button class="add-btn" onclick={addTask}>Add</button>
      </div>
      <ul class="list">
        {#each tasks as t (t.id)}
          <li>
            <span class="row-btn as-text">
              <span class="task-dot"></span>
              <span class="name">{t.name}</span>
            </span>
            <button
              class="action"
              onclick={async () => {
                await archiveTask(t.id);
                await load();
              }}
              title="Archive task"
            >
              Archive
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
</div>

<style>
  .projects-tab {
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

  .in-project {
    color: var(--ink-muted);
    font-weight: 500;
  }

  .add {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 6px;
  }

  input {
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 9px 12px;
    font-size: 13px;
    font-family: inherit;
    transition: border-color 0.14s ease;
  }
  input:hover {
    border-color: var(--line-strong);
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

  .list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 4px 6px 4px 10px;
    transition: border-color 0.14s ease;
  }
  li:hover {
    border-color: var(--line-strong);
  }
  li.selected {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .row-btn {
    background: transparent;
    color: var(--ink);
    border: none;
    text-align: left;
    cursor: pointer;
    font-size: 13px;
    padding: 6px 0;
    flex: 1;
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }
  .row-btn.as-text {
    cursor: default;
  }

  .swatch {
    width: 10px;
    height: 10px;
    border-radius: var(--r-pill);
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
  }

  .task-dot {
    width: 4px;
    height: 4px;
    border-radius: var(--r-pill);
    background: var(--ink-subtle);
    margin: 0 3px;
  }

  .name {
    color: var(--ink);
    font-weight: 500;
  }

  .action {
    background: transparent;
    color: var(--ink-muted);
    border: none;
    border-radius: 6px;
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition:
      color 0.14s ease,
      background 0.14s ease;
  }
  .action:hover {
    color: var(--danger);
    background: var(--danger-soft);
  }
</style>
