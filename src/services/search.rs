use crate::model::Skill;
use fuzzy_matcher::FuzzyMatcher;

pub(crate) fn fuzzy_search<'a>(skills: &'a [Skill], query: &str) -> Vec<(&'a Skill, i64)> {
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

    let mut results: Vec<_> = skills
        .iter()
        .filter_map(|skill| {
            let tags = skill.tags.join(" ");
            let name_score = matcher.fuzzy_match(&skill.name, query).unwrap_or(0);
            let description_score = matcher.fuzzy_match(&skill.description, query).unwrap_or(0) / 2;
            let tags_score = matcher.fuzzy_match(&tags, query).unwrap_or(0) / 2;
            let max_score = [name_score, description_score, tags_score]
                .into_iter()
                .max()
                .unwrap_or(0);

            (max_score > 0).then_some((skill, max_score))
        })
        .collect();

    results.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.name.cmp(&right.0.name))
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(name: &str, description: &str) -> Skill {
        Skill {
            name: name.to_string(),
            description: description.to_string(),
            path: format!("/tmp/{name}.py"),
            tags: Vec::new(),
            command_template: None,
            parameters: None,
            checksum: None,
            tool_name: None,
        }
    }

    #[test]
    fn test_fuzzy_search_sorts_by_score_desc() {
        let skills = vec![
            skill("image_tool", "Generate an image"),
            skill("echo_tool", "Echo user input"),
        ];

        let results = fuzzy_search(&skills, "image");

        assert_eq!(results.len(), 1);
        // 直接命中名称的技能应排在前面，避免 search 输出顺序漂移。
        assert_eq!(results[0].0.name, "image_tool");
    }

    #[test]
    fn test_fuzzy_search_uses_name_as_tie_breaker() {
        let skills = vec![
            skill("beta_skill", "shared description"),
            skill("alpha_skill", "shared description"),
        ];

        let results = fuzzy_search(&skills, "shared");

        assert_eq!(results.len(), 2);
        // 当分数相同时，结果必须按名称升序稳定排序，方便 CLI 和 JSON 输出可预测。
        assert_eq!(results[0].0.name, "alpha_skill");
        assert_eq!(results[1].0.name, "beta_skill");
    }

    #[test]
    fn test_fuzzy_search_does_not_match_by_path() {
        let skills = vec![skill("alpha_skill", "image transform")];

        let results = fuzzy_search(&skills, "/tmp/alpha_skill.py");

        assert!(results.is_empty());
    }
}
