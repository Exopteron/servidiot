use nbt::Value;
use serde::{Serialize, Deserialize};

/// Represents a Minecraft item.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ItemStack {
    /// Number of items stacked in this item. 
    #[serde(rename = "Count")]
    pub count: i8,
    /// The data value for this item. 
    #[serde(rename = "Damage")]
    pub meta: i16,
    /// The item/block ID.
    pub id: i16,
    /// This item's NBT data.
    #[serde(rename = "tag")]
    pub nbt_data: Option<Value>
}

/// Represents an inventory slot.
#[derive(PartialEq, Clone, Debug)]
pub enum InventorySlot {
    Empty,
    Filled(ItemStack)
}

impl InventorySlot {
    pub fn is_empty(&self) -> bool {
        matches!(self, InventorySlot::Empty)
    }
}