use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Module manifest (module.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub module: ModuleMetadata,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub hooks: HashMap<String, String>,
    #[serde(default)]
    pub templates: Vec<TemplateEntry>,
    #[serde(default)]
    pub config: HashMap<String, toml::Value>,
}

/// Module metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// Template entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEntry {
    pub name: String,
    pub description: Option<String>,
    pub files: Vec<String>,
}

/// Module type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    Hook,
    Template,
    Config,
    Mixed,
}

impl ModuleManifest {
    /// Load manifest from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| {
            Error::InvalidObjectType(format!("Failed to parse module manifest: {}", e))
        })
    }

    /// Save manifest to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            Error::InvalidObjectType(format!("Failed to serialize module manifest: {}", e))
        })?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Validate manifest
    pub fn validate(&self) -> Result<()> {
        // Check name
        if self.module.name.is_empty() {
            return Err(Error::InvalidObjectType(
                "Module name cannot be empty".to_string(),
            ));
        }

        // Check version format (semantic versioning)
        if !self.is_valid_version(&self.module.version) {
            return Err(Error::InvalidObjectType(format!(
                "Invalid version format: {}",
                self.module.version
            )));
        }

        // Check hook references
        for hook_path in self.hooks.values() {
            // Hooks should be executable or scripts
            if !hook_path.ends_with(".sh")
                && !hook_path.ends_with(".ps1")
                && !hook_path.ends_with(".exe")
            {
                return Err(Error::InvalidObjectType(format!(
                    "Hook must be executable or script: {}",
                    hook_path
                )));
            }
        }

        Ok(())
    }

    /// Check if version string is valid (basic semantic versioning)
    fn is_valid_version(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        parts.iter().all(|p| p.parse::<u32>().is_ok())
    }

    /// Get module type based on contents
    pub fn module_type(&self) -> ModuleType {
        let has_hooks = !self.hooks.is_empty();
        let has_templates = !self.templates.is_empty();
        let has_config = !self.config.is_empty();

        let count = [has_hooks, has_templates, has_config]
            .iter()
            .filter(|&&x| x)
            .count();

        match count {
            0 => ModuleType::Config, // Default
            1 => {
                if has_hooks {
                    ModuleType::Hook
                } else if has_templates {
                    ModuleType::Template
                } else {
                    ModuleType::Config
                }
            }
            _ => ModuleType::Mixed,
        }
    }

    /// Get full module identifier (name@version)
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.module.name, self.module.version)
    }
}

/// Module manager
pub struct ModuleManager {
    /// Global modules directory (~/.gvc/modules)
    global_dir: PathBuf,
    /// Repository modules directory (.gvc/modules)
    repo_dir: PathBuf,
}

impl ModuleManager {
    /// Create new module manager
    pub fn new(gvc_dir: &Path) -> Result<Self> {
        // Global directory in user home
        let home_dir = dirs::home_dir().ok_or_else(|| {
            Error::InvalidObjectType("Cannot determine home directory".to_string())
        })?;

        let global_dir = home_dir.join(".gvc").join("modules");
        let repo_dir = gvc_dir.join("modules");

        // Create directories if they don't exist
        fs::create_dir_all(&global_dir)?;
        fs::create_dir_all(repo_dir.join("installed"))?;
        fs::create_dir_all(repo_dir.join("active"))?;

        Ok(ModuleManager {
            global_dir,
            repo_dir,
        })
    }

    /// Install module from path (local)
    pub fn install_local(&self, source_path: &Path) -> Result<String> {
        // Load manifest
        let manifest_path = source_path.join("module.toml");
        let manifest = ModuleManifest::load(&manifest_path)?;
        manifest.validate()?;

        let identifier = manifest.identifier();
        let target_dir = self.global_dir.join(&identifier);

        // Check if already installed
        if target_dir.exists() {
            return Err(Error::InvalidObjectType(format!(
                "Module {} is already installed",
                identifier
            )));
        }

        // Copy module to global directory
        self.copy_directory(source_path, &target_dir)?;

        Ok(identifier)
    }

