use bevy::{math::{uvec2, vec3}, prelude::*, render::view::visibility};
use rand::Rng;

use crate::core::functions::TextureAtlasLayoutHandles;



#[derive(Component)]
pub struct AnimationController{
    animation_speed: f32,
    pub ticker: AnimTicker,
    pub current_animation: CharacterAnimation,
    dir_offset: usize,
    /*
    0 - idle, talk
    1 - move
    2 - attack, skill
    3 - hurt
    */
    pub priority: usize,
    direction: usize,
    armed: bool
}

enum EyeStateOverride{
    Scared,
    Angry
}

impl Default for AnimationController{
    fn default() -> Self {
        AnimationController{
            animation_speed: 1.,
            ticker: AnimTicker::default(),
            current_animation: CharacterAnimation::simple(FrameTime::Constant(0.5), vec![1]).looped(),
            dir_offset: 7,
            priority: 0,
            direction: 0,
            armed: false
        }
    }
}



pub fn spawn_player_animation_bundle(commands: &mut Commands, asset_server: &Res<AssetServer>, layout_handles: &mut ResMut<TextureAtlasLayoutHandles>) -> Entity{
    commands.spawn((
        AnimationController{
            ..default()
        },
        Name::new("Player"),
        TransformBundle::default(),
        VisibilityBundle::default()
        )).with_children(|commands|{
            commands.spawn((
                Name::new("Body"),
                SpriteBundle{
                    texture: asset_server.load("player/vampire.png"),
                    ..default()
                },
                TextureAtlas{
                    layout: layout_handles.add_or_load(&asset_server, "Vampire", TextureAtlasLayout::from_grid(uvec2(14, 20), 7, 3, Some(uvec2(1, 1)), None)),
                    index: 2
                },
                PartType::Body{variant: 0, variants: 1},
            ));
            commands.spawn((
                Name::new("Items"),
            ));
            commands.spawn((
                Name::new("Shadow"),
                SpriteBundle{
                    texture: asset_server.load("particles/shadow.png"),
                    ..default()
                },
            )).insert(Transform::from_translation(vec3(0., -3., SHADOW_Z)));
            commands.spawn((
                Name::new("Umbrella"),
                PartType::Umbrella,
                SpriteBundle{
                    texture: asset_server.load("player/umbrella.png"),
                    ..default()
                },
                TextureAtlas{
                    layout: layout_handles.add_or_load(&asset_server, "Umbrella", TextureAtlasLayout::from_grid(uvec2(19, 13), 2, 3, Some(uvec2(1, 1)), None)),
                    index: 0
                },
            )).insert(Transform::from_translation(vec3(0., 3., ITEM_Z)));
    }).id()
}

pub fn spawn_hunter_animation_bundle(mut commands: &mut Commands, asset_server: &Res<AssetServer>, layout_handles: &mut ResMut<TextureAtlasLayoutHandles>) -> Entity{
    commands.spawn((
        AnimationController{
            ..default()
        },
        VisibilityBundle::default(),
        TransformBundle::default()
    )).with_children(|commands|{
        commands.spawn((
            Name::new("Body"),
            PartType::Body{variant: 0, variants: 1},
            SpriteBundle{
                texture: asset_server.load("hunter/hunter.png"),
                ..default()
            },
            TextureAtlas{
                layout: layout_handles.add_or_load(&asset_server, "Hunter", TextureAtlasLayout::from_grid(uvec2(16, 20), 7, 3, Some(uvec2(1, 1)), None)),
                index: 2
            },
        ));

        commands.spawn((
            Name::new("Shadow"),
            SpriteBundle{
                texture: asset_server.load("particles/shadow.png"),
                ..default()
            },
        )).insert(Transform::from_translation(vec3(0., -3., SHADOW_Z)));
    }).id()
}

const BODY_Z : f32 = 0.;
const OUTFIT_Z : f32 = 0.1;
const ARMS_Z : f32 = 0.2;
const EYES_Z : f32 = 0.3;
const HAIR_Z : f32 = 0.4;
const ITEM_Z : f32 = 0.5;

// todo: BLACK SHADOW COMPONENT WITH AUTO UPDATE POSITION AND ALPHA MODULATION OF LAYER FOR OVERLAP
const SHADOW_Z : f32 = -0.1;

