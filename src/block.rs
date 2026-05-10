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
        // 【消灭 unwrap 陷阱】：使用 unwrap_or_default() 替代 unwrap()
        // 现实中服务器可能会发生“NTP时间回拨（NTP Sync Backwards）”，导致当前时间早于 UNIX_EPOCH。
        // 如果用 unwrap() 我们的区块节点会直接崩溃闪退！
        // 用 unwrap_or_default() 则是在遇到异常时温和地给个缺省值（0 秒），程序依然坚挺！
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
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
