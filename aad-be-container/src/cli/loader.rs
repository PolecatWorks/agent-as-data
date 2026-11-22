//! Multi-source YAML manifest loader (files, directories, recursive traversal, stdin).

use serde::Deserialize;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::cli::error::CtlError;
use crate::cli::manifest::{LoadedManifest, ResourceManifest};
use crate::cli::template::TemplateVars;

/// Loads, interpolates, and parses all YAML manifest documents from the given paths.
///
/// Supports single files, `-` for `stdin`, and directories (lexicographically sorted;
/// descending recursively when `recursive == true`).
pub fn load_manifests(
    paths: &[PathBuf],
    recursive: bool,
    vars: &TemplateVars,
) -> Result<Vec<LoadedManifest>, CtlError> {
    let mut loaded = Vec::new();

    for path in paths {
        if path.as_os_str() == "-" {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|source| CtlError::Io {
                    path: "<stdin>".to_string(),
                    source,
                })?;
            parse_yaml_stream(&buf, "<stdin>", vars, &mut loaded)?;
            continue;
        }

        let meta = fs::metadata(path).map_err(|source| CtlError::Io {
            path: path.display().to_string(),
            source,
        })?;

        if meta.is_file() {
            load_single_file(path, vars, &mut loaded)?;
        } else if meta.is_dir() {
            let mut yaml_files = Vec::new();
            collect_yaml_files(path, recursive, &mut yaml_files)?;
            yaml_files.sort();
            if yaml_files.is_empty() {
                return Err(CtlError::NoManifestFiles {
                    path: path.display().to_string(),
                });
            }
            for file in yaml_files {
                load_single_file(&file, vars, &mut loaded)?;
            }
        } else {
            return Err(CtlError::Io {
                path: path.display().to_string(),
                source: io::Error::new(io::ErrorKind::InvalidInput, "unsupported file type"),
            });
        }
    }

    Ok(loaded)
}

fn collect_yaml_files(
    dir: &Path,
    recursive: bool,
    out: &mut Vec<PathBuf>,
) -> Result<(), CtlError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|source| CtlError::Io {
            path: dir.display().to_string(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| CtlError::Io {
            path: dir.display().to_string(),
            source,
        })?
        .into_iter()
        .map(|entry| entry.path())
        .collect();

    entries.sort();

    for entry in entries {
        if entry.is_dir() {
            if recursive {
                collect_yaml_files(&entry, true, out)?;
            }
            continue;
        }
        if entry.is_file() && is_yaml_file(&entry) {
            out.push(entry);
        }
    }
    Ok(())
}

fn is_yaml_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml")
}

fn load_single_file(
    path: &Path,
    vars: &TemplateVars,
    out: &mut Vec<LoadedManifest>,
) -> Result<(), CtlError> {
    let source_name = path.display().to_string();
    let raw = fs::read_to_string(path).map_err(|source| CtlError::Io {
        path: source_name.clone(),
        source,
    })?;
    parse_yaml_stream(&raw, &source_name, vars, out)
}

/// Interpolates template variables and parses a single- or multi-document YAML string.
pub fn parse_yaml_stream(
    raw: &str,
    source_name: &str,
    vars: &TemplateVars,
    out: &mut Vec<LoadedManifest>,
) -> Result<(), CtlError> {
    let rendered = vars.render(raw, source_name)?;
    let de = serde_yaml::Deserializer::from_str(&rendered);

    for (idx, doc) in de.enumerate() {
        let doc_index = idx + 1;
        let yaml_val =
            serde_yaml::Value::deserialize(doc).map_err(|e| CtlError::YamlParse {
                source_name: source_name.to_string(),
                doc_index,
                message: e.to_string(),
            })?;

        // Ignore empty YAML documents (e.g. leading/trailing `---` separators or comment-only blocks)
        if yaml_val.is_null() {
            continue;
        }

        let json_val = serde_json::to_value(&yaml_val).map_err(|e| CtlError::YamlParse {
            source_name: source_name.to_string(),
            doc_index,
            message: e.to_string(),
        })?;

        let manifest: ResourceManifest =
            serde_json::from_value(json_val).map_err(|e| CtlError::SchemaValidation {
                source_name: source_name.to_string(),
                doc_index,
                message: e.to_string(),
            })?;

        out.push(LoadedManifest {
            source_name: source_name.to_string(),
            doc_index,
            manifest,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::manifest::ResourceKind;

    #[test]
    fn test_parse_multidoc_yaml_skips_empty_docs() {
        let yaml = r#"
---
# Empty leading separator
---
type: Trait
name: AuditTrait
description: Checks things
---
type: Tool
server_name: mcp-1
transport_type: http
endpoint_config:
  url: "${MCP_URL:-http://localhost:8080/api/aad/v1/mcp}"
---
"#;
        let vars = TemplateVars::default();
        let mut out = Vec::new();
        parse_yaml_stream(yaml, "bundle.yaml", &vars, &mut out).unwrap();

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].manifest.kind(), ResourceKind::Trait);
        assert_eq!(out[1].manifest.kind(), ResourceKind::Tool);
    }

    #[test]
    fn test_unknown_type_fails_schema_validation() {
        let yaml = "type: UnknownWidget\nname: foo\n";
        let vars = TemplateVars::default();
        let mut out = Vec::new();
        let err = parse_yaml_stream(yaml, "bad.yaml", &vars, &mut out).unwrap_err();
        assert!(matches!(err, CtlError::SchemaValidation { .. }));
    }

    #[test]
    fn test_directory_recursive_vs_non_recursive() {
        let root = std::env::temp_dir().join(format!("aad-ctl-load-{}", uuid::Uuid::new_v4()));
        let sub = root.join("nested");
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            root.join("top.yaml"),
            "type: Trait\nname: TopTrait\ndescription: top\n",
        )
        .unwrap();
        fs::write(
            sub.join("deep.yml"),
            "type: Trait\nname: DeepTrait\ndescription: deep\n",
        )
        .unwrap();
        fs::write(root.join("notes.txt"), "ignored").unwrap();

        let vars = TemplateVars::default();
        let non_rec = load_manifests(&[root.clone()], false, &vars).unwrap();
        assert_eq!(non_rec.len(), 1);
        assert_eq!(non_rec[0].manifest.canonical_name(), "TopTrait");

        let rec = load_manifests(&[root.clone()], true, &vars).unwrap();
        assert_eq!(rec.len(), 2);

        let _ = fs::remove_dir_all(&root);
    }
}
