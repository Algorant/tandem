---
id: task-16
type: task
title: "CLI: init emits no JSON envelope on success"
state: "in-progress"
priority: "low"
tags: ["protocol", "papercut"]
accord:
  status: "delivered"
  acceptance: ["Successful tandem init --title X --json emits exactly one stdout JSON success envelope with ok, data, and warnings; stderr is empty on ordinary success.", "The non-JSON init path retains its current behavior; failure on an existing workspace retains the standard JSON error envelope and exit code.", "Regression tests exercise fresh success and repeated-init failure without altering real project records."]
  claimedAt: "2026-09-11T14:21:52Z"
  deliveredAt: "2026-09-11T14:27:18Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["Only CLI init output and dedicated init regression tests; no adapter, protocol semantics, or unrelated command changes.", "Use the current native InitOutcome and established envelope conventions; no compatibility fallback for empty success.", "Submit a plan before editing despite small effort; wait for orchestrator approval."]
  summary: "Delivered the fix: `tandem init --title X --json` now prints exactly one stdout success envelope with ok, data, and warnings. The Command::Init arm in tandem/src/cli/commands.rs now keeps the InitOutcome and, when json is set, prints {\\\"ok\\\":true,\\\"data\\\":{\\\"title\\\":<resolved title>,\\\"root\\\":<absolute workspace root>},\\\"warnings\\\":[]}; the non-JSON path stays silent exactly as before. No protocol, app, InitOutcome, adapter, or unrelated command changes.\n\nAdded tandem/tests/init_behavior.rs with four temp-dir regression tests (never touching real records): fresh JSON success asserts a single parsed envelope line, exit 0, empty stderr, ok==true, warnings==[], data.title, canonicalized data.root, and created config; omitted-title JSON asserts the resolved default title (temp dir name); non-JSON success asserts quiet exit 0; repeated-init JSON asserts exit 1, the standard {ok:false,error:{code:\\\"io\\\"}} envelope on stdout, empty stderr, and byte-identical existing tandem.md.\n\nCommitted cleanly as 4cc773b. Both declared validations pass in order on the final tree: just dev-check exit 0 (268 unit + 31 integration tests including the 4 new, plus both native sandbox PASS checks) and cargo fmt --check exit 0. Remaining gap: none within the exclusive scope; parent owns independent validation and integration."
  evidence: ["Successful tandem init --title X --json emits exactly one stdout JSON success envelope with ok, data, and warnings; stderr is empty on ordinary success.: Release binary in a fresh temp dir produced exactly: {\"data\":{\"root\":\"/tmp/t16-final\",\"title\":\"probe\"},\"ok\":true,\"warnings\":[]} on one stdout line with exit 0 and 0 stderr bytes. Test init_json_fresh_success_emits_single_success_envelope asserts exit 0, empty stderr, one stdout line, a single parsed JSON value, ok==true, warnings==[], data.title==\"probe\", canonicalized data.root, and created .tandem/tandem.md.", "Resolved default title when --title is omitted is correct.: test init_json_without_title_reports_resolved_default_title passes: `init --json` reports data.title equal to the temp directory's file name (the same source as default_title).", "The non-JSON init path retains its current behavior; failure on an existing workspace retains the standard JSON error envelope and exit code.: test init_without_json_retains_quiet_success passes (exit 0, empty stdout and stderr). Release-binary probe of repeated `init --title probe --json` returned exit 1 with stdout {\"error\":{\"code\":\"io\",\"details\":{},\"message\":\"Tandem workspace already exists at /tmp/t16-final/.tandem.\"},\"ok\":false} and 0 stderr bytes, matching the pre-existing behavior.", "Regression tests exercise fresh success and repeated-init failure without altering real project records.: All four tests in tandem/tests/init_behavior.rs run in unique std::env::temp_dir() directories, never the checkout. repeated_init_json_retains_error_envelope_and_existing_config additionally reads .tandem/tandem.md before the failed repeat and asserts the bytes are unchanged.", "Only CLI init output and dedicated init regression tests change.: git status --porcelain on the committed tree is empty; commit 4cc773b touches only tandem/src/cli/commands.rs (7 insertions, 1 deletion in the Command::Init arm) and the new tandem/tests/init_behavior.rs. No InitOutcome, protocol, app, adapter, or other command changes."]
  filesChanged: ["tandem/src/cli/commands.rs", "tandem/tests/init_behavior.rs"]
  updatedAt: "2026-09-11T14:27:18Z"
createdAt: "2026-09-05T22:07:28Z"
updatedAt: "2026-09-11T14:27:18Z"
effort: "small"
relatedFiles: ["tandem/src/cli/commands.rs", "tandem/tests/init_behavior.rs"]
assignee: "worker-task-16-f4a47543"
---

## Description

## Description

Preserved from published tandem-v0.12.3 task-13 during reconciliation with local delegation proposals task-13–15. Original creation: 2026-09-04T01:36:29Z. Source commit: 735c4cf738a5f4b78c2c28f043ae64cd5461f90a.

Observed while writing the Pi adapter parser against 0.12.2. `tandem init --title probe --json` creates a workspace, prints nothing, and exits 0. Repeating it produces a JSON error envelope for the existing workspace.

D53 says JSON is global with one shared envelope. An adapter should not need to special-case empty stdout plus exit 0 as success for this command.
