use std::collections::BTreeMap;
use std::env;
use std::path::Path;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app;
use crate::app::queries::{PapercutSearchResult, SearchResult};
use crate::app::tasks::AddOutcome;
use crate::project::{StoredDocument as Document, StoredPapercut};
use crate::protocol::accord;
use crate::protocol::accord::status as accord_status;
use crate::protocol::config::RulesByCategory;
use crate::protocol::document::parse_field_values;
use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship, TaskRole};
use crate::protocol::review::status as review_status;
use crate::protocol::workflow::{
    completion_files_changed, completion_outcome, completion_reviewer, completion_summary,
    completion_validation, COMPLETION_OUTCOME_CANCELED,
};
use crate::CliError;

type Relationships = BTreeMap<String, Option<ParentRelationship>>;

pub(crate) fn is_canceled_log(doc: &Document) -> bool {
    doc.location == DocumentLocation::Logs && completion_outcome(doc) == COMPLETION_OUTCOME_CANCELED
}

pub(super) fn document_warnings(doc: &Document) -> Vec<String> {
    let mut warnings = accord::state_divergence_warning(doc)
        .into_iter()
        .collect::<Vec<_>>();
    if !doc.is_first_class_type() {
        warnings.push(format!(
            "{} is legacy custom type `{}`; custom-type documents are deprecated and read-only.",
            doc.id(),
            doc.doc_type()
        ));
    }
    warnings
}

pub(super) fn decision_status(doc: &Document) -> Option<&str> {
    doc.field("status")
}

pub(super) fn decision_date(doc: &Document) -> Option<&str> {
    doc.field("date")
}

pub(super) fn decision_context(doc: &Document) -> Option<&str> {
    doc.field("context")
}

pub(super) fn decision_deciders(doc: &Document) -> Vec<String> {
    decision_values(doc, "deciders")
}

pub(super) fn decision_consequences(doc: &Document) -> Vec<String> {
    decision_values(doc, "consequences")
}

pub(super) fn decision_alternatives(doc: &Document) -> Vec<String> {
    decision_values(doc, "alternatives")
}

pub(super) fn decision_supersedes(doc: &Document) -> Vec<String> {
    decision_values(doc, "supersedes")
}

pub(super) fn decision_superseded_by(doc: &Document) -> Vec<String> {
    decision_values(doc, "supersededBy")
}

pub(super) fn decision_values(doc: &Document, key: &str) -> Vec<String> {
    doc.values(key)
}

pub(super) fn print_accord_update(
    id: &str,
    previous_status: &str,
    status: &str,
    event_name: &str,
    path: &Path,
) {
    println!("Updated accord");
    println!("ID:     {id}");
    println!("From:   {previous_status}");
    println!("To:     {status}");
    println!("Path:   {}", display_path(path));
    println!("Event:  {event_name}");
}

pub(crate) fn sort_documents(docs: &mut [Document]) {
    docs.sort_by(|a, b| {
        a.field("state")
            .unwrap_or("")
            .cmp(b.field("state").unwrap_or(""))
            .then_with(|| a.id().cmp(b.id()))
    });
}

pub(super) fn parent_table_values<'a>(
    doc: &'a Document,
    relationships: &Relationships,
) -> Result<(&'static str, &'a str), CliError> {
    let Some(parent_id) = doc.field("parentId") else {
        return Ok(("-", "-"));
    };
    let relationship = relationships
        .get(doc.id())
        .copied()
        .flatten()
        .unwrap_or(ParentRelationship::Parent)
        .as_str();
    Ok((relationship, parent_id))
}

pub(super) fn print_list_table(
    docs: &[Document],
    relationships: &Relationships,
) -> Result<(), CliError> {
    if docs.is_empty() {
        println!("No active Tandem documents found.");
        return Ok(());
    }

    println!(
        "{:<12} {:<12} {:<8} {:<8} {:<9} {:<12} {:<32} {:<12}",
        "ID", "STATE", "TYPE", "KIND", "RELATION", "PARENT", "TITLE", "ASSIGNEE"
    );
    for doc in docs {
        let (relationship, parent_id) = parent_table_values(doc, relationships)?;
        println!(
            "{:<12} {:<12} {:<8} {:<8} {:<9} {:<12} {:<32} {:<12}",
            truncate(doc.id(), 12),
            truncate(doc.field("state").unwrap_or("-"), 12),
            truncate(doc.doc_type(), 8),
            truncate(doc.kind().unwrap_or("-"), 8),
            relationship,
            truncate(parent_id, 12),
            truncate(doc.title(), 32),
            truncate(doc.field("assignee").unwrap_or("-"), 12)
        );
    }
    Ok(())
}

