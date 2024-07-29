use std::time::Duration;

use bevy::{color::palettes::css::{BLUE, RED}, math::uvec2, prelude::*};
use bevy_light_2d::light::AmbientLight2d;
use pathfinding::num_traits::{Euclid, Signed};

use crate::{characters::animation::AnimationController, core::{camera::plugin::MainCamera, functions::TextureAtlasLayoutHandles, post_processing::PostProcessUniform}, npc::systems::RosesCollected, player::components::{ParentEntity, Player, UpgradeButton}, sounds::components::PlaySoundEvent};

pub const TRANSLATION_DURATION: f32 = 1.0;
pub const DAY_DURATION: f32 = 15.0;

#[derive(Resource)]
pub struct DayCycle {
    pub is_night: bool,
    pub is_translating: bool,
}

// 0 is morning
pub fn get_local_time_f(elapsed: f32) -> f32{
    ((elapsed + TRANSLATION_DURATION * 0.5) % (TRANSLATION_DURATION * 2. + DAY_DURATION * 2.)) / (TRANSLATION_DURATION * 2. + DAY_DURATION * 2.)
}


pub fn update_daycycle(
    mut cycle: ResMut<DayCycle>,
    mut post_process: Query<&mut PostProcessUniform>,
    mut cam: Query<&mut AmbientLight2d, With<MainCamera>>,
    time: Res<Time<Virtual>>,
    mut pc_q: Query<&mut AnimationController, With<Player>>
) {
    let cycle_time = (time.elapsed_seconds() + TRANSLATION_DURATION * 2. + DAY_DURATION * 2.) % (TRANSLATION_DURATION * 2. + DAY_DURATION * 2.);
    let is_night_raw = cycle_time < (TRANSLATION_DURATION + DAY_DURATION);
    let local_time = cycle_time % (TRANSLATION_DURATION + DAY_DURATION);
    cycle.is_night = is_night_raw;
    let mut light = cam.single_mut();
    cycle.is_translating = false;
    if local_time > DAY_DURATION {
        let translation = (local_time - DAY_DURATION) / TRANSLATION_DURATION;
        let v = if is_night_raw {1.-translation} else {translation};
        post_process.single_mut().daytime = v;
        light.brightness = (1. - v) * 0.8 + 0.2;
        cycle.is_translating = true;
        if translation > 0.5 {
            cycle.is_night = !cycle.is_night;
        }
    } else {
        post_process.single_mut().daytime = if cycle.is_night {1.} else {0.};
        light.brightness = if cycle.is_night {0.2} else {1.};
    }
    for mut pc in pc_q.iter_mut(){
        if cycle.is_night {
            pc.disarm();
        } else {
            pc.arm();
        }
    }
}

#[derive(Event, Debug)]
pub struct PauseEvent;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    InGame,
    Pause,
}

pub fn pause_game(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut time: ResMut<Time<Virtual>>,
    mut pause_event: EventReader<PauseEvent>,
) {
    let t = pause_event.read();
    for _pause in t {
        match state.get() {
            GameState::InGame => {
                next_state.set(GameState::Pause);
                time.pause();
            },
            GameState::Pause => {
                next_state.set(GameState::InGame);
                time.unpause();
            },
        }
    }
}

#[derive(Component)]
pub struct Score;

#[derive(Component)]
pub struct ScoreRoses;

pub fn spawn_score(
    mut commands: Commands,
    player: Query<&Player>,
    asset_server: Res<AssetServer>,
    roses: Res<RosesCollected>
) {
    if let Ok(player) = player.get_single() {
        let font = asset_server.load("fonts/Monocraft.ttf");
        commands.spawn((TextBundle {
            style: Style {
                top: Val::Percent(0.),
                left: Val::Percent(0.),
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection::new(format!(
                    "Score: {:?}", player.score as i32),
                    TextStyle { font: font.clone_weak(), font_size: 16., color: Color::WHITE })],
                ..default()
            },
            ..default()
        }, Score));
        commands.spawn((TextBundle {
            style: Style {
                top: Val::Percent(5.),
                left: Val::Percent(0.),
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection::new(format!(
                        "Roses: {:?} / {}", roses.collected, roses.max),
                        TextStyle { font, font_size: 16., color: Color::WHITE })],
                ..default()
            },
            ..default()
        }, ScoreRoses));
    }
}

