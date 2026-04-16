use anyhow::{Context, bail};
use bevy::platform::collections::HashMap;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct Registry {
    pub sorted_repositories: HashMap<String, RegistryRepo>,
    pub installed_modules: HashMap<String, Module>,
}

#[derive(Deserialize)]
pub struct Module {
    pub install_time: String,
    pub source_module: CkanModule,
    pub auto_installed: Option<bool>,
}

#[derive(Deserialize)]
pub struct SourceModule {
    pub identifier: String,
    pub name: String,
    #[serde(rename = "abstract")]
    pub short_description: String,
    #[serde(deserialize_with = "one_or_many_string")]
    pub author: Vec<String>,
    pub version: String,
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct RegistryRepo {
    pub name: String,
    pub priority: i32,
    pub uri: String,
}

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

/// Get the default instance path from the cli
pub fn default_instance_path() -> anyhow::Result<PathBuf> {
    let output = run_command(&["instance", "list"])?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());

    let header = lines
        .next()
        .context("ckan instance list produced no output")?;
    let path_col = header
        .find("Path")
        .context("'Path' column not found in header")?;

    // Skip the separator line
    lines
        .next()
        .context("Expected separator line after header")?;

    let first_row = lines.next().context("No game instances found")?;
    let path = first_row
        .get(path_col..)
        .context("'Path' column offset past end of row")?
        .trim();

    Ok(PathBuf::from(path))
}

pub fn get_registry<P: AsRef<Path>>(instance_path: P) -> anyhow::Result<Registry> {
    let registry_path = instance_path.as_ref().join("CKAN").join("registry.json");
    let registry: Registry =
        read_json_file(registry_path).context("Failed to read registry.json")?;
    Ok(registry)
}

pub fn get_repo(registry: &Registry) -> anyhow::Result<Repo> {
    let repos_dir = PathBuf::from(std::env::var("LOCALAPPDATA").context("LOCALAPPDATA not set")?)
        .join("CKAN")
        .join("repos");

    for name in registry.sorted_repositories.keys() {
        for entry in std::fs::read_dir(&repos_dir)?.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(&format!("{name}.json")))
            {
                return read_json_file(&path).with_context(|| format!("Failed to read {:?}", path));
            }
        }
    }

    bail!("No repository found")
}

pub fn run_command(args: &[&str]) -> anyhow::Result<std::process::Output> {
    std::process::Command::new("./ckan.exe")
        .args(args)
        .output()
        .context("Failed to run ckan cli command")
}

fn read_json_file<P: AsRef<Path>, T: serde::de::DeserializeOwned>(p: P) -> anyhow::Result<T> {
    let file = std::fs::File::open(p)?;
    let reader = std::io::BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}
