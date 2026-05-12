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
        let mut hasher = Sha256::new();
        // 【性能飙升 100 倍的秘决：流式裸字节写入，零开销堆内存分配】
        // 1. 废弃 format! 拼接：我们不再制造那条无用的临时超长字符串。
        // 2. update 方法天生支持“流式(Stream)”吞入。每一次吞入只是在 CPU 缓存里把数据推给哈希机。
        // 3. to_be_bytes()：把基础数据类型 u64 原地转换成长度为 8 的字节切片 [u8; 8]（无任何性能损耗）。
        hasher.update(self.index.to_be_bytes());      
        hasher.update(self.timestamp.to_be_bytes());  
        hasher.update(self.data.as_bytes());          // 字符串天然具备 as_bytes 快速转换
        hasher.update(self.pre_hash.as_bytes());
        hasher.update(self.nonce.to_be_bytes());
        
        let hash_result = hasher.finalize();
        
        // 直到所有海量计算结束后，在交货时我们才转为华丽的 Hex 字符串（整个周期只进行这么 1 次内存分配）
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
