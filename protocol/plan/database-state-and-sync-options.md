# Database-backed state and pluggable sync providers

Status: research note; no protocol decision
Date: 2026-07-13
Related: `protocol/plan/spec.md`, `protocol/plan/event-storage-options.md`, Tandem tasks `task-119` and `task-120`

## Executive summary

Tandem can gain transactional local writes, indexed queries, quiet Git status, and multi-user synchronization without making PostgreSQL—or any hosted service—the protocol. The cleanest architecture is a **local-first logical store with one writable authority per workspace mode**:

- Existing and simple workspaces remain **file-authoritative**. Markdown/YAML under `.tandem/` is the source of truth; an optional SQLite index is disposable and must never be the only copy of a mutation.
- A future database-enabled workspace is **SQLite-authoritative on each local replica**. Every accepted mutation atomically updates the current relational state and appends an immutable change envelope/outbox record. Markdown becomes an explicit, deterministic projection/export and compatibility format, not a second independently writable authority.
- PostgreSQL is a **shared synchronization and policy service**, holding the same provider-neutral changes and materialized state for a workspace. It is not required for offline use and must not contain Tandem semantics that are absent from the protocol/core mutation layer.

This is a staged migration, not a v0 replacement. Do not silently enable database authority in an existing `.tandem/` workspace. First prove a read-only SQLite projection, then transactional local authority and round-trip export, then one-way upload, and only then bidirectional PostgreSQL sync with deliberate conflict UX.

The core sync unit should be a validated **document revision/change**, not a SQL row diff, filesystem timestamp, or raw audit event. Audit events remain evidence and history; they are not sufficient to reconstruct current state under the current protocol. Provider contracts should exchange opaque canonical change envelopes and advertise capabilities. They should not expose SQL or duplicate hierarchy, accord, review, completion, or validation rules.

## Current constraints and architecture

The current protocol deliberately optimizes for readable, editable, portable files:

- `.tandem/tandem.md` contains workspace configuration and rules.
- `.tandem/board/*.md` contains active tasks and decisions.
- `.tandem/logs/*.md` contains completed/canceled documents and is explicitly the completed-history source of truth.
- Per-actor JSONL events enrich history and are explicitly insufficient to reconstruct the board or completed corpus.
- Unknown frontmatter fields must survive tool mutations.
- IDs are immutable and unique across board and logs; hierarchy roles are derived from resolved documents rather than ID shape alone.
- Workflow state, accord status, and review status are separate concepts.
- The v0 specification lists both “requiring a database for normal use” and “hiding the source of truth behind opaque binary state” as non-goals.

The Rust implementation is currently file-coupled in `tandem/src/main.rs`: `Workspace` contains board/log/config/event paths; reads parse directory files into `Document`; `HierarchyIndex` builds an in-memory board+logs graph; mutations snapshot and patch files, use atomic replacement, and append an event afterward. Cooperative hierarchy operations lock the config inode. File signature checks catch some concurrent edits, while hard-link creation lets concurrent ID allocators retry. The document mutation and event append are not one atomic transaction; the implementation reports that the file mutation may already be durable if event append fails.

These facts favor extracting a provider-neutral repository/mutation seam before changing persistence. They also mean a cache-only SQLite proof can be low risk, while making SQLite authoritative is a protocol and product decision—not an internal refactor.

## What belongs in database-backed state

A database-capable store needs enough information to preserve current behavior and losslessly export/import the file protocol.

| Data | Store as structured state? | Preserve lossless source? | Sync by default? | Notes |
| --- | --- | --- | --- | --- |
| Workspace identity and protocol/schema versions | Yes | Yes in export | Yes | Stable `workspace_id` must not be an absolute path or remote URL. |
| Workspace states/configuration | Yes | Yes | Yes | Semantic configuration; local UI preferences and credentials are excluded. |
| Active documents | Yes | Yes | Yes | Tasks, epics, subtasks, decisions, and future custom types share a document envelope. |
| Completed/canceled logs | Yes | Yes | Yes | Model as terminal document location/status plus completion metadata; retain standalone body/content. |
| Parent, blocker, and related references | Yes | Via document export | Yes | Normalize for integrity/querying, but preserve unresolved related references as protocol warnings. |
| Accord and review metadata | Yes | Yes | Yes | Keep distinct structured objects; do not infer one lifecycle from another. |
| Decision ADR metadata | Yes | Yes | Yes | Preserve unknown decision fields and Markdown body. |
| Rules | Yes | Yes | Yes | Stable rule IDs; ordering needs an explicit key. |
| Audit/event history | Yes, append-only | Exportable | Policy-dependent | Separate from sync changes. Audit records may be private or retained differently. |
| Sync changes, replica cursors, conflicts, outbox | Yes | No protocol export | Yes/operational | Required for replication; not user document content. |
| Unknown frontmatter/body representation | Yes | Yes | Yes | Retain canonical parsed value plus source-compatible representation until lossless rendering is proven. |
| TUI display config, caches, search indexes | Locally only | No | No by default | Derived or machine-specific. |
| Credentials, tokens, private keys | Secret store/config only | No | Never | Store references to credentials, not credentials in workspace rows or exports. |
| Presence, ephemeral locks, progress | Optional/leased | No | Optional capability | Must not become durable workflow truth. |

