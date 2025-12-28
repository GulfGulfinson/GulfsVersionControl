// Remote repository management

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::Error;
use crate::protocol::{Request, Response, ObjectData, RefUpdate};
use crate::hash::Oid;

/// Remote repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub name: String,
    pub url: String,
    pub fetch: String,
}

impl RemoteConfig {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            url: url.into(),
            fetch: format!("+refs/heads/*:refs/remotes/{}/*", name),
        }
    }
}

/// Manages remotes for a repository
pub struct RemoteManager {
    repo_path: PathBuf,
    remotes: HashMap<String, RemoteConfig>,
}

impl RemoteManager {
    /// Create new remote manager for a repository
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self> {
        let repo_path = repo_path.as_ref().to_path_buf();
        let remotes = Self::load_remotes(&repo_path)?;
        
        Ok(Self {
            repo_path,
            remotes,
        })
    }

    /// Load remotes from config file
    fn load_remotes(repo_path: &Path) -> Result<HashMap<String, RemoteConfig>> {
        let config_path = repo_path.join(".gvc").join("remotes.toml");
        
        if !config_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read remotes config")?;
        
        let remotes: HashMap<String, RemoteConfig> = toml::from_str(&content)
            .context("Failed to parse remotes config")?;
        
        Ok(remotes)
    }

    /// Save remotes to config file
    fn save_remotes(&self) -> Result<()> {
        let config_path = self.repo_path.join(".gvc").join("remotes.toml");
        
        let content = toml::to_string_pretty(&self.remotes)
            .context("Failed to serialize remotes")?;
        
        fs::write(&config_path, content)
            .context("Failed to write remotes config")?;
        
        Ok(())
    }

    /// Add a new remote
    pub fn add(&mut self, name: impl Into<String>, url: impl Into<String>) -> Result<()> {
        let name = name.into();
        
        if self.remotes.contains_key(&name) {
            return Err(Error::RemoteExists(name).into());
        }

        let remote = RemoteConfig::new(name.clone(), url);
        self.remotes.insert(name, remote);
        self.save_remotes()?;
        
        Ok(())
    }

    /// Remove a remote
    pub fn remove(&mut self, name: &str) -> Result<()> {
        if !self.remotes.contains_key(name) {
            return Err(Error::RemoteNotFound(name.to_string()).into());
        }

        self.remotes.remove(name);
        self.save_remotes()?;
        
        Ok(())
    }

    /// Rename a remote
    pub fn rename(&mut self, old_name: &str, new_name: impl Into<String>) -> Result<()> {
        let new_name = new_name.into();
        
        if !self.remotes.contains_key(old_name) {
            return Err(Error::RemoteNotFound(old_name.to_string()).into());
        }

        if self.remotes.contains_key(&new_name) {
            return Err(Error::RemoteExists(new_name).into());
        }

        let mut remote = self.remotes.remove(old_name).unwrap();
        remote.name = new_name.clone();
        remote.fetch = format!("+refs/heads/*:refs/remotes/{}/*", new_name);
        self.remotes.insert(new_name, remote);
        self.save_remotes()?;
        
        Ok(())
    }

    /// Get a remote by name
    pub fn get(&self, name: &str) -> Option<&RemoteConfig> {
        self.remotes.get(name)
    }

    /// List all remotes
    pub fn list(&self) -> Vec<&RemoteConfig> {
        self.remotes.values().collect()
    }

    /// Get URL for a remote
    pub fn get_url(&self, name: &str) -> Option<&str> {
        self.remotes.get(name).map(|r| r.url.as_str())
    }
}

/// HTTP client for remote operations
pub struct RemoteClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl RemoteClient {
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let base_url = url.into();
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            base_url,
            client,
        })
    }

    /// Send a request to the server
    pub fn send_request(&self, request: &Request) -> Result<Response> {
        let url = format!("{}/api/v1/gvc", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(request)
            .send()
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            return Err(Error::RemoteError(
                format!("HTTP {}: {}", status, text)
            ).into());
        }

        let result: Response = response.json()
            .context("Failed to parse response")?;
        
        Ok(result)
    }

    /// List references on the remote
    pub fn list_refs(&self, repository: &str) -> Result<HashMap<String, Oid>> {
        let request = Request::ListRefs {
            repository: repository.to_string(),
        };
        
        let response = self.send_request(&request)?;
        
        match response {
            Response::Refs { refs, .. } => Ok(refs),
            Response::Error { code, message } => {
                Err(Error::RemoteError(format!("{}: {}", code, message)).into())
            }
            _ => Err(Error::RemoteError("Unexpected response".to_string()).into()),
        }
    }

    /// Get objects from the remote
    pub fn get_objects(&self, repository: &str, oids: &[Oid]) -> Result<Vec<ObjectData>> {
        let request = Request::GetObjects {
            repository: repository.to_string(),
            oids: oids.to_vec(),
        };
        
        let response = self.send_request(&request)?;
        
        match response {
            Response::Objects { objects } => Ok(objects),
            Response::Error { code, message } => {
                Err(Error::RemoteError(format!("{}: {}", code, message)).into())
            }
            _ => Err(Error::RemoteError("Unexpected response".to_string()).into()),
        }
    }

    /// Push objects and ref updates to the remote
    pub fn push(
        &self,
        repository: &str,
        objects: Vec<ObjectData>,
        ref_updates: Vec<RefUpdate>,
    ) -> Result<String> {
        let request = Request::Push {
            repository: repository.to_string(),
            objects,
            ref_updates,
        };
        
        let response = self.send_request(&request)?;
        
        match response {
            Response::PushResult { success, message, .. } => {
                if success {
                    Ok(message)
                } else {
                    Err(Error::RemoteError(message).into())
                }
            }
            Response::Error { code, message } => {
                Err(Error::RemoteError(format!("{}: {}", code, message)).into())
            }
            _ => Err(Error::RemoteError("Unexpected response".to_string()).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_remote_manager() -> Result<()> {
        let temp = TempDir::new()?;
        let repo_path = temp.path();
        fs::create_dir_all(repo_path.join(".gvc"))?;

        let mut manager = RemoteManager::new(repo_path)?;
        
        // Add remote
        manager.add("origin", "https://example.com/repo.git")?;
        assert!(manager.get("origin").is_some());
        
        // List remotes
        let remotes = manager.list();
        assert_eq!(remotes.len(), 1);
        
        // Rename remote
        manager.rename("origin", "upstream")?;
        assert!(manager.get("upstream").is_some());
        assert!(manager.get("origin").is_none());
        
        // Remove remote
        manager.remove("upstream")?;
        assert!(manager.get("upstream").is_none());
        
        Ok(())
    }
}

