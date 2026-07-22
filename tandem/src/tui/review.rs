use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::super::{
    accord_status, display_path, parse_field_values, review_status, Document, HierarchyIndex,
};
use super::{
    document_state_label, hierarchy_index_for, markdownish_lines, rect_contains, FocusPane,
    HitAction, HitRegion, StatusTone, TuiTheme,
};

const QUEUE_ROW_HEIGHT: u16 = 3;

#[derive(Debug, Clone)]
pub(super) struct ReviewQueueItem {
    id: String,
    doc_type: String,
    task_role: Option<String>,
    title: String,
    state: String,
    priority: String,
    assignee: String,
    tags: String,
    blockers: Vec<String>,
    accord_status: String,
    review_status: String,
    updated_at: String,
    path: String,
    body: String,
    accord_assignee: Option<String>,
    accord_delivered_at: Option<String>,
    accord_summary: Option<String>,
    accord_evidence: Vec<String>,
    accord_files_changed: Vec<String>,
    accord_validations: Vec<String>,
    accord_note: Option<String>,
    accord_reason: Option<String>,
    review_reviewer: Option<String>,
    review_requested_at: Option<String>,
    review_decided_at: Option<String>,
    review_note: Option<String>,
    validation_status: Option<String>,
    parent_label: Option<String>,
    parent_value: Option<String>,
    reasons: Vec<ReviewReason>,
}

impl ReviewQueueItem {
    pub(super) fn id(&self) -> &str {
        &self.id
    }

