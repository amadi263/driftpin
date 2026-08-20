use crate::model::{Declaration, Role, Runtime};

pub fn parse_nvmrc(contents: &str, source: &str) -> Option<Declaration> {
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
        let declaration = parse_nvmrc("22\n", ".nvmrc").unwrap();

        assert_eq!(declaration.runtime, Runtime::Node);
        assert_eq!(declaration.constraint, "22");
        assert_eq!(declaration.role, Role::Development);
        assert_eq!(declaration.source, ".nvmrc");
    }

    #[test]
    fn removes_v_prefix() {
        let declaration = parse_nvmrc("v22.4.1\n", ".nvmrc").unwrap();

        assert_eq!(declaration.constraint, "22.4.1");
    }

    #[test]
    fn returns_none_for_empty_file() {
        let declaration = parse_nvmrc("   \n", ".nvmrc");

        assert_eq!(declaration, None);
    }
}
