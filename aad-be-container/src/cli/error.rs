//! Typed error hierarchy for `aad-be ctl` CLI operations.

use thiserror::Error;

/// Domain-specific errors produced during CLI manifest loading, validation,
/// planning, authentication, and REST API execution.
#[derive(Debug, Error)]
pub enum CtlError {
    #[error("I/O error on '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid base URL '{url}': {reason}")]
    InvalidUrl { url: String, reason: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid template variable '{raw}': expected KEY=VALUE")]
    InvalidVarFormat { raw: String },

    #[error("Invalid variables file '{path}': {message}")]
    InvalidVarsFile { path: String, message: String },

    #[error("Unbound template variable '${{{name}}}' in '{source_name}'")]
    UnboundTemplateVariable { name: String, source_name: String },

    #[error("YAML parse error in '{source_name}' (doc #{doc_index}): {message}")]
    YamlParse {
        source_name: String,
        doc_index: usize,
        message: String,
    },

    #[error("Schema validation error in '{source_name}' (doc #{doc_index}): {message}")]
    SchemaValidation {
        source_name: String,
        doc_index: usize,
        message: String,
    },

    #[error(
        "Duplicate {kind} resource named '{name}' (defined in '{first_source}' and '{second_source}')"
    )]
    DuplicateResource {
        kind: String,
        name: String,
        first_source: String,
        second_source: String,
    },

    #[error("Dependency cycle detected among {kind} resources: {cycle}")]
    DependencyCycle { kind: String, cycle: String },

    #[error(
        "Unresolved {ref_kind} reference '{reference}' required by {owner_kind} '{owner_name}'"
    )]
    UnresolvedReference {
        owner_kind: String,
        owner_name: String,
        ref_kind: String,
        reference: String,
    },

    #[error(
        "Unsupported resource type '{0}' (expected: trait, tool, skill, agent, or all)"
    )]
    UnsupportedResourceType(String),

    #[error("Resource not found: {kind} '{identifier}'")]
    ResourceNotFound { kind: String, identifier: String },

    #[error("No YAML manifest files found in '{path}'")]
    NoManifestFiles { path: String },

    #[error("HTTP transport error calling {method} {url}: {source}")]
    HttpTransport {
        method: String,
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("API error ({status}) on {method} {url}: {body}")]
    ApiError {
        method: String,
        url: String,
        status: u16,
        body: String,
    },

    #[error("Serialization error: {0}")]
    Serialization(String),
}
