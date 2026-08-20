use crate::model::{Declaration, Role, Runtime};

pub fn parse_node_version(contents: &str, source: &str) -> Option<Declaration> {
    let version = contents.trim();

    if version.is_empty() {
        return None;
    }

    let normalized = version.strip_prefix('v').unwrap_or(version);

    Some(Declaration {
        runtime: Runtime::Node,
        constraint: normalized.to_string(),
        role: Role::Development,
        source: source.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_node_version_as_development_declaration() {
        let declaration = parse_node_version("22\n", ".node-version").unwrap();

        assert_eq!(declaration.runtime, Runtime::Node);
        assert_eq!(declaration.constraint, "22");
        assert_eq!(declaration.role, Role::Development);
        assert_eq!(declaration.source, ".node-version");
    }

    #[test]
    fn removes_v_prefix() {
        let declaration = parse_node_version("v22.4.1\n", ".node-version").unwrap();

        assert_eq!(declaration.constraint, "22.4.1");
    }

    #[test]
    fn returns_none_for_empty_file() {
        let declaration = parse_node_version("   \n", ".node-version");

        assert_eq!(declaration, None);
    }
}
