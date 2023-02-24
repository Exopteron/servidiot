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
#[derive(Debug, Clone, Copy)]
pub enum GamemodeType {
    Survival,
    Creative,
    Adventure
}

/// The gamemode this player is in.
#[derive(Debug, Clone, Copy)]
pub struct Gamemode {
    pub ty: GamemodeType,
    pub hardcore: bool
}
impl Gamemode {
    pub fn new(ty: GamemodeType, hardcore: bool) -> Self {
        Self {
            ty,
            hardcore
        }
    }

    pub fn decode(mut n: u8) -> Option<Self> {
        let mut hardcore = false;
        if (n & 0x8) != 0 {
            n &= !0x8;
            hardcore = true;
        }
        let ty = match n {
            0 => Some(GamemodeType::Survival),
            1 => Some(GamemodeType::Creative),
            2 => Some(GamemodeType::Adventure),
            _ => None
        }?;
        Some(Self {
            ty,
            hardcore
        })
    }

    pub fn encode(&self) -> u8 {
        let mut n = match self.ty {
            GamemodeType::Survival => 0,
            GamemodeType::Creative => 1,
            GamemodeType::Adventure => 2
        };
        if self.hardcore {
            n |= 0x8;
        }
        n
    }
}
