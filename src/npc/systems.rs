use std::{f32::consts::PI, time::Duration};

use bevy::{color::palettes::css::{BLUE, RED}, math::uvec2, prelude::*};
use bevy_rapier2d::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    characters::animation::*, core::functions::TextureAtlasLayoutHandles, map::{plugin::{CivilianSpawner, CollectableRose, CollectableRoseSpawner, HunterSpawner, RespawnRosesEvent, TrespassableCells}, 
    tilemap::{RaycastableHelp, Structure, TransformToGrid}}, player::{components::{HitPlayer, KillNpc, KillPlayer, Player}, systems::{PlayerController, BULLET_CG, NPC_CG, PLAYER_CG, RAYCASTABLE_STRUCT_CG, STRUCTURES_CG}}, sounds::components::PlaySoundEvent, stuff::{spawn_angry_particle, spawn_cililian_body, spawn_hunter_body, spawn_question_particle, spawn_warn_particle}, systems::DayCycle
};

use super::{components::*, pathfinder};

const SPOT_DIST: f32 = 200.0;
const SPOT_DIST_CIV: f32 = 100.0;
const THRESHOLD: f32 = 100.0;
const UPP_THRESHOLD: f32 = THRESHOLD * 2.0;
const CIV_MAXSPEED: f32 = 40.0;
const CIV_ACCEL: f32 = 350.0;
const PROJ_V: f32 = 150.0;
const HUNTER_TIMER: f32 = 0.5;
const HUNTER_MAXSPEED: f32 = 50.0;
const HUNTER_ACCEL: f32 = 450.0;

pub fn spawn_civilian(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    pos: Vec2,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
) {
    let entity = spawn_civilian_animation_bundle(&mut commands, asset_server, layout_handles);
    commands.entity(entity).insert((
        TransformBundle::from_transform(Transform::from_translation(pos.extend(-2.))),
        RigidBody::Dynamic,
        Velocity::zero(),
        Civilian,
        Sleeping::disabled(),
        LockedAxes::ROTATION_LOCKED_Z,
        Collider::ball(4.5),
        CollisionGroups::new(
            Group::from_bits(NPC_CG).unwrap(),
            Group::from_bits(PLAYER_CG | RAYCASTABLE_STRUCT_CG  | STRUCTURES_CG).unwrap()
        ),
        NpcVelAccum {v: Vec2::ZERO},
        NpcPath {path: None},
        NpcState::Chill,
        ChillTimer {timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating)},
        AttackTimer {timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating)},
        ParticleTimer {timer: Timer::new(Duration::from_secs_f32(1.), TimerMode::Repeating)},
    ));
}

