use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CoreError, CoreResult};

use super::values::Config;

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub global_config: Option<PathBuf>,
    pub repo_config: Option<PathBuf>,
    pub global_ignore: PathBuf,
    pub repo_ignore: Option<PathBuf>,
}

/// Resolve the base configuration directory.
///
/// # Errors
/// Returns an error when the home directory cannot be resolved.
pub fn config_dir() -> CoreResult<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home).join(".config").join("goodcommit"));
    }

    if let Ok(userprofile) = env::var("USERPROFILE") {
        return Ok(PathBuf::from(userprofile)
            .join(".config")
            .join("goodcommit"));
    }

    Err(CoreError::Config(
        "unable to resolve config directory".to_string(),
    ))
}

/// Resolve config and ignore file locations.
///
/// # Errors
/// Returns an error when the config directory cannot be resolved.
pub fn resolve_paths(repo_root: Option<&Path>) -> CoreResult<ConfigPaths> {
    let config_dir = config_dir()?;

    let global_config =
        find_config_file(&config_dir, &["config.toml", "config.yaml", "config.yml"]);

    let repo_config = repo_root.and_then(|root| {
        find_config_file(
            root,
            &[".goodcommit.toml", ".goodcommit.yaml", ".goodcommit.yml"],
        )
    });

    let global_ignore = config_dir.join("ignore");
    let repo_ignore = repo_root.and_then(|root| {
        let path = root.join(".goodcommit-ignore");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    });

    Ok(ConfigPaths {
        global_config,
        repo_config,
        global_ignore,
        repo_ignore,
    })
}

/// Load config files from the resolved paths.
///
/// # Errors
/// Returns an error when any config file cannot be read or parsed.
pub fn load_config(paths: &ConfigPaths) -> CoreResult<Config> {
    let mut config = Config::default();

    if let Some(path) = &paths.global_config {
        config = config.merge(read_config_file(path)?);
    }

    if let Some(path) = &paths.repo_config {
        config = config.merge(read_config_file(path)?);
    }

    Ok(config)
}

/// Read and parse a single config file.
///
/// # Errors
/// Returns an error when the file cannot be read or parsed.
pub fn read_config_file(path: &Path) -> CoreResult<Config> {
    let content = fs::read_to_string(path).map_err(|err| {
        CoreError::Config(format!("failed reading config {}: {err}", path.display()))
    })?;

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => toml::from_str(&content)
            .map_err(|err| CoreError::Config(format!("failed parsing toml config: {err}"))),
        Some("yaml" | "yml") => serde_yaml::from_str(&content)
            .map_err(|err| CoreError::Config(format!("failed parsing yaml config: {err}"))),
        _ => toml::from_str(&content)
            .map_err(|err| CoreError::Config(format!("failed parsing config: {err}"))),
    }
}

fn find_config_file(base: &Path, candidates: &[&str]) -> Option<PathBuf> {
    for name in candidates {
        let path = base.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}
