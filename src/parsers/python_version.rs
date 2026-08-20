use crate::model::{Declaration, Role, Runtime};

pub fn parse_python_version(contents: &str, source: &str) -> Option<Declaration> {
    let version = contents
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))?;

    let normalized = version.strip_prefix("python-").unwrap_or(version);

    Some(Declaration {
        runtime: Runtime::Python,
        constraint: normalized.to_string(),
        role: Role::Development,
        source: source.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_python_version_as_development_declaration() {
        let declaration = parse_python_version("3.12.4\n", ".python-version").unwrap();

        assert_eq!(declaration.runtime, Runtime::Python);
        assert_eq!(declaration.constraint, "3.12.4");
        assert_eq!(declaration.role, Role::Development);
        assert_eq!(declaration.source, ".python-version");
    }

    #[test]
    fn removes_python_prefix() {
        let declaration = parse_python_version("python-3.11.9\n", ".python-version").unwrap();

        assert_eq!(declaration.constraint, "3.11.9");
    }

    #[test]
    fn ignores_blank_lines_and_comments() {
        let declaration =
            parse_python_version("\n# Python used locally\n3.13.0\n", ".python-version").unwrap();

        assert_eq!(declaration.constraint, "3.13.0");
    }

    #[test]
    fn returns_none_when_no_version_exists() {
        let declaration = parse_python_version("\n# comment only\n", ".python-version");

        assert_eq!(declaration, None);
    }
}
