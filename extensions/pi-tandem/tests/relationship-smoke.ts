import { existsSync } from "node:fs";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
	buildDecisionArgs,
	buildLogArgs,
	buildSearchArgs,
	buildTaskArgs,
	tandemPromptGuidance,
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

function assertFailure(result: ProcessResult, label: string, ...needles: string[]): void {
	assert(result.code !== 0, `${label} should fail`);
	const output = `${result.stdout}\n${result.stderr}`;
	for (const needle of needles) assertIncludes(output, needle, label);
}

function itemById(payload: any, id: string): any {
	return (payload.data?.items ?? []).find((item: any) => item.id === id);
}

function resultById(payload: any, id: string): any {
	return (payload.data?.results ?? []).find((result: any) => result.id === id);
}

function assertSchemaDescribesCanonicalHierarchy(): void {
	const properties = (tandemTaskParameters as any).properties ?? {};
	for (const field of ["kind", "parent", "blockers", "references", "relatedFiles"]) {
		assert(properties[field], `tandem_task schema should expose ${field}`);
		assert(typeof properties[field].description === "string" && properties[field].description.length > 20, `tandem_task schema should describe ${field}`);
	}
	assert(!properties.subtasks, "tandem_task schema should not offer deprecated inline subtask authoring");
	assert(properties.parent.description.includes("Epic gets a global-ID Task"), "parent schema should describe Epic -> global Task allocation");
	assert(properties.parent.description.includes("Task gets a parent-derived Subtask"), "parent schema should describe Task -> parent-derived Subtask allocation");
}

assertSchemaDescribesCanonicalHierarchy();

const generatedGuidance = tandemPromptGuidance("/tmp/canonical-tandem-workspace");
assertIncludes(generatedGuidance, "their direct children are global-ID Tasks", "generated canonical hierarchy guidance");
assertIncludes(generatedGuidance, "a Task's direct children are leaf, parent-derived", "generated canonical Subtask guidance");
assertIncludes(generatedGuidance, "Only Task-role roots are delegated initially", "generated Task-only delegation guidance");
assertIncludes(generatedGuidance, "Never allocate IDs or reclassify CLI output in Pi", "generated thin-adapter guidance");

