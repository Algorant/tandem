# Lightweight Tandem web server and UI research

- Status: research; approach requires review before implementation
- Task: `task-121`
- Date: 2026-07-26
- Scope: browser access to one local Tandem project, then staged parity and safe interaction

## Executive recommendation

Add a `tandem web` mode to the existing Rust binary, backed by the same protocol, project-file, and application operations used by the CLI and TUI. Serve a small bundled same-origin frontend and a versioned JSON API from an embedded Axum server. The first release should be read-only, bind only to `127.0.0.1` on an ephemeral or explicitly selected port, open no remote interface by default, and display Board (including Validation), Logs, Rules, Decisions, document details, relationships, accord/review state, and project metadata.

Start updates with a cheap revision check or modest polling. Add server-sent events (SSE) only when automatic updates prove valuable: SSE matches a server-to-browser invalidation stream, reconnects automatically, and avoids WebSocket protocol and client-state complexity. Every notification should mean “the project may have changed; refetch a canonical snapshot,” not carry authoritative state patches.

Do **not** make a database, synchronization provider, separate Node service, browser-side Markdown parser, or full SPA framework prerequisites for the local MVP. Do **not** expose mutations until the Rust refactor establishes shared `protocol` / `project` / `app` boundaries. The current implementation has reusable behavior, hierarchy validation, and cooperative locking, but much of it remains concentrated in `tandem/src/main.rs`, while some TUI projections and mutations remain in `tandem/src/tui.rs`. A web adapter added directly to those private seams now would encourage a third implementation of business rules.

## Current-system findings

- `tandem` is one Rust binary crate with only `crossterm`, `ratatui`, and `yaml-rust2` dependencies.
- CLI and TUI discover one project through `.tandem/tandem.md` and read Board plus Logs from project-local Markdown files.
- `HierarchyIndex` already creates a canonical Board-plus-Logs graph, derives Epic/Task/Subtask roles from resolved documents, validates role-specific IDs, and exposes relationships.
- Reads acquire `HierarchyLock` on the project config file before constructing a coherent snapshot. Mutations also acquire that lock, minimally patch raw source, and append audit events.
- CLI read commands already expose hand-built JSON envelopes for list/show/search/log/rules/decisions. These are useful compatibility examples, but invoking the binary once per HTTP request would add process overhead, lose a coherent in-process boundary, and constrain richer web projections.
- The TUI covers Board/Validation, Logs, Rules, Decisions, details, hierarchy, accord/review data, themes, mouse and some mutations. Its rendering and transient state are interface-specific and should not become the web data model.
- The reviewed refactor specification (`plan/refactor_spec.md`) already proposes the right long-term seams: canonical `protocol`, concrete `project::TandemProject`, shared `app` use cases, and peer CLI/TUI interfaces. The web server should become another peer adapter over those layers.

## Options analysis

| Option | Advantages | Costs and failure modes | Assessment |
| --- | --- | --- | --- |
| Embedded server and bundled frontend in `tandem` | One install and process; reuses in-process Rust semantics; straightforward project discovery; localhost startup can mirror `tandem tui`; static assets can be compiled into release artifacts | Adds async/server dependencies and browser security responsibilities to the binary; requires extracting reusable query/app seams; long-running process must observe external edits | **Recommended** for local MVP |
| Separate Rust service/binary | Clear runtime boundary; independent release and scaling; could use Tandem as a library | Repository currently intentionally has one package/binary and no supported Rust library API; risks duplicated parsing or premature public API; two artifacts and lifecycle commands | Reconsider only for hosted/multi-project deployment |
| Separate JS/TS backend and frontend | Fast UI ecosystem; rich component choices | Must shell out to CLI or duplicate protocol logic; Node/toolchain/runtime packaging; concurrency and error semantics diverge; larger supply-chain and deployment surface | Reject as canonical backend |
| Static frontend calling CLI through a thin sidecar | Can prototype against existing `--json`; preserves CLI as authority | N+1 processes, awkward streaming, lossy API/error mapping, mutations race across calls, and CLI output becomes accidental service API | Acceptable throwaway prototype only |
| Server-rendered HTML with small progressive enhancement | Lowest frontend complexity; excellent initial accessibility; easy same-origin security and no build chain | More full-page/fragment refreshes; complex Board interactions become harder later | Strong MVP variant |
| Bundled SPA | Responsive navigation and richer later parity | Framework/build/tooling/cache complexity; encourages client duplication of hierarchy/filter semantics | Defer until interaction needs justify it |

