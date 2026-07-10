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
	if (!existsSync(localTandem)) {
		console.log("Building local tandem for relationship smoke test...");
		await runProcessOk("cargo", ["build", "--manifest-path", join(repoRoot, "tandem", "Cargo.toml")], repoRoot);
	}
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

function assertSchemaDescribesRelationshipFields(): void {
	const properties = (tandemTaskParameters as any).properties ?? {};
	for (const field of ["parent", "blockers", "references", "relatedFiles", "subtasks"]) {
		assert(properties[field], `tandem_task schema should expose ${field}`);
		assert(typeof properties[field].description === "string" && properties[field].description.length > 20, `tandem_task schema should describe relationship field ${field}`);
	}
}

assertSchemaDescribesRelationshipFields();

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
		description: "Parent/supertask created through pi-tandem task arguments.",
		relatedFiles: ["docs/architecture.md"],
		subtasks: ["Define relationship graph", "Review linked work"],
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
	const childOutput = await runTandem(tandem, childArgs, workspace);
	const childId = parseId(childOutput);

	const followUpOutput = await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Validate relationship guidance",
		state: "validation",
		parent: parentId,
		blockers: [childId],
		references: [decisionId, blockerId],
		relatedFiles: ["extensions/pi-tandem/pi-tandem.md"],
		tags: ["relationship-smoke"],
	}), workspace);
	const followUpId = parseId(followUpOutput);

	const shown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: childId }), workspace));
	assert(shown.ok === true, "tandem_task show JSON should be ok for relationship child");
	assert(shown.data.document.id === childId, "tandem_task show should return child id");
	assert(shown.data.document.title === "Implement relationship display", "tandem_task show should return child title");
	assert(shown.data.document.parentId === parentId, "tandem_task show should return child parentId");
	assert(Array.isArray(shown.data.document.blockers) && shown.data.document.blockers.includes(blockerId), "tandem_task show should return child blockers");

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

	const fileSearch = parseJson(await runTandem(tandem, buildSearchArgs({ query: "src/relationships.ts" }), workspace));
	assert(resultIds(fileSearch).has(childId), "tandem_search should find child by relatedFiles path");
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
	console.log(`Verified graph: ${parentId} -> ${blockerId}/${childId}/${followUpId}; blocker chain ${blockerId} -> ${childId} -> ${followUpId}; decision ${decisionId} referenced by all children.`);
} finally {
	await rm(workspace, { recursive: true, force: true });
}
