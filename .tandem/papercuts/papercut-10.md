---
id: papercut-10
title: "TUI: Papercuts section is not Tab-reachable and its section tab is not clickable"
status: open
createdAt: "2026-08-31T02:27:15Z"
updatedAt: "2026-08-31T02:27:15Z"
---
The Papercuts Board section (fourth section, derived from tag=papercut) cannot be reached like the other sections.

1. Keyboard: Tab/Left-Right cycling only moves through configured workflow states (self.states); the derived __papercuts section is never in that cycle, so the ONLY way in is the 'i' shortcut opening the read-only inbox popover.
2. Mouse: the section-tab hit map follows self.states too, so the PAPERCUTS section tab has no click target (unlike Todo/In progress/Validation). Only the header indicator 'Papercuts N' has a TogglePapercuts hit.

Expected: Papercuts behaves like a peer section - tab/arrow reachable and clickable to select the section (section selection, not just the popover).
