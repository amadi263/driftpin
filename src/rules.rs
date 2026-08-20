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

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(constraint: &str, role: Role) -> Declaration {
        Declaration {
            runtime: Runtime::Node,
            constraint: constraint.to_string(),
            role,
            source: "test".to_string(),
        }
    }

    #[test]
    fn accepts_development_version_inside_support_range() {
        let declarations = vec![
            declaration(">=20", Role::Support),
            declaration("22", Role::Development),
        ];

        assert!(!development_outside_support(&declarations).unwrap());
    }

    #[test]
    fn detects_development_version_outside_support_range() {
        let declarations = vec![
            declaration(">=20", Role::Support),
            declaration("18", Role::Development),
        ];

        assert!(development_outside_support(&declarations).unwrap());
    }

    #[test]
    fn does_nothing_when_declarations_are_missing() {
        let declarations = vec![declaration(">=20", Role::Support)];

        assert!(!development_outside_support(&declarations).unwrap());
    }
}