The recommended frontend is deliberately flexible: semantic server-rendered shell plus small vanilla TypeScript/JavaScript modules, or a very small compiled frontend, provided all durable meaning comes from API view models. Choose a framework only after a prototype measures bundle, build, accessibility, and maintenance costs.

## Proposed component boundaries

```text
browser
  ├─ semantic UI and transient view state
  ├─ GET JSON snapshots/details
  ├─ EventSource invalidations (optional)
  └─ later: explicit command requests
          |
          v
web adapter (HTTP, authn/authz, CSRF, DTOs, assets)
          |
          v
app queries and commands (complete use cases, typed outcomes)
       /      \
protocol      project::TandemProject
(meaning)     (discovery, coherent reads, locks, atomic writes, events)
                  |
                  v
            project .tandem files
```

### Ownership rules

- **Protocol:** document semantics, hierarchy, IDs, workflow, accord, review, event vocabulary, diagnostics.
- **Project:** exactly one canonicalized project root per server instance; safe reads, snapshots/revisions, file watching, lock/conflict handling, atomic writes, audit append.
- **App:** UI-neutral queries and mutations returning typed outcomes and warnings. CLI, TUI, and web call the same functions.
- **Web adapter:** routes, DTO serialization, HTTP status/error mapping, authentication, authorization, CSRF, caching headers, rate/body limits, and static assets. It never parses Tandem Markdown or infers hierarchy.
- **Frontend:** layout, local selection/filter state, accessible interaction, and rendering only. It does not decide whether a lifecycle transition or relationship is valid.

Avoid returning raw internal Rust structs as an accidental permanent API. Define explicit API DTOs with a small compatibility contract and retain unknown protocol fields only where a deliberate raw-metadata view needs them.

## Recommended local MVP

### Command and startup

Proposed shape (subject to normal CLI review):

```text
tandem web [--port <port>] [--no-open]
```

- Auto-discover the nearest project exactly as other commands do.
- Bind `127.0.0.1`; prefer an available ephemeral port when no port is supplied.
- Print the full URL and project root, optionally open the browser, and terminate cleanly on Ctrl-C.
- Serve one project per process. Never accept a request parameter that selects an arbitrary filesystem path.
- Compile static assets into the binary or release artifact; do not serve files from the working tree or expose project files directly.
- Keep the initial UI read-only and visibly label the project and read-only status.

### MVP screens

1. **Board:** configured states and counts; hierarchical Epic → Task → Subtask projection; compact action-relevant badges; Validation as a state/filter; filters for state, priority, tag, assignee, accord, and review.
2. **Document detail:** canonical identity/role/relationship, parent and direct children, Markdown body rendered with HTML escaping/sanitization, metadata, blockers/references/related files, accord/review/validation, and event timeline where available.
3. **Logs:** completed/canceled outcome, summary, timestamp, files, validation/reviewer, body, and search.
4. **Rules:** grouped `always` / `never` / `prefer` / `context`, including stable IDs and sources.
5. **Decisions:** list/detail with ADR status and supersession metadata.
6. **Project:** title, protocol version, configured states, selected theme/display settings, canonical root display, health/warnings, and last observed revision. Never expose environment variables or unrelated files.

Validation should not become a second copy of Board data: expose a canonical attention query or deterministic app projection consumed by both interfaces.

## API sketch

Use `/api/v1` even for a local-only first API. All responses should include a project-scoped opaque `revision` computed by the project layer from a coherent snapshot (for example, a monotonic in-process generation plus content fingerprint; not only coarse mtimes).

### Read endpoints

