import { existsSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
	ACCORD_ACTIONS,
	buildAccordArgs,
	buildDecisionArgs,
	buildInitArgs,
	buildLogArgs,
	buildRulesArgs,
	buildSearchArgs,
	buildTaskArgs,
	tandemAccordParameters,
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
assert(taskSchemaProperties.reason, "tandem_task schema should expose a reason for cancel actions");
assert(taskSchemaProperties.body, "tandem_task schema should expose exact body replacement for update actions");
assert(taskSchemaProperties.parent, "tandem_task schema should expose parent for canonical hierarchy creation");
assert(taskSchemaProperties.kind, "tandem_task schema should expose kind for Epic creation");
assert(!taskSchemaProperties.subtasks, "tandem_task schema should not expose deprecated inline subtask authoring");

const acceptedAccordActions = ["claim", "deliver", "accept", "rework", "block", "fail"];
assert(JSON.stringify(ACCORD_ACTIONS) === JSON.stringify(acceptedAccordActions), "tandem_accord should expose only the accepted action list");
const accordSchemaActions = (tandemAccordParameters as any).properties?.action?.enum;
assert(JSON.stringify(accordSchemaActions) === JSON.stringify(acceptedAccordActions), "tandem_accord schema should expose only the accepted action list");
const claimAccordArgs = buildAccordArgs({ action: "claim", id: "task-1", assignee: "pi-tandem-smoke" });
assert(claimAccordArgs.join(" ") === "accord claim task-1 --assignee pi-tandem-smoke", "tandem_accord should build accepted actions");
let retiredReadyRejected = false;
try {
	buildAccordArgs({ action: "ready", id: "task-1" } as any);
} catch (err) {
	retiredReadyRejected = err instanceof Error && err.message.includes("unsupported tandem_accord action: ready");
}
assert(retiredReadyRejected, "tandem_accord builder should reject the retired ready action");

const initArgs = buildInitArgs({ title: "Pi Tandem Smoke" });
assert(initArgs.join(" ") === "init --title Pi Tandem Smoke", "tandem_init builder should map to init --title");

const epicArgs = buildTaskArgs({ action: "add", title: "Canonical Epic", kind: "epic" });
assert(epicArgs.join(" ") === "add --title Canonical Epic --kind epic", "tandem_task add builder should map Epic kind directly");

const updateArgs = buildTaskArgs({ action: "update", id: "task-1", kind: "epic", priority: "high", parent: "task-2", tags: ["cli"] });
assert(updateArgs.join(" ") === "update task-1 --kind epic --priority high --parent task-2 --tag cli", "tandem_task update builder should map kind, metadata, and parent flags");

for (const body of ["", "   ", "- first item\n\nUnicode: café 🦀\n"]) {
	const bodyArgs = buildTaskArgs({ action: "update", id: "task-1", body });
	const flagIndex = bodyArgs.indexOf("--body");
	assert(flagIndex >= 0 && bodyArgs[flagIndex + 1] === body, "tandem_task update should preserve exact body values without trimming");
}

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
const cancelArgs = buildTaskArgs({ action: "cancel", id: "task-2", reason: "Created by mistake" });
assert(cancelArgs.join(" ") === "cancel task-2 --reason Created by mistake", "tandem_task cancel builder should map id and reason");

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

	const exactBody = "- first item\n\nUnicode: café 🦀\n";
	const bodyUpdateOutput = await runTandem(tandem, buildTaskArgs({ action: "update", id: taskId, body: exactBody }), workspace);
	assert(bodyUpdateOutput.includes("body: changed"), "body update should report a content-free change summary");
	assert(!bodyUpdateOutput.includes("first item"), "body update output should not echo body contents");
	const bodyUpdated = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: taskId }), workspace));
	assert(bodyUpdated.data.body === exactBody, "show JSON should round-trip the exact updated body");
	const bodyNoopOutput = await runTandem(tandem, buildTaskArgs({ action: "update", id: taskId, body: exactBody }), workspace);
	assert(bodyNoopOutput.includes("No changes"), "byte-identical body replacement should be a no-op");
	await runTandem(tandem, buildTaskArgs({ action: "update", id: taskId, body: "" }), workspace);
	const bodyCleared = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: taskId }), workspace));
	assert(bodyCleared.data.body === "", "an explicit empty body should clear the complete Markdown body");

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
		action: "claim",
		id: taskId,
		assignee: "pi-tandem-smoke",
		deliverables: ["test:extensions/pi-tandem/tests/smoke.ts:smoke exercise"],
		validations: ["bun extensions/pi-tandem/tests/smoke.ts"],
	}), workspace);
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

	const cancelOutput = await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Canceled smoke task",
		description: "Preserve this canceled body.",
	}), workspace);
	const canceledId = parseId(cancelOutput);
	const cancelResult = await runTandem(tandem, buildTaskArgs({ action: "cancel", id: canceledId, reason: "Created by mistake" }), workspace);
	assert(cancelResult.includes(`Canceled ${canceledId}`), "cancel should report the archived Task");
	const canceled = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: canceledId }), workspace));
	assert(canceled.data.location === "logs", "canceled Task should move to Logs");
	assert(canceled.data.document.completionOutcome === "canceled", "show JSON should identify the canceled outcome");
	assert(canceled.data.body.includes("Preserve this canceled body."), "cancel should preserve the Task body");

	await runTandem(tandem, buildTaskArgs({
		action: "complete",
		id: taskId,
		summary: "Smoke complete",
		filesChanged: ["extensions/pi-tandem/index.ts"],
		validation: "smoke passed",
	}), workspace);
	const logs = parseJson(await runTandem(tandem, buildLogArgs({ action: "list" }), workspace));
	assert(logs.data.count === 2, "log list should include completed and canceled Tasks");
	assert(logs.data.items.find((item: any) => item.id === canceledId)?.outcome === "canceled", "log list should expose canceled outcome");
	const logSearch = parseJson(await runTandem(tandem, buildLogArgs({ action: "search", query: "Smoke complete" }), workspace));
	assert(logSearch.data.results.length === 1, "log search should find completed smoke task");

	console.log(`pi-tandem smoke passed with ${tandem}`);
} finally {
	await rm(workspace, { recursive: true, force: true });
}