const BODY_COUNT : usize = 8;
const OUTFIT_COUNT : usize = 6;
const HAIR_COUNT : usize = 36;
const EYE_COUNT : usize = 6;
const EYE_PUB_COUNT : usize = 3;
const WEAPON_COUNT : usize = 2;




#[derive(Component)]
pub enum PartType{    
    Body{variant: usize, variants: usize},
    Eyes{variant: usize, variants: usize},
    Item{ variant: usize, variants: usize },
    Outfit{variant: usize, variants: usize},
    Arms,
    Umbrella,
    Hair{variant: usize, variants: usize}
}

impl PartType {
    pub fn default_offset(&self) -> Vec3{
        vec3(
            0.,
            6.,
            match self {
                PartType::Body { variant: _, variants: _ } => BODY_Z,
                PartType::Eyes { variant: _, variants: _ } => EYES_Z,
                PartType::Item { variant: _, variants: _ } => ITEM_Z,
                PartType::Umbrella => 0.02,
                PartType::Outfit { variant: _, variants: _ } => OUTFIT_Z,
                PartType::Arms => ARMS_Z,
                PartType::Hair { variant: _, variants: _ } => HAIR_Z,
            }
        )
    }
}





pub fn spawn_civilian_animation_bundle(commands: &mut Commands, asset_server: &Res<AssetServer>, layout_handles: &mut ResMut<TextureAtlasLayoutHandles>) -> Entity {
    let body_variant = rand::thread_rng().gen_range(0..BODY_COUNT);
    let outfit_variant = rand::thread_rng().gen_range(0..OUTFIT_COUNT);
    let hair_variant = rand::thread_rng().gen_range(0..HAIR_COUNT);
    let weapon_variant = rand::thread_rng().gen_range(0..WEAPON_COUNT);

    let main_color = rand::thread_rng().gen::<f32>() * 0.25 + 0.5;
    let second_color = rand::thread_rng().gen::<f32>() * 0.5;
    let third_color = rand::thread_rng().gen::<f32>() * 0.2;

    let main_idx = rand::thread_rng().gen_range(0..=2);
    let second_idx = rand::thread_rng().gen_range(0..=2);
    let third_idx = rand::thread_rng().gen_range(0..=2);
    let mut eye_color = [0.; 3];
    eye_color[main_idx] = main_color;
    eye_color[second_idx] = (eye_color[second_idx] + second_color).clamp(0., 1.); 
    eye_color[third_idx] = (eye_color[third_idx] + third_color).clamp(0., 1.); 
    commands.spawn((
        AnimationController{
            dir_offset: 4 * BODY_COUNT,
            ..default()
        },
        Name::new("Civilian"),
        TransformBundle::default(),
        VisibilityBundle::default()
        )).with_children(|commands|{
        commands.spawn((
            Name::new("Body"),
            PartType::Body{variant: body_variant, variants: BODY_COUNT},
            SpriteBundle{
                    texture: asset_server.load("civilian/body.png"),
                    ..default()
            },
            TextureAtlas{
                layout: layout_handles.add_or_load(&asset_server, "Body", TextureAtlasLayout::from_grid(uvec2(14, 18), BODY_COUNT as u32 * 4, 3, Some(uvec2(1, 1)), None)),
                index: body_variant * 3 + 1
            },
        )).insert(Transform::from_translation(vec3(0., 0., BODY_Z)));
        commands.spawn((
            Name::new("Weapon"),
            PartType::Item{variant: weapon_variant, variants: WEAPON_COUNT},
            SpriteBundle{
                texture: asset_server.load("civilian/weapon.png"),
                ..default()
            },
            TextureAtlas{
                layout: layout_handles.add_or_load(&asset_server, "Weapon", TextureAtlasLayout::from_grid(uvec2(26, 22), WEAPON_COUNT as u32 * 5, 3, Some(uvec2(1, 1)), None)),
                index: 2 * weapon_variant
            },
        )).insert(Transform::from_translation(vec3(-1.5, 1., ITEM_Z)));
        commands.spawn((
            Name::new("Outfit"),
            PartType::Outfit{variant: outfit_variant, variants: OUTFIT_COUNT},
            SpriteBundle{
                texture: asset_server.load("civilian/outfit.png"),
                ..default()
            },
            TextureAtlas{
                layout: layout_handles.add_or_load(&asset_server, "Outfit", TextureAtlasLayout::from_grid(uvec2(14, 18), OUTFIT_COUNT as u32 * 4, 3, Some(uvec2(1, 1)), None)),
                index: outfit_variant * 4 + 1
            },
        )).insert(Transform::from_translation(vec3(0., 0., OUTFIT_Z)));
        commands.spawn((
            Name::new("Arms"),
            PartType::Arms,
            SpriteBundle{
                    texture: asset_server.load("civilian/body.png"),
                    ..default()
            },
            TextureAtlas{
                layout: layout_handles.add_or_load(&asset_server, "Arms", TextureAtlasLayout::from_grid(uvec2(14, 18), BODY_COUNT as u32 * 4, 3, Some(uvec2(1, 1)), Some(uvec2(0, 57)))),
                index: outfit_variant * 4 + 1
            },
        )).insert(Transform::from_translation(vec3(0., 0., ARMS_Z)));
        commands.spawn((
            Name::new("Eyes"),
            PartType::Eyes{variant: 0, variants: EYE_COUNT},
            SpriteBundle{
                    texture: asset_server.load("civilian/eyes.png"),
                    ..default()
                },
                TextureAtlas{
                    layout: layout_handles.add_or_load(&asset_server, "Eyes", TextureAtlasLayout::from_grid(uvec2(8, 4), EYE_COUNT as u32, 3, Some(uvec2(1, 1)), None)),
                    index: 0
                },
        )).insert(Transform::from_translation(vec3(0., 0., EYES_Z)));

        commands.spawn((
            Name::new("EyesColor"),
            PartType::Eyes{variant: 1, variants: EYE_COUNT},
            SpriteBundle{
                    sprite: Sprite{
                        color: Color::srgb_from_array(eye_color),
                        ..default()
                    },
                    texture: asset_server.load("civilian/eyes.png"),
                    ..default()
                },
                TextureAtlas{
                    layout: layout_handles.add_or_load(&asset_server, "Eyes", TextureAtlasLayout::from_grid(uvec2(8, 4), EYE_COUNT as u32, 3, Some(uvec2(1, 1)), None)),
                    index: 1
                },
        )).insert(Transform::from_translation(vec3(0., 0., EYES_Z)));

        commands.spawn((
            Name::new("Hair"),
            PartType::Hair{variant: hair_variant, variants: HAIR_COUNT},
            SpriteBundle{
                texture: asset_server.load("civilian/hair.png"),
                ..default()
            },
            TextureAtlas{
                layout: layout_handles.add_or_load(&asset_server, "Hair", TextureAtlasLayout::from_grid(uvec2(14, 18), HAIR_COUNT as u32, 4, Some(uvec2(1, 1)), None)),
                index: hair_variant
            },
        )).insert(Transform::from_translation(vec3(0., 0., HAIR_Z)));

        commands.spawn((
            Name::new("Shadow"),
            SpriteBundle{
                texture: asset_server.load("particles/shadow.png"),
                ..default()
            },
        )).insert(Transform::from_translation(vec3(0., -2., SHADOW_Z)));
    }).id()
}








