use crate::model::{Declaration, Role, Runtime};

pub fn parse_dockerfile(contents: &str, source: &str) -> Vec<Declaration> {
    let mut stages: Vec<Option<String>> = Vec::new();

    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();

        let Some(instruction) = parts.next() else {
            continue;
        };

        if !instruction.eq_ignore_ascii_case("FROM") {
            continue;
        }

        let image = parts.find(|part| !part.starts_with("--"));
        let version = image.and_then(extract_node_version);

        stages.push(version);
    }

    let last_stage = stages.len().checked_sub(1);
    let mut declarations = Vec::new();

    for (index, version) in stages.into_iter().enumerate() {
        let Some(constraint) = version else {
            continue;
        };

        let role = if Some(index) == last_stage {
            Role::Shipped
        } else {
            Role::Build
        };

        declarations.push(Declaration {
            runtime: Runtime::Node,
            constraint,
            role,
            source: source.to_string(),
        });
    }

    declarations
}

fn extract_node_version(image: &str) -> Option<String> {
    let image = image.split('@').next().unwrap_or(image);
    let image = image.rsplit('/').next().unwrap_or(image);

    let tag = image.strip_prefix("node:")?;
    let tag = tag.strip_prefix('v').unwrap_or(tag);

    let version = tag.split('-').next().unwrap_or(tag);

    if version
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        Some(version.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_node_image_as_shipped() {
        let declarations = parse_dockerfile("FROM node:22\n", "Dockerfile");

        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].runtime, Runtime::Node);
        assert_eq!(declarations[0].constraint, "22");
        assert_eq!(declarations[0].role, Role::Shipped);
        assert_eq!(declarations[0].source, "Dockerfile");
    }

    #[test]
    fn removes_node_image_variant_suffix() {
        let declarations = parse_dockerfile("FROM node:22.4.1-alpine\n", "Dockerfile");

        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].constraint, "22.4.1");
    }

    #[test]
    fn distinguishes_build_and_shipped_stages() {
        let dockerfile = r#"
FROM node:22 AS builder
RUN npm run build

FROM node:20-alpine
CMD ["node", "server.js"]
"#;

        let declarations = parse_dockerfile(dockerfile, "Dockerfile");

        assert_eq!(declarations.len(), 2);

        assert_eq!(declarations[0].constraint, "22");
        assert_eq!(declarations[0].role, Role::Build);

        assert_eq!(declarations[1].constraint, "20");
        assert_eq!(declarations[1].role, Role::Shipped);
    }

    #[test]
    fn node_stage_is_build_when_final_stage_is_not_node() {
        let dockerfile = r#"
FROM node:22 AS builder
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
"#;

        let declarations = parse_dockerfile(dockerfile, "Dockerfile");

        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].constraint, "22");
        assert_eq!(declarations[0].role, Role::Build);
    }

    #[test]
    fn ignores_non_numeric_node_tags_and_other_images() {
        let dockerfile = r#"
FROM node:lts AS builder
FROM nginx:alpine
"#;

        let declarations = parse_dockerfile(dockerfile, "Dockerfile");

        assert!(declarations.is_empty());
    }
}
