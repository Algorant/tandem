//! Peer command-line interface over shared application and protocol behavior.
//!
//! This module owns manual argument parsing, dispatch, command adapters, and
//! exact process output. It does not own durable filesystem mutation or infer
//! protocol semantics; those remain in `app`, `project`, and `protocol`.

mod args;
mod commands;
mod landing;
mod output;

use crate::CliError;
use args::*;
use commands::*;

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) enum StartupRequest {
    Exit,
    Tui,
    Web(crate::web::Options),
}

pub(crate) fn run(args: Vec<String>) -> Result<StartupRequest, CliError> {
    dispatch(args)
}

fn dispatch(mut args: Vec<String>) -> Result<StartupRequest, CliError> {
    if args.is_empty() {
        landing::print();
        return Ok(StartupRequest::Exit);
    }

    let command = args.remove(0);
    match command.as_str() {
        "init" => cmd_init(parse_init_args(&args)?)?,
        "upgrade" => cmd_upgrade(&args)?,
        "list" => cmd_list(parse_list_args(&args)?)?,
        "show" => cmd_show(parse_show_args(&args)?)?,
        "add" => cmd_add(parse_add_args(&args)?)?,
        "move" => cmd_move(parse_move_args(&args)?)?,
        "update" => cmd_update(parse_update_args(&args)?)?,
        "complete" => cmd_complete(parse_complete_args(&args)?)?,
        "cancel" => cmd_cancel(parse_cancel_args(&args)?)?,
        "search" => cmd_search(parse_search_args(&args)?)?,
        "log" => cmd_log(&args)?,
        "accord" => cmd_accord(&args)?,
        "rules" => cmd_rules(&args)?,
        "decision" => cmd_decision(&args)?,
        "tui" => return parse_tui_request(&args),
        "web" if args == ["--help"] => print_web_help(),
        "web" => return Ok(StartupRequest::Web(parse_web_args(&args)?)),
        "version" | "--version" => print_version(),
        "help" | "--help" => print_help(),
        other => {
            return Err(CliError::usage(format!(
                "unknown command `{other}`. Supported commands: init, upgrade, list, show, add, move, update, complete, cancel, search, log, accord, rules, decision, tui, web, version"
            )))
        }
    }

    Ok(StartupRequest::Exit)
}

fn print_help() {
    println!("tandem - Tandem CLI");
    println!();
    println!("Usage:");
    println!("  tandem init [--title <title>]");
    println!("  tandem upgrade");
    println!("  tandem list [--state <state>] [--type <type>] [--parent <id>] [--json]");
    println!("  tandem show <id> [--json]");
    println!("  tandem add --title <title> [--state <state>] [--kind epic] [--parent <id>] [--description <text>] [--priority <priority>] [--effort <effort>] [--json]");
    println!("  tandem move <id> --state <state>");
    println!("  tandem update <id> [--title <title>] [--body <markdown>] [--kind epic] [--parent <id>] [--priority <priority>] [--effort <effort>] ...");
    println!("  tandem complete <id> --summary <text>");
    println!("  tandem cancel <id> --reason <text>");
    println!("  tandem search <query> [--state <state>] [--type <type>] [--parent <id>] [--json]");
    println!("  tandem log list|show|search ...");
    println!("  tandem accord {} ...", accord_actions_help());
    println!("  tandem rules list|add|edit|delete ...");
    println!("  tandem decision list|show|add ... [--status <status>] [--date <date>]");
    println!("  tandem tui");
    println!("  tandem web [--port <port>] [--no-open]");
    println!("  tandem version");
    println!("  tandem --version");
}

fn print_web_help() {
    println!("tandem web - Open the local read-only web interface");
    println!();
    println!("Usage:");
    println!("  tandem web [--port <port>] [--no-open]");
    println!();
    println!("Options:");
    println!("  --port <port>  Use a specific loopback port (1-65535)");
    println!("  --no-open      Print the URL without opening a browser");
    println!("  --help         Show this help");
}

fn print_version() {
    println!("{}", version_text());
}

fn version_text() -> String {
    format!("tandem {PACKAGE_VERSION}")
}

