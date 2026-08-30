# Clap migration research

Research artifact for `task-246`. Research and specification only. No `clap`
dependency, no CLI behavior change, no external workspace mutation.

**Status: phase 1 of 3 complete.** Section 2 (grammar and compatibility
inventory, including the 2.13 simplification audit) and the PC1 half of section
7 are done and ready for owner review. Sections 1, 3, 4, 5, 6, 8, 9, and 10 are
deliberately unwritten pending that review.

Owner direction 2026-08-30 widened the mandate: the cutover may rewrite any
command, flag, or output as long as the result is correct. There is no
anchoring on current shapes, aliases are permitted, misaligned flags are to be
reworked rather than preserved, and one-offs need an extremely good reason to
survive. Section 2.11 is therefore an inventory of what exists, not a list of
guarantees.

A second direction the same day removed the remaining anchor: the "locked v0
decisions" in `AGENTS.md` are not binding on this work either. Nothing is
protected by having been decided before. No backups, no fallbacks, no
compatibility shims — if a change is wrong it gets reverted or fixed when it
surfaces. Section 2.15 exists because of that direction.

Evidence: `tandem 0.11.0` release binary probed against a temporary workspace,
plus source reading of `tandem/src/cli/{mod,args,commands,landing}.rs`,
`tandem/src/main.rs`, `tandem/tests/cli_behavior.rs`, and
`extensions/pi-tandem/index.ts`.

## 1. Executive recommendation

Adopt `clap` 4.6.x with default features plus `derive`, and replace the
handwritten parser in one comprehensive protocol 0.3.0/core CLI cutover. Do not
migrate the existing 39-command grammar unchanged. The parser is only one
symptom: Tandem currently expresses the same read, lifecycle, and record
operations through parallel Task, Log, Decision, Papercut, Accord, and Review
command families.

The selected cutover reduces the core to 23 invocable leaf commands across 13
root subcommands and three nested families (`add`, `accord`, `rules`). It
retains separate structured `list` and full-text `search`, makes `--json`
global, unifies lookup/update, removes direct state movement and Review status,
makes Accord mandatory for active Tasks, moves Papercuts into tagged Tasks,
and introduces a clean protocol 0.3.0 storage layout.

Implement it as one isolated Task/branch with ordered commits and no dual
parser, compatibility reader, upgrade/migrate command, or partial merge. The
complete interactive product decisions are authoritative in
[`plan/cli-protocol-cutover.md`](../../plan/cli-protocol-cutover.md); preliminary
proposals below are retained only as research history where marked.

## 2. Current CLI grammar and compatibility matrix

### 2.1 Shape of the existing parser

`main.rs` passes `env::args().skip(1)` to `cli::run`. `cli::dispatch` removes
argv[0] as the command word and routes to one of 19 arms. There is no global
option layer: every flag is parsed by a per-command `parse_*_args` function, so
the same flag name is re-declared in each command that accepts it.

Each parser is the same hand-rolled loop:

```rust
while index < args.len() {
    match args[index].as_str() {
        "--flag" => { index += 1; value = required_value(args, index, "--flag")?; }
        flag if flag.starts_with('-') => return Err(CliError::usage(...)),
        value => /* positional or error */,
    }
    index += 1;
}
```

Consequences that matter for the cutover:

- Parsing always completes before workspace discovery. `tandem list --state`
  with no `.tandem/` exits 2 (usage), not 1 (missing workspace).
- No `--flag=value` form is accepted anywhere. Values are always the next argv
  element.
- No short flags, no aliases, no abbreviation. `-h` and `-j` are unknown.
- No `--` end-of-options separator.
- Scalar flags are last-wins and silently so: `add --title A --title B` creates
  the task titled `B` with no warning.

### 2.2 Command tree

19 top-level dispatch arms: 13 flat commands, 6 nested subcommand families
containing 26 leaves. That is **39 invocable commands** and 46 places where
`--help` is meaningful (root, 13 flat, 6 bare families, 26 leaves).

| Command | Subcommands | Positional | Workspace required |
| --- | --- | --- | --- |
| *(no args)* | — | — | no (prints landing, exit 0) |
| `init` | — | — | no (creates) |
| `upgrade` | — | — | yes |
| `list` | — | — | yes |
| `show` | — | `<id>` required | yes |
| `add` | — | — (positional is an error) | yes |
| `move` | — | `<id>` required | yes |
| `update` | — | `<id>` required | yes |
| `complete` | — | `<id>` required | yes |
| `cancel` | — | `<id>` required | yes |
| `search` | — | `<query>` required | yes |
| `papercut` | `add`, `list`, `show`, `resolve` | per-subcommand | yes |
| `log` | `list`, `show`, `search` | per-subcommand | yes |
| `accord` | `claim`, `deliver`, `accept`, `rework`, `block`, `fail` | `<id>` required | yes |
| `review` | `request`, `accept`, `changes`, `reject` | `<id>` required | yes |
| `rules` | `list`, `add`, `edit`, `delete` | — | yes |
| `decision` | `list`, `show`, `add`, `update`, `withdraw` | per-subcommand | yes |
| `tui` | — | — | yes (opened after parse) |
| `web` | — | — | yes (opened after parse) |
| `version` / `--version` | — | — | no |
| `help` / `--help` | — | — | no |

Two drift findings against the documented surface:

- **F1.** `tandem --help` advertises `decision list|show|add` but `decision`
  also dispatches `update` and `withdraw`. The static help text is stale and is
  asserted verbatim by `cli_behavior.rs`, so the test locks in the drift.
- **F2.** The landing page in `landing.rs` lists 19 commands and omits `review`
  entirely, while `cli_behavior.rs` asserts a list that also omits `review`.
  Both the code and its test agree on an incomplete command list.

### 2.3 Flag inventory

49 distinct long flag names across all commands. `--json` appears in 11 parsers,
`--title` in 5, `--parent` in 4.

