#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod screen;
mod ui;

pub mod core;
pub mod player;
pub mod npc;
pub mod map;
pub mod systems;
pub mod stuff;
pub mod characters;
pub mod sounds;

use bevy::{prelude::*};

use characters::plugin::CharacterAnimationPlugin;
use map::plugin::TileMapPlugin;
use npc::systems::RosesCollected;
use npc::NPCPlugin;
use player::PlayerPlugin;
use sounds::AudioPlugin;
use stuff::{simple_anim_update, spawn_follow_blood_particle, update_blood_particles};
use systems::*;

pub struct AppPlugin;

const NUM_ROSES: u32 = 3;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );
        app
    .add_plugins((
        core::default::plugin::DefaultPlugin,
        //SwitchableEguiInspectorPlugin,
        //ScreenDiagnosticsPlugin,
        TileMapPlugin,
    ))
    .insert_state(GameState::InGame)
    .insert_resource(DayCycle {
        is_night: true,
        is_translating: false,
    })
    .insert_resource(RosesCollected {
        collected: 0,
        max: NUM_ROSES,
    })
    .add_event::<PauseEvent>()
    .add_plugins((
        PlayerPlugin,
        NPCPlugin,
        CharacterAnimationPlugin,
        AudioPlugin,
    ))
    .add_systems(Startup, spawn_starter_screen)
    .add_systems(Update, interact_start_button)
    .add_systems(Update, (
        (update_daycycle, update_score).run_if(in_state(GameState::InGame)), 
        simple_anim_update.run_if(in_state(GameState::InGame)),
        update_blood_particles.run_if(in_state(GameState::InGame)),
        pause_game
    ));
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2dBundle::default(),
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
    ));
}
