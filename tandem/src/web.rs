//! Read-only local HTTP peer interface over shared application queries.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::process::Command;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, Method, Request, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tokio::sync::Semaphore;

use crate::app;
use crate::app::queries::{ListFilter, ReadSnapshot};
use crate::project::{StoredDocument as Document, TandemProject};
use crate::protocol::accord::{status as accord_status, AccordRecord};
use crate::protocol::document::parse_field_values;
use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship, TaskRole};
use crate::protocol::review::status as review_status;
use crate::protocol::workflow::{
    completion_files_changed, completion_outcome, completion_reviewer, completion_summary,
    completion_validation, state_matches_filter,
};
use crate::CliError;

const INDEX_HTML: &str = include_str!("web/index.html");
const APP_CSS: &str = include_str!("web/app.css");
const APP_JS: &str = include_str!("web/app.js");
const API_JS: &str = include_str!("web/api.js");
const UI_JS: &str = include_str!("web/ui.js");

const MAX_REQUEST_TARGET_BYTES: usize = 4 * 1024;
const MAX_CONCURRENT_REQUESTS: usize = 64;
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; base-uri 'none'; object-src 'none'; frame-ancestors 'none'; form-action 'none'; connect-src 'self'; img-src 'self'; script-src 'self'; style-src 'self'";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Options {
    pub(crate) port: Option<u16>,
    pub(crate) no_open: bool,
}

#[derive(Clone)]
struct WebState {
    project: Arc<TandemProject>,
    expected_host: Arc<str>,
    request_slots: Arc<Semaphore>,
}

pub(crate) fn run(project: TandemProject, options: Options) -> Result<(), CliError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| CliError::user(format!("failed to start web runtime: {error}")))?;
    runtime.block_on(serve(project, options))
}

async fn serve(project: TandemProject, options: Options) -> Result<(), CliError> {
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, options.port.unwrap_or(0)))
        .await
        .map_err(|error| {
            let port = options
                .port
                .map(|port| port.to_string())
                .unwrap_or_else(|| "an available port".to_string());
            CliError::user(format!("could not bind 127.0.0.1:{port}: {error}"))
        })?;
    let address = listener.local_addr()?;
    let url = format!("http://127.0.0.1:{}/", address.port());
    println!("Tandem web is ready (read-only)");
    println!("URL:     {url}");
    println!("Project: {}", project.root().display());
    println!("Press Ctrl-C to stop.");

    if should_open_browser(options) {
        if let Err(error) = open_browser(&url) {
            eprintln!("Warning: could not open the browser: {error}");
        }
    }

    axum::serve(listener, router(project, &address.to_string()))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| CliError::user(format!("web server failed: {error}")))
}

fn should_open_browser(options: Options) -> bool {
    !options.no_open
}

fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("rundll32");
        command.args(["url.dll,FileProtocolHandler", url]);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(url);
        command
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    };
    command.spawn().map(|_| ())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

fn router(project: TandemProject, expected_host: &str) -> Router {
    let state = WebState {
        project: Arc::new(project),
        expected_host: Arc::from(expected_host),
        request_slots: Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS)),
    };
    Router::new()
        .route("/", get(index))
        .route("/assets/app.css", get(styles))
        .route("/assets/app.js", get(script))
        .route("/assets/api.js", get(api_script))
        .route("/assets/ui.js", get(ui_script))
        .route("/api/v1/project", get(project_api))
        .route("/api/v1/board", get(board_api))
        .route("/api/v1/attention", get(attention_api))
        .route("/api/v1/documents/{id}", get(document_api))
        .route("/api/v1/logs", get(logs_api))
        .route("/api/v1/logs/{id}", get(log_api))
        .route("/api/v1/rules", get(rules_api))
        .route("/api/v1/decisions", get(decisions_api))
        .route("/api/v1/decisions/{id}", get(decision_api))
        .fallback(not_found)
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state, secure_request))
}

