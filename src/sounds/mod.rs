pub mod systems;
pub mod components;

use bevy_kira_audio::prelude::*;
use bevy::prelude::*;

use components::*;
use systems::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_audio_channel::<DayChannel>()
        .add_audio_channel::<NightChannel>()
        .add_audio_channel::<SfxChannel>()
        .insert_resource(AudioHandles::default())
        .add_event::<PlaySoundEvent>()
        .add_systems(Startup, load_audio)
        .add_systems(Update, (manage_background, play_sounds))
        ;
    }
}