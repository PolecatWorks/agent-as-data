use serde::{Deserialize, Serialize};

pub fn default_version() -> String {
    "1.0.0".to_string()
}

pub fn bump_minor_version(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2
        && let Ok(major) = parts[0].parse::<i32>()
        && let Ok(minor) = parts[1].parse::<i32>()
    {
        return format!("{}.{}.0", major, minor + 1);
    }
    // Fallback if not valid SemVer
    let clean = version.trim_matches(|c: char| !c.is_numeric());
    if let Ok(num) = clean.parse::<i32>() {
        return format!("{}.0.0", num + 1);
    }
    "1.1.0".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageOptions {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

impl Default for PageOptions {
    fn default() -> Self {
        Self {
            size: Some(10),
            page: Some(0),
        }
    }
}

impl PageOptions {
    pub fn defaulting(inval: PageOptions) -> PageOptions {
        PageOptions {
            size: Some(inval.size.unwrap_or(10)),
            page: Some(inval.page.unwrap_or(0)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPages {
    pub ids: Vec<uuid::Uuid>,
    pub pagination: PageOptions,
}
