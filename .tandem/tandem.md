---
protocolVersion: 0.1.0
type: workspace
title: "tandem"
states:
  - id: todo
    title: To Do
  - id: in-progress
    title: In Progress
  - id: review
    title: Review
rules:
  always:
    - id: 1
      rule: "Keep Tandem task tags lowercase/kebab-case and useful for `tdm list --tag` filtering; tags are convention-only and not a replacement for protocol fields."
      source: "task-22"
  never: []
  prefer:
    - id: 1
      rule: "Use one primary area tag first: `protocol`, `tui`, `pi-tandem`, `docs`, `config`, `rules`, or `ui`."
      source: "task-22"
    - id: 2
      rule: "Add only a few capability/workflow tags when they aid delegation, such as `accord`, `review`, `logs`, `editor`, `relationships`, `delegation`, `taxonomy`, `smoke`, `validation`, or concrete TUI facets like `theme`, `keyboard`, `mouse`, and `markdown`."
      source: "task-22"
  context: []
---

# tandem
