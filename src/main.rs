use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    pre_hash: String,
    hash: String,
}

#[derive(Debug)]// #[derive(Debug)]是一个Rust的属性宏，
//用于自动为结构体生成Debug trait的实现，使得结构体可以使用{:?}或{:#?}格式化输出。
struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    fn new(index: u64, data: String, pre_hash: String) -> Self {
        let genesis_block = Block::new(index, data, pre_hash);
        Blockchain {
            chain: vec![genesis_block],
        }
    }

    fn add_block(&mut self, data: String) {
        let last_block = self.chain.last().unwrap();
        let new_block = Block::new(
            last_block.index + 1,
            data,
            last_block.hash.clone(),
        );
        self.chain.push(new_block);
    }
}


impl Block {
    fn new(index: u64, data: String, pre_hash: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mock_hash = format!("{}{}{}{}", index, data, pre_hash, timestamp);
        let mut hasher = Sha256::new();
        hasher.update(mock_hash.as_bytes());
        let hash_result = hasher.finalize();
        let hash: String = hash_result
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();
        Block {
            index,
            timestamp,
            data,
            pre_hash,
            hash,
        } 
    }
}

fn main() {
    let mut blockchain = Blockchain::new(0, "Genesis Block".to_string(), "0".to_string());
    blockchain.add_block("Second_Block".to_string());
    blockchain.add_block("Third_Block".to_string());
    println!("{:#?}", blockchain);

}