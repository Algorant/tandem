use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use crate::{
    accord_status, completion_files_changed, completion_reviewer, completion_summary,
    completion_validation, display_path, parse_field_values, read_document, review_status,
    Document, DocumentLocation, HierarchyIndex,
};

use super::{markdownish_lines, StatusTone, TuiTheme};

#[derive(Debug, Clone)]
pub(super) struct LogEvent {
    pub(super) ts: String,
    pub(super) event: String,
    pub(super) summary: String,
}

pub(super) type LogEventsById = BTreeMap<String, Vec<LogEvent>>;

#[derive(Debug, Clone)]
pub(super) struct LogLoad {
    pub(super) docs: Vec<Document>,
    pub(super) warnings: Vec<String>,
}

pub(super) fn load_logs(logs_dir: &Path) -> LogLoad {
    let mut docs = Vec::new();
    let mut warnings = Vec::new();

    if !logs_dir.exists() {
        return LogLoad { docs, warnings };
    }

    let entries = match fs::read_dir(logs_dir) {
        Ok(entries) => entries,
        Err(error) => {
            warnings.push(format!(
                "Logs load failed: could not read {}: {error}",
                display_path(logs_dir)
            ));
            return LogLoad { docs, warnings };
        }
    };

    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) == Some("md") {
                    paths.push(path);
                }
            }
            Err(error) => warnings.push(format!(
                "Logs load warning: could not inspect entry in {}: {error}",
                display_path(logs_dir)
            )),
        }
    }
    paths.sort();

    for path in paths {
        match read_document(&path, DocumentLocation::Logs) {
            Ok(doc) => docs.push(doc),
            Err(error) => warnings.push(format!("Logs load warning: {}", error.message)),
        }
    }

    sort_logs_by_recency(&mut docs);
    LogLoad { docs, warnings }
}

pub(super) fn sort_logs_by_recency(docs: &mut [Document]) {
    docs.sort_by(|a, b| {
        b.field("completedAt")
            .unwrap_or("")
            .cmp(a.field("completedAt").unwrap_or(""))
            .then_with(|| a.id().cmp(b.id()))
    });
}

pub(super) fn load_log_events(events_path: &Path) -> (LogEventsById, Vec<String>) {
    let mut events = LogEventsById::new();
    let mut warnings = Vec::new();

    if !events_path.exists() {
        return (events, warnings);
    }

    let content = match fs::read_to_string(events_path) {
        Ok(content) => content,
        Err(error) => {
            warnings.push(format!(
                "Events load warning: could not read {}: {error}",
                display_path(events_path)
            ));
            return (events, warnings);
        }
    };

    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let Some(id) = extract_json_string(line, "id") else {
            continue;
        };
        let event = extract_json_string(line, "event").unwrap_or_else(|| "event".to_string());
        let ts = extract_json_string(line, "ts").unwrap_or_default();
        let summary = extract_json_string(line, "summary").unwrap_or_default();
        events
            .entry(id)
            .or_default()
            .push(LogEvent { ts, event, summary });
    }

    for item_events in events.values_mut() {
        item_events.sort_by(|a, b| a.ts.cmp(&b.ts).then_with(|| a.event.cmp(&b.event)));
    }

    (events, warnings)
}

fn extract_json_string(line: &str, key: &str) -> Option<String> {
    let key_pattern = format!("\"{key}\"");
    let key_start = line.find(&key_pattern)?;
    let after_key = key_start + key_pattern.len();
    let colon_offset = line[after_key..].find(':')?;
    let mut cursor = after_key + colon_offset + 1;
    while let Some(ch) = line[cursor..].chars().next() {
        if ch.is_whitespace() {
            cursor += ch.len_utf8();
        } else {
            break;
        }
    }

    if line[cursor..].chars().next()? != '"' {
        return None;
    }
    cursor += 1;

    let mut value = String::new();
    let mut escaped = false;
    for ch in line[cursor..].chars() {
        if escaped {
            match ch {
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                other => value.push(other),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(value);
        } else {
            value.push(ch);
        }
    }

    None
}

pub(super) fn filter_logs<'a>(
    logs: &'a [Document],
    hierarchy: Option<&HierarchyIndex>,
    query: &str,
) -> Vec<&'a Document> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return logs.iter().collect();
    }

    logs.iter()
        .filter(|doc| log_matches_query(doc, hierarchy, &query))
        .collect()
}

