---
title: Web
description: Browse one Tandem workspace in a local, read-only browser interface.
---

# Web

`tandem web` opens a browser view of the nearest Tandem workspace. It uses the
same canonical project and application queries as the CLI and TUI.

```sh
tandem web
```

Tandem selects an available port, prints the URL and project path, and opens
your default browser. Keep the terminal process running. Press `Ctrl-C` to stop
the server cleanly.

Use a fixed port when another local tool needs a stable URL:

```sh
tandem web --port 43123
```

To print the URL without opening a browser, use:

```sh
tandem web --no-open
```

Run `tandem web --help` for the command summary.

## What you can browse

The web interface is read-only. It provides:

- Board workflow states, filters, canonical task roles, and relationships;
- Validation attention for validation-state work, delivered accords, pending
  review, and requested changes;
- document bodies, metadata, parent and child relationships, accords, reviews,
  completion data, blockers, references, and related files;
- completed and canceled Logs with search and detail views;
- Rules grouped by category;
- ADR-compatible Decisions with detail views; and
- project health, configured states, snapshot warnings, counts, and revision.

Use **Refresh view** for an immediate reload. While the tab is visible, the page
also checks the project revision every three seconds and refreshes the current
view after an external change. The browser keeps the active filters, focus, and
scroll position where possible.

## Local boundary and security

Each process serves exactly one discovered workspace. It binds only to IPv4
loopback (`127.0.0.1`) and does not provide a remote-bind option. Loopback
reduces exposure, but it is not an authentication boundary. Stop the process
when you no longer need it, and do not run it for an untrusted local user.

The server accepts read-only `GET` and `HEAD` requests, validates the exact
loopback `Host`, sends no permissive CORS headers, rejects request bodies, and
caps request targets and concurrent requests. Browser responses use a
restrictive Content Security Policy, frame denial, no-referrer and no-store
policies, MIME sniffing protection, and no remote assets. Project Markdown uses
a small server-side renderer that escapes project text and never emits raw
HTML, links, or images.

Static HTML, CSS, and JavaScript are compiled into the `tandem` binary. The web
mode does not need the source checkout, a frontend build, Node.js, or a network
connection at runtime.

## Appearance and accessibility

The MVP uses a responsive Verdigris palette. It follows the browser or operating
system light/dark preference; the dark preference is the web MVP's Default Dark
check. The TUI `default-dark` and `verdigris` theme selectors do not configure
the browser interface, and the web MVP has no separate theme picker.

The page uses semantic landmarks and headings, a skip link, visible keyboard
focus, text labels in status badges, reduced-motion and forced-color support,
and a single-column narrow layout. It is designed for keyboard-only use, 200%
browser zoom, and screens down to 390 CSS pixels.

## Deferred capabilities

The current web mode does not create, edit, move, accept, complete, or cancel
work. It also does not provide remote/LAN access, authentication, SSE or
WebSocket updates, a database or synchronization provider, multi-workspace
selection, or an agent-feedback channel. These capabilities need separate
product and security work; they are not implied by the local read-only server.
