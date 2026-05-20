mod block;
mod blockchain;

use block::Block;
use blockchain::Blockchain;

use axum::{
    routing::{get, post},
    Router, Json, extract::State,
};
use std::sync::{Arc, Mutex};
use serde::Deserialize;

#[derive(Deserialize)]
struct MineRequest {
    data: String,
}

type SharedState = Arc<Mutex<Blockchain>>;

#[tokio::main]
async fn main() {
    println!("正在启动 PingCAP 级微型区块链分布式节点...");

    let blockchain = Blockchain::new(0, "Genesis Block".to_string(), "0".to_string());
    
    let shared_state: SharedState = Arc::new(Mutex::new(blockchain));

    let app = Router::new()
        .route("/chain", get(get_chain))
        .route("/mine", post(mine_block))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("节点已启动！请打开浏览器访问 http://localhost:8000/chain");
    
    axum::serve(listener, app).await.unwrap();
}

async fn get_chain(State(state): State<SharedState>) -> Json<Blockchain> {
    let chain_data = state.lock().unwrap().clone();
    Json(chain_data)
}

async fn mine_block(
    State(state): State<SharedState>,
    Json(payload): Json<MineRequest>,
) -> String {
    let mut chain = state.lock().unwrap();
    chain.add_block(payload.data);
    "挖矿完成！新区块已被矿工确认并加入网络。\n".to_string()
}