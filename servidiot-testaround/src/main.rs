use std::{sync::Arc, num::{NonZeroU64, NonZeroUsize}, net::{SocketAddr, Ipv4Addr, IpAddr}};

use servidiot_core::Config;



fn main() {
    tracing_subscriber::fmt().compact().init();
    let runtime = servidiot_core::GameRuntime::create(Arc::new(Config {
        net_threads: NonZeroUsize::new(2).unwrap(),
        game_threads: NonZeroUsize::new(2).unwrap(),
        tps: NonZeroU64::new(20).unwrap(),
        bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25565),
    })).unwrap();

    runtime.run();

    // let mut locker = servidiot_utils::synchronisation::Synchroniser::<u32>::new();

    // {
    //     let res = locker.synchronise_for([1, 2, 3]).await;
    //     println!("Reserved!");
    //     let res = res.expand([4]).await;
    // }
    // {
    //     let res2 = locker.synchronise_for([3]).await;
    //     println!("Reserved once more!");
    // }

    println!("Hello, world!");
}
