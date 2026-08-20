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
}
