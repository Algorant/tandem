---
id: task-67
type: task
title: "Research how to obtain a .md TLD/domain for tandem.md"
priority: "low"
relatedFiles: ["README.md", "plan/spec.md"]
tags: ["research", "domain", "docs", "branding"]
createdAt: "2026-06-29T22:56:39Z"
updatedAt: "2026-07-01T03:10:21Z"
accord:
  status: "accepted"
  deliveredAt: "2026-07-01T03:10:03Z"
  deliverables: ["Added plan/domain-research.md with recommendation and source list.", "Finding: tandem.md is registered/active through 2026-10-15; gettandem.md returned no NIC.MD match at verification time."]
  validation:
    commands: ["Parent reran NIC.MD WHOIS via Python socket for tandem.md and gettandem.md; tandem.md returned registered/OK with Cloudflare nameservers; gettandem.md returned no entries.", "Parent inspected plan/domain-research.md and git status/diff."]
  summary: "Accepted: research deliverable satisfies requested scope with direct NIC.MD WHOIS evidence, pricing/registrar notes, DNS/hosting steps, alternatives, and a concise recommendation."
  evidence: ["plan/domain-research.md", "whois.nic.md direct response"]
  filesChanged: ["plan/domain-research.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-07-01T03:10:13Z"
completedAt: "2026-07-01T03:10:21Z"
completion:
  summary: "Completed domain research for tandem.md. Added plan/domain-research.md with recommendation, registrar/pricing notes, alternatives, DNS/hosting setup, sources, and validation commands."
  validation: "Parent verified plan/domain-research.md and reran direct NIC.MD WHOIS: tandem.md is registered/active through 2026-10-15; gettandem.md returned no entries at verification time."
  reviewer: "pi-orchestrator"
---

## Description

Investigate whether `tandem.md` can be obtained as a domain/TLD-style web address, including the current status of `.md` domain registration, registrars that support `.md`, availability/WHOIS checks for `tandem.md`, expected pricing/renewal constraints, trademark or eligibility requirements, DNS/hosting setup steps, and practical alternatives if unavailable (e.g. gettandem.md, tandem.dev, tandem.sh, tandem.app, docs subdomain). Produce a concise recommendation with next steps.