Binary attachments are not currently a first-class protocol concept. A future attachment provider should use content-addressed blobs and a separate capability; do not put large blobs into the initial change stream.

## Source-of-truth options

### Option A: Markdown remains authoritative; SQLite is only an index/cache

On every read or filesystem notification, parse files into SQLite. Queries use the index; writes still patch files first and then refresh the index. PostgreSQL could receive exported snapshots or changes derived from file revisions.

**Strengths**

- Fully compatible with current protocol goals, Git workflows, direct editing, and recovery.
- Database loss is harmless; adoption can be transparent and incremental.
- Delivers fast search, filtering, relationship queries, and TUI startup without settling sync authority.

**Weaknesses**

- Cannot make a file mutation plus audit/outbox append transactional.
- File watching is advisory; crashes, editors, Git checkout, rename storms, and coarse timestamps require periodic full reconciliation.
- Bidirectional sync creates two writers: remote changes and direct local edits. Conflict detection must compare content revisions and materialization state.
- PostgreSQL is closer to backup/index replication than a clean shared collaboration model.

**Fit:** recommended first proof and safe permanent default for current file workspaces, but not the final model for robust bidirectional collaboration.

### Option B: SQLite is authoritative; Markdown is a derived projection/export

CLI/TUI mutations commit current state, revision/change, and outbox atomically in one local transaction. A materializer writes deterministic Markdown for explicit export, compatibility, or an optional mirror. Import is an explicit mutation path with conflict checks.

**Strengths**

- Transactional invariants, atomic event/outbox recording, stable snapshots, efficient queries, and reliable offline queueing.
- One local serialization point handles concurrent human/agent processes cleanly.
- Provider-neutral changes can synchronize to PostgreSQL without reverse-engineering file edits.

**Weaknesses**

- Direct Markdown editing no longer immediately edits canonical state; opaque-only storage conflicts with current goals unless export, inspect, backup, and recovery are excellent.
- Git diffs cease to be the automatic collaboration and audit mechanism.
- Requires schema migrations, corruption recovery, database tooling, and a clear downgrade story.

**Fit:** recommended authority for an explicit future `database` workspace mode after review and proof, not a silent v0 migration.

### Option C: PostgreSQL is authoritative; clients cache locally

Clients send mutations to a server and refresh/cache state locally.

**Strengths:** familiar centralized multi-user transactions, authorization, backup, and operational control.

**Weaknesses:** weak offline behavior, server/vendor coupling, latency, unavailable-server failure modes, and a large security/operations requirement. A write-behind cache eventually recreates Option B.

**Fit:** not recommended. PostgreSQL should be a sync provider, not the only authority required for normal use.

### Option D: Markdown and database are co-equal writable authorities

Both direct file edits and database/API writes are accepted continuously and reconciled.

**Strengths:** appears to preserve every workflow.

**Weaknesses:** there is no deterministic winner after partial failure; projection loops, stale overwrites, and ambiguous recovery are unavoidable without designating an authority at each boundary. Operational complexity is greater than either model alone.

**Fit:** reject. A hybrid architecture is useful; dual authority is not.

## Recommendation: mode-specific authority over one logical model

Adopt a provider-neutral **logical workspace model**, but make authority explicit:

1. `storage.mode = files` (existing/default): Markdown is writable authority. SQLite may be a disposable projection. Sync, if later offered, imports remote changes through the same file-aware compare-and-swap mutation layer.
2. `storage.mode = database` (future, opt-in): local SQLite is writable authority. Markdown is generated only by `export`, explicit `materialize`, or a clearly marked read-only mirror. Importing edited Markdown is an explicit command that creates normal revisions and may produce conflicts.
3. `sync.provider` is independent of local storage mode. PostgreSQL is the first shared provider, but sync envelopes and capability negotiation are backend-neutral.
4. Never switch authority based merely on finding a `.db` file or connection string. Record the mode and workspace ID in portable configuration, require a migration command, create a verified backup, and support a preflight/dry run.

This preserves current users and keeps a viable file-only implementation while providing a coherent destination for collaboration. If product review concludes that directly editable Markdown must remain canonical forever, stop after Option A and treat PostgreSQL as snapshot/change exchange with explicit import; do not claim transparent, strongly consistent bidirectional sync.

## Proposed logical model

