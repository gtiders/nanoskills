use crate::config::{ConfigResolver, resolve_config};
use crate::models::{Index, ParseError, Skill};
use crate::parser::HeaderParser;
use crate::scanner::scan_files;
use anyhow::Result;
use rayon::prelude::*;
use rust_i18n::t;
use std::fs;
use std::path::Path;
use std::time::Instant;

const INDEX_FILE_NAME: &str = "index.json";

pub struct SyncResult {
    pub total_files: usize,
    pub skills_count: usize,
    pub elapsed_ms: u128,
    pub errors: Vec<ParseError>,
}

pub fn run_sync(local_dir: &Path, strict: bool) -> Result<SyncResult> {
    let start = Instant::now();

    let config = resolve_config(local_dir)?;

    let cache_dir = ConfigResolver::ensure_cache_dir()?;
    let index_path = cache_dir.join(INDEX_FILE_NAME);

    let files = scan_files(&config)?;
    let total_files = files.len();

    let results: Vec<Result<Option<Skill>, ParseError>> = files
        .par_iter()
        .map(|file_path| {
            let path = Path::new(file_path);

            match HeaderParser::parse_file(path) {
                Ok(Some(header)) => Ok(Some(Skill::from((header, file_path.clone())))),
                Ok(None) => Ok(None),
                Err(e) => {
                    if strict {
                        Err(ParseError::new(file_path.clone(), e.to_string()))
                    } else {
                        Ok(None)
                    }
                }
            }
        })
        .collect();

    let mut skills: Vec<Skill> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    for result in results {
        match result {
            Ok(Some(skill)) => skills.push(skill),
            Ok(None) => {}
            Err(error) => errors.push(error),
        }
    }

    let mut index = Index::new();
    index.skills = skills;
    let skills_count = index.skills.len();

    let content = serde_json::to_string_pretty(&index)?;
    fs::write(&index_path, content)?;

    let elapsed_ms = start.elapsed().as_millis();

    Ok(SyncResult {
        total_files,
        skills_count,
        elapsed_ms,
        errors,
    })
}

pub fn print_sync_result(result: &SyncResult) {
    println!(
        "{}",
        t!(
            "cli.sync_complete",
            time = result.elapsed_ms,
            files = result.total_files,
            skills = result.skills_count
        )
    );

    if !result.errors.is_empty() {
        println!("{}", t!("cli.parse_errors"));
        for error in &result.errors {
            println!(
                "{}",
                t!(
                    "cli.parse_error_item",
                    path = error.path,
                    reason = error.reason
                )
            );
        }
    }
}

pub struct SkillSearcher {
    index: Index,
}

impl SkillSearcher {
    pub fn new(index: Index) -> Self {
        SkillSearcher { index }
    }

    pub fn fuzzy_search(&self, query: &str) -> Vec<(&Skill, i64)> {
        use fuzzy_matcher::FuzzyMatcher;

        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

        let mut results: Vec<(&Skill, i64)> = self
            .index
            .skills
            .iter()
            .filter_map(|skill| {
                let name_score = matcher.fuzzy_match(&skill.name, query).unwrap_or(0);
                let desc_score = matcher.fuzzy_match(&skill.description, query).unwrap_or(0);
                let max_score = name_score.max(desc_score / 2);

                if max_score > 0 {
                    Some((skill, max_score))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }
}

pub fn load_index() -> Result<Option<Index>> {
    let cache_dir = ConfigResolver::get_cache_dir();
    let index_path = cache_dir.join(INDEX_FILE_NAME);

    if !index_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&index_path)?;
    let index: Index = serde_json::from_str(&content)?;
    Ok(Some(index))
}
