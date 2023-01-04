use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use anyhow::{bail, Error, Result};

#[derive(Debug)]
pub struct BlockRegistry {
    map: HashMap<String, String>,
}

impl BlockRegistry {
    fn new() -> BlockRegistry {
        let map = HashMap::new();

        BlockRegistry { map }
    }

    pub fn context(&self) -> Option<&String> {
        self.map.get("context")
    }

    pub fn template(&self) -> Option<&String> {
        self.map.get("template")
    }
}

pub fn extract_blocks<T: Read>(reader: &mut T) -> Result<BlockRegistry> {
    let mut reader = BufReader::new(reader);
    let mut registry = BlockRegistry::new();
    let mut current_block: Option<String> = None;

    for (i, line) in LinesWithEndings::from(&mut reader).enumerate() {
        let line = line?;

        if let Some(ref current_block) = current_block {
            if current_block == "template" {
                registry.map.get_mut("template").unwrap().push_str(&line);
                continue;
            }
        }

        if let Some(new_block) = line.trim_end().strip_prefix("--- ").and_then(|s| s.strip_suffix(" ---")) {
            match new_block {
                "context" | "template" => {}
                _ => {
                    bail!("unknown block name `{}` at line {}", new_block, i + 1)
                }
            }

            if registry.map.insert(new_block.to_string(), String::new()).is_some() {
                bail!("duplicate block name `{}` at line {}", new_block, i + 1);
            }

            let _ = current_block.insert(new_block.to_string());
        } else if let Some(ref current_block) = current_block {
            registry.map.get_mut(current_block).unwrap().push_str(&line);
        } else {
            // before any blocks (comment)
        }
    }

    Ok(registry)
}

struct LinesWithEndings<'a, T: BufRead> {
    reader: &'a mut T,
}

impl<'a, T: BufRead> LinesWithEndings<'a, T> {
    fn from(reader: &'a mut T) -> LinesWithEndings<'a, T> {
        LinesWithEndings { reader }
    }
}

impl<'a, T: BufRead> Iterator for LinesWithEndings<'a, T> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        let mut line = String::new();

        match self.reader.read_line(&mut line) {
            Ok(num_bytes) => {
                if num_bytes == 0 {
                    None
                } else {
                    Some(Ok(line))
                }
            }
            Err(e) => Some(Err(Error::new(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;
    use indoc::indoc;

    use super::{extract_blocks, BlockRegistry};

    fn extract_blocks_from_str(s: &str) -> Result<BlockRegistry> {
        extract_blocks(&mut Cursor::new(s))
    }

    #[test]
    fn test_extract() {
        let map = extract_blocks_from_str(indoc! {"
            this is comment
            --- context ---
            x: 42
            --- template ---
            hello
        "})
        .unwrap();

        assert_eq!(map.context().unwrap(), "x: 42\n");
        assert_eq!(map.template().unwrap(), "hello\n");
    }

    #[test]
    fn test_empty() {
        let map = extract_blocks_from_str("").unwrap();

        assert_eq!(map.context(), None);
        assert_eq!(map.template(), None);
    }

    #[test]
    fn test_no_context() {
        let map = extract_blocks_from_str(indoc! {"
            --- template ---
            hello
        "})
        .unwrap();

        assert_eq!(map.context(), None);
        assert_eq!(map.template().unwrap(), "hello\n");
    }

    #[test]
    fn test_no_template() {
        let map = extract_blocks_from_str(indoc! {"
            --- context ---
            x: 42
        "})
        .unwrap();

        assert_eq!(map.context().unwrap(), "x: 42\n");
        assert_eq!(map.template(), None);
    }

    #[test]
    fn test_marker_in_template() {
        let map = extract_blocks_from_str(indoc! {"
            --- template ---
            hello
            --- context ---
            x: 42
            --- template ---
            world
        "})
        .unwrap();

        assert_eq!(map.context(), None);

        assert_eq!(
            map.template().unwrap(),
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
    fn test_eol() {
        let map = extract_blocks_from_str(indoc! {"
            --- template ---
            hello
        "})
        .unwrap();

        assert!(map.template().unwrap().ends_with("hello\n"));

        let map = extract_blocks_from_str(indoc! {"
            --- template ---
            hello"})
        .unwrap();

        assert!(map.template().unwrap().ends_with("hello"));
    }

    #[test]
    fn test_no_block() {
        let map = extract_blocks_from_str("hi").unwrap();

        assert_eq!(map.context(), None);
        assert_eq!(map.template(), None);
    }

    #[test]
    fn test_duplicate_block() {
        let map = extract_blocks_from_str(indoc! {"
            --- context ---
            --- context ---
        "});

        assert_eq!(map.unwrap_err().to_string(), "duplicate block name `context` at line 2");
    }

    #[test]
    fn test_unknown_block() {
        let map = extract_blocks_from_str(indoc! {"
            --- foo ---
        "});

        assert_eq!(map.unwrap_err().to_string(), "unknown block name `foo` at line 1");
    }

    #[test]
    fn test_empty_block_name() {
        let map = extract_blocks_from_str(indoc! {"
            ---  ---
        "});

        assert_eq!(map.unwrap_err().to_string(), "unknown block name `` at line 1");
    }
}
