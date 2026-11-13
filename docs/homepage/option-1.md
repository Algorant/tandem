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
<span class="td-home-loop__number">01 · plan</span>
<h3>Write it down</h3>
<p>One Markdown file with its acceptance criteria, priority and related files.</p>
</div>
<div class="td-home-loop__step td-home-loop__step--agree">
<span class="td-home-loop__number">02 · claim</span>
<h3>Make ownership visible</h3>
<p>The Accord records who claimed the Task and what they owe. The current owner is visible on the Board.</p>
</div>
<div class="td-home-loop__step td-home-loop__step--review">
<span class="td-home-loop__number">03 · deliver or review</span>
<h3>Show the result</h3>
<p>A delivery carries a summary and evidence. If an unresolved criterion needs a human decision, review is the optional next step.</p>
</div>
<div class="td-home-loop__step td-home-loop__step--complete">
<span class="td-home-loop__number">04 · archive</span>
<h3>Keep the history</h3>
<p>Completion archives the Task in Logs, where the next person can read what happened and why.</p>
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
