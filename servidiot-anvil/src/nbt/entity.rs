use nbt::Value;
use serde::{Deserialize, Serialize};
use servidiot_primitives::item::ItemStack;

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityBase {
    /// Describes the current X,Y,Z
    /// position of the entity.
    #[serde(rename = "Pos")]
    pub position: [f64; 3],
    /// Describes the current dX,dY,dZ
    /// velocity of the entity in
    /// meters per tick.
    #[serde(rename = "Motion")]
    pub motion: [f64; 3],
    /// Represents entity
    /// rotation in degrees.
    #[serde(rename = "Rotation")]
    pub rotation: [f32; 2],
    /// Distance the entity has fallen.
    /// Larger values cause more damage
    /// when the entity lands.
    #[serde(rename = "FallDistance")]
    pub fall_distance: f32,
    /// Number of ticks until the fire is put out.
    /// Negative values reflect how long the entity
    /// can stand in fire before burning.
    /// Default -1 when not on fire.
    #[serde(rename = "Fire")]
    pub fire: i16,
    /// How much air the entity has, in ticks.
    #[serde(rename = "Air")]
    pub air: i16,
    /// True if the entity is touching the ground.
    #[serde(rename = "OnGround")]
    pub on_ground: bool,
    /// True if the entity should not take damage.
    #[serde(rename = "Invulnerable")]
    pub invulnerable: bool,
    /// The number of ticks before which the entity
    /// may be teleported back through a portal of
    /// any kind.
    #[serde(rename = "PortalCooldown")]
    pub portal_cooldown: i32,
    /// The most significant bits of this entity's UUID.
    #[serde(rename = "UUIDMost")]
    pub uuid_most_significant: i64,
    /// The least significant bits of this entity's UUID.
    #[serde(rename = "UUIDLeast")]
    pub uuid_least_significant: i64,
    /// The custom name of this entity.
    #[serde(rename = "CustomName")]
    pub custom_name: Option<String>,
    /// If true, and this entity has a custom name,
    /// it will always appear above them, whether or
    /// not the cursor is pointing at it.
    #[serde(rename = "CustomNameVisible")]
    pub custom_name_visible: Option<bool>,
    /// The data of the entity being ridden. Note
    /// that if an entity is being ridden, the
    /// topmost entity in the stack has the Pos tag,
    /// and the coordinates specify the location of
    /// the bottommost entity. Also note that the
    /// bottommost entity controls movement, while
    /// the topmost entity determines spawning
    /// conditions when created by a mob spawner.
    #[serde(rename = "Riding")]
    pub riding: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemSlot {
    #[serde(flatten)]
    pub stack_data: ItemStack,
    #[serde(rename = "Slot")]
    pub slot: i8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MobBase {
    /// Amount of health the entity has, in floating point format. 
    /// If this tag exists, `health` will be ignored.
    #[serde(rename = "HealF")]
    pub health_float: Option<f32>,
    /// Amount of health the entity has.
    /// If the HealF tag exists, this tag will be ignored.
    #[serde(rename = "Health")]
    pub health: i16,
    /// Amount of extra health added by Absorption effect.
    #[serde(rename = "AbsorptionAmount")]
    pub absorption_amount: f32,
    /// Number of ticks the mob's "invincibility shield" 
    /// lasts after the mob was last struck. 0 when not 
    /// recently hit.
    #[serde(rename = "AttackTime")]
    pub attack_time: i16,
    /// Number of ticks the mob turns red for after being hit. 
    /// 0 when not recently hit.
    #[serde(rename = "HurtTime")]
    pub hurt_time: i16,
    /// Number of ticks the mob has been dead for. 
    /// Controls death animations. 0 when alive.
    #[serde(rename = "DeathTime")]
    pub death_time: i16,
    /// A list of Attributes for this mob.
    #[serde(rename = "Attributes")]
    pub attributes: Vec<MobAttribute>,
    /// The list of potion effects on this mob. 
    #[serde(rename = "ActiveEffects")]
    pub effects: Option<Vec<PotionEffect>>
}

/// A mob attribute. These are used for many purposes in internal 
/// calculations, and can be considered a mob's "statistics".  
#[derive(Debug, Serialize, Deserialize)]
pub struct MobAttribute {
    /// The name of this attribute.
    #[serde(rename = "Name")]
    pub name: String,
    /// The base value of this attribute.
    #[serde(rename = "Base")]
    pub base: f64,
    /// A list of modifiers acting on this attribute.
    #[serde(rename = "Modifiers")]
    pub modifiers: Vec<AttributeModifier>
}

/// Attribute modifiers alter the Base value in internal calculations,
/// without changing the original copy. Note that a Modifier
/// will never modify Base to be higher than its maximum or
/// lower than its minimum for a given Attribute.
#[derive(Debug, Serialize, Deserialize)]
pub struct AttributeModifier {
    /// The modifier's name.
    #[serde(rename = "Name")]
    pub name: String,
    /// The amount by which this Modifier modifies the Base value in calculations.
    #[serde(rename = "Amount")]
    pub amount: f64,
    /// Defines the operation this Modifier executes on the Attribute's Base value.
    /// 0: Increment X by Amount,
    /// 1: Increment Y by X * Amount,
    /// 2: Y = Y * (1 + Amount) (equivalent to Increment Y by Y * Amount).
    ///
    /// The game first sets X = Base, then executes all Operation 0 modifiers,
    /// then sets Y = X, then executes all Operation 1 modifiers, and finally
    /// executes all Operation 2 modifiers.
    #[serde(rename = "Operation")]
    pub operation: i32,
    /// The most significant bits of this modifier's UUID.
    #[serde(rename = "UUIDMost")]
    pub uuid_most_significant: i64,
    /// The least significant bits of this modifier's UUID.
    #[serde(rename = "UUIDLeast")]
    pub uuid_least_significant: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PotionEffect {
    /// The effect ID.
    #[serde(rename = "Id")]
    pub id: i8,
    /// The potion effect level. 0 is level 1.
    #[serde(rename = "Amplifier")]
    pub level: i8,
    /// The number of ticks before the effect wears off.
    #[serde(rename = "Duration")]
    pub duration: i32,
    /// True if this effect is provided by a beacon and 
    /// therefore should be less intrusive on screen.
    #[serde(rename = "Ambient")]
    pub ambient: bool,
    /// True if particles are shown (affected by "Ambient"). 
    /// False if no particles are shown.
    #[serde(rename = "ShowParticles")]
    pub show_particles: bool
}