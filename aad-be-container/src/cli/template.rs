//! Template variable loading and `${VAR}` / `${VAR:-default}` interpolation.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::cli::error::CtlError;

/// Key-value variable bindings used to interpolate YAML manifests before parsing.
#[derive(Debug, Default, Clone)]
pub struct TemplateVars {
    values: BTreeMap<String, String>,
}

impl TemplateVars {
    /// Builds template variables from an optional YAML `--vars-file` and repeatable
    /// `--var KEY=VALUE` CLI overrides (CLI overrides take precedence).
    pub fn from_sources(
        vars_file: Option<&Path>,
        cli_vars: &[String],
    ) -> Result<Self, CtlError> {
        let mut values = BTreeMap::new();

        if let Some(path) = vars_file {
            let raw = fs::read_to_string(path).map_err(|source| CtlError::Io {
                path: path.display().to_string(),
                source,
            })?;
            let parsed: BTreeMap<String, serde_yaml::Value> =
                serde_yaml::from_str(&raw).map_err(|e| CtlError::InvalidVarsFile {
                    path: path.display().to_string(),
                    message: e.to_string(),
                })?;

            for (key, val) in parsed {
                let rendered = match val {
                    serde_yaml::Value::String(s) => s,
                    serde_yaml::Value::Number(n) => n.to_string(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    serde_yaml::Value::Null => String::new(),
                    other => {
                        return Err(CtlError::InvalidVarsFile {
                            path: path.display().to_string(),
                            message: format!(
                                "variable '{}' must be a scalar value, got {:?}",
                                key, other
                            ),
                        });
                    }
                };
                values.insert(key, rendered);
            }
        }

        for entry in cli_vars {
            let Some((key, val)) = entry.split_once('=') else {
                return Err(CtlError::InvalidVarFormat { raw: entry.clone() });
            };
            let trimmed_key = key.trim();
            if !is_valid_var_name(trimmed_key) {
                return Err(CtlError::InvalidVarFormat { raw: entry.clone() });
            }
            values.insert(trimmed_key.to_string(), val.to_string());
        }

        Ok(Self { values })
    }

    /// Inserts a variable binding programmatically (useful in tests).
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    /// Interpolates `${VAR}` and `${VAR:-default}` placeholders in `input`.
    pub fn render(&self, input: &str, source_name: &str) -> Result<String, CtlError> {
        let mut out = String::with_capacity(input.len());
        let bytes = input.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                let start = i + 2;
                let Some(rel_end) = input[start..].find('}') else {
                    out.push('$');
                    i += 1;
                    continue;
                };
                let expr = &input[start..start + rel_end];
                let (var_name, default_val) = match expr.split_once(":-") {
                    Some((name, def)) => (name, Some(def)),
                    None => (expr, None),
                };

                if !is_valid_var_name(var_name) {
                    // Not a valid template placeholder; emit verbatim.
                    out.push_str(&input[i..start + rel_end + 1]);
                    i = start + rel_end + 1;
                    continue;
                }

                if let Some(bound) = self.values.get(var_name)
                    && !bound.is_empty()
                {
                    out.push_str(bound);
                } else if let Some(def) = default_val {
                    out.push_str(def);
                } else {
                    return Err(CtlError::UnboundTemplateVariable {
                        name: var_name.to_string(),
                        source_name: source_name.to_string(),
                    });
                }

                i = start + rel_end + 1;
            } else {
                let ch = input[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
        }

        Ok(out)
    }
}

fn is_valid_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_with_defaults_and_overrides() {
        let mut vars = TemplateVars::default();
        vars.insert("HOST", "ai.k8s");

        let rendered = vars
            .render(
                "url: \"http://${HOST:-localhost}:8080/${PREFIX:-api/v1}\"",
                "test.yaml",
            )
            .unwrap();
        assert_eq!(rendered, "url: \"http://ai.k8s:8080/api/v1\"");
    }

    #[test]
    fn test_render_fails_on_unbound_required_var() {
        let vars = TemplateVars::default();
        let err = vars.render("url: \"${MISSING_URL}\"", "tool.yaml").unwrap_err();
        assert!(matches!(
            err,
            CtlError::UnboundTemplateVariable { ref name, .. } if name == "MISSING_URL"
        ));
    }

    #[test]
    fn test_cli_vars_override_vars_file() {
        let dir = std::env::temp_dir().join(format!("aad-ctl-vars-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let vars_path = dir.join("env.yaml");
        fs::write(&vars_path, "A: \"from_file\"\nB: \"only_file\"\n").unwrap();

        let vars =
            TemplateVars::from_sources(Some(&vars_path), &["A=from_cli".to_string()]).unwrap();
        assert_eq!(vars.render("${A}|${B}", "doc.yaml").unwrap(), "from_cli|only_file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_invalid_var_format_rejected() {
        let err = TemplateVars::from_sources(None, &["NO_EQUALS".to_string()]).unwrap_err();
        assert!(matches!(err, CtlError::InvalidVarFormat { .. }));
    }
}
