---
title: TUI and web
description: Browse local Tandem boards in the terminal or read-only browser view.
---

## Terminal UI

Run `tandem tui` to open the board discovered from the current directory. Press `w` for the session board chooser, which shows local activity and recorded assignees; press `Enter` to open the selected board in the full TUI. Add an existing board with `Ctrl-A`; switch away from a board, then use `Ctrl-D` to remove it from the session. Session membership is temporary and is forgotten when the TUI exits. `tandem tui --add-workspace PATH` can seed more boards for one invocation; repeat the flag to add more.

The active board syncs in the background. Inactive boards do not; while the chooser is open, their local summaries refresh about every five seconds.

[![Tandem TUI with its board tabs and task list.](/images/tui-board.png)](/images/tui-board.png)

[![Session activity and recorded assignees in the chooser.](/images/tui-session-activity.png)](/images/tui-session-activity.png)

The full TUI has Board, Logs, Rules, and Decisions views. Switch with `1`–`4` or the mouse; press `?` for the current key reference.

[![The full TUI opened on a selected session board.](/images/tui-full-board-selection.png)](/images/tui-full-board-selection.png)

## Read-only web view

`tandem web` opens a browser view of one discovered workspace. It is read-only and binds to loopback; it does not serve a workspace roster or allow remote access.

[![The read-only web view of one Tandem workspace.](/images/web-read-only.png)](/images/web-read-only.png)

See [Configuration](/reference/configuration/) for TUI themes and Board badges. Board storage and sync behavior are covered in [Concepts](/concepts/#board-storage-and-sync).
