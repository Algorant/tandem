import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { buildInitArgs, buildSearchArgs, buildTaskArgs } from "../index";
const bin = process.env.TANDEM_BIN ?? "tandem";
const assert = (v: unknown, m: string): asserts v => { if (!v) throw new Error(m); };
const run = async (args: string[], cwd: string) => { const p = Bun.spawn([bin, ...args], { cwd, stdout: "pipe", stderr: "pipe" }); const out = await new Response(p.stdout).text(); const err = await new Response(p.stderr).text(); if (await p.exited) throw new Error(`${args.join(" ")}\n${out}\n${err}`); return out; };
const ws = await mkdtemp(join(tmpdir(), "pi-tandem-relationships-"));
try { await run(buildInitArgs({ title: "Relationships" }), ws); const epic = JSON.parse(await run(buildTaskArgs({ action: "add", title: "Epic", kind: "epic", acceptance: ["ships"] }), ws)); const task = JSON.parse(await run(buildTaskArgs({ action: "add", title: "Task", parent: epic.data.id, acceptance: ["works"] }), ws)); assert(/^task-\d+$/.test(task.data.id), "Epic child uses global task ID"); const sub = JSON.parse(await run(buildTaskArgs({ action: "add", title: "Subtask", parent: task.data.id, acceptance: ["done"] }), ws)); assert(sub.data.id === `${task.data.id}-1`, "Task child uses derived ID"); const search = JSON.parse(await run(buildSearchArgs({ query: "Subtask" }), ws)); assert(search.ok && search.data.some((x: any) => x.id === sub.data.id), "search returns relationship fixture"); console.log("pi-tandem relationship smoke passed (4 assertions)"); } finally { await rm(ws, { recursive: true, force: true }); }