async fn secure_request(
    State(state): State<WebState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let revision = || "unavailable".to_string();
    let host_is_valid = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| host == state.expected_host.as_ref());
    if !host_is_valid {
        return api_error(
            StatusCode::BAD_REQUEST,
            "invalid_host",
            "the Host header does not match this local server".to_string(),
            revision(),
        );
    }
    if request.uri().to_string().len() > MAX_REQUEST_TARGET_BYTES {
        return api_error(
            StatusCode::URI_TOO_LONG,
            "request_target_too_large",
            "the request target is too large".to_string(),
            revision(),
        );
    }
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return api_error(
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
            "this local server provides read-only GET and HEAD routes".to_string(),
            revision(),
        );
    }
    let has_body = request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value != "0")
        || request.headers().contains_key(header::TRANSFER_ENCODING);
    if has_body {
        return api_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            "request_body_not_allowed",
            "request bodies are not accepted".to_string(),
            revision(),
        );
    }
    let Ok(_permit) = state.request_slots.clone().try_acquire_owned() else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "server_busy",
            "the local server is busy; retry shortly".to_string(),
            revision(),
        );
    };
    let mut response = next.run(request).await;
    security_headers(&mut response);
    response
}

async fn index() -> Response {
    static_response(Html(INDEX_HTML), "text/html; charset=utf-8")
}

async fn styles() -> Response {
    static_response(APP_CSS, "text/css; charset=utf-8")
}

async fn script() -> Response {
    static_response(APP_JS, "text/javascript; charset=utf-8")
}

async fn api_script() -> Response {
    static_response(API_JS, "text/javascript; charset=utf-8")
}

async fn ui_script() -> Response {
    static_response(UI_JS, "text/javascript; charset=utf-8")
}

fn static_response(content: impl IntoResponse, content_type: &'static str) -> Response {
    let mut response = content.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(content_type),
    );
    security_headers(&mut response);
    response
}

async fn not_found(State(state): State<WebState>, _request: Request<Body>) -> Response {
    let revision = app::queries::load_read(&state.project)
        .map(|read| read.revision)
        .unwrap_or_else(|_| "unavailable".to_string());
    api_error(
        StatusCode::NOT_FOUND,
        "route_not_found",
        "the requested read route does not exist".to_string(),
        revision,
    )
}

async fn project_api(State(state): State<WebState>) -> Response {
    with_snapshot(&state, |read| {
        let board = read.snapshot.board_documents(&ListFilter::default());
        let logs = read.snapshot.log_documents();
        let decisions = board
            .iter()
            .filter(|document| document.doc_type() == "decision")
            .count();
        let tasks = board
            .iter()
            .filter(|document| document.doc_type() == "task")
            .count();
        let mut by_state = BTreeMap::new();
        for document in board
            .iter()
            .filter(|document| document.doc_type() == "task")
        {
            *by_state
                .entry(document.field("state").unwrap_or("unknown").to_string())
                .or_insert(0usize) += 1;
        }
        Ok(ProjectDto {
            title: read.title.clone(),
            protocol_version: read.protocol_version.clone(),
            states: read.states.clone(),
            read_only: true,
            health: HealthDto {
                status: if read.warnings.is_empty() {
                    "healthy"
                } else {
                    "warnings"
                },
                board_documents: board.len(),
                tasks,
                decisions,
                logs: logs.len(),
                by_state,
            },
        })
    })
}

async fn board_api(State(state): State<WebState>, uri: Uri) -> Response {
    with_snapshot(&state, |read| {
        let query = parse_query(&uri)?;
        reject_unknown_query(
            &query,
            &[
                "state", "type", "priority", "tag", "assignee", "parent", "accord", "review",
            ],
        )?;
        let filter = ListFilter {
            state: query.get("state").map(String::as_str),
            doc_type: query.get("type").map(String::as_str),
            priority: query.get("priority").map(String::as_str),
            tag: query.get("tag").map(String::as_str),
            assignee: query.get("assignee").map(String::as_str),
            parent: query.get("parent").map(String::as_str),
            accord: query.get("accord").map(String::as_str),
            review: query.get("review").map(String::as_str),
        };
        let mut documents = read.snapshot.board_documents(&filter);
        sort_documents(&mut documents);
        let items = documents
            .iter()
            .map(|document| summary_dto(read, document))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(BoardDto {
            states: read.states.clone(),
            items,
        })
    })
}

async fn attention_api(State(state): State<WebState>) -> Response {
    with_snapshot(&state, |read| {
        let mut documents = read
            .snapshot
            .board_documents(&ListFilter::default())
            .into_iter()
            .filter(|document| {
                state_matches_filter(document.field("state"), "validation")
                    || accord_status(document) == Some("delivered")
                    || review_status(document) == Some("pending")
                    || review_status(document) == Some("changes-requested")
            })
            .collect::<Vec<_>>();
        sort_documents(&mut documents);
        Ok(AttentionDto {
            items: documents
                .iter()
                .map(|document| summary_dto(read, document))
                .collect::<Result<Vec<_>, _>>()?,
        })
    })
}

