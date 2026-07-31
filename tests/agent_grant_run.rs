use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_parakeys"))
}

fn pk(args: &[&str]) -> std::process::Output {
    Command::new(bin()).args(args).output().unwrap()
}

#[test]
fn agent_run_only_injects_grant_keys() {
    let dir = std::env::temp_dir().join(format!(
        "pk-agent-run-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let p = dir.to_str().unwrap();

    assert!(pk(&["init", "--path", p]).status.success());
    assert!(pk(&["set", "--path", p, "GRANT_ONLY=grant-secret-value"]).status.success());
    assert!(pk(&["set", "--path", p, "HUMAN_ONLY=human-secret-value"]).status.success());
    assert!(pk(&["agent", "keygen", "--path", p]).status.success());
    let pub_path = dir.join(".parakeys-agent/agent.pub");
    assert!(pk(&[
        "grant",
        "create",
        "--path",
        p,
        "--to",
        pub_path.to_str().unwrap(),
        "--keys",
        "GRANT_ONLY",
        "--out",
        "g.enc"
    ])
    .status
    .success());
    assert!(pk(&["agent", "apply", "--path", p, "g.enc"]).status.success());

    let out = pk(&[
        "agent",
        "run",
        "--path",
        p,
        "--",
        "sh",
        "-c",
        "printf 'G=%s;H=%s' \"$GRANT_ONLY\" \"$HUMAN_ONLY\"",
    ]);
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("G=grant-secret-value"), "out={s}");
    // HUMAN_ONLY must not come from grant; may be empty if not in parent env
    assert!(!s.contains("human-secret-value"), "human secret leaked: {s}");

    let _ = fs::remove_dir_all(&dir);
}
