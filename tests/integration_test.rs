use std::io::Write;
use std::process::{Command, Stdio};
use std::str;

#[test]
fn test_binary_with_pipe_delimiter() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tabbs"))
        .args(["-c", "name|age", "-d", "|"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn tabbs");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"alice|30\nbob|25")
            .expect("failed to write to stdin");
    }

    let output = child.wait_with_output().expect("failed to get output");
    let stdout = str::from_utf8(&output.stdout).unwrap();
    let stderr = str::from_utf8(&output.stderr).unwrap();
    assert!(
        output.status.success(),
        "tabbs failed: {}\nstdout: {}\nstderr: {}",
        output.status,
        stdout,
        stderr
    );
    assert!(stdout.contains("| name  | age |"));
    assert!(stdout.contains("| alice | 30  |"));
    assert!(stdout.contains("| bob   | 25  |"));
}

#[test]
fn test_binary_empty_delimiter_errors() {
    let output = Command::new(env!("CARGO_BIN_EXE_tabbs"))
        .args(["-c", "a,b", "-d", ""])
        .stdin(Stdio::null())
        .output()
        .expect("failed to run tabbs");

    let stderr = str::from_utf8(&output.stderr).unwrap();
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(
        !output.status.success(),
        "expected tabbs to fail with empty delimiter\nstdout: {}\nstderr: {}",
        stdout,
        stderr
    );
    assert!(stderr.contains("delimiter cannot be empty"));
}

#[test]
fn test_binary_header_row() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tabbs"))
        .args(["--header-row", "-d", ","])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn tabbs");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"name,age\nalice,30\nbob,25")
            .expect("failed to write to stdin");
    }

    let output = child.wait_with_output().expect("failed to get output");
    let stdout = str::from_utf8(&output.stdout).unwrap();
    let stderr = str::from_utf8(&output.stderr).unwrap();
    assert!(
        output.status.success(),
        "tabbs failed: {}\nstdout: {}\nstderr: {}",
        output.status,
        stdout,
        stderr
    );
    assert!(stdout.contains("| name  | age |"));
    assert!(stdout.contains("| alice | 30  |"));
}

#[test]
fn test_binary_file_input() {
    use std::io::Write;

    let mut file = tempfile::NamedTempFile::new().expect("create temp file");
    file.write_all(b"a,b,c\n1,2,3\n4,5,6")
        .expect("write temp file");
    let path = file.path();

    let output = Command::new(env!("CARGO_BIN_EXE_tabbs"))
        .args(["-c", "a,b,c", "-f", path.to_str().unwrap()])
        .output()
        .expect("failed to run tabbs");

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(output.status.success());
    assert!(stdout.contains("| a | b | c |"));
    assert!(stdout.contains("| 1 | 2 | 3 |"));
}