| Command | Flags |
| --- | --- |
| `init` | `--title <v>`, `--force` |
| `upgrade` | none (any argument is an error) |
| `list` | `--state`, `--type`, `--priority`, `--tag`, `--assignee`, `--parent`, `--accord`, `--review`, `--json` |
| `show` | `--json` |
| `add` | `--title`, `--state`, `--description`, `--kind`, `--priority`, `--effort`, `--tag`\*, `--assignee`, `--due-date`, `--parent`, `--blocker`\*, `--reference`\*, `--related-file`\*, `--json`, `--subtask`† |
| `move` | `--state` |
| `update` | `--title`, `--body`‡, `--kind`, `--priority`, `--effort`, `--assignee`, `--due-date`, `--parent`, `--tag`\*, `--blocker`\*, `--reference`\*, `--related-file`\*, `--state`†, `--parent-id`†, `--parentId`†, `--subtask`† |
| `complete` | `--summary`, `--file-changed`\*, `--validation`, `--reviewer` |
| `cancel` | `--reason`‡ |
| `search` | `--state`, `--type`, `--parent`, `--json` |
| `papercut add` | `--title`, `--body`‡, `--reference`\*, `--tag`\* |
| `papercut list` | `--status`, `--all`, `--json` |
| `papercut show` | `--json` (reuses `parse_show_args`) |
| `papercut resolve` | `--note`, `--reference`\* |
| `log list` | `--limit`, `--json` |
| `log show` | `--json` (reuses `parse_show_args`) |
| `log search` | `--json` |
| `accord <action>` | `--assignee`, `--summary`, `--reviewer`, `--note`, `--reason`, `--deliverable`\*, `--validation`\*, `--constraint`\*, `--evidence`\*, `--file-changed`\* |
| `review <action>` | `--reviewer`, `--note`‡, `--json` |
| `rules list` | `--category`, `--json` |
| `rules add` | `--category`, `--rule`, `--source` |
| `rules edit` | `--category`, `--id`, `--rule`, `--source` |
| `rules delete` | `--category`, `--id` |
| `decision list` | `--json` |
| `decision show` | `--json` (reuses `parse_show_args`) |
| `decision add` | `--title`, `--body`, `--status`, `--date`, `--decider`\*, `--context`, `--consequence`\*, `--alternative`\*, `--supersedes`\*, `--superseded-by`\*, `--reference`\*, `--tag`\* |
| `decision update` | `--title`, `--body`, `--status` (at least one required) |
| `decision withdraw` | `--reason` (required, exact position) |
| `tui` | none (any argument is an error) |
| `web` | `--port`, `--no-open`, `--help` |

\* repeatable, accumulates into a `Vec<String>`
† rejected with a guidance error, not accepted
‡ parsed with `required_raw_value`, so the value may begin with `-`

### 2.4 Value acceptance

Two helpers decide whether a value may start with `-`:

- `required_value` — rejects any next argument starting with `-`, reporting
  `<flag> requires a value`. Used by 45 of 49 flags.
- `required_raw_value` — accepts anything present. Used by exactly four flags:
  `update --body`, `cancel --reason`, `papercut add --body`, `review --note`.

**F3.** That split is inconsistent and looks accidental rather than designed.
Free-text flags that can legitimately carry Markdown or a leading dash are
split across both helpers with no rule behind the split:

| Flag | Helper | Accepts `-x`? |
| --- | --- | --- |
| `update --body` | raw | yes |
| `papercut add --body` | raw | yes |
| `decision add --body` | strict | **no** |
| `decision update --body` | strict | **no** |
| `cancel --reason` | raw | yes |
| `accord --reason` | strict | **no** |
| `review --note` | raw | yes |
| `accord --note` | strict | **no** |
| `papercut resolve --note` | strict | **no** |
| `complete --summary` | strict | **no** |
| every `--title` | strict | **no** |

Verified: `update task-1 --body "- bullet start"` succeeds;
`decision add --title D --body "-neg"` fails with `--body requires a value`.

Empty strings pass the parser everywhere and are rejected or accepted by the
app layer. `update --body ""` clears the body and reports `body: changed`;
`update --title ""` fails with the exit-2 app message `update --title must not
be empty`. This is the parser/app validation boundary the cutover must preserve.

Typed values, all validated inside the parser:

| Flag | Type | Rule |
| --- | --- | --- |
| `web --port` | `u16` | 1–65535, 0 rejected |
| `log list --limit` | `usize` | 0 accepted, negatives rejected as a flag-looking value |
| `rules edit/delete --id` | `usize` | must be > 0 |

**F4.** `log list --limit -1` reports `--limit requires a value` rather than a
type error, because `-1` is caught by the leading-dash guard first.

### 2.5 Positional handling

`set_single_positional` accepts the first non-flag token and errors on the
second with `unexpected extra <command> argument`. Positionals may appear
before, between, or after flags. Missing positionals produce
`<command> requires an <id>` from the parser.

Commands that reject all positionals (`add`, `init`, `list`, `log list`,
`papercut add`, `papercut list`, `rules *`, `decision add`, `upgrade`, `tui`,
`web`) each emit their own `unexpected <command> argument` message.

**F5.** `decision update` and `decision withdraw` break the shared pattern.
Both use `split_first` to take the id as the mandatory *first* argument, so a
flag before the id is consumed as the id. `decision update` has no
`starts_with('-')` guard at all, so any stray positional after the id is
reported as `unknown decision update flag`. `decision withdraw` requires
literally `<id> --reason <text>` with `rest.len() == 2`.

### 2.6 `--json` contract

`--json` is per-command, never global. `tandem --json list` fails with
`unknown command '--json'`. Accepted on 11 leaf commands: `list`, `show`, `add`,
`search`, `papercut list`, `papercut show`, `log list`, `log show`,
`log search`, `rules list`, `decision list`, `decision show`, `review <action>`.

Not accepted on: `init`, `upgrade`, `move`, `update`, `complete`, `cancel`,
`papercut add`, `papercut resolve`, `accord <action>`, `rules add|edit|delete`,
`decision add|update|withdraw`, `tui`, `web`.

The JSON envelope is `{"ok":true,"data":{...},"warnings":[...]}` on stdout.
JSON mode also suppresses the workspace deprecation warnings and per-document
warnings that human mode prints, folding them into the `warnings` array.

### 2.7 Help and version

| Invocation | Result | Exit |
| --- | --- | --- |
| *(no args)* | landing page, ANSI-styled when stdout is a tty and `NO_COLOR` is unset | 0 |
| `--help`, `help` | static 21-line usage block | 0 |
| `--version`, `version` | `tandem 0.11.0` | 0 |
| `web --help` | real per-command help with an options block | 0 |
| `-h` | `unknown command '-h'` | 2 |
| `papercut --help` | `unknown papercut subcommand '--help'` | 2 |
| `log --help` | `unknown log subcommand '--help'` | 2 |
| `accord --help` | `unknown accord subcommand '--help'` | 2 |
| `review --help` | `unknown review subcommand '--help'` | 2 |
| `rules --help` | `unknown rules subcommand '--help'` | 2 |
| `decision --help` | `unknown decision subcommand '--help'` | 2 |
| `add --help` | `unknown add flag '--help'` | 2 |
| `show`/`update`/`move`/`complete`/`cancel`/`search`/`list`/`init` `--help` | `unknown <cmd> flag '--help'` | 2 |
| `tui --help` | `tui does not accept arguments` | 2 |
| `papercut add --help` etc. | `unknown papercut add flag '--help'` | 2 |

