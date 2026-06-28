import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
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

// pi-tandem is intentionally a lightweight adapter over the installed `tdm` CLI.
// Tandem protocol parsing/mutation behavior belongs in tandem-tui/tdm, not here.

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

export type TaskToolParams = CwdFlag & ReadJsonFlag & {
	action: "list" | "show" | "add" | "move" | "complete";
	id?: string;
	title?: string;
	state?: string;
	description?: string;
	priority?: string;
	type?: string;
	tags?: string[];
	assignee?: string;
	dueDate?: string;
	parent?: string;
	blockers?: string[];
	references?: string[];
	relatedFiles?: string[];
	subtasks?: string[];
	accord?: string;
	review?: string;
	filesChanged?: string[];
	summary?: string;
	validation?: string;
	reviewer?: string;
};

export type AccordToolParams = CwdFlag & {
	action: "ready" | "claim" | "deliver" | "accept" | "rework" | "block" | "fail";
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
	references?: string[];
	tags?: string[];
};

export type SearchToolParams = CwdFlag & ReadJsonFlag & {
	query: string;
	state?: string;
	type?: string;
};

const DEFAULT_TIMEOUT_MS = 30_000;
const MAX_TIMEOUT_MS = 120_000;
const MAX_BUFFER_BYTES = 10 * 1024 * 1024;
const TDM_BIN_ENV_KEYS = ["TANDEM_TDM_BIN", "TDM_BIN"] as const;

