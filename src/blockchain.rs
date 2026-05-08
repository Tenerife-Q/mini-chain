use serde::{Deserialize, Serialize};
use std::fs;
// 同时它管辖着区块，所以也得向 block 模块借用
use crate::block::Block;

#[derive(Debug, Deserialize, Serialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new(index: u64, data: String, pre_hash: String) -> Self {
        let genesis_block = Block::new(index, data, pre_hash);
        Blockchain {
            chain: vec![genesis_block],// 直接把创世区块放进链条里
        }
    }

    pub fn add_block(&mut self, data: String) {
        let last_block = self.chain.last().unwrap();
        let new_block = Block::new(
            last_block.index + 1,
            data,
            last_block.hash.clone(),
        );
        self.chain.push(new_block);
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];
            if current_block.pre_hash != previous_block.hash {
                return false;
            }
            if current_block.hash != current_block.calculate_hash() {
                return false;
            }
        }
        true
    }

    pub fn save_to_disk(&self, file_path: &str) {
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        fs::write(file_path, serialized).expect("Unable to write file");
    }

    pub fn load_from_disk(file_path: &str) -> Self {
        let data = fs::read_to_string(file_path).expect("Unable to read file");
        serde_json::from_str(&data).expect("Unable to parse JSON")
    }
}