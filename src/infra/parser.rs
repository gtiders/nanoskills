use crate::domain::SkillHeader;
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::Path;

const HEADER_SEPARATOR: &str = "---";
const MAX_SCAN_LINES: usize = 50;

/// Parses cross-language YAML headers from source files.
pub(crate) struct HeaderParser;

impl HeaderParser {
    /// Parse a skill header from a file on disk.
    pub(crate) fn parse_file(path: &Path) -> Result<Option<SkillHeader>> {
        let content = fs::read(path)
            .with_context(|| format!("failed to read skill file {}", path.display()))?;
        let content = String::from_utf8_lossy(&content);
        Self::parse_content(&content)
    }

    /// Parse a skill header from in-memory text content.
    pub(crate) fn parse_content(content: &str) -> Result<Option<SkillHeader>> {
        let lines: Vec<&str> = content.lines().take(MAX_SCAN_LINES).collect();
        let Some((start_index, prefix)) = find_header_start(&lines) else {
            return Ok(None);
        };

        let mut yaml = String::new();
        let mut found_end = false;

        for line in lines.iter().skip(start_index + 1).copied() {
            if is_header_boundary(line, prefix) {
                found_end = true;
                break;
            }

            yaml.push_str(strip_comment_prefix(line, prefix));
            yaml.push('\n');
        }

        if !found_end || yaml.trim().is_empty() {
            return Ok(None);
        }

        serde_yaml::from_str(&yaml)
            .map(Some)
            .map_err(|error| anyhow!("YAML 解析错误: {error}"))
    }
}

fn find_header_start<'a>(lines: &'a [&'a str]) -> Option<(usize, &'a str)> {
    lines.iter().enumerate().find_map(|(index, line)| {
        let trimmed = line.trim_start();
        if index == 0 && trimmed.starts_with("#!") {
            return None;
        }

        let separator = trimmed.find(HEADER_SEPARATOR)?;
        Some((index, trimmed[..separator].trim_end()))
    })
}

fn is_header_boundary(line: &str, prefix: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix(prefix)
        .map(|suffix| suffix.trim_start().starts_with(HEADER_SEPARATOR))
        .unwrap_or(false)
}

fn strip_comment_prefix<'a>(line: &'a str, prefix: &str) -> &'a str {
    let trimmed = line.trim_start();

    if prefix.is_empty() {
        return line;
    }

    // 这里仅移除注释前缀后的一个空格，目的是保留 YAML 原始缩进。
    // 如果把所有前导空白都吞掉，嵌套对象和数组的层级会被破坏，解析结果会失真。
    trimmed
        .strip_prefix(prefix)
        .map(|stripped| stripped.strip_prefix(' ').unwrap_or(stripped))
        .unwrap_or(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_header(content: &str) -> SkillHeader {
        match HeaderParser::parse_content(content) {
            Ok(Some(header)) => header,
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn test_parse_python_header_with_shebang() {
        let content = r#"#!/usr/bin/env python3
# ---
# name: test_skill
# description: A test skill
# tags: [test]
# ---
print("hello")
"#;

        let header = parse_header(content);
        assert_eq!(header.name, "test_skill");
        assert_eq!(header.description, "A test skill");
    }

    #[test]
    fn test_parse_header_without_comment_prefix() {
        let content = r#"---
name: bare_skill
description: Header without prefix
---
echo hello
"#;

        let header = parse_header(content);
        assert_eq!(header.name, "bare_skill");
    }

    #[test]
    fn test_parse_line_comment_prefix() {
        let content = r#"// ---
// name: js_skill
// description: A JS skill
// ---
console.log("hello");
"#;

        let header = parse_header(content);
        assert_eq!(header.name, "js_skill");
    }

    #[test]
    fn test_parse_wrapped_block_comment_prefix() {
        let content = r#"/**
 * ---
 * name: block_skill
 * description: Wrapped block comment skill
 * ---
 */
fn main() {}
"#;

        let header = parse_header(content);
        assert_eq!(header.name, "block_skill");
    }

    #[test]
    fn test_parse_erlang_style_prefix() {
        let content = r#"% ---
% name: erlang_skill
% description: An Erlang-style skill
% ---
-module(test).
"#;

        let header = parse_header(content);
        assert_eq!(header.name, "erlang_skill");
    }

    #[test]
    fn test_parse_haskell_style_prefix() {
        let content = r#"-- ---
-- name: haskell_skill
-- description: A Haskell-style skill
-- ---
main = putStrLn "Hello"
"#;

        let header = parse_header(content);
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

        let header = parse_header(content);
        assert_eq!(header.name, "emoji_skill");
    }

    #[test]
    fn test_only_newlines_return_none() {
        // 只有空白内容时不应误判出 header。
        let result = HeaderParser::parse_content("\n\n\n");
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn test_missing_end_marker_returns_none() {
        let content = "# ---\n# name: incomplete\n";
        let result = HeaderParser::parse_content(content);
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn test_malformed_yaml_returns_error() {
        let content = r#"# ---
# name: broken
# description: [unterminated
# ---
"#;

        // YAML 结构损坏时必须向上返回错误，而不是悄悄吞掉问题。
        let result = HeaderParser::parse_content(content);
        assert!(result.is_err());
    }
}