pub(super) fn print_document_warnings(docs: &[Document]) {
    for warning in docs.iter().flat_map(document_warnings) {
        println!("Warning: {warning}");
    }
}

pub(super) fn print_decision_metadata(doc: &Document) {
    if let Some(status) = decision_status(doc) {
        println!("Status:    {status}");
    }
    if let Some(date) = decision_date(doc) {
        println!("Date:      {date}");
    }
    print_metadata_values("Deciders", decision_deciders(doc));
    if let Some(context) = decision_context(doc) {
        println!("Context:   {context}");
    }
    print_metadata_values("Consequences", decision_consequences(doc));
    print_metadata_values("Alternatives", decision_alternatives(doc));
    print_metadata_values("Supersedes", decision_supersedes(doc));
    print_metadata_values("Superseded by", decision_superseded_by(doc));
    print_metadata_values(
        "References",
        doc.field("references")
            .map(parse_field_values)
            .unwrap_or_default(),
    );
    print_metadata_values(
        "Tags",
        doc.field("tags")
            .map(parse_field_values)
            .unwrap_or_default(),
    );
}

pub(super) fn print_metadata_values(label: &str, values: Vec<String>) {
    if !values.is_empty() {
        println!("{label}: {}", values.join(", "));
    }
}

pub(super) fn print_show(
    doc: &Document,
    children: &[Document],
    role: Option<TaskRole>,
    relationship: Option<ParentRelationship>,
) {
    println!("ID:        {}", doc.id());
    println!("Type:      {}", doc.doc_type());
    if let Some(kind) = doc.kind() {
        println!("Kind:      {kind}");
    }
    println!("Title:     {}", doc.title());
    if doc.doc_type() == "decision" {
        print_decision_metadata(doc);
    }
    if let Some(state) = doc.field("state") {
        println!("State:     {state}");
    }
    if let Some(priority) = doc.field("priority") {
        println!("Priority:  {priority}");
    }
    if let Some(assignee) = doc.field("assignee") {
        println!("Assignee:  {assignee}");
    }
    if let Some(due_date) = doc.field("dueDate") {
        println!("Due:       {due_date}");
    }
    if let Some(parent_id) = doc.field("parentId") {
        let label = relationship
            .unwrap_or(ParentRelationship::Parent)
            .human_label();
        println!("{label}: {parent_id}");
    }
    if !children.is_empty() {
        let label = if role == Some(TaskRole::Epic) {
            "Tasks"
        } else {
            "Subtasks"
        };
        println!("{label}:   {}", children.len());
        for child in children {
            let status = child
                .field("state")
                .or_else(|| {
                    (child.location == DocumentLocation::Logs).then(|| completion_outcome(child))
                })
                .unwrap_or(child.location.as_str());
            println!("  {} [{}] {}", child.id(), status, child.title());
        }
    }
    if let Some(created_at) = doc.field("createdAt") {
        println!("Created:   {created_at}");
    }
    if let Some(updated_at) = doc.field("updatedAt") {
        println!("Updated:   {updated_at}");
    }
    if let Some(completed_at) = doc.field("completedAt") {
        if is_canceled_log(doc) {
            println!("Canceled:  {completed_at}");
        } else {
            println!("Completed: {completed_at}");
        }
    }
    if doc.location == DocumentLocation::Logs && doc.doc_type() == "task" {
        println!("Outcome:   {}", completion_outcome(doc));
    }
    if let Some(status) = accord_status(doc) {
        println!("Accord:    {status}");
    }
    if let Some(status) = review_status(doc) {
        println!("Review:    {status}");
    }
    for warning in document_warnings(doc) {
        println!("Warning:   {warning}");
    }
    if let Some(summary) = completion_summary(doc) {
        println!("Summary:   {summary}");
    }
    println!("Location:  {}", doc.location.as_str());
    println!("Path:      {}", display_path(&doc.path));
    println!();
    println!("Body:");
    if doc.body.trim().is_empty() {
        println!("(empty)");
    } else {
        print!("{}", doc.body);
        if !doc.body.ends_with('\n') {
            println!();
        }
    }
}