async fn document_api(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    document_response(&state, &id, None, None)
}

async fn logs_api(State(state): State<WebState>, uri: Uri) -> Response {
    with_snapshot(&state, |read| {
        let query = parse_query(&uri)?;
        reject_unknown_query(&query, &["query", "limit"])?;
        let limit = parse_limit(query.get("limit"))?;
        let search = query.get("query").map(|value| value.to_lowercase());
        let mut documents = read.snapshot.log_documents();
        documents.retain(|document| {
            search.as_ref().is_none_or(|query| {
                document.id().to_lowercase().contains(query)
                    || document.title().to_lowercase().contains(query)
                    || document.body.to_lowercase().contains(query)
                    || document
                        .fields
                        .values()
                        .any(|value| value.to_lowercase().contains(query))
            })
        });
        documents.sort_by(|a, b| {
            b.field("completedAt")
                .unwrap_or("")
                .cmp(a.field("completedAt").unwrap_or(""))
                .then_with(|| a.id().cmp(b.id()))
        });
        let total = documents.len();
        documents.truncate(limit);
        Ok(LogsDto {
            total,
            limit,
            items: documents.iter().map(log_summary_dto).collect(),
        })
    })
}

async fn log_api(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    document_response(&state, &id, Some(DocumentLocation::Logs), None)
}

async fn rules_api(State(state): State<WebState>) -> Response {
    with_snapshot(&state, |read| {
        Ok(RulesDto {
            categories: read
                .rules
                .iter()
                .map(|(category, items)| {
                    (
                        category.clone(),
                        items
                            .iter()
                            .map(|item| RuleDto {
                                id: item.id,
                                rule: item.rule.clone(),
                                source: item.source.clone(),
                            })
                            .collect(),
                    )
                })
                .collect(),
        })
    })
}

async fn decisions_api(State(state): State<WebState>) -> Response {
    with_snapshot(&state, |read| {
        let mut decisions = read
            .snapshot
            .board_documents(&ListFilter::default())
            .into_iter()
            .filter(|document| document.doc_type() == "decision")
            .collect::<Vec<_>>();
        decisions.sort_by(|a, b| a.id().cmp(b.id()));
        Ok(DecisionsDto {
            items: decisions.iter().map(decision_dto).collect(),
        })
    })
}

async fn decision_api(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    document_response(&state, &id, None, Some("decision"))
}

fn document_response(
    state: &WebState,
    id: &str,
    location: Option<DocumentLocation>,
    document_type: Option<&str>,
) -> Response {
    with_snapshot(state, |read| {
        validate_id(id)?;
        let document = read
            .snapshot
            .document(id)
            .filter(|document| location.is_none_or(|expected| document.location == expected))
            .filter(|document| document_type.is_none_or(|expected| document.doc_type() == expected))
            .ok_or_else(|| {
                if document_type == Some("decision") {
                    ApiFailure::not_found("decision_not_found", "decision was not found")
                } else {
                    ApiFailure::not_found("document_not_found", "document was not found")
                }
            })?;
        detail_dto(read, &document)
    })
}

fn with_snapshot<T, F>(state: &WebState, build: F) -> Response
where
    T: Serialize,
    F: FnOnce(&ReadSnapshot) -> Result<T, ApiFailure>,
{
    let read = match app::queries::load_read(&state.project) {
        Ok(read) => read,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "project_read_failed",
                "the project snapshot could not be loaded".to_string(),
                "unavailable".to_string(),
            )
        }
    };
    match build(&read) {
        Ok(data) => api_ok(data, &read),
        Err(error) => api_error(error.status, error.code, error.message, read.revision),
    }
}

fn api_ok<T: Serialize>(data: T, read: &ReadSnapshot) -> Response {
    let mut response = Json(Envelope {
        ok: true,
        data,
        revision: read.revision.clone(),
        warnings: read.warnings.clone(),
    })
    .into_response();
    api_headers(&mut response);
    response
}

fn api_error(
    status: StatusCode,
    code: &'static str,
    message: String,
    revision: String,
) -> Response {
    let mut response = (
        status,
        Json(ErrorEnvelope {
            ok: false,
            error: ErrorDto { code, message },
            revision,
            warnings: Vec::new(),
        }),
    )
        .into_response();
    api_headers(&mut response);
    response
}

