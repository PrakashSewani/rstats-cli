use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rstats"))
        .args(args)
        .output()
        .expect("failed to run rstats binary")
}

#[test]
fn once_json_emits_a_parseable_snapshot() {
    let output = run(&["--once", "--json"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout is a JSON snapshot");
    assert!(value.get("cpu").is_some());
    assert!(value.get("memory").is_some());
    assert!(value.get("swap").is_some());
    assert!(value.get("disks").is_some());
    assert!(value.get("networks").is_some());
    assert!(value.get("timestamp").is_some());
}

#[test]
fn once_text_prints_a_summary() {
    let output = run(&["--once"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cpu"));
    assert!(stdout.contains("memory"));
    assert!(stdout.contains("swap"));
    assert!(stdout.contains("top processes by cpu"));
}
