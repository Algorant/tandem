---
id: decision-2
type: decision
title: "Use Bun as the default JavaScript package manager and runner"
status: "accepted"
date: "2026-07-04"
deciders: ["Algorant"]
context: "Tandem's docs site and supporting JavaScript/TypeScript tooling currently use npm in places such as site/package-lock.json, site/README.md, justfile recipes, and the GitHub Pages workflow. The project owner prefers Bun by default and wants npm used only when Bun is actually impossible after reasonable avenues have been tried."
consequences: ["Docs-site package management and GitHub Pages workflow should migrate from npm/package-lock to Bun/bun.lock when validated.", "Future JS/TS automation should default to Bun and document concrete evidence before falling back to npm.", "Local just recipes and CI should use the same package manager to avoid mismatched validation."]
alternatives: ["Keep npm for the docs site because it already works with package-lock.json and GitHub Pages; rejected because the project owner prefers Bun unless impossible.", "Allow either npm or Bun without a default; rejected because it creates inconsistent validation and agent behavior."]
references: ["decision-1", "task-66"]
tags: ["docs", "tooling", "bun", "package-manager"]
createdAt: "2026-07-04T13:03:06Z"
updatedAt: "2026-07-04T13:03:06Z"
---

## Status

Accepted

## Context

Tandem uses JavaScript tooling for the Astro/Starlight docs site and Pi extension checks. Some workflows currently default to npm because the docs site was grounded in package-lock.json and GitHub Pages used npm ci. The project direction is to use Bun as the default package manager and runner unless a specific toolchain path makes Bun impossible after reasonable investigation.

## Decision

Use Bun over npm for JavaScript package management, scripts, and CI/local automation by default. Do not keep or introduce npm-based workflows merely because npm is familiar or already present. Prefer Bun lockfiles, Bun install/build/test commands, and Bun-backed just/GitHub Actions recipes where practical.

npm remains allowed only as an explicit exception when Bun cannot satisfy a concrete requirement after reasonable attempts, such as an upstream incompatibility, unsupported installation behavior, broken generated output, or an action/platform limitation.

## Consequences

- Existing npm-based docs-site workflow should be migrated to Bun where practical.
- `package-lock.json`, `npm ci`, and npm-specific documentation should be removed or replaced when migration is validated.
- Exceptions to Bun must document what was tried, why Bun failed, and what condition would allow revisiting the exception.
- CI and local recipes should stay aligned so agents do not validate with a different package manager than deployment uses.
