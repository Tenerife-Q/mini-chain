# 分布式网络节点与高并发：基于 Axum 与 Tokio

在这一阶段，我们将原本只能在本地通过命令行（CLI）执行的单机区块链，升级为**“能够被全网访问、支持高并发读写的微型网络分布式节点”**。

## 1. 痛点：为什么原来的代码不能作为分布式节点？

在之前的代码中：
- 我们的 `main()` 是普通函数，从头执行到尾就退出了。
- 我们的 `blockchain` 变量被 `add_block` 方法获取的是**独立可变借用** (`&mut self`)。
如果我们要让全世界的矿工（浏览器/客户端）同时连接我们的程序并挖掘新区块，就会发生：
> “成百上千个网络请求同时涌入，要求查看账本项目或者同时写入新区块。如果大家同时修改同一个 `Vec<Block>` 数组，会引发经典的**数据竞争（Data Race）**，导致严重崩溃。”

C++ 和 Java 开发者在这里通常会焦头烂额，而 Rust 凭借**并发双剑客（`Arc` 与 `Mutex`）**以及**异步核心（Tokio）**，确保了在全宇宙没有任何人能引发数据竞争。

---

## 2. 核心架构拆解：全新的四把利器

### A. Tokio（异步运行时引擎）
- **是什么**：底层的多线程环境发动机（类似 Node.js 的 Event Loop，但性能要强千百倍，且无垃圾回收）。
- **解决了什么**：原来我们处理一个网络请求，当前线程就卡死了（阻塞）。使用了 `#[tokio::main]` 后，我们能够同时容纳成千上万个矿工并发连接而不卡顿。

### B. Axum（超高性能 Web 框架）
- **是什么**：一个极简且暴力的 Rust Web API 服务框架。
- **解决了什么**：帮你把底层的 TCP 字节流，完美转换为上层应用可以懂的路由（Router）如 `GET /chain` 和 `POST /mine`，它由 Tokio 官方团队亲自操刀。

### C. `Arc` (Atomic Reference Count - 原子引用计数)
- **是什么**：它是一个具有极强克隆能力的共享智能指针。
- **为什么要用它**：网络框架需要把我们的 `Blockchain` 账本传给成百上千个负责处理请求的子任务，可是 Rust 规定所有权只有一份！`Arc::new(...)` 给账本发放了一个原子计数的身份牌，每次 `.clone()` 并不是复制整个厚重的账本，而只是让**记票器 +1**。它允许无数个路由去“引用”它。

### D. `Mutex` (Mutual Exclusion - 互斥锁)
- **是什么**：控制同时写权限的保安。
- **为什么要用它**：`Arc` 虽然搞定了多线程共享读取，但依然不允许改变数据！我们必须用 `Mutex::new(...)` 再包裹一层。任何网络请求如果想改账本（挖新块），必须先抢到这个唯一的锁：`.lock().unwrap()`。抢不到的人只能在门外异步排队等待。

---

## 3. 代码演进与填空对比解析

### 【填空 1】状态包裹：打造神圣的 SharedState
**修改前**：
```rust
let mut blockchain = Blockchain::new(...);
// 孤零零的变量，一传入闭包或者函数就会抛出 “所有权已转移” 的错误
```
**修改后**：
```rust
// 组装黄金三件套：Arc( Mutex( 真身 ) )
let shared_state: SharedState = Arc::new(Mutex::new(blockchain));
```
**好处**：这句代码确立了全局唯一、且线程安全的源数据。你可以把它挂载在 `Router::new().with_state(shared_state)` 上，这样所有路由函数都能触碰到它。

### 【填空 2】GET 接口：访问账本并克隆
```rust
async fn get_chain(State(state): State<SharedState>) -> Json<Blockchain> {
    // 【解析】：
    // 1. state.lock().unwrap() -> 抢互斥锁，抢到后我们有了临时查阅或修改的特权。
    // 2. .clone() -> 获取整个链条的数据副本（深拷贝）。
    // 说明：因为我们马上要把数据丢进序列化器（Json()），它要消耗数据。这也就是为什么
    // 我们需要在 block.rs 和 blockchain.rs 顶层宏补上 `Clone` 的原因！
    let chain_data = state.lock().unwrap().clone();
    Json(chain_data)
}
```

### 【填空 3】POST 接口：加互斥锁修改源数据
```rust
async fn mine_block(
    State(state): State<SharedState>,
    Json(payload): Json<MineRequest>,
) -> String {
    // 【解析】：
    // 对于写入类的高并发操作，获取到的不仅是锁，而且是可变借用（mut chain）。
    let mut chain = state.lock().unwrap();
    // 放入区块链中，这里的 add_block 就是你当初写的单机版逻辑，无需任何修改！
    chain.add_block(payload.data);
    
    // 大括号结束，锁自动被销毁，下一个排队的矿工请求才能进来。
    "挖矿完成...\n".to_string()
}
```

---

## 4. 总结：工业代码的平滑演进

你看，即使我们引入了极高难度的并发模型、异步 IO、以及强一致性的多线程锁，我们在第 1 课到第 6 课手敲的那个最核心的 `add_block` 和 `calculate_hash` 也是**只字未改的**。

这就得益于我们在最开始架构时设计好的：
> **「实体层（Block）」自洽 -> 「业务管理器（Blockchain）」全负责 -> 「终端/Web 展示（main/axum）」拼装拼图。**

现在的它，已经具有了被百万点击流量冲击也不会死库的基底。这正式标志着你从语法学习，成功跃迁到了**严肃底层系统构建**！