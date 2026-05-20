use serde::{Deserialize, Serialize};
use std::fs;
// 同时它管辖着区块，所以也得向 block 模块借用
use crate::block::Block;

#[derive(Debug, Deserialize, Serialize, Clone)]
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

    // 【消灭 unwrap】返回值改为 Result 允许向外报告错误。Box<dyn std::error::Error> 动态特质代表“接收任何实现了 Error 特质的错误”。
    pub fn save_to_disk(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 【问号语法糖 ? 的威力】：如果序列化失败，直接代替 panic 返回 Err(当前错误)；如果成功，就扒下 Ok 框，取出里面的字符给 serialized！
        let serialized = serde_json::to_string_pretty(&self)?;
        // 写盘同理，磁盘满了或者没权限不再闪退，而是优雅向上抛出异常。
        fs::write(file_path, serialized)?;
        Ok(()) // 完全无错误，返回表示啥都不占用的单元类型 ()
    }

    // 【消灭 unwrap】与写入同理，从强行返回 Self 改成返回 Result<Self, ...>
    pub fn load_from_disk(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(file_path)?;
        let recovered_blockchain: Self = serde_json::from_str(&data)?;
        Ok(recovered_blockchain)
    }
}