Use stable opaque identifiers internally (UUIDv7/ULID-class values are suitable) for workspace, replica, actor, change, and revision identity. User-facing IDs such as `task-120` remain immutable protocol identities and retain existing allocation/role rules. Internal IDs avoid coupling synchronization to filenames or sequential allocation.

### Core records

- **workspace**: `workspace_id`, title, protocol version, storage schema version, created/updated time, settings document/revision.
- **replica**: `replica_id`, workspace, actor/device metadata, creation/last-seen time, optional retired time. A clone should create a new replica ID while retaining workspace ID.
- **document**: internal ID, workspace, protocol ID, type, kind, location (`board`/`logs`), title, body, structured known metadata, unknown metadata, current revision, tombstone/archived markers, timestamps.
- **document_revision**: immutable revision ID, document ID, parent revision(s), canonical content hash, full canonical snapshot initially, actor/replica, logical and wall-clock metadata. Full snapshots are simpler for the first proof; delta compression can come later.
- **relation**: workspace, source document, relation kind (`parent`, `blocker`, `reference`, `supersedes`, etc.), target protocol ID/internal ID when resolved, stable element ID, ordering key where meaningful.
- **rule**: workspace, category, stable rule ID, rule text, source, ordering key, revision.
- **change**: immutable provider-neutral envelope, workspace, change ID, replica ID, per-replica sequence, subject, base revision, resulting revision, operation kind, payload version/payload, timestamp, hash, optional signature.
- **audit_event**: existing event identity and content, separate retention/export policy, optional causal `change_id`.
- **sync_cursor/outbox**: provider/remote, last pulled token, acknowledgement state, retry metadata. Payloads are immutable; retries must not mutate a change.
- **conflict**: subject, local/base/remote revision IDs and snapshots, conflict kind, detection time, resolution status, optional resolving change.

Accord, review, completion, and decision metadata can begin as versioned JSON objects inside the document snapshot while high-value query fields are projected into columns or indexed JSON paths. Prematurely normalizing every optional field makes unknown-field preservation and protocol evolution harder. Relationships and rules deserve normalized tables because integrity, ordering, and graph queries are central.

### Invariants enforced by the core, not providers

- Protocol ID uniqueness across active and completed documents.
- Parent/blocker resolution rules and strict Epic/Task/Subtask role/ID validation.
- No parented Epic, child beneath Subtask, invalid reparenting, or completion with active descendants where prohibited.
- Separate workflow, accord, and review state machines.
- Completed/canceled document completeness.
- Unknown-field preservation and canonical serialization.
- Optimistic base-revision checks and deterministic conflict creation.

Database constraints should reinforce these invariants (unique keys, foreign keys where targets are resolved, non-null checks), but SQL schemas are not the sole specification.

## Initial SQLite design

Place the authoritative database outside tracked source by default, keyed by workspace/checkout identity under the platform state directory; a workspace-local ignored location can be an explicit option. A cache-only database may live under cache directories. Do not place a live WAL database on a network filesystem: SQLite documents that WAL relies on same-host shared memory. SQLite also permits concurrent readers with a writer in WAL mode but only one writer at a time, which is appropriate for Tandem's small local transactions.

Illustrative schema (names and exact types are provisional):

```sql
CREATE TABLE workspace (
  workspace_id TEXT PRIMARY KEY,
  protocol_version TEXT NOT NULL,
  schema_version INTEGER NOT NULL,
  title TEXT NOT NULL,
  settings_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE replica (
  replica_id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspace(workspace_id),
  next_seq INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL
);

CREATE TABLE document (
  document_uuid TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspace(workspace_id),
  protocol_id TEXT NOT NULL,
  doc_type TEXT NOT NULL,
  kind TEXT,
  location TEXT NOT NULL CHECK (location IN ('board', 'logs')),
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  metadata_json TEXT NOT NULL,
  current_revision_id TEXT NOT NULL,
  content_hash BLOB NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (workspace_id, protocol_id)
);

CREATE TABLE document_revision (
  revision_id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  document_uuid TEXT NOT NULL REFERENCES document(document_uuid),
  parent_revisions_json TEXT NOT NULL,
  snapshot_json TEXT NOT NULL,
  content_hash BLOB NOT NULL,
  actor_id TEXT NOT NULL,
  replica_id TEXT NOT NULL,
  replica_seq INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (workspace_id, replica_id, replica_seq)
);

CREATE TABLE relation (
  workspace_id TEXT NOT NULL,
  source_uuid TEXT NOT NULL REFERENCES document(document_uuid),
  element_id TEXT NOT NULL,
  relation_kind TEXT NOT NULL,
  target_protocol_id TEXT NOT NULL,
  target_uuid TEXT,
  order_key TEXT,
  PRIMARY KEY (workspace_id, source_uuid, element_id)
);

CREATE TABLE change_log (
  change_id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  replica_id TEXT NOT NULL,
  replica_seq INTEGER NOT NULL,
  subject_id TEXT NOT NULL,
  base_revision_id TEXT,
  result_revision_id TEXT NOT NULL,
  operation TEXT NOT NULL,
  payload_version INTEGER NOT NULL,
  payload_json TEXT NOT NULL,
  payload_hash BLOB NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (workspace_id, replica_id, replica_seq)
);

CREATE TABLE sync_ack (
  provider_id TEXT NOT NULL,
  change_id TEXT NOT NULL REFERENCES change_log(change_id),
  acknowledged_at TEXT,
  attempt_count INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  PRIMARY KEY (provider_id, change_id)
);

CREATE TABLE sync_cursor (
  provider_id TEXT PRIMARY KEY,
  opaque_cursor TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE conflict (
  conflict_id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  subject_id TEXT NOT NULL,
  base_revision_id TEXT,
  local_revision_id TEXT NOT NULL,
  remote_revision_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  details_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  resolved_at TEXT
);
```

