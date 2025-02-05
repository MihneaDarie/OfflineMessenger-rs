mod serv;
use serv::ServerManager;
use common::{IP,ServerDetails};


#[tokio::main]
async fn main() {
    let server = ServerManager::new(10, IP::V4(127, 0, 0, 1), 2098);
    server.run().await;
}
