use ethers::types::U256;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

use super::types::UniV2Pool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub pools: Vec<UniV2Pool>,
    pub block: U256,
}

impl Storage {
    pub fn new(pools: Vec<UniV2Pool>, block: U256) -> Self {
        Self { pools, block }
    }

    pub fn save_to_file(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let serialized = serde_json::to_string_pretty(self)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file(file_path: &str) -> std::io::Result<Storage> {
        let file = File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let storage: Storage = serde_json::from_reader(reader)?;
        Ok(storage)
    }
}
