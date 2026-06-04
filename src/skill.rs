//! Skill pack parser: parse "intent: {{param}}" templates, validate params.

use std::collections::HashMap;
use std::fmt;

/// A parsed skill template parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub required: bool,
    pub default: Option<String>,
}

/// A single skill entry.
#[derive(Clone, Debug, PartialEq)]
pub struct Skill {
    pub intent: String,
    pub params: Vec<Param>,
    pub raw_template: String,
}

/// A skill pack (collection of skills).
#[derive(Clone, Debug, PartialEq)]
pub struct SkillPack {
    pub name: String,
    pub version: String,
    pub skills: Vec<Skill>,
}

impl SkillPack {
    /// Parse a skill pack from text.
    ///
    /// Format:
    /// ```text
    /// ---
    /// name: my-pack
    /// version: "1.0"
    /// ---
    /// intent_name: do something with {{param1}} and {{param2?}}
    /// another_intent: {{required}} goes here {{optional?default_value}}
    /// ```
    ///
    /// Parameters:
    /// - `{{name}}` — required
    /// - `{{name?}}` — optional
    /// - `{{name?default}}` — optional with default value
    pub fn parse(input: &str) -> Result<Self, SkillParseError> {
        let mut lines = input.lines();
        let mut name = String::from("unnamed");
        let mut version = String::from("0.1.0");
        let mut skills = Vec::new();

        // Parse optional YAML front matter
        let first = lines.next().map(|l| l.trim());
        if first.as_deref() == Some("---") {
            // Read front matter until closing ---
            for line in &mut lines {
                let trimmed = line.trim();
                if trimmed == "---" {
                    break;
                }
                if let Some((key, value)) = trimmed.split_once(':') {
                    match key.trim() {
                        "name" => name = value.trim().trim_matches('"').to_string(),
                        "version" => version = value.trim().trim_matches('"').to_string(),
                        _ => {}
                    }
                }
            }
        } else {
            // No front matter — reset and parse all lines as skills
            lines = input.lines();
        }

        // Parse skill entries
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((intent, template)) = trimmed.split_once(':') {
                let intent = intent.trim().to_string();
                let template = template.trim().to_string();
                let params = parse_params(&template)?;

                skills.push(Skill {
                    intent,
                    params,
                    raw_template: template,
                });
            }
        }

        Ok(SkillPack {
            name,
            version,
            skills,
        })
    }

    /// Render a skill template with given parameter values.
    pub fn render(&self, skill_idx: usize, values: &HashMap<&str, &str>) -> Result<String, RenderError> {
        let skill = self.skills.get(skill_idx).ok_or(RenderError::InvalidIndex)?;

        // Check required params
        for param in &skill.params {
            if param.required && !values.contains_key(param.name.as_str()) {
                return Err(RenderError::MissingRequired(param.name.clone()));
            }
        }

        let mut result = skill.raw_template.clone();
        for param in &skill.params {
            let value = values
                .get(param.name.as_str())
                .map(|s| s.to_string())
                .or_else(|| param.default.clone())
                .unwrap_or_default();

            // Replace all param placeholder forms
            let brace_name = format!("{{{{{param}}}}}", param = param.name);  // {{name}}
            let brace_opt = format!("{{{{{param}?}}}}", param = param.name);   // {{name?}}
            result = result.replace(&brace_name, &value);
            result = result.replace(&brace_opt, &value);
            if let Some(default) = &param.default {
                let brace_def = format!("{{{{{param}?{default}}}}}", param = param.name, default = default);
                result = result.replace(&brace_def, &value);
            }
        }

        Ok(result)
    }
}

/// Parse {{param}} placeholders from a template string.
fn parse_params(template: &str) -> Result<Vec<Param>, SkillParseError> {
    let mut params = Vec::new();
    let mut in_brace = false;
    let mut brace_content = String::new();
    let mut chars = template.chars().peekable();

    loop {
        let c = chars.next();
        match c {
            Some('{') => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    in_brace = true;
                    brace_content.clear();
                }
            }
            Some('}') => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    in_brace = false;
                    let param = parse_param_content(&brace_content)?;
                    // Avoid duplicates
                    if !params.iter().any(|p: &Param| p.name == param.name) {
                        params.push(param);
                    }
                }
            }
            Some(c) if in_brace => {
                brace_content.push(c);
            }
            _ => {}
        }
        if c.is_none() {
            break;
        }
    }

    Ok(params)
}

fn parse_param_content(content: &str) -> Result<Param, SkillParseError> {
    let content = content.trim();
    if content.is_empty() {
        return Err(SkillParseError::InvalidParam("empty param name".to_string()));
    }

    if let Some(idx) = content.find('?') {
        let name = content[..idx].trim().to_string();
        let default_part = content[idx + 1..].trim().to_string();
        Ok(Param {
            name,
            required: false,
            default: if default_part.is_empty() {
                None
            } else {
                Some(default_part)
            },
        })
    } else {
        Ok(Param {
            name: content.to_string(),
            required: true,
            default: None,
        })
    }
}

#[derive(Debug)]
pub enum SkillParseError {
    InvalidParam(String),
}

impl fmt::Display for SkillParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillParseError::InvalidParam(msg) => write!(f, "invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for SkillParseError {}

#[derive(Debug)]
pub enum RenderError {
    InvalidIndex,
    MissingRequired(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::InvalidIndex => write!(f, "invalid skill index"),
            RenderError::MissingRequired(name) => {
                write!(f, "missing required parameter: {}", name)
            }
        }
    }
}

impl std::error::Error for RenderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let pack = SkillPack::parse("greet: Hello {{name}}, welcome!").unwrap();
        assert_eq!(pack.skills.len(), 1);
        assert_eq!(pack.skills[0].intent, "greet");
        assert_eq!(pack.skills[0].params.len(), 1);
        assert_eq!(pack.skills[0].params[0].name, "name");
        assert!(pack.skills[0].params[0].required);
    }

    #[test]
    fn test_parse_optional() {
        let pack = SkillPack::parse("greet: Hello {{name}} with {{title?}}").unwrap();
        assert_eq!(pack.skills[0].params.len(), 2);
        assert!(pack.skills[0].params[0].required);
        assert!(!pack.skills[0].params[1].required);
    }

    #[test]
    fn test_parse_optional_with_default() {
        let pack = SkillPack::parse("greet: Hello {{name}} ({{title?friend}})").unwrap();
        assert_eq!(pack.skills[0].params[1].default, Some("friend".to_string()));
    }

    #[test]
    fn test_parse_front_matter() {
        let input = "---\nname: test-pack\nversion: \"2.0\"\n---\ngreet: Hi {{name}}";
        let pack = SkillPack::parse(input).unwrap();
        assert_eq!(pack.name, "test-pack");
        assert_eq!(pack.version, "2.0");
    }

    #[test]
    fn test_render() {
        let pack = SkillPack::parse("greet: Hello {{name}}!").unwrap();
        let mut values = HashMap::new();
        values.insert("name", "World");
        let result = pack.render(0, &values).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_render_missing_required() {
        let pack = SkillPack::parse("greet: Hello {{name}}!").unwrap();
        let values = HashMap::new();
        let result = pack.render(0, &values);
        assert!(matches!(result, Err(RenderError::MissingRequired(_))));
    }
}