    pub(super) fn reason_summary(&self) -> String {
        self.reasons
            .iter()
            .map(|reason| reason.label.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn from_document(
        doc: &Document,
        active_docs: &[Document],
        completed_logs: &[Document],
        hierarchy: Option<&HierarchyIndex>,
    ) -> Option<Self> {
        let reasons = typed_attention_reasons(doc);
        if reasons.is_empty() {
            return None;
        }
        let parent_id = doc
            .field("parentId")
            .map(str::trim)
            .filter(|id| !id.is_empty());
        let parent_doc = parent_id.and_then(|id| {
            active_docs
                .iter()
                .chain(completed_logs.iter())
                .find(|candidate| candidate.id() == id)
        });
        let task_role = hierarchy
            .and_then(|hierarchy| hierarchy.validate_task_hierarchy(doc).ok())
            .map(|role| role.as_str().to_string());
        let parent_label = parent_id.map(|_| {
            hierarchy
                .filter(|hierarchy| {
                    doc.doc_type() != "task" || hierarchy.validate_task_hierarchy(doc).is_ok()
                })
                .and_then(|hierarchy| hierarchy.relationship(doc).ok().flatten())
                .map(|relationship| relationship.human_label().to_string())
                .unwrap_or_else(|| "Parent".to_string())
        });
        let parent_value = parent_id.map(|id| {
            parent_doc
                .map(|parent| format!("{} ({id})", parent.title()))
                .unwrap_or_else(|| format!("missing parent {id}"))
        });

        Some(Self {
            id: doc.id().to_string(),
            doc_type: doc.doc_type().to_string(),
            task_role,
            title: doc.title().to_string(),
            state: document_state_label(doc),
            priority: doc.field("priority").unwrap_or("-").to_string(),
            assignee: doc
                .field("assignee")
                .or_else(|| doc.field("accord.assignee"))
                .unwrap_or("-")
                .to_string(),
            tags: doc.field("tags").unwrap_or("").to_string(),
            blockers: doc
                .field("blockers")
                .map(parse_field_values)
                .unwrap_or_default(),
            accord_status: accord_status(doc).unwrap_or("-").to_string(),
            review_status: review_status(doc).unwrap_or("-").to_string(),
            updated_at: doc.field("updatedAt").unwrap_or("").to_string(),
            path: display_path(&doc.path),
            body: doc.body.clone(),
            accord_assignee: doc.field("accord.assignee").map(str::to_string),
            accord_delivered_at: doc.field("accord.deliveredAt").map(str::to_string),
            accord_summary: doc.field("accord.summary").map(str::to_string),
            accord_evidence: doc
                .field("accord.evidence")
                .map(parse_field_values)
                .unwrap_or_default(),
            accord_files_changed: doc
                .field("accord.filesChanged")
                .map(parse_field_values)
                .unwrap_or_default(),
            accord_validations: doc
                .field("accord.validation.commands")
                .or_else(|| doc.field("accord.validation"))
                .or_else(|| doc.field("accord.validations"))
                .map(parse_field_values)
                .unwrap_or_default(),
            accord_note: doc.field("accord.note").map(str::to_string),
            accord_reason: doc.field("accord.reason").map(str::to_string),
            review_reviewer: doc.field("review.reviewer").map(str::to_string),
            review_requested_at: doc.field("review.requestedAt").map(str::to_string),
            review_decided_at: doc.field("review.decidedAt").map(str::to_string),
            review_note: doc
                .field("review.note")
                .or_else(|| doc.field("review.notes"))
                .map(str::to_string),
            validation_status: validation_status(doc),
            parent_label,
            parent_value,
            reasons,
        })
    }

    fn priority_rank(&self) -> u8 {
        priority_rank(&self.priority)
    }

    fn reason_rank(&self) -> u8 {
        self.reasons
            .iter()
            .map(|reason| reason.kind.rank())
            .min()
            .unwrap_or(99)
    }

    fn sort_timestamp(&self) -> &str {
        self.accord_delivered_at
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&self.updated_at)
    }
}

#[derive(Debug, Clone)]
struct ReviewReason {
    label: String,
    detail: String,
    kind: ReviewReasonKind,
}

impl ReviewReason {
    fn new(label: impl Into<String>, detail: impl Into<String>, kind: ReviewReasonKind) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewReasonKind {
    AccordDelivered,
    ReviewPending,
    ReviewNeedsChanges,
    ReviewRejected,
    ReviewFailed,
    AccordBlocked,
    AccordFailed,
    AccordRework,
    AccordAcceptedActive,
    Blockers,
    ValidationFailed,
}

impl ReviewReasonKind {
    fn rank(self) -> u8 {
        match self {
            Self::ReviewFailed | Self::AccordFailed | Self::ValidationFailed => 0,
            Self::AccordBlocked | Self::Blockers => 1,
            Self::AccordDelivered | Self::ReviewPending => 2,
            Self::ReviewNeedsChanges | Self::ReviewRejected | Self::AccordRework => 3,
            Self::AccordAcceptedActive => 4,
        }
    }
}

#[cfg(test)]
pub(super) fn queue_items(
    active_docs: &[Document],
    completed_logs: &[Document],
) -> Vec<ReviewQueueItem> {
    let hierarchy = hierarchy_index_for(active_docs, completed_logs).ok();
    queue_items_with_hierarchy(active_docs, completed_logs, hierarchy.as_ref())
}

pub(super) fn queue_items_with_hierarchy(
    active_docs: &[Document],
    completed_logs: &[Document],
    hierarchy: Option<&HierarchyIndex>,
) -> Vec<ReviewQueueItem> {
    let mut items = active_docs
        .iter()
        .filter_map(|doc| {
            ReviewQueueItem::from_document(doc, active_docs, completed_logs, hierarchy)
        })
        .collect::<Vec<_>>();

    items.sort_by(|a, b| {
        a.priority_rank()
            .cmp(&b.priority_rank())
            .then_with(|| a.reason_rank().cmp(&b.reason_rank()))
            .then_with(|| b.sort_timestamp().cmp(a.sort_timestamp()))
            .then_with(|| a.id.cmp(&b.id))
    });
    items
}

pub(super) fn queue_len(docs: &[Document]) -> usize {
    docs.iter()
        .filter(|doc| !typed_attention_reasons(doc).is_empty())
        .count()
}

pub(super) fn selected_item(
    active_docs: &[Document],
    completed_logs: &[Document],
    selected: usize,
) -> Option<ReviewQueueItem> {
    let hierarchy = hierarchy_index_for(active_docs, completed_logs).ok();
    queue_items_with_hierarchy(active_docs, completed_logs, hierarchy.as_ref())
        .into_iter()
        .nth(selected)
}

pub(super) fn detail_line_count(item: Option<&ReviewQueueItem>, theme: &TuiTheme) -> usize {
    item.map(|item| detail_lines(item, theme).len())
        .unwrap_or(1)
}

pub(super) fn render_review(
    frame: &mut Frame<'_>,
    area: Rect,
    items: &[ReviewQueueItem],
    selected: usize,
    focus: FocusPane,
    detail_scroll: u16,
    theme: &TuiTheme,
    load_errors: &[String],
    hits: &mut Vec<HitRegion>,
) {
    let horizontal = area.width >= 92;
    let chunks = if horizontal {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
            .split(area)
    };

    render_queue_list(
        frame,
        chunks[0],
        items,
        selected,
        focus,
        theme,
        load_errors,
        hits,
    );
    render_detail(
        frame,
        chunks[1],
        items.get(selected),
        focus,
        detail_scroll,
        theme,
        hits,
    );
}

fn render_queue_list(
    frame: &mut Frame<'_>,
    area: Rect,
    items: &[ReviewQueueItem],
    selected: usize,
    focus: FocusPane,
    theme: &TuiTheme,
    load_errors: &[String],
    hits: &mut Vec<HitRegion>,
) {
    hits.push(HitRegion {
        rect: area,
        action: HitAction::FocusReviewList,
    });

    if items.is_empty() {
        let mut lines = vec![
            Line::from(Span::styled(
                "No active items currently need review attention.",
                theme.status_style(StatusTone::Success),
            )),
            Line::from(Span::styled(
                "The queue includes delivered accords, pending/failed reviews, blocked/rework/failed accords, accepted active accords, and blockers.",
                theme.muted_style(),
            )),
        ];
        append_load_warnings(&mut lines, load_errors, theme);
        let empty = Paragraph::new(lines)
            .style(theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Review queue (0) ")
                    .border_style(theme.border_style(focus == FocusPane::Board))
                    .style(theme.panel_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(empty, area);
        return;
    }

    register_row_hits(area, items.len(), hits);
    let rows = items
        .iter()
        .map(|item| queue_list_item(item, theme))
        .collect::<Vec<_>>();
    let list = List::new(rows)
        .style(theme.panel_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Review queue ({}) ", items.len()))
                .border_style(theme.border_style(focus == FocusPane::Board))
                .style(theme.panel_style()),
        )
        .highlight_style(theme.selected_style())
        .highlight_symbol("▸ ");
    let mut state = ListState::default();
    state.select(Some(selected.min(items.len() - 1)));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_detail(
    frame: &mut Frame<'_>,
    area: Rect,
    item: Option<&ReviewQueueItem>,
    focus: FocusPane,
    detail_scroll: u16,
    theme: &TuiTheme,
    hits: &mut Vec<HitRegion>,
) {
    hits.push(HitRegion {
        rect: area,
        action: HitAction::FocusReviewDetail,
    });

    let (title, lines) = match item {
        Some(item) => (format!(" Inspect {} ", item.id), detail_lines(item, theme)),
        None => (
            " Inspect ".to_string(),
            vec![Line::from(Span::styled(
                "No review item selected.",
                theme.muted_style(),
            ))],
        ),
    };
    let detail = Paragraph::new(lines)
        .style(theme.panel_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(theme.border_style(focus == FocusPane::Detail))
                .style(theme.panel_style()),
        )
        .scroll((detail_scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(detail, area);
}

fn queue_list_item(item: &ReviewQueueItem, theme: &TuiTheme) -> ListItem<'static> {
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(
                format!("{:<4}", ellipsize(&item.priority, 4).to_uppercase()),
                theme.priority_style(&item.priority),
            ),
            Span::raw(" "),
            Span::styled(
                format!("[{}] ", item.task_role.as_deref().unwrap_or(&item.doc_type)),
                theme.muted_style(),
            ),
            Span::styled(
                ellipsize(&item.title, 56),
                theme.text_style().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", item.id),
                theme.status_style(StatusTone::Accent),
            ),
            Span::styled(format!("{} ", item.state), theme.muted_style()),
            Span::styled(format!("@{} ", item.assignee), theme.muted_style()),
            Span::styled(
                format!("A:{} ", item.accord_status),
                theme.accord_style(&item.accord_status),
            ),
            Span::styled(
                format!("R:{} ", item.review_status),
                theme.review_style(&item.review_status),
            ),
        ]),
        reason_badge_line(item, theme),
    ])
}

fn reason_badge_line(item: &ReviewQueueItem, theme: &TuiTheme) -> Line<'static> {
    let mut spans = vec![Span::styled("needs: ", theme.label_style())];
    for (index, reason) in item.reasons.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", theme.muted_style()));
        }
        spans.push(Span::styled(
            reason.label.clone(),
            reason_style(reason, theme).add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(spans)
}

fn detail_lines(item: &ReviewQueueItem, theme: &TuiTheme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Needs attention: ", theme.label_style()),
        Span::styled(
            item.reason_summary(),
            theme.status_style(StatusTone::Warning),
        ),
    ]));
    for reason in &item.reasons {
        lines.push(Line::from(vec![
            Span::styled("  • ", theme.muted_style()),
            Span::styled(
                reason.label.clone(),
                reason_style(reason, theme).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" — {}", reason.detail), theme.text_style()),
        ]));
    }
    lines.push(Line::from(""));

    lines.push(section_heading("Document", theme));
    lines.push(field_line("Title", &item.title, theme));
    lines.push(field_line("ID", &item.id, theme));
    lines.push(field_line("Type", &item.doc_type, theme));
    optional_field_line(&mut lines, "Role", item.task_role.as_deref(), theme);
    lines.push(field_line("State", &item.state, theme));
    lines.push(field_line("Priority", &item.priority, theme));
    lines.push(field_line("Assignee", &item.assignee, theme));
    if let (Some(label), Some(value)) = (&item.parent_label, &item.parent_value) {
        lines.push(field_line(label, value, theme));
    }
    optional_field_line(&mut lines, "Tags", non_empty(&item.tags), theme);
    optional_field_line(&mut lines, "Updated", non_empty(&item.updated_at), theme);
    if !item.blockers.is_empty() {
        lines.push(field_line("Blockers", &item.blockers.join(", "), theme));
    }
    lines.push(field_line("Path", &item.path, theme));
    lines.push(Line::from(""));

    lines.push(section_heading("Accord", theme));
    lines.push(status_line(
        "Status",
        &item.accord_status,
        theme.accord_style(&item.accord_status),
        theme,
    ));
    optional_field_line(
        &mut lines,
        "Accord assignee",
        item.accord_assignee.as_deref(),
        theme,
    );
    optional_field_line(
        &mut lines,
        "Delivered",
        item.accord_delivered_at.as_deref(),
        theme,
    );
    optional_field_line(&mut lines, "Summary", item.accord_summary.as_deref(), theme);
    append_list_field(&mut lines, "Evidence", &item.accord_evidence, theme);
    append_list_field(
        &mut lines,
        "Files changed",
        &item.accord_files_changed,
        theme,
    );
    append_list_field(&mut lines, "Validation", &item.accord_validations, theme);
    optional_field_line(&mut lines, "Note", item.accord_note.as_deref(), theme);
    optional_field_line(&mut lines, "Reason", item.accord_reason.as_deref(), theme);
    lines.push(Line::from(""));

    lines.push(section_heading("Review", theme));
    lines.push(status_line(
        "Status",
        &item.review_status,
        theme.review_style(&item.review_status),
        theme,
    ));
    optional_field_line(
        &mut lines,
        "Reviewer",
        item.review_reviewer.as_deref(),
        theme,
    );
    optional_field_line(
        &mut lines,
        "Requested",
        item.review_requested_at.as_deref(),
        theme,
    );
    optional_field_line(
        &mut lines,
        "Decided",
        item.review_decided_at.as_deref(),
        theme,
    );
    optional_field_line(
        &mut lines,
        "Review note",
        item.review_note.as_deref(),
        theme,
    );
    optional_field_line(
        &mut lines,
        "Validation status",
        item.validation_status.as_deref(),
        theme,
    );
    lines.push(Line::from(""));

    lines.push(section_heading("Action hints", theme));
    lines.push(Line::from(Span::styled(
        "Read-only in this TUI slice. Use CLI actions for now:",
        theme.muted_style(),
    )));
    lines.push(Line::from(Span::styled(
        "tandem accord accept|rework|block|fail <id>; tandem complete <id> --summary <text>",
        theme.text_style(),
    )));
    lines.push(Line::from(Span::styled(
        "Accepted accord != completed log; completion/archive is a separate action.",
        theme.muted_style(),
    )));
    lines.push(Line::from(""));

    lines.push(section_heading("Body", theme));
    if item.body.trim().is_empty() {
        lines.push(Line::from(Span::styled("(empty)", theme.muted_style())));
    } else {
        lines.extend(markdownish_lines(&item.body, theme));
    }
    lines
}

