use std::io::{self, IsTerminal};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";

const GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "Work",
        &[
            ("add", "Create a task or epic"),
            ("move", "Move work to another state"),
            ("update", "Update task details"),
            ("complete", "Complete and archive work"),
            ("cancel", "Cancel and archive work"),
        ],
    ),
    (
        "Collaborate",
        &[
            ("accord", "Manage work agreements"),
            ("rules", "Manage project rules"),
            ("decision", "Record and inspect decisions"),
        ],
    ),
    (
        "Explore",
        &[
            ("list", "List active work"),
            ("show", "Show a document"),
            ("search", "Search work, logs, and Papercuts"),
            ("papercut", "Capture small non-blocking friction"),
            ("log", "Browse completed work"),
        ],
    ),
    (
        "Workspace",
        &[
            ("init", "Create a Tandem workspace"),
            ("upgrade", "Upgrade the workspace protocol"),
            ("tui", "Open the terminal interface"),
            ("web", "Open the local read-only web interface"),
            ("version", "Show the installed version"),
        ],
    ),
];

pub(super) fn print() {
    let styled = io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    print!("{}", render(styled));
}

pub(super) fn render(styled: bool) -> String {
    let mut output = String::new();
    if styled {
        output.push_str(BOLD);
    }
    output.push_str("Tandem");
    if styled {
        output.push_str(RESET);
    }
    output.push_str("\nLocal-first coordination for humans and agents.\n");

    for (heading, commands) in GROUPS {
        output.push('\n');
        if styled {
            output.push_str(BOLD);
        }
        output.push_str(heading);
        if styled {
            output.push_str(RESET);
        }
        output.push('\n');

        for (command, description) in *commands {
            output.push_str("  ");
            if styled {
                output.push_str(CYAN);
            }
            output.push_str(command);
            if styled {
                output.push_str(RESET);
            }
            output.push_str(&" ".repeat(10 - command.len()));
            output.push_str(description);
            output.push('\n');
        }
    }

    output.push_str("\nRun `tandem <command> --help` for detailed usage.\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_landing_is_aligned_and_covers_every_command() {
        let output = render(false);
        assert!(!output.contains("\x1b["));
        assert!(output.starts_with("Tandem\nLocal-first coordination for humans and agents.\n"));
        assert!(output.ends_with("Run `tandem <command> --help` for detailed usage.\n"));

        let rendered_commands = output
            .lines()
            .filter_map(|line| line.strip_prefix("  "))
            .map(|line| line.split_whitespace().next().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            rendered_commands,
            [
                "add", "move", "update", "complete", "cancel", "accord", "rules", "decision",
                "list", "show", "search", "papercut", "log", "init", "upgrade", "tui", "web",
                "version"
            ]
        );
    }

    #[test]
    fn styled_landing_only_styles_titles_and_command_names() {
        let output = render(true);
        assert!(output.starts_with("\x1b[1mTandem\x1b[0m\n"));
        assert!(output.contains("\x1b[1mWork\x1b[0m\n"));
        assert!(output.contains("  \x1b[36madd\x1b[0m       Create a task or epic\n"));
        assert!(output.contains("  \x1b[36mupgrade\x1b[0m   Upgrade the workspace protocol\n"));
        assert!(!output.contains("\x1b[36mCreate"));
    }
}
