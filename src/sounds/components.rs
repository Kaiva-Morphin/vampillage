use bevy_kira_audio::prelude::*;
use bevy::prelude::*;

#[derive(Resource)]
pub struct DayChannel;

#[derive(Resource)]
pub struct NightChannel;

#[derive(Resource)]
pub struct SfxChannel;

#[derive(Resource, Default)]
pub struct AudioHandles {
    pub day: Handle<bevy_kira_audio::AudioSource>,
    pub night: Handle<bevy_kira_audio::AudioSource>,
    pub dash: Handle<bevy_kira_audio::AudioSource>,
    pub lvlup: Vec<Handle<bevy_kira_audio::AudioSource>>,
    pub hit: Vec<Handle<bevy_kira_audio::AudioSource>>,
    pub kill: Vec<Handle<bevy_kira_audio::AudioSource>>,
    pub throw: Handle<bevy_kira_audio::AudioSource>,
    pub select: Handle<bevy_kira_audio::AudioSource>,
    pub selected: Handle<bevy_kira_audio::AudioSource>,
    pub dash_cd: Handle<bevy_kira_audio::AudioSource>,
}

#[derive(Event)]
pub enum PlaySoundEvent {
    LvlUp,
    Dash,
    DashCD,
    Hit,
    Kill,
    Throw,
    Select,
    Selected,
}