This is PC1. The landing page ends with `Run 'tandem <command> --help' for
detailed usage.` — a promise satisfied by 2 of 46 help surfaces (the root and
`web`).

### 2.8 Errors and exit codes

Three codes, defined in `main.rs`:

- `0` — success.
- `1` — `CliError::user`: runtime, data, and write failures, including
  `No Tandem workspace found. Run 'tandem init' first.`, `document not found`,
  every `io::Error`, and every `protocol::diagnostic::Diagnostic`.
- `2` — `CliError::usage`: all argument and grammar failures, plus a set of app
  guard messages such as `update --title must not be empty`,
  `move requires --state <state>`, and `rules add requires --rule <text>`.

Errors go to stderr, prefixed `Error: `, exactly one line. Success output goes
to stdout. Errors are returned as values, never printed by the parser, and
never panic. Stdout is empty on every error path.

**F6.** Exit code 2 is not exclusively parser-owned. `move --state`,
`rules add --rule`, and `update --title` produce usage-class errors from
`commands.rs` and the app layer after parsing succeeds. Any cutover that assumes
"clap owns exit 2" will get this wrong.

### 2.9 Behavior depended on by tests

`tandem/tests/cli_behavior.rs` runs the real binary at the process boundary.
`process_help_version_usage_and_missing_project_contracts` asserts, byte for
byte:

- the entire landing page prefix, the four group headings, the 18-command list,
  and the trailing `Run 'tandem <command> --help'` line;
- `NO_COLOR=1` output is byte-identical to the non-tty default;
- the complete 21-line `--help` text;
- `tandem {CARGO_PKG_VERSION}\n` for `--version`;
- the exact `unknown command` message including the full supported-command list;
- `Error: unknown list flag '--unknown'\n` with exit 2 and empty stdout;
- `Error: No Tandem workspace found. Run 'tandem init' first.\n` with exit 1;
- `Error: tui does not accept arguments\n` with exit 2.

`command_families_preserve_exact_success_output` and the other 10 integration
tests assert exact success output for every command family. Two unit tests in
`args.rs` assert `parse_web_args` port validation and that
`parse_list_args(["-j"])` fails with code 2 and message
`unknown list flag '-j'`.

Every one of these is a compatibility constraint the cutover has to either
satisfy or explicitly renegotiate.

### 2.10 Behavior depended on by `extensions/pi-tandem`

The adapter builds argument arrays in TypeScript and parses the `--json`
envelope. It never touches Markdown. Relevant couplings:

- `addOptionalFlag` skips empty/whitespace strings; `addPresentStringFlag`
  passes any string including `""`, used for `update --body` and
  `papercut add --body` — it depends on empty-string acceptance.
- `--tag` is emitted repeatedly for `add`, but for `list` only
  `params.tags?.[0]` is sent, matching the CLI's scalar `list --tag`.
- Numeric `--limit` is stringified after a positive-integer check.
- The adapter relies on exit code and stderr text to surface failures.

**F7 (bug).** `tandem_task action=list` emits `--effort <value>` when the caller
passes `effort`, but `parse_list_args` has no `--effort` flag. `tandem list
--effort small` exits 2 with `unknown list flag '--effort'`. This is a live
adapter/CLI mismatch that exists today and is independent of the clap question.

### 2.11 Intentional vs accidental

Intentional *as currently designed*. Under the widened mandate these are
rationales to re-derive, not fixed constraints; C1 in particular is explicitly
reopened (aliases are now permitted):

- **C1.** Long flags only, no short aliases, no abbreviation.
- **C2.** Exit codes 0/1/2 with the documented category split.
- **C3.** `Error: ` prefix on stderr, empty stdout on failure.
- **C4.** Help, version, and the landing page work without a workspace.
- **C5.** Parse errors precede workspace discovery.
- **C6.** `--json` envelope shape and per-command placement.
- **C7.** Repeatable list flags accumulate.
- **C8.** Empty-string values reach the app layer rather than failing at parse.
- **C9.** Free-text values may begin with `-` where a body, reason, or note is
  expected.
- **C10.** Deprecation guidance errors for `--subtask`, `update --state`, and
  `--parent-id`/`--parentId`.
- **C11.** The landing page, including `NO_COLOR` and tty behavior.

Accidental and should not be preserved:

- **A1.** PC1: generated/per-command help fails on 44 of 46 current help surfaces; only root help and `web --help` work.
- **A2.** F3: the arbitrary `required_value`/`required_raw_value` split.
- **A3.** F1/F2: stale `--help` text and the missing `review` landing entry.
- **A4.** F5: the two divergent `decision` parser shapes.
- **A5.** F4: `--limit -1` reported as a missing value.
- **A6.** Silent last-wins on repeated scalar flags.
- **A7.** 27 separately worded `unknown <command> flag` messages that a
  generated parser would render uniformly.

### 2.12 Compatibility questions — resolved

- Byte-exact current help/output is not a compatibility target; correctness and
  the selected new contract replace it.
- `-h`, `-V`, and `-j` are the only short aliases.
- Every prose-class value accepts leading hyphens; typed values remain strict.
- `list --effort` belongs in core Tandem because effort is a reasonable sibling
  filter. Pi is not authoritative.
- Existing workspace migration/retention is outside the protocol and cutover
  implementation; workspace owners handle it project by project.

### 2.13 Preliminary simplification audit (superseded by the interactive ledger)

The following S1–S15 proposals were the first audit pass. They identified real
redundancy but are not the selected design where they conflict with
`plan/cli-protocol-cutover.md`. In particular, the final design keeps full-text
`search`, keeps Task state plus mandatory Accord status, keeps intent-shaped Pi
tool choices open for the owning workspace, stores Rules as per-file records,
and preserves the current TUI Board architecture.

### 2.13.1 Original audit detail

The parser is large partly because the command set is. 39 commands and 49 flags
cover a protocol with four document kinds and three lifecycle axes. Most of the
excess is one capability expressed once per document type instead of once.

