#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runtime {
    Node,
    Python,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Support,
    Development,
    Test,
    Build,
    Shipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub runtime: Runtime,
    pub constraint: String,
    pub role: Role,
    pub source: String,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_node_support_declaration() {
        let declaration = Declaration {
            runtime: Runtime::Node,
            constraint: ">=20".to_string(),
            role: Role::Support,
            source: "package.json".to_string(),
        };

        assert_eq!(declaration.runtime, Runtime::Node);
        assert_eq!(declaration.constraint, ">=20");
        assert_eq!(declaration.role, Role::Support);
        assert_eq!(declaration.source, "package.json");
    }
}