fn api_headers(response: &mut Response) {
    security_headers(response);
}

fn security_headers(response: &mut Response) {
    let headers = response.headers_mut();
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        header::HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        header::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        header::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        header::HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        header::HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
}

fn parse_query(uri: &Uri) -> Result<BTreeMap<String, String>, ApiFailure> {
    let mut values = BTreeMap::new();
    let Some(query) = uri.query() else {
        return Ok(values);
    };
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = percent_decode(key)?;
        let value = percent_decode(value)?;
        if values.insert(key.clone(), value).is_some() {
            return Err(ApiFailure::bad_request(
                "query parameters must not be repeated",
            ));
        }
    }
    Ok(values)
}

fn percent_decode(value: &str) -> Result<String, ApiFailure> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let high = hex(bytes[index + 1]);
                let low = hex(bytes[index + 2]);
                let (Some(high), Some(low)) = (high, low) else {
                    return Err(ApiFailure::bad_request(
                        "query contains invalid percent encoding",
                    ));
                };
                decoded.push(high * 16 + low);
                index += 3;
            }
            b'%' => {
                return Err(ApiFailure::bad_request(
                    "query contains invalid percent encoding",
                ))
            }
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded)
        .map_err(|_| ApiFailure::bad_request("query parameters must be UTF-8"))
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn reject_unknown_query(
    query: &BTreeMap<String, String>,
    allowed: &[&str],
) -> Result<(), ApiFailure> {
    if query.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(ApiFailure::bad_request("unknown query parameter"));
    }
    Ok(())
}

fn parse_limit(value: Option<&String>) -> Result<usize, ApiFailure> {
    match value {
        None => Ok(100),
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|limit| (1..=500).contains(limit))
            .ok_or_else(|| ApiFailure::bad_request("limit must be between 1 and 500")),
    }
}

fn validate_id(id: &str) -> Result<(), ApiFailure> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ApiFailure::bad_request("document ID is malformed"));
    }
    Ok(())
}

fn sort_documents(documents: &mut [Document]) {
    documents.sort_by(|a, b| {
        a.field("state")
            .unwrap_or("")
            .cmp(b.field("state").unwrap_or(""))
            .then_with(|| a.id().cmp(b.id()))
    });
}

fn summary_dto(read: &ReadSnapshot, document: &Document) -> Result<DocumentSummaryDto, ApiFailure> {
    let role = read
        .snapshot
        .hierarchy
        .task_role(document)
        .map_err(ApiFailure::from_diagnostic)?;
    let relationship = read
        .snapshot
        .hierarchy
        .relationship(document)
        .map_err(ApiFailure::from_diagnostic)?;
    Ok(DocumentSummaryDto {
        id: document.id().to_string(),
        document_type: document.doc_type().to_string(),
        kind: document.kind().map(str::to_string),
        role: role.map(TaskRole::as_str),
        title: document.title().to_string(),
        location: document.location.as_str(),
        state: document.field("state").map(str::to_string),
        priority: document.field("priority").map(str::to_string),
        effort: document.field("effort").map(str::to_string),
        assignee: document.field("assignee").map(str::to_string),
        parent_id: document.field("parentId").map(str::to_string),
        parent_relationship: relationship.map(ParentRelationship::as_str),
        tags: values(document, "tags"),
        accord_status: accord_status(document).map(str::to_string),
        review_status: review_status(document).map(str::to_string),
    })
}

