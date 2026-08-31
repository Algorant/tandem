---
id: decision-5
type: decision
title: "Curate concise public release notes without installation guidance"
status: "accepted"
date: "2026-07-14"
deciders: ["Algorant"]
context: "Task task-72 established curated, version-specific GitHub Release notes as the preferred public artifact, separate from the reusable release checklist. The current tandem/RELEASE.md and tandem/GITHUB_RELEASE_NOTES.md mostly reflect that direction, but release documentation has historically mixed public notes with installation and operational guidance, including task-27's first-release context. External research converges on a small conditional set of conventional sections, human curation, and rejection of raw commit-log output. The project owner has clarified that public release notes must never include installation commands or installation guidance and that shipped bug fixes must be clearly grouped rather than buried among features. Existing release files are cited as context and implementation inputs, not treated as authoritative where they conflict with this accepted guidance."
consequences: ["Public release notes contain no installation commands or installation guidance; installation and operational material remains in dedicated docs and tandem/RELEASE.md.", "Every release with shipped user-visible defect corrections includes a dedicated Bug fixes section.", "Release owners curate and review generated commit/PR notes rather than publishing them unchanged.", "Release notes use a small conditional set of conventional headings, omit empty sections and unshipped work, and use product surfaces only as optional grouping when useful.", "Migration and compatibility actions appear under Breaking changes or Compatibility when relevant, never as installation guidance."]
alternatives: ["Reuse tandem/RELEASE.md as the public release body; rejected because checklist, operational, and installation material should not appear in public notes.", "Publish commit-generated or GitHub-generated drafts unchanged; rejected because they are noisy and not reliably organized by user impact.", "Mix fixes into feature or product-surface sections; rejected because fixes need a dedicated Bug fixes section.", "Document rejected or not-shipped work in release notes; rejected because release notes should describe shipped behavior and relevant current limitations."]
references: ["task-131", "task-72", "task-27"]
tags: ["docs", "release", "decision", "guidance"]
createdAt: "2026-07-14T20:05:05Z"
updatedAt: "2026-07-15T04:24:43Z"
---

## Status

Accepted — approved by the product owner after review of the conventional conditional template and supporting release-note research. Minor naming choices between conventional synonyms remain editorial rather than reasons to invent Tandem-specific headings.

## Context

Public release notes serve readers deciding what changed in a specific version. They should not duplicate the reusable release procedure, installation documentation, or a raw commit/PR inventory. Prior Tandem research in `task-72` recommended curated per-release notes and separated `tandem/GITHUB_RELEASE_NOTES.md` from the operational checklist in `tandem/RELEASE.md`. The first-release work in `task-27` and current files provide historical context, but any installation-oriented public-note guidance in them is superseded by this accepted decision.

External conventions support the same direction:

- [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) calls for human-curated notable changes grouped under conventional categories such as Added, Changed, Deprecated, Removed, Fixed, and Security, and rejects raw version-control log dumps.
- [GitHub's generated release notes documentation](https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes) describes generated PR, contributor, and full-changelog content and requires review so the result contains all and only wanted information.
- Current Rust-tool releases provide concise conventional examples: [uv](https://github.com/astral-sh/uv/releases) uses Enhancements and Bug fixes, [bat](https://github.com/sharkdp/bat/releases) uses Features and Bugfixes, and [ripgrep](https://github.com/BurntSushi/ripgrep/releases) uses concise narrative with bug entries.

These sources are research inputs, not templates to copy mechanically. They converge on a small conditional vocabulary rather than novel project-specific top-level headings.

## Decision

Public release notes are concise, version-specific, manually curated by user impact, and built from this conditional template:

1. A version/title plus a one- or two-sentence summary.
2. Optional `Highlights`, only for substantial releases whose most important outcomes benefit from emphasis.
3. Optional, prominent `Breaking changes`, whenever shipped changes break compatibility or require migration.
4. `Features` or `New features` when features shipped.
5. `Improvements` or `Changed` when meaningful non-feature improvements shipped.
6. A dedicated `Bug fixes` section whenever one or more user-visible defect corrections shipped. Describe fixes in outcome language; do not bury them among features or improvements.
7. Optional `Security`, `Deprecations`/`Removals`, `Compatibility`, or `Known issues`, only when applicable and useful to users.

Never emit an empty section. Product surfaces such as Protocol, CLI, TUI, Docs, and Integrations may be subheadings or grouping within a conventional section when a large release benefits from them; they are not required top-level sections and should not make a small release more complex.

Public notes must never contain installation commands, installer snippets, package-manager commands, PATH setup, or installation guidance. Installation and release-operation material belongs in product installation documentation and the reusable `tandem/RELEASE.md` checklist, not in `tandem/GITHUB_RELEASE_NOTES.md` or the published GitHub Release body. Migration or compatibility actions belong under `Breaking changes` or `Compatibility`; they must never turn into installation guidance.

Do not mention rejected, shelved, reverted-before-release, experimental-only, or otherwise not-shipped work. Do not add a `Not included` section. Release notes describe what shipped and relevant current limitations, not internal planning history.

Commit ranges, merged PRs, contributors, full-changelog links, and GitHub-generated notes are drafting and completeness-check inputs only. The release owner verifies that the draft contains all and only wanted user-facing information, rewrites entries around user impact, removes contributor/PR noise and duplicates, and publishes only reviewed curated notes. Generated output is never authoritative or published unchanged.

## Approval

The product owner approved the conventional conditional template. Per-release editors may choose conventional synonyms such as `Features` versus `New features` and `Improvements` versus `Changed`, but must preserve the no-install-content rule, dedicated bug-fix rule, breaking-change prominence, conditional omission of empty sections, and curated-over-generated rule.

## Consequences

Release authors maintain a concise curated notes file per release and keep procedural detail in the release checklist. Public notes become easier to scan and distinguish features from fixes. Generated notes can reduce omission risk but require editorial review. Existing release documentation must be reconciled with the accepted decision rather than assumed authoritative.

## Alternatives considered

- Publish the reusable release checklist as the GitHub Release body: rejected because it exposes operational and installation detail and repeats boilerplate.
- Publish commit-generated or GitHub-generated notes unchanged: rejected because repository history is not consistently grouped or written around user impact.
- Mix bug fixes into feature/product sections: rejected because releases containing fixes need a clearly discoverable `Bug fixes` section.
- Include rejected or not-shipped work for transparency: rejected because public release notes are a shipped-change record, not a planning ledger.