pub fn update_score(
    player: Query<&Player>,
    mut score: Query<&mut Text, (With<Score>, Without<ScoreRoses>)>,
    mut score_roses: Query<&mut Text, With<ScoreRoses>>,
    roses: Res<RosesCollected>,
) {
    if let Ok(player) = player.get_single() {
        let mut score = score.single_mut();
        let mut score_roses = score_roses.single_mut();
        score.sections[0].value = format!("Score: {:?}", player.score as i32);
        score_roses.sections[0].value = format!("Roses: {:?} / {}", roses.collected, roses.max);
    }
}

#[derive(Component)]
pub struct StartButton;

pub fn spawn_starter_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pause_event: EventWriter<PauseEvent>,
) {
    pause_event.send(PauseEvent);
    let font = asset_server.load("fonts/Monocraft.ttf");
    let parent = commands.spawn((
        ImageBundle {
            style: Style {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                justify_content: JustifyContent::Center,
                justify_items: JustifyItems::Center,
                flex_direction: FlexDirection::Column,
                width: Val::Px(400.),
                height: Val::Px(600.),
                ..default()
            },
            image: UiImage::from(asset_server.load("scroll.png")),
            ..default()
        },
        Name::new("StarterScreen"),
    )).with_children(|parent| {
        parent.spawn(TextBundle {
            style: Style {
                width: Val::Percent(40.),
                height: Val::Percent(40.),
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                ..default()
            },
            text: Text {
                sections: vec![TextSection::new(
                    "   A game by yaroyanuo and kaiva-morphin 
                                For Bevy Game Jam 5.

                    Controls:
                    WASD - movement, LShift - Dash.
                    If you are experiencing  lags, press F4
                    ",
                    TextStyle { font: font.clone_weak(), font_size: 16., color: Color::srgb_u8(169, 96, 45) })],
                ..default()
            },
            ..default()
        });
    }).id();
    let child = commands.spawn((ButtonBundle {
        style: Style {
            width: Val::Px(150.),
            height: Val::Px(30.),
            justify_items: JustifyItems::Center,
            justify_content: JustifyContent::Center,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            margin: UiRect::top(Val::Px(30.)),
            ..default()
        },
        image: UiImage::from(asset_server.load("button.png")),
        ..default()
    },
    StartButton,
    ParentEntity {entity: parent},
    )).with_children(|parent| {
        parent.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                ..default()
            },
            text: Text {
                sections: vec![TextSection::new("Start", TextStyle { font, font_size: 16., color: Color::srgb_u8(169, 96, 45) })],
                ..default()
            },
            ..default()
        });
    }).id();
    commands.entity(parent).add_child(child);
}

pub fn interact_start_button(
    mut commands: Commands,
    mut button_q: Query<(&Interaction, &mut UiImage, &ParentEntity), (With<StartButton>, Changed<Interaction>)>,
    mut pause_event: EventWriter<PauseEvent>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    asset_server: Res<AssetServer>,
) {
    if let Ok((interaction, mut image, parent_entity)) = button_q.get_single_mut() {
        match *interaction {
            Interaction::Pressed => {
                commands.entity(parent_entity.entity).despawn_recursive();
                play_sound.send(PlaySoundEvent::Selected);
                pause_event.send(PauseEvent);
            }
            Interaction::Hovered => {
                play_sound.send(PlaySoundEvent::Select);
                *image = UiImage::from(asset_server.load("select_button.png"));
            }
            Interaction::None => {
                *image = UiImage::from(asset_server.load("button.png"));
            }
        }
    }
}