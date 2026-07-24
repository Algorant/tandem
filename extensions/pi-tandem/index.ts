import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { createTandemToolRenderer } from "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-ui/tool-ui/custom-tools/tandem";
import {
	DEFAULT_MAX_BYTES,
	DEFAULT_MAX_LINES,
	formatSize,
	truncateHead,
	type TruncationResult,
} from "@earendil-works/pi-coding-agent";
import { StringEnum } from "@earendil-works/pi-ai";
import { Type } from "typebox";
import { execFile } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, resolve } from "node:path";

// pi-tandem is intentionally a lightweight adapter over the installed `tandem` CLI.
// Tandem protocol parsing/mutation behavior belongs in tandem/tandem, not here.

type ToolContent = { type: "text"; text: string };
type ToolResult = { content: ToolContent[]; details?: Record<string, unknown>; isError?: boolean };
type RunResult = {
	ok: boolean;
	code: number | null;
	stdout: string;
	stderr: string;
	args: string[];
	cwd: string;
	command: string;
	error?: string;
	timedOut?: boolean;
	aborted?: boolean;
};

type ReadJsonFlag = { json?: boolean };
type CwdFlag = { cwd?: string; timeoutMs?: number };

export type InitToolParams = CwdFlag & {
	title: string;
};

export type TaskToolParams = CwdFlag & ReadJsonFlag & {
	action: "list" | "show" | "add" | "move" | "update" | "complete" | "cancel";
	id?: string;
	title?: string;
	state?: string;
	description?: string;
	body?: string;
	priority?: string;
	type?: string;
	kind?: "epic";
	tags?: string[];
	assignee?: string;
	dueDate?: string;
	parent?: string;
	blockers?: string[];
	references?: string[];
	relatedFiles?: string[];
	accord?: string;
	review?: string;
	filesChanged?: string[];
	summary?: string;
	validation?: string;
	reviewer?: string;
	reason?: string;
};

export type AccordToolParams = CwdFlag & {
	action: "claim" | "deliver" | "accept" | "rework" | "block" | "fail";
	id: string;
	assignee?: string;
	summary?: string;
	reviewer?: string;
	note?: string;
	reason?: string;
	deliverables?: string[];
	validations?: string[];
	constraints?: string[];
	evidence?: string[];
	filesChanged?: string[];
};

export type LogToolParams = CwdFlag & ReadJsonFlag & {
	action: "list" | "show" | "search";
	id?: string;
	query?: string;
	limit?: number;
};

export type RulesToolParams = CwdFlag & ReadJsonFlag & {
	action: "list" | "add" | "edit" | "delete";
	category?: "always" | "never" | "prefer" | "context";
	id?: number;
	rule?: string;
	source?: string;
};

export type DecisionToolParams = CwdFlag & ReadJsonFlag & {
	action: "list" | "show" | "add";
	id?: string;
	title?: string;
	body?: string;
	status?: "proposed" | "accepted" | "rejected" | "deprecated" | "superseded";
	date?: string;
	deciders?: string[];
	context?: string;
	consequences?: string[];
	alternatives?: string[];
	supersedes?: string[];
	supersededBy?: string[];
	references?: string[];
	tags?: string[];
};

export type SearchToolParams = CwdFlag & ReadJsonFlag & {
	query: string;
	state?: string;
	type?: string;
	parent?: string;
};

const DEFAULT_TIMEOUT_MS = 30_000;
const MAX_TIMEOUT_MS = 120_000;
const MAX_BUFFER_BYTES = 10 * 1024 * 1024;
const TANDEM_BIN_ENV_KEYS = ["TANDEM_BIN"] as const;

function tandemBinary(): string {
	for (const key of TANDEM_BIN_ENV_KEYS) {
		const value = process.env[key]?.trim();
		if (value) return value;
	}
	return "tandem";
}

function normalizeError(err: unknown): string {
	if (err instanceof Error) return err.message;
	return String(err);
}

function stripAtPrefix(value: string): string {
	return value.startsWith("@") ? value.slice(1) : value;
}

function resolveCwd(baseCwd: string, cwd?: string): string {
	if (!cwd?.trim()) return baseCwd;
	const cleaned = stripAtPrefix(cwd.trim());
	return isAbsolute(cleaned) ? cleaned : resolve(baseCwd, cleaned);
}

function clampTimeout(value: unknown): number {
	const n = Number(value ?? DEFAULT_TIMEOUT_MS);
	if (!Number.isFinite(n)) return DEFAULT_TIMEOUT_MS;
	return Math.max(1_000, Math.min(MAX_TIMEOUT_MS, Math.floor(n)));
}

function requireString(value: unknown, message: string): string {
	if (typeof value !== "string" || !value.trim()) throw new Error(message);
	return value;
}

function requirePositiveInteger(value: unknown, message: string): number {
	const n = Number(value);
	if (!Number.isInteger(n) || n <= 0) throw new Error(message);
	return n;
}

function addOptionalFlag(args: string[], flag: string, value: unknown): void {
	if (typeof value === "string" && value.trim()) args.push(flag, value);
}

function addPresentStringFlag(args: string[], flag: string, value: unknown): void {
	if (typeof value === "string") args.push(flag, value);
}

