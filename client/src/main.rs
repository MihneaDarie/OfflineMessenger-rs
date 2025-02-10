mod cli;
use cli::ClientManager;
use common::IP;


#[tokio::main]
async fn main() {
    let client = ClientManager::new(IP::V4(127, 0, 0, 1), 2098);
    client.run().await;
}
