import { existsSync } from "node:fs";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
	buildDecisionArgs,
	buildSearchArgs,
	buildTaskArgs,
	tandemTaskParameters,
} from "../index";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");
const localTandem = join(repoRoot, "tandem", "target", "debug", process.platform === "win32" ? "tandem.exe" : "tandem");

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

async function ensureTandem(): Promise<string> {
	const envBin = process.env.TANDEM_BIN;
	if (envBin) return envBin;
	console.log("Building current repository tandem for relationship smoke test...");
	await runProcessOk("cargo", ["build", "--manifest-path", join(repoRoot, "tandem", "Cargo.toml")], repoRoot);
	return existsSync(localTandem) ? localTandem : "tandem";
}

async function runTandem(tandem: string, args: string[], cwd: string): Promise<string> {
	return runProcessOk(tandem, args, cwd);
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

function itemById(payload: any, id: string): any {
	return (payload.data?.items ?? []).find((item: any) => item.id === id);
}

function resultById(payload: any, id: string): any {
	return (payload.data?.results ?? []).find((result: any) => result.id === id);
}

function assertSchemaDescribesRelationshipFields(): void {
	const properties = (tandemTaskParameters as any).properties ?? {};
	for (const field of ["parent", "blockers", "references", "relatedFiles"]) {
		assert(properties[field], `tandem_task schema should expose ${field}`);
		assert(typeof properties[field].description === "string" && properties[field].description.length > 20, `tandem_task schema should describe relationship field ${field}`);
	}
	assert(!properties.subtasks, "tandem_task schema should not offer deprecated inline subtask authoring");
	assert(properties.parent.description.includes("first-class tracked subtask"), "parent schema guidance should describe tracked child tasks");
}

assertSchemaDescribesRelationshipFields();

let legacySubtasksRejected = false;
try {
	buildTaskArgs({ action: "add", title: "Legacy inline checklist", subtasks: ["Do work"] } as any);
} catch (err) {
	legacySubtasksRejected = err instanceof Error && err.message.includes("deprecated inline subtasks") && err.message.includes("parent=<task-id>");
}
assert(legacySubtasksRejected, "buildTaskArgs should reject legacy inline subtask authoring instead of forwarding --subtask");

const tandem = await ensureTandem();
const workspace = await mkdtemp(join(tmpdir(), "pi-tandem-relationships-"));

try {
	await runTandem(tandem, ["init", "--title", "Pi Tandem Relationship Smoke"], workspace);
	await mkdir(join(workspace, "docs"), { recursive: true });
	await mkdir(join(workspace, "src"), { recursive: true });
	await mkdir(join(workspace, "tests", "fixtures"), { recursive: true });
	await writeFile(join(workspace, "docs", "architecture.md"), "# Relationship smoke architecture\n", "utf8");
	await writeFile(join(workspace, "src", "relationships.ts"), "export const relationshipSmoke = true;\n", "utf8");
	await writeFile(join(workspace, "tests", "fixtures", "relationship.json"), "{\"ok\":true}\n", "utf8");

	const parentOutput = await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Relationship smoke parent",
		description: "Parent task whose independently tracked children are linked through parentId.",
		relatedFiles: ["docs/architecture.md"],
		tags: ["relationship-smoke"],
	}), workspace);
	const parentId = parseId(parentOutput);

	const decisionOutput = await runTandem(tandem, buildDecisionArgs({
		action: "add",
		title: "Relationship smoke decision",
		body: "## Decision\nUse explicit Tandem relationship fields in agent-created work.",
		references: [parentId],
		tags: ["relationship-smoke"],
	}), workspace);
	const decisionId = parseId(decisionOutput);

	const blockerOutput = await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Prepare relationship fixtures",
		parent: parentId,
		references: [decisionId],
		relatedFiles: ["tests/fixtures/relationship.json"],
		tags: ["relationship-smoke"],
	}), workspace);
	const blockerId = parseId(blockerOutput);

	const childParams = {
		action: "add" as const,
		title: "Implement relationship display",
		description: "First-class child task with its own blocker, references, files, and lifecycle.",
		parent: parentId,
		blockers: [blockerId],
		references: [decisionId, parentId],
		relatedFiles: ["src/relationships.ts", "docs/architecture.md"],
		tags: ["relationship-smoke"],
	};
	const childArgs = buildTaskArgs(childParams);
	assert(childArgs.includes("--parent") && childArgs.includes(parentId), "buildTaskArgs should pass --parent");
	assert(childArgs.filter((arg) => arg === "--blocker").length === 1, "buildTaskArgs should pass one --blocker");
	assert(childArgs.filter((arg) => arg === "--reference").length === 2, "buildTaskArgs should pass repeated --reference flags");
	assert(childArgs.filter((arg) => arg === "--related-file").length === 2, "buildTaskArgs should pass repeated --related-file flags");
	assert(!childArgs.includes("--subtask"), "buildTaskArgs should never forward deprecated --subtask authoring");
	const childOutput = await runTandem(tandem, childArgs, workspace);
	const childId = parseId(childOutput);

	const followUpOutput = await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Validate relationship guidance",
		state: "validation",
		blockers: [childId],
		references: [decisionId, blockerId],
		relatedFiles: ["extensions/pi-tandem/pi-tandem.md"],
		tags: ["relationship-smoke"],
	}), workspace);
	const followUpId = parseId(followUpOutput);
	const attachArgs = buildTaskArgs({ action: "update", id: followUpId, parent: parentId });
	assert(attachArgs.includes("--parent") && attachArgs.includes(parentId), "buildTaskArgs update should pass --parent for attach/reparent");
	await runTandem(tandem, attachArgs, workspace);

	const shown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: childId }), workspace));
	assert(shown.ok === true, "tandem_task show JSON should be ok for relationship child");
	assert(shown.data.document.id === childId, "tandem_task show should return child id");
	assert(shown.data.document.parentId === parentId, "tandem_task show should return child parentId");
	assert(shown.data.parentRelationship === "subtask", "tandem_task show should classify task-parent links as subtasks");
	assert(Array.isArray(shown.data.subtasks) && shown.data.subtasks.length === 0, "child show should naturally include an empty computed subtask summary");
	assert(Array.isArray(shown.data.document.blockers) && shown.data.document.blockers.includes(blockerId), "tandem_task show should return child blockers");

	const parentShown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: parentId }), workspace));
	assert(parentShown.data.document.id === parentId, "parent show should return the parent task");
	assert(Array.isArray(parentShown.data.subtasks), "parent show should naturally expose computed subtask summaries");
	assert(parentShown.data.subtasks.length === 3, "parent show should include all three parent-linked children");
	for (const id of [blockerId, childId, followUpId]) {
		const summary = parentShown.data.subtasks.find((subtask: any) => subtask.id === id);
		assert(summary, `parent show should include computed summary for ${id}`);
		assert(typeof summary.title === "string" && typeof summary.location === "string", `computed summary for ${id} should include title/location`);
	}

	const listed = parseJson(await runTandem(tandem, buildTaskArgs({ action: "list" }), workspace));
	for (const id of [blockerId, childId, followUpId]) {
		const item = itemById(listed, id);
		assert(item?.parentId === parentId, `tandem_task list should return parentId for ${id}`);
		assert(item?.parentRelationship === "subtask", `tandem_task list should return subtask relationship for ${id}`);
	}
	const parentFilteredList = parseJson(await runTandem(tandem, buildTaskArgs({ action: "list", parent: parentId }), workspace));
	assert(parentFilteredList.data.counts.total === 3, "tandem_task list parent filter should return all tracked children");

	const parentFile = await readFile(join(workspace, ".tandem", "board", `${parentId}.md`), "utf8");
	assertIncludes(parentFile, "relatedFiles: [\"docs/architecture.md\"]", "parent task");
	assert(!parentFile.includes("\nsubtasks:"), "parent task should not contain legacy inline subtasks metadata");

	const blockerFile = await readFile(join(workspace, ".tandem", "board", `${blockerId}.md`), "utf8");
	assertIncludes(blockerFile, `parentId: \"${parentId}\"`, "blocker task");
	assertIncludes(blockerFile, `references: [\"${decisionId}\"]`, "blocker task");
	assertIncludes(blockerFile, "relatedFiles: [\"tests/fixtures/relationship.json\"]", "blocker task");
	assert(!blockerFile.includes("\nsubtasks:"), "blocker child should not contain legacy inline subtasks metadata");

	const childFile = await readFile(join(workspace, ".tandem", "board", `${childId}.md`), "utf8");
	assertIncludes(childFile, `parentId: \"${parentId}\"`, "child task");
	assertIncludes(childFile, `blockers: [\"${blockerId}\"]`, "child task");
	assertIncludes(childFile, `references: [\"${decisionId}\", \"${parentId}\"]`, "child task");
	assertIncludes(childFile, "relatedFiles: [\"src/relationships.ts\", \"docs/architecture.md\"]", "child task");
	assert(!childFile.includes("\nsubtasks:"), "implementation child should not contain legacy inline subtasks metadata");

	const followUpFile = await readFile(join(workspace, ".tandem", "board", `${followUpId}.md`), "utf8");
	assertIncludes(followUpFile, `parentId: \"${parentId}\"`, "follow-up task");
	assertIncludes(followUpFile, `blockers: [\"${childId}\"]`, "follow-up task");
	assertIncludes(followUpFile, `references: [\"${decisionId}\", \"${blockerId}\"]`, "follow-up task");
	assertIncludes(followUpFile, "relatedFiles: [\"extensions/pi-tandem/pi-tandem.md\"]", "follow-up task");

	const fileSearch = parseJson(await runTandem(tandem, buildSearchArgs({ query: "src/relationships.ts" }), workspace));
	assert(resultIds(fileSearch).has(childId), "tandem_search should find child by relatedFiles path");
	const childSearchResult = resultById(fileSearch, childId);
	assert(childSearchResult?.parentId === parentId, "tandem_search should naturally return child parentId");
	assert(childSearchResult?.parentRelationship === "subtask", "tandem_search should naturally return child relationship classification");

	const parentFilteredSearch = parseJson(await runTandem(tandem, buildSearchArgs({ query: "relationship-smoke", parent: parentId }), workspace));
	assert(resultIds(parentFilteredSearch).size === 3, "tandem_search parent filter should return all tagged tracked children");
	for (const result of parentFilteredSearch.data.results) {
		assert(result.parentId === parentId && result.parentRelationship === "subtask", "parent-filtered search results should expose CLI relationship fields");
	}

	const decisionSearch = parseJson(await runTandem(tandem, buildSearchArgs({ query: decisionId }), workspace));
	assert(resultIds(decisionSearch).has(blockerId), "tandem_search should find blocker by decision reference");
	assert(resultIds(decisionSearch).has(childId), "tandem_search should find child by decision reference");
	assert(resultIds(decisionSearch).has(followUpId), "tandem_search should find follow-up by decision reference");

	const missingParent = await runProcess(tandem, buildTaskArgs({ action: "add", title: "Bad missing parent", parent: "task-999" }), workspace);
	assert(missingParent.code !== 0, "tandem_task add should reject unresolved parent core reference");
	assertIncludes(`${missingParent.stdout}\n${missingParent.stderr}`, "Validation failed: parent document not found: task-999", "missing parent failure");

	const looseReferenceOutput = await runTandem(tandem, buildTaskArgs({ action: "add", title: "Loose related reference", references: ["task-999"] }), workspace);
	assertIncludes(looseReferenceOutput, "Warning: reference not found: task-999", "loose reference warning");

	console.log(`pi-tandem relationship smoke passed with ${tandem}`);
	console.log(`Verified parent-linked children: ${parentId} -> ${blockerId}/${childId}/${followUpId}; blocker chain ${blockerId} -> ${childId} -> ${followUpId}; decision ${decisionId} referenced by all children.`);
} finally {
	await rm(workspace, { recursive: true, force: true });
}
