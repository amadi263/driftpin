use crate::model::{Declaration, Role, Runtime};
use serde_json::Value;

pub fn parse_package_json(
    contents: &str,
    source: &str,
) -> Result<Option<Declaration>, serde_json::Error> {
    let value: Value = serde_json::from_str(contents)?;

    let node_constraint = value
        .get("engines")
        .and_then(|engines| engines.get("node"))
        .and_then(Value::as_str);

    Ok(node_constraint.map(|constraint| Declaration {
        runtime: Runtime::Node,
        constraint: constraint.to_string(),
        role: Role::Support,
        source: source.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_node_engine_as_support_declaration() {
        let json = r#"
                                                                                                            {
                                                                                                                        "engines": {
                                                                                                                                        "node": ">=20"
                                                                                                                                                    }
                                                                                                                                                            }
                                                                                                                                                                    "#;

        let declaration = parse_package_json(json, "package.json").unwrap().unwrap();

        assert_eq!(declaration.runtime, Runtime::Node);
        assert_eq!(declaration.constraint, ">=20");
        assert_eq!(declaration.role, Role::Support);
        assert_eq!(declaration.source, "package.json");
    }

    #[test]
    fn returns_none_when_node_engine_is_missing() {
        let json = r#"
                                                                                                                                                                                                                                                                {
                                                                                                                                                                                                                                                                            "name": "example-project"
                                                                                                                                                                                                                                                                                    }
                                                                                                                                                                                                                                                                                            "#;

        let declaration = parse_package_json(json, "package.json").unwrap();

        assert_eq!(declaration, None);
    }

    #[test]
    fn rejects_invalid_json() {
        let result = parse_package_json("{ invalid json", "package.json");

        assert!(result.is_err());
    }
}