#[derive(Default)]
pub struct AnimTicker{
    pub frame: f32
}

impl AnimTicker{
    pub fn to_start(&mut self){
        self.frame = 0.;
    }
    pub fn tick(&mut self, dt: f32){
        self.frame += dt;
    }

    fn get<T: Clone>(&self, frametime: &FrameTime, vec: &Vec<T>) -> Option<T> {
        if vec.len() == 0 {return None}
        match frametime {
            FrameTime::Constant(t) => {
                let current_anim_frame = (self.frame / t).floor() as usize;
                if current_anim_frame >= vec.len() {return vec.last().cloned()}
                return vec.get(current_anim_frame).cloned()
            },
            FrameTime::Sequence(v) => {
                let mut ac = 0.;
                let mut lasti = 0;
                for (i, t) in v.iter().enumerate() {
                    if ac >= self.frame {
                        if i >= vec.len() {return vec.last().cloned()}
                        return vec.get(i.checked_sub(1).unwrap_or(0)).cloned()
                    }
                    ac += t;
                    lasti = i;
                }
                return vec.get(lasti).cloned()
            },
        }
    }
}


pub trait PlayerAnims{
    fn play_dash(&mut self){}
}

pub trait HunterAnims{
    fn play_hunter_throw(&mut self){}
}