fn log_matches_query(doc: &Document, hierarchy: Option<&HierarchyIndex>, query: &str) -> bool {
    let mut haystack = String::new();
    haystack.push_str(doc.id());
    haystack.push('\n');
    haystack.push_str(doc.title());
    haystack.push('\n');
    haystack.push_str(completion_summary(doc).unwrap_or(""));
    haystack.push('\n');
    haystack.push_str(completion_validation(doc).unwrap_or(""));
    haystack.push('\n');
    haystack.push_str(&completion_files_changed(doc).join("\n"));
    haystack.push('\n');
    haystack.push_str(&doc.body);
    if let Some(hierarchy) = hierarchy {
        let valid_task = doc.doc_type() != "task" || hierarchy.validate_task_hierarchy(doc).is_ok();
        if valid_task {
            if let Ok(Some(role)) = hierarchy.task_role(doc) {
                haystack.push('\n');
                haystack.push_str(role.as_str());
            }
        }
        if valid_task {
            if let Ok(Some(relationship)) = hierarchy.relationship(doc) {
                haystack.push('\n');
                haystack.push_str(relationship.as_str());
                haystack.push('\n');
                haystack.push_str(relationship.human_label());
            }
        }
        if let Some(parent_id) = doc.field("parentId") {
            haystack.push('\n');
            haystack.push_str(parent_id);
            if let Some(parent) = hierarchy.document(parent_id) {
                haystack.push('\n');
                haystack.push_str(parent.title());
            }
        }
    }
    haystack
        .to_ascii_lowercase()
        .contains(&query.to_ascii_lowercase())
}

pub(super) fn list_item_for_log(
    doc: &Document,
    hierarchy: Option<&HierarchyIndex>,
    theme: &TuiTheme,
    available_width: u16,
) -> ListItem<'static> {
    ListItem::new(line_for_log(doc, hierarchy, theme, available_width))
}

fn line_for_log(
    doc: &Document,
    hierarchy: Option<&HierarchyIndex>,
    theme: &TuiTheme,
    available_width: u16,
) -> Line<'static> {
    let title = log_row_title(doc);
    let role = hierarchy
        .and_then(|hierarchy| hierarchy.validate_task_hierarchy(doc).ok())
        .map(|role| format!("[{}] ", role.as_str().to_ascii_uppercase()))
        .unwrap_or_default();
    let prefix_width = doc.id().chars().count() + role.chars().count() + 2;
    let title_width = (available_width as usize).saturating_sub(prefix_width);

    Line::from(vec![
        Span::styled(
            format!("{}  ", doc.id()),
            theme.status_style(StatusTone::Accent),
        ),
        Span::styled(role, theme.muted_style()),
        Span::styled(truncate_for_log(&title, title_width), theme.text_style()),
    ])
}

fn log_row_title(doc: &Document) -> String {
    let title = doc.title().trim();
    if !title.is_empty() {
        return title.to_string();
    }
    let summary = completion_summary(doc).unwrap_or("").trim();
    if !summary.is_empty() {
        return summary.to_string();
    }
    doc.doc_type().to_string()
}

pub(super) fn completed_at_compact(value: &str) -> String {
    if value == "-" || value == "unknown" || value.trim().is_empty() {
        return value.to_string();
    }
    if value.contains('T') {
        return format!(
            "{} {}",
            completed_at_date_label(value),
            completed_at_time_label(value)
        );
    }
    value.to_string()
}

fn completed_at_date_label(value: &str) -> String {
    if value == "-" || value == "unknown" || value.trim().is_empty() {
        return "-----".to_string();
    }
    if let Some((date, _)) = value.split_once('T') {
        return date.get(5..).unwrap_or(date).to_string();
    }
    value.chars().take(5).collect()
}

