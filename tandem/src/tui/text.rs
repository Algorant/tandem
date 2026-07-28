//! Shared Markdown-style text rendering used by multiple TUI features.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::{StatusTone, TuiTheme};

pub(super) fn markdownish_lines(markdown: &str, theme: &TuiTheme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_fence = false;

    for line in markdown.lines() {
        if markdown_fence_marker(line).is_some() {
            lines.push(markdown_code_fence_line(line, theme));
            in_code_fence = !in_code_fence;
        } else if in_code_fence {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                theme.markdown_code_style(),
            )));
        } else {
            lines.push(markdownish_line(line, theme));
        }
    }

    lines
}

pub(super) fn markdownish_line(line: &str, theme: &TuiTheme) -> Line<'static> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return Line::from("");
    }

    let indent = &line[..line.len() - trimmed.len()];
    if let Some(heading) = markdown_heading_text(trimmed) {
        return Line::from(with_indent(
            indent,
            vec![Span::styled(
                heading.to_string(),
                theme.markdown_heading_style().add_modifier(Modifier::BOLD),
            )],
            theme,
        ));
    }

    if let Some(quote) = markdown_blockquote_text(trimmed) {
        let mut spans = with_indent(indent, vec![Span::styled("│ ", theme.muted_style())], theme);
        spans.extend(markdown_inline_spans(
            quote,
            theme,
            theme.muted_style().add_modifier(Modifier::ITALIC),
        ));
        return Line::from(spans);
    }

    if let Some((marker, content)) = markdown_list_parts(trimmed) {
        let mut spans = with_indent(
            indent,
            vec![Span::styled(marker, theme.markdown_list_style())],
            theme,
        );
        spans.extend(markdown_inline_spans(content, theme, theme.text_style()));
        return Line::from(spans);
    }

    Line::from(markdown_inline_spans(line, theme, theme.text_style()))
}

pub(super) fn markdown_fence_marker(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

pub(super) fn markdown_code_fence_line(line: &str, theme: &TuiTheme) -> Line<'static> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let marker = markdown_fence_marker(line).unwrap_or("```");
    let info = trimmed[marker.len()..].trim();
    let mut spans = with_indent(
        indent,
        vec![Span::styled(
            marker.to_string(),
            theme.markdown_code_style().add_modifier(Modifier::BOLD),
        )],
        theme,
    );
    if !info.is_empty() {
        spans.push(Span::styled(" ", theme.markdown_code_style()));
        spans.push(Span::styled(info.to_string(), theme.muted_style()));
    }
    Line::from(spans)
}

pub(super) fn markdown_heading_text(trimmed: &str) -> Option<&str> {
    let marker_count = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&marker_count) {
        return None;
    }

    let rest = &trimmed[marker_count..];
    if rest.is_empty() || rest.chars().next().is_some_and(|ch| ch.is_whitespace()) {
        let heading = rest.trim_start();
        Some(if heading.is_empty() { trimmed } else { heading })
    } else {
        None
    }
}

pub(super) fn markdown_blockquote_text(trimmed: &str) -> Option<&str> {
    let quote = trimmed.strip_prefix('>')?;
    Some(quote.strip_prefix(' ').unwrap_or(quote))
}

pub(super) fn markdown_list_parts(trimmed: &str) -> Option<(String, &str)> {
    for bullet in ["-", "*", "+"] {
        let unchecked = format!("{bullet} [ ] ");
        if let Some(content) = trimmed.strip_prefix(&unchecked) {
            return Some(("☐ ".to_string(), content));
        }
        let checked_lower = format!("{bullet} [x] ");
        if let Some(content) = trimmed.strip_prefix(&checked_lower) {
            return Some(("☑ ".to_string(), content));
        }
        let checked_upper = format!("{bullet} [X] ");
        if let Some(content) = trimmed.strip_prefix(&checked_upper) {
            return Some(("☑ ".to_string(), content));
        }
        let marker = format!("{bullet} ");
        if let Some(content) = trimmed.strip_prefix(&marker) {
            return Some(("• ".to_string(), content));
        }
    }

    let digit_count = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digit_count == 0 || digit_count + 2 > trimmed.len() {
        return None;
    }
    let suffix = &trimmed[digit_count..];
    if suffix.starts_with(". ") || suffix.starts_with(") ") {
        Some((
            trimmed[..digit_count + 2].to_string(),
            &trimmed[digit_count + 2..],
        ))
    } else {
        None
    }
}

pub(super) fn markdown_inline_spans(
    text: &str,
    theme: &TuiTheme,
    base_style: Style,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut rest = text;

    while !rest.is_empty() {
        if let Some(strong) = rest.strip_prefix("**") {
            if let Some(end) = strong.find("**") {
                push_span(
                    &mut spans,
                    &strong[..end],
                    base_style.add_modifier(Modifier::BOLD),
                );
                rest = &strong[end + 2..];
                continue;
            }
        }

        if let Some(code) = rest.strip_prefix('`') {
            if let Some(end) = code.find('`') {
                push_span(&mut spans, &code[..end], theme.markdown_code_style());
                rest = &code[end + 1..];
                continue;
            }
        }

        if let Some((consumed, label, url)) = markdown_link_parts(rest) {
            let link_label = if label.is_empty() { url } else { label };
            push_span(
                &mut spans,
                link_label,
                theme
                    .status_style(StatusTone::Accent)
                    .add_modifier(Modifier::UNDERLINED),
            );
            if !url.is_empty() && url != label {
                push_span(&mut spans, " (", theme.muted_style());
                push_span(&mut spans, url, theme.muted_style());
                push_span(&mut spans, ")", theme.muted_style());
            }
            rest = &rest[consumed..];
            continue;
        }

        let next_special = next_markdown_inline_special(rest).unwrap_or(rest.len());
        if next_special > 0 {
            push_span(&mut spans, &rest[..next_special], base_style);
            rest = &rest[next_special..];
        } else {
            let ch = rest.chars().next().expect("rest is not empty");
            push_span(&mut spans, &rest[..ch.len_utf8()], base_style);
            rest = &rest[ch.len_utf8()..];
        }
    }

    spans
}

pub(super) fn markdown_link_parts(text: &str) -> Option<(usize, &str, &str)> {
    let label_rest = text.strip_prefix('[')?;
    let label_end = label_rest.find(']')?;
    let after_label = &label_rest[label_end + 1..];
    let url_rest = after_label.strip_prefix('(')?;
    let url_end = url_rest.find(')')?;
    let consumed = 1 + label_end + 1 + 1 + url_end + 1;
    Some((consumed, &label_rest[..label_end], &url_rest[..url_end]))
}

pub(super) fn next_markdown_inline_special(text: &str) -> Option<usize> {
    ['`', '[', '*']
        .iter()
        .filter_map(|needle| text.find(*needle))
        .min()
}

pub(super) fn with_indent(
    indent: &str,
    mut spans: Vec<Span<'static>>,
    theme: &TuiTheme,
) -> Vec<Span<'static>> {
    if !indent.is_empty() {
        spans.insert(0, Span::styled(indent.to_string(), theme.text_style()));
    }
    spans
}

pub(super) fn push_span(spans: &mut Vec<Span<'static>>, content: &str, style: Style) {
    if !content.is_empty() {
        spans.push(Span::styled(content.to_string(), style));
    }
}
