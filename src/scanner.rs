use crate::model::Declaration;
use crate::parsers::github_actions::parse_github_actions;
use std::{fs, path::Path};

pub fn scan_github_actions_workflows(directory: &str) -> Result<Vec<Declaration>, String> {
    let directory = Path::new(directory);

    if !directory.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(directory)
        .map_err(|error| format!("Failed to read {}: {error}", directory.display()))?;

    let mut declarations = Vec::new();

    for entry in entries {
        let entry = entry
            .map_err(|error| format!("Failed to read entry in {}: {error}", directory.display()))?;

        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|extension| extension.to_str());

        if extension != Some("yml") && extension != Some("yaml") {
            continue;
        }

        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;

        let source = path.to_string_lossy().to_string();

        let parsed = parse_github_actions(&contents, &source)
            .map_err(|error| format!("Failed to parse {}: {error}", path.display()))?;

        declarations.extend(parsed);
    }

    declarations.sort_by(|left, right| left.source.cmp(&right.source));

    Ok(declarations)
}
