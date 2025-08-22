use crate::models::plugin::{PluginMetadata, PluginRegistryEntry, PluginSearchFilters, PluginSortBy};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// Plugin discovery service for finding and loading plugins
#[async_trait]
pub trait PluginDiscovery: Send + Sync {
    /// Discover plugins in a directory
    async fn discover_plugins(&self, directory: &Path) -> Result<Vec<PluginMetadata>>;
    
    /// Search for plugins in the marketplace
    async fn search_marketplace(&self, filters: PluginSearchFilters) -> Result<Vec<PluginRegistryEntry>>;
    
    /// Download a plugin from the marketplace
    async fn download_plugin(&self, plugin_id: &str, version: &str, destination: &Path) -> Result<PathBuf>;
    
    /// Validate a plugin package
    async fn validate_plugin(&self, plugin_path: &Path) -> Result<PluginMetadata>;
    
    /// Get plugin dependencies
    async fn get_dependencies(&self, plugin_metadata: &PluginMetadata) -> Result<Vec<PluginMetadata>>;
    
    /// Get all configured plugin directories
    fn get_plugin_directories(&self) -> Vec<PathBuf>;
}

/// Default implementation of plugin discovery
pub struct DefaultPluginDiscovery {
    plugin_directories: Vec<PathBuf>,
    marketplace_url: Option<String>,
}

impl DefaultPluginDiscovery {
    /// Create a new plugin discovery service
    pub fn new(plugin_directories: Vec<PathBuf>, marketplace_url: Option<String>) -> Self {
        Self {
            plugin_directories,
            marketplace_url,
        }
    }
    
    /// Get plugin directories
    pub fn plugin_directories(&self) -> &[PathBuf] {
        &self.plugin_directories
    }
    
    /// Add a plugin directory to search
    pub fn add_plugin_directory(&mut self, directory: PathBuf) {
        self.plugin_directories.push(directory);
    }
    
    /// Set the marketplace URL
    pub fn set_marketplace_url(&mut self, url: String) {
        self.marketplace_url = Some(url);
    }
    
    /// Parse plugin metadata from a manifest file
    async fn parse_plugin_manifest(&self, manifest_path: &Path) -> Result<PluginMetadata> {
        let content = fs::read_to_string(manifest_path).await?;
        
        // Try to parse as JSON first, then TOML
        if let Ok(metadata) = serde_json::from_str::<PluginMetadata>(&content) {
            Ok(metadata)
        } else if let Ok(metadata) = toml::from_str::<PluginMetadata>(&content) {
            Ok(metadata)
        } else {
            Err(anyhow!("Failed to parse plugin manifest: {}", manifest_path.display()))
        }
    }
    
    /// Check if a directory contains a valid plugin
    async fn is_plugin_directory(&self, directory: &Path) -> bool {
        let manifest_json = directory.join("plugin.json");
        let manifest_toml = directory.join("plugin.toml");
        let cargo_toml = directory.join("Cargo.toml");
        
        manifest_json.exists() || manifest_toml.exists() || cargo_toml.exists()
    }
}