fn typed_attention_reasons(doc: &Document) -> Vec<ReviewReason> {
    let mut reasons = Vec::new();
    let accord = normalized(accord_status(doc).unwrap_or(""));
    match accord.as_str() {
        "delivered" => reasons.push(ReviewReason::new(
            "sign-off",
            "delivered work is awaiting human sign-off",
            ReviewReasonKind::AccordDelivered,
        )),
        "blocked" => reasons.push(ReviewReason::new(
            "A:blocked",
            doc.field("accord.reason")
                .unwrap_or("accord is blocked")
                .to_string(),
            ReviewReasonKind::AccordBlocked,
        )),
        "failed" => reasons.push(ReviewReason::new(
            "A:failed",
            doc.field("accord.reason")
                .unwrap_or("accord failed")
                .to_string(),
            ReviewReasonKind::AccordFailed,
        )),
        "rework" => reasons.push(ReviewReason::new(
            "A:rework",
            doc.field("accord.note")
                .unwrap_or("accord needs rework")
                .to_string(),
            ReviewReasonKind::AccordRework,
        )),
        "accepted" => reasons.push(ReviewReason::new(
            "A:accepted active",
            "accord accepted, but the item is still active and has not been completed to logs",
            ReviewReasonKind::AccordAcceptedActive,
        )),
        _ => {}
    }

    match normalized(review_status(doc).unwrap_or("")).as_str() {
        "pending" => reasons.push(ReviewReason::new(
            "R:pending",
            "review is pending",
            ReviewReasonKind::ReviewPending,
        )),
        "changes-requested" => reasons.push(ReviewReason::new(
            "R:changes",
            doc.field("review.note")
                .or_else(|| doc.field("review.notes"))
                .unwrap_or("review requested changes")
                .to_string(),
            ReviewReasonKind::ReviewNeedsChanges,
        )),
        "rejected" => reasons.push(ReviewReason::new(
            "R:rejected",
            doc.field("review.note")
                .or_else(|| doc.field("review.notes"))
                .unwrap_or("review rejected")
                .to_string(),
            ReviewReasonKind::ReviewRejected,
        )),
        "failed" => reasons.push(ReviewReason::new(
            "R:failed",
            doc.field("review.note")
                .or_else(|| doc.field("review.notes"))
                .unwrap_or("review failed")
                .to_string(),
            ReviewReasonKind::ReviewFailed,
        )),
        _ => {
            if doc.doc_type() == "task" && normalized(&document_state_label(doc)) == "review" {
                reasons.push(ReviewReason::new(
                    "state:review",
                    "task is in the review workflow state",
                    ReviewReasonKind::ReviewPending,
                ));
            }
        }
    }

    if let Some(status) = validation_status(doc) {
        if is_failed_status(&status) {
            reasons.push(ReviewReason::new(
                "validation failed",
                status,
                ReviewReasonKind::ValidationFailed,
            ));
        }
    }

    let blockers = doc
        .field("blockers")
        .map(parse_field_values)
        .unwrap_or_default();
    if !blockers.is_empty() {
        reasons.push(ReviewReason::new(
            "blockers",
            blockers.join(", "),
            ReviewReasonKind::Blockers,
        ));
    }

    reasons
}

