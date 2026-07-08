---
title: Tandem documentation
description: Local-first coordination for humans and agents working in the same repository.
hero:
  title: Coordinate humans and agents in the repo
  tagline: "Tandem keeps task state, work agreements, decisions, validation, and completed history in local Markdown so every actor works from the same durable context."
  image:
    html: |
      <svg class="td-hero-mark" viewBox="0 0 260 260" role="img" aria-labelledby="td-hero-mark-title">
        <title id="td-hero-mark-title">Two linked Tandem work loops</title>
        <defs>
          <radialGradient id="tdHeroGlow" cx="50%" cy="45%" r="58%">
            <stop offset="0" stop-color="#8ec07c" stop-opacity="0.28"/>
            <stop offset="0.7" stop-color="#83a598" stop-opacity="0.08"/>
            <stop offset="1" stop-color="#1d2021" stop-opacity="0"/>
          </radialGradient>
          <linearGradient id="tdHeroLoopA" x1="35" x2="220" y1="78" y2="174" gradientUnits="userSpaceOnUse">
            <stop offset="0" stop-color="#d5e7d0"/>
            <stop offset="0.52" stop-color="#8ec07c"/>
            <stop offset="1" stop-color="#83a598"/>
          </linearGradient>
          <linearGradient id="tdHeroLoopB" x1="55" x2="208" y1="188" y2="64" gradientUnits="userSpaceOnUse">
            <stop offset="0" stop-color="#e6bf86"/>
            <stop offset="1" stop-color="#c7e5df"/>
          </linearGradient>
        </defs>
        <circle cx="130" cy="130" r="116" fill="url(#tdHeroGlow)"/>
        <g fill="none" stroke-linecap="round" stroke-linejoin="round">
          <path d="M84 162c-29 0-53-20-53-45s24-45 53-45c20 0 34 8 52 25l18 18c16 16 27 24 43 24 18 0 32-10 32-24s-14-24-32-24c-12 0-22 5-33 15" stroke="url(#tdHeroLoopA)" stroke-width="19"/>
          <path d="M176 98c29 0 53 20 53 45s-24 45-53 45c-20 0-34-8-52-25l-18-18c-16-16-27-24-43-24-18 0-32 10-32 24s14 24 32 24c12 0 22-5 33-15" stroke="url(#tdHeroLoopB)" stroke-width="19" opacity="0.96"/>
          <path d="M112 130h36" stroke="#fbf1c7" stroke-width="11" opacity="0.9"/>
        </g>
      </svg>
  actions:
    - text: Start the quickstart
      link: /quick-start/
      icon: right-arrow
    - text: Browse concepts
      link: /concepts/
      icon: open-book
      variant: secondary
    - text: View GitHub
      link: https://github.com/Algorant/tandem
      icon: github
      variant: minimal
      attrs:
        rel: noreferrer
---

Tandem is a local-first coordination system for people and agents working in the same repository. It stores active tasks, explicit accords, durable decisions, validation notes, and completed-work logs in plain Markdown under `.tandem/`, then layers a CLI, TUI, and lightweight integrations on top.

The goal is shared context that survives beyond chat: project state that can be read in any editor, reviewed in Git, searched later, and trusted by both humans and automation.

## Why teams reach for Tandem

<div class="td-home-card-grid">
  <a class="td-home-card" href="/concepts/#board-task-states">
    <span class="td-home-card__eyebrow">Board state</span>
    <strong>Keep active work honest</strong>
    <span>Tasks move through <code>todo</code>, <code>in-progress</code>, and <code>validation</code>; accepted work leaves the board as a log.</span>
  </a>
  <a class="td-home-card" href="/concepts/#accords">
    <span class="td-home-card__eyebrow">Accords</span>
    <strong>Separate delivery from approval</strong>
    <span>Workers can claim and deliver evidence without self-approving; reviewers decide acceptance, rework, blocking, or failure.</span>
  </a>
  <a class="td-home-card" href="/concepts/#decisions">
    <span class="td-home-card__eyebrow">Decisions</span>
    <strong>Record durable context</strong>
    <span>ADR-compatible decision documents live beside tasks instead of disappearing into chat or issue comments.</span>
  </a>
  <a class="td-home-card" href="/concepts/#logs">
    <span class="td-home-card__eyebrow">Logs</span>
    <strong>Search completed history</strong>
    <span>Completion archives the task body, validation, changed files, and accord metadata into first-class history.</span>
  </a>
</div>

## Run one explicit loop

```sh
# Install the latest release from the cargo-dist generated installer.
curl -fsSL https://trytandem.dev/install.sh | sh

# Initialize a local coordination workspace.
tandem init --title "My Project"

# Add work, claim it, deliver evidence, accept, and archive.
tandem add --title "Write project brief" --description "Draft and validate the first docs slice."
tandem accord claim task-1 --assignee alice
tandem accord deliver task-1 --summary "Drafted and checked" --validation "Ran docs check"
tandem accord accept task-1 --reviewer bob --summary "Looks good"
tandem complete task-1 --summary "Published the brief" --validation "Reviewed by Bob"
```

The [Quickstart](/quick-start/) walks through the same lifecycle with install notes, validation steps, logs, and `tandem tui`.

## Explore by role

<div class="td-home-card-grid td-home-card-grid--compact">
  <a class="td-home-card td-home-card--compact" href="/quick-start/">
    <span class="td-home-card__eyebrow">New users</span>
    <strong>Install and run the first task</strong>
  </a>
  <a class="td-home-card td-home-card--compact" href="/cli/">
    <span class="td-home-card__eyebrow">CLI users</span>
    <strong>Use the command families</strong>
  </a>
  <a class="td-home-card td-home-card--compact" href="/tui/">
    <span class="td-home-card__eyebrow">Terminal users</span>
    <strong>Navigate the Ratatui interface</strong>
  </a>
  <a class="td-home-card td-home-card--compact" href="/protocol/">
    <span class="td-home-card__eyebrow">Builders</span>
    <strong>Understand the file protocol</strong>
  </a>
</div>

## Current status

Tandem is in early v0 implementation. The core CLI surface is implemented, the Ratatui TUI is growing around Board/Validation/Logs/Rules/Decisions workflows, and the docs site keeps canonical content in `docs/` so the public site can evolve with the protocol.