#### Evidence: the splits are implementation artifacts, not user needs

Probed against a live workspace containing a canceled task, a decision, and a
papercut:

| Probe | Result |
| --- | --- |
| `show task-1` where `task-1` is archived | **succeeds** — `show` already spans board *and* logs |
| `log show task-1` | same document, plus a `Log document` header line |
| `show decision-1` | **succeeds** — `show` already handles decisions |
| `decision show decision-1` | same document, plus a type guard |
| `show papercut-1` | **fails**, `document not found` |
| `papercut show papercut-1` | succeeds |
| `list --type decision` | same rows as `decision list`, different columns |
| `search one` | already returns board + logs + papercut hits in one call |

So `show` is already the general document reader for three of four kinds.
`log show` and `decision show` are wrappers that add a header and a type check.
`papercut show` exists because `show` has a lookup gap, not because papercuts
need their own verb. The same pattern holds for `list` and `search`.

#### Consolidation proposals

**S1 — one `show`.** Absorb `log show`, `decision show`, and `papercut show`
into `show <id>`. Fix the papercut lookup gap rather than routing around it.
Per-type rendering is a display concern keyed off `type`, not a command.
*4 commands → 1.*

**S2 — one `list`.** Absorb `log list`, `papercut list`, and `decision list`.
Location becomes a filter (`--location board|logs|papercuts|all`), `--limit`
becomes available everywhere instead of only on `log list`, and `--status`
/`--all` collapse into the existing filter vocabulary. Differing columns are a
renderer choice. *4 commands → 1.*

**S3 — fold `search` into `list --query`.** `search` is `list` plus a query
string; `log search` is `search` with a location filter S2 already provides.
The two commands share `--state`, `--type`, `--parent`, and `--json` today.
*2 commands → 0.*

**S4 — delete `move`.** `update <id> --state <state>` is the natural form.
`update` currently carries a hand-written arm that rejects `--state` and points
at `move`; deleting `move` deletes the command *and* the one-off error.
*1 command and 1 special case → 0.*

**S5 — delete `decision update` and `decision withdraw`.** A decision is a
document. `update <id> --status <s>` and `cancel <id> --reason <r>` already
express both. This also removes F5, the two parsers in the codebase that use a
different shape from the other 24. *2 commands and 2 parser shapes → 0.*

**S6 — one `add`.** `add --type task|decision|papercut` replaces `add`,
`decision add`, and `papercut add`. Type-specific flags (`--decider`,
`--consequence`, `--status`, `--date`) become conditionally-valid flags on one
command, which clap validates natively via `requires`/`conflicts_with`.
*3 commands → 1.*

**S7 — delete `papercut resolve`.** It is a status transition plus a note:
`update papercut-1 --status resolved --note <text>`. With S1, S2, S6, and this,
the `papercut` family disappears entirely. *1 command → 0.*

**S8 — composite rule IDs.** `rules edit --category always --id 12 --rule X`
becomes `rules edit always-12 --rule X`. `always-12` is already the form used
in prose and in `AGENTS.md`. This removes `--category` from four commands and
`--id` from two, and makes rules use positional identity like every other
command. *0 commands, 6 flag declarations → 0.*

**S9 — global `--json`.** Declared in 11 parsers today and accepted on 13
commands, absent on 26 with no principle behind the split. One global flag,
accepted everywhere, ignored where output is not structured.
*11 declarations → 1.*

**S10 — delete the deprecation arms.** `add --subtask`, `update --subtask`,
`update --state`, `update --parent-id`, `update --parentId` are five hand-written
error arms that exist to redirect pre-0.11 usage. A rewrite is the moment to
drop them. *5 special cases → 0.*

**S11 — align free-text flags.** Resolve F3 by one rule: every flag whose value
is prose (`--body`, `--reason`, `--note`, `--summary`, `--title`, `--rule`,
`--context`, `--consequence`, `--alternative`) accepts leading dashes. No
per-flag helper choice. *2 helpers → 1 policy.*

**S12 — align repeatable flags.** `--tag` is repeatable on `add` and scalar on
`list` (verified: `list --tag a --tag b` silently keeps `b`). Every list-valued
field should be repeatable in both filter and mutation positions.

#### Retained without change

- `complete` and `cancel` stay distinct verbs. They differ by
  `completion.outcome` and could be one command with a flag, but two verbs read
  better at the call site and cost one parser each.
- `init`, `tui`, `web`, `version` are irreducible.

`accord` and `review` were initially retained here on the authority of the
`AGENTS.md` v0 lock. That authority was withdrawn; see 2.15.

#### Decided

- **S13 — delete `upgrade`.** Approved by the owner 2026-08-30. Removes the
  command, `UpgradeOutcome`, `LEGACY_PROTOCOL_VERSION` handling, and the
  `print_workspace_deprecation_warnings` plumbing threaded through `list`,
  `show`, and `search`. A `0.1.0` workspace becomes an error, not a migration.

#### Net effect

| | Now | After S1–S12 |
| --- | --- | --- |
| Invocable commands | 39 | 25 |
| Subcommand families | 6 | 3 (`accord`, `review`, `rules`) |
| `--help` surfaces | 46 | 29 |
| Distinct long flags | 49 | ~38 |
| Flag declaration sites | 205 | ~60 |
| Hand-written parser fns | 26 | 0 |

Proposed command set:

```
tandem init | upgrade? | version | tui | web
tandem add    [--type task|decision|papercut] ...
tandem list   [--query <q>] [--location ...] [--limit n] [filters]
tandem show   <id>
tandem update <id> [--state <s>] [--status <s>] ...
tandem complete <id> | cancel <id>
tandem accord claim|deliver|accept|rework|block|fail <id>
tandem review request|accept|changes|reject <id>
tandem rules  list|add|edit|delete [<category-id>]
```

This is a command-model proposal, not a clap proposal. It is worth deciding
*before* the parser design in phase 2, because the two answers interact: a
25-command surface with one global flag layer is a materially different clap
type hierarchy than a 39-command one.

### 2.14 pi-tandem is not an authority

Per owner direction, `extensions/pi-tandem/` does not constrain the CLI. Where
the adapter exposes something the CLI lacks, the question is whether the
capability should exist:

- **F7, `list --effort`.** The adapter sends it; the CLI rejects it. Filtering
  active work by effort is reasonable and every sibling filter (`--priority`,
  `--assignee`, `--tag`) already exists. **Fix in Tandem** by adding
  `--effort` to the list filter set. No adapter change needed.