pub(super) fn print_search_table(
    results: &[SearchResult],
    relationships: &Relationships,
) -> Result<(), CliError> {
    if results.is_empty() {
        println!("No matching Tandem documents found.");
        return Ok(());
    }
    println!(
        "{:<12} {:<8} {:<12} {:<8} {:<8} {:<9} {:<12} {:<24} MATCH",
        "ID", "WHERE", "STATE", "TYPE", "KIND", "RELATION", "PARENT", "TITLE"
    );
    for result in results {
        let doc = &result.doc;
        let (relationship, parent_id) = parent_table_values(doc, relationships)?;
        println!(
            "{:<12} {:<8} {:<12} {:<8} {:<8} {:<9} {:<12} {:<24} {}",
            truncate(doc.id(), 12),
            doc.location.as_str(),
            truncate(
                doc.field("state")
                    .or_else(|| {
                        (doc.location == DocumentLocation::Logs).then(|| completion_outcome(doc))
                    })
                    .unwrap_or("-"),
                12,
            ),
            truncate(doc.doc_type(), 8),
            truncate(doc.kind().unwrap_or("-"), 8),
            relationship,
            truncate(parent_id, 12),
            truncate(doc.title(), 24),
            truncate(&result.snippet, 80)
        );
    }
    Ok(())
}

pub(super) fn print_papercut_list(items: &[StoredPapercut]) {
    if items.is_empty() {
        println!("No matching Papercuts found.");
        return;
    }
    println!(
        "{:<14} {:<10} {:<40} {:<20} TAGS",
        "ID", "STATUS", "TITLE", "UPDATED"
    );
    for item in items {
        println!(
            "{:<14} {:<10} {:<40} {:<20} {}",
            truncate(item.id(), 14),
            truncate(item.status(), 10),
            truncate(item.title(), 40),
            truncate(item.field("updatedAt").unwrap_or("-"), 20),
            truncate(&item.values("tags").join(", "), 40)
        );
    }
}

pub(super) fn print_papercut_show(item: &StoredPapercut) {
    println!("ID:        {}", item.id());
    println!("Title:     {}", item.title());
    println!("Status:    {}", item.status());
    if let Some(created) = item.field("createdAt") {
        println!("Created:   {created}");
    }
    if let Some(updated) = item.field("updatedAt") {
        println!("Updated:   {updated}");
    }
    print_metadata_values("References", item.values("references"));
    print_metadata_values("Tags", item.values("tags"));
    if let Some(note) = item.field("resolution.note") {
        println!("Resolution: {note}");
    }
    if let Some(at) = item.field("resolution.resolvedAt") {
        println!("Resolved:  {at}");
    }
    println!("Location:  papercuts");
    println!("Path:      {}", display_path(&item.path));
    println!("\nBody:");
    if item.body.trim().is_empty() {
        println!("(empty)");
    } else {
        print!("{}", item.body);
        if !item.body.ends_with('\n') {
            println!();
        }
    }
}

pub(super) fn print_papercut_search(results: &[PapercutSearchResult]) {
    if results.is_empty() {
        return;
    }
    println!(
        "{:<14} {:<10} {:<10} {:<36} MATCH",
        "ID", "WHERE", "STATUS", "TITLE"
    );
    for result in results {
        println!(
            "{:<14} {:<10} {:<10} {:<36} {}",
            truncate(result.papercut.id(), 14),
            "papercuts",
            truncate(result.papercut.status(), 10),
            truncate(result.papercut.title(), 36),
            truncate(&result.snippet, 80)
        );
    }
}

pub(super) fn print_log_table(docs: &[Document]) {
    if docs.is_empty() {
        println!("No archived Tandem logs found.");
        return;
    }
    println!(
        "{:<12} {:<20} {:<10} {:<36} SUMMARY",
        "ID", "ARCHIVED", "OUTCOME", "TITLE"
    );
    for doc in docs {
        println!(
            "{:<12} {:<20} {:<10} {:<36} {}",
            truncate(doc.id(), 12),
            truncate(doc.field("completedAt").unwrap_or("-"), 20),
            truncate(completion_outcome(doc), 10),
            truncate(doc.title(), 36),
            truncate(completion_summary(doc).unwrap_or("-"), 80)
        );
    }
}

