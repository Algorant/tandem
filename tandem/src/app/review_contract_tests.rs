#[cfg(test)]
mod tests {
    use crate::app::review::{transition, ReviewOptions};
    use crate::project::TandemProject;
    use std::fs;

    #[test]
    fn escalation_persists_validation_evidence_without_review_status() {
        let root =
            std::env::temp_dir().join(format!("tandem-review-contract-{}", std::process::id()));
        let data = root.join(".tandem");
        let project = TandemProject {
            root: root.clone(),
            data_dir: data.clone(),
            tasks_dir: data.join("tasks"),
            logs_dir: data.join("logs"),
            config_path: data.join("tandem.md"),
            events_path: data.join("events.jsonl"),
        };
        fs::create_dir_all(&project.tasks_dir).unwrap();
        fs::create_dir_all(&project.logs_dir).unwrap();
        fs::create_dir_all(project.events_dir()).unwrap();
        fs::write(
            &project.config_path,
            "---\nprotocolVersion: 0.3.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        fs::write(project.tasks_dir.join("task-1.md"), "---\nid: task-1\ntype: task\ntitle: Review me\nstate: in-progress\naccord:\n  status: ready\n  acceptance: [Pass]\n---\n").unwrap();
        let result = transition(
            &project,
            "request",
            ReviewOptions {
                id: "task-1".into(),
                criterion: Some("Pass".into()),
                note: Some("Needs human confirmation".into()),
                reviewer: Some("human".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(result.state, "validation");
        let content = fs::read_to_string(project.tasks_dir.join("task-1.md")).unwrap();
        assert!(content.contains("validation.criterion: \"Pass\""));
        assert!(content.contains("validation.note:"));
        assert!(content.contains("validation.reviewer: \"human\""));
        assert!(!content.contains("validation.state"));
        let events = fs::read_to_string(
            project.actor_events_path(&crate::project::events::actor_id(&project).unwrap()),
        )
        .unwrap();
        assert!(events.contains("review.requested"));
        fs::remove_dir_all(root).unwrap();
    }
}
