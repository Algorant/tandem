//! Static semantic inventory for Tandem's fixed TUI input model.
//!
//! This is intentionally metadata, not a configurable keymap framework.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BindingScope {
    Global,
    Navigation,
    CurrentView,
    Board,
    Validation,
    Logs,
    Rules,
    Decisions,
    Utility,
    Dialogs,
    Mouse,
}

impl BindingScope {
    pub(super) const ALL: [Self; 11] = [
        Self::Global,
        Self::Navigation,
        Self::CurrentView,
        Self::Board,
        Self::Validation,
        Self::Logs,
        Self::Rules,
        Self::Decisions,
        Self::Utility,
        Self::Dialogs,
        Self::Mouse,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::Navigation => "Navigation",
            Self::CurrentView => "Current view",
            Self::Board => "Board actions",
            Self::Validation => "Validation",
            Self::Logs => "Logs",
            Self::Rules => "Rules",
            Self::Decisions => "Decisions",
            Self::Utility => "Utility inbox",
            Self::Dialogs => "Dialogs and text input",
            Self::Mouse => "Mouse",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Binding {
    pub(super) scope: BindingScope,
    pub(super) keys: &'static str,
    pub(super) description: &'static str,
}

pub(super) const BINDINGS: &[Binding] = &[
    Binding {
        scope: BindingScope::Global,
        keys: "q",
        description: "quit from any non-text context",
    },
    Binding {
        scope: BindingScope::Global,
        keys: "Ctrl-C",
        description: "emergency safe quit, including prompts",
    },
    Binding {
        scope: BindingScope::Global,
        keys: "?",
        description: "open this universal keybinding reference",
    },
    Binding {
        scope: BindingScope::Global,
        keys: "r",
        description: "reload project, theme, and utility data",
    },
    Binding {
        scope: BindingScope::Global,
        keys: "1–4",
        description: "switch Board, Logs, Rules, and Decisions",
    },
    Binding {
        scope: BindingScope::Global,
        keys: "i",
        description: "open or close the utility inbox",
    },
    Binding {
        scope: BindingScope::Global,
        keys: "Esc",
        description: "close or leave the top temporary layer",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: "j/k · ↑/↓",
        description: "move selection or scroll the focused pane",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: "h/l · ←/→",
        description: "move between local tabs, categories, or panes",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: "g/G · Home/End",
        description: "move to the beginning or end",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: "Ctrl-U/D · PgUp/PgDn",
        description: "move one page",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: "Tab · Shift-Tab",
        description: "change pane focus only",
    },
    Binding {
        scope: BindingScope::Navigation,
        keys: "Enter",
        description: "open, expand, confirm, or activate selection",
    },
    Binding {
        scope: BindingScope::CurrentView,
        keys: "context",
        description: "the current view or open layer is prioritized in this reference",
    },
    Binding {
        scope: BindingScope::Board,
        keys: "a",
        description: "add a task",
    },
    Binding {
        scope: BindingScope::Board,
        keys: "e",
        description: "edit the selected active task",
    },
    Binding {
        scope: BindingScope::Board,
        keys: "b",
        description: "toggle State Board and Epic Board",
    },
    Binding {
        scope: BindingScope::Board,
        keys: "f",
        description: "open Board filter controls",
    },
    Binding {
        scope: BindingScope::Board,
        keys: "m",
        description: "open task state-movement picker",
    },
    Binding {
        scope: BindingScope::Board,
        keys: "v",
        description: "open context-aware Validation actions",
    },
    Binding {
        scope: BindingScope::Board,
        keys: "Space",
        description: "toggle selected row inline preview",
    },
    Binding {
        scope: BindingScope::Validation,
        keys: "v",
        description: "choose accept, rework, or apply/archive when valid",
    },
    Binding {
        scope: BindingScope::Logs,
        keys: "/",
        description: "search completed logs",
    },
    Binding {
        scope: BindingScope::Rules,
        keys: "a/e/d",
        description: "add, edit, or delete a rule",
    },
    Binding {
        scope: BindingScope::Rules,
        keys: "Enter",
        description: "open or close selected rule preview",
    },
    Binding {
        scope: BindingScope::Rules,
        keys: "Tab · Shift-Tab",
        description: "focus the open preview or return to the rule list",
    },
    Binding {
        scope: BindingScope::Rules,
        keys: "j/k · Ctrl-U/D · PgUp/PgDn",
        description: "scroll the focused rule preview",
    },
    Binding {
        scope: BindingScope::Decisions,
        keys: "a",
        description: "add a decision",
    },
    Binding {
        scope: BindingScope::Decisions,
        keys: "Enter",
        description: "expand or collapse selected decision",
    },
    Binding {
        scope: BindingScope::Utility,
        keys: "i · Esc",
        description: "close and restore the underlying view",
    },
    Binding {
        scope: BindingScope::Utility,
        keys: "Tab · Shift-Tab",
        description: "change list/detail focus",
    },
    Binding {
        scope: BindingScope::Utility,
        keys: "Enter",
        description: "open the selected utility item in detail",
    },
    Binding {
        scope: BindingScope::Dialogs,
        keys: "Enter · Esc",
        description: "confirm/advance or cancel",
    },
    Binding {
        scope: BindingScope::Dialogs,
        keys: "printable text",
        description: "belongs to the active text field, including ?, q, and i",
    },
    Binding {
        scope: BindingScope::Dialogs,
        keys: "Ctrl-U",
        description: "clear the active text field",
    },
    Binding {
        scope: BindingScope::Mouse,
        keys: "click",
        description: "select rows, panes, tabs, actions, or close targets",
    },
    Binding {
        scope: BindingScope::Mouse,
        keys: "wheel",
        description: "scroll the pane under the pointer",
    },
    Binding {
        scope: BindingScope::Mouse,
        keys: "drag",
        description: "not bound; drag and drop is not supported",
    },
];

pub(super) fn bindings_for(scope: BindingScope) -> impl Iterator<Item = &'static Binding> {
    BINDINGS
        .iter()
        .filter(move |binding| binding.scope == scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_contains_required_sections_and_no_removed_aliases() {
        for scope in BindingScope::ALL {
            assert!(
                bindings_for(scope).next().is_some(),
                "missing {}",
                scope.label()
            );
        }
        let keys = BINDINGS
            .iter()
            .map(|binding| binding.keys)
            .collect::<Vec<_>>();
        for removed in ["P", "H/L", "A/R/C", "t/p/F", "a or n", "u/d"] {
            assert!(!keys.contains(&removed), "removed alias remains: {removed}");
        }
    }
}
