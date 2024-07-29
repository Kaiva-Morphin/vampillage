use std::time::Duration;

use bevy::prelude::*;
use bevy::app::Plugin;

pub struct DespawnLifetimePlugin;

impl Plugin for DespawnLifetimePlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (update_timer, despawn_after_timer).chain());
    }
}


#[derive(Component)]
pub struct DespawnTimer(Timer);

impl DespawnTimer{
    pub fn seconds(seconds: f32) -> Self{
        DespawnTimer(Timer::new(Duration::from_secs_f32(seconds), TimerMode::Once))
    }
}


fn update_timer(time: Res<Time>, mut query: Query<&mut DespawnTimer>) {
    for mut timer in query.iter_mut() {
        timer.0.tick(time.delta());
    }
}

fn despawn_after_timer(mut commands: Commands, query: Query<(Entity, &DespawnTimer)>) {
    for (entity, timer) in query.iter() {
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}