const epicArgs = buildTaskArgs({ action: "add", title: "Canonical Epic", kind: "epic" });
assert(epicArgs.includes("--kind") && epicArgs.includes("epic"), "buildTaskArgs should pass kind=epic directly to Tandem");

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
	await writeFile(join(workspace, "docs", "architecture.md"), "# Canonical hierarchy\n", "utf8");
	await writeFile(join(workspace, "src", "relationships.ts"), "export const relationshipSmoke = true;\n", "utf8");
	await writeFile(join(workspace, "tests", "fixtures", "relationship.json"), "{\"ok\":true}\n", "utf8");

	const epicOutput = await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Ship canonical relationship support",
		kind: "epic",
		relatedFiles: ["docs/architecture.md"],
		tags: ["relationship-smoke"],
	}), workspace);
	const epicId = parseId(epicOutput);
	assert(/^task-\d+$/.test(epicId), `Epic should have a global task-N ID, got ${epicId}`);

	const decisionId = parseId(await runTandem(tandem, buildDecisionArgs({
		action: "add",
		title: "Canonical relationship policy",
		body: "## Decision\nUse strict Epic -> Task -> Subtask roles.",
		references: [epicId],
		tags: ["relationship-smoke"],
	}), workspace));

	const taskParams = {
		action: "add" as const,
		title: "Implement relationship display",
		description: "Delegatable global-ID Task directly beneath an Epic.",
		parent: epicId,
		references: [decisionId],
		relatedFiles: ["src/relationships.ts", "docs/architecture.md"],
		tags: ["relationship-smoke"],
	};
	const taskArgs = buildTaskArgs(taskParams);
	assert(taskArgs.includes("--parent") && taskArgs.includes(epicId), "buildTaskArgs should pass the Epic parent directly");
	const taskOutput = await runTandem(tandem, taskArgs, workspace);
	const taskId = parseId(taskOutput);
	assert(/^task-\d+$/.test(taskId), `direct Epic child should be a global-ID Task, got ${taskId}`);
	assert(!taskId.startsWith(`${epicId}-`), `direct Epic child must not use an erroneous hierarchical ID, got ${taskId}`);
	assertIncludes(taskOutput, `Task of Epic: ${epicId}`, "Epic Task creation output");

	const blockerId = parseId(await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Prepare relationship fixtures",
		parent: taskId,
		references: [decisionId],
		relatedFiles: ["tests/fixtures/relationship.json"],
		tags: ["relationship-smoke"],
	}), workspace));
	assert(blockerId === `${taskId}-1`, `first Task child should be parent-derived Subtask ${taskId}-1, got ${blockerId}`);

	const subtaskArgs = buildTaskArgs({
		action: "add",
		title: "Render canonical relationships",
		description: "Leaf Subtask owned by the delegated Task worker.",
		parent: taskId,
		blockers: [blockerId],
		references: [decisionId, epicId],
		relatedFiles: ["src/relationships.ts"],
		tags: ["relationship-smoke"],
	});
	assert(subtaskArgs.filter((arg) => arg === "--blocker").length === 1, "buildTaskArgs should pass blockers");
	assert(subtaskArgs.filter((arg) => arg === "--reference").length === 2, "buildTaskArgs should pass repeated references");
	assert(!subtaskArgs.includes("--subtask"), "buildTaskArgs should never forward deprecated --subtask authoring");
	const subtaskId = parseId(await runTandem(tandem, subtaskArgs, workspace));
	assert(subtaskId === `${taskId}-2`, `second Task child should be parent-derived Subtask ${taskId}-2, got ${subtaskId}`);

	const genericId = parseId(await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Decision-parented implementation Task",
		parent: decisionId,
		tags: ["relationship-smoke"],
	}), workspace));
	assert(/^task-\d+$/.test(genericId), `generic-parent Task should retain global task-N allocation, got ${genericId}`);
	const genericSubtaskId = parseId(await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Generic Task checklist item",
		parent: genericId,
	}), workspace));
	assert(genericSubtaskId === `${genericId}-1`, `a generic-parent Task should own parent-derived Subtasks, got ${genericSubtaskId}`);

	const epicShown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: epicId }), workspace));
	assert(epicShown.data.document.kind === "epic", "Epic show should retain kind=epic");
	assert(Array.isArray(epicShown.data.tasks) && epicShown.data.tasks.some((task: any) => task.id === taskId), "Epic show should expose direct global Tasks in data.tasks");
	assert(epicShown.data.subtasks === undefined, "Epic show should not label direct Tasks as subtasks");

	const taskShown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: taskId }), workspace));
	assert(taskShown.data.document.parentId === epicId, "Task show should preserve Epic parentId");
	assert(taskShown.data.parentRelationship === "epic-task", "direct Epic child should consume CLI relationship epic-task");
	assert(Array.isArray(taskShown.data.subtasks), "Task show should expose its Subtask worklist");
	assert(taskShown.data.subtasks.length === 2, "Task show should expose both direct Subtasks");
	assert(taskShown.data.subtasks.some((subtask: any) => subtask.id === subtaskId), "Task show should include implementation Subtask");

	const subtaskShown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: subtaskId }), workspace));
	assert(subtaskShown.data.document.parentId === taskId, "Subtask show should preserve Task parentId");
	assert(subtaskShown.data.parentRelationship === "subtask", "Task child should consume CLI relationship subtask");
	assert(subtaskShown.data.subtasks === undefined && subtaskShown.data.tasks === undefined, "Subtasks are leaves and should expose no child collection");
	assert(Array.isArray(subtaskShown.data.document.blockers) && subtaskShown.data.document.blockers.includes(blockerId), "Subtask show should retain blockers");

	const genericShown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: genericId }), workspace));
	assert(genericShown.data.document.parentId === decisionId, "generic-parent Task should preserve parentId");
	assert(genericShown.data.parentRelationship === "parent", "decision-parented Task should consume generic parent relationship");
	assert(genericShown.data.subtasks.some((subtask: any) => subtask.id === genericSubtaskId), "generic-parent Task should expose its Subtasks");

	const listed = parseJson(await runTandem(tandem, buildTaskArgs({ action: "list" }), workspace));
	assert(itemById(listed, taskId)?.parentRelationship === "epic-task", "list should pass through epic-task");
	for (const id of [blockerId, subtaskId, genericSubtaskId]) {
		assert(itemById(listed, id)?.parentRelationship === "subtask", `list should pass through subtask for ${id}`);
	}
	assert(itemById(listed, genericId)?.parentRelationship === "parent", "list should pass through generic parent");

	const epicFiltered = parseJson(await runTandem(tandem, buildTaskArgs({ action: "list", parent: epicId }), workspace));
	assert(epicFiltered.data.counts.total === 1 && epicFiltered.data.items[0].id === taskId, "exact Epic parent filter should return its global Task");
	assert(epicFiltered.data.items[0].parentRelationship === "epic-task", "filtered list should retain epic-task");
	const taskFiltered = parseJson(await runTandem(tandem, buildTaskArgs({ action: "list", parent: taskId }), workspace));
	assert(taskFiltered.data.counts.total === 2, "exact Task parent filter should return its Subtask worklist");
	assert(taskFiltered.data.items.every((item: any) => item.parentRelationship === "subtask"), "Task-filtered list should retain subtask relationships");

	const searched = parseJson(await runTandem(tandem, buildSearchArgs({ query: "relationship-smoke" }), workspace));
	assert(resultById(searched, taskId)?.parentRelationship === "epic-task", "search should pass through epic-task");
	assert(resultById(searched, subtaskId)?.parentRelationship === "subtask", "search should pass through subtask");
	assert(resultById(searched, genericId)?.parentRelationship === "parent", "search should pass through generic parent");

	const taskFile = await readFile(join(workspace, ".tandem", "board", `${taskId}.md`), "utf8");
	assertIncludes(taskFile, `parentId: "${epicId}"`, "Epic Task file");
	assert(!taskFile.includes("\nsubtasks:"), "Task file should not contain inline checklist metadata");
	const subtaskFile = await readFile(join(workspace, ".tandem", "board", `${subtaskId}.md`), "utf8");
	assertIncludes(subtaskFile, `parentId: "${taskId}"`, "Subtask file");
	assertIncludes(subtaskFile, `blockers: ["${blockerId}"]`, "Subtask file");
	assert(!subtaskFile.includes("\nsubtasks:"), "Subtask file should not contain inline checklist metadata");

	await runTandem(tandem, buildTaskArgs({
		action: "complete",
		id: blockerId,
		summary: "Archive first Subtask for sequence continuity smoke",
	}), workspace);
	const loggedSubtask = parseJson(await runTandem(tandem, buildLogArgs({ action: "show", id: blockerId }), workspace));
	assert(loggedSubtask.data.document.id === blockerId, "completed Subtask should be readable from logs");
	assert(loggedSubtask.data.parentRelationship === "subtask", "logged Subtask should retain CLI-computed relationship");
	const successorId = parseId(await runTandem(tandem, buildTaskArgs({
		action: "add",
		title: "Continue Subtask sequence after completed log",
		parent: taskId,
	}), workspace));
	assert(successorId === `${taskId}-3`, `completed Subtask suffix should not be reused; expected ${taskId}-3, got ${successorId}`);
	const taskHistory = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: taskId }), workspace));
	const completedSummary = taskHistory.data.subtasks.find((subtask: any) => subtask.id === blockerId);
	assert(completedSummary?.location === "logs" && typeof completedSummary.completedAt === "string", "Task show should include completed Subtasks from logs with completedAt");
	const activeSummary = taskHistory.data.subtasks.find((subtask: any) => subtask.id === successorId);
	assert(activeSummary?.location === "board", "Task show should include active Subtasks from the board");

	const nestedEpic = await runProcess(tandem, buildTaskArgs({ action: "add", title: "Invalid nested Epic", kind: "epic", parent: epicId }), workspace);
	assertFailure(nestedEpic, "nested Epic", "an Epic cannot have parentId");

	const childBeneathSubtask = await runProcess(tandem, buildTaskArgs({ action: "add", title: "Invalid depth", parent: subtaskId }), workspace);
	assertFailure(childBeneathSubtask, "child beneath Subtask", `cannot attach a child beneath Subtask ${subtaskId}`);

	const standaloneId = parseId(await runTandem(tandem, buildTaskArgs({ action: "add", title: "Standalone Task" }), workspace));
	const roleChangingReparent = await runProcess(tandem, buildTaskArgs({ action: "update", id: standaloneId, parent: taskId }), workspace);
	assertFailure(roleChangingReparent, "role-changing reparent", `reparenting ${standaloneId} would change its canonical role from task to subtask`, "IDs are immutable");
	const standaloneShown = parseJson(await runTandem(tandem, buildTaskArgs({ action: "show", id: standaloneId }), workspace));
	assert(standaloneShown.data.document.parentId === undefined, "rejected reparent should not mutate the standalone Task");

	const erroneousEpicChildId = `${epicId}-99`;
	const erroneousEpicChildPath = join(workspace, ".tandem", "board", `${erroneousEpicChildId}.md`);
	await writeFile(erroneousEpicChildPath, `---\nid: ${erroneousEpicChildId}\ntype: task\ntitle: "Erroneous hierarchical Epic child"\nstate: todo\nparentId: "${epicId}"\n---\n`, "utf8");
	const erroneousCompatibilityRead = await runProcess(tandem, buildTaskArgs({ action: "list" }), workspace);
	assertFailure(erroneousCompatibilityRead, "erroneous Epic hierarchical child", erroneousEpicChildId, "expected global `task-N`");
	await rm(erroneousEpicChildPath, { force: true });

	const erroneousGlobalSubtaskId = "task-9998";
	const erroneousGlobalSubtaskPath = join(workspace, ".tandem", "board", `${erroneousGlobalSubtaskId}.md`);
	await writeFile(erroneousGlobalSubtaskPath, `---\nid: ${erroneousGlobalSubtaskId}\ntype: task\ntitle: "Erroneous global-ID Subtask"\nstate: todo\nparentId: "${taskId}"\n---\n`, "utf8");
	const erroneousGlobalSubtaskRead = await runProcess(tandem, buildTaskArgs({ action: "list" }), workspace);
	assertFailure(erroneousGlobalSubtaskRead, "erroneous global-ID Subtask", erroneousGlobalSubtaskId, `expected \`${taskId}-M\``);
	await rm(erroneousGlobalSubtaskPath, { force: true });

	const missingParent = await runProcess(tandem, buildTaskArgs({ action: "add", title: "Bad missing parent", parent: "task-99999" }), workspace);
	assertFailure(missingParent, "missing parent", "Validation failed: parent document not found: task-99999");
	const looseReferenceOutput = await runTandem(tandem, buildTaskArgs({ action: "add", title: "Loose related reference", references: ["task-99999"] }), workspace);
	assertIncludes(looseReferenceOutput, "Warning: reference not found: task-99999", "loose reference warning");

	console.log(`pi-tandem relationship smoke passed with ${tandem}`);
	console.log(`Verified canonical hierarchy: ${epicId} (Epic) -> ${taskId} (Task) -> ${subtaskId}/${successorId} (Subtasks); generic ${decisionId} -> ${genericId}.`);
} finally {
	await rm(workspace, { recursive: true, force: true });
}
