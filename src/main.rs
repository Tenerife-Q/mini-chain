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

    // 【容错落地演示 1】：使用 if let 优雅捕捉 Result 这个盲盒里可能装的 Err
    if let Err(e) = blockchain.save_to_disk("blockchain_data.json") {
        println!("【节点告警】保存区块链到磁盘失败，原因：{}", e);
    } else {
        println!("区块链已成功落盘备份！不会因为写入权限问题导致节点崩溃。");
    }

    // 【容错落地演示 2】：使用经典的 match 分支拆分所有可能，模拟容错回滚
    match Blockchain::load_from_disk("blockchain_bad_file.json") { // 这里故意放错一个文件名测试
        Ok(recovered_blockchain) => {
            println!("成功从磁盘恢复区块链！记录如下：\n{:#?}", recovered_blockchain);
        }
        Err(e) => {
            // 如果现实中遇到黑客喂的假 JSON ，我们的节点坚挺存活，这里只会打印一句警告，然后可以重新同步别人的链！
            println!("【致命节点抛弃】探测到损坏的文件或被修改的文件头！节点存活不受影响。错误信息：{}", e);
        }
    }
}