pub(super) fn print_log_show(doc: &Document, relationship: Option<ParentRelationship>) {
    println!("Log document");
    print_show(doc, &[], None, relationship);
    if let Some(validation) = completion_validation(doc) {
        println!();
        println!("Validation: {validation}");
    }
    let files = completion_files_changed(doc);
    if !files.is_empty() {
        println!("Files changed: {}", files.join(", "));
    }
    if let Some(reviewer) = completion_reviewer(doc) {
        println!("Reviewer: {reviewer}");
    }
}

pub(super) fn print_decision_table(docs: &[Document]) {
    if docs.is_empty() {
        println!("No Tandem decisions found.");
        return;
    }
    println!(
        "{:<14} {:<12} {:<10} {:<34} {:<20} SUMMARY",
        "ID", "STATUS", "DATE", "TITLE", "REFERENCES"
    );
    for doc in docs {
        println!(
            "{:<14} {:<12} {:<10} {:<34} {:<20} {}",
            truncate(doc.id(), 14),
            truncate(decision_status(doc).unwrap_or("-"), 12),
            truncate(decision_date(doc).unwrap_or("-"), 10),
            truncate(doc.title(), 34),
            truncate(doc.field("references").unwrap_or("-"), 20),
            truncate(&first_body_line(doc), 80)
        );
    }
}

pub(super) fn require_rule_category(category: Option<&str>) -> Result<&str, CliError> {
    let category =
        category.ok_or_else(|| CliError::usage("rules mutation requires --category <category>"))?;
    app::rules::validate_rule_category(category)?;
    Ok(category)
}

pub(super) fn print_rules(rules: &RulesByCategory, category_filter: Option<&str>) {
    let categories = ["always", "never", "prefer", "context"];
    let mut printed_any = false;
    for category in categories {
        if category_filter.is_some_and(|filter| filter != category) {
            continue;
        }
        println!("{category}:");
        let items = rules.get(category).map(Vec::as_slice).unwrap_or(&[]);
        if items.is_empty() {
            println!("  (none)");
        } else {
            printed_any = true;
            for item in items {
                match item.source.as_deref() {
                    Some(source) => println!("  {}. {} ({source})", item.id, item.rule),
                    None => println!("  {}. {}", item.id, item.rule),
                }
            }
        }
    }
    if !printed_any && category_filter.is_some() {
        // The category heading above is the intended empty-list output.
    }
}

pub(super) fn add_outcome_json(outcome: &AddOutcome) -> String {
    let mut fields = vec![
        format!("\"id\":{}", json_string(&outcome.id)),
        "\"type\":\"task\"".to_string(),
        format!("\"state\":{}", json_string(&outcome.state)),
        format!("\"title\":{}", json_string(&outcome.title)),
    ];
    if let Some(kind) = outcome.kind.as_deref() {
        fields.push(format!("\"kind\":{}", json_string(kind)));
    }
    if let Some(parent) = outcome.parent.as_deref() {
        fields.push(format!("\"parentId\":{}", json_string(parent)));
    }
    if let Some(relationship) = outcome.parent_relationship {
        fields.push(format!(
            "\"parentRelationship\":{}",
            json_string(relationship.as_str())
        ));
    }
    fields.push(format!(
        "\"path\":{}",
        json_string(&display_path(&outcome.path))
    ));
    format!(
        "{{\"ok\":true,\"data\":{{\"document\":{{{}}}}},\"warnings\":{}}}",
        fields.join(","),
        json_array_strings(&outcome.warnings)
    )
}

pub(super) fn list_json(
    docs: &[Document],
    relationships: &Relationships,
) -> Result<String, CliError> {
    let mut by_state = BTreeMap::<String, usize>::new();
    for doc in docs {
        let state = doc.field("state").unwrap_or("unknown").to_string();
        *by_state.entry(state).or_insert(0) += 1;
    }

    let items = docs
        .iter()
        .map(|doc| document_summary_json(doc, relationships.get(doc.id()).copied().flatten()))
        .collect::<Vec<_>>();
    let states = by_state
        .iter()
        .map(|(state, count)| format!("{}:{count}", json_string(state)))
        .collect::<Vec<_>>()
        .join(",");
    let warnings = docs.iter().flat_map(document_warnings).collect::<Vec<_>>();

    Ok(format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{}],\"counts\":{{\"total\":{},\"byState\":{{{}}}}}}},\"warnings\":{}}}",
        items.join(","),
        docs.len(),
        states,
        json_array_strings(&warnings)
    ))
}

