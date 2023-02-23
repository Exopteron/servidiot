use ahash::HashMap;
use nbt::{Value};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LevelRoot {
    /// This tag contains all the level data. 
    #[serde(rename = "Data")]
    data: LevelData
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LevelData {
    /// The NBT version of the level, 19133.
    version: i32,
    /// Normally true after a world has been 
    /// initialized properly after creation. 
    /// If the initial simulation was canceled 
    /// somehow, this can be false and the world 
    /// will be re-initialized on next load.
    initialized: bool,
    /// The name of the level.
    #[serde(rename = "LevelName")]
    level_name: String,
    /// The name of the generator.
    #[serde(rename = "generatorName")]
    generator_name: String,
    /// The version of the level generator. 
    #[serde(rename = "generatorVersion")]
    generator_version: i32,
    /// Controls options for the generator.
    #[serde(rename = "generatorOptions")]
    generator_options: String,
    /// The random level seed used to 
    /// generate consistent terrain.
    #[serde(rename = "RandomSeed")]
    world_seed: i64,
    /// True if the map generator should 
    /// place structures such as villages, 
    /// strongholds, and mineshafts.
    #[serde(rename = "MapFeatures")]
    map_features: bool,
    /// The Unix time when the level was last loaded.
    #[serde(rename = "LastPlayed")]
    last_played: i64,
    /// True if cheats are enabled.
    #[serde(rename = "allowCommands")]
    cheats_enabled: bool,
    /// True if the player must delete their 
    /// world on death in singleplayer. Affects 
    /// all three game modes.
    hardcore: bool,
    /// The default game mode for the singleplayer 
    /// player when they spawn or respawn.
    #[serde(rename = "GameType")]
    game_type: i32,
    /// The current difficulty setting. 
    #[serde(rename = "Difficulty", default = "difficulty_default")]
    difficulty: i8,
    /// True if the difficulty has been locked.
    #[serde(rename = "DifficultyLocked", default = "difficulty_locked_default")]
    difficulty_locked: bool,
    /// The number of ticks since the start of the level.
    #[serde(rename = "Time")]
    level_ticks: i64,
    /// The time of day.
    #[serde(rename = "DayTime")]
    day_time: i64,
    /// The X coordinate of the world spawn.
    #[serde(rename = "SpawnX")]
    spawn_x: i32,
    /// The Y coordinate of the world spawn.
    #[serde(rename = "SpawnY")]
    spawn_y: i32,
    /// The Z coordinate of the world spawn.
    #[serde(rename = "SpawnZ")]
    spawn_z: i32,
    /// True if the level is currently 
    /// experiencing rain, snow, and 
    /// cloud cover.
    raining: bool,
    /// The number of ticks before "raining" 
    /// is toggled and this value gets 
    /// set to another random value.
    #[serde(rename = "rainTime")]
    rain_time: i32,
    /// True if the rain/snow/cloud cover is 
    /// a lightning storm and dark enough for 
    /// mobs to spawn under the sky.
    thundering: bool,
    /// The number of ticks before "thundering" 
    /// is toggled and this value gets set to 
    /// another random value.
    #[serde(rename = "thunderTime")]
    thunder_time: i32,
    /// The state of the Singleplayer player.
    #[serde(rename = "Player")]
    player: Value,
    /// The game rules. Each rule is a string 
    /// that is either "true" or "false". 
    #[serde(rename = "GameRules")]
    game_rules: HashMap<String, String>
}

const fn difficulty_default() -> i8 {
    0
}
const fn difficulty_locked_default() -> bool {
    false
}