pub fn manage_civilians(
    mut commands: Commands,
    mut civilians_data: Query<(&Transform, &mut Velocity, &mut NpcVelAccum, &mut NpcPath, &mut NpcState,
        &mut ChillTimer, &mut AnimationController, &mut AttackTimer, &mut ParticleTimer, Entity), With<Civilian>>,
    mut player_data: Query<(&Transform, Entity, &mut Player)>,
    time: Res<Time>,
    day_cycle: Res<DayCycle>,
    transformer: Res<TransformToGrid>,
    trespassable: Res<TrespassableCells>,
    rapier_context: Res<RapierContext>,
    mut layout_handles: ResMut<TextureAtlasLayoutHandles>,
    asset_server: Res<AssetServer>,
    mut hit_player: EventWriter<HitPlayer>,
    mut play_sound: EventWriter<PlaySoundEvent>,
) {
    if let Ok((player_transform, player_entity, mut player)) = player_data.get_single_mut() {
    if player.is_dead {return;}
    let player_pos = player_transform.translation.xy();
    let player_ipos = transformer.from_world_i32(player_pos);
    let dt = time.delta_seconds();
    let mut rng = thread_rng();
    for (civ_transform, mut civ_controller,
        mut vel_accum , mut civ_path,
        mut civ_state, mut chill_timer,
        mut animation_controller,
        mut attack_timer, mut particle_timer,
        civ_entity) in civilians_data.iter_mut() {
        let civ_pos = civ_transform.translation.xy();
        if civ_pos.distance(player_pos) > 1000. {
            continue;
        }
        let civ_ipos = transformer.from_world_i32(civ_pos);
        let direction = player_pos - civ_pos;
        let length = direction.length();
        let mut player_in_sight = false;
        if let Some(last_seen_entity) = raycast(civ_pos, direction / length, length, &rapier_context) {
        if last_seen_entity == player_entity && length < SPOT_DIST_CIV {
            player_in_sight = true;
        }
        // println!("{:?} {}", civ_state, player_in_sight);
        match *civ_state {
            NpcState::Look => {},
            NpcState::Dead => {
                attack_timer.timer.tick(Duration::from_secs_f32(dt));
                animation_controller.play_hurt();
                commands.entity(civ_entity).remove::<Collider>();
                if attack_timer.timer.finished() {
                    spawn_cililian_body(&mut commands, &mut layout_handles, &asset_server, civ_pos.extend(0.));
                    commands.entity(civ_entity).despawn_recursive();
                }
            },
            NpcState::Attack => {
                particle_timer.timer.tick(Duration::from_secs_f32(dt));
                if particle_timer.timer.finished() {
                    spawn_angry_particle(&mut commands, &mut layout_handles, &asset_server, civ_pos.extend(0.));
                }
                if attack_timer.timer.elapsed_secs() == 0. {
                    play_sound.send(PlaySoundEvent::Hit);
                    animation_controller.play_civil_attack();
                }
                attack_timer.timer.tick(Duration::from_secs_f32(dt));
                if attack_timer.timer.finished() {
                    if player_pos.distance(civ_pos) < 16. {
                        hit_player.send(HitPlayer { dmg_type: 1,});
                    }
                    *civ_state = NpcState::Chase;
                    attack_timer.timer.set_elapsed(Duration::from_secs(0))
                }
            },
            state => { // esc cha chi
                let mut stop = false;
                if state == NpcState::Chill {
                    animation_controller.disarm();
                    animation_controller.play_idle_priority(1);

                    if civ_path.path.is_none() {
                        chill_timer.timer.tick(Duration::from_secs_f32(dt));
                        if chill_timer.timer.finished() {
                            let end = civ_ipos + IVec2::new(rng.gen_range(-2..2), rng.gen_range(-2..2));
                            if trespassable.is_trespassable(&end) {
                                civ_path.path = pathfinder(civ_ipos, end, &trespassable, &transformer, state, false);
                            }
                        }
                    }
                    if player_in_sight {
                        spawn_warn_particle(&mut commands, &mut layout_handles, &asset_server, civ_pos.extend(0.));
                        if day_cycle.is_night {
                            *civ_state = NpcState::Escape;
                        } else {
                            *civ_state = NpcState::Chase;
                        }
                    }
                } else if state == NpcState::Escape {
                    animation_controller.disarm();
                    civ_path.path = pathfinder(civ_ipos, player_ipos, &trespassable, &transformer, state, false);
                    if !day_cycle.is_night {
                        if player_in_sight {
                            *civ_state = NpcState::Chase;
                        } else {
                            *civ_state = NpcState::Chill;
                        }
                    }
                } else { // chase
                    particle_timer.timer.tick(Duration::from_secs_f32(dt));
                    if particle_timer.timer.finished() {
                        spawn_angry_particle(&mut commands, &mut layout_handles, &asset_server, civ_pos.extend(0.));
                    }
                    animation_controller.arm();
                    civ_path.path = pathfinder(civ_ipos, player_ipos, &trespassable, &transformer, state, false);
                    if player_in_sight {
                        if day_cycle.is_night {
                            *civ_state = NpcState::Escape;
                        }
                    } else {
                        spawn_question_particle(&mut commands, &mut layout_handles, &asset_server, civ_pos.extend(0.));
                        *civ_state = NpcState::Chill;
                        civ_path.path = None;
                    }
                    if player_pos.distance(civ_pos) < 16. {
                        *civ_state = NpcState::Attack;
                        stop = true;
                    }
                    // println!("{:?}", civ_path.path);
                }
                
                let mut del = false;
                if let Some(path) = &mut civ_path.path {
                    if civ_ipos == path[1] {
                        path.remove(0);
                    }
                    if path.len() < 2 {
                        del = true;
                    }
                }
                if del {
                    civ_path.path = None;
                }
                
                if let Some(path) = &civ_path.path {
                    let move_dir = transformer.to_world(path[1]) - civ_pos;

                    if move_dir.x.abs() < 0.1 { // x axis is priotirized 
                        if move_dir.y.abs() > 0.1 {
                            if move_dir.y.is_sign_positive(){animation_controller.turn_up()}
                            if move_dir.y.is_sign_negative(){animation_controller.turn_down()}
                        }
                    } else {
                        if move_dir.x.is_sign_positive(){animation_controller.turn_right()}
                        if move_dir.x.is_sign_negative(){animation_controller.turn_left()}
                    }
                    if vel_accum.v.length() > 0.1 {
                        animation_controller.play_walk_unlooped();
                    } else {
                        animation_controller.play_idle_priority(1);
                    }
                    vel_accum.v = vel_accum.v.move_towards(move_dir.normalize_or_zero() * CIV_MAXSPEED, dt * CIV_ACCEL);
                    if vel_accum.v.length() > CIV_MAXSPEED {
                        vel_accum.v = vel_accum.v.normalize() * CIV_MAXSPEED
                    }
                    civ_controller.linvel = vel_accum.v;
                } else {
                    civ_controller.linvel = Vec2::ZERO;
                    if civ_pos.distance(player_pos) > THRESHOLD {
                        *civ_state = NpcState::Chill
                    } else {
                        if player_in_sight {
                            if day_cycle.is_night {
                                *civ_state = NpcState::Escape
                            } else {
                                *civ_state = NpcState::Chase
                            }
                        } else {
                            *civ_state = NpcState::Chill;
                        }
                    }
                }
                if stop {
                    civ_controller.linvel = Vec2::ZERO;
                    animation_controller.play_idle_priority(1);
                }
            }
        }
    }
    }
    }
}


