use std::{f32::consts::PI, time::Duration};

use bevy::{ecs::system::EntityCommands, math::{uvec2, vec3}, prelude::*};
use bevy_easings::*;
use bevy_rapier2d::prelude::Velocity;
use rand::Rng;

use crate::core::{despawn_lifetime::DespawnTimer, functions::{ExpDecay, TextureAtlasLayoutHandles}};

pub enum SimpleAnimatedTexture{
    HeartGain,
    HeartLoss,
    Soul,
    Fork,
    Knife,
    Garlic,
    Fire
}

#[derive(Component)]
pub struct SimpleAnimated{
    effect: SimpleAnimatedTexture,
    timer: Timer
}

pub fn simple_anim_update(
    mut to_anim: Query<(&mut SimpleAnimated, &mut TextureAtlas)>,
    time: Res<Time>
){
    let dt = time.delta_seconds();
    for (mut anim_type, mut atlas) in to_anim.iter_mut(){
        anim_type.timer.tick(Duration::from_secs_f32(dt.into()));

        let frames = match anim_type.effect {
            SimpleAnimatedTexture::HeartGain => 2,
            SimpleAnimatedTexture::HeartLoss => 2,
            SimpleAnimatedTexture::Soul => 5,
            SimpleAnimatedTexture::Fork => 8,
            SimpleAnimatedTexture::Knife => 8,
            SimpleAnimatedTexture::Garlic => 4,
            SimpleAnimatedTexture::Fire => 19,
        };
        let offset = match anim_type.effect {
            SimpleAnimatedTexture::HeartGain => 2,
            SimpleAnimatedTexture::HeartLoss => 0,
            SimpleAnimatedTexture::Soul => 0,
            SimpleAnimatedTexture::Fork => 10,
            SimpleAnimatedTexture::Knife => 0,
            SimpleAnimatedTexture::Garlic => 30,
            SimpleAnimatedTexture::Fire => 0,
        };

        if anim_type.timer.finished(){
            atlas.index = (atlas.index + 1 - offset) % frames + offset
        }
    }
}

pub fn fire_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>, idx: usize) -> impl Bundle {
    (
        SimpleAnimated{effect: SimpleAnimatedTexture::Fire, timer: Timer::from_seconds(0.06, TimerMode::Repeating)},
        SpriteBundle{
            texture: asset_server.load("particles/fire.png"),
            sprite: Sprite{
                color: Color::srgb(1.5, 1.5, 1.5),
                ..default()
            },
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Fire", TextureAtlasLayout::from_grid(uvec2(16, 23), 8, 3, None, None)),
            index: idx
        },
    )
}
pub fn emotion_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>, idx: usize) -> impl Bundle {
    (
        SpriteBundle{
            texture: asset_server.load("particles/emotions.png"),
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Emotion", TextureAtlasLayout::from_grid(uvec2(10, 14), 3, 3, Some(uvec2(1, 1)), None)),
            index: idx
        },
    )
}

pub fn animated_heart_gain_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>, ) -> impl Bundle {
    (
        SimpleAnimated{effect: SimpleAnimatedTexture::HeartGain, timer: Timer::from_seconds(0.06, TimerMode::Repeating)},
        SpriteBundle{
            texture: asset_server.load("particles/heart.png"),
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Hearts", TextureAtlasLayout::from_grid(uvec2(10, 9), 2, 2, Some(uvec2(1, 1)), None)),
            index: 2
        },
    )
}

pub fn animated_heart_loss_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>,) -> impl Bundle {
    (
        SimpleAnimated{effect: SimpleAnimatedTexture::HeartLoss, timer: Timer::from_seconds(0.06, TimerMode::Repeating)},
        SpriteBundle{
            texture: asset_server.load("particles/heart.png"),
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Hearts", TextureAtlasLayout::from_grid(uvec2(10, 9), 2, 2, Some(uvec2(1, 1)), None)),
            index: 0
        },
    )
}

pub fn animated_fork_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>,) -> impl Bundle {
    (
        SimpleAnimated{effect: SimpleAnimatedTexture::Fork, timer: Timer::from_seconds(0.06, TimerMode::Repeating)},
        SpriteBundle{
            texture: asset_server.load("hunter/throwables.png"),
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Throwables", TextureAtlasLayout::from_grid(uvec2(13, 13), 10, 4, Some(uvec2(1, 1)), None)),
            index: 10
        },
    )
}