```text
GET /api/v1/project
GET /api/v1/board?state=&priority=&tag=&assignee=&accord=&review=
GET /api/v1/attention
GET /api/v1/documents/{id}
GET /api/v1/logs?query=&limit=&cursor=
GET /api/v1/logs/{id}
GET /api/v1/rules
GET /api/v1/decisions
GET /api/v1/decisions/{id}
GET /api/v1/events?since=<revision>       # optional SSE stream
```

Response conventions:

```json
{
  "ok": true,
  "data": {},
  "revision": "opaque-project-revision",
  "warnings": []
}
```

- Paginate potentially unbounded Logs and event timelines.
- Return stable machine-readable error codes plus safe messages; do not return absolute paths, stack traces, or source snippets by default.
- Use `ETag` / `If-None-Match` on snapshot reads where practical.
- A missing ID is `404`; malformed input `400`; revision conflict `409`; invalid transition/structure `422`; unauthenticated `401`; unauthorized `403`; unexpected failures `500` with a correlation ID.

### Later command endpoints

Prefer intent-shaped commands over generic document replacement:

```text
POST  /api/v1/tasks
PATCH /api/v1/tasks/{id}                  # supported app fields only
POST  /api/v1/tasks/{id}/move
POST  /api/v1/tasks/{id}/complete
POST  /api/v1/tasks/{id}/cancel
POST  /api/v1/tasks/{id}/accord/{action}
POST  /api/v1/rules
PATCH /api/v1/rules/{category}/{ruleId}
DELETE /api/v1/rules/{category}/{ruleId}
POST  /api/v1/decisions
POST  /api/v1/feedback                    # later, constrained interaction record
POST  /api/v1/reviews/{taskId}/{action}   # later accept/rework/etc.
```

Every mutation includes `If-Match: <revision>` or an equivalent expected-revision field, an idempotency key for retry-prone commands, and actor/session context. The app layer revalidates under the project lock; the server returns the new revision and typed outcome. Never accept arbitrary frontmatter patches, shell commands, event names, agent process identifiers, or filesystem destinations.

## Updates and consistency

### Stage 1: revision polling

Poll `GET /api/v1/project` or a small `HEAD /api/v1/revision` every 2–5 seconds while the tab is visible, then refetch changed views. This is easy to debug, survives watcher limitations, and is adequate for a local task board.

### Stage 2: file watcher plus SSE invalidation

- Watch only the selected `.tandem/` config, Board, Logs, and event directories.
- Debounce/coalesce editor save bursts, then rebuild and validate a complete snapshot under the same read discipline as CLI/TUI.
- Publish `project.changed` with a new revision through a bounded broadcast channel.
- SSE subscribers refetch canonical data. If a receiver lags, reconnects, or presents an unknown `Last-Event-ID`, send `resync-required`; never assume all deltas arrived.
- Retain periodic revision reconciliation because native filesystem notifications can be absent or lossy on network filesystems, Docker/WSL combinations, editor replace-on-save behavior, and watcher-limit failures.

SSE is preferable to WebSockets here because updates are server-to-browser and browser commands remain ordinary authenticated HTTP requests. MDN documents automatic EventSource reconnection and event IDs; it also notes a low per-origin connection limit under HTTP/1, so use one stream per tab and tolerate polling fallback. WebSockets become justified only if a later agent interaction model truly needs sustained bidirectional messaging rather than auditable commands/replies.

### Multi-process and concurrent writes

The current config-inode lock coordinates cooperating local Tandem writers for a snapshot/mutation, but it is not a distributed transaction system. Web mutation must:

1. canonicalize and pin the project root at startup;
2. acquire the shared project lock;
3. reload and validate current state;
4. compare the client's expected revision;
5. execute one app command;
6. write atomically and append its audit event;
7. release the lock and publish invalidation.

A conflict returns `409` with enough safe detail to refresh and retry. Never silently apply last-writer-wins to bodies, parent changes, lifecycle transitions, or review decisions. Multi-user remote writes require stronger identity, authorization, and durable conflict semantics and should not be inferred from a local process lock.

## Security model

### Local read-only baseline

