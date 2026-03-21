use crate::domain::Skill;

#[derive(Debug, Clone, Default)]
pub(super) struct MatchHighlights {
    pub(super) name: Vec<usize>,
    pub(super) description: Vec<usize>,
    pub(super) tags: Vec<usize>,
}

#[derive(Debug, Clone)]
pub(super) struct FilteredSkill {
    pub(super) item_index: usize,
    pub(super) score: i64,
    pub(super) highlights: MatchHighlights,
}

#[derive(Debug, Clone)]
pub(super) struct SearchProjection {
    pub(super) haystack: String,
    pub(super) name_len: usize,
    pub(super) description_len: usize,
    pub(super) tags_len: usize,
}

#[derive(Debug, Clone)]
pub(super) struct SkillViewModel {
    pub(super) skill: Skill,
    pub(super) tags_text: String,
    pub(super) projection: SearchProjection,
}

impl SkillViewModel {
    pub(super) fn new(skill: Skill) -> Self {
        let tags_text = if skill.tags.is_empty() {
            "-".to_string()
        } else {
            skill.tags.join(", ")
        };

        let projection = build_search_projection(&skill, &tags_text);

        Self {
            skill,
            tags_text,
            projection,
        }
    }
}

fn build_search_projection(skill: &Skill, tags_text: &str) -> SearchProjection {
    let haystack = format!(
        "{}\n{}\n{}\n{}",
        skill.name, skill.description, tags_text, skill.path
    );

    SearchProjection {
        haystack,
        name_len: skill.name.chars().count(),
        description_len: skill.description.chars().count(),
        tags_len: tags_text.chars().count(),
    }
}

pub(super) fn split_match_indices(
    indices: &[usize],
    projection: &SearchProjection,
) -> MatchHighlights {
    let description_start = projection.name_len + 1;
    let tags_start = description_start + projection.description_len + 1;

    let mut highlights = MatchHighlights::default();

    // fuzzy matcher 返回的是字符位置而不是字节偏移；这里按字符区间切分，
    // 才能保证后续高亮在 Unicode 文本上不越界，也不会把中文/emoji 切坏。
    for &index in indices {
        if index < projection.name_len {
            highlights.name.push(index);
        } else if (description_start..description_start + projection.description_len)
            .contains(&index)
        {
            highlights.description.push(index - description_start);
        } else if (tags_start..tags_start + projection.tags_len).contains(&index) {
            highlights.tags.push(index - tags_start);
        }
    }

    highlights
}