fn parse_tui_request(args: &[String]) -> Result<StartupRequest, CliError> {
    if !args.is_empty() {
        return Err(CliError::usage("tui does not accept arguments"));
    }
    Ok(StartupRequest::Tui)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::output::*;
    use super::*;
    use crate::app::decisions::AddOptions as DecisionAddOptions;
    use crate::app::queries::SearchResult;
    use crate::app::tasks::{
        add as add_task, cancel as cancel_task, display_change_field,
        move_to_state as move_task_to_state, update as update_task_metadata,
        validate_update_options,
    };
    use crate::app::tasks::{AddOptions, UpdateOptions};
    use crate::project::rules::{empty_rules, parse_rules_from_yaml};
    use crate::project::{
        parse_frontmatter_fields as parse_raw_frontmatter_fields, patch_frontmatter_content,
        split_frontmatter, ProjectHierarchy as HierarchyIndex, StoredDocument as Document,
        TandemProject,
    };
    use crate::project::{read_document, read_documents};
    use crate::protocol::accord;
    use crate::protocol::document::parse_field_values;
    use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship, TaskRole};
    use crate::protocol::review::status as review_status;
    use crate::protocol::workflow::state_matches_filter;
    use crate::protocol::workflow::{
        self, completion_files_changed, completion_outcome, completion_reviewer,
        completion_summary, completion_validation, COMPLETION_OUTCOME_CANCELED,
        COMPLETION_OUTCOME_COMPLETED,
    };
    use crate::{app, project, protocol};

    fn parse_frontmatter_fields(
        frontmatter: &str,
    ) -> Result<std::collections::HashMap<String, String>, String> {
        let mut fields = parse_raw_frontmatter_fields(frontmatter)?;
        protocol::document::normalize_fields(&mut fields);
        Ok(fields)
    }

    fn hierarchy_from_workspace(project: &TandemProject) -> Result<HierarchyIndex, CliError> {
        app::support::hierarchy_from_project(project)
    }

    fn find_hierarchy_children(
        hierarchy: &HierarchyIndex,
        parent: &Document,
    ) -> Result<Vec<Document>, CliError> {
        app::queries::children_for(hierarchy, parent)
    }

    fn relationships_for(
        hierarchy: &HierarchyIndex,
        docs: &[Document],
    ) -> BTreeMap<String, Option<ParentRelationship>> {
        docs.iter()
            .map(|doc| (doc.id().to_string(), hierarchy.relationship(doc).unwrap()))
            .collect()
    }

    fn search_documents(docs: Vec<Document>, options: &SearchOptions) -> Vec<SearchResult> {
        app::queries::search_documents(
            docs,
            &app::queries::SearchFilter {
                query: &options.query,
                state: options.state.as_deref(),
                doc_type: options.doc_type.as_deref(),
                parent: options.parent.as_deref(),
            },
        )
    }

    fn filter_documents(docs: Vec<Document>, options: &ListOptions) -> Vec<Document> {
        app::queries::filter_documents(
            docs,
            &app::queries::ListFilter {
                state: options.state.as_deref(),
                doc_type: options.doc_type.as_deref(),
                priority: options.priority.as_deref(),
                tag: options.tag.as_deref(),
                assignee: options.assignee.as_deref(),
                parent: options.parent.as_deref(),
                accord: options.accord.as_deref(),
                review: options.review.as_deref(),
            },
        )
    }

    fn test_workspace(root: &Path) -> TandemProject {
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        workspace
    }

    fn canonical_event_content(workspace: &TandemProject) -> String {
        let mut paths = fs::read_dir(workspace.events_dir())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        paths.sort();
        paths
            .into_iter()
            .map(|path| fs::read_to_string(path).unwrap())
            .collect()
    }

    #[test]
    fn cli_version_uses_cargo_package_version() {
        assert_eq!(
            version_text(),
            format!("tandem {}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn web_help_does_not_require_a_workspace() {
        assert!(matches!(
            dispatch(vec!["web".into(), "--help".into()]).unwrap(),
            StartupRequest::Exit
        ));
    }

    #[test]
    fn accord_usage_lists_only_current_actions() {
        assert_eq!(
            accord::ACTIONS,
            ["claim", "deliver", "accept", "rework", "block", "fail"]
        );
        assert_eq!(
            accord_actions_help(),
            "claim|deliver|accept|rework|block|fail"
        );

        let bare_error = cmd_accord(&[]).unwrap_err();
        assert_eq!(bare_error.code, 2);
        assert_eq!(
            bare_error.message,
            "tandem accord requires claim, deliver, accept, rework, block, or fail"
        );
        assert!(!bare_error.message.contains("ready"));

        let retired_error = cmd_accord(&["ready".to_string()]).unwrap_err();
        assert_eq!(retired_error.code, 2);
        assert!(retired_error
            .message
            .contains("unknown accord subcommand `ready`"));
        assert!(!retired_error.message.contains("use ready"));
    }

    #[test]
    fn derives_workspace_title_from_directory_basename() {
        assert_eq!(
            app::project::default_title(Path::new("/tmp/Exact Project.Name")),
            "Exact Project.Name"
        );
        assert_eq!(
            app::project::default_title(Path::new("/tmp/  spaced  ")),
            "  spaced  "
        );
        assert_eq!(
            app::project::default_title(Path::new("/")),
            "Tandem Workspace"
        );
    }

    #[test]
    fn concurrent_task_adds_allocate_unique_ids_without_overwrite() {
        let root = std::env::temp_dir().join(format!(
            "tandem-concurrent-add-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        fs::write(&workspace.events_path, "").unwrap();

        let thread_count = 8;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(thread_count));
        let handles = (0..thread_count)
            .map(|index| {
                let workspace = workspace.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    add_task(
                        &workspace,
                        AddOptions {
                            title: Some(format!("Concurrent task {index}")),
                            ..AddOptions::default()
                        },
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();

        let outcomes = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        let mut ids = outcomes
            .iter()
            .map(|outcome| outcome.id.clone())
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| id.strip_prefix("task-").unwrap().parse::<usize>().unwrap());
        assert_eq!(
            ids,
            (1..=thread_count)
                .map(|number| format!("task-{number}"))
                .collect::<Vec<_>>()
        );

        let docs = read_documents(&workspace.board_dir, DocumentLocation::Board).unwrap();
        assert_eq!(docs.len(), thread_count);
        for outcome in outcomes {
            let content = fs::read_to_string(outcome.path).unwrap();
            assert!(content.contains(&format!("id: {}", outcome.id)));
            assert!(content.contains(&outcome.title));
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_relationships_show_collections_and_task_delegation_metadata() {
        let root = std::env::temp_dir().join(format!(
            "tandem-canonical-relationships-{}-{}",
            std::process::id(),
            current_timestamp()
        ));
        let workspace = test_workspace(&root);
        fs::write(
            workspace.board_dir.join("decision-1.md"),
            "---\nid: decision-1\ntype: decision\ntitle: Parent decision\nstatus: accepted\n---\n",
        )
        .unwrap();
        let epic = add_task(
            &workspace,
            AddOptions {
                title: Some("Epic".to_string()),
                kind: Some("epic".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let task = add_task(
            &workspace,
            AddOptions {
                title: Some("Task of epic".to_string()),
                parent: Some(epic.id.clone()),
                assignee: Some("worker-a".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let subtask = add_task(
            &workspace,
            AddOptions {
                title: Some("Task checklist item".to_string()),
                parent: Some(task.id.clone()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let generic = add_task(
            &workspace,
            AddOptions {
                title: Some("Decision-parented Task".to_string()),
                parent: Some("decision-1".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let generic_subtask = add_task(
            &workspace,
            AddOptions {
                title: Some("Generic Task checklist item".to_string()),
                parent: Some(generic.id.clone()),
                ..AddOptions::default()
            },
        )
        .unwrap();

        assert_eq!(epic.id, "task-1");
        assert_eq!(task.id, "task-2");
        assert_eq!(subtask.id, "task-2-1");
        assert_eq!(generic.id, "task-3");
        assert_eq!(generic_subtask.id, "task-3-1");
        assert_eq!(task.parent_relationship, Some(ParentRelationship::EpicTask));
        assert_eq!(
            subtask.parent_relationship,
            Some(ParentRelationship::Subtask)
        );
        assert_eq!(
            generic.parent_relationship,
            Some(ParentRelationship::Parent)
        );

        let hierarchy = hierarchy_from_workspace(&workspace).unwrap();
        hierarchy.validate_all_task_hierarchies().unwrap();
        let task_doc = hierarchy.document(&task.id).unwrap();
        assert_eq!(hierarchy.task_role(task_doc).unwrap(), Some(TaskRole::Task));
        assert_eq!(task_doc.field("assignee"), Some("worker-a"));
        let epic_doc = hierarchy.document(&epic.id).unwrap();
        let epic_children = find_hierarchy_children(&hierarchy, epic_doc).unwrap();
        let task_children = find_hierarchy_children(&hierarchy, task_doc).unwrap();
        assert!(
            show_json(epic_doc, &epic_children, Some(TaskRole::Epic), None)
                .contains("\"tasks\":[{\"id\":\"task-2\"")
        );
        assert!(show_json(
            task_doc,
            &task_children,
            Some(TaskRole::Task),
            Some(ParentRelationship::EpicTask)
        )
        .contains("\"subtasks\":[{\"id\":\"task-2-1\""));
        let docs = hierarchy.documents.values().cloned().collect::<Vec<_>>();
        assert!(list_json(&docs, &relationships_for(&hierarchy, &docs))
            .unwrap()
            .contains("\"parentRelationship\":\"epic-task\""));
        let filtered = filter_documents(
            docs.clone(),
            &ListOptions {
                parent: Some(epic.id.clone()),
                ..ListOptions::default()
            },
        );
        assert_eq!(
            filtered.iter().map(|doc| doc.id()).collect::<Vec<_>>(),
            ["task-2"]
        );
        let results = search_documents(
            docs,
            &SearchOptions {
                query: "Task of epic".to_string(),
                parent: Some(epic.id.clone()),
                ..SearchOptions::default()
            },
        );
        assert!(search_json(
            "Task of epic",
            &results,
            &relationships_for(
                &hierarchy,
                &results
                    .iter()
                    .map(|result| result.doc.clone())
                    .collect::<Vec<_>>()
            )
        )
        .unwrap()
        .contains("\"parentRelationship\":\"epic-task\""));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_allocation_scans_logs_for_global_and_subtask_sequences() {
        let root = std::env::temp_dir().join(format!(
            "tandem-canonical-allocation-{}-{}",
            std::process::id(),
            current_timestamp()
        ));
        let workspace = test_workspace(&root);
        fs::write(
            workspace.logs_dir.join("task-103.md"),
            "---\nid: task-103\ntype: task\ntitle: Logged Task\ncompletedAt: now\ncompletion:\n  summary: done\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.logs_dir.join("task-103-1.md"),
            "---\nid: task-103-1\ntype: task\ntitle: Logged Subtask\nparentId: task-103\ncompletedAt: now\ncompletion:\n  summary: done\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("decision-1.md"),
            "---\nid: decision-1\ntype: decision\ntitle: Decision\nstatus: accepted\n---\n",
        )
        .unwrap();

        let second_subtask = add_task(
            &workspace,
            AddOptions {
                title: Some("Second Subtask".to_string()),
                parent: Some("task-103".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let generic_task = add_task(
            &workspace,
            AddOptions {
                title: Some("Generic-parent Task".to_string()),
                parent: Some("decision-1".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        assert_eq!(second_subtask.id, "task-103-2");
        assert_eq!(generic_task.id, "task-104");
        assert_eq!(
            second_subtask.parent_relationship,
            Some(ParentRelationship::Subtask)
        );
        let hierarchy = hierarchy_from_workspace(&workspace).unwrap();
        hierarchy.validate_all_task_hierarchies().unwrap();
        let logged_parent = hierarchy.document("task-103").unwrap();
        assert_eq!(
            hierarchy.task_role(logged_parent).unwrap(),
            Some(TaskRole::Task)
        );
        let logged_child = hierarchy.document("task-103-1").unwrap();
        assert!(
            log_show_json(logged_child, hierarchy.relationship(logged_child).unwrap())
                .contains("\"parentRelationship\":\"subtask\"")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolved_graph_rejects_duplicates_unresolved_parents_cycles_and_invalid_depth() {
        let make_doc = |path: &str, frontmatter: &str| {
            Document::new(
                PathBuf::from(path),
                DocumentLocation::Board,
                parse_frontmatter_fields(frontmatter).unwrap(),
                String::new(),
            )
        };

        let duplicate = HierarchyIndex::from_documents(vec![
            make_doc("a.md", "id: task-1\ntype: task\ntitle: A\nstate: todo\n"),
            make_doc("b.md", "id: task-1\ntype: task\ntitle: B\nstate: todo\n"),
        ])
        .unwrap_err();
        assert!(duplicate.message.contains("duplicate document ID `task-1`"));

        let unsupported_kind = HierarchyIndex::from_documents(vec![make_doc(
            "task-1.md",
            "id: task-1\ntype: task\nkind: Epic\ntitle: Wrong kind casing\nstate: todo\n",
        )])
        .unwrap();
        let unsupported_doc = unsupported_kind.document("task-1").unwrap();
        assert!(unsupported_kind
            .task_role(unsupported_doc)
            .unwrap_err()
            .message
            .contains("invalid kind `Epic`"));
        let error = unsupported_kind
            .validate_all_task_hierarchies()
            .unwrap_err();
        assert!(error.message.contains("invalid kind `Epic`"));
        assert!(error.message.contains("expected one of: epic"));

        let aggregate = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: Task\nstate: todo\n",
            ),
            make_doc(
                "task-2.md",
                "id: task-2\ntype: task\ntitle: First global child\nstate: todo\nparentId: task-1\n",
            ),
            make_doc(
                "task-3.md",
                "id: task-3\ntype: task\ntitle: Second global child\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        let error = aggregate.validate_all_task_hierarchies().unwrap_err();
        assert!(error
            .message
            .contains("hierarchy contains 2 structural errors"));
        assert!(error.message.contains("task-2"));
        assert!(error.message.contains("task-3"));

        let unresolved = HierarchyIndex::from_documents(vec![make_doc(
            "task-1.md",
            "id: task-1\ntype: task\ntitle: Missing parent\nstate: todo\nparentId: task-9\n",
        )])
        .unwrap();
        assert!(unresolved
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("unresolved parentId `task-9`"));

        let cycle = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: A\nstate: todo\nparentId: task-2\n",
            ),
            make_doc(
                "task-2.md",
                "id: task-2\ntype: task\ntitle: B\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        assert!(cycle
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("task hierarchy cycle"));

        let direct_epic_hierarchical = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\nkind: epic\ntitle: Epic\nstate: todo\n",
            ),
            make_doc(
                "task-1-1.md",
                "id: task-1-1\ntype: task\ntitle: Wrong Task ID\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        let error = direct_epic_hierarchical
            .validate_all_task_hierarchies()
            .unwrap_err();
        assert!(error.message.contains("expected global `task-N`"));

        let parented_epic = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: Task\nstate: todo\n",
            ),
            make_doc(
                "task-2.md",
                "id: task-2\ntype: task\nkind: epic\ntitle: Nested Epic\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        assert!(parented_epic
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("Epic task-2 cannot have parentId"));

        let global_subtask = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: Task\nstate: todo\n",
            ),
            make_doc(
                "task-2.md",
                "id: task-2\ntype: task\ntitle: Wrong Subtask ID\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        assert!(global_subtask
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("expected `task-1-M`"));

        let child_beneath_subtask = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: Task\nstate: todo\n",
            ),
            make_doc(
                "task-1-1.md",
                "id: task-1-1\ntype: task\ntitle: Subtask\nstate: todo\nparentId: task-1\n",
            ),
            make_doc(
                "task-1-1-1.md",
                "id: task-1-1-1\ntype: task\ntitle: Invalid depth\nstate: todo\nparentId: task-1-1\n",
            ),
        ])
        .unwrap();
        let error = child_beneath_subtask
            .validate_all_task_hierarchies()
            .unwrap_err();
        assert!(
            error
                .message
                .contains("Subtask task-1-1 cannot have children")
                || error.message.contains("child of Subtask task-1-1")
        );
    }

    #[test]
    fn prospective_updates_reject_role_changes_id_mismatches_and_invalid_descendants() {
        let root = std::env::temp_dir().join(format!(
            "tandem-prospective-hierarchy-{}-{}",
            std::process::id(),
            current_timestamp()
        ));
        let workspace = test_workspace(&root);
        for (name, content) in [
            (
                "task-1.md",
                "---\nid: task-1\ntype: task\nkind: epic\ntitle: Epic one\nstate: todo\n---\n",
            ),
            (
                "task-2.md",
                "---\nid: task-2\ntype: task\ntitle: Task\nstate: todo\nparentId: task-1\n---\n",
            ),
            (
                "task-2-1.md",
                "---\nid: task-2-1\ntype: task\ntitle: Subtask\nstate: todo\nparentId: task-2\n---\n",
            ),
            (
                "task-3.md",
                "---\nid: task-3\ntype: task\nkind: epic\ntitle: Epic two\nstate: todo\n---\n",
            ),
            (
                "task-4.md",
                "---\nid: task-4\ntype: task\ntitle: Other Task\nstate: todo\nparentId: task-3\n---\n",
            ),
            (
                "task-5.md",
                "---\nid: task-5\ntype: task\ntitle: Root with child\nstate: todo\n---\n",
            ),
            (
                "task-5-1.md",
                "---\nid: task-5-1\ntype: task\ntitle: Child\nstate: todo\nparentId: task-5\n---\n",
            ),
        ] {
            fs::write(workspace.board_dir.join(name), content).unwrap();
        }

        let valid = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-2".to_string(),
                parent: Some("task-3".to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert_eq!(
            valid.parent_relationship,
            Some(ParentRelationship::EpicTask)
        );

        let mismatch = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-2-1".to_string(),
                parent: Some("task-4".to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap_err();
        assert!(mismatch.message.contains("expected `task-4-M`"));
        assert_eq!(
            read_document(
                &workspace.board_dir.join("task-2-1.md"),
                DocumentLocation::Board
            )
            .unwrap()
            .field("parentId"),
            Some("task-2")
        );

        let descendant = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-5".to_string(),
                kind: Some("epic".to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap_err();
        assert!(
            descendant.message.contains("task-5-1")
                && descendant.message.contains("expected global `task-N`")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn concurrent_subtask_adds_allocate_unique_parent_derived_ids() {
        let root = std::env::temp_dir().join(format!(
            "tandem-concurrent-subtask-add-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        fs::write(
            workspace.board_dir.join("task-103.md"),
            "---\nid: task-103\ntype: task\ntitle: Parent\nstate: todo\n---\n",
        )
        .unwrap();

        let thread_count = 8;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(thread_count));
        let handles = (0..thread_count)
            .map(|index| {
                let workspace = workspace.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    add_task(
                        &workspace,
                        AddOptions {
                            title: Some(format!("Concurrent child {index}")),
                            parent: Some("task-103".to_string()),
                            ..AddOptions::default()
                        },
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        let mut ids = handles
            .into_iter()
            .map(|handle| handle.join().unwrap().id)
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| {
            id.strip_prefix("task-103-")
                .unwrap()
                .parse::<usize>()
                .unwrap()
        });
        assert_eq!(
            ids,
            (1..=thread_count)
                .map(|number| format!("task-103-{number}"))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            read_documents(&workspace.board_dir, DocumentLocation::Board)
                .unwrap()
                .len(),
            thread_count + 1
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn hierarchy_change_labels_follow_resolved_parent_type() {
        assert_eq!(
            display_change_field("parentId", Some(ParentRelationship::EpicTask)),
            "Task of Epic"
        );
        assert_eq!(
            display_change_field("parentId", Some(ParentRelationship::Subtask)),
            "Subtask of"
        );
        assert_eq!(
            display_change_field("parentId", Some(ParentRelationship::Parent)),
            "Parent"
        );
        assert_eq!(display_change_field("priority", None), "priority");
    }

    #[test]
    fn inline_subtask_authoring_flags_point_to_parent_linked_tasks() {
        let add_error = parse_add_args(&[
            "--title".to_string(),
            "Parent".to_string(),
            "--subtask".to_string(),
            "Checklist".to_string(),
        ])
        .unwrap_err();
        assert!(add_error.message.contains("add --subtask is deprecated"));
        assert!(add_error.message.contains("--parent <task-id>"));

        let update_error = parse_update_args(&[
            "task-1".to_string(),
            "--subtask".to_string(),
            "Checklist".to_string(),
        ])
        .unwrap_err();
        assert!(update_error
            .message
            .contains("update --subtask is deprecated"));
    }

    #[test]
    fn update_body_parser_accepts_empty_and_flag_looking_markdown() {
        let leading_dash = parse_update_args(&[
            "task-1".to_string(),
            "--body".to_string(),
            "- first item\n\nBody".to_string(),
        ])
        .unwrap();
        assert_eq!(leading_dash.body.as_deref(), Some("- first item\n\nBody"));

        let empty = parse_update_args(&["task-1".to_string(), "--body".to_string(), String::new()])
            .unwrap();
        assert_eq!(empty.body.as_deref(), Some(""));
    }

    #[test]
    fn cancel_parser_accepts_reason_and_requires_an_id() {
        let parsed = parse_cancel_args(&[
            "task-1".to_string(),
            "--reason".to_string(),
            "- no longer needed".to_string(),
        ])
        .unwrap();
        assert_eq!(parsed.id, "task-1");
        assert_eq!(parsed.reason.as_deref(), Some("- no longer needed"));
        assert!(parse_cancel_args(&["--reason".to_string(), "why".to_string()]).is_err());
    }

    #[test]
    fn parses_yaml_frontmatter_and_preserves_body() {
        let input = "---\nid: task-1\ntitle: \"Hello\"\nstate: todo\n---\n\nBody\n";
        let (frontmatter, body) = split_frontmatter(input).unwrap();
        let fields = parse_frontmatter_fields(&frontmatter).unwrap();
        assert_eq!(fields.get("id").map(String::as_str), Some("task-1"));
        assert_eq!(fields.get("title").map(String::as_str), Some("Hello"));
        assert_eq!(fields.get("state").map(String::as_str), Some("todo"));
        assert_eq!(body, "\nBody\n");
    }

    #[test]
    fn parses_nested_accord_and_review_statuses() {
        let frontmatter = r#"
id: task-1
accord:
  status: delivered
  assignee: pi
review:
  status: pending
tags: ["tui", "cli"]
"#;
        let fields = parse_frontmatter_fields(frontmatter).unwrap();
        assert_eq!(
            fields.get("accord.status").map(String::as_str),
            Some("delivered")
        );
        assert_eq!(
            fields.get("accordStatus").map(String::as_str),
            Some("delivered")
        );
        assert_eq!(
            fields.get("accord.assignee").map(String::as_str),
            Some("pi")
        );
        assert_eq!(
            fields.get("review.status").map(String::as_str),
            Some("pending")
        );
        assert_eq!(
            fields.get("reviewStatus").map(String::as_str),
            Some("pending")
        );
        assert_eq!(
            parse_field_values(fields.get("tags").unwrap()),
            vec!["tui", "cli"]
        );
    }

    #[test]
    fn parses_block_arrays_and_quoted_commas() {
        let frontmatter = r#"
tags:
  - "ui, polish"
  - cli
blockers: [task-1, "task-2"]
"#;
        let fields = parse_frontmatter_fields(frontmatter).unwrap();
        assert_eq!(
            parse_field_values(fields.get("tags").unwrap()),
            vec!["ui, polish", "cli"]
        );
        assert_eq!(
            parse_field_values(fields.get("blockers").unwrap()),
            vec!["task-1", "task-2"]
        );
    }

    #[test]
    fn parses_structured_rules_with_sources() {
        let root = project::parse_frontmatter_yaml(
            r#"
rules:
  always:
    - id: 3
      rule: "Run tests"
      source: decision-1
  prefer:
    - "Keep changes small"
"#,
        )
        .unwrap();
        let rules = parse_rules_from_yaml(root.as_ref());
        assert_eq!(rules["always"][0].id, 3);
        assert_eq!(rules["always"][0].rule, "Run tests");
        assert_eq!(rules["always"][0].source.as_deref(), Some("decision-1"));
        assert_eq!(rules["prefer"][0].id, 1);
        assert_eq!(rules["prefer"][0].rule, "Keep changes small");
    }

    #[test]
    fn patches_rules_category_without_touching_other_categories_or_body() {
        let input = "---\ntitle: Demo\nrules:\n  always: []\n  never:\n    - id: 9\n      rule: \"Keep me\"\nstate: ignored\n---\n\n# Body\n";
        let mut rules = empty_rules();
        rules
            .get_mut("always")
            .unwrap()
            .push(protocol::config::RuleItem {
                id: 1,
                rule: "Run tests".to_string(),
                source: Some("decision-1".to_string()),
            });
        let output = project::rules::patch_rules_category_content(input, "always", &rules).unwrap();
        assert!(output.contains("rules:\n  always:\n    - id: 1\n"));
        assert!(output.contains("      source: \"decision-1\"\n"));
        assert!(output.contains("  never:\n    - id: 9\n      rule: \"Keep me\"\n"));
        assert!(output.contains("state: ignored\n"));
        assert!(output.ends_with("\n# Body\n"));
    }

    #[test]
    fn divergence_warning_reports_sync_candidate_without_collapsing_state() {
        let doc = Document::new(PathBuf::from("task-1.md"), DocumentLocation::Board, parse_frontmatter_fields(
                "id: task-1\ntype: task\ntitle: Demo\nstate: in-progress\naccord:\n  status: delivered\nreview:\n  status: pending\n",
            )
            .unwrap(), String::new());

        let warning = accord::state_divergence_warning(&doc).unwrap();
        assert!(warning.contains("workflow state `in-progress`"));
        assert!(warning.contains("accord.status `delivered` suggests `validation`"));
        assert_eq!(doc.field("state"), Some("in-progress"));
        assert_eq!(review_status(&doc), Some("pending"));
    }

    #[test]
    fn show_and_list_json_include_divergence_warnings() {
        let doc = Document::new(
            PathBuf::from("task-1.md"),
            DocumentLocation::Board,
            parse_frontmatter_fields(
                "id: task-1\ntype: task\ntitle: Demo\nstate: todo\naccord:\n  status: claimed\n",
            )
            .unwrap(),
            String::new(),
        );

        let hierarchy = HierarchyIndex::from_documents(vec![doc.clone()]).unwrap();
        assert!(show_json(&doc, &[], Some(TaskRole::Task), None)
            .contains("accord.status `claimed` suggests `in-progress`"));
        assert!(list_json(
            std::slice::from_ref(&doc),
            &relationships_for(&hierarchy, std::slice::from_ref(&doc))
        )
        .unwrap()
        .contains("accord.status `claimed` suggests `in-progress`"));
    }

    #[test]
    fn move_task_to_state_reuses_ready_to_claimed_sync() {
        let root = std::env::temp_dir().join(format!(
            "tandem-move-sync-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        let task_path = workspace.board_dir.join("task-1.md");
        fs::write(
            &task_path,
            "---\nid: task-1\ntype: task\ntitle: Demo\nstate: todo\naccord:\n  status: ready\n---\n\nBody\n",
        )
        .unwrap();

        let outcome = move_task_to_state(&workspace, "task-1", "in-progress").unwrap();
        let output = fs::read_to_string(&task_path).unwrap();
        assert!(outcome.changed);
        assert_eq!(outcome.accord_sync.as_deref(), Some("ready -> claimed"));
        assert!(output.contains("state: \"in-progress\"\n"));
        assert!(output.contains("accord:\n  status: \"claimed\"\n"));
        assert!(output.contains("Body\n"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completion_helpers_read_nested_and_legacy_flat_metadata() {
        let nested = Document::new(PathBuf::from("task-1.md"), DocumentLocation::Logs, parse_frontmatter_fields(
                "completion:\n  summary: Done\n  validation: passed\n  reviewer: Algorant\n  filesChanged: [src/main.rs]\n",
            )
            .unwrap(), String::new());
        assert_eq!(completion_summary(&nested), Some("Done"));
        assert_eq!(completion_validation(&nested), Some("passed"));
        assert_eq!(completion_reviewer(&nested), Some("Algorant"));
        assert_eq!(completion_files_changed(&nested), vec!["src/main.rs"]);

        let legacy = Document::new(PathBuf::from("task-2.md"), DocumentLocation::Logs, parse_frontmatter_fields(
                "completionSummary: Done\ncompletionValidation: passed\ncompletionReviewer: Algorant\nfilesChanged: [src/lib.rs]\n",
            )
            .unwrap(), String::new());
        assert_eq!(completion_summary(&legacy), Some("Done"));
        assert_eq!(completion_validation(&legacy), Some("passed"));
        assert_eq!(completion_reviewer(&legacy), Some("Algorant"));
        assert_eq!(completion_files_changed(&legacy), vec!["src/lib.rs"]);
    }

    #[test]
    fn validation_reports_invalid_review_status() {
        let doc = Document::new(
            PathBuf::from(".tandem/board/task-1.md"),
            DocumentLocation::Logs,
            parse_frontmatter_fields(
                "id: task-1\ntype: task\ntitle: Demo\nstate: todo\nreview:\n  status: maybe\n",
            )
            .unwrap(),
            String::new(),
        );
        let messages = protocol::diagnostic::metadata_diagnostics(&doc, true)
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect::<Vec<_>>();
        assert!(messages
            .iter()
            .any(|message| message.contains("invalid review.status `maybe`")));
    }

    #[test]
    fn validation_reports_invalid_task_kind() {
        let doc = Document::new(
            PathBuf::from(".tandem/logs/task-1.md"),
            DocumentLocation::Logs,
            parse_frontmatter_fields(
                "id: task-1\ntype: task\nkind: feature\ntitle: Demo\nstate: todo\n",
            )
            .unwrap(),
            String::new(),
        );
        let error = protocol::document::validate_task_kind(doc.field("kind").unwrap()).unwrap_err();
        assert!(error.contains("invalid kind `feature`"));
    }

    #[test]
    fn update_task_metadata_changes_scalars_and_appends_lists() {
        let root = std::env::temp_dir().join(format!(
            "tandem-update-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        fs::write(
            workspace.board_dir.join("task-2.md"),
            "---\nid: task-2\ntype: task\nkind: epic\ntitle: Blocker Epic\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("decision-1.md"),
            "---\nid: decision-1\ntype: decision\ntitle: Parent decision\nstatus: accepted\n---\n",
        )
        .unwrap();
        let task_path = workspace.board_dir.join("task-1.md");
        fs::write(
            &task_path,
            "---\nid: task-1\ntype: task\ntitle: Old\nstate: todo\npriority: low\ntags: [cli]\ncustom: keep\ncreatedAt: \"2026-06-26T00:00:00Z\"\nupdatedAt: \"2026-06-26T00:00:00Z\"\n---\n\nBody\n",
        )
        .unwrap();

        let outcome = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                title: Some("New".to_string()),
                priority: Some("high".to_string()),
                parent: Some("decision-1".to_string()),
                tags: vec!["cli".to_string(), "metadata".to_string()],
                blockers: vec!["task-2".to_string()],
                references: vec!["missing-decision".to_string()],
                related_files: vec!["src/main.rs".to_string()],
                ..UpdateOptions::default()
            },
        )
        .unwrap();

        let output = fs::read_to_string(&task_path).unwrap();
        assert_eq!(outcome.changes.len(), 7);
        assert_eq!(
            outcome.parent_relationship,
            Some(ParentRelationship::Parent)
        );
        assert_eq!(
            outcome.warnings,
            vec!["reference not found: missing-decision"]
        );
        assert!(output.contains("title: \"New\"\n"));
        assert!(output.contains("priority: \"high\"\n"));
        assert!(output.contains("parentId: \"decision-1\"\n"));
        assert!(output.contains("tags: [\"cli\", \"metadata\"]\n"));
        assert!(output.contains("blockers: [\"task-2\"]\n"));
        assert!(output.contains("references: [\"missing-decision\"]\n"));
        assert!(output.contains("relatedFiles: [\"src/main.rs\"]\n"));
        assert!(output.contains("custom: keep\n"));
        assert!(output.ends_with("\nBody\n"));
        assert!(canonical_event_content(&workspace).contains("task.updated"));

        let epic_parent_outcome = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                parent: Some("task-2".to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert_eq!(epic_parent_outcome.changes.len(), 1);
        assert_eq!(
            epic_parent_outcome.parent_relationship,
            Some(ParentRelationship::EpicTask)
        );
        assert!(fs::read_to_string(&task_path)
            .unwrap()
            .contains("parentId: \"task-2\"\n"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn update_task_metadata_noops_existing_list_entries_without_touching_file() {
        let root = std::env::temp_dir().join(format!(
            "tandem-update-noop-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(&workspace.config_path, "---\nstates: [todo]\n---\n").unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        let task_path = workspace.board_dir.join("task-1.md");
        let before = "---\nid: task-1\ntype: task\ntitle: Demo\nstate: todo\ntags: [cli]\nupdatedAt: \"old\"\n---\n\nBody\n";
        fs::write(&task_path, before).unwrap();

        let outcome = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                tags: vec!["cli".to_string()],
                ..UpdateOptions::default()
            },
        )
        .unwrap();

        assert!(outcome.changes.is_empty());
        assert_eq!(fs::read_to_string(&task_path).unwrap(), before);
        assert_eq!(fs::read_to_string(&workspace.events_path).unwrap(), "");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn update_task_body_replaces_clears_and_noops_exactly() {
        let root = std::env::temp_dir().join(format!(
            "tandem-update-body-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(&workspace.config_path, "---\nstates: [todo]\n---\n").unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        let task_path = workspace.board_dir.join("task-1.md");
        fs::write(
            &task_path,
            "---\nid: task-1\ntype: task\ntitle: Demo\nstate: todo\ncustom: keep\nupdatedAt: \"old\"\n---\n\nOld body\n",
        )
        .unwrap();

        let replacement = "- first item\n\nUnicode: café 🦀\n";
        let changed = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                body: Some(replacement.to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert_eq!(changed.changes.len(), 1);
        assert_eq!(changed.changes[0].field, "body");
        let after_change = fs::read_to_string(&task_path).unwrap();
        assert!(after_change.contains("custom: keep\n"));
        assert_eq!(split_frontmatter(&after_change).unwrap().1, replacement);
        let events_after_change = canonical_event_content(&workspace);
        assert!(events_after_change.contains("task.updated"));
        assert!(!events_after_change.contains("Unicode"));
        assert!(!events_after_change.contains("first item"));

        let noop = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                body: Some(replacement.to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert!(noop.changes.is_empty());
        assert_eq!(fs::read_to_string(&task_path).unwrap(), after_change);
        assert_eq!(canonical_event_content(&workspace), events_after_change);

        let cleared = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                body: Some(String::new()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert_eq!(cleared.changes.len(), 1);
        let after_clear = fs::read_to_string(&task_path).unwrap();
        assert!(after_clear.contains("custom: keep\n"));
        assert_eq!(split_frontmatter(&after_clear).unwrap().1, "");
        assert_eq!(
            canonical_event_content(&workspace)
                .matches("task.updated")
                .count(),
            2
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cancel_task_archives_auditable_outcome_and_rejects_active_descendants() {
        let root = std::env::temp_dir().join(format!(
            "tandem-cancel-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        fs::write(
            workspace.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\ntitle: Parent\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-2.md"),
            "---\nid: task-2\ntype: task\ntitle: Blocker\nstate: in-progress\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-3.md"),
            "---\nid: task-3\ntype: task\ntitle: Dependent\nstate: todo\nblockers: [task-2]\n---\n",
        )
        .unwrap();
        let child_path = workspace.board_dir.join("task-1-1.md");
        fs::write(
            &child_path,
            "---\nid: task-1-1\ntype: task\ntitle: Child\nstate: validation\nparentId: task-1\nblockers: [task-2]\nreview:\n  status: pending\naccord:\n  status: delivered\ncustom: keep\n---\n\nCanceled body\n",
        )
        .unwrap();

        let parent_error = cancel_task(&workspace, "task-1", "Parent canceled").unwrap_err();
        assert!(parent_error
            .message
            .contains("active descendants: task-1-1"));
        assert!(workspace.board_dir.join("task-1.md").exists());

        let child = cancel_task(&workspace, "task-1-1", "Created by mistake").unwrap();
        assert_eq!(child.id, "task-1-1");
        assert!(!child_path.exists());
        let canceled_content = fs::read_to_string(&child.log_path).unwrap();
        let canceled_doc = read_document(&child.log_path, DocumentLocation::Logs).unwrap();
        assert_eq!(
            completion_outcome(&canceled_doc),
            COMPLETION_OUTCOME_CANCELED
        );
        assert_eq!(
            completion_summary(&canceled_doc),
            Some("Canceled: Created by mistake")
        );
        assert!(canceled_doc.field("state").is_none());
        assert!(canceled_doc.field("completedAt").is_some());
        assert!(canceled_doc.field("updatedAt").is_some());
        assert!(canceled_content.contains("custom: keep\n"));
        assert_eq!(canceled_doc.body, "\nCanceled body\n");
        assert!(canceled_content.contains("review:\n  status: pending\n"));
        assert!(canceled_content.contains("accord:\n  status: delivered\n"));
        assert!(log_summary_json(&canceled_doc).contains("\"outcome\":\"canceled\""));
        assert!(document_detail_json(&canceled_doc).contains("\"completionOutcome\":\"canceled\""));
        assert!(canonical_event_content(&workspace).contains("task.canceled"));

        let next_child = add_task(
            &workspace,
            AddOptions {
                title: Some("Fresh child".to_string()),
                parent: Some("task-1".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        assert_eq!(next_child.id, "task-1-2");

        cancel_task(&workspace, "task-2", "Dependency intentionally waived").unwrap();
        let hierarchy = hierarchy_from_workspace(&workspace).unwrap();
        assert!(
            app::support::unresolved_blockers_in_hierarchy(&hierarchy, Some("[task-2]")).is_empty()
        );

        let legacy_completed = Document::new(PathBuf::from(".tandem/logs/task-99.md"), DocumentLocation::Logs, parse_frontmatter_fields(
                "id: task-99\ntype: task\ntitle: Legacy\ncompletedAt: now\ncompletion:\n  summary: Done\n",
            )
            .unwrap(), String::new());
        assert_eq!(
            completion_outcome(&legacy_completed),
            COMPLETION_OUTCOME_COMPLETED
        );

        fs::write(
            workspace.board_dir.join("task-4.md"),
            "---\nid: task-4\ntype: task\ntitle: Duplicate board\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.logs_dir.join("task-4.md"),
            "---\nid: task-4\ntype: task\ntitle: Duplicate log\ncompletedAt: now\ncompletion:\n  summary: Existing\n---\n",
        )
        .unwrap();
        let duplicate_error = cancel_task(&workspace, "task-4", "Should fail").unwrap_err();
        assert!(duplicate_error
            .message
            .contains("duplicate document ID `task-4`"));
        assert!(workspace.board_dir.join("task-4.md").exists());
        assert!(workspace.logs_dir.join("task-4.md").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn show_json_includes_parent_id_only_when_present() {
        let child = Document::new(
            PathBuf::from("task-1-1.md"),
            DocumentLocation::Board,
            parse_frontmatter_fields(
                "id: task-1-1\ntype: task\ntitle: Child\nstate: todo\nparentId: task-1\n",
            )
            .unwrap(),
            String::new(),
        );
        let parent = Document::new(
            PathBuf::from("task-1.md"),
            DocumentLocation::Board,
            parse_frontmatter_fields("id: task-1\ntype: task\ntitle: Parent\nstate: todo\n")
                .unwrap(),
            String::new(),
        );

        let parent_json = show_json(
            &parent,
            std::slice::from_ref(&child),
            Some(TaskRole::Task),
            None,
        );
        assert!(show_json(
            &child,
            &[],
            Some(TaskRole::Subtask),
            Some(ParentRelationship::Subtask)
        )
        .contains("\"parentId\":\"task-1\""));
        assert!(parent_json.contains("\"subtasks\":[{\"id\":\"task-1-1\""));
        assert!(!document_detail_json(&parent).contains("\"parentId\""));
    }

    #[test]
    fn update_rejects_invalid_priority_and_kind_while_json_exposes_metadata() {
        let doc = Document::new(PathBuf::from("task-1.md"), DocumentLocation::Board, parse_frontmatter_fields(
                "id: task-1\ntype: task\nkind: epic\ntitle: Demo\nstate: todo\npriority: high\nblockers: [task-2]\nreferences: [decision-1]\nrelatedFiles: [src/main.rs]\n",
            )
            .unwrap(), String::new());
        assert!(document_summary_json(&doc, None).contains("\"kind\":\"epic\""));
        assert!(document_detail_json(&doc).contains("\"kind\":\"epic\""));
        let hierarchy = HierarchyIndex::from_documents(vec![doc.clone()]).unwrap();
        let search = search_json(
            "epic",
            &[SearchResult {
                doc: doc.clone(),
                snippet: "epic".to_string(),
            }],
            &relationships_for(&hierarchy, std::slice::from_ref(&doc)),
        )
        .unwrap();
        assert!(search.contains("\"kind\":\"epic\""));
        assert!(document_detail_json(&doc).contains("\"blockers\":[\"task-2\"]"));
        assert!(document_detail_json(&doc).contains("\"references\":[\"decision-1\"]"));
        assert!(document_detail_json(&doc).contains("\"relatedFiles\":[\"src/main.rs\"]"));

        let error = validate_update_options(
            &UpdateOptions {
                id: "task-1".to_string(),
                priority: Some("urgent".to_string()),
                ..UpdateOptions::default()
            },
            &hierarchy,
        )
        .unwrap_err();
        assert!(error.message.contains("invalid priority `urgent`"));

        let error = validate_update_options(
            &UpdateOptions {
                id: "task-1".to_string(),
                kind: Some("feature".to_string()),
                ..UpdateOptions::default()
            },
            &hierarchy,
        )
        .unwrap_err();
        assert!(error.message.contains("invalid kind `feature`"));

        let error = validate_update_options(
            &UpdateOptions {
                id: "task-1".to_string(),
                parent: Some("task-1".to_string()),
                ..UpdateOptions::default()
            },
            &hierarchy,
        )
        .unwrap_err();
        assert!(error.message.contains("cannot be its own parent"));
    }

    #[test]
    fn decision_metadata_is_status_not_workflow_state() {
        let doc = Document::new(PathBuf::from("decision-1.md"), DocumentLocation::Board, parse_frontmatter_fields(
                "id: decision-1\ntype: decision\ntitle: Choose cache\nstatus: accepted\ndate: 2026-07-01\ndeciders: [Algorant, pi]\ncontext: Need a cache policy\nconsequences: [Faster reads]\nalternatives: [No cache]\nsupersedes: [decision-0]\nsupersededBy: [decision-2]\n",
            )
            .unwrap(), "## Decision\nUse the small cache.\n".to_string());

        let detail = document_detail_json(&doc);
        assert!(detail.contains("\"status\":\"accepted\""));
        assert!(detail.contains("\"date\":\"2026-07-01\""));
        assert!(detail.contains("\"deciders\":[\"Algorant\",\"pi\"]"));
        assert!(detail.contains("\"supersededBy\":[\"decision-2\"]"));
        assert!(!detail.contains("\"state\""));

        let filtered = filter_documents(
            vec![doc],
            &ListOptions {
                state: Some("accepted".to_string()),
                ..ListOptions::default()
            },
        );
        assert!(filtered.is_empty());
    }

    #[test]
    fn decision_status_validation_rejects_workflow_states() {
        assert!(app::decisions::validate_status("accepted").is_ok());
        let error = app::decisions::validate_status("todo").unwrap_err();
        assert!(error.message.contains("invalid decision status `todo`"));
        assert!(error.message.contains("proposed, accepted, rejected"));
    }

    #[test]
    fn decision_add_reference_and_supersession_diagnostics_are_warnings() {
        let root = std::env::temp_dir().join(format!(
            "tandem-decision-warnings-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(&workspace.config_path, "---\nstates: [todo]\n---\n").unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        fs::write(
            workspace.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\ntitle: Task\nstate: todo\n---\n",
        )
        .unwrap();

        let warnings = app::decisions::diagnostics(
            &workspace,
            &DecisionAddOptions {
                references: vec!["missing-ref".to_string()],
                supersedes: vec!["task-1".to_string()],
                superseded_by: vec!["missing-decision".to_string()],
                ..DecisionAddOptions::default()
            },
        )
        .unwrap();

        assert_eq!(
            warnings,
            vec![
                "reference not found: missing-ref".to_string(),
                "supersedes target task-1 is type task, not decision".to_string(),
                "supersededBy decision not found: missing-decision".to_string(),
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn date_from_timestamp_uses_utc_calendar_date() {
        assert_eq!(
            app::support::date_from_timestamp("2026-07-01T18:05:47Z"),
            "2026-07-01"
        );
    }

    #[test]
    fn escapes_json_strings() {
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
    }

    #[test]
    fn patches_frontmatter_without_touching_body() {
        let input = "---\nid: task-1\nstate: todo\ntitle: Old\n---\n\nBody\n";
        let mut updates = BTreeMap::new();
        updates.insert("state".to_string(), "validation".to_string());
        updates.insert("updatedAt".to_string(), "2026-06-26T00:00:00Z".to_string());
        let output = patch_frontmatter_content(input, &updates, &[]).unwrap();
        assert!(output.contains("state: \"validation\"\n"));
        assert!(output.contains("updatedAt: \"2026-06-26T00:00:00Z\"\n"));
        assert!(output.ends_with("\nBody\n"));
    }

    #[test]
    fn validation_state_filter_accepts_legacy_review_alias() {
        assert!(state_matches_filter(Some("validation"), "validation"));
        assert!(state_matches_filter(Some("review"), "validation"));
        assert!(state_matches_filter(Some("validation"), "review"));
        assert!(!state_matches_filter(Some("todo"), "validation"));
    }

    #[test]
    fn configured_review_state_accepts_preferred_validation_writes() {
        let legacy_states = vec![
            "todo".to_string(),
            "in-progress".to_string(),
            "review".to_string(),
        ];
        assert!(workflow::is_known_or_legacy_state(
            &legacy_states,
            "validation"
        ));
        assert!(
            workflow::display_known_states(&legacy_states).contains("validation (preferred alias)")
        );

        let current_states = vec![
            "todo".to_string(),
            "in-progress".to_string(),
            "validation".to_string(),
        ];
        assert!(workflow::is_known_or_legacy_state(
            &current_states,
            "review"
        ));
        assert!(workflow::display_known_states(&current_states).contains("review (legacy alias)"));
    }

    #[test]
    fn parses_inline_arrays_for_filters() {
        let values = parse_field_values("[\"tui\", \"cli\"]");
        assert_eq!(values, vec!["tui".to_string(), "cli".to_string()]);
    }

    #[test]
    fn formats_unix_epoch_as_utc() {
        assert_eq!(format_unix_seconds(0), "1970-01-01T00:00:00Z");
    }
}
