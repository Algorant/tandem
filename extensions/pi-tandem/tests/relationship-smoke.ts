import { existsSync } from "node:fs";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
	buildDecisionArgs,
	buildSearchArgs,
	buildTaskArgs,
	tdmTaskParameters,
} from "../index";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");
const localTdm = join(repoRoot, "tandem-tui", "target", "debug", process.platform === "win32" ? "tdm.exe" : "tdm");

type ProcessResult = {
	stdout: string;
	stderr: string;
	code: number;
};

function assert(condition: unknown, message: string): asserts condition {
	if (!condition) throw new Error(message);
}

async function runProcess(command: string, args: string[], cwd: string): Promise<ProcessResult> {
	const proc = Bun.spawn([command, ...args], { cwd, stdout: "pipe", stderr: "pipe" });
	const stdout = await new Response(proc.stdout).text();
	const stderr = await new Response(proc.stderr).text();
	const code = await proc.exited;
	return { stdout, stderr, code };
}

async function runProcessOk(command: string, args: string[], cwd: string): Promise<string> {
	const result = await runProcess(command, args, cwd);
	if (result.code !== 0) {
		throw new Error(`command failed (${result.code}): ${command} ${args.join(" ")}\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`);
	}
	return result.stdout;
}

async function ensureTdm(): Promise<string> {
	const envBin = process.env.TANDEM_TDM_BIN || process.env.TDM_BIN;
	if (envBin) return envBin;
	if (!existsSync(localTdm)) {
		console.log("Building local tdm for relationship smoke test...");
		await runProcessOk("cargo", ["build", "--manifest-path", join(repoRoot, "tandem-tui", "Cargo.toml")], repoRoot);
	}
	return existsSync(localTdm) ? localTdm : "tdm";
}

async function runTdm(tdm: string, args: string[], cwd: string): Promise<string> {
	return runProcessOk(tdm, args, cwd);
}

function parseJson(text: string): any {
	return JSON.parse(text.trim());
}

function parseId(output: string): string {
	const match = /^ID:\s+(\S+)/m.exec(output);
	assert(match, `could not parse ID from output:\n${output}`);
	return match[1];
}

function assertIncludes(haystack: string, needle: string, label: string): void {
	assert(haystack.includes(needle), `${label} missing ${JSON.stringify(needle)} in:\n${haystack}`);
}

function resultIds(searchPayload: any): Set<string> {
	return new Set((searchPayload.data?.results ?? []).map((result: any) => result.id));
}

function assertSchemaDescribesRelationshipFields(): void {
	const properties = (tdmTaskParameters as any).properties ?? {};
	for (const field of ["parent", "blockers", "references", "relatedFiles", "subtasks"]) {
		assert(properties[field], `tdm_task schema should expose ${field}`);
		assert(typeof properties[field].description === "string" && properties[field].description.length > 20, `tdm_task schema should describe relationship field ${field}`);
	}
}

assertSchemaDescribesRelationshipFields();

const tdm = await ensureTdm();
const workspace = await mkdtemp(join(tmpdir(), "pi-tandem-relationships-"));

