import { existsSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { buildInitArgs, buildTaskArgs } from "../index";
const bin = process.env.TANDEM_BIN ?? "tandem";
const assert = (v: unknown, m: string): asserts v => { if (!v) throw new Error(m); };
const run = async (args: string[], cwd: string) => { const p = Bun.spawn([bin, ...args], { cwd, stdout: "pipe", stderr: "pipe" }); const out = await new Response(p.stdout).text(); const err = await new Response(p.stderr).text(); if (await p.exited) throw new Error(`${args.join(" ")}\n${out}\n${err}`); return out; };
const ws = await mkdtemp(join(tmpdir(), "pi-tandem-runtime-"));
try { await run(buildInitArgs({ title: "Runtime" }), ws); const out = JSON.parse(await run(buildTaskArgs({ action: "add", title: "Runtime task", acceptance: ["loads"] }), ws)); assert(out.ok && out.data.id, "runtime add envelope"); const shown = JSON.parse(await run(buildTaskArgs({ action: "show", id: out.data.id }), ws)); assert(shown.ok && shown.data.id === out.data.id, "runtime show envelope"); console.log("pi-tandem project-local Pi runtime smoke passed (2 assertions)"); } finally { await rm(ws, { recursive: true, force: true }); }
