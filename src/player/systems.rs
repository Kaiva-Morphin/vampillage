use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::characters::animation::{spawn_player_animation_bundle, AnimationController, PartType};
use crate::core::camera::plugin::CameraFollow;
use crate::core::functions::{ExpDecay, TextureAtlasLayoutHandles};
use crate::core::ui::PlayerUINode;
use crate::npc::systems::RosesCollected;
use crate::sounds::components::PlaySoundEvent;
use crate::systems::DayCycle;
use crate::PauseEvent;
use bevy::math::{uvec2, vec2};
use pathfinding::num_traits::Signed;

use super::components::*;
use super::upgrade_ui::{lvl_up, spawn_death_text, update_death_text};

pub const PLAYER_CG: u32 = 0b0000_0000_0000_0001;
pub const NPC_CG: u32 = 0b0000_0000_0000_0010;
pub const STRUCTURES_CG: u32 = 0b0000_0000_0000_0100;
pub const BULLET_CG: u32 = 0b0000_0000_0000_1000;
pub const RAYCASTABLE_STRUCT_CG: u32 = 0b0000_0000_0001_0000;

#[derive(Component)]
pub struct PlayerController{
    pub accumulated_velocity: Vec2,
}
impl Default for PlayerController {
    fn default() -> Self {
        PlayerController{accumulated_velocity: Vec2::ZERO}
    }
}

#[derive(Default, Debug)]
pub enum Direction {
    Up,
    Right,
    #[default]
    Down,
    Left
}

#[derive(Component, Default)]
pub struct PlayerAnimationState{
    pub dir: Direction
}

pub fn spawn_player_first_time(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layout_handles: ResMut<TextureAtlasLayoutHandles>,
) {
    spawn_player(&mut commands, &asset_server, &mut layout_handles);
}

pub fn spawn_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
){
    let entity = spawn_player_animation_bundle(commands, asset_server, layout_handles);
    commands.entity(entity).insert((
        VisibilityBundle::default(),
        TransformBundle::from_transform(Transform::from_xyz(16., 16., -1.)),
        Name::new("Player"),
        CameraFollow{order: 0, speed: 10.},
        Player::default(),
        AnimationController::default(),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED_Z,
        Collider::ball(4.),
        ActiveEvents::COLLISION_EVENTS,
        Velocity::zero(),
        PlayerController::default(),
        DashTimer {timer: Timer::new(Duration::from_secs_f32(0.35), TimerMode::Repeating)},
        Sleeping::disabled(),
        CollisionGroups::new(
            Group::from_bits(PLAYER_CG).unwrap(),
            Group::from_bits(BULLET_CG | STRUCTURES_CG | NPC_CG | RAYCASTABLE_STRUCT_CG).unwrap()
        ),
    ));
}

pub fn player_controller(
    mut commands: Commands,
    mut player_q: Query<(&mut Velocity, &mut PlayerController,
        &mut AnimationController, &mut DashTimer, &mut Player, Entity)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    day_cycle: Res<DayCycle>,
    time: Res<Time>,
    mut dash_dir: Local<Vec2>,
    mut dash_cd: Local<f32>,
    mut play_sound: EventWriter<PlaySoundEvent>,
) {
    if let Ok((mut character_controller, mut controller,
        mut animation_controller, mut dash_timer,
        mut player, player_entity)) = player_q.get_single_mut() {
    character_controller.linvel = Vec2::ZERO;
    if player.is_dead{return}
    let dt = time.delta_seconds();


    *dash_cd += dt;
    if dash_timer.timer.elapsed_secs() == 0. {
        let input_dir = vec2(
            keyboard.pressed(KeyCode::KeyD) as i32 as f32 - keyboard.pressed(KeyCode::KeyA) as i32 as f32,
            keyboard.pressed(KeyCode::KeyW) as i32 as f32 - keyboard.pressed(KeyCode::KeyS) as i32 as f32
        );
        
        controller.accumulated_velocity = controller.accumulated_velocity.move_towards(input_dir.normalize_or_zero() * player.max_speed, dt * player.accumulation_gain);
        if controller.accumulated_velocity.length() > player.max_speed {controller.accumulated_velocity = controller.accumulated_velocity.normalize() * player.max_speed}
        character_controller.linvel = controller.accumulated_velocity;
    
        if input_dir.x.abs() < 0.1 { // x axis is priotirized 
            if input_dir.y.abs() > 0.1 {
                if input_dir.y.is_positive(){animation_controller.turn_up()}
                if input_dir.y.is_negative(){animation_controller.turn_down()}
            }
        } else {
            if input_dir.x.is_positive(){animation_controller.turn_right()}
            if input_dir.x.is_negative(){animation_controller.turn_left()}
        }
        if controller.accumulated_velocity.length() > 0.1 {
            animation_controller.play_walk();
        } else {
            animation_controller.play_idle_priority(1);
        }
        player.hp -= dt * player.hunger_rate;
        
        if keyboard.just_pressed(KeyCode::ShiftLeft) {
            if *dash_cd < player.dash_cd {
                play_sound.send(PlaySoundEvent::DashCD);
                return;
            }
            play_sound.send(PlaySoundEvent::Dash);
            dash_timer.timer.set_duration(Duration::from_secs_f32(0.35));
            dash_timer.timer.tick(Duration::from_secs_f32(dt * player.dash_tick));
            *dash_cd = 0.;
            *dash_cd += dt;
            
            *dash_dir = input_dir;
            if day_cycle.is_night {
                commands.entity(player_entity).insert(
                    (CollisionGroups::new(
                        Group::from_bits(PLAYER_CG).unwrap(),
                        Group::from_bits(STRUCTURES_CG | NPC_CG | RAYCASTABLE_STRUCT_CG).unwrap()
                    ),
                    Sensor,)
                );
            } else {
                commands.entity(player_entity).insert(
                    CollisionGroups::new(
                        Group::from_bits(PLAYER_CG).unwrap(),
                        Group::from_bits(STRUCTURES_CG | RAYCASTABLE_STRUCT_CG).unwrap()
                    ),
                );
            }
        }
    } else {
        dash_timer.timer.tick(Duration::from_secs_f32(dt * player.dash_tick));
        let t = dash_timer.timer.elapsed_secs();

        let new_max = player.max_speed * g(t);
        let new_gain = player.accumulation_gain * g(t);

        controller.accumulated_velocity = controller.accumulated_velocity.move_towards(dash_dir.normalize_or_zero() * new_max, dt * new_gain);
        if controller.accumulated_velocity.length() > new_max {controller.accumulated_velocity = controller.accumulated_velocity.normalize() * new_max}
        character_controller.linvel = controller.accumulated_velocity;
        
        if dash_timer.timer.finished() {
            dash_timer.timer.set_elapsed(Duration::from_secs_f32(0.));
            commands.entity(player_entity).insert(
            CollisionGroups::new(
                Group::from_bits(PLAYER_CG).unwrap(),
                Group::from_bits(BULLET_CG | STRUCTURES_CG | NPC_CG | RAYCASTABLE_STRUCT_CG).unwrap()
            )).remove::<Sensor>();
        }
    }
    }
}

