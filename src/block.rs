use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

pub const DIFFICULTY: usize = 2; // 先设置为2，表示哈希必须以 "00" 开头

#[derive(Debug, Deserialize, Serialize)] // 让区块链结构体支持序列化和反序列化，方便保存和加载区块链数据。
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub data: String,
    pub pre_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, data: String, pre_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let mut block = Block {
            index,
            timestamp,
            data,
            pre_hash,
            hash: String::new(), // 先给个空字符串占位
            nonce: 0,
        };
        
        // 【深入挖掘：从“计算一次”变成“疯狂计算”】
        block.mine();
        block
    }

    // &self: 不可变借用区块自身的字段用于计算。如果不用引用，区块在第一次算完哈希后就会灰飞烟灭。
    pub fn calculate_hash(&self) -> String {
        let mock_hash = format!(
            "{}{}{}{}{}",self.index, self.data, self.pre_hash, self.timestamp, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(mock_hash.as_bytes());
        let hash_result = hasher.finalize();
        hash_result
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect()
    }

    pub fn mine(&mut self) {
        let target = "0".repeat(DIFFICULTY);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block {} mined !!! nonce: {}, Hash: {}", self.index, self.nonce, self.hash);
    }
}
