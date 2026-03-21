pub(crate) fn validate_explicit_tool_name(tool_name: &str) -> Option<String> {
    let sanitized = sanitize_tool_name(tool_name);
    (!sanitized.is_empty()).then_some(sanitized)
}

pub(crate) fn build_auto_tool_name(
    display_name: &str,
    path: &str,
    fallback_stem: Option<&str>,
) -> String {
    let base = [
        sanitize_tool_name(display_name),
        fallback_stem.map(sanitize_tool_name).unwrap_or_default(),
        "skill".to_string(),
    ]
    .into_iter()
    .find(|candidate| !candidate.is_empty())
    .unwrap_or_else(|| "skill".to_string());

    format!("{base}_{}", short_stable_hash(path))
}

fn sanitize_tool_name(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut previous_separator = false;

    for character in input.chars().flat_map(char::to_lowercase) {
        let mapped = match character {
            'a'..='z' | '0'..='9' => character,
            '_' | '-' => character,
            _ => '_',
        };

        let is_separator = mapped == '_' || mapped == '-';
        if is_separator && previous_separator {
            continue;
        }

        previous_separator = is_separator;
        output.push(mapped);
    }

    let trimmed = output.trim_matches(|character| character == '_' || character == '-');
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic())
    {
        trimmed.to_string()
    } else {
        format!("skill_{trimmed}")
    }
}

fn short_stable_hash(input: &str) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let hash = input
        .as_bytes()
        .iter()
        .fold(FNV_OFFSET_BASIS, |hash, byte| {
            hash.wrapping_mul(FNV_PRIME) ^ u64::from(*byte)
        });

    format!("{:08x}", hash as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_tool_name_is_stable_and_safe() {
        let first = build_auto_tool_name("Image Resize", "/tmp/demo/image.py", Some("image"));
        let second = build_auto_tool_name("Image Resize", "/tmp/demo/image.py", Some("image"));

        assert_eq!(first, second);
        assert!(first.starts_with("image_resize_"));
        assert!(first.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '_'
                || character == '-'
        }));
    }

    #[test]
    fn test_auto_tool_name_falls_back_when_name_is_non_ascii() {
        let tool_name = build_auto_tool_name("图片压缩", "/tmp/demo/图片压缩.py", Some("图片压缩"));
        assert!(tool_name.starts_with("skill_"));
    }

    #[test]
    fn test_validate_explicit_tool_name_rejects_empty_after_normalization() {
        assert_eq!(validate_explicit_tool_name("🚀🚀"), None);
    }

    #[test]
    fn test_validate_explicit_tool_name_normalizes_to_ascii() {
        assert_eq!(
            validate_explicit_tool_name("SQL Query@Prod"),
            Some("sql_query_prod".to_string())
        );
    }
}
