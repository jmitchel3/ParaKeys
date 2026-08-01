//! Integration tests drive the real `parakeys` binary entry point.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_parakeys"))
}

fn run_pk(args: &[&str]) -> std::process::Output {
    // File wallet keeps recovery wipe deterministic without Keychain side effects.
    Command::new(bin())
        .args(args)
        .env("PARAKEYS_FORCE_FILE_WALLET", "1")
        .output()
        .expect("spawn parakeys")
}

#[test]
fn init_import_run_and_recover() {
    let dir = std::env::temp_dir().join(format!(
        "parakeys-cli-it-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.to_str().unwrap();

    let init = run_pk(&["init", "--path", path]);
    assert!(
        init.status.success(),
        "init stderr={}",
        String::from_utf8_lossy(&init.stderr)
    );
    let stdout = String::from_utf8_lossy(&init.stdout);
    let combined = format!(
        "{}{}",
        stdout,
        String::from_utf8_lossy(&init.stderr)
    );
    let code = combined
        .lines()
        .map(str::trim)
        .skip_while(|l| !l.contains("RECOVERY CODE"))
        .nth(1)
        .filter(|l| l.contains('-') && l.len() > 20)
        .expect("recovery code line after RECOVERY CODE header")
        .to_string();

    assert!(
        dir.join(".parakeys/local.key").is_file(),
        "FORCE_FILE_WALLET should create local.key"
    );

    fs::write(
        dir.join(".env"),
        "DATABASE_URL=postgres://integration-secret\nAPI_KEY=sk-integration-test-key-123456\n",
    )
    .unwrap();

    let imp = run_pk(&["import", "--path", path, ".env"]);
    assert!(
        imp.status.success(),
        "import stderr={}",
        String::from_utf8_lossy(&imp.stderr)
    );
    let env_after = fs::read_to_string(dir.join(".env")).unwrap();
    assert!(env_after.contains("<set in parakeys>"));
    assert!(!env_after.contains("integration-secret"));
    assert!(!env_after.contains("sk-integration-test-key"));

    let run = run_pk(&[
        "run",
        "--path",
        path,
        "--",
        "sh",
        "-c",
        "printf '%s' \"$DATABASE_URL\"",
    ]);
    assert!(
        run.status.success(),
        "run stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "postgres://integration-secret"
    );

    let env_still = fs::read_to_string(dir.join(".env")).unwrap();
    assert!(!env_still.contains("integration-secret"));

    // Recover after wiping local key
    fs::remove_file(dir.join(".parakeys/local.key")).unwrap();
    let recover = run_pk(&["init", "--path", path, "--recover", &code]);
    assert!(
        recover.status.success(),
        "recover stderr={}",
        String::from_utf8_lossy(&recover.stderr)
    );
    let run2 = run_pk(&[
        "run",
        "--path",
        path,
        "--",
        "sh",
        "-c",
        "printf '%s' \"$API_KEY\"",
    ]);
    assert!(run2.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run2.stdout),
        "sk-integration-test-key-123456"
    );

    let _ = fs::remove_dir_all(&dir);
}