- **`tandem_task` `description`/`accord`/`review` on update.** The adapter
  tracks these as unsupported and reports them. No CLI change implied.

Any change the adapter itself needs is out of scope here under rule never-2 and
owner direction. The destination workspace is **`~/.pi`**, which is a live
Tandem workspace (29 documents, protocol 0.11.0). Note that
`extensions/pi-tandem/` and `~/.dotfiles/pi` are *not* Tandem workspaces, so
`~/.pi` is the only valid handoff target. No task has been created there;
section 9 will draft it for owner approval first.

### 2.15 The three-axis lifecycle is redundant state

A task carries three independent status fields:

| Field | Values | Count |
| --- | --- | --- |
| `state` | `todo`, `in-progress`, `validation` | 3 |
| `accord.status` | `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked` (+ legacy `ready`) | 6 |
| `review.status` | `not-ready`, `pending`, `accepted`, `changes-requested`, `rejected` | 5 |

That is 90 nominal combinations. A small number are legal, and legality is
enforced by three separate mechanisms in three files:
`accord::validate_transition`, `accord::state_effect`, and the inline checks in
`app::review`.

#### The axes are not independent

`state` is already largely a function of the other two, maintained by
hand-written coupling:

- `accord claim` with `state == todo` sets `state = in-progress`.
- `accord rework` with `state == validation` and `review == pending` sets
  `state = in-progress` and clears the review.
- `review request` forces `state = validation` and `review.status = pending`.
- `review accept|changes|reject` each write an explicit `state`.

#### The smoking gun

`protocol/accord.rs` contains `state_divergence_warning`:

> `{id} has workflow state 'todo' but accord.status 'claimed' suggests
> 'in-progress'; preserving recorded state until a mutation synchronizes it.`

The model stores state that can contradict itself, and rather than making the
contradiction unrepresentable, there is code to detect it, a message to report
it, and a documented decision to preserve the inconsistency. That is the
definition of redundant state.

#### Proposal S14 — one lifecycle axis

Merge `accord.status` and `review.status` into `state`:

```
todo → in-progress → delivered → review → accepted → (complete | cancel)
         ↑                │           │
         └── rework ───────┴───────────┘
         └── blocked / failed
```

Seven states on one axis, replacing 3 × 6 × 5. Divergence becomes
unrepresentable, and `state_divergence_warning` is deleted rather than
maintained.

Critically, this merges *status*, not payload. The accord record is a real
agreement document — `assignee`, `deliverables`, `validations`, `constraints`,
`evidence`, `filesChanged`, `summary`, `claimedAt`, `deliveredAt` — and the
review record carries `reviewer` and `note`. All of that survives as document
metadata, set through `update` flags. Only the three parallel status enums
collapse.

CLI consequence: the `accord` and `review` families disappear.

```
tandem accord deliver task-1 --summary S --evidence E
tandem review request task-1 --note N
```

becomes

```
tandem update task-1 --state delivered --summary S --evidence E
tandem update task-1 --state review --note N
```

Event names derive from the state transition rather than from which of two
command families was used.

#### Scope warning

S14 is a protocol change, not a CLI cleanup. Under `AGENTS.md` it requires
changing `protocol/` first, then `tandem/src/protocol/`, then the interfaces.
It touches the TUI Board, the web view, `.tandem` documents in every existing
workspace, and `pi-tandem`'s `tandem_accord`/`tandem_review` tools.

It should be decided *before* the clap cutover and implemented as its own Task,
because it changes the target command set a second time. Sequencing it after
the cutover means designing a clap hierarchy for 10 commands that are about to
be deleted.

#### Net effect with S14

| | Now | S1–S13 | + S14 |
| --- | --- | --- | --- |
| Invocable commands | 39 | 24 | **14** |
| Subcommand families | 6 | 3 | **1** |
| `--help` surfaces | 46 | 28 | **16** |
| Task status fields | 3 | 3 | **1** |
| Nominal status combinations | 90 | 90 | **7** |

Final command set under S1–S14:

```
tandem init | version | tui | web
tandem add    [--type task|decision|papercut] ...
tandem list   [--query <q>] [--location ...] [--state ...] [--limit n]
tandem show   <id>
tandem update <id> [--state <s>] [--assignee ...] [--evidence ...] ...
tandem complete <id> | cancel <id>
tandem rules  list|add|edit|delete [<category-id>]
```

#### S15 — rules as documents (not recommended yet)

The last family could go too: rules become `type: rule` documents, giving
`add --type rule`, `list --type rule`, `update rule-3`, `cancel rule-3`, and a
zero-family CLI of 10 commands. This moves rules out of `.tandem/tandem.md`,
which is the protocol config file, and rules genuinely need hard deletion
rather than archival. Recorded as an option; not proposed.

## 3. Clap design

### Dependency and features

```toml
clap = { version = "4.6", features = ["derive"] }
```

Current stable is 4.6.6, MSRV 1.85, license MIT OR Apache-2.0. Keep default
features (`std`, `color`, `help`, `usage`, `error-context`, `suggestions`) and
add `derive`. Do not enable `env`, `unicode`, `wrap_help`, `cargo`, unstable
features, completions, or manpages. Do not add a Tandem `rust-version`
declaration in this cutover.

### Typed command model

Use derive for one static command tree:

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, short = 'j', global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init(InitArgs),
    Add(AddArgs),
    Show(ShowArgs),
    List(ListArgs),
    Search(SearchArgs),
    Update(UpdateArgs),
    Accord(AccordArgs),
    Review(ReviewArgs),
    Complete(CompleteArgs),
    Cancel(CancelArgs),
    Rules(RulesArgs),
    Tui,
    Web(WebArgs),
}

