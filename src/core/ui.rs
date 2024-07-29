use std::f32::consts::PI;

use bevy::math::{uvec2, vec2};
use bevy::prelude::*;
use bevy::app::Plugin;
use bevy::ui::widget::UiImageSize;
use bevy::ui::ContentSize;
use bevy::window::WindowResized;

use crate::player::components::Player;
use crate::{get_local_time_f, DAY_DURATION, TRANSLATION_DURATION};

use super::camera::plugin::{MainCamera, TARGET_ASPECT, TARGET_HEIGHT, TARGET_WIDTH};
use super::functions::TextureAtlasLayoutHandles;




#[derive(Resource)]
pub struct PlayerUI{
    hp: f32,
    xp: f32,
    time: f32,
    hunger: f32
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup);
        app.add_systems(Update, on_resize_system);
        app.add_systems(Update, update);
    }
}


#[derive(Component)]
pub struct PlayerUINode;

#[derive(Component)]
pub struct Daynight;

#[derive(Component)]
pub struct Blood;


fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {

    commands.spawn((
        ImageBundle {
            style: Style{
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::End,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            image: UiImage::new(asset_server.load("ui/new_ui.png")),
            ..default()
        },
        PlayerUINode,
    )).with_children(|commands|{
        commands.spawn(
            NodeBundle {
                style: Style{
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::End,
                    align_content: AlignContent::Start,
                    align_items: AlignItems::End,
                    flex_direction: FlexDirection::ColumnReverse,
                    position_type: PositionType::Absolute,
                    ..default()
                },..default()}).with_children(|commands|{
                    commands.spawn((
                        TextureAtlas{
                            layout: asset_server.add(TextureAtlasLayout::from_grid(uvec2(22, 22), 29, 1, None, None)),
                            index: 0
                        },
                        PlayerUINode,
                        Daynight,
                        ImageBundle{
                            image: UiImage::new(asset_server.load("ui/daynight.png")),
                            style: Style{
                                justify_self: JustifySelf::End,
                                ..default()
                            },
                            ..default()
                        }));
                });
                commands.spawn(
                    NodeBundle {
                        style: Style{
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            justify_content: JustifyContent::End,
                            align_content: AlignContent::Start,
                            align_items: AlignItems::Start,
                            flex_direction: FlexDirection::ColumnReverse,
                            position_type: PositionType::Absolute,
                            ..default()
                        },..default()}).with_children(|commands|{
                            commands.spawn((
                                TextureAtlas{
                                    layout: asset_server.add(TextureAtlasLayout::from_grid(uvec2(22, 22), 20, 1, None, None)),
                                    index: 0
                                },
                                PlayerUINode,
                                Blood,
                                ImageBundle{
                                    image: UiImage::new(asset_server.load("ui/blood.png")),
                                    style: Style{
                                        justify_self: JustifySelf::Start,
                                        ..default()
                                    },
                                    ..default()
                                }));
                        });
                commands.spawn((
                    ImageBundle {
                        style: Style{
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            ..default()
                        },
                        image: UiImage::new(asset_server.load("ui/new_ui_fg.png")),
                        ..default()
                    },
                ));
            });

        
}

fn on_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut ui_style: Query<(&mut Style, &UiImageSize), With<PlayerUINode>>,
) {
    for e in resize_reader.read() {
        for (mut style, size) in ui_style.iter_mut() {
            let size = size.size();
            if TARGET_ASPECT > e.width / e.height {
                // width is smaller than now, resize relative to width
                style.width = Val::Px(2. * e.width / TARGET_WIDTH * size.x as f32);
                style.height = Val::Px(2. * e.width / TARGET_WIDTH * size.y as f32);
            } else {
                style.height = Val::Px(2. * e.height / TARGET_HEIGHT * size.y as f32);
                style.width = Val::Px(2. * e.height / TARGET_HEIGHT * size.x as f32);
            }
        }
    }
}


fn update(
    mut blood_e: Query<&mut TextureAtlas, (With<Blood>, Without<Daynight>)>,
    mut daynight_e: Query<(&mut TextureAtlas, &mut UiImage), (Without<Blood>, With<Daynight>)>,
    time: Res<Time<Virtual>>,
    player_stats: Query<&Player>
){
    if let Ok(stats) = player_stats.get_single(){
        blood_e.single_mut().index = ((1. - (stats.hp as f32 / stats.max_hp as f32)) * 20.).round() as usize % 20;
    }
    let (mut atlas, mut image) = daynight_e.single_mut();
    let t = (get_local_time_f(time.elapsed_seconds()) + 0.75) % 1.;
    let d = (t * (29. * 2. - 2.)).ceil() as usize;
    atlas.index = if d < 29 {image.flip_x = false; d} else {image.flip_x = true; (29 * 2) - d - 2};
}





