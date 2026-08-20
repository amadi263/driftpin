use crate::model::{Declaration, Role, Runtime};
use node_semver::Range;

pub fn development_outside_support(declarations: &[Declaration]) -> Result<bool, String> {
    let support = declarations.iter().find(|declaration| {
        declaration.runtime == Runtime::Node && declaration.role == Role::Support
    });

    let development = declarations.iter().find(|declaration| {
        declaration.runtime == Runtime::Node && declaration.role == Role::Development
    });

    let (Some(support), Some(development)) = (support, development) else {
        return Ok(false);
    };

    let support_range: Range = support
        .constraint
        .parse()
        .map_err(|error| format!("Invalid support range: {error}"))?;

    let development_range: Range = development
        .constraint
        .parse()
        .map_err(|error| format!("Invalid development range: {error}"))?;

    let overlap = support_range.intersect(&development_range);

    Ok(overlap.as_ref() != Some(&development_range))
}

pub fn development_declarations_conflict(declarations: &[Declaration]) -> Result<bool, String> {
    let development: Vec<&Declaration> = declarations
        .iter()
        .filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Development
        })
        .collect();

    for (index, left) in development.iter().enumerate() {
        let left_range: Range = left
            .constraint
            .parse()
            .map_err(|error| format!("Invalid development range in {}: {error}", left.source))?;

        for right in development.iter().skip(index + 1) {
            let right_range: Range = right.constraint.parse().map_err(|error| {
                format!("Invalid development range in {}: {error}", right.source)
            })?;

            if left_range.intersect(&right_range).is_none() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

pub fn development_and_test_conflict(declarations: &[Declaration]) -> Result<bool, String> {
    let developments: Vec<&Declaration> = declarations
        .iter()
        .filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Development
        })
        .collect();

    let tests: Vec<&Declaration> = declarations
        .iter()
        .filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Test
        })
        .collect();

    if developments.is_empty() || tests.is_empty() {
        return Ok(false);
    }

    for development in developments {
        let development_range: Range = development.constraint.parse().map_err(|error| {
            format!(
                "Invalid development range in {}: {error}",
                development.source
            )
        })?;

        let mut compatible_with_ci = false;

        for test in &tests {
            let test_range: Range = test
                .constraint
                .parse()
                .map_err(|error| format!("Invalid test range in {}: {error}", test.source))?;

            if development_range.intersect(&test_range).is_some() {
                compatible_with_ci = true;
                break;
            }
        }

        if !compatible_with_ci {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn development_and_shipped_conflict(declarations: &[Declaration]) -> Result<bool, String> {
    let developments: Vec<&Declaration> = declarations
        .iter()
        .filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Development
        })
        .collect();

    let shipped: Vec<&Declaration> = declarations
        .iter()
        .filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Shipped
        })
        .collect();

    if developments.is_empty() || shipped.is_empty() {
        return Ok(false);
    }

    for development in developments {
        let development_range: Range = development.constraint.parse().map_err(|error| {
            format!(
                "Invalid development range in {}: {error}",
                development.source
            )
        })?;

        let mut compatible = false;

        for shipped_runtime in &shipped {
            let shipped_range: Range = shipped_runtime.constraint.parse().map_err(|error| {
                format!(
                    "Invalid shipped range in {}: {error}",
                    shipped_runtime.source
                )
            })?;

            if development_range.intersect(&shipped_range).is_some() {
                compatible = true;
                break;
            }
        }

        if !compatible {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(constraint: &str, role: Role, source: &str) -> Declaration {
        Declaration {
            runtime: Runtime::Node,
            constraint: constraint.to_string(),
            role,
            source: source.to_string(),
        }
    }

    #[test]
    fn accepts_development_version_inside_support_range() {
        let declarations = vec![
            declaration(">=20", Role::Support, "package.json"),
            declaration("22", Role::Development, ".nvmrc"),
        ];

        assert!(!development_outside_support(&declarations).unwrap());
    }

    #[test]
    fn detects_development_version_outside_support_range() {
        let declarations = vec![
            declaration(">=20", Role::Support, "package.json"),
            declaration("18", Role::Development, ".nvmrc"),
        ];

        assert!(development_outside_support(&declarations).unwrap());
    }

    #[test]
    fn support_rule_does_nothing_when_declarations_are_missing() {
        let declarations = vec![declaration(">=20", Role::Support, "package.json")];

        assert!(!development_outside_support(&declarations).unwrap());
    }

    #[test]
    fn detects_conflicting_development_declarations() {
        let declarations = vec![
            declaration("22", Role::Development, ".nvmrc"),
            declaration("20", Role::Development, ".node-version"),
        ];

        assert!(development_declarations_conflict(&declarations).unwrap());
    }

    #[test]
    fn accepts_matching_development_declarations() {
        let declarations = vec![
            declaration("22", Role::Development, ".nvmrc"),
            declaration("22", Role::Development, ".node-version"),
        ];

        assert!(!development_declarations_conflict(&declarations).unwrap());
    }

    #[test]
    fn development_conflict_rule_ignores_single_declaration() {
        let declarations = vec![declaration("22", Role::Development, ".nvmrc")];

        assert!(!development_declarations_conflict(&declarations).unwrap());
    }

    #[test]
    fn detects_development_and_ci_conflict() {
        let declarations = vec![
            declaration("22", Role::Development, ".nvmrc"),
            declaration("18", Role::Test, ".github/workflows/ci.yml"),
        ];

        assert!(development_and_test_conflict(&declarations).unwrap());
    }

    #[test]
    fn accepts_matching_development_and_ci_versions() {
        let declarations = vec![
            declaration("22", Role::Development, ".nvmrc"),
            declaration("22", Role::Test, ".github/workflows/ci.yml"),
        ];

        assert!(!development_and_test_conflict(&declarations).unwrap());
    }

    #[test]
    fn ci_rule_does_nothing_without_ci_declaration() {
        let declarations = vec![declaration("22", Role::Development, ".nvmrc")];

        assert!(!development_and_test_conflict(&declarations).unwrap());
    }

    #[test]
    fn accepts_when_one_ci_matrix_version_matches_development() {
        let declarations = vec![
            declaration("22", Role::Development, ".nvmrc"),
            declaration("20", Role::Test, ".github/workflows/ci.yml"),
            declaration("22", Role::Test, ".github/workflows/ci.yml"),
        ];

        assert!(!development_and_test_conflict(&declarations).unwrap());
    }
}

#[cfg(test)]
mod shipped_tests {
    use super::*;

    fn declaration(constraint: &str, role: Role, source: &str) -> Declaration {
        Declaration {
            runtime: Runtime::Node,
            constraint: constraint.to_string(),
            role,
            source: source.to_string(),
        }
    }

    #[test]
    fn detects_development_and_shipped_conflict() {
        let declarations = vec![
            declaration("22", Role::Development, ".nvmrc"),
            declaration("18", Role::Shipped, "Dockerfile"),
        ];

        assert!(development_and_shipped_conflict(&declarations).unwrap());
    }

    #[test]
    fn accepts_matching_development_and_shipped_versions() {
        let declarations = vec![
            declaration("22", Role::Development, ".nvmrc"),
            declaration("22", Role::Shipped, "Dockerfile"),
        ];

        assert!(!development_and_shipped_conflict(&declarations).unwrap());
    }

    #[test]
    fn shipped_rule_does_nothing_without_container_runtime() {
        let declarations = vec![declaration("22", Role::Development, ".nvmrc")];

        assert!(!development_and_shipped_conflict(&declarations).unwrap());
    }
}