#[derive(Subcommand)]
enum AddCommand { Task(AddTaskArgs), Decision(AddDecisionArgs) }
#[derive(Subcommand)]
enum AccordCommand { Claim(...), Deliver(...), Rework(...), Block(...), Resume(...), Release(...), Fail(...) }
#[derive(Subcommand)]
enum RulesCommand { List(...), Add(...), Edit(...), Delete(...) }
```

Clap owns grammar, occurrence, numeric parsing, `Scope`, and `ClearField`.
Protocol/app owns every semantic vocabulary and transition; do not duplicate
priority, effort, Decision status, Accord status, Rule category, hierarchy, or
reference validity as parser enums.

### Modules and conversion boundary

```text
cli/model.rs     derive structs/enums only
cli/parse.rs     global JSON extraction, try_parse_from, clap error mapping
cli/commands.rs  typed command to app options/invocation
cli/output.rs    human/JSON rendering
cli/landing.rs   no-args surface
cli/mod.rs       startup wiring
```

Delete `args.rs`. Do not create one file per command. Parser structs contain
interface values; conversion into app option types lives in `commands.rs`.
Protocol/filesystem behavior never enters `model.rs` or `parse.rs`.

### Startup and errors

Extract exact `-j`/`--json` tokens before clap so grammar failures can honor the
global JSON contract. Exact tokens are reserved; literal prose `--json` uses
`--body=--json`. Call `Cli::try_parse_from` and intercept
`DisplayHelp`/`DisplayVersion` for stdout/exit 0. Map real clap errors to usage
exit 2 and the selected human/JSON renderer. Preserve
`StartupRequest::{Exit,Tui,Web}` and defer workspace opening until after typed
dispatch.

Replace app-layer `CliError` coupling with `app::Error { kind, message,
details }`; CLI and TUI render it independently.

## 4. Difficult-case prototype results

A disposable `/tmp/tandem-clap-probe` used exact clap 4.6.6 with derive.
Nothing was committed to production.

| Case | Result |
| --- | --- |
| root/family/leaf `--help` | generated successfully through `DisplayHelp` |
| global `--json` before root command | parsed |
| global `--json` after nested leaf args | parsed |
| `-j` alias | parsed |
| positional title `"- Fix help"` | parsed with `allow_hyphen_values` |
| `--body "- bullet"` | parsed |
| repeated `--tag` | accumulated in order |
| repeated scalar `--priority` | `ArgumentConflict`, exit 2 |
| `--body ""` | reusable nonempty parser rejects it |
| unknown flag | contextual usage + help hint |
| `Scope`/`ClearField` | derive `ValueEnum` works for CLI-owned values |
| `--body --json` | clap consumes `--json` as prose when hyphens are allowed; bootstrap extraction therefore reserves exact global tokens, while literal text uses `--body=--json` |

The prototype confirmed that help/version are clap errors by design; Tandem must
print those two kinds to stdout rather than using `Error::exit`.

Measured locally:

- cold release build: ~5.1 seconds;
- stripped minimal plain Rust binary: 342,472 bytes;
- stripped minimal clap probe: 843,224 bytes;
- isolated clap parser cost: ~500,752 bytes;
- current stripped Tandem release binary: 4,626,256 bytes.

This is an upper-bound minimal-binary comparison, not a final Tandem size
prediction. Record actual before/after release size during implementation.

## 5. Cutover specification

### Target command surface

Global: `-h/--help`, `-V/--version`, `-j/--json`. There is no `version` command,
`upgrade`, command alias, global migration flag, or per-command JSON option.

```text
tandem init [--title <title>]

tandem add task <title>
  --acceptance <text>... [--body <markdown>] [--kind epic]
  [--priority <value>] [--effort <value>] [--tag <tag>]...
  [--due-date <date>] [--parent <task-id>] [--blocker <id>]...
  [--reference <id>]... [--related-file <path>]...
  [--constraint <text>]... [--validation <text>]...

tandem add decision <title>
  [--body <markdown>] [--decider <name>]... [--supersedes <id>]...
  [--reference <id>]... [--tag <tag>]...

tandem show <id>

tandem list [--scope active|archived|all] [--type task|decision]
  [--state <value>] [--priority <value>] [--effort <value>]
  [--tag <tag>]... [--assignee <name>] [--parent <id>]
  [--accord <status>] [--decision-status <status>]
  [--resolution <completed|canceled|failed>] [--limit <n>]

tandem search <query> [--scope active|archived|all]
  [--type task|decision] [--state <value>] [--tag <tag>]...
  [--parent <id>] [--limit <n>]

tandem update <id>
  [--title <text>] [--body <markdown>] [type-specific metadata/list flags]
  [--clear <field>]...

tandem accord claim <id> --assignee <name>
tandem accord deliver <id> --summary <text> --evidence <text>...
  [--file-changed <path>]...
tandem accord rework <id> --note <text>
tandem accord block <id> --note <text>
tandem accord resume <id>
tandem accord release <id> --note <text>
tandem accord fail <id> --note <text>

tandem review <id> --criterion <text> --note <text> [--reviewer <name>]
tandem complete <id> [--reviewer <name>]
tandem cancel <id> --note <text>

tandem rules list [always|never|prefer|context]
tandem rules add <category> <text> [--source <id>]
tandem rules edit <category-id> <text> [--source <id>] [--clear source]
tandem rules delete <category-id>

