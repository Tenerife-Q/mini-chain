use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use std::fs;

const DIFFICULTY: usize = 2; // 先设置为2，表示哈希必须以 "00" 开头

#[derive(Debug, Deserialize, Serialize)] // 让区块链结构体支持序列化和反序列化，方便保存和加载区块链数据。
struct Block {
    index: u64,
    timestamp: u64,
    data: String,
    pre_hash: String,
    hash: String,
    Nonce: u64,
}

#[derive(Debug, Deserialize, Serialize)]// #[derive(Debug)]是一个Rust的属性宏，
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

    pub fn save_to_disk(&self, file_path: &str) {
        // 【深入挖掘：序列化与所有权】
        // 1. to_string_pretty 扮演一个“抄写员”：
        //    它只拿到了当前区块链的“只读借用 (&self)”。它挨个探头去看里面的区块、看区块里的字段。
        // 2. 然后，它在内存（Heap堆）中申请一块全新的空间，把看到的内容，一块一块拼接成了 JSON 纯文本。
        //    这个动作绝对没有剥夺原本区块链里任何一个字符的所有权，原件依然完好无损。
        // 3. 为什么用 .unwrap()？
        //    to_string_pretty 返回的是 Result 枚举（要么是 Ok(转换好的那坨文本), 要么是 Err(转换出错了)）。
        //    unwrap() 就是命令编译器：强行把 Ok 框切开，取出里面的内容；如果里面是炸弹(Err)，直接让程序崩溃！
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        // 4. fs::write 调用底层操作系统 API (如 C 语言的 write)。它把 serialized 无缝转移写进硬盘，随后 serialized 被从内存销毁。
        fs::write(file_path, serialized).expect("Unable to write file");
    }

    pub fn load_from_disk(file_path: &str) -> Self {
        // 【深入挖掘：反序列化与从“无”到“有”的所有权新生】
        // 1. 从硬盘文件读进内存，操作系统的驱动层把硬盘字节拷贝到内存，系统返还给你一个全新的、极其巨大的 String。
        let file_content = fs::read_to_string(file_path).unwrap();
        
        // 2. 为什么这里必须用借用 &file_content 而不是直接交出所有权？
        //    因为 serde_json::from_str 这个底层函数的作者，把它设计成了只接收“字符串切片借用(&str)”。
        //    在 Rust 的规矩里：如果一个函数只是需要“看”里面的数据来干活，绝不应该索要所有权。
        //    哪怕我们外面这个 file_content 马上就要报废了，函数依然只收“参观票”。
        //
        // 3. 为什么必须指定 : Self ？
        //    因为 from_str 是一个“万能转化器(泛型)”，它能把 JSON 变成数字、数组、别的结构体。
        //    如果我们不加 : Self（或者显式写出 : Blockchain），编译器就会懵逼：“你要我捏成啥样？”
        let recovered_blockchain: Self = serde_json::from_str(&file_content).unwrap();
        
        // 4. 返回动作与生命周期终结：
        //    这个函数结束时（大括号结束），那个巨大的 file_content 文本变量失去了作用，它的所有权结束，被原地扔进垃圾壳销毁（Drop）。
        //    而那个靠组装图生拔出来的新 Blockchain 的生命才刚开始，它随着隐式返回（不带分号），被移交（Move）给了外部函数 main 中的调用者。
        recovered_blockchain
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
            hash: String::new(), // 先给个空字符串占位
            Nonce: 0,
        };
        // 【深入挖掘：从“计算一次”变成“疯狂计算”】
        // 以前我们只调用一次 block.hash = block.calculate_hash(); 就完事了。
        // 现在要满足工作量证明，我们直接让刚打包出来的区块调用自己的 mine 方法。
        // 由于 mine 参数签名为 &mut self，此处会对刚生成一半的 block 进行原地疯狂开采。
        // 只有开采出符合 DIFFICULTY 条件的哈希后，mine 方法才会结束，随后返回一个完全合法的 block 给调用者！
        block.mine();
        block

    }

    // &self: 不可变借用区块自身的字段用于计算。如果不用引用，区块在第一次算完哈希后就会灰飞烟灭。
    fn calculate_hash(&self) -> String {
        let mock_hash = format!(
            "{}{}{}{}{}",self.index, self.data, self.pre_hash, self.timestamp, self.Nonce
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
        // 【深入挖掘：切片与 starts_with 的安全性】
        // 之前你写的 &self.hash[..DIFFICULTY] 如果 hash 长度不够（比如刚初始化为空字符串 String::new()）
        // 程序就会发生 "out of bounds" （越界）导致直接崩溃！
        // .starts_with() 则是 Rust 里的安全防线，就算字符串是空的，它也只会默默返回 false 而不崩溃。
        while !self.hash.starts_with(&target) {
            self.Nonce += 1;
            // 因为 Nonce 变了，所以重新算一次哈希
            self.hash = self.calculate_hash();
        }
        println!("Block {} mined !!! Nonce: {}, Hash: {}", self.index, self.Nonce, self.hash);
    }
}

fn main() {
    let mut blockchain = Blockchain::new(0, "Genesis Block".to_string(), "0".to_string());
    blockchain.add_block("Second_Block".to_string());
    blockchain.add_block("Third_Block".to_string());
    //to_string() 是 Rust 中的一个方法，用于将字符串字面量转换为 String 类型。
    //字符串字面量（例如 "Hello"）是不可变的字符串切片（&str），
    //而 String 是一个可变的、拥有所有权的字符串类型。
    //使用 to_string() 可以方便地将字符串字面量转换为 String，以便在需要 String 类型的地方使用。
    println!("{:#?}", blockchain);
    //  {:?} 是一行流打印，紧凑但可能很长
    // format!("{:?}", blockchain); 

    //  {:#?} 中的 # 意思是 "Pretty-print"（美化打印），它会自动帮你换行、缩进，看起来像 JSON。
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




    blockchain.save_to_disk("blockchain_data.json");
    let recoverd_blockchain = Blockchain::load_from_disk("blockchain_data.json");
    println!("Recovered Blockchain: {:#?}", recoverd_blockchain);
}