fn detail_dto(read: &ReadSnapshot, document: &Document) -> Result<DocumentDetailDto, ApiFailure> {
    let summary = summary_dto(read, document)?;
    let parent = document
        .field("parentId")
        .and_then(|id| read.snapshot.document(id))
        .map(|parent| summary_dto(read, &parent))
        .transpose()?;
    let children = read
        .snapshot
        .children(document)
        .map_err(ApiFailure::from_cli)?
        .iter()
        .map(|child| summary_dto(read, child))
        .collect::<Result<Vec<_>, _>>()?;
    let completion = (document.location == DocumentLocation::Logs && document.doc_type() == "task")
        .then(|| CompletionDto {
            outcome: completion_outcome(document).to_string(),
            summary: completion_summary(document).map(str::to_string),
            files_changed: completion_files_changed(document),
            validation: completion_validation(document).map(str::to_string),
            reviewer: completion_reviewer(document).map(str::to_string),
        });
    let accord = accord_status(document).map(|_| {
        AccordDto::from(AccordRecord::from_document(
            document,
            document.field("updatedAt").unwrap_or(""),
        ))
    });
    let review = review_status(document).map(|status| ReviewDto {
        status: status.to_string(),
        reviewer: document.field("review.reviewer").map(str::to_string),
        requested_at: document.field("review.requestedAt").map(str::to_string),
        decided_at: document.field("review.decidedAt").map(str::to_string),
        note: document
            .field("review.note")
            .or_else(|| document.field("review.reason"))
            .map(str::to_string),
    });
    Ok(DocumentDetailDto {
        summary,
        body: document.body.clone(),
        body_html: render_markdown(&document.body),
        due_date: document.field("dueDate").map(str::to_string),
        created_at: document.field("createdAt").map(str::to_string),
        updated_at: document.field("updatedAt").map(str::to_string),
        completed_at: document.field("completedAt").map(str::to_string),
        blockers: values(document, "blockers"),
        references: values(document, "references"),
        related_files: values(document, "relatedFiles"),
        parent: parent.map(Box::new),
        children,
        accord,
        review,
        completion,
        decision: (document.doc_type() == "decision").then(|| decision_dto(document)),
    })
}

fn log_summary_dto(document: &Document) -> LogSummaryDto {
    LogSummaryDto {
        id: document.id().to_string(),
        document_type: document.doc_type().to_string(),
        title: document.title().to_string(),
        completed_at: document.field("completedAt").map(str::to_string),
        outcome: completion_outcome(document).to_string(),
        summary: completion_summary(document).map(str::to_string),
        validation: completion_validation(document).map(str::to_string),
    }
}

fn decision_dto(document: &Document) -> DecisionDto {
    DecisionDto {
        id: document.id().to_string(),
        title: document.title().to_string(),
        status: document.field("status").map(str::to_string),
        date: document.field("date").map(str::to_string),
        deciders: values(document, "deciders"),
        context: document.field("context").map(str::to_string),
        consequences: values(document, "consequences"),
        alternatives: values(document, "alternatives"),
        supersedes: values(document, "supersedes"),
        superseded_by: values(document, "supersededBy"),
        summary: document
            .body
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string),
    }
}

/// Render a deliberately small, safe Markdown subset for the browser peer.
///
/// The web adapter receives an already separated document body. It does not
/// parse Tandem files or infer protocol meaning. Project-controlled text is
/// always escaped, and the renderer never emits links, images, or raw HTML.
fn render_markdown(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_code = false;
    let mut in_list = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            if in_code {
                html.push_str("</code></pre>");
            } else {
                html.push_str("<pre><code>");
            }
            in_code = !in_code;
            continue;
        }
        if in_code {
            escape_html_into(line, &mut html);
            html.push('\n');
            continue;
        }
        if trimmed.is_empty() {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            continue;
        }
        if let Some(item) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str("<li>");
            escape_html_into(item, &mut html);
            html.push_str("</li>");
            continue;
        }
        if in_list {
            html.push_str("</ul>");
            in_list = false;
        }
        let hashes = trimmed.bytes().take_while(|byte| *byte == b'#').count();
        if (1..=6).contains(&hashes) && trimmed.as_bytes().get(hashes) == Some(&b' ') {
            let content = &trimmed[hashes + 1..];
            html.push_str(&format!("<h{hashes}>"));
            escape_html_into(content, &mut html);
            html.push_str(&format!("</h{hashes}>"));
        } else if let Some(quote) = trimmed.strip_prefix("> ") {
            html.push_str("<blockquote><p>");
            escape_html_into(quote, &mut html);
            html.push_str("</p></blockquote>");
        } else {
            html.push_str("<p>");
            escape_html_into(trimmed, &mut html);
            html.push_str("</p>");
        }
    }
    if in_list {
        html.push_str("</ul>");
    }
    if in_code {
        html.push_str("</code></pre>");
    }
    html
}

fn escape_html_into(value: &str, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            character => output.push(character),
        }
    }
}

fn values(document: &Document, key: &str) -> Vec<String> {
    document
        .field(key)
        .map(parse_field_values)
        .unwrap_or_default()
}