Rules, audit events, and provider metadata would add tables in the proof slice that needs them. All mutations should use one transaction: validate base/current graph, reserve any protocol ID, write current document/relations/rules, append revision/change/audit event, and enqueue each configured provider. Set foreign key enforcement explicitly, use a bounded busy timeout, and retry `BUSY` only at whole-transaction boundaries.

Use an application-owned migration table (and optionally SQLite `application_id`/`user_version` as diagnostics), ordered forward migrations, and transactional migration where SQLite permits it. Never infer compatibility only from application binary version. Before destructive migration, make a consistent snapshot with SQLite's Online Backup API or `VACUUM INTO`, verify it can be opened, and retain it until migration success. A plain copy of only the main file is unsafe while WAL is active.

## Initial PostgreSQL sync design

PostgreSQL should store provider-neutral envelopes and a materialized current view, partitioned by workspace. An authenticated sync service is preferable to distributing raw database credentials and schema access to every CLI: it stabilizes the protocol, centralizes authorization/rate limits, and allows other provider implementations.

Minimum server tables mirror `workspace`, membership/role, accepted `change_log`, current documents/relations/rules, conflicts, and per-replica acknowledgement/watermark. Every row includes `workspace_id`; composite unique/foreign keys should include it to prevent cross-workspace references. The acceptance transaction should:

1. authenticate actor and authorize workspace/action;
2. reject an unsupported envelope/schema version or capability;
3. deduplicate by `change_id` and `(workspace_id, replica_id, replica_seq)`;
4. verify payload hash and sequence policy;
5. compare `base_revision_id` with current subject revision;
6. apply a non-conflicting change or record/return a conflict;
7. append the immutable envelope and update materialized state atomically;
8. return a monotonic server cursor/receipt.

Use database-generated cursor ordering only as a **delivery order**, not as semantic causality. Wall-clock timestamps are display metadata and cannot resolve conflicts. PostgreSQL `READ COMMITTED` is sufficient for simple row compare-and-swap with `SELECT ... FOR UPDATE`/conditional update; graph-wide allocations or invariants may need explicit locking or `SERIALIZABLE`. PostgreSQL documents that Serializable/Repeatable Read transactions can fail and that the complete transaction logic must be retried. The service must bound and jitter retries and return conflict rather than retry forever.

Native PostgreSQL logical replication is not the initial Tandem provider protocol. It replicates database tables and operational schema, requires replication privileges/slots, and does not define Tandem's offline outbox, authorization, conflict, or provider portability semantics. It may later replicate the sync service operationally, but clients should speak the Tandem sync contract.

## Provider extension boundary and capabilities

Keep two interfaces separate:

### Local state store

The core needs a small repository/transaction abstraction, conceptually:

```text
StateStore
  open(workspace_ref) -> StoreInfo
  snapshot(query, consistency) -> WorkspaceSnapshot
  transact(expected_revisions, commands) -> CommitResult
  changes_after(local_cursor, limit) -> ChangePage
  import_changes(changes, policy) -> ImportResult
  export(format, destination) -> ExportReport
  backup(destination) / verify() / migrate(target_version)
```

`commands` are semantic operations validated by the Tandem core, not arbitrary SQL patches. The file store and SQLite store implement the same observable model where possible. The file store may truthfully report weaker transactional capabilities.

### Remote sync provider

```text
SyncProvider
  describe() -> ProviderInfo + CapabilitySet
  connect(auth_ref, workspace_id, replica_id) -> Session
  push(changes, idempotency_context) -> PushResult
  pull(cursor, limit) -> ChangePage
  acknowledge(cursor/change_ids) -> AckResult       # optional
  get_snapshot(checkpoint) -> Snapshot              # optional
  put/get_blob(hash)                                # optional future capability
  health() -> diagnostics
```

