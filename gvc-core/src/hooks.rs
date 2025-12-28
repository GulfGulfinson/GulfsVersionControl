use crate::{Error, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Hook types supported by GVC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    PreCommit,
    PostCommit,
    PrePush,
    PostPull,
    PreCheckout,
    PostCheckout,
}

impl HookType {
    pub fn as_str(&self) -> &str {
        match self {
            HookType::PreCommit => "pre-commit",
            HookType::PostCommit => "post-commit",
            HookType::PrePush => "pre-push",
            HookType::PostPull => "post-pull",
            HookType::PreCheckout => "pre-checkout",
            HookType::PostCheckout => "post-checkout",
        }
    }

    pub fn all() -> Vec<HookType> {
        vec![
            HookType::PreCommit,
            HookType::PostCommit,
            HookType::PrePush,
            HookType::PostPull,
            HookType::PreCheckout,
            HookType::PostCheckout,
        ]
    }
}

/// Hook execution result
#[derive(Debug)]
pub struct HookResult {
    pub hook_type: HookType,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl HookResult {
    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// Hook manager
pub struct HookManager {
    hooks_dir: PathBuf,
    modules_dir: PathBuf,
}

impl HookManager {
    /// Create new hook manager
    pub fn new(gvc_dir: &Path) -> Self {
        HookManager {
            hooks_dir: gvc_dir.join("hooks"),
            modules_dir: gvc_dir.join("modules").join("active"),
        }
    }

    /// Initialize hooks directory
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.hooks_dir)?;

        // Create example hooks
        for hook_type in HookType::all() {
            let hook_path = self
                .hooks_dir
                .join(format!("{}.sample", hook_type.as_str()));
            if !hook_path.exists() {
                let sample_content = self.create_sample_hook(hook_type);
                fs::write(&hook_path, sample_content)?;
            }
        }

        Ok(())
    }

    /// Execute a hook
    pub fn execute(
        &self,
        hook_type: HookType,
        env_vars: HashMap<String, String>,
    ) -> Result<Vec<HookResult>> {
        let mut results = Vec::new();

        // Execute repository hooks
        if let Some(result) = self.execute_repo_hook(hook_type, &env_vars)? {
            results.push(result);
        }

        // Execute module hooks
        let module_results = self.execute_module_hooks(hook_type, &env_vars)?;
        results.extend(module_results);

        Ok(results)
    }

    /// Execute repository hook
    fn execute_repo_hook(
        &self,
        hook_type: HookType,
        env_vars: &HashMap<String, String>,
    ) -> Result<Option<HookResult>> {
        let hook_name = hook_type.as_str();

        // Try different extensions
        let extensions = if cfg!(windows) {
            vec!["ps1", "bat", "exe", ""]
        } else {
            vec!["sh", ""]
        };

        for ext in extensions {
            let filename = if ext.is_empty() {
                hook_name.to_string()
            } else {
                format!("{}.{}", hook_name, ext)
            };

            let hook_path = self.hooks_dir.join(&filename);

            if hook_path.exists() {
                return Ok(Some(self.run_hook(&hook_path, hook_type, env_vars)?));
            }
        }

        Ok(None)
    }

