#![feature(exit_status_error)]

mod source;

use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{Context as _, Result};
use handlebars::{Context, Handlebars, RenderContext, Renderable};
use indoc::formatdoc;
use serde_yaml::{Mapping, Sequence, Value};
use source::Source;

pub fn render(src: &str, on_stderr: fn(&str)) -> Result<String> {
    let src = Source::parse(src)?;
    let ctx = evaluate_context(src.context, on_stderr)?;

    let output = src.template.renders(&Handlebars::new(), &Context::wraps(ctx)?, &mut RenderContext::new(None))?;

    Ok(output)
}

fn evaluate_context(ctx: Value, on_stderr: fn(&str)) -> Result<Value> {
    match ctx {
        Value::String(s) => {
            let (out, err) = run_script(&s)?;

            if !err.is_empty() {
                on_stderr(&err);
            }

            Ok(Value::String(out))
        }
        Value::Sequence(seq) => {
            let seq: Sequence = seq.into_iter().map(|v| evaluate_context(v, on_stderr)).collect::<Result<_>>()?;

            Ok(Value::Sequence(seq))
        }
        Value::Mapping(map) => {
            let map: Mapping = map.into_iter().map(|(k, v)| Ok((k, evaluate_context(v, on_stderr)?))).collect::<Result<_>>()?;

            Ok(Value::Mapping(map))
        }
        v => Ok(v),
    }
}

fn run_script(script: &str) -> Result<(String, String)> {
    let mut shell = Command::new("sh")
        .args(["-s", "-e"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

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

    let stdout = chomp(&stdout).to_string();

    Ok((stdout, stderr))
}

fn chomp(s: &str) -> &str {
    s.strip_suffix('\n').unwrap_or(s)
}
