use bevy::prelude::*;
use systems::*;
use pathfinder::*;

use crate::systems::GameState;

pub mod components;
mod pathfinder;
pub mod systems;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app
        // .add_systems(Startup, (spawn_civilian, spawn_hunter))
        .add_event::<Win>()
        .add_systems(Update, (manage_civilians, manage_hunters, manage_projectiles,
            process_collisions, entity_spawner, victory).run_if(in_state(GameState::InGame)))
        ;
    }
}