Initial capabilities should be explicit and versioned:

- `push_changes`, `pull_changes`, `bidirectional`
- `idempotent_change_ids`
- `ordered_cursor` (provider delivery order only)
- `compare_base_revision`
- `conflict_records`
- `snapshot_bootstrap`
- `transactions` and maximum batch size
- `server_authorization`, `workspace_membership`
- `audit_retention`, `history_compaction`
- `encryption_in_transit`, `provider_managed_at_rest`
- future `realtime_notifications`, `blobs`, `signed_changes`, `end_to_end_encryption`

Capability negotiation must fail closed when correctness depends on a missing capability. For example, a provider lacking compare-and-swap cannot be selected for automatic bidirectional writes; it can still be an export/backup target. Keep provider configuration declarative, but do not load arbitrary in-process plugins in the first POC. A subprocess/HTTP protocol or separately compiled adapter offers a safer compatibility and trust boundary later.

## Change tracking, ordering, and idempotency

A change envelope should contain at least:

```json
{
  "envelopeVersion": 1,
  "changeId": "...",
  "workspaceId": "...",
  "replicaId": "...",
  "replicaSeq": 42,
  "actorId": "...",
  "subject": {"kind": "document", "id": "task-120"},
  "operation": "document.update",
  "baseRevisionId": "...",
  "resultRevisionId": "...",
  "parents": ["..."],
  "payload": {},
  "createdAt": "...",
  "payloadHash": "..."
}
```

- `change_id` makes retries idempotent; the same ID with a different hash is corruption/security failure.
- `(replica_id, replica_seq)` detects duplicate/gapped local streams, but gaps should not permanently block independent later changes unless strict audit mode requires it.
- Revision parents express causality. A server cursor orders transport, not concurrent creation.
- Payloads should begin as complete canonical subject snapshots plus operation metadata. This costs space but simplifies replay, migration, unknown-field preservation, and recovery. Optimize to patches only after measurement.
- Apply a batch in a transaction where supported. Acknowledgement advances only after local durable apply.
- Never synthesize last-write-wins from timestamps. Clocks skew and arrival order is not user intent.

## Conflict model and concurrent updates

The first design should use optimistic concurrency with three-way comparison:

1. A mutation names the base revision it observed.
2. If current equals base, apply normally.
3. If current differs, compare base/local/remote at semantic fields and stable collection element IDs.
4. Auto-merge only demonstrably independent changes; otherwise preserve both revisions and create a conflict.

Safe early auto-merges include different scalar fields, additions to stable-ID sets, and append-only audit records. Potentially dangerous changes include competing workflow/accord/review transitions, body edits to overlapping text, completion versus update, reparenting, delete/cancel versus edit, rule reorder/edit, and two allocations of the same user-facing ID. Those require protocol-aware resolution or user choice.

Conflict UX should show base, local, and remote values; actor/replica and timestamps; affected invariants; and choices to keep local, keep remote, manually merge, or defer. Resolution creates a new revision parented by both competing revisions and is itself synchronized. Never discard the losing revision silently.

For Markdown body merge, a standard three-way textual merge can propose a result but must not auto-commit conflict markers into canonical state. For ordered lists, assign stable element IDs plus fractional/order keys rather than treating array position as identity. Rebalancing order keys must be a deterministic maintenance change.

Sequential `task-N` allocation is a special multi-writer problem. PostgreSQL can serialize server-connected allocation, but offline replicas cannot safely predict a globally unused next number. POC choices are: reserve ranges while online; use opaque IDs for offline-created drafts and assign protocol IDs at publish; or allow collisions that must be renamed before acceptance. Because current protocol IDs are immutable and never reused, **offline creation policy is an open product decision**; do not silently rename synchronized documents or references.

## Local-first and offline behavior

- Every database-mode command reads and commits SQLite without contacting a provider.
- Sync is explicit (`tandem sync`) initially; background sync can follow after diagnostics and cancellation are reliable.
- The outbox is durable and visible through status: pending count, last push/pull, conflict count, provider, cursor, and last error.
- Pull downloads a bounded page, validates it, and atomically applies it with cursor advancement. A crash before commit safely retries the page.
- Provider downtime never prevents local reads/writes unless workspace policy explicitly requires an online lease for a sensitive action.
- Backpressure and quota errors retain local changes. Auth failure pauses sync without rolling back local work.
- Initial bootstrap uses a consistent provider snapshot plus a cursor, then pulls changes after that cursor. Snapshot/content hashes are verified before adoption.
- Compaction may delete old payloads only after a durable snapshot/checkpoint and retention policy; audit retention is independent.

## Schema versioning and migrations

Track three versions independently:

1. **Protocol model version**: semantics and serialized document fields.
2. **Local database schema version**: SQLite tables/indexes.
3. **Sync envelope/provider API version**: wire compatibility and capabilities.