fn completed_at_time_label(value: &str) -> String {
    if value == "-" || value == "unknown" || value.trim().is_empty() {
        return "--:--".to_string();
    }
    if let Some((_, time)) = value.split_once('T') {
        return time
            .get(0..5)
            .unwrap_or(time)
            .trim_end_matches('Z')
            .to_string();
    }
    value.to_string()
}

pub(super) fn detail_lines_for_log(
    doc: &Document,
    hierarchy: Option<&HierarchyIndex>,
    events: &[LogEvent],
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        doc.title().to_string(),
        theme.title_style(),
    )));

    let summary = completion_summary(doc);
    let validation = completion_validation(doc);
    let reviewer = completion_reviewer(doc);
    if summary.is_some() || validation.is_some() || reviewer.is_some() {
        lines.push(Line::from(""));
        lines.push(section_heading("Completion", theme));
        push_compact_optional(&mut lines, "summary", summary, theme);
        push_compact_optional(&mut lines, "validation", validation, theme);
        push_compact_optional(&mut lines, "reviewer", reviewer, theme);
    }

    let files = completion_files_changed(doc);
    lines.push(Line::from(""));
    lines.push(section_heading("Files changed", theme));
    if files.is_empty() {
        lines.push(Line::from(Span::styled(
            "(none recorded)",
            theme.muted_style(),
        )));
    } else {
        for file in files {
            lines.push(Line::from(vec![
                Span::styled("- ", theme.markdown_list_style()),
                Span::styled(file, theme.text_style()),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(section_heading("Log reference", theme));
    lines.push(Line::from(vec![
        Span::styled(doc.id().to_string(), theme.status_style(StatusTone::Accent)),
        Span::styled(" · ", theme.muted_style()),
        Span::styled(doc.doc_type().to_string(), theme.muted_style()),
        Span::styled(" · completed ", theme.muted_style()),
        Span::styled(
            completed_at_compact(doc.field("completedAt").unwrap_or("unknown")),
            theme.text_style(),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        format!("path {}", display_path(&doc.path)),
        theme.muted_style(),
    )));

    if let Some(hierarchy) = hierarchy {
        let validation = if doc.doc_type() == "task" {
            hierarchy.validate_task_hierarchy(doc).map(Some)
        } else {
            Ok(None)
        };
        let role = validation.as_ref().ok().copied().flatten();
        let relationship = hierarchy.relationship(doc).ok().flatten();
        if role.is_some() || relationship.is_some() || validation.is_err() {
            lines.push(Line::from(""));
            lines.push(section_heading("Hierarchy", theme));
            if let Some(role) = role {
                push_compact_optional(&mut lines, "role", Some(role.as_str()), theme);
            }
            if let Err(error) = validation {
                push_compact_optional(&mut lines, "Hierarchy error", Some(&error.message), theme);
            } else if let (Some(relationship), Some(parent_id)) =
                (relationship, doc.field("parentId"))
            {
                let parent = hierarchy
                    .document(parent_id)
                    .map(|parent| format!("{} ({parent_id})", parent.title()))
                    .unwrap_or_else(|| format!("missing parent {parent_id}"));
                push_compact_optional(&mut lines, relationship.human_label(), Some(&parent), theme);
            }
        }
    }

    let chips = compact_metadata(doc);
    if !chips.is_empty() {
        lines.push(Line::from(""));
        lines.push(section_heading("Process metadata", theme));
        lines.push(Line::from(Span::styled(
            chips.join(" · "),
            theme.muted_style(),
        )));
    }

    append_accord_lines(&mut lines, doc, theme);
    append_event_lines(&mut lines, events, theme);

    lines.push(Line::from(""));
    lines.push(section_heading("Body", theme));
    if doc.body.trim().is_empty() {
        lines.push(Line::from(Span::styled("(empty)", theme.muted_style())));
    } else {
        lines.extend(markdownish_lines(&doc.body, theme));
    }

    lines
}

fn section_heading(label: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(Span::styled(
        label.to_string(),
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    ))
}

fn push_compact_optional(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: Option<&str>,
    theme: &TuiTheme,
) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(Line::from(vec![
            Span::styled("• ", theme.markdown_list_style()),
            Span::styled(format!("{label}: "), theme.label_style()),
            Span::styled(value.to_string(), theme.text_style()),
        ]));
    }
}

fn compact_metadata(doc: &Document) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(value) = accord_status(doc).filter(|value| !value.trim().is_empty()) {
        items.push(format!("accord {value}"));
    }
    if let Some(value) = review_status(doc).filter(|value| !value.trim().is_empty()) {
        items.push(format!("review {value}"));
    }
    if let Some(value) = doc
        .field("priority")
        .filter(|value| !value.trim().is_empty())
    {
        items.push(format!("priority {value}"));
    }
    if let Some(value) = doc
        .field("assignee")
        .filter(|value| !value.trim().is_empty())
    {
        items.push(format!("assignee {value}"));
    }
    if let Some(value) = doc
        .field("updatedAt")
        .filter(|value| !value.trim().is_empty())
    {
        items.push(format!("updated {}", completed_at_compact(value)));
    }
    items
}

fn append_accord_lines(lines: &mut Vec<Line<'static>>, doc: &Document, theme: &TuiTheme) {
    let has_accord_detail = [
        "accord.assignee",
        "accord.claimedAt",
        "accord.deliveredAt",
        "accord.summary",
        "accord.evidence",
        "accord.validation.commands",
        "accord.deliverables",
        "accord.filesChanged",
        "accord.note",
        "accord.reason",
    ]
    .iter()
    .any(|key| doc.field(key).is_some());

    if !has_accord_detail {
        return;
    }

    lines.push(Line::from(""));
    lines.push(section_heading("Accord detail", theme));
    push_compact_optional(lines, "assignee", doc.field("accord.assignee"), theme);
    push_compact_optional(lines, "claimed", doc.field("accord.claimedAt"), theme);
    push_compact_optional(lines, "delivered", doc.field("accord.deliveredAt"), theme);
    push_compact_optional(lines, "summary", doc.field("accord.summary"), theme);
    push_array_detail_lines(lines, "evidence", doc.field("accord.evidence"), theme);
    push_array_detail_lines(
        lines,
        "validation commands",
        doc.field("accord.validation.commands")
            .or_else(|| doc.field("accord.validation"))
            .or_else(|| doc.field("accord.validations")),
        theme,
    );
    push_array_detail_lines(
        lines,
        "deliverables",
        doc.field("accord.deliverables"),
        theme,
    );
    push_array_detail_lines(
        lines,
        "accord files",
        doc.field("accord.filesChanged"),
        theme,
    );
    push_compact_optional(lines, "note", doc.field("accord.note"), theme);
    push_compact_optional(lines, "reason", doc.field("accord.reason"), theme);
}

fn push_array_detail_lines(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: Option<&str>,
    theme: &TuiTheme,
) {
    let values = value.map(parse_field_values).unwrap_or_default();
    if values.is_empty() {
        return;
    }
    lines.push(Line::from(Span::styled(
        format!("{label}:"),
        theme.label_style(),
    )));
    for item in values {
        lines.push(Line::from(vec![
            Span::styled("  - ", theme.markdown_list_style()),
            Span::styled(item, theme.text_style()),
        ]));
    }
}

fn append_event_lines(lines: &mut Vec<Line<'static>>, events: &[LogEvent], theme: &TuiTheme) {
    if events.is_empty() {
        return;
    }

    lines.push(Line::from(""));
    lines.push(section_heading("Event timeline", theme));
    for event in events.iter().rev().take(8).rev() {
        let ts = if event.ts.is_empty() {
            "unknown time".to_string()
        } else {
            event.ts.clone()
        };
        let summary = if event.summary.is_empty() {
            "-".to_string()
        } else {
            event.summary.clone()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{ts} "), theme.muted_style()),
            Span::styled(
                format!("{} ", event.event),
                theme.status_style(StatusTone::Accent),
            ),
            Span::styled(summary, theme.text_style()),
        ]));
    }
}