function addRepeatedFlag(args: string[], flag: string, values: unknown): void {
	if (!Array.isArray(values)) return;
	for (const value of values) {
		if (typeof value === "string" && value.trim()) args.push(flag, value);
	}
}

function hasStringValue(value: unknown): boolean {
	return typeof value === "string" && value.trim().length > 0;
}

function rejectDeprecatedInlineSubtasks(params: TaskToolParams): void {
	const legacySubtasks = (params as TaskToolParams & { subtasks?: unknown }).subtasks;
	if (legacySubtasks === undefined) return;
	throw new Error(
		"tandem_task no longer authors deprecated inline subtasks. Create each independently tracked child with a separate tandem_task action=add call using parent=<task-id>.",
	);
}

function rejectUnsupportedUpdateFields(params: TaskToolParams): void {
	const unsupported: string[] = [];
	if (hasStringValue(params.description)) unsupported.push("description");
	if (hasStringValue(params.accord)) unsupported.push("accord");
	if (hasStringValue(params.review)) unsupported.push("review");
	if (!unsupported.length) return;
	throw new Error(
		`tandem_task update does not support ${unsupported.join(", ")}. ` +
			"Use tandem_task action=add for descriptions when creating tasks; use tandem_task action=update with body for complete Markdown body replacement; use tandem_accord for accord lifecycle changes; review metadata is managed by review/validation flows, not tandem update. Supported update fields are title, body, kind, priority, assignee, dueDate, parent, tags, blockers, references, and relatedFiles.",
	);
}

function wantsJson(params: ReadJsonFlag): boolean {
	return params.json !== false;
}

export function buildInitArgs(params: InitToolParams): string[] {
	return ["init", "--title", requireString(params.title, "tandem_init requires title")];
}

