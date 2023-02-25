use servidiot_primitives::item::InventorySlot;

use crate::{io::{
    packet::{def_packets, def_user_enum, packet_enum},
    LengthPrefixedVec,
}, server::id::NetworkID};

bitflags::bitflags! {
    struct ChatFlags: i8 {
        const COMMANDS_ONLY = 0b01;
        const HIDDEN = 0b10;
    }
}

def_packets! {
    ClientSettings {
        locale: String,
        view_distance: i8,
        chat_flags: i8,
        chat_colours: bool,
        difficuty: i8,
        show_cape: bool
    },
    PluginMessage {
        channel: String,
        data: LengthPrefixedVec<i16, u8>
    },
    PlayerPositionAndLook {
        x: f64,
        feet_y: f64,
        head_y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool
    },
    PlayerPosition {
        x: f64,
        feet_y: f64,
        head_y: f64,
        z: f64,
        on_ground: bool
    },
    PlayerLook {
        yaw: f32,
        pitch: f32,
        on_ground: bool
    },
    Player {
        on_ground: bool
    },
    PlayerAbilities {
        flags: i8,
        flying_speed: f32,
        walking_speed: f32
    },
    KeepAlive {
        id: i32
    },
    EntityAction {
        eid: NetworkID,
        action: EntityActionType,
        jump_boost: i32
    },
    Animation {
        eid: NetworkID,
        animation: AnimationType
    },
    ClientStatus {
        ty: ClientStatusType
    },
    CloseWindow {
        window_id: i8
    },
    PlayerBlockPlacement {
        x: i32,
        y: u8,
        z: i32,
        direction: i8,
        held_item: InventorySlot,
        cursor_position_x: i8,
        cursor_position_y: i8,
        cursor_position_z: i8
    },
    PlayerDigging {
        status: DiggingStatus,
        x: i32,
        y: u8,
        z: i32,
        face: Face
    },
    CreativeInventoryAction {
        slot: i16,
        item: InventorySlot
    },
    HeldItemChange {
        slot: i16
    }
}

packet_enum!(ClientPlayPacket {
    ClientSettings = 0x15,
    PluginMessage = 0x17,
    Player = 0x03,
    PlayerPosition = 0x04,
    PlayerLook = 0x05,
    PlayerPositionAndLook = 0x06,
    KeepAlive = 0x00,
    PlayerAbilities = 0x13,
    EntityAction = 0x0B,
    Animation = 0x0A,
    ClientStatus = 0x16,
    CloseWindow = 0x0D,
    PlayerBlockPlacement = 0x08,
    PlayerDigging = 0x07,
    CreativeInventoryAction = 0x10,
    HeldItemChange = 0x09
});

def_user_enum! {
    EntityActionType (i8) {
        Crouch = 1,
        Uncrouch = 2,
        LeaveBed = 3,
        StartSprinting = 4,
        StopSprintingOrHorse = 5,
        HorseJump = 6
    }
}


def_user_enum! {
    AnimationType (i8) {
        NoAnimation = 0,
        SwingArm = 1,
        DamageAnimation = 2,
        LeaveBed = 3,
        EatFood = 5,
        Critical = 6,
        MagicCrit = 7,
        Unknown = 102,
        Crouch = 104,
        Uncrouch = 105
    }
}


def_user_enum! {
    ClientStatusType (i8) {
        PerformRespawn = 0,
        RequestStats = 1,
        TakingInventoryAchievement = 2
    }
}


def_user_enum! {
    DiggingStatus (i8) {
        Started = 0,
        Cancelled = 0,
        Finished = 0,
        DropItemStack = 3,
        DropItem = 4,
        ShootArrow = 5
    }
}



def_user_enum! {
    Face (i8) {
        Invalid = -1,
        NegativeY = 0,
        PositiveY = 1,
        NegativeZ = 2,
        PositiveZ = 3,
        NegativeX = 4,
        PositiveX = 5
    }
}