pub fn spawn_hunter(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    pos: Vec2,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
) {
    let entity = spawn_hunter_animation_bundle(commands, asset_server, layout_handles);
    commands.entity(entity).insert((
        (
            Name::new("Hunter"),
            RigidBody::Dynamic,
            TransformBundle::from_transform(Transform::from_translation(pos.extend(0.))),
            VisibilityBundle::default(),
            Collider::ball(4.5),
            Sleeping::disabled(),
        ),
        Hunter,
        LockedAxes::ROTATION_LOCKED_Z,
        NpcVelAccum {v: Vec2::ZERO},
        NpcPath {path: None},
        Velocity::zero(),
        CollisionGroups::new(
            Group::from_bits(NPC_CG).unwrap(),
            Group::from_bits(PLAYER_CG | RAYCASTABLE_STRUCT_CG  | STRUCTURES_CG).unwrap(),
        ),
        HunterTimer { timer: Timer::new(Duration::from_secs_f32(HUNTER_TIMER), TimerMode::Repeating) },
        NpcState::Chill,
        ChillTimer {timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating)},
        PlayerLastPos {pos: IVec2::ZERO},
    ));
}

pub fn manage_hunters(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut hunters_data: Query<(&Transform, &mut Velocity,
        &mut NpcVelAccum, &mut NpcPath, &mut HunterTimer, &mut NpcState,
        &mut ChillTimer, &mut AnimationController, &mut PlayerLastPos, Entity), Without<Player>>,
    player_data: Query<(&Transform, &PlayerController, Entity, &Player)>,
    transformer: Res<TransformToGrid>,
    trespassable: Res<TrespassableCells>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
    mut atlas_handles: ResMut<TextureAtlasLayoutHandles>,
    mut play_sound: EventWriter<PlaySoundEvent>,
) {
    if let Ok(player_data) = player_data.get_single() {
    if player_data.3.is_dead {return;}
    let player_pos = player_data.0.translation.xy();
    let player_ipos = transformer.from_world_i32(player_pos);
    let player_vel = player_data.1.accumulated_velocity;
    let player_entity = player_data.2;
    let dt = time.delta_seconds();
    for (hunter_transform, mut hunter_controller,
        mut vel_accum , mut hunter_path,
        mut hunter_timer, mut hunter_state, mut chill_timer,
        mut animation_controller, mut player_last_pos,
        hunter_entity) in hunters_data.iter_mut() {
        hunter_controller.linvel = Vec2::ZERO;
        let hunter_pos = hunter_transform.translation.xy();
        let hunter_ipos = transformer.from_world_i32(hunter_pos);
        if hunter_pos.distance(player_pos) > 1000. {
            continue;
        }
        let direction = player_pos - hunter_pos;
        let length = direction.length();
        let mut player_in_sight = false;
        if let Some(last_seen_entity) = raycast(hunter_pos, direction / length, length, &rapier_context) {
        if last_seen_entity == player_entity && length < SPOT_DIST{
            player_in_sight = true;
        }

        match *hunter_state {
            NpcState::Attack => {
                hunter_timer.timer.tick(Duration::from_secs_f32(dt));
                let dir = player_pos - hunter_pos;
                if dir.x.abs() > dir.y.abs() {
                    if dir.x > 0. {
                        animation_controller.turn_right()
                    } else {
                        animation_controller.turn_left()
                    }
                } else {
                    if dir.y > 0. {
                        animation_controller.turn_up()
                    } else {
                        animation_controller.turn_down()
                    }
                }
                if player_in_sight {
                if hunter_timer.timer.finished() {
                animation_controller.play_hunter_throw();
                play_sound.send(PlaySoundEvent::Throw);
                if let Some(intercept) = calculate_intercept(hunter_pos, player_pos, player_vel, PROJ_V) {
                    let dir = intercept - hunter_pos;
                    let dir = dir / dir.length();
                    let throwable_variant = rand::thread_rng().gen_range(0..4);
                    


                    commands.spawn((TransformBundle::default(), VisibilityBundle::default())).insert((
                        Transform::from_translation(hunter_pos.extend(0.)),
                        RigidBody::Dynamic,
                        Collider::cuboid(3., 3.),
                        CollisionGroups::new(
                            Group::from_bits(BULLET_CG).unwrap(),
                            Group::from_bits(PLAYER_CG | STRUCTURES_CG).unwrap()
                        ),
                        LockedAxes::ROTATION_LOCKED_Z,
                        Velocity {
                            linvel: PROJ_V * dir,
                            angvel: 0.0,
                        },
                        DespawnTimer { timer: Timer::new(Duration::from_secs(6), TimerMode::Once) },
                        Projectile,
                        Sensor,
                        ActiveEvents::COLLISION_EVENTS,
                        Sleeping::disabled(),
                    )).with_children(|commands|{
                        match throwable_variant {
                            0 => {commands.spawn(crate::stuff::animated_fork_bundle(&asset_server, &mut atlas_handles));},
                            1 => {commands.spawn(crate::stuff::animated_knife_bundle(&asset_server, &mut atlas_handles));},
                            2 => {commands.spawn(crate::stuff::animated_garlic_bundle(&asset_server, &mut atlas_handles));},
                            _ => {commands.spawn(crate::stuff::stake_bundle(&asset_server, &mut atlas_handles, dir)).insert(
                                Transform::from_rotation(Quat::from_rotation_z(if throwable_variant != 3 {0.} else {dir.to_angle() + PI * 0.75}),
                            ));},
                        };
                        commands.spawn(
                            SpriteBundle{
                                transform: Transform::from_xyz(0.,-6., 0.),
                                texture: asset_server.load("particles/minishadow.png"),
                                ..default()
                            }
                        );
                    });
                }
            }
            let dist = player_pos.distance(hunter_pos);
            if dist < THRESHOLD {
                *hunter_state = NpcState::Escape;
            } else if dist < UPP_THRESHOLD {
                *hunter_state = NpcState::Chase;
            }
            } else {
            *hunter_state = NpcState::Look;
            player_last_pos.pos = player_ipos;
            }
            }
            NpcState::Dead => {
                hunter_timer.timer.tick(Duration::from_secs_f32(dt));
                animation_controller.play_hurt();
                commands.entity(hunter_entity).remove::<Collider>();
                if hunter_timer.timer.finished() {
                    spawn_hunter_body(&mut commands, &mut atlas_handles, &asset_server, hunter_pos.extend(0.));
                    commands.entity(hunter_entity).despawn_recursive();
                }
            }
            state => {
                if state == NpcState::Chill {
                    animation_controller.play_idle_priority(1);
                    if player_in_sight {
                        *hunter_state = NpcState::Chase;
                        spawn_warn_particle(&mut commands, &mut atlas_handles, &asset_server, hunter_pos.extend(0.));
                    } else {
                        if hunter_path.path.is_none() {
                            let mut rng = thread_rng();
                            chill_timer.timer.tick(Duration::from_secs_f32(dt));
                            if chill_timer.timer.finished() {
                                let end = hunter_ipos + IVec2::new(rng.gen_range(-2..2), rng.gen_range(-2..2));
                                if trespassable.is_trespassable(&end) {
                                    hunter_path.path = pathfinder(hunter_ipos, end, &trespassable, &transformer, state, true);
                                }
                            }
                        }
                    }
                } else if state == NpcState::Look {
                    if player_in_sight {
                        *hunter_state = NpcState::Chase;
                        spawn_warn_particle(&mut commands, &mut atlas_handles, &asset_server, hunter_pos.extend(0.));
                    } else {
                        hunter_path.path = pathfinder(hunter_ipos, player_last_pos.pos, &trespassable, &transformer, state, true);
                        if hunter_path.path.is_none() {
                            spawn_question_particle(&mut commands, &mut atlas_handles, &asset_server, hunter_pos.extend(0.));
                            *hunter_state = NpcState::Chill;
                        }
                    }
                    
                } else { // chase & escape
                    if player_in_sight {
                        hunter_path.path = pathfinder(hunter_ipos, player_ipos, &trespassable, &transformer, state, true);
                        if hunter_path.path.is_none() {
                        *hunter_state = NpcState::Attack;
                        }
                    } else {
                        *hunter_state = NpcState::Look;
                        player_last_pos.pos = player_ipos;
                    }
                }
                
                let mut del = false;
                if let Some(path) = &mut hunter_path.path {
                    if hunter_ipos == path[1] {
                        path.remove(0);
                    }
                    if path.len() < 2 {
                        del = true;
                    }
                }
                if del {
                    hunter_path.path = None;
                }
                
                if let Some(path) = &hunter_path.path {
                    let move_dir = transformer.to_world(path[1]) - hunter_pos;

                    if move_dir.x.abs() < 0.1 { // x axis is priotirized 
                        if move_dir.y.abs() > 0.1 {
                            if move_dir.y.is_sign_positive(){animation_controller.turn_up()}
                            if move_dir.y.is_sign_negative(){animation_controller.turn_down()}
                        }
                    } else {
                        if move_dir.x.is_sign_positive(){animation_controller.turn_right()}
                        if move_dir.x.is_sign_negative(){animation_controller.turn_left()}
                    }
                    if vel_accum.v.length() > 0.1 {
                        animation_controller.play_walk_unlooped();
                    } else {
                        animation_controller.play_idle_priority(1);
                    }
                    
                    vel_accum.v = vel_accum.v.move_towards(move_dir.normalize_or_zero() * HUNTER_MAXSPEED, dt * HUNTER_ACCEL);
                    if vel_accum.v.length() > HUNTER_MAXSPEED {
                        vel_accum.v = vel_accum.v.normalize() * HUNTER_MAXSPEED
                    }
                    hunter_controller.linvel = vel_accum.v;
                }
            }
        }
    }
    }
    }
}