export function buildTaskArgs(params: TaskToolParams): string[] {
	rejectDeprecatedInlineSubtasks(params);
	const action = params.action;
	if (params.body !== undefined && action !== "update") {
		throw new Error("tandem_task body is supported only for action=update; use description when creating a Task");
	}
	if (action === "list") {
		const args = ["list"];
		addOptionalFlag(args, "--state", params.state);
		addOptionalFlag(args, "--type", params.type);
		addOptionalFlag(args, "--priority", params.priority);
		addOptionalFlag(args, "--parent", params.parent);
		addOptionalFlag(args, "--tag", params.tags?.[0]);
		addOptionalFlag(args, "--assignee", params.assignee);
		addOptionalFlag(args, "--accord", params.accord);
		addOptionalFlag(args, "--review", params.review);
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (action === "show") {
		const args = ["show", requireString(params.id, "tandem_task show requires id")];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (action === "add") {
		const args = ["add", "--title", requireString(params.title, "tandem_task add requires title")];
		addOptionalFlag(args, "--state", params.state);
		addOptionalFlag(args, "--description", params.description);
		addOptionalFlag(args, "--priority", params.priority);
		addOptionalFlag(args, "--kind", params.kind);
		addRepeatedFlag(args, "--tag", params.tags);
		addOptionalFlag(args, "--assignee", params.assignee);
		addOptionalFlag(args, "--due-date", params.dueDate);
		addOptionalFlag(args, "--parent", params.parent);
		addRepeatedFlag(args, "--blocker", params.blockers);
		addRepeatedFlag(args, "--reference", params.references);
		addRepeatedFlag(args, "--related-file", params.relatedFiles);
		return args;
	}
	if (action === "move") {
		return ["move", requireString(params.id, "tandem_task move requires id"), "--state", requireString(params.state, "tandem_task move requires state")];
	}
	if (action === "update") {
		rejectUnsupportedUpdateFields(params);
		const args = ["update", requireString(params.id, "tandem_task update requires id")];
		addOptionalFlag(args, "--title", params.title);
		addPresentStringFlag(args, "--body", params.body);
		addOptionalFlag(args, "--kind", params.kind);
		addOptionalFlag(args, "--priority", params.priority);
		addOptionalFlag(args, "--assignee", params.assignee);
		addOptionalFlag(args, "--due-date", params.dueDate);
		addOptionalFlag(args, "--parent", params.parent);
		addRepeatedFlag(args, "--tag", params.tags);
		addRepeatedFlag(args, "--blocker", params.blockers);
		addRepeatedFlag(args, "--reference", params.references);
		addRepeatedFlag(args, "--related-file", params.relatedFiles);
		return args;
	}
	if (action === "cancel") {
		return ["cancel", requireString(params.id, "tandem_task cancel requires id"), "--reason", requireString(params.reason, "tandem_task cancel requires reason")];
	}
	if (action === "complete") {
		const args = ["complete", requireString(params.id, "tandem_task complete requires id"), "--summary", requireString(params.summary, "tandem_task complete requires summary")];
		addRepeatedFlag(args, "--file-changed", params.filesChanged);
		addOptionalFlag(args, "--validation", params.validation);
		addOptionalFlag(args, "--reviewer", params.reviewer);
		return args;
	}
	throw new Error(`unsupported tandem_task action: ${String(action)}`);
}

export function buildAccordArgs(params: AccordToolParams): string[] {
	const args = ["accord", params.action, requireString(params.id, "tandem_accord requires id")];
	addOptionalFlag(args, "--assignee", params.assignee);
	addOptionalFlag(args, "--summary", params.summary);
	addOptionalFlag(args, "--reviewer", params.reviewer);
	addOptionalFlag(args, "--note", params.note);
	addOptionalFlag(args, "--reason", params.reason);
	addRepeatedFlag(args, "--deliverable", params.deliverables);
	addRepeatedFlag(args, "--validation", params.validations);
	addRepeatedFlag(args, "--constraint", params.constraints);
	addRepeatedFlag(args, "--evidence", params.evidence);
	addRepeatedFlag(args, "--file-changed", params.filesChanged);
	return args;
}

export function buildLogArgs(params: LogToolParams): string[] {
	if (params.action === "list") {
		const args = ["log", "list"];
		if (params.limit !== undefined) args.push("--limit", String(requirePositiveInteger(params.limit, "tandem_log list limit must be a positive integer")));
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "show") {
		const args = ["log", "show", requireString(params.id, "tandem_log show requires id")];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "search") {
		const args = ["log", "search", requireString(params.query, "tandem_log search requires query")];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	throw new Error(`unsupported tandem_log action: ${String(params.action)}`);
}

export function buildRulesArgs(params: RulesToolParams): string[] {
	if (params.action === "list") {
		const args = ["rules", "list"];
		addOptionalFlag(args, "--category", params.category);
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "add") {
		const args = ["rules", "add", "--category", requireString(params.category, "tandem_rules add requires category"), "--rule", requireString(params.rule, "tandem_rules add requires rule")];
		addOptionalFlag(args, "--source", params.source);
		return args;
	}
	if (params.action === "edit") {
		const args = ["rules", "edit", "--category", requireString(params.category, "tandem_rules edit requires category"), "--id", String(requirePositiveInteger(params.id, "tandem_rules edit requires a positive id")), "--rule", requireString(params.rule, "tandem_rules edit requires rule")];
		addOptionalFlag(args, "--source", params.source);
		return args;
	}
	if (params.action === "delete") {
		return ["rules", "delete", "--category", requireString(params.category, "tandem_rules delete requires category"), "--id", String(requirePositiveInteger(params.id, "tandem_rules delete requires a positive id"))];
	}
	throw new Error(`unsupported tandem_rules action: ${String(params.action)}`);
}

export function buildDecisionArgs(params: DecisionToolParams): string[] {
	if (params.action === "list") {
		const args = ["decision", "list"];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "show") {
		const args = ["decision", "show", requireString(params.id, "tandem_decision show requires id")];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "add") {
		const args = ["decision", "add", "--title", requireString(params.title, "tandem_decision add requires title")];
		addOptionalFlag(args, "--body", params.body);
		addOptionalFlag(args, "--status", params.status);
		addOptionalFlag(args, "--date", params.date);
		addRepeatedFlag(args, "--decider", params.deciders);
		addOptionalFlag(args, "--context", params.context);
		addRepeatedFlag(args, "--consequence", params.consequences);
		addRepeatedFlag(args, "--alternative", params.alternatives);
		addRepeatedFlag(args, "--supersedes", params.supersedes);
		addRepeatedFlag(args, "--superseded-by", params.supersededBy);
		addRepeatedFlag(args, "--reference", params.references);
		addRepeatedFlag(args, "--tag", params.tags);
		return args;
	}
	throw new Error(`unsupported tandem_decision action: ${String(params.action)}`);
}

export function buildSearchArgs(params: SearchToolParams): string[] {
	const args = ["search", requireString(params.query, "tandem_search requires query")];
	addOptionalFlag(args, "--state", params.state);
	addOptionalFlag(args, "--type", params.type);
	addOptionalFlag(args, "--parent", params.parent);
	if (wantsJson(params)) args.push("--json");
	return args;
}

function findWorkspaceRoot(startCwd: string): string | undefined {
	let current = resolve(startCwd);
	while (true) {
		if (existsSync(join(current, ".tandem", "tandem.md"))) return current;
		const parent = dirname(current);
		if (parent === current) return undefined;
		current = parent;
	}
}

function parseJsonIfPossible(text: string): unknown | undefined {
	const trimmed = text.trim();
	if (!trimmed || (!trimmed.startsWith("{") && !trimmed.startsWith("["))) return undefined;
	try {
		return JSON.parse(trimmed);
	} catch {
		return undefined;
	}
}

function classifyFailure(result: RunResult): string {
	const combined = `${result.stderr}\n${result.stdout}\n${result.error ?? ""}`.trim();
	if (/ENOENT|not found/i.test(result.error ?? "")) {
		return `Missing tandem CLI: could not execute ${JSON.stringify(result.command)}. Build/install tandem so \`tandem\` is on PATH, or set ${TANDEM_BIN_ENV_KEYS.join("/")} to the binary path.`;
	}
	if (/No Tandem workspace found|\.tandem/i.test(combined) && /tandem init/i.test(combined)) {
		return "Missing Tandem workspace: no .tandem/tandem.md was discovered from this cwd. Run `tandem init --title <title>` only after the user wants a workspace.";
	}
	if (/unknown .*flag|unknown .*subcommand|unknown command/i.test(combined)) {
		return "Unsupported tandem CLI surface: the installed `tandem` may be older than pi-tandem expects, or this wrapper passed a flag/subcommand not supported by the current CLI.";
	}
	if (result.timedOut) return "tandem command timed out before it completed.";
	if (result.aborted) return "tandem command was aborted before it completed.";
	return "tandem command failed.";
}

function runTandem(args: string[], cwd: string, signal?: AbortSignal, timeoutMs?: number): Promise<RunResult> {
	const command = tandemBinary();
	return new Promise((resolveRun) => {
		let settled = false;
		const child = execFile(command, args, {
			cwd,
			timeout: clampTimeout(timeoutMs),
			maxBuffer: MAX_BUFFER_BYTES,
		}, (err, stdout, stderr) => {
			if (settled) return;
			settled = true;
			signal?.removeEventListener("abort", abort);
			const error = err as (Error & { code?: string | number; signal?: string; killed?: boolean }) | null;
			const code = typeof error?.code === "number" ? error.code : (error ? 1 : 0);
			resolveRun({
				ok: !err,
				code,
				stdout: String(stdout ?? ""),
				stderr: String(stderr ?? ""),
				args,
				cwd,
				command,
				error: error?.message,
				timedOut: Boolean(error?.killed && /timed out/i.test(error.message)),
			});
		});

		const abort = () => {
			if (settled) return;
			settled = true;
			child.kill("SIGTERM");
			signal?.removeEventListener("abort", abort);
			resolveRun({
				ok: false,
				code: null,
				stdout: "",
				stderr: "",
				args,
				cwd,
				command,
				error: "tandem command aborted",
				aborted: true,
			});
		};

		if (signal?.aborted) abort();
		else signal?.addEventListener("abort", abort, { once: true });
	});
}

async function truncateForTool(text: string): Promise<{ text: string; truncation?: TruncationResult; fullOutputPath?: string }> {
	const truncation = truncateHead(text, { maxBytes: DEFAULT_MAX_BYTES, maxLines: DEFAULT_MAX_LINES });
	if (!truncation.truncated) return { text: truncation.content };

	const tempDir = await mkdtemp(join(tmpdir(), "pi-tandem-"));
	const fullOutputPath = join(tempDir, "tandem-output.txt");
	await writeFile(fullOutputPath, text, "utf8");
	const suffix = `\n\n[Output truncated: showing ${truncation.outputLines} of ${truncation.totalLines} lines (${formatSize(truncation.outputBytes)} of ${formatSize(truncation.totalBytes)}). Full output saved to: ${fullOutputPath}]`;
	return { text: `${truncation.content}${suffix}`, truncation, fullOutputPath };
}

async function formatRunResult(label: string, result: RunResult): Promise<ToolResult> {
	const parsed = parseJsonIfPossible(result.stdout);
	const commandLine = `${result.command} ${result.args.join(" ")}`.trim();
	if (!result.ok) {
		const diagnostic = classifyFailure(result);
		const body = [
			`${label} failed.`,
			diagnostic,
			`cwd: ${result.cwd}`,
			`command: ${commandLine}`,
			`exit: ${result.code ?? "unknown"}`,
			result.stderr.trim() ? `stderr:\n${result.stderr.trim()}` : undefined,
			result.stdout.trim() ? `stdout:\n${result.stdout.trim()}` : undefined,
			result.error ? `error: ${result.error}` : undefined,
		].filter(Boolean).join("\n\n");
		const truncated = await truncateForTool(body);
		return {
			content: [{ type: "text", text: truncated.text }],
			details: { ...result, diagnostic, truncation: truncated.truncation, fullOutputPath: truncated.fullOutputPath },
			isError: true,
		};
	}

	const raw = result.stdout.trim() || result.stderr.trim() || "OK";
	const body = parsed === undefined ? raw : JSON.stringify(parsed, null, 2);
	const stderr = result.stderr.trim() ? `\n\nstderr:\n${result.stderr.trim()}` : "";
	const truncated = await truncateForTool(`${label}\n\n${body}${stderr}`);
	return {
		content: [{ type: "text", text: truncated.text }],
		details: {
			ok: true,
			args: result.args,
			cwd: result.cwd,
			command: result.command,
			stdout: result.stdout,
			stderr: result.stderr,
			parsedJson: parsed,
			truncation: truncated.truncation,
			fullOutputPath: truncated.fullOutputPath,
		},
	};
}

async function executeTandemTool(label: string, args: string[], baseCwd: string, params: CwdFlag, signal?: AbortSignal, onUpdate?: (update: ToolResult) => void): Promise<ToolResult> {
	const cwd = resolveCwd(baseCwd, params.cwd);
	onUpdate?.({ content: [{ type: "text", text: `tandem ${args.join(" ")}` }], details: { cwd, args } });
	const result = await runTandem(args, cwd, signal, params.timeoutMs);
	return formatRunResult(label, result);
}

async function buildStatusText(baseCwd: string, params: CwdFlag = {}, signal?: AbortSignal): Promise<ToolResult> {
	const cwd = resolveCwd(baseCwd, params.cwd);
	const workspaceRoot = findWorkspaceRoot(cwd);
	const help = await runTandem(["--help"], cwd, signal, params.timeoutMs);
	const lines = ["# pi-tandem status", "", `cwd: ${cwd}`, `tandem binary: ${tandemBinary()}`];
	lines.push(workspaceRoot ? `workspace: ${workspaceRoot}` : "workspace: not found (.tandem/tandem.md missing from cwd or parents)");
	if (!help.ok) {
		const diagnostic = classifyFailure(help);
		lines.push("", diagnostic);
		return { content: [{ type: "text", text: lines.join("\n") }], details: { cwd, workspaceRoot, help, diagnostic }, isError: true };
	}
	lines.push("tandem: available");
	if (workspaceRoot) {
		const list = await runTandem(["list", "--json"], cwd, signal, params.timeoutMs);
		if (list.ok) {
			const parsed = parseJsonIfPossible(list.stdout) as { data?: { counts?: unknown } } | undefined;
			lines.push("tandem list --json: ok");
			if (parsed?.data?.counts) lines.push(`counts: ${JSON.stringify(parsed.data.counts)}`);
			return { content: [{ type: "text", text: lines.join("\n") }], details: { cwd, workspaceRoot, help, list, parsedJson: parsed } };
		}
		const diagnostic = classifyFailure(list);
		lines.push("", `tandem list --json failed: ${diagnostic}`);
		return { content: [{ type: "text", text: lines.join("\n") }], details: { cwd, workspaceRoot, help, list, diagnostic }, isError: true };
	}
	lines.push("hint: run `tandem init --title <title>` only after the user wants this directory to become a Tandem workspace.");
	return { content: [{ type: "text", text: lines.join("\n") }], details: { cwd, workspaceRoot, help } };
}

const cwdSchema = {
	cwd: Type.Optional(Type.String({ description: "Working directory. Defaults to Pi's current cwd; relative paths resolve from there." })),
	timeoutMs: Type.Optional(Type.Number({ description: `Command timeout in milliseconds. Defaults to ${DEFAULT_TIMEOUT_MS}; max ${MAX_TIMEOUT_MS}.` })),
};

const jsonSchema = {
	json: Type.Optional(Type.Boolean({ description: "For read actions, request `tandem --json` output. Defaults to true." })),
};

export const tandemTaskParameters = Type.Object({
	...cwdSchema,
	...jsonSchema,
	action: StringEnum(["list", "show", "add", "move", "update", "complete", "cancel"] as const),
	id: Type.Optional(Type.String()),
	title: Type.Optional(Type.String()),
	state: Type.Optional(Type.String()),
	type: Type.Optional(Type.String({ description: "Filter document type for list, usually task or decision." })),
	description: Type.Optional(Type.String({ description: "Task description for action=add only; action=update rejects this field. Use body for exact update-time Markdown body replacement." })),
	body: Type.Optional(Type.String({ description: "Exact complete Markdown body for action=update, including an empty string to clear it. Maps to `tandem update --body` without trimming." })),
	priority: Type.Optional(Type.String()),
	kind: Type.Optional(StringEnum(["epic"] as const, { description: "Optional task classifier for add/update. Use kind=epic only for a root global task; Epics cannot have parentId and are not delegation roots." })),
	tags: Type.Optional(Type.Array(Type.String())),
	assignee: Type.Optional(Type.String()),
	dueDate: Type.Optional(Type.String()),
	parent: Type.Optional(Type.String({ description: "Existing Tandem document ID passed directly to add/update as --parent, or used by list to filter exact parentId matches. On add, the CLI resolves the parent role: an Epic gets a global-ID Task, a Task gets a parent-derived Subtask, and a decision/custom document gets a global-ID Task with a generic parent relationship. pi-tandem never allocates IDs or classifies roles. Create or inspect the parent first." })),
	blockers: Type.Optional(Type.Array(Type.String(), { description: "Existing Tandem document IDs that block this task. These are strict core references; missing IDs make tandem add fail." })),
	references: Type.Optional(Type.Array(Type.String(), { description: "Related Tandem document IDs such as decisions, sibling tasks, or logs. Prefer existing IDs; missing IDs are warnings, not hard blockers." })),
	relatedFiles: Type.Optional(Type.Array(Type.String(), { description: "Project-relative file paths relevant to the task for implementation or review context." })),
	accord: Type.Optional(Type.String({ description: "Filter value for action=list only. Use tandem_accord for accord lifecycle changes; action=update intentionally rejects this field." })),
	review: Type.Optional(Type.String({ description: "Filter value for action=list only. Review metadata is managed by review/validation flows; action=update intentionally rejects this field." })),
	filesChanged: Type.Optional(Type.Array(Type.String())),
	summary: Type.Optional(Type.String({ description: "Required for action=complete; maps to `tandem complete --summary`." })),
	validation: Type.Optional(Type.String()),
	reviewer: Type.Optional(Type.String()),
	reason: Type.Optional(Type.String({ description: "Required for action=cancel; maps to `tandem cancel --reason` and is retained in the canceled Log summary." })),
});

export function tandemPromptGuidance(workspaceRoot?: string): string {
	const workspaceLine = workspaceRoot ? `A Tandem workspace is present at ${workspaceRoot}.` : "No Tandem workspace is currently detected from the working directory.";
	return `\n\n## Tandem coordination guidance\n\n${workspaceLine}\n\n- Prefer pi-tandem tools (tandem_status, tandem_init, tandem_task, tandem_accord, tandem_log, tandem_rules, tandem_decision, tandem_search) over manual edits to .tandem files for durable coordination.\n- Use tandem_status before tandem_init; if tandem_status reports no workspace, ask before initializing a new Tandem workspace. Do not create .tandem state implicitly.\n- Keep Tandem behavior in the tandem CLI/protocol; use pi-tandem as a thin adapter and diagnostics layer.\n- Use workflow state \`validation\` for delivered work awaiting acceptance, rejection, redirection, or human/product judgment; existing \`state: review\` files are legacy reads, not the preferred new state.\n- Keep workflow state, accord status, and \`review:\` metadata distinct. Review metadata can record reviewer decisions/status without renaming it to validation.\n- Use tandem_decision for durable project/product/architecture decisions, including ADR-compatible records; do not model decisions as task lifecycle state or a separate ADR type.\n- Create each independently tracked work unit with tandem_task action=add and pass parent directly to Tandem. The CLI resolves canonical roles and IDs: Epics are root global \`task-N\` documents, their direct children are global-ID Tasks with \`parentRelationship: epic-task\`, and a Task's direct children are leaf, parent-derived \`task-N-M\` Subtasks with \`parentRelationship: subtask\`. Decision/custom parents produce global-ID Tasks with generic \`parent\`. Never allocate IDs or reclassify CLI output in Pi. Inline checklist subtasks are legacy read-only metadata. Use blockers for strict dependencies, references for related Tandem docs, and relatedFiles for project paths.\n- Only Task-role roots are delegated initially. Epics and Subtasks are not delegation roots; one Task worker owns its direct Subtasks through the todo projection and produces one Task-root handoff. Child workers report evidence but do not accept, complete, or archive Tandem work.\n- Use tandem_task action=update for supported active Task edits: body replaces the exact complete Markdown body; title, kind, priority, assignee, dueDate, parent, tags, blockers, references, and relatedFiles edit metadata. State remains action=move, description remains an add-time convenience field, and accord changes go through tandem_accord. Role-changing or ID-invalidating reparenting is rejected by Tandem.\n- Use tandem_task action=cancel only when the user or orchestrator explicitly asks to archive abandoned or mistaken work. Provide a reason; Tandem preserves an auditable canceled Log and rejects cancellation while active descendants remain.\n- Epics are ordinary root tasks with \`type: task\` plus \`kind: epic\`; use references for loose context. Do not invent \`type: epic\`, ADR-style epic records, custom folders, or special epic lifecycle behavior.\n- Use tandem_accord for claiming, delivering, accepting, reworking, blocking, or failing work agreements. Deliver finished agent work into Validation; child/subagent workers must only report and deliver evidence, never accept, complete, or archive tasks themselves.\n- Use tandem_log and tandem_search for completed-work history instead of treating logs as trash/archive only.\n`;
}

function promptMentionsDurableCoordination(prompt: string): boolean {
	return /\b(tandem|durable coordination|task board|accord|work agreement|validation|review queue|project rules|completed logs?|epics?)\b/i.test(prompt);
}

export default function piTandem(pi: ExtensionAPI) {
	pi.registerTool({
		name: "tandem_status",
		label: "Tandem Status",
		...createTandemToolRenderer("tandem_status", "Tandem Status"),
		description: "Diagnose the installed `tandem` CLI and the nearest `.tandem` workspace.",
		promptSnippet: "Use tandem_status to inspect Tandem/tandem health before durable project coordination or when .tandem diagnostics are needed.",
		promptGuidelines: [
			"Use tandem_status when entering a Tandem repo, when `.tandem/tandem.md` may exist, or before bootstrapping durable coordination.",
			"If tandem_status reports no workspace, ask before initializing a new Tandem workspace; do not create `.tandem` state implicitly.",
		],
		parameters: Type.Object({ ...cwdSchema }),
		async execute(_toolCallId, params, signal, _onUpdate, ctx) {
			return buildStatusText(ctx.cwd, params, signal);
		},
	});

	pi.registerTool({
		name: "tandem_init",
		label: "Tandem Init",
		...createTandemToolRenderer("tandem_init", "Tandem Init"),
		description: "Initialize a Tandem workspace by running `tandem init --title <title>`.",
		promptSnippet: "Use tandem_init only after tandem_status reports no workspace and the user confirms this directory should become a Tandem workspace.",
		promptGuidelines: [
			"Use tandem_status first to inspect the current cwd and nearest .tandem workspace before initializing.",
			"Ask before initializing a new workspace; do not create `.tandem` state implicitly.",
			"Use tandem_init instead of bash for workspace initialization; it maps directly to `tandem init --title <title>`.",
		],
		parameters: Type.Object({
			...cwdSchema,
			title: Type.String({ description: "Workspace title passed to `tandem init --title`." }),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTandemTool("tandem_init", buildInitArgs(params as InitToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tandem_init argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tandem_task",
		label: "Tandem Task",
		...createTandemToolRenderer("tandem_task", "Tandem Task"),
		description: "Run task-oriented `tandem` commands: list, show, add, move, update, complete, or cancel. Update supports exact Markdown body replacement plus supported metadata edits; cancel archives an active Task with an auditable canceled outcome. Add/update pass kind and parent directly to the CLI: Epics contain global-ID Tasks, Tasks contain parent-derived leaf Subtasks, and generic parents retain global Task IDs. pi-tandem never allocates IDs or reclassifies CLI relationships. Deprecated inline checklist subtasks are not authored. Read actions default to `--json`; mutations preserve human-readable CLI output.",
		promptSnippet: "Use tandem_task for Tandem task list/show/add/move/update/complete/cancel operations instead of editing .tandem files directly.",
		promptGuidelines: [
			"Use tandem_task for active Tandem task reads and mutations when `.tandem/tandem.md` exists.",
			"Prefer tandem_task read actions with the default JSON output for reliable task inspection.",
			"Pass parent directly to Tandem and consume the CLI result without reclassification: an Epic gets global-ID Tasks with epic-task, a Task gets parent-derived leaf Subtasks with subtask, and a decision/custom parent gets global-ID Tasks with generic parent. Never construct IDs in Pi. Use blockers for hard dependencies, references for related tasks/decisions/logs, and relatedFiles for repo paths. Inline checklist subtasks are legacy metadata and tandem_task does not author them.",
			"Model Epics as root tasks with type: task plus kind: epic. Only Tasks are delegation roots initially; one Task worker owns its Subtasks through the todo projection, while Epics and Subtasks are not independently delegated.",
			"Use tandem_task action=update for supported active Task edits: body replaces the exact complete Markdown body; title, kind, priority, assignee, dueDate, parent, tags, blockers, references, and relatedFiles edit metadata. State remains action=move, description remains an add-time convenience field, and accord changes go through tandem_accord. Let Tandem reject role-changing or ID-invalidating reparenting.",
			"Create or inspect parent and blocker documents before referencing them; tandem validates parent/blockers strictly, while references are related context and only warn if unresolved.",
			"Prefer state=validation for delivered work awaiting human/product judgment; existing state=review is a legacy alias only.",
			"Do not use tandem_task complete or cancel unless the user or orchestrator explicitly asks to archive completed or canceled work. Cancellation requires a reason and is rejected while active descendants remain.",
		],
		parameters: tandemTaskParameters,
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTandemTool("tandem_task", buildTaskArgs(params as TaskToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tandem_task argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tandem_accord",
		label: "Tandem Accord",
		...createTandemToolRenderer("tandem_accord", "Tandem Accord"),
		description: "Run `tandem accord claim|deliver|accept|rework|block|fail` as a thin Pi wrapper.",
		promptSnippet: "Use tandem_accord for Tandem work-agreement lifecycle actions (claim, deliver, accept, rework, block, fail).",
		promptGuidelines: [
			"Use tandem_accord for accord state changes instead of direct frontmatter edits.",
			"Deliver finished agent work into the Validation workflow state for acceptance/rework decisions; do not treat automated validation evidence as human acceptance.",
			"Do not accept, complete, or otherwise finalize Tandem work unless the user/orchestrator explicitly asks for that lifecycle transition.",
		],
		parameters: Type.Object({
			...cwdSchema,
			action: StringEnum(["claim", "deliver", "accept", "rework", "block", "fail"] as const),
			id: Type.String(),
			assignee: Type.Optional(Type.String()),
			summary: Type.Optional(Type.String()),
			reviewer: Type.Optional(Type.String()),
			note: Type.Optional(Type.String()),
			reason: Type.Optional(Type.String()),
			deliverables: Type.Optional(Type.Array(Type.String())),
			validations: Type.Optional(Type.Array(Type.String({ description: "Validation commands/descriptions; maps to repeated `--validation`." }))),
			constraints: Type.Optional(Type.Array(Type.String())),
			evidence: Type.Optional(Type.Array(Type.String())),
			filesChanged: Type.Optional(Type.Array(Type.String())),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTandemTool("tandem_accord", buildAccordArgs(params as AccordToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tandem_accord argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tandem_log",
		label: "Tandem Logs",
		...createTandemToolRenderer("tandem_log", "Tandem Logs"),
		description: "Run completed-work log reads: `tandem log list|show|search`. Defaults to JSON output.",
		promptSnippet: "Use tandem_log for Tandem completed-work history list/show/search.",
		promptGuidelines: [
			"Use tandem_log for completed Tandem work instead of manually reading `.tandem/logs` unless raw source inspection is required.",
		],
		parameters: Type.Object({
			...cwdSchema,
			...jsonSchema,
			action: StringEnum(["list", "show", "search"] as const),
			id: Type.Optional(Type.String()),
			query: Type.Optional(Type.String()),
			limit: Type.Optional(Type.Number()),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTandemTool("tandem_log", buildLogArgs(params as LogToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tandem_log argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tandem_rules",
		label: "Tandem Rules",
		...createTandemToolRenderer("tandem_rules", "Tandem Rules"),
		description: "Run `tandem rules list|add|edit|delete`. List defaults to JSON; mutations preserve human-readable CLI output.",
		promptSnippet: "Use tandem_rules to inspect or update Tandem project rules through tandem.",
		promptGuidelines: [
			"Use tandem_rules for Tandem project rules rather than editing `.tandem/tandem.md` by hand.",
		],
		parameters: Type.Object({
			...cwdSchema,
			...jsonSchema,
			action: StringEnum(["list", "add", "edit", "delete"] as const),
			category: Type.Optional(StringEnum(["always", "never", "prefer", "context"] as const)),
			id: Type.Optional(Type.Number()),
			rule: Type.Optional(Type.String()),
			source: Type.Optional(Type.String()),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTandemTool("tandem_rules", buildRulesArgs(params as RulesToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tandem_rules argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tandem_decision",
		label: "Tandem Decisions",
		...createTandemToolRenderer("tandem_decision", "Tandem Decisions"),
		description: "Run `tandem decision list|show|add`. Read actions default to JSON; add preserves human-readable CLI output and supports ADR-compatible metadata.",
		promptSnippet: "Use tandem_decision for Tandem decision document list/show/add operations, including ADR-compatible durable records.",
		promptGuidelines: [
			"Use tandem_decision for first-class Tandem decision documents; do not model decisions as task lifecycle state.",
			"Decision status is ADR metadata (proposed/accepted/rejected/deprecated/superseded), not a workflow state.",
			"For ADR-compatible records, keep type=decision and put Status/Context/Decision/Consequences/Supersession sections in the body instead of inventing a separate ADR type.",
		],
		parameters: Type.Object({
			...cwdSchema,
			...jsonSchema,
			action: StringEnum(["list", "show", "add"] as const),
			id: Type.Optional(Type.String()),
			title: Type.Optional(Type.String()),
			body: Type.Optional(Type.String({ description: "Markdown body. For ADR-compatible records, include sections such as Status, Context, Decision, Consequences, Supersession, and References." })),
			status: Type.Optional(StringEnum(["proposed", "accepted", "rejected", "deprecated", "superseded"] as const)),
			date: Type.Optional(Type.String()),
			deciders: Type.Optional(Type.Array(Type.String())),
			context: Type.Optional(Type.String()),
			consequences: Type.Optional(Type.Array(Type.String())),
			alternatives: Type.Optional(Type.Array(Type.String())),
			supersedes: Type.Optional(Type.Array(Type.String())),
			supersededBy: Type.Optional(Type.Array(Type.String())),
			references: Type.Optional(Type.Array(Type.String())),
			tags: Type.Optional(Type.Array(Type.String())),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTandemTool("tandem_decision", buildDecisionArgs(params as DecisionToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tandem_decision argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tandem_search",
		label: "Tandem Search",
		...createTandemToolRenderer("tandem_search", "Tandem Search"),
		description: "Run `tandem search <query>` across active Tandem documents and completed logs. Defaults to JSON output.",
		promptSnippet: "Use tandem_search for project work search across active Tandem documents and logs.",
		promptGuidelines: [
			"Use tandem_search before ad hoc file grep when the user asks about Tandem tasks, decisions, accords, rules, reviews, or logs.",
		],
		parameters: Type.Object({
			...cwdSchema,
			...jsonSchema,
			query: Type.String(),
			state: Type.Optional(Type.String()),
			type: Type.Optional(Type.String()),
			parent: Type.Optional(Type.String({ description: "Filter results to documents whose parentId exactly matches this ID." })),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTandemTool("tandem_search", buildSearchArgs(params as SearchToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tandem_search argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerCommand("tandem", {
		description: "Show pi-tandem help or status",
		handler: async (args, ctx) => {
			const subcommand = args.trim() || "status";
			if (subcommand === "help") {
				const text = [
					"pi-tandem commands:",
					"/tandem status  Diagnose tandem and the nearest .tandem workspace.",
					"/tandem help    Show this help.",
					"",
					"LLM-callable tools: tandem_status, tandem_init, tandem_task, tandem_accord, tandem_log, tandem_rules, tandem_decision, tandem_search.",
					"Adapter rule: these tools call the installed tandem CLI; they do not reimplement Tandem protocol behavior.",
				].join("\n");
				ctx.ui.setWidget("pi-tandem", text.split("\n"));
				ctx.ui.notify("pi-tandem help shown", "info");
				return;
			}
			if (subcommand !== "status") {
				ctx.ui.notify("Unknown /tandem subcommand. Use /tandem help or /tandem status.", "warning");
				return;
			}
			const status = await buildStatusText(ctx.cwd, {}, ctx.signal);
			ctx.ui.setWidget("pi-tandem", status.content[0]?.text.split("\n") ?? []);
			ctx.ui.notify(status.isError ? "pi-tandem status found a problem" : "pi-tandem status ok", status.isError ? "warning" : "info");
		},
	});

	pi.on("before_agent_start", async (event, ctx) => {
		const workspaceRoot = findWorkspaceRoot(ctx.cwd);
		if (!workspaceRoot && !promptMentionsDurableCoordination(event.prompt)) return;
		return { systemPrompt: `${event.systemPrompt}${tandemPromptGuidance(workspaceRoot)}` };
	});
}