#[async_trait]
impl PluginDiscovery for DefaultPluginDiscovery {
    async fn discover_plugins(&self, directory: &Path) -> Result<Vec<PluginMetadata>> {
        let mut plugins = Vec::new();
        
        if !directory.exists() {
            return Ok(plugins);
        }
        
        let mut entries = fs::read_dir(directory).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() && self.is_plugin_directory(&path).await {
                // Look for plugin manifest files
                let manifest_paths = vec![
                    path.join("plugin.json"),
                    path.join("plugin.toml"),
                    path.join("Cargo.toml"), // For Rust plugins
                ];
                
                for manifest_path in manifest_paths {
                    if manifest_path.exists() {
                        match self.parse_plugin_manifest(&manifest_path).await {
                            Ok(metadata) => {
                                plugins.push(metadata);
                                break; // Found a valid manifest, move to next directory
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to parse plugin manifest {}: {}", 
                                         manifest_path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(plugins)
    }
    
    async fn search_marketplace(&self, filters: PluginSearchFilters) -> Result<Vec<PluginRegistryEntry>> {
        if let Some(marketplace_url) = &self.marketplace_url {
            // In a real implementation, this would make HTTP requests to the marketplace API
            // For now, return empty results
            println!("Searching marketplace at {} with filters: {:?}", marketplace_url, filters);
            Ok(vec![])
        } else {
            Err(anyhow!("No marketplace URL configured"))
        }
    }
    
    async fn download_plugin(&self, plugin_id: &str, version: &str, destination: &Path) -> Result<PathBuf> {
        if let Some(marketplace_url) = &self.marketplace_url {
            // In a real implementation, this would download the plugin from the marketplace
            println!("Downloading plugin {} version {} from {} to {}", 
                    plugin_id, version, marketplace_url, destination.display());
            
            // Create destination directory if it doesn't exist
            fs::create_dir_all(destination).await?;
            
            // Return the path where the plugin would be downloaded
            Ok(destination.join(format!("{}-{}", plugin_id, version)))
        } else {
            Err(anyhow!("No marketplace URL configured"))
        }
    }
    
    async fn validate_plugin(&self, plugin_path: &Path) -> Result<PluginMetadata> {
        if !plugin_path.exists() {
            return Err(anyhow!("Plugin path does not exist: {}", plugin_path.display()));
        }
        
        // Look for plugin manifest
        let manifest_paths = vec![
            plugin_path.join("plugin.json"),
            plugin_path.join("plugin.toml"),
            plugin_path.join("Cargo.toml"),
        ];
        
        for manifest_path in manifest_paths {
            if manifest_path.exists() {
                return self.parse_plugin_manifest(&manifest_path).await;
            }
        }
        
        Err(anyhow!("No valid plugin manifest found in: {}", plugin_path.display()))
    }
    
    async fn get_dependencies(&self, plugin_metadata: &PluginMetadata) -> Result<Vec<PluginMetadata>> {
        let mut dependencies = Vec::new();
        
        // For each dependency, try to find it in the plugin directories
        for dependency in &plugin_metadata.dependencies {
            if dependency.optional {
                continue; // Skip optional dependencies for now
            }
            
            // Search for the dependency in all plugin directories
            for plugin_dir in &self.plugin_directories {
                if let Ok(discovered_plugins) = self.discover_plugins(plugin_dir).await {
                    for discovered in discovered_plugins {
                        if discovered.name == dependency.name {
                            // TODO: Check version compatibility
                            dependencies.push(discovered);
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }
    
    fn get_plugin_directories(&self) -> Vec<PathBuf> {
        self.plugin_directories.clone()
    }
}

/// Plugin loader for loading plugins from the file system
pub struct PluginLoader {
    discovery: Arc<dyn PluginDiscovery>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(discovery: Arc<dyn PluginDiscovery>) -> Self {
        Self { discovery }
    }
    
    /// Load a plugin from a directory
    pub async fn load_from_directory(&self, plugin_path: &Path) -> Result<PluginMetadata> {
        self.discovery.validate_plugin(plugin_path).await
    }
    
    /// Install a plugin from the marketplace
    pub async fn install_from_marketplace(&self, plugin_id: &str, version: &str, install_dir: &Path) -> Result<PathBuf> {
        // Download the plugin
        let plugin_path = self.discovery.download_plugin(plugin_id, version, install_dir).await?;
        
        // Validate the downloaded plugin
        let _metadata = self.discovery.validate_plugin(&plugin_path).await?;
        
        Ok(plugin_path)
    }
    
    /// Get all available plugins from configured directories
    pub async fn get_available_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let mut all_plugins = Vec::new();
        
        // Discover plugins from all configured directories
        for plugin_dir in self.discovery.get_plugin_directories() {
            if let Ok(plugins) = self.discovery.discover_plugins(&plugin_dir).await {
                all_plugins.extend(plugins);
            }
        }
        
        Ok(all_plugins)
    }
}