fn truncate_for_log(value: &str, max_chars: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }

    let mut output = value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>()
        .trim_end()
        .to_string();
    output.push('…');
    output
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn test_hierarchy(active_docs: &[Document], logs: &[Document]) -> HierarchyIndex {
        HierarchyIndex::from_documents(active_docs.iter().chain(logs.iter()).cloned().collect())
            .unwrap()
    }

    fn line_for_test_log(doc: &Document, theme: &TuiTheme, available_width: u16) -> Line<'static> {
        let hierarchy = test_hierarchy(&[], std::slice::from_ref(doc));
        line_for_log(doc, Some(&hierarchy), theme, available_width)
    }

    fn log_doc(id: &str, title: &str, summary: &str, completed_at: &str, body: &str) -> Document {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), id.to_string());
        fields.insert("type".to_string(), "task".to_string());
        fields.insert("title".to_string(), title.to_string());
        fields.insert("completedAt".to_string(), completed_at.to_string());
        fields.insert("completion.summary".to_string(), summary.to_string());
        Document {
            path: PathBuf::from(format!("{id}.md")),
            location: DocumentLocation::Logs,
            fields,
            body: body.to_string(),
        }
    }

    #[test]
    fn filters_logs_by_id_title_summary_and_body() {
        let logs = vec![
            log_doc(
                "task-1",
                "Theme work",
                "Palette done",
                "2026-01-01T00:00:00Z",
                "Body",
            ),
            log_doc(
                "task-2",
                "Other",
                "No match",
                "2026-01-02T00:00:00Z",
                "mentions logs",
            ),
        ];

        assert_eq!(filter_logs(&logs, None, "task-1").len(), 1);
        assert_eq!(filter_logs(&logs, None, "theme").len(), 1);
        assert_eq!(filter_logs(&logs, None, "palette").len(), 1);
        assert_eq!(filter_logs(&logs, None, "logs").len(), 1);
        assert_eq!(filter_logs(&logs, None, "missing").len(), 0);
    }

    #[test]
    fn logs_render_and_filter_canonical_epic_task_and_subtask_context() {
        let mut epic = log_doc("task-100", "Epic", "", "2026-01-01T00:00:00Z", "");
        epic.location = DocumentLocation::Board;
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut task = log_doc("task-101", "Epic task", "", "2026-01-02T00:00:00Z", "");
        task.fields
            .insert("parentId".to_string(), "task-100".to_string());
        let mut subtask = log_doc("task-101-1", "Leaf subtask", "", "2026-01-03T00:00:00Z", "");
        subtask
            .fields
            .insert("parentId".to_string(), "task-101".to_string());
        let logs = vec![task, subtask];
        let theme = TuiTheme::default_dark();
        let hierarchy = test_hierarchy(std::slice::from_ref(&epic), &logs);

        assert_eq!(
            filter_logs(&logs, Some(&hierarchy), "task of epic").len(),
            1
        );
        assert_eq!(filter_logs(&logs, Some(&hierarchy), "subtask of").len(), 1);
        let task_row = line_text(&line_for_log(&logs[0], Some(&hierarchy), &theme, 80));
        let subtask_row = line_text(&line_for_log(&logs[1], Some(&hierarchy), &theme, 80));
        assert!(task_row.contains("[TASK]"));
        assert!(!task_row.contains("[SUBTASK]"));
        assert!(subtask_row.contains("[SUBTASK]"));

        let detail = detail_lines_for_log(&logs[0], Some(&hierarchy), &[], &theme)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>();
        assert!(detail.iter().any(|line| line.contains("role: task")));
        assert!(detail
            .iter()
            .any(|line| line.contains("Task of Epic: Epic (task-100)")));
    }

    #[test]
    fn sorts_logs_by_completed_at_descending() {
        let mut logs = vec![
            log_doc("task-1", "Old", "", "2026-01-01T00:00:00Z", ""),
            log_doc("task-2", "New", "", "2026-01-02T00:00:00Z", ""),
        ];
        sort_logs_by_recency(&mut logs);
        assert_eq!(logs[0].id(), "task-2");
        assert_eq!(logs[1].id(), "task-1");
    }

    #[test]
    fn log_list_item_adds_canonical_role_to_task_id_and_title() {
        let theme = TuiTheme::default_dark();
        let mut doc = log_doc(
            "task-36",
            "Implement Tandem docs site foundation",
            "Long completion summary belongs in detail",
            "2026-06-28T17:34:12Z",
            "Body",
        );
        doc.fields
            .insert("accord.status".to_string(), "accepted".to_string());
        doc.fields.insert(
            "completion.filesChanged".to_string(),
            "[\"docs/index.md\", \"tandem/src/tui.rs\"]".to_string(),
        );

        let row = line_text(&line_for_test_log(&doc, &theme, 80));
        assert_eq!(row, "task-36  [TASK] Implement Tandem docs site foundation");
        assert!(!row.contains("17:34"));
        assert!(!row.contains("06-28"));
        assert!(!row.contains("2026-06-28T17:34:12Z"));
        assert!(!row.contains("accepted"));
        assert!(!row.contains("docs/index.md"));
        assert!(!row.contains("Long completion summary"));
        assert!(row.chars().count() <= 80);
    }

    #[test]
    fn log_list_item_uses_available_width_for_title_truncation() {
        let theme = TuiTheme::default_dark();
        let doc = log_doc(
            "task-36",
            "Implement Tandem docs site foundation",
            "Long completion summary belongs in detail",
            "2026-06-28T17:34:12Z",
            "Body",
        );

        let row = line_text(&line_for_test_log(&doc, &theme, 32));
        assert_eq!(row, "task-36  [TASK] Implement Tande…");
        assert!(row.chars().count() <= 32);
        assert!(row.ends_with('…'));
        assert!(!row.contains("17:34"));
    }

    #[test]
    fn log_list_item_falls_back_to_summary_when_title_is_missing() {
        let theme = TuiTheme::default_dark();
        let doc = log_doc(
            "task-36",
            "",
            "Completed the useful thing",
            "2026-06-28T17:34:12Z",
            "Body",
        );

        let row = line_text(&line_for_test_log(&doc, &theme, 80));
        assert_eq!(row, "task-36  [TASK] Completed the useful thing");
    }

    #[test]
    fn log_detail_prioritizes_completion_and_files_before_process_metadata() {
        let theme = TuiTheme::default_dark();
        let mut doc = log_doc(
            "task-36",
            "Implement Tandem docs site foundation",
            "Finished the useful docs foundation.",
            "2026-06-28T17:34:12Z",
            "Body",
        );
        doc.fields
            .insert("accord.status".to_string(), "accepted".to_string());
        doc.fields.insert(
            "completion.filesChanged".to_string(),
            "[\"docs/index.md\"]".to_string(),
        );

        let hierarchy = test_hierarchy(&[], std::slice::from_ref(&doc));
        let lines = detail_lines_for_log(&doc, Some(&hierarchy), &[], &theme)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>();
        let completion_index = lines.iter().position(|line| line == "Completion").unwrap();
        let files_index = lines
            .iter()
            .position(|line| line == "Files changed")
            .unwrap();
        let reference_index = lines
            .iter()
            .position(|line| line == "Log reference")
            .unwrap();
        let process_index = lines
            .iter()
            .position(|line| line == "Process metadata")
            .unwrap();

        assert!(completion_index < files_index);
        assert!(files_index < reference_index);
        assert!(reference_index < process_index);
        assert!(lines
            .iter()
            .any(|line| line.contains("Finished the useful")));
        assert!(lines.iter().any(|line| line.contains("docs/index.md")));
    }

    #[test]
    fn extracts_json_string_values_for_event_context() {
        let line = r#"{"ts":"2026-01-01T00:00:00Z","event":"task.completed","id":"task-1","summary":"Done \"well\""}"#;
        assert_eq!(extract_json_string(line, "id").as_deref(), Some("task-1"));
        assert_eq!(
            extract_json_string(line, "summary").as_deref(),
            Some("Done \"well\"")
        );
    }
}
