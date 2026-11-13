---
title: tandem
description: Local-first coordination for people and agents working in the same repository.
template: splash
pagefind: false
editUrl: false
head:
  - tag: meta
    attrs:
      name: robots
      content: noindex
hero:
  title: Work your<br>agents can<br>read.
  tagline: tandem keeps the plan in the repository as plain Markdown. Tasks, Accords, Decisions and Rules that a person and an agent open, edit and trust together.
  actions:
    - text: Start a Task
      link: /quick-start/
      variant: primary
      icon: rocket
    - text: How the board works
      link: /concepts/
      variant: minimal
      icon: open-book
---

<div class="td-home-loop">
<div class="td-home-loop__step">
<span class="td-home-loop__number">01 · todo</span>
<h3>Write it down</h3>
<p>One Markdown file with its acceptance criteria, its priority and the files it relates to.</p>
</div>
<div class="td-home-loop__step td-home-loop__step--agree">
<span class="td-home-loop__number">02 · in progress</span>
<h3>Claim the work</h3>
<p>An Accord records who took the Task and what they owe. Two agents do not quietly work the same thing.</p>
</div>
<div class="td-home-loop__step td-home-loop__step--review">
<span class="td-home-loop__number">03 · validation</span>
<h3>Hand it back</h3>
<p>A delivery carries a summary and evidence. A result someone must judge goes to review instead of being declared done.</p>
</div>
<div class="td-home-loop__step td-home-loop__step--complete">
<span class="td-home-loop__number">04 · logs</span>
<h3>It becomes history</h3>
<p>Completion archives the Task in Logs, so the next person reads what happened and why.</p>
</div>
</div>

## Pick a starting point

<div class="td-home-card-grid">
<a class="td-home-card" href="/quick-start/">
<span class="td-home-card__eyebrow">Quickstart</span>
<strong>Install and finish one Task</strong>
<span>Install the binary, initialize a board, claim a Task, deliver it with evidence and complete it.</span>
</a>
<a class="td-home-card" href="/concepts/">
<span class="td-home-card__eyebrow">Concepts</span>
<strong>Tasks, Accords and the board</strong>
<span>How work, review and storage fit together, and how the board syncs on its own branch.</span>
</a>
<a class="td-home-card" href="/agents/">
<span class="td-home-card__eyebrow">Agents</span>
<strong>Point an agent at it</strong>
<span>A short section in <code>AGENTS.md</code>, then project rules that the agent reads before working.</span>
</a>
<a class="td-home-card" href="/cli/">
<span class="td-home-card__eyebrow">CLI</span>
<strong>The whole interface</strong>
<span>Every command in one table, with <code>--json</code> for anything that needs to call tandem from code.</span>
</a>
</div>

The board is ordinary files in `.tandem/`, so reading it needs no service. The TUI and the read-only web view read the same records.
