use serde::{Serialize, Deserialize};


/// Player abilities.
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerAbilities {
    /// The player's walking speed.
    #[serde(rename = "walkSpeed")]
    pub walk_speed: f32,
    /// The player's flight speed.
    #[serde(rename = "flySpeed")]
    pub fly_speed: f32,
    /// Whether the player can fly or not.
    #[serde(rename = "mayfly")]
    pub can_fly: bool,
    /// True if the player is currently flying.
    #[serde(rename = "flying")]
    pub is_flying: bool,
    /// True if the player is immune to damage.
    pub invulnerable: bool,
    /// True if the player is allowed to build.
    #[serde(rename = "mayBuild")]
    pub may_build: bool,
    /// True if the player is allowed to
    /// instantly break blocks.
    #[serde(rename = "instabuild")]
    pub instabreak: bool
}