pub fn animated_garlic_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>,) -> impl Bundle {
    (
        SimpleAnimated{effect: SimpleAnimatedTexture::Garlic, timer: Timer::from_seconds(0.06, TimerMode::Repeating)},
        SpriteBundle{
            texture: asset_server.load("hunter/throwables.png"),
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Throwables", TextureAtlasLayout::from_grid(uvec2(13, 13), 10, 4, Some(uvec2(1, 1)), None)),
            index: 30
        },
    )
}

pub fn animated_knife_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>,) -> impl Bundle {
    (
        SimpleAnimated{effect: SimpleAnimatedTexture::Knife, timer: Timer::from_seconds(0.06, TimerMode::Repeating)},
        SpriteBundle{
            texture: asset_server.load("hunter/throwables.png"),
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Throwables", TextureAtlasLayout::from_grid(uvec2(13, 13), 10, 4, Some(uvec2(1, 1)), None)),
            index: 0
        },
    )
}

pub fn animated_soul_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>,) -> impl Bundle {
    (
        SimpleAnimated{effect: SimpleAnimatedTexture::Soul, timer: Timer::from_seconds(0.1, TimerMode::Repeating)},
        DespawnTimer::seconds(0.5),
        SpriteBundle{
            texture: asset_server.load("particles/soul.png"),
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Soul", TextureAtlasLayout::from_grid(uvec2(9, 12), 5, 1, Some(uvec2(1, 1)), None)),
            index: 0
        },
    )
}

#[derive(Component)]
pub struct Stake;

pub fn stake_bundle(asset_server: &Res<AssetServer>, atlas_handles: &mut ResMut<TextureAtlasLayoutHandles>, direction: Vec2) -> impl Bundle {
    let angle = direction.to_angle() + PI * 0.75;
    (
        SpriteBundle{
            texture: asset_server.load("hunter/throwables.png"),
            transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, angle, 0., 0.)),
            ..default()
        },
        TextureAtlas{
            layout: atlas_handles.add_or_load(asset_server, "Throwables", TextureAtlasLayout::from_grid(uvec2(13, 13), 10, 4, Some(uvec2(1, 1)), None)),
            index: 20
        },
    )
}




pub fn spawn_cililian_body(
    commands: &mut Commands,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
    asset_server: &Res<AssetServer>,
    pos: Vec3,
) -> Entity {

    let max_offset = 4.;
    let start = pos + vec3(
        rand::random::<f32>() * max_offset * 2. - max_offset,
        rand::random::<f32>() * max_offset * 2. - max_offset,
        0.
    );
    let flipped = rand::thread_rng().gen_bool(0.5);
    let offset = if flipped{vec3(-2., 0., 0.)} else {vec3(2., 0., 0.)};
    commands.spawn(animated_soul_bundle(asset_server, layout_handles))
    .insert(Transform::from_translation(offset+vec3(0., 8., 10.) + start).ease_to(
        Transform::from_translation(offset+start+vec3(0., 12. + rand::thread_rng().gen::<f32>() * 5., 1.)),
        EaseFunction::ExponentialOut,
        EasingType::Once {
            duration: std::time::Duration::from_secs(1),
        },
    ));

    commands.spawn((
        TransformBundle::default(),
        VisibilityBundle::default(),
        DespawnTimer::seconds(5.),
    ))
    .insert(Transform::from_translation(vec3(0., -1., -2.) + start).with_scale(vec3(if flipped{-1.} else {1.}, 1., 0.)))
    .with_children(|commands| {
        commands.spawn((
            Name::new("Particle"),
            SpriteBundle{
                texture: asset_server.load("particles/body_civilian.png"),
                ..default()
            },
        )).insert(
            Sprite{..default()}.ease_to(
                Sprite { color: Color::Srgba(Srgba::new(1., 1., 1., 0.)), ..default() },
                EaseFunction::ExponentialIn,
                EasingType::Once {
                    duration: std::time::Duration::from_secs(5),
                },
            )
        );
    }).id()
}

