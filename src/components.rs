use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Facing(pub Direction);

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    pub fn indicator_offset(&self, half: f32, bar_half: f32) -> (f32, f32, f32, f32) {
        match self {
            Direction::Up => (0.0, half - bar_half, 2.0 * half, 2.0 * bar_half),
            Direction::Down => (0.0, -(half - bar_half), 2.0 * half, 2.0 * bar_half),
            Direction::Left => (-(half - bar_half), 0.0, 2.0 * bar_half, 2.0 * half),
            Direction::Right => (half - bar_half, 0.0, 2.0 * bar_half, 2.0 * half),
        }
    }
}

#[derive(Component)]
pub struct Carrying(pub Option<(Entity, ItemKind)>);

impl Carrying {
    pub fn empty() -> Self {
        Carrying(None)
    }

    pub fn clear(&mut self, commands: &mut Commands) {
        if let Some((entity, _)) = self.0.take() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    DoughBatch,
    RawCrustTray,
    BakedCrustTray,
    Case,
}

impl ItemKind {
    pub fn color(&self) -> Color {
        match self {
            ItemKind::DoughBatch => Color::srgb(0.95, 0.85, 0.65),
            ItemKind::RawCrustTray => Color::srgb(0.85, 0.65, 0.35),
            ItemKind::BakedCrustTray => Color::srgb(0.65, 0.45, 0.25),
            ItemKind::Case => Color::srgb(0.55, 0.30, 0.10),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ItemKind::DoughBatch => "Dough",
            ItemKind::RawCrustTray => "Raw Tray",
            ItemKind::BakedCrustTray => "Baked Tray",
            ItemKind::Case => "Case",
        }
    }
}

#[derive(Component)]
pub struct Item {
    pub kind: ItemKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StationKind {
    Source,
    Former,
    Oven,
    Packer,
    Palletizer,
    Table,
}

impl StationKind {
    pub fn colors(&self) -> (Color, Color, Color) {
        match self {
            StationKind::Source => (Color::srgb(0.2, 0.8, 0.2), Color::srgb(0.12, 0.48, 0.12), Color::srgb(0.4, 1.0, 0.4)),
            StationKind::Former => (Color::srgb(0.8, 0.5, 0.2), Color::srgb(0.48, 0.30, 0.12), Color::srgb(1.0, 0.7, 0.4)),
            StationKind::Oven => (Color::srgb(0.9, 0.3, 0.1), Color::srgb(0.54, 0.18, 0.06), Color::srgb(1.0, 0.5, 0.3)),
            StationKind::Packer => (Color::srgb(0.3, 0.3, 0.8), Color::srgb(0.18, 0.18, 0.48), Color::srgb(0.5, 0.5, 1.0)),
            StationKind::Palletizer => (Color::srgb(0.8, 0.2, 0.8), Color::srgb(0.48, 0.12, 0.48), Color::srgb(1.0, 0.4, 1.0)),
            StationKind::Table => (Color::srgb(0.6, 0.4, 0.2), Color::srgb(0.6, 0.4, 0.2), Color::srgb(0.6, 0.4, 0.2)),
        }
    }

    pub fn color_idle(&self) -> Color { self.colors().0 }
    pub fn color_busy(&self) -> Color { self.colors().1 }
    pub fn color_ready(&self) -> Color { self.colors().2 }

    pub fn label(&self) -> String {
        match self {
            StationKind::Source => "Source",
            StationKind::Former => "Former",
            StationKind::Oven => "Oven",
            StationKind::Packer => "Packer",
            StationKind::Palletizer => "Pallet",
            StationKind::Table => "Table",
        }.to_string()
    }
}

#[derive(Component)]
pub struct Station {
    pub kind: StationKind,
    pub accepted_kind: ItemKind,
    pub output_kind: ItemKind,
    pub process_duration: f32,
    pub timer: f32,
    pub busy: bool,
    pub has_output: bool,
    pub packer_count: u32,
    pub inputs_needed: u32,
    pub spawn_timer: f32,
    pub spawn_interval: f32,
}

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct GameOverText;

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct GridLine;

#[derive(Component)]
pub struct DirectionIndicator;

#[derive(Component)]
pub struct NpcDirectionIndicator {
    pub npc_entity: Entity,
}

#[derive(Component)]
pub struct StationLabel {
    pub station_entity: Entity,
}

#[derive(Component)]
pub struct TableMarker;

#[derive(Component)]
pub struct FloorTimer(pub f32);

#[derive(Component)]
pub struct ConveyorBelt {
    pub direction: Direction,
}

#[derive(Component)]
pub struct ConveyorArrow {
    pub direction: Direction,
    pub base: Vec3,
}

#[derive(Component)]
pub struct Npc {
    pub move_timer: f32,
    pub action_timer: f32,
    pub move_cooldown: f32,
    pub action_cooldown: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConveyorLoaderState {
    WaitingAtConveyor,
    PickingUp,
    MovingToFormer,
    InsertingToFormer,
    WaitingForFormer,
    CollectingFromFormer,
    InsertingToOven,
    ReturningToConveyor,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OvenHaulerState {
    WaitingAtOven,
    CollectingFromOven,
    MovingToPacker,
    InsertingToPacker,
    ReturningToOvenWait,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PackerHaulerState {
    WaitingAtPacker,
    CollectingFromPacker,
    MovingToPalletizer,
    InsertingToPalletizer,
    ReturningToPackerWait,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum NpcKind {
    ConveyorLoader,
    OvenHauler,
    PackerHauler,
}

#[derive(Component)]
pub struct ConveyorLoaderTargets {
    pub spawn: GridPos,
}

impl ConveyorLoaderTargets {
    pub fn new(spawn: GridPos) -> Self {
        Self { spawn }
    }

    pub fn item_check(&self) -> GridPos {
        GridPos { x: self.spawn.x - 1, y: self.spawn.y }
    }

    pub fn operate_pos(&self) -> GridPos {
        GridPos { x: self.spawn.x, y: self.spawn.y - 1 }
    }

    pub fn former_pos(&self) -> GridPos {
        GridPos { x: self.spawn.x - 1, y: self.spawn.y - 1 }
    }
}

#[derive(Component)]
pub struct OvenHaulerTargets {
    pub spawn: GridPos,
}

impl OvenHaulerTargets {
    pub fn new(spawn: GridPos) -> Self {
        Self { spawn }
    }

    pub fn oven_pos(&self) -> GridPos {
        GridPos { x: self.spawn.x - 1, y: self.spawn.y }
    }

    pub fn packer_stand(&self) -> GridPos {
        GridPos { x: self.spawn.x + 3, y: self.spawn.y + 2 }
    }
}

#[derive(Component)]
pub struct PackerHaulerTargets {
    pub spawn: GridPos,
}

impl PackerHaulerTargets {
    pub fn new(spawn: GridPos) -> Self {
        Self { spawn }
    }

    pub fn packer_pos(&self) -> GridPos {
        GridPos { x: self.spawn.x + 1, y: self.spawn.y }
    }

    pub fn palletizer_stand(&self) -> GridPos {
        GridPos { x: self.spawn.x, y: self.spawn.y - 4 }
    }
}


