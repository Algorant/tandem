import { existsSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
	buildAccordArgs,
	buildDecisionArgs,
	buildInitArgs,
	buildLogArgs,
	buildRulesArgs,
	buildSearchArgs,
	buildTaskArgs,
	tandemTaskParameters,
} from "../index";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");
const localTandem = join(repoRoot, "tandem", "target", "debug", process.platform === "win32" ? "tandem.exe" : "tandem");

function assert(condition: unknown, message: string): asserts condition {
	if (!condition) throw new Error(message);
}

async function runProcess(command: string, args: string[], cwd: string): Promise<string> {
	const proc = Bun.spawn([command, ...args], { cwd, stdout: "pipe", stderr: "pipe" });
	const stdout = await new Response(proc.stdout).text();
	const stderr = await new Response(proc.stderr).text();
	const code = await proc.exited;
	if (code !== 0) {
		throw new Error(`command failed (${code}): ${command} ${args.join(" ")}\nstdout:\n${stdout}\nstderr:\n${stderr}`);
	}
	return stdout;
}

async function ensureTandem(): Promise<string> {
	const envBin = process.env.TANDEM_BIN;
	if (envBin) return envBin;
	console.log("Building current repository tandem for smoke test...");
	await runProcess("cargo", ["build", "--manifest-path", join(repoRoot, "tandem", "Cargo.toml")], repoRoot);
	return existsSync(localTandem) ? localTandem : "tandem";
}

async function runTandem(tandem: string, args: string[], cwd: string): Promise<string> {
	return runProcess(tandem, args, cwd);
}

function parseJson(text: string): any {
	return JSON.parse(text.trim());
}

function parseId(output: string): string {
	const match = /^ID:\s+(\S+)/m.exec(output);
	assert(match, `could not parse ID from output:\n${output}`);
	return match[1];
}

async function runRepoReadSmoke(tandem: string): Promise<void> {
	if (!existsSync(join(repoRoot, ".tandem", "tandem.md"))) {
		console.log("pi-tandem repo read smoke skipped: this checkout has no local .tandem workspace");
		return;
	}
	const list = parseJson(await runTandem(tandem, buildTaskArgs({ action: "list" }), repoRoot));
	assert(list.ok === true, "repo task list JSON should be ok");
	const items = list.data?.items ?? [];
	assert(Array.isArray(items), "repo task list should expose an items array");
	if (items.length === 0) {
		console.log("pi-tandem repo read smoke skipped: checkout Tandem workspace has no active items");
		return;
	}
	const taskId = items.find((item: any) => item.id === "task-14")?.id ?? items[0]?.id;
	assert(typeof taskId === "string", "repo task list should expose an item id");

	const shown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: taskId }), repoRoot));
	assert(shown.data?.document?.id === taskId, "repo task show should return the selected task");

	const logs = parseJson(await runTandem(tandem, buildLogArgs({ action: "list" }), repoRoot));
	assert(logs.ok === true, "repo log list JSON should be ok");
	const logSearch = parseJson(await runTandem(tandem, buildLogArgs({ action: "search", query: "pi-tandem" }), repoRoot));
	assert(logSearch.ok === true, "repo log search JSON should be ok");

	const rules = parseJson(await runTandem(tandem, buildRulesArgs({ action: "list" }), repoRoot));
	assert(rules.ok === true, "repo rules list JSON should be ok");
	assert(typeof rules.data?.counts?.total === "number", "repo rules list should expose counts");

	const decisions = parseJson(await runTandem(tandem, buildDecisionArgs({ action: "list" }), repoRoot));
	assert(decisions.ok === true, "repo decision list JSON should be ok");
	assert(typeof decisions.data?.count === "number", "repo decision list should expose a count");

	const search = parseJson(await runTandem(tandem, buildSearchArgs({ query: "pi-tandem" }), repoRoot));
	assert(search.ok === true, "repo search JSON should be ok");
	assert((search.data?.results ?? []).length > 0, "repo search should find pi-tandem work");

	console.log(`pi-tandem repo read smoke passed against ${repoRoot}`);
}

