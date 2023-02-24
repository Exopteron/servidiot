use std::time::Duration;

use servidiot_core::{MinecraftServer, Server};


#[tokio::main]
async fn main() {
    env_logger::init();
    let mut server = Server::bind("127.0.0.1:7878").await.unwrap();
    println!("Bound ");
    tokio::task::spawn_blocking(move || {
        let mut game = MinecraftServer::new(server).unwrap();
        loop {
            game.update();
            std::thread::sleep(Duration::from_millis(5));
        }
    });
    loop {}
    println!("Hello, world!");
}
