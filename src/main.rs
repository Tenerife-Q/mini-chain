mod block;
mod blockchain; // 注册新模块

// 按需导入需要的东西
use blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new(0, "Genesis Block".to_string(), "0".to_string());
    blockchain.add_block("Second_Block".to_string());
    blockchain.add_block("Third_Block".to_string());

    println!("{:#?}", blockchain);

    println!("Is blockchain valid? {}", blockchain.is_chain_valid());

    println!("Tampering with blockchain...");
    // main 文件现在是调用层，但因为它引入了 Block，所以依然可以强行修改 Block 的内部字段进行黑客实验
    blockchain.chain[1].data = "Hacked Data".to_string();
    println!("Is blockchain valid after tampering? {}", blockchain.is_chain_valid());

    println!("Advanced Tampering with blockchain ...");
    blockchain.chain[1].data = "Hacked Data again".to_string(); 
    blockchain.chain[1].hash = blockchain.chain[1].calculate_hash(); 
    
    println!("Is blockchain valid after rewriting hash self? {}", blockchain.is_chain_valid());

    blockchain.save_to_disk("blockchain_data.json");
    let recovered_blockchain = Blockchain::load_from_disk("blockchain_data.json");
    println!("Recovered Blockchain: {:#?}", recovered_blockchain);
}
