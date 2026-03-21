use crate::app::SyncResult;
use crate::domain::Skill;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use rust_i18n::t;

pub(crate) fn print_sync_result(result: &SyncResult) {
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

pub(crate) fn print_skills_table(skills: &[&Skill]) {
    if skills.is_empty() {
        println!("{}", t!("cli.no_skills"));
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        t!("ui.table_header.index"),
        t!("ui.table_header.name"),
        t!("ui.table_header.description"),
        t!("ui.table_header.tags"),
        t!("ui.table_header.path"),
    ]);

    for (index, skill) in skills.iter().enumerate() {
        table.add_row(vec![
            Cell::new(index + 1),
            Cell::new(&skill.name),
            Cell::new(&skill.description),
            Cell::new(skill.tags.join(", ")),
            Cell::new(&skill.path),
        ]);
    }

    println!("{table}");
}

pub(crate) fn print_detailed_table(skills: &[&Skill]) {
    if skills.is_empty() {
        println!("{}", t!("cli.no_skills"));
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        t!("ui.table_header.index"),
        t!("ui.table_header.name"),
        t!("ui.table_header.description"),
        t!("ui.table_header.tags"),
        t!("ui.table_header.parameters"),
    ]);

    for (index, skill) in skills.iter().enumerate() {
        let parameters = skill
            .parameters
            .as_ref()
            .map(|params| {
                serde_yaml::to_string(params)
                    .unwrap_or_else(|error| format!("<invalid parameters: {error}>"))
            })
            .unwrap_or_default();

        table.add_row(vec![
            Cell::new(index + 1),
            Cell::new(&skill.name),
            Cell::new(&skill.description),
            Cell::new(skill.tags.join(", ")),
            Cell::new(parameters.trim()),
        ]);
    }

    println!("{table}");
    println!("{}", t!("cli.total_skills", count = skills.len()));
}