A binary may understand multiple protocol/envelope versions while migrating its local schema. Migration rules:

- acquire exclusive migration ownership and refuse normal writes while migrating;
- backup first, record migration start/result, and run integrity checks;
- prefer expand/migrate/contract changes so mixed client versions can coexist during staged server rollout;
- preserve unknown payload fields and reject only versions that affect correctness;
- never let an older client destructively rewrite a newer store;
- make server minimum/maximum client envelope versions discoverable;
- test upgrade, interrupted upgrade, rollback-from-backup, and export after every migration.

## Workspace identity, clones, and boundaries

Generate a random portable `workspace_id` once. It identifies the conceptual project across clones and providers. Generate a separate local `replica_id` for each checkout/store. Do not derive either from path, Git remote, title, or database URL: these change, collide, and may reveal private data.

Fork/copy UX must distinguish:

- **clone/continue**: retain workspace ID, create replica ID, and connect only with authorization;
- **fork/new workspace**: generate new workspace ID, retain imported content provenance optionally;
- **restore**: retain both identity and change history only when intentionally replacing the same replica; otherwise issue a new replica ID to avoid sequence reuse.

All server authorization and keys are scoped at least by workspace. Cross-workspace references remain textual/explicit unless the protocol later defines them; foreign keys must never accidentally resolve across tenants.

## Security and privacy

### Authentication and authorization

- Keep provider credentials in OS keychain/secret files/environment-backed auth profiles, never tracked config, SQLite exports, event payloads, or command history.
- Prefer short-lived OAuth/device or service tokens for hosted providers and TLS with certificate verification. PostgreSQL deployments may use TLS and managed identities, but direct database credentials should be an administrator option, not the default client design.
- Authorize operations, not just rows: reader, contributor, reviewer/acceptor, workspace admin, audit reader/exporter. Accord acceptance, review decisions, destructive migration, membership changes, and audit export may need distinct permissions.
- PostgreSQL row-level security can reinforce tenant isolation, but table owners and roles with `BYPASSRLS` bypass it; service authorization tests and least-privilege roles remain necessary. Constraints can also reveal cross-tenant existence, so composite tenant keys and careful error handling matter.

### Privacy and encryption

- Treat bodies, summaries, paths, actor names, timestamps, accord evidence, and audit history as potentially sensitive.
- Encrypt transport. Require documented provider at-rest encryption and backup policy; local full-disk encryption is the baseline for SQLite. Field/database encryption can be added, but key management and searchable metadata leakage must be explicit.
- End-to-end encryption conflicts with server-side validation, search, conflict inspection, and RLS. Advertise it as a separate future capability, not a checkbox over the initial PostgreSQL design.
- Support audit/history retention independently from current documents; provide redactable exports and make deletion/legal-hold semantics explicit before hosted use.
- Logs and backups must not contain tokens or complete sensitive payloads by default.

### Integrity

Hash canonical change payloads and snapshots. Hashes detect corruption but do not prove actor identity; optional signatures require key rotation/revocation and canonical encoding. Server receipts/checkpoints can make truncation detectable. None of these replace backup or authorization.

## Backup, export, and recovery

Required operational paths before database mode is production-ready:

- `status/doctor`: identity, mode, provider, schema versions, integrity result, outbox/cursor/conflicts, backup age.
- SQLite consistent backup using the Online Backup API or `VACUUM INTO`; verify with integrity check and manifest hash.
- PostgreSQL service backups using documented SQL dump, filesystem/base backup, or continuous archiving appropriate to deployment; periodically test restore.
- Deterministic portable export to the existing `.tandem/` Markdown/JSONL shape, plus a manifest containing workspace/protocol version and content hashes.
- Import dry run reporting additions, updates, unknown fields, invalid relations, ID collisions, and conflicts.
- Disaster recovery from local backup without a provider, and bootstrap from provider snapshot after local loss.
- Escape hatch to file-authoritative mode only through verified export and an explicit authority switch.

Portability means a user can inspect and export their complete semantic workspace without the original provider. It does not require copying a live SQLite file between machines or treating PostgreSQL dumps as the interchange protocol.

## CLI/TUI configuration and UX

Portable `.tandem/tandem.md` may eventually declare intent without secrets, for example:

```yaml
workspaceId: ws_...
storage:
  mode: files            # files | database
sync:
  provider: postgres-team
  policy: manual         # disabled | manual | auto
```

Machine-local configuration resolves `postgres-team` to provider type, endpoint, auth-profile reference, timeouts, and TLS policy. Workspace policy may require a provider/capability, but it must not embed a vendor URL/token as protocol semantics.

Proposed commands are product proposals, not commitments:

- `tandem storage status|migrate|export|import|backup|verify`
- `tandem sync status|push|pull|run|resolve`
- `tandem provider list|describe|test`

