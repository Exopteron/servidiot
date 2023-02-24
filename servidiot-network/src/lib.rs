#![feature(cursor_remaining, never_type)]
//! Minecraft networking.
pub mod io;
pub mod connection;
pub mod server;

// #[cfg(test)]
// mod tests {
//     use std::{sync::Arc, time::Duration};

//     use rsa::RsaPrivateKey;

//     use crate::connection::{listener::Listener, ServerState};

//     #[tokio::test]
//     async fn net_test() {
//         let (send, recv) = flume::unbounded();

//         let mut rng = rand::thread_rng();
//         let bits = 1024;
//         let server_state = ServerState {
//             rsa_key: RsaPrivateKey::new(&mut rng, bits).unwrap()
//         };
//         let server_state = Arc::new(server_state);
//         let listener = Listener::bind("localhost:7878", send, server_state).await.unwrap();
//         listener.start().await;
//         loop {
//             tokio::time::sleep(Duration::MAX).await;
//         }
//     }
// }