fn validation_status(doc: &Document) -> Option<String> {
    [
        "validation.status",
        "accord.validation.status",
        "review.validation.status",
        "completion.validation.status",
    ]
    .iter()
    .find_map(|key| doc.field(key).map(str::to_string))
}

fn is_failed_status(status: &str) -> bool {
    matches!(
        normalized(status).as_str(),
        "failed" | "fail" | "failure" | "error"
    )
}

fn register_row_hits(area: Rect, item_count: usize, hits: &mut Vec<HitRegion>) {
    if area.width <= 2 || area.height <= 2 {
        return;
    }
    let list_top = area.y.saturating_add(1);
    let list_bottom = area.y.saturating_add(area.height).saturating_sub(1);
    for index in 0..item_count {
        let y = list_top.saturating_add((index as u16).saturating_mul(QUEUE_ROW_HEIGHT));
        if y >= list_bottom {
            break;
        }
        let height = QUEUE_ROW_HEIGHT.min(list_bottom.saturating_sub(y));
        let rect = Rect {
            x: area.x.saturating_add(1),
            y,
            width: area.width.saturating_sub(2),
            height,
        };
        if rect_contains(rect, rect.x, rect.y) {
            hits.push(HitRegion {
                rect,
                action: HitAction::SelectReviewItem(index),
            });
        }
    }
}

