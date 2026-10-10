---
name: rust-refactoring-standards
description: >-
  Standard practices and rules for refactoring Rust code in the agent-as-data project, 
  including flattening control flow, strict sqlx error handling, and using domain enums.
---

# Rust Refactoring Standards

This skill defines the code quality and refactoring standards used when maintaining and extending the backend Rust (`aad-be-container`) codebase. Adhere to these principles whenever adding new features or refactoring existing implementations.

## Core Refactoring Principles

1. **Flatten Control Flow (`let else`)**
   Avoid deep nesting of `if let Some` or `match` blocks. Rely on early returns, `let else` guard clauses, and inverted conditionals (`if !condition { return; }`) to keep the primary "happy path" flat and highly readable.
   
2. **Strict SQLx Error Surfacing**
   Never silently swallow errors from database queries (e.g., `let _ = sqlx::query(...).execute().await;`).
   - If executing within a standard handler or function that returns a `Result`, explicitly surface the error using the `?` operator.
   - If executing within a detached background task (`tokio::spawn`), catch the error and explicitly log it using `tracing::error!`. Do not let database failures go unrecorded.

3. **Domain Enums over Strings**
   Avoid using raw strings for strongly typed concepts such as lifecycle statuses, phases, or user roles.
   Map these concepts natively to Rust `enum` types using `sqlx::Type` and `serde`.
   
   **Example Implementation:**
   ```rust
   #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, sqlx::Type)]
   #[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
   #[serde(rename_all = "lowercase")]
   pub enum MessageRole {
       User,
       Assistant,
       System,
   }
   ```

4. **Targeted AppError Variants**
   Do not overload generic error variants like `AppError::Message` for specific failure domains. Always define specific, descriptive error variants (e.g., `AppError::ToolHallucination`) in `src/error.rs`. This allows higher-level control flows and HTTP mappers to respond accurately and semantically to failures.
