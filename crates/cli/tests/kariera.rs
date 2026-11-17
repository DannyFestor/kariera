use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "../../vault/tests/support/vault_copy.rs"]
mod vault_copy;

// clippy.toml allows expect inside #[test] functions only, so the helpers return errors for the tests to expect.
fn kariera(arguments: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kariera"))
        .args(arguments)
        .output()
}

#[test]
fn version_goes_to_stdout_and_exits_zero() {
    let output = kariera(&["--version"]).expect("kariera runs");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        format!("kariera {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is UTF-8"),
        ""
    );
}

#[test]
fn an_unknown_argument_goes_to_stderr_and_exits_two() {
    let output = kariera(&["build"]).expect("kariera runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr is UTF-8")
            .contains("\"build\"")
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        ""
    );
}

/// A home directory whose kariera config names a fresh copy of the fixture vault.
fn home_with_vault(test: &str) -> io::Result<(PathBuf, PathBuf)> {
    let vault = vault_copy::copy_of_fixture_vault(test)?;
    let home = vault.with_extension("home");
    let config = home.join(".config/kariera");
    fs::create_dir_all(&config)?;
    fs::write(
        config.join("config.toml"),
        format!("vault = {:?}\n", vault.display().to_string()),
    )?;
    Ok((home, vault))
}

fn kariera_at(home: &Path, arguments: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_kariera"))
        .env("HOME", home)
        .args(arguments)
        .output()
}

#[test]
fn new_log_creates_the_note_in_the_configured_vault() {
    let (home, vault) = home_with_vault("cli_new_log").unwrap();

    let output = kariera_at(
        &home,
        &[
            "new",
            "log",
            "--engagement",
            "2023-10 Globex",
            "--month",
            "2026-11",
        ],
    )
    .unwrap();

    let note = vault.join("Monthly Logs/2026/2026-11 Globex Log.md");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Created {}\n", note.display())
    );
    assert!(
        fs::read_to_string(note)
            .unwrap()
            .contains("engagement: \"[[2023-10 Globex]]\"\nmonth: 2026-11\n")
    );
}

#[test]
fn new_refuses_to_overwrite_and_exits_one() {
    let (home, _) = home_with_vault("cli_new_overwrite").unwrap();
    let arguments = [
        "new",
        "log",
        "--engagement",
        "2023-10 Globex",
        "--month",
        "2025-06",
    ];

    let output = kariera_at(&home, &arguments).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "Monthly Logs/2025/2025-06 Globex Log.md already has that name, ignoring case\n"
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
}

#[test]
fn new_with_bad_arguments_prints_the_usage_and_exits_two() {
    let output = kariera(&["new", "log", "--month", "2026-11"]).unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("kariera: --engagement is required\n\nUsage: kariera"),
        "{stderr}"
    );
}

#[test]
fn new_without_a_config_says_so_and_exits_one() {
    let empty_home = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli_no_config_home");
    fs::create_dir_all(&empty_home).unwrap();

    let output = kariera_at(
        &empty_home,
        &[
            "new",
            "project",
            "--engagement",
            "2023-10 Globex",
            "--title",
            "Billing",
        ],
    )
    .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("cannot read "),
    );
}