pub trait CivilianAnims{
    fn play_civil_attack(&mut self){}
}

impl CivilianAnims for AnimationController{
    fn play_civil_attack(&mut self) {
        if self.priority > 2 {return}
        self.arm();
        self.current_animation = CharacterAnimation::simple(FrameTime::Constant(0.1), vec![1, 1, 1, 1, 1])
            .with_item_idx(vec![0, 1, 2, 3, 4]);
        self.priority = 2;
        self.ticker.to_start();
    }
}

impl HunterAnims for AnimationController{
    fn play_hunter_throw(&mut self){
        if self.priority > 2 {return}
        self.current_animation = CharacterAnimation::simple(FrameTime::Sequence(vec![0.2, 0.15, 0.2]), vec![4, 5, 6]);
        self.priority = 2;
        self.ticker.to_start();
    }
}



const IDX_HURT : usize = 3;


impl AnimationController{
    pub fn play_idle(&mut self){
        if self.priority > 0 {return}
        self.play_idle_forced();
    }

    pub fn play_idle_forced(&mut self){
        self.current_animation = CharacterAnimation::simple(FrameTime::Constant(0.5), vec![1]).looped();
        self.priority = 0;
        self.ticker.to_start();
    }

    pub fn play_idle_priority(&mut self, priority: usize){
        if self.priority > priority {return}
        self.current_animation = CharacterAnimation::simple(FrameTime::Constant(0.5), vec![1]).looped();
        self.priority = 0;
        self.ticker.to_start();
    }

    pub fn play_hurt(&mut self){
        self.current_animation = CharacterAnimation::simple(FrameTime::Constant(0.2), vec![3]);
        self.priority = 3;
        self.ticker.to_start();
    }

    pub fn play_walk(&mut self){
        if self.priority >= 1 {return} // prevent walk reloop
        self.current_animation = CharacterAnimation::simple(FrameTime::Constant(0.25), vec![0, 1, 2, 1]).looped()
            .with_offsets(vec![vec3(0., -1., 0.), vec3(0., 0., 0.), vec3(0., -1., 0.), vec3(0., 0., 0.)]);
        self.priority = 1;
        self.ticker.to_start();
    }

    pub fn play_walk_unlooped(&mut self){
        if self.priority >= 1 {return} // prevent walk reloop
        self.current_animation = CharacterAnimation::simple(FrameTime::Constant(0.25), vec![0, 1, 2, 1])
            .with_offsets(vec![vec3(0., -1., 0.), vec3(0., 0., 0.), vec3(0., -1., 0.), vec3(0., 0., 0.)]);
        self.priority = 1;
    }

    pub fn turn_left(&mut self){
        self.direction = 1;
    }

    pub fn turn_right(&mut self){
        self.direction = 3;
    }

    pub fn turn_up(&mut self){
        self.direction = 2;
    }

    pub fn turn_down(&mut self){
        self.direction = 0;
    }

    pub fn arm(&mut self){
        self.armed = true
    }

    pub fn disarm(&mut self){
        self.armed = false
    }

    pub fn get_idx(&self) -> usize{
        let Some(idx) = self.ticker.get(&self.current_animation.frame_time, &self.current_animation.frame_idx) else {panic!("Failed to get idx! Probably anim is empty")};
        let mut d = self.direction;
        if self.direction == 3 {d = 1} 
        idx + d * self.dir_offset
    }

    pub fn is_mirrored(&self) -> bool {
        self.direction == 3
    } 

    pub fn get_dir_custom_offset(&self, offset: usize) -> usize {
        let mut d = self.direction;
        if self.direction == 3 {d = 1}
        d * offset
    }

    pub fn get_hair_idx(&self, offset: usize) -> usize {
        let Some(idx) = self.ticker.get(&self.current_animation.frame_time, &self.current_animation.frame_idx) else {panic!("Failed to get idx! Probably anim is empty")};
        if idx == IDX_HURT {
            return offset * 3
        }
        let mut d = self.direction;
        if self.direction == 3 {d = 1}
        d * offset
    }

    pub fn get_eyes_idx(&self, offset: usize, variant: usize) -> usize {
        let Some(idx) = self.ticker.get(&self.current_animation.frame_time, &self.current_animation.frame_idx) else {panic!("Failed to get idx! Probably anim is empty")};
        if idx == IDX_HURT {
            return 4 + variant
        }
        let mut d = self.direction;
        if self.direction == 3 {d = 1}
        d * offset + variant
    }

