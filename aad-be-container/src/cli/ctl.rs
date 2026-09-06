use clap::Subcommand;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::path::PathBuf;

use crate::models::agent::Agent;
use crate::models::skill::Skill;

#[derive(Subcommand, Debug)]
pub enum CtlCommands {
    /// Apply resources from a multi-part YAML file
    Apply {
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Get details of a resource or list resources
    Get {
        /// Resource type (e.g., 'agent', 'skill')
        resource_type: String,
        /// Optional resource ID/Name
        id: Option<String>,
    },
    /// Delete a resource
    Delete {
        /// Resource type (e.g., 'agent', 'skill')
        resource_type: String,
        /// Resource ID/Name
        id: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ResourceManifest {
    Agent(Agent),
    Skill(Skill),
}

pub async fn run_ctl(
    cmd: CtlCommands,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    match cmd {
        CtlCommands::Apply { file } => apply_resources(file, &client, base_url).await?,
        CtlCommands::Get { resource_type, id } => get_resource(resource_type, id, &client, base_url).await?,
        CtlCommands::Delete { resource_type, id } => delete_resource(resource_type, id, &client, base_url).await?,
    }
    Ok(())
}

async fn apply_resources(
    file_path: PathBuf,
    client: &Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(&file_path)?;
    let de = serde_yaml::Deserializer::from_reader(file);

    for doc in de {
        // First deserialize to a Value to inject strict validation early or check type
        let value = Value::deserialize(doc).map_err(|e| format!("YAML parsing error: {}", e))?;

        let manifest: ResourceManifest = serde_json::from_value(value.clone())
            .map_err(|e| format!("Schema validation error for document: {}\nJSON: {}", e, value))?;

        match manifest {
            ResourceManifest::Agent(agent) => {
                let url = format!("{}/v1/agents", base_url);
                let res = client.post(&url).json(&agent).send().await?;
                if !res.status().is_success() {
                    let err = res.text().await?;
                    return Err(format!("Failed to apply Agent {}: {}", agent.name, err).into());
                }
                println!("Agent '{}' applied successfully.", agent.name);
            }
            ResourceManifest::Skill(skill) => {
                let url = format!("{}/v1/skills", base_url);
                let res = client.post(&url).json(&skill).send().await?;
                if !res.status().is_success() {
                    let err = res.text().await?;
                    return Err(format!("Failed to apply Skill {}: {}", skill.name, err).into());
                }
                println!("Skill '{}' applied successfully.", skill.name);
            }
        }
    }
    Ok(())
}

async fn get_resource(
    resource_type: String,
    id: Option<String>,
    client: &Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let endpoint = match resource_type.to_lowercase().as_str() {
        "agent" | "agents" => "agents",
        "skill" | "skills" => "skills",
        _ => return Err(format!("Unsupported resource type: {}", resource_type).into()),
    };

    let url = if let Some(id) = id {
        format!("{}/v1/{}/{}", base_url, endpoint, id)
    } else {
        format!("{}/v1/{}", base_url, endpoint)
    };

    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        let err = res.text().await?;
        return Err(format!("Failed to get resource: {}", err).into());
    }

    let json: Value = res.json().await?;
    println!("{}", serde_yaml::to_string(&json)?);
    Ok(())
}

async fn delete_resource(
    resource_type: String,
    id: String,
    client: &Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let endpoint = match resource_type.to_lowercase().as_str() {
        "agent" | "agents" => "agents",
        "skill" | "skills" => "skills",
        _ => return Err(format!("Unsupported resource type: {}", resource_type).into()),
    };

    let url = format!("{}/v1/{}/{}", base_url, endpoint, id);
    let res = client.delete(&url).send().await?;

    if !res.status().is_success() {
        let err = res.text().await?;
        return Err(format!("Failed to delete resource: {}", err).into());
    }

    println!("{} '{}' deleted successfully.", resource_type, id);
    Ok(())
}