fn append_load_warnings(lines: &mut Vec<Line<'static>>, load_errors: &[String], theme: &TuiTheme) {
    if load_errors.is_empty() {
        return;
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Load warnings:",
        theme
            .status_style(StatusTone::Error)
            .add_modifier(Modifier::BOLD),
    )));
    for error in load_errors {
        lines.push(Line::from(Span::styled(
            error.clone(),
            theme.status_style(StatusTone::Error),
        )));
    }
}

fn reason_style(reason: &ReviewReason, theme: &TuiTheme) -> Style {
    match reason.kind {
        ReviewReasonKind::AccordDelivered => theme.accord_style("delivered"),
        ReviewReasonKind::AccordBlocked => theme.accord_style("blocked"),
        ReviewReasonKind::AccordFailed => theme.accord_style("failed"),
        ReviewReasonKind::AccordRework => theme.accord_style("rework"),
        ReviewReasonKind::AccordAcceptedActive => theme.accord_style("accepted"),
        ReviewReasonKind::ReviewPending => theme.review_style("pending"),
        ReviewReasonKind::ReviewNeedsChanges => theme.review_style("changes-requested"),
        ReviewReasonKind::ReviewRejected => theme.review_style("rejected"),
        ReviewReasonKind::ReviewFailed => theme.review_style("failed"),
        ReviewReasonKind::ValidationFailed => theme.status_style(StatusTone::Error),
        ReviewReasonKind::Blockers => theme.status_style(StatusTone::Warning),
    }
}