fn calculate_intercept(shooter_pos: Vec2, target_pos: Vec2, target_vel: Vec2, proj_vel: f32) -> Option<Vec2> {
    let direction = target_pos - shooter_pos;
    let a = target_vel.dot(target_vel) - proj_vel * proj_vel;
    let b = 2. * direction.dot(target_vel);
    let c = direction.dot(direction);
    let dis = b * b - 4. * a * c;
    if dis < 0. {
        return None;
    }
    let t = (-b - dis.sqrt()) / (2. * a);
    if t < 0. {
        return None;
    }
    let i = target_pos + target_vel * t;
    return Some(i);
}

pub fn manage_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(&mut DespawnTimer, Entity), With<Projectile>>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (mut timer, entity) in projectiles.iter_mut() {
        timer.timer.tick(Duration::from_secs_f32(delta));
        if timer.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Resource)]
pub struct RosesCollected {
    pub collected: u32,
    pub max: u32,
}

pub fn process_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player: Query<(Entity, &Player)>,
    mut hunters: Query<&mut NpcState, (With<Hunter>, Without<Civilian>)>,
    mut civilians: Query<&mut NpcState, With<Civilian>>,
    projectiles: Query<&Projectile>,
    structures: Query<&Structure>,
    help: Query<&RaycastableHelp>,
    roses: Query<Entity, With<CollectableRose>>,
    mut roses_collected: ResMut<RosesCollected>,
    day_cycle: Res<DayCycle>,
    mut hit_player: EventWriter<HitPlayer>,
    mut kill_npc: EventWriter<KillNpc>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    mut win: EventWriter<Win>
) {
    if let Ok((player_entity, player)) = player.get_single_mut() {
        for collision_event in collision_events.read() {
            if let CollisionEvent::Started(reciever_entity, sender_entity, _) = collision_event {
                // player appears to always be reciever
                let sender_entity = *sender_entity;
                if let Ok(_) = projectiles.get(sender_entity) {
                    if *reciever_entity == player_entity {
                        hit_player.send(HitPlayer { dmg_type: 0});
                    }
                    commands.entity(sender_entity).despawn_recursive();
                } else if let Ok(mut state) = civilians.get_mut(sender_entity) {
                    if day_cycle.is_night {
                        // kill civilian
                        *state = NpcState::Dead;
                        kill_npc.send(KillNpc { npc_type: 0 });
                        play_sound.send(PlaySoundEvent::Kill);
                    }
                } else if let Ok(mut state) = hunters.get_mut(sender_entity) {
                    if day_cycle.is_night {
                        // kill hunter
                        *state = NpcState::Dead;
                        kill_npc.send(KillNpc { npc_type: 1 });
                        play_sound.send(PlaySoundEvent::Kill);
                    } else {
                        if *reciever_entity == player_entity {
                            hit_player.send(HitPlayer { dmg_type: 2});
                        }
                    }
                } else if let Ok(_) = structures.get(sender_entity) {
                    commands.entity(player_entity).remove::<Sensor>();
                } else if let Ok(_) = help.get(sender_entity) {
                    commands.entity(player_entity).remove::<Sensor>();
                } else if let Ok(rose_entity) = roses.get(sender_entity) {
                    if player.is_dead {return;}
                    commands.entity(rose_entity).despawn_recursive();
                    roses_collected.collected += 1;
                    if roses_collected.max == roses_collected.collected {
                        win.send(Win);
                    }
                }
            }
        }
    }
}

