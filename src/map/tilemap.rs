use bevy::{math::{ivec2, uvec2, vec2, vec3}, prelude::*, utils::{HashMap, HashSet}};
use bevy_ecs_ldtk::prelude::*;
use bevy_light_2d::light::PointLight2d;
use bevy_rapier2d::{dynamics::RigidBody, geometry::{Collider, Friction}, prelude::{CollisionGroups, Group}};
use noise::{core::perlin, NoiseFn, Perlin};
use rand::Rng;
use bevy_easings::*;

use crate::{core::{camera::plugin::{CameraController, MainCamera}, functions::TextureAtlasLayoutHandles, post_processing::PostProcessUniform}, player::{components::Player, systems::{RAYCASTABLE_STRUCT_CG, STRUCTURES_CG}}, stuff::fire_bundle, DayCycle};

#[derive(Component)]
pub struct Structure;

pub fn pre_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
){
    let ldtk_handle = asset_server.load("map/larger_map.ldtk");
    commands.spawn(LdtkWorldBundle {
        ldtk_handle,
        transform: Transform::from_translation(Vec3::Z * -11.),
        ..Default::default()
    });
    commands.insert_resource(TransformToGrid{
        height: 0.,
        transform: vec2(0., 0.),
        cell_size: vec2(16., 16.),
        ready: false,
        grid_size: ivec2(0, 0)
    });
}

pub fn watcher (
    mut commands: Commands,
    res: Res<TransformToGrid>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    levels: Query<(&LevelIid, &GlobalTransform)>,
){
    if res.ready {return;}
    for (level_iid, level_transform) in levels.iter() {
        let ldtk_project = ldtk_project_assets
            .get(ldtk_projects.single())
            .expect("ldtk project should be loaded before player is spawned");
        let level = ldtk_project
            .get_raw_level_by_iid(level_iid.get())
            .expect("level should exist in only project");
        commands.insert_resource(TransformToGrid{
            height: level.px_hei as f32,
            transform: level_transform.translation().xy(),
            cell_size: vec2(16., 16.),
            ready: true,
            grid_size: ivec2(level.px_wid / 16, level.px_hei / 16)
        });
        return;
    }
}

#[derive(Resource)]
pub struct TransformToGrid{
    height: f32,
    transform: Vec2,
    cell_size: Vec2,
    pub grid_size: IVec2,
    pub ready: bool
}

impl TransformToGrid{
    pub fn from_world(&self, position: Vec2) -> Vec2{
        ((vec2(0., self.height) + self.transform) - position) / self.cell_size * vec2(-1., 1.)
    }
    pub fn from_world_i32(&self, position: Vec2) -> IVec2{
        (((vec2(0., self.height) + self.transform) - position) / self.cell_size * vec2(-1., 1.)).floor().as_ivec2()
    }
    pub fn to_world(&self, position: IVec2) -> Vec2{
        (vec2(0., self.height) + self.transform) - position.as_vec2() * self.cell_size * vec2(-1., 1.) + self.cell_size * vec2(0.5, -0.5)
    } 
}


