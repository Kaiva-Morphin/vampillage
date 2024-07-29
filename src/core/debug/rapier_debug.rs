pub mod plugin {
    use bevy::{app::{Plugin, Update}, input::ButtonInput, prelude::{KeyCode, Res, ResMut}};
    use bevy_rapier2d::render::DebugRenderContext;

    pub struct SwitchableRapierDebugPlugin;
    
    impl Plugin for SwitchableRapierDebugPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_systems(Update, update);
        }
    }

    fn update(
        q: Option<ResMut<DebugRenderContext>>,
        k: Res<ButtonInput<KeyCode>>,
    ){
        let Some(mut c) = q else {return};
        if k.just_pressed(KeyCode::F2){
            c.enabled = !c.enabled;
        }
    }
}