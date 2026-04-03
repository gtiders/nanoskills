use crate::model::Skill;
use crate::services::SyncResult;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};

pub(crate) fn print_sync_result(result: &SyncResult) {
    println!(
        "Index built in {} ms. Scanned {} files, indexed {} skills.",
        result.elapsed_ms, result.total_files, result.skills_count
    );

    if !result.errors.is_empty() {
        println!("\nParse errors:");
        for error in &result.errors {
            println!("  • {} - {}", error.path, error.reason);
        }
    }
}

pub(crate) fn print_skills_table(skills: &[&Skill]) {
    if skills.is_empty() {
        println!("No skills found");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        "#", "Name", "Description", "Tags", "Path",
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
