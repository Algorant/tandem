//! Workspace-free landing surface for Tandem 0.3.0.
pub(crate) fn print() {
    println!("tandem - Tandem CLI 0.3.0");
    println!();
    println!("Work");
    println!("  init                 Create a Tandem workspace");
    println!("  add task|decision    Add a task or decision");
    println!("  show                 Show a task or decision");
    println!("  list                 List active tasks and decisions");
    println!("  search               Search active and completed work");
    println!("  update               Update task metadata and content");
    println!();
    println!("Agreements");
    println!("  accord claim         Claim a task");
    println!("  accord deliver       Deliver a task for acceptance");
    println!("  accord rework        Return a task for rework");
    println!("  accord block         Mark a task blocked");
    println!("  accord resume        Resume a blocked task");
    println!("  accord release       Release a claimed task");
    println!("  accord fail          Mark a task failed");
    println!("  review               Request exceptional human validation");
    println!("  complete             Complete and archive a task");
    println!("  cancel               Cancel and archive a task");
    println!();
    println!("Workspace");
    println!("  rules list|add|edit|delete  Manage project rules");
    println!("  tui                  Open the terminal interface");
    println!("  web                  Open the web interface");
    println!();
    println!("Run 'tandem <command> --help' for detailed usage.");
}