Every destructive or authority-changing command needs dry-run, backup destination, and rollback instructions. The TUI should display offline/pending/conflict state without blocking local work; conflicts need a dedicated queue. Avoid a green “synced” indicator when audit history or attachments use unsynchronized policy.

## Compatibility and migration from file workspaces

A safe migration is explicit and reversible until cutover:

1. Discover and validate `.tandem/tandem.md`, board, logs, all event sources, unknown fields, duplicate IDs, and unresolved references under current protocol rules.
2. Create a complete file backup/archive and manifest.
3. Generate/confirm workspace ID and a new local replica ID.
4. Import configuration, rules, documents, relations, completed metadata, and events into a temporary SQLite database.
5. Export SQLite back to a temporary `.tandem/` tree and compare canonical semantics plus unknown fields and bodies. Byte identity is not required unless promised, but no semantic loss is allowed.
6. Run graph validation, counts, content hashes, and representative CLI reads against both stores.
7. Atomically install the database and record `storage.mode: database`; retain the original tree as backup. Do not leave an apparently writable mirror unless writes are detected and rejected/imported explicitly.
8. Configure provider and bootstrap only after local cutover succeeds.

For a cache-only rollout, no authority switch occurs: build SQLite from files, record source content hashes, invalidate/rebuild on mismatch, and permit `rm` of the cache at any time. Cache schema upgrades may rebuild rather than migrate.

Mixed old/new client behavior is hazardous in database mode because old clients will edit Markdown. Minimum choices are to remove the writable projection, place a conspicuous generated marker/readme, and have new tools detect modified exports. A database-mode workspace should declare a minimum capable client version and fail old tooling safely where possible.

## Staged proof-of-concept plan

### Stage 0 — define seams and golden semantics

- Extract/describe provider-neutral workspace snapshot, semantic commands, validation, and commit results around the current implementation.
- Build golden tests for unknown fields, hierarchy, IDs, accord/review separation, completion/logs, rules, decisions, and event failure behavior.
- No storage behavior change.

**Exit:** file backend behavior is characterized and mutation semantics do not depend on SQL.

### Stage 1 — disposable SQLite read model

- Import a file workspace into SQLite, query list/search/review/log/hierarchy views, invalidate by content hashes, and rebuild safely.
- Measure startup/query cost and database size on synthetic large boards.
- Exercise concurrent readers and cache rebuild; document WAL same-host restriction.

**Exit:** deleting SQLite loses no state; query parity and complete rebuild are proven.

### Stage 2 — round-trip and transactional local authority experiment

- In a test-only/opt-in workspace, atomically commit document + revision + change + audit/outbox.
- Deterministically export/import Markdown and prove semantic round trips, including unknown fields.
- Test process crashes at each transaction/materialization boundary, concurrent writers, backup/restore, and schema migration interruption.

**Exit:** no dual-write ambiguity; recovery and escape export are demonstrated. Architecture review is required before product adoption.

### Stage 3 — provider contract with a local fake

- Implement push/pull, idempotent retries, opaque cursors, snapshot bootstrap, capability negotiation, and injected failures against an in-process or filesystem fake.
- Test duplicate, reordered, delayed, corrupt, unsupported-version, partial-page, and authorization errors.

**Exit:** provider contract contains no SQL assumptions and converges for non-conflicting changes.

### Stage 4 — PostgreSQL single-workspace POC

- Add authenticated service/API, workspace scoping, immutable changes, materialized state, compare-and-swap, cursor receipts, backup/restore.
- Prove two replicas: offline edits, reconnect, duplicate push, concurrent independent edits, and explicit conflict.
- Security test tenant isolation and role restrictions.

**Exit:** objective convergence/idempotency tests pass; no claim of general conflict-free merging.

### Stage 5 — conflict and migration UX

- Implement three-way conflict records/resolution, body proposals, ID allocation policy, dry-run file migration, diagnostics, and provider removal/export.
- Human validation for TUI/UX and recovery messaging.

**Exit:** users can understand pending work, resolve conflicts without loss, and leave the provider.

### Stage 6 — production hardening decision

- Load/chaos testing, quotas/backpressure, key rotation, retention/compaction, telemetry privacy, operational runbooks, compatibility matrix, and restore drills.
- Review whether plugin loading, realtime notifications, signatures, blobs, or E2EE are justified by evidence.

**Exit:** record a first-class Tandem decision before changing canonical protocol/storage defaults.

## Risks and mitigations

