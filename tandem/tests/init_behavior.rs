//! Regression coverage for `tandem init` process output contracts.
//!
//! Every test runs in a unique temporary directory and never touches the real
//! project records.
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

fn root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-cli-init-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn init(root: &std::path::Path, args: &[&str]) -> Output {
    bin().current_dir(root).args(args).output().unwrap()
}

#[test]
fn init_json_fresh_success_emits_single_success_envelope() {
    let root_dir = root("json-success");
    std::fs::create_dir_all(&root_dir).unwrap();

    let output = init(&root_dir, &["init", "--title", "probe", "--json"]);
    assert!(
        output.status.success(),
        "fresh init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "ordinary success must not write stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.lines().count(),
        1,
        "expected exactly one envelope line, got: {stdout:?}"
    );
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("stdout is not one JSON value ({error}): {stdout:?}"));
    assert_eq!(value["ok"], serde_json::json!(true));
    assert_eq!(value["data"]["title"], serde_json::json!("probe"));
    assert_eq!(
        value["data"]["root"],
        serde_json::json!(std::fs::canonicalize(&root_dir)
            .unwrap()
            .display()
            .to_string())
    );
    assert_eq!(value["warnings"], serde_json::json!([]));
    assert!(root_dir.join(".tandem/tandem.md").is_file());
}

#[test]
fn init_json_without_title_reports_resolved_default_title() {
    let root_dir = root("json-default-title");
    std::fs::create_dir_all(&root_dir).unwrap();

    let output = init(&root_dir, &["init", "--json"]);
    assert!(
        output.status.success(),
        "fresh init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert_eq!(value["ok"], serde_json::json!(true));
    assert_eq!(
        value["data"]["title"],
        serde_json::json!(root_dir.file_name().unwrap().to_string_lossy().to_string())
    );
    assert_eq!(value["warnings"], serde_json::json!([]));
}

#[test]
fn init_without_json_retains_quiet_success() {
    let root_dir = root("quiet-success");
    std::fs::create_dir_all(&root_dir).unwrap();

    let output = init(&root_dir, &["init", "--title", "quiet"]);
    assert!(
        output.status.success(),
        "fresh init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stdout.is_empty(),
        "non-JSON init must stay quiet: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn repeated_init_json_retains_error_envelope_and_existing_config() {
    let root_dir = root("repeat-failure");
    std::fs::create_dir_all(&root_dir).unwrap();

    let first = init(&root_dir, &["init", "--title", "probe", "--json"]);
    assert!(first.status.success());
    let config_path = root_dir.join(".tandem/tandem.md");
    let before = std::fs::read(&config_path).unwrap();

    let output = init(&root_dir, &["init", "--title", "probe", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stderr.is_empty(),
        "JSON failure must not write stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["ok"], serde_json::json!(false));
    assert_eq!(value["error"]["code"], serde_json::json!("io"));
    assert!(
        value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("already exists"),
        "unexpected error message: {stdout}"
    );
    assert_eq!(
        std::fs::read(&config_path).unwrap(),
        before,
        "a failed repeat init must not rewrite the existing config"
    );
}
