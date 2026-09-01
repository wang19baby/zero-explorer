#![allow(dead_code)]

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    Disabled,
    Enabled,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

impl PluginMetadata {
    pub fn new(name: &str, version: &str, author: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            description: description.to_string(),
            dependencies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginEvent {
    Activate,
    Deactivate,
    ConfigChanged,
}

pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    fn status(&self) -> &PluginStatus;
    fn activate(&mut self) -> Result<(), String>;
    fn deactivate(&mut self) -> Result<(), String>;
    fn on_event(&mut self, event: &PluginEvent);
    fn config(&self) -> Option<&serde_json::Value> {
        None
    }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub metadata: PluginMetadata,
    pub status: PluginStatus,
    pub path: PathBuf,
    pub config: Option<serde_json::Value>,
}

impl PluginInfo {
    pub fn new(metadata: PluginMetadata, path: PathBuf) -> Self {
        Self {
            metadata,
            status: PluginStatus::Disabled,
            path,
            config: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginManager {
    plugins: Vec<PluginInfo>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: Vec::new(),
            plugin_dir,
        }
    }

    pub fn plugins(&self) -> &[PluginInfo] {
        &self.plugins
    }

    pub fn plugins_mut(&mut self) -> &mut Vec<PluginInfo> {
        &mut self.plugins
    }

    pub fn plugin_dir(&self) -> &PathBuf {
        &self.plugin_dir
    }

    pub fn load_plugins(&mut self) -> Result<Vec<String>, String> {
        let loaded = Vec::new();
        
        // In real implementation, this would scan the plugin directory
        // and load each plugin
        
        Ok(loaded)
    }

    pub fn install_plugin(&mut self, metadata: PluginMetadata, path: PathBuf) -> usize {
        let info = PluginInfo::new(metadata, path);
        self.plugins.push(info);
        self.plugins.len() - 1
    }

    pub fn uninstall_plugin(&mut self, index: usize) -> bool {
        if index < self.plugins.len() {
            self.plugins.remove(index);
            true
        } else {
            false
        }
    }

    pub fn enable_plugin(&mut self, index: usize) -> Result<(), String> {
        if let Some(plugin) = self.plugins.get_mut(index) {
            plugin.status = PluginStatus::Enabled;
            Ok(())
        } else {
            Err("Plugin not found".to_string())
        }
    }

    pub fn disable_plugin(&mut self, index: usize) -> Result<(), String> {
        if let Some(plugin) = self.plugins.get_mut(index) {
            plugin.status = PluginStatus::Disabled;
            Ok(())
        } else {
            Err("Plugin not found".to_string())
        }
    }

    pub fn get_plugin(&self, index: usize) -> Option<&PluginInfo> {
        self.plugins.get(index)
    }

    pub fn get_plugin_mut(&mut self, index: usize) -> Option<&mut PluginInfo> {
        self.plugins.get_mut(index)
    }

    pub fn enabled_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins
            .iter()
            .filter(|p| p.status == PluginStatus::Enabled)
            .collect()
    }

    pub fn find_plugin_by_name(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins.iter().find(|p| p.metadata.name == name)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new(PathBuf::from("plugins"))
    }
}

#[derive(Debug, Clone)]
pub struct PluginSandbox {
    permissions: Vec<String>,
    memory_limit: usize,
    cpu_time_limit: f64,
}

impl PluginSandbox {
    pub fn new() -> Self {
        Self {
            permissions: Vec::new(),
            memory_limit: 100 * 1024 * 1024, // 100MB default
            cpu_time_limit: 10.0, // 10 seconds default
        }
    }

    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }

    pub fn add_permission(&mut self, permission: &str) {
        if !self.permissions.contains(&permission.to_string()) {
            self.permissions.push(permission.to_string());
        }
    }

    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.retain(|p| p != permission);
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    pub fn memory_limit(&self) -> usize {
        self.memory_limit
    }

    pub fn set_memory_limit(&mut self, limit: usize) {
        self.memory_limit = limit;
    }

    pub fn cpu_time_limit(&self) -> f64 {
        self.cpu_time_limit
    }

    pub fn set_cpu_time_limit(&mut self, limit: f64) {
        self.cpu_time_limit = limit;
    }
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata_new() {
        let meta = PluginMetadata::new("test", "1.0.0", "author", "description");
        assert_eq!(meta.name, "test");
        assert_eq!(meta.version, "1.0.0");
    }

    #[test]
    fn test_plugin_manager_new() {
        let manager = PluginManager::new(PathBuf::from("/plugins"));
        assert!(manager.plugins().is_empty());
    }

    #[test]
    fn test_plugin_manager_install() {
        let mut manager = PluginManager::new(PathBuf::from("/plugins"));
        let meta = PluginMetadata::new("test", "1.0.0", "author", "description");
        let index = manager.install_plugin(meta, PathBuf::from("/plugins/test"));
        
        assert_eq!(index, 0);
        assert_eq!(manager.plugins().len(), 1);
    }

    #[test]
    fn test_plugin_manager_enable_disable() {
        let mut manager = PluginManager::new(PathBuf::from("/plugins"));
        let meta = PluginMetadata::new("test", "1.0.0", "author", "description");
        manager.install_plugin(meta, PathBuf::from("/plugins/test"));
        
        manager.enable_plugin(0).unwrap();
        assert_eq!(manager.plugins()[0].status, PluginStatus::Enabled);
        
        manager.disable_plugin(0).unwrap();
        assert_eq!(manager.plugins()[0].status, PluginStatus::Disabled);
    }

    #[test]
    fn test_plugin_sandbox_permissions() {
        let mut sandbox = PluginSandbox::new();
        
        sandbox.add_permission("read");
        sandbox.add_permission("write");
        
        assert!(sandbox.has_permission("read"));
        assert!(sandbox.has_permission("write"));
        assert!(!sandbox.has_permission("execute"));
        
        sandbox.remove_permission("write");
        assert!(!sandbox.has_permission("write"));
    }

    #[test]
    fn test_plugin_sandbox_limits() {
        let mut sandbox = PluginSandbox::new();
        
        assert_eq!(sandbox.memory_limit(), 100 * 1024 * 1024);
        assert_eq!(sandbox.cpu_time_limit(), 10.0);
        
        sandbox.set_memory_limit(200 * 1024 * 1024);
        sandbox.set_cpu_time_limit(20.0);
        
        assert_eq!(sandbox.memory_limit(), 200 * 1024 * 1024);
        assert_eq!(sandbox.cpu_time_limit(), 20.0);
    }
}
