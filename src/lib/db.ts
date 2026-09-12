import Database from "@tauri-apps/plugin-sql";

export type Project = {
  id: number;
  name: string;
  color: string | null;
  archived_at: string | null;
};

export type Task = {
  id: number;
  project_id: number;
  name: string;
  archived_at: string | null;
};

export type Entry = {
  id: number;
  start_time: string;
  end_time: string | null;
  project_id: number | null;
  task_id: number | null;
  note: string | null;
};

let dbInstance: Database | null = null;

export async function db(): Promise<Database> {
  if (!dbInstance) {
    dbInstance = await Database.load("sqlite:jellytrack.db");
  }
  return dbInstance;
}

export async function listProjects(includeArchived = false): Promise<Project[]> {
  const d = await db();
  const where = includeArchived ? "" : "WHERE archived_at IS NULL";
  return d.select<Project[]>(`SELECT * FROM projects ${where} ORDER BY name`);
}

export async function listTasks(projectId: number | null, includeArchived = false): Promise<Task[]> {
  const d = await db();
  if (projectId == null) return [];
  const archivedClause = includeArchived ? "" : "AND archived_at IS NULL";
  return d.select<Task[]>(
    `SELECT * FROM tasks WHERE project_id = $1 ${archivedClause} ORDER BY name`,
    [projectId],
  );
}

export async function createProject(name: string): Promise<number> {
  const d = await db();
  const existing = await d.select<{ id: number; archived_at: string | null }[]>(
    "SELECT id, archived_at FROM projects WHERE name = $1",
    [name],
  );
  if (existing[0]) {
    if (existing[0].archived_at) {
      await d.execute("UPDATE projects SET archived_at = NULL WHERE id = $1", [
        existing[0].id,
      ]);
    }
    return existing[0].id;
  }
  const res = await d.execute("INSERT INTO projects (name) VALUES ($1)", [name]);
  return res.lastInsertId as number;
}

export async function createTask(projectId: number, name: string): Promise<number> {
  const d = await db();
  const existing = await d.select<{ id: number; archived_at: string | null }[]>(
    "SELECT id, archived_at FROM tasks WHERE project_id = $1 AND name = $2",
    [projectId, name],
  );
  if (existing[0]) {
    if (existing[0].archived_at) {
      await d.execute("UPDATE tasks SET archived_at = NULL WHERE id = $1", [
        existing[0].id,
      ]);
    }
    return existing[0].id;
  }
  const res = await d.execute(
    "INSERT INTO tasks (project_id, name) VALUES ($1, $2)",
    [projectId, name],
  );
  return res.lastInsertId as number;
}

export async function archiveProject(id: number): Promise<void> {
  const d = await db();
  await d.execute("UPDATE projects SET archived_at = $1 WHERE id = $2", [
    new Date().toISOString(),
    id,
  ]);
}

export async function archiveTask(id: number): Promise<void> {
  const d = await db();
  await d.execute("UPDATE tasks SET archived_at = $1 WHERE id = $2", [
    new Date().toISOString(),
    id,
  ]);
}

export async function getPreference(key: string): Promise<string | null> {
  const d = await db();
  const rows = await d.select<{ value: string }[]>(
    "SELECT value FROM preferences WHERE key = $1",
    [key],
  );
  return rows[0]?.value ?? null;
}

export async function setPreference(key: string, value: string): Promise<void> {
  const d = await db();
  await d.execute(
    `INSERT INTO preferences (key, value) VALUES ($1, $2)
     ON CONFLICT(key) DO UPDATE SET value = excluded.value`,
    [key, value],
  );
}

export async function getRunningEntry(): Promise<Entry | null> {
  const d = await db();
  const rows = await d.select<Entry[]>(
    "SELECT * FROM entries WHERE end_time IS NULL ORDER BY start_time DESC LIMIT 1",
  );
  return rows[0] ?? null;
}

export async function startEntry(
  projectId: number | null,
  taskId: number | null,
): Promise<number> {
  const d = await db();
  const graceRaw = await getPreference("resume_grace_seconds");
  const graceSec = graceRaw != null ? Number(graceRaw) : 60;
  if (Number.isFinite(graceSec) && graceSec > 0) {
    const rows = await d.select<Entry[]>(
      `SELECT * FROM entries
       WHERE end_time IS NOT NULL
       ORDER BY end_time DESC
       LIMIT 1`,
    );
    const last = rows[0];
    if (
      last &&
      last.project_id === projectId &&
      last.task_id === taskId &&
      last.end_time
    ) {
      const stoppedAgoSec = (Date.now() - new Date(last.end_time).getTime()) / 1000;
      if (stoppedAgoSec <= graceSec) {
        await d.execute("UPDATE entries SET end_time = NULL WHERE id = $1", [last.id]);
        return last.id;
      }
    }
  }
  const res = await d.execute(
    "INSERT INTO entries (start_time, project_id, task_id) VALUES ($1, $2, $3)",
    [new Date().toISOString(), projectId, taskId],
  );
  return res.lastInsertId as number;
}

export async function createEntry(input: {
  start_time: string;
  end_time: string | null;
  project_id: number | null;
  task_id: number | null;
  note?: string | null;
}): Promise<number> {
  const d = await db();
  const res = await d.execute(
    `INSERT INTO entries (start_time, end_time, project_id, task_id, note)
     VALUES ($1, $2, $3, $4, $5)`,
    [
      input.start_time,
      input.end_time,
      input.project_id,
      input.task_id,
      input.note ?? null,
    ],
  );
  return res.lastInsertId as number;
}

export async function stopEntry(id: number): Promise<void> {
  const d = await db();
  await d.execute("UPDATE entries SET end_time = $1 WHERE id = $2", [
    new Date().toISOString(),
    id,
  ]);
}

export async function listRecentEntries(limit = 50): Promise<Entry[]> {
  const d = await db();
  return d.select<Entry[]>(
    "SELECT * FROM entries ORDER BY start_time DESC LIMIT $1",
    [limit],
  );
}

export async function updateEntry(
  id: number,
  patch: Partial<Pick<Entry, "project_id" | "task_id" | "note" | "start_time" | "end_time">>,
): Promise<void> {
  const d = await db();
  const fields = Object.keys(patch);
  if (fields.length === 0) return;
  const setClause = fields.map((f, i) => `${f} = $${i + 1}`).join(", ");
  const values = fields.map((f) => (patch as Record<string, unknown>)[f]);
  await d.execute(`UPDATE entries SET ${setClause} WHERE id = $${fields.length + 1}`, [
    ...values,
    id,
  ]);
}

export async function deleteEntry(id: number): Promise<void> {
  const d = await db();
  await d.execute("DELETE FROM entries WHERE id = $1", [id]);
}
