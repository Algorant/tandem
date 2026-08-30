//! Workspace-free landing surface for Tandem 0.3.0.
pub(crate) fn print() {
    println!("tandem - Tandem CLI 0.3.0");
    println!();
    println!("Commands: init, add task|decision, show, list, search, update");
    println!("  accord claim|deliver|rework|block|resume|release|fail");
    println!("  review, complete, cancel, rules list|add|edit|delete, tui, web");
    println!();
    println!("Run 'tandem <command> --help' for detailed usage.");
}
