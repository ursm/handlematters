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
    let (ctx, tpl) = load_from(reader)?;
    let ctx = evaluate_context(ctx)?;
    let mut hbs = Handlebars::new();

    hbs.register_template("main", tpl);

    let stdout = BufWriter::new(io::stdout());

    hbs.render_to_write("main", &ctx, stdout)?;

    Ok(())
}

fn load_from<T: Read>(reader: &mut T) -> Result<(Value, Template)> {
    let registry = extract_blocks(reader)?;
    let null = "null".to_string();
    let context = registry.context().unwrap_or(&null);

    let context = serde_yaml::from_str(context).with_context(|| {
        formatdoc! {"
            error while parsing context as YAML

            --- context ---
            {}", chomp(context)
        }
    })?;

    let template = registry.template().with_context(|| {
        indoc! {"
            no template block found

            Example:

                --- context ---
                to: echo world

                --- template ---
                hello, {{to}}"
        }
    })?;

    let template = Template::compile(template)?;

    Ok((context, template))
}

fn chomp(s: &str) -> &str {
    s.strip_suffix('\n').unwrap_or(s)
}
