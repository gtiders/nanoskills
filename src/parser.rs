use crate::models::SkillHeader;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

const MAX_SCAN_LINES: usize = 50;

pub struct HeaderParser;

impl HeaderParser {
    pub fn parse_file(path: &Path) -> Result<Option<SkillHeader>> {
        let content = fs::read_to_string(path)?;
        Self::parse_content(&content)
    }

    pub fn parse_content(content: &str) -> Result<Option<SkillHeader>> {
        let lines: Vec<&str> = content.lines().take(MAX_SCAN_LINES).collect();

        let start_idx = lines
            .iter()
            .enumerate()
            .position(|(idx, line)| (idx != 0 || !line.starts_with("#!")) && line.contains("---"));

        let start_idx = match start_idx {
            Some(idx) => idx,
            None => return Ok(None),
        };

        let start_line = lines[start_idx].trim_start();
        let pos = start_line.find("---").unwrap();
        let prefix = start_line[..pos].trim_end();

        let mut yaml_string = String::new();
        let mut found_end = false;

        for line in lines.into_iter().skip(start_idx + 1) {
            let trimmed = line.trim_start();

            if trimmed.starts_with(prefix)
                && trimmed[prefix.len()..].trim_start().starts_with("---")
            {
                found_end = true;
                break;
            }

            let clean_line = if prefix.is_empty() {
                line
            } else if let Some(stripped) = trimmed.strip_prefix(prefix) {
                stripped.strip_prefix(' ').unwrap_or(stripped)
            } else {
                trimmed
            };

            yaml_string.push_str(clean_line);
            yaml_string.push('\n');
        }

        if !found_end || yaml_string.trim().is_empty() {
            return Ok(None);
        }

        let header: SkillHeader =
            serde_yaml::from_str(&yaml_string).map_err(|e| anyhow!("YAML 解析错误: {}", e))?;

        Ok(Some(header))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_python_header() {
        let content = r#"#!/usr/bin/env python3
# ---
# name: test_skill
# description: A test skill
# tags: [test]
# ---
print("hello")
"#;
        let result = HeaderParser::parse_content(content).unwrap();
        assert!(result.is_some());
        let header = result.unwrap();
        assert_eq!(header.name, "test_skill");
        assert_eq!(header.description, "A test skill");
    }

    #[test]
    fn test_parse_js_header() {
        let content = r#"// ---
// name: js_skill
// description: A JS skill
// ---
console.log("hello");
"#;
        let result = HeaderParser::parse_content(content).unwrap();
        assert!(result.is_some());
        let header = result.unwrap();
        assert_eq!(header.name, "js_skill");
    }

    #[test]
    fn test_parse_block_comment_header() {
        let content = r#"/*
---
name: block_skill
description: A block comment skill
---
*/
int main() {}
"#;
        let result = HeaderParser::parse_content(content).unwrap();
        assert!(result.is_some());
        let header = result.unwrap();
        assert_eq!(header.name, "block_skill");
    }

    #[test]
    fn test_parse_erlang_style() {
        let content = r#"% ---
% name: erlang_skill
% description: An Erlang-style skill
% ---
-module(test).
"#;
        let result = HeaderParser::parse_content(content).unwrap();
        assert!(result.is_some());
        let header = result.unwrap();
        assert_eq!(header.name, "erlang_skill");
    }

    #[test]
    fn test_parse_haskell_style() {
        let content = r#"-- ---
-- name: haskell_skill
-- description: A Haskell-style skill
-- ---
main = putStrLn "Hello"
"#;
        let result = HeaderParser::parse_content(content).unwrap();
        assert!(result.is_some());
        let header = result.unwrap();
        assert_eq!(header.name, "haskell_skill");
    }

    #[test]
    fn test_parse_custom_emoji_prefix() {
        let content = r#"🚀 ---
🚀 name: emoji_skill
🚀 description: A skill with emoji prefix
🚀 ---
print("emoji!")
"#;
        let result = HeaderParser::parse_content(content).unwrap();
        assert!(result.is_some());
        let header = result.unwrap();
        assert_eq!(header.name, "emoji_skill");
    }
}