    /// Activate module in repository
    pub fn activate(&self, identifier: &str) -> Result<()> {
        let source = self.global_dir.join(identifier);

        if !source.exists() {
            return Err(Error::InvalidObjectType(format!(
                "Module {} is not installed",
                identifier
            )));
        }

        let target = self.repo_dir.join("active").join(identifier);

        if target.exists() {
            return Err(Error::InvalidObjectType(format!(
                "Module {} is already active",
                identifier
            )));
        }

        // Create symlink (or copy on Windows if symlink fails)
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&source, &target)?;
        }

        #[cfg(windows)]
        {
            // Windows: try symlink, fallback to copy
            if std::os::windows::fs::symlink_dir(&source, &target).is_err() {
                self.copy_directory(&source, &target)?;
            }
        }

        Ok(())
    }

    /// Deactivate module in repository
    pub fn deactivate(&self, identifier: &str) -> Result<()> {
        let target = self.repo_dir.join("active").join(identifier);

        if !target.exists() {
            return Err(Error::InvalidObjectType(format!(
                "Module {} is not active",
                identifier
            )));
        }

        // Remove symlink or directory
        if target.is_symlink() {
            fs::remove_file(&target)?;
        } else {
            fs::remove_dir_all(&target)?;
        }

        Ok(())
    }

    /// List installed modules
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let mut modules = Vec::new();

        if !self.global_dir.exists() {
            return Ok(modules);
        }

        for entry in fs::read_dir(&self.global_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    modules.push(name.to_string());
                }
            }
        }

        modules.sort();
        Ok(modules)
    }

    /// List active modules in repository
    pub fn list_active(&self) -> Result<Vec<String>> {
        let mut modules = Vec::new();
        let active_dir = self.repo_dir.join("active");

        if !active_dir.exists() {
            return Ok(modules);
        }

        for entry in fs::read_dir(&active_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() || entry.path().is_symlink() {
                if let Some(name) = entry.file_name().to_str() {
                    modules.push(name.to_string());
                }
            }
        }

        modules.sort();
        Ok(modules)
    }

    /// Get module manifest
    pub fn get_manifest(&self, identifier: &str, active_only: bool) -> Result<ModuleManifest> {
        let base_dir = if active_only {
            self.repo_dir.join("active")
        } else {
            self.global_dir.clone()
        };

        let manifest_path = base_dir.join(identifier).join("module.toml");
        ModuleManifest::load(&manifest_path)
    }

    /// Copy directory recursively
    fn copy_directory(&self, source: &Path, target: &Path) -> Result<()> {
        fs::create_dir_all(target)?;

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());

            if file_type.is_dir() {
                self.copy_directory(&source_path, &target_path)?;
            } else {
                fs::copy(&source_path, &target_path)?;
            }
        }

        Ok(())
    }

    /// Create a new module scaffold
    pub fn create_scaffold(target_dir: &Path, name: &str, authors: Vec<String>) -> Result<()> {
        fs::create_dir_all(target_dir)?;

        // Create manifest
        let manifest = ModuleManifest {
            module: ModuleMetadata {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: Some(format!("GVC module: {}", name)),
                authors,
                license: Some("MIT".to_string()),
                homepage: None,
                repository: None,
                keywords: vec![],
            },
            dependencies: HashMap::new(),
            hooks: HashMap::new(),
            templates: vec![],
            config: HashMap::new(),
        };

        manifest.save(&target_dir.join("module.toml"))?;

        // Create directories
        fs::create_dir_all(target_dir.join("hooks"))?;
        fs::create_dir_all(target_dir.join("templates"))?;

        // Create example hook
        let example_hook = r#"#!/bin/bash
# Example pre-commit hook
echo "Running pre-commit hook from module: $MODULE_NAME"

# Add your validation here
# exit 1  # Uncomment to prevent commit
"#;
        fs::write(target_dir.join("hooks").join("pre-commit.sh"), example_hook)?;

        // Create README
        let readme = format!(
            r#"# {} Module

## Description

[Describe your module here]

## Installation

```bash
gvc module install .
```

## Usage

```bash
gvc module add {}@0.1.0
```

## Hooks

- `pre-commit` - Runs before each commit

## Templates

[List templates here]

## Configuration

[Describe configuration options here]
"#,
            name, name
        );
        fs::write(target_dir.join("README.md"), readme)?;

        Ok(())
    }
}

// Add dirs dependency for home_dir
// This will need to be added to Cargo.toml

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        let manifest = ModuleManifest {
            module: ModuleMetadata {
                name: "test-module".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: vec!["Test Author".to_string()],
                license: None,
                homepage: None,
                repository: None,
                keywords: vec![],
            },
            dependencies: HashMap::new(),
            hooks: HashMap::new(),
            templates: vec![],
            config: HashMap::new(),
        };

        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_invalid_version() {
        let manifest = ModuleManifest {
            module: ModuleMetadata {
                name: "test".to_string(),
                version: "invalid".to_string(),
                description: None,
                authors: vec![],
                license: None,
                homepage: None,
                repository: None,
                keywords: vec![],
            },
            dependencies: HashMap::new(),
            hooks: HashMap::new(),
            templates: vec![],
            config: HashMap::new(),
        };

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_module_identifier() {
        let manifest = ModuleManifest {
            module: ModuleMetadata {
                name: "my-module".to_string(),
                version: "2.1.3".to_string(),
                description: None,
                authors: vec![],
                license: None,
                homepage: None,
                repository: None,
                keywords: vec![],
            },
            dependencies: HashMap::new(),
            hooks: HashMap::new(),
            templates: vec![],
            config: HashMap::new(),
        };

        assert_eq!(manifest.identifier(), "my-module@2.1.3");
    }
}

