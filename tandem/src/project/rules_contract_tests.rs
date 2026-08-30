#[cfg(test)]
mod tests {
    use crate::project::rules::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn per_file_rules_allocate_composite_ids_and_preserve_provenance() {
        let root = std::env::temp_dir().join(format!(
            "tandem-rules-contract-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let id = next_rule_id(&root, "always").unwrap();
        assert_eq!(id, "always-1");
        let rule = RuleRecord {
            id: id.clone(),
            category: "always".into(),
            source: Some("decision-1".into()),
            created_at: Some("now".into()),
            updated_at: Some("now".into()),
            text: "Run tests".into(),
            path: root.join("always-1.md"),
        };
        write_rule_file(&root, &rule).unwrap();
        let read = read_rule_file(&rule.path).unwrap();
        assert_eq!(read.id, "always-1");
        assert_eq!(read.source.as_deref(), Some("decision-1"));
        assert_eq!(next_rule_id(&root, "always").unwrap(), "always-2");
        fs::remove_dir_all(root).unwrap();
    }
}
