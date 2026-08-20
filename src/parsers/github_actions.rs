use crate::model::{Declaration, Role, Runtime};
use serde_yaml_ng::{Mapping, Value};

pub fn parse_github_actions(
    contents: &str,
    source: &str,
) -> Result<Vec<Declaration>, serde_yaml_ng::Error> {
    let document: Value = serde_yaml_ng::from_str(contents)?;
    let mut declarations = Vec::new();

    let Some(root) = document.as_mapping() else {
        return Ok(declarations);
    };

    let Some(jobs) = mapping_get(root, "jobs").and_then(Value::as_mapping) else {
        return Ok(declarations);
    };

    for job in jobs.values() {
        let Some(job) = job.as_mapping() else {
            continue;
        };

        let Some(steps) = mapping_get(job, "steps").and_then(Value::as_sequence) else {
            continue;
        };

        for step in steps {
            let Some(step) = step.as_mapping() else {
                continue;
            };

            let Some(uses) = mapping_get(step, "uses").and_then(Value::as_str) else {
                continue;
            };

            if !uses.starts_with("actions/setup-node@") {
                continue;
            }

            let Some(with) = mapping_get(step, "with").and_then(Value::as_mapping) else {
                continue;
            };

            let Some(node_version) = mapping_get(with, "node-version") else {
                continue;
            };

            let Some(constraint) = scalar_to_string(node_version) else {
                continue;
            };

            declarations.push(Declaration {
                runtime: Runtime::Node,
                constraint,
                role: Role::Test,
                source: source.to_string(),
            });
        }
    }

    Ok(declarations)
}

fn mapping_get<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(&Value::String(key.to_string()))
}

fn scalar_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_setup_node_version_as_test_declaration() {
        let yaml = r#"
name: CI

on:
  push:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
"#;

        let declarations = parse_github_actions(yaml, ".github/workflows/ci.yml").unwrap();

        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].runtime, Runtime::Node);
        assert_eq!(declarations[0].constraint, "22");
        assert_eq!(declarations[0].role, Role::Test);
        assert_eq!(declarations[0].source, ".github/workflows/ci.yml");
    }

    #[test]
    fn parses_numeric_node_version() {
        let yaml = r#"
jobs:
  test:
    steps:
      - uses: actions/setup-node@v4
        with:
          node-version: 20
"#;

        let declarations = parse_github_actions(yaml, ".github/workflows/test.yml").unwrap();

        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].constraint, "20");
    }

    #[test]
    fn ignores_steps_without_setup_node() {
        let yaml = r#"
jobs:
  test:
    steps:
      - uses: actions/checkout@v4
      - run: npm test
"#;

        let declarations = parse_github_actions(yaml, ".github/workflows/ci.yml").unwrap();

        assert!(declarations.is_empty());
    }

    #[test]
    fn rejects_invalid_yaml() {
        let result = parse_github_actions("jobs: [this is: invalid", ".github/workflows/ci.yml");

        assert!(result.is_err());
    }
}