const taskSchemaProperties = (tandemTaskParameters as any).properties ?? {};
assert(taskSchemaProperties.summary, "tandem_task schema should expose summary for complete actions");
assert(taskSchemaProperties.parent, "tandem_task schema should expose parent for canonical hierarchy creation");
assert(taskSchemaProperties.kind, "tandem_task schema should expose kind for Epic creation");
assert(!taskSchemaProperties.subtasks, "tandem_task schema should not expose deprecated inline subtask authoring");

const initArgs = buildInitArgs({ title: "Pi Tandem Smoke" });
assert(initArgs.join(" ") === "init --title Pi Tandem Smoke", "tandem_init builder should map to init --title");

const epicArgs = buildTaskArgs({ action: "add", title: "Canonical Epic", kind: "epic" });
assert(epicArgs.join(" ") === "add --title Canonical Epic --kind epic", "tandem_task add builder should map Epic kind directly");

const updateArgs = buildTaskArgs({ action: "update", id: "task-1", kind: "epic", priority: "high", parent: "task-2", tags: ["cli"] });
assert(updateArgs.join(" ") === "update task-1 --kind epic --priority high --parent task-2 --tag cli", "tandem_task update builder should map kind, metadata, and parent flags");

for (const [field, params] of [
	["description", { description: "new body" }],
	["accord", { accord: "ready" }],
	["review", { review: "pending" }],
] as const) {
	let rejected = false;
	try {
		buildTaskArgs({ action: "update", id: "task-1", ...params });
	} catch (err) {
		rejected = err instanceof Error && err.message.includes(`does not support ${field}`);
	}
	assert(rejected, `tandem_task update builder should reject unsupported ${field}`);
}

let legacySubtasksRejected = false;
try {
	buildTaskArgs({ action: "add", title: "Legacy inline checklist", subtasks: ["new checklist item"] } as any);
} catch (err) {
	legacySubtasksRejected = err instanceof Error && err.message.includes("deprecated inline subtasks") && err.message.includes("parent=<task-id>");
}
assert(legacySubtasksRejected, "tandem_task builder should reject deprecated inline subtask authoring");

const completeArgs = buildTaskArgs({ action: "complete", id: "task-1", summary: "Schema smoke" });
assert(completeArgs.includes("--summary"), "tandem_task complete builder should pass --summary");
assert(completeArgs.includes("Schema smoke"), "tandem_task complete builder should include summary value");

const tandem = await ensureTandem();
await runRepoReadSmoke(tandem);

const workspace = await mkdtemp(join(tmpdir(), "pi-tandem-smoke-"));

