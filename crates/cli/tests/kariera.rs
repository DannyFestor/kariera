use std::io;
use std::process::{Command, Output};

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
