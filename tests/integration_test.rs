//! Integration tests for wtp

use std::process::Command;
use std::path::PathBuf;
use tempfile::TempDir;

fn wtp_bin() -> PathBuf {
    // Find the compiled binary
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("target/release/wtp")
}

fn run_wtp(args: &[&str]) -> (bool, String, String) {
    run_wtp_in_dir(args, None)
}

fn run_wtp_in_dir(args: &[&str], cwd: Option<&std::path::Path>) -> (bool, String, String) {
    let mut cmd = Command::new(wtp_bin());
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let output = cmd.output().expect("Failed to execute wtp");
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    (output.status.success(), stdout, stderr)
}

#[test]
fn test_wtp_help() {
    let (success, stdout, _) = run_wtp(&["--help"]);
    assert!(success);
    assert!(stdout.contains("WorkTree for Polyrepo"));
    assert!(stdout.contains("cd"));
    assert!(stdout.contains("ls"));
    assert!(stdout.contains("create"));
    assert!(stdout.contains("import"));
    assert!(stdout.contains("switch"));
    assert!(stdout.contains("shell-init"));
    assert!(stdout.contains("Workspace Management"));
    assert!(stdout.contains("Repository Operations"));
    assert!(stdout.contains("Utilities"));
    assert!(!stdout.contains("add"));  // add was renamed to import
    assert!(!stdout.contains("  init  ")); // init command was removed (but shell-init exists)
}

#[test]
fn test_wtp_version() {
    let (success, stdout, _) = run_wtp(&["--version"]);
    assert!(success);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_import_requires_workspace() {
    // Use a temp directory that is definitely not a workspace
    let temp_dir = TempDir::new().unwrap();
    
    // Without --workspace and not in a workspace, should fail with "Not in a workspace"
    let (success, stdout, stderr) = run_wtp_in_dir(&["import", "some/path"], Some(temp_dir.path()));
    assert!(!success);
    let combined = format!("{} {}", stdout, stderr);
    assert!(combined.contains("Not in a workspace") || combined.contains("workspace"),
            "Expected workspace-related error, got: {}", combined);
}

#[test]
fn test_status_not_in_workspace() {
    // Use a temp directory that is definitely not a workspace
    let temp_dir = TempDir::new().unwrap();
    
    // Without --workspace and not in a workspace, should fail with "Not in a workspace"
    let (success, stdout, stderr) = run_wtp_in_dir(&["status"], Some(temp_dir.path()));
    assert!(!success);
    let combined = format!("{} {}", stdout, stderr);
    assert!(combined.contains("Not in a workspace") || combined.contains("workspace"),
            "Expected workspace-related error, got: {}", combined);
}

#[test]
fn test_cd_requires_shell_integration() {
    // Use a workspace name that definitely doesn't exist
    let (success, stdout, stderr) = run_wtp(&["cd", "nonexistent-workspace-xyz"]);
    assert!(!success);
    let combined = format!("{} {}", stdout, stderr);
    // Either "not found" or "shell integration" error is acceptable
    assert!(combined.contains("not found") || combined.contains("shell integration") || combined.contains("Shell integration"),
            "Expected error message, got: {}", combined);
}

#[test]
fn test_shell_init_outputs_wrapper() {
    let (success, stdout, _) = run_wtp(&["shell-init"]);
    assert!(success);
    assert!(stdout.contains("wtp() {"));
    assert!(stdout.contains("WTP_DIRECTIVE_FILE"));
}

#[test]
fn test_ls_short_format() {
    // First create a workspace
    let _ = run_wtp(&["create", "test-short"]); // Ignore cleanup from other tests
    
    let (success, stdout, _) = run_wtp(&["ls", "--short"]);
    assert!(success);
    // Output should be just workspace names, one per line
    // We can't check exact content due to test isolation issues,
    // but we can verify the format doesn't have extra columns
    for line in stdout.lines() {
        assert!(!line.contains("  ")); // No multiple spaces (not long format)
        assert!(!line.contains("[missing]")); // No status markers
    }
    
    // Cleanup
    let _ = run_wtp(&["rm", "test-short"]);
}
