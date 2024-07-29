use bevy::prelude::*;

use crate::{npc::systems::RosesCollected, sounds::components::PlaySoundEvent, PauseEvent, NUM_ROSES};

use super::components::{DeathText, DeathTime, ParentEntity, Player, UpgradeButton};

pub fn lvl_up(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let font = asset_server.load("fonts/Monocraft.ttf");
    let parent = commands.spawn((
        ImageBundle {
            style: Style {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                justify_items: JustifyItems::Start,
                justify_content: JustifyContent::Start,
                flex_direction: FlexDirection::Column,
                width: Val::Px(200.),
                height: Val::Px(300.),
                ..default()
            },
            image: UiImage::from(asset_server.load("scroll.png")),
            ..default()
        },
        Name::new("LvlUpScreen"),
    )).id();
    let mut children = vec![];
    
    children.push(spawn_button(commands, asset_server, font.clone_weak(), "Max HP + 10%", UpgradeButton::MaxHp, parent));
    children.push(spawn_button(commands, asset_server, font.clone_weak(), "Armor + 10%", UpgradeButton::Armor, parent));
    children.push(spawn_button(commands, asset_server, font.clone_weak(), "HP Gain + 10%", UpgradeButton::HpGain, parent));
    children.push(spawn_button(commands, asset_server, font.clone_weak(), "XP Gain + 10%", UpgradeButton::XpGain, parent));
    children.push(spawn_button(commands, asset_server, font.clone_weak(), "Speed + 10%", UpgradeButton::Speed, parent));
    children.push(spawn_button(commands, asset_server, font.clone_weak(), "Dash CD - 8%", UpgradeButton::DashCD, parent));
    children.push(spawn_button(commands, asset_server, font.clone_weak(), "Dash Time + 8%", UpgradeButton::DashTick, parent));
    children.push(spawn_button(commands, asset_server, font.clone_weak(), "Hunger Rate - 20%", UpgradeButton::HungerRate, parent));

    for child in children {
        commands.entity(parent).add_child(child);
    }
}

fn spawn_button(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    font: Handle<Font>,
    text: &str,
    upgrade_type: UpgradeButton,
    parent: Entity,
) -> Entity {
    commands.spawn((ButtonBundle {
        style: Style {
            top: Val::Percent(25.),
            left: Val::Percent(20.),
            width: Val::Px(190. / 2.5),
            height: Val::Px(30. / 2.5),
            justify_items: JustifyItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::bottom(Val::Px(5.0)),
            ..default()
        },
        image: UiImage::from(asset_server.load("button.png")),
        ..default()
    },
    upgrade_type,
    ParentEntity { entity: parent},
    )).with_children(|parent| {
        parent.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                ..default()
            },
            text: Text {
                sections: vec![TextSection::new(text, TextStyle { font, font_size: 16. / 2., color: Color::srgb_u8(169, 96, 45) })],
                ..default()
            },
            ..default()
        });
    }).id()
}

pub fn interact_upgrade_button(
    mut commands: Commands,
    mut button_q: Query<(&Interaction, &mut UiImage, &UpgradeButton, &ParentEntity), Changed<Interaction>>,
    mut player: Query<&mut Player>,
    mut pause_event: EventWriter<PauseEvent>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    asset_server: Res<AssetServer>,
) {
    if let Ok(mut player) = player.get_single_mut() {
        if let Ok((interaction, mut image,
            upgrade_type, parent_entity)) = button_q.get_single_mut() {
            match *interaction {
                Interaction::Pressed => {
                    match *upgrade_type {
                        UpgradeButton::MaxHp => {
                            player.max_hp *= 1.1;
                        },
                        UpgradeButton::Armor => {
                            player.phys_res *= 1.1;
                        },
                        UpgradeButton::HpGain => {
                            player.hp_gain *= 1.1;
                        },
                        UpgradeButton::XpGain => {
                            player.xp_gain *= 1.1;
                        },
                        UpgradeButton::Speed => {
                            player.max_speed *= 1.1;
                            player.accumulation_gain *= 1.1;
                        },
                        UpgradeButton::HungerRate => {
                            player.hunger_rate *= 0.8;
                        },
                        UpgradeButton::DashCD => {
                            player.dash_cd *= 0.92;
                        },
                        UpgradeButton::DashTick => {
                            player.dash_tick *= 0.92;
                        },
                        
                    }
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
}

pub fn spawn_death_text(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    time: f32,
    roses_collected: &Res<RosesCollected>,
    won: bool,
) {
    let font = asset_server.load("fonts/Monocraft.ttf");
    let text;
    if won {
        text = "You Won"
    } else {
        text = "You Died"
    }
    commands.spawn((TextBundle {
        style: Style {
            top: Val::Percent(20.),
            justify_self: JustifySelf::Center,
            ..default()
        },
        text: Text {
            sections: vec![TextSection::new(text, TextStyle { font: font.clone(), font_size: 64. / 2.5, color: Color::WHITE })],
            ..default()
        },
        ..default()
    }, DeathText));
    commands.spawn((TextBundle {
        style: Style {
            top: Val::Percent(40.),
            justify_self: JustifySelf::Center,
            ..default()
        },
        text: Text {
            sections: vec![TextSection::new((time as i32).to_string(), TextStyle { font: font.clone(), font_size: 50. / 2.5, color: Color::WHITE })],
            ..default()
        },
        ..default()
    }, DeathTime, DeathText));

    if won {
        commands.spawn((TextBundle {
            style: Style {
                top: Val::Percent(60.),
                justify_self: JustifySelf::Center,
                ..default()
            },
            text: Text {
                sections: vec![TextSection::new("But the next Vampire\nis looking for roses!", TextStyle { font: font.clone(), font_size: 40. / 2.5, color: Color::WHITE })],
                ..default()
            },
            ..default()
        }, DeathText));
    } else {
        commands.spawn((TextBundle {
            style: Style {
                top: Val::Percent(60.),
                justify_self: JustifySelf::Center,
                ..default()
            },
            text: Text {
                sections: vec![TextSection::new("Till the next cycle", TextStyle { font: font.clone(), font_size: 40. / 2.5, color: Color::WHITE })],
                ..default()
            },
            ..default()
        }, DeathText));
    }
    
    let collected;
    if won {
        collected = NUM_ROSES
    } else {
        collected = roses_collected.collected;
    }
    commands.spawn((TextBundle {
        style: Style {
            top: Val::Percent(80.),
            justify_self: JustifySelf::Center,
            ..default()
        },
        text: Text {
            sections: vec![TextSection::new(format!("Roses Collected: {} / {}", collected, NUM_ROSES), TextStyle { font: font.clone(), font_size: 40. / 2.5, color: Color::WHITE })],
            ..default()
        },
        ..default()
    }, DeathText));
}


pub fn update_death_text(
    time: f32,
    death_time: &mut Query<&mut Text, With<DeathTime>>,
) {
    if let Ok(mut death_time) = death_time.get_single_mut() {
        death_time.sections[0].value = (time as i32).to_string();
    }
}