pub(super) fn show_json(
    doc: &Document,
    children: &[Document],
    role: Option<TaskRole>,
    relationship: Option<ParentRelationship>,
) -> String {
    let warnings = document_warnings(doc);
    let mut data_fields = vec![format!("\"document\":{}", document_detail_json(doc))];
    if let Some(relationship) = relationship {
        data_fields.push(format!(
            "\"parentRelationship\":{}",
            json_string(relationship.as_str())
        ));
    }
    if matches!(role, Some(TaskRole::Epic | TaskRole::Task)) {
        let children = children
            .iter()
            .map(child_task_summary_json)
            .collect::<Vec<_>>()
            .join(",");
        let key = if role == Some(TaskRole::Epic) {
            "tasks"
        } else {
            "subtasks"
        };
        data_fields.push(format!("\"{key}\":[{children}]"));
    }
    data_fields.push(format!("\"body\":{}", json_string(&doc.body)));
    data_fields.push(format!(
        "\"path\":{}",
        json_string(&display_path(&doc.path))
    ));
    data_fields.push(format!(
        "\"location\":{}",
        json_string(doc.location.as_str())
    ));
    format!(
        "{{\"ok\":true,\"data\":{{{}}},\"warnings\":{}}}",
        data_fields.join(","),
        json_array_strings(&warnings)
    )
}

pub(super) fn log_list_json(docs: &[Document]) -> String {
    let items = docs.iter().map(log_summary_json).collect::<Vec<_>>();
    format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{}],\"count\":{}}},\"warnings\":[]}}",
        items.join(","),
        docs.len()
    )
}

pub(super) fn log_show_json(doc: &Document, relationship: Option<ParentRelationship>) -> String {
    let files = completion_files_changed(doc);
    let relationship_field = relationship
        .map(|relationship| {
            format!(
                ",\"parentRelationship\":{}",
                json_string(relationship.as_str())
            )
        })
        .unwrap_or_default();
    format!(
        "{{\"ok\":true,\"data\":{{\"document\":{}{},\"completion\":{{\"outcome\":{},\"summary\":{},\"filesChanged\":{},\"validation\":{},\"reviewer\":{}}},\"body\":{},\"path\":{}}},\"warnings\":[]}}",
        document_detail_json(doc),
        relationship_field,
        json_string(completion_outcome(doc)),
        json_string(completion_summary(doc).unwrap_or("")),
        json_array_strings(&files),
        json_string(completion_validation(doc).unwrap_or("")),
        json_string(completion_reviewer(doc).unwrap_or("")),
        json_string(&doc.body),
        json_string(&display_path(&doc.path))
    )
}

pub(super) fn search_json(
    query: &str,
    results: &[SearchResult],
    relationships: &Relationships,
) -> Result<String, CliError> {
    let items = results
        .iter()
        .map(|result| {
            let doc = &result.doc;
            let mut fields = Vec::new();
            push_json_field(&mut fields, "id", doc.id());
            push_json_field(&mut fields, "type", doc.doc_type());
            push_optional_json_field(&mut fields, "kind", doc.kind());
            push_json_field(&mut fields, "title", doc.title());
            push_json_field(&mut fields, "location", doc.location.as_str());
            push_optional_json_field(&mut fields, "state", doc.field("state"));
            push_optional_json_field(&mut fields, "completedAt", doc.field("completedAt"));
            if doc.location == DocumentLocation::Logs && doc.doc_type() == "task" {
                push_json_field(&mut fields, "completionOutcome", completion_outcome(doc));
            }
            push_optional_json_field(&mut fields, "parentId", doc.field("parentId"));
            push_parent_relationship_json_field(
                &mut fields,
                relationships.get(doc.id()).copied().flatten(),
            );
            push_json_field(&mut fields, "snippet", &result.snippet);
            Ok(format!("{{{}}}", fields.join(",")))
        })
        .collect::<Result<Vec<_>, CliError>>()?;
    Ok(format!(
        "{{\"ok\":true,\"data\":{{\"query\":{},\"results\":[{}]}},\"warnings\":[]}}",
        json_string(query),
        items.join(",")
    ))
}

pub(super) fn papercut_list_json(items: &[StoredPapercut], warnings: &[String]) -> String {
    let rendered = items
        .iter()
        .map(papercut_summary_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{rendered}],\"count\":{}}},\"warnings\":{}}}",
        items.len(),
        json_array_strings(warnings)
    )
}

