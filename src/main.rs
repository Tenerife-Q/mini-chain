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

    // &mut self: 可变借用。我们需要修改（push）链条数组，但不能剥夺原区块链的所有权。
    fn add_block(&mut self, data: String) {
        let last_block = self.chain.last().unwrap();
        let new_block = Block::new(
            last_block.index + 1,
            data, // 所有权 Move (转移)：传入的 data 在这里无缝移交给了新区块，外部和此函数以后都不能再使用它。
            last_block.hash.clone(), // Clone (深拷贝)：不能抢走老区块的 hash 所有权，否则老区块就残损了，必须复印一份给新区块。
        );
        self.chain.push(new_block); // 所有权 Move (转移)：new_block 变量在此终结，它的所有权被塞进数组，由 Blockchain 结构体永久保管。
    }

    // &self: 不可变借用。防伪检查是“只读”操作，函数结束时归还所有权。如果写成 self，验证完区块链就会在内存中被销毁报错！
    fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];
            // 错误点在这里：原来写的是 current_block.hash != current_block.pre_hash
            // 应该是：当前块的 pre_hash 必须等于 前一个块 的 hash！
            if current_block.pre_hash != previous_block.hash {
                return false;
            }
            // 校验自身数据是否被篡改
            if current_block.hash != current_block.calculate_hash() {
                return false;
            }
        }
        true
    }
}


impl Block {
    fn new(index: u64, data: String, pre_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        /* 
        let mock_hash = format!("{}{}{}{}", index, data, pre_hash, timestamp);
        let mut hasher = Sha256::new();
        hasher.update(mock_hash.as_bytes());
        let hash_result = hasher.finalize();
        let hash: String = hash_result
            .iter()
            .map(|byte| format!("{:02x}", byte))
            // format!("{:02x}", byte) 将每个字节格式化为两位十六进制字符串，
            // 确保单个字节（0-255）总是占用两位（例如，0x0a会被格式化为"0a"而不是"a"
            // x表示十六进制，02表示至少两位，不足两位时在前面补0。
            // 大括号里，冒号左边是变量的位置（留空就是按先后顺序接纳变量）
            .collect();
        */
        let mut block = Block {
            index,
            timestamp,
            data,
            pre_hash,
            hash: String::new(), // 先占位，后面计算hash
        };
        block.hash = block.calculate_hash();
        block

    }

    // &self: 不可变借用区块自身的字段用于计算。如果不用引用，区块在第一次算完哈希后就会灰飞烟灭。
    fn calculate_hash(&self) -> String {
        let mock_hash = format!(
            "{}{}{}{}",self.index, self.data, self.pre_hash, self.timestamp
        );
        let mut hasher = Sha256::new();
        hasher.update(mock_hash.as_bytes());
        let hash_result = hasher.finalize();
        hash_result.
            iter()
            .map(|byte| format!("{:02x}", byte))
            .collect()
    }
}

fn main() {
    let mut blockchain = Blockchain::new(0, "Genesis Block".to_string(), "0".to_string());
    blockchain.add_block("Second_Block".to_string());
    blockchain.add_block("Third_Block".to_string());
    println!("{:#?}", blockchain);
// // {:?} 是一行流打印，紧凑但可能很长
// format!("{:?}", blockchain); 

// // {:#?} 中的 # 意思是 "Pretty-print"（美化打印），它会自动帮你换行、缩进，看起来像 JSON。
// format!("{:#?}", blockchain); 

    println!("Is blockchain valid? {}", blockchain.is_chain_valid());

    println!("Tampering with blockchain...");
    blockchain.chain[1].data = "Hacked Data".to_string();
    println!("Is blockchain valid after tampering? {}", blockchain.is_chain_valid());
    println!("{:#?}", blockchain);


    println!("Advanced Tampering with blockchain ...");
    // 1. 高级黑客再次篡改数据
    blockchain.chain[1].data = "Hacked Data again".to_string(); 
    // 2. 重新计算该区块的哈希并覆盖老哈希，试图制造区块“内部自洽”来蒙混过关
    blockchain.chain[1].hash = blockchain.chain[1].calculate_hash(); 
    
    // 尽管 chain[1] 内部看似合法，但为什么验证还是会返回 false？
    // 因为 chain[2].pre_hash 里记录的是链条生成时的“旧哈希”！
    // 我们的 is_chain_valid 遍历到 chain[2] 时，发现 chain[2].pre_hash 和刚刚被重写的 chain[1].hash 彻底对不上号了。
    // 真正做到了“牵一发而动全身”，除非黑客把后续所有区块的哈希连带重算，否则必定断链。
    println!("Is blockchain valid after rewriting hash self? {}", blockchain.is_chain_valid());
    println!("{:#?}", blockchain);
}