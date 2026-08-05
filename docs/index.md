---
title: Home
description: Human and agent coordination for local-first project work.
hero:
  title: Tandem
  tagline: "Local-first coordination for human and agent work in the same repository."
  actions:
    - text: Start with the Quickstart
      link: /quick-start/
      variant: primary
    - text: Explore the Concepts
      link: /concepts/
      variant: minimal
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
---

Tandem keeps human and agent work in the same repository. It gives teams a shared, local-first record of what needs to happen, who agreed to it, and how the work is reviewed.

## Work that stays together

<div class="td-home-card-grid" role="list">
  <a class="td-home-card" href="/concepts/" role="listitem">
    <span class="td-home-card__eyebrow">Plan</span>
    <strong>Tasks and workflows</strong>
    <span>Turn project intent into visible tasks, states, and repeatable workflows.</span>
  </a>
  <a class="td-home-card" href="/guides/agents-and-adapters/" role="listitem">
    <span class="td-home-card__eyebrow">Collaborate</span>
    <strong>Accords and review</strong>
    <span>Make acceptance criteria explicit, then keep delivery and review separate.</span>
  </a>
  <a class="td-home-card" href="/guides/decisions/" role="listitem">
    <span class="td-home-card__eyebrow">Remember</span>
    <strong>Decisions and logs</strong>
    <span>Keep durable decisions and completed work close to the project history.</span>
  </a>
</div>

## A simple loop

<div class="td-home-loop" role="list">
  <article class="td-home-loop__step td-home-loop__step--define" role="listitem">
    <span class="td-home-loop__number" aria-hidden="true">01</span>
    <h3>Define the work as tasks.</h3>
  </article>
  <article class="td-home-loop__step td-home-loop__step--agree" role="listitem">
    <span class="td-home-loop__number" aria-hidden="true">02</span>
    <h3>Agree on acceptance criteria.</h3>
  </article>
  <article class="td-home-loop__step td-home-loop__step--complete" role="listitem">
    <span class="td-home-loop__number" aria-hidden="true">03</span>
    <h3>Give the work to an agent to complete.</h3>
  </article>
  <article class="td-home-loop__step td-home-loop__step--review" role="listitem">
    <span class="td-home-loop__number" aria-hidden="true">04</span>
    <h3>Have another agent review that work or show it to you for final approval.</h3>
  </article>
</div>