pub(super) fn papercut_show_json(item: &StoredPapercut, warnings: &[String]) -> String {
    format!("{{\"ok\":true,\"data\":{{\"papercut\":{},\"body\":{},\"path\":{},\"location\":\"papercuts\"}},\"warnings\":{}}}",
        papercut_detail_json(item), json_string(&item.body), json_string(&display_path(&item.path)), json_array_strings(warnings))
}

pub(super) fn global_search_json(
    query: &str,
    documents: &[SearchResult],
    papercuts: &[PapercutSearchResult],
    relationships: &Relationships,
    warnings: &[String],
) -> Result<String, CliError> {
    let document_json = search_json(query, documents, relationships)?;
    let marker = "]},\"warnings\":[]}";
    let prefix = document_json
        .strip_suffix(marker)
        .ok_or_else(|| CliError::user("internal search JSON composition failure"))?;
    let extra = papercuts.iter().map(|result| format!("{{\"id\":{},\"title\":{},\"location\":\"papercuts\",\"status\":{},\"snippet\":{}}}",
        json_string(result.papercut.id()), json_string(result.papercut.title()), json_string(result.papercut.status()), json_string(&result.snippet))).collect::<Vec<_>>();
    let separator = if documents.is_empty() || extra.is_empty() {
        ""
    } else {
        ","
    };
    Ok(format!(
        "{prefix}{separator}{}]}},\"warnings\":{}}}",
        extra.join(","),
        json_array_strings(warnings)
    ))
}

fn papercut_summary_json(item: &StoredPapercut) -> String {
    let mut fields = Vec::new();
    for key in ["id", "title", "status", "createdAt", "updatedAt"] {
        push_optional_json_field(&mut fields, key, item.field(key));
    }
    for key in ["references", "tags"] {
        if item.field(key).is_some() {
            fields.push(format!(
                "{}:{}",
                json_string(key),
                json_array_strings(&item.values(key))
            ));
        }
    }
    format!("{{{}}}", fields.join(","))
}

fn papercut_detail_json(item: &StoredPapercut) -> String {
    let mut fields = Vec::new();
    let mut keys = item.fields.keys().collect::<Vec<_>>();
    keys.sort();
    for key in keys {
        if key == "references" || key == "tags" {
            fields.push(format!(
                "{}:{}",
                json_string(key),
                json_array_strings(&item.values(key))
            ));
        } else if key.starts_with("resolution.") {
            continue;
        } else {
            push_json_field(&mut fields, key, item.field(key).unwrap_or(""));
        }
    }
    if item.field("resolution.note").is_some() || item.field("resolution.resolvedAt").is_some() {
        fields.push(format!(
            "\"resolution\":{{\"note\":{},\"resolvedAt\":{}}}",
            json_string(item.field("resolution.note").unwrap_or("")),
            json_string(item.field("resolution.resolvedAt").unwrap_or(""))
        ));
    }
    format!("{{{}}}", fields.join(","))
}

pub(super) fn rules_json(rules: &RulesByCategory, category_filter: Option<&str>) -> String {
    let categories = ["always", "never", "prefer", "context"];
    let mut category_fields = Vec::new();
    let mut count_fields = Vec::new();
    let mut total = 0usize;
    for category in categories {
        let items = rules.get(category).map(Vec::as_slice).unwrap_or(&[]);
        let included_items = if category_filter.is_some_and(|filter| filter != category) {
            Vec::new()
        } else {
            items.to_vec()
        };
        total += included_items.len();
        let json_items = included_items
            .iter()
            .map(|item| {
                let mut fields = Vec::new();
                fields.push(format!("\"id\":{}", item.id));
                push_json_field(&mut fields, "rule", &item.rule);
                push_optional_json_field(&mut fields, "source", item.source.as_deref());
                format!("{{{}}}", fields.join(","))
            })
            .collect::<Vec<_>>();
        category_fields.push(format!(
            "{}:[{}]",
            json_string(category),
            json_items.join(",")
        ));
        count_fields.push(format!(
            "{}:{}",
            json_string(category),
            included_items.len()
        ));
    }
    count_fields.push(format!("\"total\":{total}"));
    format!(
        "{{\"ok\":true,\"data\":{{\"rules\":{{{}}},\"counts\":{{{}}}}},\"warnings\":[]}}",
        category_fields.join(","),
        count_fields.join(",")
    )
}

