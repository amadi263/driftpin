use crate::model::{Declaration, Role, Runtime};

pub fn parse_tool_versions(contents: &str, source: &str) -> Vec<Declaration> {
    let mut declarations = Vec::new();

    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();

        let Some(tool) = parts.next() else {
            continue;
        };

        let Some(version) = parts.next() else {
            continue;
        };

        let runtime = match tool {
            "nodejs" | "node" => Runtime::Node,
            "python" => Runtime::Python,
            _ => continue,
        };

        declarations.push(Declaration {
            runtime,
            constraint: version.to_string(),
            role: Role::Development,
            source: source.to_string(),
        });
    }

    declarations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_node_version() {
        let declarations = parse_tool_versions("nodejs 22.4.1\n", ".tool-versions");

        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].runtime, Runtime::Node);
        assert_eq!(declarations[0].constraint, "22.4.1");
        assert_eq!(declarations[0].role, Role::Development);
    }

    #[test]
    fn parses_python_version() {
        let declarations = parse_tool_versions("python 3.12.4\n", ".tool-versions");

        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].runtime, Runtime::Python);
        assert_eq!(declarations[0].constraint, "3.12.4");
    }

    #[test]
    fn parses_node_and_python_together() {
        let contents = r#"
nodejs 22
python 3.12
"#;

        let declarations = parse_tool_versions(contents, ".tool-versions");

        assert_eq!(declarations.len(), 2);
        assert_eq!(declarations[0].runtime, Runtime::Node);
        assert_eq!(declarations[1].runtime, Runtime::Python);
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let contents = r#"
# local runtimes

nodejs 22
"#;

        let declarations = parse_tool_versions(contents, ".tool-versions");

        assert_eq!(declarations.len(), 1);
    }

    #[test]
    fn ignores_unknown_tools() {
        let contents = r#"
ruby 3.3
golang 1.24
nodejs 22
"#;

        let declarations = parse_tool_versions(contents, ".tool-versions");

        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].runtime, Runtime::Node);
    }
}