    pub fn get_eyes_offset(&self) -> Vec3 {
        let Some(idx) = self.ticker.get(&self.current_animation.frame_time, &self.current_animation.frame_idx) else {panic!("Failed to get idx! Probably anim is empty")};
        let m = if self.is_mirrored() {vec3(-1., 1., 1.)} else {vec3(1., 1., 1.)};
        if idx == IDX_HURT {
            return vec3(-2., 1., 0.2) * m
        }
        Vec3::ZERO
    }

    pub fn get_parts_offset(&self) -> Vec3 {
        if self.direction == 1 || self.direction == 3 {
            if self.is_mirrored() {vec3(0., 0., 0.)} else {vec3(-0., 0., 0.)}
        } else {
            self.ticker.get(&self.current_animation.frame_time, &self.current_animation.parts_offsets).unwrap_or(Vec3::ZERO)
        }
    }

    pub fn get_idx_custom_offset(&self, offset: usize) -> usize{
        let Some(idx) = self.ticker.get(&self.current_animation.frame_time, &self.current_animation.frame_idx) else {panic!("Failed to get idx! Probably anim is empty")};
        let mut d = self.direction;
        if self.direction == 3 {d = 1} 
        idx + d * offset
    }

    pub fn get_item_visibility(&self) -> Visibility{
        if self.armed {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    }
    pub fn get_arms_visibility(&self) -> Visibility{
        if self.armed {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        }
    }

    pub fn get_item_idx(&self, variants: usize, variant: usize) -> usize{
        let Some(idx) = self.ticker.get(&self.current_animation.frame_time, &self.current_animation.item_idx) else {return self.get_dir_custom_offset(variants * 5) + variant * 5};
        let mut d = self.direction;
        if self.direction == 3 {d = 1} 
        idx + d * variants * 5 + variant * 5
    }

    pub fn get_item_offset(&self) -> Vec3{
        if self.direction == 1 || self.direction == 3 {
            self.ticker.get(&self.current_animation.frame_time, &self.current_animation.item_offsets).unwrap_or(Vec3::ZERO)
            + if self.is_mirrored() {vec3(0., 0., 0.)} else {vec3(0., 0., 0.)}
        } else {
            self.ticker.get(&self.current_animation.frame_time, &self.current_animation.item_offsets).unwrap_or(Vec3::ZERO)
        }
    }
    
    pub fn tick(&mut self, dt: f32){
        self.ticker.tick(dt * self.animation_speed);
        if self.ticker.frame > self.current_animation.duration{
            if self.current_animation.looped {
                self.ticker.frame %= self.current_animation.duration;
            } else {
                self.play_idle_forced()
            }
        };
    }
}

pub enum FrameTime{
    Constant(f32),
    Sequence(Vec<f32>),
}
// reserved idx:
// 0 1 2 - walk
// 4 - damage
pub struct CharacterAnimation {
    frame_time: FrameTime,
    pub frame_idx: Vec<usize>,
    // umbrella, candle, etc
    item_offsets: Vec<Vec3>,
    item_idx: Vec<usize>,

