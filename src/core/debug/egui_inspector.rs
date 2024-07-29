pub mod plugin{

use bevy::{app::{App, Plugin, Update}, input::ButtonInput, prelude::{KeyCode, Res, ResMut, Resource}};

#[derive(Resource, Default)]
struct InspectorEnabled(bool);
pub struct SwitchableEguiInspectorPlugin;

impl Plugin for SwitchableEguiInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InspectorEnabled::default());
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new().run_if(inspector_toggled));
        app.add_systems(Update, toggle_inspector);
    }
}

fn inspector_toggled(
    cond: Res<InspectorEnabled>
) -> bool {
    return cond.0
}

fn toggle_inspector(
    mut cond: ResMut<InspectorEnabled>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F1){
        cond.0 = !cond.0;
    }
}

}