use std::io::Write;

use anyhow::Result;
use assert_cmd::Command;
use indoc::indoc;
use tempfile::NamedTempFile;

#[test]
fn no_such_file() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.arg("/foo/bar");
    cmd.assert().failure().stderr(indoc! {r#"
        Error: failed to open file: /foo/bar

        Caused by:
            No such file or directory (os error 2)
    "#});

    Ok(())
}

#[test]
fn from_stdin() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.write_stdin(indoc! {"
        --- context ---
        x: echo 42
        --- template ---
        x = {{x}}
    "});

    cmd.assert().stdout("x = 42\n");

    Ok(())
}

#[test]
fn from_file() -> Result<()> {
    let mut file = NamedTempFile::new()?;

    let src = indoc! {"
        --- context ---
        x: echo 42
        --- template ---
        x = {{x}}
    "};

    write!(file, "{}", src)?;

    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.arg(file.path());
    cmd.assert().stdout("x = 42\n");

    Ok(())
}

#[test]
fn stderr() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.write_stdin(indoc! {"
        --- context ---
        |
          echo world
          echo foo >&2
        --- template ---
        hello, {{this}}
    "});

    cmd.assert().stdout("hello, world\n").stderr("foo\n");

    Ok(())
}

#[test]
fn script_nonzero() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.write_stdin(indoc! {"
        --- context ---
        |
          echo world
          echo foo >&2
          exit 42
        --- template ---
        hello, {{this}}
    "});

    cmd.assert().failure().stderr(indoc! {"
        Error: failed to execute script

        ---- script ----
        echo world
        echo foo >&2
        exit 42

        ---- stdout ----
        world

        ---- stderr ----
        foo

        Caused by:
            process exited unsuccessfully: exit status: 42
    "});

    Ok(())
}

#[test]
fn script_killed() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.write_stdin(indoc! {"
        --- context ---
        |
          echo world
          echo foo >&2
          kill $$
        --- template ---
        hello, {{this}}
    "});

    cmd.assert().failure().stderr(indoc! {"
        Error: failed to execute script

        ---- script ----
        echo world
        echo foo >&2
        kill $$

        ---- stdout ----
        world

        ---- stderr ----
        foo

        Caused by:
            process exited unsuccessfully: signal: 15
    "});

    Ok(())
}

#[test]
fn no_context_block() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.write_stdin(indoc! {"
        --- template ---
        hey
    "});

    cmd.assert().success().stdout("hey\n");

    Ok(())
}

#[test]
fn no_template_block() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.write_stdin(indoc! {"
        --- context ---
        true
    "});

    cmd.assert().failure().stderr(indoc! {"
        Error: no template block found

        Example:

            --- context ---
            to: echo world

            --- template ---
            hello, {{to}}
    "});

    Ok(())
}

#[test]
fn invalid_yaml() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.write_stdin(indoc! {"
        --- context ---
        &
        --- template ---
    "});

    cmd.assert().failure().stderr(indoc! {"
        Error: error while parsing context as YAML

        --- context ---
        &

        Caused by:
            0: while scanning an anchor or alias, did not find expected alphabetic or numeric character at line 1 column 1
            1: while scanning an anchor or alias, did not find expected alphabetic or numeric character at line 1 column 1
    "});

    Ok(())
}

#[test]
fn invalid_haml() -> Result<()> {
    let mut cmd = Command::cargo_bin("handlematters")?;

    cmd.write_stdin(indoc! {"
        --- context ---
        true
        --- template ---
        {{
    "});

    cmd.assert().failure().stderr(indoc! {r#"
        Error: Template error: invalid handlebars syntax.
            --> Template error in "Unnamed template":2:1
             |
           0 | {{
             |
             = reason: invalid handlebars syntax.

    "#});

    Ok(())
}