pub(super) fn decision_list_json(docs: &[Document]) -> String {
    let items = docs
        .iter()
        .map(|doc| {
            let references = doc
                .field("references")
                .map(parse_field_values)
                .unwrap_or_default();
            let mut fields = Vec::new();
            push_json_field(&mut fields, "id", doc.id());
            push_json_field(&mut fields, "type", doc.doc_type());
            push_json_field(&mut fields, "title", doc.title());
            push_decision_metadata_json_fields(&mut fields, doc);
            fields.push(format!(
                "\"references\":{}",
                json_array_strings(&references)
            ));
            if let Some(tags) = doc.field("tags") {
                fields.push(format!(
                    "\"tags\":{}",
                    json_array_strings(&parse_field_values(tags))
                ));
            }
            push_json_field(&mut fields, "summary", &first_body_line(doc));
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>();
    format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{}],\"count\":{}}},\"warnings\":[]}}",
        items.join(","),
        docs.len()
    )
}

pub(super) fn decision_show_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_json_field(&mut fields, "title", doc.title());
    push_decision_metadata_json_fields(&mut fields, doc);
    let references = doc
        .field("references")
        .map(parse_field_values)
        .unwrap_or_default();
    fields.push(format!(
        "\"references\":{}",
        json_array_strings(&references)
    ));
    if let Some(tags) = doc.field("tags") {
        fields.push(format!(
            "\"tags\":{}",
            json_array_strings(&parse_field_values(tags))
        ));
    }
    format!(
        "{{\"ok\":true,\"data\":{{\"decision\":{{{}}},\"body\":{},\"path\":{}}},\"warnings\":[]}}",
        fields.join(","),
        json_string(&doc.body),
        json_string(&display_path(&doc.path))
    )
}

pub(super) fn first_body_line(doc: &Document) -> String {
    doc.body
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_string()
}

pub(super) fn document_summary_json(
    doc: &Document,
    relationship: Option<ParentRelationship>,
) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_optional_json_field(&mut fields, "kind", doc.kind());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "state", doc.field("state"));
    push_optional_json_field(&mut fields, "priority", doc.field("priority"));
    push_optional_json_field(&mut fields, "effort", doc.field("effort"));
    push_optional_json_field(&mut fields, "assignee", doc.field("assignee"));
    push_optional_json_field(&mut fields, "parentId", doc.field("parentId"));
    push_parent_relationship_json_field(&mut fields, relationship);
    if let Some(tags) = doc.field("tags") {
        fields.push(format!(
            "\"tags\":{}",
            json_array_strings(&parse_field_values(tags))
        ));
    }
    if doc.doc_type() == "decision" {
        push_decision_metadata_json_fields(&mut fields, doc);
    }
    push_status_object_json(&mut fields, "accord", accord_status(doc));
    push_status_object_json(&mut fields, "review", review_status(doc));
    format!("{{{}}}", fields.join(","))
}

pub(super) fn document_detail_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_optional_json_field(&mut fields, "kind", doc.kind());
    push_json_field(&mut fields, "title", doc.title());
    for key in [
        "state",
        "priority",
        "effort",
        "assignee",
        "dueDate",
        "parentId",
        "createdAt",
        "updatedAt",
        "completedAt",
    ] {
        push_optional_json_field(&mut fields, key, doc.field(key));
    }
    push_optional_json_field(&mut fields, "completionSummary", completion_summary(doc));
    if doc.location == DocumentLocation::Logs && doc.doc_type() == "task" {
        push_json_field(&mut fields, "completionOutcome", completion_outcome(doc));
    }
    for key in ["tags", "blockers", "references", "relatedFiles"] {
        if let Some(value) = doc.field(key) {
            fields.push(format!(
                "{}:{}",
                json_string(key),
                json_array_strings(&parse_field_values(value))
            ));
        }
    }
    if doc.doc_type() == "decision" {
        push_decision_metadata_json_fields(&mut fields, doc);
    }
    push_status_object_json(&mut fields, "accord", accord_status(doc));
    push_status_object_json(&mut fields, "review", review_status(doc));
    format!("{{{}}}", fields.join(","))
}

pub(super) fn child_task_summary_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "state", doc.field("state"));
    push_optional_json_field(&mut fields, "completedAt", doc.field("completedAt"));
    if doc.location == DocumentLocation::Logs {
        push_json_field(&mut fields, "completionOutcome", completion_outcome(doc));
    }
    push_json_field(&mut fields, "location", doc.location.as_str());
    format!("{{{}}}", fields.join(","))
}

