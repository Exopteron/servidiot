use serde::{Serialize, Deserialize};
use servidiot_primitives::player::PlayerAbilities;

use super::entity::{EntityBase, MobBase, ItemSlot};


#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerData {
    /// Inherited from entities.
    #[serde(flatten)]
    pub entity_data: EntityBase,
    /// Inherited from mobs.
    #[serde(flatten)]
    pub mob_data: MobBase,
    /// The dimension the player is in.
    /// Invalid values are interpreted as zero.
    #[serde(rename = "Dimension")]
    pub dimension: i32,
    /// The game mode of the player.
    #[serde(rename = "playerGameType")]
    pub game_mode: i32,
    /// The Score displayed upon death.
    #[serde(rename = "Score")]
    pub score: i32,
    /// The selected hotbar slot of the player.
    #[serde(rename = "SelectedItemSlot")]
    pub selected_item_slot: i32,
    /// See below.
    #[serde(rename = "SpawnX")]
    pub spawnpoint_x: Option<i32>,
    /// The coordinates of the player's bed.
    #[serde(rename = "SpawnY")]
    pub spawnpoint_y: Option<i32>,
    /// See above.
    #[serde(rename = "SpawnZ")]
    pub spawnpoint_z: Option<i32>,
    /// True if the player should spawn at their 
    /// spawnpoint coordinates even if no bed 
    /// can be found.
    #[serde(rename = "SpawnForced")]
    pub spawn_forced: Option<bool>,
    /// True if the player was in a bed 
    /// when this tag was saved.
    #[serde(rename = "Sleeping")]
    pub sleeping: bool,
    /// The number of ticks the player had 
    /// been in bed when this tag was saved. 
    #[serde(rename = "SleepTimer")]
    pub sleep_timer: i16,
    /// The value of the hunger bar.
    #[serde(rename = "foodLevel")]
    pub food_level: i32,
    #[serde(rename = "foodExhaustionLevel")]
    pub food_exhaustion_level: f32,
    #[serde(rename = "foodSaturationLevel")]
    pub food_saturation_level: f32,
    #[serde(rename = "foodTickTimer")]
    pub food_tick_timer: i32,
    /// The level shown on the XP bar.
    #[serde(rename = "XpLevel")]
    pub xp_level: i32,
    /// The progress/percent across the XP bar to the next level.
    #[serde(rename = "XpP")]
    pub xp_percentage: f32,
    /// The total amount of XP the player has collected 
    /// over time; used for the Score upon death.
    #[serde(rename = "XpTotal")]
    pub xp_total: i32,
    /// The seed used for the next enchantment 
    /// in Enchantment Tables.
    #[serde(rename = "XpSeed")]
    pub xp_seed: Option<i32>,
    /// Each entry in this list is an 
    /// item in the player's inventory. 
    #[serde(rename = "Inventory")]
    pub inventory: Vec<ItemSlot>,
    /// The items in the player's ender chest.
    #[serde(rename = "EnderItems")]
    pub ender_chest: Vec<ItemSlot>,
    /// The player's abilities.
    pub abilities: PlayerAbilities

}
