---
id: task-13
type: task
title: "CLI: init emits no JSON envelope on success"
state: todo
priority: "low"
tags: ["papercut"]
accord:
  status: "ready"
  acceptance: ["tandem init --title X --json prints one success envelope on stdout when it creates a workspace."]
  updatedAt: "2026-09-04T01:36:29Z"
createdAt: "2026-09-04T01:36:29Z"
updatedAt: "2026-09-04T01:36:29Z"
---

## Description

Observed while writing the Pi adapter parser against 0.12.2.

`tandem init --title X -j` creates the workspace, prints nothing, and exits 0. Every other command emits an envelope under `-j`, and the failure path already does:

```
$ tandem init --title probe -j     # fresh dir
(no output, exit 0)
$ tandem init --title probe -j     # existing workspace
{"error":{"code":"io","details":{},"message":"Tandem workspace already exists at /tmp/tw."},"ok":false}
```

D53 says JSON is global with one shared envelope. An adapter has to special-case empty stdout plus exit 0 as success for this one command.