try {
	await runTdm(tdm, ["init", "--title", "Pi Tandem Relationship Smoke"], workspace);
	await mkdir(join(workspace, "docs"), { recursive: true });
	await mkdir(join(workspace, "src"), { recursive: true });
	await mkdir(join(workspace, "tests", "fixtures"), { recursive: true });
	await writeFile(join(workspace, "docs", "architecture.md"), "# Relationship smoke architecture\n", "utf8");
	await writeFile(join(workspace, "src", "relationships.ts"), "export const relationshipSmoke = true;\n", "utf8");
	await writeFile(join(workspace, "tests", "fixtures", "relationship.json"), "{\"ok\":true}\n", "utf8");

	const parentOutput = await runTdm(tdm, buildTaskArgs({
		action: "add",
		title: "Relationship smoke parent",
		description: "Parent/supertask created through pi-tandem task arguments.",
		relatedFiles: ["docs/architecture.md"],
		subtasks: ["Define relationship graph", "Review linked work"],
		tags: ["relationship-smoke"],
	}), workspace);
	const parentId = parseId(parentOutput);

	const decisionOutput = await runTdm(tdm, buildDecisionArgs({
		action: "add",
		title: "Relationship smoke decision",
		body: "## Decision\nUse explicit Tandem relationship fields in agent-created work.",
		references: [parentId],
		tags: ["relationship-smoke"],
	}), workspace);
	const decisionId = parseId(decisionOutput);

	const blockerOutput = await runTdm(tdm, buildTaskArgs({
		action: "add",
		title: "Prepare relationship fixtures",
		parent: parentId,
		references: [decisionId],
		relatedFiles: ["tests/fixtures/relationship.json"],
		subtasks: ["Create fixture", "Document fixture"],
		tags: ["relationship-smoke"],
	}), workspace);
	const blockerId = parseId(blockerOutput);

	const childParams = {
		action: "add" as const,
		title: "Implement relationship display",
		description: "Child task with parent, blocker, references, related files, and subtasks.",
		parent: parentId,
		blockers: [blockerId],
		references: [decisionId, parentId],
		relatedFiles: ["src/relationships.ts", "docs/architecture.md"],
		subtasks: ["Render parent link", "Render blockers", "Render references"],
		tags: ["relationship-smoke"],
	};
	const childArgs = buildTaskArgs(childParams);
	assert(childArgs.includes("--parent") && childArgs.includes(parentId), "buildTaskArgs should pass --parent");
	assert(childArgs.filter((arg) => arg === "--blocker").length === 1, "buildTaskArgs should pass one --blocker");
	assert(childArgs.filter((arg) => arg === "--reference").length === 2, "buildTaskArgs should pass repeated --reference flags");
	assert(childArgs.filter((arg) => arg === "--related-file").length === 2, "buildTaskArgs should pass repeated --related-file flags");
	assert(childArgs.filter((arg) => arg === "--subtask").length === 3, "buildTaskArgs should pass repeated --subtask flags");
	const childOutput = await runTdm(tdm, childArgs, workspace);
	const childId = parseId(childOutput);

	const followUpOutput = await runTdm(tdm, buildTaskArgs({
		action: "add",
		title: "Review relationship guidance",
		state: "review",
		parent: parentId,
		blockers: [childId],
		references: [decisionId, blockerId],
		relatedFiles: ["extensions/pi-tandem/pi-tandem.md"],
		tags: ["relationship-smoke"],
	}), workspace);
	const followUpId = parseId(followUpOutput);

	const shown = parseJson(await runTdm(tdm, buildTaskArgs({ action: "show", id: childId }), workspace));
	assert(shown.ok === true, "tdm_task show JSON should be ok for relationship child");
	assert(shown.data.document.id === childId, "tdm_task show should return child id");
	assert(shown.data.document.title === "Implement relationship display", "tdm_task show should return child title");
	if (!("parentId" in shown.data.document) || !("blockers" in shown.data.document)) {
		console.log("Note: tdm show --json currently omits relationship fields; smoke validates raw Tandem documents and search visibility.");
	}

	const parentFile = await readFile(join(workspace, ".tandem", "board", `${parentId}.md`), "utf8");
	assertIncludes(parentFile, "relatedFiles: [\"docs/architecture.md\"]", "parent task");
	assertIncludes(parentFile, `  - id: ${parentId}-1`, "parent subtasks");
	assertIncludes(parentFile, "    title: \"Define relationship graph\"", "parent subtasks");

	const blockerFile = await readFile(join(workspace, ".tandem", "board", `${blockerId}.md`), "utf8");
	assertIncludes(blockerFile, `parentId: \"${parentId}\"`, "blocker task");
	assertIncludes(blockerFile, `references: [\"${decisionId}\"]`, "blocker task");
	assertIncludes(blockerFile, "relatedFiles: [\"tests/fixtures/relationship.json\"]", "blocker task");

	const childFile = await readFile(join(workspace, ".tandem", "board", `${childId}.md`), "utf8");
	assertIncludes(childFile, `parentId: \"${parentId}\"`, "child task");
	assertIncludes(childFile, `blockers: [\"${blockerId}\"]`, "child task");
	assertIncludes(childFile, `references: [\"${decisionId}\", \"${parentId}\"]`, "child task");
	assertIncludes(childFile, "relatedFiles: [\"src/relationships.ts\", \"docs/architecture.md\"]", "child task");
	assertIncludes(childFile, `  - id: ${childId}-1`, "child subtasks");
	assertIncludes(childFile, "    title: \"Render parent link\"", "child subtasks");
	assertIncludes(childFile, "    title: \"Render blockers\"", "child subtasks");
	assertIncludes(childFile, "    title: \"Render references\"", "child subtasks");

	const followUpFile = await readFile(join(workspace, ".tandem", "board", `${followUpId}.md`), "utf8");
	assertIncludes(followUpFile, `parentId: \"${parentId}\"`, "follow-up task");
	assertIncludes(followUpFile, `blockers: [\"${childId}\"]`, "follow-up task");
	assertIncludes(followUpFile, `references: [\"${decisionId}\", \"${blockerId}\"]`, "follow-up task");
	assertIncludes(followUpFile, "relatedFiles: [\"extensions/pi-tandem/pi-tandem.md\"]", "follow-up task");

	const fileSearch = parseJson(await runTdm(tdm, buildSearchArgs({ query: "src/relationships.ts" }), workspace));
	assert(resultIds(fileSearch).has(childId), "tdm_search should find child by relatedFiles path");
	const decisionSearch = parseJson(await runTdm(tdm, buildSearchArgs({ query: decisionId }), workspace));
	assert(resultIds(decisionSearch).has(blockerId), "tdm_search should find blocker by decision reference");
	assert(resultIds(decisionSearch).has(childId), "tdm_search should find child by decision reference");
	assert(resultIds(decisionSearch).has(followUpId), "tdm_search should find follow-up by decision reference");

	const missingParent = await runProcess(tdm, buildTaskArgs({ action: "add", title: "Bad missing parent", parent: "task-999" }), workspace);
	assert(missingParent.code !== 0, "tdm_task add should reject unresolved parent core reference");
	assertIncludes(`${missingParent.stdout}\n${missingParent.stderr}`, "Validation failed: parent document not found: task-999", "missing parent failure");

	const looseReferenceOutput = await runTdm(tdm, buildTaskArgs({ action: "add", title: "Loose related reference", references: ["task-999"] }), workspace);
	assertIncludes(looseReferenceOutput, "Warning: reference not found: task-999", "loose reference warning");

	console.log(`pi-tandem relationship smoke passed with ${tdm}`);
	console.log(`Verified graph: ${parentId} -> ${blockerId}/${childId}/${followUpId}; blocker chain ${blockerId} -> ${childId} -> ${followUpId}; decision ${decisionId} referenced by all children.`);
} finally {
	await rm(workspace, { recursive: true, force: true });
}