function tdmBinary(): string {
	for (const key of TDM_BIN_ENV_KEYS) {
		const value = process.env[key]?.trim();
		if (value) return value;
	}
	return "tdm";
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

function addRepeatedFlag(args: string[], flag: string, values: unknown): void {
	if (!Array.isArray(values)) return;
	for (const value of values) {
		if (typeof value === "string" && value.trim()) args.push(flag, value);
	}
}

function wantsJson(params: ReadJsonFlag): boolean {
	return params.json !== false;
}

export function buildTaskArgs(params: TaskToolParams): string[] {
	const action = params.action;
	if (action === "list") {
		const args = ["list"];
		addOptionalFlag(args, "--state", params.state);
		addOptionalFlag(args, "--type", params.type);
		addOptionalFlag(args, "--priority", params.priority);
		addOptionalFlag(args, "--tag", params.tags?.[0]);
		addOptionalFlag(args, "--assignee", params.assignee);
		addOptionalFlag(args, "--accord", params.accord);
		addOptionalFlag(args, "--review", params.review);
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (action === "show") {
		const args = ["show", requireString(params.id, "tdm_task show requires id")];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (action === "add") {
		const args = ["add", "--title", requireString(params.title, "tdm_task add requires title")];
		addOptionalFlag(args, "--state", params.state);
		addOptionalFlag(args, "--description", params.description);
		addOptionalFlag(args, "--priority", params.priority);
		addRepeatedFlag(args, "--tag", params.tags);
		addOptionalFlag(args, "--assignee", params.assignee);
		addOptionalFlag(args, "--due-date", params.dueDate);
		addOptionalFlag(args, "--parent", params.parent);
		addRepeatedFlag(args, "--blocker", params.blockers);
		addRepeatedFlag(args, "--reference", params.references);
		addRepeatedFlag(args, "--related-file", params.relatedFiles);
		addRepeatedFlag(args, "--subtask", params.subtasks);
		return args;
	}
	if (action === "move") {
		return ["move", requireString(params.id, "tdm_task move requires id"), "--state", requireString(params.state, "tdm_task move requires state")];
	}
	if (action === "complete") {
		const args = ["complete", requireString(params.id, "tdm_task complete requires id"), "--summary", requireString(params.summary, "tdm_task complete requires summary")];
		addRepeatedFlag(args, "--file-changed", params.filesChanged);
		addOptionalFlag(args, "--validation", params.validation);
		addOptionalFlag(args, "--reviewer", params.reviewer);
		return args;
	}
	throw new Error(`unsupported tdm_task action: ${String(action)}`);
}

export function buildAccordArgs(params: AccordToolParams): string[] {
	const args = ["accord", params.action, requireString(params.id, "tdm_accord requires id")];
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
		if (params.limit !== undefined) args.push("--limit", String(requirePositiveInteger(params.limit, "tdm_log list limit must be a positive integer")));
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "show") {
		const args = ["log", "show", requireString(params.id, "tdm_log show requires id")];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "search") {
		const args = ["log", "search", requireString(params.query, "tdm_log search requires query")];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	throw new Error(`unsupported tdm_log action: ${String(params.action)}`);
}

export function buildRulesArgs(params: RulesToolParams): string[] {
	if (params.action === "list") {
		const args = ["rules", "list"];
		addOptionalFlag(args, "--category", params.category);
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "add") {
		const args = ["rules", "add", "--category", requireString(params.category, "tdm_rules add requires category"), "--rule", requireString(params.rule, "tdm_rules add requires rule")];
		addOptionalFlag(args, "--source", params.source);
		return args;
	}
	if (params.action === "edit") {
		const args = ["rules", "edit", "--category", requireString(params.category, "tdm_rules edit requires category"), "--id", String(requirePositiveInteger(params.id, "tdm_rules edit requires a positive id")), "--rule", requireString(params.rule, "tdm_rules edit requires rule")];
		addOptionalFlag(args, "--source", params.source);
		return args;
	}
	if (params.action === "delete") {
		return ["rules", "delete", "--category", requireString(params.category, "tdm_rules delete requires category"), "--id", String(requirePositiveInteger(params.id, "tdm_rules delete requires a positive id"))];
	}
	throw new Error(`unsupported tdm_rules action: ${String(params.action)}`);
}

export function buildDecisionArgs(params: DecisionToolParams): string[] {
	if (params.action === "list") {
		const args = ["decision", "list"];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "show") {
		const args = ["decision", "show", requireString(params.id, "tdm_decision show requires id")];
		if (wantsJson(params)) args.push("--json");
		return args;
	}
	if (params.action === "add") {
		const args = ["decision", "add", "--title", requireString(params.title, "tdm_decision add requires title")];
		addOptionalFlag(args, "--body", params.body);
		addRepeatedFlag(args, "--reference", params.references);
		addRepeatedFlag(args, "--tag", params.tags);
		return args;
	}
	throw new Error(`unsupported tdm_decision action: ${String(params.action)}`);
}

export function buildSearchArgs(params: SearchToolParams): string[] {
	const args = ["search", requireString(params.query, "tdm_search requires query")];
	addOptionalFlag(args, "--state", params.state);
	addOptionalFlag(args, "--type", params.type);
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
		return `Missing tdm CLI: could not execute ${JSON.stringify(result.command)}. Build/install tandem-tui so \`tdm\` is on PATH, or set ${TDM_BIN_ENV_KEYS.join("/")} to the binary path.`;
	}
	if (/No Tandem workspace found|\.tandem/i.test(combined) && /tdm init/i.test(combined)) {
		return "Missing Tandem workspace: no .tandem/tandem.md was discovered from this cwd. Run `tdm init --title <title>` only after the user wants a workspace.";
	}
	if (/unknown .*flag|unknown .*subcommand|unknown command/i.test(combined)) {
		return "Unsupported tdm CLI surface: the installed `tdm` may be older than pi-tandem expects, or this wrapper passed a flag/subcommand not supported by the current CLI.";
	}
	if (result.timedOut) return "tdm command timed out before it completed.";
	if (result.aborted) return "tdm command was aborted before it completed.";
	return "tdm command failed.";
}

function runTdm(args: string[], cwd: string, signal?: AbortSignal, timeoutMs?: number): Promise<RunResult> {
	const command = tdmBinary();
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
				error: "tdm command aborted",
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
	const fullOutputPath = join(tempDir, "tdm-output.txt");
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

async function executeTdmTool(label: string, args: string[], baseCwd: string, params: CwdFlag, signal?: AbortSignal, onUpdate?: (update: ToolResult) => void): Promise<ToolResult> {
	const cwd = resolveCwd(baseCwd, params.cwd);
	onUpdate?.({ content: [{ type: "text", text: `tdm ${args.join(" ")}` }], details: { cwd, args } });
	const result = await runTdm(args, cwd, signal, params.timeoutMs);
	return formatRunResult(label, result);
}

async function buildStatusText(baseCwd: string, params: CwdFlag = {}, signal?: AbortSignal): Promise<ToolResult> {
	const cwd = resolveCwd(baseCwd, params.cwd);
	const workspaceRoot = findWorkspaceRoot(cwd);
	const help = await runTdm(["--help"], cwd, signal, params.timeoutMs);
	const lines = ["# pi-tandem status", "", `cwd: ${cwd}`, `tdm binary: ${tdmBinary()}`];
	lines.push(workspaceRoot ? `workspace: ${workspaceRoot}` : "workspace: not found (.tandem/tandem.md missing from cwd or parents)");
	if (!help.ok) {
		const diagnostic = classifyFailure(help);
		lines.push("", diagnostic);
		return { content: [{ type: "text", text: lines.join("\n") }], details: { cwd, workspaceRoot, help, diagnostic }, isError: true };
	}
	lines.push("tdm: available");
	if (workspaceRoot) {
		const list = await runTdm(["list", "--json"], cwd, signal, params.timeoutMs);
		if (list.ok) {
			const parsed = parseJsonIfPossible(list.stdout) as { data?: { counts?: unknown } } | undefined;
			lines.push("tdm list --json: ok");
			if (parsed?.data?.counts) lines.push(`counts: ${JSON.stringify(parsed.data.counts)}`);
			return { content: [{ type: "text", text: lines.join("\n") }], details: { cwd, workspaceRoot, help, list, parsedJson: parsed } };
		}
		const diagnostic = classifyFailure(list);
		lines.push("", `tdm list --json failed: ${diagnostic}`);
		return { content: [{ type: "text", text: lines.join("\n") }], details: { cwd, workspaceRoot, help, list, diagnostic }, isError: true };
	}
	lines.push("hint: run `tdm init --title <title>` only after the user wants this directory to become a Tandem workspace.");
	return { content: [{ type: "text", text: lines.join("\n") }], details: { cwd, workspaceRoot, help } };
}

const cwdSchema = {
	cwd: Type.Optional(Type.String({ description: "Working directory. Defaults to Pi's current cwd; relative paths resolve from there." })),
	timeoutMs: Type.Optional(Type.Number({ description: `Command timeout in milliseconds. Defaults to ${DEFAULT_TIMEOUT_MS}; max ${MAX_TIMEOUT_MS}.` })),
};

const jsonSchema = {
	json: Type.Optional(Type.Boolean({ description: "For read actions, request `tdm --json` output. Defaults to true." })),
};

function tandemPromptGuidance(workspaceRoot?: string): string {
	const workspaceLine = workspaceRoot ? `A Tandem workspace is present at ${workspaceRoot}.` : "No Tandem workspace is currently detected from the working directory.";
	return `\n\n## Tandem coordination guidance\n\n${workspaceLine}\n\n- Prefer pi-tandem tools (tdm_status, tdm_task, tdm_accord, tdm_log, tdm_rules, tdm_decision, tdm_search) over manual edits to .tandem files for durable coordination.\n- Keep Tandem behavior in the tdm CLI/protocol; use pi-tandem as a thin adapter and diagnostics layer.\n- Use tdm_accord for claiming, delivering, accepting, reworking, blocking, or failing work agreements. Do not mark accords accepted/completed unless the user or orchestrator asks.\n- Use tdm_log and tdm_search for completed-work history instead of treating logs as trash/archive only.\n`;
}

function promptMentionsDurableCoordination(prompt: string): boolean {
	return /\b(tandem|tdm|durable coordination|task board|accord|work agreement|review queue|project rules|completed logs?)\b/i.test(prompt);
}

export default function piTandem(pi: ExtensionAPI) {
	pi.registerTool({
		name: "tdm_status",
		label: "Tandem Status",
		description: "Diagnose the installed `tdm` CLI and the nearest `.tandem` workspace.",
		promptSnippet: "Use tdm_status to inspect Tandem/tdm health before durable project coordination or when .tandem diagnostics are needed.",
		promptGuidelines: [
			"Use tdm_status when entering a Tandem repo, when `.tandem/tandem.md` may exist, or before bootstrapping durable coordination.",
			"If tdm_status reports no workspace, ask before initializing a new Tandem workspace; do not create `.tandem` state implicitly.",
		],
		parameters: Type.Object({ ...cwdSchema }),
		async execute(_toolCallId, params, signal, _onUpdate, ctx) {
			return buildStatusText(ctx.cwd, params, signal);
		},
	});

	pi.registerTool({
		name: "tdm_task",
		label: "Tandem Task",
		description: "Run task-oriented `tdm` commands: list, show, add, move, or complete. Read actions default to `--json`; mutations preserve human-readable CLI output.",
		promptSnippet: "Use tdm_task for Tandem task list/show/add/move/complete operations instead of editing .tandem files directly.",
		promptGuidelines: [
			"Use tdm_task for active Tandem task reads and mutations when `.tandem/tandem.md` exists.",
			"Prefer tdm_task read actions with the default JSON output for reliable task inspection.",
			"Do not use tdm_task complete unless the user or orchestrator explicitly asks to archive completed work.",
		],
		parameters: Type.Object({
			...cwdSchema,
			...jsonSchema,
			action: StringEnum(["list", "show", "add", "move", "complete"] as const),
			id: Type.Optional(Type.String()),
			title: Type.Optional(Type.String()),
			state: Type.Optional(Type.String()),
			type: Type.Optional(Type.String({ description: "Filter document type for list, usually task or decision." })),
			description: Type.Optional(Type.String()),
			priority: Type.Optional(Type.String()),
			tags: Type.Optional(Type.Array(Type.String())),
			assignee: Type.Optional(Type.String()),
			dueDate: Type.Optional(Type.String()),
			parent: Type.Optional(Type.String()),
			blockers: Type.Optional(Type.Array(Type.String())),
			references: Type.Optional(Type.Array(Type.String())),
			relatedFiles: Type.Optional(Type.Array(Type.String())),
			subtasks: Type.Optional(Type.Array(Type.String())),
			accord: Type.Optional(Type.String()),
			review: Type.Optional(Type.String()),
			filesChanged: Type.Optional(Type.Array(Type.String())),
			validation: Type.Optional(Type.String()),
			reviewer: Type.Optional(Type.String()),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTdmTool("tdm_task", buildTaskArgs(params as TaskToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tdm_task argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tdm_accord",
		label: "Tandem Accord",
		description: "Run `tdm accord ready|claim|deliver|accept|rework|block|fail` as a thin Pi wrapper.",
		promptSnippet: "Use tdm_accord for Tandem work-agreement lifecycle actions (ready, claim, deliver, accept, rework, block, fail).",
		promptGuidelines: [
			"Use tdm_accord for accord state changes instead of direct frontmatter edits.",
			"Do not accept, complete, or otherwise finalize Tandem work unless the user/orchestrator explicitly asks for that lifecycle transition.",
		],
		parameters: Type.Object({
			...cwdSchema,
			action: StringEnum(["ready", "claim", "deliver", "accept", "rework", "block", "fail"] as const),
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
				return await executeTdmTool("tdm_accord", buildAccordArgs(params as AccordToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tdm_accord argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tdm_log",
		label: "Tandem Logs",
		description: "Run completed-work log reads: `tdm log list|show|search`. Defaults to JSON output.",
		promptSnippet: "Use tdm_log for Tandem completed-work history list/show/search.",
		promptGuidelines: [
			"Use tdm_log for completed Tandem work instead of manually reading `.tandem/logs` unless raw source inspection is required.",
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
				return await executeTdmTool("tdm_log", buildLogArgs(params as LogToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tdm_log argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tdm_rules",
		label: "Tandem Rules",
		description: "Run `tdm rules list|add|edit|delete`. List defaults to JSON; mutations preserve human-readable CLI output.",
		promptSnippet: "Use tdm_rules to inspect or update Tandem project rules through tdm.",
		promptGuidelines: [
			"Use tdm_rules for Tandem project rules rather than editing `.tandem/tandem.md` by hand.",
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
				return await executeTdmTool("tdm_rules", buildRulesArgs(params as RulesToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tdm_rules argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tdm_decision",
		label: "Tandem Decisions",
		description: "Run `tdm decision list|show|add`. Read actions default to JSON; add preserves human-readable CLI output.",
		promptSnippet: "Use tdm_decision for Tandem decision document list/show/add operations.",
		promptGuidelines: [
			"Use tdm_decision for first-class Tandem decision documents; do not model decisions as task lifecycle state.",
		],
		parameters: Type.Object({
			...cwdSchema,
			...jsonSchema,
			action: StringEnum(["list", "show", "add"] as const),
			id: Type.Optional(Type.String()),
			title: Type.Optional(Type.String()),
			body: Type.Optional(Type.String()),
			references: Type.Optional(Type.Array(Type.String())),
			tags: Type.Optional(Type.Array(Type.String())),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTdmTool("tdm_decision", buildDecisionArgs(params as DecisionToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tdm_decision argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
			}
		},
	});

	pi.registerTool({
		name: "tdm_search",
		label: "Tandem Search",
		description: "Run `tdm search <query>` across active Tandem documents and completed logs. Defaults to JSON output.",
		promptSnippet: "Use tdm_search for project work search across active Tandem documents and logs.",
		promptGuidelines: [
			"Use tdm_search before ad hoc file grep when the user asks about Tandem tasks, decisions, accords, rules, reviews, or logs.",
		],
		parameters: Type.Object({
			...cwdSchema,
			...jsonSchema,
			query: Type.String(),
			state: Type.Optional(Type.String()),
			type: Type.Optional(Type.String()),
		}),
		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			try {
				return await executeTdmTool("tdm_search", buildSearchArgs(params as SearchToolParams), ctx.cwd, params, signal, onUpdate);
			} catch (err) {
				return { content: [{ type: "text", text: `tdm_search argument error: ${normalizeError(err)}` }], details: { error: normalizeError(err) }, isError: true };
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
					"/tandem status  Diagnose tdm and the nearest .tandem workspace.",
					"/tandem help    Show this help.",
					"",
					"LLM-callable tools: tdm_status, tdm_task, tdm_accord, tdm_log, tdm_rules, tdm_decision, tdm_search.",
					"Adapter rule: these tools call the installed tdm CLI; they do not reimplement Tandem protocol behavior.",
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