    /// Execute hooks from modules
    fn execute_module_hooks(
        &self,
        hook_type: HookType,
        env_vars: &HashMap<String, String>,
    ) -> Result<Vec<HookResult>> {
        let mut results = Vec::new();

        if !self.modules_dir.exists() {
            return Ok(results);
        }

        // Iterate through active modules
        for entry in fs::read_dir(&self.modules_dir)? {
            let entry = entry?;
            let module_path = entry.path();

            if !module_path.is_dir() {
                continue;
            }

            // Load module manifest
            let manifest_path = module_path.join("module.toml");
            if !manifest_path.exists() {
                continue;
            }

            let manifest_content = fs::read_to_string(&manifest_path)?;
            let manifest: toml::Value = toml::from_str(&manifest_content).map_err(|e| {
                Error::InvalidObjectType(format!("Failed to parse module manifest: {}", e))
            })?;

            // Check if module has this hook
            if let Some(hooks) = manifest.get("hooks").and_then(|h| h.as_table()) {
                if let Some(hook_path_str) = hooks.get(hook_type.as_str()).and_then(|v| v.as_str())
                {
                    let hook_path = module_path.join(hook_path_str);

                    if hook_path.exists() {
                        let mut module_env = env_vars.clone();
                        module_env.insert(
                            "MODULE_NAME".to_string(),
                            entry.file_name().to_string_lossy().to_string(),
                        );
                        module_env.insert(
                            "MODULE_PATH".to_string(),
                            module_path.to_string_lossy().to_string(),
                        );

                        results.push(self.run_hook(&hook_path, hook_type, &module_env)?);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Run a hook script
    fn run_hook(
        &self,
        hook_path: &Path,
        hook_type: HookType,
        env_vars: &HashMap<String, String>,
    ) -> Result<HookResult> {
        // Determine how to execute the hook
        let extension = hook_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let mut cmd = match extension {
            "sh" => {
                let mut c = Command::new("sh");
                c.arg(hook_path);
                c
            }
            "bash" => {
                let mut c = Command::new("bash");
                c.arg(hook_path);
                c
            }
            "ps1" => {
                let mut c = Command::new("powershell");
                c.arg("-ExecutionPolicy")
                    .arg("Bypass")
                    .arg("-File")
                    .arg(hook_path);
                c
            }
            "bat" | "cmd" => {
                let mut c = Command::new("cmd");
                c.arg("/C").arg(hook_path);
                c
            }
            "exe" | "" => {
                // Executable
                Command::new(hook_path)
            }
            _ => {
                return Err(Error::InvalidObjectType(format!(
                    "Unsupported hook type: {}",
                    extension
                )));
            }
        };

        // Add environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Execute hook
        let output = cmd.output()?;

        Ok(HookResult {
            hook_type,
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Create sample hook content
    fn create_sample_hook(&self, hook_type: HookType) -> String {
        let hook_name = hook_type.as_str();

        if cfg!(windows) {
            format!(
                r#"# GVC {} Hook (PowerShell)
# Rename this file to {}.ps1 to activate

Write-Host "Running {} hook"

# Add your logic here
# Exit 1  # Uncomment to prevent the operation
"#,
                hook_name, hook_name, hook_name
            )
        } else {
            format!(
                r#"#!/bin/sh
# GVC {} Hook
# Rename this file to {}.sh and make it executable to activate

echo "Running {} hook"

# Add your logic here
# exit 1  # Uncomment to prevent the operation
"#,
                hook_name, hook_name, hook_name
            )
        }
    }

    /// Check if any hooks are active for a given type
    pub fn has_active_hooks(&self, hook_type: HookType) -> bool {
        // Check repo hooks
        let hook_name = hook_type.as_str();
        let extensions = if cfg!(windows) {
            vec!["ps1", "bat", "exe", ""]
        } else {
            vec!["sh", ""]
        };

        for ext in extensions {
            let filename = if ext.is_empty() {
                hook_name.to_string()
            } else {
                format!("{}.{}", hook_name, ext)
            };

            let hook_path = self.hooks_dir.join(&filename);
            if hook_path.exists() {
                return true;
            }
        }

        // Check module hooks (simplified check)
        if self.modules_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.modules_dir) {
                for entry in entries.flatten() {
                    let manifest_path = entry.path().join("module.toml");
                    if manifest_path.exists() {
                        return true; // Assume module might have hooks
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hook_manager_init() {
        let temp = TempDir::new().unwrap();
        let manager = HookManager::new(temp.path());
        manager.init().unwrap();

        assert!(temp.path().join("hooks").exists());
        assert!(temp.path().join("hooks").join("pre-commit.sample").exists());
    }

    #[test]
    fn test_hook_type_str() {
        assert_eq!(HookType::PreCommit.as_str(), "pre-commit");
        assert_eq!(HookType::PostCommit.as_str(), "post-commit");
    }
}
