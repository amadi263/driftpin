use crate::model::{Declaration, Role, Runtime};
use pep440_rs::{Version, VersionSpecifiers};
use std::str::FromStr;

pub fn python_development_outside_support(declarations: &[Declaration]) -> Result<bool, String> {
    let support = declarations.iter().find(|declaration| {
        declaration.runtime == Runtime::Python && declaration.role == Role::Support
    });

    let development = declarations.iter().find(|declaration| {
        declaration.runtime == Runtime::Python && declaration.role == Role::Development
    });

    let (Some(support), Some(development)) = (support, development) else {
        return Ok(false);
    };

    let support_range = VersionSpecifiers::from_str(&support.constraint).map_err(|error| {
        format!(
            "Invalid Python support range in {}: {error}",
            support.source
        )
    })?;

    let development_version = Version::from_str(&development.constraint).map_err(|error| {
        format!(
            "Invalid Python development version in {}: {error}",
            development.source
        )
    })?;

    Ok(!support_range.contains(&development_version))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(constraint: &str, role: Role, source: &str) -> Declaration {
        Declaration {
            runtime: Runtime::Python,
            constraint: constraint.to_string(),
            role,
            source: source.to_string(),
        }
    }

    #[test]
    fn accepts_python_inside_support_range() {
        let declarations = vec![
            declaration(">=3.11,<3.13", Role::Support, "pyproject.toml"),
            declaration("3.12.4", Role::Development, ".python-version"),
        ];

        assert!(!python_development_outside_support(&declarations).unwrap());
    }

    #[test]
    fn detects_python_outside_support_range() {
        let declarations = vec![
            declaration(">=3.12", Role::Support, "pyproject.toml"),
            declaration("3.11", Role::Development, ".python-version"),
        ];

        assert!(python_development_outside_support(&declarations).unwrap());
    }

    #[test]
    fn does_nothing_without_python_support_range() {
        let declarations = vec![declaration("3.12", Role::Development, ".python-version")];

        assert!(!python_development_outside_support(&declarations).unwrap());
    }
}