tandem tui
tandem web [--port <1..65535>] [--no-open]
```

`update` infers record type. Common fields are title, body, tags, and references.
Task-only fields include priority, effort, due date, parent, blockers, related
files, Accord acceptance/constraints/planned validation. Decision-only fields
include status, deciders, and supersedes. `update` never writes Task state,
assignee, or Accord status. Present list flags replace the complete list; absent
means unchanged; `--clear` removes it.

This is 23 invocable leaves: 10 flat leaves plus 13 leaves under three families
(`add` 2, `accord` 7, `rules` 4). Root, families, and leaves produce 27 generated
help surfaces.

### Protocol and persistence cutover

Normative version becomes 0.3.0. Layout:

```text
.tandem/tasks/
.tandem/decisions/
.tandem/rules/
.tandem/logs/
.tandem/events/<actor-id>.jsonl
.tandem/tandem.md
```

Papercuts become low-priority Tasks tagged `papercut`. Active Tasks require a
mandatory Accord with status `ready`, at least one acceptance criterion, and
optional constraints/planned validation. Task state remains
`todo|in-progress|validation`; Review status is deleted. Validation is only an
explicit human escalation. Full definitions and transitions are in the living
cutover ledger.

No production upgrade/migrate/converter, compatibility reader, dual protocol,
or workspace backup convention ships. Existing workspace handling is owned
project by project outside this cutover.

### Files/modules

- normative: update `protocol/README.md`, `protocol/plan/spec.md`, protocol todo;
- protocol Rust: rewrite Accord/review/workflow/document/event/rule semantics;
  remove Papercut and legacy-version paths;
- project: replace Board/Papercut/config-Rule storage with typed directories and
  per-file Rules; keep per-actor events only;
- app: implement typed reads, deterministic update replacement, Accord/review/
  archive transitions, `app::Error`;
- CLI: add clap/model/parse; delete `args.rs`; shrink dispatch; unify output;
- TUI: preserve current Board list/subviews and State/Epic arrangements, add
  Papercuts as fourth Board section, remove Add/direct Move, adapt Validation;
- web: compile and read the new app/storage model; broader redesign deferred;
- tests/docs: replace stale exact-help/current-protocol contracts.

### Implementation order and rollback

One comprehensive Task and isolated branch/worktree. Ordered commits:
normative protocol → Rust protocol → project → app/error → clap CLI → output and
tests → targeted TUI → docs/removal. Nothing merges until coherent. Rollback is
discard/rework of the branch, never a shipped second parser or protocol path.

### Task-ready acceptance

- protocol 0.3.0 normative and executable semantics agree;
- only the target command tree parses; every removed command/flag fails usage;
- all 27 help surfaces work without a workspace;
- manual parser and `args.rs` are gone;
- global JSON covers every command and error;
- Task/Accord/validation/archive flows match the ledger;
- new directory layout and per-file Rules are the only runtime storage path;
- PC9 replacement/clear semantics pass process tests;
- TUI preserves existing architecture with the targeted changes;
- release build/tests/lint pass and binary-size delta is recorded;
- no adapter, migration, backup, fallback, completion, or manpage work is
  smuggled into core scope.

## 6. Test and validation plan

### Parser and generated help

- `Cli::command().debug_assert()`;
- table-driven `try_parse_from` tests for every leaf, required positional/flag,
  scalar repetition, repeated lists, clear fields, typed numbers, leading
  hyphen prose, global JSON placement, and removed grammar;
- recursively traverse root/family/leaf command tree and assert help exit 0,
  nonempty Usage/description, no workspace access, and exactly 27 surfaces;
- help/version text always stdout and uncolored when non-TTY/`NO_COLOR`;
- JSON bootstrap tests, including parse failure with `--json` before/after the
  failing command and literal `--body=--json`.

### Process contract

Rewrite `tests/cli_behavior.rs` around semantic assertions rather than freezing
all generated prose byte-for-byte. Assert stdout/stderr separation, exit 0/1/2,
JSON success/error envelopes, stable error codes, unknown suggestions, missing
workspace behavior after successful parse, and landing output.

### Protocol/app/project

- new-workspace layout and 0.3.0 config;
- mandatory active Accord, ready/claim/deliver/rework/block/resume/release/
  fail/complete transitions and evidence requirements;
- exceptional validation entry/accept/rework only;
- Epic/Task/Subtask identity and task-only parent hierarchy;
- Papercut-tagged Tasks and Board exclusion rules;
- typed Task/Decision/Rule lookup across active/archive stores;
- deterministic list replacement/clear and no lifecycle bypass through update;
- minimal resolution logs and optional Accord on historical Logs;
- per-file Rule allocation/mutation;
- per-actor structured events for every durable mutation.

### TUI/web/release

- TUI rendering tests for four Board sections and unchanged State/Epic layout;
- TUI interaction tests proving Add/Move removal and adapted Validation paths;
- render a release TUI pane for layout/colors if implementation changes visible
  structure; escalate temporal behavior only when unverified;
- web smoke tests against new read models; no visual redesign acceptance;
- `cargo test`, release build, lint/format, real-command smoke suite;
- record clean/cached build timing and stripped release-size delta.

No human review is required for criteria that automated/process/rendered evidence
can settle. Request it only for any remaining product/visual judgment.

## 7. PC1 and PC9 dispositions

### PC1 — `<command> --help`

Resolve in the core cutover. Every root/family/leaf help surface (27 in the
target tree) must emit generated help on stdout with exit 0 and no workspace.
Delete static per-command help and the false landing promise becomes true by
construction.

### PC9 — list field replacement

Resolve in the same cutover because update grammar is being replaced. For every
list-valued field: absent means unchanged; present repeated values replace the
complete list; `--clear <field>` removes it. Scalar supplied values must be
nonempty. There are no additive defaults, add/remove flag pairs, or empty-string
clearing aliases.

## 8. Decision draft

**Proposed title:** Adopt protocol 0.3.0 and a clap-derived minimal CLI

**Status:** Proposed — do not mark accepted until owner review.

**Deciders:** Algorant

**References:** `task-246`, `papercut-1`, `papercut-9`, `decision-3`,
`decision-8`, `plan/cli-protocol-cutover.md`

### Context

Tandem's handwritten parser spans 1,147 lines in `args.rs`, repeats flag and
error logic across 39 invocable commands, and provides working help on only two
of 46 current help-relevant surfaces. Static help and landing output already
drift from dispatch. Scalar options silently overwrite, prose hyphen handling
is arbitrary, list metadata cannot be replaced/cleared, and Pi can emit a core
flag the CLI rejects.

Sustained use also showed that migrating the current grammar unchanged would
preserve product duplication: Task, Log, Decision, and Papercut read families;
separate Move/update state paths; Review status duplicating Validation;
Papercuts implemented as a parallel record system; Rules embedded in shared
config; and completion metadata duplicating Accord delivery.

### Decision

Adopt protocol 0.3.0 and the complete product/CLI contract in
`plan/cli-protocol-cutover.md`.

Use clap 4.6.x with default features plus derive for one static command model.
Remove the handwritten parser in one comprehensive cutover and do not ship dual
parsers. Use 13 root subcommands and 23 invocable leaves, with typed `add`,
`accord`, and `rules` families; unified lookup/update; separate structured list
and full-text search; global JSON; generated help; mandatory active Task Accord;
tagged-Task Papercuts; per-file Rules; typed storage directories; minimal Logs;
and structured per-actor events.

Preserve Decision as first-class ADR content, fixed Epic → Task → Subtask roles,
parent-derived Subtask IDs, Task state plus separate mandatory Accord status,
and exceptional human Validation. Preserve the current TUI list/subview and
State/Epic arrangement architecture; make only the targeted protocol
adaptations.

Use one isolated implementation Task/branch. Normative protocol changes precede
Rust implementation inside the branch. Merge only the coherent candidate. Old
workspace handling is outside the protocol/cutover; the new binary supports
0.3.0 only and fails clearly on older versions.

### Compatibility policy

Correct new behavior replaces current byte-exact output, command, flag, field,
storage, and protocol compatibility. No upgrade/migrate command, implicit
conversion, compatibility reader, backup convention, parser fallback, alias
shim, or old protocol path ships.

Help/version remain generated text. Human output uses stdout for results and
stderr for warnings/errors. Global JSON uses stdout-only success/error
envelopes. Exit statuses remain 0 success, 2 usage, 1 operational failure.

### Consequences

- PC1 and PC9 resolve in the core cutover.
- Protocol/app/project/CLI/TUI tests and docs require broad replacement.
- `app::Error` removes shared app dependence on CLI process errors.
- The isolated parser cost is approximately 501 KB stripped and raises the
  effective toolchain floor to clap's Rust 1.85, though Tandem does not declare
  an MSRV.
- Shell completions, manpages, web redesign, TUI contextual lifecycle picker,
  workspace migration, and Pi adapter implementation remain separate.
- The Pi integration requires an owning-workspace overhaul after the core CLI
  exists; its final tool inventory is intentionally not predetermined here.

### Supersession and amendment

This Decision supersedes the v0 guidance prohibiting clap and the current v0
CLI/protocol command/storage/lifecycle decisions that conflict with the
cutover ledger. It does not supersede decision-8's protocol/project/app/peer
interface ownership direction; `app::Error` and the typed CLI strengthen it. It
amends decision-3's Rust stack by adding clap as the canonical CLI parser.

### Alternatives

- Preserve handwritten parsing and patch help: rejected because grammar,
  generated documentation, error uniformity, and repetition remain manually
  synchronized.
- Port the existing 39-command grammar directly to clap: rejected because it
  preserves product duplication and conditional one-offs.
- Split the core cutover into separately merged protocol/CLI/TUI Tasks:
  rejected because intermediate compatibility or broken main states would be
  required across heavily overlapping files.
- Builder-only or derive+parallel-builder models: rejected because the command
  tree is static and one derive source is sufficient.
- One universal record/tool abstraction: rejected where type-specific Task,
  Decision, Rule, and agent intent semantics improve clarity.

## 9. Cross-workspace impact and handoff tasks

### Core Tandem repository

**Core implementation Task (ready to create after Decision approval)**

**Title:** Implement protocol 0.3.0 and the comprehensive clap CLI cutover

**Priority/effort:** critical / large

**Blockers:** accepted Decision drafted in section 8; completion of `task-246`

**References:** `task-246`, `papercut-1`, `papercut-9`, accepted Decision ID,
`decision-3`, `decision-8`

**Related files:** `protocol/`, `tandem/Cargo.toml`, `tandem/src/protocol/`,
`tandem/src/project/`, `tandem/src/app/`, `tandem/src/cli/`,
`tandem/src/main.rs`, targeted `tandem/src/tui/`, `tandem/src/web/`,
`tandem/tests/cli_behavior.rs`, `tandem/README.md`, planning/spec docs.

**Body/acceptance:** use section 5 Task-ready acceptance and section 6 test plan
verbatim. Work in one isolated branch/worktree with the section 5 commit order.
Do not modify adapters, migrate workspaces, ship fallbacks, or partially merge.

### Project-local `extensions/pi-tandem/`

Affected because every argument builder and guidance surface assumes current
CLI/protocol shapes. Rule never-2 prohibits changing it inside core work. Create
an explicit adapter Task in this Tandem workspace after core Decision approval,
blocked by the core cutover, if this project-local adapter remains a maintained
distribution surface. Its requirements should match the owning Pi overhaul
below rather than copying protocol behavior.

### Canonical Pi configuration/extensions workspace

Owning Tandem workspace: `~/.pi`. Implementation source:
`~/.dotfiles/pi/.pi/agent/extensions/pi-tandem/` and related skill/agency/alias/
manifest files.

**Handoff title:** Overhaul pi-tandem for Tandem protocol 0.3.0 and the
comprehensive CLI cutover

**Blocker:** completed and installed core cutover

**Requirements:** use the complete ready-to-create handoff in
`plan/cli-protocol-cutover.md` section 10. Audit the integration after the CLI
exists; retain/refactor/combine/create/remove tools by agent utility, not CLI
mirroring. Preserve convenient Papercut-tagged Task capture. Use argument arrays
and global JSON only; never parse/mutate files or own protocol behavior.

Do not create or mutate the external Task/workspace during research. Owner
reviews the map first.

### TUI and web

Required TUI adaptation is part of the core Task because the protocol/storage
change otherwise breaks the peer interface. A contextual lifecycle picker is a
separate proposed research Task, not a blocker. Web must compile/read the new
app model in core; broader visual/information redesign is deferred until the
TUI settles.

### Docs, scripts, release automation, tests

Repository help/README/spec/tests construct or assert old argv and output and
must change in core. No independent release script consumer was found that
requires a separate workspace Task. Release notes must call out the breaking
0.3.0 workspace/CLI contract and absence of migration support.

## 10. Risks, rejected alternatives, open questions

### Risks

- **R1 — cutover breadth.** Protocol/storage/app/CLI/TUI overlap makes the Task
  large. Mitigation: one isolated branch with ordered commits and full review,
  not partial merges.
- **R2 — semantic drift between normative and Rust protocol.** Mitigation:
  normative commit first, executable protocol tests before interface work.
- **R3 — JSON parse-error bootstrap.** Reserved `-j`/`--json` extraction is a
  narrow preparse rule. Test every placement and literal `--flag=--json` case.
- **R4 — TUI accidental redesign.** The research briefly inferred kanban
  columns. Final contract explicitly preserves current subview/list and
  State/Epic arrangement architecture.
- **R5 — adapter break window.** Core and Pi overhaul are separate. Coordinate
  installation/handoff at the workspace-owner level; do not add core shims.
- **R6 — size/build cost.** Isolated clap cost is ~501 KB and ~5.1 second cold
  probe build. Measure actual release delta before integration.
- **R7 — mandatory Accord friction.** Every active Task needs acceptance
  criteria. TUI creation is removed rather than shipping an incomplete form;
  CLI and adapters must make the requirement clear.

### Rejected alternatives

Rejected alternatives are recorded in the cutover ledger per sequence. The
major rejected paths are: direct port of current grammar; dual parser; protocol
fallbacks; global task IDs for Subtasks; a separate Papercut protocol; generic
parent documents; Review status; routine Validation; accepted-but-active Tasks;
optional Accord; duplicate assignees; completion evidence duplication; shared
events JSONL; Rules in shared config; kanban TUI redesign; speculative Pi tool
inventory; shell/man generation; and separately merged core cutover stages.

### Open questions

No unresolved owner/product questions remain from the interactive sequence.
Implementation may discover technical defects, but it must return for a new
Decision rather than silently changing the selected contract.
