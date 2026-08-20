use crate::model::{Declaration, Role, Runtime};
use toml::Table;

pub fn parse_pyproject_toml(
    contents: &str,
    source: &str,
) -> Result<Option<Declaration>, toml::de::Error> {
    let document: Table = contents.parse()?;

    let Some(project) = document.get("project").and_then(|value| value.as_table()) else {
        return Ok(None);
    };

    let Some(requires_python) = project
        .get("requires-python")
        .and_then(|value| value.as_str())
    else {
        return Ok(None);
    };

    Ok(Some(Declaration {
        runtime: Runtime::Python,
        constraint: requires_python.to_string(),
        role: Role::Support,
        source: source.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_requires_python_as_support_declaration() {
        let contents = r#"
[project]
name = "example"
version = "0.1.0"
requires-python = ">=3.11"
"#;

        let declaration = parse_pyproject_toml(contents, "pyproject.toml")
            .unwrap()
            .unwrap();

        assert_eq!(declaration.runtime, Runtime::Python);
        assert_eq!(declaration.constraint, ">=3.11");
        assert_eq!(declaration.role, Role::Support);
        assert_eq!(declaration.source, "pyproject.toml");
    }

    #[test]
    fn supports_bounded_python_range() {
        let contents = r#"
[project]
requires-python = ">=3.10,<3.13"
"#;

        let declaration = parse_pyproject_toml(contents, "pyproject.toml")
            .unwrap()
            .unwrap();

        assert_eq!(declaration.constraint, ">=3.10,<3.13");
    }

    #[test]
    fn returns_none_when_requires_python_is_missing() {
        let contents = r#"
[project]
name = "example"
version = "0.1.0"
"#;

        let declaration = parse_pyproject_toml(contents, "pyproject.toml").unwrap();

        assert!(declaration.is_none());
    }

    #[test]
    fn rejects_invalid_toml() {
        let result =
            parse_pyproject_toml("[project\nrequires-python = \">=3.11\"", "pyproject.toml");

        assert!(result.is_err());
    }
}
