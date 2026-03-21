use crate::domain::{ParseError, Skill, build_auto_tool_name, validate_explicit_tool_name};
use rust_i18n::t;
use std::collections::HashMap;

pub(super) fn finalize_tool_names(skills: &mut [Skill], strict: bool) -> Vec<ParseError> {
    let mut errors = Vec::new();

    for skill in skills.iter_mut() {
        let fallback_stem = skill.path_file_stem();
        let explicit = skill.tool_name.take();

        skill.tool_name = match explicit {
            Some(explicit) => match validate_explicit_tool_name(&explicit) {
                Some(validated) => Some(validated),
                None if strict => {
                    errors.push(ParseError::new(
                        skill.path.clone(),
                        "Invalid tool_name. Normalization produced an empty identifier."
                            .to_string(),
                    ));
                    None
                }
                None => {
                    eprintln!("{}", t!("cli.invalid_tool_name", path = skill.path));
                    Some(build_auto_tool_name(
                        &skill.name,
                        &skill.path,
                        fallback_stem.as_deref(),
                    ))
                }
            },
            None => Some(build_auto_tool_name(
                &skill.name,
                &skill.path,
                fallback_stem.as_deref(),
            )),
        };
    }

    let mut collisions: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, skill) in skills.iter().enumerate() {
        if let Some(tool_name) = &skill.tool_name {
            collisions.entry(tool_name.clone()).or_default().push(index);
        }
    }

    for (tool_name, indices) in collisions {
        if indices.len() < 2 {
            continue;
        }

        if strict {
            for index in indices {
                errors.push(ParseError::new(
                    skills[index].path.clone(),
                    format!("Duplicate tool_name '{tool_name}'."),
                ));
                skills[index].tool_name = None;
            }
        } else {
            for index in indices {
                let path = skills[index].path.clone();
                let unique_name = format!("{tool_name}_{}", short_path_hash(&path));
                eprintln!(
                    "{}",
                    t!(
                        "cli.duplicate_tool_name",
                        tool_name = tool_name,
                        path = path,
                        new_name = unique_name
                    )
                );
                skills[index].tool_name = Some(unique_name);
            }
        }
    }

    errors
}

fn short_path_hash(path: &str) -> String {
    build_auto_tool_name("skill", path, None)
        .rsplit('_')
        .next()
        .unwrap_or("00000000")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(name: &str, path: &str, tool_name: Option<&str>) -> Skill {
        Skill {
            name: name.to_string(),
            description: format!("{name} description"),
            path: path.to_string(),
            tags: Vec::new(),
            command_template: None,
            parameters: None,
            checksum: None,
            tool_name: tool_name.map(str::to_string),
        }
    }

    #[test]
    fn test_finalize_tool_names_falls_back_for_invalid_explicit_name_in_non_strict_mode() {
        let mut skills = [skill("Echo Skill", "/tmp/echo.py", Some("🔥🔥"))];

        let errors = finalize_tool_names(&mut skills, false);

        // 非 strict 模式下，非法显式名不能让技能消失，而是回退到稳定自动名。
        assert!(errors.is_empty());
        assert_eq!(
            skills[0].tool_name,
            Some(build_auto_tool_name(
                "Echo Skill",
                "/tmp/echo.py",
                Some("echo")
            ))
        );
    }

    #[test]
    fn test_finalize_tool_names_reports_invalid_explicit_name_in_strict_mode() {
        let mut skills = [skill("Echo Skill", "/tmp/echo.py", Some("🔥🔥"))];

        let errors = finalize_tool_names(&mut skills, true);

        // strict 模式下必须显式暴露错误，并把非法 tool_name 清空留给上游过滤。
        assert_eq!(errors.len(), 1);
        assert!(skills[0].tool_name.is_none());
        assert!(errors[0].reason.contains("Invalid tool_name"));
    }

    #[test]
    fn test_finalize_tool_names_deduplicates_collisions_in_non_strict_mode() {
        let mut skills = [
            skill("Echo One", "/tmp/echo_one.py", Some("echo_skill")),
            skill("Echo Two", "/tmp/echo_two.py", Some("echo_skill")),
        ];

        let errors = finalize_tool_names(&mut skills, false);

        // 非 strict 模式下冲突应该自动加路径哈希，保证 search --json 输出可去重。
        assert!(errors.is_empty());
        assert_ne!(skills[0].tool_name, skills[1].tool_name);
        assert!(
            skills[0]
                .tool_name
                .as_deref()
                .expect("tool_name should be present")
                .starts_with("echo_skill_")
        );
        assert!(
            skills[1]
                .tool_name
                .as_deref()
                .expect("tool_name should be present")
                .starts_with("echo_skill_")
        );
    }

    #[test]
    fn test_finalize_tool_names_reports_collisions_in_strict_mode() {
        let mut skills = [
            skill("Echo One", "/tmp/echo_one.py", Some("echo_skill")),
            skill("Echo Two", "/tmp/echo_two.py", Some("echo_skill")),
        ];

        let errors = finalize_tool_names(&mut skills, true);

        // strict 模式下重复名必须被视为配置错误，而不是静默改写。
        assert_eq!(errors.len(), 2);
        assert!(skills.iter().all(|skill| skill.tool_name.is_none()));
        assert!(
            errors
                .iter()
                .all(|error| error.reason.contains("Duplicate tool_name"))
        );
    }
}
