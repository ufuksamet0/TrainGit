//! Configuration structures and loading

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::errors::{Result, ConfigError};

/// Main configuration for TrainGit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub general: GeneralConfig,
    pub github: GitHubConfig,
    pub gitlab: GitLabConfig,
    pub bitbucket: BitbucketConfig,
    pub gitea: GiteaConfig,
    pub collection: CollectionConfig,
    pub processing: ProcessingConfig,
    pub export: ExportConfig,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            github: GitHubConfig::default(),
            gitlab: GitLabConfig::default(),
            bitbucket: BitbucketConfig::default(),
            gitea: GiteaConfig::default(),
            collection: CollectionConfig::default(),
            processing: ProcessingConfig::default(),
            export: ExportConfig::default(),
        }
    }
}

impl TrainingConfig {
    /// Load configuration from a TOML file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(ConfigError::Io)?;
        
        let config: TrainingConfig = toml::from_str(&content)
            .map_err(ConfigError::Toml)?;
        
        Ok(config)
    }
    
    /// Save configuration to a TOML file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(ConfigError::TomlSer)?;
        
        std::fs::write(path, content)
            .map_err(ConfigError::Io)?;
        
        Ok(())
    }
    
    /// Generate a default configuration template
    pub fn template() -> String {
        toml::to_string_pretty(&Self::default())
            .unwrap_or_default()
    }
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub workspace: PathBuf,
    pub max_concurrent: usize,
    pub checkpoint_interval: usize,
    pub log_level: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            workspace: PathBuf::from("./traingit-workspace"),
            max_concurrent: 8,
            checkpoint_interval: 50,
            log_level: "info".to_string(),
        }
    }
}

/// GitHub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub api_url: String,
    pub rate_limit_buffer: u64,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            api_url: "https://api.github.com".to_string(),
            rate_limit_buffer: 100,
        }
    }
}

/// GitLab instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabInstance {
    pub url: String,
    pub token: Option<String>,
}

/// GitLab configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    pub token: Option<String>,
    pub api_url: String,
    pub instances: Vec<GitLabInstance>,
}

impl Default for GitLabConfig {
    fn default() -> Self {
        Self {
            token: None,
            api_url: "https://gitlab.com/api/v4".to_string(),
            instances: vec![
                GitLabInstance {
                    url: "https://gitlab.com".to_string(),
                    token: None,
                },
            ],
        }
    }
}

/// Bitbucket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketConfig {
    pub username: Option<String>,
    pub app_password: Option<String>,
    pub api_url: String,
}

impl Default for BitbucketConfig {
    fn default() -> Self {
        Self {
            username: None,
            app_password: None,
            api_url: "https://api.bitbucket.org/2.0".to_string(),
        }
    }
}

/// Gitea instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiteaInstance {
    pub url: String,
    pub token: Option<String>,
}

/// Gitea configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiteaConfig {
    pub instances: Vec<GiteaInstance>,
}

impl Default for GiteaConfig {
    fn default() -> Self {
        Self {
            instances: vec![
                GiteaInstance {
                    url: "https://gitea.com".to_string(),
                    token: None,
                },
                GiteaInstance {
                    url: "https://codeberg.org".to_string(),
                    token: None,
                },
            ],
        }
    }
}

/// Collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub shallow_clone: bool,
    pub clone_depth: usize,
    pub max_repo_size_mb: u64,
    pub include_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size_kb: u64,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            shallow_clone: true,
            clone_depth: 1,
            max_repo_size_mb: 500,
            include_extensions: vec![
                "rs".to_string(), "py".to_string(), "js".to_string(), "ts".to_string(), "jsx".to_string(), "tsx".to_string(), 
                "go".to_string(), "java".to_string(), "kt".to_string(), "cpp".to_string(), "cc".to_string(), "c".to_string(), "h".to_string(), "hpp".to_string(),
                "cs".to_string(), "rb".to_string(), "php".to_string(), "swift".to_string(), "scala".to_string(), "clj".to_string(), "hs".to_string(), "ml".to_string(), "fs".to_string(), "ex".to_string(), "exs".to_string(),
                "md".to_string(), "mdx".to_string(), "rst".to_string(), "txt".to_string(), "adoc".to_string(), "org".to_string(),
                "json".to_string(), "yaml".to_string(), "yml".to_string(), "toml".to_string(), "xml".to_string(), "proto".to_string(),
                "sql".to_string(), "sh".to_string(), "bash".to_string(), "zsh".to_string(), "fish".to_string(), "ps1".to_string(),
                "dockerfile".to_string(), "makefile".to_string(), "cmake".to_string(), "gradle".to_string(), "sbt".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/*.min.js".to_string(),
                "**/*.lock".to_string(),
                "**/vendor/**".to_string(),
            ],
            max_file_size_kb: 1000,
        }
    }
}

/// Processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub min_tokens: u32,
    pub max_tokens: u32,
    pub min_quality_score: f32,
    pub exact_dedup: bool,
    pub near_dedup: bool,
    pub near_dedup_threshold: f32,
    pub chunk_strategy: ChunkStrategy,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            min_tokens: 50,
            max_tokens: 8192,
            min_quality_score: 0.3,
            exact_dedup: true,
            near_dedup: true,
            near_dedup_threshold: 0.85,
            chunk_strategy: ChunkStrategy::Semantic,
            chunk_size: 2048,
            chunk_overlap: 256,
        }
    }
}

/// Chunking strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkStrategy {
    Semantic,
    Fixed,
    Sliding,
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub hf_dataset_name: String,
    pub hf_private: bool,
    pub push_to_hub: bool,
    pub train_ratio: f32,
    pub val_ratio: f32,
    pub test_ratio: f32,
    pub raw_formats: Vec<String>,
    pub encrypt_output: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            hf_dataset_name: "user/traingit-dataset".to_string(),
            hf_private: true,
            push_to_hub: false,
            train_ratio: 0.8,
            val_ratio: 0.1,
            test_ratio: 0.1,
            raw_formats: vec!["jsonl".to_string(), "parquet".to_string()],
            encrypt_output: false,
        }
    }
}

/// Filter criteria for repository filtering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterCriteria {
    pub min_stars: Option<u64>,
    pub max_stars: Option<u64>,
    pub languages: Vec<String>,
    pub excluded_languages: Vec<String>,
    pub required_topics: Vec<String>,
    pub excluded_topics: Vec<String>,
    pub licenses: Vec<String>,
    pub excluded_licenses: Vec<String>,
    pub min_size_kb: Option<u64>,
    pub max_size_kb: Option<u64>,
    pub exclude_forks: bool,
    pub exclude_archived: bool,
    pub min_activity_days: Option<u64>,
}