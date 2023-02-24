use crate::MinecraftServer;
mod network;

use super::system::SystemExecutor;

pub fn register_systems(s: &mut SystemExecutor<MinecraftServer>) {
    network::register_systems(s);
}