fn status_line(label: &str, value: &str, style: Style, theme: &TuiTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme.label_style()),
        Span::styled(value.to_string(), style),
    ])
}

fn section_heading(label: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(Span::styled(
        label.to_string(),
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    ))
}

fn field_line(label: &str, value: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme.label_style()),
        Span::styled(value.to_string(), theme.text_style()),
    ])
}

fn optional_field_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: Option<&str>,
    theme: &TuiTheme,
) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(field_line(label, value, theme));
    }
}

fn append_list_field(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    values: &[String],
    theme: &TuiTheme,
) {
    if values.is_empty() {
        return;
    }
    lines.push(field_line(label, &values.join(", "), theme));
}

fn priority_rank(priority: &str) -> u8 {
    match normalized(priority).as_str() {
        "critical" | "urgent" => 0,
        "high" => 1,
        "medium" | "med" => 2,
        "low" => 3,
        "" | "-" | "none" => 4,
        _ => 5,
    }
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.trim().is_empty()).then_some(value)
}

fn ellipsize(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let keep = max.saturating_sub(1);
    let mut output = value.chars().take(keep).collect::<String>();
    output.push('…');
    output
}

fn normalized(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;
    use crate::DocumentLocation;

    fn doc_with_fields(id: &str, fields: &[(&str, &str)]) -> Document {
        let mut map = HashMap::new();
        map.insert("id".to_string(), id.to_string());
        map.insert("type".to_string(), "task".to_string());
        map.insert("title".to_string(), format!("Task {id}"));
        for (key, value) in fields {
            map.insert((*key).to_string(), (*value).to_string());
        }
        Document {
            path: PathBuf::from(format!("{id}.md")),
            location: DocumentLocation::Board,
            fields: map,
            body: String::new(),
        }
    }

    fn reason_labels(doc: &Document) -> Vec<String> {
        typed_attention_reasons(doc)
            .into_iter()
            .map(|reason| reason.label)
            .collect()
    }

    #[test]
    fn reasons_cover_review_queue_inputs() {
        let delivered = doc_with_fields("task-1", &[("accord.status", "delivered")]);
        assert!(reason_labels(&delivered).contains(&"sign-off".to_string()));

        let changes = doc_with_fields("task-2", &[("review.status", "changes-requested")]);
        assert!(reason_labels(&changes).contains(&"R:changes".to_string()));

        let blockers = doc_with_fields("task-3", &[("blockers", "[task-1]")]);
        assert!(reason_labels(&blockers).contains(&"blockers".to_string()));

        let accepted = doc_with_fields("task-4", &[("accord.status", "accepted")]);
        assert!(reason_labels(&accepted).contains(&"A:accepted active".to_string()));
    }

    #[test]
    fn review_detail_preserves_epic_task_subtask_and_generic_parent_context() {
        let mut epic_parent = doc_with_fields("task-200", &[]);
        epic_parent.location = DocumentLocation::Logs;
        epic_parent
            .fields
            .insert("kind".to_string(), "epic".to_string());
        let mut task_parent = doc_with_fields("task-103", &[]);
        task_parent.location = DocumentLocation::Logs;
        let mut decision_parent = doc_with_fields("decision-4", &[]);
        decision_parent.location = DocumentLocation::Logs;
        decision_parent
            .fields
            .insert("type".to_string(), "decision".to_string());
        let epic_task = doc_with_fields(
            "task-201",
            &[("parentId", "task-200"), ("accord.status", "delivered")],
        );
        let task_child = doc_with_fields(
            "task-103-1",
            &[("parentId", "task-103"), ("accord.status", "delivered")],
        );
        let generic_child = doc_with_fields(
            "task-7",
            &[("parentId", "decision-4"), ("accord.status", "delivered")],
        );
        let docs = vec![epic_task, task_child, generic_child];
        let logs = vec![epic_parent, task_parent, decision_parent];
        let items = queue_items(&docs, &logs);
        let theme = TuiTheme::default_dark();
        let epic_task_item = items.iter().find(|item| item.id() == "task-201").unwrap();
        let task_item = items.iter().find(|item| item.id() == "task-103-1").unwrap();
        let generic_item = items.iter().find(|item| item.id() == "task-7").unwrap();
        let epic_task_text = detail_lines(epic_task_item, &theme)
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        let task_text = detail_lines(task_item, &theme)
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        let generic_text = detail_lines(generic_item, &theme)
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        assert!(epic_task_text.contains(&"Role: task".to_string()));
        assert!(epic_task_text.contains(&"Task of Epic: Task task-200 (task-200)".to_string()));
        assert!(task_text.contains(&"Role: subtask".to_string()));
        assert!(task_text.contains(&"Subtask of: Task task-103 (task-103)".to_string()));
        assert!(generic_text.contains(&"Parent: Task decision-4 (decision-4)".to_string()));
        assert!(!generic_text.iter().any(|line| line.contains("Subtask")));
    }

    #[test]
    fn queue_sorts_priority_before_timestamp() {
        let low_recent = doc_with_fields(
            "task-1",
            &[
                ("accord.status", "delivered"),
                ("priority", "low"),
                ("updatedAt", "2026-06-27T12:00:00Z"),
            ],
        );
        let high_older = doc_with_fields(
            "task-2",
            &[
                ("review.status", "pending"),
                ("priority", "high"),
                ("updatedAt", "2026-06-26T12:00:00Z"),
            ],
        );
        let items = queue_items(&[low_recent, high_older], &[]);
        assert_eq!(items[0].id(), "task-2");
        assert_eq!(items[1].id(), "task-1");
    }
}
