use anyhow::{Context as _, Result};
use handlebars::Template;
use indoc::{formatdoc, indoc};
use serde_yaml::Value;

pub struct Source {
    pub context: Value,
    pub template: Template,
}

impl Source {
    pub fn parse(s: &str) -> Result<Source> {
        let (context, template) = extract_blocks(s);

        let context = context.unwrap_or_else(|| "null".to_string());

        let context = serde_yaml::from_str(&context).with_context(|| {
            formatdoc! {"
                error while parsing context as YAML

                --- context ---
                {}", super::chomp(&context)
            }
        })?;

        let template = template.with_context(|| {
            indoc! {"
                no template block found

                Example:

                    --- context ---
                    to: echo world

                    --- template ---
                    hello, {{to}}"
            }
        })?;

        let template = Template::compile(&template)?;

        Ok(Source { context, template })
    }
}

fn extract_blocks(s: &str) -> (Option<String>, Option<String>) {
    let (remains, tpl) = match split_line(s, "--- template ---") {
        Some((remains, tpl)) => (remains, Some(tpl)),
        None => (s.to_string(), None),
    };

    match split_line(&remains, "--- context ---") {
        Some((_, ctx)) => (Some(ctx), tpl),
        None => (None, tpl),
    }
}

fn split_line(s: &str, sep: &str) -> Option<(String, String)> {
    if let Some(after) = s.strip_prefix(&format!("{}\n", sep)) {
        Some((String::new(), after.to_string()))
    } else if let Some(before) = s.strip_suffix(&format!("\n{}", sep)) {
        Some((before.to_string(), String::new()))
    } else if let Some((before, after)) = s.split_once(&format!("\n{}\n", sep)) {
        Some((format!("{}\n", before), after.to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse() {
        let (ctx, tpl) = extract_blocks(indoc! {"
            this is comment
            --- context ---
            x: 42
            --- template ---
            hello
        "});

        assert_eq!(ctx.unwrap(), "x: 42\n");
        assert_eq!(tpl.unwrap(), "hello\n");
    }

    #[test]
    fn test_empty() {
        let (ctx, tpl) = extract_blocks("");

        assert_eq!(ctx, None);
        assert_eq!(tpl, None);
    }

    #[test]
    fn test_no_context() {
        let (ctx, tpl) = extract_blocks(indoc! {"
            --- template ---
            hello
        "});

        assert_eq!(ctx, None);
        assert_eq!(tpl.unwrap(), "hello\n");
    }

    #[test]
    fn test_no_template() {
        let (ctx, tpl) = extract_blocks(indoc! {"
            --- context ---
            x: 42
        "});

        assert_eq!(ctx.unwrap(), "x: 42\n");
        assert_eq!(tpl, None);
    }

    #[test]
    fn test_marker_in_template() {
        let (ctx, tpl) = extract_blocks(indoc! {"
            --- template ---
            hello
            --- context ---
            x: 42
            --- template ---
            world
        "});

        assert_eq!(ctx, None);

        assert_eq!(
            tpl.unwrap(),
            indoc! {"
                hello
                --- context ---
                x: 42
                --- template ---
                world
            "}
        );
    }

    #[test]
    fn test_no_marker() {
        let (ctx, tpl) = extract_blocks("hi");

        assert_eq!(ctx, None);
        assert_eq!(tpl, None);
    }
}
