use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
}

#[derive(Component)]
pub struct Carrying(pub Option<Entity>);

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
}

impl StationKind {
    pub fn color(&self) -> Color {
        match self {
            StationKind::Source => Color::srgb(0.2, 0.8, 0.2),
            StationKind::Former => Color::srgb(0.8, 0.5, 0.2),
            StationKind::Oven => Color::srgb(0.9, 0.3, 0.1),
            StationKind::Packer => Color::srgb(0.3, 0.3, 0.8),
            StationKind::Palletizer => Color::srgb(0.8, 0.2, 0.8),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            StationKind::Source => "Source",
            StationKind::Former => "Former",
            StationKind::Oven => "Oven",
            StationKind::Packer => "Packer",
            StationKind::Palletizer => "Pallet",
        }
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
    pub spawn_timer: f32,
    pub spawn_interval: f32,
}

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct GameOverText;

#[derive(Component)]
pub struct HudText;