pub(super) fn log_summary_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_optional_json_field(&mut fields, "kind", doc.kind());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "completedAt", doc.field("completedAt"));
    push_json_field(&mut fields, "outcome", completion_outcome(doc));
    push_optional_json_field(&mut fields, "summary", completion_summary(doc));
    push_optional_json_field(&mut fields, "validationStatus", completion_validation(doc));
    format!("{{{}}}", fields.join(","))
}

pub(super) fn push_json_field(fields: &mut Vec<String>, key: &str, value: &str) {
    fields.push(format!("{}:{}", json_string(key), json_string(value)));
}

pub(super) fn push_parent_relationship_json_field(
    fields: &mut Vec<String>,
    relationship: Option<ParentRelationship>,
) {
    if let Some(relationship) = relationship {
        push_json_field(fields, "parentRelationship", relationship.as_str());
    }
}

pub(super) fn push_status_object_json(fields: &mut Vec<String>, key: &str, status: Option<&str>) {
    if let Some(status) = status {
        fields.push(format!(
            "{}:{{\"status\":{}}}",
            json_string(key),
            json_string(status)
        ));
    }
}

pub(super) fn push_optional_json_field(fields: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_json_field(fields, key, value);
    }
}

pub(super) fn push_optional_json_array_field(
    fields: &mut Vec<String>,
    key: &str,
    values: Vec<String>,
) {
    if !values.is_empty() {
        fields.push(format!(
            "{}:{}",
            json_string(key),
            json_array_strings(&values)
        ));
    }
}

pub(super) fn push_decision_metadata_json_fields(fields: &mut Vec<String>, doc: &Document) {
    push_optional_json_field(fields, "status", decision_status(doc));
    push_optional_json_field(fields, "date", decision_date(doc));
    push_optional_json_field(fields, "context", decision_context(doc));
    push_optional_json_array_field(fields, "deciders", decision_deciders(doc));
    push_optional_json_array_field(fields, "consequences", decision_consequences(doc));
    push_optional_json_array_field(fields, "alternatives", decision_alternatives(doc));
    push_optional_json_array_field(fields, "supersedes", decision_supersedes(doc));
    push_optional_json_array_field(fields, "supersededBy", decision_superseded_by(doc));
}

pub(super) fn json_array_strings(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(super) fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output.push('"');
    output
}

pub(super) fn require_nonempty<'a>(
    value: Option<&'a str>,
    message: &str,
) -> Result<&'a str, CliError> {
    let value = value.ok_or_else(|| CliError::usage(message))?.trim();
    if value.is_empty() {
        Err(CliError::usage(message))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
pub(super) fn current_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_unix_seconds(seconds)
}

#[cfg(test)]
pub(super) fn format_unix_seconds(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let hour = second_of_day / 3_600;
    let minute = (second_of_day % 3_600) / 60;
    let second = second_of_day % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
pub(super) fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

pub(crate) fn truncate(value: &str, max_chars: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars {
        return value.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }
    let mut truncated = chars[..max_chars.saturating_sub(1)]
        .iter()
        .collect::<String>();
    truncated.push('…');
    truncated
}

pub(crate) fn display_path(path: &Path) -> String {
    match env::current_dir() {
        Ok(current_dir) => path
            .strip_prefix(&current_dir)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| path.display().to_string()),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_string_preserves_exact_cli_escaping() {
        assert_eq!(
            json_string("quote \" slash \\ line\n\t\u{0001}"),
            "\"quote \\\" slash \\\\ line\\n\\t\\u0001\""
        );
    }

    #[test]
    fn global_search_json_combines_papercuts_and_warnings() {
        let papercut = StoredPapercut::new(
            "papercut-1.md".into(),
            std::collections::HashMap::from([
                ("id".to_string(), "papercut-1".to_string()),
                ("title".to_string(), "Friction".to_string()),
                ("status".to_string(), "open".to_string()),
                ("createdAt".to_string(), "now".to_string()),
                ("updatedAt".to_string(), "now".to_string()),
            ]),
            String::new(),
        );
        let output = global_search_json(
            "friction",
            &[],
            &[PapercutSearchResult {
                papercut,
                snippet: "Friction".to_string(),
            }],
            &BTreeMap::new(),
            &["reference warning".to_string()],
        )
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["data"]["results"][0]["location"], "papercuts");
        assert_eq!(parsed["warnings"][0], "reference warning");
    }
}