| Risk | Consequence | Mitigation |
| --- | --- | --- |
| Dual writable authorities | Lost/oscillating updates | Explicit mode; import/export boundary; never background two-way materialization. |
| Protocol semantics leak into providers | Vendor lock-in and divergent validation | Semantic core owns commands/invariants; providers transport versioned envelopes. |
| Unknown-field/format loss | Breaks forward compatibility and user trust | Lossless snapshots, golden round-trip tests, full snapshot changes initially. |
| Offline ID collision | Immutable ID conflict | Decide reservation/draft/collision policy before offline shared creation. |
| Incorrect auto-merge | Silent workflow corruption | Conservative three-way merge; preserve revisions; human conflict queue. |
| SQLite/WAL misuse | Busy failures or corruption risk | Same-host local disk, short transactions, busy policy, tested backup/checkpointing. |
| PostgreSQL retry bugs | Duplicate or partial effects | Idempotent IDs, whole-transaction retry, bounded backoff, transactional materialization. |
| Tenant/auth error | Data disclosure or unauthorized acceptance | Composite workspace keys, service authorization, RLS defense-in-depth, adversarial tests. |
| Sync history grows forever | Cost/performance degradation | Snapshot/checkpoint and explicit retention after restore proof. |
| Database opacity | User lock-in and difficult repair | Deterministic full export, inspect/doctor, documented schema/version, provider-independent backup. |
| Old clients edit projections | Divergence | Capability/min-version marker, no writable mirror, explicit import and modification detection. |
| Audit and state semantics conflated | Bad replay and retention assumptions | Separate change log, current state, and audit event models. |
| Provider plugin supply chain | Credential/code execution compromise | No arbitrary in-process plugins initially; signed/distributed adapters or subprocess/HTTP boundary later. |

## Open questions for architecture review

1. Is database mode acceptable as an explicit alternative to the current “readable editable Markdown source” goal, or must files remain canonical permanently?
2. Should `workspaceId` become protocol frontmatter, and what exact clone/fork/restore UX governs it?
3. How are user-facing sequential IDs allocated while replicas create documents offline without violating immutability and no-reuse rules?
4. What is the minimal canonical change payload: full document snapshot, semantic command plus result, or both?
5. Which fields/collections may auto-merge, and which always require human review?
6. Are completed logs mutable through corrections, or append-only after completion except via a specific amendment operation?
7. Are decisions and workspace rules synchronized with the same conflict policy as tasks, or do they need stricter permissions/leases?
8. Is PostgreSQL accessed through a Tandem sync service (recommended) or directly by trusted clients for the first POC?
9. What multi-user roles are required, especially for accord acceptance, review acceptance, completion, cancellation, rule changes, and audit export?
10. What audit guarantee and retention does sync promise? Are raw audit events replicated by default or separately configured as task-119 suggests?
11. Must database-mode Markdown export preserve bytes/comments/order, or only canonical semantics, body, and unknown fields?
12. What provider conformance suite and compatibility/version policy is required before calling the interface pluggable?
13. What are hosted-service privacy, data residency, deletion, legal hold, and encryption requirements?
14. Should local SQLite live in workspace state, user state, or an ignored sidecar, and how should Herdr/Git worktrees share or isolate replicas?
15. What objective scale/latency target justifies SQLite indexing and PostgreSQL sync complexity?

## Factual references

Primary references used to validate database behavior (accessed 2026-07-13):

- SQLite, **Write-Ahead Logging**: WAL readers and a writer can proceed concurrently, there is one writer at a time, checkpointing needs management, and WAL requires same-host shared memory rather than a network filesystem. <https://sqlite.org/wal.html>
- SQLite, **Online Backup API**: consistent live snapshots can be copied incrementally; copying a live database file naively has locking/crash limitations. <https://www.sqlite.org/backup.html>
- SQLite, **PRAGMA statements**: `application_id`, `user_version`, foreign-key/integrity, busy, and WAL controls are available but are SQLite-specific. <https://www.sqlite.org/pragma.html>
- PostgreSQL, **Transaction Isolation** and **Serialization Failure Handling**: Serializable/Repeatable Read may require retries, and correctness requires retrying the complete transaction logic. <https://www.postgresql.org/docs/current/transaction-iso.html> and <https://www.postgresql.org/docs/current/mvcc-serialization-failure-handling.html>
- PostgreSQL, **Row Security Policies**: RLS can restrict rows but owners and `BYPASSRLS` roles normally bypass it; constraints require care. <https://www.postgresql.org/docs/current/ddl-rowsecurity.html>
- PostgreSQL, **Backup and Restore**: documented approaches include SQL dump, filesystem-level backup, and continuous archiving. <https://www.postgresql.org/docs/current/backup.html>
- PostgreSQL, **Logical Replication**: native publication/subscription is database-table replication, not an application-level offline sync/conflict protocol. <https://www.postgresql.org/docs/current/logical-replication.html>

Repository claims are grounded in `protocol/plan/spec.md`, `protocol/plan/event-storage-options.md`, and the current file storage/mutation implementation in `tandem/src/main.rs`.