- Bind loopback only. Treat `localhost` as reduced exposure, **not authentication**: malicious web pages can target local services, and DNS rebinding/Host-header attacks are relevant.
- Reject unexpected `Host` values; allow only the actual loopback host/port set. Set no permissive CORS headers.
- Serve UI and API from one origin. Use a restrictive Content Security Policy (`default-src 'self'`; no inline script where practical), `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and frame denial (`frame-ancestors 'none'`).
- Escape all project-controlled text. Sanitize rendered Markdown with a strict allowlist; do not permit raw HTML, scriptable URLs, inline event handlers, or automatic remote image loads. Display related paths as text; opening files needs a separately reviewed local action.
- Set API responses and HTML containing project data to `Cache-Control: no-store`; fingerprint immutable static assets separately.
- Cap request URL/body sizes, connection counts, search complexity, and event clients. Avoid logging bodies, tokens, secrets, or full project content.
- Generate a random startup capability token if browser-targeting tests show loopback read exposure needs it. Keep tokens in URL fragments or a bootstrap exchange rather than query strings that leak to logs/history; settle exact UX in a security spike.

### Mutation-enabled local mode

- Require an explicit startup flag or interactive confirmation to enable writes; show a persistent UI indicator.
- Use an unpredictable session credential in an `HttpOnly`, `Secure` (when HTTPS), `SameSite=Strict` cookie, short expiry, and no parent-domain scope.
- Protect every state-changing endpoint with a synchronizer CSRF token in a custom header, verify `Origin` and Fetch Metadata as defense in depth, reject cross-origin requests, and never mutate on `GET`. OWASP recommends tokens for state-changing requests and notes that XSS defeats CSRF defenses, making output sanitization and CSP essential.
- Require recent user confirmation for destructive/authority-heavy actions such as cancel, complete, accord accept/fail, and review acceptance. Return a preview of warnings before confirmation rather than hiding protocol warnings.
- Separate roles at minimum into `viewer`, `editor`, and `reviewer/admin`; local single-user mode may map its one session to all roles only after explicit write enablement.

### Remote mode

Remote binding should be a separate, explicitly enabled product mode—not `--host 0.0.0.0` casually added to local MVP.

- Require TLS through a documented reverse proxy or native TLS, real authentication, per-user sessions, authorization checks on every command, rate limiting, secure secret storage, and proxy/header trust configuration.
- Default deny; never infer identity from display-name headers unless a specifically trusted proxy is configured.
- Store only password hashes or external identity references, never raw passwords/tokens in project Markdown or tracked config.
- Isolate each server process/tenant to an allowlisted canonical project root and OS identity. Prevent symlink/path traversal and never expose arbitrary `relatedFiles` contents.
- Define backup, retention, audit, session revocation, and incident behavior before advertising remote writes.

## Agent identity and safe feedback routing

Browser identity, Tandem actor identity, task assignee, accord assignee, and a live Herdr/Pi process are different concepts. Do not collapse them into a user-supplied string.

A safe future channel should use an explicit, durable command:

```text
feedback request
  id, projectId, taskId, threadId
  authenticated human actor
  target kind + stable server-resolved target ID
  action (comment | accept | request-rework | reject)
  bounded Markdown text
  expected task revision
  createdAt, idempotency key
