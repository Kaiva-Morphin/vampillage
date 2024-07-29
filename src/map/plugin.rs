use std::time::Duration;

use bevy::{math::ivec2, prelude::*, transform::commands, utils::HashSet};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::{ActiveEvents, Collider, RigidBody, Sensor, Velocity};
use bevy_light_2d::prelude::Light2dPlugin;
use crate::player::components::Player;

use super::tilemap::{self, setup_camera_bounds, update_emitter_tiles, RaycastableTileObsticle, TileObsticle, TransformToGrid};

pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LdtkPlugin,
            Light2dPlugin,
        ));
        app.insert_resource(LevelSelection::Uid(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: false,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        });
        app.add_event::<RespawnRosesEvent>();
        app.add_systems(PreStartup, tilemap::pre_setup);
        app.add_systems(Update, (tilemap::watcher, spawn_collectables, respawn_collectables));
        app.add_systems(Update, (tilemap::spawn_tile_collision, update_emitter_tiles, setup_camera_bounds, update_unit_grid, tilemap::spawn_tile_tree, tilemap::spawn_raycastable_tile_collision, tilemap::update_animated_trees));
        app.add_systems(PreUpdate, trespassable_spawn_listener);
        app.register_ldtk_entity::<HunterSpawnerBundle>("HunterSpawner");
        app.register_ldtk_entity::<CivilianSpawnerBundle>("CivilianSpawner");
        app.register_ldtk_entity::<CollectableRoseBundle>("Rose");
        app.register_ldtk_int_cell_for_layer::<tilemap::RaycastableTileObsticleBundle>("Ground", 1);
        app.register_ldtk_int_cell_for_layer::<tilemap::TiledTreeBundle>("Ground", 3);
        app.register_ldtk_int_cell_for_layer::<tilemap::TileObsticleBundle>("Ground", 4);
        app.register_ldtk_int_cell_for_layer::<tilemap::RaycastableTileObsticleBundle>("Ground", 5);

        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterPYBundle>("Emitters", 1);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterPYNXBundle>("Emitters", 2);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterPYPXBundle>("Emitters", 3);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterPYBundle>("Emitters", 4);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterPYBundle>("Emitters", 5);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterPYBundle>("Emitters", 6);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterPYBundle>("Emitters", 7);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterNYBundle>("Emitters", 8);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterNYBundle>("Emitters", 9);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterNYBundle>("Emitters", 10);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterNYBundle>("Emitters", 11 );
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterAlwaysTorchBundle>("Emitters", 12);
        app.register_ldtk_int_cell_for_layer::<tilemap::LightEmitterAlwaysCampfireBundle>("Emitters", 13);

        app.insert_resource(TrespassableCells::default());
    }
}


#[derive(Component, Default)]
pub struct CollectableRose;

#[derive(Component, Default)]
pub struct CollectableRoseSpawner;

#[derive(Bundle, Default, LdtkEntity)]
pub struct CollectableRoseBundle{
    rose: CollectableRoseSpawner
}

#[derive(Event)]
pub struct RespawnRosesEvent;

pub fn respawn_collectables(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    spawners: Query<Entity, With<CollectableRoseSpawner>>,
    roses: Query<Entity, With<CollectableRose>>,
    mut event: EventReader<RespawnRosesEvent>,
){
    for _ in event.read() {
        for rose in roses.iter(){commands.entity(rose).despawn()};
        for e in spawners.iter(){
            commands.entity(e).with_children(|commands|{commands.spawn((
                SpriteBundle{
                    texture: asset_server.load("map/rose.png"),
                    ..default()
                },
                RigidBody::Fixed,
                Collider::ball(1.),
                ActiveEvents::COLLISION_EVENTS,
                CollectableRose,
                Sensor,
                Name::new("Rose"),
            ));});
        }
    }
}

pub fn spawn_collectables(
    to_spawn: Query<Entity, Added<CollectableRoseSpawner>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
){
    for e in to_spawn.iter(){
        commands.entity(e).with_children(|commands|{commands.spawn((
            SpriteBundle{
                texture: asset_server.load("map/rose.png"),
                ..default()
            },
            RigidBody::Fixed,
            Collider::ball(1.),
            ActiveEvents::COLLISION_EVENTS,
            CollectableRose,
            Sensor,
            Name::new("Rose"),
        ));});
    }
}




#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct TrespassableCell;


#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct TrespassableCellBundle{
    cell: TrespassableCell
}


#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct HunterSpawner {
    pub timer: Timer,
}

#[derive(Clone, Debug, Bundle, LdtkEntity)]
pub struct HunterSpawnerBundle {
    spanwer: HunterSpawner,
}

impl Default for HunterSpawnerBundle {
    fn default() -> Self {
        Self {
            spanwer: HunterSpawner { timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating) }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct CivilianSpawner {
    pub timer: Timer,
}

#[derive(Clone, Debug, Bundle, LdtkEntity)]
pub struct CivilianSpawnerBundle {
    spanwer: CivilianSpawner,
}

impl Default for CivilianSpawnerBundle {
    fn default() -> Self {
        Self {
            spanwer: CivilianSpawner { timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating) }
        }
    }
}

#[derive(Resource, Default)]
pub struct TrespassableCells{
    pub cells: Vec<Vec<bool>>,
    pub units: HashSet<IVec2>,
    pub ready: bool
}

impl TrespassableCells {
    pub fn is_trespassable(&self, pos: &IVec2) -> bool{
        let Some(column) = self.cells.get(pos.x as usize) else {return false};
        let Some(value) = column.get(pos.y as usize) else {return false};
        *value
    }
}




fn update_unit_grid(
    mut trespassable: ResMut<TrespassableCells>,
    transfromer: Res<TransformToGrid>,
    units_q: Query<&Transform, (With<Velocity>, Without<Player>)>
){
    trespassable.units.clear();
    for t in units_q.iter(){
        let pos = transfromer.from_world_i32(t.translation.xy());
        trespassable.units.insert(pos);
    }
}

fn trespassable_spawn_listener(
    //mut commands: Commands,
    entity_q: Query<&GridCoords, Added<TileObsticle>>,
    ray_entity_q: Query<&GridCoords, Added<RaycastableTileObsticle>>,
    mut trespassable_cells: ResMut<TrespassableCells>,
    transfromer: Res<TransformToGrid>,
    //level_query: Query<(Entity, &LevelIid)>,
    //ldtk_projects: Query<&Handle<LdtkProject>>,
    //ldtk_project_assets: Res<Assets<LdtkProject>>,
){
    if !entity_q.is_empty() && transfromer.ready {
        let cells_column = vec![true; transfromer.grid_size.y as usize];
        let mut cells_grid = vec![cells_column; transfromer.grid_size.x as usize];
        
        for coords in entity_q.iter(){
            let pos = ivec2(coords.x, transfromer.grid_size.y - coords.y - 1);
            cells_grid[pos.x as usize][pos.y as usize] = false;
        }

        for coords in ray_entity_q.iter(){
            let pos = ivec2(coords.x, transfromer.grid_size.y - coords.y - 1);
            cells_grid[pos.x as usize][pos.y as usize] = false;
        }

        trespassable_cells.cells = cells_grid;
        info!("Trespassable cells inited!");
        trespassable_cells.ready = true;
    }
}