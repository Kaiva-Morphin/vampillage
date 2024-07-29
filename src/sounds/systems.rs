use std::time::Duration;

use bevy_kira_audio::prelude::*;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{DayCycle, TRANSLATION_DURATION};

use super::components::*;

pub fn load_audio(
    asset_server: Res<AssetServer>,
    mut audio_handles: ResMut<AudioHandles>,
    night_channel: Res<AudioChannel<NightChannel>>,
) {
    audio_handles.day = asset_server.load("sounds/day.flac");
    audio_handles.night = asset_server.load("sounds/night.wav");
    audio_handles.dash = asset_server.load("sounds/dash.wav");
    audio_handles.throw = asset_server.load("sounds/throw.wav");
    audio_handles.lvlup.extend(vec![
        asset_server.load("sounds/lvlup.wav"),
        asset_server.load("sounds/lvlup1.wav"),
        asset_server.load("sounds/lvlup2.wav"),
        asset_server.load("sounds/lvlup3.wav"),
    ]);
    audio_handles.hit.extend(vec![
        asset_server.load("sounds/hit.wav"),
        // asset_server.load("sounds/hit1.wav"),
        asset_server.load("sounds/hit2.wav"),
    ]);
    audio_handles.kill.extend(vec![
        asset_server.load("sounds/kill.wav"),
        asset_server.load("sounds/kill1.wav"),
    ]);
    audio_handles.select = asset_server.load("sounds/select.wav");
    audio_handles.selected = asset_server.load("sounds/selected.wav");
    audio_handles.dash_cd = asset_server.load("sounds/dash_cd.wav");
    
    night_channel.play(audio_handles.night.clone_weak())
    .start_from(0.)
    .fade_in(AudioTween::new(Duration::from_secs_f32(1.), AudioEasing::OutPowf(2.)))
    .with_volume(1.)
    ;
}

pub fn manage_background(
    day_channel: Res<AudioChannel<DayChannel>>,
    night_channel: Res<AudioChannel<NightChannel>>,
    day_cycle: Res<DayCycle>,
    audio_handles: Res<AudioHandles>,
    mut prev_state: Local<bool>
) {
    if day_cycle.is_translating && *prev_state != day_cycle.is_translating {
        let dur = Duration::from_secs_f32(TRANSLATION_DURATION);
        let easing = AudioEasing::OutPowf(2.);
        
        if day_cycle.is_night { // translating into day
            night_channel.stop()
            .fade_out(AudioTween::new(dur, easing))
            ;
            
            day_channel.play(audio_handles.day.clone_weak())
            .start_from(0.)
            .fade_in(AudioTween::new(dur, easing))
            .with_volume(0.4)
            ;
        } else { // translating into night
            day_channel.stop()
            .fade_out(AudioTween::new(dur, easing))
            ;
            
            night_channel.play(audio_handles.night.clone_weak())
            .start_from(0.)
            .fade_in(AudioTween::new(dur, easing))
            .with_volume(1.)
            ;
        }
    }
    *prev_state = day_cycle.is_translating;
}

pub fn play_sounds(
    audio_handles: Res<AudioHandles>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    mut sound_events: EventReader<PlaySoundEvent>,
) {
    let mut rng = thread_rng();
    for sound in sound_events.read() {
        match *sound {
            PlaySoundEvent::Dash => {
                sfx_channel.play(audio_handles.dash.clone_weak());
            }
            PlaySoundEvent::LvlUp => {
                let set = &audio_handles.lvlup;
                let idx = rng.gen_range(0..set.len());
                sfx_channel.play(set[idx].clone_weak());
            },
            PlaySoundEvent::Hit => {
                let set = &audio_handles.hit;
                let idx = rng.gen_range(0..set.len());
                sfx_channel.play(set[idx].clone_weak());
            },
            PlaySoundEvent::Kill => {
                let set = &audio_handles.kill;
                let idx = rng.gen_range(0..set.len());
                sfx_channel.play(set[idx].clone_weak());
            },
            PlaySoundEvent::Throw => {
                sfx_channel.play(audio_handles.throw.clone_weak());
            }
            PlaySoundEvent::Select => {
                sfx_channel.play(audio_handles.select.clone_weak());
            },
            PlaySoundEvent::Selected => {
                sfx_channel.play(audio_handles.selected.clone_weak());
            },
            PlaySoundEvent::DashCD => {
                sfx_channel.play(audio_handles.dash_cd.clone_weak());
            }
        }
    }
}