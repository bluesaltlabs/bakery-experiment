use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::components::{ItemKind, StationKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StationBehavior {
    Normal,
    Source,
    Packer,
    Palletizer,
    Table,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationDef {
    pub accepted_kind: ItemKind,
    pub output_kind: ItemKind,
    pub process_duration: f32,
    pub spawn_interval: f32,
    pub inputs_needed: u32,
    pub behavior: StationBehavior,
    pub label: String,
    pub color_idle: (f32, f32, f32),
    pub color_busy: (f32, f32, f32),
    pub color_ready: (f32, f32, f32),
}

impl StationDef {
    pub fn col_idle(&self) -> Color { Color::srgb(self.color_idle.0, self.color_idle.1, self.color_idle.2) }
    pub fn col_busy(&self) -> Color { Color::srgb(self.color_busy.0, self.color_busy.1, self.color_busy.2) }
    pub fn col_ready(&self) -> Color { Color::srgb(self.color_ready.0, self.color_ready.1, self.color_ready.2) }
}

#[derive(Resource, Clone)]
pub struct StationConfig(pub HashMap<StationKind, StationDef>);

impl StationConfig {
    pub fn def(&self, kind: StationKind) -> &StationDef {
        &self.0[&kind]
    }
}

impl Default for StationConfig {
    fn default() -> Self {
        StationConfig(HashMap::from([
            (StationKind::Source, StationDef {
                accepted_kind: ItemKind::DoughBatch,
                output_kind: ItemKind::DoughBatch,
                process_duration: 0.0,
                spawn_interval: 4.0,
                inputs_needed: 1,
                behavior: StationBehavior::Source,
                label: "Source".into(),
                color_idle: (0.2, 0.8, 0.2),
                color_busy: (0.12, 0.48, 0.12),
                color_ready: (0.4, 1.0, 0.4),
            }),
            (StationKind::Former, StationDef {
                accepted_kind: ItemKind::DoughBatch,
                output_kind: ItemKind::RawCrustTray,
                process_duration: 3.0,
                spawn_interval: 0.0,
                inputs_needed: 1,
                behavior: StationBehavior::Normal,
                label: "Former".into(),
                color_idle: (0.8, 0.5, 0.2),
                color_busy: (0.48, 0.30, 0.12),
                color_ready: (1.0, 0.7, 0.4),
            }),
            (StationKind::Oven, StationDef {
                accepted_kind: ItemKind::RawCrustTray,
                output_kind: ItemKind::BakedCrustTray,
                process_duration: 5.0,
                spawn_interval: 0.0,
                inputs_needed: 1,
                behavior: StationBehavior::Normal,
                label: "Oven".into(),
                color_idle: (0.9, 0.3, 0.1),
                color_busy: (0.54, 0.18, 0.06),
                color_ready: (1.0, 0.5, 0.3),
            }),
            (StationKind::Packer, StationDef {
                accepted_kind: ItemKind::BakedCrustTray,
                output_kind: ItemKind::Case,
                process_duration: 2.0,
                spawn_interval: 0.0,
                inputs_needed: 3,
                behavior: StationBehavior::Packer,
                label: "Packer".into(),
                color_idle: (0.3, 0.3, 0.8),
                color_busy: (0.18, 0.18, 0.48),
                color_ready: (0.5, 0.5, 1.0),
            }),
            (StationKind::Palletizer, StationDef {
                accepted_kind: ItemKind::Case,
                output_kind: ItemKind::Case,
                process_duration: 0.0,
                spawn_interval: 0.0,
                inputs_needed: 1,
                behavior: StationBehavior::Palletizer,
                label: "Pallet".into(),
                color_idle: (0.8, 0.2, 0.8),
                color_busy: (0.48, 0.12, 0.48),
                color_ready: (1.0, 0.4, 1.0),
            }),
            (StationKind::Table, StationDef {
                accepted_kind: ItemKind::DoughBatch,
                output_kind: ItemKind::DoughBatch,
                process_duration: 0.0,
                spawn_interval: 0.0,
                inputs_needed: 1,
                behavior: StationBehavior::Table,
                label: "Table".into(),
                color_idle: (0.6, 0.4, 0.2),
                color_busy: (0.6, 0.4, 0.2),
                color_ready: (0.6, 0.4, 0.2),
            }),
        ]))
    }
}