try {
	await runTandem(tandem, buildInitArgs({ title: "Pi Tandem Smoke" }), workspace);

	const emptyList = parseJson(await runTandem(tandem, buildTaskArgs({ action: "list" }), workspace));
	assert(emptyList.ok === true, "task list JSON should be ok");
	assert(emptyList.data.counts.total === 0, "new workspace should start with zero active items");

	const addOutput = await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Smoke task",
		state: "todo",
		description: "Created by pi-tandem smoke test.",
		priority: "medium",
		tags: ["smoke"],
	}), workspace);
	const taskId = parseId(addOutput);

	const shown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: taskId }), workspace));
	assert(shown.data.document.title === "Smoke task", "show should return created task");

	let unsupportedUpdateRejected = false;
	try {
		buildTaskArgs({ action: "update", id: taskId, description: "Not supported by tandem update" });
	} catch (err) {
		unsupportedUpdateRejected = err instanceof Error && err.message.includes("description") && err.message.includes("Supported update fields");
	}
	assert(unsupportedUpdateRejected, "unsupported update fields should fail before invoking tandem update");

	const updateOutput = await runTandem(tandem, buildTaskArgs({
		action: "update",
		id: taskId,
		title: "Updated smoke task",
		priority: "high",
		assignee: "pi-tandem-smoke",
		dueDate: "2026-07-01",
		tags: ["smoke", "metadata"],
		relatedFiles: ["extensions/pi-tandem/index.ts"],
	}), workspace);
	assert(updateOutput.includes("Updated"), "update should report changed metadata");
	const updated = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: taskId }), workspace));
	assert(updated.data.document.title === "Updated smoke task", "update should change title");
	assert(updated.data.document.priority === "high", "update should change priority");
	assert(updated.data.document.tags.includes("metadata"), "update should append tags");
	assert(updated.data.document.relatedFiles.includes("extensions/pi-tandem/index.ts"), "show JSON should expose relatedFiles");

	await runTandem(tandem, buildTaskArgs({ action: "move", id: taskId, state: "in-progress" }), workspace);
	const moved = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: taskId }), workspace));
	assert(moved.data.document.state === "in-progress", "move should update task state");

	await runTandem(tandem, buildAccordArgs({
		action: "ready",
		id: taskId,
		assignee: "pi-tandem-smoke",
		deliverables: ["test:extensions/pi-tandem/tests/smoke.ts:smoke exercise"],
		validations: ["bun extensions/pi-tandem/tests/smoke.ts"],
	}), workspace);
	await runTandem(tandem, buildAccordArgs({ action: "claim", id: taskId, assignee: "pi-tandem-smoke" }), workspace);
	await runTandem(tandem, buildAccordArgs({
		action: "deliver",
		id: taskId,
		summary: "Smoke delivery",
		evidence: ["tandem wrapper command paths executed"],
		filesChanged: ["extensions/pi-tandem/index.ts"],
	}), workspace);

	await runTandem(tandem, buildRulesArgs({ action: "add", category: "always", rule: "Smoke rule" }), workspace);
	const rules = parseJson(await runTandem(tandem, buildRulesArgs({ action: "list" }), workspace));
	assert(rules.data.counts.total === 1, "rules list should include added rule");
	await runTandem(tandem, buildRulesArgs({ action: "edit", category: "always", id: 1, rule: "Edited smoke rule" }), workspace);
	await runTandem(tandem, buildRulesArgs({ action: "delete", category: "always", id: 1 }), workspace);

	const decisionOutput = await runTandem(tandem, buildDecisionArgs({
		action: "add",
		title: "Smoke decision",
		body: "## Decision\nExercise pi-tandem decision command mapping.",
		status: "accepted",
		date: "2026-07-01",
		deciders: ["pi-tandem-smoke"],
		context: "Exercise ADR-compatible decision metadata.",
		consequences: ["Decision adapter flags are covered."],
		alternatives: ["Only test title/body."],
		references: [taskId],
		tags: ["smoke"],
	}), workspace);
	const decisionId = parseId(decisionOutput);
	const decision = parseJson(await runTandem(tandem, buildDecisionArgs({ action: "show", id: decisionId }), workspace));
	assert(decision.data.decision.title === "Smoke decision", "decision show should return created decision");
	assert(decision.data.decision.status === "accepted", "decision show should expose ADR status");
	assert(decision.data.decision.deciders.includes("pi-tandem-smoke"), "decision show should expose deciders");

	const search = parseJson(await runTandem(tandem, buildSearchArgs({ query: "Smoke" }), workspace));
	assert(search.data.results.length >= 2, "search should find smoke task and decision");

	await runTandem(tandem, buildTaskArgs({
		action: "complete",
		id: taskId,
		summary: "Smoke complete",
		filesChanged: ["extensions/pi-tandem/index.ts"],
		validation: "smoke passed",
	}), workspace);
	const logs = parseJson(await runTandem(tandem, buildLogArgs({ action: "list" }), workspace));
	assert(logs.data.count === 1, "log list should include completed task");
	const logSearch = parseJson(await runTandem(tandem, buildLogArgs({ action: "search", query: "Smoke complete" }), workspace));
	assert(logSearch.data.results.length === 1, "log search should find completed smoke task");

	console.log(`pi-tandem smoke passed with ${tandem}`);
} finally {
	await rm(workspace, { recursive: true, force: true });
}
