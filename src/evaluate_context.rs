use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{Context as _, Result};
use indoc::formatdoc;
use serde_yaml::Value;

use crate::chomp;

pub fn evaluate_context(ctx: Value) -> Result<Value> {
    match ctx {
        Value::String(s) => {
            let output = run_script(&s)?;

            Ok(Value::String(chomp(&output).to_string()))
        }
        Value::Sequence(seq) => {
            let seq = seq.into_iter().map(evaluate_context).collect::<Result<_>>()?;

            Ok(Value::Sequence(seq))
        }
        Value::Mapping(map) => {
            let map = map.into_iter().map(|(k, v)| Ok((k, evaluate_context(v)?))).collect::<Result<_>>()?;

            Ok(Value::Mapping(map))
        }
        v => Ok(v),
    }
}

fn run_script(script: &str) -> Result<String> {
    let mut shell = Command::new("sh")
        .args(["-s", "-e"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| "failed to start shell: sh")?;

    shell.stdin.as_mut().unwrap().write_all(script.as_bytes())?;

    let output = shell.wait_with_output()?;
    let [stdout, stderr] = [output.stdout, output.stderr].map(|io| String::from_utf8_lossy(&io).to_string());

    output.status.exit_ok().with_context(|| {
        formatdoc! {"
            failed to execute script

            ---- script ----
            {}

            ---- stdout ----
            {}

            ---- stderr ----
            {}", chomp(script), chomp(&stdout), chomp(&stderr)
        }
    })?;

    eprint!("{stderr}");

    Ok(stdout)
}
