use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub sharding: ShardingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub peers: Vec<String>,
    pub timeout_ms: u64,
    pub retry_interval_ms: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub engine: String,
    pub path: String,
    pub cache_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardingConfig {
    pub num_shards: u32,
    pub replication_factor: u32,
    pub auto_rebalance: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        let mut file = File::open(path)
            .map_err(|e| anyhow!("Failed to open config file {}: {}", path.display(), e))?;
            
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| anyhow!("Failed to read config file {}: {}", path.display(), e))?;
            
        let config: Config = serde_json::from_str(&contents)
            .map_err(|e| anyhow!("Failed to parse config file {}: {}", path.display(), e))?;
            
        Ok(config)
    }
    
    pub fn default() -> Self {
        Self {
            node: NodeConfig {
                id: format!("node-{}", uuid::Uuid::new_v4()),
                host: "127.0.0.1".to_string(),
                port: 9000,
                data_dir: "./data".to_string(),
            },
            network: NetworkConfig {
                peers: vec![],
                timeout_ms: 5000,
                retry_interval_ms: 1000,
                max_retries: 3,
            },
            storage: StorageConfig {
                engine: "memory".to_string(),
                path: "./data/vectors".to_string(),
                cache_size_mb: 1024,
            },
            sharding: ShardingConfig {
                num_shards: 16,
                replication_factor: 3,
                auto_rebalance: true,
            },
        }
    }
}