pub fn spawn_hunter_body(
    commands: &mut Commands,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
    asset_server: &Res<AssetServer>,
    pos: Vec3,
) -> Entity {
    let max_offset = 4.;
    let start = pos + vec3(
        rand::random::<f32>() * max_offset * 2. - max_offset,
        rand::random::<f32>() * max_offset * 2. - max_offset,
        0.
    );
    let flipped = rand::thread_rng().gen_bool(0.5);
    let offset = if flipped{vec3(-2., 0., 0.)} else {vec3(2., 0., 0.)};
    commands.spawn(animated_soul_bundle(asset_server, layout_handles))
    .insert(Transform::from_translation(offset+vec3(0., 8., 10.) + start).ease_to(
        Transform::from_translation(offset+start+vec3(0., 12. + rand::thread_rng().gen::<f32>() * 5., 0.)),
        EaseFunction::ExponentialOut,
        EasingType::Once {
            duration: std::time::Duration::from_secs(1),
        },
    ));

    commands.spawn((
        TransformBundle::default(),
        VisibilityBundle::default(),
        DespawnTimer::seconds(5.),
    ))
    .insert(Transform::from_translation(vec3(0., -3., -2.) + start).with_scale(vec3(if flipped{-1.} else {1.}, 1., 1.)))
    .with_children(|commands| {
        commands.spawn((
            Name::new("Particle"),
            SpriteBundle{
                texture: asset_server.load("particles/body_hunter.png"),
                ..default()
            },
        )).insert(
            Sprite{..default()}.ease_to(
                Sprite { color: Color::Srgba(Srgba::new(1., 1., 1., 0.)), ..default() },
                EaseFunction::ExponentialIn,
                EasingType::Once {
                    duration: std::time::Duration::from_secs(5),
                },
            )
        );
    }).id()
}

pub fn spawn_question_particle(
    commands: &mut Commands,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
    asset_server: &Res<AssetServer>,
    pos: Vec3,
){
    let lifetime = 2.;
    let max_offset = 2.;
    let start = pos+vec3(
        rand::random::<f32>() * max_offset * 2. - max_offset,
        rand::random::<f32>() * max_offset * 2. - max_offset,
        0.
    );
    commands.spawn((
        TransformBundle::default(),
        VisibilityBundle::default(),
        DespawnTimer::seconds(lifetime),
    ))
    .insert(Transform::from_translation(vec3(0., 8., 8.) + start))
    .with_children(|commands| {
        commands.spawn((
            Name::new("Particle"),
            emotion_bundle(asset_server, layout_handles, rand::thread_rng().gen_range(0..3) + 6),
            Transform::from_translation(vec3(0., 0., 0.)).with_rotation(Quat::from_rotation_z(rand::thread_rng().gen::<f32>() - 0.5)).with_scale(Vec3::splat(0.5))
                .ease_to(
                    Transform::from_translation(vec3(rand::thread_rng().gen::<f32>() * 3. - 1.5, 4. + rand::thread_rng().gen::<f32>() * 5., 0.)).with_rotation(Quat::from_rotation_z(rand::thread_rng().gen::<f32>() - 0.5))
                    .with_scale(Vec3::splat(1.5)),
                    EaseFunction::ExponentialOut,
                    EasingType::Once {
                        duration: std::time::Duration::from_secs_f32(lifetime),
                    },
                )
        )).insert(
            Sprite{..default()}.ease_to(
                Sprite { color: Color::Srgba(Srgba::new(1., 1., 1., 0.)),..default() },
                EaseFunction::ExponentialIn,
                EasingType::Once {
                    duration: std::time::Duration::from_secs_f32(lifetime),
                },
            )
        );
    });
}

