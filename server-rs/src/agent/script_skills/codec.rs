//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - **Subsystem**: Script skills / Markdown codec
//! - Pure frontmatter/body parsing is independent of registry state and filesystem access.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::SkillDefinition;
use serde_json::json;

/// ### 🧪 Logic: Semantic Skill Extraction (parse_skill_md)
/// Parses a self-describing `SKILL.md` file using YAML frontmatter extraction.
///
/// ### 🧬 Rationale: Self-Documenting Swarms
/// Allows agents to discover toolsets that include their own documentation,
/// schema, and execution commands in a single human-readable Markdown file.
/// Following the industry standard for SSG (Static Site Generator) metadata.
pub fn parse_skill_md(content: &str) -> Option<SkillDefinition> {
    let content_trimmed = content.trim_start();
    let lines = content_trimmed.lines();

    let mut yaml_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut found_start = false;
    let mut found_end = false;

    for line in lines {
        if !found_start {
            if line.trim() == "---" {
                found_start = true;
            }
        } else if !found_end {
            if line.trim() == "---" {
                found_end = true;
            } else {
                yaml_lines.push(line);
            }
        } else {
            body_lines.push(line);
        }
    }

    if !found_start || !found_end {
        return None;
    }

    let yaml_str = yaml_lines.join("\n");
    let body = body_lines.join("\n");

    if yaml_str.len() > 65536 {
        tracing::warn!(
            "⚠️ Guard: frontmatter size exceeds limit of 64 KiB ({} bytes)",
            yaml_str.len()
        );
        return None;
    }

    let metadata: serde_json::Value = serde_norway::from_str(&yaml_str).ok()?;
    let name = metadata
        .get("name")
        .and_then(|v| v.as_str())
        .or_else(|| metadata.get("title").and_then(|v| v.as_str()))
        .or(None)?;

    let description = metadata
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(SkillDefinition {
        id: None,
        name: name.to_string(),
        description,
        execution_command: metadata
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        schema: metadata
            .get("schema")
            .cloned()
            .unwrap_or(json!({ "type": "object", "properties": {} })),
        oversight_required: metadata
            .get("oversight")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        doc_url: metadata
            .get("doc_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        tags: metadata.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        }),
        full_instructions: Some(body.trim().to_string()),
        negative_constraints: None,
        verification_script: None,
        category: "user".to_string(),
        security_score: None,
        security_severity: None,
        security_report: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the basic markdown parsing functionality for skills.
    /// This follows industry standards for Arrange-Act-Assert (AAA) pattern.
    #[test]
    fn test_parse_skill_md_basic() {
        // Arrange
        let content = r#"---
name: test_skill
description: A test skill
command: python test.py
oversight: false
tags: ["test", "verify"]
---
This is the body content."#;

        // Act
        let skill = parse_skill_md(content).expect("Should parse valid markdown");

        // Assert
        assert_eq!(skill.name, "test_skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.execution_command, "python test.py");
        assert!(!skill.oversight_required);
        assert_eq!(
            skill.tags.unwrap(),
            vec!["test".to_string(), "verify".to_string()]
        );
        assert_eq!(
            skill.full_instructions.unwrap(),
            "This is the body content."
        );
    }

    /// Tests the title fallback mechanism when 'name' is missing in frontmatter.
    #[test]
    fn test_parse_skill_md_title_fallback() {
        // Arrange
        let content = r#"---
title: My Advanced Skill
description: Fallback test
---
Body"#;

        // Act
        let skill = parse_skill_md(content).expect("Should fallback to title");

        // Assert
        assert_eq!(skill.name, "My Advanced Skill");
    }

    /// Tests that invalid markdown (missing frontmatter) returns None.
    #[test]
    fn test_parse_skill_md_invalid() {
        // Arrange
        let content = "Just some random text";

        // Act
        let skill = parse_skill_md(content);

        // Assert
        assert!(
            skill.is_none(),
            "Should return None for invalid markdown structure"
        );
    }

    /// Tests parsing with a complex JSON schema in the frontmatter.
    #[test]
    fn test_parse_skill_md_with_schema() {
        // Arrange
        let content = r#"---
name: schema_skill
schema:
  type: object
  properties:
    query:
      type: string
---
Body"#;

        // Act
        let skill = parse_skill_md(content).expect("Should parse schema");

        // Assert
        assert_eq!(skill.schema["type"], "object");
        assert_eq!(skill.schema["properties"]["query"]["type"], "string");
    }

    #[test]
    fn test_parse_skill_md_robustness() {
        let content = " \n\n  ---\nname: test_whitespace\n---\nbody";
        let skill = parse_skill_md(content).expect("Should parse with leading whitespace");
        assert_eq!(skill.name, "test_whitespace");
        assert_eq!(skill.full_instructions.unwrap(), "body");
    }

    #[test]
    fn test_parse_skill_md_body_separators() {
        let content = "---\nname: test_separators\n---\nbody\n---\ninner\n---\nmore";
        let skill = parse_skill_md(content).expect("Should parse with separators in body");
        assert_eq!(skill.name, "test_separators");
        assert_eq!(
            skill.full_instructions.unwrap(),
            "body\n---\ninner\n---\nmore"
        );
    }

    #[test]
    fn test_parse_skill_md_size_limit() {
        let large_yaml = "a: ".repeat(40000);
        let content = format!("---\nname: test_large\n{}---\nbody", large_yaml);
        let skill = parse_skill_md(&content);
        assert!(skill.is_none(), "Should reject extremely large YAML block");
    }
}