#[derive(Event)]
pub struct Win;

pub fn victory(
    mut win: EventReader<Win>,
    mut kill_player: EventWriter<KillPlayer>,
    mut roses_collected: ResMut<RosesCollected>,
    mut event: EventWriter<RespawnRosesEvent>,
) {
    for _ in win.read() {
        kill_player.send(KillPlayer { won: true });
        roses_collected.collected = 0;
        event.send(RespawnRosesEvent);
    }
}

pub fn entity_spawner(
    mut commands: Commands,
    mut civilian_spawners: Query<(&mut CivilianSpawner, &GlobalTransform)>,
    mut hunter_spawners: Query<(&mut HunterSpawner, &GlobalTransform)>,
    civilians: Query<&Civilian>,
    hunters: Query<&Hunter>,
    mut layout_handles: ResMut<TextureAtlasLayoutHandles>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    day_cycle: Res<DayCycle>,
) {
    let dt = time.delta_seconds();
    let mut rand = thread_rng();
    for (mut spawner, spawner_gpos) in civilian_spawners.iter_mut() {
        spawner.timer.tick(Duration::from_secs_f32(dt));
        if spawner.timer.finished() {
            let spawner_pos = spawner_gpos.translation().xy();
            if rand.gen_bool(0.15) {
                if civilians.iter().len() < 200 && !day_cycle.is_night{
                    spawn_civilian(&mut commands, &asset_server, spawner_pos, &mut layout_handles);
                }
            }
        }
    }
    for (mut spawner, spawner_gpos) in hunter_spawners.iter_mut() {
        spawner.timer.tick(Duration::from_secs_f32(dt));
        if spawner.timer.finished() {
            let spawner_pos = spawner_gpos.translation().xy();
            if rand.gen_bool(0.15) {
                if hunters.iter().len() < 200 && day_cycle.is_night{
                    spawn_hunter(&mut commands, &asset_server, spawner_pos, &mut layout_handles);
                }
            }
        }
    }
}

fn raycast(
    origin: Vec2,
    dir: Vec2,
    max_toi: f32,
    rapier_context: &Res<RapierContext>,
) -> Option<Entity> {
    let solid = true;
    let filter = QueryFilter::default();
    let filter = filter.groups(CollisionGroups::new(
        Group::all(),
        Group::from_bits(STRUCTURES_CG | PLAYER_CG).unwrap())
    );
    if let Some((entity, _)) = rapier_context.cast_ray(
        origin, dir, max_toi, solid, filter
    ) {
        return Some(entity);
    } else {
        return None;
    }
}