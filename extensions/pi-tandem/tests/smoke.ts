import { existsSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
	buildAccordArgs,
	buildDecisionArgs,
	buildLogArgs,
	buildRulesArgs,
	buildSearchArgs,
	buildTaskArgs,
} from "../index";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");
const localTdm = join(repoRoot, "tandem-tui", "target", "debug", process.platform === "win32" ? "tdm.exe" : "tdm");

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

async function ensureTdm(): Promise<string> {
	const envBin = process.env.TANDEM_TDM_BIN || process.env.TDM_BIN;
	if (envBin) return envBin;
	if (!existsSync(localTdm)) {
		console.log("Building local tdm for smoke test...");
		await runProcess("cargo", ["build", "--manifest-path", join(repoRoot, "tandem-tui", "Cargo.toml")], repoRoot);
	}
	return existsSync(localTdm) ? localTdm : "tdm";
}

async function runTdm(tdm: string, args: string[], cwd: string): Promise<string> {
	return runProcess(tdm, args, cwd);
}

function parseJson(text: string): any {
	return JSON.parse(text.trim());
}

function parseId(output: string): string {
	const match = /^ID:\s+(\S+)/m.exec(output);
	assert(match, `could not parse ID from output:\n${output}`);
	return match[1];
}

const tdm = await ensureTdm();
const workspace = await mkdtemp(join(tmpdir(), "pi-tandem-smoke-"));

try {
	await runTdm(tdm, ["init", "--title", "Pi Tandem Smoke"], workspace);

	const emptyList = parseJson(await runTdm(tdm, buildTaskArgs({ action: "list" }), workspace));
	assert(emptyList.ok === true, "task list JSON should be ok");
	assert(emptyList.data.counts.total === 0, "new workspace should start with zero active items");

	const addOutput = await runTdm(tdm, buildTaskArgs({
		action: "add",
		title: "Smoke task",
		state: "todo",
		description: "Created by pi-tandem smoke test.",
		priority: "medium",
		tags: ["smoke"],
	}), workspace);
	const taskId = parseId(addOutput);

	const shown = parseJson(await runTdm(tdm, buildTaskArgs({ action: "show", id: taskId }), workspace));
	assert(shown.data.document.title === "Smoke task", "show should return created task");

	await runTdm(tdm, buildAccordArgs({
		action: "ready",
		id: taskId,
		assignee: "pi-tandem-smoke",
		deliverables: ["test:extensions/pi-tandem/tests/smoke.ts:smoke exercise"],
		validations: ["bun extensions/pi-tandem/tests/smoke.ts"],
	}), workspace);
	await runTdm(tdm, buildAccordArgs({ action: "claim", id: taskId, assignee: "pi-tandem-smoke" }), workspace);
	await runTdm(tdm, buildAccordArgs({
		action: "deliver",
		id: taskId,
		summary: "Smoke delivery",
		evidence: ["tdm wrapper command paths executed"],
		filesChanged: ["extensions/pi-tandem/index.ts"],
	}), workspace);

	await runTdm(tdm, buildRulesArgs({ action: "add", category: "always", rule: "Smoke rule" }), workspace);
	const rules = parseJson(await runTdm(tdm, buildRulesArgs({ action: "list" }), workspace));
	assert(rules.data.counts.total === 1, "rules list should include added rule");
	await runTdm(tdm, buildRulesArgs({ action: "edit", category: "always", id: 1, rule: "Edited smoke rule" }), workspace);
	await runTdm(tdm, buildRulesArgs({ action: "delete", category: "always", id: 1 }), workspace);

	const decisionOutput = await runTdm(tdm, buildDecisionArgs({
		action: "add",
		title: "Smoke decision",
		body: "## Decision\nExercise pi-tandem decision command mapping.",
		references: [taskId],
		tags: ["smoke"],
	}), workspace);
	const decisionId = parseId(decisionOutput);
	const decision = parseJson(await runTdm(tdm, buildDecisionArgs({ action: "show", id: decisionId }), workspace));
	assert(decision.data.decision.title === "Smoke decision", "decision show should return created decision");

	const search = parseJson(await runTdm(tdm, buildSearchArgs({ query: "Smoke" }), workspace));
	assert(search.data.results.length >= 2, "search should find smoke task and decision");

	await runTdm(tdm, buildTaskArgs({
		action: "complete",
		id: taskId,
		summary: "Smoke complete",
		filesChanged: ["extensions/pi-tandem/index.ts"],
		validation: "smoke passed",
	}), workspace);
	const logs = parseJson(await runTdm(tdm, buildLogArgs({ action: "list" }), workspace));
	assert(logs.data.count === 1, "log list should include completed task");
	const logSearch = parseJson(await runTdm(tdm, buildLogArgs({ action: "search", query: "Smoke complete" }), workspace));
	assert(logSearch.data.results.length === 1, "log search should find completed smoke task");

	console.log(`pi-tandem smoke passed with ${tdm}`);
} finally {
	await rm(workspace, { recursive: true, force: true });
}
