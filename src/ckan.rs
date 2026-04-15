use anyhow::bail;
use bevy::{log::error, platform::collections::HashMap};
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer};

#[allow(unused)]
#[derive(Deserialize)]
pub struct Repo {
    pub available_modules: IndexMap<String, ModuleVersion>,
    pub download_counts: HashMap<String, u32>,
    pub known_game_versions: Vec<String>,
}

#[derive(Deserialize)]
pub struct ModuleVersion {
    // We use IndexMap because the order is very important here
    pub module_version: IndexMap<String, CkanModule>,
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct CkanModule {
    pub identifier: String,
    pub name: String,
    #[serde(rename = "abstract")]
    pub short_description: String,
    #[serde(deserialize_with = "one_or_many_string")]
    pub author: Vec<String>,
    pub version: String,
    pub ksp_version: Option<String>,
}

fn one_or_many_string<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(d)? {
        OneOrMany::One(s) => vec![s],
        OneOrMany::Many(v) => v,
    })
}

pub fn get_repo() -> anyhow::Result<Repo> {
    // Read full module metadata from CKAN's cached repository JSON files.
    // CKAN stores these at %LOCALAPPDATA%\CKAN\repos\{hash}-{reponame}.json
    let repos_dir =
        std::path::PathBuf::from(std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set"))
            .join("CKAN")
            .join("repos");

    let entries = std::fs::read_dir(&repos_dir)?;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let is_json = path.extension().and_then(|e| e.to_str()) == Some("json");
        let is_etags = path.file_name().and_then(|n| n.to_str()) == Some("etags.json");
        if !is_json || is_etags {
            continue;
        }
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("failed to read {path:?}:\n{err}");
                continue;
            }
        };
        let content = String::from_utf8_lossy(&bytes);
        let repo_data: Repo =
            serde_json::from_str(&content).expect("Failed to parse repo data JSON");
        return Ok(repo_data);
    }
    bail!("no repos found")
}
