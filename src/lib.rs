#![feature(exit_status_error)]

mod evaluate_context;
mod extract_blocks;

use std::io::{self, BufWriter, Read};

use anyhow::{Context as _, Result};
use handlebars::{Handlebars, Template};
use indoc::{formatdoc, indoc};
use serde_yaml::Value;

use evaluate_context::evaluate_context;
use extract_blocks::extract_blocks;

pub fn run<T: Read>(reader: &mut T) -> Result<()> {
    let registry = extract_blocks(reader)?;

    let ctx = parse_context(registry.context())?;
    let ctx = evaluate_context(ctx)?;

    let mut hbs = Handlebars::new();
    let tpl = parse_template(registry.template())?;

    hbs.register_template("main", tpl);

    let stdout = BufWriter::new(io::stdout());

    hbs.render_to_write("main", &ctx, stdout)?;

    Ok(())
}

fn parse_context(src: Option<&String>) -> Result<Value> {
    if let Some(src) = src {
        serde_yaml::from_str(src).with_context(|| {
            formatdoc! {"
                error while parsing context as YAML

                --- context ---
                {}", chomp(src)
            }
        })
    } else {
        Ok(Value::Null)
    }
}

fn parse_template(src: Option<&String>) -> Result<Template> {
    let src = src.with_context(|| {
        indoc! {"
            no template block found

            Example:

                --- context ---
                to: echo world

                --- template ---
                hello, {{to}}"
        }
    })?;

    Ok(Template::compile(src)?)
}

fn chomp(s: &str) -> &str {
    s.strip_suffix('\n').unwrap_or(s)
}