pub fn spawn_angry_particle(
    commands: &mut Commands,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
    asset_server: &Res<AssetServer>,
    pos: Vec3,
){
    let max_offset = 2.;
    let start = pos+vec3(
        rand::random::<f32>() * max_offset * 2. - max_offset,
        rand::random::<f32>() * max_offset * 2. - max_offset,
        0.
    );

    let flipped = rand::thread_rng().gen::<bool>();
    commands.spawn((
        TransformBundle::default(),
        VisibilityBundle::default(),
        DespawnTimer::seconds(1.),
    ))
    .insert(Transform::from_translation(vec3(0., 8., 8.) + start))
    .with_children(|commands| {
        commands.spawn((
            Name::new("Particle"),
            emotion_bundle(asset_server, layout_handles, rand::thread_rng().gen_range(0..3)),
            Transform::from_translation(vec3(0., 0., 0.)).with_rotation(Quat::from_rotation_z(rand::thread_rng().gen::<f32>() - 0.5)).with_scale(Vec3::splat(0.5) * vec3(if flipped{-1.} else {1.}, 1., 1.))
                .ease_to(
                    Transform::from_translation(vec3(rand::thread_rng().gen::<f32>() * 3. - 1.5, 4. + rand::thread_rng().gen::<f32>() * 5., 0.)).with_rotation(Quat::from_rotation_z(rand::thread_rng().gen::<f32>() - 0.5)).with_scale(vec3(if flipped{-1.} else {1.}, 1., 1.)),
                    EaseFunction::ExponentialOut,
                    EasingType::Once {
                        duration: std::time::Duration::from_secs(1),
                    },
                )
        )).insert(
            Sprite{flip_x: flipped, ..default()}.ease_to(
                Sprite { color: Color::Srgba(Srgba::new(1., 1., 1., 0.)), flip_x: flipped,..default() },
                EaseFunction::ExponentialIn,
                EasingType::Once {
                    duration: std::time::Duration::from_secs(1),
                },
            )
        );
    });
}

pub fn spawn_warn_particle(
    commands: &mut Commands,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
    asset_server: &Res<AssetServer>,
    pos: Vec3,
){
    let lifetime = 2.;
    let max_offset = 2.;
    let start = pos+vec3(
        rand::random::<f32>() * max_offset * 2. - max_offset,
        rand::random::<f32>() * max_offset * 2. - max_offset,
        0.
    );
    commands.spawn((
        TransformBundle::default(),
        VisibilityBundle::default(),
        DespawnTimer::seconds(lifetime),
    ))
    .insert(Transform::from_translation(vec3(0., 8., 8.) + start))
    .with_children(|commands| {
        commands.spawn((
            Name::new("Particle"),
            emotion_bundle(asset_server, layout_handles, rand::thread_rng().gen_range(0..3) + 3),
            Transform::from_translation(vec3(0., 0., 0.)).with_rotation(Quat::from_rotation_z(rand::thread_rng().gen::<f32>() - 0.5)).with_scale(Vec3::splat(0.5))
                .ease_to(
                    Transform::from_translation(vec3(rand::thread_rng().gen::<f32>() * 3. - 1.5, 4. + rand::thread_rng().gen::<f32>() * 5., 0.)).with_rotation(Quat::from_rotation_z(rand::thread_rng().gen::<f32>() - 0.5))
                    .with_scale(Vec3::splat(1.5)),
                    EaseFunction::ExponentialOut,
                    EasingType::Once {
                        duration: std::time::Duration::from_secs_f32(lifetime),
                    },
                )
        )).insert(
            Sprite{..default()}.ease_to(
                Sprite { color: Color::Srgba(Srgba::new(1., 1., 1., 0.)), ..default() },
                EaseFunction::ExponentialOut,
                EasingType::Once {
                    duration: std::time::Duration::from_secs_f32(lifetime),
                },
            )
        );
    });
}

#[derive(Component)]
pub struct FollowingBloodParticle{
    pub follow: Entity,
    pub tail: Option<Entity>
}

#[derive(Component)]
pub struct FollowingBloodParticlePart{
    pub max_dist: f32,
    pub tail: Option<Entity>
}