fn g(x: f32) -> f32 {
    let x = 3. - 5. * x;
    5. * std::f32::consts::E.powf(-(x - 1.639964).powf(2.)/(2.*0.800886f32.powf(2.)))
}

pub fn hit_player(
    mut hit_player: EventReader<HitPlayer>,
    mut player: Query<(&mut Player, &mut AnimationController)>,
    mut kill_player: EventWriter<KillPlayer>,
) {
    if let Ok((mut player, mut animation_controller)) = player.get_single_mut() {
        for hit in hit_player.read() {
            animation_controller.play_hurt();
            if hit.dmg_type == 0 { // proj
                player.hp -= player.max_hp * 0.1 * (1. - player.phys_res)
            } else if hit.dmg_type == 1 { // civ
                player.hp -= player.max_hp * 0.05 * (1. - player.phys_res)
            } else if hit.dmg_type == 2 { // hun
                player.hp -= 15. * (1. - player.phys_res)
            }
        }
        if player.hp < 0. && !player.is_dead {
            kill_player.send(KillPlayer {won: false});
        }
    }
}

pub fn kill_player(
    mut player_entity: Query<(Entity, &mut Player)>,
    mut kill_player: EventReader<KillPlayer>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut death_timer: ResMut<DeathTimer>,
    time: Res<Time>,
    mut death_time: Query<&mut Text, With<DeathTime>>,
    death_text: Query<Entity, With<DeathText>>,
    roses: Res<RosesCollected>,
) {
    let dt = time.delta_seconds();
    let t = death_timer.timer.duration().as_secs_f32() - death_timer.timer.elapsed_secs();
    let (entity, mut player) = player_entity.single_mut();
    for event in kill_player.read() {
        if !event.won {
            play_sound.send(PlaySoundEvent::Kill);
        }
        commands.entity(entity).insert(Visibility::Hidden);
        player.is_dead = true;
        death_timer.timer.tick(Duration::from_secs_f32(dt));
        spawn_death_text(&mut commands, &asset_server, t, &roses, event.won);
    }
    if death_timer.timer.elapsed_secs() != 0. {
        death_timer.timer.tick(Duration::from_secs_f32(dt));
        update_death_text(t, &mut death_time);
        if death_timer.timer.finished() {
            commands.entity(entity).insert((
                Visibility::Visible,
                Transform::from_xyz(16., 16., 0.),
                Player::default()
            ));
            death_timer.timer.set_elapsed(Duration::ZERO);
            for entity in death_text.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn kill_npc(
    mut kill_npc: EventReader<KillNpc>,
    mut player: Query<&mut Player>,
) {
    if let Ok(mut player) = player.get_single_mut() {
        for kill in kill_npc.read() {
            player.hp = (player.hp + player.hp_gain).clamp(0.0, player.max_hp);
            if kill.npc_type == 0 { // civ
                player.score += 100.;
                player.xp += player.xp_gain;
            } else if kill.npc_type == 1 { // hun
                player.score += 500.;
                player.xp += player.xp_gain * 3.;
            }
        }
    }
}

pub fn manage_xp(
    mut player: Query<&mut Player>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    mut commands: Commands,
    mut pause_event: EventWriter<PauseEvent>,
    asset_server: Res<AssetServer>,
    mut t: Local<bool>,
) {
    if let Ok(mut player) = player.get_single_mut() {
        if player.xp > player.max_xp {
            player.xp -= player.max_hp;
            player.max_xp *= 1.2;
            play_sound.send(PlaySoundEvent::LvlUp);
            lvl_up(&mut commands, &asset_server);
            pause_event.send(PauseEvent);
            *t = true;
        }
    }
}