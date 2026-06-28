use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use crate::{
    accord_status, completion_files_changed, completion_reviewer, completion_summary,
    completion_validation, display_path, parse_field_values, read_document, review_status,
    Document, DocumentLocation,
};

use super::{
    detail_field_line, markdownish_lines, push_optional_detail_line, StatusTone, TuiTheme,
};

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

pub(super) fn filter_logs<'a>(logs: &'a [Document], query: &str) -> Vec<&'a Document> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return logs.iter().collect();
    }

    logs.iter()
        .filter(|doc| log_matches_query(doc, &query))
        .collect()
}

fn log_matches_query(doc: &Document, query: &str) -> bool {
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
    haystack
        .to_ascii_lowercase()
        .contains(&query.to_ascii_lowercase())
}

pub(super) fn list_item_for_log(doc: &Document, theme: &TuiTheme) -> ListItem<'static> {
    let completed = doc.field("completedAt").unwrap_or("-");
    let summary = completion_summary(doc).unwrap_or("-");
    let validation = completion_validation(doc).unwrap_or("-");
    let accord = accord_status(doc).unwrap_or("-");
    let files = completion_files_changed(doc);
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(
                truncate_for_log(completed, 20),
                theme.status_style(StatusTone::Accent),
            ),
            Span::raw(" "),
            Span::styled(
                truncate_for_log(doc.title(), 64),
                theme.text_style().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", doc.id()),
                theme.status_style(StatusTone::Accent),
            ),
            Span::styled(format!("A:{accord} "), theme.accord_style(accord)),
            Span::styled(
                format!("V:{} ", truncate_for_log(validation, 22)),
                theme.muted_style(),
            ),
            Span::styled(
                format!(
                    "{} file{} · ",
                    files.len(),
                    if files.len() == 1 { "" } else { "s" }
                ),
                theme.muted_style(),
            ),
            Span::styled(truncate_for_log(summary, 72), theme.muted_style()),
        ]),
    ])
}

pub(super) fn detail_lines_for_log(
    doc: &Document,
    events: &[LogEvent],
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Title: ", theme.label_style()),
        Span::styled(doc.title().to_string(), theme.text_style()),
    ]));
    lines.push(detail_field_line("ID", doc.id(), theme));
    lines.push(detail_field_line("Type", doc.doc_type(), theme));
    push_optional_detail_line(&mut lines, "Completed", doc.field("completedAt"), theme);
    push_optional_detail_line(&mut lines, "Updated", doc.field("updatedAt"), theme);
    push_optional_detail_line(&mut lines, "Summary", completion_summary(doc), theme);
    push_optional_detail_line(&mut lines, "Validation", completion_validation(doc), theme);
    push_optional_detail_line(&mut lines, "Reviewer", completion_reviewer(doc), theme);
    push_optional_detail_line(&mut lines, "Accord", accord_status(doc), theme);
    push_optional_detail_line(&mut lines, "Review", review_status(doc), theme);
    push_optional_detail_line(&mut lines, "Priority", doc.field("priority"), theme);
    push_optional_detail_line(&mut lines, "Assignee", doc.field("assignee"), theme);
    lines.push(detail_field_line("Path", &display_path(&doc.path), theme));

    let files = completion_files_changed(doc);
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Files changed",
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    )));
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

    append_accord_lines(&mut lines, doc, theme);
    append_event_lines(&mut lines, events, theme);

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Body",
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    )));
    if doc.body.trim().is_empty() {
        lines.push(Line::from(Span::styled("(empty)", theme.muted_style())));
    } else {
        lines.extend(markdownish_lines(&doc.body, theme));
    }

    lines
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
    lines.push(Line::from(Span::styled(
        "Accord detail",
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    )));
    push_optional_detail_line(lines, "Assignee", doc.field("accord.assignee"), theme);
    push_optional_detail_line(lines, "Claimed", doc.field("accord.claimedAt"), theme);
    push_optional_detail_line(lines, "Delivered", doc.field("accord.deliveredAt"), theme);
    push_optional_detail_line(lines, "Accord summary", doc.field("accord.summary"), theme);
    push_array_detail_lines(lines, "Evidence", doc.field("accord.evidence"), theme);
    push_array_detail_lines(
        lines,
        "Validation commands",
        doc.field("accord.validation.commands")
            .or_else(|| doc.field("accord.validation"))
            .or_else(|| doc.field("accord.validations")),
        theme,
    );
    push_array_detail_lines(
        lines,
        "Deliverables",
        doc.field("accord.deliverables"),
        theme,
    );
    push_array_detail_lines(
        lines,
        "Accord files",
        doc.field("accord.filesChanged"),
        theme,
    );
    push_optional_detail_line(lines, "Note", doc.field("accord.note"), theme);
    push_optional_detail_line(lines, "Reason", doc.field("accord.reason"), theme);
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
    lines.push(Line::from(Span::styled(
        "Event timeline",
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    )));
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
    let mut output = String::new();
    for (index, ch) in value.chars().enumerate() {
        if index >= max_chars {
            output.push('…');
            return output;
        }
        output.push(ch);
    }
    output
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;

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

        assert_eq!(filter_logs(&logs, "task-1").len(), 1);
        assert_eq!(filter_logs(&logs, "theme").len(), 1);
        assert_eq!(filter_logs(&logs, "palette").len(), 1);
        assert_eq!(filter_logs(&logs, "logs").len(), 1);
        assert_eq!(filter_logs(&logs, "missing").len(), 0);
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
    fn extracts_json_string_values_for_event_context() {
        let line = r#"{"ts":"2026-01-01T00:00:00Z","event":"task.completed","id":"task-1","summary":"Done \"well\""}"#;
        assert_eq!(extract_json_string(line, "id").as_deref(), Some("task-1"));
        assert_eq!(
            extract_json_string(line, "summary").as_deref(),
            Some("Done \"well\"")
        );
    }
}