fn spawn_blood_particle(
    commands: &mut Commands,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
    asset_server: &Res<AssetServer>,
    size: f32,
) -> Entity {
    if size < 0. {panic!("Neg size  blood particle!")}
    let mut index = size as usize;
    if index >= MAX_BLOOD_PARTICLE_SIZE {index = MAX_BLOOD_PARTICLE_SIZE; warn!("Cant spawn blood particle with size {}, clamping", size);}
    commands.spawn((
        TransformBundle::default(),
        VisibilityBundle::default(),
    )).with_children(|commands|{
        commands.spawn((
            SpriteBundle{
                transform: Transform::from_xyz(0., 0., 10.),
                texture: asset_server.load("particles/blood.png"),
                ..default()
            },
            TextureAtlas{
                layout: layout_handles.add_or_load(&asset_server, "BloodParticle", TextureAtlasLayout::from_grid(uvec2(7, 7), 5, 2, Some(uvec2(1, 1)), None)),
                index,
                ..default()
            }
        ));
        commands.spawn((
            SpriteBundle{
                transform: Transform::from_xyz(0., 0., 0.),
                texture: asset_server.load("particles/blood.png"),
                ..default()
            },
            TextureAtlas{
                layout: layout_handles.add_or_load(&asset_server, "BloodParticle", TextureAtlasLayout::from_grid(uvec2(7, 7), 5, 2, Some(uvec2(1, 1)), None)),
                index: index + MAX_BLOOD_PARTICLE_SIZE,
                ..default()
            }
        ));
    }).id()
}

const MAX_BLOOD_PARTICLE_SIZE : usize = 5;


pub fn spawn_follow_blood_particle(
    mut commands: &mut Commands,
    layout_handles: &mut ResMut<TextureAtlasLayoutHandles>,
    asset_server: &Res<AssetServer>,
    follow: Entity,
    pos: Vec3,
    length: usize
){
    if length == 0 {return}
    let sizes: Vec<f32> = (0..MAX_BLOOD_PARTICLE_SIZE).map(|v|{v as f32}).collect();
    let mut to_add: Vec<f32> = vec![];
    let m = sizes.len() as f32 / length as f32;
    for i in 0..length{
        let size_idx = (i as f32 * m).floor() as usize;
        to_add.push(sizes[size_idx]);
    }
    let e = spawn_blood_particle(commands, layout_handles, asset_server, to_add[0]);
    let root = commands.entity(e).insert((
        Transform::from_translation(pos),
        FollowingBloodParticle{follow, tail: None},
        Name::new("BLOOD ROOT"),
    )).id();
    let mut last: Entity = root;
    let mut root_is_dad = false;
    for i in 1..length{
        let Some(size) = to_add.get(i) else {break;};
        let size = *size;
        let e = spawn_blood_particle(commands, layout_handles, asset_server, size);
        if root_is_dad{
            commands.entity(last).insert(FollowingBloodParticlePart{max_dist: size * 0.45, tail: Some(e)});
        } else {
            root_is_dad = true;
            commands.entity(last).insert(FollowingBloodParticle{follow, tail: Some(e)});
        }
        last = e;
        if i == length - 1 {
            commands.entity(last).insert(FollowingBloodParticlePart{max_dist: size * 0.45, tail: None});
        }
    }
}



pub fn update_blood_particles(
    transforms: Query<&Transform, (Without<FollowingBloodParticle>, Without<FollowingBloodParticlePart>)>,
    mut particle_heads: Query<(&FollowingBloodParticle, &mut Transform), Without<FollowingBloodParticlePart>>,
    mut particle_tails: Query<(&FollowingBloodParticlePart, &mut Transform), Without<FollowingBloodParticle>>,
    time: Res<Time<Virtual>>,
){
    let dt = time.delta_seconds();
    for (head, mut current_pos) in particle_heads.iter_mut(){
        let Ok(target_pos) = transforms.get(head.follow) else {continue};
        let last_pos= current_pos.translation.xy().exp_decay(target_pos.translation.xy(), 10., dt).extend(10.);
        current_pos.translation = last_pos;
        let Some(tail) = head.tail else {continue;};

        fn rec_update(tail: Entity, mut last_pos: Vec2, particle_tails: &mut Query<'_, '_, (&FollowingBloodParticlePart, &mut Transform), Without<FollowingBloodParticle>>){
            let Ok((p, mut transform)) = particle_tails.get_mut(tail) else {return;};
            let d = (last_pos.xy() - transform.translation.xy()).length_squared();
            if d > p.max_dist * p.max_dist {
                transform.translation = (last_pos + ((transform.translation.xy() - last_pos.xy()).normalize_or_zero() * p.max_dist)).extend(10.);
            }
            last_pos = transform.translation.truncate();
            let Some(tail) = p.tail else {return;};
            rec_update(tail, last_pos, particle_tails);
        }

        rec_update(tail, last_pos.truncate(), &mut particle_tails);
    }
}