    parts_offsets: Vec<Vec3>,
    looped: bool,
    pub duration: f32
}

impl CharacterAnimation {
    pub fn looped(mut self) -> Self{
        self.looped = true;
        self
    }
    pub fn with_offsets(mut self, offsets: Vec<Vec3>) -> Self{
        self.parts_offsets = offsets;
        self
    }
    pub fn with_item_idx(mut self, idxs: Vec<usize>) -> Self{
        self.item_idx = idxs;
        self
    }
    pub fn player(frame_time: FrameTime, frame_idx: Vec<usize>, item_offsets: Vec<Vec3>) -> Self {
        if frame_idx.len() == 0 {panic!("Empty animation!")}
        let duration = match &frame_time{
            FrameTime::Constant(t) => {t * frame_idx.len() as f32}
            FrameTime::Sequence(s) => {s.into_iter().sum()}
        };
        CharacterAnimation{
            duration,
            frame_idx,
            frame_time,
            item_offsets,
            parts_offsets: vec![],
            item_idx: vec![],
            looped: false
        }
    }
    pub fn simple(frame_time: FrameTime, frame_idx: Vec<usize>) -> Self {
        if frame_idx.len() == 0 {panic!("Empty animation!")}
        let duration = match &frame_time{
            FrameTime::Constant(t) => {t * frame_idx.len() as f32}
            FrameTime::Sequence(s) => {s.into_iter().sum()}
        };
        CharacterAnimation{
            duration,
            frame_idx,
            frame_time,
            item_offsets: vec![],
            item_idx: vec![],
            parts_offsets: vec![],
            looped: false,

        }
    }
    pub fn civilian(frame_time: FrameTime, frame_idx: Vec<usize>, parts_offsets: Vec<Vec3>) -> Self {
        if frame_idx.len() == 0 {panic!("Empty animation!")}
        let duration = match &frame_time{
            FrameTime::Constant(t) => {t * frame_idx.len() as f32}
            FrameTime::Sequence(s) => {s.into_iter().sum()}
        };
        CharacterAnimation{
            duration,
            frame_idx,
            frame_time,
            item_idx: vec![],
            item_offsets: vec![],
            parts_offsets,
            looped: false,

        }
    }

}

pub(super) fn update_sprites(
    mut commands: Commands,
    mut player_controllers: Query<(Entity, &mut AnimationController, &Children)>,
    mut sprites: Query<(&mut Sprite, &mut TextureAtlas, &PartType, &mut Transform, &mut Visibility)>,
    time: Res<Time>,
){
    let dt = time.delta_seconds();
    for (e, mut c, children) in player_controllers.iter_mut(){
        c.tick(dt);
        let mirrored = c.is_mirrored();
        let offset = c.get_parts_offset();
        for child in children.iter(){
            if let Ok((mut sprite, mut atlas, sprite_type, mut transform, mut visibility)) = sprites.get_mut(*child){
                match sprite_type {
                    PartType::Body{variant: _, variants: _} => {
                        transform.translation = sprite_type.default_offset();
                        atlas.index = c.get_idx();
                        sprite.flip_x = mirrored;
                    },
                    PartType::Umbrella => {
                        transform.translation = sprite_type.default_offset() + offset + match c.direction {
                            0 => {vec3(3.5, 4.5, 0.)}
                            1 => {vec3(1.5, 3.5, 0.)}
                            2 => {vec3(-0.5, 4.5, 0.)}
                            _ => {vec3(-1.5, 3.5, 0.)}
                        };

                        atlas.index = c.get_dir_custom_offset(2);
                        if c.get_idx() % c.dir_offset == IDX_HURT {
                            atlas.index = 1;
                            transform.translation = sprite_type.default_offset() + offset + if c.is_mirrored(){vec3(-3.5, 4.5, 0.)} else {vec3(3.5, 4.5, 0.)}
                        }
                        *visibility = c.get_item_visibility();
                        //transform.translation = sprite_type.default_offset() + offset + c.get_item_offset();
                        sprite.flip_x = mirrored;
                    },
                    PartType::Eyes{variant, variants} => {
                        atlas.index = c.get_eyes_idx(*variants, *variant);
                        sprite.flip_x = mirrored;
                        transform.translation = sprite_type.default_offset() + offset + c.get_eyes_offset();
                    },
                    PartType::Item{variant, variants} => {
                        atlas.index = c.get_item_idx(*variants, *variant);
                        sprite.flip_x = mirrored;
                        *visibility = c.get_item_visibility();
                        transform.translation = vec3(0., 2., ITEM_Z) + offset * vec3(0., 1., 0.) + c.get_item_offset() + sprite_type.default_offset();
                    },
                    PartType::Outfit{variant, variants} => {
                        transform.translation = sprite_type.default_offset();
                        atlas.index = c.get_idx_custom_offset(variants * 4) + variant * 4;
                        sprite.flip_x = mirrored;
                    },
                    PartType::Arms => {
                        atlas.index = c.get_idx();
                        sprite.flip_x = mirrored;
                        transform.translation = sprite_type.default_offset();
                        *visibility = c.get_arms_visibility();
                    },
                    PartType::Hair{variant, variants} => {
                        atlas.index = c.get_hair_idx(*variants) + variant;
                        sprite.flip_x = mirrored;
                        transform.translation = sprite_type.default_offset() + offset;
                    },
                }
            }
        }
    }
}

