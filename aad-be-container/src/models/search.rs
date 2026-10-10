use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, schemars::JsonSchema)]
#[sqlx(type_name = "VARCHAR")]
pub enum EntityType {
    #[sqlx(rename = "agents")]
    #[serde(rename = "agents")]
    Agents,
    #[sqlx(rename = "skills")]
    #[serde(rename = "skills")]
    Skills,
    #[sqlx(rename = "tools")]
    #[serde(rename = "tools")]
    Tools,
    #[sqlx(rename = "traits")]
    #[serde(rename = "traits")]
    Traits,
    #[sqlx(rename = "knowledge_nodes")]
    #[serde(rename = "knowledge_nodes")]
    KnowledgeNodes,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Agents => write!(f, "agents"),
            EntityType::Skills => write!(f, "skills"),
            EntityType::Tools => write!(f, "tools"),
            EntityType::Traits => write!(f, "traits"),
            EntityType::KnowledgeNodes => write!(f, "knowledge_nodes"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, schemars::JsonSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    Semantic,
    Fulltext,
    Hybrid,
}

impl Default for SearchType {
    fn default() -> Self {
        SearchType::Semantic
    }
}

impl fmt::Display for SearchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchType::Semantic => write!(f, "semantic"),
            SearchType::Fulltext => write!(f, "fulltext"),
            SearchType::Hybrid => write!(f, "hybrid"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SemanticSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SemanticSearchResult {
    pub entity_id: Uuid,
    pub entity_type: EntityType,
    pub name: Option<String>,
    pub description: Option<String>,
    pub field_name: String,
    pub content: String,
    pub score: f64,
    pub match_reason: String,
    pub search_type: SearchType,
    pub origin_id: Option<Uuid>,
    pub origin_type: Option<EntityType>,
    pub origin_uri: Option<String>,
    pub origin_name: Option<String>,
}