#[derive(Debug)]
struct ApiFailure {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiFailure {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_request",
            message: message.into(),
        }
    }

    fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.into(),
        }
    }

    fn from_cli(_error: CliError) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "query_failed",
            message: "the canonical project query failed".to_string(),
        }
    }

    fn from_diagnostic(_error: crate::protocol::diagnostic::Diagnostic) -> Self {
        Self::from_cli(CliError::user("diagnostic"))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Envelope<T> {
    ok: bool,
    data: T,
    revision: String,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEnvelope {
    ok: bool,
    error: ErrorDto,
    revision: String,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorDto {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDto {
    title: String,
    protocol_version: String,
    states: Vec<String>,
    read_only: bool,
    health: HealthDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthDto {
    status: &'static str,
    board_documents: usize,
    tasks: usize,
    decisions: usize,
    logs: usize,
    by_state: BTreeMap<String, usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BoardDto {
    states: Vec<String>,
    items: Vec<DocumentSummaryDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AttentionDto {
    items: Vec<DocumentSummaryDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentSummaryDto {
    id: String,
    #[serde(rename = "type")]
    document_type: String,
    kind: Option<String>,
    role: Option<&'static str>,
    title: String,
    location: &'static str,
    state: Option<String>,
    priority: Option<String>,
    effort: Option<String>,
    assignee: Option<String>,
    parent_id: Option<String>,
    parent_relationship: Option<&'static str>,
    tags: Vec<String>,
    accord_status: Option<String>,
    review_status: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentDetailDto {
    #[serde(flatten)]
    summary: DocumentSummaryDto,
    body: String,
    body_html: String,
    due_date: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    completed_at: Option<String>,
    blockers: Vec<String>,
    references: Vec<String>,
    related_files: Vec<String>,
    parent: Option<Box<DocumentSummaryDto>>,
    children: Vec<DocumentSummaryDto>,
    accord: Option<AccordDto>,
    review: Option<ReviewDto>,
    completion: Option<CompletionDto>,
    decision: Option<DecisionDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccordDto {
    status: String,
    assignee: Option<String>,
    claimed_at: Option<String>,
    delivered_at: Option<String>,
    deliverables: Vec<String>,
    validations: Vec<String>,
    constraints: Vec<String>,
    summary: Option<String>,
    evidence: Vec<String>,
    files_changed: Vec<String>,
    reviewer: Option<String>,
    note: Option<String>,
    reason: Option<String>,
}

impl From<AccordRecord> for AccordDto {
    fn from(record: AccordRecord) -> Self {
        Self {
            status: record.status,
            assignee: record.assignee,
            claimed_at: record.claimed_at,
            delivered_at: record.delivered_at,
            deliverables: record.deliverables,
            validations: record.validations,
            constraints: record.constraints,
            summary: record.summary,
            evidence: record.evidence,
            files_changed: record.files_changed,
            reviewer: record.reviewer,
            note: record.note,
            reason: record.reason,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewDto {
    status: String,
    reviewer: Option<String>,
    requested_at: Option<String>,
    decided_at: Option<String>,
    note: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletionDto {
    outcome: String,
    summary: Option<String>,
    files_changed: Vec<String>,
    validation: Option<String>,
    reviewer: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LogsDto {
    total: usize,
    limit: usize,
    items: Vec<LogSummaryDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LogSummaryDto {
    id: String,
    #[serde(rename = "type")]
    document_type: String,
    title: String,
    completed_at: Option<String>,
    outcome: String,
    summary: Option<String>,
    validation: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RulesDto {
    categories: BTreeMap<String, Vec<RuleDto>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleDto {
    id: usize,
    rule: String,
    source: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DecisionsDto {
    items: Vec<DecisionDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DecisionDto {
    id: String,
    title: String,
    status: Option<String>,
    date: Option<String>,
    deciders: Vec<String>,
    context: Option<String>,
    consequences: Vec<String>,
    alternatives: Vec<String>,
    supersedes: Vec<String>,
    superseded_by: Vec<String>,
    summary: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    use axum::body::to_bytes;
    use axum::http::Request;
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    const TEST_HOST: &str = "127.0.0.1:43123";

    fn test_project() -> (PathBuf, TandemProject) {
        let root = std::env::temp_dir().join(format!(
            "tandem-web-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            &crate::protocol::config::default_project_config("Web test"),
        )
        .unwrap();
        fs::write(
            project.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\ntitle: Validate API\nstate: validation\npriority: high\ntags: [web]\naccord:\n  status: delivered\n  assignee: worker\n  summary: Ready to inspect\n  validation:\n    commands: [cargo test]\nreview:\n  status: pending\n  reviewer: owner\n---\n\n## Body\n",
        )
        .unwrap();
        fs::write(
            project.board_dir.join("decision-1.md"),
            "---\nid: decision-1\ntype: decision\ntitle: Use Axum\nstatus: accepted\ndate: 2026-08-05\n---\n\nDecision body\n",
        )
        .unwrap();
        fs::write(
            project.logs_dir.join("task-2.md"),
            "---\nid: task-2\ntype: task\ntitle: Finished\ncompletedAt: 2026-08-05T00:00:00Z\ncompletion:\n  outcome: completed\n  summary: Done\n---\n",
        )
        .unwrap();
        (root, project)
    }

    async fn json_request(app: Router, uri: &str) -> (StatusCode, Value) {
        let response = app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header(header::HOST, TEST_HOST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn read_routes_return_versioned_envelopes_and_canonical_relationship_data() {
        let (root, project) = test_project();
        let app = router(project, TEST_HOST);
        for uri in [
            "/api/v1/project",
            "/api/v1/board?state=validation",
            "/api/v1/attention",
            "/api/v1/documents/task-1",
            "/api/v1/logs?limit=10",
            "/api/v1/logs/task-2",
            "/api/v1/rules",
            "/api/v1/decisions",
            "/api/v1/decisions/decision-1",
        ] {
            let (status, value) = json_request(app.clone(), uri).await;
            assert_eq!(status, StatusCode::OK, "{uri}: {value}");
            assert_eq!(value["ok"], true);
            assert!(value["revision"].as_str().unwrap().starts_with("r1-"));
            assert!(value["warnings"].is_array());
        }
        let (_, detail) = json_request(app, "/api/v1/documents/task-1").await;
        assert_eq!(detail["data"]["role"], "task");
        assert_eq!(detail["data"]["accordStatus"], "delivered");
        assert_eq!(detail["data"]["accord"]["assignee"], "worker");
        assert_eq!(detail["data"]["accord"]["validations"][0], "cargo test");
        assert_eq!(detail["data"]["review"]["status"], "pending");
        assert_eq!(detail["data"]["review"]["reviewer"], "owner");
        assert_eq!(detail["data"]["body"], "\n## Body\n");
        assert_eq!(detail["data"]["bodyHtml"], "<h2>Body</h2>");
        assert!(detail["data"].get("path").is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn bundled_interface_has_semantic_landmarks_and_module_assets() {
        let (root, project) = test_project();
        let app = router(project, TEST_HOST);
        for (uri, content_type, evidence) in [
            ("/", "text/html", "<main id=\"content\""),
            ("/assets/app.css", "text/css", ":focus-visible"),
            ("/assets/app.js", "text/javascript", "renderRoute"),
            ("/assets/api.js", "text/javascript", "'/api/v1'"),
            ("/assets/ui.js", "text/javascript", "renderBoard"),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(uri)
                        .header(header::HOST, TEST_HOST)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            assert!(response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with(content_type));
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert!(String::from_utf8_lossy(&body).contains(evidence), "{uri}");
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn every_response_has_restrictive_browser_security_headers_and_no_cors() {
        let (root, project) = test_project();
        let app = router(project, TEST_HOST);
        for uri in ["/", "/assets/app.js", "/api/v1/project", "/missing"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(uri)
                        .header(header::HOST, TEST_HOST)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let headers = response.headers();
            assert_eq!(headers[header::CACHE_CONTROL], "no-store", "{uri}");
            assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff", "{uri}");
            assert_eq!(headers[header::X_FRAME_OPTIONS], "DENY", "{uri}");
            assert_eq!(headers[header::REFERRER_POLICY], "no-referrer", "{uri}");
            let csp = headers[header::CONTENT_SECURITY_POLICY].to_str().unwrap();
            assert!(csp.contains("default-src 'self'"), "{uri}: {csp}");
            assert!(csp.contains("frame-ancestors 'none'"), "{uri}: {csp}");
            assert!(
                headers.get("access-control-allow-origin").is_none(),
                "{uri}"
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn host_method_target_and_body_safeguards_reject_bad_requests() {
        let (root, project) = test_project();
        let app = router(project, TEST_HOST);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/project")
                    .header(header::HOST, "attacker.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/project")
                    .header(header::HOST, TEST_HOST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

        let long_target = format!(
            "/api/v1/logs?query={}",
            "x".repeat(MAX_REQUEST_TARGET_BYTES)
        );
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(long_target)
                    .header(header::HOST, TEST_HOST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::URI_TOO_LONG);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/project")
                    .header(header::HOST, TEST_HOST)
                    .header(header::CONTENT_LENGTH, "1")
                    .body(Body::from("x"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bundled_script_polls_only_visible_pages_and_refetches_changed_revisions() {
        assert!(APP_JS.contains("const REVISION_POLL_MS = 3000"));
        assert!(APP_JS.contains("document.hidden"));
        assert!(APP_JS.contains("visibilitychange"));
        assert!(APP_JS.contains("envelope.revision === state.revision"));
        assert!(APP_JS.contains("renderRoute({ preserveViewport: true, changed: true })"));
        assert!(APP_JS.contains("hashchange', () => renderRoute({ focusHeading: true })"));
        assert!(APP_JS.contains("else if (focusHeading)"));
        assert!(APP_JS.contains("skipLink.addEventListener('click'"));
        assert!(APP_JS.contains("main.focus()"));
        assert!(APP_JS.contains("captureTransientState"));
    }

    #[test]
    fn markdown_rendering_is_styled_but_never_emits_project_html() {
        let rendered = render_markdown(
            "# Heading\n\n- one\n- <script>alert('x')</script>\n\n[remote](https://example.com)\n\n![pixel](https://example.com/pixel)\n\n```html\n<img src=x>\n```\n",
        );
        assert!(rendered.contains("<h1>Heading</h1>"));
        assert!(rendered.contains("<ul><li>one</li>"));
        assert!(rendered.contains("&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"));
        assert!(rendered.contains("&lt;img src=x&gt;"));
        assert!(!rendered.contains("<script>"));
        assert!(!rendered.contains("<img"));
        assert!(!rendered.contains("href=\""));
        assert!(!rendered.contains("src=\""));
    }

    #[tokio::test]
    async fn route_errors_are_safe_and_machine_readable() {
        let (root, project) = test_project();
        let app = router(project, TEST_HOST);
        let (status, missing) = json_request(app.clone(), "/api/v1/documents/task-999").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(missing["error"]["code"], "document_not_found");
        assert!(missing["revision"].as_str().unwrap().starts_with("r1-"));

        let (status, invalid) = json_request(app.clone(), "/api/v1/logs?limit=0").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(invalid["error"]["code"], "invalid_request");

        let (status, mutation) = json_request(app, "/api/v1/tasks?token=do-not-reflect").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(mutation["error"]["code"], "route_not_found");
        assert!(!mutation.to_string().contains("do-not-reflect"));
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn project_read_failures_do_not_expose_source_paths() {
        let (root, project) = test_project();
        let app = router(project.clone(), TEST_HOST);
        fs::write(project.board_dir.join("task-1.md"), "not frontmatter").unwrap();
        let (status, error) = json_request(app, "/api/v1/project").await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error["error"]["code"], "project_read_failed");
        assert_eq!(
            error["error"]["message"],
            "the project snapshot could not be loaded"
        );
        assert!(!error.to_string().contains(root.to_str().unwrap()));
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn revision_changes_after_project_content_changes() {
        let (root, project) = test_project();
        let app = router(project.clone(), TEST_HOST);
        let (_, before) = json_request(app.clone(), "/api/v1/project").await;
        fs::write(
            project.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\ntitle: Changed\nstate: validation\n---\n",
        )
        .unwrap();
        let (_, after) = json_request(app, "/api/v1/project").await;
        assert_ne!(before["revision"], after["revision"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn startup_options_control_browser_behavior() {
        assert!(should_open_browser(Options {
            port: None,
            no_open: false
        }));
        assert!(!should_open_browser(Options {
            port: Some(8080),
            no_open: true
        }));
    }

    #[tokio::test]
    async fn loopback_listener_selects_an_available_port() {
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let SocketAddr::V4(address) = listener.local_addr().unwrap() else {
            panic!("expected IPv4 loopback listener")
        };
        assert!(address.ip().is_loopback());
        assert_ne!(address.port(), 0);
    }
}
