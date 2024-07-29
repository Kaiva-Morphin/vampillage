use bevy::prelude::*;

use super::animation::update_sprites;


pub struct CharacterAnimationPlugin;

impl Plugin for CharacterAnimationPlugin{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostUpdate, update_sprites.run_if(in_state(crate::systems::GameState::InGame)));
    }
}