```

- Resolve targets server-side from a registry of currently valid sessions/tasks; never let the browser address OS processes, sockets, worktrees, or arbitrary callback URLs.
- Persist the human intent/audit record **before** attempting delivery. Record delivery status separately so agent availability cannot erase user intent.
- Acceptance/rework actions call canonical Tandem app operations and append normal actor-owned audit events. Freeform comments are not lifecycle transitions.
- An agent adapter may subscribe/poll authorized feedback for Tasks it owns, acknowledge receipt, and write a correlated response. It may not self-accept, complete, or broaden its authority because a message exists.
- Render an explicit confirmation showing task, action, target, and current revision for authority-bearing review actions. Stale tasks fail with conflict and require rereview.
- Task creation from the web must use canonical allocation and parent validation. Creating work and routing it to an agent are separate operations.

This design remains useful without a live agent: feedback is durable, inspectable, retryable, and auditable.

## Database and sync-provider relationship

The local MVP should continue to treat `.tandem/` Markdown and actor event logs as source of truth. A future database or sync provider can improve remote deployment without changing the web API's app-command boundary:

- **Read model/cache:** index snapshots for fast filtering/search and rebuild from project files/events. It is disposable, not canonical.
- **Hosted persistence:** implement a concrete project/storage capability behind the already-proven app boundary only when a real second backend exists. Do not invent a generic repository trait during MVP.
- **Synchronization:** define stable event IDs, actor sequence rules, idempotency keys, causal/base revisions, and deterministic conflict policy first. File copying or last-write-wins is unsafe for concurrent lifecycle decisions.
- **Remote replicas:** preserve Markdown export/import and unknown-field semantics; keep audit records append-only and tamper-evident enough for the deployment threat model.
- **Offline clients:** defer until command conflict/replay semantics are specified. A browser cache is not a sync protocol.

Task `task-120` should be treated as related context for storage/sync direction, not a blocker or prerequisite. The web DTO/app boundary should avoid assuming files are directly browser-visible, while the first implementation remains concrete and file-native.

## Accessibility, responsive layout, and theming

- Use semantic landmarks, headings, tables/lists, buttons, labels, and status regions before custom widgets.
- Provide complete keyboard operation, visible focus, skip links, logical tab order, and no drag-only action. A Board can progressively collapse to state tabs plus one list on narrow screens, matching the TUI's small-screen philosophy.
- Use native dialogs/forms where possible; announce validation/update status through appropriately scoped live regions without making every refresh noisy.
- Meet WCAG 2.2 AA contrast and target-size expectations; do not encode state only by color. Respect reduced motion, zoom to 200%, forced-colors/high-contrast modes, and touch targets.
- Map existing semantic theme tokens (background, foreground, accent, success, warning, error, muted, selection, accord/review tones) to CSS custom properties. Do not expose terminal color syntax directly as the web contract.
- Sanitize Markdown links, mark external destinations, and do not fetch external content automatically.
- Test desktop, narrow/mobile, keyboard-only, and at least one screen reader at each feature-parity stage.

## Staged roadmap

### 0. Boundary preparation

- Finish/reuse the reviewed Rust `protocol` / `project` / `app` extraction.
- Define typed query outcomes and revision semantics shared by interfaces.
- Add tests proving web queries and CLI/TUI projections use canonical hierarchy and validation.
- Make an explicit dependency decision before adding Tokio/Axum/serialization/static-asset tooling.

### 1. Read-only local MVP

- `tandem web`, loopback-only, one project, bundled semantic UI.
- Project/Board/Validation/details/Logs/Rules/Decisions read endpoints.
- Manual refresh plus revision polling, responsive layout, base themes, keyboard and screen-reader checks.
- Security headers, escaping/sanitization, Host validation, body/connection limits, and no CORS.

### 2. Automatic updates and read parity

- Debounced project watcher, snapshot revision, SSE invalidations, polling fallback/resync.
- Search/filter parity, event timeline, richer hierarchy and warning/health views.
- Performance tests on large Boards/Logs and watcher failure tests.

### 3. Constrained mutations

- Explicit write-enabled mode, authentication/session and CSRF defenses.
- Add/update/move, then Rules/Decision operations through shared app commands.
- Optimistic concurrency, idempotency, confirmation, audit actor, and conflict UI.
- No remote binding yet.

### 4. Validation and TUI action parity

- Accord actions, accept/rework/block/fail, complete/cancel, review metadata, warning previews, role checks, and destructive confirmation.
- Parity matrix maintained against the current TUI and CLI; human product/security validation for every authority-bearing flow.

### 5. Agent interaction

- Durable feedback/thread records, server-resolved routing, delivery acknowledgements, correlated agent responses, and audit views.
- Separate comment from lifecycle authority; enforce Task ownership and role policy.
- Threat-model prompt injection, impersonation, stale decisions, replay, and unavailable targets.

### 6. Optional remote/multi-user deployment

- TLS/proxy profile, identity provider or local accounts, RBAC, tenant/project isolation, secret management, backups, retention, rate limits, observability, and operational docs.
- Introduce database/read model/sync only for demonstrated scaling, persistence, or replication needs.

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Third copy of protocol/business rules | Block web mutations until shared app boundaries exist; contract tests across interfaces |
| Stale or torn reads during editor/CLI writes | Canonical project snapshot under existing lock discipline; revisions; atomic writes |
| Watcher misses/coalesces events | Notifications only invalidate; periodic revision reconciliation and full refetch |
| Concurrent writers overwrite intent | Expected revision under lock; `409`; no silent merge of authority-bearing fields |
| Localhost service attacked from browser | Loopback, Host allowlist, no CORS, capability/session, CSRF for writes, CSP/sanitization |
| Markdown/frontmatter produces XSS or data exfiltration | Escape/sanitize, block raw HTML/remote loads, strict CSP, safe links |
| Remote feature quietly weakens defaults | Separate explicit mode with required TLS/auth/RBAC and threat-model review |
| UI grows into a heavy second product | Progressive stages, parity matrix, semantic/simple frontend, measured framework choice |
| SSE clients drift or lag | Revision-based resync; bounded channels; polling fallback; one stream per tab |
| Agent feedback impersonates authority | Authenticated actor, server-resolved target, explicit action schema, revision check, durable audit |
| Database becomes premature source of truth | File-native MVP; cache rebuildability; add storage abstraction only for a real second backend |
| Bundled frontend complicates releases | Reproducible pinned build, embedded fingerprinted assets, license/SBOM review, release-size budget |

## Open questions for review

1. Should `tandem web` ship only after the architecture refactor, or may a **read-only** prototype use a narrow query facade earlier?
2. Should the first UI use server-rendered HTML plus progressive enhancement, or a small bundled SPA? Prototype both only if maintenance/interaction trade-offs remain unclear.
3. Is loopback read access acceptable without a capability token, or should every session require a random startup capability from day one?
4. Should automatic browser opening be default, opt-in, or platform-dependent?
5. What is the canonical project revision: content hash, actor-event frontier, generation plus fingerprint, or another project-layer value?
6. Which event timeline fields are safe in the default UI/API, particularly paths, actor IDs, evidence, and freeform bodies?
7. What exact authorization distinguishes editing a task, accepting an accord, recording review acceptance, and completing/canceling work?
8. Should feedback/comments become a first-class Tandem protocol document/event, or remain an integration-side record linked to Tasks until interaction semantics mature?
9. How will a web user select a live agent target when task assignment and process identity differ?
10. What deployment is the first real remote target: single-user LAN/VPN, team server, or hosted multi-tenant service? Its answer materially changes auth and storage choices.
11. What scale targets (documents, Logs, concurrent viewers/writers) justify indexing, pagination defaults, or a database read model?
12. Should web semantic theme tokens become a documented cross-interface contract or remain a web mapping from existing TUI concepts?

## Authoritative references

- Axum official static-file example: <https://github.com/tokio-rs/axum/blob/main/examples/static-file-server/src/main.rs>
- Tokio bounded broadcast and lag semantics: <https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html>
- `notify` cross-platform watcher behavior and known limitations: <https://docs.rs/notify>
- MDN, using server-sent events (reconnect, event IDs, connection limits): <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events>
- WHATWG HTML Living Standard, Server-sent events: <https://html.spec.whatwg.org/multipage/server-sent-events.html>
- OWASP CSRF Prevention Cheat Sheet: <https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html>
- OWASP XSS Prevention Cheat Sheet: <https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html>
- OWASP Authentication Cheat Sheet: <https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html>
- WCAG 2.2: <https://www.w3.org/TR/WCAG22/>
- Local architecture direction: `plan/refactor_spec.md`
- Current CLI/TUI behavior and parity target: `tandem/plan/spec.md`
- Normative protocol direction: `protocol/plan/spec.md`