#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct TileObsticle;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct TileObsticleBundle {
    obsticle: TileObsticle,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct RaycastableTileObsticle;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct RaycastableTileObsticleBundle {
    obsticle: RaycastableTileObsticle,
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct TiledTreeBundle {
    obsticle: TileObsticle,
    tree: AnimatedTree
}

#[derive(Copy, Clone, PartialEq, Debug, Default, Component)]
pub struct AnimatedTree;

#[derive(Component)]
pub struct AnimatedTreePart(u8);




#[derive(Component, Default)]
pub struct LightEmitterPY;
#[derive(Bundle, Default, LdtkIntCell)]
pub struct LightEmitterPYBundle{b: LightEmitterPY}
#[derive(Component, Default)]
pub struct LightEmitterPYNX;
#[derive(Bundle, Default, LdtkIntCell)]
pub struct LightEmitterPYNXBundle{b: LightEmitterPYNX}
#[derive(Component, Default)]
pub struct LightEmitterPYPX;
#[derive(Bundle, Default, LdtkIntCell)]
pub struct LightEmitterPYPXBundle{b: LightEmitterPYPX}
#[derive(Component, Default)]
pub struct LightEmitterNY;
#[derive(Bundle, Default, LdtkIntCell)]
pub struct LightEmitterNYBundle{b: LightEmitterNY}

#[derive(Component)]
pub struct LightEmitter;

#[derive(Component, Default)]
pub struct LightEmitterAlwaysCampfire;

#[derive(Bundle, Default, LdtkIntCell)]
pub struct LightEmitterAlwaysCampfireBundle{b: LightEmitterAlwaysCampfire}


#[derive(Component, Default)]
pub struct LightEmitterAlwaysTorch;

#[derive(Bundle, Default, LdtkIntCell)]
pub struct LightEmitterAlwaysTorchBundle{b: LightEmitterAlwaysTorch}

#[derive(Component)]
pub struct LightEmitterCyclic;

#[derive(Component)]
pub struct NightWindow;

pub fn update_emitter_tiles(
    mut commands: Commands,
    new_py: Query<(Entity,&TileMetadata), Added<LightEmitterPY>>,
    new_pynx: Query<(Entity,&TileMetadata), Added<LightEmitterPYNX>>,
    new_pypx: Query<(Entity,&TileMetadata), Added<LightEmitterPYPX>>,
    new_ny: Query<(Entity,&TileMetadata), Added<LightEmitterNY>>,

    mut overlay: Query<&mut Visibility, With<NightWindow>>,

    mut new_torch: Query<Entity, Added<LightEmitterAlwaysTorch>>,
    mut new_camp: Query<Entity, Added<LightEmitterAlwaysCampfire>>,
    
    mut handles: ResMut<TextureAtlasLayoutHandles>,

    mut emitters: Query<&mut PointLight2d>,
    
    post_process: Query<&PostProcessUniform>,
    daycycle: Res<DayCycle>,
    asset_server: Res<AssetServer>,
    time: Res<Time<Virtual>>
){
    let max_intensity = 0.8;
    let default_radius = 150.;
    let default_falloff = 0.;
    let color = Color::srgb(0.9, 0.791, 0.396);
    let mut spawn_light_bundle = |c: &mut ChildBuilder, offset: Vec3, n: String|{
        let v = n.parse::<usize>().unwrap();
        c.spawn((
            bevy_light_2d::light::PointLight2dBundle{
                point_light: PointLight2d{
                    color: color,
                    intensity: max_intensity,
                    radius: default_radius,
                    falloff: default_falloff,
                },
                transform: Transform::from_translation(offset + vec3(0., 0., 1.)),
                ..Default::default()
            },
            LightEmitter
        ));
        c.spawn((
            SpriteBundle{
                texture: asset_server.load("map/light_emitters.png"),
                transform: Transform::from_translation(vec3(0., 0., 0.5)),
                ..default()
            },
            TextureAtlas{
                layout : handles.add_or_load(&asset_server, "Emitters", TextureAtlasLayout::from_grid(uvec2(16, 16), 7, 4, None, None)),
                index : v + 7
            },
            NightWindow
        ));
    };
    for b in new_py.iter() {commands.entity(b.0).with_children(|c|{spawn_light_bundle(c, vec3(0., 4., 0.), b.1.data.clone())});}
    for b in new_pynx.iter() {commands.entity(b.0).with_children(|c|{spawn_light_bundle(c, vec3(-4., 4., 0.), b.1.data.clone())});}
    for b in new_pypx.iter() {commands.entity(b.0).with_children(|c|{spawn_light_bundle(c, vec3(4., 4., 0.), b.1.data.clone())});}
    for b in new_ny.iter() {commands.entity(b.0).with_children(|c|{spawn_light_bundle(c, vec3(0., 0., 0.), b.1.data.clone())});}

    for e in new_camp.iter_mut() {
        commands.entity(e).with_children(|c|{
            c.spawn((
                bevy_light_2d::light::PointLight2dBundle{
                    point_light: PointLight2d{
                        color: Color::srgb(0.8, 0.3, 0.),
                        intensity: max_intensity,
                        radius: default_radius,
                        falloff: default_falloff,
                    },
                    ..Default::default()
                },
                LightEmitter,
            )).insert(
                fire_bundle(&asset_server, &mut handles, rand::thread_rng().gen_range(0..19))
            ).insert(Transform::from_translation(vec3(0., 10., 1.)));
        });
        commands.entity(e).insert(Transform::from_xyz(0., 0., 0.));
    }

    for e in new_camp.iter_mut() {
        commands.entity(e).with_children(|c|{
            c.spawn(
                SpriteBundle{
                    transform: Transform::from_xyz(0., 0., -10.),
                    texture: asset_server.load("map/campfire.png"),
                    ..default()
                }
            );
            c.spawn((
                bevy_light_2d::light::PointLight2dBundle{
                    point_light: PointLight2d{
                        color: Color::srgb(0.8, 0.3, 0.),
                        intensity: max_intensity,
                        radius: default_radius,
                        falloff: default_falloff,
                    },
                    ..Default::default()
                },
                LightEmitter,
            )).insert(
                fire_bundle(&asset_server, &mut handles, rand::thread_rng().gen_range(0..19))
            ).insert(Transform::from_translation(vec3(0., 10., 1.)));
        });
        commands.entity(e).insert(Transform::from_xyz(0., 0., 0.));
    }
    for e in new_torch.iter_mut() {
        commands.entity(e).with_children(|c|{
            c.spawn(
                SpriteBundle{
                    transform: Transform::from_xyz(0., 0., -10.),
                    texture: asset_server.load("map/torch.png"),
                    ..default()
                }
            );
            c.spawn((
                bevy_light_2d::light::PointLight2dBundle{
                    point_light: PointLight2d{
                        color: Color::srgb(0.8, 0.3, 0.),
                        intensity: max_intensity,
                        radius: default_radius,
                        falloff: default_falloff,
                    },
                    ..Default::default()
                },
                LightEmitter,
            )).insert(
                fire_bundle(&asset_server, &mut handles, rand::thread_rng().gen_range(0..19))
            ).insert(Transform::from_translation(vec3(0., 12., -9.)));
        });
    }

    let p = post_process.single();
    for mut l in emitters.iter_mut(){

        l.intensity = p.daytime.powi(2) * max_intensity;
        
        if (time.elapsed_seconds() * 16.).round() as usize % 2 == 0 {
            l.radius = default_radius + rand::thread_rng().gen_range(0..100) as f32 * 0.01 * 0.4 * default_radius;
        }
    }
    if daycycle.is_night || daycycle.is_translating{
        for mut v in overlay.iter_mut(){*v = Visibility::Inherited}
    } else {
        for mut v in overlay.iter_mut(){*v = Visibility::Hidden}
    }
}


pub fn update_animated_trees(
    mut tree_q: Query<(&GlobalTransform, &mut Transform, &AnimatedTreePart, &mut Sprite), Without<Player>>,
    time: Res<Time<Virtual>>,
    mut perlin: Local<Option<Perlin>>,
    mut is_static: Local<bool>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, (With<Player>, Without<AnimatedTree>)>
){
    if keyboard.just_pressed(KeyCode::F4){
        *is_static = !*is_static;
        if *is_static {
            tree_q.par_iter_mut().for_each(|(_, mut transform, tree, _)|{
                if tree.0 == 0 {return;}
                transform.rotation = Quat::from_rotation_z(0.);
            });
        }
    }
    if *is_static {return;}

    if perlin.is_none(){*perlin = Some(Perlin::new(rand::thread_rng().gen::<u32>()))}
    
    let pos = player.get_single();
    let pos = if let Ok(pos) = pos {pos.translation.xy()} else {Vec2::ZERO};
    
    tree_q.par_iter_mut().for_each(|(glob, mut transform, tree,  mut s)|{
        if tree.0 == 0 {return;}
        if pos.distance_squared(glob.translation().xy()) > (400.0f32).powi(2) {return;}

        let offset = vec3(0., -match tree.0 {
            1 => 6.,
            2 => 16.,
            _ => 26.
        }, 0.);

        let p = perlin.unwrap().get([
            (offset.x + glob.translation().x) as f64 * 0.002 + time.elapsed_seconds_f64() * 0.8,// + time.elapsed_seconds_f64() * 0.1,
            (offset.y + glob.translation().y) as f64 * 0.01 + time.elapsed_seconds_f64() * 0.5,
            time.elapsed_seconds_f64() * 1.,
        ]) as f32;
        let perlin_inf = p * 0.1 + 0.9;
        s.color = Color::srgb(perlin_inf, perlin_inf, perlin_inf);
        let angle = p * 0.15 + 0.1;
        transform.rotation = Quat::from_rotation_z(angle);
    });
}
pub fn spawn_tile_tree(
    mut commands: Commands,
    tree_q: Query<Entity, Added<AnimatedTree>>,
    asset_server: Res<AssetServer>,
    mut layout_handles: ResMut<TextureAtlasLayoutHandles>
) {
    if !tree_q.is_empty() {
        let mut r = rand::thread_rng();
        for new_tree in tree_q.iter(){
            commands.entity(new_tree).with_children(|cmd|{
                cmd.spawn((
                    SpriteBundle{
                        texture: asset_server.load("map/trees.png"),
                        ..default()
                    },
                    TextureAtlas{
                        layout: layout_handles.add_or_load(&asset_server, "Tree", TextureAtlasLayout::from_grid(uvec2(28, 17), 3, 4, Some(uvec2(1, 1)), None)),
                        index: r.gen_range(1..3) + 9
                    },
                    AnimatedTreePart(0)
                )).insert(Transform::from_xyz(0., -3., 6.).with_rotation(Quat::from_rotation_x(0.3))).with_children(|cmd|{
                    cmd.spawn((
                        AnimatedTreePart(1),
                        SpriteBundle{
                            texture: asset_server.load("map/trees.png"),
                            ..default()
                        },
                        TextureAtlas{
                            layout: layout_handles.add_or_load(&asset_server, "Tree", TextureAtlasLayout::from_grid(uvec2(28, 17), 3, 4, Some(uvec2(1, 1)), None)),
                            index: r.gen_range(1..3) + 6
                        },
                    )).insert(Transform::from_xyz(0., 6., 0.).with_rotation(Quat::from_rotation_y(0.))).with_children(|cmd|{
                        cmd.spawn((
                            AnimatedTreePart(2),
                            SpriteBundle{
                                texture: asset_server.load("map/trees.png"),
                                ..default()
                            },
                            TextureAtlas{
                                layout: layout_handles.add_or_load(&asset_server, "Tree", TextureAtlasLayout::from_grid(uvec2(28, 17), 3, 4, Some(uvec2(1, 1)), None)),
                                index: r.gen_range(1..3) + 3
                            },
                        )).insert(Transform::from_xyz(0., 10., 0.).with_rotation(Quat::from_rotation_y(0.))).with_children(|cmd|{
                            cmd.spawn((
                                AnimatedTreePart(3),
                                SpriteBundle{
                                    texture: asset_server.load("map/trees.png"),
                                    ..default()
                                },
                                TextureAtlas{
                                    layout: layout_handles.add_or_load(&asset_server, "Tree", TextureAtlasLayout::from_grid(uvec2(28, 17), 3, 4, Some(uvec2(1, 1)), None)),
                                    index: r.gen_range(1..3)
                                },
                            )).insert(Transform::from_xyz(0., 10., 0.1).with_rotation(Quat::from_rotation_y(0.05)));
                        });
                    });
                });
            });
        }
        
        /*level_query.iter().for_each(|(level_entity, level_iid)| {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_projects.single())
                .expect("Project should be loaded if level has spawned");

            let level = ldtk_project
                .as_standalone()
                .get_loaded_level_by_iid(&level_iid.to_string())
                .expect("Spawned level should exist in LDtk project");

            let LayerInstance {
                c_wid: width,
                c_hei: height,
                grid_size,
                ..
            } = level.layer_instances()[0];
        });*/
    }
}

pub fn setup_camera_bounds(
    mut commands: Commands,
    mut cameras_q: Query<&mut CameraController>,
    level_query: Query<(&Transform, &LevelIid)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    level_selection: Res<LevelSelection>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    mut done: Local<bool>
) {
    if *done {return}
    for (level_transform, level_iid) in &level_query {
        let ldtk_project = ldtk_project_assets
            .get(ldtk_projects.single())
            .expect("Project should be loaded if level has spawned");

        let level = ldtk_project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("Spawned level should exist in LDtk project");

        if level_selection.is_match(&LevelIndices::default(), level) {
            for mut cam in cameras_q.iter_mut(){
                cam.corners = Some((
                    level_transform.translation.xy(),
                    level_transform.translation.xy() + vec2(level.px_wid as f32, level.px_hei as f32)
                ));
                *done = true;
            }
            
            /*let level_ratio = level.px_wid as f32 / level.px_hei as f32;
            orthographic_projection.viewport_origin = Vec2::ZERO;
            if level_ratio > ASPECT_RATIO {
                // level is wider than the screen
                let height = (level.px_hei as f32 / 9.).round() * 9.;
                let width = height * ASPECT_RATIO;
                orthographic_projection.scaling_mode =
                    bevy::render::camera::ScalingMode::Fixed { width, height };
                camera_transform.translation.x =
                    (player_translation.x - level_transform.translation.x - width / 2.)
                        .clamp(0., level.px_wid as f32 - width);
                camera_transform.translation.y = 0.;
            } else {
                // level is taller than the screen
                let width = (level.px_wid as f32 / 16.).round() * 16.;
                let height = width / ASPECT_RATIO;
                orthographic_projection.scaling_mode =
                    bevy::render::camera::ScalingMode::Fixed { width, height };
                camera_transform.translation.y =
                    (player_translation.y - level_transform.translation.y - height / 2.)
                        .clamp(0., level.px_hei as f32 - height);
                camera_transform.translation.x = 0.;
            }

            camera_transform.translation.x += level_transform.translation.x;
            camera_transform.translation.y += level_transform.translation.y;*/
        }
    }
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_tile_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<TileObsticle>>,
    parent_query: Query<&Parent, Without<TileObsticle>>,
    level_query: Query<(Entity, &LevelIid)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.iter().for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.iter().for_each(|(level_entity, level_iid)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let ldtk_project = ldtk_project_assets
                    .get(ldtk_projects.single())
                    .expect("Project should be loaded if level has spawned");

                let level = ldtk_project
                    .as_standalone()
                    .get_loaded_level_by_iid(&level_iid.to_string())
                    .expect("Spawned level should exist in LDtk project");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level.layer_instances()[0];
                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert((
                                RigidBody::Fixed,
                                Structure,
                                CollisionGroups::new(
                                    Group::from_bits(STRUCTURES_CG).unwrap(),
                                    Group::ALL,
                                ),
                            ))
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(Name::new("COLLIDER"))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}

#[derive(Component)]
pub struct RaycastableHelp;

pub fn spawn_raycastable_tile_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<RaycastableTileObsticle>>,
    parent_query: Query<&Parent, Without<RaycastableTileObsticle>>,
    level_query: Query<(Entity, &LevelIid)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.iter().for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.iter().for_each(|(level_entity, level_iid)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let ldtk_project = ldtk_project_assets
                    .get(ldtk_projects.single())
                    .expect("Project should be loaded if level has spawned");

                let level = ldtk_project
                    .as_standalone()
                    .get_loaded_level_by_iid(&level_iid.to_string())
                    .expect("Spawned level should exist in LDtk project");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level.layer_instances()[0];
                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert((
                                RigidBody::Fixed,
                                RaycastableHelp,
                                CollisionGroups::new(
                                    Group::from_bits(RAYCASTABLE_STRUCT_CG).unwrap(),
                                    Group::ALL,
                                ),
                            ))
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(Name::new("COLLIDER"))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}
























