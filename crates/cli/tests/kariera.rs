use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "../../vault/tests/support/replace_in.rs"]
mod replace_in_support;
#[path = "../../vault/tests/support/scratch.rs"]
mod scratch;
#[path = "../../vault/tests/support/vault_copy.rs"]
mod vault_copy;

use replace_in_support::replace_in;

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
    let output = kariera(&["publish"]).expect("kariera runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr is UTF-8")
            .contains("\"publish\"")
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
    let empty_home = scratch::fresh_directory("cli_no_config_home").unwrap();

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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const GO_ALIASES_AS_FLOW: &str = "aliases: [Golang]\n";

/// Two errors: a name that is not the file name, and a list in flow style.
fn break_two_notes(vault: &Path) -> io::Result<()> {
    replace_in(
        vault,
        "Skills/Go.md",
        "aliases:\n  - Golang\n",
        GO_ALIASES_AS_FLOW,
    )?;
    replace_in(
        vault,
        "Organizations/Globex.md",
        "name: Globex",
        "name: Globex Inc",
    )
}

#[test]
fn validate_says_so_when_a_vault_has_no_findings_and_exits_zero() {
    let (home, _) = home_with_vault("cli_validate_clean").unwrap();

    let output = kariera_at(&home, &["validate"]).unwrap();

    assert_eq!(stdout(&output), "No problems in 44 notes.\n");
    assert_eq!(stderr(&output), "");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn validate_lists_errors_per_note_and_exits_one() {
    let (home, vault) = home_with_vault("cli_validate_errors").unwrap();
    break_two_notes(&vault).unwrap();

    let output = kariera_at(&home, &["validate"]).unwrap();

    assert_eq!(
        stdout(&output),
        "Organizations/Globex.md: error [file-name] the note must be named `Globex Inc` (its field `name`), not `Globex`\n\
         Skills/Go.md:4: error [yaml-style] field `aliases` must be written in block style, not as `[…]` or `{…}`\n\
         2 errors, 0 warnings in 44 notes.\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn validate_measures_last_used_against_the_clock() {
    let (home, vault) = home_with_vault("cli_validate_last_used").unwrap();
    replace_in(
        &vault,
        "Skills/Docker.md",
        "last_used: 2025-08",
        "last_used: 9999-12",
    )
    .unwrap();

    let output = kariera_at(&home, &["validate"]).unwrap();

    assert!(
        stdout(&output).starts_with(
            "Skills/Docker.md:5: error [period] field `last_used` 9999-12 is after the current month ("
        ),
        "{}",
        stdout(&output)
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn validate_does_not_pluralize_one_error() {
    let (home, vault) = home_with_vault("cli_validate_one_error").unwrap();
    replace_in(&vault, "Skills/Go.md", "name: Go", "name: Golang").unwrap();

    let output = kariera_at(&home, &["validate"]).unwrap();

    assert!(
        stdout(&output).ends_with("1 error, 0 warnings in 44 notes.\n"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn validate_exits_zero_with_warnings_only() {
    let (home, vault) = home_with_vault("cli_validate_warning").unwrap();
    replace_in(
        &vault,
        "Monthly Logs/2025/2025-07 Globex Log.md",
        "\"Hours mismatch: the timesheet leaves out 2.25 h of planning that the invoice bills\"",
        "split by project",
    )
    .unwrap();

    let output = kariera_at(&home, &["validate"]).unwrap();

    assert_eq!(
        stdout(&output),
        "Monthly Logs/2025/2025-07 Globex Log.md:7: warning [hours] field `project_hours` adds up to 150.25 h, but `hours` is 152.5: a Source `note` starting with `Hours mismatch:` must explain the difference\n\
         0 errors, 1 warning in 44 notes.\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn validate_reports_a_note_the_reader_refuses_and_still_runs_the_other_rules() {
    let (home, vault) = home_with_vault("cli_validate_reader").unwrap();
    replace_in(
        &vault,
        "Skills/Go.md",
        "category: language",
        "category: dialect",
    )
    .unwrap();
    replace_in(
        &vault,
        "Organizations/Globex.md",
        "name: Globex",
        "name: Globex Inc",
    )
    .unwrap();

    let output = kariera_at(&home, &["validate"]).unwrap();

    assert_eq!(
        stdout(&output),
        "Organizations/Globex.md: error [file-name] the note must be named `Globex Inc` (its field `name`), not `Globex`\n\
         Skills/Go.md:6: error [schema] field `category` must be one of language, framework, library, database, cloud, platform, tool, ai-tool, method, human-language\n\
         Some notes could not be read, so the rules that need every note (a Contract listed by exactly one Engagement, exactly one Profile) did not run: fix the problems above first.\n\
         2 errors, 0 warnings in 43 notes.\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn validate_json_is_clean_for_a_clean_vault() {
    let (home, _) = home_with_vault("cli_validate_json_clean").unwrap();

    let output = kariera_at(&home, &["validate", "--json"]).unwrap();

    assert_eq!(
        stdout(&output),
        "{\"errors\":0,\"warnings\":0,\"notes_checked\":44,\"complete\":true,\"findings\":[]}\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn validate_json_lists_every_finding_with_its_parts() {
    let (home, vault) = home_with_vault("cli_validate_json_errors").unwrap();
    break_two_notes(&vault).unwrap();

    let output = kariera_at(&home, &["validate", "--json"]).unwrap();

    assert_eq!(
        stdout(&output),
        "{\"errors\":2,\"warnings\":0,\"notes_checked\":44,\"complete\":true,\"findings\":[\
         {\"path\":\"Organizations/Globex.md\",\"line\":null,\"severity\":\"error\",\"rule\":\"file-name\",\
         \"location\":{\"kind\":\"note\",\"name\":null},\
         \"message\":\"must be named `Globex Inc` (its field `name`), not `Globex`\"},\
         {\"path\":\"Skills/Go.md\",\"line\":4,\"severity\":\"error\",\"rule\":\"yaml-style\",\
         \"location\":{\"kind\":\"field\",\"name\":\"aliases\"},\
         \"message\":\"must be written in block style, not as `[…]` or `{…}`\"}]}\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn validate_json_names_a_section() {
    let (home, vault) = home_with_vault("cli_validate_json_section").unwrap();
    replace_in(
        &vault,
        "Engagements/2021-04 Acme.md",
        "Built backends",
        "Built [[Acme]] backends",
    )
    .unwrap();

    let output = kariera_at(&home, &["validate", "--json"]).unwrap();

    assert!(
        stdout(&output).contains("\"location\":{\"kind\":\"section\",\"name\":\"## Summary\"}"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn validate_without_a_config_says_so_and_exits_one() {
    let empty_home = scratch::fresh_directory("cli_validate_no_config_home").unwrap();

    let output = kariera_at(&empty_home, &["validate"]).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).starts_with("cannot read "),
        "{}",
        stderr(&output)
    );
}

#[test]
fn validate_with_an_unknown_option_prints_the_usage_and_exits_two() {
    let output = kariera(&["validate", "--xml"]).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).starts_with("kariera: unknown argument \"--xml\"\n\nUsage: kariera"),
        "{}",
        stderr(&output)
    );
}
