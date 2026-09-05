//! Clap derive model for the protocol 0.3.0 command surface.
//!
//! This module contains grammar only. Semantic validation remains in protocol
//! and app modules.
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "tandem", version, about = "Local-first project coordination")]
pub(crate) struct Cli {
    #[arg(short = 'j', long, global = true)]
    pub(crate) json: bool,
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Init(InitArgs),
    Add(AddArgs),
    Show(IdArgs),
    Assignment(IdArgs),
    List(ListArgs),
    Search(SearchArgs),
    Update(UpdateArgs),
    Accord(AccordArgs),
    Review(ReviewArgs),
    Complete(CompleteArgs),
    Cancel(CancelArgs),
    Rules(RulesArgs),
    Tui,
    Web(WebArgs),
}

#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) title: Option<String>,
}

#[derive(Debug, Args)]
pub(crate) struct AddArgs {
    #[command(subcommand)]
    pub(crate) command: AddCommand,
}
#[derive(Debug, Subcommand)]
pub(crate) enum AddCommand {
    Task(AddTaskArgs),
    Decision(AddDecisionArgs),
}
#[derive(Debug, Args)]
pub(crate) struct AddTaskArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) title: String,
    #[arg(long, required = true, allow_hyphen_values = true)]
    pub(crate) acceptance: Vec<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) body: Option<String>,
    #[arg(long)]
    pub(crate) kind: Option<String>,
    #[arg(long)]
    pub(crate) priority: Option<String>,
    #[arg(long)]
    pub(crate) effort: Option<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) tag: Vec<String>,
    #[arg(long)]
    pub(crate) due_date: Option<String>,
    #[arg(long)]
    pub(crate) parent: Option<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) blocker: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) reference: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) related_file: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) constraint: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) validation: Vec<String>,
}
#[derive(Debug, Args)]
pub(crate) struct AddDecisionArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) title: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) body: Option<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) decider: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) supersedes: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) reference: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) tag: Vec<String>,
}

#[derive(Debug, Args)]
pub(crate) struct IdArgs {
    #[arg(allow_hyphen_values = false)]
    pub(crate) id: String,
}
#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum Scope {
    Active,
    Archived,
    All,
}
#[derive(Debug, Args)]
pub(crate) struct ListArgs {
    #[arg(long, value_enum, default_value_t = Scope::Active)]
    pub(crate) scope: Scope,
    #[arg(long)]
    pub(crate) r#type: Option<String>,
    #[arg(long)]
    pub(crate) state: Option<String>,
    #[arg(long)]
    pub(crate) priority: Option<String>,
    #[arg(long)]
    pub(crate) effort: Option<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) tag: Vec<String>,
    #[arg(long)]
    pub(crate) assignee: Option<String>,
    #[arg(long)]
    pub(crate) parent: Option<String>,
    #[arg(long)]
    pub(crate) accord: Option<String>,
    #[arg(long)]
    pub(crate) decision_status: Option<String>,
    #[arg(long)]
    pub(crate) resolution: Option<String>,
    #[arg(long)]
    pub(crate) limit: Option<usize>,
}
#[derive(Debug, Args)]
pub(crate) struct SearchArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) query: String,
    #[arg(long, value_enum, default_value_t = Scope::Active)]
    pub(crate) scope: Scope,
    #[arg(long)]
    pub(crate) r#type: Option<String>,
    #[arg(long)]
    pub(crate) state: Option<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) tag: Vec<String>,
    #[arg(long)]
    pub(crate) parent: Option<String>,
    #[arg(long)]
    pub(crate) limit: Option<usize>,
}
#[derive(Debug, Args)]
pub(crate) struct UpdateArgs {
    pub(crate) id: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) title: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) body: Option<String>,
    #[arg(long)]
    pub(crate) kind: Option<String>,
    #[arg(long)]
    pub(crate) priority: Option<String>,
    #[arg(long)]
    pub(crate) effort: Option<String>,
    #[arg(long)]
    pub(crate) due_date: Option<String>,
    #[arg(long)]
    pub(crate) parent: Option<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) tag: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) reference: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) blocker: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) related_file: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) acceptance: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) constraint: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) validation: Vec<String>,
    #[arg(long)]
    pub(crate) status: Option<String>,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) decider: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) supersedes: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) clear: Vec<String>,
}

#[derive(Debug, Args)]
pub(crate) struct AccordArgs {
    #[command(subcommand)]
    pub(crate) command: AccordCommand,
}
#[derive(Debug, Subcommand)]
pub(crate) enum AccordCommand {
    Claim(ClaimArgs),
    Deliver(DeliverArgs),
    Rework(NoteArgs),
    Block(NoteArgs),
    Resume(IdArgs),
    Release(NoteArgs),
    Fail(NoteArgs),
}
#[derive(Debug, Args)]
pub(crate) struct ClaimArgs {
    pub(crate) id: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) assignee: String,
}
#[derive(Debug, Args)]
pub(crate) struct DeliverArgs {
    pub(crate) id: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) summary: String,
    #[arg(long, action = clap::ArgAction::Append, allow_hyphen_values = true)]
    pub(crate) evidence: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) file_changed: Vec<String>,
}
#[derive(Debug, Args)]
pub(crate) struct NoteArgs {
    pub(crate) id: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) note: String,
}
#[derive(Debug, Args)]
pub(crate) struct ReviewArgs {
    pub(crate) id: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) criterion: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) note: String,
    #[arg(long)]
    pub(crate) reviewer: Option<String>,
}
#[derive(Debug, Args)]
pub(crate) struct CompleteArgs {
    pub(crate) id: String,
    #[arg(long)]
    pub(crate) reviewer: Option<String>,
}
#[derive(Debug, Args)]
pub(crate) struct CancelArgs {
    pub(crate) id: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) note: String,
}
#[derive(Debug, Args)]
pub(crate) struct RulesArgs {
    #[command(subcommand)]
    pub(crate) command: RulesCommand,
}
#[derive(Debug, Subcommand)]
pub(crate) enum RulesCommand {
    List {
        category: Option<String>,
    },
    Add {
        category: String,
        #[arg(allow_hyphen_values = true)]
        text: String,
        #[arg(long)]
        source: Option<String>,
    },
    Edit {
        id: String,
        #[arg(allow_hyphen_values = true)]
        text: String,
        #[arg(long)]
        source: Option<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        clear: Vec<String>,
    },
    Delete {
        id: String,
    },
}
#[derive(Debug, Args)]
pub(crate) struct WebArgs {
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    pub(crate) port: Option<u16>,
    #[arg(long)]
    pub(crate) no_open: bool,
}
